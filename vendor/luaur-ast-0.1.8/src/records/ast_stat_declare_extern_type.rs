use crate::records::ast_array::AstArray;
use crate::records::ast_declared_extern_type_property::AstDeclaredExternTypeProperty;
use crate::records::ast_name::AstName;
use crate::records::ast_stat::AstStat;
use crate::records::ast_table_indexer::AstTableIndexer;

#[repr(C)]
#[derive(Debug)]
pub struct AstStatDeclareExternType {
    pub base: AstStat,
    pub name: AstName,
    pub super_name: Option<AstName>,
    pub props: AstArray<AstDeclaredExternTypeProperty>,
    pub indexer: *mut AstTableIndexer,
}

impl crate::rtti::AstNodeClass for AstStatDeclareExternType {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstStatDeclareExternType");
}
