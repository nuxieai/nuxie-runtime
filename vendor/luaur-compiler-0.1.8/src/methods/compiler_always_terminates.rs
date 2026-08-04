use crate::functions::always_terminates::always_terminates;
use crate::records::compiler::Compiler;
use luaur_ast::records::ast_stat::AstStat;

impl Compiler {
    pub fn always_terminates(&self, node: *mut AstStat) -> bool {
        always_terminates(&self.constants, node)
    }
}
