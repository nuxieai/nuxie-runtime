//! The old binding crate's `Error`, with the same variants the bindings match
//! and the same `Display` text, plus the two bridges to the safe VM: its errors
//! in ([`Error::from_vm`]) and a callback's error out ([`Error::into_host`]).

use std::error::Error as StdError;
use std::fmt;
use std::sync::Arc;

use super::vm;

type DynStdError = dyn StdError + Send + Sync;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Error {
    SyntaxError {
        message: String,
        incomplete_input: bool,
    },
    RuntimeError(String),
    /// Raised like `luaL_error`: the calling script position is prepended.
    LuaLRuntimeError(String),
    MemoryError(String),
    FromLuaConversionError {
        from: &'static str,
        to: String,
        message: Option<String>,
    },
    ToLuaConversionError {
        from: &'static str,
        to: &'static str,
        message: Option<String>,
    },
    UserDataTypeMismatch,
    UserDataDestructed,
    CallbackDestructed,
    CallbackError {
        traceback: String,
        cause: Arc<Error>,
    },
    UserDataBorrowError,
    UserDataBorrowMutError,
    CoroutineUnresumable,
    MismatchedRegistryKey,
    RecursiveMutCallback,
    PreviouslyResumedPanic,
    ExternalError(Arc<DynStdError>),
}

impl Error {
    pub fn runtime<S: fmt::Display>(message: S) -> Self {
        Error::RuntimeError(message.to_string())
    }

    pub fn lua_l_runtime<S: fmt::Display>(message: S) -> Self {
        Error::LuaLRuntimeError(message.to_string())
    }

    pub fn downcast_ref<T: StdError + 'static>(&self) -> Option<&T> {
        match self {
            Error::ExternalError(e) => e.downcast_ref::<T>(),
            _ => None,
        }
    }

    pub fn external<T: Into<Box<DynStdError>>>(err: T) -> Self {
        let boxed: Box<DynStdError> = err.into();
        match boxed.downcast::<Error>() {
            Ok(e) => *e,
            Err(other) => Error::ExternalError(other.into()),
        }
    }

    /// An error the safe VM returned, classified as the old binding crate's
    /// `Error::from_vm` classifies a raised status:
    ///
    /// - out of memory is `MemoryError("not enough memory")`;
    /// - any other Lua error is `RuntimeError` with the error value's text
    ///   (a non-string object is classified later, with the VM at hand:
    ///   [`Error::recover_error_object`]).
    pub(crate) fn from_vm(error: vm::Error) -> Error {
        match error {
            vm::Error::Lua { kind, message } => match kind {
                vm::ErrorKind::Memory => Error::MemoryError("not enough memory".to_owned()),
                vm::ErrorKind::ErrorHandler => {
                    Error::RuntimeError("error in error handling".to_owned())
                }
                // The text as raised. A non-string error object renders as
                // `(error object is a <type> value)`; `recover_error_object`
                // tells it from a string that only reads like one.
                vm::ErrorKind::Runtime => {
                    Error::RuntimeError(String::from_utf8_lossy(&message).into_owned())
                }
            },
            vm::Error::Conversion { from, to, .. } => Error::FromLuaConversionError {
                from: static_type_name(&from),
                to: to.to_owned(),
                message: None,
            },
            vm::Error::Load(error) => Error::RuntimeError(error.to_string()),
            vm::Error::Host(message) => Error::RuntimeError(message),
        }
    }

    /// How a callback's error is raised into Lua (the old binding crate's
    /// `trampoline`): `LuaLRuntimeError` like `luaL_error`, positioned; anything
    /// else as its `Display` text, unpositioned. Out of memory keeps its status,
    /// so a host allocation failure still reaches the caller as one (the old
    /// binding crate cannot see such a failure: its allocations abort instead).
    pub(crate) fn into_host(self) -> vm::HostError {
        match self {
            Error::LuaLRuntimeError(message) => vm::HostError::new(message),
            Error::MemoryError(_) => vm::HostError::memory(),
            other => vm::HostError::raw(other.to_string()),
        }
    }

    /// Whether this error is raised as a userdata error object so the
    /// caller can match it after a protected boundary (the old binding crate's
    /// `callback::is_structured`): the scope-destruction errors, and a
    /// `CallbackError` around one, which gains a layer per boundary.
    pub(crate) fn is_structured(&self) -> bool {
        match self {
            Error::CallbackDestructed
            | Error::UserDataDestructed
            | Error::RecursiveMutCallback => true,
            Error::CallbackError { cause, .. } => cause.is_structured(),
            _ => false,
        }
    }

    /// How a callback's error is raised: a structured error as a
    /// [`WrappedError`] userdata error object (the old binding crate's
    /// `raise_structured_error`, which also falls back to text when the
    /// object cannot be made), anything else as [`Error::into_host`].
    pub(crate) fn raise(self, vm: &mut vm::Vm) -> vm::HostError {
        if !self.is_structured() {
            return self.into_host();
        }
        match vm.create_userdata(WrappedError(self.clone())) {
            Ok(object) => vm::HostError::from_value(vm::Value::UserData(object)),
            Err(_) => vm::HostError::raw(self.to_string()),
        }
    }

    /// A Lua error whose value was not a string, as the old binding crate's
    /// `pop_error` reads it: the structured error a [`WrappedError`] object
    /// carries, as `CallbackError`, or else `<non-string error>`. The safe
    /// VM renders such an object as `(error object is a <type> value)` and
    /// keeps it ([`vm::Vm::take_error_object`]); a string error that only
    /// reads like that rendering has no kept object and stays as raised.
    pub(crate) fn recover_error_object(self, vm: &mut vm::Vm) -> Error {
        let Error::RuntimeError(message) = &self else {
            return self;
        };
        if !(message.starts_with("(error object is a ") && message.ends_with(" value)")) {
            return self;
        }
        let Some(object) = vm.take_error_object() else {
            return self;
        };
        let cause = match &object {
            vm::Value::UserData(object) => object.payload(vm).ok().and_then(|payload| {
                payload
                    .downcast_ref::<std::cell::RefCell<WrappedError>>()
                    .and_then(|cell| cell.try_borrow().ok().map(|wrapped| wrapped.0.clone()))
            }),
            _ => None,
        };
        match cause {
            Some(cause) => Error::CallbackError {
                traceback: String::new(),
                cause: Arc::new(cause),
            },
            None => Error::RuntimeError(NON_STRING_ERROR.to_owned()),
        }
    }

    /// How an interrupt's error is raised (the old binding crate's
    /// `interrupt::raise_error`): a runtime error bare, anything else as its
    /// `Display` text.
    pub(crate) fn into_interrupt_error(self) -> vm::HostError {
        match self {
            Error::RuntimeError(message) => vm::HostError::raw(message),
            Error::MemoryError(_) => vm::HostError::memory(),
            other => vm::HostError::raw(other.to_string()),
        }
    }
}

/// The userdata error object a structured error travels in.
pub(crate) struct WrappedError(Error);

/// What the old binding crate reads from a raised error value that is not a
/// string.
pub(crate) const NON_STRING_ERROR: &str = "<non-string error>";


/// A Luau type name with a `'static` lifetime, for the conversion errors
/// the old binding crate builds from `lua_typename`.
pub(crate) fn static_type_name(name: &str) -> &'static str {
    match name {
        "nil" => "nil",
        "boolean" => "boolean",
        "number" => "number",
        "integer" => "integer",
        "vector" => "vector",
        "string" => "string",
        "table" => "table",
        "function" => "function",
        "thread" => "thread",
        "buffer" => "buffer",
        _ => "userdata",
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::SyntaxError { message, .. } => write!(f, "syntax error: {message}"),
            Error::RuntimeError(msg) => write!(f, "runtime error: {msg}"),
            Error::LuaLRuntimeError(msg) => write!(f, "runtime error: {msg}"),
            Error::MemoryError(msg) => write!(f, "memory error: {msg}"),
            Error::FromLuaConversionError { from, to, message } => {
                write!(f, "error converting Lua {from} to {to}")?;
                if let Some(m) = message {
                    write!(f, " ({m})")?;
                }
                Ok(())
            }
            Error::ToLuaConversionError { from, to, message } => {
                write!(f, "error converting {from} to Lua {to}")?;
                if let Some(m) = message {
                    write!(f, " ({m})")?;
                }
                Ok(())
            }
            Error::UserDataTypeMismatch => write!(f, "userdata type mismatch"),
            Error::UserDataDestructed => write!(f, "userdata used after being destructed"),
            Error::CallbackDestructed => write!(
                f,
                "a destructed callback or destructed userdata method was called"
            ),
            Error::CallbackError { cause, .. } => write!(f, "{cause}"),
            Error::UserDataBorrowError => write!(f, "userdata already mutably borrowed"),
            Error::UserDataBorrowMutError => write!(f, "userdata already borrowed"),
            Error::CoroutineUnresumable => write!(f, "cannot resume this coroutine"),
            Error::MismatchedRegistryKey => {
                write!(f, "registry key used with the wrong Lua instance")
            }
            Error::RecursiveMutCallback => write!(f, "mutable callback called recursively"),
            Error::PreviouslyResumedPanic => {
                write!(f, "previously resumed panic returned again")
            }
            Error::ExternalError(err) => write!(f, "{err}"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::ExternalError(err) => Some(&**err),
            Error::CallbackError { cause, .. } => Some(&**cause),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::external(err)
    }
}

impl From<&str> for Error {
    fn from(msg: &str) -> Self {
        Error::RuntimeError(msg.to_string())
    }
}

impl From<String> for Error {
    fn from(msg: String) -> Self {
        Error::RuntimeError(msg)
    }
}

impl From<vm::Error> for Error {
    fn from(error: vm::Error) -> Self {
        Error::from_vm(error)
    }
}

pub trait ExternalError {
    fn into_lua_err(self) -> Error;
}

impl<E: Into<Box<DynStdError>>> ExternalError for E {
    fn into_lua_err(self) -> Error {
        Error::external(self)
    }
}

pub trait ExternalResult<T> {
    fn into_lua_err(self) -> Result<T>;
}

impl<T, E: ExternalError> ExternalResult<T> for std::result::Result<T, E> {
    fn into_lua_err(self) -> Result<T> {
        self.map_err(ExternalError::into_lua_err)
    }
}
