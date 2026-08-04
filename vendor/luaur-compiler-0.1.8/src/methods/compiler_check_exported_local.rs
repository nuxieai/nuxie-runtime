use crate::records::compiler::Compiler;
use luaur_ast::records::ast_local::AstLocal;
use luaur_ast::records::location::Location;
use luaur_common::functions::vformat::vformat;
use luaur_common::macros::luau_noinline::LUAU_NOINLINE;

impl Compiler {
    pub fn check_exported_local(&mut self, local: *mut AstLocal, location: &Location) {
        unsafe {
            if (*local).is_exported {
                if !self.at_top_level() {
                    // C++ `CompileError::raise(...)`: throw a typed CompileError, not a String.
                    crate::records::compile_error::CompileError::raise(
                        location,
                        format_args!("'export' may only be applied to top-level statements"),
                    );
                }

                self.exported_locals.push(local);
            }
        }
    }
}
