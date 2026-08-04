use crate::records::ast_node::AstNode;
use crate::records::ast_stat::AstStat;
use crate::records::location::Location;

impl AstStat {
    pub fn new(class_index: i32, location: Location) -> Self {
        Self {
            base: AstNode {
                class_index,
                location,
            },
            has_semicolon: false,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_ast_stat(class_index: i32, location: Location) -> AstStat {
    AstStat::new(class_index, location)
}
