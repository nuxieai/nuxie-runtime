//! A VM with its host (docs/luau-safe-vm-embedding-core.md, section 2.2).
//!
//! [`HostedVm`] is the [`Core`] (plain data), the host's state `H`, and
//! the host's function table. The core cannot run a host function: it
//! does not know `H`. Its frame loop leaves with [`Exit::Host`] when the
//! native on top is one, and the driver here runs the entry and hands the
//! result back with [`Core::finish_native`]. The driver also makes every
//! call that runs to completion from Rust ([`HostedVm::call_value`] and
//! what is built on it), since such a call may reach a host function.
//!
//! A host function is a plain `fn` pointer in the table, named in the core
//! by its [`HostFnId`] and a `u32` of host data (`NativeFn::Host`). A host
//! that keeps state per native (the lane-6 API's closures, `compat.rs`)
//! marks its entries as owning their data: when the collector frees such a
//! native, the core lists `(id, data)` ([`Heap::take_freed_natives`](crate::heap::Heap::take_freed_natives)) for
//! the host to release.

use crate::call::{CallMode, Exit, Flow, HostCall};
use crate::error::{Error, ErrorKind, Throw, VmResult};
use crate::host::{CallStatus, HostCtx, HostError};
use crate::meta::Meta;
use crate::native::HostFnId;
use crate::thread::ThreadId;
use crate::value::{StrId, Value};
use crate::vm::{Core, MIN_STACK};

/// The host side of a [`HostedVm`]: what the frame loop asks the host
/// while it runs (through [`Dispatch`]).
pub trait Host: 'static {
    /// C++ `lua_registeruserdatadirectfieldget`: the value of field
    /// `field` of the userdata whose payload key is `key`, read by
    /// `GETTABLEKS` before `__index`; `None` falls back to `__index`. Asked
    /// only for tags the host marked (`Core::mark_direct_field_tag`).
    fn direct_field(&mut self, tag: u8, key: u64, field: &[u8]) -> Option<f64> {
        let _ = (tag, key, field);
        None
    }
}

/// The host as the frame loop sees it: object-safe, so the interpreter is
/// compiled once, on `Core`, for every host.
pub(crate) trait Dispatch {
    fn direct_field(&mut self, tag: u8, key: u64, field: &[u8]) -> Option<f64>;
}

/// The driver's [`Dispatch`]: the host, split from the core by field.
struct Hosted<'a, H> {
    host: &'a mut H,
}

impl<H: Host> Dispatch for Hosted<'_, H> {
    #[inline]
    fn direct_field(&mut self, tag: u8, key: u64, field: &[u8]) -> Option<f64> {
        self.host.direct_field(tag, key, field)
    }
}

/// A host function: its arguments are in `ctx`; it returns how many of its
/// top values are its results (C++ `lua_CFunction`).
pub type HostFnPtr<H> = fn(&mut HostCtx<'_, H>) -> Result<usize, HostError>;

/// A host function's continuation (C++ `lua_Continuation`): runs when a
/// call the function requested returns (or, protected, fails), or when a
/// yield it made is resumed.
pub type HostContPtr<H> = fn(&mut HostCtx<'_, H>, CallStatus) -> Result<usize, HostError>;

/// One entry of a host function table.
pub enum HostEntry<H> {
    /// A function. `k` is the continuation entry its requested calls and
    /// yields resume in, if it has one.
    Full {
        f: HostFnPtr<H>,
        k: Option<HostFnId>,
    },
    /// A continuation.
    Cont(HostContPtr<H>),
}

/// The VM core with a host.
pub struct HostedVm<H> {
    pub(crate) core: Core,
    pub(crate) host: H,
    fns: Vec<HostEntry<H>>,
}

impl<H> std::ops::Deref for HostedVm<H> {
    type Target = Core;
    #[inline]
    fn deref(&self) -> &Core {
        &self.core
    }
}

impl<H> std::ops::DerefMut for HostedVm<H> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Core {
        &mut self.core
    }
}

impl<H: Host> HostedVm<H> {
    pub(crate) fn from_core(core: Core, host: H) -> HostedVm<H> {
        HostedVm {
            core,
            host,
            fns: Vec::new(),
        }
    }

    /// Adds an entry to the host function table. With `owns_data`, a
    /// native of this entry names host state by its data, which the host
    /// releases when the collector frees the native.
    pub(crate) fn add_host_fn(
        &mut self,
        entry: HostEntry<H>,
        owns_data: bool,
    ) -> Result<HostFnId, Error> {
        let id = u32::try_from(self.fns.len())
            .map(HostFnId)
            .map_err(|_| Error::Host("too many host function entries".to_owned()))?;
        self.fns.push(entry);
        self.core.heap.set_host_owned(id, owns_data);
        Ok(id)
    }

    // ---- the driver --------------------------------------------------------

    /// Runs frames until the frame on top at entry returns (or, for the
    /// main thread's base frame, until it finishes or yields to the host).
    /// Errors are caught by `pcall`/`xpcall` frames at or above that frame;
    /// an error no frame catches is returned with the frames left in place
    /// for the caller to unwind.
    pub(crate) fn execute(&mut self) -> VmResult<Exit> {
        self.execute_from(self.core.frames.len().saturating_sub(1))
    }

    /// `execute` catching errors in the frames from `floor` up.
    pub(crate) fn execute_from(&mut self, floor: usize) -> VmResult<Exit> {
        let entry = self.core.current;
        let mut host_call = None;
        loop {
            let next = match host_call.take() {
                Some(call) => Ok(Exit::Host(call)),
                None => self.core.run_frames(&mut Hosted {
                    host: &mut self.host,
                }),
            };
            let throw = match next {
                Ok(Exit::Host(call)) => match self.run_host_call(call) {
                    Ok(Flow::Exit(Exit::Host(call))) => {
                        host_call = Some(call);
                        continue;
                    }
                    Ok(Flow::Exit(exit)) => return Ok(exit),
                    Ok(Flow::Inline | Flow::Continue) => continue,
                    Err(throw) => throw,
                },
                Ok(exit) => return Ok(exit),
                Err(throw) => throw,
            };
            self.catch(throw, entry, floor)?;
        }
    }

    /// Runs the host entry the core left its loop for, then finishes the
    /// native frame.
    fn run_host_call(&mut self, call: HostCall) -> VmResult<Flow> {
        let args = match call {
            HostCall::Call { args, .. } | HostCall::Cont { args, .. } => args,
        };
        let pending = self.core.host_errors.len();
        self.core.host_depth = self.core.host_depth.saturating_add(1);
        let result = self.invoke_host(call);
        self.core.host_depth = self.core.host_depth.saturating_sub(1);
        let result = result.map_err(|e| self.core.raise_host_error(e));
        self.core.host_errors.truncate(pending);
        self.core.finish_native(args, result)
    }

    fn invoke_host(&mut self, call: HostCall) -> Result<usize, HostError> {
        match call {
            HostCall::Call { id, data, args } => {
                let Some(&HostEntry::Full { f, k }) = self.fns.get(id.index()) else {
                    return Err(HostError::raw("internal error: not a host function entry"));
                };
                let mut ctx = HostCtx {
                    vm: self,
                    args,
                    cont: k,
                    data,
                };
                f(&mut ctx)
            }
            HostCall::Cont {
                k,
                data,
                args,
                status,
            } => {
                let Some(&HostEntry::Cont(f)) = self.fns.get(k.index()) else {
                    return Err(HostError::raw("internal error: not a host continuation entry"));
                };
                let mut ctx = HostCtx {
                    vm: self,
                    args,
                    cont: Some(k),
                    data,
                };
                f(&mut ctx, status)
            }
        }
    }

    /// Delivers an error to the nearest catching frame (`Core::find_catcher`),
    /// running an `xpcall` handler first.
    fn catch(&mut self, throw: Throw, entry: ThreadId, floor: usize) -> VmResult<()> {
        let Some(mut catcher) = self.core.find_catcher(throw, entry, floor)? else {
            return Ok(());
        };
        if let Some(slot) = catcher.catch.handler {
            catcher.throw = self.run_error_handler(slot, catcher.throw);
        }
        self.core.finish_catch(catcher);
        Ok(())
    }

    /// C++ `luaD_pcall`'s error-function call: runs the handler at `slot`
    /// on the stack as it was when the error was raised, with the error's
    /// C-call count, and maps its failure as C++ does.
    fn run_error_handler(&mut self, slot: usize, throw: Throw) -> Throw {
        let at = self.core.scratch();
        let result = (|| {
            self.core
                .ensure_stack(at.saturating_add(2).saturating_add(MIN_STACK))?;
            let handler = self.core.get(slot);
            self.core.set(at, handler);
            self.core.set(at.saturating_add(1), throw.value);
            self.core.top = at.saturating_add(2);
            self.call_value(at, 1, 1)
        })();
        match result {
            Ok(()) => Throw {
                value: self.core.get(at),
                kind: ErrorKind::Runtime,
            },
            Err(second) => self.core.handler_failed(throw, second),
        }
    }

    // ---- calls from Rust ---------------------------------------------------

    /// C++ `luaD_call` from Rust: calls the value at `func` with the `nargs`
    /// values above it and runs it to completion. Results are left at
    /// `func ..`: exactly `nresults` of them, or all of them with `top`
    /// after the last when `nresults` is `MULTRET`. Each nesting holds one
    /// `execute` on the Rust stack, bounded by `Limits::max_reentry`.
    pub(crate) fn call_value(&mut self, func: usize, nargs: usize, nresults: i32) -> VmResult<()> {
        if self.core.reentry >= self.core.limits.max_reentry {
            return Err(self.core.runtime_error("C stack overflow"));
        }
        self.core.reentry = self.core.reentry.saturating_add(1);
        let saved_pc = self.core.pc;
        let result = self.call_value_inner(func, nargs, nresults);
        self.core.reentry = self.core.reentry.saturating_sub(1);
        self.core.pc = saved_pc;
        result
    }

    fn call_value_inner(&mut self, func: usize, nargs: usize, nresults: i32) -> VmResult<()> {
        self.core.push_call(func, nargs, nresults, CallMode::Counted)?;
        if let Some(frame) = self.core.frames.last_mut() {
            frame.boundary = true;
        }
        match self.execute()? {
            Exit::Returned => Ok(()),
            _ => Err(self
                .core
                .runtime_error("internal error: a thread left a host call")),
        }
    }

    /// Calls `f(args..)` from Rust with `nresults` results (0 or 1) and
    /// returns the first, restoring `top`.
    pub(crate) fn call_function(
        &mut self,
        f: Value,
        args: &[Value],
        nresults: i32,
    ) -> VmResult<Value> {
        let saved_top = self.core.top;
        let at = self.core.scratch();
        self.core.ensure_stack(
            at.saturating_add(1)
                .saturating_add(args.len())
                .saturating_add(MIN_STACK),
        )?;
        self.core.set(at, f);
        for (i, &v) in args.iter().enumerate() {
            self.core.set(at.saturating_add(1).saturating_add(i), v);
        }
        self.core.top = at.saturating_add(1).saturating_add(args.len());
        self.call_value(at, args.len(), nresults)?;
        let result = if nresults > 0 {
            self.core.get(at)
        } else {
            Value::Nil
        };
        self.core.top = saved_top;
        Ok(result)
    }

    /// Finishes a metamethod step from Rust: makes its call (`call_value`)
    /// and applies its `Post`.
    pub(crate) fn meta_now(&mut self, meta: Meta) -> VmResult<Value> {
        match meta {
            Meta::Done(v) => Ok(v),
            Meta::Call(call) => {
                let args = call.args.get(..call.nargs).unwrap_or_default();
                let v = self.call_function(call.f, args, call.nresults)?;
                self.core.apply_post(call.post, v)
            }
        }
    }

    /// C++ `luaL_tolstring`: `__tostring`, else the default rendering.
    pub(crate) fn tostring_value(&mut self, value: Value) -> VmResult<StrId> {
        let step = self.core.tostring_step(value)?;
        match self.meta_now(step)? {
            Value::String(id) => Ok(id),
            _ => Err(self.core.lib_error("'__tostring' must return a string")),
        }
    }
}
