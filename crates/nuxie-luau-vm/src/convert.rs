//! Conversions between Rust values and [`Value`] (phase 2 lane 6), in
//! luaur-rt's (mlua's) shape: [`IntoLua`] / [`FromLua`] for one value,
//! [`IntoLuaMulti`] / [`FromLuaMulti`] for argument and result lists
//! (tuples up to eight, [`MultiValue`], or one value).
//!
//! A failed conversion is [`Error::Conversion`]. Inside a typed host
//! function (`Vm::create_function`) it becomes C++'s `luaL_typeerror` for
//! that argument, `invalid argument #n to 'f' (number expected, got
//! string)`, so a host function's argument errors read as the Rive C++
//! library's do (D8).
//!
//! Number conversions follow `lua_tonumberx`: numbers, integers and
//! numeric strings convert. Strings follow `lua_tolstring`: numbers convert
//! with `%.14g` (seam row R31).

use crate::error::Error;
use crate::handle::{
    AnyUserData, Buffer, Function, LightUserData, LuaString, MultiValue, Table, Thread, Value,
};
use crate::host::HostValue;
use crate::compat::Vm;

/// A Rust value that becomes one Lua value.
pub trait IntoLua {
    fn into_lua(self, vm: &mut Vm) -> Result<Value, Error>;
}

/// A Rust value made from one Lua value.
pub trait FromLua: Sized {
    fn from_lua(value: Value, vm: &mut Vm) -> Result<Self, Error>;
}

/// A Rust value that becomes a list of Lua values (arguments or results).
pub trait IntoLuaMulti {
    fn into_lua_multi(self, vm: &mut Vm) -> Result<MultiValue, Error>;
}

/// A Rust value made from a list of Lua values. A conversion error names
/// the 1-based position it failed at (`Error::Conversion::position`).
pub trait FromLuaMulti: Sized {
    fn from_lua_multi(values: MultiValue, vm: &mut Vm) -> Result<Self, Error>;
}

pub(crate) fn conversion(value: &Value, vm: &Vm, to: &'static str) -> Error {
    Error::Conversion {
        from: vm.value_type_name(value),
        to,
        position: None,
    }
}

// ---- one value ----------------------------------------------------------

impl IntoLua for Value {
    fn into_lua(self, _: &mut Vm) -> Result<Value, Error> {
        Ok(self)
    }
}

impl FromLua for Value {
    fn from_lua(value: Value, _: &mut Vm) -> Result<Value, Error> {
        Ok(value)
    }
}

impl IntoLua for &Value {
    fn into_lua(self, _: &mut Vm) -> Result<Value, Error> {
        Ok(self.clone())
    }
}

impl IntoLua for bool {
    fn into_lua(self, _: &mut Vm) -> Result<Value, Error> {
        Ok(Value::Boolean(self))
    }
}

/// `lua_toboolean`: everything but nil and false is true.
impl FromLua for bool {
    fn from_lua(value: Value, _: &mut Vm) -> Result<bool, Error> {
        Ok(!matches!(value, Value::Nil | Value::Boolean(false)))
    }
}

impl IntoLua for f64 {
    fn into_lua(self, _: &mut Vm) -> Result<Value, Error> {
        Ok(Value::Number(self))
    }
}

impl IntoLua for f32 {
    fn into_lua(self, _: &mut Vm) -> Result<Value, Error> {
        Ok(Value::Number(f64::from(self)))
    }
}

impl FromLua for f64 {
    fn from_lua(value: Value, vm: &mut Vm) -> Result<f64, Error> {
        vm.coerce_number(&value)?
            .ok_or_else(|| conversion(&value, vm, "number"))
    }
}

impl FromLua for f32 {
    fn from_lua(value: Value, vm: &mut Vm) -> Result<f32, Error> {
        f64::from_lua(value, vm).map(|n| n as f32)
    }
}

macro_rules! integer {
    ($($t:ty),*) => {$(
        impl IntoLua for $t {
            fn into_lua(self, _: &mut Vm) -> Result<Value, Error> {
                Ok(Value::Number(self as f64))
            }
        }

        /// An integer value converts exactly; a number is truncated toward
        /// zero (`luaL_checkinteger`'s cast). Out of range or not finite is
        /// a conversion error.
        impl FromLua for $t {
            fn from_lua(value: Value, vm: &mut Vm) -> Result<$t, Error> {
                if let Value::Integer(i) = value {
                    return <$t>::try_from(i).map_err(|_| conversion(&value, vm, stringify!($t)));
                }
                let t = f64::from_lua(value.clone(), vm)?.trunc();
                // Both bounds are powers of two, so exact in `f64`; `MAX as
                // f64` would round up to the excluded bound.
                let low = <$t>::MIN as f64;
                let high = if <$t>::MIN == 0 {
                    2f64.powi(<$t>::BITS as i32)
                } else {
                    -low
                };
                if t >= low && t < high {
                    Ok(t as $t)
                } else {
                    Err(conversion(&value, vm, stringify!($t)))
                }
            }
        }
    )*};
}

integer!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

impl IntoLua for &str {
    fn into_lua(self, vm: &mut Vm) -> Result<Value, Error> {
        vm.create_string(self).map(Value::String)
    }
}

impl IntoLua for String {
    fn into_lua(self, vm: &mut Vm) -> Result<Value, Error> {
        vm.create_string(self).map(Value::String)
    }
}

impl IntoLua for &[u8] {
    fn into_lua(self, vm: &mut Vm) -> Result<Value, Error> {
        vm.create_string(self).map(Value::String)
    }
}

/// `lua_tolstring`: a string, or a number rendered with `%.14g`; must be
/// UTF-8.
impl FromLua for String {
    fn from_lua(value: Value, vm: &mut Vm) -> Result<String, Error> {
        let Some(s) = vm.coerce_string(&value)? else {
            return Err(conversion(&value, vm, "string"));
        };
        s.to_str(vm)
    }
}

/// A vector (`LUA_VECTOR_SIZE` 3).
impl IntoLua for [f32; 3] {
    fn into_lua(self, _: &mut Vm) -> Result<Value, Error> {
        Ok(Value::Vector(self))
    }
}

impl FromLua for [f32; 3] {
    fn from_lua(value: Value, vm: &mut Vm) -> Result<[f32; 3], Error> {
        match value {
            Value::Vector(v) => Ok(v),
            other => Err(conversion(&other, vm, "vector")),
        }
    }
}

impl IntoLua for LightUserData {
    fn into_lua(self, _: &mut Vm) -> Result<Value, Error> {
        Ok(Value::LightUserData(self))
    }
}

impl FromLua for LightUserData {
    fn from_lua(value: Value, vm: &mut Vm) -> Result<LightUserData, Error> {
        match value {
            Value::LightUserData(l) => Ok(l),
            other => Err(conversion(&other, vm, "userdata")),
        }
    }
}

macro_rules! handle_conversions {
    ($($t:ident => $variant:ident),*) => {$(
        impl IntoLua for $t {
            fn into_lua(self, _: &mut Vm) -> Result<Value, Error> {
                Ok(Value::$variant(self))
            }
        }

        impl IntoLua for &$t {
            fn into_lua(self, _: &mut Vm) -> Result<Value, Error> {
                Ok(Value::$variant(self.clone()))
            }
        }

        impl FromLua for $t {
            fn from_lua(value: Value, vm: &mut Vm) -> Result<$t, Error> {
                match value {
                    Value::$variant(h) => Ok(h),
                    other => Err(conversion(&other, vm, $t::TYPE_NAME)),
                }
            }
        }
    )*};
}

handle_conversions!(
    Table => Table,
    Function => Function,
    Thread => Thread,
    AnyUserData => UserData,
    Buffer => Buffer
);

impl IntoLua for LuaString {
    fn into_lua(self, _: &mut Vm) -> Result<Value, Error> {
        Ok(Value::String(self))
    }
}

impl IntoLua for &LuaString {
    fn into_lua(self, _: &mut Vm) -> Result<Value, Error> {
        Ok(Value::String(self.clone()))
    }
}

/// A string, or a number converted (`lua_tolstring`).
impl FromLua for LuaString {
    fn from_lua(value: Value, vm: &mut Vm) -> Result<LuaString, Error> {
        vm.coerce_string(&value)?
            .ok_or_else(|| conversion(&value, vm, "string"))
    }
}

impl<T: IntoLua> IntoLua for Option<T> {
    fn into_lua(self, vm: &mut Vm) -> Result<Value, Error> {
        match self {
            Some(v) => v.into_lua(vm),
            None => Ok(Value::Nil),
        }
    }
}

/// `nil` is `None`; anything else converts.
impl<T: FromLua> FromLua for Option<T> {
    fn from_lua(value: Value, vm: &mut Vm) -> Result<Option<T>, Error> {
        match value {
            Value::Nil => Ok(None),
            other => T::from_lua(other, vm).map(Some),
        }
    }
}

/// A sequence table.
impl<T: IntoLua> IntoLua for Vec<T> {
    fn into_lua(self, vm: &mut Vm) -> Result<Value, Error> {
        let table = vm.create_table_with_capacity(self.len(), 0)?;
        for (i, v) in self.into_iter().enumerate() {
            let v = v.into_lua(vm)?;
            table.raw_set(vm, i.saturating_add(1) as f64, v)?;
        }
        Ok(Value::Table(table))
    }
}

/// A table's sequence part, `t[1]` up to the first nil (raw reads).
impl<T: FromLua> FromLua for Vec<T> {
    fn from_lua(value: Value, vm: &mut Vm) -> Result<Vec<T>, Error> {
        match value {
            Value::Table(t) => t.sequence_values(vm),
            other => Err(conversion(&other, vm, "table")),
        }
    }
}

/// The plain-data value of the pre-lane-6 host API.
impl IntoLua for HostValue {
    fn into_lua(self, vm: &mut Vm) -> Result<Value, Error> {
        Ok(match self {
            HostValue::Nil | HostValue::Object(_) => Value::Nil,
            HostValue::Boolean(b) => Value::Boolean(b),
            HostValue::Number(n) => Value::Number(n),
            HostValue::Vector(v) => Value::Vector(v),
            HostValue::String(bytes) => Value::String(vm.create_string(bytes)?),
        })
    }
}

// ---- lists --------------------------------------------------------------

impl IntoLuaMulti for MultiValue {
    fn into_lua_multi(self, _: &mut Vm) -> Result<MultiValue, Error> {
        Ok(self)
    }
}

impl FromLuaMulti for MultiValue {
    fn from_lua_multi(values: MultiValue, _: &mut Vm) -> Result<MultiValue, Error> {
        Ok(values)
    }
}

impl<T: IntoLua> IntoLuaMulti for T {
    fn into_lua_multi(self, vm: &mut Vm) -> Result<MultiValue, Error> {
        Ok(MultiValue(vec![self.into_lua(vm)?]))
    }
}

/// The first value (nil when there is none); the rest are ignored, as Lua
/// adjusts a call's results to one.
impl<T: FromLua> FromLuaMulti for T {
    fn from_lua_multi(values: MultiValue, vm: &mut Vm) -> Result<T, Error> {
        let first = values.0.into_iter().next().unwrap_or_default();
        T::from_lua(first, vm).map_err(|e| e.at_position(1))
    }
}

impl FromLuaMulti for () {
    fn from_lua_multi(_: MultiValue, _: &mut Vm) -> Result<(), Error> {
        Ok(())
    }
}

impl IntoLuaMulti for () {
    fn into_lua_multi(self, _: &mut Vm) -> Result<MultiValue, Error> {
        Ok(MultiValue::new())
    }
}

macro_rules! tuple {
    ($($name:ident $index:tt),+) => {
        impl<$($name: IntoLua),+> IntoLuaMulti for ($($name,)+) {
            fn into_lua_multi(self, vm: &mut Vm) -> Result<MultiValue, Error> {
                Ok(MultiValue(vec![$(self.$index.into_lua(vm)?),+]))
            }
        }

        impl<$($name: FromLua),+> FromLuaMulti for ($($name,)+) {
            fn from_lua_multi(values: MultiValue, vm: &mut Vm) -> Result<Self, Error> {
                let mut values = values.0.into_iter();
                Ok(($(
                    $name::from_lua(values.next().unwrap_or_default(), vm)
                        .map_err(|e| e.at_position($index + 1))?,
                )+))
            }
        }
    };
}

tuple!(A 0);
tuple!(A 0, B 1);
tuple!(A 0, B 1, C 2);
tuple!(A 0, B 1, C 2, D 3);
tuple!(A 0, B 1, C 2, D 3, E 4);
tuple!(A 0, B 1, C 2, D 3, E 4, F 5);
tuple!(A 0, B 1, C 2, D 3, E 4, F 5, G 6);
tuple!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7);
