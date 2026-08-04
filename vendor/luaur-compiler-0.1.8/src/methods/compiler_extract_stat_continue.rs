use luaur_ast::records::ast_node::AstNode;
use luaur_ast::records::ast_stat::AstStat;
use luaur_ast::records::ast_stat_block::AstStatBlock;
use luaur_ast::records::ast_stat_continue::AstStatContinue;

impl crate::records::compiler::Compiler {
    pub fn extract_stat_continue(&mut self, block: *mut AstStatBlock) -> *mut AstStatContinue {
        unsafe {
            let body = &(*block).body;
            if body.size == 1 {
                let stat_ptr = body.data.add(0);
                let stat_node = &mut **stat_ptr as *mut AstStat as *mut AstNode;
                let continue_ptr = luaur_ast::rtti::ast_node_as::<AstStatContinue>(stat_node);
                continue_ptr
            } else {
                core::ptr::null_mut()
            }
        }
    }
}
