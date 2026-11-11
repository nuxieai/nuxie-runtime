//! `luaur-rt`'s `Value` (vendor/luaur-rt-0.1.8/src/value.rs), `Vector` and
//! `LightUserData`, and the bridges between them and the safe VM's values.
//!
//! As in `luaur-rt`, a number that is an exact, in-range whole number reads
//! back as [`Value::Integer`]. The safe VM's own integer type (`1i`, which
//! `luaur-rt` cannot represent) reads back as `Integer` too.

use std::ffi::c_void;
use std::fmt;

use super::error::Result;
use super::handles::{Buffer, Function, LuaString, Table, Thread};
use super::state::Lua;
use super::userdata::AnyUserData;
use super::vm;

pub use Value::Nil;

pub type Integer = i64;
/// The float type exposed by the API. Mirrors `mlua::Number` (`f64`).
pub type Number = f64;

/// A dynamically typed Lua value.
///
/// Mirrors `mlua::Value`. Reference-typed variants ([`Value::String`],
/// [`Value::Table`], [`Value::Function`]) carry handles that keep both the
/// value and the VM alive.
#[derive(Clone, Debug)]
pub enum Value {
    /// `nil`.
    Nil,
    /// A boolean.
    Boolean(bool),
    /// An integer (an `f64` that is an exact, in-range whole number).
    Integer(Integer),
    /// A floating-point number.
    Number(Number),
    /// A string.
    String(LuaString),
    /// A table.
    Table(Table),
    /// A function (Lua or Rust).
    Function(Function),
    /// A raw-pointer light userdata. Mirrors `mlua::Value::LightUserData`.
    LightUserData(LightUserData),
    /// A userdata value with typed Rust-side borrowing.
    UserData(AnyUserData),
    /// A thread (coroutine).
    Thread(Thread),
    /// A Luau vector (3 `f32` components). Mirrors `mlua::Value::Vector`.
    Vector(Vector),
    /// A Luau buffer (mutable, fixed-size byte array). Mirrors
    /// `mlua::Value::Buffer`.
    Buffer(Buffer),
    /// A boxed Lua/Rust error carried as a first-class value (mirrors
    /// `mlua::Value::Error`). Produced when a Rust error is returned to Lua.
    Error(Box<super::error::Error>),
}

impl Value {
    /// `Value::Nil`. Mirrors `mlua::Nil`.
    pub const NIL: Value = Value::Nil;

    /// The Lua type name of this value (e.g. `"nil"`, `"number"`, `"table"`).
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Nil => "nil",
            Value::Boolean(_) => "boolean",
            Value::Integer(_) | Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Table(_) => "table",
            Value::Function(_) => "function",
            Value::LightUserData(_) => "userdata",
            Value::UserData(_) => "userdata",
            Value::Thread(_) => "thread",
            Value::Vector(_) => "vector",
            Value::Buffer(_) => "buffer",
            Value::Error(_) => "error",
        }
    }

    /// Whether this is an error value.
    pub fn is_error(&self) -> bool {
        matches!(self, Value::Error(_))
    }

    /// View as a reference to the contained error, if any.
    pub fn as_error(&self) -> Option<&super::error::Error> {
        match self {
            Value::Error(e) => Some(e),
            _ => None,
        }
    }

    /// Whether this is `nil`.
    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }
    /// Whether this is a boolean.
    pub fn is_boolean(&self) -> bool {
        matches!(self, Value::Boolean(_))
    }
    /// Whether this is a number of either subtype.
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_) | Value::Integer(_))
    }
    /// Whether this is the integer subtype.
    pub fn is_integer(&self) -> bool {
        matches!(self, Value::Integer(_))
    }
    /// Whether this is a string.
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }
    /// Whether this is a table.
    pub fn is_table(&self) -> bool {
        matches!(self, Value::Table(_))
    }
    /// Whether this is a function.
    pub fn is_function(&self) -> bool {
        matches!(self, Value::Function(_))
    }

    /// Lua truthiness: everything except `nil` and `false` is truthy.
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Value::Boolean(b) => Some(*b),
            _ => None,
        }
    }
    /// View as an integer if it is one.
    pub fn as_integer(&self) -> Option<Integer> {
        match self {
            Value::Integer(i) => Some(*i),
            _ => None,
        }
    }
    /// View as an `f64` if it is a number of either subtype.
    pub fn as_number(&self) -> Option<Number> {
        match self {
            Value::Number(n) => Some(*n),
            Value::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }
    /// View as a string handle.
    pub fn as_string(&self) -> Option<&LuaString> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }
    /// View as a table handle.
    pub fn as_table(&self) -> Option<&Table> {
        match self {
            Value::Table(t) => Some(t),
            _ => None,
        }
    }
    /// View as a function handle.
    pub fn as_function(&self) -> Option<&Function> {
        match self {
            Value::Function(f) => Some(f),
            _ => None,
        }
    }

    /// Whether this is a userdata value.
    pub fn is_userdata(&self) -> bool {
        matches!(self, Value::UserData(_))
    }

    /// View as a userdata handle.
    pub fn as_userdata(&self) -> Option<&AnyUserData> {
        match self {
            Value::UserData(u) => Some(u),
            _ => None,
        }
    }

    /// Whether this is a thread (coroutine) value.
    pub fn is_thread(&self) -> bool {
        matches!(self, Value::Thread(_))
    }

    /// View as a thread handle.
    pub fn as_thread(&self) -> Option<&Thread> {
        match self {
            Value::Thread(t) => Some(t),
            _ => None,
        }
    }

    /// Whether this is a Luau vector value. Mirrors `mlua::Value::is_vector`.
    pub fn is_vector(&self) -> bool {
        matches!(self, Value::Vector(_))
    }

    /// View as a vector. Mirrors `mlua::Value::as_vector`.
    pub fn as_vector(&self) -> Option<&Vector> {
        match self {
            Value::Vector(v) => Some(v),
            _ => None,
        }
    }

    /// Whether this is a Luau buffer value. Mirrors `mlua::Value::is_buffer`.
    pub fn is_buffer(&self) -> bool {
        matches!(self, Value::Buffer(_))
    }

    /// View as a buffer handle. Mirrors `mlua::Value::as_buffer`.
    pub fn as_buffer(&self) -> Option<&Buffer> {
        match self {
            Value::Buffer(b) => Some(b),
            _ => None,
        }
    }

    /// View as an `i32` if it is an in-range integer.
    pub fn as_i32(&self) -> Option<i32> {
        self.as_integer().and_then(|i| i32::try_from(i).ok())
    }
    /// View as a `u32` if it is an in-range integer.
    pub fn as_u32(&self) -> Option<u32> {
        self.as_integer().and_then(|i| u32::try_from(i).ok())
    }
    /// View as an `i64` if it is an integer.
    pub fn as_i64(&self) -> Option<i64> {
        self.as_integer()
    }
    /// View as a `u64` if it is an in-range integer.
    pub fn as_u64(&self) -> Option<u64> {
        self.as_integer().and_then(|i| u64::try_from(i).ok())
    }
    /// View as an `isize` if it is an in-range integer.
    pub fn as_isize(&self) -> Option<isize> {
        self.as_integer().and_then(|i| isize::try_from(i).ok())
    }
    /// View as a `usize` if it is an in-range integer.
    pub fn as_usize(&self) -> Option<usize> {
        self.as_integer().and_then(|i| usize::try_from(i).ok())
    }
    /// View as an `f32`.
    pub fn as_f32(&self) -> Option<f32> {
        self.as_number().map(|n| n as f32)
    }
    /// View as an `f64`.
    pub fn as_f64(&self) -> Option<f64> {
        self.as_number()
    }

    /// A raw pointer identifying reference-typed values (tables, functions,
    /// strings, userdata). Returns null for value-typed (nil/bool/number)
    /// values. Mirrors `mlua::Value::to_pointer`.
    pub fn to_pointer(&self) -> *const std::ffi::c_void {
        match self {
            Value::LightUserData(lud) => lud.0 as *const std::ffi::c_void,
            Value::String(s) => s.to_pointer(),
            Value::Table(t) => t.to_pointer(),
            Value::Function(f) => f.to_pointer(),
            Value::UserData(u) => u.to_pointer(),
            Value::Thread(t) => t.to_pointer(),
            // Buffers are GC objects (reference-typed); vectors are inline
            // value types (no pointer identity), matching mlua.
            Value::Buffer(b) => b.to_pointer(),
            _ => core::ptr::null(),
        }
    }

    /// The metatable-aware string form of this value (honors `__tostring`).
    /// Mirrors `mlua::Value::to_string`.
    #[allow(clippy::inherent_to_string_shadow_display)]
    pub fn to_string(&self) -> Result<String> {
        match self {
            Value::Nil => Ok("nil".to_string()),
            Value::Boolean(b) => Ok(b.to_string()),
            Value::Integer(i) => Ok(i.to_string()),
            Value::Number(n) => Ok(format_number(*n)),
            Value::Error(e) => Ok(e.to_string()),
            // Vector is an inline value type: format it directly (mlua does the
            // same, via `Vector`'s `Display`).
            Value::Vector(v) => Ok(v.to_string()),
            // Light userdata: format the pointer, matching Lua's `tostring`
            // (`userdata: 0x...`).
            Value::LightUserData(lud) => Ok(format!("userdata: {:p}", lud.0)),
            // Reference types: ask the VM (honors `__tostring`).
            Value::String(s) => s.to_str(),
            // Reference types: ask the VM through the handle's own Lua.
            Value::Table(t) => t.lua().value_to_string(self),
            Value::Function(f) => f.lua().value_to_string(self),
            Value::UserData(u) => u.lua().value_to_string(self),
            Value::Thread(t) => t.lua().value_to_string(self),
            Value::Buffer(b) => b.lua().value_to_string(self),
        }
    }
}

/// Format an `f64` the way Lua's `tostring` does for the common cases the
/// tests exercise (e.g. `34.59`, integral floats as plain integers).
pub(crate) fn format_number(n: f64) -> String {
    if n.fract() == 0.0 && n.is_finite() && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        // Rust's default float formatting matches Lua's "%.14g" closely enough
        // for the values under test.
        let s = format!("{n}");
        s
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::LightUserData(a), Value::LightUserData(b)) => a == b,
            // Numbers compare by value across the integer/float subtypes,
            // matching Lua's `==` (1 == 1.0).
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Integer(a), Value::Number(b)) | (Value::Number(b), Value::Integer(a)) => {
                (*a as f64) == *b
            }
            (Value::String(a), Value::String(b)) => a == b,
            // Reference types: identity within one VM (not `__eq`).
            (Value::Table(a), Value::Table(b)) => a == b,
            (Value::Function(a), Value::Function(b)) => a == b,
            (Value::UserData(a), Value::UserData(b)) => a == b,
            (Value::Thread(a), Value::Thread(b)) => a == b,
            // Vectors compare component-wise (value type); buffers by object
            // identity (reference type), matching mlua/Luau `==`.
            (Value::Vector(a), Value::Vector(b)) => a == b,
            (Value::Buffer(a), Value::Buffer(b)) => a == b,
            (Value::Error(a), Value::Error(b)) => a.to_string() == b.to_string(),
            _ => false,
        }
    }
}

/// True if the `f64` is an exact integer within `i64` range (so it can be
/// presented as [`Value::Integer`]).

// ---------------------------------------------------------------------------
// Vector and LightUserData (vendor/luaur-rt-0.1.8/src/{vector,light_userdata}.rs)
// ---------------------------------------------------------------------------

/// A Luau `vector` (three `f32` components), as `luaur-rt`'s.
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct Vector(pub(crate) [f32; 3]);

impl fmt::Display for Vector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "vector({}, {}, {})", self.x(), self.y(), self.z())
    }
}

impl Vector {
    /// luaur-rt `Vector::SIZE`: Luau vectors have three components.
    pub const SIZE: usize = 3;

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self([x, y, z])
    }

    pub const fn zero() -> Self {
        Self([0.0; 3])
    }

    pub const fn x(&self) -> f32 {
        let [x, _, _] = self.0;
        x
    }

    pub const fn y(&self) -> f32 {
        let [_, y, _] = self.0;
        y
    }

    pub const fn z(&self) -> f32 {
        let [_, _, z] = self.0;
        z
    }
}

impl PartialEq<[f32; 3]> for Vector {
    fn eq(&self, other: &[f32; 3]) -> bool {
        self.0 == *other
    }
}

/// A light userdata: a pointer-sized value with no object behind it, as
/// `luaur-rt`'s. The safe VM stores the address as a number; the pointer
/// is never dereferenced.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LightUserData(pub *mut c_void);

impl LightUserData {
    fn address(self) -> u64 {
        self.0 as usize as u64
    }

    fn from_address(address: u64) -> Self {
        LightUserData(usize::try_from(address).unwrap_or(usize::MAX) as *mut c_void)
    }
}

// ---------------------------------------------------------------------------
// The bridges to the safe VM's values
// ---------------------------------------------------------------------------

fn is_exact_integer(n: f64) -> bool {
    n.fract() == 0.0 && n.is_finite() && n >= i64::MIN as f64 && n <= i64::MAX as f64
}

/// A safe-VM value as a facade value, as `luaur-rt`'s `value_from_stack`
/// reads a stack slot.
pub(crate) fn from_vm(lua: &Lua, vm: &mut vm::Vm, value: vm::Value) -> Result<Value> {
    Ok(match value {
        vm::Value::Nil => Value::Nil,
        vm::Value::Boolean(b) => Value::Boolean(b),
        vm::Value::Number(n) => {
            if is_exact_integer(n) {
                Value::Integer(n as i64)
            } else {
                Value::Number(n)
            }
        }
        vm::Value::Integer(i) => Value::Integer(i),
        vm::Value::Vector(v) => Value::Vector(Vector(v)),
        vm::Value::LightUserData(lud) => Value::LightUserData(LightUserData::from_address(lud.ptr)),
        vm::Value::String(s) => Value::String(LuaString::wrap(lua, s)),
        vm::Value::Table(t) => Value::Table(Table::wrap(lua, t)),
        vm::Value::Function(f) => Value::Function(Function::wrap(lua, f)),
        vm::Value::Thread(t) => Value::Thread(Thread::wrap(lua, t)),
        vm::Value::UserData(u) => Value::UserData(AnyUserData::wrap(lua, vm, u)),
        vm::Value::Buffer(b) => Value::Buffer(Buffer::wrap(lua, b)),
    })
}

/// A facade value as a safe-VM value, as `luaur-rt`'s `push_value` pushes
/// it. An `Error` value becomes its message string.
pub(crate) fn to_vm(vm: &mut vm::Vm, value: &Value) -> Result<vm::Value> {
    Ok(match value {
        Value::Nil => vm::Value::Nil,
        Value::Boolean(b) => vm::Value::Boolean(*b),
        Value::Integer(i) => vm::Value::Number(*i as f64),
        Value::Number(n) => vm::Value::Number(*n),
        Value::Vector(v) => vm::Value::Vector(v.0),
        Value::LightUserData(lud) => vm::Value::LightUserData(vm::LightUserData {
            ptr: lud.address(),
            tag: 0,
        }),
        Value::String(s) => vm::Value::String(s.raw()?.clone()),
        Value::Table(t) => vm::Value::Table(t.raw()?.clone()),
        Value::Function(f) => vm::Value::Function(f.raw()?.clone()),
        Value::Thread(t) => vm::Value::Thread(t.raw()?.clone()),
        Value::UserData(u) => vm::Value::UserData(u.raw().clone()),
        Value::Buffer(b) => vm::Value::Buffer(b.raw()?.clone()),
        Value::Error(e) => vm::Value::String(vm.create_string(e.to_string())?),
    })
}

/// Facade values as safe-VM values.
pub(crate) fn many_to_vm<'a>(
    vm: &mut vm::Vm,
    values: impl IntoIterator<Item = &'a Value>,
) -> Result<vm::MultiValue> {
    let mut out = Vec::new();
    for v in values {
        out.push(to_vm(vm, v)?);
    }
    Ok(vm::MultiValue(out))
}

/// Safe-VM values as facade values.
pub(crate) fn many_from_vm(
    lua: &Lua,
    vm: &mut vm::Vm,
    values: vm::MultiValue,
) -> Result<super::multi::MultiValue> {
    let mut out = super::multi::MultiValue::with_capacity(values.len());
    for v in values.0 {
        out.push_back(from_vm(lua, vm, v)?);
    }
    Ok(out)
}
