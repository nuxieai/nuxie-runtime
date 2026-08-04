use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_continue::AstStatContinue;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstStatContinue {
    pub fn new(location: Location) -> Self {
        Self {
            base: AstStat {
                base: crate::records::ast_node::AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
                has_semicolon: false,
            },
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_continue_ast_stat_continue(location: Location) -> AstStatContinue {
    AstStatContinue::new(location)
}
