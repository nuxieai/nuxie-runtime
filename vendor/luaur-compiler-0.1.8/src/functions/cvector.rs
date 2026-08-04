use crate::enums::type_constant_folding::Type;
use crate::records::constant::Constant;

pub(crate) fn cvector(x: f64, y: f64, z: f64, w: f64) -> Constant {
    let mut res = Constant {
        r#type: Type::Type_Vector,
        string_length: 0,
        data: unsafe { core::mem::zeroed() },
    };

    unsafe {
        res.data.value_vector[0] = x as f32;
        res.data.value_vector[1] = y as f32;
        res.data.value_vector[2] = z as f32;
        res.data.value_vector[3] = w as f32;
    }

    res
}
