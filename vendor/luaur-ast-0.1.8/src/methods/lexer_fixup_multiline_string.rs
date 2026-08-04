use crate::records::lexer::Lexer;

impl Lexer {
    pub fn fixup_multiline_string(data: &mut alloc::string::String) {
        Self::fixup_multiline_bytes(unsafe { data.as_mut_vec() });
    }
}
