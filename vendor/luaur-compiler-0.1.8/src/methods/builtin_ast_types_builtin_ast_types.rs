use crate::records::builtin_ast_types::BuiltinAstTypes;
use core::ffi::c_char;
use luaur_ast::records::ast_array::AstArray;
use luaur_ast::records::ast_name::AstName;
use luaur_ast::records::ast_type_reference::AstTypeReference;
use luaur_ast::records::location::Location;

impl BuiltinAstTypes {
    pub fn new(host_vector_type_name: *const c_char) -> Self {
        // C++ initializes each builtin as `AstTypeReference{{}, nullopt, AstName{"<name>"}, nullopt, {}}`,
        // whose ctor stamps the AstTypeReference RTTI class index. A prior port zero-filled an
        // `empty_ref` (class_index 0, empty name) and cloned it, so get_type failed to recognize the
        // builtins as type references and resolved every literal to ANY — disabling number/vector type
        // info (e.g. the commutative ADDK/MULK optimization).
        let loc = Location::default();
        let empty_arr = AstArray::default();

        let make_ref = |name: AstName| {
            AstTypeReference::new(loc, None, name, None, loc, false, empty_arr.clone())
        };

        Self {
            boolean_type: make_ref(AstName::ast_name_c_char(c"boolean".as_ptr())),
            number_type: make_ref(AstName::ast_name_c_char(c"number".as_ptr())),
            integer_type: make_ref(AstName::ast_name_c_char(c"integer".as_ptr())),
            string_type: make_ref(AstName::ast_name_c_char(c"string".as_ptr())),
            vector_type: make_ref(AstName::ast_name_c_char(c"vector".as_ptr())),
            host_vector_type: make_ref(AstName::ast_name_c_char(host_vector_type_name)),
        }
    }
}
