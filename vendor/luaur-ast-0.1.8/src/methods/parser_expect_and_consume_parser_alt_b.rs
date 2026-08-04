use crate::records::lexeme::Type;
use crate::records::parser::Parser;

impl Parser {
    pub fn expect_and_consume_type(&mut self, type_: Type, context: &str) -> bool {
        if self.lexer.current().r#type != type_ {
            self.expect_and_consume_fail_with_lookahead(type_, context);

            false
        } else {
            self.next_lexeme();

            true
        }
    }
}
