use crate::functions::is_type_follow::is_type_follow;
use crate::functions::should_parse_type_pack::should_parse_type_pack;
use crate::records::ast_array::AstArray;
use crate::records::ast_type::AstType;
use crate::records::ast_type_group::AstTypeGroup;
use crate::records::ast_type_or_pack::AstTypeOrPack;
use crate::records::ast_type_pack_explicit::AstTypePackExplicit;
use crate::records::cst_type_group::CstTypeGroup;
use crate::records::cst_type_pack_explicit::CstTypePackExplicit;
use crate::records::lexeme::Type;
use crate::records::match_lexeme::MatchLexeme;
use crate::records::parser::Parser;
use crate::records::position::Position;
use crate::records::temp_vector::TempVector;
use luaur_common::macros::luau_assert::LUAU_ASSERT;
use luaur_common::FFlag;

impl Parser {
    pub fn parse_type_params(
        &mut self,
        opening_position: Option<&mut Position>,
        mut comma_positions: Option<&mut TempVector<'_, Position>>,
        closing_position: Option<&mut Position>,
    ) -> AstArray<AstTypeOrPack> {
        let mut parameters: Vec<AstTypeOrPack> = Vec::new();

        if self.lexer.current().r#type == Type::Less {
            let begin = *self.lexer.current();
            if let Some(pos) = opening_position {
                *pos = begin.location.begin;
            }
            self.next_lexeme();

            loop {
                if should_parse_type_pack(&mut self.lexer) {
                    let type_pack = self.parse_type_pack();
                    parameters.push(AstTypeOrPack {
                        r#type: core::ptr::null_mut(),
                        type_pack,
                    });
                } else if self.lexer.current().r#type == Type::Char_OPEN {
                    let begin_loc = self.lexer.current().location;
                    let mut type_ = core::ptr::null_mut();
                    let mut type_pack = core::ptr::null_mut();
                    let c = self.lexer.current().r#type;

                    if c != Type::Char_PIPE && c != Type::Char_AMPERSAND {
                        let type_or_type_pack = self.parse_simple_type(true, false);
                        type_ = type_or_type_pack.r#type;
                        type_pack = type_or_type_pack.type_pack;
                    }

                    if !type_pack.is_null() {
                        let explicit_type_pack = unsafe {
                            crate::rtti::ast_node_as::<AstTypePackExplicit>(
                                type_pack as *mut crate::records::ast_node::AstNode,
                            )
                        };
                        if !explicit_type_pack.is_null()
                            && unsafe { (*explicit_type_pack).type_list.tail_type.is_null() }
                            && unsafe { (*explicit_type_pack).type_list.types.size == 1 }
                            && is_type_follow(self.lexer.current().r#type)
                        {
                            let parenthesized_type =
                                unsafe { *(*explicit_type_pack).type_list.types.data.add(0) };

                            if FFlag::LuauCstTypeGroup.get() && self.options.store_cst_data {
                                let type_group = unsafe {
                                    (*self.allocator).alloc(AstTypeGroup::new(
                                        (*parenthesized_type).base.location,
                                        parenthesized_type,
                                    ))
                                };

                                if let Some(cst_node) = self.cst_node_map.find(
                                    &(explicit_type_pack as *mut crate::records::ast_node::AstNode),
                                ) {
                                    if !cst_node.is_null() {
                                        let cst_explicit_type_pack = unsafe {
                                            crate::rtti::cst_node_as::<CstTypePackExplicit>(
                                                *cst_node,
                                            )
                                        };
                                        if !cst_explicit_type_pack.is_null() {
                                            let close_pos = unsafe {
                                                (*cst_explicit_type_pack).close_parentheses_position
                                            };
                                            let cst_node_group = unsafe {
                                                (*self.allocator)
                                                    .alloc(CstTypeGroup::new(close_pos))
                                            };
                                            self.cst_node_map.try_insert(
                                                type_group
                                                    as *mut crate::records::ast_node::AstNode,
                                                cst_node_group
                                                    as *mut crate::records::cst_node::CstNode,
                                            );
                                        }
                                    }
                                }

                                parameters.push(AstTypeOrPack {
                                    r#type: self
                                        .parse_type_suffix(type_group as *mut AstType, &begin_loc),
                                    type_pack: core::ptr::null_mut(),
                                });
                            } else {
                                let type_group = unsafe {
                                    (*self.allocator).alloc(AstTypeGroup::new(
                                        (*parenthesized_type).base.location,
                                        parenthesized_type,
                                    ))
                                };
                                parameters.push(AstTypeOrPack {
                                    r#type: self
                                        .parse_type_suffix(type_group as *mut AstType, &begin_loc),
                                    type_pack: core::ptr::null_mut(),
                                });
                            }
                        } else {
                            parameters.push(AstTypeOrPack {
                                r#type: core::ptr::null_mut(),
                                type_pack,
                            });
                        }
                    } else {
                        parameters.push(AstTypeOrPack {
                            r#type: self.parse_type_suffix(type_, &begin_loc),
                            type_pack: core::ptr::null_mut(),
                        });
                    }
                } else if self.lexer.current().r#type == Type::Greater && parameters.is_empty() {
                    break;
                } else {
                    parameters.push(AstTypeOrPack {
                        r#type: self.parse_type_bool(false),
                        type_pack: core::ptr::null_mut(),
                    });
                }

                if self.lexer.current().r#type == Type::Char_COMMA {
                    if let Some(vec) = comma_positions.as_deref_mut() {
                        vec.push_back(self.lexer.current().location.begin);
                    }
                    self.next_lexeme();
                } else {
                    break;
                }
            }

            let closing_bracket_found =
                self.expect_match_and_consume('>', &MatchLexeme::new(&begin), false);
            if let Some(pos) = closing_position {
                if closing_bracket_found {
                    *pos = self.lexer.previous_location().begin;
                }
            }
        }

        self.copy_initializer_list_t(&parameters)
    }
}
