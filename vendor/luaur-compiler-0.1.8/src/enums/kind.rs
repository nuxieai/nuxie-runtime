#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Kind_Local,
    Kind_Upvalue,
    Kind_Global,
    Kind_IndexName,
    Kind_IndexNumber,
    Kind_IndexExpr,
}

impl Kind {
    pub const Kind_Local: Kind = Kind::Kind_Local;
    pub const Kind_Upvalue: Kind = Kind::Kind_Upvalue;
    pub const Kind_Global: Kind = Kind::Kind_Global;
    pub const Kind_IndexName: Kind = Kind::Kind_IndexName;
    pub const Kind_IndexNumber: Kind = Kind::Kind_IndexNumber;
    pub const Kind_IndexExpr: Kind = Kind::Kind_IndexExpr;
}
