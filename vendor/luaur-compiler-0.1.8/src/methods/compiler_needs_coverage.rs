use crate::records::compiler::Compiler;
use luaur_ast::records::ast_node::AstNode;

use luaur_ast::records::ast_stat_block::AstStatBlock;
use luaur_ast::records::ast_stat_type_alias::AstStatTypeAlias;

impl Compiler {
    pub fn needs_coverage(&mut self, node: *mut AstNode) -> bool {
        let is_stat_block = unsafe { (*node).is::<AstStatBlock>() };
        let is_stat_type_alias = unsafe { (*node).is::<AstStatTypeAlias>() };
        !(is_stat_block || is_stat_type_alias)
    }
}
