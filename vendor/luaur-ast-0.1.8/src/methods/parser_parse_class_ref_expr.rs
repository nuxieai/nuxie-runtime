use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_index_expr::AstExprIndexExpr;
use crate::records::ast_expr_index_name::AstExprIndexName;
use crate::records::lexeme::Type;
use crate::records::location::Location;
use crate::records::parser::Parser;

impl Parser {
    // classrefexp -> NAME { `.' NAME | `[' exp `]' }
    pub fn parse_class_ref_expr(&mut self) -> *mut AstExpr {
        let start = self.lexer.current().location.begin;
        let name = self.parse_name_expr("class reference expression");
        let dot_or_bracket = *self.lexer.current();

        if dot_or_bracket.r#type == Type('.' as i32) {
            self.next_lexeme();
            let dot_position = dot_or_bracket.location.begin;
            let index = self.parse_index_name("class reference expression", &dot_position);

            unsafe {
                (*self.allocator).alloc(AstExprIndexName::new(
                    Location::new(start, index.location.end),
                    name,
                    index.name,
                    index.location,
                    dot_position,
                    b'.' as core::ffi::c_char,
                )) as *mut AstExpr
            }
        } else if dot_or_bracket.r#type == Type('[' as i32) {
            self.next_lexeme();
            let key = self.parse_expr_i32(0);
            self.expect_and_consume_char(']', "class reference expression");

            unsafe {
                (*self.allocator).alloc(AstExprIndexExpr::new(
                    Location::new(start, self.lexer.previous_location().end),
                    name,
                    key,
                )) as *mut AstExpr
            }
        } else {
            name
        }
    }
}
