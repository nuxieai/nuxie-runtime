use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_index_name::AstExprIndexName;
use crate::records::ast_name::AstName;
use crate::records::lexeme::Type;
use crate::records::location::Location;
use crate::records::name::Name;
use crate::records::parser::Parser;
use crate::records::position::Position;

impl Parser {
    pub fn parse_function_name(
        &mut self,
        hasself: &mut bool,
        debugname: &mut AstName,
    ) -> *mut AstExpr {
        let current = self.lexer.current();
        if current.r#type == Type::Name {
            *debugname = AstName {
                value: unsafe { current.data.name },
            };
        }

        // parse funcname into a chain of indexing operators
        let mut expr = self.parse_name_expr("function name");

        let old_recursion_count = self.recursion_counter;

        while self.lexer.current().r#type == Type('.' as i32) {
            let op_position = self.lexer.current().location.begin;
            self.next_lexeme();

            let name = self.parse_name("field name");

            // while we could concatenate the name chain, for now let's just write the short name
            *debugname = name.name;

            let location = unsafe { Location::new((*expr).base.location.begin, name.location.end) };
            expr = unsafe {
                (*self.allocator).alloc(AstExprIndexName {
                    base: AstExpr {
                        base: crate::records::ast_node::AstNode {
                            class_index:
                                <AstExprIndexName as crate::rtti::AstNodeClass>::CLASS_INDEX,
                            location,
                        },
                    },
                    expr,
                    index: name.name,
                    index_location: name.location,
                    op_position,
                    op: '.' as core::ffi::c_char,
                }) as *mut AstExpr
            };

            // note: while the parser isn't recursive here, we're generating recursive structures of unbounded depth
            self.increment_recursion_counter("function name");
        }

        self.recursion_counter = old_recursion_count;

        // finish with :
        if self.lexer.current().r#type == Type(':' as i32) {
            let op_position = self.lexer.current().location.begin;
            self.next_lexeme();

            let name = self.parse_name("method name");

            // while we could concatenate the name chain, for now let's just write the short name
            *debugname = name.name;

            let location = unsafe { Location::new((*expr).base.location.begin, name.location.end) };
            expr = unsafe {
                (*self.allocator).alloc(AstExprIndexName {
                    base: AstExpr {
                        base: crate::records::ast_node::AstNode {
                            class_index:
                                <AstExprIndexName as crate::rtti::AstNodeClass>::CLASS_INDEX,
                            location,
                        },
                    },
                    expr,
                    index: name.name,
                    index_location: name.location,
                    op_position,
                    op: ':' as core::ffi::c_char,
                }) as *mut AstExpr
            };

            *hasself = true;
        }

        expr
    }
}
