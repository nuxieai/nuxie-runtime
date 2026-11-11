//! What the core and a host exchange (docs/luau-safe-vm-embedding-core.md,
//! section 2.2): the error a host function raises ([`HostError`]), how a
//! requested call ended ([`CallStatus`]), and a host function's view of
//! its call ([`HostedCtx`]). The core never runs host code itself: the
//! driver (`hosted.rs`) does.

use std::any::Any;

use crate::error::{Error, ErrorKind, Throw};
use crate::hosted::HostedVm;
use crate::native::{Args, HostFnId};
use crate::roots::Root;
use crate::value::Value;

/// An error a host function raises.
pub struct HostError(pub(crate) HostErrorKind);

pub(crate) enum HostErrorKind {
    /// Raised like `luaL_error`: prefixed with the calling Lua position.
    Message(String),
    /// Raised with the message as the error value, unprefixed, with this
    /// status.
    Raw(Vec<u8>, ErrorKind),
    /// A Lua error the host received from the VM, propagated.
    Throw(Throw),
    /// Raised with this value as the error object (C++ `lua_error` on any
    /// value): one that references no object.
    Value(Value),
    /// Raised with the value `root` keeps as the error object. `_guard`,
    /// if any, keeps the root alive until the error is raised (the lane-6
    /// API's handle); without one the root is the host's.
    Rooted {
        root: Root,
        _guard: Option<Box<dyn Any>>,
    },
}

impl HostError {
    /// An error prefixed with the calling script position (`luaL_error`).
    pub fn new(message: impl Into<String>) -> HostError {
        HostError(HostErrorKind::Message(message.into()))
    }

    /// An error whose value is exactly `message`.
    pub fn raw(message: impl Into<String>) -> HostError {
        HostError(HostErrorKind::Raw(message.into().into_bytes(), ErrorKind::Runtime))
    }

    /// An error whose value is exactly these bytes.
    pub fn raw_bytes(message: impl Into<Vec<u8>>) -> HostError {
        HostError(HostErrorKind::Raw(message.into(), ErrorKind::Runtime))
    }

    /// The out-of-memory error (`LUA_ERRMEM`, "not enough memory").
    pub fn memory() -> HostError {
        HostError(HostErrorKind::Raw(
            b"not enough memory".to_vec(),
            ErrorKind::Memory,
        ))
    }

    /// An error whose value is the one `root` keeps (any Lua value, as
    /// `error(v, 0)`). The root stays the host's: a host keeps an error
    /// value rooted so raising it allocates nothing.
    pub fn from_root(root: Root) -> HostError {
        HostError(HostErrorKind::Rooted { root, _guard: None })
    }
}

/// A VM error the host got back (from a handle operation inside a
/// callback) and returns: raised again with the same message and status,
/// unprefixed. `Error` carries only the rendered text, so a non-string
/// error object arrives as its rendering.
impl From<Error> for HostError {
    fn from(error: Error) -> HostError {
        match error {
            Error::Lua { kind, message } => HostError(HostErrorKind::Raw(message, kind)),
            other => HostError(HostErrorKind::Raw(
                other.message().into_bytes(),
                ErrorKind::Runtime,
            )),
        }
    }
}

impl std::fmt::Debug for HostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            HostErrorKind::Message(m) => write!(f, "HostError({m:?})"),
            HostErrorKind::Raw(m, kind) => {
                write!(f, "HostError::raw({:?}, {kind:?})", String::from_utf8_lossy(m))
            }
            HostErrorKind::Throw(_) => f.write_str("HostError(<lua error>)"),
            HostErrorKind::Value(v) => write!(f, "HostError::from_value({})", v.type_name()),
            HostErrorKind::Rooted { .. } => f.write_str("HostError::from_root(..)"),
        }
    }
}

impl std::fmt::Display for HostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            HostErrorKind::Message(m) => f.write_str(m),
            HostErrorKind::Raw(m, _) => f.write_str(&String::from_utf8_lossy(m)),
            HostErrorKind::Throw(_) => f.write_str("<lua error>"),
            HostErrorKind::Value(v) => write!(f, "<{} error object>", v.type_name()),
            HostErrorKind::Rooted { .. } => f.write_str("<error object>"),
        }
    }
}

/// How the call a host function requested ended, as its continuation sees
/// it (C++ `lua_Continuation`'s `status`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallStatus {
    /// The call returned (its results are on top), or the thread was
    /// resumed after a yield (the resume values are on top).
    Ok,
    /// A protected call ([`HostCtx::pcall_yieldable`]) failed: the error
    /// value is on top, in place of the function and its arguments.
    Error(ErrorKind),
}

/// A host function's view of its call: arguments in, results out.
///
/// The stack API (`top`, `value`, `push_*`, `set_top`, `replace`, `insert`,
/// `remove`) is C++'s for the function's own frame: index 1 is the first
/// argument; negative indices count from the top. A host function returns
/// how many of its top values are results.
///
/// The lane-6 API names it `HostCtx`, with its own host.
pub struct HostedCtx<'a, H> {
    pub(crate) vm: &'a mut HostedVm<H>,
    pub(crate) args: Args,
    /// The continuation entry the running function's requested calls and
    /// yields resume in, when it has one
    /// ([`Vm::create_host_function_k`]).
    pub(crate) cont: Option<HostFnId>,
    /// The running function's host data (its `NativeFn::Host` data).
    pub(crate) data: u32,
}

impl<H> std::ops::Deref for HostedCtx<'_, H> {
    type Target = HostedVm<H>;
    fn deref(&self) -> &HostedVm<H> {
        self.vm
    }
}

impl<H> std::ops::DerefMut for HostedCtx<'_, H> {
    fn deref_mut(&mut self) -> &mut HostedVm<H> {
        self.vm
    }
}

/// A collector tuning parameter for [`HostCtx::gc_tune`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GcTune {
    /// Heap goal, percent (C++ `gcgoal`, default 200).
    Goal,
    /// Work per step, percent of the step size (C++ `gcstepmul`, default 200).
    StepMul,
    /// Step size in KiB (C++ `gcstepsize >> 10`, default 1).
    StepSize,
}

