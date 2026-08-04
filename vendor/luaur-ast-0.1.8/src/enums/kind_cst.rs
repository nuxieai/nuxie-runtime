#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum Kind {
    Indexer,
    Property,
    StringProperty,
}

impl Kind {
    pub const Indexer: Kind = Kind::Indexer;
    pub const Property: Kind = Kind::Property;
    pub const StringProperty: Kind = Kind::StringProperty;
}
