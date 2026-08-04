use crate::records::cst_node::CstNode;
use crate::rtti::CstNodeClass;

impl CstNode {
    #[allow(non_snake_case)]
    pub fn as_item_mut<T: CstNodeClass>(&mut self) -> *mut T {
        if self.class_index == T::CLASS_INDEX {
            self as *mut CstNode as *mut T
        } else {
            core::ptr::null_mut()
        }
    }
}
