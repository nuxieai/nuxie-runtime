use crate::records::lexeme::Lexeme;
use crate::records::match_lexeme::MatchLexeme;

impl MatchLexeme {
    pub fn new(l: &Lexeme) -> Self {
        Self {
            type_: l.r#type,
            position: l.location.begin,
        }
    }

    pub fn missing() -> Self {
        Self {
            type_: crate::records::lexeme::Type::Eof,
            position: crate::records::position::Position::missing(),
        }
    }
}

#[allow(non_snake_case)]
pub fn parser_match_lexeme_match_lexeme(l: &Lexeme) -> MatchLexeme {
    MatchLexeme::new(l)
}
