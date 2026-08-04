use crate::records::ast_name::AstName;
use crate::records::ast_type::AstType;
use crate::records::location::Location;

#[derive(Debug, Clone, Copy)]
pub struct AstClassProperty {
    pub qualifier_location: Location,
    pub name: AstName,
    pub name_location: Location,
    pub type_colon_location: Option<Location>,
    pub ty: *mut AstType,
}

impl Default for AstClassProperty {
    fn default() -> Self {
        Self {
            qualifier_location: Location::default(),
            name: AstName {
                value: core::ptr::null(),
            },
            name_location: Location::default(),
            type_colon_location: None,
            ty: core::ptr::null_mut(),
        }
    }
}
