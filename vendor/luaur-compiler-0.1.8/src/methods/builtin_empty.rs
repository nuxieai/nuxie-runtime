use crate::records::builtin::Builtin;
use luaur_ast::records::ast_name::AstName;

impl Builtin {
    pub fn empty(&self) -> bool {
        self.object == AstName::default() && self.method == AstName::default()
    }
}
