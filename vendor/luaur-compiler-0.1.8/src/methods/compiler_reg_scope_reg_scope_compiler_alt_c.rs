use crate::records::reg_scope::RegScope;

impl RegScope {
    pub fn drop(&mut self) {
        unsafe {
            (*self.self_).reg_top = self.old_top;
        }
    }
}
