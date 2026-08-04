use crate::records::ast_array::AstArray;
use crate::records::ast_table_indexer::AstTableIndexer;
use crate::records::ast_table_prop::AstTableProp;
use crate::records::ast_type::AstType;

#[repr(C)]
#[derive(Debug)]
pub struct AstTypeTable {
    pub base: AstType,
    pub props: AstArray<AstTableProp>,
    pub indexer: *mut AstTableIndexer,
}

impl crate::rtti::AstNodeClass for AstTypeTable {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstTypeTable");
}
