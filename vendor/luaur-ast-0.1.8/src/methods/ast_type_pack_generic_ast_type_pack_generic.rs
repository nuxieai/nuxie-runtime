use crate::records::ast_name::AstName;
use crate::records::ast_node::AstNode;
use crate::records::ast_type_pack::AstTypePack;
use crate::records::ast_type_pack_generic::AstTypePackGeneric;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstTypePackGeneric {
    pub fn new(location: Location, name: AstName) -> Self {
        Self {
            base: AstTypePack {
                base: AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
            },
            generic_name: name,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_type_pack_generic_ast_type_pack_generic(
    location: Location,
    name: AstName,
) -> AstTypePackGeneric {
    AstTypePackGeneric::new(location, name)
}
