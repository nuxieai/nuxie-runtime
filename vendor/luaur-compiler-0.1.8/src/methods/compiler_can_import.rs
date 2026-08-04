use crate::functions::get_global_state::get_global_state;
use crate::records::compiler::Compiler;
use luaur_ast::records::ast_expr_global::AstExprGlobal;

impl Compiler {
    pub fn can_import(&self, expr: *mut AstExprGlobal) -> bool {
        if self.options.optimization_level < 1 {
            return false;
        }

        unsafe {
            let name = (*expr).name;
            get_global_state(&self.globals, name) != crate::enums::global::Global::Written
        }
    }
}
