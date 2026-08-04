use crate::records::ast_expr_function::AstExprFunction;
use crate::records::ast_name::AstName;
use crate::records::ast_node::AstNode;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_type_function::AstStatTypeFunction;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstStatTypeFunction {
    pub fn new(
        location: Location,
        name: AstName,
        name_location: Location,
        body: *mut AstExprFunction,
        exported: bool,
        has_errors: bool,
    ) -> Self {
        Self {
            base: AstStat {
                base: AstNode {
                    class_index: <Self as crate::rtti::AstNodeClass>::CLASS_INDEX,
                    location,
                },
                has_semicolon: false,
            },
            name,
            name_location,
            body,
            exported,
            has_errors,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_type_function_ast_stat_type_function(
    location: Location,
    name: AstName,
    name_location: Location,
    body: *mut AstExprFunction,
    exported: bool,
    has_errors: bool,
) -> AstStatTypeFunction {
    AstStatTypeFunction::new(location, name, name_location, body, exported, has_errors)
}
