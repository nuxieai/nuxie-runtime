use crate::records::compiler::Compiler;
use luaur_ast::records::ast_node::AstNode;

impl Compiler {
    pub fn set_debug_line_ast_node(&mut self, node: *mut AstNode) {
        if self.options.debug_level >= 1 {
            let line = unsafe { (*node).location.begin.line + 1 };
            unsafe { (*self.bytecode).set_debug_line(line as i32) };
        }
    }
}
