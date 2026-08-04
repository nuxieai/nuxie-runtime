use crate::enums::quote_style_ast::QuoteStyle;
use crate::records::ast_array::AstArray;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_constant_string::AstExprConstantString;
use crate::records::ast_node::AstNode;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstExprConstantString {
    pub fn new(
        location: Location,
        value: AstArray<core::ffi::c_char>,
        quote_style: QuoteStyle,
    ) -> Self {
        Self {
            base: AstExpr {
                base: AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
            },
            value,
            quote_style,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_expr_constant_string_ast_expr_constant_string(
    location: Location,
    value: AstArray<core::ffi::c_char>,
    quote_style: QuoteStyle,
) -> AstExprConstantString {
    AstExprConstantString::new(location, value, quote_style)
}
