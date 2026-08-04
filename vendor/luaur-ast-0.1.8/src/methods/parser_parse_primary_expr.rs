use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_index_name::AstExprIndexName;
use crate::records::lexeme::Lexeme;
use crate::records::lexeme::Type;
use crate::records::location::Location;
use crate::records::parser::Parser;
use crate::records::position::Position;

impl Parser {
    pub fn parse_primary_expr(&mut self, as_statement: bool) -> *mut AstExpr {
        let start = self.lexer.current().location.begin;

        let mut expr = self.parse_prefix_expr();

        let old_recursion_count = self.recursion_counter;

        loop {
            let current = self.lexer.current();
            if current.r#type == Type('.' as i32) {
                let op_position = current.location.begin;
                self.next_lexeme();

                let index = self.parse_index_name("index", &op_position);

                expr = unsafe {
                    (*self.allocator).alloc(AstExprIndexName::new(
                        Location::new(start, index.location.end),
                        expr,
                        index.name,
                        index.location,
                        op_position,
                        '.' as core::ffi::c_char,
                    )) as *mut AstExpr
                };
            } else if current.r#type == Type('[' as i32) {
                expr = self.parse_index_expr(start, expr);
            } else if current.r#type == Type(':' as i32) {
                expr = self.parse_method_call(start, expr);
            } else if current.r#type == Type('(' as i32) {
                if !as_statement
                    && unsafe {
                        (*expr).base.location.end.line != self.lexer.current().location.begin.line
                    }
                {
                    self.report_ambiguous_call_error();
                    break;
                }

                expr = self.parse_function_args(expr, false);
            } else if current.r#type == Type('{' as i32)
                || current.r#type == Type::RawString
                || current.r#type == Type::QuotedString
            {
                expr = self.parse_function_args(expr, false);
            } else if current.r#type == Type('<' as i32)
                && self.lexer.lookahead().r#type == Type('<' as i32)
            {
                expr = self.parse_explicit_type_instantiation_expr(start, unsafe { &mut *expr });
            } else {
                break;
            }

            self.increment_recursion_counter("expression");
        }

        self.recursion_counter = old_recursion_count;

        expr
    }
}
