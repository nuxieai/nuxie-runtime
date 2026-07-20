use std::cell::Cell;
use std::rc::Rc;

use luaur_rt::{Error, Function, Lua, MultiValue, Result, Table, Value};

const MEMORY_EXHAUSTED: &str = "not enough memory";
const MEMORY_LIMIT_ERROR: &str = "script VM exceeded its 16 MiB memory ceiling";
const HOST_IDENTIFIER_LIMIT_ERROR: &str = "host identifier exceeds maximum size 4096 bytes";
const HOST_STRING_LIMIT_ERROR: &str = "host string exceeds maximum size 1048576 bytes";
const HOST_DEPTH_LIMIT_ERROR: &str = "host value exceeds maximum depth 32";
const HOST_NODE_LIMIT_ERROR: &str = "host value exceeds maximum node count 4096";
const HOST_EDGE_LIMIT_ERROR: &str = "host value exceeds maximum edge count 16384";
const HOST_VALUE_BYTES_LIMIT_ERROR: &str =
    "host value exceeds maximum aggregate size 4194304 bytes";
const COMMAND_LIMIT_ERROR: &str = "script cycle exceeds 256 host commands";
const COMMAND_CONTENT_LIMIT_ERROR: &str =
    "script cycle exceeds 4194304 bytes of host-command content";
const SAFEPOINT_LIMIT_ERROR: &str = "script cycle exceeds 100000 script safepoints";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptResourceLimit {
    Memory,
    HostIdentifier,
    HostString,
    HostDepth,
    HostNodes,
    HostEdges,
    HostValueBytes,
    Commands,
    CommandContent,
    Safepoints,
}

impl ScriptResourceLimit {
    /// Stable machine-readable identity for diagnostics above the VM layer.
    pub const fn code(self) -> &'static str {
        match self {
            Self::Memory => "script.resource.memory",
            Self::HostIdentifier => "script.resource.host_identifier",
            Self::HostString => "script.resource.host_string",
            Self::HostDepth => "script.resource.host_depth",
            Self::HostNodes => "script.resource.host_nodes",
            Self::HostEdges => "script.resource.host_edges",
            Self::HostValueBytes => "script.resource.host_value_bytes",
            Self::Commands => "script.resource.host_commands",
            Self::CommandContent => "script.resource.host_command_content",
            Self::Safepoints => "script.resource.safepoints",
        }
    }
}

/// Per-VM, per-cycle terminal resource state. Nuxie-owned limit sites trip this
/// tracker before raising their ordinary Luau error. Protected-call wrappers
/// then consult the typed side channel instead of trying to recover the kind
/// from Luau's stringified callback error.
#[derive(Clone, Default)]
pub(super) struct ResourceLimitTracker {
    tripped: Rc<Cell<Option<ScriptResourceLimit>>>,
}

impl ResourceLimitTracker {
    pub(super) fn begin_cycle(&self) {
        self.tripped.set(None);
    }

    pub(super) fn terminal_limit(&self) -> Option<ScriptResourceLimit> {
        self.tripped.get()
    }

    pub(super) fn fail(&self, limit: ScriptResourceLimit) -> Error {
        let terminal = self.record(limit);
        resource_limit_error(terminal)
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

    fn record(&self, limit: ScriptResourceLimit) -> ScriptResourceLimit {
        match self.tripped.get() {
            Some(terminal) => terminal,
            None => {
                self.tripped.set(Some(limit));
                limit
            }
        }
    }

    pub(super) fn reject_if_tripped(&self) -> Result<()> {
        match self.tripped.get() {
            Some(limit) => Err(resource_limit_error(limit)),
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
        ScriptResourceLimit::HostIdentifier => Error::runtime(HOST_IDENTIFIER_LIMIT_ERROR),
        ScriptResourceLimit::HostString => Error::runtime(HOST_STRING_LIMIT_ERROR),
        ScriptResourceLimit::HostDepth => Error::runtime(HOST_DEPTH_LIMIT_ERROR),
        ScriptResourceLimit::HostNodes => Error::runtime(HOST_NODE_LIMIT_ERROR),
        ScriptResourceLimit::HostEdges => Error::runtime(HOST_EDGE_LIMIT_ERROR),
        ScriptResourceLimit::HostValueBytes => Error::runtime(HOST_VALUE_BYTES_LIMIT_ERROR),
        ScriptResourceLimit::Commands => Error::runtime(COMMAND_LIMIT_ERROR),
        ScriptResourceLimit::CommandContent => Error::runtime(COMMAND_CONTENT_LIMIT_ERROR),
        ScriptResourceLimit::Safepoints => Error::runtime(SAFEPOINT_LIMIT_ERROR),
    }
}
