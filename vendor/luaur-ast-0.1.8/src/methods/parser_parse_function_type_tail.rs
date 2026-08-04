use crate::records::ast_array::AstArray;
use crate::records::ast_attr::AstAttr;
use crate::records::ast_generic_type::AstGenericType;
use crate::records::ast_generic_type_pack::AstGenericTypePack;
use crate::records::ast_type::AstType;
use crate::records::ast_type_function::AstTypeFunction;
use crate::records::ast_type_list::AstTypeList;
use crate::records::ast_type_pack::AstTypePack;
use crate::records::lexeme::Lexeme;
use crate::records::lexeme::Type;
use crate::records::location::Location;
use crate::records::parser::Parser;
use crate::type_aliases::ast_argument_name::AstArgumentName;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Parser {
    pub fn parse_function_type_tail(
        &mut self,
        begin: &Lexeme,
        attributes: AstArray<*mut AstAttr>,
        generics: AstArray<*mut AstGenericType>,
        generic_packs: AstArray<*mut AstGenericTypePack>,
        params: AstArray<*mut AstType>,
        param_names: AstArray<Option<AstArgumentName>>,
        vararg_annotation: *mut AstTypePack,
    ) -> *mut AstType {
        self.increment_recursion_counter("type annotation");

        if self.lexer.current().r#type == Type(':' as i32) {
            self.report_location_c_char_item(self.lexer.current().location, format_args!("Return types in function type annotations are written after '->' instead of ':'"));
            self.next_lexeme();
        } else if self.lexer.current().r#type != Type::SkinnyArrow
            && generics.size == 0
            && generic_packs.size == 0
            && params.size == 0
        {
            self.report_location_c_char_item(
                Location::new(begin.location.begin, self.lexer.previous_location().end),
                format_args!(
                    "Expected '->' after '()' when parsing function type; did you mean 'nil'?"
                ),
            );
            return unsafe {
                (*self.allocator).alloc(crate::records::ast_type_reference::AstTypeReference::new(
                    begin.location,
                    None,
                    self.name_nil,
                    None,
                    begin.location,
                    false,
                    AstArray {
                        data: core::ptr::null_mut(),
                        size: 0,
                    },
                )) as *mut AstType
            };
        } else {
            self.expect_and_consume_type(Type::SkinnyArrow, "function type");
        }

        let return_type = self.parse_return_type();
        LUAU_ASSERT!(return_type.is_null() == false);

        unsafe {
            (*self.allocator).alloc(AstTypeFunction::ast_type_function_location_ast_array_ast_attr_ast_array_ast_generic_type_ast_array_ast_generic_type_pack_ast_type_list_ast_array_optional_ast_argument_name_ast_type_pack(
            Location::new(begin.location.begin, (*return_type).base.location.end),
            attributes,
            generics,
            generic_packs,
            AstTypeList { types: params, tail_type: vararg_annotation },
            param_names,
            return_type,
        )) as *mut AstType
        }
    }
}
