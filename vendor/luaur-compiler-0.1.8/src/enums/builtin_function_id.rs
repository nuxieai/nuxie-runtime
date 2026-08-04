//! Re-export: translations consistently reference the builtin-function enum
//! by its graph node name (`builtin_function_id`); the type itself is
//! `LuauBuiltinFunction` from `Common/include/Luau/Bytecode.h` (luau-common).

pub use luaur_common::enums::luau_builtin_function::*;
