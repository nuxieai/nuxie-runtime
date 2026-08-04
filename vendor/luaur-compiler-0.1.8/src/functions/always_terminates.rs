use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_node::AstNode;
use luaur_ast::records::ast_stat::AstStat;
use luaur_ast::records::ast_stat_block::AstStatBlock;
use luaur_ast::records::ast_stat_break::AstStatBreak;
use luaur_ast::records::ast_stat_continue::AstStatContinue;
use luaur_ast::records::ast_stat_if::AstStatIf;
use luaur_ast::records::ast_stat_return::AstStatReturn;
use luaur_common::records::dense_hash_map::DenseHashMap;

use crate::functions::is_constant_false::is_constant_false;
use crate::functions::is_constant_true::is_constant_true;
use crate::records::constant::Constant;

pub fn always_terminates(
    constants: &DenseHashMap<*mut AstExpr, Constant>,
    node: *mut AstStat,
) -> bool {
    unsafe {
        let stat_block = luaur_ast::rtti::ast_node_as::<AstStatBlock>(node as *mut AstNode);
        if !stat_block.is_null() {
            let body = (*stat_block).body;
            for i in 0..body.size {
                let item = *body.data.add(i);
                if always_terminates(constants, item) {
                    return true;
                }
            }
            return false;
        }

        if luaur_ast::rtti::ast_node_is::<AstStatReturn>(&*(node as *mut AstNode)) {
            return true;
        }

        if luaur_ast::rtti::ast_node_is::<AstStatBreak>(&*(node as *mut AstNode))
            || luaur_ast::rtti::ast_node_is::<AstStatContinue>(&*(node as *mut AstNode))
        {
            return true;
        }

        let stat_if = luaur_ast::rtti::ast_node_as::<AstStatIf>(node as *mut AstNode);
        if !stat_if.is_null() {
            let condition = (*stat_if).condition;
            let thenbody = (*stat_if).thenbody;
            let elsebody = (*stat_if).elsebody;

            if is_constant_true(constants, condition) {
                return always_terminates(constants, thenbody as *mut AstStat);
            }

            if is_constant_false(constants, condition) && !elsebody.is_null() {
                return always_terminates(constants, elsebody);
            }

            return !elsebody.is_null()
                && always_terminates(constants, thenbody as *mut AstStat)
                && always_terminates(constants, elsebody);
        }

        false
    }
}
