#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum DumpFlags {
    Dump_Code = 1 << 0,
    Dump_Lines = 1 << 1,
    Dump_Source = 1 << 2,
    Dump_Locals = 1 << 3,
    Dump_Remarks = 1 << 4,
    Dump_Types = 1 << 5,
    Dump_Constants = 1 << 6,
}

impl DumpFlags {
    pub const Dump_Code: DumpFlags = DumpFlags::Dump_Code;
    pub const Dump_Lines: DumpFlags = DumpFlags::Dump_Lines;
    pub const Dump_Source: DumpFlags = DumpFlags::Dump_Source;
    pub const Dump_Locals: DumpFlags = DumpFlags::Dump_Locals;
    pub const Dump_Remarks: DumpFlags = DumpFlags::Dump_Remarks;
    pub const Dump_Types: DumpFlags = DumpFlags::Dump_Types;
    pub const Dump_Constants: DumpFlags = DumpFlags::Dump_Constants;
}
