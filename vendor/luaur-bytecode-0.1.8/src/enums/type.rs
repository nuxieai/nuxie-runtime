#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum Type {
    Type_Nil,
    Type_Boolean,
    Type_Number,
    Type_Integer,
    Type_Vector,
    Type_String,
    Type_Import,
    Type_Table,
    Type_Closure,
    Type_ClassShape,
}

impl Type {
    pub const Type_Nil: Type = Type::Type_Nil;
    pub const Type_Boolean: Type = Type::Type_Boolean;
    pub const Type_Number: Type = Type::Type_Number;
    pub const Type_Integer: Type = Type::Type_Integer;
    pub const Type_Vector: Type = Type::Type_Vector;
    pub const Type_String: Type = Type::Type_String;
    pub const Type_Import: Type = Type::Type_Import;
    pub const Type_Table: Type = Type::Type_Table;
    pub const Type_Closure: Type = Type::Type_Closure;
    pub const Type_ClassShape: Type = Type::Type_ClassShape;
}
