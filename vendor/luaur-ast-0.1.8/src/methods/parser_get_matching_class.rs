use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_global::AstExprGlobal;
use crate::records::ast_node::AstNode;
use crate::records::ast_stat_class::AstStatClass;
use crate::records::parser::Parser;

impl Parser {
    pub fn get_matching_class(&self, expr: *mut AstExpr) -> *mut AstStatClass {
        luaur_common::LUAU_ASSERT!(luaur_common::FFlag::DebugLuauUserDefinedClasses.get());
        let global = unsafe { crate::rtti::ast_node_as::<AstExprGlobal>(expr as *mut AstNode) };
        if !global.is_null() {
            if let Some(class_decl) = self.classes_within_module.find(unsafe { &(*global).name }) {
                return *class_decl;
            }
        }
        core::ptr::null_mut()
    }
}
