use crate::records::ast_expr_constant_string::AstExprConstantString;

impl AstExprConstantString {
    pub fn is_quoted(&self) -> bool {
        self.quote_style == AstExprConstantString::QuotedSimple
            || self.quote_style == AstExprConstantString::QuotedRaw
    }
}

#[allow(non_snake_case)]
pub fn ast_expr_constant_string_is_quoted(this: &AstExprConstantString) -> bool {
    this.is_quoted()
}
