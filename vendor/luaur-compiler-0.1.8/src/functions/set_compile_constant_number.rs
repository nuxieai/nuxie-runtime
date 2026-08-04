use crate::enums::type_constant_folding::Type;
use crate::records::constant::Constant;
use crate::type_aliases::compile_constant::CompileConstant;

pub fn set_compile_constant_number(constant: CompileConstant, n: f64) {
    let target = constant as *mut Constant;

    unsafe {
        (*target).r#type = Type::Type_Number;
        (*target).data.value_number = n;
    }
}
