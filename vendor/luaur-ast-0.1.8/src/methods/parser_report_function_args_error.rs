use crate::records::ast_expr::AstExpr;
use crate::records::location::Location;
use crate::records::parser::Parser;
use luaur_common::macros::luau_noinline::LUAU_NOINLINE;

impl Parser {
    LUAU_NOINLINE! {
        pub fn report_function_args_error(&mut self, func: *mut AstExpr, self_flag: bool) -> *mut AstExpr {
            let current_lexeme = self.lexer.current();
            let func_loc = unsafe { (*func).base.location };

            if self_flag && current_lexeme.location.begin.line != func_loc.end.line {
                let expressions = self.copy_initializer_list_t(&[func]);
                self.report_expr_error(
                    func_loc,
                    expressions,
                    format_args!("Expected function call arguments after '('"),
                ) as *mut AstExpr
            } else {
                // Read everything off `current_lexeme` (which borrows self.lexer)
                // BEFORE the `&mut self` calls below, or the borrow conflicts.
                let loc = Location::new(func_loc.begin, current_lexeme.location.begin);
                let lexeme_str = current_lexeme.to_string();
                let expressions = self.copy_initializer_list_t(&[func]);
                self.report_expr_error(
                    loc,
                    expressions,
                    format_args!(
                        "Expected '(', '{{' or <string> when parsing function call, got {}",
                        lexeme_str
                    ),
                ) as *mut AstExpr
            }
        }
    }
}
