use crate::records::compiler::Compiler;

impl Compiler {
    pub fn at_top_level(&self) -> bool {
        !self.current_function.is_null()
            && unsafe { (*self.current_function).function_depth == 0 }
            && self.block_depth == 0
            && self.loops.is_empty()
    }
}
