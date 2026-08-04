use crate::records::ast_array::AstArray;
use crate::records::ast_node::AstNode;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_block::AstStatBlock;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstStatBlock {
    pub fn new(location: Location, body: AstArray<*mut AstStat>, has_end: bool) -> Self {
        Self {
            base: AstStat {
                base: AstNode {
                    class_index: <Self as crate::rtti::AstNodeClass>::CLASS_INDEX,
                    location,
                },
                has_semicolon: false,
            },
            body,
            has_end,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_block_ast_stat_block(
    location: Location,
    body: AstArray<*mut AstStat>,
    has_end: bool,
) -> AstStatBlock {
    AstStatBlock::new(location, body, has_end)
}
