use crate::records::lexeme::Type;
use crate::records::match_lexeme::MatchLexeme;
use crate::records::parser::Parser;
use luaur_common::macros::luau_noinline::LUAU_NOINLINE;

impl Parser {
    #[allow(non_snake_case)]
    pub(crate) fn expect_match_and_consume_recover(
        &mut self,
        value: char,
        _begin: &MatchLexeme,
        search_for_missing: bool,
    ) -> bool {
        LUAU_NOINLINE! {
            fn inner(parser: &mut Parser, value: char, search_for_missing: bool) -> bool {
                let r#type = Type(value as i32);

                if search_for_missing {
                    // previous location is taken because 'current' lexeme is already the next token
                    let current_line = parser.lexer.previous_location().end.line;

                    // search to the end of the line for expected token
                    // we will also stop if we hit a token that can be handled by parsing function above the current one
                    let mut lexeme_type = parser.lexer.current().r#type;

                    while current_line == parser.lexer.current().location.begin.line
                        && lexeme_type != r#type
                        && parser.match_recovery_stop_on_token[lexeme_type.0 as usize] == 0
                    {
                        parser.next_lexeme();
                        lexeme_type = parser.lexer.current().r#type;
                    }

                    if lexeme_type == r#type {
                        parser.next_lexeme();

                        return true;
                    }
                } else {
                    // check if this is an extra token and the expected token is next
                    if parser.lexer.lookahead().r#type == r#type {
                        // skip invalid and consume expected
                        parser.next_lexeme();
                        parser.next_lexeme();

                        return true;
                    }
                }

                false
            }
        }

        inner(self, value, search_for_missing)
    }
}
