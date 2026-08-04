use crate::records::lexeme::Lexeme;
use crate::records::parser::Parser;

impl Parser {
    pub(crate) fn block_follow(&self, l: &Lexeme) -> bool {
        l.r#type == crate::records::lexeme::Type::Eof
            || l.r#type == crate::records::lexeme::Type::ReservedElse
            || l.r#type == crate::records::lexeme::Type::ReservedElseif
            || l.r#type == crate::records::lexeme::Type::ReservedEnd
            || l.r#type == crate::records::lexeme::Type::ReservedUntil
    }
}

#[allow(non_snake_case)]
pub fn parser_block_follow(this: &Parser, l: &Lexeme) -> bool {
    this.block_follow(l)
}
