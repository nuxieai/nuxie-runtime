use crate::records::lexeme::Lexeme;
use luaur_common::LUAU_ASSERT;

impl Lexeme {
    pub fn get_quote_style(&self) -> crate::records::lexeme::QuoteStyle {
        LUAU_ASSERT!(self.r#type == crate::records::lexeme::Type::QuotedString);

        // If we have a well-formed string, we are guaranteed to see a closing delimiter after the string
        let data_ptr = unsafe { self.data.data };
        LUAU_ASSERT!(!data_ptr.is_null());

        let quote = unsafe { *data_ptr.add(self.length as usize) as u8 as char };
        if quote == '\'' {
            return crate::records::lexeme::QuoteStyle::Single;
        } else if quote == '"' {
            return crate::records::lexeme::QuoteStyle::Double;
        }

        LUAU_ASSERT!(false);
        crate::records::lexeme::QuoteStyle::Double // unreachable, but required due to compiler warning
    }
}
