use crate::records::ast_expr_function::AstExprFunction;
use crate::records::ast_name::AstName;
use crate::records::location::Location;

#[derive(Debug, Clone)]
pub struct AstClassMethod {
    pub qualifier_location: Option<Location>,
    pub keyword_location: Location,
    pub function_name: AstName,
    pub name_location: Location,
    pub function: *mut AstExprFunction,
}

#[allow(non_snake_case)]
impl AstClassMethod {
    pub const fn qualifier_location(&self) -> Option<Location> {
        self.qualifier_location
    }

    pub const fn keyword_location(&self) -> Location {
        self.keyword_location
    }

    pub const fn function_name(&self) -> AstName {
        self.function_name
    }

    pub const fn name_location(&self) -> Location {
        self.name_location
    }
}
