use crate::records::compiler::Compiler;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Compiler {
    pub fn are_locals_captured(&mut self, start: usize) -> bool {
        LUAU_ASSERT!(start <= self.local_stack.len());

        for i in start..self.local_stack.len() {
            let local = self.local_stack[i];
            let l = self.locals.find(&local);
            LUAU_ASSERT!(l.is_some());

            if l.map_or(false, |l| l.captured) {
                return true;
            }
        }

        false
    }
}
