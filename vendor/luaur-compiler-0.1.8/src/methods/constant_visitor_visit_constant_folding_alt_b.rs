use crate::records::constant::Constant;
use crate::records::constant_visitor::ConstantVisitor;
use crate::enums::type_constant_folding::Type;
use luaur_ast::records::ast_stat_local::AstStatLocal;
use luaur_ast::records::ast_expr_call::AstExprCall;
use luaur_ast::records::ast_expr_varargs::AstExprVarargs;
use luaur_ast::rtti;

impl<'a> ConstantVisitor<'a> {
    pub fn visit_ast_stat_local(&mut self, node: *mut AstStatLocal) -> bool {
        unsafe {
            let node_ref = &*node;

            let vars = node_ref.vars.as_slice();
            let values = node_ref.values.as_slice();

            let vars_size = vars.len();
            let values_size = values.len();

            let limit = if vars_size < values_size { vars_size } else { values_size };

            for i in 0..limit {
                let rhs = values[i];
                let arg = self.analyze(rhs);

                // FFlag::LuauCompilePropagateTableProps2 is not available in this crate snapshot.
                // Match behavior as if the flag is disabled.
                self.record_value(vars[i], &arg);
            }

            if vars_size > values_size {
                let last = if values_size > 0 { values[values_size - 1] } else { core::ptr::null_mut() };
                
                let mult_ret = !last.is_null() && (
                    !rtti::ast_node_as::<AstExprCall>(last as *mut luaur_ast::records::ast_node::AstNode).is_null() ||
                    !rtti::ast_node_as::<AstExprVarargs>(last as *mut luaur_ast::records::ast_node::AstNode).is_null()
                );

                if !mult_ret {
                    for i in values_size..vars_size {
                        let mut nil: Constant = core::mem::zeroed();
                        nil.r#type = Type::Type_Nil;
                        self.record_value(vars[i], &nil);
                    }
                }
            } else {
                for i in vars_size..values_size {
                    self.analyze(values[i]);
                }
            }
        }

        false
    }
}
