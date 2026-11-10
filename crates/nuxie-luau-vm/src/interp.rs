//! The interpreter loop (C++ `luau_execute`, `lvmexecute.cpp`).
//!
//! One `match` per instruction over verified code. Each opcode follows the
//! C++ case; the comments name the deviations. `FASTCALL*` skip their fast
//! path and fall through to the `CALL` that follows, which computes the
//! same result through the library function: the fast paths are speed work
//! for a later lane, not semantics.
//!
//! `self.pc` is kept equal to the running frame's `savedpc` (the index
//! after the current instruction word), so an error raised anywhere below
//! can name the line, and a frame pushed above can record where to resume.

use std::rc::Rc;

use crate::bytecode::{insn_a, insn_b, insn_c, insn_d, insn_e, insn_op, op};
use crate::error::VmResult;
use crate::heap::{Function, LuaClosure};
use crate::meta::Arith;
use crate::value::{FnId, LU_TAG_ITERATOR, TableId, Value};
use crate::vm::Vm;

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

impl Vm {
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
        match self.heap.functions.get_mut(id.0) {
            Function::Lua(c) => c.upvals.get_mut(i),
            Function::Native(_) => None,
        }
    }

    /// Runs Lua frames from the top frame until a boundary frame returns.
    pub(crate) fn execute(&mut self) -> VmResult<()> {
        'frame: loop {
            let Some(frame) = self.frames.last() else {
                return Ok(());
            };
            let Some(proto) = frame.proto.clone() else {
                return Err(self.runtime_error("attempt to execute a native frame"));
            };
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
            macro_rules! arith {
                ($op:expr, $x:expr, $y:expr) => {{
                    let (x, y) = ($x, $y);
                    match (x, y) {
                        (Value::Number(p), Value::Number(q)) => Value::Number($op.number(p, q)),
                        _ => self.arith($op, x, y)?,
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
                    op::NOP
                    | op::COVERAGE
                    | op::PREPVARARGS
                    | op::FASTCALL
                    | op::FASTCALL1
                    | op::CAPTURE => {}
                    op::FASTCALL2 | op::FASTCALL2K | op::FASTCALL3 => pc = pc.saturating_add(1),
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
                            self.get_table(Value::Table(env), key)?
                        } else {
                            v
                        };
                        set_reg!(a, v);
                    }
                    op::SETGLOBAL => {
                        let key = kst!(aux!());
                        pc = pc.saturating_add(1);
                        let env = self.closure_env(closure);
                        self.set_table(Value::Table(env), key, reg!(a))?;
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
                            let count = aux >> 30;
                            let mut v =
                                self.get_table(Value::Table(env), kst!((aux >> 20) & 1023))?;
                            if count > 1 {
                                v = self.get_table(v, kst!((aux >> 10) & 1023))?;
                            }
                            if count > 2 {
                                v = self.get_table(v, kst!(aux & 1023))?;
                            }
                            set_reg!(a, v);
                        }
                    }
                    op::GETTABLE => {
                        let v = self.get_table(reg!(b), reg!(c))?;
                        set_reg!(a, v);
                    }
                    op::SETTABLE => self.set_table(reg!(b), reg!(c), reg!(a))?,
                    op::GETTABLEKS => {
                        let key = kst!(aux!());
                        pc = pc.saturating_add(1);
                        let v = self.get_table(reg!(b), key)?;
                        set_reg!(a, v);
                    }
                    op::SETTABLEKS => {
                        let key = kst!(aux!());
                        pc = pc.saturating_add(1);
                        self.set_table(reg!(b), key, reg!(a))?;
                    }
                    op::GETTABLEN => {
                        let v = self.get_table(reg!(b), Value::Number(f64::from(c) + 1.0))?;
                        set_reg!(a, v);
                    }
                    op::SETTABLEN => {
                        self.set_table(reg!(b), Value::Number(f64::from(c) + 1.0), reg!(a))?
                    }
                    op::NEWCLOSURE => {
                        let Some(&child_id) = usize::try_from(insn_d(insn))
                            .ok()
                            .and_then(|i| proto.children.get(i))
                        else {
                            return Err(self.runtime_error("invalid child prototype"));
                        };
                        let child = Rc::clone(self.heap.protos.get(child_id.0));
                        let env = self.closure_env(closure);
                        let nups = usize::from(child.nups);
                        let mut upvals = Vec::with_capacity(nups);
                        for _ in 0..nups {
                            let capture = aux!();
                            pc = pc.saturating_add(1);
                            let ub = insn_b(capture);
                            upvals.push(match insn_a(capture) {
                                CAPTURE_VAL => reg!(ub),
                                CAPTURE_REF => Value::UpRef(
                                    self.find_upval(base.saturating_add(usize::from(ub)))?,
                                ),
                                _ => self.closure_upval_raw(closure, usize::from(ub)),
                            });
                        }
                        let closure_value = LuaClosure {
                            proto: child,
                            proto_id: child_id,
                            env,
                            upvals,
                            preload: false,
                        };
                        let f = self.heap.new_function(Function::Lua(closure_value))?;
                        set_reg!(a, Value::Function(f));
                        self.safepoint();
                    }
                    op::DUPCLOSURE => {
                        let Value::Function(kid) = kst!(insn_d(insn)) else {
                            return Err(self.runtime_error("invalid closure constant"));
                        };
                        let (kproto, kproto_id, kenv, nups) = match self.heap.function(kid) {
                            Function::Lua(kc) => {
                                (Rc::clone(&kc.proto), kc.proto_id, kc.env, kc.upvals.len())
                            }
                            Function::Native(_) => {
                                return Err(self.runtime_error("invalid closure constant"));
                            }
                        };
                        let env = self.closure_env(closure);
                        let fresh = |vm: &mut Vm| {
                            let c = LuaClosure {
                                proto: Rc::clone(&kproto),
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
                        if let Function::Lua(c) = self.heap.functions.get_mut(ncl.0) {
                            c.preload = false;
                        }
                        pc = pc.saturating_add(nups);
                        if ncl != kid {
                            self.safepoint();
                        }
                    }
                    op::NAMECALL => {
                        let key = kst!(aux!());
                        pc = pc.saturating_add(1);
                        let rb = reg!(b);
                        if let Value::Table(t) = rb {
                            let v = self.heap.table(t).get(key);
                            let v = if v.is_nil() {
                                self.get_table(rb, key)?
                            } else {
                                v
                            };
                            set_reg!(usize::from(a).saturating_add(1), rb);
                            set_reg!(a, v);
                            if v.is_nil() {
                                let Value::String(name) = key else {
                                    return Err(self.runtime_error("invalid method name"));
                                };
                                let tn = self.obj_type_name(rb);
                                let name = crate::vm::c_lossy(self.heap.str_bytes(name));
                                return Err(self.runtime_error(format!(
                                    "attempt to call missing method '{name}' of {tn}"
                                )));
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
                            let v = self.get_table(rb, key)?;
                            set_reg!(usize::from(a).saturating_add(1), rb);
                            set_reg!(a, v);
                        }
                    }
                    op::CALL | op::CALLFB => {
                        self.interrupt()?;
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
                        let (id, nargs) = self.resolve_callable(callee, nargs)?;
                        if matches!(self.heap.function(id), Function::Lua(_)) {
                            self.enter_lua(callee, id, nargs, nresults, false)?;
                            continue 'frame;
                        }
                        self.call_native(callee, id, nargs, nresults)?;
                        if nresults >= 0 {
                            self.top = ftop;
                        }
                    }
                    op::RETURN => {
                        self.interrupt()?;
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
                            return Ok(());
                        };
                        self.move_results(done.func, ra, n, done.nresults)?;
                        if done.boundary {
                            return Ok(());
                        }
                        if done.nresults >= 0
                            && let Some(caller) = self.frames.last()
                        {
                            self.top = caller.top;
                        }
                        continue 'frame;
                    }
                    op::JUMP => pc = jump(pc, insn_d(insn)),
                    op::JUMPBACK => {
                        self.interrupt()?;
                        pc = jump(pc, insn_d(insn));
                    }
                    op::JUMPX => {
                        self.interrupt()?;
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
                        let result = match o {
                            op::JUMPIFEQ | op::JUMPIFNOTEQ => {
                                ra.same_type(rb) && self.equal(ra, rb)?
                            }
                            op::JUMPIFLE | op::JUMPIFNOTLE => match (ra, rb) {
                                (Value::Number(x), Value::Number(y)) => x <= y,
                                _ => self.less_equal(ra, rb)?,
                            },
                            _ => match (ra, rb) {
                                (Value::Number(x), Value::Number(y)) => x < y,
                                _ => self.less_than(ra, rb)?,
                            },
                        };
                        let want = matches!(o, op::JUMPIFEQ | op::JUMPIFLE | op::JUMPIFLT);
                        pc = if result == want {
                            jump(pc, insn_d(insn))
                        } else {
                            pc.saturating_add(1)
                        };
                    }
                    op::ADD => set_reg!(a, arith!(Arith::Add, reg!(b), reg!(c))),
                    op::SUB => set_reg!(a, arith!(Arith::Sub, reg!(b), reg!(c))),
                    op::MUL => set_reg!(a, arith!(Arith::Mul, reg!(b), reg!(c))),
                    op::DIV => set_reg!(a, arith!(Arith::Div, reg!(b), reg!(c))),
                    op::MOD => set_reg!(a, arith!(Arith::Mod, reg!(b), reg!(c))),
                    op::POW => set_reg!(a, arith!(Arith::Pow, reg!(b), reg!(c))),
                    op::IDIV => set_reg!(a, arith!(Arith::Idiv, reg!(b), reg!(c))),
                    op::ADDK => set_reg!(a, arith!(Arith::Add, reg!(b), kst!(c))),
                    op::SUBK => set_reg!(a, arith!(Arith::Sub, reg!(b), kst!(c))),
                    op::MULK => set_reg!(a, arith!(Arith::Mul, reg!(b), kst!(c))),
                    op::DIVK => set_reg!(a, arith!(Arith::Div, reg!(b), kst!(c))),
                    op::MODK => set_reg!(a, arith!(Arith::Mod, reg!(b), kst!(c))),
                    op::IDIVK => set_reg!(a, arith!(Arith::Idiv, reg!(b), kst!(c))),
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
                            _ => self.arith(Arith::Pow, x, y)?,
                        };
                        set_reg!(a, v);
                    }
                    op::SUBRK => set_reg!(a, arith!(Arith::Sub, kst!(b), reg!(c))),
                    op::DIVRK => set_reg!(a, arith!(Arith::Div, kst!(b), reg!(c))),
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
                        self.concat(total, base.saturating_add(usize::from(c)))?;
                        set_reg!(a, reg!(b));
                        self.safepoint();
                    }
                    op::NOT => set_reg!(a, Value::Boolean(!reg!(b).truthy())),
                    op::MINUS => {
                        let x = reg!(b);
                        let v = match x {
                            Value::Number(n) => Value::Number(-n),
                            _ => self.arith(Arith::Unm, x, x)?,
                        };
                        set_reg!(a, v);
                    }
                    op::LENGTH => {
                        let v = self.length(reg!(b))?;
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
                        let copy = self.heap.table(template).clone();
                        let t = self.heap.new_table(copy)?;
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
                            let last = start.saturating_add(count);
                            if last > self.heap.table(t).array.len() {
                                self.resize_array(t, last)?;
                            }
                            for i in 0..count {
                                let v = self.get(first.saturating_add(i));
                                if let Some(slot) = self
                                    .heap
                                    .table_mut(t)
                                    .array
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
                        self.interrupt()?;
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
                            let mt = match ra {
                                Value::Table(t) => self.heap.table(t).meta,
                                _ => None,
                            };
                            if let Some(f) = self.fast_tm(mt, crate::vm::Tm::Iter) {
                                let slot = base.saturating_add(usize::from(a));
                                set_reg!(usize::from(a).saturating_add(1), ra);
                                set_reg!(a, f);
                                self.top = slot.saturating_add(2);
                                self.call_value(slot, 1, 3)?;
                                self.top = ftop;
                                if reg!(a).is_nil() {
                                    return Err(self.type_error(Value::Nil, "call"));
                                }
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
                        self.interrupt()?;
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
                            if vars < 0 && table.array.get(index).is_none_or(|v| v.is_nil()) {
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
                            self.ensure_stack(
                                call.saturating_add(3).saturating_add(crate::vm::MIN_STACK),
                            )?;
                            self.top = call.saturating_add(3);
                            self.call_value(call, 2, (aux & 0xff) as i32)?;
                            self.top = ftop;
                            let first = reg!(ra.saturating_add(3));
                            set_reg!(ra.saturating_add(2), first);
                            pc = if first.is_nil() {
                                pc.saturating_add(1)
                            } else {
                                jump(pc, insn_d(insn))
                            };
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
                            self.ensure_stack(
                                ra.saturating_add(n).saturating_add(crate::vm::MIN_STACK),
                            )?;
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
        let before = self.heap.table(table).capacity_bytes();
        self.heap.check_alloc(
            size.saturating_sub(self.heap.table(table).array.len())
                .saturating_mul(16),
        )?;
        let result = self.heap.table_mut(table).resize_array(size);
        result.map_err(|e| self.table_error(e))?;
        let after = self.heap.table(table).capacity_bytes();
        self.heap.charge(after.saturating_sub(before))
    }
}
