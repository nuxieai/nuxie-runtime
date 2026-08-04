use core::ffi::c_char;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_call::AstExprCall;
use luaur_ast::records::ast_expr_global::AstExprGlobal;
use luaur_ast::records::ast_expr_table::AstExprTable;
use luaur_ast::records::ast_name::AstName;
use luaur_ast::records::ast_node::AstNode;
use luaur_ast::rtti;

pub fn get_table_hint(expr: *mut AstExpr) -> *mut AstExprTable {
    if expr.is_null() {
        return core::ptr::null_mut();
    }

    let table = unsafe { rtti::ast_node_as::<AstExprTable>(expr as *mut AstNode) };
    if !table.is_null() {
        return table;
    }

    let call = unsafe { rtti::ast_node_as::<AstExprCall>(expr as *mut AstNode) };
    if !call.is_null() {
        let call_ref = unsafe { &*call };
        if !call_ref.self_ && call_ref.args.size == 2 {
            let func = unsafe { rtti::ast_node_as::<AstExprGlobal>(call_ref.func as *mut AstNode) };
            if !func.is_null() {
                let func_ref = unsafe { &*func };
                if func_ref.name.operator_eq_c_char(c"setmetatable".as_ptr()) {
                    let arg0 = unsafe { *call_ref.args.data.add(0) };
                    let table_arg =
                        unsafe { rtti::ast_node_as::<AstExprTable>(arg0 as *mut AstNode) };
                    if !table_arg.is_null() {
                        return table_arg;
                    }
                }
            }
        }
    }

    core::ptr::null_mut()
}
