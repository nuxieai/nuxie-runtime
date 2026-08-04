use crate::records::ast_array::AstArray;
use crate::records::ast_type_pack::AstTypePack;
use crate::records::binding::Binding;
use crate::records::location::Location;
use crate::records::parser::Parser;
use crate::records::position::Position;
use crate::records::temp_vector::TempVector;
use luaur_common::macros::luau_noinline::LUAU_NOINLINE;

impl Parser {
    pub fn parse_binding_list(
        &mut self,
        result: &mut TempVector<'_, Binding>,
        allow_dot_3: bool,
        comma_positions: *mut AstArray<Position>,
        initial_comma_position: *mut Position,
        vararg_annotation_colon_position: *mut Position,
        is_const: bool,
    ) -> (bool, Location, *mut AstTypePack) {
        let mut local_comma_positions = TempVector::new(&mut self.scratch_position);

        if !comma_positions.is_null() && !initial_comma_position.is_null() {
            unsafe {
                local_comma_positions.push_back(*initial_comma_position);
            }
        }

        loop {
            if self.lexer.current().r#type == crate::records::lexeme::Type::Dot3 && allow_dot_3 {
                let vararg_location = self.lexer.current().location;
                self.next_lexeme();

                let mut tail_annotation: *mut AstTypePack = std::ptr::null_mut();
                if self.lexer.current().r#type == crate::records::lexeme::Type::Colon {
                    if !vararg_annotation_colon_position.is_null() {
                        unsafe {
                            *vararg_annotation_colon_position = self.lexer.current().location.begin;
                        }
                    }

                    self.next_lexeme();
                    tail_annotation = self.parse_variadic_argument_type_pack();
                }

                if !comma_positions.is_null() {
                    unsafe {
                        *comma_positions = self.copy_temp_vector_t(&local_comma_positions);
                    }
                }

                return (true, vararg_location, tail_annotation);
            }

            result.push_back(self.parse_binding(is_const));

            if self.lexer.current().r#type != crate::records::lexeme::Type::Comma {
                break;
            }

            if !comma_positions.is_null() {
                local_comma_positions.push_back(self.lexer.current().location.begin);
            }

            self.next_lexeme();
        }

        if !comma_positions.is_null() {
            unsafe {
                *comma_positions = self.copy_temp_vector_t(&local_comma_positions);
            }
        }

        (false, Location::default(), std::ptr::null_mut())
    }
}
