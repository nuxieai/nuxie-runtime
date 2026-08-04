use crate::functions::is_constant_literal::is_constant_literal;
use crate::functions::is_literal_table::is_literal_table;
use crate::records::ast_array::AstArray;
use crate::records::ast_attr::{AstAttr, AstAttrType};
use crate::records::ast_expr::AstExpr;
use crate::records::ast_name::AstName;
use crate::records::cst_attr::CstAttr;
use crate::records::cst_attr_list::CstAttrList;
use crate::records::cst_node::CstNode;
use crate::records::cst_parametrized_attr::CstParametrizedAttr;
use crate::records::lexeme::Type;
use crate::records::location::Location;
use crate::records::match_lexeme::MatchLexeme;
use crate::records::parser::Parser;
use crate::records::position::Position;
use crate::records::temp_vector::TempVector;

impl Parser {
    pub fn parse_attr_list(
        &mut self,
        attributes: &mut TempVector<'_, *mut AstAttr>,
        cst_attr_lists: *mut TempVector<'_, *mut CstAttrList>,
    ) {
        luaur_common::LUAU_ASSERT!(luaur_common::FFlag::LuauCstAttr.get());
        let open = *self.lexer.current();
        luaur_common::LUAU_ASSERT!(open.r#type == Type::AttributeOpen);
        self.next_lexeme();

        let empty: AstArray<*mut AstExpr> = AstArray::default();
        let mut comma_positions = TempVector::new(&mut self.scratch_position);

        if self.lexer.current().r#type != Type(']' as i32) {
            loop {
                let name = self.parse_name("attribute name");
                let name_loc = name.location;
                let attr_name = name.name.value;
                let arg_open = *self.lexer.current();
                let arg_open_type = arg_open.r#type;

                if arg_open_type == Type::RawString
                    || arg_open_type == Type::QuotedString
                    || arg_open_type == Type('{' as i32)
                    || arg_open_type == Type('(' as i32)
                {
                    let open_paren_position = if arg_open_type == Type('(' as i32) {
                        arg_open.location.begin
                    } else {
                        Position::missing()
                    };
                    let mut arg_comma_positions = TempVector::new(&mut self.scratch_position_2);
                    let mut close_paren_position = Position::missing();
                    let (args, args_location, _) = if self.options.store_cst_data {
                        self.parse_call_list(
                            &mut arg_comma_positions,
                            &mut close_paren_position,
                        )
                    } else {
                        self.parse_call_list(core::ptr::null_mut(), core::ptr::null_mut())
                    };

                    for arg in args.iter() {
                        if !is_constant_literal(*arg) && !is_literal_table(*arg) {
                            self.report(
                                args_location,
                                format_args!("Only literals can be passed as arguments for attributes"),
                            );
                        }
                    }

                    let attr_name_str =
                        unsafe { core::ffi::CStr::from_ptr(attr_name).to_string_lossy() };
                    let ty = self.validate_attribute(
                        name_loc,
                        &attr_name_str,
                        attributes,
                        &args,
                    );
                    let node = unsafe {
                        (*self.allocator).alloc(
                            AstAttr::ast_attr_location_type_item_ast_array_ast_expr_ast_name(
                                Location::new(name_loc.begin, args_location.end),
                                ty.unwrap_or(AstAttrType::Unknown),
                                args,
                                AstName { value: attr_name },
                            ),
                        )
                    };
                    if self.options.store_cst_data {
                        let cst_node = unsafe {
                            (*self.allocator).alloc(CstParametrizedAttr::new(
                                open_paren_position,
                                close_paren_position,
                                self.copy_temp_vector_t(&arg_comma_positions),
                            ))
                        };
                        self.cst_node_map.try_insert(
                            node as *mut crate::records::ast_node::AstNode,
                            cst_node as *mut CstNode,
                        );
                    }
                    attributes.push_back(node);
                } else {
                    let attr_name_str =
                        unsafe { core::ffi::CStr::from_ptr(attr_name).to_string_lossy() };
                    let ty = self.validate_attribute(
                        name_loc,
                        &attr_name_str,
                        attributes,
                        &empty,
                    );
                    let node = unsafe {
                        (*self.allocator).alloc(
                            AstAttr::ast_attr_location_type_item_ast_array_ast_expr_ast_name(
                                name_loc,
                                ty.unwrap_or(AstAttrType::Unknown),
                                empty,
                                AstName { value: attr_name },
                            ),
                        )
                    };
                    if self.options.store_cst_data {
                        let cst_node = unsafe { (*self.allocator).alloc(CstAttr::new(false)) };
                        self.cst_node_map.try_insert(
                            node as *mut crate::records::ast_node::AstNode,
                            cst_node as *mut CstNode,
                        );
                    }
                    attributes.push_back(node);
                }

                if self.lexer.current().r#type == Type(',' as i32) {
                    if self.options.store_cst_data {
                        comma_positions.push_back(self.lexer.current().location.begin);
                    }
                    self.next_lexeme();
                } else {
                    break;
                }
            }
        } else {
            let end_loc = self.lexer.current().location;
            self.report(
                Location::new(open.location.begin, end_loc.end),
                format_args!("Attribute list cannot be empty"),
            );
            let node = unsafe {
                (*self.allocator).alloc(
                    AstAttr::ast_attr_location_type_item_ast_array_ast_expr_ast_name(
                        Location::new(open.location.begin, end_loc.end),
                        AstAttrType::Unknown,
                        empty,
                        self.name_error,
                    ),
                )
            };
            if self.options.store_cst_data {
                let cst_node = unsafe { (*self.allocator).alloc(CstAttr::new(false)) };
                self.cst_node_map.try_insert(
                    node as *mut crate::records::ast_node::AstNode,
                    cst_node as *mut CstNode,
                );
            }
            attributes.push_back(node);
        }

        let closing_bracket_found =
            self.expect_match_and_consume(']', &MatchLexeme::new(&open), false);
        if self.options.store_cst_data {
            luaur_common::LUAU_ASSERT!(!cst_attr_lists.is_null());
            let close_bracket_position = if closing_bracket_found {
                self.lexer.previous_location().begin
            } else {
                Position::missing()
            };
            let cst_list = unsafe {
                (*self.allocator).alloc(CstAttrList::new(
                    open.location.begin,
                    close_bracket_position,
                    self.copy_temp_vector_t(&comma_positions),
                ))
            };
            unsafe { (*cst_attr_lists).push_back(cst_list) };
        }
    }
}
