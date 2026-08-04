use crate::functions::is_digit_lexer::is_digit;
use crate::functions::is_hex_digit::is_hex_digit;
use crate::functions::is_space::is_space;
use crate::functions::to_utf_8::to_utf_8;
use crate::functions::unescape::unescape;
use crate::records::lexer::Lexer;
use alloc::vec::Vec;
use luaur_common::LUAU_ASSERT;

impl Lexer {
    #[allow(non_snake_case)]
    pub fn fixup_quoted_bytes(data: &mut Vec<u8>) -> bool {
        if data.is_empty() || !data.contains(&b'\\') {
            return true;
        }

        let size = data.len();
        let mut write = 0;
        let mut i = 0;

        while i < size {
            if data[i] != b'\\' {
                data[write] = data[i];
                write += 1;
                i += 1;
                continue;
            }

            if i + 1 == size {
                return false;
            }

            let escape = data[i + 1] as char;
            i += 2; // skip \e

            match escape {
                '\n' => {
                    data[write] = b'\n';
                    write += 1;
                }

                '\r' => {
                    data[write] = b'\n';
                    write += 1;
                    if i < size && data[i] == b'\n' {
                        i += 1;
                    }
                }

                '\0' => {
                    return false;
                }

                'x' => {
                    // hex escape codes are exactly 2 hex digits long
                    if i + 2 > size {
                        return false;
                    }

                    let mut code: u32 = 0;

                    for j in 0..2 {
                        let ch = data[i + j] as char;
                        if !is_hex_digit(ch) {
                            return false;
                        }

                        // use or trick to convert to lower case
                        code = 16 * code
                            + (if is_digit(ch) {
                                (ch as u32) - ('0' as u32)
                            } else {
                                ((ch as u32) | 32) - ('a' as u32) + 10
                            });
                    }

                    data[write] = code as u8;
                    write += 1;
                    i += 2;
                }

                'z' => {
                    while i < size && is_space(data[i] as char) {
                        i += 1;
                    }
                }

                'u' => {
                    // unicode escape codes are at least 3 characters including braces
                    if i + 3 > size {
                        return false;
                    }

                    if data[i] != b'{' {
                        return false;
                    }
                    i += 1;

                    if i < size && data[i] == b'}' {
                        return false;
                    }

                    let mut code: u32 = 0;

                    let mut found_brace = false;
                    for _ in 0..16 {
                        if i == size {
                            return false;
                        }

                        let ch = data[i] as char;

                        if ch == '}' {
                            found_brace = true;
                            break;
                        }

                        if !is_hex_digit(ch) {
                            return false;
                        }

                        // use or trick to convert to lower case.
                        // C++ `code` is `unsigned int` and wraps on overflow.
                        code = code.wrapping_mul(16).wrapping_add(if is_digit(ch) {
                            (ch as u32) - ('0' as u32)
                        } else {
                            ((ch as u32) | 32) - ('a' as u32) + 10
                        });
                        i += 1;
                    }

                    if !found_brace || i == size || data[i] != b'}' {
                        return false;
                    }
                    i += 1;

                    let utf8_len = to_utf_8(&mut data[write..], code);
                    if utf8_len == 0 {
                        return false;
                    }

                    write += utf8_len;
                }

                _ => {
                    if is_digit(escape) {
                        let mut code = (escape as u32) - ('0' as u32);

                        for _ in 0..2 {
                            if i == size || !is_digit(data[i] as char) {
                                break;
                            }

                            code = 10 * code + ((data[i] as u32) - ('0' as u32));
                            i += 1;
                        }

                        if code > 255 {
                            return false;
                        }

                        data[write] = code as u8;
                        write += 1;
                    } else {
                        data[write] = unescape(escape) as u8;
                        write += 1;
                    }
                }
            }
        }

        LUAU_ASSERT!(write <= size);
        data.truncate(write);

        true
    }
}
