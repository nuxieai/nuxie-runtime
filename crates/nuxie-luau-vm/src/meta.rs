//! Operations with metamethod fallbacks (C++ `lvmutils.cpp`, `ltm.cpp`):
//! indexing, arithmetic, comparison, concatenation, length, `tostring`.
//!
//! Each operation has a step form (`*_step`) that does the work up to the
//! metamethod call and returns that call as a [`MetaCall`] instead of
//! making it. The interpreter pushes the call as a frame (`call.rs`); a
//! library continuation can request it. Code that needs the result in
//! place passes the step to the driver's `HostedVm::meta_now`, which makes
//! the call from Rust through `call_value` (bounded by
//! `Limits::max_reentry`).

use crate::error::VmResult;
use crate::num::{num2str, str2d};
use crate::value::{StrId, TableId, Value};
use crate::vm::{Tm, Core};

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

/// A metamethod call an operation needs before it can finish.
#[derive(Clone, Copy, Debug)]
pub(crate) struct MetaCall {
    pub(crate) f: Value,
    pub(crate) args: [Value; 3],
    pub(crate) nargs: usize,
    /// 1, or 0 for `__newindex`.
    pub(crate) nresults: i32,
    /// What the operation makes of the first result.
    pub(crate) post: Post,
}

impl MetaCall {
    fn new(f: Value, args: [Value; 3], nargs: usize, post: Post) -> MetaCall {
        MetaCall {
            f,
            args,
            nargs,
            nresults: 1,
            post,
        }
    }
}

/// How an operation turns a metamethod's first result into its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Post {
    /// The result as it is.
    Value,
    /// `Boolean(truthy)`: comparisons.
    Truthy,
    /// `Boolean(!truthy)`: `a <= b` as `not (b < a)`.
    Falsy,
    /// A number, or "'__len' must return a number".
    Len,
    /// A string (numbers converted), or "'__tostring' must return a string".
    ToString,
}

/// An operation's outcome: its value, or the metamethod call it needs.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Meta {
    Done(Value),
    Call(MetaCall),
}

/// One step of `luaV_concat`.
pub(crate) enum ConcatStep {
    Done,
    /// Call `call`; store its result at `slot`; continue with `total` and
    /// `last`.
    Call {
        call: MetaCall,
        slot: usize,
        total: usize,
        last: usize,
    },
}

/// C++ `luai_numidiv` on a vector component.
fn vidiv(a: f32, b: f32) -> f32 {
    (f64::from(a) / f64::from(b)).floor() as f32
}

impl Core {
    /// Applies `post` to a metamethod's first result.
    pub(crate) fn apply_post(&mut self, post: Post, v: Value) -> VmResult<Value> {
        match post {
            Post::Value => Ok(v),
            Post::Truthy => Ok(Value::Boolean(v.truthy())),
            Post::Falsy => Ok(Value::Boolean(!v.truthy())),
            Post::Len => {
                if matches!(v, Value::Number(_)) {
                    Ok(v)
                } else {
                    Err(self.runtime_error("'__len' must return a number"))
                }
            }
            Post::ToString => match self.coerce_to_str(v)? {
                Some(id) => Ok(Value::String(id)),
                None => Err(self.lib_error("'__tostring' must return a string")),
            },
        }
    }

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

    /// C++ `luaV_gettable`, up to its `__index` call.
    pub(crate) fn get_table_step(&mut self, object: Value, key: Value) -> VmResult<Meta> {
        let mut t = object;
        for _ in 0..self.limits.max_meta_chain {
            let tm = if let Value::Table(id) = t {
                let table = self.heap.table(id);
                let res = table.get(key);
                if !res.is_nil() {
                    return Ok(Meta::Done(res));
                }
                match self.fast_tm(table.meta, Tm::Index) {
                    None => return Ok(Meta::Done(Value::Nil)),
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
                return Ok(Meta::Call(MetaCall::new(
                    tm,
                    [t, key, Value::Nil],
                    2,
                    Post::Value,
                )));
            }
            t = tm;
        }
        Err(self.runtime_error("'__index' chain too long; possible loop"))
    }

    /// C++ `luaV_settable`, up to its `__newindex` call.
    pub(crate) fn set_table_step(
        &mut self,
        object: Value,
        key: Value,
        value: Value,
    ) -> VmResult<Meta> {
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
                        self.raw_set(id, key, value)?;
                        return Ok(Meta::Done(Value::Nil));
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
                return Ok(Meta::Call(MetaCall {
                    nresults: 0,
                    ..MetaCall::new(tm, [t, key, value], 3, Post::Value)
                }));
            }
            t = tm;
        }
        Err(self.runtime_error("'__newindex' chain too long; possible loop"))
    }

    // ---- arithmetic --------------------------------------------------

    /// C++ `luaV_doarith`, up to its metamethod call.
    pub(crate) fn arith_step(&mut self, op: Arith, b: Value, c: Value) -> VmResult<Meta> {
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
                    return Ok(Meta::Done(Value::Vector([x, y, z])));
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
                        return Ok(Meta::Done(Value::Vector([x, y, z])));
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
                        return Ok(Meta::Done(Value::Vector([x, y, z])));
                    }
                }
            }
            _ => {}
        }
        if let (Some(nb), Some(nc)) = (self.to_number(b), self.to_number(c)) {
            return Ok(Meta::Done(Value::Number(op.number(nb, nc))));
        }
        if let Some(call) = self.bin_tm(b, c, op.tm()) {
            return Ok(Meta::Call(call));
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

    /// The call `call_bin_tm` makes, if there is a metamethod.
    fn bin_tm(&self, a: Value, b: Value, tm: Tm) -> Option<MetaCall> {
        let mut f = self.tm_by_obj(a, tm);
        if f.is_nil() {
            f = self.tm_by_obj(b, tm);
        }
        if f.is_nil() {
            return None;
        }
        Some(MetaCall::new(f, [a, b, Value::Nil], 2, Post::Value))
    }

    // ---- comparison --------------------------------------------------

    fn order_error(&mut self, a: Value, b: Value, op: &str) -> crate::error::Throw {
        let t1 = self.obj_type_name(a);
        let t2 = self.obj_type_name(b);
        self.runtime_error(format!("attempt to compare {t1} {op} {t2}"))
    }

    /// C++ `call_orderTM`: `None` when the operands have no common
    /// metamethod for `tm`.
    fn order_tm(&self, a: Value, b: Value, tm: Tm, post: Post) -> Option<MetaCall> {
        let tm1 = self.tm_by_obj(a, tm);
        if tm1.is_nil() {
            return None;
        }
        let tm2 = self.tm_by_obj(b, tm);
        if !tm1.raw_eq(tm2) {
            return None;
        }
        Some(MetaCall::new(tm1, [a, b, Value::Nil], 2, post))
    }

    /// C++ `luaV_lessthan`, up to its `__lt` call; the value is a boolean.
    pub(crate) fn less_than_step(&mut self, l: Value, r: Value) -> VmResult<Meta> {
        match (l, r) {
            (Value::Number(a), Value::Number(b)) => Ok(Meta::Done(Value::Boolean(a < b))),
            (Value::String(a), Value::String(b)) => Ok(Meta::Done(Value::Boolean(
                self.heap.str_bytes(a) < self.heap.str_bytes(b),
            ))),
            _ if !l.same_type(r) => Err(self.order_error(l, r, "<")),
            _ => match self.order_tm(l, r, Tm::Lt, Post::Truthy) {
                Some(call) => Ok(Meta::Call(call)),
                None => Err(self.order_error(l, r, "<")),
            },
        }
    }

    /// C++ `luaV_lessequal` (`__le`, else `not (r < l)` through `__lt`), up
    /// to its metamethod call; the value is a boolean.
    pub(crate) fn less_equal_step(&mut self, l: Value, r: Value) -> VmResult<Meta> {
        match (l, r) {
            (Value::Number(a), Value::Number(b)) => Ok(Meta::Done(Value::Boolean(a <= b))),
            (Value::String(a), Value::String(b)) => Ok(Meta::Done(Value::Boolean(
                self.heap.str_bytes(a) <= self.heap.str_bytes(b),
            ))),
            _ if !l.same_type(r) => Err(self.order_error(l, r, "<=")),
            _ => {
                if let Some(call) = self.order_tm(l, r, Tm::Le, Post::Truthy) {
                    return Ok(Meta::Call(call));
                }
                match self.order_tm(r, l, Tm::Lt, Post::Falsy) {
                    Some(call) => Ok(Meta::Call(call)),
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

    /// C++ `luaV_equalobj`: raw equality, with `__eq` for tables (called even
    /// for the same table, as C++ does); the value is a boolean.
    pub(crate) fn equal_step(&self, a: Value, b: Value) -> Meta {
        match (a, b) {
            (Value::Table(x), Value::Table(y)) => {
                let mt1 = self.heap.table(x).meta;
                let mt2 = self.heap.table(y).meta;
                match self.comp_tm(mt1, mt2) {
                    None => Meta::Done(Value::Boolean(x == y)),
                    Some(tm) => Meta::Call(MetaCall::new(tm, [a, b, Value::Nil], 2, Post::Truthy)),
                }
            }
            // C++ `luaV_equalobj`'s `LUA_TUSERDATA` case: as for tables.
            (Value::Userdata(x), Value::Userdata(y)) => {
                let mt1 = self.heap.userdata(x).meta;
                let mt2 = self.heap.userdata(y).meta;
                match self.comp_tm(mt1, mt2) {
                    None => Meta::Done(Value::Boolean(x == y)),
                    Some(tm) => Meta::Call(MetaCall::new(tm, [a, b, Value::Nil], 2, Post::Truthy)),
                }
            }
            // The interpreter compares integers exactly (`luau_execute`'s
            // `LUA_TINTEGER` case); `rawequal` compares their doubles.
            (Value::Integer(x), Value::Integer(y)) => Meta::Done(Value::Boolean(x == y)),
            _ => Meta::Done(Value::Boolean(a.raw_eq(b))),
        }
    }

    // ---- length ------------------------------------------------------

    /// C++ `luaV_dolen`, up to its `__len` call.
    pub(crate) fn length_step(&mut self, value: Value) -> VmResult<Meta> {
        let tm = match value {
            Value::Table(id) => {
                let table = self.heap.table(id);
                match self.fast_tm(table.meta, Tm::Len) {
                    None => return Ok(Meta::Done(Value::Number(table.len() as f64))),
                    Some(tm) => tm,
                }
            }
            Value::String(id) => {
                return Ok(Meta::Done(Value::Number(
                    self.heap.str_bytes(id).len() as f64
                )));
            }
            _ => self.tm_by_obj(value, Tm::Len),
        };
        if tm.is_nil() {
            return Err(self.type_error(value, "get length of"));
        }
        Ok(Meta::Call(MetaCall::new(
            tm,
            [value, Value::Nil, Value::Nil],
            2,
            Post::Len,
        )))
    }

    // ---- concatenation -------------------------------------------------

    /// C++ `luaV_concat`: concatenates the `total` slots ending at `last`
    /// (inclusive) right to left, in place, up to its next `__concat` call;
    /// the result is in the first slot.
    pub(crate) fn concat_step(&mut self, total: usize, last: usize) -> VmResult<ConcatStep> {
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
                None => match self.bin_tm(a, b, Tm::Concat) {
                    Some(call) => {
                        // Two operands become one.
                        return Ok(ConcatStep::Call {
                            call,
                            slot: a_slot,
                            total: total.saturating_sub(1),
                            last: last.saturating_sub(1),
                        });
                    }
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
                        self.heap.check_alloc(length)?;
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
        Ok(ConcatStep::Done)
    }

    // ---- tostring ----------------------------------------------------

    /// C++ `luaL_tolstring` up to its `__tostring` call (the driver's
    /// `HostedVm::tostring_value` makes it); the value is a string.
    pub(crate) fn tostring_step(&mut self, value: Value) -> VmResult<Meta> {
        if let Some(mt) = self.metatable(value) {
            let f = self.heap.table(mt).get_str(self.tm(Tm::ToString));
            if !f.is_nil() {
                // `luaL_tolstring` calls `__tostring` with the object alone.
                return Ok(Meta::Call(MetaCall::new(
                    f,
                    [value, Value::Nil, Value::Nil],
                    1,
                    Post::ToString,
                )));
            }
        }
        self.tostring_plain(value)
            .map(|id| Meta::Done(Value::String(id)))
    }

    /// The default rendering, without `__tostring`.
    fn tostring_plain(&mut self, value: Value) -> VmResult<StrId> {
        let text = match value {
            Value::Nil => "nil".to_owned(),
            Value::Boolean(true) => "true".to_owned(),
            Value::Boolean(false) => "false".to_owned(),
            Value::Number(n) => num2str(n),
            // `luai_int2str`: plain decimal digits, no suffix.
            Value::Integer(i) => i.to_string(),
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
            Value::Thread(id) => (5 << 32) | u64::from(id.0),
            Value::Buffer(id) => (6 << 32) | u64::from(id.0),
            Value::Userdata(id) => (7 << 32) | u64::from(id.0),
            Value::LightUserdata(p, _) => p,
            _ => 0,
        }
    }
}
