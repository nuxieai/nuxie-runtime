use crate::enums::constant_number_parse_result::ConstantNumberParseResult;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_constant_number::AstExprConstantNumber;
use crate::records::ast_node::AstNode;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstExprConstantNumber {
    pub fn new(location: Location, value: f64, parse_result: ConstantNumberParseResult) -> Self {
        Self {
            base: AstExpr {
                base: AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
            },
            value,
            parse_result,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_expr_constant_number_ast_expr_constant_number(
    location: Location,
    value: f64,
    parse_result: ConstantNumberParseResult,
) -> AstExprConstantNumber {
    AstExprConstantNumber::new(location, value, parse_result)
}
