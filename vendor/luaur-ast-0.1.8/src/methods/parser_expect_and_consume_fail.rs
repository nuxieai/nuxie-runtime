use crate::records::lexeme::{Lexeme, Type};
use crate::records::location::Location;
use crate::records::parser::Parser;
use crate::records::position::Position;
use luaur_common::macros::luau_noinline::LUAU_NOINLINE;

impl Parser {
    LUAU_NOINLINE! {
        pub fn expect_and_consume_fail(&mut self, type_: Type, context: &str) {
            let type_string = Lexeme::new(Location::new(Position::new(0, 0), Position::new(0, 0)), type_).to_string();
            let curr_lexeme_string = self.lexer.current().to_string();

            if !context.is_empty() {
                self.report(
                    self.lexer.current().location,
                    format_args!(
                        "Expected {} when parsing {}, got {}",
                        type_string, context, curr_lexeme_string
                    ),
                );
            } else {
                self.report(
                    self.lexer.current().location,
                    format_args!("Expected {}, got {}", type_string, curr_lexeme_string),
                );
            }
        }
    }
}
