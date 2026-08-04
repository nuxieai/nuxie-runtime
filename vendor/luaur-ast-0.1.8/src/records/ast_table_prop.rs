use crate::enums::ast_table_access::AstTableAccess;
use crate::records::ast_name::AstName;
use crate::records::ast_type::AstType;
use crate::records::location::Location;

#[derive(Debug, Clone)]
pub struct AstTableProp {
    pub name: AstName,
    pub location: Location,
    pub r#type: *mut AstType,
    pub access: AstTableAccess,
    pub access_location: Option<Location>,
}
