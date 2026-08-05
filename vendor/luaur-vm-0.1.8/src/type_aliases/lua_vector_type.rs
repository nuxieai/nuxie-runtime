#[cfg(feature = "lua_vector_double")]
pub type LuaVectorType = f64;

#[cfg(not(feature = "lua_vector_double"))]
pub type LuaVectorType = f32;
