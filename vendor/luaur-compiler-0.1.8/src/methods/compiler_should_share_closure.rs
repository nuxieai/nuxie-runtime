use crate::records::compiler::Compiler;
use luaur_ast::records::ast_expr_function::AstExprFunction;
use luaur_ast::records::ast_node::AstNode;

impl Compiler {
    pub fn should_share_closure(&mut self, func: *mut AstExprFunction) -> bool {
        let f = match self.functions.find(&func) {
            Some(f) => f,
            None => return false,
        };

        let upvals = f.upvals.clone();

        for uv in upvals {
            let ul = match self.variables.find(&uv) {
                Some(ul) => *ul,
                None => return false,
            };

            if ul.written {
                return false;
            }

            unsafe {
                if (*uv).function_depth != 0 || (*uv).loop_depth != 0 {
                    let uf = if !ul.init.is_null() {
                        luaur_ast::rtti::ast_node_as::<AstExprFunction>(ul.init as *mut AstNode)
                    } else {
                        core::ptr::null_mut()
                    };

                    if uf.is_null() {
                        return false;
                    }

                    if uf != func && !self.should_share_closure(uf) {
                        return false;
                    }
                }
            }
        }

        true
    }
}
