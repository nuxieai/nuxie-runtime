use crate::enums::type_constant_folding::Type;
use crate::records::constant::Constant;
use crate::type_aliases::compile_constant::CompileConstant;

pub fn set_compile_constant_boolean(constant: CompileConstant, b: bool) {
    let target = constant as *mut Constant;

    unsafe {
        (*target).r#type = Type::Type_Boolean;
        (*target).data.value_boolean = b;
    }
}
