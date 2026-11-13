//! The value representation (plan decision D1).
//!
//! A [`Value`] is a 16-byte tagged enum: an 8-byte (or 12-byte, for vectors)
//! payload and a tag, the size of C++ Luau's `TValue`. Object references are
//! typed 32-bit indices into per-kind arenas (`heap.rs`), never pointers, so
//! the type is `Copy`, has no lifetime and holds no reference count.
//!
//! The type is crate-private. Host code sees the public `api::Value`, whose
//! objects are handles (`handle.rs`) rooted in the VM's handle table, or
//! stack slots through `HostCtx`, so every object reference the host can
//! reach stays rooted (D2).
//!
//! A new object kind is one more variant here, one more arena in
//! `heap.rs`, and the exhaustive matches the compiler then points at (lane
//! 5 added `Buffer` and `Integer` this way, lane 6 `Userdata`).

/// An interned string: its arena index and its Luau hash.
///
/// The hash is C++ `luaS_hash` over the string's bytes
/// ([`luau_string_hash`]), computed once by `Heap::intern` when it creates
/// the string. It is a fixed property of an immutable string, so it never
/// needs invalidating, and this handle is the only place it lives (`StrObj`
/// holds no copy): the table's string lookups read it here with no memory
/// access (docs/luau-safe-vm-pairs-order.md, 3.1). Equality is the index;
/// equal indices have equal hashes, so the derived traits agree with it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub(crate) struct StrId {
    pub(crate) index: u32,
    pub(crate) hash: u32,
}

impl StrId {
    /// The handle of the string `bytes` stored at arena slot `index`.
    #[inline]
    pub(crate) const fn new(index: u32, bytes: &[u8]) -> StrId {
        StrId {
            index,
            hash: luau_string_hash(bytes),
        }
    }
}

/// C++ Luau's `luaS_hash` (`VM/src/lstring.cpp`, no seed): the prefix in
/// 12-byte blocks with lookup3's mix while at least 32 bytes remain, then
/// Lua 5.1's hash over the rest from the last byte down. Blocks read native-endian words, as C++ does.
pub(crate) const fn luau_string_hash(bytes: &[u8]) -> u32 {
    let mut a: u32 = 0;
    let mut b: u32 = 0;
    // C++ truncates the length to 32 bits.
    let mut h: u32 = bytes.len() as u32;
    let mut rest = bytes;
    while rest.len() >= 32 {
        let [b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, tail @ ..] = rest else {
            break;
        };
        a = a.wrapping_add(u32::from_ne_bytes([*b0, *b1, *b2, *b3]));
        b = b.wrapping_add(u32::from_ne_bytes([*b4, *b5, *b6, *b7]));
        h = h.wrapping_add(u32::from_ne_bytes([*b8, *b9, *b10, *b11]));
        // `mix(14, 11, 25)`. C++'s `rol(x, s)` is `(x >> s) | (x << (32 - s))`,
        // a right rotation despite its name.
        a ^= h;
        a = a.wrapping_sub(h.rotate_right(14));
        b ^= a;
        b = b.wrapping_sub(a.rotate_right(11));
        h ^= b;
        h = h.wrapping_sub(b.rotate_right(25));
        rest = tail;
    }
    while let [head @ .., last] = rest {
        h ^= h
            .wrapping_shl(5)
            .wrapping_add(h.wrapping_shr(2))
            .wrapping_add(*last as u32);
        rest = head;
    }
    h
}
/// A table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub(crate) struct TableId(pub(crate) u32);
/// A function: a Lua closure or a native closure (see `heap::Function`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub(crate) struct FnId(pub(crate) u32);
/// A buffer (`LUA_TBUFFER`): a fixed-size mutable byte array.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub(crate) struct BufId(pub(crate) u32);
/// A full userdata (`LUA_TUSERDATA`): a host value owned by the VM.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub(crate) struct UdId(pub(crate) u32);
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
    Buffer(BufId),
    /// A full userdata (lane 6, `userdata.rs`).
    Userdata(UdId),
    /// Only ever stored in a closure's upvalue slots: a by-reference capture
    /// (C++ stores a `LUA_TUPVAL` there for the same purpose). Never visible
    /// to scripts.
    UpRef(UpvalId),
}

/// Every name `Value::type_name` gives, in `type_index` order.
pub(crate) const TYPE_NAMES: [&str; 12] = [
    "nil", "boolean", "number", "integer", "vector", "userdata", "string", "table", "function",
    "thread", "buffer", "upvalue",
];

// D1: the size claim, checked at compile time. A string handle is 8 bytes
// (index and hash) and fits the payload.
const _: () = assert!(core::mem::size_of::<Value>() == 16);
const _: () = assert!(core::mem::size_of::<StrId>() == 8);

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
        TYPE_NAMES.get(self.type_index()).copied().unwrap_or("nil")
    }

    /// The index of `type_name` in [`TYPE_NAMES`] (`Core::type_name_str`
    /// keeps them interned).
    #[inline]
    pub(crate) fn type_index(self) -> usize {
        match self {
            Value::Nil => 0,
            Value::Boolean(_) => 1,
            Value::Number(_) => 2,
            Value::Integer(_) => 3,
            Value::Vector(_) => 4,
            Value::LightUserdata(..) | Value::Userdata(_) => 5,
            Value::String(_) => 6,
            Value::Table(_) => 7,
            Value::Function(_) => 8,
            Value::Thread(_) => 9,
            Value::Buffer(_) => 10,
            Value::UpRef(_) => 11,
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
            (Value::Buffer(a), Value::Buffer(b)) => a == b,
            (Value::Userdata(a), Value::Userdata(b)) => a == b,
            (Value::UpRef(a), Value::UpRef(b)) => a == b,
            _ => false,
        }
    }

    #[inline]
    pub(crate) fn same_type(self, other: Value) -> bool {
        core::mem::discriminant(&self) == core::mem::discriminant(&other)
    }
}

#[cfg(test)]
mod tests {
    use super::luau_string_hash;

    /// The bytes `(i * 7 + 3) as u8` for `i` in `0..n`.
    fn pattern(n: usize) -> Vec<u8> {
        (0..n).map(|i| (i.wrapping_mul(7).wrapping_add(3)) as u8).collect()
    }

    /// Expected values printed by the vendored `luaS_hash` for the same
    /// bytes: both branches (under 32 bytes, and the 12-byte blocks).
    #[test]
    fn string_hash_matches_the_vendored_vm() {
        let named: [(&[u8], u32); 5] = [
            (b"", 0x0000_0000),
            (b"a", 0x0000_0080),
            (b"id", 0x0000_15f4),
            (b"defaultStateMachineId", 0x298f_3bd0),
            (b"not enough memory", 0x7c72_8eaf),
        ];
        for (bytes, hash) in named {
            assert_eq!(luau_string_hash(bytes), hash, "{bytes:?}");
        }
        let lengths: [(usize, u32); 8] = [
            (31, 0x17a7_69f5),
            (32, 0xfd1a_b52b),
            (33, 0x3677_cfe0),
            (43, 0xa3cb_065e),
            (44, 0x7e74_882f),
            (45, 0xe05f_5485),
            (100, 0x90f8_ba84),
            (200, 0x0047_0b3d),
        ];
        for (n, hash) in lengths {
            assert_eq!(luau_string_hash(&pattern(n)), hash, "length {n}");
        }
    }
}
