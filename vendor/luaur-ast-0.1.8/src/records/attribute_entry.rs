use crate::records::ast_attr::AstAttr;
use crate::type_aliases::attribute_arguments_validator::AttributeArgumentsValidator;

pub struct AttributeEntry {
    pub(crate) name: *const core::ffi::c_char,
    pub(crate) r#type: crate::records::ast_attr::AstAttrType,
    pub(crate) args_validator: Option<AttributeArgumentsValidator>,
}
