use crate::records::compiler::Compiler;
use crate::records::undefined_local_visitor::UndefinedLocalVisitor;
use luaur_common::records::dense_hash_set::DenseHashSet;

impl Compiler {
    pub fn undefined_local_visitor_undefined_local_visitor(&mut self) -> UndefinedLocalVisitor {
        UndefinedLocalVisitor {
            self_: self,
            undef: core::ptr::null_mut(),
            locals: DenseHashSet::new(core::ptr::null_mut()),
        }
    }
}
