use crate::records::ast_node::AstNode;
use crate::records::printer::Printer;
use crate::rtti::CstNodeClass;

impl<'a> Printer<'a> {
    pub(crate) fn lookup_cst_node_impl<T: CstNodeClass>(&self, ast_node: *mut AstNode) -> *mut T {
        if let Some(&cst_node) = self.cst_node_map.find(&ast_node) {
            unsafe {
                if (*cst_node).class_index == T::CLASS_INDEX {
                    cst_node as *mut T
                } else {
                    core::ptr::null_mut()
                }
            }
        } else {
            core::ptr::null_mut()
        }
    }
}
