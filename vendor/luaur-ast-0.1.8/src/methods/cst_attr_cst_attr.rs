use crate::records::cst_attr::CstAttr;
use crate::records::cst_node::CstNode;
use crate::rtti::CstNodeClass;

impl CstAttr {
    pub fn new(has_at: bool) -> Self {
        Self {
            base: CstNode {
                class_index: <Self as CstNodeClass>::CLASS_INDEX,
            },
            has_at,
        }
    }
}
