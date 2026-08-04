use crate::records::lexeme::Type;
use crate::records::parser::Parser;
use luaur_common::macros::luau_noinline::LUAU_NOINLINE;

impl Parser {
    #[allow(non_snake_case)]
    pub(crate) fn expect_and_consume_fail_with_lookahead(
        &mut self,
        type_: Type,
        context: &str,
    ) -> bool {
        LUAU_NOINLINE! {
            fn inner(parser: &mut Parser, type_: Type, context: &str) -> bool {
                parser.expect_and_consume_fail(type_, context);

                // check if this is an extra token and the expected token is next
                if parser.lexer.lookahead().r#type == type_ {
                    // skip invalid and consume expected
                    parser.next_lexeme();
                    parser.next_lexeme();
                }

                false
            }
        }

        inner(self, type_, context)
    }
}
