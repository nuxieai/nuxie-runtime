use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_error::AstExprError;
use crate::records::ast_expr_local::AstExprLocal;
use crate::records::parser::Parser;

impl Parser {
    pub fn report_l_value_error(&mut self, expr: *mut AstExpr) -> *mut AstExprError {
        if luaur_common::FFlag::LuauConst2.get() {
            let local_expr = unsafe {
                crate::rtti::ast_node_as::<AstExprLocal>(
                    expr as *mut crate::records::ast_node::AstNode,
                )
            };
            if !local_expr.is_null() {
                let local = unsafe { &*local_expr };
                if !local.local.is_null() && unsafe { (*local.local).is_const } {
                    let location = unsafe { (*expr).base.location };
                    let expressions = self.copy_initializer_list_t(&[expr]);
                    let name = unsafe { (*local.local).name.value };
                    return self.report_expr_error(
                        location,
                        expressions,
                        format_args!(
                            "Variable '{}' is constant and may not be reassigned",
                            unsafe { core::ffi::CStr::from_ptr(name).to_string_lossy() }
                        ),
                    );
                }
            }
        }

        let location = unsafe { (*expr).base.location };
        let expressions = self.copy_initializer_list_t(&[expr]);
        self.report_expr_error(
            location,
            expressions,
            format_args!("Assigned expression must be a variable or a field"),
        )
    }
}
