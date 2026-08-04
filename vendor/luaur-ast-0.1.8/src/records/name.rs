#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Name {
    pub name: crate::records::ast_name::AstName,
    pub location: crate::records::location::Location,
}
