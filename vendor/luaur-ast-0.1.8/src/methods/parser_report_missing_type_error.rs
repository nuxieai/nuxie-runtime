use crate::records::ast_array::AstArray;
use crate::records::ast_type::AstType;
use crate::records::ast_type_error::AstTypeError;
use crate::records::location::Location;
use crate::records::parser::Parser;

impl Parser {
    pub fn report_missing_type_error(
        &mut self,
        parse_error_location: Location,
        ast_error_location: Location,
        format: core::fmt::Arguments<'_>,
    ) -> *mut AstTypeError {
        self.report(parse_error_location, format);

        let message_index = (self.parse_errors.len() as u32).saturating_sub(1);

        unsafe {
            (*self.allocator).alloc(AstTypeError {
                base: AstType {
                    base: crate::records::ast_node::AstNode {
                        class_index: <AstTypeError as crate::rtti::AstNodeClass>::CLASS_INDEX,
                        location: ast_error_location,
                    },
                },
                types: AstArray {
                    data: core::ptr::null_mut(),
                    size: 0,
                },
                is_missing: true,
                message_index,
            })
        }
    }
}
