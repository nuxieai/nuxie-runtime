//! Threads (C++ `lua_State` per coroutine, `lstate.cpp`), phase 2 lane 4.
//!
//! A thread is an arena object (`Heap::threads`) addressed by a
//! [`ThreadId`]. Its execution state, [`Exec`], is the value stack, the
//! frame stack, the open upvalues and the C-call counters.
//!
//! **The running thread is checked out.** The running thread's `Exec` lives
//! in `Core::exec`, so the interpreter and the libraries address it as
//! `self.exec.stack`, `self.exec.top`, ... at a fixed offset; its arena
//! slot holds an empty `Exec` while it runs. Switching threads
//! (`Core::switch_to`) is one `mem::swap` of the `Exec` per side
//! (docs/luau-safe-vm-call-path.md, 4.4): a few `Vec` headers and
//! integers, never a copy of a stack or a frame.
//!
//! **What a thread roots** (`Thread::trace`, the collector's only view of a
//! parked thread): every stack slot below its live top (the innermost
//! frame's `top` if that is a Lua frame, else `top`, as gc.rs roots the
//! running thread), the closure of every frame, every open upvalue, the
//! thread that resumed it, the error value it died with and its globals.
//! Frame continuations hold no values by rule (see `call::LibCont`).
//! The running thread is rooted by `Core::gc_roots` from `Core::exec`
//! instead, plus its id, so its arena slot is not freed. A parked thread is
//! traversed again in the atomic step (heap.rs `GcRef`), so its stack
//! needs no write barrier.
//!
//! **Accounting.** A thread costs a fixed `THREAD_BYTES` (heap.rs) plus its
//! stack, an external buffer (budget.rs, rule 4): charged when it is
//! created or grows (`ensure_stack`, on the running thread), credited when
//! it is closed (`reset_thread`) or swept.

use std::mem;

use crate::call::{Cont, Exit, Flow, LibCont, Outcome, Wait};
use crate::error::{ErrorKind, Throw, VmResult};
use crate::heap::Upval;
pub(crate) use crate::value::ThreadId;
use crate::value::{TableId, UpvalId, Value};
use crate::vm::{Core, Frame, MIN_STACK, MULTRET};

/// Where a thread is in its life (C++ `lua_Status` plus `lua_costatus`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) enum ThreadStatus {
    /// Created; the function is in slot 0 and has not run.
    #[default]
    Fresh,
    /// Yielded; its top frame is the yielding native.
    Suspended,
    /// The running thread.
    Running,
    /// Resumed another thread and waits for it.
    Normal,
    /// Returned from its function, or was closed.
    Finished,
    /// Died with an uncaught error; the stack is kept as it was.
    Failed(ErrorKind),
}

/// The execution state that moves in and out of the `Core` on a switch.
#[derive(Default)]
pub(crate) struct Exec {
    pub(crate) stack: Vec<Value>,
    pub(crate) top: usize,
    pub(crate) frames: crate::frames::Frames,
    /// Open upvalues, sorted by stack slot.
    pub(crate) open_upvals: Vec<(usize, UpvalId)>,
    pub(crate) pc: usize,
    /// C++ `nCcalls`.
    pub(crate) c_calls: u32,
    /// C++ `baseCcalls`: the thread may yield while `c_calls` is at or
    /// below it.
    pub(crate) base_c_calls: u32,
    /// Times the thread was resumed (`call::Saved`).
    pub(crate) epoch: u32,
    /// `c_calls` at the last resume.
    pub(crate) resume_calls: u32,
    pub(crate) ci_overflow: bool,
    /// C++ `L->gt`: the thread's globals (`setfenv(0, t)` changes them for
    /// this thread only).
    pub(crate) globals: TableId,
}

#[derive(Default)]
pub(crate) struct Thread {
    pub(crate) exec: Box<Exec>,
    pub(crate) status: ThreadStatus,
    /// The thread that resumed this one, while it runs or is normal.
    pub(crate) resumer: Option<ThreadId>,
    /// The error a `Failed` thread died with (for `coroutine.close`).
    pub(crate) error: Value,
}

impl Thread {
    /// Visits every value this thread roots; see the module comment.
    pub(crate) fn trace(&self, mut visit: impl FnMut(Value)) {
        let exec = &self.exec;
        for &v in exec.stack.iter().take(self.live_top()) {
            visit(v);
        }
        for frame in exec.frames.iter() {
            visit(Value::Function(frame.closure));
        }
        for &(_, id) in &exec.open_upvals {
            visit(Value::UpRef(id));
        }
        if let Some(resumer) = self.resumer {
            visit(Value::Thread(resumer));
        }
        visit(self.error);
        visit(Value::Table(exec.globals));
    }

    /// Where the live part of the stack ends, as `Core::live_stack_top`
    /// (gc.rs): the frame's top for a Lua frame, the current top for a
    /// native one.
    fn live_top(&self) -> usize {
        let exec = &self.exec;
        match exec.frames.last() {
            Some(frame) if frame.has(Frame::LUA) => exec.top.max(frame.top()),
            _ => exec.top,
        }
        .min(exec.stack.len())
    }

    /// Sets the dead slots above the live top to nil (the collector, before
    /// it traces a parked thread).
    pub(crate) fn clear_dead_slots(&mut self) {
        let live = self.live_top();
        if let Some(dead) = self.exec.stack.get_mut(live..) {
            dead.fill(Value::Nil);
        }
    }

    /// Bytes of value stack this thread holds (charged as it grows).
    pub(crate) fn stack_bytes(&self) -> usize {
        self.exec.stack.len().saturating_mul(16)
    }
}

/// C++ `LUAI_MAXCSTACK`: the most values `lua_checkstack` lets a native
/// frame hold.
const MAX_C_STACK: usize = 8000;

/// `lua_costatus`'s names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CoStatus {
    Running,
    Suspended,
    Normal,
    Dead,
}

impl CoStatus {
    pub(crate) fn name(self) -> &'static str {
        match self {
            CoStatus::Running => "running",
            CoStatus::Suspended => "suspended",
            CoStatus::Normal => "normal",
            CoStatus::Dead => "dead",
        }
    }
}

impl Core {
    /// Swaps the running thread's checked-out state with `id`'s parked
    /// state: one swap of two [`Exec`] boxes (pointers).
    fn swap_exec(&mut self, id: ThreadId) {
        mem::swap(&mut self.exec, &mut self.heap.thread_mut(id).exec);
    }

    /// Parks the running thread and checks `to` out.
    fn switch_to(&mut self, to: ThreadId) {
        if to == self.current {
            return;
        }
        self.swap_exec(self.current);
        self.swap_exec(to);
        self.current = to;
    }

    /// A new thread whose stack holds `f` in slot 0 (C++ `lua_newthread`
    /// then `lua_xpush`).
    pub(crate) fn new_coroutine(&mut self, f: Value) -> VmResult<ThreadId> {
        let mut stack = vec![Value::Nil; MIN_STACK.saturating_mul(2)];
        if let Some(slot) = stack.first_mut() {
            *slot = f;
        }
        self.heap.new_thread(Thread {
            exec: Box::new(Exec {
                stack,
                top: 1,
                // C++ `lua_newthread`: the creator's globals.
                globals: self.exec.globals,
                ..Exec::default()
            }),
            status: ThreadStatus::Fresh,
            resumer: None,
            error: Value::Nil,
        })
    }

    /// C++ `lua_costatus` as the running thread sees `co`.
    pub(crate) fn co_status(&self, co: ThreadId) -> CoStatus {
        if co == self.current {
            return CoStatus::Running;
        }
        match self.heap.thread(co).status {
            // C++: a thread with no frames and an empty stack is dead
            // (`coroutine.close` empties it; see `thread_finished`).
            ThreadStatus::Fresh if self.heap.thread(co).exec.top == 0 => CoStatus::Dead,
            ThreadStatus::Fresh | ThreadStatus::Suspended => CoStatus::Suspended,
            ThreadStatus::Normal | ThreadStatus::Running => CoStatus::Normal,
            ThreadStatus::Finished | ThreadStatus::Failed(_) => CoStatus::Dead,
        }
    }

    /// C++ `lua_checkstack(L, n)` for the running thread: may its top
    /// frame push `n` more values?
    fn has_room(&self, n: usize) -> bool {
        let base = self.exec.frames.last().map_or(0, Frame::base);
        n <= MAX_C_STACK && self.exec.top.saturating_sub(base).saturating_add(n) <= MAX_C_STACK
    }

    /// Copies `n` values at `first` of the parked thread `from` to the top
    /// of the running thread's stack: a coroutine's results to the thread
    /// that resumed it, checked as C++ `auxresume` checks them.
    fn push_from_parked(&mut self, from: ThreadId, first: usize, n: usize) -> VmResult<()> {
        if n.saturating_add(1) > MIN_STACK && !self.has_room(n.saturating_add(1)) {
            return Err(self.lib_error("too many results to resume"));
        }
        self.push_parked_values(from, first, n)
    }

    fn push_parked_values(&mut self, from: ThreadId, first: usize, n: usize) -> VmResult<()> {
        let at = self.exec.top;
        self.ensure_stack(at.saturating_add(n).saturating_add(MIN_STACK))?;
        for i in 0..n {
            let v = self
                .heap
                .thread(from)
                .exec
                .stack
                .get(first.saturating_add(i))
                .copied()
                .unwrap_or_default();
            self.set(at.saturating_add(i), v);
        }
        self.exec.top = at.saturating_add(n);
        Ok(())
    }

    /// Hands `outcome` to the continuation of the native on top (the one
    /// that requested a resume).
    fn ready_requester(&mut self, outcome: Outcome) -> VmResult<Flow> {
        let Some(pending) = self.exec.frames.top_pending() else {
            if self.exec.frames.is_empty() {
                return Ok(Flow::Exit(Exit::Returned));
            }
            return Err(self.runtime_error("internal error: a resume without a requester"));
        };
        // `coroutine.resume` or a wrap function: it completes here, its
        // continuation read where it lies
        // (docs/luau-safe-vm-register-window.md, 4.8).
        if matches!(outcome, Outcome::Ok)
            && let Wait::Native(Cont::Lib(LibCont::Resume(cont))) = &pending.wait
        {
            let (at, wrap) = cont.parts();
            return self.resume_returned(at, wrap);
        }
        match mem::take(&mut pending.wait) {
            Wait::Native(cont) => {
                pending.wait = Wait::Ready(cont, outcome);
                Ok(Flow::Continue)
            }
            other => {
                pending.wait = other;
                Err(self.runtime_error("internal error: a resume without a requester"))
            }
        }
    }

    /// C++ `lua_resume(co, L, nargs)` for the `nargs` values at `first` of
    /// the running thread, whose top native frame waits on it. The caller
    /// has checked that `co` is suspended.
    pub(crate) fn resume_thread(
        &mut self,
        co: ThreadId,
        first: usize,
        nargs: usize,
    ) -> VmResult<Flow> {
        let from = self.current;
        let c_calls = self.exec.c_calls;
        if c_calls >= self.limits.max_c_calls {
            let throw = self.throw_string(b"C stack overflow", ErrorKind::Runtime);
            return self.ready_requester(Outcome::Err(throw));
        }
        // C++ `auxresume`'s `lua_checkstack(co, narg)`, in the resumer.
        let exec = &self.heap.thread(co).exec;
        let co_base = exec.frames.last().map_or(0, Frame::base);
        let co_top = exec.top;
        let used = co_top.saturating_sub(co_base);
        if nargs > MAX_C_STACK || used.saturating_add(nargs) > MAX_C_STACK {
            return Err(self.lib_error("too many arguments to resume"));
        }
        let fresh = self.heap.thread(co).status == ThreadStatus::Fresh;
        self.heap.thread_mut(from).status = ThreadStatus::Normal;
        {
            let thread = self.heap.thread_mut(co);
            thread.status = ThreadStatus::Running;
            thread.resumer = Some(from);
        }
        self.switch_to(co);
        self.exec.c_calls = c_calls.saturating_add(1);
        self.exec.base_c_calls = self.exec.c_calls;
        self.exec.resume_calls = self.exec.c_calls;
        self.exec.epoch = self.exec.epoch.wrapping_add(1);
        self.push_parked_values(from, first, nargs)?;
        // `lua_xmove`: the arguments leave the resumer, whose results will
        // land where they were.
        self.heap.thread_mut(from).exec.top = first;
        if fresh {
            // C++ `lua_resume` on a thread with no frames calls the value
            // below the arguments: slot 0 for a new coroutine, the last
            // result for one whose results did not fit its resumer.
            let func = co_top.saturating_sub(1);
            self.push_call(func, nargs, MULTRET, crate::call::CallMode::Plain)?;
            if let Some(frame) = self.exec.frames.first_mut() {
                frame.set(Frame::THREAD_BASE);
            }
            return Ok(Flow::Continue);
        }
        self.complete_yield(nargs)
    }

    /// `complete_yield` for a host resume of the main thread: `Some` when
    /// that already ended the run.
    pub(crate) fn complete_yield_from_host(&mut self, n: usize) -> VmResult<Option<Exit>> {
        match self.complete_yield(n)? {
            Flow::Exit(exit) => Ok(Some(exit)),
            Flow::Inline | Flow::Continue => Ok(None),
        }
    }

    /// The thread resumes after `coroutine.yield`: the `n` values on top
    /// are the yield's results.
    fn complete_yield(&mut self, n: usize) -> VmResult<Flow> {
        // A host function that yielded with a continuation: it runs now,
        // with the resume values on top of its frame.
        if let Some(pending) = self.exec.frames.top_pending()
            && matches!(pending.wait, Wait::YieldThen(_))
            && let Wait::YieldThen(cont) = mem::take(&mut pending.wait)
        {
            pending.wait = Wait::Ready(cont, Outcome::Ok);
            return Ok(Flow::Continue);
        }
        let Some((done, pending)) = self.exec.frames.pop() else {
            return Ok(Flow::Exit(Exit::Returned));
        };
        let first = self.exec.top.saturating_sub(n);
        self.move_results(done.func(), first, n, done.nresults())?;
        self.deliver(done, pending)
    }

    /// C++ `lua_yield` for the top `n` values; the native on top is
    /// `coroutine.yield` or a host function, and the thread may yield.
    /// `cont`, if any, runs when the thread is resumed.
    pub(crate) fn yield_thread(
        &mut self,
        n: usize,
        cont: Option<Cont>,
    ) -> VmResult<Flow> {
        let first = self.exec.top.saturating_sub(n);
        if let Some(pending) = self.exec.frames.top_pending_or_insert() {
            pending.wait = match cont {
                Some(cont) => Wait::YieldThen(cont),
                None => Wait::Yield,
            };
        }
        let co = self.current;
        let Some(resumer) = self.heap.thread(co).resumer else {
            // The main thread yields to the host (`Vm::resume`).
            self.heap.thread_mut(co).status = ThreadStatus::Suspended;
            return Ok(Flow::Exit(Exit::Yielded { first, n }));
        };
        {
            let thread = self.heap.thread_mut(co);
            thread.status = ThreadStatus::Suspended;
            thread.resumer = None;
        }
        self.switch_to(resumer);
        self.heap.thread_mut(resumer).status = ThreadStatus::Running;
        self.push_from_parked(co, first, n)?;
        // `lua_xmove`: the yielded values leave the coroutine.
        self.heap.thread_mut(co).exec.top = first;
        self.ready_requester(Outcome::Ok)
    }

    /// The running thread's function returned its results to `func ..`.
    pub(crate) fn thread_finished(&mut self, func: usize) -> VmResult<Flow> {
        let co = self.current;
        let n = self.exec.top.saturating_sub(func);
        let Some(resumer) = self.heap.thread(co).resumer else {
            self.heap.thread_mut(co).status = ThreadStatus::Finished;
            return Ok(Flow::Exit(Exit::Finished));
        };
        self.heap.thread_mut(co).resumer = None;
        self.switch_to(resumer);
        self.heap.thread_mut(resumer).status = ThreadStatus::Running;
        if let Err(throw) = self.push_from_parked(co, func, n) {
            // C++ leaves the results on the coroutine, whose status is then
            // `LUA_OK` with no frames: suspended until they are gone.
            self.heap.thread_mut(co).status = ThreadStatus::Fresh;
            return Err(throw);
        }
        // C++ moves the results out, leaving the dead thread empty.
        let thread = self.heap.thread_mut(co);
        thread.status = ThreadStatus::Finished;
        thread.exec.top = 0;
        self.ready_requester(Outcome::Ok)
    }

    /// An error no frame of the running coroutine catches: it dies, and its
    /// resumer's request gets the error. The dead thread's stack is kept as
    /// it was (C++ keeps it for `debug.traceback(co)`).
    pub(crate) fn thread_failed(&mut self, throw: Throw) -> VmResult<()> {
        let co = self.current;
        let Some(resumer) = self.heap.thread(co).resumer else {
            return Err(throw);
        };
        {
            let thread = self.heap.thread_mut(co);
            thread.status = ThreadStatus::Failed(throw.kind);
            thread.resumer = None;
            thread.error = throw.value;
        }
        self.switch_to(resumer);
        self.heap.thread_mut(resumer).status = ThreadStatus::Running;
        self.ready_requester(Outcome::Err(throw)).map(drop)
    }

    /// C++ `lua_resetthread` on a parked thread: closes its open upvalues
    /// and empties it.
    pub(crate) fn reset_thread(&mut self, co: ThreadId) {
        let exec = mem::take(&mut self.heap.thread_mut(co).exec);
        self.heap.thread_mut(co).exec.globals = exec.globals;
        for &(slot, id) in &exec.open_upvals {
            let v = exec.stack.get(slot).copied().unwrap_or_default();
            self.heap.set_upval(id, Upval::Closed(v));
        }
        self.heap
            .credit_external(exec.stack.len().saturating_mul(16));
        let thread = self.heap.thread_mut(co);
        thread.status = ThreadStatus::Finished;
        thread.resumer = None;
        thread.error = Value::Nil;
    }
}
