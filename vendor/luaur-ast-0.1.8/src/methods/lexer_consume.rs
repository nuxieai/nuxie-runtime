use crate::functions::is_newline::is_newline;
use crate::records::lexer::Lexer;
use luaur_common::LUAU_ASSERT;

impl Lexer {
    #[inline(always)]
    pub(crate) fn consume(&mut self) {
        // consume() assumes current character is known to not be a newline; use consume_any if this is not guaranteed
        unsafe {
            LUAU_ASSERT!(!is_newline(
                *self.buffer.add(self.offset as usize) as u8 as char
            ));
        }

        self.offset += 1;
    }
}

#[allow(non_snake_case)]
pub fn lexer_consume(lexer: &mut Lexer) {
    lexer.consume();
}
