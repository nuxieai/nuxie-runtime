use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_call::AstExprCall;
use luaur_ast::records::ast_local::AstLocal;
use luaur_ast::records::ast_name_table::AstNameTable;
use luaur_ast::records::ast_node::AstNode;
use luaur_common::records::dense_hash_map::DenseHashMap;
use luaur_common::FFlag::LuauCompileFoldOptimize;
use luaur_common::FFlag::LuauCompilePropagateTableProps2;
use luaur_common::LUAU_ASSERT;

use crate::enums::table_constant_kind::TableConstantKind;
use crate::records::constant::Constant;
use crate::records::constant_visitor::ConstantVisitor;
use crate::records::table_mutation_tracker_deprecated::TableMutationTrackerDeprecated;
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
    let mut constant_tables_deprecated = DenseHashMap::new(core::ptr::null_mut::<AstLocal>());

    if LuauCompilePropagateTableProps2.get() && !LuauCompileFoldOptimize.get() {
        let mut mutation_tracker = TableMutationTrackerDeprecated {
            constant_tables: &mut constant_tables_deprecated,
            variables,
        };
        unsafe {
            luaur_ast::visit::ast_node_visit(root, &mut mutation_tracker);
        }
    }

    let constant_tables_for_visitor = if LuauCompileFoldOptimize.get() {
        table_constants
    } else {
        &constant_tables_deprecated
    };

    let mut visitor = ConstantVisitor::constant_visitor(
        constants,
        variables,
        locals,
        builtins,
        fold_library_k,
        library_member_constant_cb,
        string_table,
        constant_tables_for_visitor,
        expr_change_log,
        local_change_log,
    );

    unsafe {
        luaur_ast::visit::ast_node_visit(root, &mut visitor);
    }

    if LuauCompilePropagateTableProps2.get() && !LuauCompileFoldOptimize.get() {
        for (_key, constant) in constants.iter_mut() {
            if constant.r#type == crate::enums::type_constant_folding::Type::Type_Table {
                constant.r#type = crate::enums::type_constant_folding::Type::Type_Unknown;
            }
        }

        for (_key, constant) in locals.iter_mut() {
            if constant.r#type == crate::enums::type_constant_folding::Type::Type_Table {
                constant.r#type = crate::enums::type_constant_folding::Type::Type_Unknown;
            }
        }
    }
}
