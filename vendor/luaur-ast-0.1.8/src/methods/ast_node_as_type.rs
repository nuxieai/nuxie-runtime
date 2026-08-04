use crate::records::ast_node::AstNode;
use crate::records::ast_type::AstType;
use crate::rtti::AstNodeClass;

impl AstNode {
    pub fn as_type(&self) -> *mut AstType {
        let is_type = self.class_index == crate::records::ast_type_error::AstTypeError::CLASS_INDEX
            || self.class_index == crate::records::ast_type_function::AstTypeFunction::CLASS_INDEX
            || self.class_index == crate::records::ast_type_group::AstTypeGroup::CLASS_INDEX
            || self.class_index
                == crate::records::ast_type_intersection::AstTypeIntersection::CLASS_INDEX
            || self.class_index == crate::records::ast_type_optional::AstTypeOptional::CLASS_INDEX
            || self.class_index
                == crate::records::ast_type_reference::AstTypeReference::CLASS_INDEX
            || self.class_index
                == crate::records::ast_type_singleton_bool::AstTypeSingletonBool::CLASS_INDEX
            || self.class_index
                == crate::records::ast_type_singleton_string::AstTypeSingletonString::CLASS_INDEX
            || self.class_index == crate::records::ast_type_table::AstTypeTable::CLASS_INDEX
            || self.class_index == crate::records::ast_type_typeof::AstTypeTypeof::CLASS_INDEX
            || self.class_index == crate::records::ast_type_union::AstTypeUnion::CLASS_INDEX;

        if is_type {
            self as *const AstNode as *mut AstType
        } else {
            core::ptr::null_mut()
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_node_as_type(node: *mut AstNode) -> *mut AstType {
    if node.is_null() {
        return core::ptr::null_mut();
    }
    unsafe { (*node).as_type() }
}
