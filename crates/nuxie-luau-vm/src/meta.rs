//! Operations with metamethod fallbacks (C++ `lvmutils.cpp`, `ltm.cpp`):
//! indexing, arithmetic, comparison, concatenation, length, `tostring`.

use crate::error::VmResult;
use crate::num::{num2str, str2d};
use crate::value::{StrId, TableId, Value};
use crate::vm::{Tm, Vm};

/// An arithmetic operation (C++ `TMS` from `TM_ADD` to `TM_UNM`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Arith {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Unm,
    Idiv,
}

impl Arith {
    fn tm(self) -> Tm {
        match self {
            Arith::Add => Tm::Add,
            Arith::Sub => Tm::Sub,
            Arith::Mul => Tm::Mul,
            Arith::Div => Tm::Div,
            Arith::Mod => Tm::Mod,
            Arith::Pow => Tm::Pow,
            Arith::Unm => Tm::Unm,
            Arith::Idiv => Tm::Idiv,
        }
    }

    /// The metamethod name without `__`, as the error message prints it.
    fn opname(self) -> &'static str {
        match self {
            Arith::Add => "add",
            Arith::Sub => "sub",
            Arith::Mul => "mul",
            Arith::Div => "div",
            Arith::Mod => "mod",
            Arith::Pow => "pow",
            Arith::Unm => "unm",
            Arith::Idiv => "idiv",
        }
    }

    /// The operation on numbers (C++ `luai_num*`).
    #[inline]
    pub(crate) fn number(self, a: f64, b: f64) -> f64 {
        match self {
            Arith::Add => a + b,
            Arith::Sub => a - b,
            Arith::Mul => a * b,
            Arith::Div => a / b,
            Arith::Mod => a - (a / b).floor() * b,
            Arith::Pow => a.powf(b),
            Arith::Unm => -a,
            Arith::Idiv => (a / b).floor(),
        }
    }
}

/// C++ `luai_numidiv` on a vector component.
fn vidiv(a: f32, b: f32) -> f32 {
    (f64::from(a) / f64::from(b)).floor() as f32
}

impl Vm {
    // ---- coercions ---------------------------------------------------

    /// C++ `luaV_tonumber`: a number, or a string that reads as one.
    pub(crate) fn to_number(&self, value: Value) -> Option<f64> {
        match value {
            Value::Number(n) => Some(n),
            Value::String(id) => str2d(self.heap.str_bytes(id)),
            _ => None,
        }
    }

    /// C++ `luaV_tostring` on a value: a string, or a number rendered.
    pub(crate) fn coerce_to_str(&mut self, value: Value) -> VmResult<Option<StrId>> {
        match value {
            Value::String(id) => Ok(Some(id)),
            Value::Number(n) => Ok(Some(self.heap.intern(num2str(n).as_bytes())?)),
            _ => Ok(None),
        }
    }

    // ---- indexing ----------------------------------------------------

    /// C++ `luaV_gettable`.
    pub(crate) fn get_table(&mut self, object: Value, key: Value) -> VmResult<Value> {
        let mut t = object;
        for _ in 0..self.limits.max_meta_chain {
            let tm = if let Value::Table(id) = t {
                let table = self.heap.table(id);
                let res = table.get(key);
                if !res.is_nil() {
                    return Ok(res);
                }
                match self.fast_tm(table.meta, Tm::Index) {
                    None => return Ok(Value::Nil),
                    Some(tm) => tm,
                }
            } else {
                let tm = self.tm_by_obj(t, Tm::Index);
                if tm.is_nil() {
                    return Err(self.index_error(t, key));
                }
                tm
            };
            if let Value::Function(_) = tm {
                return self.call_tm_res(tm, t, key);
            }
            t = tm;
        }
        Err(self.runtime_error("'__index' chain too long; possible loop"))
    }

    /// C++ `luaV_settable`.
    pub(crate) fn set_table(&mut self, object: Value, key: Value, value: Value) -> VmResult<()> {
        let mut t = object;
        for _ in 0..self.limits.max_meta_chain {
            let tm = if let Value::Table(id) = t {
                let table = self.heap.table(id);
                let tm = if table.get(key).is_nil() {
                    self.fast_tm(table.meta, Tm::NewIndex)
                } else {
                    None
                };
                match tm {
                    None => {
                        if table.readonly {
                            return Err(self.runtime_error("attempt to modify a readonly table"));
                        }
                        return self.raw_set(id, key, value);
                    }
                    Some(tm) => tm,
                }
            } else {
                let tm = self.tm_by_obj(t, Tm::NewIndex);
                if tm.is_nil() {
                    return Err(self.index_error(t, key));
                }
                tm
            };
            if let Value::Function(_) = tm {
                return self.call_tm(tm, t, key, value);
            }
            t = tm;
        }
        Err(self.runtime_error("'__newindex' chain too long; possible loop"))
    }

    // ---- arithmetic --------------------------------------------------

    /// C++ `luaV_doarith`: vectors, numbers and numeric strings, then the
    /// metamethod of either operand.
    pub(crate) fn arith(&mut self, op: Arith, b: Value, c: Value) -> VmResult<Value> {
        match (b, c) {
            (Value::Vector(vb), Value::Vector(vc)) => {
                let f = |x: f32, y: f32| match op {
                    Arith::Add => Some(x + y),
                    Arith::Sub => Some(x - y),
                    Arith::Mul => Some(x * y),
                    Arith::Div => Some(x / y),
                    Arith::Idiv => Some(vidiv(x, y)),
                    Arith::Unm => Some(-x),
                    _ => None,
                };
                if let (Some(x), Some(y), Some(z)) =
                    (f(vb[0], vc[0]), f(vb[1], vc[1]), f(vb[2], vc[2]))
                {
                    return Ok(Value::Vector([x, y, z]));
                }
            }
            (Value::Vector(vb), _) => {
                if let Some(nc) = self.to_number(c) {
                    let nc = nc as f32;
                    let f = |x: f32| match op {
                        Arith::Mul => Some(x * nc),
                        Arith::Div => Some(x / nc),
                        Arith::Idiv => Some(vidiv(x, nc)),
                        _ => None,
                    };
                    if let (Some(x), Some(y), Some(z)) = (f(vb[0]), f(vb[1]), f(vb[2])) {
                        return Ok(Value::Vector([x, y, z]));
                    }
                }
            }
            (_, Value::Vector(vc)) => {
                if let Some(nb) = self.to_number(b) {
                    let nb = nb as f32;
                    let f = |y: f32| match op {
                        Arith::Mul => Some(nb * y),
                        Arith::Div => Some(nb / y),
                        Arith::Idiv => Some(vidiv(nb, y)),
                        _ => None,
                    };
                    if let (Some(x), Some(y), Some(z)) = (f(vc[0]), f(vc[1]), f(vc[2])) {
                        return Ok(Value::Vector([x, y, z]));
                    }
                }
            }
            _ => {}
        }
        if let (Some(nb), Some(nc)) = (self.to_number(b), self.to_number(c)) {
            return Ok(Value::Number(op.number(nb, nc)));
        }
        if let Some(result) = self.call_bin_tm(b, c, op.tm())? {
            return Ok(result);
        }
        let t1 = self.obj_type_name(b);
        let t2 = self.obj_type_name(c);
        let message = if t1 == t2 {
            format!("attempt to perform arithmetic ({}) on {t1}", op.opname())
        } else {
            format!(
                "attempt to perform arithmetic ({}) on {t1} and {t2}",
                op.opname()
            )
        };
        Err(self.runtime_error(message))
    }

    /// C++ `call_binTM`: the first operand's metamethod, else the second's.
    pub(crate) fn call_bin_tm(&mut self, a: Value, b: Value, tm: Tm) -> VmResult<Option<Value>> {
        let mut f = self.tm_by_obj(a, tm);
        if f.is_nil() {
            f = self.tm_by_obj(b, tm);
        }
        if f.is_nil() {
            return Ok(None);
        }
        self.call_tm_res(f, a, b).map(Some)
    }

    // ---- comparison --------------------------------------------------

    fn order_error(&mut self, a: Value, b: Value, op: &str) -> crate::error::Throw {
        let t1 = self.obj_type_name(a);
        let t2 = self.obj_type_name(b);
        self.runtime_error(format!("attempt to compare {t1} {op} {t2}"))
    }

    /// C++ `call_orderTM`: `None` when the operands have no common
    /// metamethod for `tm`.
    fn call_order_tm(&mut self, a: Value, b: Value, tm: Tm) -> VmResult<Option<bool>> {
        let tm1 = self.tm_by_obj(a, tm);
        if tm1.is_nil() {
            return Ok(None);
        }
        let tm2 = self.tm_by_obj(b, tm);
        if !tm1.raw_eq(tm2) {
            return Ok(None);
        }
        Ok(Some(self.call_tm_res(tm1, a, b)?.truthy()))
    }

    /// C++ `luaV_lessthan`.
    pub(crate) fn less_than(&mut self, l: Value, r: Value) -> VmResult<bool> {
        match (l, r) {
            (Value::Number(a), Value::Number(b)) => Ok(a < b),
            (Value::String(a), Value::String(b)) => {
                Ok(self.heap.str_bytes(a) < self.heap.str_bytes(b))
            }
            _ if !l.same_type(r) => Err(self.order_error(l, r, "<")),
            _ => match self.call_order_tm(l, r, Tm::Lt)? {
                Some(result) => Ok(result),
                None => Err(self.order_error(l, r, "<")),
            },
        }
    }

    /// C++ `luaV_lessequal`: `__le`, else `not (r < l)` through `__lt`.
    pub(crate) fn less_equal(&mut self, l: Value, r: Value) -> VmResult<bool> {
        match (l, r) {
            (Value::Number(a), Value::Number(b)) => Ok(a <= b),
            (Value::String(a), Value::String(b)) => {
                Ok(self.heap.str_bytes(a) <= self.heap.str_bytes(b))
            }
            _ if !l.same_type(r) => Err(self.order_error(l, r, "<=")),
            _ => {
                if let Some(result) = self.call_order_tm(l, r, Tm::Le)? {
                    return Ok(result);
                }
                match self.call_order_tm(r, l, Tm::Lt)? {
                    Some(result) => Ok(!result),
                    None => Err(self.order_error(l, r, "<=")),
                }
            }
        }
    }

    /// C++ `get_compTM`: the `__eq` both metatables agree on.
    fn comp_tm(&self, mt1: Option<TableId>, mt2: Option<TableId>) -> Option<Value> {
        let tm1 = self.fast_tm(mt1, Tm::Eq)?;
        if mt1 == mt2 {
            return Some(tm1);
        }
        let tm2 = self.fast_tm(mt2, Tm::Eq)?;
        if tm1.raw_eq(tm2) { Some(tm1) } else { None }
    }

    /// C++ `luaV_equalobj`: raw equality, with `__eq` for tables (called
    /// even for the same table, as C++ does).
    pub(crate) fn equal(&mut self, a: Value, b: Value) -> VmResult<bool> {
        match (a, b) {
            (Value::Table(x), Value::Table(y)) => {
                let mt1 = self.heap.table(x).meta;
                let mt2 = self.heap.table(y).meta;
                match self.comp_tm(mt1, mt2) {
                    None => Ok(x == y),
                    Some(tm) => Ok(self.call_tm_res(tm, a, b)?.truthy()),
                }
            }
            _ => Ok(a.raw_eq(b)),
        }
    }

    // ---- length ------------------------------------------------------

    /// C++ `luaV_dolen`.
    pub(crate) fn length(&mut self, value: Value) -> VmResult<Value> {
        let tm = match value {
            Value::Table(id) => {
                let table = self.heap.table(id);
                match self.fast_tm(table.meta, Tm::Len) {
                    None => return Ok(Value::Number(table.len() as f64)),
                    Some(tm) => tm,
                }
            }
            Value::String(id) => return Ok(Value::Number(self.heap.str_bytes(id).len() as f64)),
            _ => self.tm_by_obj(value, Tm::Len),
        };
        if tm.is_nil() {
            return Err(self.type_error(value, "get length of"));
        }
        let result = self.call_tm_res(tm, value, Value::Nil)?;
        if !matches!(result, Value::Number(_)) {
            return Err(self.runtime_error("'__len' must return a number"));
        }
        Ok(result)
    }

    // ---- concatenation -------------------------------------------------

    /// C++ `luaV_concat`: concatenates the `total` slots ending at `last`
    /// (inclusive) right to left, in place; the result is in the first.
    pub(crate) fn concat(&mut self, total: usize, last: usize) -> VmResult<()> {
        let mut total = total;
        let mut last = last;
        while total > 1 {
            let top = last.saturating_add(1);
            let a_slot = top.saturating_sub(2);
            let b_slot = top.saturating_sub(1);
            let a = self.get(a_slot);
            let b = self.get(b_slot);
            let mut n = 2usize;
            let a_ok = matches!(a, Value::String(_) | Value::Number(_));
            let b_str = if a_ok { self.coerce_to_str(b)? } else { None };
            match b_str {
                None => match self.call_bin_tm(a, b, Tm::Concat)? {
                    Some(result) => self.set(a_slot, result),
                    None => {
                        let t1 = self.obj_type_name(a);
                        let t2 = self.obj_type_name(b);
                        return Err(
                            self.runtime_error(format!("attempt to concatenate {t1} with {t2}"))
                        );
                    }
                },
                Some(b_id) => {
                    self.set(b_slot, Value::String(b_id));
                    if self.heap.str_bytes(b_id).is_empty() {
                        if let Some(a_id) = self.coerce_to_str(a)? {
                            self.set(a_slot, Value::String(a_id));
                        }
                    } else {
                        // Collect as many string-convertible operands as
                        // possible, right to left.
                        let mut length = self.heap.str_bytes(b_id).len();
                        n = 1;
                        while n < total {
                            let slot = top.saturating_sub(n.saturating_add(1));
                            let Some(id) = self.coerce_to_str(self.get(slot))? else {
                                break;
                            };
                            self.set(slot, Value::String(id));
                            let l = self.heap.str_bytes(id).len();
                            if l > self.limits.max_string_bytes.saturating_sub(length) {
                                return Err(self.runtime_error("string length overflow"));
                            }
                            length = length.saturating_add(l);
                            n = n.saturating_add(1);
                        }
                        let mut buffer = Vec::with_capacity(length);
                        for i in (1..=n).rev() {
                            if let Value::String(id) = self.get(top.saturating_sub(i)) {
                                buffer.extend_from_slice(self.heap.str_bytes(id));
                            }
                        }
                        let id = self.heap.intern(&buffer)?;
                        self.set(top.saturating_sub(n), Value::String(id));
                    }
                }
            }
            total = total.saturating_sub(n.saturating_sub(1));
            last = last.saturating_sub(n.saturating_sub(1));
        }
        Ok(())
    }

    // ---- tostring ----------------------------------------------------

    /// C++ `luaL_tolstring`: `__tostring`, else the default rendering.
    pub(crate) fn tostring_value(&mut self, value: Value) -> VmResult<StrId> {
        if let Some(mt) = self.metatable(value) {
            let f = self.heap.table(mt).get_str(self.tm(Tm::ToString));
            if !f.is_nil() {
                let result = self.call_tm_res(f, value, Value::Nil)?;
                return match self.coerce_to_str(result)? {
                    Some(id) => Ok(id),
                    None => Err(self.lib_error("'__tostring' must return a string")),
                };
            }
        }
        let text = match value {
            Value::Nil => "nil".to_owned(),
            Value::Boolean(true) => "true".to_owned(),
            Value::Boolean(false) => "false".to_owned(),
            Value::Number(n) => num2str(n),
            Value::Vector(v) => {
                format!(
                    "{}, {}, {}",
                    num2str(f64::from(v[0])),
                    num2str(f64::from(v[1])),
                    num2str(f64::from(v[2]))
                )
            }
            Value::String(id) => return Ok(id),
            other => format!(
                "{}: 0x{:016x}",
                other.type_name(),
                self.pseudo_address(other)
            ),
        };
        self.heap.intern(text.as_bytes())
    }

    /// A stable stand-in for C++'s encoded object pointer: the arena id with
    /// the kind in the high bits. Differs from C++ by design.
    pub(crate) fn pseudo_address(&self, value: Value) -> u64 {
        match value {
            Value::Table(id) => (1 << 32) | u64::from(id.0),
            Value::Function(id) => (2 << 32) | u64::from(id.0),
            Value::String(id) => (3 << 32) | u64::from(id.0),
            Value::UpRef(id) => (4 << 32) | u64::from(id.0),
            Value::LightUserdata(p, _) => p,
            _ => 0,
        }
    }
}
