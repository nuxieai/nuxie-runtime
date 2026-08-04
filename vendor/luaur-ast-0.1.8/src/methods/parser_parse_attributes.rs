use crate::records::ast_array::AstArray;
use crate::records::ast_attr::AstAttr;
use crate::records::cst_attr_list::CstAttrList;
use crate::records::lexeme::Type;
use crate::records::parser::Parser;
use crate::records::temp_vector::TempVector;

impl Parser {
    // attributes ::= {attribute}
    pub fn parse_attributes(
        &mut self,
        cst_attr_lists: *mut TempVector<'_, *mut CstAttrList>,
    ) -> AstArray<*mut AstAttr> {
        luaur_common::LUAU_ASSERT!(
            cst_attr_lists.is_null() || luaur_common::FFlag::LuauCstAttr.get()
        );
        let r#type = self.lexer.current().r#type;

        luaur_common::macros::luau_assert::LUAU_ASSERT!(
            r#type == Type::Attribute || r#type == Type::AttributeOpen
        );

        let mut attributes = TempVector::new(&mut self.scratch_attr);

        while self.lexer.current().r#type == Type::Attribute
            || self.lexer.current().r#type == Type::AttributeOpen
        {
            if luaur_common::FFlag::LuauCstAttr.get() {
                if self.lexer.current().r#type == Type::Attribute {
                    self.parse_attribute(&mut attributes);
                } else {
                    self.parse_attr_list(&mut attributes, cst_attr_lists);
                }
            } else {
                self.parse_attribute_deprecated(&mut attributes);
            }
        }

        self.copy_temp_vector_t(&attributes)
    }
}
