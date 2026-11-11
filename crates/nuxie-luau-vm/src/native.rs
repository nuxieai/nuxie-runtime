//! Native (Rust) functions: the VM's own builtins and host closures.
//!
//! Calling convention (C++ `lua_CFunction`): the arguments are the stack
//! slots `args.base .. args.base + args.nargs`; the function pushes its
//! results and returns how many there are; they are the top `n` slots.
//! A builtin may call back into the VM (`Vm::call_value`), which can run
//! the collector, so any object it holds across such a call must be in a
//! stack slot, never only in a Rust local.

use std::rc::Rc;

use crate::error::VmResult;
use crate::host::{HostCtx, HostError};
use crate::value::Value;
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

/// A builtin function of the VM's libraries.
pub(crate) type BuiltinFn = fn(&mut Vm, Args) -> VmResult<usize>;

/// A host closure registered through the public API.
pub type HostFn = dyn Fn(&mut HostCtx<'_>) -> Result<usize, HostError>;

/// A host function's continuation (C++ `lua_Continuation`): runs when a
/// call it requested returns (or, for a protected call, fails), or when a
/// yield it made is resumed.
pub type HostContFn = dyn Fn(&mut HostCtx<'_>, crate::host::CallStatus) -> Result<usize, HostError>;

#[derive(Clone)]
pub(crate) enum NativeFn {
    Builtin(BuiltinFn),
    /// A builtin with upvalues (C++ `lua_pushcclosure` with `nup > 0`).
    BuiltinWith(BuiltinFn, Rc<[Value]>),
    Host(Rc<HostFn>),
    /// A host function with a continuation.
    HostK(Rc<HostFn>, Rc<HostContFn>),
}

impl NativeFn {
    /// Visits the values this function references, for the collector.
    pub(crate) fn trace(&self, mut visit: impl FnMut(Value)) {
        if let NativeFn::BuiltinWith(_, upvals) = self {
            for &v in upvals.iter() {
                visit(v);
            }
        }
    }
}
