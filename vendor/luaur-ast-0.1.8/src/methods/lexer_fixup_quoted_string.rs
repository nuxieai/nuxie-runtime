use crate::records::lexer::Lexer;

impl Lexer {
    #[allow(non_snake_case)]
    pub fn fixup_quoted_string(data: &mut alloc::string::String) -> bool {
        Self::fixup_quoted_bytes(unsafe { data.as_mut_vec() })
    }
}
