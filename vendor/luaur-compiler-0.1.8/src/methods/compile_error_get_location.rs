use crate::records::compile_error::CompileError;
use luaur_ast::records::location::Location;

impl CompileError {
    pub fn get_location(&self) -> &Location {
        &self.location
    }
}
