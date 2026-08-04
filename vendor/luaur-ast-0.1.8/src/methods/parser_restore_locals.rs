use crate::records::ast_local::AstLocal;
use crate::records::parser::Parser;

impl Parser {
    #[allow(non_snake_case)]
    pub fn restore_locals(&mut self, offset: u32) {
        let offset = offset as usize;

        for i in (offset + 1..=self.local_stack.len()).rev() {
            let l_ptr = self.local_stack[i - 1];

            // Safety: local_stack holds live arena pointers; `l` borrows the arena,
            // not `self`, so the local_map mutation below does not alias it.
            let l = unsafe { &*l_ptr };
            *self.local_map.get_or_insert(l.name) = l.shadow;
        }

        self.local_stack.truncate(offset);
    }
}
