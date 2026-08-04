use crate::records::ast_stat_block::AstStatBlock;
use crate::records::parser::Parser;

impl Parser {
    pub fn parse_block(&mut self) -> *mut AstStatBlock {
        let locals_begin = self.save_locals();
        let result = self.parse_block_no_scope();
        self.restore_locals(locals_begin);
        result
    }
}
