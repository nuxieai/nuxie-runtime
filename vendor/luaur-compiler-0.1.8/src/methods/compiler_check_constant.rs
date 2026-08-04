use crate::records::compiler::Compiler;
use luaur_ast::records::location::Location;

impl Compiler {
    pub fn check_constant(&mut self, constant: i32, location: &Location) {
        if constant < 0 {
            crate::methods::compile_error_raise::compile_error_raise(
                *location,
                core::format_args!("Exceeded constant limit; simplify the code to compile"),
            );
        }
    }
}
