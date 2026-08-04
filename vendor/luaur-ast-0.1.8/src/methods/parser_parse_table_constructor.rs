use crate::records::ast_expr::AstExpr;
use crate::records::parser::Parser;

impl Parser {
    pub fn parse_table_constructor(&mut self) -> *mut AstExpr {
        use crate::records::ast_array::AstArray;
        use crate::records::lexeme::Type;
        use crate::records::location::Location;
        use crate::records::match_lexeme::MatchLexeme;
        use crate::records::position::Position;
        use crate::records::temp_vector::TempVector;
        use luaur_common::FFlag;

        let mut items = TempVector::new(&mut self.scratch_item);
        let mut cst_items = TempVector::new(&mut self.scratch_cst_item);

        let start = self.lexer.current().location;

        let match_brace = MatchLexeme::new(self.lexer.current());
        self.expect_and_consume_char('{', "table literal");

        let mut last_element_indent_deprecated = 0u32;

        while self.lexer.current().r#type != Type(b'}' as i32) {
            if !FFlag::LuauTableEntriesDontNeedToMatchIndent.get() {
                last_element_indent_deprecated = self.lexer.current().location.begin.column;
            }

            if self.lexer.current().r#type == Type(b'[' as i32) {
                let indexer_open_position = self.lexer.current().location.begin;
                let match_location_bracket = MatchLexeme::new(self.lexer.current());
                self.next_lexeme();

                let key = self.parse_expr_i32(0);

                let closing_bracket_found =
                    self.expect_match_and_consume(']', &match_location_bracket, false);
                let indexer_close_position = if closing_bracket_found {
                    self.lexer.previous_location().begin
                } else {
                    Position::missing()
                };

                let equals_found = self.expect_and_consume_char('=', "table field");
                let equals_position = if equals_found {
                    self.lexer.previous_location().begin
                } else {
                    Position::missing()
                };

                let value = self.parse_expr_i32(0);

                items.push_back(crate::records::ast_expr_table::Item {
                    kind: crate::records::ast_expr_table::ItemKind::General,
                    key,
                    value,
                });

                if self.options.store_cst_data {
                    let sep = self.table_separator();
                    let separator = match sep {
                        crate::enums::separator::Separator::Comma => {
                            crate::records::cst_expr_table::CstExprTableSeparator::Comma
                        }
                        crate::enums::separator::Separator::Semicolon => {
                            crate::records::cst_expr_table::CstExprTableSeparator::Semicolon
                        }
                        crate::enums::separator::Separator::Missing => {
                            crate::records::cst_expr_table::CstExprTableSeparator::Missing
                        }
                    };
                    cst_items.push_back(crate::records::cst_expr_table::CstExprTableItem {
                        indexer_open_position,
                        indexer_close_position,
                        equals_position,
                        separator,
                        separator_position: if separator
                            == crate::records::cst_expr_table::CstExprTableSeparator::Missing
                        {
                            Position::missing()
                        } else {
                            self.lexer.current().location.begin
                        },
                    });
                }
            } else if self.lexer.current().r#type == Type::Name
                && self.lexer.lookahead().r#type == Type(b'=' as i32)
            {
                let name = self.parse_name("table field");

                let equals_position = self.lexer.current().location.begin;
                self.expect_and_consume_char('=', "table field");

                let len = unsafe { core::ffi::CStr::from_ptr(name.name.value).to_bytes().len() };
                let name_string = AstArray {
                    data: name.name.value as *mut core::ffi::c_char,
                    size: len,
                };

                let key = unsafe {
                    (*self.allocator).alloc(crate::methods::ast_expr_constant_string_ast_expr_constant_string::ast_expr_constant_string_ast_expr_constant_string(
                        name.location,
                        name_string,
                        crate::enums::quote_style_ast::QuoteStyle::Unquoted,
                    )) as *mut crate::records::ast_expr::AstExpr
                };
                let value = self.parse_expr_i32(0);

                let func = unsafe {
                    crate::rtti::ast_node_as::<crate::records::ast_expr_function::AstExprFunction>(
                        value as *mut crate::records::ast_node::AstNode,
                    )
                };
                if !func.is_null() {
                    unsafe {
                        (*func).debugname = name.name;
                    }
                }

                items.push_back(crate::records::ast_expr_table::Item {
                    kind: crate::records::ast_expr_table::ItemKind::Record,
                    key,
                    value,
                });

                if self.options.store_cst_data {
                    let sep = self.table_separator();
                    let separator = match sep {
                        crate::enums::separator::Separator::Comma => {
                            crate::records::cst_expr_table::CstExprTableSeparator::Comma
                        }
                        crate::enums::separator::Separator::Semicolon => {
                            crate::records::cst_expr_table::CstExprTableSeparator::Semicolon
                        }
                        crate::enums::separator::Separator::Missing => {
                            crate::records::cst_expr_table::CstExprTableSeparator::Missing
                        }
                    };
                    cst_items.push_back(crate::records::cst_expr_table::CstExprTableItem {
                        indexer_open_position: Position::missing(),
                        indexer_close_position: Position::missing(),
                        equals_position,
                        separator,
                        separator_position: if separator
                            == crate::records::cst_expr_table::CstExprTableSeparator::Missing
                        {
                            Position::missing()
                        } else {
                            self.lexer.current().location.begin
                        },
                    });
                }
            } else {
                let expr = self.parse_expr_i32(0);

                items.push_back(crate::records::ast_expr_table::Item {
                    kind: crate::records::ast_expr_table::ItemKind::List,
                    key: core::ptr::null_mut(),
                    value: expr,
                });

                if self.options.store_cst_data {
                    let sep = self.table_separator();
                    let separator = match sep {
                        crate::enums::separator::Separator::Comma => {
                            crate::records::cst_expr_table::CstExprTableSeparator::Comma
                        }
                        crate::enums::separator::Separator::Semicolon => {
                            crate::records::cst_expr_table::CstExprTableSeparator::Semicolon
                        }
                        crate::enums::separator::Separator::Missing => {
                            crate::records::cst_expr_table::CstExprTableSeparator::Missing
                        }
                    };
                    cst_items.push_back(crate::records::cst_expr_table::CstExprTableItem {
                        indexer_open_position: Position::missing(),
                        indexer_close_position: Position::missing(),
                        equals_position: Position::missing(),
                        separator,
                        separator_position: if separator
                            == crate::records::cst_expr_table::CstExprTableSeparator::Missing
                        {
                            Position::missing()
                        } else {
                            self.lexer.current().location.begin
                        },
                    });
                }
            }

            let current_type = self.lexer.current().r#type;
            if current_type == Type(b',' as i32) || current_type == Type(b';' as i32) {
                self.next_lexeme();
            } else if (current_type == Type(b'[' as i32) || current_type == Type::Name)
                && (FFlag::LuauTableEntriesDontNeedToMatchIndent.get()
                    || self.lexer.current().location.begin.column == last_element_indent_deprecated)
            {
                self.report(
                    self.lexer.current().location,
                    format_args!("Expected ',' after table constructor element"),
                );
            } else if current_type != Type(b'}' as i32) {
                break;
            }
        }

        let mut end = self.lexer.current().location;

        if !self.expect_match_and_consume('}', &match_brace, false) {
            end = *self.lexer.previous_location();
        }

        let items_array = self.copy_temp_vector_t(&items);

        let node = unsafe {
            (*self.allocator).alloc(
                crate::methods::ast_expr_table_ast_expr_table::ast_expr_table_ast_expr_table(
                    Location::new(start.begin, end.end),
                    items_array,
                ),
            )
        };

        if self.options.store_cst_data {
            let cst_items_array = self.copy_temp_vector_t(&cst_items);
            let cst_node = unsafe {
                (*self.allocator).alloc(
                    crate::methods::cst_expr_table_cst_expr_table::cst_expr_table_cst_expr_table(
                        cst_items_array,
                    ),
                )
            };
            self.cst_node_map.try_insert(
                node as *mut crate::records::ast_node::AstNode,
                cst_node as *mut crate::records::cst_node::CstNode,
            );
        }

        node as *mut AstExpr
    }
}
