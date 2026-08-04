//! The [`MetaMethod`] enum. Mirrors `mlua::MetaMethod` (Luau-relevant subset).
//!
//! A metamethod name can be supplied to `add_meta_method` / `add_meta_field`
//! either as a `&str` (`"__add"`) or as a [`MetaMethod`] variant
//! ([`MetaMethod::Add`]). Both implement [`Into<String>`] via the
//! `From<MetaMethod> for String` impl below, so the userdata registrars accept
//! either spelling, exactly like mlua.

use std::fmt;

/// Kinds of metamethods that can be overridden on userdata.
///
/// Mirrors `mlua::MetaMethod`. The variant set is the Luau-relevant subset
/// (bitwise operators specific to Lua 5.3/5.4 are omitted; Luau does not use
/// them as metamethods).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MetaMethod {
    /// The `+` operator (`__add`).
    Add,
    /// The `-` operator (`__sub`).
    Sub,
    /// The `*` operator (`__mul`).
    Mul,
    /// The `/` operator (`__div`).
    Div,
    /// The `%` operator (`__mod`).
    Mod,
    /// The `^` operator (`__pow`).
    Pow,
    /// The unary minus operator (`__unm`).
    Unm,
    /// The floor-division `//` operator (`__idiv`).
    IDiv,
    /// The string concatenation operator `..` (`__concat`).
    Concat,
    /// The length operator `#` (`__len`).
    Len,
    /// The `==` operator (`__eq`).
    Eq,
    /// The `<` operator (`__lt`).
    Lt,
    /// The `<=` operator (`__le`).
    Le,
    /// Index access `obj[key]` (`__index`).
    Index,
    /// Index write access `obj[key] = value` (`__newindex`).
    NewIndex,
    /// The call operator `obj(...)` (`__call`).
    Call,
    /// The `__tostring` metamethod.
    ToString,
    /// The `__iter` metamethod (Luau).
    Iter,
    /// The `__type`/`__name` metafield.
    Type,
}

impl MetaMethod {
    /// The Lua metamethod name (e.g. `"__add"`). Mirrors `mlua::MetaMethod::name`.
    pub const fn name(self) -> &'static str {
        match self {
            MetaMethod::Add => "__add",
            MetaMethod::Sub => "__sub",
            MetaMethod::Mul => "__mul",
            MetaMethod::Div => "__div",
            MetaMethod::Mod => "__mod",
            MetaMethod::Pow => "__pow",
            MetaMethod::Unm => "__unm",
            MetaMethod::IDiv => "__idiv",
            MetaMethod::Concat => "__concat",
            MetaMethod::Len => "__len",
            MetaMethod::Eq => "__eq",
            MetaMethod::Lt => "__lt",
            MetaMethod::Le => "__le",
            MetaMethod::Index => "__index",
            MetaMethod::NewIndex => "__newindex",
            MetaMethod::Call => "__call",
            MetaMethod::ToString => "__tostring",
            MetaMethod::Iter => "__iter",
            MetaMethod::Type => "__type",
        }
    }
}

impl fmt::Display for MetaMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl From<MetaMethod> for String {
    fn from(mm: MetaMethod) -> String {
        mm.name().to_string()
    }
}

impl PartialEq<MetaMethod> for &str {
    fn eq(&self, other: &MetaMethod) -> bool {
        *self == other.name()
    }
}

impl PartialEq<MetaMethod> for String {
    fn eq(&self, other: &MetaMethod) -> bool {
        self == other.name()
    }
}
