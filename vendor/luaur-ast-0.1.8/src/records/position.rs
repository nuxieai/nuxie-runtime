//! Faithful port of Luau `Position` (`Ast/include/Luau/Location.h`).
//!
//! `unsigned int line, column`. C++ `operator<` orders by line then column,
//! which is exactly what the derived `Ord` does over the fields in declaration
//! order — so the six comparison operators (`==`, `!=`, `<`, `<=`, `>`, `>=`)
//! collapse into the derives here, and their separate method items stay stubs.
//! `Default` is `(0, 0)`, matching the `Position(0, 0)` used by `Location()`.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}

// `Position::missing()` (the `{UINT_MAX, UINT_MAX}` sentinel) is its own
// one-item-per-file method, `methods/position_missing.rs`.
