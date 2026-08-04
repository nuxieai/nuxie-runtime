use crate::enums::bc_imm_kind::BcImmKind;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BcImm {
    pub kind: BcImmKind,
    pub value: BcImmValue,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union BcImmValue {
    pub valueBoolean: bool,
    pub valueInt: i32,
    pub valueImport: u32,
}

impl std::fmt::Debug for BcImmValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BcImmValue").finish_non_exhaustive()
    }
}

impl PartialEq for BcImmValue {
    fn eq(&self, _other: &Self) -> bool {
        // Safety: Union equality is context-dependent on BcImm.kind
        unsafe { self.valueImport == _other.valueImport }
    }
}

impl Eq for BcImmValue {}

impl std::hash::Hash for BcImmValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { self.valueImport.hash(state) }
    }
}
