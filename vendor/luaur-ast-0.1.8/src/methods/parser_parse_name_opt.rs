use crate::records::ast_name::AstName;
use crate::records::lexeme::Type;
use crate::records::name::Name;
use crate::records::parser::Parser;

impl Parser {
    pub fn parse_name_opt(&mut self, context: &str) -> Option<Name> {
        if self.lexer.current().r#type != Type::Name {
            self.report_name_error(context);

            return None;
        }

        let current = self.lexer.current();
        let result = Name {
            name: AstName {
                value: unsafe { current.data.name },
            },
            location: current.location,
        };

        self.next_lexeme();

        Some(result)
    }
}
