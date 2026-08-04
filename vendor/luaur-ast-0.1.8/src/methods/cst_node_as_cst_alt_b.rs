use crate::records::cst_node::CstNode;
use crate::rtti::CstNodeClass;

impl CstNode {
    #[allow(non_snake_case)]
    pub fn as_item<T: CstNodeClass>(&self) -> *const T {
        if self.class_index == T::CLASS_INDEX {
            self as *const CstNode as *const T
        } else {
            core::ptr::null()
        }
    }
}
