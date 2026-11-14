use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::luau::{Error, Function, Lua, MultiValue, Result, Table, Value};

const MEMORY_EXHAUSTED: &str = "not enough memory";
const MEMORY_LIMIT_ERROR: &str = "script VM exceeded its 16 MiB memory ceiling";
const SAFEPOINT_LIMIT_ERROR: &str = "script cycle exceeds 100000 script safepoints";
const ORIGINAL_PCALL: &str = "nuxie.resource_limits.pcall";
const ORIGINAL_XPCALL: &str = "nuxie.resource_limits.xpcall";
const GUARDED_XPCALL_HANDLER: &str = "nuxie.resource_limits.xpcall_handler";
const ORIGINAL_RESUME: &str = "nuxie.resource_limits.resume";

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

    pub(crate) fn note_host_command(&self) {
        self.tracker
            .host_commands
            .set(self.tracker.host_commands.get().saturating_add(1));
    }

    pub(crate) fn host_callback_failure(&self) -> Option<String> {
        self.tracker.host_callback_failure.borrow().clone()
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
    host_callback_failure: Rc<RefCell<Option<String>>>,
    host_commands: Rc<Cell<usize>>,
}

impl ResourceLimitTracker {
    pub(super) fn begin_cycle(&self) {
        self.tripped.set(None);
        self.message.borrow_mut().take();
        self.callback_failure.borrow_mut().take();
        self.host_callback_failure.borrow_mut().take();
        self.host_commands.set(0);
    }

    pub(super) fn host_command_count(&self) -> usize {
        self.host_commands.get()
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
        // A callback error's cause is never a memory error
        // (`CallbackCause`), so there is no chain to follow.
        if let Error::MemoryError(_) = error {
            self.record(ScriptResourceLimit::Memory);
        }
    }

    pub(super) fn observe_callback_failure(&self, error: &Error) {
        self.observe_callback_diagnostic(error);
        let mut failure = self.host_callback_failure.borrow_mut();
        if failure.is_none() {
            *failure = Some(error.to_string());
        }
    }

    pub(super) fn observe_callback_diagnostic(&self, error: &Error) {
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
    lua.set_named_registry_value(ORIGINAL_PCALL, original)?;
    let pcall_tracker = tracker.clone();
    // Each guard passes on the loan its call carries (a draw call's
    // renderer), so the protected body reaches what the caller could.
    let guarded = lua.create_function_lent(move |lua, loan, args: MultiValue| {
        pcall_tracker.reject_if_tripped()?;
        let original: Function = lua.named_registry_value(ORIGINAL_PCALL)?;
        let results: MultiValue = original.call_with(loan, args)?;
        pcall_tracker.record_protected_memory_result(&results)?;
        pcall_tracker.reject_if_tripped()?;
        Ok(results)
    })?;
    lua.globals().set("pcall", guarded)?;

    let original: Function = lua.globals().get("xpcall")?;
    lua.set_named_registry_value(ORIGINAL_XPCALL, original)?;
    // The script's handlers for the xpcalls in progress, innermost last. A
    // guarded xpcall runs its body through a host call, which nothing yields
    // across, so the xpcalls a handler can run under are properly nested.
    let handlers: Rc<RefCell<Vec<Function>>> = Rc::default();
    // One guarded handler for every xpcall, created here and never scoped:
    // a script that keeps it (through `debug.info`) and calls it later gets
    // an ordinary error, not a destructed-callback error object. It is a host
    // function, as the per-call handler was, so the frames a handler sees are
    // unchanged.
    let handler_tracker = tracker.clone();
    let active_handlers = Rc::clone(&handlers);
    let guarded_handler = lua.create_function_lent(move |_, loan, error: Value| {
        let handler = active_handlers
            .try_borrow()
            .ok()
            .and_then(|handlers| handlers.last().cloned())
            .ok_or_else(|| Error::runtime("xpcall error handler called outside its xpcall"))?;
        handler_tracker.record_protected_memory_error(&error)?;
        handler_tracker.reject_if_tripped()?;
        handler.call_with::<MultiValue>(loan, error)
    })?;
    // In the registry, like the originals: a host closure that held the
    // handle would keep the VM alive.
    lua.set_named_registry_value(GUARDED_XPCALL_HANDLER, guarded_handler)?;
    let xpcall_tracker = tracker.clone();
    let guarded = lua.create_function_lent(move |lua, loan, mut args: MultiValue| {
        xpcall_tracker.reject_if_tripped()?;
        let original: Function = lua.named_registry_value(ORIGINAL_XPCALL)?;
        // A function handler runs behind the guard; anything else reaches
        // the original xpcall as it is.
        let mut active = None;
        if let Some(slot) = args.get_mut(1)
            && let Value::Function(handler) = slot
        {
            let guarded_handler: Function = lua.named_registry_value(GUARDED_XPCALL_HANDLER)?;
            active = Some(ActiveHandler::push(&handlers, handler.clone())?);
            *slot = Value::Function(guarded_handler);
        }
        let results = original.call_with::<MultiValue>(loan, args);
        drop(active);
        let results = results?;
        xpcall_tracker.record_protected_memory_result(&results)?;
        xpcall_tracker.reject_if_tripped()?;
        Ok(results)
    })?;
    lua.globals().set("xpcall", guarded)?;

    let coroutine: Table = lua.globals().get("coroutine")?;
    let original: Function = coroutine.get("resume")?;
    lua.set_named_registry_value(ORIGINAL_RESUME, original)?;
    let guarded = lua.create_function_lent(move |lua, loan, args: MultiValue| {
        tracker.reject_if_tripped()?;
        let original: Function = lua.named_registry_value(ORIGINAL_RESUME)?;
        let results: MultiValue = original.call_with(loan, args)?;
        tracker.record_protected_memory_result(&results)?;
        tracker.reject_if_tripped()?;
        Ok(results)
    })?;
    coroutine.set("resume", guarded)
}

/// A script handler on the stack of xpcalls in progress, popped when the
/// xpcall that pushed it returns.
struct ActiveHandler {
    handlers: Rc<RefCell<Vec<Function>>>,
}

impl ActiveHandler {
    fn push(handlers: &Rc<RefCell<Vec<Function>>>, handler: Function) -> Result<Self> {
        handlers
            .try_borrow_mut()
            .map_err(|_| Error::runtime("xpcall handlers are in use"))?
            .push(handler);
        Ok(Self {
            handlers: Rc::clone(handlers),
        })
    }
}

impl Drop for ActiveHandler {
    fn drop(&mut self) {
        // Only the guarded handler borrows the stack, and only for the
        // statement that reads it, so this borrow is free; a drop must not
        // panic either way. The popped handle drops after the borrow ends.
        let popped = self
            .handlers
            .try_borrow_mut()
            .ok()
            .and_then(|mut handlers| handlers.pop());
        drop(popped);
    }
}

fn resource_limit_error(limit: ScriptResourceLimit) -> Error {
    match limit {
        ScriptResourceLimit::Memory => Error::MemoryError(MEMORY_LIMIT_ERROR.to_owned()),
        ScriptResourceLimit::Safepoints => Error::runtime(SAFEPOINT_LIMIT_ERROR),
        ScriptResourceLimit::Extension(code) => Error::runtime(code),
    }
}
