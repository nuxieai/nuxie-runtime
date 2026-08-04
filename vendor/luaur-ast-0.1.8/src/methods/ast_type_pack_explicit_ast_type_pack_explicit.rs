use crate::records::ast_node::AstNode;
use crate::records::ast_type_list::AstTypeList;
use crate::records::ast_type_pack::AstTypePack;
use crate::records::ast_type_pack_explicit::AstTypePackExplicit;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstTypePackExplicit {
    pub fn new(location: Location, type_list: AstTypeList) -> Self {
        Self {
            base: AstTypePack {
                base: AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
            },
            type_list: type_list,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_type_pack_explicit_ast_type_pack_explicit(
    location: Location,
    type_list: AstTypeList,
) -> AstTypePackExplicit {
    AstTypePackExplicit::new(location, type_list)
}
