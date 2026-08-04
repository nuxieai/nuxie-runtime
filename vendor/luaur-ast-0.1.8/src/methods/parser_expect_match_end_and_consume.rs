use crate::records::lexeme::Type;
use crate::records::match_lexeme::MatchLexeme;
use crate::records::parser::Parser;
use luaur_common::macros::luau_noinline::LUAU_NOINLINE;

impl Parser {
    LUAU_NOINLINE! {
        pub(crate) fn expect_match_end_and_consume(
            &mut self,
            type_: Type,
            begin: &MatchLexeme,
        ) -> bool {
            if self.lexer.current().r#type != type_ {
                return self.expect_match_end_and_consume_fail_with_lookahead(type_, begin);
            }

            // If the token matches on a different line and a different column, it suggests misleading indentation
            // This can be used to pinpoint the problem location for a possible future *actual* mismatch
            let current_loc = self.lexer.current().location;
            if current_loc.begin.line != begin.position.line
                && current_loc.begin.column != begin.position.column
            {
                if let Some(suspect) = self.end_mismatch_suspect {
                    if suspect.position.line < begin.position.line {
                        self.end_mismatch_suspect = Some(*begin);
                    }
                } else {
                    self.end_mismatch_suspect = Some(*begin);
                }
            }

            self.next_lexeme();

            true
        }
    }
}
