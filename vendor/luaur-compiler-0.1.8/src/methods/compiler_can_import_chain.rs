use crate::functions::get_global_state::get_global_state;
use crate::records::compiler::Compiler;
use luaur_ast::records::ast_expr_global::AstExprGlobal;
use luaur_ast::records::ast_name::AstName;

use crate::enums::global::Global;

impl Compiler {
    pub fn can_import_chain(&self, expr: *mut AstExprGlobal) -> bool {
        if self.options.optimization_level < 1 {
            return false;
        }
        if expr.is_null() {
            return false;
        }

        unsafe {
            let name: AstName = (*expr).name;
            get_global_state(&self.globals, name) == Global::Default
        }
    }
}
