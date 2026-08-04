use crate::records::ast_node::AstNode;
use crate::rtti::AstNodeClass;

impl AstNode {
    pub fn as_item<T: AstNodeClass>(&self) -> *const T {
        if self.class_index == T::CLASS_INDEX {
            self as *const AstNode as *const T
        } else {
            core::ptr::null()
        }
    }
}
