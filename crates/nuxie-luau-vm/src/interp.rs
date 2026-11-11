//! The interpreter loop (C++ `luau_execute`, `lvmexecute.cpp`).
//!
//! One `match` per instruction over verified code. Each opcode follows the
//! C++ case; the comments name the deviations. `FASTCALL*` skip their fast
//! path and fall through to the `CALL` that follows, which computes the
//! same result through the library function: the fast paths are speed work
//! for a later lane, not semantics. The exception is a builtin whose fast
//! body gives different results from its library function (`libs/fast.rs`
//! lists them), whose fast body runs here as upstream runs it.
//!
//! `self.pc` is kept equal to the running frame's `savedpc` (the index
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
use crate::value::{FnId, LU_TAG_ITERATOR, TableId, UdId, Value};
use crate::vm::{MIN_STACK, Core};

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
            Function::Native(_) => self.globals,
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
        self.top = at.saturating_add(1).saturating_add(nargs);
        self.push_call(at, nargs, call.nresults, CallMode::Counted)?;
        self.set_caller_wait(Wait::Op(Finish {
            kind,
            post: call.post,
            pc,
        }));
        Ok(())
    }

    /// Sets the wait state of the frame below the one just pushed.
    fn set_caller_wait(&mut self, wait: Wait) {
        let below = self.frames.len().checked_sub(2);
        if let Some(frame) = below.and_then(|i| self.frames.get_mut(i)) {
            frame.wait = wait;
        }
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
        let Some(frame) = self.frames.last() else {
            return Ok(Flow::Exit(Exit::Returned));
        };
        let (base, ftop, savedpc) = (frame.base, frame.top, frame.savedpc);
        let proto_id = frame.proto;
        let protos = std::sync::Arc::clone(self.heap.proto_table());
        let proto = proto_id.map(|id| protos.get(id));
        self.top = ftop;
        // Errors below name the waiting instruction.
        self.pc = savedpc;
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
        if let Some(frame) = self.frames.last_mut() {
            frame.savedpc = next;
        }
        Ok(Flow::Continue)
    }

    /// Runs the frame stack until `execute`'s frame returns, the thread
    /// finishes, or the main thread yields to the host.
    pub(crate) fn run_frames(&mut self, host: &mut dyn Dispatch) -> VmResult<Exit> {
        // The prototypes, held for the loop (heap.rs, "Prototypes"). Nothing
        // the loop runs adds one, but a newer table is picked up anyway.
        let mut protos = std::sync::Arc::clone(self.heap.proto_table());
        'frame: loop {
            let Some(frame) = self.frames.last() else {
                return Ok(Exit::Returned);
            };
            let Some(proto_id) = frame.proto else {
                match self.step_native(host)? {
                    Flow::Exit(exit) => return Ok(exit),
                    Flow::Inline | Flow::Continue => continue 'frame,
                }
            };
            if !std::sync::Arc::ptr_eq(&protos, self.heap.proto_table()) {
                protos = std::sync::Arc::clone(self.heap.proto_table());
            }
            let proto = protos.get(proto_id);
            let closure = frame.closure;
            let base = frame.base;
            let func = frame.func;
            let ftop = frame.top;
            let mut pc = frame.savedpc;
            let code: &[u32] = &proto.code;
            let k: &[Value] = &proto.k;

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
            macro_rules! arith {
                ($a:expr, $op:expr, $x:expr, $y:expr) => {{
                    let (x, y) = ($x, $y);
                    match (x, y) {
                        (Value::Number(p), Value::Number(q)) => Value::Number($op.number(p, q)),
                        _ => meta!(self.arith_step($op, x, y)?, FinishKind::Store($a), pc),
                    }
                }};
            }

            loop {
                let Some(&insn) = code.get(pc) else {
                    return Err(self.runtime_error("execution ran past the end of the function"));
                };
                pc = pc.saturating_add(1);
                self.pc = pc;
                let a = insn_a(insn);
                let b = insn_b(insn);
                let c = insn_c(insn);
                match insn_op(insn) {
                    op::NOP | op::COVERAGE | op::PREPVARARGS | op::CAPTURE => {}
                    op::FASTCALL
                    | op::FASTCALL1
                    | op::FASTCALL2
                    | op::FASTCALL2K
                    | op::FASTCALL3 => {
                        let op = insn_op(insn);
                        let has_aux = !matches!(op, op::FASTCALL | op::FASTCALL1);
                        let aux_word = if has_aux { aux!() } else { 0 };
                        if has_aux {
                            pc = pc.saturating_add(1);
                        }
                        // Upstream runs the builtin's fast body here under
                        // safeenv and skips the CALL. Only bodies whose
                        // results differ from the library function are
                        // ported (`libs/fast.rs`); every other builtin
                        // falls through to the CALL.
                        if crate::libs::fast::has_fast_body(a) {
                            // C counts from the instruction after FASTCALL,
                            // which for the forms with an aux word is the
                            // aux word itself (upstream: `skip = C - 1`
                            // after `pc++` past aux).
                            let call_at = if has_aux {
                                pc.saturating_add(usize::from(c)).saturating_sub(1)
                            } else {
                                pc.saturating_add(usize::from(c))
                            };
                            let call = code.get(call_at).copied().unwrap_or(0);
                            let env = self.closure_env(closure);
                            if insn_op(call) == op::CALL && self.heap.table(env).safeenv {
                                let ra = base.saturating_add(usize::from(insn_a(call)));
                                let nresults = i32::from(insn_c(call)).saturating_sub(1);
                                let (arg0, rest, nparams) = match op {
                                    op::FASTCALL => {
                                        let nparams = match insn_b(call) {
                                            0 => self.top.saturating_sub(ra.saturating_add(1)),
                                            nb => usize::from(nb).saturating_sub(1),
                                        };
                                        (
                                            self.get(ra.saturating_add(1)),
                                            [self.get(ra.saturating_add(2)), self.get(ra.saturating_add(3))],
                                            nparams,
                                        )
                                    }
                                    op::FASTCALL1 => (reg!(b), [Value::Nil; 2], 1),
                                    op::FASTCALL2 => (reg!(b), [reg!(aux_word & 0xff), Value::Nil], 2),
                                    op::FASTCALL2K => (reg!(b), [kst!(aux_word), Value::Nil], 2),
                                    _ => (
                                        reg!(b),
                                        [reg!(aux_word & 0xff), reg!((aux_word >> 8) & 0xff)],
                                        3,
                                    ),
                                };
                                if let Some((results, n)) =
                                    crate::libs::fast::run(a, arg0, rest, nparams, nresults)
                                {
                                    for (i, &v) in results.iter().take(n).enumerate() {
                                        self.set(ra.saturating_add(i), v);
                                    }
                                    self.top = if nresults < 0 { ra.saturating_add(n) } else { ftop };
                                    pc = call_at.saturating_add(1);
                                }
                            }
                        }
                    }
                    op::LOADNIL => set_reg!(a, Value::Nil),
                    op::LOADB => {
                        set_reg!(a, Value::Boolean(b != 0));
                        pc = pc.saturating_add(usize::from(c));
                    }
                    op::LOADN => set_reg!(a, Value::Number(f64::from(insn_d(insn)))),
                    op::LOADK => set_reg!(a, kst!(insn_d(insn))),
                    op::LOADKX => {
                        let aux = aux!();
                        pc = pc.saturating_add(1);
                        set_reg!(a, kst!(aux));
                    }
                    op::MOVE => set_reg!(a, reg!(b)),
                    op::GETGLOBAL => {
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
                        let key = kst!(aux!());
                        pc = pc.saturating_add(1);
                        let env = self.closure_env(closure);
                        meta!(
                            self.set_table_step(Value::Table(env), key, reg!(a))?,
                            FinishKind::Discard,
                            pc
                        );
                    }
                    op::GETUPVAL => {
                        let v = match self.closure_upval_raw(closure, usize::from(b)) {
                            Value::UpRef(id) => self.upval_get(id),
                            v => v,
                        };
                        set_reg!(a, v);
                    }
                    op::SETUPVAL => {
                        let v = reg!(a);
                        match self.closure_upval_raw(closure, usize::from(b)) {
                            Value::UpRef(id) => self.upval_set(id, v),
                            _ => {
                                if let Some(slot) = self.closure_upval_slot(closure, usize::from(b))
                                {
                                    *slot = v;
                                }
                            }
                        }
                    }
                    op::CLOSEUPVALS => self.close_upvals(base.saturating_add(usize::from(a))),
                    op::GETIMPORT => {
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
                    op::GETTABLE => {
                        let v = meta!(
                            self.get_table_step(reg!(b), reg!(c))?,
                            FinishKind::Store(a),
                            pc
                        );
                        set_reg!(a, v);
                    }
                    op::SETTABLE => {
                        meta!(
                            self.set_table_step(reg!(b), reg!(c), reg!(a))?,
                            FinishKind::Discard,
                            pc
                        );
                    }
                    op::GETTABLEKS => {
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
                        let key = kst!(aux!());
                        pc = pc.saturating_add(1);
                        meta!(
                            self.set_table_step(reg!(b), key, reg!(a))?,
                            FinishKind::Discard,
                            pc
                        );
                    }
                    op::GETTABLEN => {
                        let key = Value::Number(f64::from(c) + 1.0);
                        let v = meta!(self.get_table_step(reg!(b), key)?, FinishKind::Store(a), pc);
                        set_reg!(a, v);
                    }
                    op::SETTABLEN => {
                        let key = Value::Number(f64::from(c) + 1.0);
                        meta!(
                            self.set_table_step(reg!(b), key, reg!(a))?,
                            FinishKind::Discard,
                            pc
                        );
                    }
                    op::NEWCLOSURE => {
                        let Some(&child_id) = usize::try_from(insn_d(insn))
                            .ok()
                            .and_then(|i| proto.children.get(i))
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
                        self.interrupt(host)?;
                        if insn_op(insn) == op::CALLFB {
                            pc = pc.saturating_add(1);
                            self.pc = pc;
                        }
                        let callee = base.saturating_add(usize::from(a));
                        let nresults = i32::from(c).saturating_sub(1);
                        let nargs = if b == 0 {
                            self.top.saturating_sub(callee.saturating_add(1))
                        } else {
                            usize::from(b).saturating_sub(1)
                        };
                        self.top = callee.saturating_add(1).saturating_add(nargs);
                        self.push_call(callee, nargs, nresults, CallMode::Plain)?;
                        if self.frames.last().is_some_and(|f| f.proto.is_some()) {
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
                        self.interrupt(host)?;
                        let ra = base.saturating_add(usize::from(a));
                        let n = if b == 0 {
                            self.top.saturating_sub(ra)
                        } else {
                            usize::from(b).saturating_sub(1)
                        };
                        // C++ relies on the compiler's CLOSEUPVALS here;
                        // closing again is a no-op for compiled code and
                        // keeps hand-written bytecode from leaving open
                        // upvalues on a dead frame.
                        self.close_upvals(base);
                        let Some(done) = self.frames.pop() else {
                            return Ok(Exit::Returned);
                        };
                        self.move_results(done.func, ra, n, done.nresults)?;
                        match self.deliver(done)? {
                            Flow::Exit(exit) => return Ok(exit),
                            Flow::Inline | Flow::Continue => continue 'frame,
                        }
                    }
                    op::JUMP => pc = jump(pc, insn_d(insn)),
                    op::JUMPBACK => {
                        self.interrupt(host)?;
                        pc = jump(pc, insn_d(insn));
                    }
                    op::JUMPX => {
                        self.interrupt(host)?;
                        pc = jump(pc, insn_e(insn));
                    }
                    op::JUMPIF => {
                        if reg!(a).truthy() {
                            pc = jump(pc, insn_d(insn));
                        }
                    }
                    op::JUMPIFNOT => {
                        if !reg!(a).truthy() {
                            pc = jump(pc, insn_d(insn));
                        }
                    }
                    o @ (op::JUMPIFEQ
                    | op::JUMPIFNOTEQ
                    | op::JUMPIFLE
                    | op::JUMPIFNOTLE
                    | op::JUMPIFLT
                    | op::JUMPIFNOTLT) => {
                        let ra = reg!(a);
                        let rb = reg!(aux!());
                        let want = matches!(o, op::JUMPIFEQ | op::JUMPIFLE | op::JUMPIFLT);
                        let branch = FinishKind::Branch {
                            want,
                            target: jump(pc, insn_d(insn)),
                        };
                        let next = pc.saturating_add(1);
                        let result = match o {
                            op::JUMPIFEQ | op::JUMPIFNOTEQ => {
                                ra.same_type(rb)
                                    && meta!(self.equal_step(ra, rb), branch, next).truthy()
                            }
                            op::JUMPIFLE | op::JUMPIFNOTLE => match (ra, rb) {
                                (Value::Number(x), Value::Number(y)) => x <= y,
                                _ => meta!(self.less_equal_step(ra, rb)?, branch, next).truthy(),
                            },
                            _ => match (ra, rb) {
                                (Value::Number(x), Value::Number(y)) => x < y,
                                _ => meta!(self.less_than_step(ra, rb)?, branch, next).truthy(),
                            },
                        };
                        pc = if result == want {
                            jump(pc, insn_d(insn))
                        } else {
                            pc.saturating_add(1)
                        };
                    }
                    op::ADD => set_reg!(a, arith!(a, Arith::Add, reg!(b), reg!(c))),
                    op::SUB => set_reg!(a, arith!(a, Arith::Sub, reg!(b), reg!(c))),
                    op::MUL => set_reg!(a, arith!(a, Arith::Mul, reg!(b), reg!(c))),
                    op::DIV => set_reg!(a, arith!(a, Arith::Div, reg!(b), reg!(c))),
                    op::MOD => set_reg!(a, arith!(a, Arith::Mod, reg!(b), reg!(c))),
                    op::POW => set_reg!(a, arith!(a, Arith::Pow, reg!(b), reg!(c))),
                    op::IDIV => set_reg!(a, arith!(a, Arith::Idiv, reg!(b), reg!(c))),
                    op::ADDK => set_reg!(a, arith!(a, Arith::Add, reg!(b), kst!(c))),
                    op::SUBK => set_reg!(a, arith!(a, Arith::Sub, reg!(b), kst!(c))),
                    op::MULK => set_reg!(a, arith!(a, Arith::Mul, reg!(b), kst!(c))),
                    op::DIVK => set_reg!(a, arith!(a, Arith::Div, reg!(b), kst!(c))),
                    op::MODK => set_reg!(a, arith!(a, Arith::Mod, reg!(b), kst!(c))),
                    op::IDIVK => set_reg!(a, arith!(a, Arith::Idiv, reg!(b), kst!(c))),
                    op::POWK => {
                        let (x, y) = (reg!(b), kst!(c));
                        let v = match (x, y) {
                            // C++ specializes ^2, ^0.5 and ^3; sqrt(-0) is
                            // -0 where pow gives 0, so this is observable.
                            (Value::Number(nb), Value::Number(nk)) => Value::Number(if nk == 2.0 {
                                nb * nb
                            } else if nk == 0.5 {
                                nb.sqrt()
                            } else if nk == 3.0 {
                                nb * nb * nb
                            } else {
                                nb.powf(nk)
                            }),
                            _ => {
                                meta!(self.arith_step(Arith::Pow, x, y)?, FinishKind::Store(a), pc)
                            }
                        };
                        set_reg!(a, v);
                    }
                    op::SUBRK => set_reg!(a, arith!(a, Arith::Sub, kst!(b), reg!(c))),
                    op::DIVRK => set_reg!(a, arith!(a, Arith::Div, kst!(b), reg!(c))),
                    op::AND => {
                        let x = reg!(b);
                        set_reg!(a, if x.truthy() { reg!(c) } else { x });
                    }
                    op::OR => {
                        let x = reg!(b);
                        set_reg!(a, if x.truthy() { x } else { reg!(c) });
                    }
                    op::ANDK => {
                        let x = reg!(b);
                        set_reg!(a, if x.truthy() { kst!(c) } else { x });
                    }
                    op::ORK => {
                        let x = reg!(b);
                        set_reg!(a, if x.truthy() { x } else { kst!(c) });
                    }
                    op::CONCAT => {
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
                    op::NOT => set_reg!(a, Value::Boolean(!reg!(b).truthy())),
                    op::MINUS => {
                        let x = reg!(b);
                        let v = match x {
                            Value::Number(n) => Value::Number(-n),
                            _ => {
                                meta!(self.arith_step(Arith::Unm, x, x)?, FinishKind::Store(a), pc)
                            }
                        };
                        set_reg!(a, v);
                    }
                    op::LENGTH => {
                        let v = meta!(self.length_step(reg!(b))?, FinishKind::Store(a), pc);
                        set_reg!(a, v);
                    }
                    op::NEWTABLE => {
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
                        let Value::Table(template) = kst!(insn_d(insn)) else {
                            return Err(self.runtime_error("invalid table constant"));
                        };
                        // Charged before it is built (`Heap::clone_table`).
                        let t = self.heap.clone_table(template)?;
                        set_reg!(a, Value::Table(t));
                        self.safepoint();
                    }
                    op::SETLIST => {
                        let index = aux!() as usize;
                        pc = pc.saturating_add(1);
                        let first = base.saturating_add(usize::from(b));
                        let count = if c == 0 {
                            let n = self.top.saturating_sub(first);
                            self.top = ftop;
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
                    op::FORNPREP => {
                        if !(matches!(reg!(a), Value::Number(_))
                            && matches!(reg!(usize::from(a).saturating_add(1)), Value::Number(_))
                            && matches!(reg!(usize::from(a).saturating_add(2)), Value::Number(_)))
                        {
                            for (offset, what) in
                                [(0usize, "limit"), (1, "step"), (2, "initial value")]
                            {
                                let slot = usize::from(a).saturating_add(offset);
                                let v = reg!(slot);
                                if matches!(v, Value::Number(_)) {
                                    continue;
                                }
                                match self.to_number(v) {
                                    Some(n) => set_reg!(slot, Value::Number(n)),
                                    None => {
                                        let tn = self.obj_type_name(v);
                                        return Err(self.runtime_error(format!(
                                            "invalid 'for' {what} (number expected, got {tn})"
                                        )));
                                    }
                                }
                            }
                        }
                        let limit = number(reg!(a));
                        let step = number(reg!(usize::from(a).saturating_add(1)));
                        let idx = number(reg!(usize::from(a).saturating_add(2)));
                        let run = if step > 0.0 {
                            idx <= limit
                        } else {
                            limit <= idx
                        };
                        if !run {
                            pc = jump(pc, insn_d(insn));
                        }
                    }
                    op::FORNLOOP => {
                        self.interrupt(host)?;
                        let limit = number(reg!(a));
                        let step = number(reg!(usize::from(a).saturating_add(1)));
                        let idx = number(reg!(usize::from(a).saturating_add(2))) + step;
                        set_reg!(usize::from(a).saturating_add(2), Value::Number(idx));
                        let run = if step > 0.0 {
                            idx <= limit
                        } else {
                            limit <= idx
                        };
                        if run {
                            pc = jump(pc, insn_d(insn));
                        }
                    }
                    op::FORGPREP => {
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
                                self.top = slot.saturating_add(2);
                                self.push_call(slot, 1, 3, CallMode::Counted)?;
                                self.set_caller_wait(Wait::Op(Finish {
                                    kind: FinishKind::IterPrep { a },
                                    post: Post::Value,
                                    pc: jump(pc, insn_d(insn)),
                                }));
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
                            self.top = call.saturating_add(3);
                            // C++ `luaD_performcall`: counted, and yieldable.
                            self.push_call(
                                call,
                                2,
                                (aux & 0xff) as i32,
                                CallMode::CountedYieldable,
                            )?;
                            self.set_caller_wait(Wait::Op(Finish {
                                kind: FinishKind::ForGLoop {
                                    a,
                                    target: jump(pc, insn_d(insn)),
                                },
                                post: Post::Value,
                                pc: pc.saturating_add(1),
                            }));
                            continue 'frame;
                        }
                    }
                    o @ (op::FORGPREP_INEXT | op::FORGPREP_NEXT) => {
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
                        let numparams = usize::from(proto.numparams);
                        let n = base
                            .saturating_sub(func)
                            .saturating_sub(1)
                            .saturating_sub(numparams);
                        let from = base.saturating_sub(n);
                        let ra = base.saturating_add(usize::from(a));
                        if b == 0 {
                            self.ensure_stack(ra.saturating_add(n).saturating_add(MIN_STACK))?;
                            self.copy_slots(from, ra, n);
                            self.top = ra.saturating_add(n);
                        } else {
                            let want = usize::from(b).saturating_sub(1);
                            let copy = want.min(n);
                            self.copy_slots(from, ra, copy);
                            self.fill_nil(ra.saturating_add(copy), ra.saturating_add(want));
                        }
                    }
                    op::JUMPXEQKNIL => {
                        let aux = aux!();
                        let hit = reg!(a).is_nil();
                        pc = if hit != (aux >> 31 != 0) {
                            jump(pc, insn_d(insn))
                        } else {
                            pc.saturating_add(1)
                        };
                    }
                    op::JUMPXEQKB => {
                        let aux = aux!();
                        let hit = matches!(reg!(a), Value::Boolean(v) if v == (aux & 1 != 0));
                        pc = if hit != (aux >> 31 != 0) {
                            jump(pc, insn_d(insn))
                        } else {
                            pc.saturating_add(1)
                        };
                    }
                    op::JUMPXEQKN => {
                        let aux = aux!();
                        let kv = kst!(aux & 0x00ff_ffff);
                        let hit =
                            matches!((reg!(a), kv), (Value::Number(x), Value::Number(y)) if x == y);
                        pc = if hit != (aux >> 31 != 0) {
                            jump(pc, insn_d(insn))
                        } else {
                            pc.saturating_add(1)
                        };
                    }
                    op::JUMPXEQKS => {
                        let aux = aux!();
                        let kv = kst!(aux & 0x00ff_ffff);
                        let hit =
                            matches!((reg!(a), kv), (Value::String(x), Value::String(y)) if x == y);
                        pc = if hit != (aux >> 31 != 0) {
                            jump(pc, insn_d(insn))
                        } else {
                            pc.saturating_add(1)
                        };
                    }
                    _ => return Err(self.runtime_error("invalid opcode")),
                }
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
