use crate::records::compiler::Compiler;
use luaur_ast::records::ast_node::AstNode;

impl Compiler {
    pub fn patch_jumps(&mut self, node: *mut AstNode, labels: &mut Vec<usize>, target: usize) {
        for l in labels.iter().copied() {
            self.patch_jump(node, l, target);
        }
    }
}
