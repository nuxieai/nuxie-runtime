use crate::records::call_inliner::CallInliner;

impl CallInliner<'_> {
    pub fn validate(&self) -> bool {
        if !self.validate_cfg() {
            return false;
        }
        if !self.validate_phis() {
            return false;
        }
        true
    }
}
