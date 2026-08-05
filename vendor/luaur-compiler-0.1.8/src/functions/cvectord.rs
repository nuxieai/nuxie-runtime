use crate::enums::type_constant_folding::Type;
use crate::records::constant::{Constant, ConstantData};

pub(crate) fn cvectord(x: f64, y: f64, z: f64, w: f64) -> Constant {
    Constant {
        r#type: Type::Type_Vectord,
        string_length: 0,
        data: ConstantData {
            value_vectord: [x, y, z, w],
        },
    }
}
