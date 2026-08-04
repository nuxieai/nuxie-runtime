use crate::records::ast_array::AstArray;
use crate::records::cst_node::CstNode;
use crate::records::cst_type_pack_explicit::CstTypePackExplicit;
use crate::records::position::Position;
use crate::rtti::CstNodeClass;

impl CstTypePackExplicit {
    pub fn cst_type_pack_explicit() -> Self {
        Self {
            base: CstNode::new(<Self as CstNodeClass>::CLASS_INDEX),
            open_parentheses_position: Position::missing(),
            close_parentheses_position: Position::missing(),
            comma_positions: AstArray::default(),
        }
    }
}

#[allow(non_snake_case)]
pub fn cst_type_pack_explicit() -> CstTypePackExplicit {
    CstTypePackExplicit::cst_type_pack_explicit()
}
