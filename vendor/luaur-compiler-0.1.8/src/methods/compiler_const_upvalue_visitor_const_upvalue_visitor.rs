use crate::records::compiler::Compiler;
use crate::records::const_upvalue_visitor::ConstUpvalueVisitor;

impl Compiler {
    pub fn const_upvalue_visitor_const_upvalue_visitor(&mut self) -> ConstUpvalueVisitor {
        ConstUpvalueVisitor {
            self_: self,
            upvals: Vec::new(),
        }
    }
}
