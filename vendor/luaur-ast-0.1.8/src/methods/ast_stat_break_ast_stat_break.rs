use crate::records::ast_node::AstNode;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_break::AstStatBreak;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstStatBreak {
    pub fn new(location: Location) -> Self {
        Self {
            base: AstStat {
                base: AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
                has_semicolon: false,
            },
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_break_ast_stat_break(location: Location) -> AstStatBreak {
    AstStatBreak::new(location)
}
