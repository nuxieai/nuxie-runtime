use crate::records::binding::Binding;
use crate::records::lexeme::Type;
use crate::records::parser::Parser;
use crate::records::position::Position;

impl Parser {
    pub fn parse_binding(&mut self, is_const: bool) -> Binding {
        let name = self.parse_name_opt("variable name");

        let name = name.unwrap_or_else(|| crate::records::name::Name {
            name: self.name_error,
            location: self.lexer.current().location,
        });

        let colon_position = if self.lexer.current().r#type == Type(':' as i32) {
            self.lexer.current().location.begin
        } else {
            Position::missing()
        };
        let annotation = self.parse_optional_type();

        if self.options.store_cst_data {
            Binding::new(name, annotation, colon_position, is_const)
        } else {
            Binding::new(name, annotation, Position::missing(), is_const)
        }
    }
}
