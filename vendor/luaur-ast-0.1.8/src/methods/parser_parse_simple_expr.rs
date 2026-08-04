use crate::records::ast_array::AstArray;
use crate::records::ast_attr::AstAttr;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_constant_bool::AstExprConstantBool;
use crate::records::ast_expr_constant_nil::AstExprConstantNil;
use crate::records::ast_expr_varargs::AstExprVarargs;
use crate::records::ast_name::AstName;
use crate::records::lexeme::Type;
use crate::records::parser::Parser;

impl Parser {
    pub fn parse_simple_expr(&mut self) -> *mut AstExpr {
        let start = self.lexer.current().location;

        let mut attributes: AstArray<*mut AstAttr> = AstArray {
            data: core::ptr::null_mut(),
            size: 0,
        };

        if self.lexer.current().r#type == Type::Attribute
            || self.lexer.current().r#type == Type::AttributeOpen
        {
            attributes = self.parse_attributes();

            if self.lexer.current().r#type != Type::ReservedFunction {
                return self.report_expr_error(
                    start,
                    AstArray {
                        data: core::ptr::null_mut(),
                        size: 0,
                    },
                    format_args!(
                        "Expected 'function' declaration after attribute, but got {} instead",
                        self.lexer.current().to_string()
                    ),
                ) as *mut AstExpr;
            }
        }

        if self.lexer.current().r#type == Type::ReservedNil {
            self.next_lexeme();
            unsafe { (*self.allocator).alloc(AstExprConstantNil::new(start)) as *mut AstExpr }
        } else if self.lexer.current().r#type == Type::ReservedTrue {
            self.next_lexeme();
            unsafe {
                (*self.allocator).alloc(AstExprConstantBool::new(start, true)) as *mut AstExpr
            }
        } else if self.lexer.current().r#type == Type::ReservedFalse {
            self.next_lexeme();
            unsafe {
                (*self.allocator).alloc(AstExprConstantBool::new(start, false)) as *mut AstExpr
            }
        } else if self.lexer.current().r#type == Type::ReservedFunction {
            let match_function = self.lexer.current().clone();
            self.next_lexeme();

            self.parse_function_body(
                false,
                &match_function,
                &AstName::new(),
                None,
                &attributes,
                false,
            )
            .0 as *mut AstExpr
        } else if self.lexer.current().r#type == Type::Number {
            self.parse_number()
        } else if self.lexer.current().r#type == Type::RawString
            || self.lexer.current().r#type == Type::QuotedString
            || self.lexer.current().r#type == Type::InterpStringSimple
        {
            self.parse_string()
        } else if self.lexer.current().r#type == Type::InterpStringBegin {
            self.parse_interp_string()
        } else if self.lexer.current().r#type == Type::BrokenString {
            self.next_lexeme();
            self.report_expr_error(
                start,
                AstArray {
                    data: core::ptr::null_mut(),
                    size: 0,
                },
                format_args!("Malformed string; did you forget to finish it?"),
            ) as *mut AstExpr
        } else if self.lexer.current().r#type == Type::BrokenInterpDoubleBrace {
            self.next_lexeme();
            self.report_expr_error(
                start,
                AstArray {
                    data: core::ptr::null_mut(),
                    size: 0,
                },
                format_args!("Double braces are not permitted within interpolated strings; did you mean '\\{{'?"),
            ) as *mut AstExpr
        } else if self.lexer.current().r#type == Type::Dot3 {
            if self.function_stack.last().map_or(false, |f| f.vararg) {
                self.next_lexeme();
                unsafe { (*self.allocator).alloc(AstExprVarargs::new(start)) as *mut AstExpr }
            } else {
                self.next_lexeme();
                self.report_expr_error(
                    start,
                    AstArray {
                        data: core::ptr::null_mut(),
                        size: 0,
                    },
                    format_args!("Cannot use '...' outside of a vararg function"),
                ) as *mut AstExpr
            }
        } else if self.lexer.current().r#type == Type('{' as i32) {
            // C++ `else if (lexer.current().type == '{') return parseTableConstructor();`
            // The model mistranslated the brace literal `'{'` as `'('`, routing
            // parenthesized expressions into the table-constructor parser and causing
            // unbounded recursion on any `(expr)`.
            self.parse_table_constructor()
        } else if self.lexer.current().r#type == Type::ReservedIf {
            self.parse_if_else_expr()
        } else {
            self.parse_primary_expr(false)
        }
    }
}
