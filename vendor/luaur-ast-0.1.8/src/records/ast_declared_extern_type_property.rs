use crate::enums::ast_table_access::AstTableAccess;
use crate::records::ast_name::AstName;
use crate::records::ast_type::AstType;
use crate::records::location::Location;

#[derive(Debug, Clone, Copy)]
pub struct AstDeclaredExternTypeProperty {
    pub name: AstName,
    pub name_location: Location,
    pub ty: *mut AstType,
    pub is_method: bool,
    pub location: Location,
    pub access: AstTableAccess,
}

impl Default for AstDeclaredExternTypeProperty {
    fn default() -> Self {
        Self {
            name: AstName {
                value: core::ptr::null(),
            },
            name_location: Location::default(),
            ty: core::ptr::null_mut(),
            is_method: false,
            location: Location::default(),
            access: AstTableAccess::ReadWrite,
        }
    }
}
