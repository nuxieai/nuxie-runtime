//! Native (Rust) functions: the VM's own builtins and host closures.
//!
//! Calling convention (C++ `lua_CFunction`): the arguments are the stack
//! slots `args.base .. args.base + args.nargs`; the function pushes its
//! results and returns how many there are; they are the top `n` slots.
//!
//! **Two kinds of builtin, told apart by type.** A [`PlainFn`] takes the
//! VM and cannot call Lua or yield: the request methods are not on `Vm`.
//! A [`ResumableFn`] takes a [`Resume`](crate::call::Resume), the only type
//! with `request_call`, `request_yield` and `request_resume`; it finishes
//! in a [`Continuation`](crate::call::Continuation). A builtin that requests
//! does not compile until it is declared resumable, so the classification
//! is the compiler's, not a list's (docs/luau-safe-vm-embedding-core.md,
//! section 2.1).

use std::rc::Rc;

use crate::call::Resume;
use crate::error::VmResult;
use crate::host::{HostCtx, HostError};
use crate::vm::Vm;

/// Where a native call's arguments are.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Args {
    /// First argument slot (the function itself is at `base - 1`).
    pub(crate) base: usize,
    pub(crate) nargs: usize,
}

impl Args {
    /// The stack slot of argument `i` (0-based), if it was passed.
    #[inline]
    pub(crate) fn slot(self, i: usize) -> Option<usize> {
        if i < self.nargs {
            Some(self.base.saturating_add(i))
        } else {
            None
        }
    }
}

/// A builtin that neither calls Lua nor yields.
pub(crate) type PlainFn = fn(&mut Vm, Args) -> VmResult<usize>;

/// A builtin that may request a call, a yield or a resume.
pub(crate) type ResumableFn = fn(&mut Resume<'_>, Args) -> VmResult<usize>;

/// A plain builtin as a native function.
pub(crate) fn plain(f: PlainFn) -> NativeFn {
    NativeFn::Plain(f)
}

/// A resumable builtin as a native function.
pub(crate) fn resumable(f: ResumableFn) -> NativeFn {
    NativeFn::Resumable(f)
}

/// A host closure registered through the public API.
pub type HostFn = dyn Fn(&mut HostCtx<'_>) -> Result<usize, HostError>;

/// A host function's continuation (C++ `lua_Continuation`): runs when a
/// call it requested returns (or, for a protected call, fails), or when a
/// yield it made is resumed.
pub type HostContFn = dyn Fn(&mut HostCtx<'_>, crate::host::CallStatus) -> Result<usize, HostError>;

#[derive(Clone)]
pub(crate) enum NativeFn {
    /// A builtin (its upvalues, if any, are in the `NativeClosure`).
    Plain(PlainFn),
    Resumable(ResumableFn),
    Host(Rc<HostFn>),
    /// A host function with a continuation.
    HostK(Rc<HostFn>, Rc<HostContFn>),
}
