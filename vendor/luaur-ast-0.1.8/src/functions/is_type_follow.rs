use crate::records::lexeme::Type;

#[allow(non_snake_case)]
pub(crate) fn is_type_follow(c: Type) -> bool {
    c == Type('|' as i32) || c == Type('?' as i32) || c == Type('&' as i32)
}
