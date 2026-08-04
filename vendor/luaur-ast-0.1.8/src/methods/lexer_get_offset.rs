use crate::records::lexer::Lexer;

impl Lexer {
    #[inline]
    pub fn get_offset(&self) -> u32 {
        self.offset
    }
}
