use crate::records::parse_error::ParseError;
use crate::records::parser::Parser;

impl Parser {
    pub fn increment_recursion_counter(&mut self, context: &str) {
        self.recursion_counter += 1;

        let limit = luaur_common::FInt::LuauRecursionLimit.get();

        if self.recursion_counter > limit as u32 {
            ParseError::raise(
                self.lexer.current().location,
                format_args!(
                    "Exceeded allowed recursion depth; simplify your {} to make the code compile",
                    context
                ),
            );
        }
    }
}

#[allow(non_snake_case)]
pub fn parser_increment_recursion_counter(parser: &mut Parser, context: &str) {
    parser.increment_recursion_counter(context);
}
