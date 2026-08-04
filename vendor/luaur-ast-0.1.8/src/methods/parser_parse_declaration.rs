use crate::enums::ast_table_access::AstTableAccess;
use crate::records::ast_array::AstArray;
use crate::records::ast_attr::AstAttr;
use crate::records::ast_declared_extern_type_property::AstDeclaredExternTypeProperty;
use crate::records::ast_name::AstName;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_declare_extern_type::AstStatDeclareExternType;
use crate::records::ast_stat_declare_function::AstStatDeclareFunction;
use crate::records::ast_stat_declare_global::AstStatDeclareGlobal;
use crate::records::ast_type_list::AstTypeList;
use crate::records::ast_type_pack_explicit::AstTypePackExplicit;
use crate::records::lexeme::Type;
use crate::records::location::Location;
use crate::records::match_lexeme::MatchLexeme;
use crate::records::name::Name;
use crate::records::parser::Parser;
use crate::records::temp_vector::TempVector;
use crate::type_aliases::ast_argument_name::AstArgumentName;
use luaur_common::FFlag;

impl Parser {
    pub fn parse_declaration(
        &mut self,
        start: &Location,
        attributes: &AstArray<*mut AstAttr>,
    ) -> *mut AstStat {
        // `declare` token is already parsed at this point

        if (attributes.size != 0) && (self.lexer.current().r#type != Type::ReservedFunction) {
            return self.report_stat_error(
                self.lexer.current().location,
                AstArray::default(),
                AstArray::default(),
                format_args!(
                    "Expected a function type declaration after attribute, but got {} instead",
                    self.lexer.current().to_string()
                ),
            ) as *mut AstStat;
        }

        if self.lexer.current().r#type == Type::ReservedFunction {
            self.next_lexeme();

            let global_name = self.parse_name("global function name");
            let (generics, generic_packs) = self.parse_generic_type_list(false, None, None, None);

            let match_paren = MatchLexeme::new(self.lexer.current());

            self.expect_and_consume_type(Type('(' as i32), "global function declaration");

            let mut args = TempVector::new(&mut self.scratch_binding);

            let mut vararg = false;
            let mut vararg_location = Location::default();
            let mut vararg_annotation = core::ptr::null_mut();

            if self.lexer.current().r#type != Type(')' as i32) {
                let res = self.parse_binding_list(
                    &mut args,
                    true,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    false,
                );
                vararg = res.0;
                vararg_location = res.1;
                vararg_annotation = res.2;
            }

            self.expect_match_and_consume(')', &match_paren, false);

            let mut ret_types = self.parse_optional_return_type(None);
            if ret_types.is_null() {
                ret_types = unsafe {
                    (*self.allocator).alloc(AstTypePackExplicit::new(
                        self.lexer.current().location,
                        AstTypeList::default(),
                    ))
                } as *mut crate::records::ast_type_pack::AstTypePack;
            }
            let end = self.lexer.current().location;

            let mut vars = TempVector::new(&mut self.scratch_type);
            let mut var_names = TempVector::new(&mut self.scratch_arg_name);

            for i in 0..args.size() {
                if args[i].annotation.is_null() {
                    return self.report_stat_error(
                        Location::new(start.begin, end.end),
                        AstArray::default(),
                        AstArray::default(),
                        format_args!("All declaration parameters must be annotated"),
                    ) as *mut AstStat;
                }

                vars.push_back(args[i].annotation);
                var_names.push_back((args[i].name.name, args[i].name.location));
            }

            if vararg && vararg_annotation.is_null() {
                return self.report_stat_error(
                    Location::new(start.begin, end.end),
                    AstArray::default(),
                    AstArray::default(),
                    format_args!("All declaration parameters must be annotated"),
                ) as *mut AstStat;
            }

            unsafe {
                (*self.allocator).alloc(AstStatDeclareFunction {
                    base: crate::records::ast_stat::AstStat::new(
                        <AstStatDeclareFunction as crate::rtti::AstNodeClass>::CLASS_INDEX,
                        Location::new(start.begin, end.end),
                    ),
                    attributes: *attributes,
                    name: global_name.name,
                    name_location: global_name.location,
                    generics,
                    generic_packs,
                    params: AstTypeList {
                        types: self.copy_temp_vector_t(&vars),
                        tail_type: vararg_annotation,
                    },
                    param_names: self.copy_temp_vector_t(&var_names),
                    vararg,
                    vararg_location,
                    ret_types,
                }) as *mut AstStat
            }
        } else if (unsafe {
            AstName::ast_name_c_char(self.lexer.current().data.name)
                .operator_eq_c_char(c"class".as_ptr())
        } && (if FFlag::LuauAllowGlobalDeclarationToBeCalledClass.get() {
            self.lexer.lookahead().r#type != Type(':' as i32)
        } else {
            true
        })) || unsafe {
            AstName::ast_name_c_char(self.lexer.current().data.name)
                .operator_eq_c_char(c"extern".as_ptr())
        } {
            let mut found_extern = false;
            if unsafe {
                AstName::ast_name_c_char(self.lexer.current().data.name)
                    .operator_eq_c_char(c"extern".as_ptr())
            } {
                found_extern = true;
                self.next_lexeme();
                if unsafe {
                    AstName::ast_name_c_char(self.lexer.current().data.name)
                        .operator_ne_c_char(c"type".as_ptr())
                } {
                    return self.report_stat_error(
                        self.lexer.current().location,
                        AstArray::default(),
                        AstArray::default(),
                        format_args!(
                            "Expected `type` keyword after `extern`, but got {} instead",
                            unsafe {
                                core::ffi::CStr::from_ptr(self.lexer.current().data.name)
                                    .to_string_lossy()
                            }
                        ),
                    ) as *mut AstStat;
                }
            }

            self.next_lexeme();

            let class_start = self.lexer.current().location;
            let class_name = self.parse_name("type name");
            let mut super_name: Option<AstName> = None;

            if unsafe {
                AstName::ast_name_c_char(self.lexer.current().data.name)
                    .operator_eq_c_char(c"extends".as_ptr())
            } {
                self.next_lexeme();
                super_name = Some(self.parse_name("supertype name").name);
            }

            if found_extern {
                if unsafe {
                    AstName::ast_name_c_char(self.lexer.current().data.name)
                        .operator_ne_c_char(c"with".as_ptr())
                } {
                    self.report(
                        self.lexer.current().location,
                        format_args!(
                            "Expected `with` keyword before listing properties of the external type, but got {} instead",
                            unsafe { core::ffi::CStr::from_ptr(self.lexer.current().data.name).to_string_lossy() }
                        )
                    );
                } else {
                    self.next_lexeme();
                }
            }

            let mut props = TempVector::new(&mut self.scratch_declared_class_props);
            let mut indexer: *mut crate::records::ast_table_indexer::AstTableIndexer =
                core::ptr::null_mut();

            while self.lexer.current().r#type != Type::ReservedEnd {
                let mut attributes = AstArray::<*mut AstAttr>::default();

                if self.lexer.current().r#type == Type::Attribute
                    || self.lexer.current().r#type == Type::AttributeOpen
                {
                    attributes = self.parse_attributes();

                    if self.lexer.current().r#type != Type::ReservedFunction {
                        return self.report_stat_error(
                            self.lexer.current().location,
                            AstArray::default(),
                            AstArray::default(),
                            format_args!(
                                "Expected a method type declaration after attribute, but got {} instead",
                                self.lexer.current().to_string()
                            ),
                        ) as *mut AstStat;
                    }
                }

                // There are two possibilities: Either it's a property or a function.
                if self.lexer.current().r#type == Type::ReservedFunction {
                    props.push_back(self.parse_declared_extern_type_method(&attributes));
                } else if self.lexer.current().r#type == Type('[' as i32) {
                    let begin = self.lexer.current().clone();
                    self.next_lexeme(); // [

                    if (self.lexer.current().r#type == Type::RawString
                        || self.lexer.current().r#type == Type::QuotedString)
                        && self.lexer.lookahead().r#type == Type(']' as i32)
                    {
                        let name_begin = self.lexer.current().location;
                        let chars = self.parse_char_array(None);

                        let name_end = *self.lexer.previous_location();

                        self.expect_match_and_consume(']', &MatchLexeme::new(&begin), false);
                        self.expect_and_consume_char(':', "property type annotation");
                        let ty = self.parse_type_bool(false);

                        // since AstName contains a char*, it can't contain null
                        let mut contains_null = false;
                        if let Some(ref c) = chars {
                            for &ch in c.as_slice() {
                                if ch == 0 {
                                    contains_null = true;
                                    break;
                                }
                            }
                        }

                        if chars.is_some() && !contains_null {
                            props.push_back(AstDeclaredExternTypeProperty {
                                name: AstName {
                                    value: chars.unwrap().data,
                                },
                                name_location: Location::new(name_begin.begin, name_end.end),
                                ty,
                                is_method: false,
                                location: Location::new(
                                    begin.location.begin,
                                    self.lexer.previous_location().end,
                                ),
                                access: AstTableAccess::ReadWrite,
                            });
                        } else {
                            self.report(
                                begin.location,
                                format_args!(
                                    "String literal contains malformed escape sequence or \\0"
                                ),
                            );
                        }
                    } else if !indexer.is_null() {
                        let bad_indexer_res =
                            self.parse_table_indexer(AstTableAccess::ReadWrite, None, begin);
                        let bad_indexer = bad_indexer_res.node;
                        self.report(
                            unsafe { (*bad_indexer).location },
                            format_args!("Cannot have more than one indexer on an extern type"),
                        );
                    } else {
                        indexer = self
                            .parse_table_indexer(AstTableAccess::ReadWrite, None, begin)
                            .node;
                    }
                } else {
                    let mut access = AstTableAccess::ReadWrite;

                    if self.lexer.current().r#type == Type::Name
                        && self.lexer.lookahead().r#type != Type(':' as i32)
                    {
                        let current_name = unsafe { self.lexer.current().data.name };
                        if AstName::ast_name_c_char(current_name)
                            .operator_eq_c_char(c"read".as_ptr())
                        {
                            access = AstTableAccess::Read;
                            self.lexer.next();
                        } else if AstName::ast_name_c_char(current_name)
                            .operator_eq_c_char(c"write".as_ptr())
                        {
                            access = AstTableAccess::Write;
                            self.lexer.next();
                        } else {
                            self.report(
                                self.lexer.current().location,
                                format_args!(
                                    "Expected blank or 'read' or 'write' attribute, got '{}'",
                                    unsafe {
                                        core::ffi::CStr::from_ptr(self.lexer.current().data.name)
                                            .to_string_lossy()
                                    }
                                ),
                            );
                            self.lexer.next();
                        }
                    }

                    let prop_start = self.lexer.current().location;
                    let prop_name = self.parse_name_opt("property name");

                    if prop_name.is_none() {
                        break;
                    }
                    let prop_name = prop_name.unwrap();

                    self.expect_and_consume_char(':', "property type annotation");
                    let prop_type = self.parse_type_bool(false);
                    props.push_back(AstDeclaredExternTypeProperty {
                        name: prop_name.name,
                        name_location: prop_name.location,
                        ty: prop_type,
                        is_method: false,
                        location: Location::new(
                            prop_start.begin,
                            self.lexer.previous_location().end,
                        ),
                        access,
                    });
                }
            }

            let class_end = self.lexer.current().location;
            self.next_lexeme(); // skip past `end`

            unsafe {
                (*self.allocator).alloc(AstStatDeclareExternType::new(
                    Location::new(class_start.begin, class_end.end),
                    class_name.name,
                    super_name,
                    self.copy_temp_vector_t(&props),
                    indexer,
                )) as *mut AstStat
            }
        } else if let Some(global_name) = self.parse_name_opt("global variable name") {
            self.expect_and_consume_char(':', "global variable declaration");

            let ty = self.parse_type_bool(true);
            unsafe {
                (*self.allocator).alloc(AstStatDeclareGlobal::new(
                    Location::new(start.begin, (*ty).base.location.end),
                    global_name.name,
                    global_name.location,
                    ty,
                )) as *mut AstStat
            }
        } else {
            self.report_stat_error(
                *start,
                AstArray::default(),
                AstArray::default(),
                format_args!(
                    "declare must be followed by an identifier, 'function', or 'extern type'"
                ),
            ) as *mut AstStat
        }
    }
}
