//! Native (Rust) functions: the VM's own builtins and host closures.
//!
//! Calling convention (C++ `lua_CFunction`): the arguments are the stack
//! slots `args.base .. args.base + args.nargs`; the function pushes its
//! results and returns how many there are; they are the top `n` slots.
//!
//! **Two kinds of builtin, told apart by type.** A [`PlainFn`] takes the
//! VM and cannot call Lua or yield: the request methods are not on `Core`.
//! A [`ResumableFn`] takes a [`Resume`](crate::call::Resume), the only type
//! with `request_call`, `request_yield` and `request_resume`; it finishes
//! in a [`Continuation`](crate::call::Continuation). A builtin that requests
//! does not compile until it is declared resumable, so the classification
//! is the compiler's, not a list's (docs/luau-safe-vm-embedding-core.md,
//! section 2.1). An [`InterruptibleFn`] is a plain builtin that also asks
//! the host's interrupt while it works (the string matcher, which C++
//! interrupts on every pattern step).
//!
//! A **host function** is an entry of the host's table (`HostedVm`), named
//! by a [`HostFnId`] and a `u32` of host data: plain data in the core. A
//! plain one runs through the frame loop's `Dispatch`; a full one runs at
//! the driver, which knows the host.

use crate::call::Resume;
use crate::error::VmResult;
use crate::hosted::Dispatch;
use crate::vm::Core;

/// Where a native call's arguments are.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
pub(crate) type PlainFn = fn(&mut Core, Args) -> VmResult<usize>;

/// A builtin that may request a call, a yield or a resume.
pub(crate) type ResumableFn = fn(&mut Resume<'_>, Args) -> VmResult<usize>;

/// A plain builtin that asks the host's interrupt (`Dispatch::interrupt`)
/// as it goes; it cannot call Lua or yield either.
pub(crate) type InterruptibleFn = fn(&mut Core, &mut dyn Dispatch, Args) -> VmResult<usize>;

/// A plain builtin as a native function.
pub(crate) fn plain(f: PlainFn) -> NativeFn {
    NativeFn::Plain(f)
}

/// A resumable builtin as a native function.
pub(crate) fn resumable(f: ResumableFn) -> NativeFn {
    NativeFn::Resumable(f)
}

/// An interruptible builtin as a native function.
pub(crate) fn interruptible(f: InterruptibleFn) -> NativeFn {
    NativeFn::Interruptible(f)
}

/// An entry of a `HostedVm`'s host function table. It names its VM, so an
/// id used with another VM is refused rather than finding that VM's entry
/// at the same index.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HostFnId {
    vm: u32,
    index: u32,
}

impl HostFnId {
    #[inline]
    pub(crate) fn new(vm: u32, index: u32) -> HostFnId {
        HostFnId { vm, index }
    }

    #[inline]
    pub(crate) fn index(self) -> usize {
        self.index as usize
    }

    /// The id of the VM whose table the entry is in.
    #[inline]
    pub(crate) fn vm(self) -> u32 {
        self.vm
    }
}

/// A native's code. `CALL` from Lua runs the plain kinds (`Plain`,
/// `Interruptible`, `HostPlain`) frameless, in place over their argument
/// window (docs/luau-safe-vm-embedding-core.md, section 2.1); the others
/// run as frames.
#[derive(Clone, Copy)]
pub(crate) enum NativeFn {
    /// A builtin (its upvalues, if any, are in the `NativeClosure`).
    Plain(PlainFn),
    Resumable(ResumableFn),
    Interruptible(InterruptibleFn),
    /// Host entry `id` (a plain host function) with the host's `data`.
    HostPlain(HostFnId, u32),
    /// Host entry `id` (a full host function) with the host's `data`.
    HostFull(HostFnId, u32),
}

impl NativeFn {
    /// The host entry and data of a host function.
    #[inline]
    pub(crate) fn host(self) -> Option<(HostFnId, u32)> {
        match self {
            NativeFn::HostPlain(id, data) | NativeFn::HostFull(id, data) => Some((id, data)),
            _ => None,
        }
    }
}
