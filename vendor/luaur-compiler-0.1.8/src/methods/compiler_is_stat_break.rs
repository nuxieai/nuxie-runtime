use crate::records::compiler::Compiler;
use luaur_ast::records::ast_node::AstNode;
use luaur_ast::records::ast_stat::AstStat;
use luaur_ast::records::ast_stat_block::AstStatBlock;
use luaur_ast::records::ast_stat_break::AstStatBreak;

impl Compiler {
    pub fn is_stat_break(&mut self, node: *mut AstStat) -> bool {
        unsafe {
            if node.is_null() {
                return false;
            }

            let stat_block = luaur_ast::rtti::ast_node_as::<AstStatBlock>(node as *mut AstNode);
            if !stat_block.is_null() {
                let stat_block = &*stat_block;

                stat_block.body.size == 1
                    && !luaur_ast::rtti::ast_node_as::<AstStatBreak>(
                        *stat_block.body.data.add(0) as *mut AstNode
                    )
                    .is_null()
            } else {
                !luaur_ast::rtti::ast_node_as::<AstStatBreak>(node as *mut AstNode).is_null()
            }
        }
    }
}
