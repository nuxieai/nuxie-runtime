use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_block::AstStatBlock;
use crate::records::printer::Printer;
use crate::rtti::ast_node_as;

pub trait IntoAstStatMut {
    unsafe fn into_ast_stat_mut(self) -> *mut AstStat;
}

impl IntoAstStatMut for *mut AstStat {
    unsafe fn into_ast_stat_mut(self) -> *mut AstStat {
        self
    }
}

impl IntoAstStatMut for &mut AstStat {
    unsafe fn into_ast_stat_mut(self) -> *mut AstStat {
        self
    }
}

impl<'a> Printer<'a> {
    pub fn visualize_block_ast_stat<S: IntoAstStatMut>(&mut self, stat: S) {
        let stat = unsafe { stat.into_ast_stat_mut() };
        let block = unsafe {
            ast_node_as::<AstStatBlock>(
                stat as *mut AstStat as *mut crate::records::ast_node::AstNode,
            )
        };
        if !block.is_null() {
            let block_ref = unsafe { &mut *block };
            self.visualize_block_ast_stat_block(block_ref);
            return;
        }

        luaur_common::LUAU_ASSERT!(false);
    }
}
