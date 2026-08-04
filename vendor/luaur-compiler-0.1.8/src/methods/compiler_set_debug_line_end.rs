use crate::records::compiler::Compiler;
use luaur_ast::records::ast_node::AstNode;

impl Compiler {
    pub fn set_debug_line_end(&mut self, node: *mut AstNode) {
        if self.options.debug_level >= 1 {
            let line = unsafe { (*node).location.end.line + 1 };
            unsafe { (*self.bytecode).set_debug_line(line as i32) };
        }
    }
}
