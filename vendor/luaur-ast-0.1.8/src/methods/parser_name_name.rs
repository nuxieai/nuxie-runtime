use crate::records::ast_name::AstName;
use crate::records::location::Location;
use crate::records::name::Name;

impl Name {
    pub fn new(name: AstName, location: Location) -> Self {
        Self { name, location }
    }
}

#[allow(non_snake_case)]
pub fn parser_name_name(name: AstName, location: Location) -> Name {
    Name::new(name, location)
}
