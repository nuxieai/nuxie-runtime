use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_index_name::AstExprIndexName;
use crate::records::ast_name::AstName;
use crate::records::ast_node::AstNode;
use crate::records::location::Location;
use crate::records::position::Position;
use crate::rtti::AstNodeClass;

impl AstExprIndexName {
    pub fn new(
        location: Location,
        expr: *mut AstExpr,
        index: AstName,
        index_location: Location,
        op_position: Position,
        op: core::ffi::c_char,
    ) -> Self {
        Self {
            base: AstExpr {
                base: AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
            },
            expr,
            index,
            index_location,
            op_position,
            op,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_expr_index_name_ast_expr_index_name(
    location: Location,
    expr: *mut AstExpr,
    index: AstName,
    index_location: Location,
    op_position: Position,
    op: core::ffi::c_char,
) -> AstExprIndexName {
    AstExprIndexName::new(location, expr, index, index_location, op_position, op)
}
