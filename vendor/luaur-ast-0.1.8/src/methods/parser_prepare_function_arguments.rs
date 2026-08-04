use crate::records::ast_array::AstArray;
use crate::records::ast_local::AstLocal;
use crate::records::binding::Binding;
use crate::records::location::Location;
use crate::records::parser::Parser;
use crate::records::temp_vector::TempVector;

impl Parser {
    #[allow(non_snake_case)]
    pub fn prepare_function_arguments(
        &mut self,
        start: &Location,
        hasself: bool,
        args: &TempVector<'_, Binding>,
    ) -> (*mut AstLocal, AstArray<*mut AstLocal>) {
        let mut self_local: *mut AstLocal = core::ptr::null_mut();

        if hasself {
            // C++: push_local(Binding(Name(name_self, start), nullptr));
            // `Parser::Name { name, location }` is the (AstName, Location) pair.
            let binding = Binding::new(
                crate::records::name::Name {
                    name: self.name_self,
                    location: *start,
                },
                core::ptr::null_mut(),
                crate::records::position::Position::default(),
                false,
            );
            self_local = self.push_local(&binding);
        }

        // C++ uses a `TempVector<AstLocal*> vars(scratch_local)` here; a local Vec
        // produces an identical `copy` result and avoids holding a borrow of
        // `self.scratch_local` across the `self.push_local` calls (the scratch
        // buffer is only an allocation-reuse optimization, not observable).
        let mut vars: alloc::vec::Vec<*mut AstLocal> = alloc::vec::Vec::new();
        for i in 0..args.size() {
            let local = self.push_local(&args[i]);
            vars.push(local);
        }

        let copied = self.copy_initializer_list_t(&vars);
        (self_local, copied)
    }
}
