use crate::records::ast_node::AstNode;
use crate::rtti::AstNodeClass;

impl AstNode {
    #[inline]
    pub fn is<T: AstNodeClass>(&self) -> bool {
        self.class_index == T::CLASS_INDEX
    }
}
