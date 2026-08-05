use crate::enums::type_constant_folding::Type;
use crate::records::constant::Constant;
use crate::type_aliases::compile_constant::CompileConstant;

pub fn set_compile_constant_vectord(
    constant: CompileConstant,
    x: f64,
    y: f64,
    z: f64,
    w: f64,
) {
    let target = constant as *mut Constant;

    unsafe {
        (*target).r#type = Type::Type_Vectord;
        (*target).data.value_vectord = [x, y, z, w];
    }
}
