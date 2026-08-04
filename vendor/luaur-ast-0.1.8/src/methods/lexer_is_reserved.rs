#[allow(non_snake_case)]
pub const kReserved: &[&str] = &[
    "and", "break", "do", "else", "elseif", "end", "false", "for", "function", "if", "in", "local",
    "nil", "not", "or", "repeat", "return", "then", "true", "until", "while",
];

impl crate::records::lexer::Lexer {
    pub fn is_reserved(word: &str) -> bool {
        for i in (crate::records::lexeme::Type::Reserved_BEGIN.0 as usize)
            ..(crate::records::lexeme::Type::Reserved_END.0 as usize)
        {
            if word == kReserved[i - (crate::records::lexeme::Type::Reserved_BEGIN.0 as usize)] {
                return true;
            }
        }

        false
    }
}
