use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_break::AstStatBreak;
use crate::records::ast_stat_continue::AstStatContinue;
use crate::records::ast_stat_return::AstStatReturn;

pub fn is_stat_last(stat: *mut AstStat) -> bool {
    crate::rtti::ast_node_is::<AstStatBreak>(unsafe {
        &*(stat as *mut crate::records::ast_node::AstNode)
    }) || crate::rtti::ast_node_is::<AstStatContinue>(unsafe {
        &*(stat as *mut crate::records::ast_node::AstNode)
    }) || crate::rtti::ast_node_is::<AstStatReturn>(unsafe {
        &*(stat as *mut crate::records::ast_node::AstNode)
    })
}
