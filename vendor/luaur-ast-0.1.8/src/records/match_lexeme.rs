use crate::enums::type_lexer::Type;
use crate::records::position::Position;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub struct MatchLexeme {
    pub type_: Type,
    pub position: Position,
}
