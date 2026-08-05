use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_error::AstExprError;
use crate::records::ast_expr_local::AstExprLocal;
use crate::records::parser::Parser;

impl Parser {
    pub fn report_l_value_error(&mut self, expr: *mut AstExpr) -> *mut AstExprError {
        let local_expr = unsafe {
            crate::rtti::ast_node_as::<AstExprLocal>(expr as *mut crate::records::ast_node::AstNode)
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

        if luaur_common::FFlag::DebugLuauUserDefinedClasses.get() {
            let class_stat = self.get_matching_class(expr);
            if !class_stat.is_null() {
                let name = unsafe { (*(*class_stat).name).name.value };
                let line = unsafe { (*class_stat).base.base.location.begin.line + 1 };
                let location = unsafe { (*expr).base.location };
                let expressions = self.copy_initializer_list_t(&[expr]);
                return self.report_expr_error(
                    location,
                    expressions,
                    format_args!(
                        "'{}' refers to a class and cannot be used as a variable name (defined on line {})",
                        unsafe { core::ffi::CStr::from_ptr(name).to_string_lossy() },
                        line
                    ),
                );
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
