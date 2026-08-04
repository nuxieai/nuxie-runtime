use crate::records::ast_array::AstArray;
use crate::records::lexeme::Type;
use crate::records::lexer::Lexer;
use crate::records::parser::Parser;
use core::ffi::c_char;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Parser {
    pub(crate) fn parse_char_array(
        &mut self,
        original_string: Option<*mut AstArray<c_char>>,
    ) -> Option<AstArray<c_char>> {
        let current_lexeme = *self.lexer.current();
        let current_type = current_lexeme.r#type;
        LUAU_ASSERT!(
            current_type == Type::QuotedString
                || current_type == Type::RawString
                || current_type == Type::InterpStringSimple
        );

        let data_ptr = unsafe { current_lexeme.data.data };
        let length = current_lexeme.get_length() as usize;
        let bytes = unsafe { core::slice::from_raw_parts(data_ptr as *const u8, length) };

        if let Some(original) = original_string {
            unsafe {
                *original = self.copy_bytes(bytes);
            }
        }

        let mut data = bytes.to_vec();

        if current_type == Type::QuotedString || current_type == Type::InterpStringSimple {
            if !Lexer::fixup_quoted_bytes(&mut data) {
                self.next_lexeme();
                return None;
            }
        } else {
            Lexer::fixup_multiline_bytes(&mut data);
        }

        let value = self.copy_bytes(&data);
        self.next_lexeme();
        Some(value)
    }
}

#[allow(non_snake_case)]
pub fn parser_parse_char_array(
    this: &mut Parser,
    original_string: Option<*mut AstArray<c_char>>,
) -> Option<AstArray<c_char>> {
    this.parse_char_array(original_string)
}
