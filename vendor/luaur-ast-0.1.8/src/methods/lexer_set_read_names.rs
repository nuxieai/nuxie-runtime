use crate::records::lexer::Lexer;

impl Lexer {
    pub fn set_read_names(&mut self, read: bool) {
        self.read_names = read;
    }
}
