use crate::records::ast_name::AstName;
use crate::records::ast_node::AstNode;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_declare_global::AstStatDeclareGlobal;
use crate::records::ast_type::AstType;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstStatDeclareGlobal {
    pub fn new(
        location: Location,
        name: AstName,
        name_location: Location,
        type_: *mut AstType,
    ) -> Self {
        Self {
            base: AstStat {
                base: AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
                has_semicolon: false,
            },
            name,
            name_location,
            type_,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_declare_global_ast_stat_declare_global(
    location: Location,
    name: AstName,
    name_location: Location,
    type_: *mut AstType,
) -> AstStatDeclareGlobal {
    AstStatDeclareGlobal::new(location, name, name_location, type_)
}
