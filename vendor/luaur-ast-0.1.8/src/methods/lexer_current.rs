use crate::records::lexeme::Lexeme;
use crate::records::lexer::Lexer;

impl Lexer {
    pub fn current(&self) -> &Lexeme {
        &self.lexeme
    }
}
