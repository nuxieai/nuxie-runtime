use crate::records::ast_array::AstArray;
use crate::records::ast_attr::AstAttr;
use crate::records::lexeme::Type;
use crate::records::parser::Parser;
use crate::records::temp_vector::TempVector;

impl Parser {
    // attributes ::= {attribute}
    pub fn parse_attributes(&mut self) -> AstArray<*mut AstAttr> {
        let r#type = self.lexer.current().r#type;

        luaur_common::macros::luau_assert::LUAU_ASSERT!(
            r#type == Type::Attribute || r#type == Type::AttributeOpen
        );

        let mut attributes = TempVector::new(&mut self.scratch_attr);

        while self.lexer.current().r#type == Type::Attribute
            || self.lexer.current().r#type == Type::AttributeOpen
        {
            self.parse_attribute(&mut attributes);
        }

        self.copy_temp_vector_t(&attributes)
    }
}
