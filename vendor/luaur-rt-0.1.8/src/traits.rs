//! The conversion traits. Mirror `mlua::{IntoLua, FromLua, IntoLuaMulti,
//! FromLuaMulti}`.
//!
//! These are simplified relative to mlua: mlua's traits carry extra
//! stack-oriented methods (`push_into_stack`, `from_stack`, ...) for its FFI
//! fast paths. Since luaur is a pure-Rust engine we route everything through
//! the safe [`Value`] / [`MultiValue`] representation, so our traits only need
//! the value-level methods. The public method *names and shapes*
//! (`into_lua(self, &Lua)`, `from_lua(value, &Lua)`,
//! `into_lua_multi`, `from_lua_multi`) match mlua exactly.
//!
//! The hidden `from_stack_args` / `into_return_values` / `push_into_multi`
//! methods let the callback trampoline and argument lists skip the
//! intermediate [`MultiValue`] (a heap-allocated deque) for fixed-arity
//! conversions. Each one converts exactly the values the value-level method
//! would, in the same order, so the two paths cannot be told apart.

use crate::error::Result;
use crate::multi::MultiValue;
use crate::stack_slot::FrameSlot;
use crate::state::Lua;
use crate::sys::c_int;
use crate::value::Value;

/// Convert a Rust value into a single Lua [`Value`].
///
/// Mirrors `mlua::IntoLua`.
pub trait IntoLua: Sized {
    /// Perform the conversion.
    fn into_lua(self, lua: &Lua) -> Result<Value>;

    /// The same conversion as [`IntoLua::into_lua`], as a callback's return
    /// value (a new userdata is created on the stack, needing no handle).
    #[doc(hidden)]
    fn into_return_value(self, lua: &Lua) -> Result<ReturnValues> {
        Ok(ReturnValues::One(self.into_lua(lua)?))
    }
}

/// Convert a single Lua [`Value`] into a Rust value.
///
/// Mirrors `mlua::FromLua`.
pub trait FromLua: Sized {
    /// Perform the conversion.
    fn from_lua(value: Value, lua: &Lua) -> Result<Self>;

    /// Convert an argument at 1-based position `i`. The default forwards to
    /// [`FromLua::from_lua`]; specific impls can produce nicer messages.
    /// Mirrors `mlua::FromLua::from_lua_arg`.
    fn from_lua_arg(arg: Value, _i: usize, _to: Option<&str>, lua: &Lua) -> Result<Self> {
        Self::from_lua(arg, lua)
    }
}

/// The values a callback returns, before they are pushed for the VM.
///
/// Zero and one value need no [`MultiValue`]; the callback trampoline pushes
/// them after the callback's own locals (argument handles, borrows) are gone,
/// exactly where it used to push the [`MultiValue`].
#[doc(hidden)]
pub enum ReturnValues {
    /// No values.
    None,
    /// Exactly one value.
    One(Value),
    /// Any number of values.
    Many(MultiValue),
    /// Values a luaur-rt conversion already pushed onto the top of the
    /// stack, having made room for them.
    Pushed(PushedCount),
}

/// How many values a luaur-rt conversion pushed for a callback's return.
/// Only luaur-rt makes one, right after pushing; the trampoline also checks
/// the count against the stack before the VM reads the values.
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct PushedCount(c_int);

impl PushedCount {
    pub(crate) fn new(count: c_int) -> Self {
        Self(count)
    }

    pub(crate) fn get(self) -> c_int {
        self.0
    }
}

impl ReturnValues {
    /// The number of values.
    pub(crate) fn len(&self) -> usize {
        match self {
            ReturnValues::None => 0,
            ReturnValues::One(_) => 1,
            ReturnValues::Many(values) => values.len(),
            ReturnValues::Pushed(count) => count.get() as usize,
        }
    }
}

/// Convert a Rust value into a sequence of Lua values (multiple returns / args).
///
/// Mirrors `mlua::IntoLuaMulti`.
pub trait IntoLuaMulti: Sized {
    /// Perform the conversion.
    fn into_lua_multi(self, lua: &Lua) -> Result<MultiValue>;

    /// The same conversion as [`IntoLuaMulti::into_lua_multi`], for a
    /// callback's return values.
    #[doc(hidden)]
    fn into_return_values(self, lua: &Lua) -> Result<ReturnValues> {
        Ok(ReturnValues::Many(self.into_lua_multi(lua)?))
    }

    /// The same conversion as [`IntoLuaMulti::into_lua_multi`], appended to
    /// `out` (tuples build one list instead of one per element).
    #[doc(hidden)]
    fn push_into_multi(self, lua: &Lua, out: &mut MultiValue) -> Result<()> {
        out.extend(self.into_lua_multi(lua)?);
        Ok(())
    }

    /// The same conversion as [`IntoLuaMulti::into_lua_multi`], with each
    /// value pushed onto the stack as a call argument; returns the count.
    #[doc(hidden)]
    fn push_call_args(self, lua: &Lua) -> Result<c_int> {
        let values = self.into_lua_multi(lua)?;
        for value in values.iter() {
            push_call_arg(lua, value)?;
        }
        Ok(values.len() as c_int)
    }
}

/// Push one call argument, failing as [`Function::call`](crate::Function)
/// does when the stack cannot hold its arguments.
pub(crate) fn push_call_arg(lua: &Lua, value: &Value) -> Result<()> {
    // Room for this value plus the call machinery's slack slot.
    if unsafe { crate::sys::lua_checkstack(lua.state(), 2) } == 0 {
        return Err(crate::error::Error::RuntimeError(
            "stack overflow: too many arguments to function call".to_string(),
        ));
    }
    lua.push_value(value)
}

/// Convert a sequence of Lua values into a Rust value.
///
/// Mirrors `mlua::FromLuaMulti`.
pub trait FromLuaMulti: Sized {
    /// Perform the conversion.
    fn from_lua_multi(values: MultiValue, lua: &Lua) -> Result<Self>;

    /// Convert the stack values `args` names (the values
    /// [`FromLuaMulti::from_lua_multi`] would receive: a callback's
    /// arguments, or a call's results). The default collects them into a
    /// [`MultiValue`].
    #[doc(hidden)]
    fn from_stack_args(args: StackRange<'_>) -> Result<Self> {
        Self::from_lua_multi(args.collect()?, args.lua())
    }
}

/// A run of stack values luaur-rt hands to
/// [`FromLuaMulti::from_stack_args`]. Only luaur-rt makes one, for values of
/// the frame current when it does; every read checks that frame and the stack
/// top again, so a range kept past them fails instead of reading other slots.
#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct StackRange<'a> {
    lua: &'a Lua,
    start: FrameSlot,
    count: c_int,
}

impl<'a> StackRange<'a> {
    /// The `count` values from `start` (a positive index) of the frame
    /// current on `lua`'s state.
    pub(crate) fn new(lua: &'a Lua, start: c_int, count: c_int) -> Self {
        Self {
            lua,
            start: FrameSlot::current(lua.state(), start),
            count: count.max(0),
        }
    }

    pub(crate) fn lua(&self) -> &'a Lua {
        self.lua
    }

    pub(crate) fn len(&self) -> usize {
        self.count as usize
    }

    /// The stack index of value `i` (below [`StackRange::len`]), checked
    /// against the range's frame and the stack top.
    pub(crate) fn slot(&self, i: usize) -> Result<c_int> {
        let start = self.start.index()?;
        if i >= self.len() {
            return Err(crate::error::Error::runtime(
                "luaur-rt: stack range read past its end",
            ));
        }
        let slot = start + i as c_int;
        if slot > unsafe { crate::sys::lua_gettop(self.lua.state()) } {
            return Err(crate::error::Error::runtime(
                "luaur-rt: stack range read past the stack top",
            ));
        }
        Ok(slot)
    }

    /// Value `i` (0-based), or nil past the end, as
    /// [`FromLuaMulti::from_lua_multi`]'s `pop_front` would see it.
    pub(crate) fn value(&self, i: usize) -> Result<Value> {
        if i >= self.len() {
            return Ok(Value::Nil);
        }
        self.lua.value_from_stack(self.slot(i)?)
    }

    /// Value `i`'s slot, bound to the range's frame.
    pub(crate) fn frame_slot(&self, i: usize) -> FrameSlot {
        self.start.above(i.min(self.len()) as c_int)
    }

    /// The values after the first `n`.
    pub(crate) fn skip(&self, n: usize) -> Self {
        let n = n.min(self.len()) as c_int;
        Self {
            lua: self.lua,
            start: self.start.above(n),
            count: self.count - n,
        }
    }

    /// Every value, as a [`MultiValue`].
    pub(crate) fn collect(&self) -> Result<MultiValue> {
        let mut values = MultiValue::with_capacity(self.len());
        for i in 0..self.len() {
            values.push_back(self.value(i)?);
        }
        Ok(values)
    }
}

// Any single-value type is trivially a multi-value of length one.
impl<T: IntoLua> IntoLuaMulti for T {
    fn into_lua_multi(self, lua: &Lua) -> Result<MultiValue> {
        let mut m = MultiValue::with_capacity(1);
        m.push_back(self.into_lua(lua)?);
        Ok(m)
    }

    fn into_return_values(self, lua: &Lua) -> Result<ReturnValues> {
        self.into_return_value(lua)
    }

    fn push_into_multi(self, lua: &Lua, out: &mut MultiValue) -> Result<()> {
        out.push_back(self.into_lua(lua)?);
        Ok(())
    }

    fn push_call_args(self, lua: &Lua) -> Result<c_int> {
        push_call_arg(lua, &self.into_lua(lua)?)?;
        Ok(1)
    }
}

// And any single-value type can be parsed from the first of a multi-value
// (extra values are ignored, matching Lua's "take what you need" calling
// convention).
impl<T: FromLua> FromLuaMulti for T {
    fn from_lua_multi(mut values: MultiValue, lua: &Lua) -> Result<Self> {
        let v = values.pop_front().unwrap_or(Value::Nil);
        T::from_lua(v, lua)
    }

    fn from_stack_args(args: StackRange<'_>) -> Result<Self> {
        T::from_lua(args.value(0)?, args.lua())
    }
}
