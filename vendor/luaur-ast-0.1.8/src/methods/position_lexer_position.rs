use crate::records::lexer::Lexer;
use crate::records::position::Position;

impl Lexer {
    #[inline(always)]
    pub(crate) fn position(&self) -> Position {
        Position {
            line: self.line,
            column: self.offset.wrapping_sub(self.line_offset),
        }
    }
}
