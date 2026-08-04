use crate::records::ast_local::AstLocal;
use crate::records::binding::Binding;
use crate::records::parser::Parser;

impl Parser {
    #[allow(non_snake_case)]
    pub fn push_local(&mut self, binding: &Binding) -> *mut AstLocal {
        let name = binding.name;

        // C++: `AstLocal*& local = local_map[name.name];` — operator[] inserts a
        // null slot when absent, and the prior value becomes the new local's
        // shadow before the slot is reassigned to the freshly allocated local.
        let shadow = *self.local_map.get_or_insert(name.name);

        let function_depth = self.function_stack.len() - 1;
        let loop_depth = self.function_stack.last().unwrap().loop_depth as usize;

        let new_local = AstLocal {
            name: name.name,
            location: name.location,
            shadow,
            function_depth,
            loop_depth,
            is_const: binding.is_const,
            is_exported: false,
            annotation: binding.annotation,
        };

        // Safety: self.allocator is a raw pointer to the arena Allocator.
        let local = unsafe { (*self.allocator).alloc(new_local) };

        *self.local_map.get_or_insert(name.name) = local;
        self.local_stack.push(local);

        local
    }
}
