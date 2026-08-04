use crate::functions::is_constant_literal::is_constant_literal;
use crate::functions::is_literal_table::is_literal_table;
use crate::records::ast_array::AstArray;
use crate::records::ast_attr::{AstAttr, AstAttrType};
use crate::records::ast_expr::AstExpr;
use crate::records::ast_name::AstName;
use crate::records::lexeme::Type;
use crate::records::location::Location;
use crate::records::match_lexeme::MatchLexeme;
use crate::records::parser::Parser;
use crate::records::temp_vector::TempVector;

impl Parser {
    pub fn parse_attribute_deprecated(
        &mut self,
        attributes: &mut TempVector<'_, *mut AstAttr>,
    ) {
        luaur_common::LUAU_ASSERT!(!luaur_common::FFlag::LuauCstAttr.get());

        let empty: AstArray<*mut AstExpr> = AstArray::default();
        luaur_common::LUAU_ASSERT!(
            self.lexer.current().r#type == Type::Attribute
                || self.lexer.current().r#type == Type::AttributeOpen
        );

        if self.lexer.current().r#type == Type::Attribute {
            let loc = self.lexer.current().location;
            let name = unsafe { self.lexer.current().data.name };
            let name_str = unsafe { core::ffi::CStr::from_ptr(name).to_string_lossy() };
            let ty = self.validate_attribute(loc, &name_str, attributes, &empty);
            self.next_lexeme();
            let node = unsafe {
                (*self.allocator).alloc(
                    AstAttr::ast_attr_location_type_item_ast_array_ast_expr_ast_name(
                        loc,
                        ty.unwrap_or(AstAttrType::Unknown),
                        empty,
                        AstName { value: name },
                    ),
                )
            };
            attributes.push_back(node);
        } else {
            let open = *self.lexer.current();
            self.next_lexeme();

            if self.lexer.current().r#type != Type(']' as i32) {
                loop {
                    let name = self.parse_name("attribute name");
                    let name_loc = name.location;
                    let attr_name = name.name.value;
                    let current_type = self.lexer.current().r#type;

                    if current_type == Type::RawString
                        || current_type == Type::QuotedString
                        || current_type == Type('{' as i32)
                        || current_type == Type('(' as i32)
                    {
                        let (args, args_location, _) =
                            self.parse_call_list(core::ptr::null_mut(), core::ptr::null_mut());
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
                        attributes.push_back(node);
                    }

                    if self.lexer.current().r#type == Type(',' as i32) {
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
                attributes.push_back(node);
            }

            self.expect_match_and_consume(']', &MatchLexeme::new(&open), false);
        }
    }
}
