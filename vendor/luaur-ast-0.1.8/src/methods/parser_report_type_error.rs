use crate::records::ast_array::AstArray;
use crate::records::ast_type::AstType;
use crate::records::ast_type_error::AstTypeError;
use crate::records::location::Location;
use crate::records::parser::Parser;

impl Parser {
    pub fn report_type_error(
        &mut self,
        location: Location,
        types: AstArray<*mut AstType>,
        args: core::fmt::Arguments<'_>,
    ) -> *mut AstTypeError {
        self.report(location, args);

        let message_index = (self.parse_errors.len() as u32).saturating_sub(1);

        unsafe {
            let allocator = &mut *self.allocator;
            allocator.alloc(AstTypeError {
                base: AstType {
                    base: crate::records::ast_node::AstNode {
                        class_index: <AstTypeError as crate::rtti::AstNodeClass>::CLASS_INDEX,
                        location,
                    },
                },
                types,
                is_missing: false,
                message_index,
            })
        }
    }
}
