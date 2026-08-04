#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Function {
    pub vararg: bool,
    pub loop_depth: u32,
}

impl Default for Function {
    fn default() -> Self {
        Self {
            vararg: false,
            loop_depth: 0,
        }
    }
}
