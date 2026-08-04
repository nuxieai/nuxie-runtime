use crate::records::ast_array::AstArray;
use crate::records::ast_attr::{AstAttr, AstAttrType};

#[allow(non_snake_case)]
pub(crate) fn find_attribute_in_array(
    attributes: AstArray<*mut AstAttr>,
    attribute_type: AstAttrType,
) -> *mut AstAttr {
    for &attribute in attributes.as_slice() {
        if attribute.is_null() {
            continue;
        }

        unsafe {
            if (*attribute).r#type == attribute_type {
                return attribute;
            }
        }
    }

    core::ptr::null_mut()
}
