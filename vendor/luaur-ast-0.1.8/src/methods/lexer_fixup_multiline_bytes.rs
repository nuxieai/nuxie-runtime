use crate::records::lexer::Lexer;
use alloc::vec::Vec;

impl Lexer {
    pub fn fixup_multiline_bytes(data: &mut Vec<u8>) {
        if data.is_empty() {
            return;
        }

        let len = data.len();
        let mut src = 0;
        let mut dst = 0;

        // skip leading newline
        if src + 1 < len && data[src] == b'\r' && data[src + 1] == b'\n' {
            src += 2;
        } else if src < len && data[src] == b'\n' {
            src += 1;
        }

        // parse the rest of the string, converting newlines as we go
        while src < len {
            if src + 1 < len && data[src] == b'\r' && data[src + 1] == b'\n' {
                data[dst] = b'\n';
                dst += 1;
                src += 2;
            } else {
                data[dst] = data[src];
                dst += 1;
                src += 1;
            }
        }

        data.truncate(dst);
    }
}
