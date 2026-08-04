use crate::enums::type_constant_folding::Type;
use crate::records::constant::{Constant, ConstantData};

pub(crate) fn cvectorf(x: f32, y: f32, z: f32, w: f32) -> Constant {
    Constant {
        r#type: Type::Type_Vectorf,
        string_length: 0,
        data: ConstantData {
            value_vectorf: [x, y, z, w],
        },
    }
}
