use crate::records::lexeme::Lexeme;
use crate::records::lexeme::Type;
use crate::records::location::Location;
use crate::records::match_lexeme::MatchLexeme;
use crate::records::parser::Parser;
use crate::records::position::Position;
use luaur_common::macros::luau_noinline::LUAU_NOINLINE;

impl Parser {
    LUAU_NOINLINE! {
        pub(crate) fn expect_match_end_and_consume_fail_with_lookahead(
            &mut self,
            type_: Type,
            begin: &MatchLexeme,
        ) -> bool {
            if let Some(end_mismatch_suspect) = self.end_mismatch_suspect {
                if end_mismatch_suspect.type_ != Type::Eof && end_mismatch_suspect.position.line > begin.position.line {
                    let match_string = Lexeme::new(
                        Location::new(Position::new(0, 0), Position::new(0, 0)),
                        end_mismatch_suspect.type_,
                    )
                    .to_string();

                    let suggestion = format!(
                        "; did you forget to close {} at line {}?",
                        match_string,
                        end_mismatch_suspect.position.line + 1
                    );

                    self.expect_match_and_consume_fail(type_, begin, Some(&suggestion));
                } else {
                    self.expect_match_and_consume_fail(type_, begin, None);
                }
            } else {
                self.expect_match_and_consume_fail(type_, begin, None);
            }

            // check if this is an extra token and the expected token is next
            if self.lexer.lookahead().r#type == type_ {
                // skip invalid and consume expected
                self.next_lexeme();
                self.next_lexeme();

                return true;
            }

            false
        }
    }
}
