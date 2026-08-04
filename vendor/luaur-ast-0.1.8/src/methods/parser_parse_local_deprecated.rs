use crate::records::ast_array::AstArray;
use crate::records::ast_attr::AstAttr;
use crate::records::ast_stat::AstStat;
use crate::records::parser::Parser;
use crate::records::cst_attr_list::CstAttrList;
use crate::records::temp_vector::TempVector;

impl Parser {
    #[allow(non_snake_case)]
    pub fn parseLocal_DEPRECATED(
        &mut self,
        attributes: &AstArray<*mut AstAttr>,
        cst_attr_lists: *mut TempVector<'_, *mut CstAttrList>,
    ) -> *mut AstStat {
        // C++: `Location start = lexer.current().location; if (attributes.size > 0)
        // start = attributes.data[0]->location;` — when attributes are present the
        // statement begins at the attribute, so a `local function` start location
        // includes the leading `@native`/`@checked`. The port ignored attributes.
        luaur_common::LUAU_ASSERT!(
            cst_attr_lists.is_null() || luaur_common::FFlag::LuauCstAttr.get()
        );
        let mut start = if luaur_common::FFlag::LuauCstAttr.get() {
            self.get_attribute_start_location(
                attributes,
                cst_attr_lists,
                self.lexer.current().location,
            )
        } else {
            self.lexer.current().location
        };
        if !luaur_common::FFlag::LuauCstAttr.get() && attributes.size > 0 {
            start = unsafe { (**attributes.data.add(0)).base.location };
        }
        self.parse_local(
            start,
            self.lexer.current().location.begin,
            attributes,
            false,
            cst_attr_lists,
        )
    }
}
