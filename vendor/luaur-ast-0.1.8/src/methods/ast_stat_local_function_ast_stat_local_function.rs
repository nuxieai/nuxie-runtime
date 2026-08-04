use crate::records::ast_expr_function::AstExprFunction;
use crate::records::ast_local::AstLocal;
use crate::records::ast_node::AstNode;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_local_function::AstStatLocalFunction;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstStatLocalFunction {
    pub fn new(
        location: Location,
        name: *mut AstLocal,
        func: *mut AstExprFunction,
        is_const: bool,
    ) -> Self {
        Self {
            base: AstStat {
                base: AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
                has_semicolon: false,
            },
            name,
            func,
            is_const,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_local_function_ast_stat_local_function(
    location: Location,
    name: *mut AstLocal,
    func: *mut AstExprFunction,
    is_const: bool,
) -> AstStatLocalFunction {
    AstStatLocalFunction::new(location, name, func, is_const)
}
