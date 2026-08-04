use crate::records::compile_error::CompileError;
use crate::records::compiler::Compiler;
use luaur_ast::records::ast_node::AstNode;
use luaur_ast::records::location::Location;

impl Compiler {
    pub fn patch_jump(&mut self, node: *mut AstNode, label: usize, target: usize) {
        let ok = unsafe { (*self.bytecode).patch_jump_d(label, target) };
        if !ok {
            let location = unsafe { (*node).location };
            CompileError::raise(
                &location,
                format_args!("Exceeded jump distance limit; simplify the code to compile"),
            );
        }
    }
}
