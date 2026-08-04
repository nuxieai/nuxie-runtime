use crate::records::ast_array::AstArray;
use crate::records::ast_generic_type::AstGenericType;
use crate::records::ast_generic_type_pack::AstGenericTypePack;
use crate::records::parser::Parser;
use crate::records::position::Position;
use crate::records::temp_vector::TempVector;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Parser {
    pub fn parse_generic_type_list(
        &mut self,
        with_default_values: bool,
        open_position: Option<&mut Position>,
        comma_positions: Option<&mut TempVector<'_, Position>>,
        close_position: Option<&mut Position>,
    ) -> (
        AstArray<*mut AstGenericType>,
        AstArray<*mut AstGenericTypePack>,
    ) {
        let mut comma_positions = comma_positions;
        let mut names: TempVector<'_, *mut AstGenericType> =
            TempVector::new(&mut self.scratch_generic_types);
        let mut name_packs: TempVector<'_, *mut AstGenericTypePack> =
            TempVector::new(&mut self.scratch_generic_type_packs);

        if self.lexer.current().r#type == crate::records::lexeme::Type::Less {
            let begin = self.lexer.current();
            // C++ saves `Lexeme begin = lexer.current()` (the `<`) and later passes
            // MatchLexeme(begin) to expectMatchAndConsume('>'). The port passed
            // MatchLexeme::missing() instead, whose position is {MAX,MAX} -> the
            // "to close '<' at column N" message did `MAX + 1` and overflow-panicked.
            let begin_match = crate::records::match_lexeme::MatchLexeme::new(begin);
            if let Some(open_position) = open_position {
                *open_position = begin.location.begin;
            }
            self.next_lexeme();

            let mut seen_pack = false;
            let mut seen_default = false;

            loop {
                let name_location = self.lexer.current().location;
                // C++ `parseName()` here passes NO context (nullptr default), so the
                // error is "Expected identifier, got X" — not "...when parsing
                // generic type parameter". Pass "" to match.
                let name = self.parse_name("");
                if self.lexer.current().r#type == crate::records::lexeme::Type::Dot3 || seen_pack {
                    seen_pack = true;

                    let mut ellipsis_position = Position::missing();
                    if self.lexer.current().r#type != crate::records::lexeme::Type::Dot3 {
                        self.report(
                            self.lexer.current().location,
                            format_args!("Generic types come before generic type packs"),
                        );
                    } else {
                        ellipsis_position = self.lexer.current().location.begin;
                        self.next_lexeme();
                    }

                    if with_default_values
                        && self.lexer.current().r#type == crate::records::lexeme::Type::Operator
                    {
                        seen_default = true;
                        let equals_position = self.lexer.current().location.begin;
                        self.next_lexeme();

                        if crate::functions::should_parse_type_pack::should_parse_type_pack(
                            &mut self.lexer,
                        ) {
                            let type_pack = self.parse_type_pack();

                            let node = unsafe {
                                (*self.allocator).alloc(AstGenericTypePack::new(
                                    name_location,
                                    name.name,
                                    type_pack,
                                ))
                            };
                            if self.options.store_cst_data {
                                let cst_node = unsafe {
                                    (*self.allocator).alloc(crate::records::cst_generic_type_pack::CstGenericTypePack::new(ellipsis_position, equals_position))
                                };
                                self.cst_node_map.try_insert(
                                    node as *mut crate::records::ast_node::AstNode,
                                    cst_node as *mut crate::records::cst_node::CstNode,
                                );
                            }
                            name_packs.push_back(node);
                        } else {
                            let type_or_pack = self.parse_simple_type_or_pack();
                            let ty = type_or_pack.r#type;
                            let type_pack = type_or_pack.type_pack;

                            if !ty.is_null() {
                                unsafe {
                                    self.report(
                                        (*ty).base.location,
                                        format_args!("Expected type pack after '=', got type"),
                                    );
                                }
                            }

                            let node = unsafe {
                                (*self.allocator).alloc(AstGenericTypePack::new(
                                    name_location,
                                    name.name,
                                    type_pack,
                                ))
                            };
                            if self.options.store_cst_data {
                                let cst_node = unsafe {
                                    (*self.allocator).alloc(crate::records::cst_generic_type_pack::CstGenericTypePack::new(ellipsis_position, equals_position))
                                };
                                self.cst_node_map.try_insert(
                                    node as *mut crate::records::ast_node::AstNode,
                                    cst_node as *mut crate::records::cst_node::CstNode,
                                );
                            }
                            name_packs.push_back(node);
                        }
                    } else {
                        if seen_default {
                            self.report_location_c_char_item(
                                self.lexer.current().location,
                                format_args!("Expected default type pack after type pack name"),
                            );
                        }

                        let node = unsafe {
                            (*self.allocator).alloc(AstGenericTypePack::new(
                                name_location,
                                name.name,
                                core::ptr::null_mut(),
                            ))
                        };
                        if self.options.store_cst_data {
                            let cst_node = unsafe {
                                (*self.allocator).alloc(
                                    crate::records::cst_generic_type_pack::CstGenericTypePack::new(
                                        ellipsis_position,
                                        Position::missing(),
                                    ),
                                )
                            };
                            self.cst_node_map.try_insert(
                                node as *mut crate::records::ast_node::AstNode,
                                cst_node as *mut crate::records::cst_node::CstNode,
                            );
                        }
                        name_packs.push_back(node);
                    }
                } else {
                    if with_default_values
                        && self.lexer.current().r#type == crate::records::lexeme::Type::Operator
                    {
                        seen_default = true;
                        let equals_position = self.lexer.current().location.begin;
                        self.next_lexeme();

                        let default_type = self.parse_type_bool(false);

                        let node = unsafe {
                            (*self.allocator).alloc(AstGenericType::new(
                                name_location,
                                name.name,
                                default_type,
                            ))
                        };
                        if self.options.store_cst_data {
                            let cst_node = unsafe {
                                (*self.allocator).alloc(
                                    crate::records::cst_generic_type::CstGenericType::new(
                                        equals_position,
                                    ),
                                )
                            };
                            self.cst_node_map.try_insert(
                                node as *mut crate::records::ast_node::AstNode,
                                cst_node as *mut crate::records::cst_node::CstNode,
                            );
                        }
                        names.push_back(node);
                    } else {
                        if seen_default {
                            self.report_location_c_char_item(
                                self.lexer.current().location,
                                format_args!("Expected default type after type name"),
                            );
                        }

                        let node = unsafe {
                            (*self.allocator).alloc(AstGenericType::new(
                                name_location,
                                name.name,
                                core::ptr::null_mut(),
                            ))
                        };
                        if self.options.store_cst_data {
                            let cst_node = unsafe {
                                (*self.allocator).alloc(
                                    crate::records::cst_generic_type::CstGenericType::new(
                                        Position::missing(),
                                    ),
                                )
                            };
                            self.cst_node_map.try_insert(
                                node as *mut crate::records::ast_node::AstNode,
                                cst_node as *mut crate::records::cst_node::CstNode,
                            );
                        }
                        names.push_back(node);
                    }
                }

                if self.lexer.current().r#type == crate::records::lexeme::Type::Comma {
                    if let Some(comma_positions) = comma_positions.as_deref_mut() {
                        comma_positions.push_back(self.lexer.current().location.begin);
                    }
                    self.next_lexeme();

                    if self.lexer.current().r#type == crate::records::lexeme::Type::Greater {
                        self.report_location_c_char_item(
                            self.lexer.current().location,
                            format_args!("Expected type after ',' but got '>' instead"),
                        );
                        break;
                    }
                } else {
                    break;
                }
            }

            let closing_bracket_found = self.expect_match_and_consume('>', &begin_match, false);
            if let Some(close_position) = close_position {
                if closing_bracket_found {
                    *close_position = self.lexer.previous_location().begin;
                }
            }
        }

        let generics: AstArray<*mut AstGenericType> = self.copy_temp_vector_t(&names);
        let generic_packs: AstArray<*mut AstGenericTypePack> = self.copy_temp_vector_t(&name_packs);
        (generics, generic_packs)
    }
}
