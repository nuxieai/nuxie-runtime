use crate::functions::find_attribute_in_array::find_attribute_in_array;
use crate::records::ast_attr::{AstAttr, AstAttrType};
use crate::records::ast_stat_declare_function::AstStatDeclareFunction;

impl AstStatDeclareFunction {
    pub fn get_attribute(&self, attribute_type: AstAttrType) -> *mut AstAttr {
        find_attribute_in_array(self.attributes, attribute_type)
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_declare_function_get_attribute(
    this: &AstStatDeclareFunction,
    attribute_type: AstAttrType,
) -> *mut AstAttr {
    this.get_attribute(attribute_type)
}
