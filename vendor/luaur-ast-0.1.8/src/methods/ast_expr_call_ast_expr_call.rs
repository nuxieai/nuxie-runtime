use crate::records::ast_array::AstArray;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_call::AstExprCall;
use crate::records::ast_node::AstNode;
use crate::records::ast_type_or_pack::AstTypeOrPack;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstExprCall {
    pub fn new(
        location: Location,
        func: *mut AstExpr,
        args: AstArray<*mut AstExpr>,
        self_: bool,
        explicit_types: AstArray<AstTypeOrPack>,
        arg_location: Location,
    ) -> Self {
        Self {
            base: AstExpr {
                base: AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
            },
            func,
            type_arguments: explicit_types,
            args,
            self_,
            arg_location,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_expr_call_ast_expr_call(
    location: Location,
    func: *mut AstExpr,
    args: AstArray<*mut AstExpr>,
    self_: bool,
    explicit_types: AstArray<AstTypeOrPack>,
    arg_location: Location,
) -> AstExprCall {
    AstExprCall::new(location, func, args, self_, explicit_types, arg_location)
}
