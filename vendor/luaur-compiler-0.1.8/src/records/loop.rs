use luaur_ast::records::ast_stat_continue::AstStatContinue;

#[derive(Debug, Clone, Copy)]
pub struct Loop {
    pub(crate) local_offset: usize,
    pub(crate) local_offset_continue: usize,
    pub(crate) continue_used: *mut AstStatContinue,
}
