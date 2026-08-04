//! Node: `cxx:Method:Luau.Ast:Ast/src/Parser.cpp:1683:parseDeclaredExternTypeMethod`
//!
//! Faithful port of `Parser::parseDeclaredExternTypeMethod` — parse one method
//! signature inside a `declare class`: `name(self, args...): ret`. The first
//! parameter must be an unannotated `self`; every other must be annotated. The
//! result is an `AstTypeFunction` wrapped in an `AstDeclaredExternTypeProperty`
//! (generic method declarations are not yet supported, matching the C++ TODO).

use crate::enums::ast_table_access::AstTableAccess;
use crate::records::allocator::Allocator;
use crate::records::ast_array::AstArray;
use crate::records::ast_attr::AstAttr;
use crate::records::ast_declared_extern_type_property::AstDeclaredExternTypeProperty;
use crate::records::ast_generic_type::AstGenericType;
use crate::records::ast_generic_type_pack::AstGenericTypePack;
use crate::records::ast_type::AstType;
use crate::records::ast_type_function::AstTypeFunction;
use crate::records::ast_type_list::AstTypeList;
use crate::records::ast_type_pack::AstTypePack;
use crate::records::ast_type_pack_explicit::AstTypePackExplicit;
use crate::records::lexeme::Type;
use crate::records::location::Location;
use crate::records::match_lexeme::MatchLexeme;
use crate::records::parser::Parser;
use crate::records::temp_vector::TempVector;

impl Parser {
    pub fn parse_declared_extern_type_method(
        &mut self,
        attributes: &AstArray<*mut AstAttr>,
    ) -> AstDeclaredExternTypeProperty {
        let start = self.lexer.current().location;

        self.next_lexeme();

        let fn_name = self.parse_name("function name");

        // TODO: generic method declarations CLI-39909
        let generics: AstArray<*mut AstGenericType> = AstArray {
            data: core::ptr::null_mut(),
            size: 0,
        };
        let generic_packs: AstArray<*mut AstGenericTypePack> = AstArray {
            data: core::ptr::null_mut(),
            size: 0,
        };

        let match_paren = *self.lexer.current();
        self.expect_and_consume_char('(', "function parameter list start");

        let mut args = TempVector::new(&mut self.scratch_binding);

        let mut vararg = false;
        let mut vararg_annotation: *mut AstTypePack = core::ptr::null_mut();
        if self.lexer.current().r#type != Type(b')' as i32) {
            let (v, _vl, va) = self.parse_binding_list(
                &mut args,
                true,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                false,
            );
            vararg = v;
            vararg_annotation = va;
        }

        self.expect_match_and_consume(')', &MatchLexeme::new(&match_paren), false);

        let mut ret_types = self.parse_optional_return_type(None);
        if ret_types.is_null() {
            let loc = self.lexer.current().location;
            let type_list = AstTypeList {
                types: AstArray {
                    data: core::ptr::null_mut(),
                    size: 0,
                },
                tail_type: core::ptr::null_mut(),
            };
            ret_types = unsafe { (*self.allocator).alloc(AstTypePackExplicit::new(loc, type_list)) }
                as *mut AstTypePack;
        }
        let end = *self.lexer.previous_location();

        let mut vars = TempVector::new(&mut self.scratch_type);
        let mut var_names = TempVector::new(&mut self.scratch_opt_arg_name);

        if args.size() == 0
            || args
                .operator_index(0)
                .name
                .name
                .operator_ne_c_char(c"self".as_ptr())
            || !args.operator_index(0).annotation.is_null()
        {
            let err = self.report_type_error(
                Location::new(start.begin, end.end),
                AstArray {
                    data: core::ptr::null_mut(),
                    size: 0,
                },
                format_args!("'self' must be present as the unannotated first parameter"),
            );
            return AstDeclaredExternTypeProperty {
                name: fn_name.name,
                name_location: fn_name.location,
                ty: err as *mut AstType,
                is_method: true,
                location: Location::default(),
                access: AstTableAccess::ReadWrite,
            };
        }

        // Skip the first index ('self').
        for i in 1..args.size() {
            let arg_name = (
                args.operator_index(i).name.name,
                args.operator_index(i).name.location,
            );
            var_names.push_back(Some(arg_name));

            let annotation = args.operator_index(i).annotation;
            if !annotation.is_null() {
                vars.push_back(annotation);
            } else {
                let err = self.report_type_error(
                    Location::new(start.begin, end.end),
                    AstArray {
                        data: core::ptr::null_mut(),
                        size: 0,
                    },
                    format_args!("All declaration parameters aside from 'self' must be annotated"),
                );
                vars.push_back(err as *mut AstType);
            }
        }

        if vararg && vararg_annotation.is_null() {
            self.report(
                start,
                format_args!("All declaration parameters aside from 'self' must be annotated"),
            );
        }

        let arg_types = AstTypeList {
            types: self.copy_temp_vector_t(&vars),
            tail_type: vararg_annotation,
        };
        let arg_names = self.copy_temp_vector_t(&var_names);

        let fn_type = unsafe {
            (*self.allocator).alloc(AstTypeFunction::ast_type_function_location_ast_array_ast_attr_ast_array_ast_generic_type_ast_array_ast_generic_type_pack_ast_type_list_ast_array_optional_ast_argument_name_ast_type_pack(
                Location::new(start.begin, end.end),
                *attributes,
                generics,
                generic_packs,
                arg_types,
                arg_names,
                ret_types,
            )) as *mut AstType
        };

        AstDeclaredExternTypeProperty {
            name: fn_name.name,
            name_location: fn_name.location,
            ty: fn_type,
            is_method: true,
            location: Location::new(start.begin, end.end),
            access: AstTableAccess::ReadWrite,
        }
    }
}
