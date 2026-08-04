use crate::records::lexeme::Type;
use crate::records::parser::Parser;

impl Parser {
    pub fn expect_and_consume_char(&mut self, value: char, context: &str) -> bool {
        let type_ = Type(value as i32);
        self.expect_and_consume_type(type_, context)
    }
}
