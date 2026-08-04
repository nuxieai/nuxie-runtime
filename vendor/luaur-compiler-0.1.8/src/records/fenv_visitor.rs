use luaur_ast::records::ast_expr_global::AstExprGlobal;
use luaur_ast::records::ast_node::AstNode;
use luaur_ast::records::ast_visitor::AstVisitor;

#[derive(Debug)]
pub struct FenvVisitor<'a> {
    pub(crate) getfenv_used: &'a mut bool,
    pub(crate) setfenv_used: &'a mut bool,
}

impl<'a> AstVisitor for FenvVisitor<'a> {
    fn visit_expr_global(&mut self, node: *mut core::ffi::c_void) -> bool {
        let node = unsafe { luaur_ast::rtti::ast_node_as::<AstExprGlobal>(node as *mut AstNode) };
        if let Some(node) = unsafe { node.as_ref() } {
            let name = unsafe { core::ffi::CStr::from_ptr(node.name.value).to_string_lossy() };
            if name == "getfenv" {
                *self.getfenv_used = true;
            }
            if name == "setfenv" {
                *self.setfenv_used = true;
            }
        }
        false
    }
}
