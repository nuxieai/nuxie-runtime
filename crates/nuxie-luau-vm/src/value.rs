//! The value representation (plan decision D1).
//!
//! A [`Value`] is a 16-byte tagged enum: an 8-byte (or 12-byte, for vectors)
//! payload and a tag, the size of C++ Luau's `TValue`. Object references are
//! typed 32-bit indices into per-kind arenas (`heap.rs`), never pointers, so
//! the type is `Copy`, has no lifetime and holds no reference count.
//!
//! The type is crate-private. Host code never names a `Value`; it sees stack
//! slots (`host.rs`), which is how every object reference the host can reach
//! stays rooted (D2).
//!
//! Seams: lane 5 (libraries) adds `Buffer` and the `integer` type with their
//! arenas; lane 6 (embedding) adds full userdata. Each is one more variant
//! here, one more arena in `heap.rs`, and the exhaustive matches the
//! compiler then points at.

/// An interned string.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub(crate) struct StrId(pub(crate) u32);
/// A table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub(crate) struct TableId(pub(crate) u32);
/// A function: a Lua closure or a native closure (see `heap::Function`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub(crate) struct FnId(pub(crate) u32);
/// An upvalue captured by reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub(crate) struct UpvalId(pub(crate) u32);
/// A thread (coroutine); see `thread.rs`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub(crate) struct ThreadId(pub(crate) u32);
/// A function prototype loaded from bytecode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub(crate) struct ProtoId(pub(crate) u32);

/// The light userdata tag the generalized-iteration cursor uses (C++
/// `LU_TAG_ITERATOR`, which is `LUA_UTAG_LIMIT`). No other code creates a
/// light userdata with this tag, so a script cannot forge a cursor.
pub(crate) const LU_TAG_ITERATOR: u8 = 128;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) enum Value {
    #[default]
    Nil,
    Boolean(bool),
    Number(f64),
    /// The 64-bit integer type (`LUA_TINTEGER`), behind the fork's
    /// `LuauIntegerType2` flag: `1i` literals and the `integer` library.
    Integer(i64),
    /// `LUA_VECTOR_SIZE` is 3 in this profile.
    Vector([f32; 3]),
    /// A light userdata: an opaque word and a tag.
    LightUserdata(u64, u8),
    String(StrId),
    Table(TableId),
    Function(FnId),
    /// A coroutine (lane 4).
    Thread(ThreadId),
    /// Only ever stored in a closure's upvalue slots: a by-reference capture
    /// (C++ stores a `LUA_TUPVAL` there for the same purpose). Never visible
    /// to scripts.
    UpRef(UpvalId),
}

// D1: the size claim, checked at compile time.
const _: () = assert!(core::mem::size_of::<Value>() == 16);

impl Value {
    #[inline]
    pub(crate) fn is_nil(self) -> bool {
        matches!(self, Value::Nil)
    }

    /// Lua truthiness: everything but `nil` and `false`.
    #[inline]
    pub(crate) fn truthy(self) -> bool {
        !matches!(self, Value::Nil | Value::Boolean(false))
    }

    /// `lua_typename` of the value's basic type.
    pub(crate) fn type_name(self) -> &'static str {
        match self {
            Value::Nil => "nil",
            Value::Boolean(_) => "boolean",
            Value::Number(_) => "number",
            Value::Integer(_) => "integer",
            Value::Vector(_) => "vector",
            Value::LightUserdata(..) => "userdata",
            Value::String(_) => "string",
            Value::Table(_) => "table",
            Value::Function(_) => "function",
            Value::Thread(_) => "thread",
            Value::UpRef(_) => "upvalue",
        }
    }

    /// `luaO_rawequalObj`: equality without metamethods. Numbers compare as
    /// floats (so `0 == -0` and NaN differs from itself); strings are
    /// interned, so their ids compare.
    #[inline]
    pub(crate) fn raw_eq(self, other: Value) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            // The vendored `luaO_rawequalObj` compares integers through
            // `luai_inteq`, which takes doubles: two integers that round to
            // the same double are raw-equal there, so they are here.
            (Value::Integer(a), Value::Integer(b)) => a as f64 == b as f64,
            (Value::Vector(a), Value::Vector(b)) => a == b,
            (Value::LightUserdata(a, at), Value::LightUserdata(b, bt)) => a == b && at == bt,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Table(a), Value::Table(b)) => a == b,
            (Value::Function(a), Value::Function(b)) => a == b,
            (Value::Thread(a), Value::Thread(b)) => a == b,
            (Value::UpRef(a), Value::UpRef(b)) => a == b,
            _ => false,
        }
    }

    #[inline]
    pub(crate) fn same_type(self, other: Value) -> bool {
        core::mem::discriminant(&self) == core::mem::discriminant(&other)
    }
}
