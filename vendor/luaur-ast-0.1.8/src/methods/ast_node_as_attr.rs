use crate::records::ast_attr::AstAttr;
use crate::records::ast_node::AstNode;

impl AstNode {
    #[inline]
    pub fn as_attr(&mut self) -> *mut AstAttr {
        core::ptr::null_mut()
    }
}
