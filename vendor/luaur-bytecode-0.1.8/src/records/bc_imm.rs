use crate::enums::bc_imm_kind::BcImmKind;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BcImm {
    pub kind: BcImmKind,
    pub value: BcImmValue,
}

impl PartialEq for BcImm {
    fn eq(&self, rhs: &Self) -> bool {
        unsafe {
            if self.kind == BcImmKind::Boolean && rhs.kind == BcImmKind::Boolean {
                self.value.valueBoolean == rhs.value.valueBoolean
            } else if self.kind == BcImmKind::Int && rhs.kind == BcImmKind::Int {
                self.value.valueInt == rhs.value.valueInt
            } else if self.kind == BcImmKind::Import && rhs.kind == BcImmKind::Import {
                self.value.valueImport == rhs.value.valueImport
            } else {
                false
            }
        }
    }
}

impl Eq for BcImm {}

impl core::hash::Hash for BcImm {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        core::hash::Hash::hash(&self.kind, state);
        unsafe {
            match self.kind {
                BcImmKind::Boolean => core::hash::Hash::hash(&self.value.valueBoolean, state),
                BcImmKind::Int => core::hash::Hash::hash(&self.value.valueInt, state),
                BcImmKind::Import => core::hash::Hash::hash(&self.value.valueImport, state),
            }
        }
    }
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
