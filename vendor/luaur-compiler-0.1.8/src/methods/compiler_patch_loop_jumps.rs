use crate::enums::type_compiler::Type;
use crate::records::compiler::Compiler;
use crate::records::loop_jump::LoopJump;
use luaur_ast::records::ast_node::AstNode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Compiler {
    pub fn patch_loop_jumps(
        &mut self,
        node: *mut AstNode,
        old_jumps: usize,
        end_label: usize,
        cont_label: usize,
    ) {
        LUAU_ASSERT!(old_jumps <= self.loop_jumps.len());

        for i in old_jumps..self.loop_jumps.len() {
            let lj: &LoopJump = &self.loop_jumps[i];

            match lj.r#type {
                Type::Break => {
                    self.patch_jump(node, lj.label, end_label);
                }
                Type::Continue => {
                    self.patch_jump(node, lj.label, cont_label);
                }
                _ => {
                    LUAU_ASSERT!(!true);
                }
            }
        }
    }
}
