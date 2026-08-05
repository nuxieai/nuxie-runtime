use crate::records::ast_array::AstArray;
use crate::records::ast_attr::AstAttr;
use crate::records::cst_attr_list::CstAttrList;
use crate::records::printer::Printer;

impl<'a> Printer<'a> {
    pub fn visualize_attributes(
        &mut self,
        attributes: &AstArray<*mut AstAttr>,
        attr_lists: *const AstArray<*mut CstAttrList>,
    ) {

        if attr_lists.is_null() {
            for attribute in attributes.iter() {
                self.visualize_attribute(unsafe { &mut **attribute });
            }
            return;
        }

        let attr_lists = unsafe { &*attr_lists };
        let mut attribute_index = 0usize;
        let mut list_index = 0usize;

        while attribute_index != attributes.size || list_index != attr_lists.size {
            let use_standalone = list_index == attr_lists.size
                || unsafe {
                    (**attributes.data.add(attribute_index)).base.location.begin
                        < (**attr_lists.data.add(list_index)).at_bracket_position
                };

            if use_standalone {
                self.visualize_attribute(unsafe {
                    &mut **attributes.data.add(attribute_index)
                });
                attribute_index += 1;
            } else {
                let cst_attr_list = unsafe { *attr_lists.data.add(list_index) };
                self.advance(unsafe { &(*cst_attr_list).at_bracket_position });
                self.writer.symbol("@[");

                for comma_position in unsafe { (*cst_attr_list).comma_positions.iter() } {
                    luaur_common::LUAU_ASSERT!(attribute_index != attributes.size);
                    luaur_common::LUAU_ASSERT!(unsafe {
                        (**attributes.data.add(attribute_index)).base.location.begin
                            < *comma_position
                    });
                    self.visualize_attribute(unsafe {
                        &mut **attributes.data.add(attribute_index)
                    });
                    attribute_index += 1;
                    self.advance(comma_position);
                    self.writer.symbol(",");
                }

                luaur_common::LUAU_ASSERT!(attribute_index != attributes.size);
                self.visualize_attribute(unsafe {
                    &mut **attributes.data.add(attribute_index)
                });
                attribute_index += 1;
                self.maybe_advance_and_write(
                    unsafe { &(*cst_attr_list).close_bracket_position },
                    "]",
                    false,
                );
                list_index += 1;
            }
        }
    }
}
