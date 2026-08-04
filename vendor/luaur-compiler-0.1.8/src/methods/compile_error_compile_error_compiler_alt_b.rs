use crate::records::compile_error::CompileError;

impl CompileError {
    pub fn drop(&mut self) {
        // The destructor is trivial: std::string and Location have their own destructors
    }
}
