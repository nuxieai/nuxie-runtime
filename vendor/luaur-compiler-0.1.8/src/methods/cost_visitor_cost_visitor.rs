use crate::records::constant::Constant;
use crate::records::cost::Cost;
use crate::records::cost_visitor::CostVisitor;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_call::AstExprCall;
use luaur_common::records::dense_hash_map::DenseHashMap;

impl CostVisitor {
    pub fn cost_visitor(
        builtins: &DenseHashMap<*mut AstExprCall, i32>,
        constants: &DenseHashMap<*mut AstExpr, Constant>,
    ) -> Self {
        Self {
            builtins: builtins as *const DenseHashMap<*mut AstExprCall, i32>,
            constants: constants as *const DenseHashMap<*mut AstExpr, Constant>,
            vars: DenseHashMap::new(core::ptr::null_mut()),
            result: Cost::default(),
        }
    }
}
