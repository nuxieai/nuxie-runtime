use crate::enums::lua_type::lua_Type;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! setvvalue {
    ($L:expr, $obj:expr, $x:expr, $y:expr, $z:expr, $w:expr) => {
        unsafe {
            let i_o: *mut TValue = $obj;
            #[cfg(feature = "lua_vector_double")]
            {
                let i_vec = $crate::functions::lua_vec_newvector::luaVec_newvector(
                    $L,
                    $x as f64,
                    $y as f64,
                    $z as f64,
                    if $crate::macros::lua_vector_size::LUA_VECTOR_SIZE == 4 {
                        $w as f64
                    } else {
                        0.0
                    },
                );
                (*i_o).value.gc = i_vec as *mut $crate::records::gc_object::GCObject;
                (*i_o).tt = $crate::enums::lua_type::lua_Type::LUA_TVECTOR as i32;
                $crate::macros::checkliveness::checkliveness!((*($L)).global, i_o);
            }
            #[cfg(not(feature = "lua_vector_double"))]
            {
                let _ = $L;
                // C stores v[0],v[1] in value.v and v[2] (plus v[3] for size 4) in
                // TValue::extra, accessed as one contiguous float run. Derive the float
                // pointer from the TValue base so its provenance spans value + extra.
                let i_v = i_o as *mut f32;
                *i_v.add(0) = $x as f32;
                *i_v.add(1) = $y as f32;
                *i_v.add(2) = $z as f32;
                if $crate::macros::lua_vector_size::LUA_VECTOR_SIZE == 4 {
                    *i_v.add(3) = $w as f32;
                } else {
                    let _ = $w;
                }
                (*i_o).tt = $crate::enums::lua_type::lua_Type::LUA_TVECTOR as i32;
            }
        }
    };
}

pub use setvvalue;
