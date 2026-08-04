use crate::records::lexeme::Lexeme;
use luaur_common::LUAU_ASSERT;

impl Lexeme {
    #[allow(non_snake_case)]
    pub fn get_length(&self) -> u32 {
        LUAU_ASSERT!(
            self.r#type == crate::records::lexeme::Type::RawString
                || self.r#type == crate::records::lexeme::Type::QuotedString
                || self.r#type == crate::records::lexeme::Type::InterpStringBegin
                || self.r#type == crate::records::lexeme::Type::InterpStringMid
                || self.r#type == crate::records::lexeme::Type::InterpStringEnd
                || self.r#type == crate::records::lexeme::Type::InterpStringSimple
                || self.r#type == crate::records::lexeme::Type::BrokenInterpDoubleBrace
                || self.r#type == crate::records::lexeme::Type::Number
                || self.r#type == crate::records::lexeme::Type::Comment
                || self.r#type == crate::records::lexeme::Type::BlockComment
        );

        self.length
    }
}
