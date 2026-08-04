use crate::records::ast_array::AstArray;
use crate::records::ast_type_or_pack::AstTypeOrPack;
use crate::records::comma_separator_inserter::CommaSeparatorInserter;
use crate::records::cst_type_instantiation::CstTypeInstantiation;
use crate::records::printer::Printer;
use crate::records::writer::Writer;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> Printer<'a> {
    pub fn visualize_explicit_type_instantiation(
        &mut self,
        type_arguments: AstArray<AstTypeOrPack>,
        cst_node: *const CstTypeInstantiation,
    ) {
        if !cst_node.is_null() {
            unsafe {
                self.maybe_advance_and_write(&(*cst_node).left_arrow_1_position, "<", false);
            }
        } else {
            self.writer.symbol("<");
        }

        if !cst_node.is_null() {
            unsafe {
                self.maybe_advance_and_write(&(*cst_node).left_arrow_2_position, "<", false);
            }
        } else {
            self.writer.symbol("<");
        }

        let comma_position = if !cst_node.is_null() {
            unsafe { (*cst_node).comma_positions.begin() }
        } else {
            core::ptr::null()
        };

        let mut comma = CommaSeparatorInserter::new(self.writer, comma_position);

        for type_or_pack in type_arguments.as_slice() {
            comma.operator_call(self.writer);

            if !type_or_pack.r#type.is_null() {
                self.visualize_type_annotation(unsafe { &mut *type_or_pack.r#type });
            } else {
                LUAU_ASSERT!(!type_or_pack.type_pack.is_null());
                self.visualize_type_pack_annotation(
                    unsafe { &mut *type_or_pack.type_pack },
                    false,
                    false,
                    false,
                );
            }
        }

        if !cst_node.is_null() {
            unsafe {
                self.maybe_advance_and_write(&(*cst_node).right_arrow_1_position, ">", false);
            }
        } else {
            self.writer.symbol(">");
        }

        if !cst_node.is_null() {
            unsafe {
                self.maybe_advance_and_write(&(*cst_node).right_arrow_2_position, ">", false);
            }
        } else {
            self.writer.symbol(">");
        }
    }
}
