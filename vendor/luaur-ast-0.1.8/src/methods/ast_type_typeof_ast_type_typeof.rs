use crate::records::ast_expr::AstExpr;
use crate::records::ast_node::AstNode;
use crate::records::ast_type::AstType;
use crate::records::ast_type_typeof::AstTypeTypeof;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstTypeTypeof {
    pub fn new(location: Location, expr: *mut AstExpr) -> Self {
        Self {
            base: AstType {
                base: AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
            },
            expr,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_type_typeof_ast_type_typeof(location: Location, expr: *mut AstExpr) -> AstTypeTypeof {
    AstTypeTypeof::new(location, expr)
}
