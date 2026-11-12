//! The interpreter loop (C++ `luau_execute`, `lvmexecute.cpp`).
//!
//! One `match` per instruction over verified code. Each opcode follows the
//! C++ case; the comments name the deviations. `FASTCALL*` run the
//! builtin's fast body under `safeenv` and skip the `CALL` that follows
//! (`fastcall.rs`), as upstream does.
//!
//! `self.exec.pc` is kept equal to the running frame's `savedpc` (the index
//! after the current instruction word), so an error raised anywhere below
//! can name the line, and a frame pushed above can record where to resume.
//!
//! **Calls do not recurse** (`call.rs`). `CALL` pushes the callee's frame
//! and re-dispatches; `RETURN` pops and hands the results back
//! (`Core::deliver`). An instruction that needs a metamethod pushes the
//! metamethod's call with a [`Finish`] on its own frame and re-dispatches;
//! when the call returns, [`Core::finish_op`] completes the instruction and
//! the frame continues at the next one (C++ `luau_finishop`, which upstream
//! needs only after a yield).


use crate::bytecode::{insn_a, insn_b, insn_c, insn_d, insn_e, insn_op, op};
use crate::call::{CallMode, Exit, Flow, Wait};
use crate::hosted::Dispatch;
use crate::error::VmResult;
use crate::heap::{Function, LuaClosure};
use crate::meta::{Arith, ConcatStep, Meta, MetaCall, Post};
use crate::native::NativeFn;
use crate::value::{FnId, LU_TAG_ITERATOR, TableId, UdId, Value};
use crate::heap::Upval;
use crate::vm::{Core, Frame, MIN_STACK, WINDOW};

/// The rest of an instruction that waits on a metamethod call: where its
/// frame continues and what it does with the result.
pub(crate) struct Finish {
    pub(crate) kind: FinishKind,
    /// Applied to the call's first result first.
    pub(crate) post: Post,
    /// The instruction after this one.
    pub(crate) pc: usize,
}

pub(crate) enum FinishKind {
    /// Store the result in register `a`.
    Store(u8),
    /// `__newindex`: no result.
    Discard,
    /// A conditional jump: to `target` when the result's truth is `want`.
    Branch { want: bool, target: usize },
    /// `CONCAT`: the result goes to `slot`, then the rest of the operands.
    Concat {
        a: u8,
        b: u8,
        slot: usize,
        total: usize,
        last: usize,
    },
    /// `NAMECALL` through `__index`: the method goes to `a`; nil is an error
    /// naming the constant `key`.
    NameCall { a: u8, key: u32 },
    /// `GETIMPORT`: `done` of the path's names are resolved.
    Import { a: u8, aux: u32, done: u32 },
    /// `FORGPREP` after `__iter`.
    IterPrep { a: u8 },
    /// `FORGLOOP` after the iterator call: to `target` unless it ended.
    ForGLoop { a: u8, target: usize },
}

/// The constant index of `GETIMPORT`'s `i`th name.
fn import_key(aux: u32, i: u32) -> u32 {
    match i {
        0 => (aux >> 20) & 1023,
        1 => (aux >> 10) & 1023,
        _ => aux & 1023,
    }
}

const CAPTURE_VAL: u8 = 0;
const CAPTURE_REF: u8 = 1;

#[inline]
fn jump(pc: usize, offset: i32) -> usize {
    pc.wrapping_add_signed(offset as isize)
}

#[inline]
fn number(v: Value) -> f64 {
    match v {
        Value::Number(n) => n,
        _ => 0.0,
    }
}

impl Core {
    /// The `GETTABLEKS` vector fast path (C++ `lvmexecute.cpp`): a
    /// one-letter key `x`, `y` or `z`, either case, is that component.
    #[inline]
    pub(crate) fn vector_component(&self, v: [f32; 3], key: Value) -> Option<f64> {
        let Value::String(key) = key else {
            return None;
        };
        let index = match self.heap.str_bytes(key) {
            [c] => usize::from((c | b' ').checked_sub(b'x')?),
            _ => return None,
        };
        v.get(index).map(|&c| f64::from(c))
    }

    /// The `GETTABLEKS` direct-field read: `Some` when `ud`'s tag has
    /// direct fields and the host has one named `key` for it.
    #[inline]
    pub(crate) fn direct_field(
        &self,
        host: &mut dyn Dispatch,
        ud: UdId,
        key: Value,
    ) -> Option<Value> {
        if self.direct_field_tags == 0 {
            return None;
        }
        let Value::String(key) = key else {
            return None;
        };
        let data = self.heap.userdata(ud);
        let bit = 1u128.checked_shl(u32::from(data.tag))?;
        if self.direct_field_tags & bit == 0 {
            return None;
        }
        host.direct_field(data.tag, data.key, self.heap.str_bytes(key))
            .map(Value::Number)
    }

    /// Makes `GETTABLEKS` ask the host for direct fields of userdata with
    /// `tag` (below `UTAG_LIMIT`).
    pub(crate) fn mark_direct_field_tag(&mut self, tag: u8) {
        if let Some(bit) = 1u128.checked_shl(u32::from(tag)) {
            self.direct_field_tags |= bit;
        }
    }
}

impl Core {
    fn closure_env(&self, id: FnId) -> TableId {
        match self.heap.function(id) {
            Function::Lua(c) => c.env,
            Function::Native(_) => self.exec.globals,
        }
    }

    /// The raw upvalue slot `i` of a Lua closure (a value or `UpRef`).
    fn closure_upval_raw(&self, id: FnId, i: usize) -> Value {
        match self.heap.function(id) {
            Function::Lua(c) => c.upvals.get(i).copied().unwrap_or_default(),
            Function::Native(_) => Value::Nil,
        }
    }

    fn closure_upval_slot(&mut self, id: FnId, i: usize) -> Option<&mut Value> {
        match self.heap.function_mut(id) {
            Function::Lua(c) => c.upvals.get_mut(i),
            Function::Native(_) => None,
        }
    }

    /// "attempt to call missing method 'name' of type" (C++ `luaG_callerror`
    /// after a NAMECALL).
    fn missing_method(&mut self, receiver: Value, key: Value) -> crate::error::Throw {
        let Value::String(name) = key else {
            return self.runtime_error("invalid method name");
        };
        let tn = self.obj_type_name(receiver);
        let name = crate::vm::c_lossy(self.heap.str_bytes(name));
        self.runtime_error(format!("attempt to call missing method '{name}' of {tn}"))
    }

    /// Pushes a metamethod call for an instruction of the running Lua frame
    /// and leaves `kind` to finish it.
    pub(crate) fn push_meta(
        &mut self,
        call: MetaCall,
        kind: FinishKind,
        pc: usize,
    ) -> VmResult<()> {
        let at = self.scratch();
        let nargs = call.nargs.min(call.args.len());
        self.ensure_stack(
            at.saturating_add(1)
                .saturating_add(nargs)
                .saturating_add(MIN_STACK),
        )?;
        self.set(at, call.f);
        for (i, &v) in call.args.iter().take(nargs).enumerate() {
            self.set(at.saturating_add(1).saturating_add(i), v);
        }
        self.exec.top = at.saturating_add(1).saturating_add(nargs);
        self.wait_then_call(
            Finish {
                kind,
                post: call.post,
                pc,
            },
            at,
            nargs,
            call.nresults,
            CallMode::Counted,
        )
    }

    /// Leaves `finish` on the running Lua frame (its pending entry, pushed
    /// before the callee's own) and pushes the call it waits on. If the
    /// push fails the error unwinds this frame, entry and all.
    fn wait_then_call(
        &mut self,
        finish: Finish,
        func: usize,
        nargs: usize,
        nresults: i32,
        mode: CallMode,
    ) -> VmResult<()> {
        if let Some(pending) = self.exec.frames.top_pending_or_insert() {
            pending.wait = Wait::Op(finish);
        }
        self.push_call(func, nargs, nresults, mode)
    }

    /// `GETIMPORT`'s path from name `from` on, starting at `v`: the value,
    /// or the metamethod call that is needed and the names done after it.
    fn import_steps(
        &mut self,
        k: &[Value],
        aux: u32,
        from: u32,
        mut v: Value,
    ) -> VmResult<Result<Value, (MetaCall, u32)>> {
        let count = aux >> 30;
        for i in from..count {
            let key = k
                .get(import_key(aux, i) as usize)
                .copied()
                .unwrap_or_default();
            match self.get_table_step(v, key)? {
                Meta::Done(next) => v = next,
                Meta::Call(call) => return Ok(Err((call, i.saturating_add(1)))),
            }
        }
        Ok(Ok(v))
    }

    /// Completes the instruction of the Lua frame on top, whose metamethod
    /// call returned its first result to `func` (C++ `luau_finishop`).
    pub(crate) fn finish_op(&mut self, finish: Finish, func: usize) -> VmResult<Flow> {
        let result = self.get(func);
        let Some(frame) = self.exec.frames.last() else {
            return Ok(Flow::Exit(Exit::Returned));
        };
        let (base, ftop, savedpc) = (frame.base(), frame.top(), frame.savedpc());
        let proto_id = frame.proto();
        let protos = std::sync::Arc::clone(self.heap.proto_table());
        let proto = proto_id.map(|id| protos.get(id));
        self.exec.top = ftop;
        // Errors below name the waiting instruction.
        self.exec.pc = savedpc;
        let v = self.apply_post(finish.post, result)?;
        let reg = |r: u8| base.saturating_add(usize::from(r));
        let mut next = finish.pc;
        match finish.kind {
            FinishKind::Store(a) => self.set(reg(a), v),
            FinishKind::Discard => {}
            FinishKind::Branch { want, target } => {
                if v.truthy() == want {
                    next = target;
                }
            }
            FinishKind::Concat {
                a,
                b,
                slot,
                total,
                last,
            } => {
                self.set(slot, v);
                match self.concat_step(total, last)? {
                    ConcatStep::Done => {
                        self.set(reg(a), self.get(reg(b)));
                        self.safepoint();
                    }
                    ConcatStep::Call {
                        call,
                        slot,
                        total,
                        last,
                    } => {
                        let kind = FinishKind::Concat {
                            a,
                            b,
                            slot,
                            total,
                            last,
                        };
                        self.push_meta(call, kind, finish.pc)?;
                        return Ok(Flow::Continue);
                    }
                }
            }
            FinishKind::NameCall { a, key } => {
                self.set(reg(a), v);
                if v.is_nil() {
                    let key = proto
                        .and_then(|p| p.k.get(key as usize).copied())
                        .unwrap_or_default();
                    let receiver = self.get(reg(a).saturating_add(1));
                    return Err(self.missing_method(receiver, key));
                }
            }
            FinishKind::Import { a, aux, done } => {
                let empty: &[Value] = &[];
                let k = proto.map_or(empty, |p| &p.k[..]);
                match self.import_steps(k, aux, done, v)? {
                    Ok(v) => self.set(reg(a), v),
                    Err((call, done)) => {
                        self.push_meta(call, FinishKind::Import { a, aux, done }, finish.pc)?;
                        return Ok(Flow::Continue);
                    }
                }
            }
            FinishKind::IterPrep { a } => {
                if self.get(reg(a)).is_nil() {
                    return Err(self.type_error(Value::Nil, "call"));
                }
            }
            FinishKind::ForGLoop { a, target } => {
                let first = self.get(reg(a).saturating_add(3));
                self.set(reg(a).saturating_add(2), first);
                if !first.is_nil() {
                    next = target;
                }
            }
        }
        if let Some(frame) = self.exec.frames.last_mut() {
            frame.set_savedpc(next);
        }
        Ok(Flow::Continue)
    }

    /// Register `a` of the running Lua frame (the frame on top), for a
    /// general tail outside the dispatch loop's locals.
    #[inline]
    fn set_top_reg(&mut self, a: u8, v: Value) {
        let base = self.exec.frames.last().map_or(0, Frame::base);
        self.set(base.saturating_add(usize::from(a)), v);
    }

    /// The rest of an instruction storing to register `a` after its
    /// metamethod step: true when it stored (the frame continues in
    /// place), false when it pushed the metamethod's call.
    #[cold]
    #[inline(never)]
    fn store_tail(&mut self, step: Meta, a: u8, pc: usize) -> VmResult<bool> {
        match step {
            Meta::Done(v) => {
                self.set_top_reg(a, v);
                Ok(true)
            }
            Meta::Call(call) => {
                self.push_meta(call, FinishKind::Store(a), pc)?;
                Ok(false)
            }
        }
    }

    /// An arithmetic register arm (`insn`, with constants `k`) whose
    /// operands are not two numbers: vectors, string coercion or the
    /// metamethod (`store_tail`). The operands are read again here from
    /// the instruction, so the arm's fast path keeps them in registers
    /// rather than in memory for this call.
    #[cold]
    #[inline(never)]
    fn arith_tail(&mut self, insn: u32, k: &[Value], pc: usize) -> VmResult<bool> {
        let base = self.exec.frames.last().map_or(0, Frame::base);
        let (a, b, c) = (insn_a(insn), insn_b(insn), insn_c(insn));
        let reg = |vm: &Core, r: u8| vm.get(base.saturating_add(usize::from(r)));
        let kv = |i: u8| k.get(usize::from(i)).copied().unwrap_or_default();
        let (op, x, y) = match insn_op(insn) {
            op::ADD => (Arith::Add, reg(self, b), reg(self, c)),
            op::SUB => (Arith::Sub, reg(self, b), reg(self, c)),
            op::MUL => (Arith::Mul, reg(self, b), reg(self, c)),
            op::DIV => (Arith::Div, reg(self, b), reg(self, c)),
            op::MOD => (Arith::Mod, reg(self, b), reg(self, c)),
            op::POW => (Arith::Pow, reg(self, b), reg(self, c)),
            op::IDIV => (Arith::Idiv, reg(self, b), reg(self, c)),
            op::ADDK => (Arith::Add, reg(self, b), kv(c)),
            op::SUBK => (Arith::Sub, reg(self, b), kv(c)),
            op::MULK => (Arith::Mul, reg(self, b), kv(c)),
            op::DIVK => (Arith::Div, reg(self, b), kv(c)),
            op::MODK => (Arith::Mod, reg(self, b), kv(c)),
            op::POWK => (Arith::Pow, reg(self, b), kv(c)),
            op::IDIVK => (Arith::Idiv, reg(self, b), kv(c)),
            op::SUBRK => (Arith::Sub, kv(b), reg(self, c)),
            op::DIVRK => (Arith::Div, kv(b), reg(self, c)),
            op::MINUS => (Arith::Unm, reg(self, b), reg(self, b)),
            _ => return Err(self.runtime_error("invalid opcode")),
        };
        let step = self.arith_step(op, x, y)?;
        self.store_tail(step, a, pc)
    }

    /// A conditional jump (`insn`, its `aux` word, `pc` past the
    /// instruction word) the register arm could not decide: the pc to go
    /// on from, or `None` when it pushed the metamethod, whose result
    /// branches.
    #[cold]
    #[inline(never)]
    fn compare_tail(&mut self, insn: u32, aux: u32, pc: usize) -> VmResult<Option<usize>> {
        let base = self.exec.frames.last().map_or(0, Frame::base);
        let ra = self.get(base.saturating_add(usize::from(insn_a(insn))));
        let rb = self.get(base.saturating_add(aux as usize));
        let o = insn_op(insn);
        let want = matches!(o, op::JUMPIFEQ | op::JUMPIFLE | op::JUMPIFLT);
        let target = jump(pc, insn_d(insn));
        let next = pc.saturating_add(1);
        let branch = |result: bool| if result == want { target } else { next };
        let step = match o {
            op::JUMPIFEQ | op::JUMPIFNOTEQ => {
                if !ra.same_type(rb) {
                    return Ok(Some(branch(false)));
                }
                self.equal_step(ra, rb)
            }
            op::JUMPIFLE | op::JUMPIFNOTLE => self.less_equal_step(ra, rb)?,
            _ => self.less_than_step(ra, rb)?,
        };
        match step {
            Meta::Done(v) => Ok(Some(branch(v.truthy()))),
            Meta::Call(call) => {
                self.push_meta(call, FinishKind::Branch { want, target }, next)?;
                Ok(None)
            }
        }
    }

    /// `FORNPREP` with an operand that is not a number: each is converted
    /// from a string or the loop raises.
    #[cold]
    #[inline(never)]
    fn fornprep_tail(&mut self, a: u8) -> VmResult<()> {
        let base = self.exec.frames.last().map_or(0, Frame::base);
        for (offset, what) in [(0usize, "limit"), (1, "step"), (2, "initial value")] {
            let slot = base.saturating_add(usize::from(a)).saturating_add(offset);
            let v = self.get(slot);
            if matches!(v, Value::Number(_)) {
                continue;
            }
            match self.to_number(v) {
                Some(n) => self.set(slot, Value::Number(n)),
                None => {
                    let tn = self.obj_type_name(v);
                    return Err(self.runtime_error(format!(
                        "invalid 'for' {what} (number expected, got {tn})"
                    )));
                }
            }
        }
        Ok(())
    }

    /// Runs the frame stack until `execute`'s frame returns, the thread
    /// finishes, or the main thread yields to the host.
    pub(crate) fn run_frames(&mut self, host: &mut dyn Dispatch) -> VmResult<Exit> {
        // The prototypes, held for the loop (heap.rs, "Prototypes"). Nothing
        // the loop runs adds one, but a newer table is picked up anyway.
        let mut protos = std::sync::Arc::clone(self.heap.proto_table());
        'frame: loop {
            let Some(frame) = self.exec.frames.last() else {
                return Ok(Exit::Returned);
            };
            let Some(proto_id) = frame.proto() else {
                match self.step_native(host)? {
                    Flow::Exit(exit) => return Ok(exit),
                    Flow::Inline | Flow::Continue => continue 'frame,
                }
            };
            if !std::sync::Arc::ptr_eq(&protos, self.heap.proto_table()) {
                protos = std::sync::Arc::clone(self.heap.proto_table());
            }
            // The running frame's locals. `CALL` of a Lua function and
            // `RETURN` to a plain Lua caller switch them in place
            // (docs/luau-safe-vm-call-path.md, 4.3) without leaving the
            // dispatch loop; everything else re-enters this loop. The loop
            // carries the window, the code, the constants, `pc` and the
            // closure (docs/luau-safe-vm-register-window.md, 4.2); what a
            // general arm needs of the frame besides (`func`, `top`, the
            // prototype) it reads from the frame on top, where it lives.
            // `base` is set at the top of every general arm (`general!`).
            let proto = protos.get(proto_id);
            let mut closure = frame.closure;
            let mut base = frame.base();
            let mut pc = frame.savedpc();
            let mut code: &[u32] = &proto.code;
            let mut k: &[Value] = &proto.k;

            // Switches the locals to frame `$frame` running `$proto` from
            // `$pc`.
            macro_rules! switch_to {
                ($frame:expr, $proto:expr, $pc:expr) => {{
                    let frame: Frame = $frame;
                    let proto = $proto;
                    closure = frame.closure;
                    base = frame.base();
                    pc = $pc;
                    code = &proto.code;
                    k = &proto.k;
                }};
            }
            // The running frame (the frame on top) as it is stored.
            macro_rules! top_frame {
                () => {
                    self.exec.frames.last().copied().unwrap_or_default()
                };
            }

            macro_rules! reg {
                ($i:expr) => {
                    self.get(base.saturating_add($i as usize))
                };
            }
            macro_rules! set_reg {
                ($i:expr, $v:expr) => {{
                    let v = $v;
                    self.set(base.saturating_add($i as usize), v);
                }};
            }
            macro_rules! kst {
                ($i:expr) => {
                    k.get($i as usize).copied().unwrap_or_default()
                };
            }
            macro_rules! aux {
                () => {
                    code.get(pc).copied().unwrap_or(0)
                };
            }
            // A `FASTCALL*` form: builtin `$id`, the `CALL` at `$call_at`,
            // and the arguments with their count (`$args`, given the
            // `CALL`'s result register and word).
            macro_rules! fastcall {
                ($id:expr, $call_at:expr, |$ra:ident, $call:ident| $args:expr) => {{
                    if let Some(fast) = crate::fastcall::entry($id) {
                        // The running closure's environment's flag, read
                        // where it lives each time (no cached copy).
                        let fast_safe = self.heap.table(self.closure_env(closure)).safeenv;
                        let call_at: usize = $call_at;
                        let $call = code.get(call_at).copied().unwrap_or(0);
                        if fast_safe && insn_op($call) == op::CALL {
                            let $ra = base.saturating_add(usize::from(insn_a($call)));
                            let nresults = i32::from(insn_c($call)).saturating_sub(1);
                            let (args, nparams): ([Value; 3], usize) = $args;
                            if let Some(n) = (fast.f)(self, $ra, nresults, args, nparams) {
                                self.exec.top = if nresults < 0 {
                                    $ra.saturating_add(n)
                                } else {
                                    top_frame!().top()
                                };
                                pc = call_at.saturating_add(1);
                                if fast.allocates {
                                    self.safepoint();
                                }
                            }
                        }
                    }
                }};
            }
            // A metamethod step's value, or push its call and re-dispatch.
            macro_rules! meta {
                ($step:expr, $kind:expr, $next:expr) => {
                    match $step {
                        Meta::Done(v) => v,
                        Meta::Call(call) => {
                            self.push_meta(call, $kind, $next)?;
                            continue 'frame;
                        }
                    }
                };
            }
            // The running frame's register window
            // (docs/luau-safe-vm-register-window.md, 4.1): a borrow of the
            // 256 stack slots from `base`, which every Lua frame has
            // (`Core::ensure_frame`). Taken here, after `switch_to!`, and at
            // the end of every general arm; the compiler rejects a use after
            // a `&mut self` call that did not take it again.
            macro_rules! window {
                () => {
                    match self
                        .exec
                        .stack
                        .get_mut(base..)
                        .and_then(|s| s.first_chunk_mut::<WINDOW>())
                    {
                        Some(w) => w,
                        None => return Err(self.window_error()),
                    }
                };
            }
            let mut regs = window!();
            // Register `$i` (a `u8` operand) of the window: in bounds by
            // type, so no check.
            // (`get` on a `u8` index into 256 slots: the compiler drops the
            // check, and no index expression appears for the lints.)
            macro_rules! r {
                ($i:expr) => {
                    regs.get(usize::from($i)).copied().unwrap_or_default()
                };
            }
            macro_rules! set_r {
                ($i:expr, $v:expr) => {{
                    let v = $v;
                    if let Some(slot) = regs.get_mut(usize::from($i)) {
                        *slot = v;
                    }
                }};
            }
            // `self.exec.pc` is written by general arms only, first thing
            // (4.2): a register arm cannot raise, call, allocate or be
            // observed, so nothing reads it while one runs.
            macro_rules! sync_pc {
                () => {{
                    self.exec.pc = pc;
                    base = top_frame!().base();
                }};
            }
            // An arithmetic register arm: numbers in the window, anything
            // else a general tail (`Core::arith_tail`).
            macro_rules! arith {
                ($insn:ident, $a:ident, $op:expr, $x:expr, $y:expr) => {{
                    if let (Value::Number(p), Value::Number(q)) = ($x, $y) {
                        set_r!($a, Value::Number($op.number(p, q)));
                        continue;
                    }
                    sync_pc!();
                    if !self.arith_tail($insn, k, pc)? {
                        continue 'frame;
                    }
                }};
            }
            // A conditional jump whose operands are not both numbers (or
            // raw-comparable): the general tail (`Core::compare_tail`).
            macro_rules! compare_tail {
                ($insn:ident, $aux:expr) => {{
                    sync_pc!();
                    match self.compare_tail($insn, $aux, pc)? {
                        Some(next) => pc = next,
                        None => continue 'frame,
                    }
                }};
            }

            loop {
                let Some(&insn) = code.get(pc) else {
                    self.exec.pc = pc;
                    return Err(self.runtime_error("execution ran past the end of the function"));
                };
                pc = pc.saturating_add(1);
                let a = insn_a(insn);
                let b = insn_b(insn);
                let c = insn_c(insn);
                // Register arms end in `continue`; general arms start with
                // `sync_pc!()` and fall through to take the window again.
                match insn_op(insn) {
                    op::NOP | op::COVERAGE | op::PREPVARARGS | op::CAPTURE => continue,
                    // Under safeenv the builtin's fast body runs here and
                    // the CALL is skipped (`fastcall.rs`); a body that does
                    // not apply falls through to the CALL. C counts from the
                    // instruction after FASTCALL, which for the forms with an
                    // aux word is the aux word itself (upstream: `skip = C -
                    // 1` after `pc++` past aux).
                    op::FASTCALL => {
                        sync_pc!();
                        fastcall!(a, pc.saturating_add(usize::from(c)), |ra, call| {
                            let nparams = match insn_b(call) {
                                0 => self.exec.top.saturating_sub(ra.saturating_add(1)),
                                nb => usize::from(nb).saturating_sub(1),
                            };
                            let arg = |i: usize| self.get(ra.saturating_add(i));
                            ([arg(1), arg(2), arg(3)], nparams)
                        })
                    }
                    // The other forms read their arguments from the window,
                    // then run the body as a general arm.
                    op::FASTCALL1 => {
                        let args = ([r!(b), Value::Nil, Value::Nil], 1);
                        sync_pc!();
                        fastcall!(a, pc.saturating_add(usize::from(c)), |_ra, _call| args)
                    }
                    op::FASTCALL2 => {
                        let aux = aux!();
                        let args = ([r!(b), r!(aux as u8), Value::Nil], 2);
                        sync_pc!();
                        pc = pc.saturating_add(1);
                        fastcall!(a, pc.saturating_add(usize::from(c)).saturating_sub(1), |_ra, _call| args)
                    }
                    op::FASTCALL2K => {
                        let aux = aux!();
                        let args = ([r!(b), kst!(aux), Value::Nil], 2);
                        sync_pc!();
                        pc = pc.saturating_add(1);
                        fastcall!(a, pc.saturating_add(usize::from(c)).saturating_sub(1), |_ra, _call| args)
                    }
                    op::FASTCALL3 => {
                        let aux = aux!();
                        let args = ([r!(b), r!(aux as u8), r!((aux >> 8) as u8)], 3);
                        sync_pc!();
                        pc = pc.saturating_add(1);
                        fastcall!(a, pc.saturating_add(usize::from(c)).saturating_sub(1), |_ra, _call| args)
                    }
                    op::LOADNIL => {
                        set_r!(a, Value::Nil);
                        continue;
                    }
                    op::LOADB => {
                        set_r!(a, Value::Boolean(b != 0));
                        pc = pc.saturating_add(usize::from(c));
                        continue;
                    }
                    op::LOADN => {
                        set_r!(a, Value::Number(f64::from(insn_d(insn))));
                        continue;
                    }
                    op::LOADK => {
                        set_r!(a, kst!(insn_d(insn)));
                        continue;
                    }
                    op::LOADKX => {
                        let aux = aux!();
                        pc = pc.saturating_add(1);
                        set_r!(a, kst!(aux));
                        continue;
                    }
                    op::MOVE => {
                        set_r!(a, r!(b));
                        continue;
                    }
                    op::GETGLOBAL => {
                        sync_pc!();
                        let key = kst!(aux!());
                        pc = pc.saturating_add(1);
                        let env = self.closure_env(closure);
                        let v = self.heap.table(env).get(key);
                        let v = if v.is_nil() {
                            meta!(
                                self.get_table_step(Value::Table(env), key)?,
                                FinishKind::Store(a),
                                pc
                            )
                        } else {
                            v
                        };
                        set_reg!(a, v);
                    }
                    op::SETGLOBAL => {
                        sync_pc!();
                        let key = kst!(aux!());
                        pc = pc.saturating_add(1);
                        let env = self.closure_env(closure);
                        meta!(
                            self.set_table_step(Value::Table(env), key, reg!(a))?,
                            FinishKind::Discard,
                            pc
                        );
                    }
                    // Upvalues (4.5): a value capture and a closed
                    // reference are read and written beside the window, by
                    // heap field; an open reference names an absolute stack
                    // slot, so it is general.
                    op::GETUPVAL => {
                        let raw = match self.heap.function(closure) {
                            Function::Lua(c) => c.upvals.get(usize::from(b)).copied().unwrap_or_default(),
                            Function::Native(_) => Value::Nil,
                        };
                        let v = match raw {
                            Value::UpRef(id) => match self.heap.upval(id) {
                                Upval::Closed(v) => v,
                                Upval::Open(..) => {
                                    sync_pc!();
                                    let v = self.upval_get(id);
                                    regs = window!();
                                    set_r!(a, v);
                                    continue;
                                }
                            },
                            v => v,
                        };
                        set_r!(a, v);
                        continue;
                    }
                    op::SETUPVAL => {
                        let v = r!(a);
                        let raw = match self.heap.function(closure) {
                            Function::Lua(c) => c.upvals.get(usize::from(b)).copied().unwrap_or_default(),
                            Function::Native(_) => Value::Nil,
                        };
                        match raw {
                            Value::UpRef(id) => match self.heap.upval(id) {
                                Upval::Closed(_) => self.heap.set_upval(id, Upval::Closed(v)),
                                Upval::Open(..) => {
                                    sync_pc!();
                                    self.upval_set(id, v);
                                    regs = window!();
                                }
                            },
                            _ => {
                                if let Function::Lua(c) = self.heap.function_mut(closure)
                                    && let Some(slot) = c.upvals.get_mut(usize::from(b))
                                {
                                    *slot = v;
                                }
                            }
                        }
                        continue;
                    }
                    op::CLOSEUPVALS => {
                        sync_pc!();
                        self.close_upvals(base.saturating_add(usize::from(a)));
                    }
                    op::GETIMPORT => {
                        sync_pc!();
                        let kv = kst!(insn_d(insn));
                        let aux = aux!();
                        pc = pc.saturating_add(1);
                        let env = self.closure_env(closure);
                        if !kv.is_nil() && self.heap.table(env).safeenv {
                            set_reg!(a, kv);
                        } else {
                            // C++ `luaV_getimport` with `propagatenil` false.
                            match self.import_steps(k, aux, 0, Value::Table(env))? {
                                Ok(v) => set_reg!(a, v),
                                Err((call, done)) => {
                                    self.push_meta(call, FinishKind::Import { a, aux, done }, pc)?;
                                    continue 'frame;
                                }
                            }
                        }
                    }
                    // Tables beside the window (4.4): a raw read that finds a
                    // value, or a store to a key that holds one, by heap
                    // field. Nil, another receiver, a read-only table and a
                    // new key take the general arm.
                    op::GETTABLE => {
                        if let Value::Table(t) = r!(b) {
                            let v = self.heap.table(t).get(r!(c));
                            if !v.is_nil() {
                                set_r!(a, v);
                                continue;
                            }
                        }
                        sync_pc!();
                        let v = meta!(
                            self.get_table_step(reg!(b), reg!(c))?,
                            FinishKind::Store(a),
                            pc
                        );
                        set_reg!(a, v);
                    }
                    op::SETTABLE => {
                        if let Value::Table(t) = r!(b) {
                            let v = r!(a);
                            if let Some(slot) = self.heap.table_slot_mut(t, r!(c)) {
                                *slot = v;
                                continue;
                            }
                        }
                        sync_pc!();
                        meta!(
                            self.set_table_step(reg!(b), reg!(c), reg!(a))?,
                            FinishKind::Discard,
                            pc
                        );
                    }
                    op::GETTABLEKS => {
                        if let Value::Table(t) = r!(b)
                            && let Value::String(key) = kst!(aux!())
                        {
                            let v = self.heap.table(t).get_str(key);
                            if !v.is_nil() {
                                set_r!(a, v);
                                pc = pc.saturating_add(1);
                                continue;
                            }
                        }
                        sync_pc!();
                        let key = kst!(aux!());
                        pc = pc.saturating_add(1);
                        let object = reg!(b);
                        // Lane 6: a userdata tag's direct-field getter
                        // (`LuauDirectFieldGet`), before `__index`.
                        if let Value::Userdata(ud) = object
                            && let Some(v) = self.direct_field(host, ud, key)
                        {
                            set_reg!(a, v);
                        } else if let Value::Vector(v) = object
                            && let Some(x) = self.vector_component(v, key)
                        {
                            // C++ GETTABLEKS reads x/y/z (either case)
                            // before any metatable, so a host vector
                            // metatable cannot hide them.
                            set_reg!(a, Value::Number(x));
                        } else {
                            let v = meta!(self.get_table_step(object, key)?, FinishKind::Store(a), pc);
                            set_reg!(a, v);
                        }
                    }
                    op::SETTABLEKS => {
                        if let Value::Table(t) = r!(b)
                            && let Value::String(key) = kst!(aux!())
                        {
                            let v = r!(a);
                            if let Some(slot) = self.heap.table_str_slot_mut(t, key) {
                                *slot = v;
                                pc = pc.saturating_add(1);
                                continue;
                            }
                        }
                        sync_pc!();
                        let key = kst!(aux!());
                        pc = pc.saturating_add(1);
                        meta!(
                            self.set_table_step(reg!(b), key, reg!(a))?,
                            FinishKind::Discard,
                            pc
                        );
                    }
                    op::GETTABLEN => {
                        if let Value::Table(t) = r!(b) {
                            let v = self.heap.table(t).get_int(i64::from(c).saturating_add(1));
                            if !v.is_nil() {
                                set_r!(a, v);
                                continue;
                            }
                        }
                        sync_pc!();
                        let key = Value::Number(f64::from(c) + 1.0);
                        let v = meta!(self.get_table_step(reg!(b), key)?, FinishKind::Store(a), pc);
                        set_reg!(a, v);
                    }
                    op::SETTABLEN => {
                        if let Value::Table(t) = r!(b) {
                            let v = r!(a);
                            let key = Value::Number(f64::from(c) + 1.0);
                            if let Some(slot) = self.heap.table_slot_mut(t, key) {
                                *slot = v;
                                continue;
                            }
                        }
                        sync_pc!();
                        let key = Value::Number(f64::from(c) + 1.0);
                        meta!(
                            self.set_table_step(reg!(b), key, reg!(a))?,
                            FinishKind::Discard,
                            pc
                        );
                    }
                    op::NEWCLOSURE => {
                        sync_pc!();
                        let Some(&child_id) = usize::try_from(insn_d(insn))
                            .ok()
                            .and_then(|i| protos.get(top_frame!().lua_proto()).children.get(i))
                        else {
                            return Err(self.runtime_error("invalid child prototype"));
                        };
                        let nups = usize::from(protos.get(child_id).nups);
                        let env = self.closure_env(closure);
                        let closure_value = LuaClosure {
                            proto_id: child_id,
                            env,
                            upvals: vec![Value::Nil; nups],
                            preload: false,
                        };
                        let f = self.heap.new_function(Function::Lua(closure_value))?;
                        // As upstream: the closure is in its register before the
                        // captures run, so a local function that captures itself
                        // by value sees itself.
                        set_reg!(a, Value::Function(f));
                        for ui in 0..nups {
                            let capture = aux!();
                            pc = pc.saturating_add(1);
                            let ub = insn_b(capture);
                            let uv = match insn_a(capture) {
                                CAPTURE_VAL => reg!(ub),
                                CAPTURE_REF => Value::UpRef(
                                    self.find_upval(base.saturating_add(usize::from(ub)))?,
                                ),
                                _ => self.closure_upval_raw(closure, usize::from(ub)),
                            };
                            if let Some(slot) = self.closure_upval_slot(f, ui) {
                                *slot = uv;
                            }
                        }
                        self.safepoint();
                    }
                    op::DUPCLOSURE => {
                        sync_pc!();
                        let Value::Function(kid) = kst!(insn_d(insn)) else {
                            return Err(self.runtime_error("invalid closure constant"));
                        };
                        let (kproto_id, kenv, nups) = match self.heap.function(kid) {
                            Function::Lua(kc) => (kc.proto_id, kc.env, kc.upvals.len()),
                            Function::Native(_) => {
                                return Err(self.runtime_error("invalid closure constant"));
                            }
                        };
                        let env = self.closure_env(closure);
                        let fresh = |vm: &mut Core| {
                            let c = LuaClosure {
                                proto_id: kproto_id,
                                env,
                                upvals: vec![Value::Nil; nups],
                                preload: false,
                            };
                            vm.heap.new_function(Function::Lua(c))
                        };
                        let mut ncl = if kenv == env { kid } else { fresh(self)? };
                        set_reg!(a, Value::Function(ncl));
                        let mut ui = 0usize;
                        while ui < nups {
                            let capture = code.get(pc.saturating_add(ui)).copied().unwrap_or(0);
                            let ub = insn_b(capture);
                            let uv = if insn_a(capture) == CAPTURE_VAL {
                                reg!(ub)
                            } else {
                                self.closure_upval_raw(closure, usize::from(ub))
                            };
                            if ncl == kid {
                                if self.closure_upval_raw(ncl, ui).raw_eq(uv) {
                                    ui = ui.saturating_add(1);
                                    continue;
                                }
                                let preload = matches!(self.heap.function(kid), Function::Lua(kc) if kc.preload);
                                if !preload {
                                    ncl = fresh(self)?;
                                    set_reg!(a, Value::Function(ncl));
                                    ui = 0;
                                    continue;
                                }
                            }
                            if let Some(slot) = self.closure_upval_slot(ncl, ui) {
                                *slot = uv;
                            }
                            ui = ui.saturating_add(1);
                        }
                        if let Function::Lua(c) = self.heap.function_mut(ncl) {
                            c.preload = false;
                        }
                        pc = pc.saturating_add(nups);
                        if ncl != kid {
                            self.safepoint();
                        }
                    }
                    op::NAMECALL => {
                        sync_pc!();
                        let kidx = aux!();
                        let key = kst!(kidx);
                        pc = pc.saturating_add(1);
                        let rb = reg!(b);
                        if let Value::Table(t) = rb {
                            let v = self.heap.table(t).get(key);
                            set_reg!(usize::from(a).saturating_add(1), rb);
                            let v = if v.is_nil() {
                                meta!(
                                    self.get_table_step(rb, key)?,
                                    FinishKind::NameCall { a, key: kidx },
                                    pc
                                )
                            } else {
                                v
                            };
                            set_reg!(a, v);
                            if v.is_nil() {
                                return Err(self.missing_method(rb, key));
                            }
                        } else if let Some(f) =
                            self.fast_tm(self.metatable(rb), crate::vm::Tm::NameCall)
                        {
                            set_reg!(usize::from(a).saturating_add(1), rb);
                            set_reg!(a, f);
                            if let Value::String(name) = key {
                                self.namecall = Some(name);
                            }
                        } else {
                            set_reg!(usize::from(a).saturating_add(1), rb);
                            let v = meta!(
                                self.get_table_step(rb, key)?,
                                FinishKind::NameCall { a, key: kidx },
                                pc
                            );
                            set_reg!(a, v);
                            // Upstream raises this when the CALL after a
                            // NAMECALL finds nil, for any receiver.
                            if v.is_nil() {
                                return Err(self.missing_method(rb, key));
                            }
                        }
                    }
                    op::CALL | op::CALLFB => {
                        sync_pc!();
                        self.interrupt(host)?;
                        if insn_op(insn) == op::CALLFB {
                            pc = pc.saturating_add(1);
                            self.exec.pc = pc;
                        }
                        let callee = base.saturating_add(usize::from(a));
                        let nresults = i32::from(c).saturating_sub(1);
                        let nargs = if b == 0 {
                            self.exec.top.saturating_sub(callee.saturating_add(1))
                        } else {
                            usize::from(b).saturating_sub(1)
                        };
                        self.exec.top = callee.saturating_add(1).saturating_add(nargs);
                        if let Value::Function(id) = self.get(callee) {
                            match self.heap.function(id) {
                                // A Lua callee: its frame is pushed and its
                                // first instruction dispatched here. A stale
                                // prototype table takes the general path.
                                Function::Lua(lua)
                                    if std::sync::Arc::ptr_eq(&protos, self.heap.proto_table()) =>
                                {
                                    let proto_id = lua.proto_id;
                                    let callee_proto = protos.get(proto_id);
                                    let frame = if callee_proto.is_vararg {
                                        self.enter_lua(callee, id, nargs, nresults)?;
                                        self.exec.frames.last().copied().unwrap_or_default()
                                    } else {
                                        self.enter_lua_fixed(
                                            callee,
                                            (id, proto_id),
                                            callee_proto,
                                            nargs,
                                            nresults,
                                        )?
                                    };
                                    switch_to!(frame, callee_proto, 0);
                                    regs = window!();
                                    continue;
                                }
                                Function::Native(native) => {
                                    if let func @ (NativeFn::Plain(_)
                                    | NativeFn::Interruptible(_)
                                    | NativeFn::HostPlain(..)) = native.func
                                    {
                                        if self.call_frameless(
                                            host,
                                            callee,
                                            (id, func),
                                            nargs,
                                            nresults,
                                            top_frame!().top(),
                                        )? {
                                            continue 'frame;
                                        }
                                        regs = window!();
                                        continue;
                                    }
                                }
                                Function::Lua(_) => {}
                            }
                        }
                        // `__call`, a framed native, a stale prototype table.
                        self.push_call(callee, nargs, nresults, CallMode::Plain)?;
                        if self.exec.frames.last().is_some_and(|f| f.has(Frame::LUA)) {
                            continue 'frame;
                        }
                        // A native callee runs now; one that returns plainly
                        // leaves this frame running (`deliver` reset `top`).
                        match self.step_native(host)? {
                            Flow::Inline => {}
                            Flow::Continue => continue 'frame,
                            Flow::Exit(exit) => return Ok(exit),
                        }
                    }
                    op::RETURN => {
                        sync_pc!();
                        self.interrupt(host)?;
                        let ra = base.saturating_add(usize::from(a));
                        let n = if b == 0 {
                            self.exec.top.saturating_sub(ra)
                        } else {
                            usize::from(b).saturating_sub(1)
                        };
                        // C++ relies on the compiler's CLOSEUPVALS here;
                        // closing again is a no-op for compiled code and
                        // keeps hand-written bytecode from leaving open
                        // upvalues on a dead frame.
                        self.close_upvals(base);
                        // Back to a plain Lua caller: results move down by
                        // register moves and the caller's locals reload here.
                        if std::sync::Arc::ptr_eq(&protos, self.heap.proto_table())
                            && let Some((done, caller)) = self.exec.frames.pop_plain()
                        {
                            self.return_plain(done, caller, ra, n);
                            switch_to!(caller, protos.get(caller.lua_proto()), caller.savedpc());
                            regs = window!();
                            continue;
                        }
                        let Some((done, pending)) = self.exec.frames.pop() else {
                            return Ok(Exit::Returned);
                        };
                        self.move_results(done.func(), ra, n, done.nresults())?;
                        match self.deliver(done, pending)? {
                            Flow::Exit(exit) => return Ok(exit),
                            Flow::Inline | Flow::Continue => continue 'frame,
                        }
                    }
                    op::JUMP => {
                        pc = jump(pc, insn_d(insn));
                        continue;
                    }
                    // The interrupt is read by field; only when the host
                    // has one does the arm become general (4.2).
                    op::JUMPBACK => {
                        if self.interrupt_on {
                            sync_pc!();
                            self.interrupt(host)?;
                            regs = window!();
                        }
                        pc = jump(pc, insn_d(insn));
                        continue;
                    }
                    op::JUMPX => {
                        if self.interrupt_on {
                            sync_pc!();
                            self.interrupt(host)?;
                            regs = window!();
                        }
                        pc = jump(pc, insn_e(insn));
                        continue;
                    }
                    op::JUMPIF => {
                        if r!(a).truthy() {
                            pc = jump(pc, insn_d(insn));
                        }
                        continue;
                    }
                    op::JUMPIFNOT => {
                        if !r!(a).truthy() {
                            pc = jump(pc, insn_d(insn));
                        }
                        continue;
                    }
                    // Equality of values `equal_step` compares raw (all but
                    // two tables or two userdata, which may have `__eq`).
                    o @ (op::JUMPIFEQ | op::JUMPIFNOTEQ) => {
                        let (ra, rb) = (r!(a), r!(aux!() as u8));
                        let want = o == op::JUMPIFEQ;
                        let result = match (ra, rb) {
                            (Value::Number(x), Value::Number(y)) => Some(x == y),
                            (Value::Table(_), Value::Table(_))
                            | (Value::Userdata(_), Value::Userdata(_)) => None,
                            (Value::Integer(x), Value::Integer(y)) => Some(x == y),
                            _ => Some(ra.same_type(rb) && ra.raw_eq(rb)),
                        };
                        if let Some(result) = result {
                            pc = if result == want {
                                jump(pc, insn_d(insn))
                            } else {
                                pc.saturating_add(1)
                            };
                            continue;
                        }
                        compare_tail!(insn, aux!());
                    }
                    o @ (op::JUMPIFLE | op::JUMPIFNOTLE) => {
                        let (ra, rb) = (r!(a), r!(aux!() as u8));
                        let want = o == op::JUMPIFLE;
                        if let (Value::Number(x), Value::Number(y)) = (ra, rb) {
                            pc = if (x <= y) == want {
                                jump(pc, insn_d(insn))
                            } else {
                                pc.saturating_add(1)
                            };
                            continue;
                        }
                        compare_tail!(insn, aux!());
                    }
                    o @ (op::JUMPIFLT | op::JUMPIFNOTLT) => {
                        let (ra, rb) = (r!(a), r!(aux!() as u8));
                        let want = o == op::JUMPIFLT;
                        if let (Value::Number(x), Value::Number(y)) = (ra, rb) {
                            pc = if (x < y) == want {
                                jump(pc, insn_d(insn))
                            } else {
                                pc.saturating_add(1)
                            };
                            continue;
                        }
                        compare_tail!(insn, aux!());
                    }
                    op::ADD => arith!(insn, a, Arith::Add, r!(b), r!(c)),
                    op::SUB => arith!(insn, a, Arith::Sub, r!(b), r!(c)),
                    op::MUL => arith!(insn, a, Arith::Mul, r!(b), r!(c)),
                    op::DIV => arith!(insn, a, Arith::Div, r!(b), r!(c)),
                    op::MOD => arith!(insn, a, Arith::Mod, r!(b), r!(c)),
                    op::POW => arith!(insn, a, Arith::Pow, r!(b), r!(c)),
                    op::IDIV => arith!(insn, a, Arith::Idiv, r!(b), r!(c)),
                    op::ADDK => arith!(insn, a, Arith::Add, r!(b), kst!(c)),
                    op::SUBK => arith!(insn, a, Arith::Sub, r!(b), kst!(c)),
                    op::MULK => arith!(insn, a, Arith::Mul, r!(b), kst!(c)),
                    op::DIVK => arith!(insn, a, Arith::Div, r!(b), kst!(c)),
                    op::MODK => arith!(insn, a, Arith::Mod, r!(b), kst!(c)),
                    op::IDIVK => arith!(insn, a, Arith::Idiv, r!(b), kst!(c)),
                    op::POWK => {
                        let (x, y) = (r!(b), kst!(c));
                        if let (Value::Number(nb), Value::Number(nk)) = (x, y) {
                            // C++ specializes ^2, ^0.5 and ^3; sqrt(-0) is
                            // -0 where pow gives 0, so this is observable.
                            let v = if nk == 2.0 {
                                nb * nb
                            } else if nk == 0.5 {
                                nb.sqrt()
                            } else if nk == 3.0 {
                                nb * nb * nb
                            } else {
                                nb.powf(nk)
                            };
                            set_r!(a, Value::Number(v));
                            continue;
                        }
                        sync_pc!();
                        if !self.arith_tail(insn, k, pc)? {
                            continue 'frame;
                        }
                    }
                    op::SUBRK => arith!(insn, a, Arith::Sub, kst!(b), r!(c)),
                    op::DIVRK => arith!(insn, a, Arith::Div, kst!(b), r!(c)),
                    op::AND => {
                        let x = r!(b);
                        set_r!(a, if x.truthy() { r!(c) } else { x });
                        continue;
                    }
                    op::OR => {
                        let x = r!(b);
                        set_r!(a, if x.truthy() { x } else { r!(c) });
                        continue;
                    }
                    op::ANDK => {
                        let x = r!(b);
                        set_r!(a, if x.truthy() { kst!(c) } else { x });
                        continue;
                    }
                    op::ORK => {
                        let x = r!(b);
                        set_r!(a, if x.truthy() { x } else { kst!(c) });
                        continue;
                    }
                    op::CONCAT => {
                        sync_pc!();
                        let total = usize::from(c)
                            .saturating_sub(usize::from(b))
                            .saturating_add(1);
                        match self.concat_step(total, base.saturating_add(usize::from(c)))? {
                            ConcatStep::Done => {}
                            ConcatStep::Call {
                                call,
                                slot,
                                total,
                                last,
                            } => {
                                let kind = FinishKind::Concat {
                                    a,
                                    b,
                                    slot,
                                    total,
                                    last,
                                };
                                self.push_meta(call, kind, pc)?;
                                continue 'frame;
                            }
                        }
                        set_reg!(a, reg!(b));
                        self.safepoint();
                    }
                    op::NOT => {
                        set_r!(a, Value::Boolean(!r!(b).truthy()));
                        continue;
                    }
                    op::MINUS => {
                        let x = r!(b);
                        if let Value::Number(n) = x {
                            set_r!(a, Value::Number(-n));
                            continue;
                        }
                        sync_pc!();
                        if !self.arith_tail(insn, k, pc)? {
                            continue 'frame;
                        }
                    }
                    // A table with no metatable, or a string, beside the
                    // window; anything else (a `__len` question) is general.
                    op::LENGTH => {
                        let x = r!(b);
                        match x {
                            Value::Table(t) if self.heap.table(t).meta.is_none() => {
                                set_r!(a, Value::Number(self.heap.table(t).len() as f64));
                                continue;
                            }
                            Value::String(id) => {
                                set_r!(a, Value::Number(self.heap.str_bytes(id).len() as f64));
                                continue;
                            }
                            _ => {}
                        }
                        sync_pc!();
                        let step = self.length_step(x)?;
                        if !self.store_tail(step, a, pc)? {
                            continue 'frame;
                        }
                    }
                    op::NEWTABLE => {
                        sync_pc!();
                        let narray = aux!() as usize;
                        pc = pc.saturating_add(1);
                        let nhash = if b == 0 {
                            0
                        } else {
                            1usize
                                .checked_shl(u32::from(b).saturating_sub(1))
                                .unwrap_or(usize::MAX)
                        };
                        let t = self.new_table(narray, nhash)?;
                        set_reg!(a, Value::Table(t));
                        self.safepoint();
                    }
                    op::DUPTABLE => {
                        sync_pc!();
                        let Value::Table(template) = kst!(insn_d(insn)) else {
                            return Err(self.runtime_error("invalid table constant"));
                        };
                        // Charged before it is built (`Heap::clone_table`).
                        let t = self.heap.clone_table(template)?;
                        set_reg!(a, Value::Table(t));
                        self.safepoint();
                    }
                    op::SETLIST => {
                        sync_pc!();
                        let index = aux!() as usize;
                        pc = pc.saturating_add(1);
                        let first = base.saturating_add(usize::from(b));
                        let count = if c == 0 {
                            let n = self.exec.top.saturating_sub(first);
                            self.exec.top = top_frame!().top();
                            n
                        } else {
                            usize::from(c).saturating_sub(1)
                        };
                        // C++ skips a non-table (a guard against forged
                        // bytecode); so does this.
                        if let Value::Table(t) = reg!(a)
                            && let Some(start) = index.checked_sub(1)
                        {
                            // Compiled code only fills fresh tables; forged
                            // code must not write a sandboxed one.
                            if self.heap.table(t).readonly {
                                return Err(
                                    self.runtime_error("attempt to modify a readonly table")
                                );
                            }
                            let last = start.saturating_add(count);
                            if last > self.heap.table(t).array().len() {
                                self.resize_array(t, last)?;
                            }
                            for i in 0..count {
                                let v = self.get(first.saturating_add(i));
                                if let Some(slot) = self
                                    .heap
                                    .table_mut(t)
                                    .array_mut()
                                    .get_mut(start.saturating_add(i))
                                {
                                    *slot = v;
                                }
                            }
                        }
                    }
                    // Registers `a + 1`, `a + 2` wrap as `u8`: the verifier
                    // keeps them below `maxstacksize`, so they never do.
                    op::FORNPREP => {
                        if !(matches!(r!(a), Value::Number(_))
                            && matches!(r!(a.wrapping_add(1)), Value::Number(_))
                            && matches!(r!(a.wrapping_add(2)), Value::Number(_)))
                        {
                            sync_pc!();
                            self.fornprep_tail(a)?;
                            regs = window!();
                        }
                        let limit = number(r!(a));
                        let step = number(r!(a.wrapping_add(1)));
                        let idx = number(r!(a.wrapping_add(2)));
                        let run = if step > 0.0 {
                            idx <= limit
                        } else {
                            limit <= idx
                        };
                        if !run {
                            pc = jump(pc, insn_d(insn));
                        }
                        continue;
                    }
                    op::FORNLOOP => {
                        if self.interrupt_on {
                            sync_pc!();
                            self.interrupt(host)?;
                            regs = window!();
                        }
                        let limit = number(r!(a));
                        let step = number(r!(a.wrapping_add(1)));
                        let idx = number(r!(a.wrapping_add(2))) + step;
                        set_r!(a.wrapping_add(2), Value::Number(idx));
                        let run = if step > 0.0 {
                            idx <= limit
                        } else {
                            limit <= idx
                        };
                        if run {
                            pc = jump(pc, insn_d(insn));
                        }
                        continue;
                    }
                    op::FORGPREP => {
                        sync_pc!();
                        let ra = reg!(a);
                        if !matches!(ra, Value::Function(_)) {
                            // C++ reads `__iter` and `__call` from tables and
                            // full userdata only.
                            let mt = match ra {
                                Value::Table(t) => self.heap.table(t).meta,
                                Value::Userdata(u) => self.heap.userdata(u).meta,
                                _ => None,
                            };
                            if let Some(f) = self.fast_tm(mt, crate::vm::Tm::Iter) {
                                let slot = base.saturating_add(usize::from(a));
                                set_reg!(usize::from(a).saturating_add(1), ra);
                                set_reg!(a, f);
                                self.exec.top = slot.saturating_add(2);
                                let finish = Finish {
                                    kind: FinishKind::IterPrep { a },
                                    post: Post::Value,
                                    pc: jump(pc, insn_d(insn)),
                                };
                                self.wait_then_call(finish, slot, 1, 3, CallMode::Counted)?;
                                continue 'frame;
                            } else if self.fast_tm(mt, crate::vm::Tm::Call).is_some() {
                                // Called by FORGLOOP.
                            } else if matches!(ra, Value::Table(_)) {
                                set_reg!(usize::from(a).saturating_add(1), ra);
                                set_reg!(
                                    usize::from(a).saturating_add(2),
                                    Value::LightUserdata(0, LU_TAG_ITERATOR)
                                );
                                set_reg!(a, Value::Nil);
                            } else {
                                return Err(self.type_error(ra, "iterate over"));
                            }
                        }
                        pc = jump(pc, insn_d(insn));
                    }
                    op::FORGLOOP => {
                        sync_pc!();
                        self.interrupt(host)?;
                        let aux = aux!();
                        let ra = usize::from(a);
                        if let (Value::Nil, Value::Table(t)) =
                            (reg!(ra), reg!(ra.saturating_add(1)))
                        {
                            let index = match reg!(ra.saturating_add(2)) {
                                Value::LightUserdata(p, _) => {
                                    usize::try_from(p).unwrap_or(usize::MAX)
                                }
                                _ => 0,
                            };
                            let vars = aux as i32;
                            if vars > 2 {
                                for i in 2..vars.unsigned_abs() as usize {
                                    set_reg!(ra.saturating_add(3).saturating_add(i), Value::Nil);
                                }
                            }
                            let table = self.heap.table(t);
                            if vars < 0 && table.array().get(index).is_none_or(|v| v.is_nil()) {
                                pc = pc.saturating_add(1);
                                regs = window!();
                                continue;
                            }
                            match table.next_from(index) {
                                Some((next, key, value)) => {
                                    set_reg!(
                                        ra.saturating_add(2),
                                        Value::LightUserdata(next as u64, LU_TAG_ITERATOR)
                                    );
                                    set_reg!(ra.saturating_add(3), key);
                                    set_reg!(ra.saturating_add(4), value);
                                    pc = jump(pc, insn_d(insn));
                                }
                                None => pc = pc.saturating_add(1),
                            }
                        } else {
                            let call = base.saturating_add(ra).saturating_add(3);
                            set_reg!(ra.saturating_add(5), reg!(ra.saturating_add(2)));
                            set_reg!(ra.saturating_add(4), reg!(ra.saturating_add(1)));
                            set_reg!(ra.saturating_add(3), reg!(ra));
                            self.ensure_stack(call.saturating_add(3).saturating_add(MIN_STACK))?;
                            self.exec.top = call.saturating_add(3);
                            // C++ `luaD_performcall`: counted, and yieldable.
                            let finish = Finish {
                                kind: FinishKind::ForGLoop {
                                    a,
                                    target: jump(pc, insn_d(insn)),
                                },
                                post: Post::Value,
                                pc: pc.saturating_add(1),
                            };
                            let nresults = (aux & 0xff) as i32;
                            self.wait_then_call(finish, call, 2, nresults, CallMode::CountedYieldable)?;
                            continue 'frame;
                        }
                    }
                    o @ (op::FORGPREP_INEXT | op::FORGPREP_NEXT) => {
                        sync_pc!();
                        let ra = usize::from(a);
                        let safe = self.heap.table(self.closure_env(closure)).safeenv;
                        let state = reg!(ra.saturating_add(1));
                        let control = reg!(ra.saturating_add(2));
                        let builtin = safe
                            && matches!(state, Value::Table(_))
                            && if o == op::FORGPREP_INEXT {
                                matches!(control, Value::Number(n) if n == 0.0)
                            } else {
                                control.is_nil()
                            };
                        if builtin {
                            set_reg!(ra, Value::Nil);
                            set_reg!(
                                ra.saturating_add(2),
                                Value::LightUserdata(0, LU_TAG_ITERATOR)
                            );
                        } else if !matches!(reg!(ra), Value::Function(_)) {
                            return Err(self.type_error(reg!(ra), "iterate over"));
                        }
                        pc = jump(pc, insn_d(insn));
                    }
                    op::GETVARARGS => {
                        sync_pc!();
                        let frame = top_frame!();
                        let numparams = usize::from(protos.get(frame.lua_proto()).numparams);
                        let func = frame.func();
                        let n = base
                            .saturating_sub(func)
                            .saturating_sub(1)
                            .saturating_sub(numparams);
                        let from = base.saturating_sub(n);
                        let ra = base.saturating_add(usize::from(a));
                        if b == 0 {
                            self.ensure_stack(ra.saturating_add(n).saturating_add(MIN_STACK))?;
                            self.copy_slots(from, ra, n);
                            self.exec.top = ra.saturating_add(n);
                        } else {
                            let want = usize::from(b).saturating_sub(1);
                            let copy = want.min(n);
                            self.copy_slots(from, ra, copy);
                            self.fill_nil(ra.saturating_add(copy), ra.saturating_add(want));
                        }
                    }
                    op::JUMPXEQKNIL => {
                        let aux = aux!();
                        let hit = r!(a).is_nil();
                        pc = if hit != (aux >> 31 != 0) {
                            jump(pc, insn_d(insn))
                        } else {
                            pc.saturating_add(1)
                        };
                        continue;
                    }
                    op::JUMPXEQKB => {
                        let aux = aux!();
                        let hit = matches!(r!(a), Value::Boolean(v) if v == (aux & 1 != 0));
                        pc = if hit != (aux >> 31 != 0) {
                            jump(pc, insn_d(insn))
                        } else {
                            pc.saturating_add(1)
                        };
                        continue;
                    }
                    op::JUMPXEQKN => {
                        let aux = aux!();
                        let kv = kst!(aux & 0x00ff_ffff);
                        let hit =
                            matches!((r!(a), kv), (Value::Number(x), Value::Number(y)) if x == y);
                        pc = if hit != (aux >> 31 != 0) {
                            jump(pc, insn_d(insn))
                        } else {
                            pc.saturating_add(1)
                        };
                        continue;
                    }
                    op::JUMPXEQKS => {
                        let aux = aux!();
                        let kv = kst!(aux & 0x00ff_ffff);
                        let hit =
                            matches!((r!(a), kv), (Value::String(x), Value::String(y)) if x == y);
                        pc = if hit != (aux >> 31 != 0) {
                            jump(pc, insn_d(insn))
                        } else {
                            pc.saturating_add(1)
                        };
                        continue;
                    }
                    _ => {
                        self.exec.pc = pc;
                        return Err(self.runtime_error("invalid opcode"));
                    }
                }
                regs = window!();
            }
        }
    }

    /// C++ `luaH_resizearray`, charging growth to the memory budget.
    pub(crate) fn resize_array(&mut self, table: TableId, size: usize) -> VmResult<()> {
        self.heap
            .table_resize_array(table, size)
            .map_err(|e| self.table_error(e))
    }
}
