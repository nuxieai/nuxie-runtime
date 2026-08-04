use crate::functions::find_attribute_in_array::find_attribute_in_array;
use crate::records::ast_attr::{AstAttr, AstAttrType};
use crate::records::ast_type_function::AstTypeFunction;

impl AstTypeFunction {
    #[allow(non_snake_case)]
    pub fn get_attribute(&self, attribute_type: AstAttrType) -> *mut AstAttr {
        find_attribute_in_array(self.attributes, attribute_type)
    }
}
