//! The call protocol (C++ `ldo.cpp`), phase 2 lane 4: every call is a
//! frame driven by one loop, never a nested Rust call, except where the
//! host itself calls back in.
//!
//! **Frames and the driver.** The driver (`HostedVm::execute`, hosted.rs)
//! runs the frame stack until the frame it was entered for returns. Lua
//! frames run in `interp.rs`; a native frame is invoked by
//! [`Core::step_native`], except a host function, which needs the host:
//! the core leaves its loop with [`Exit::Host`], the driver runs it and
//! calls [`Core::finish_native`] (docs/luau-safe-vm-embedding-core.md,
//! section 2.2). Three things that re-entered the interpreter through
//! Rust recursion in lane 1 are frames now:
//!
//! - **Metamethods** called by an instruction (`__index`, arithmetic,
//!   comparison, `__len`, `__concat`, `__iter`, the generic `for` iterator)
//!   push their call and leave the instruction's completion in the calling
//!   Lua frame ([`Wait::Op`], see `interp::Finish`), as upstream's
//!   `luau_finishop` does for a yield.
//! - **Native functions that call back** (`pcall`, `xpcall`, `tostring`,
//!   `table.sort`, `string.gsub`, `string.format`'s `%*`,
//!   `coroutine.resume`) return a [`Request`] instead of calling, with a
//!   [`Continuation`] that runs when the callee returns: upstream's
//!   continuation functions (`lua_CFunction cont`) for every native that
//!   calls Lua, not only the yieldable ones. They are the resumable
//!   builtins: only [`Resume`] can request (native.rs). A host function's
//!   continuation is plain data ([`HostCont`]) the driver runs.
//! - **Errors** are caught in the loop: [`Core::find_catcher`] finds the
//!   nearest frame whose continuation catches ([`Catch`], `pcall` and
//!   `xpcall`), the driver runs the `xpcall` handler on the stack as it
//!   was, and [`Core::finish_catch`] unwinds and resumes that continuation.
//!
//! **What still recurses.** `HostedVm::call_value` runs a call to
//! completion from Rust: the host entry points, host callbacks (`HostCtx`)
//! and the `xpcall` handler. No builtin does. Each nesting is one
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

use crate::error::{ErrorKind, Throw, VmResult};
use crate::heap::Function;
use crate::hosted::Dispatch;
use crate::host::{CallStatus, HostError, HostErrorKind};
use crate::native::{Args, HostFnId, NativeFn};
use crate::thread::ThreadId;
use crate::value::{FnId, Value};
use crate::vm::{Core, Frame, MIN_STACK, Tm};

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
pub(crate) trait Continuation: Send {
    /// Runs when the requested callee finished. Returns the native's result
    /// count, as a builtin does, or requests another call.
    fn resume(self: Box<Self>, vm: &mut Resume<'_>, args: Args, outcome: Outcome)
    -> VmResult<usize>;

    /// Set for `pcall` and `xpcall`.
    fn catch(&self) -> Option<Catch> {
        None
    }
}

/// A host function's continuation, waiting on a call it requested or a
/// yield it made. Plain data: the driver runs host entry `k` with `data`
/// (docs/luau-safe-vm-embedding-core.md, section 2.2).
#[derive(Clone, Copy, Debug)]
pub(crate) struct HostCont {
    /// The continuation entry; `None` for a call made without one, whose
    /// results are the function's results.
    pub(crate) k: Option<HostFnId>,
    /// The running function's host data, handed to `k`.
    pub(crate) data: u32,
    /// For a protected call: what it protects.
    pub(crate) catch: Option<Catch>,
    /// The requested call's function slot, where its results land.
    pub(crate) results_from: usize,
}

/// The rest of a native after what it requested: a builtin's continuation
/// (code in the core) or a host function's (an entry the driver runs).
pub(crate) enum Cont {
    Lib(Box<dyn Continuation>),
    Host(HostCont),
}

impl Cont {
    fn catch(&self) -> Option<Catch> {
        match self {
            Cont::Lib(cont) => cont.catch(),
            Cont::Host(cont) => cont.catch,
        }
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
    Native(Cont),
    /// A native whose callee finished; its continuation runs next.
    Ready(Cont, Outcome),
    /// A native that yielded its thread (`coroutine.yield`).
    Yield,
    /// A host function that yielded with a continuation: on resume the
    /// continuation runs with the resume values on top (C++ `lua_yield`
    /// from a function pushed with `lua_pushcclosurek`).
    YieldThen(Cont),
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

/// What the native that just returned asked for. The field is private to
/// this module: a builtin requests through [`Resume`], which only the call
/// protocol hands out, and host code through [`RequestSlot::set_by_host`].
#[derive(Default)]
pub(crate) struct RequestSlot(Option<Request>);

impl RequestSlot {
    /// A host function's request (`HostCtx::call_yieldable` and friends).
    /// Library code never calls this: it requests through `Resume`.
    pub(crate) fn set_by_host(&mut self, request: Request) {
        self.0 = Some(request);
    }

    pub(crate) fn clear(&mut self) {
        self.0 = None;
    }

    fn take(&mut self) -> Option<Request> {
        self.0.take()
    }
}

/// A resumable builtin's (or continuation's) access to the VM: everything
/// `Core` offers, plus the requests. Only the call protocol constructs one.
pub(crate) struct Resume<'a> {
    vm: &'a mut Core,
    host: &'a mut dyn Dispatch,
}

impl std::ops::Deref for Resume<'_> {
    type Target = Core;
    #[inline]
    fn deref(&self) -> &Core {
        self.vm
    }
}

impl std::ops::DerefMut for Resume<'_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Core {
        self.vm
    }
}

impl Resume<'_> {
    /// The core and the host apart, for work that asks the host while it
    /// uses the core (the string matcher's interrupt).
    pub(crate) fn split(&mut self) -> (&mut Core, &mut dyn Dispatch) {
        (self.vm, self.host)
    }

    /// Call the value at `func` with the `nargs` values above it, then run
    /// `cont`. Returns the native's (ignored) result count.
    pub(crate) fn request_call(
        &mut self,
        func: usize,
        nargs: usize,
        nresults: i32,
        mode: CallMode,
        cont: Box<dyn Continuation>,
    ) -> VmResult<usize> {
        self.vm.request.0 = Some(Request::Call {
            func,
            nargs,
            nresults,
            mode,
            cont: Cont::Lib(cont),
        });
        Ok(0)
    }

    /// Yield the running thread with the top `n` slots.
    pub(crate) fn request_yield(&mut self, n: usize) -> VmResult<usize> {
        self.vm.request.0 = Some(Request::Yield { n, cont: None });
        Ok(0)
    }

    /// Resume `co` with `nargs` values at `first`.
    pub(crate) fn request_resume(
        &mut self,
        co: ThreadId,
        first: usize,
        nargs: usize,
        cont: Box<dyn Continuation>,
    ) -> VmResult<usize> {
        self.vm.request.0 = Some(Request::Resume {
            co,
            first,
            nargs,
            cont: Cont::Lib(cont),
        });
        Ok(0)
    }
}

/// What a native asked the driver to do after it returned.
pub(crate) enum Request {
    Call {
        func: usize,
        nargs: usize,
        nresults: i32,
        mode: CallMode,
        cont: Cont,
    },
    /// Yield the running thread with its top `n` slots; `cont`, if any,
    /// runs when it is resumed.
    Yield { n: usize, cont: Option<Cont> },
    /// Resume `co` with the `nargs` values at `first`; results go to the
    /// slots at the requester's `top`.
    Resume {
        co: ThreadId,
        first: usize,
        nargs: usize,
        cont: Cont,
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
    /// The native frame on top is a host function, or a host function's
    /// continuation, for the driver to run (`hosted.rs`); it then finishes
    /// the frame with [`Core::finish_native`]. Never leaves the driver.
    Host(HostCall),
}

/// A host call the core cannot make itself (it does not know the host).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HostCall {
    /// Host entry `id`, a function, invoked with `data` and `args`.
    Call { id: HostFnId, data: u32, args: Args },
    /// Host entry `k`, a continuation, after the call or yield it waited
    /// on ended with `status`.
    Cont {
        k: HostFnId,
        data: u32,
        args: Args,
        status: CallStatus,
    },
}

/// Where the driver resumes after a caught error.
pub(crate) struct Catcher {
    /// The catching frame.
    pub(crate) index: usize,
    pub(crate) catch: Catch,
    pub(crate) throw: Throw,
}

impl Core {
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
        let proto_id = closure.proto_id;
        let proto = self.heap.proto(proto_id);
        let numparams = usize::from(proto.numparams);
        let maxstack = usize::from(proto.maxstacksize);
        let is_vararg = proto.is_vararg;
        self.save_caller_pc();
        let args = func.saturating_add(1);
        let argtop = args.saturating_add(nargs.max(numparams));
        let need = argtop
            .saturating_add(numparams)
            .saturating_add(maxstack)
            .saturating_add(MIN_STACK);
        self.ensure_stack(need)?;
        self.fill_nil(args.saturating_add(nargs), argtop);
        let base = if is_vararg {
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
            proto: Some(proto_id),
            ..Frame::default()
        });
        self.top = top;
        Ok(())
    }

    /// Pushes a native frame for `id` at `func`, to be invoked by
    /// [`Core::step_native`].
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

    /// `CALL` of a plain native (a `Plain` or `Interruptible` builtin, or a
    /// plain host function) at `func`, frameless (docs/luau-safe-vm-call-
    /// path.md, section 4.1): nothing is written for it but
    /// [`Core::frameless`], its callee slot plus one. An observer that
    /// looks at the frames while it runs pushes its frame first
    /// ([`Core::frames_observed`]). On return its results move down to
    /// `func ..` in place and the calling Lua frame keeps running; true
    /// when an observer pushed the frame (popped here), so the caller
    /// re-enters the frame loop and reloads what it cached (`getfenv` and
    /// `setfenv` change the environment). On error the frame is pushed if
    /// no observer did, so the catcher unwinds through it as through any
    /// frame.
    #[inline]
    pub(crate) fn call_frameless(
        &mut self,
        host: &mut dyn Dispatch,
        func: usize,
        call: (FnId, NativeFn),
        nargs: usize,
        nresults: i32,
        caller_top: usize,
    ) -> VmResult<bool> {
        let (_, native) = call;
        self.check_call_depth()?;
        let base = func.saturating_add(1);
        let argtop = base.saturating_add(nargs);
        self.ensure_stack(argtop.saturating_add(MIN_STACK))?;
        self.top = argtop;
        // Slots fit a `u32`: `Limits::max_stack_slots` is capped below it.
        self.frameless = u32::try_from(base).unwrap_or(0);
        let args = Args { base, nargs };
        let result = match native {
            NativeFn::Plain(f) => f(self, args),
            NativeFn::Interruptible(f) => f(self, host, args),
            NativeFn::HostPlain(hid, data) => host
                .call_plain(self, hid, data, args)
                .map_err(|e| self.raise_host_error(e)),
            NativeFn::Resumable(_) | NativeFn::HostFull(..) => {
                Err(self.runtime_error("internal error: a framed native ran frameless"))
            }
        };
        let observed = self.frameless == 0;
        let n = match result {
            Ok(n) => n,
            Err(throw) => {
                self.materialize_frameless();
                return Err(throw);
            }
        };
        self.frameless = 0;
        if observed {
            self.frames.pop();
        }
        let n = n.min(self.top.saturating_sub(base));
        let first = self.top.saturating_sub(n);
        // Results move down by register copies, not a library `memmove`:
        // the counts are small, one result into one slot most often. The
        // destination is inside the calling frame (the verifier bounds
        // `CALL`'s results by its registers): no growth.
        if let Ok(want) = usize::try_from(nresults) {
            if want == 1 {
                let v = if n == 0 { Value::Nil } else { self.get(first) };
                self.set(func, v);
            } else {
                let copy = n.min(want);
                for i in 0..copy {
                    let v = self.get(first.saturating_add(i));
                    self.set(func.saturating_add(i), v);
                }
                for i in copy..want {
                    self.set(func.saturating_add(i), Value::Nil);
                }
            }
            self.top = caller_top;
        } else {
            for i in 0..n {
                let v = self.get(first.saturating_add(i));
                self.set(func.saturating_add(i), v);
            }
            self.top = func.saturating_add(n);
        }
        self.safepoint();
        Ok(observed)
    }

    /// The frames as an observer outside the call protocol may read them
    /// (frames.rs): a pending frameless native is pushed as the real frame
    /// it stands for first.
    #[inline]
    pub(crate) fn frames_observed(&mut self) -> crate::frames::Observed<'_> {
        self.materialize_frameless();
        crate::frames::Observed::new(self, &self.frames, self.pc)
    }

    /// The frames of `thread`: the running thread's through
    /// [`Core::frames_observed`], a parked thread's as they are (a parked
    /// thread has no frameless native: a plain native cannot switch).
    pub(crate) fn thread_frames_observed(
        &mut self,
        thread: ThreadId,
    ) -> crate::frames::Observed<'_> {
        if thread == self.current {
            return self.frames_observed();
        }
        let exec = &self.heap.thread(thread).exec;
        crate::frames::Observed::new(self, &exec.frames, exec.pc)
    }

    /// Pushes the frame of the frameless native at `frameless`, if one is
    /// pending: callee slot, the closure in it, the caller's `savedpc` from
    /// `self.pc`. The frame is never run: `CALL` pops it when the native
    /// returns, a catcher truncates it on error.
    #[inline]
    fn materialize_frameless(&mut self) {
        if self.frameless != 0 {
            let base = self.frameless as usize;
            self.frameless = 0;
            self.push_frameless_frame(base);
        }
    }

    #[cold]
    fn push_frameless_frame(&mut self, base: usize) {
        let func = base.saturating_sub(1);
        let closure = match self.get(func) {
            Value::Function(id) => id,
            _ => FnId::default(),
        };
        self.save_caller_pc();
        self.frames.push(Frame {
            func,
            base,
            top: self.top.max(base).saturating_add(MIN_STACK),
            nresults: crate::vm::MULTRET,
            closure,
            nargs: self.top.saturating_sub(base),
            ..Frame::default()
        });
    }

    /// The closure of the running native (a frameless one's from its
    /// callee slot), for its upvalues: not an observation of the frames,
    /// so nothing is pushed.
    pub(crate) fn running_closure(&self) -> Option<FnId> {
        if self.frameless != 0 {
            return match self.get((self.frameless as usize).saturating_sub(1)) {
                Value::Function(id) => Some(id),
                _ => None,
            };
        }
        self.frames.last().map(|f| f.closure)
    }

    /// A free slot above everything the running frame uses, for pushing a
    /// call (C++ uses `L->top`, which equals the frame top outside of
    /// multiple-result sequences).
    pub(crate) fn scratch(&self) -> usize {
        if self.frameless != 0 {
            return self.top;
        }
        let frame_top = self.frames.last().map_or(0, |f| f.top);
        self.top.max(frame_top)
    }

    /// Whether a call is in progress on the running thread (a frameless
    /// native counts).
    pub(crate) fn has_frames(&self) -> bool {
        self.frameless != 0 || !self.frames.is_empty()
    }

    /// Marks the frame just pushed as a boundary: returning from it
    /// returns from `execute` (C++ `LUA_CALLINFO_RETURN`).
    pub(crate) fn mark_boundary(&mut self) {
        if let Some(frame) = self.frames.last_mut() {
            frame.boundary = true;
        }
    }

    /// Marks the bottom frame as the thread's base.
    pub(crate) fn mark_thread_base(&mut self) {
        if let Some(frame) = self.frames.first_mut() {
            frame.thread_base = true;
        }
    }

    /// Restores the thread after an error caught from Rust (C++
    /// `luaD_pcall`'s unwinding): closes upvalues at or above `func`, drops
    /// the frames above `depth` and restores the C-call counters.
    pub(crate) fn unwind(&mut self, depth: usize, func: usize, saved: Saved) {
        self.close_upvals(func);
        self.frames.truncate(depth);
        if self.frames.len().saturating_add(1) < self.limits.max_calls {
            self.ci_overflow = false;
        }
        self.restore_calls(saved);
    }

    // ---- running -----------------------------------------------------------

    /// Invokes the native frame on top: its function if fresh, else its
    /// continuation. Then pops it, or starts what it requested. A host
    /// function (or a host continuation) leaves the loop as
    /// [`Exit::Host`]: the driver runs it and calls [`Core::finish_native`].
    pub(crate) fn step_native(&mut self, host: &mut dyn Dispatch) -> VmResult<Flow> {
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
                match native.func {
                    NativeFn::Plain(f) => f(self, args),
                    NativeFn::Resumable(f) => f(&mut Resume { vm: self, host }, args),
                    NativeFn::Interruptible(f) => f(self, host, args),
                    NativeFn::HostPlain(id, data) => host
                        .call_plain(self, id, data, args)
                        .map_err(|e| self.raise_host_error(e)),
                    NativeFn::HostFull(id, data) => {
                        return Ok(Flow::Exit(Exit::Host(HostCall::Call { id, data, args })));
                    }
                }
            }
            Wait::Ready(Cont::Lib(cont), outcome) => {
                cont.resume(&mut Resume { vm: self, host }, args, outcome)
            }
            Wait::Ready(Cont::Host(cont), outcome) => {
                let status = match outcome {
                    Outcome::Ok => CallStatus::Ok,
                    Outcome::Err(throw) => {
                        // C++ `luaD_pcall`'s `seterrorobj` at the function's
                        // slot.
                        let func = cont.catch.map_or(self.top, |c| c.callee);
                        self.ensure_stack(func.saturating_add(1))?;
                        self.set(func, throw.value);
                        self.top = func.saturating_add(1);
                        CallStatus::Error(throw.kind)
                    }
                };
                match cont.k {
                    Some(k) => {
                        return Ok(Flow::Exit(Exit::Host(HostCall::Cont {
                            k,
                            data: cont.data,
                            args,
                            status,
                        })));
                    }
                    // No continuation (calls only; a yield without one has
                    // none): the function returns the call's results, which
                    // start at the callee's slot (the error value after a
                    // failed protected call), whatever the arguments were.
                    None => Ok(self.top.saturating_sub(cont.results_from)),
                }
            }
            other => {
                if let Some(frame) = self.frames.last_mut() {
                    frame.wait = other;
                }
                return Err(self.runtime_error("internal error: a waiting native frame ran"));
            }
        };
        self.finish_native(args, result)
    }

    /// The native on top returned `result` (its result count): pops it and
    /// hands its results on, or starts what it requested.
    pub(crate) fn finish_native(&mut self, args: Args, result: VmResult<usize>) -> VmResult<Flow> {
        let n = match result {
            Ok(n) => n,
            Err(throw) => {
                self.request.clear();
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

    /// The Lua error a host error raises: `luaL_error` (prefixed with the
    /// calling Lua position), a raw value, or a propagated Lua error.
    pub(crate) fn raise_host_error(&mut self, error: HostError) -> Throw {
        match error.0 {
            HostErrorKind::Message(message) => self.lib_error(message),
            HostErrorKind::Raw(message, kind) => self.throw_string(&message, kind),
            HostErrorKind::Throw(throw) => throw,
            HostErrorKind::Value(value) => Throw {
                value,
                kind: ErrorKind::Runtime,
            },
            HostErrorKind::Rooted { root, .. } => match self.roots.get(root) {
                Ok(value) => Throw {
                    value,
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

    /// Finds the nearest catching frame of the running thread at or above
    /// `floor` (any frame, on a thread other than the one `execute` was
    /// entered on). A coroutine that does not catch dies and its resumer
    /// gets the error (`None`). Otherwise the error is returned. The driver
    /// runs an `xpcall` handler, then [`Core::finish_catch`].
    pub(crate) fn find_catcher(
        &mut self,
        throw: Throw,
        entry: ThreadId,
        floor: usize,
    ) -> VmResult<Option<Catcher>> {
        self.request.clear();
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
            self.thread_failed(throw)?;
            return Ok(None);
        };
        Ok(Some(Catcher {
            index,
            catch,
            throw,
        }))
    }

    /// Unwinds to the catching frame and hands it the error (after the
    /// `xpcall` handler, if any, replaced it).
    pub(crate) fn finish_catch(&mut self, catcher: Catcher) {
        let Catcher {
            index,
            catch,
            throw,
        } = catcher;
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
    }

    /// C++ `luaD_pcall`'s mapping of an error handler's own failure.
    pub(crate) fn handler_failed(&mut self, throw: Throw, second: Throw) -> Throw {
        if throw.kind == ErrorKind::Memory && second.kind == ErrorKind::Memory {
            second
        } else {
            self.error_in_handler()
        }
    }
}
