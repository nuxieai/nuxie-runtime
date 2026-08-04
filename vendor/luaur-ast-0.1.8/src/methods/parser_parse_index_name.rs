use crate::records::ast_name::AstName;
use crate::records::lexeme::Type;
use crate::records::location::Location;
use crate::records::name::Name;
use crate::records::parser::Parser;
use crate::records::position::Position;

impl Parser {
    pub fn parse_index_name(&mut self, context: &str, previous: &Position) -> Name {
        if let Some(name) = self.parse_name_opt(context) {
            return name;
        }

        // If we have a reserved keyword next at the same line, assume it's an incomplete name
        let current = self.lexer.current();
        if current.r#type >= Type::Reserved_BEGIN
            && current.r#type < Type::Reserved_END
            && current.location.begin.line == previous.line
        {
            let result = Name {
                name: AstName {
                    value: unsafe { current.data.name },
                },
                location: current.location,
            };

            self.next_lexeme();

            return result;
        }

        let mut location = self.lexer.current().location;
        location.end = location.begin;

        Name {
            name: self.name_error,
            location,
        }
    }
}
