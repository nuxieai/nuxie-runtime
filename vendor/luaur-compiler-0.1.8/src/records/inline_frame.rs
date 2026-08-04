use luaur_ast::records::ast_expr_function::AstExprFunction;

#[derive(Debug, Clone)]
pub struct InlineFrame {
    pub(crate) func: *mut AstExprFunction,
    pub(crate) local_offset: usize,
    pub(crate) target: u8,
    pub(crate) target_count: u8,
    pub(crate) return_jumps: alloc::vec::Vec<usize>,
}
