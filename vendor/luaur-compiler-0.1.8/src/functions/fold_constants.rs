use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_call::AstExprCall;
use luaur_ast::records::ast_local::AstLocal;
use luaur_ast::records::ast_name_table::AstNameTable;
use luaur_ast::records::ast_node::AstNode;
use luaur_common::records::dense_hash_map::DenseHashMap;

use crate::enums::table_constant_kind::TableConstantKind;
use crate::records::constant::Constant;
use crate::records::constant_visitor::ConstantVisitor;
use crate::records::variable::Variable;
use crate::type_aliases::expr_constant_change_log::ExprConstantChangeLog;
use crate::type_aliases::library_member_constant_callback::LibraryMemberConstantCallback;
use crate::type_aliases::local_constant_change_log::LocalConstantChangeLog;

pub fn fold_constants(
    constants: &mut DenseHashMap<*mut AstExpr, Constant>,
    variables: &mut DenseHashMap<*mut AstLocal, Variable>,
    locals: &mut DenseHashMap<*mut AstLocal, Constant>,
    builtins: *const DenseHashMap<*mut AstExprCall, i32>,
    fold_library_k: bool,
    library_member_constant_cb: LibraryMemberConstantCallback,
    root: *mut AstNode,
    string_table: &mut AstNameTable,
    table_constants: &DenseHashMap<*mut AstLocal, TableConstantKind>,
    expr_change_log: *mut ExprConstantChangeLog,
    local_change_log: *mut LocalConstantChangeLog,
) {
    let mut visitor = ConstantVisitor::constant_visitor(
        constants,
        variables,
        locals,
        builtins,
        fold_library_k,
        library_member_constant_cb,
        string_table,
        table_constants,
        expr_change_log,
        local_change_log,
    );

    unsafe {
        luaur_ast::visit::ast_node_visit(root, &mut visitor);
    }
}
