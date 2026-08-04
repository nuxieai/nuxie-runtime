use crate::records::ast_node::AstNode;
use crate::records::ast_stat::AstStat;
use crate::rtti::AstNodeClass;

impl AstNode {
    #[inline]
    pub fn as_stat(&mut self) -> *mut AstStat {
        let is_stat = self.class_index == crate::records::ast_stat_assign::AstStatAssign::CLASS_INDEX
            || self.class_index == crate::records::ast_stat_block::AstStatBlock::CLASS_INDEX
            || self.class_index == crate::records::ast_stat_break::AstStatBreak::CLASS_INDEX
            || self.class_index == crate::records::ast_stat_class::AstStatClass::CLASS_INDEX
            || self.class_index == crate::records::ast_stat_compound_assign::AstStatCompoundAssign::CLASS_INDEX
            || self.class_index == crate::records::ast_stat_continue::AstStatContinue::CLASS_INDEX
            || self.class_index == crate::records::ast_stat_declare_extern_type::AstStatDeclareExternType::CLASS_INDEX
            || self.class_index == crate::records::ast_stat_declare_function::AstStatDeclareFunction::CLASS_INDEX
            || self.class_index == crate::records::ast_stat_declare_global::AstStatDeclareGlobal::CLASS_INDEX
            || self.class_index == crate::records::ast_stat_error::AstStatError::CLASS_INDEX
            || self.class_index == crate::records::ast_stat_expr::AstStatExpr::CLASS_INDEX
            || self.class_index == crate::records::ast_stat_for::AstStatFor::CLASS_INDEX
            || self.class_index == crate::records::ast_stat_for_in::AstStatForIn::CLASS_INDEX
            || self.class_index == crate::records::ast_stat_function::AstStatFunction::CLASS_INDEX
            || self.class_index == crate::records::ast_stat_if::AstStatIf::CLASS_INDEX
            || self.class_index == crate::records::ast_stat_local::AstStatLocal::CLASS_INDEX
            || self.class_index == crate::records::ast_stat_local_function::AstStatLocalFunction::CLASS_INDEX
            || self.class_index == crate::records::ast_stat_repeat::AstStatRepeat::CLASS_INDEX
            || self.class_index == crate::records::ast_stat_return::AstStatReturn::CLASS_INDEX
            || self.class_index == crate::records::ast_stat_type_alias::AstStatTypeAlias::CLASS_INDEX
            || self.class_index == crate::records::ast_stat_type_function::AstStatTypeFunction::CLASS_INDEX
            || self.class_index == crate::records::ast_stat_while::AstStatWhile::CLASS_INDEX;

        if is_stat {
            self as *mut AstNode as *mut AstStat
        } else {
            core::ptr::null_mut()
        }
    }

    #[inline]
    pub fn as_stat_const(&self) -> *const AstStat {
        let node = self as *const AstNode as *mut AstNode;
        unsafe { (*node).as_stat() as *const AstStat }
    }
}
