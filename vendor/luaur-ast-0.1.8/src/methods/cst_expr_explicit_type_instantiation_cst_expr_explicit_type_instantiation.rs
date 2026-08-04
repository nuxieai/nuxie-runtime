use crate::records::cst_expr_explicit_type_instantiation::CstExprExplicitTypeInstantiation;
use crate::records::cst_node::CstNode;
use crate::records::cst_type_instantiation::CstTypeInstantiation;
use crate::rtti::CstNodeClass;

impl CstExprExplicitTypeInstantiation {
    pub fn new(instantiation: CstTypeInstantiation) -> Self {
        Self {
            base: CstNode {
                class_index: <Self as CstNodeClass>::CLASS_INDEX,
            },
            instantiation,
        }
    }
}

#[allow(non_snake_case)]
pub fn cst_expr_explicit_type_instantiation_cst_expr_explicit_type_instantiation(
    instantiation: CstTypeInstantiation,
) -> CstExprExplicitTypeInstantiation {
    CstExprExplicitTypeInstantiation::new(instantiation)
}
