use crate::records::ast_array::AstArray;
use crate::records::ast_declared_extern_type_property::AstDeclaredExternTypeProperty;
use crate::records::ast_name::AstName;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_declare_extern_type::AstStatDeclareExternType;
use crate::records::ast_table_indexer::AstTableIndexer;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstStatDeclareExternType {
    pub fn new(
        location: Location,
        name: AstName,
        super_name: Option<AstName>,
        props: AstArray<AstDeclaredExternTypeProperty>,
        indexer: *mut AstTableIndexer,
    ) -> Self {
        Self {
            base: AstStat {
                base: crate::records::ast_node::AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
                has_semicolon: false,
            },
            name,
            super_name,
            props,
            indexer,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_declare_extern_type_ast_stat_declare_extern_type(
    location: Location,
    name: AstName,
    super_name: Option<AstName>,
    props: AstArray<AstDeclaredExternTypeProperty>,
    indexer: *mut AstTableIndexer,
) -> AstStatDeclareExternType {
    AstStatDeclareExternType::new(location, name, super_name, props, indexer)
}
