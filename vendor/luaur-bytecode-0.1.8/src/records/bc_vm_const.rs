use crate::enums::bc_vm_const_kind::BcVmConstKind;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BcVmConst {
    pub kind: BcVmConstKind,
    pub value: BcVmConstValue,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union BcVmConstValue {
    pub valueBoolean: bool,
    pub valueNumber: f64,
    pub valueVector: [f32; 4],
    pub valueString: &'static str,
    pub valueImport: u32,
    pub valueTable: u32,
    pub valueClosure: u32,
    pub valueInteger: i64,
}

impl core::fmt::Debug for BcVmConstValue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("BcVmConstValue(..)")
    }
}
