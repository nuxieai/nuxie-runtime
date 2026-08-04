use crate::records::ast_array::AstArray;
use crate::records::ast_name::AstName;
use crate::records::ast_type::AstType;
use crate::records::ast_type_or_pack::AstTypeOrPack;
use crate::records::location::Location;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct AstTypeReference {
    pub base: AstType,
    pub has_parameter_list: bool,
    pub prefix: Option<AstName>,
    pub prefix_location: Option<Location>,
    pub name: AstName,
    pub name_location: Location,
    pub parameters: AstArray<AstTypeOrPack>,
}

impl crate::rtti::AstNodeClass for AstTypeReference {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstTypeReference");
}
