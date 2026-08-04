use crate::records::parser::Parser;

impl Parser {
    #[allow(non_snake_case)]
    pub fn save_locals(&self) -> u32 {
        self.local_stack.len() as u32
    }
}
