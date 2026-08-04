use crate::records::compiler::Compiler;
use crate::records::return_visitor::ReturnVisitor;

impl Compiler {
    pub fn return_visitor_return_visitor(&mut self) -> ReturnVisitor {
        ReturnVisitor {
            self_: self,
            returns_one: true,
        }
    }
}
