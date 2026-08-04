use crate::records::ast_local::AstLocal;
use crate::records::ast_name::AstName;
use crate::records::ast_type::AstType;
use crate::records::location::Location;

impl AstLocal {
    pub fn new(
        name: AstName,
        location: Location,
        shadow: *mut AstLocal,
        function_depth: usize,
        loop_depth: usize,
        annotation: *mut AstType,
        is_const: bool,
    ) -> Self {
        Self {
            name,
            location,
            shadow,
            function_depth,
            loop_depth,
            is_const,
            is_exported: false,
            annotation,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_local_ast_local(
    name: AstName,
    location: Location,
    shadow: *mut AstLocal,
    function_depth: usize,
    loop_depth: usize,
    annotation: *mut AstType,
    is_const: bool,
) -> AstLocal {
    AstLocal::new(
        name,
        location,
        shadow,
        function_depth,
        loop_depth,
        annotation,
        is_const,
    )
}
