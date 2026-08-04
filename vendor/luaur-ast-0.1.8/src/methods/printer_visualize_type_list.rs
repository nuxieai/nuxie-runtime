use crate::records::ast_array::AstArray;
use crate::records::ast_type_list::AstTypeList;
use crate::records::position::Position;
use crate::records::printer::Printer;

impl<'a> Printer<'a> {
    pub fn visualize_type_list(
        &mut self,
        list: &AstTypeList,
        unconditionally_parenthesize: bool,
        open_parentheses_position: Position,
        close_parentheses_position: Position,
        comma_positions: AstArray<Position>,
    ) {
        self.visualize_named_type_list(
            list,
            unconditionally_parenthesize,
            open_parentheses_position,
            close_parentheses_position,
            &comma_positions,
            &AstArray {
                data: core::ptr::null_mut(),
                size: 0,
            },
            &AstArray {
                data: core::ptr::null_mut(),
                size: 0,
            },
        );
    }
}
