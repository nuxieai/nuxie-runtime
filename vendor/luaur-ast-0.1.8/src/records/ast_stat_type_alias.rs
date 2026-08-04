#[repr(C)]
#[derive(Debug, Clone)]
pub struct AstStatTypeAlias {
    pub base: crate::records::ast_stat::AstStat,
    pub name: crate::records::ast_name::AstName,
    pub name_location: crate::records::location::Location,
    pub generics:
        crate::records::ast_array::AstArray<*mut crate::records::ast_generic_type::AstGenericType>,
    pub generic_packs: crate::records::ast_array::AstArray<
        *mut crate::records::ast_generic_type_pack::AstGenericTypePack,
    >,
    pub type_ptr: *mut crate::records::ast_type::AstType,
    pub exported: bool,
}

impl crate::rtti::AstNodeClass for AstStatTypeAlias {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstStatTypeAlias");
}
