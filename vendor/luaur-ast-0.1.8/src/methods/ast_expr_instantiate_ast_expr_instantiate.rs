use crate::records::ast_array::AstArray;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_instantiate::AstExprInstantiate;
use crate::records::ast_node::AstNode;
use crate::records::ast_type_or_pack::AstTypeOrPack;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstExprInstantiate {
    pub fn new(location: Location, expr: *mut AstExpr, types: AstArray<AstTypeOrPack>) -> Self {
        Self {
            base: AstExpr {
                base: AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
            },
            expr,
            type_arguments: types,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_expr_instantiate_ast_expr_instantiate(
    location: Location,
    expr: *mut AstExpr,
    types: AstArray<AstTypeOrPack>,
) -> AstExprInstantiate {
    AstExprInstantiate::new(location, expr, types)
}
