use crate::records::ast_array::AstArray;
use crate::records::ast_type::AstType;
use crate::records::ast_type_function::AstTypeFunction;
use crate::records::ast_type_group::AstTypeGroup;
use crate::records::ast_type_intersection::AstTypeIntersection;
use crate::records::ast_type_list::AstTypeList;
use crate::records::ast_type_pack::AstTypePack;
use crate::records::ast_type_pack_explicit::AstTypePackExplicit;
use crate::records::ast_type_union::AstTypeUnion;
use crate::records::cst_type_function::CstTypeFunction;
use crate::records::cst_type_group::CstTypeGroup;
use crate::records::cst_type_pack_explicit::CstTypePackExplicit;
use crate::records::lexeme::Type;
use crate::records::location::Location;
use crate::records::parser::Parser;
use crate::records::position::Position;
use crate::records::temp_vector::TempVector;
use crate::type_aliases::ast_argument_name::AstArgumentName;

impl Parser {
    pub fn parse_return_type(&mut self) -> *mut AstTypePack {
        self.increment_recursion_counter("type annotation");
        let begin = *self.lexer.current();

        if self.lexer.current().r#type != Type('(' as i32) {
            if crate::functions::should_parse_type_pack::should_parse_type_pack(&mut self.lexer) {
                return self.parse_type_pack();
            } else {
                let ty = self.parse_type_bool(false);
                let types_array = self.copy_t_usize(&ty as *const *mut AstType, 1);
                let node = unsafe {
                    (*self.allocator).alloc(AstTypePackExplicit::new(
                        (*ty).base.location,
                        AstTypeList {
                            types: types_array,
                            tail_type: core::ptr::null_mut(),
                        },
                    ))
                };
                if self.options.store_cst_data {
                    self.cst_node_map.try_insert(
                        node as *mut crate::records::ast_node::AstNode,
                        unsafe {
                            (*self.allocator).alloc(CstTypePackExplicit::cst_type_pack_explicit())
                        } as *mut crate::records::cst_node::CstNode,
                    );
                }
                return node as *mut AstTypePack;
            }
        }

        self.next_lexeme();
        self.match_recovery_stop_on_token[Type::SkinnyArrow.0 as usize] += 1;

        let mut result: TempVector<'_, *mut AstType> = TempVector::new(&mut self.scratch_type);
        let mut result_names: TempVector<'_, Option<AstArgumentName>> =
            TempVector::new(&mut self.scratch_opt_arg_name);
        let mut comma_positions: TempVector<'_, Position> =
            TempVector::new(&mut self.scratch_position);
        let mut name_colon_positions: TempVector<'_, Position> =
            TempVector::new(&mut self.scratch_position_2);

        let mut vararg_annotation: *mut AstTypePack = core::ptr::null_mut();

        if self.lexer.current().r#type != Type(')' as i32) {
            if self.options.store_cst_data {
                vararg_annotation = self.parse_type_list(
                    &mut result,
                    &mut result_names,
                    &mut comma_positions,
                    &mut name_colon_positions,
                );
            } else {
                vararg_annotation = self.parse_type_list(
                    &mut result,
                    &mut result_names,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
            }
        }

        let location = Location::new(begin.location.begin, self.lexer.current().location.end);
        let close_paren_found = self.expect_match_and_consume(
            ')' as char,
            &crate::records::match_lexeme::MatchLexeme::new(&begin),
            true,
        );
        let close_parentheses_position = if close_paren_found {
            self.lexer.previous_location().begin
        } else {
            Position::missing()
        };

        self.match_recovery_stop_on_token[Type::SkinnyArrow.0 as usize] -= 1;

        if self.lexer.current().r#type != Type::SkinnyArrow && result_names.empty() {
            if result.size() == 1 {
                let mut inner: *mut AstType = core::ptr::null_mut();

                if luaur_common::FFlag::LuauCstTypeGroup.get() {
                    if vararg_annotation.is_null() {
                        inner = unsafe {
                            (*self.allocator)
                                .alloc(AstTypeGroup::new(location, *result.operator_index(0)))
                                as *mut AstType
                        };
                        if self.options.store_cst_data {
                            self.cst_node_map.try_insert(
                                inner as *mut crate::records::ast_node::AstNode,
                                unsafe {
                                    (*self.allocator).alloc(CstTypeGroup::new(
                                        if close_paren_found {
                                            close_parentheses_position
                                        } else {
                                            Position::missing()
                                        },
                                    ))
                                }
                                    as *mut crate::records::cst_node::CstNode,
                            );
                        }
                    } else {
                        inner = *result.operator_index(0);
                    }
                } else {
                    inner = if vararg_annotation.is_null() {
                        unsafe {
                            (*self.allocator)
                                .alloc(AstTypeGroup::new(location, *result.operator_index(0)))
                                as *mut AstType
                        }
                    } else {
                        *result.operator_index(0)
                    };
                }

                let return_type = self.parse_type_suffix(inner, &begin.location);

                if luaur_common::DFFlag::DebugLuauReportReturnTypeVariadicWithTypeSuffix.get()
                    && !vararg_annotation.is_null()
                    && (crate::rtti::ast_node_is::<AstTypeUnion>(unsafe { &*return_type })
                        || crate::rtti::ast_node_is::<AstTypeIntersection>(unsafe {
                            &*return_type
                        }))
                {
                    unsafe {
                        crate::LUAU_TELEMETRY_PARSED_RETURN_TYPE_VARIADIC_WITH_TYPE_SUFFIX = true;
                    }
                }

                let end_pos = if result.size() == 1 {
                    location.end
                } else {
                    unsafe { (*return_type).base.location.end }
                };

                let types_array = self.copy_t_usize(&return_type as *const *mut AstType, 1);
                let node = unsafe {
                    (*self.allocator).alloc(AstTypePackExplicit::new(
                        Location::new(location.begin, end_pos),
                        AstTypeList {
                            types: types_array,
                            tail_type: vararg_annotation,
                        },
                    ))
                };

                if self.options.store_cst_data {
                    self.cst_node_map.try_insert(
                        node as *mut crate::records::ast_node::AstNode,
                        unsafe {
                            (*self.allocator).alloc(CstTypePackExplicit::cst_type_pack_explicit())
                        } as *mut crate::records::cst_node::CstNode,
                    );
                }
                return node as *mut AstTypePack;
            }

            let types_array = self.copy_temp_vector_t(&result);
            let node = unsafe {
                (*self.allocator).alloc(AstTypePackExplicit::new(
                    location,
                    AstTypeList {
                        types: types_array,
                        tail_type: vararg_annotation,
                    },
                ))
            };

            if self.options.store_cst_data {
                let comma_positions_array = self.copy_temp_vector_t(&comma_positions);
                self.cst_node_map.try_insert(
                    node as *mut crate::records::ast_node::AstNode,
                    unsafe {
                        (*self.allocator).alloc(CstTypePackExplicit::cst_type_pack_explicit_position_position_ast_array_position(
                            location.begin,
                            close_parentheses_position,
                            comma_positions_array,
                        ))
                    } as *mut crate::records::cst_node::CstNode,
                );
            }
            return node as *mut AstTypePack;
        }

        let return_arrow_position = self.lexer.current().location.begin;
        // C++ passes `copy(result)` / `copy(resultNames)` — allocator copies. The port
        // passed raw pointers into the scratch TempVectors, but parse_function_type_tail
        // recursively parses the `-> ret` which CLEARS scratch_opt_arg_name (and
        // scratch_type), corrupting the names to None. Copy before the recursion.
        let params_copy = self.copy_temp_vector_t(&result);
        let param_names_copy = self.copy_temp_vector_t(&result_names);
        let tail = self.parse_function_type_tail(
            &begin,
            AstArray {
                data: core::ptr::null_mut(),
                size: 0,
            },
            AstArray {
                data: core::ptr::null_mut(),
                size: 0,
            },
            AstArray {
                data: core::ptr::null_mut(),
                size: 0,
            },
            params_copy,
            param_names_copy,
            vararg_annotation,
        );

        if self.options.store_cst_data
            && !tail.is_null()
            && unsafe { crate::rtti::ast_node_is::<AstTypeFunction>(&*tail) }
        {
            let name_colon_positions_array = self.copy_temp_vector_t(&name_colon_positions);
            let comma_positions_array = self.copy_temp_vector_t(&comma_positions);
            self.cst_node_map
                .try_insert(tail as *mut crate::records::ast_node::AstNode, unsafe {
                    (*self.allocator).alloc(CstTypeFunction::new(
                        Position::missing(),
                        AstArray {
                            data: core::ptr::null_mut(),
                            size: 0,
                        },
                        Position::missing(),
                        location.begin,
                        name_colon_positions_array,
                        comma_positions_array,
                        close_parentheses_position,
                        return_arrow_position,
                    ))
                }
                    as *mut crate::records::cst_node::CstNode);
        }

        let types_array = self.copy_t_usize(&tail as *const *mut AstType, 1);
        let node = unsafe {
            (*self.allocator).alloc(AstTypePackExplicit::new(
                Location::new(location.begin, (*tail).base.location.end),
                AstTypeList {
                    types: types_array,
                    tail_type: core::ptr::null_mut(),
                },
            ))
        };

        if self.options.store_cst_data {
            self.cst_node_map
                .try_insert(node as *mut crate::records::ast_node::AstNode, unsafe {
                    (*self.allocator).alloc(CstTypePackExplicit::cst_type_pack_explicit())
                }
                    as *mut crate::records::cst_node::CstNode);
        }

        node as *mut AstTypePack
    }
}
