use crate::enums::lua_type::lua_Type;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! setvvalue {
    ($obj:expr, $x:expr, $y:expr, $z:expr, $w:expr) => {
        unsafe {
            let i_o: *mut TValue = $obj;
            // C stores v[0],v[1] in value.v and v[2] (plus v[3] for size 4) in
            // TValue::extra, accessed as one contiguous float run. Derive the float
            // pointer from the TValue base so its provenance spans value + extra
            // (indexing value.v[2..] directly is out-of-bounds UB).
            let i_v = i_o as *mut f32;
            *i_v.add(0) = $x as f32;
            *i_v.add(1) = $y as f32;
            *i_v.add(2) = $z as f32;
            if $crate::macros::lua_vector_size::LUA_VECTOR_SIZE == 4 {
                *i_v.add(3) = $w as f32;
            }
            (*i_o).tt = $crate::enums::lua_type::lua_Type::LUA_TVECTOR as i32;
        }
    };
}

pub use setvvalue;
