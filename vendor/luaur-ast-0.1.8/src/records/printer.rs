use crate::records::position::Position;
use crate::records::writer::Writer;
use crate::type_aliases::cst_node_map::CstNodeMap;

pub struct Printer<'a> {
    pub(crate) write_types: bool,
    pub(crate) writer: &'a mut dyn Writer,
    pub(crate) cst_node_map: CstNodeMap,
}

impl<'a> Printer<'a> {
    pub(crate) fn lookup_cst_node<T: crate::rtti::CstNodeClass>(
        &self,
        ast_node: *mut crate::records::ast_node::AstNode,
    ) -> *mut T {
        if let Some(&cst_node) = self.cst_node_map.find(&ast_node) {
            return unsafe { crate::rtti::cst_node_as::<T>(cst_node) };
        }
        core::ptr::null_mut()
    }
}
