use crate::records::ast_array::AstArray;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_local::AstLocal;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_local::AstStatLocal;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstStatLocal {
    pub fn new(
        location: Location,
        vars: AstArray<*mut AstLocal>,
        values: AstArray<*mut AstExpr>,
        equals_sign_location: Option<Location>,
        is_const: bool,
    ) -> Self {
        Self {
            base: AstStat {
                base: crate::records::ast_node::AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
                has_semicolon: false,
            },
            vars,
            values,
            is_const,
            is_exported: false,
            equals_sign_location,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_local_ast_stat_local(
    location: Location,
    vars: AstArray<*mut AstLocal>,
    values: AstArray<*mut AstExpr>,
    equals_sign_location: Option<Location>,
    is_const: bool,
) -> AstStatLocal {
    AstStatLocal::new(location, vars, values, equals_sign_location, is_const)
}
