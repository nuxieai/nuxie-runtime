use crate::records::table_mutation_tracker::TableMutationTracker;
use luaur_ast::records::ast_expr::AstExpr;

#[allow(non_snake_case)]
pub fn table_mutation_tracker_mark_escaped_table_index(
    tracker: &mut TableMutationTracker,
    expr: *mut AstExpr,
    is_lvalue: bool,
) {
    tracker.mark_escaped_table_index(expr, is_lvalue);
}
