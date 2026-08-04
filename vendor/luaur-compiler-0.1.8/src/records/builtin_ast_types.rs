#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub struct BuiltinAstTypes {
    pub boolean_type: luaur_ast::records::ast_type_reference::AstTypeReference,
    pub number_type: luaur_ast::records::ast_type_reference::AstTypeReference,
    pub integer_type: luaur_ast::records::ast_type_reference::AstTypeReference,
    pub string_type: luaur_ast::records::ast_type_reference::AstTypeReference,
    pub vector_type: luaur_ast::records::ast_type_reference::AstTypeReference,
    pub host_vector_type: luaur_ast::records::ast_type_reference::AstTypeReference,
}

impl Default for BuiltinAstTypes {
    fn default() -> Self {
        use luaur_ast::records::ast_name::AstName;
        use luaur_ast::records::ast_type_reference::AstTypeReference;
        use luaur_ast::records::location::Location;
        use luaur_ast::records::position::Position;

        let loc = Location::new(Position::new(0, 0), Position::new(0, 0));
        let empty_arr = luaur_ast::records::ast_array::AstArray::default();

        let make_ref = |name: &core::ffi::CStr| {
            AstTypeReference::new(
                loc,
                None,
                AstName::ast_name_c_char(name.as_ptr()),
                None,
                loc,
                false,
                empty_arr.clone(),
            )
        };

        Self {
            boolean_type: make_ref(c"boolean"),
            number_type: make_ref(c"number"),
            integer_type: make_ref(c"integer"),
            string_type: make_ref(c"string"),
            vector_type: make_ref(c"vector"),
            host_vector_type: make_ref(c"vector"),
        }
    }
}
