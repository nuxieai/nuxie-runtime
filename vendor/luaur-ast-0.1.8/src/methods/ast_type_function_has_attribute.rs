use crate::functions::has_attribute_in_array::has_attribute_in_array;
use crate::records::ast_attr::AstAttrType;
use crate::records::ast_type_function::AstTypeFunction;

impl AstTypeFunction {
    pub fn has_attribute(&self, attribute_type: AstAttrType) -> bool {
        has_attribute_in_array(self.attributes, attribute_type)
    }
}
