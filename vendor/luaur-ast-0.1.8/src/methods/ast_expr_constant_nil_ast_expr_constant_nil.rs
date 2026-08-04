use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_constant_nil::AstExprConstantNil;
use crate::records::ast_node::AstNode;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstExprConstantNil {
    pub fn new(location: Location) -> Self {
        Self {
            base: AstExpr {
                base: AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
            },
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_expr_constant_nil_ast_expr_constant_nil(location: Location) -> AstExprConstantNil {
    AstExprConstantNil::new(location)
}
