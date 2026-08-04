use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_global::AstExprGlobal;
use crate::records::ast_name::AstName;
use crate::records::ast_node::AstNode;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstExprGlobal {
    pub fn new(location: Location, name: AstName) -> Self {
        Self {
            base: AstExpr {
                base: AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
            },
            name,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_expr_global_ast_expr_global(location: Location, name: AstName) -> AstExprGlobal {
    AstExprGlobal::new(location, name)
}
