use crate::enums::bc_vm_const_kind::BcVmConstKind;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BcVmConst {
    pub kind: BcVmConstKind,
    pub value: BcVmConstValue,
}

impl PartialEq for BcVmConst {
    fn eq(&self, rhs: &Self) -> bool {
        if self.kind != rhs.kind {
            return false;
        }

        unsafe {
            match self.kind {
                BcVmConstKind::Nil => true,
                BcVmConstKind::Boolean => self.value.valueBoolean == rhs.value.valueBoolean,
                BcVmConstKind::Number => self.value.valueNumber == rhs.value.valueNumber,
                BcVmConstKind::Vector => {
                    self.value.valueVector[0] == rhs.value.valueVector[0]
                        && self.value.valueVector[1] == rhs.value.valueVector[1]
                        && self.value.valueVector[2] == rhs.value.valueVector[2]
                        && self.value.valueVector[3] == rhs.value.valueVector[3]
                }
                BcVmConstKind::String => self.value.valueString == rhs.value.valueString,
                BcVmConstKind::Import => self.value.valueImport == rhs.value.valueImport,
                BcVmConstKind::Table => self.value.valueTable == rhs.value.valueTable,
                BcVmConstKind::Closure => self.value.valueClosure == rhs.value.valueClosure,
                BcVmConstKind::Integer => self.value.valueInteger == rhs.value.valueInteger,
            }
        }
    }
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
