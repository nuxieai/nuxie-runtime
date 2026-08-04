use crate::records::constant::Constant;
use crate::records::constant_visitor::ConstantVisitor;
use crate::records::variable::Variable;
use crate::type_aliases::expr_constant_change_log::ExprConstantChangeLog;
use crate::type_aliases::library_member_constant_callback::LibraryMemberConstantCallback;
use crate::type_aliases::local_constant_change_log::LocalConstantChangeLog;
use crate::enums::table_constant_kind::TableConstantKind;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_call::AstExprCall;
use luaur_ast::records::ast_local::AstLocal;
use luaur_ast::records::ast_name_table::AstNameTable;
use luaur_common::records::dense_hash_map::DenseHashMap;

impl<'a> ConstantVisitor<'a> {
    pub fn constant_visitor(
        constants: &'a mut DenseHashMap<*mut AstExpr, Constant>,
        variables: &'a mut DenseHashMap<*mut AstLocal, Variable>,
        locals: &'a mut DenseHashMap<*mut AstLocal, Constant>,
        builtins: *const DenseHashMap<*mut AstExprCall, i32>,
        fold_library_k: bool,
        library_member_constant_cb: LibraryMemberConstantCallback,
        string_table: &'a mut AstNameTable,
        constant_table_locals: &'a DenseHashMap<*mut AstLocal, TableConstantKind>,
        expr_change_log: *mut ExprConstantChangeLog,
        local_change_log: *mut LocalConstantChangeLog,
    ) -> Self {
        let was_empty = constants.empty() && locals.empty();
        Self {
            constants,
            variables,
            locals,
            builtins,
            fold_library_k,
            library_member_constant_cb,
            string_table,
            constant_tables: alloc::vec::Vec::new(),
            was_empty,
            builtin_args: alloc::vec::Vec::new(),
            constant_table_locals,
            table_locals: DenseHashMap::new(core::ptr::null_mut()),
            expr_change_log,
            local_change_log,
        }
    }
}
