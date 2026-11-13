//! Errors (plan decision D3): every guest-reachable failure is a returned
//! error, never a panic.
//!
//! Inside the VM a Lua error is a [`Throw`]: the error *value* plus its
//! status, carried by `Result` through every Rust frame between the raise
//! and the `pcall` (or host call) that catches it. The value is rooted in
//! `Core::thrown` while it is in flight, so a collection between raise and
//! catch cannot free it.
//!
//! The host sees [`Error`], which owns only plain data (the rendered message
//! and a kind), never an unrooted object reference.

use std::fmt;

use crate::value::Value;

/// Why a chunk was rejected at load time (D7). Loading never runs code, so
/// these are never Lua errors; the host decides what to do with them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadError {
    /// The compiler's error marker: the chunk is a compile error message.
    CompileError(String),
    /// The bytecode version is outside what this VM admits.
    UnsupportedVersion(u8),
    /// The type-info version is outside 1..=3.
    UnsupportedTypesVersion(u8),
    /// The chunk is larger than `Limits::max_bytecode_bytes`.
    TooLarge { size: usize, limit: usize },
    /// The bytes are not a well-formed chunk, or a prototype fails
    /// verification. The message names the prototype and instruction.
    Malformed(String),
    /// Well-formed bytecode that uses a feature this VM does not implement
    /// yet (a later lane's work). The message names the feature.
    Unsupported(String),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::CompileError(message) => f.write_str(message),
            LoadError::UnsupportedVersion(version) => {
                write!(
                    f,
                    "bytecode version mismatch (expected [{}..{}], got {version})",
                    crate::bytecode::VERSION_MIN,
                    crate::bytecode::VERSION_MAX
                )
            }
            LoadError::UnsupportedTypesVersion(version) => {
                write!(
                    f,
                    "bytecode type version mismatch (expected [1..3], got {version})"
                )
            }
            LoadError::TooLarge { size, limit } => {
                write!(f, "bytecode is {size} bytes, over the {limit}-byte limit")
            }
            LoadError::Malformed(message) => write!(f, "malformed bytecode: {message}"),
            LoadError::Unsupported(message) => write!(f, "unsupported bytecode: {message}"),
        }
    }
}

/// The status a Lua error was raised with (C++ `lua_Status`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// `LUA_ERRRUN`: `error()`, a failed operation, a host error.
    Runtime,
    /// `LUA_ERRMEM`: a memory limit was hit.
    Memory,
    /// `LUA_ERRERR`: an error while running an error handler.
    ErrorHandler,
}

/// An error the VM returns to the host.
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    /// `Core::load` rejected the chunk.
    Load(LoadError),
    /// A Lua error escaped to the host. `message` is the error value
    /// rendered as text (`tostring` for strings and numbers, otherwise
    /// `(error object is a <type> value)`).
    Lua { kind: ErrorKind, message: Vec<u8> },
    /// The host passed a stale or out-of-range slot or handle, or used the
    /// API where it cannot run (a call while the main thread is parked in a
    /// yield).
    Host(String),
    /// A value could not be converted (`convert.rs`). `position` is the
    /// 1-based argument it was at, when it was one of several.
    Conversion {
        from: String,
        to: &'static str,
        position: Option<usize>,
    },
}

impl Error {
    /// The message as lossy UTF-8.
    pub fn message(&self) -> String {
        match self {
            Error::Load(error) => error.to_string(),
            Error::Lua { message, .. } => String::from_utf8_lossy(message).into_owned(),
            Error::Host(message) => message.clone(),
            Error::Conversion { from, to, .. } => {
                format!("error converting Lua {from} to {to}")
            }
        }
    }

    /// A runtime error with this message (the error value is the message,
    /// unprefixed; the old binding crate's `Error::runtime`).
    pub fn runtime(message: impl Into<String>) -> Error {
        Error::Lua {
            kind: ErrorKind::Runtime,
            message: message.into().into_bytes(),
        }
    }

    /// The status of a Lua error; `None` for load, host and conversion
    /// errors.
    pub fn kind(&self) -> Option<ErrorKind> {
        match self {
            Error::Lua { kind, .. } => Some(*kind),
            _ => None,
        }
    }

    /// Whether this is the out-of-memory error (`LUA_ERRMEM`).
    pub fn is_memory(&self) -> bool {
        self.kind() == Some(ErrorKind::Memory)
    }

    /// A conversion error without a position gets `position`.
    pub(crate) fn at_position(self, position: usize) -> Error {
        match self {
            Error::Conversion {
                from,
                to,
                position: None,
            } => Error::Conversion {
                from,
                to,
                position: Some(position),
            },
            other => other,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message())
    }
}

impl std::error::Error for Error {}

impl From<LoadError> for Error {
    fn from(error: LoadError) -> Error {
        Error::Load(error)
    }
}

/// A Lua error in flight inside the VM.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Throw {
    pub(crate) value: Value,
    pub(crate) kind: ErrorKind,
}

/// The result type of every fallible VM-internal operation.
pub(crate) type VmResult<T> = Result<T, Throw>;
