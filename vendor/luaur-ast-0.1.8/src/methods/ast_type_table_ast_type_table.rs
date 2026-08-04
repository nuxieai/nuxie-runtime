use crate::records::ast_array::AstArray;
use crate::records::ast_node::AstNode;
use crate::records::ast_table_indexer::AstTableIndexer;
use crate::records::ast_table_prop::AstTableProp;
use crate::records::ast_type::AstType;
use crate::records::ast_type_table::AstTypeTable;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstTypeTable {
    pub fn new(
        location: Location,
        props: AstArray<AstTableProp>,
        indexer: *mut AstTableIndexer,
    ) -> Self {
        Self {
            base: AstType {
                base: AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
            },
            props,
            indexer,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_type_table_ast_type_table(
    location: Location,
    props: AstArray<AstTableProp>,
    indexer: *mut AstTableIndexer,
) -> AstTypeTable {
    AstTypeTable::new(location, props, indexer)
}
