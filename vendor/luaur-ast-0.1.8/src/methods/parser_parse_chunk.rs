use crate::records::ast_stat_block::AstStatBlock;
use crate::records::lexeme::Type;
use crate::records::parser::Parser;

impl Parser {
    pub fn parse_chunk(&mut self) -> *mut AstStatBlock {
        let result = self.parse_block();

        if self.lexer.current().r#type != Type::Eof {
            self.expect_and_consume_fail(Type::Eof, "");
        }

        result
    }
}
