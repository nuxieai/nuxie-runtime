use crate::records::compiler::Compiler;
use crate::records::reg_scope::RegScope;

impl Compiler {
    pub fn reg_scope_compiler_i32(&mut self, top: u32) -> RegScope {
        assert!(top <= self.reg_top);
        let old_top = self.reg_top;
        self.reg_top = top;
        RegScope {
            self_: self as *mut Compiler,
            old_top,
        }
    }
}
