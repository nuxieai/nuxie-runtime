use crate::records::cst_node::CstNode;
use crate::rtti::CstNodeClass;

impl CstNode {
    pub fn is<T: CstNodeClass>(&self) -> bool {
        self.class_index == T::CLASS_INDEX
    }
}
