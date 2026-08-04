use crate::functions::find_attribute_in_array::find_attribute_in_array;
use crate::records::ast_array::AstArray;
use crate::records::ast_attr::{AstAttr, AstAttrType};

#[allow(non_snake_case)]
pub(crate) fn has_attribute_in_array(
    attributes: AstArray<*mut AstAttr>,
    attribute_type: AstAttrType,
) -> bool {
    !find_attribute_in_array(attributes, attribute_type).is_null()
}
