use crate::records::lexeme::Type;
use crate::records::lexer::Lexer;

#[allow(non_snake_case)]
pub(crate) fn should_parse_type_pack(lexer: &mut Lexer) -> bool {
    if lexer.current().r#type == Type::Dot3 {
        return true;
    } else if lexer.current().r#type == Type::Name && lexer.lookahead().r#type == Type::Dot3 {
        return true;
    }

    false
}
