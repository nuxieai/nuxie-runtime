use crate::records::ast_name::AstName;
use crate::records::ast_stat::AstStat;
use crate::records::ast_type::AstType;
use crate::records::location::Location;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct AstStatDeclareGlobal {
    pub base: AstStat,
    pub name: AstName,
    pub name_location: Location,
    pub type_: *mut AstType,
}

impl crate::rtti::AstNodeClass for AstStatDeclareGlobal {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstStatDeclareGlobal");
}

#[allow(non_snake_case)]
impl AstStatDeclareGlobal {
    pub const ClassIndex: i32 = <Self as crate::rtti::AstNodeClass>::CLASS_INDEX;
}
