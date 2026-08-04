use crate::enums::type_constant_folding::Type;
use crate::records::constant::Constant;

pub(crate) fn cnum(v: f64) -> Constant {
    let mut res = Constant {
        r#type: Type::Type_Number,
        string_length: 0,
        data: unsafe { core::mem::zeroed() },
    };

    unsafe {
        res.data.value_number = v;
    }

    res
}
