use luaur_ast::records::location::Location;
use luaur_bytecode::records::string_ref::StringRef;

use crate::enums::kind::Kind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LValue {
    pub(crate) kind: Kind,
    pub(crate) reg: u8,
    pub(crate) upval: u8,
    pub(crate) index: u8,
    pub(crate) number: u8,
    pub(crate) name: StringRef,
    pub(crate) location: Location,
}

impl Default for LValue {
    fn default() -> Self {
        Self {
            kind: Kind::Kind_Local,
            reg: 0,
            upval: 0,
            index: 0,
            number: 0,
            name: StringRef::default(),
            location: Location::default(),
        }
    }
}
