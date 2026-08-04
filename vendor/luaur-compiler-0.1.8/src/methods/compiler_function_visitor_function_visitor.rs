use crate::records::function_visitor::FunctionVisitor;
use luaur_ast::records::ast_expr_function::AstExprFunction;

pub fn compiler_function_visitor_function_visitor<'a>(
    functions: &'a mut Vec<*mut AstExprFunction>,
) -> FunctionVisitor<'a> {
    // preallocate the result; this works around std::vector's inefficient growth policy for small arrays
    functions.reserve(16);

    FunctionVisitor {
        functions,
        has_types: false,
        has_native_function: false,
    }
}
