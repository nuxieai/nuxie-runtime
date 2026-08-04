use crate::records::parser::Parser;
use luaur_common::macros::luau_noinline::LUAU_NOINLINE;

impl Parser {
    LUAU_NOINLINE! {
        pub fn report_name_error(&mut self, context: &str) {
            let location = self.lexer.current().location;
            let current_lexeme_string = self.lexer.current().to_string();

            if !context.is_empty() {
                self.report(
                    location,
                    format_args!(
                        "Expected identifier when parsing {}, got {}",
                        context, current_lexeme_string
                    ),
                );
            } else {
                self.report(
                    location,
                    format_args!("Expected identifier, got {}", current_lexeme_string),
                );
            }
        }
    }
}
