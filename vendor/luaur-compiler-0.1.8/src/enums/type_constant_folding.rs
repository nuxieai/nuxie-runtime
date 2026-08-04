#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    Type_Unknown,
    Type_Nil,
    Type_Boolean,
    Type_Number,
    Type_Integer,
    Type_Vector,
    Type_String,
    Type_Table,
}

impl Type {
    pub const Type_Unknown: Self = Self::Type_Unknown;
    pub const Type_Nil: Self = Self::Type_Nil;
    pub const Type_Boolean: Self = Self::Type_Boolean;
    pub const Type_Number: Self = Self::Type_Number;
    pub const Type_Integer: Self = Self::Type_Integer;
    pub const Type_Vector: Self = Self::Type_Vector;
    pub const Type_String: Self = Self::Type_String;
    pub const Type_Table: Self = Self::Type_Table;
}

impl Default for Type {
    fn default() -> Self {
        Self::Type_Unknown
    }
}
