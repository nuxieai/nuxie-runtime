use crate::records::ast_array::AstArray;
use crate::records::ast_attr::AstAttr;
use crate::records::cst_attr_list::CstAttrList;
use crate::records::location::Location;
use crate::records::parser::Parser;
use crate::records::temp_vector::TempVector;

impl Parser {
    pub fn get_attribute_start_location(
        &self,
        attributes: &AstArray<*mut AstAttr>,
        cst_attr_lists: *const TempVector<'_, *mut CstAttrList>,
        default_location: Location,
    ) -> Location {
        luaur_common::LUAU_ASSERT!(luaur_common::FFlag::LuauCstAttr.get());
        if attributes.size > 0 {
            if !cst_attr_lists.is_null() && unsafe { (*cst_attr_lists).size() > 0 } {
                let first_attr_location = unsafe { (**attributes.data).base.location };
                let at_bracket_position =
                    unsafe { (**(*cst_attr_lists).operator_index(0)).at_bracket_position };
                if first_attr_location.begin < at_bracket_position {
                    first_attr_location
                } else {
                    Location::new(at_bracket_position, at_bracket_position)
                }
            } else {
                unsafe { (**attributes.data).base.location }
            }
        } else if !cst_attr_lists.is_null() && unsafe { (*cst_attr_lists).size() > 0 } {
            let at_bracket_position =
                unsafe { (**(*cst_attr_lists).operator_index(0)).at_bracket_position };
            Location::new(at_bracket_position, at_bracket_position)
        } else {
            default_location
        }
    }
}
