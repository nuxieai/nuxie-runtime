#[allow(non_upper_case_globals)]
pub const LUAU_ASSERTENABLED: bool = cfg!(any(debug_assertions, feature = "luau_assert"));
