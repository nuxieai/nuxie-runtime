use std::cell::{Cell, RefCell};
use std::rc::Rc;

use luaur_rt::{Error, Function, Lua, MultiValue, Result, Table, Value};

const MEMORY_EXHAUSTED: &str = "not enough memory";
const MEMORY_LIMIT_ERROR: &str = "script VM exceeded its 16 MiB memory ceiling";
const SAFEPOINT_LIMIT_ERROR: &str = "script cycle exceeds 100000 script safepoints";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptResourceLimit {
    Memory,
    Safepoints,
    /// A terminal resource identity supplied by an embedding-host module.
    Extension(&'static str),
}

impl ScriptResourceLimit {
    /// Stable machine-readable identity for diagnostics above the VM layer.
    pub const fn code(self) -> &'static str {
        match self {
            Self::Memory => "script.resource.memory",
            Self::Safepoints => "script.resource.safepoints",
            Self::Extension(code) => code,
        }
    }
}

/// Cloneable access to the VM's terminal resource-exhaustion side channel.
///
/// Product hosts can make their own limits survive Luau protected calls
/// without teaching the baseline VM their vocabulary or resource policy.
#[derive(Clone)]
pub struct ScriptResourceGuard {
    tracker: ResourceLimitTracker,
}

impl ScriptResourceGuard {
    pub(super) fn new(tracker: ResourceLimitTracker) -> Self {
        Self { tracker }
    }

    pub fn reject_if_tripped(&self) -> Result<()> {
        self.tracker.reject_if_tripped()
    }

    pub fn fail(&self, code: &'static str, message: impl Into<String>) -> Error {
        self.tracker
            .fail_with_message(ScriptResourceLimit::Extension(code), message)
    }

    /// First authored callback failure observed in the active script cycle.
    /// Module-registration retries are intentionally excluded: only live
    /// [`LuaScriptInstance`](crate::vm::LuaScriptInstance) callbacks record
    /// this side channel.
    pub fn callback_failure(&self) -> Option<String> {
        self.tracker.callback_failure()
    }
}

/// Per-VM, per-cycle terminal resource state. Nuxie-owned limit sites trip this
/// tracker before raising their ordinary Luau error. Protected-call wrappers
/// then consult the typed side channel instead of trying to recover the kind
/// from Luau's stringified callback error.
#[derive(Clone, Default)]
pub(super) struct ResourceLimitTracker {
    tripped: Rc<Cell<Option<ScriptResourceLimit>>>,
    message: Rc<RefCell<Option<String>>>,
    callback_failure: Rc<RefCell<Option<String>>>,
}

impl ResourceLimitTracker {
    pub(super) fn begin_cycle(&self) {
        self.tripped.set(None);
        self.message.borrow_mut().take();
        self.callback_failure.borrow_mut().take();
    }

    pub(super) fn terminal_limit(&self) -> Option<ScriptResourceLimit> {
        self.tripped.get()
    }

    pub(super) fn fail(&self, limit: ScriptResourceLimit) -> Error {
        let terminal = self.record(limit);
        self.error(terminal)
    }

    pub(super) fn fail_with_message(
        &self,
        limit: ScriptResourceLimit,
        message: impl Into<String>,
    ) -> Error {
        let terminal = match self.tripped.get() {
            Some(terminal) => terminal,
            None => {
                self.tripped.set(Some(limit));
                *self.message.borrow_mut() = Some(message.into());
                limit
            }
        };
        self.error(terminal)
    }

    fn error(&self, limit: ScriptResourceLimit) -> Error {
        match self.message.borrow().as_ref() {
            Some(message) => Error::runtime(message.clone()),
            None => resource_limit_error(limit),
        }
    }

    pub(super) fn observe_vm_error(&self, error: &Error) {
        match error {
            Error::MemoryError(_) => {
                self.record(ScriptResourceLimit::Memory);
            }
            Error::CallbackError { cause, .. } => self.observe_vm_error(cause),
            _ => {}
        }
    }

    pub(super) fn observe_callback_failure(&self, error: &Error) {
        let mut failure = self.callback_failure.borrow_mut();
        if failure.is_none() {
            *failure = Some(error.to_string());
        }
    }

    fn callback_failure(&self) -> Option<String> {
        self.callback_failure.borrow().clone()
    }

    fn record(&self, limit: ScriptResourceLimit) -> ScriptResourceLimit {
        match self.tripped.get() {
            Some(terminal) => terminal,
            None => {
                self.message.borrow_mut().take();
                self.tripped.set(Some(limit));
                limit
            }
        }
    }

    pub(super) fn reject_if_tripped(&self) -> Result<()> {
        match self.tripped.get() {
            Some(limit) => Err(self.error(limit)),
            None => Ok(()),
        }
    }

    fn record_protected_memory_error(&self, error: &Value) -> Result<()> {
        let Value::String(message) = error else {
            return Ok(());
        };
        // Luau's allocator longjmp is flattened to this fixed string inside
        // the built-in protected-call functions. All Nuxie-owned limits use
        // the typed tracker directly; this is the sole unavoidable bridge for
        // the VM allocator's existing protected-call representation.
        if message.to_str()? == MEMORY_EXHAUSTED && self.tripped.get().is_none() {
            self.record(ScriptResourceLimit::Memory);
        }
        Ok(())
    }

    fn record_protected_memory_result(&self, results: &MultiValue) -> Result<()> {
        if matches!(results.front(), Some(Value::Boolean(false)))
            && let Some(error) = results.get(1)
        {
            self.record_protected_memory_error(error)?;
        }
        Ok(())
    }
}

/// Keep VM and per-cycle resource exhaustion terminal across Luau's protected
/// call surfaces. Ordinary validation/runtime errors remain ordinary false
/// results from `pcall`, `xpcall`, and `coroutine.resume`.
pub(super) fn install_protected_call_guards(
    lua: &Lua,
    tracker: ResourceLimitTracker,
) -> Result<()> {
    let original: Function = lua.globals().get("pcall")?;
    let pcall_tracker = tracker.clone();
    let guarded = lua.create_function(move |_, args: MultiValue| {
        pcall_tracker.reject_if_tripped()?;
        let results: MultiValue = original.call(args)?;
        pcall_tracker.record_protected_memory_result(&results)?;
        pcall_tracker.reject_if_tripped()?;
        Ok(results)
    })?;
    lua.globals().set("pcall", guarded)?;

    let original: Function = lua.globals().get("xpcall")?;
    let xpcall_tracker = tracker.clone();
    let guarded = lua.create_function(move |lua, mut args: MultiValue| {
        xpcall_tracker.reject_if_tripped()?;
        if let Some(Value::Function(handler)) = args.get(1).cloned() {
            let handler_tracker = xpcall_tracker.clone();
            let guarded_handler = lua.create_function(move |_, error: Value| {
                handler_tracker.record_protected_memory_error(&error)?;
                handler_tracker.reject_if_tripped()?;
                handler.call::<MultiValue>(error)
            })?;
            let handler_slot = args
                .get_mut(1)
                .ok_or_else(|| Error::runtime("xpcall handler argument disappeared"))?;
            *handler_slot = Value::Function(guarded_handler);
        }
        let results: MultiValue = original.call(args)?;
        xpcall_tracker.record_protected_memory_result(&results)?;
        xpcall_tracker.reject_if_tripped()?;
        Ok(results)
    })?;
    lua.globals().set("xpcall", guarded)?;

    let coroutine: Table = lua.globals().get("coroutine")?;
    let original: Function = coroutine.get("resume")?;
    let guarded = lua.create_function(move |_, args: MultiValue| {
        tracker.reject_if_tripped()?;
        let results: MultiValue = original.call(args)?;
        tracker.record_protected_memory_result(&results)?;
        tracker.reject_if_tripped()?;
        Ok(results)
    })?;
    coroutine.set("resume", guarded)
}

fn resource_limit_error(limit: ScriptResourceLimit) -> Error {
    match limit {
        ScriptResourceLimit::Memory => Error::MemoryError(MEMORY_LIMIT_ERROR.to_owned()),
        ScriptResourceLimit::Safepoints => Error::runtime(SAFEPOINT_LIMIT_ERROR),
        ScriptResourceLimit::Extension(code) => Error::runtime(code),
    }
}
