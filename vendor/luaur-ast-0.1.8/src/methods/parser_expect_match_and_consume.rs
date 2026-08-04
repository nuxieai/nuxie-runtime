use crate::records::lexeme::Type;
use crate::records::match_lexeme::MatchLexeme;
use crate::records::parser::Parser;

impl Parser {
    pub fn expect_match_and_consume(
        &mut self,
        value: char,
        begin: &MatchLexeme,
        search_for_missing: bool,
    ) -> bool {
        let r#type = Type(value as i32);

        if self.lexer.current().r#type != r#type {
            self.expect_match_and_consume_fail(r#type, begin, None);

            self.expect_match_and_consume_recover(value, begin, search_for_missing)
        } else {
            self.next_lexeme();

            true
        }
    }
}
