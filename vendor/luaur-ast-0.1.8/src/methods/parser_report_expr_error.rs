use crate::records::ast_array::AstArray;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_error::AstExprError;
use crate::records::location::Location;
use crate::records::parser::Parser;

impl Parser {
    pub fn report_expr_error(
        &mut self,
        location: Location,
        expressions: AstArray<*mut AstExpr>,
        format: core::fmt::Arguments<'_>,
    ) -> *mut AstExprError {
        self.report(location, format);

        let message_index = (self.parse_errors.len() as u32).saturating_sub(1);

        unsafe {
            crate::records::allocator::Allocator::alloc(
                &mut *self.allocator,
                AstExprError {
                    base: AstExpr {
                        base: crate::records::ast_node::AstNode {
                            class_index: <AstExprError as crate::rtti::AstNodeClass>::CLASS_INDEX,
                            location,
                        },
                    },
                    expressions,
                    message_index,
                },
            )
        }
    }
}
