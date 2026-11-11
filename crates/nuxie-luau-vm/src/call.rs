//! The call protocol (C++ `ldo.cpp`), phase 2 lane 4: every call is a
//! frame driven by one loop, never a nested Rust call, except where the
//! host itself calls back in.
//!
//! **Frames and the driver.** [`Vm::execute`] runs the frame stack until
//! the frame it was entered for returns. Lua frames run in `interp.rs`; a
//! native frame is invoked by [`Vm::step_native`]. Three things that
//! re-entered the interpreter through Rust recursion in lane 1 are frames
//! now:
//!
//! - **Metamethods** called by an instruction (`__index`, arithmetic,
//!   comparison, `__len`, `__concat`, `__iter`, the generic `for` iterator)
//!   push their call and leave the instruction's completion in the calling
//!   Lua frame ([`Wait::Op`], see `interp::Finish`), as upstream's
//!   `luau_finishop` does for a yield.
//! - **Native functions that call back** (`pcall`, `xpcall`, `tostring`,
//!   `table.sort`, `string.gsub`, `coroutine.resume`) return a
//!   [`Request`] instead of calling, with a [`Continuation`] that runs when
//!   the callee returns: upstream's continuation functions (`lua_CFunction
//!   cont`) for every native that calls Lua, not only the yieldable ones.
//! - **Errors** are caught in the loop: [`Vm::catch`] finds the nearest
//!   frame whose continuation catches ([`Catch`], `pcall` and `xpcall`),
//!   runs the `xpcall` handler on the stack as it was, unwinds and resumes
//!   that continuation.
//!
//! **What still recurses.** [`Vm::call_value`] runs a call to completion
//! from Rust: the host entry points, host callbacks (`HostCtx`), the
//! `xpcall` handler, and library code that calls a metamethod in the
//! middle of its own work (`string.format`'s `%*`). Each nesting is one
//! `execute` on the Rust stack, so it is bounded by
//! `Limits::max_reentry`, which is sized for a 1 MiB thread stack in an
//! unoptimized build (`limits.rs` records the measurement).
//!
//! **C-call counting** follows C++ exactly although nothing recurses:
//! `c_calls`/`base_c_calls` are `nCcalls`/`baseCcalls`. A counted call
//! ([`CallMode`]) adds one for as long as its frame lives; "C stack
//! overflow" is raised at `Limits::max_c_calls`; a thread may yield only
//! while `c_calls <= base_c_calls`. Counter values a frame restores are
//! dropped once its thread yields (C++ resets them in `lua_resume`), see
//! [`Saved`].
//!
//! **Rooting.** A continuation holds no `Value`: whatever it needs across
//! the call is in a stack slot of its frame (arguments, or slots it pushed),
//! which the collector roots. It may hold slot numbers, counters and plain
//! data (`gsub`'s output bytes).

use std::rc::Rc;

use crate::error::{ErrorKind, Throw, VmResult};
use crate::heap::Function;
use crate::host::{HostCtx, HostError, HostErrorKind};
use crate::native::{Args, HostContFn, HostFn, NativeFn};
use crate::thread::ThreadId;
use crate::value::{FnId, Value};
use crate::vm::{Frame, MIN_STACK, Tm, Vm};

/// How a callee finished, as its caller's continuation sees it.
pub(crate) enum Outcome {
    /// Returned; the results are where the request put them.
    Ok,
    /// Raised (delivered only to a continuation that catches).
    Err(Throw),
}

/// C-call counters to restore, valid while the thread has not yielded
/// since they were saved (`epoch`).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Saved {
    pub(crate) c_calls: u32,
    pub(crate) base_c_calls: u32,
    pub(crate) epoch: u32,
}

/// What a catching continuation (`pcall`, `xpcall`) protects.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Catch {
    /// The callee's function slot: upvalues at or above it close on error.
    pub(crate) callee: usize,
    /// `xpcall`'s handler slot.
    pub(crate) handler: Option<usize>,
    /// The counters at the protected call (C++ `luaD_pcall`'s
    /// `oldnCcalls`).
    pub(crate) saved: Saved,
}

/// The rest of a native function, after a call it requested (C++
/// continuation functions). See the module comment for the rooting rule.
pub(crate) trait Continuation {
    /// Runs when the requested callee finished. Returns the native's result
    /// count, as a builtin does, or requests another call.
    fn resume(self: Box<Self>, vm: &mut Vm, args: Args, outcome: Outcome) -> VmResult<usize>;

    /// Set for `pcall` and `xpcall`.
    fn catch(&self) -> Option<Catch> {
        None
    }
}

/// What a frame is waiting for.
#[derive(Default)]
pub(crate) enum Wait {
    /// A Lua frame that called with `CALL`, or a running native.
    #[default]
    Idle,
    /// A native frame not invoked yet.
    Fresh,
    /// A Lua frame whose instruction waits on a metamethod.
    Op(crate::interp::Finish),
    /// A native waiting on the callee it requested.
    Native(Box<dyn Continuation>),
    /// A native whose callee finished; its continuation runs next.
    Ready(Box<dyn Continuation>, Outcome),
    /// A native that yielded its thread (`coroutine.yield`).
    Yield,
    /// A host function that yielded with a continuation: on resume the
    /// continuation runs with the resume values on top (C++ `lua_yield`
    /// from a function pushed with `lua_pushcclosurek`).
    YieldThen(Box<dyn Continuation>),
}

/// How a call counts against the C-call limit (C++ `luaD_call` and its
/// variants).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CallMode {
    /// A Lua-to-Lua call, a resumed thread's first call: not counted.
    Plain,
    /// `pcall` from a yieldable thread: C++ counts it while it sets the
    /// call up (so the limit applies) and then schedules it uncounted.
    Checked,
    /// `luaD_call` from the interpreter or a native without a continuation:
    /// counted, and not yieldable inside.
    Counted,
    /// `luaD_call` from a native with a continuation, and the generic `for`
    /// iterator call (`luaD_performcally`): counted, yieldability kept.
    CountedYieldable,
}

/// What a native asked the driver to do after it returned.
pub(crate) enum Request {
    Call {
        func: usize,
        nargs: usize,
        nresults: i32,
        mode: CallMode,
        cont: Box<dyn Continuation>,
    },
    /// Yield the running thread with its top `n` slots; `cont`, if any,
    /// runs when it is resumed.
    Yield {
        n: usize,
        cont: Option<Box<dyn Continuation>>,
    },
    /// Resume `co` with the `nargs` values at `first`; results go to the
    /// slots at the requester's `top`.
    Resume {
        co: ThreadId,
        first: usize,
        nargs: usize,
        cont: Box<dyn Continuation>,
    },
}

/// Where the driver goes next.
pub(crate) enum Flow {
    /// A plain Lua caller is on top again and may continue in place.
    Inline,
    /// Re-dispatch on the top frame.
    Continue,
    /// Leave `execute`.
    Exit(Exit),
}

impl Flow {
    /// After a thread switch the frame on top is not the one that called:
    /// the caller's loop must re-dispatch.
    fn switched(self) -> Flow {
        match self {
            Flow::Inline => Flow::Continue,
            other => other,
        }
    }
}

/// Why `execute` returned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Exit {
    /// The frame `execute` was entered for returned.
    Returned,
    /// The main thread yielded to the host: `n` values at `first`.
    Yielded { first: usize, n: usize },
    /// The main thread's base function returned (results from slot 0).
    Finished,
}

impl Vm {
    // ---- requests (for natives) ------------------------------------------

    /// From a native: call the value at `func` with the `nargs` values above
    /// it, then run `cont`. Returns the native's (ignored) result count.
    pub(crate) fn request_call(
        &mut self,
        func: usize,
        nargs: usize,
        nresults: i32,
        mode: CallMode,
        cont: Box<dyn Continuation>,
    ) -> VmResult<usize> {
        self.request = Some(Request::Call {
            func,
            nargs,
            nresults,
            mode,
            cont,
        });
        Ok(0)
    }

    /// From a native: yield the running thread with the top `n` slots.
    pub(crate) fn request_yield(&mut self, n: usize) -> VmResult<usize> {
        self.request = Some(Request::Yield { n, cont: None });
        Ok(0)
    }

    /// From a native: resume `co` with `nargs` values at `first`.
    pub(crate) fn request_resume(
        &mut self,
        co: ThreadId,
        first: usize,
        nargs: usize,
        cont: Box<dyn Continuation>,
    ) -> VmResult<usize> {
        self.request = Some(Request::Resume {
            co,
            first,
            nargs,
            cont,
        });
        Ok(0)
    }

    // ---- C-call counters -------------------------------------------------

    pub(crate) fn saved_calls(&self) -> Saved {
        Saved {
            c_calls: self.c_calls,
            base_c_calls: self.base_c_calls,
            epoch: self.epoch,
        }
    }

    /// Restores saved counters; after a yield, the values the resume set.
    pub(crate) fn restore_calls(&mut self, saved: Saved) {
        if saved.epoch == self.epoch {
            self.c_calls = saved.c_calls;
            self.base_c_calls = saved.base_c_calls;
        } else {
            self.c_calls = self.resume_calls;
            self.base_c_calls = self.resume_calls;
        }
    }

    /// C++ `luaD_checkCstack` for a count of `c_calls`.
    fn check_c_stack(&mut self, c_calls: u32) -> VmResult<()> {
        let max = self.limits.max_c_calls;
        if c_calls == max {
            return Err(self.runtime_error("C stack overflow"));
        }
        if c_calls >= max.saturating_add(max / 8) {
            return Err(self.error_in_handler());
        }
        Ok(())
    }

    /// C++ `lua_isyieldable`.
    #[inline]
    pub(crate) fn yieldable(&self) -> bool {
        self.c_calls <= self.base_c_calls
    }

    // ---- pushing calls -----------------------------------------------------

    /// Records the running Lua frame's pc before a frame is pushed above it.
    #[inline]
    fn save_caller_pc(&mut self) {
        if let Some(frame) = self.frames.last_mut()
            && frame.proto.is_some()
        {
            frame.savedpc = self.pc;
        }
    }

    /// C++ `luaD_growCI`: a thread holds at most `max_calls` call infos,
    /// its base one included, before "stack overflow". The overflow grows
    /// the limit by an eighth (room for an `xpcall` handler; past it is an
    /// error in error handling) until a catch unwinds below the limit
    /// again (C++ `restore_stack_limit`), see `ci_overflow`.
    fn check_call_depth(&mut self) -> VmResult<()> {
        // The frame being pushed, plus C++'s base call info.
        let infos = self.frames.len().saturating_add(1);
        let max = self.limits.max_calls;
        if infos >= max.saturating_add(max >> 3) {
            return Err(self.error_in_handler());
        }
        if infos >= max && !self.ci_overflow {
            self.ci_overflow = true;
            return Err(self.runtime_error("stack overflow"));
        }
        Ok(())
    }

    /// C++ `restore_stack_limit`, after a catch unwound the frames.
    fn restore_call_limit(&mut self) {
        if self.frames.len().saturating_add(1) < self.limits.max_calls {
            self.ci_overflow = false;
        }
    }

    /// Pushes a call of the value at `func` (a Lua frame, or a native frame
    /// to be invoked by the driver), counted as `mode` says.
    pub(crate) fn push_call(
        &mut self,
        func: usize,
        nargs: usize,
        nresults: i32,
        mode: CallMode,
    ) -> VmResult<()> {
        let saved = self.saved_calls();
        match mode {
            CallMode::Plain => {}
            CallMode::Checked => self.check_c_stack(self.c_calls.saturating_add(1))?,
            CallMode::Counted | CallMode::CountedYieldable => {
                self.c_calls = self.c_calls.saturating_add(1);
                self.check_c_stack(self.c_calls)?;
                if mode == CallMode::CountedYieldable {
                    self.base_c_calls = self.base_c_calls.saturating_add(1);
                }
            }
        }
        let (id, nargs) = self.resolve_callable(func, nargs)?;
        if matches!(self.heap.function(id), Function::Lua(_)) {
            self.enter_lua(func, id, nargs, nresults)?;
        } else {
            self.push_native(func, id, nargs, nresults)?;
        }
        if matches!(mode, CallMode::Counted | CallMode::CountedYieldable)
            && let Some(frame) = self.frames.last_mut()
        {
            frame.restore = Some(saved);
        }
        Ok(())
    }

    /// The function to call at `func`: the value itself, or its `__call`
    /// metamethod with the value shifted in as the first argument (C++
    /// `luaV_tryfuncTM`).
    pub(crate) fn resolve_callable(
        &mut self,
        func: usize,
        nargs: usize,
    ) -> VmResult<(FnId, usize)> {
        let callee = self.get(func);
        if let Value::Function(id) = callee {
            return Ok((id, nargs));
        }
        let tm = self.tm_by_obj(callee, Tm::Call);
        let Value::Function(id) = tm else {
            return Err(self.type_error(callee, "call"));
        };
        let end = func.saturating_add(nargs).saturating_add(2);
        self.ensure_stack(end.saturating_add(MIN_STACK))?;
        self.copy_slots(func, func.saturating_add(1), nargs.saturating_add(1));
        self.set(func, tm);
        self.top = end;
        Ok((id, nargs.saturating_add(1)))
    }

    /// Pushes a Lua frame for the closure at `func` (C++ `luaD_precall`
    /// plus `PREPVARARGS`, done here so the frame layout is fixed from the
    /// first instruction).
    pub(crate) fn enter_lua(
        &mut self,
        func: usize,
        id: FnId,
        nargs: usize,
        nresults: i32,
    ) -> VmResult<()> {
        self.check_call_depth()?;
        let Function::Lua(closure) = self.heap.function(id) else {
            return Err(self.type_error(Value::Function(id), "call"));
        };
        let proto = Rc::clone(&closure.proto);
        self.save_caller_pc();
        let numparams = usize::from(proto.numparams);
        let maxstack = usize::from(proto.maxstacksize);
        let args = func.saturating_add(1);
        let argtop = args.saturating_add(nargs.max(numparams));
        let need = argtop
            .saturating_add(numparams)
            .saturating_add(maxstack)
            .saturating_add(MIN_STACK);
        self.ensure_stack(need)?;
        self.fill_nil(args.saturating_add(nargs), argtop);
        let base = if proto.is_vararg {
            for i in 0..numparams {
                let v = self.get(args.saturating_add(i));
                self.set(argtop.saturating_add(i), v);
                self.set(args.saturating_add(i), Value::Nil);
            }
            argtop
        } else {
            args
        };
        let top = base.saturating_add(maxstack);
        self.fill_nil(base.saturating_add(numparams), top);
        self.frames.push(Frame {
            func,
            base,
            top,
            nresults,
            closure: id,
            proto: Some(proto),
            ..Frame::default()
        });
        self.top = top;
        Ok(())
    }

    /// Pushes a native frame for `id` at `func`, to be invoked by
    /// [`Vm::step_native`].
    pub(crate) fn push_native(
        &mut self,
        func: usize,
        id: FnId,
        nargs: usize,
        nresults: i32,
    ) -> VmResult<()> {
        self.check_call_depth()?;
        self.save_caller_pc();
        let base = func.saturating_add(1);
        let argtop = base.saturating_add(nargs);
        self.ensure_stack(argtop.saturating_add(MIN_STACK))?;
        self.top = argtop;
        self.frames.push(Frame {
            func,
            base,
            top: argtop.saturating_add(MIN_STACK),
            nresults,
            closure: id,
            nargs,
            wait: Wait::Fresh,
            ..Frame::default()
        });
        Ok(())
    }

    // ---- running -----------------------------------------------------------

    /// Runs frames until the frame on top at entry returns (or, for the
    /// main thread's base frame, until it finishes or yields to the host).
    /// Errors are caught here by `pcall`/`xpcall` frames at or above that
    /// frame; an error no frame catches is returned with the frames left in
    /// place for the caller to unwind.
    pub(crate) fn execute(&mut self) -> VmResult<Exit> {
        self.execute_from(self.frames.len().saturating_sub(1))
    }

    /// `execute` catching errors in the frames from `floor` up.
    pub(crate) fn execute_from(&mut self, floor: usize) -> VmResult<Exit> {
        let entry = self.current;
        loop {
            match self.run_frames() {
                Ok(exit) => return Ok(exit),
                Err(throw) => self.catch(throw, entry, floor)?,
            }
        }
    }

    /// C++ `luaD_call` from Rust: calls the value at `func` with the `nargs`
    /// values above it and runs it to completion. Results are left at
    /// `func ..`: exactly `nresults` of them, or all of them with `top`
    /// after the last when `nresults` is `MULTRET`. Each nesting holds one
    /// `execute` on the Rust stack, bounded by `Limits::max_reentry`.
    pub(crate) fn call_value(&mut self, func: usize, nargs: usize, nresults: i32) -> VmResult<()> {
        if self.reentry >= self.limits.max_reentry {
            return Err(self.runtime_error("C stack overflow"));
        }
        self.reentry = self.reentry.saturating_add(1);
        let saved_pc = self.pc;
        let result = self.call_value_inner(func, nargs, nresults);
        self.reentry = self.reentry.saturating_sub(1);
        self.pc = saved_pc;
        result
    }

    fn call_value_inner(&mut self, func: usize, nargs: usize, nresults: i32) -> VmResult<()> {
        self.push_call(func, nargs, nresults, CallMode::Counted)?;
        if let Some(frame) = self.frames.last_mut() {
            frame.boundary = true;
        }
        match self.execute()? {
            Exit::Returned => Ok(()),
            _ => Err(self.runtime_error("internal error: a thread left a host call")),
        }
    }

    /// Invokes the native frame on top: its function if fresh, else its
    /// continuation. Then pops it, or starts what it requested.
    pub(crate) fn step_native(&mut self) -> VmResult<Flow> {
        let Some(frame) = self.frames.last_mut() else {
            return Ok(Flow::Exit(Exit::Returned));
        };
        let wait = std::mem::take(&mut frame.wait);
        let args = Args {
            base: frame.base,
            nargs: frame.nargs,
        };
        let closure = frame.closure;
        let result = match wait {
            Wait::Fresh => {
                let Function::Native(native) = self.heap.function(closure) else {
                    return Err(self.type_error(Value::Function(closure), "call"));
                };
                match native.func.clone() {
                    NativeFn::Builtin(b) | NativeFn::BuiltinWith(b, _) => b(self, args),
                    NativeFn::Host(h) => self.call_host(&h, None, args),
                    NativeFn::HostK(h, k) => self.call_host(&h, Some(k), args),
                }
            }
            Wait::Ready(cont, outcome) => cont.resume(self, args, outcome),
            other => {
                if let Some(frame) = self.frames.last_mut() {
                    frame.wait = other;
                }
                return Err(self.runtime_error("internal error: a waiting native frame ran"));
            }
        };
        let n = match result {
            Ok(n) => n,
            Err(throw) => {
                self.request = None;
                return Err(throw);
            }
        };
        if let Some(request) = self.request.take() {
            return self.begin_request(request);
        }
        let n = n.min(self.top.saturating_sub(args.base));
        let first = self.top.saturating_sub(n);
        let Some(done) = self.frames.pop() else {
            return Ok(Flow::Exit(Exit::Returned));
        };
        self.move_results(done.func, first, n, done.nresults)?;
        self.safepoint();
        self.deliver(done)
    }

    fn begin_request(&mut self, request: Request) -> VmResult<Flow> {
        match request {
            Request::Call {
                func,
                nargs,
                nresults,
                mode,
                cont,
            } => {
                if let Some(frame) = self.frames.last_mut() {
                    frame.wait = Wait::Native(cont);
                }
                self.push_call(func, nargs, nresults, mode)?;
                Ok(Flow::Continue)
            }
            // A switch leaves another thread's frame on top: never `Inline`.
            Request::Yield { n, cont } => self.yield_thread(n, cont).map(Flow::switched),
            Request::Resume {
                co,
                first,
                nargs,
                cont,
            } => {
                if let Some(frame) = self.frames.last_mut() {
                    frame.wait = Wait::Native(cont);
                }
                self.resume_thread(co, first, nargs).map(Flow::switched)
            }
        }
    }

    /// Runs a host function, or its continuation `k`, as a native.
    pub(crate) fn call_host(
        &mut self,
        h: &Rc<HostFn>,
        k: Option<Rc<HostContFn>>,
        args: Args,
    ) -> VmResult<usize> {
        self.run_host(args, k, |ctx| h(ctx))
    }

    pub(crate) fn run_host(
        &mut self,
        args: Args,
        k: Option<Rc<HostContFn>>,
        body: impl FnOnce(&mut HostCtx<'_>) -> Result<usize, HostError>,
    ) -> VmResult<usize> {
        let pending = self.host_errors.len();
        self.host_depth = self.host_depth.saturating_add(1);
        let result = {
            let mut ctx = HostCtx {
                vm: self,
                args,
                cont: k,
            };
            body(&mut ctx)
        };
        self.host_depth = self.host_depth.saturating_sub(1);
        let result = result.map_err(|e| self.raise_host_error(e));
        self.host_errors.truncate(pending);
        if result.is_err() {
            self.request = None;
        }
        result
    }

    /// The Lua error a host error raises: `luaL_error` (prefixed with the
    /// calling Lua position), a raw value, or a propagated Lua error.
    pub(crate) fn raise_host_error(&mut self, error: HostError) -> Throw {
        match error.0 {
            HostErrorKind::Message(message) => self.lib_error(message),
            HostErrorKind::Raw(message, kind) => self.throw_string(&message, kind),
            HostErrorKind::Throw(throw) => throw,
            HostErrorKind::Value(value) => match self.lower(&value) {
                Ok(v) => Throw {
                    value: v,
                    kind: ErrorKind::Runtime,
                },
                Err(e) => self.throw_string(e.message().as_bytes(), ErrorKind::Runtime),
            },
        }
    }

    /// Moves `n` results at `first` to `func ..`, adjusting to `nresults`.
    pub(crate) fn move_results(
        &mut self,
        func: usize,
        first: usize,
        n: usize,
        nresults: i32,
    ) -> VmResult<()> {
        if let Ok(want) = usize::try_from(nresults) {
            self.ensure_stack(func.saturating_add(want))?;
            let copy = n.min(want);
            self.copy_slots(first, func, copy);
            self.fill_nil(func.saturating_add(copy), func.saturating_add(want));
            self.top = func.saturating_add(want);
        } else {
            self.copy_slots(first, func, n);
            self.top = func.saturating_add(n);
        }
        Ok(())
    }

    /// After `done` was popped and its results moved to `done.func`: hands
    /// them to whatever waits for them.
    pub(crate) fn deliver(&mut self, done: Frame) -> VmResult<Flow> {
        if let Some(saved) = done.restore {
            self.restore_calls(saved);
        }
        if done.boundary {
            return Ok(Flow::Exit(Exit::Returned));
        }
        if done.thread_base {
            return self.thread_finished(done.func);
        }
        let Some(caller) = self.frames.last_mut() else {
            return Ok(Flow::Exit(Exit::Returned));
        };
        match std::mem::take(&mut caller.wait) {
            Wait::Idle => {
                if done.nresults >= 0 {
                    self.top = caller.top;
                }
                Ok(Flow::Inline)
            }
            Wait::Op(finish) => self.finish_op(finish, done.func),
            Wait::Native(cont) => {
                caller.wait = Wait::Ready(cont, Outcome::Ok);
                Ok(Flow::Continue)
            }
            other => {
                caller.wait = other;
                Err(self.runtime_error("internal error: results for a frame that did not call"))
            }
        }
    }

    // ---- errors ------------------------------------------------------------

    /// Delivers an error to the nearest catching frame of the running
    /// thread at or above `floor` (any frame, on a thread other than the
    /// one `execute` was entered on). A coroutine that does not catch dies
    /// and its resumer gets the error. Otherwise the error is returned.
    pub(crate) fn catch(&mut self, throw: Throw, entry: ThreadId, floor: usize) -> VmResult<()> {
        self.request = None;
        let low = if self.current == entry { floor } else { 0 };
        let found = (low..self.frames.len())
            .rev()
            .find_map(|i| match &self.frames.get(i)?.wait {
                Wait::Native(cont) => cont.catch().map(|c| (i, c)),
                _ => None,
            });
        let Some((index, catch)) = found else {
            if self.current == entry {
                return Err(throw);
            }
            return self.thread_failed(throw);
        };
        let throw = match catch.handler {
            Some(slot) => self.run_error_handler(slot, throw),
            None => throw,
        };
        self.close_upvals(catch.callee);
        self.frames.truncate(index.saturating_add(1));
        self.restore_call_limit();
        self.restore_calls(catch.saved);
        if let Some(frame) = self.frames.last_mut() {
            match std::mem::take(&mut frame.wait) {
                Wait::Native(cont) => frame.wait = Wait::Ready(cont, Outcome::Err(throw)),
                other => frame.wait = other,
            }
        }
        Ok(())
    }

    /// C++ `luaD_pcall`'s error-function call: runs the handler at `slot`
    /// on the stack as it was when the error was raised, with the error's
    /// C-call count, and maps its failure as C++ does.
    fn run_error_handler(&mut self, slot: usize, throw: Throw) -> Throw {
        let at = self.scratch();
        let result = (|| {
            self.ensure_stack(at.saturating_add(2).saturating_add(MIN_STACK))?;
            self.set(at, self.get(slot));
            self.set(at.saturating_add(1), throw.value);
            self.top = at.saturating_add(2);
            self.call_value(at, 1, 1)
        })();
        match result {
            Ok(()) => Throw {
                value: self.get(at),
                kind: ErrorKind::Runtime,
            },
            Err(second) if throw.kind == ErrorKind::Memory && second.kind == ErrorKind::Memory => {
                second
            }
            Err(_) => self.error_in_handler(),
        }
    }
}
