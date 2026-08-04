use crate::records::ast_array::AstArray;
use crate::records::ast_attr::AstAttr;
use crate::records::ast_type_or_pack::AstTypeOrPack;
use crate::records::location::Location;
use crate::records::parser::Parser;

impl Parser {
    pub fn parse_function_type(
        &mut self,
        allow_pack: bool,
        attributes: &AstArray<*mut AstAttr>,
    ) -> AstTypeOrPack {
        use crate::records::ast_node::AstNode;
        use crate::records::ast_type::AstType;
        use crate::records::ast_type_function::AstTypeFunction;
        use crate::records::ast_type_group::AstTypeGroup;
        use crate::records::ast_type_list::AstTypeList;
        use crate::records::ast_type_pack::AstTypePack;
        use crate::records::ast_type_pack_explicit::AstTypePackExplicit;
        use crate::records::cst_node::CstNode;
        use crate::records::cst_type_function::CstTypeFunction;
        use crate::records::cst_type_group::CstTypeGroup;
        use crate::records::cst_type_pack_explicit::CstTypePackExplicit;
        use crate::records::lexeme::Type;
        use crate::records::match_lexeme::MatchLexeme;
        use crate::records::position::Position;
        use crate::records::temp_vector::TempVector;
        use luaur_common::FFlag;

        self.increment_recursion_counter("type annotation");

        let mut force_function_type = self.lexer.current().r#type == Type('<' as i32);

        let begin = *self.lexer.current();

        let mut generics_open_position = Position::missing();
        let mut generics_comma_positions = AstArray::default();
        let mut generics_close_position = Position::missing();

        let (generics, generic_packs) = if self.options.store_cst_data {
            let mut local_generics_comma_positions = TempVector::new(&mut self.scratch_position);
            let res = self.parse_generic_type_list(
                false,
                Some(&mut generics_open_position),
                Some(&mut local_generics_comma_positions),
                Some(&mut generics_close_position),
            );
            generics_comma_positions = self.copy_temp_vector_t(&local_generics_comma_positions);
            res
        } else {
            self.parse_generic_type_list(false, None, None, None)
        };

        let parameter_start = *self.lexer.current();

        let open_args_found = self.expect_and_consume_type(Type('(' as i32), "function parameters");

        self.match_recovery_stop_on_token[Type::SkinnyArrow.0 as usize] += 1;

        let mut params = TempVector::new(&mut self.scratch_type);
        let mut names = TempVector::new(&mut self.scratch_opt_arg_name);
        let mut name_colon_positions = TempVector::new(&mut self.scratch_position);
        let mut arg_comma_positions = TempVector::new(&mut self.scratch_position_2);
        let mut vararg_annotation = core::ptr::null_mut();

        if self.lexer.current().r#type != Type(')' as i32) {
            if self.options.store_cst_data {
                vararg_annotation = self.parse_type_list(
                    &mut params,
                    &mut names,
                    &mut arg_comma_positions,
                    &mut name_colon_positions,
                );
            } else {
                vararg_annotation = self.parse_type_list(
                    &mut params,
                    &mut names,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
            }
        }

        let close_args_location = self.lexer.current().location;
        let close_args_found =
            self.expect_match_and_consume(')', &MatchLexeme::new(&parameter_start), true);

        self.match_recovery_stop_on_token[Type::SkinnyArrow.0 as usize] -= 1;

        let param_types = self.copy_temp_vector_t(&params);

        if !names.empty() {
            force_function_type = true;
        }

        let return_type_introducer = self.lexer.current().r#type == Type::SkinnyArrow
            || self.lexer.current().r#type == Type(':' as i32);

        // Not a function at all. Just a parenthesized type. Or maybe a type pack with a single element
        if params.size() == 1
            && vararg_annotation.is_null()
            && !force_function_type
            && !return_type_introducer
        {
            if allow_pack {
                let node = unsafe {
                    (*self.allocator).alloc(AstTypePackExplicit::new(
                        begin.location,
                        AstTypeList {
                            types: param_types,
                            tail_type: core::ptr::null_mut(),
                        },
                    ))
                };
                if self.options.store_cst_data {
                    let open_pos = if open_args_found {
                        parameter_start.location.begin
                    } else {
                        Position::missing()
                    };
                    let close_pos = if close_args_found {
                        close_args_location.begin
                    } else {
                        Position::missing()
                    };
                    let comma_positions_array = self.copy_temp_vector_t(&arg_comma_positions);
                    let cst_node = unsafe {
                        (*self.allocator).alloc(CstTypePackExplicit::cst_type_pack_explicit_position_position_ast_array_position(
                            open_pos,
                            close_pos,
                            comma_positions_array,
                        ))
                    };
                    self.cst_node_map
                        .try_insert(node as *mut AstNode, cst_node as *mut CstNode);
                }
                return AstTypeOrPack {
                    r#type: core::ptr::null_mut(),
                    type_pack: node as *mut AstTypePack,
                };
            } else {
                let node = unsafe {
                    (*self.allocator).alloc(AstTypeGroup::new(
                        Location::new(parameter_start.location.begin, close_args_location.end),
                        params[0],
                    ))
                };
                if FFlag::LuauCstTypeGroup.get() && self.options.store_cst_data {
                    let close_pos = if close_args_found {
                        close_args_location.begin
                    } else {
                        Position::missing()
                    };
                    let cst_node = unsafe { (*self.allocator).alloc(CstTypeGroup::new(close_pos)) };
                    self.cst_node_map
                        .try_insert(node as *mut AstNode, cst_node as *mut CstNode);
                }
                return AstTypeOrPack {
                    r#type: node as *mut AstType,
                    type_pack: core::ptr::null_mut(),
                };
            }
        }

        if !force_function_type && !return_type_introducer && allow_pack {
            let node = unsafe {
                (*self.allocator).alloc(AstTypePackExplicit::new(
                    begin.location,
                    AstTypeList {
                        types: param_types,
                        tail_type: vararg_annotation,
                    },
                ))
            };
            if self.options.store_cst_data {
                let open_pos = if open_args_found {
                    parameter_start.location.begin
                } else {
                    Position::missing()
                };
                let close_pos = if close_args_found {
                    close_args_location.begin
                } else {
                    Position::missing()
                };
                let comma_positions_array = self.copy_temp_vector_t(&arg_comma_positions);
                let cst_node = unsafe {
                    (*self.allocator).alloc(CstTypePackExplicit::cst_type_pack_explicit_position_position_ast_array_position(
                        open_pos,
                        close_pos,
                        comma_positions_array,
                    ))
                };
                self.cst_node_map
                    .try_insert(node as *mut AstNode, cst_node as *mut CstNode);
            }
            return AstTypeOrPack {
                r#type: core::ptr::null_mut(),
                type_pack: node as *mut AstTypePack,
            };
        }

        let param_names = self.copy_temp_vector_t(&names);
        let return_arrow_position = self.lexer.current().location.begin;
        let node = self.parse_function_type_tail(
            &begin,
            *attributes,
            generics,
            generic_packs,
            param_types,
            param_names,
            vararg_annotation,
        );

        if self.options.store_cst_data
            && !node.is_null()
            && unsafe { crate::rtti::ast_node_is::<AstTypeFunction>(&*node) }
        {
            let names_colon_array = self.copy_temp_vector_t(&name_colon_positions);
            let args_comma_array = self.copy_temp_vector_t(&arg_comma_positions);
            let open_args_pos = if open_args_found {
                parameter_start.location.begin
            } else {
                Position::missing()
            };
            let close_args_pos = if close_args_found {
                close_args_location.begin
            } else {
                Position::missing()
            };
            let cst_node = unsafe {
                (*self.allocator).alloc(CstTypeFunction::new(
                    generics_open_position,
                    generics_comma_positions,
                    generics_close_position,
                    open_args_pos,
                    names_colon_array,
                    args_comma_array,
                    close_args_pos,
                    return_arrow_position,
                ))
            };
            self.cst_node_map
                .try_insert(node as *mut AstNode, cst_node as *mut CstNode);
        }

        AstTypeOrPack {
            r#type: node,
            type_pack: core::ptr::null_mut(),
        }
    }
}
