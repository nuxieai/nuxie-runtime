//! The collector's driver: safepoints, the root set, pacing and the
//! `collectgarbage` operations (C++ `luaC_step`, `luaC_fullgc`, `lua_gc`).
//! The mark, the weak-table pass and the sweep are in `heap.rs`.
//!
//! **Safepoints.** Collection runs only in [`Vm::safepoint`] and the
//! explicit operations below, all called where every live value is in a
//! root (heap.rs). A safepoint runs one incremental step when the bytes in
//! use reach `Gc::threshold`, as C++ `luaC_checkGC` does. With a memory
//! limit, it also finishes a whole cycle once the bytes in use pass half
//! the budget left after the last cycle, so a small budget is collected
//! before it runs out.
//!
//! **Pacing** follows C++ `luaC_step` and `lua_gc(LUA_GCSTEP)` with the
//! same defaults (goal 200%, step multiplier 200%, step size 1 KiB), so
//! `collectgarbage("step", n)` takes as many calls to finish a cycle as
//! the work model allows (gc.luau `dosteps`). One difference: C++
//! `getheaptrigger` starts the next cycle early by the allocation rate it
//! measured with a clock; here the trigger is the heap goal itself (C++'s
//! answer when the measured interval is under a millisecond), which keeps
//! collection deterministic.
//!
//! **Roots** (`gc_roots`): the thread (stack up to its high-water mark,
//! each frame's closure, the open upvalues), the globals, the registry,
//! the string metatable, the preallocated memory-error string, the
//! metamethod names, the pinned library values, pending host errors and
//! the pending `__namecall` name.
//!
//! Seam for lane 4 (coroutines): the running thread and the main thread
//! stay roots here (`push_thread_roots` covers the one thread lane 1 has).
//! Every other thread is a heap object reached through `Value::Thread`
//! and traversed by the collector (`GcRef` in heap.rs): its traversal does
//! what `push_thread_roots` does for its own stack, frames and open
//! upvalues, and it is re-queued for the atomic step rather than turned
//! black, because stack stores are not barriered.

use crate::heap::{CollectStats, Phase};
use crate::host::GcTune;
use crate::value::{StrId, Value};
use crate::vm::Vm;

impl Vm {
    /// Runs collector work if the allocation count calls for it.
    #[inline]
    pub(crate) fn safepoint(&mut self) {
        let used = self.heap.total_bytes();
        if used >= self.heap.gc.threshold {
            self.luac_step(true);
        } else if self.memory_pressure(used) {
            self.full_gc();
        }
    }

    /// Whether a memory limit is close enough that the cycle should finish
    /// now rather than incrementally.
    fn memory_pressure(&self, used: usize) -> bool {
        let Some(limit) = self.heap.budget.limit() else {
            return false;
        };
        if self.heap.gc.threshold == usize::MAX {
            return false;
        }
        let live = self.heap.gc.live_after_cycle;
        used > live.saturating_add(limit.saturating_sub(live) / 2)
    }

    /// The step multiplier, never zero (C++ divides by it).
    fn stepmul(&self) -> usize {
        usize::try_from(self.heap.gc.stepmul).unwrap_or(0).max(1)
    }

    /// C++ `luaC_step`: one step of `stepsize * stepmul / 100` work (more
    /// for an assist step that has fallen behind), then the next
    /// threshold. Returns the step size the work stands for.
    fn luac_step(&mut self, assist: bool) -> usize {
        let stepmul = self.stepmul();
        let stepsize = usize::try_from(self.heap.gc.stepsize).unwrap_or(0);
        let mut limit = stepsize.saturating_mul(stepmul) / 100;
        let debt = self
            .heap
            .total_bytes()
            .saturating_sub(self.heap.gc.threshold);
        if assist {
            limit = limit.max(debt.saturating_mul(stepmul) / 100);
        }
        // At least one object per step, so a zero step size (`setstepsize`
        // 0) still finishes a cycle.
        let limit = limit.max(1);
        let roots = if self.heap.step_needs_roots() {
            self.gc_roots()
        } else {
            Vec::new()
        };
        let work = self.heap.gc_step(limit, &roots);
        let actual = work
            .saturating_mul(100)
            .checked_div(stepmul)
            .unwrap_or(work);
        let used = self.heap.total_bytes();
        self.heap.gc.threshold = if self.heap.gc.phase == Phase::Pause {
            self.heap_goal(used)
        } else {
            let threshold = used.saturating_add(actual);
            if threshold >= debt {
                threshold.saturating_sub(debt)
            } else {
                threshold
            }
        };
        actual
    }

    /// C++ `(totalbytes / 100) * gcgoal`.
    fn heap_goal(&self, used: usize) -> usize {
        let goal = usize::try_from(self.heap.gc.goal).unwrap_or(0);
        (used / 100).saturating_mul(goal)
    }

    /// C++ `luaC_fullgc`: abandons a cycle in progress, runs a whole one,
    /// and sets the next threshold.
    pub(crate) fn full_gc(&mut self) -> CollectStats {
        self.heap.abandon_cycle();
        let roots = self.gc_roots();
        loop {
            self.heap.gc_step(usize::MAX, &roots);
            if self.heap.gc.phase == Phase::Pause {
                break;
            }
        }
        let used = self.heap.total_bytes();
        let goal = i64::from(self.heap.gc.goal);
        let stepmul = i64::from(self.heap.gc.stepmul).max(1);
        let factor = (goal.saturating_mul(stepmul) / 100).saturating_sub(100);
        let threshold = usize::try_from(factor)
            .ok()
            .zip(usize::try_from(stepmul).ok())
            .and_then(|(f, m)| used.checked_mul(f)?.checked_div(m))
            .unwrap_or(used);
        self.heap.gc.threshold = threshold.max(used);
        self.heap.gc.last_stats
    }

    /// C++ `lua_gc(LUA_GCSTEP, kb)`: performs about `kb` KiB of allocation
    /// worth of work (at least one step). Returns true when a cycle
    /// finished.
    pub(crate) fn gc_step(&mut self, kb: i32) -> bool {
        // C++ converts `kb` to `size_t`, so a negative amount is huge: the
        // step runs the whole cycle.
        let amount = usize::try_from(kb).map_or(usize::MAX, |kb| kb.saturating_mul(1 << 10));
        let used = self.heap.total_bytes();
        let oldcredit: i128 = if self.heap.gc.phase == Phase::Pause {
            0
        } else {
            // Signed, as C++'s `ptrdiff_t` conversion: the stopped
            // threshold (`usize::MAX`) is a debt, not a credit.
            i128::from(self.heap.gc.threshold as i64).saturating_sub(used as i128)
        };
        self.heap.gc.threshold = used.saturating_sub(amount);
        let mut actualwork = 0usize;
        let mut finished = false;
        while self.heap.gc.threshold <= self.heap.total_bytes() {
            actualwork = actualwork.saturating_add(self.luac_step(false));
            if self.heap.gc.phase == Phase::Pause {
                finished = true;
                break;
            }
        }
        if self.heap.gc.phase != Phase::Pause {
            let threshold = (self.heap.total_bytes() as i128)
                .saturating_add(actualwork as i128)
                .saturating_add(oldcredit);
            self.heap.gc.threshold = usize::try_from(threshold.max(0)).unwrap_or(usize::MAX);
        }
        finished
    }

    /// `collectgarbage("stop")` / `("restart")` (C++ sets the threshold
    /// to the maximum, or to the bytes in use).
    pub(crate) fn set_gc_running(&mut self, running: bool) {
        self.heap.gc.threshold = if running {
            self.heap.total_bytes()
        } else {
            usize::MAX
        };
    }

    /// `lua_gc(LUA_GCSETGOAL | SETSTEPMUL | SETSTEPSIZE)`.
    pub(crate) fn gc_tune(&mut self, option: GcTune, value: i32) -> i32 {
        let gc = &mut self.heap.gc;
        match option {
            GcTune::Goal => std::mem::replace(&mut gc.goal, value),
            GcTune::StepMul => std::mem::replace(&mut gc.stepmul, value),
            GcTune::StepSize => {
                let old = gc.stepsize / 1024;
                gc.stepsize = value.saturating_mul(1 << 10);
                old
            }
        }
    }

    pub(crate) fn gc_running(&self) -> bool {
        self.heap.gc.threshold != usize::MAX
    }

    /// The root set (module comment). Clears the stack above every frame's
    /// top first, so it holds nothing stale.
    pub(crate) fn gc_roots(&mut self) -> Vec<Value> {
        let frames_high = self
            .frames
            .iter()
            .map(|f| f.top)
            .fold(self.top, usize::max)
            .min(self.stack.len());
        let stack_len = self.stack.len();
        self.fill_nil(frames_high, stack_len);
        let mut roots: Vec<Value> = Vec::with_capacity(frames_high.saturating_add(64));
        self.push_thread_roots(self.live_stack_top(), &mut roots);
        roots.push(Value::Table(self.globals));
        roots.push(Value::Table(self.registry));
        if let Some(mt) = self.string_meta {
            roots.push(Value::Table(mt));
        }
        roots.push(Value::String(StrId(0)));
        roots.extend(self.tm_names().iter().map(|&s| Value::String(s)));
        roots.extend(self.pinned.iter().copied());
        roots.extend(self.host_errors.iter().copied());
        if let Some(name) = self.namecall {
            roots.push(Value::String(name));
        }
        roots
    }

    /// Where the live part of the stack ends (C++ `traversestack` marks
    /// `stack .. L->top`). In a Lua frame that is the frame's top: every
    /// register, as C++ keeps `L->top` at `ci->top` while Lua code runs.
    /// In a native frame it is the current top: the native function's
    /// arguments and pushes. Registers of the Lua frame below it that lie
    /// above the native call are dead (the compiler places a call above
    /// every live register), so they are not marked, which is what lets
    /// gc.luau's weak entries held only by a finished loop's register go.
    fn live_stack_top(&self) -> usize {
        let top = match self.frames.last() {
            Some(frame) if frame.proto.is_some() => self.top.max(frame.top),
            _ => self.top,
        };
        top.min(self.stack.len())
    }

    /// One thread's roots: its stack up to `high`, each frame's closure and
    /// its open upvalues.
    fn push_thread_roots(&self, high: usize, roots: &mut Vec<Value>) {
        roots.extend(self.stack.iter().take(high).copied());
        roots.extend(self.frames.iter().map(|f| Value::Function(f.closure)));
        roots.extend(self.open_upvals.iter().map(|&(_, id)| Value::UpRef(id)));
    }

    /// `Budget::used` minus the cost recounted from the arenas: zero
    /// unless something allocated around the budget (budget.rs). Tooling
    /// for the differential harness, not API.
    #[doc(hidden)]
    pub fn accounting_drift(&self) -> i128 {
        (self.heap.total_bytes() as i128).saturating_sub(self.heap.recount() as i128)
    }
}
