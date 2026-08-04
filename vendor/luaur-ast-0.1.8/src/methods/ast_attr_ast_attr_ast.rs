use crate::records::ast_array::AstArray;
use crate::records::ast_attr::{AstAttr, AstAttrType};
use crate::records::ast_expr::AstExpr;
use crate::records::ast_node::AstNode;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstAttr {
    pub fn ast_attr_location_type_item_ast_array_ast_expr(
        location: Location,
        r#type: AstAttrType,
        args: AstArray<*mut AstExpr>,
    ) -> Self {
        Self {
            base: AstNode {
                class_index: <Self as AstNodeClass>::CLASS_INDEX,
                location,
            },
            r#type,
            args,
            name: Default::default(),
        }
    }
}
