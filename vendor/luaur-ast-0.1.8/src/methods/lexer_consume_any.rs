use crate::functions::is_newline::is_newline;
use crate::records::lexer::Lexer;

impl Lexer {
    #[inline(always)]
    pub(crate) fn consume_any(&mut self) {
        unsafe {
            let ch = *self.buffer.add(self.offset as usize) as u8 as char;
            if is_newline(ch) {
                self.line += 1;
                self.line_offset = self.offset + 1;
            }
        }

        self.offset += 1;
    }
}
