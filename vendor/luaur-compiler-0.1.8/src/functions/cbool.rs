use crate::enums::type_constant_folding::Type;
use crate::records::constant::Constant;

pub(crate) fn cbool(v: bool) -> Constant {
    let mut res = Constant {
        r#type: Type::Type_Boolean,
        string_length: 0,
        data: unsafe { core::mem::zeroed() },
    };

    unsafe {
        res.data.value_boolean = v;
    }

    res
}
