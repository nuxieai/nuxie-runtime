use crate::records::allocator::Allocator;
use crate::records::ast_array::AstArray;
use crate::records::ast_type::AstType;
use crate::records::ast_type_intersection::AstTypeIntersection;
use crate::records::ast_type_optional::AstTypeOptional;
use crate::records::ast_type_union::AstTypeUnion;
use crate::records::cst_type_intersection::CstTypeIntersection;
use crate::records::cst_type_union::CstTypeUnion;
use crate::records::lexeme::Type;
use crate::records::location::Location;
use crate::records::parse_error::ParseError;
use crate::records::parser::Parser;
use crate::records::position::Position;
use crate::records::temp_vector::TempVector;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Parser {
    pub fn parse_type_suffix(&mut self, type_: *mut AstType, begin: &Location) -> *mut AstType {
        let mut parts = TempVector::new(&mut self.scratch_type);
        let mut separator_positions = TempVector::new(&mut self.scratch_position);
        let mut leading_position = Position::missing();

        if !type_.is_null() {
            parts.push_back(type_);
        }

        self.increment_recursion_counter("type annotation");

        let mut is_union = false;
        let mut is_intersection = false;
        let mut optional_count = 0;

        let mut location = *begin;

        loop {
            let c = self.lexer.current().r#type;
            let separator_position = self.lexer.current().location.begin;

            if c == Type::Pipe {
                self.next_lexeme();

                let old_recursion_count = self.recursion_counter;
                let part = self.parse_simple_type(false, false).r#type;
                self.recursion_counter = old_recursion_count;

                parts.push_back(part);
                is_union = true;

                if self.options.store_cst_data {
                    if type_.is_null() && !leading_position.has_value() {
                        leading_position = separator_position;
                    } else {
                        separator_positions.push_back(separator_position);
                    }
                }
            } else if c == Type::Question {
                LUAU_ASSERT!(parts.size() >= 1);

                let loc = self.lexer.current().location;
                self.next_lexeme();

                parts.push_back(unsafe {
                    (*self.allocator).alloc(AstTypeOptional::new(loc)) as *mut AstType
                });
                optional_count += 1;

                is_union = true;
            } else if c == Type::Ampersand {
                self.next_lexeme();

                let old_recursion_count = self.recursion_counter;
                let part = self.parse_simple_type(false, false).r#type;
                self.recursion_counter = old_recursion_count;

                parts.push_back(part);
                is_intersection = true;

                if self.options.store_cst_data {
                    if type_.is_null() && !leading_position.has_value() {
                        leading_position = separator_position;
                    } else {
                        separator_positions.push_back(separator_position);
                    }
                }
            } else if c == Type::Dot3 {
                self.report_location_c_char_item(
                    self.lexer.current().location,
                    format_args!("Unexpected '...' after type annotation"),
                );
                self.next_lexeme();
            } else {
                break;
            }

            let limit = luaur_common::FInt::LuauTypeLengthLimit.get() as u32;
            if parts.size() as u32 > limit + optional_count {
                ParseError::raise(
                    unsafe { (**parts.back()).base.location },
                    format_args!(
                        "Exceeded allowed type length; simplify your type annotation to make the code compile"
                    ),
                );
            }
        }

        if parts.size() == 1 && !is_union && !is_intersection {
            return unsafe { *parts.front() };
        }

        if is_union && is_intersection {
            let error_location =
                Location::between(*begin, unsafe { (**parts.back()).base.location });
            let error_types = self.copy_temp_vector_t(&parts);
            return self.report_type_error(
                error_location,
                error_types,
                format_args!(
                    "Mixing union and intersection types is not allowed; consider wrapping in parentheses."
                ),
            ) as *mut AstType;
        }

        location.end = unsafe { (**parts.back()).base.location.end };

        if is_union {
            let node = unsafe {
                (*self.allocator)
                    .alloc(AstTypeUnion::new(location, self.copy_temp_vector_t(&parts)))
            };
            if self.options.store_cst_data {
                let cst_node = unsafe {
                    (*self.allocator).alloc(CstTypeUnion::new(
                        leading_position,
                        self.copy_temp_vector_t(&separator_positions),
                    ))
                };
                self.cst_node_map.try_insert(
                    node as *mut crate::records::ast_node::AstNode,
                    cst_node as *mut crate::records::cst_node::CstNode,
                );
            }
            return node as *mut AstType;
        }

        if is_intersection {
            let node = unsafe {
                (*self.allocator).alloc(AstTypeIntersection::new(
                    location,
                    self.copy_temp_vector_t(&parts),
                ))
            };
            if self.options.store_cst_data {
                let cst_node = unsafe {
                    (*self.allocator).alloc(CstTypeIntersection::new(
                        leading_position,
                        self.copy_temp_vector_t(&separator_positions),
                    ))
                };
                self.cst_node_map.try_insert(
                    node as *mut crate::records::ast_node::AstNode,
                    cst_node as *mut crate::records::cst_node::CstNode,
                );
            }
            return node as *mut AstType;
        }

        LUAU_ASSERT!(false);
        ParseError::raise(
            *begin,
            format_args!("Composite type was not an intersection or union."),
        );
        core::ptr::null_mut()
    }
}
