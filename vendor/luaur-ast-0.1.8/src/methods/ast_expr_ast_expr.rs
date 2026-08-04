use crate::records::ast_expr::AstExpr;
use crate::records::ast_node::AstNode;
use crate::records::location::Location;

impl AstExpr {
    pub fn new(class_index: i32, location: Location) -> Self {
        Self {
            base: AstNode {
                class_index,
                location,
            },
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_expr_ast_expr(classIndex: i32, location: Location) -> AstExpr {
    AstExpr::new(classIndex, location)
}
