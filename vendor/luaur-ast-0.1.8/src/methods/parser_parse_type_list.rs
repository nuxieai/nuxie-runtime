//! Node: `cxx:Method:Luau.Ast:Ast/src/Parser.cpp:2424:parseTypeList`
//!
//! Faithful port of `Parser::parseTypeList` — a comma-separated list of types,
//! optionally with `name:` argument labels (which back-fill empty name slots for
//! earlier unlabelled entries). Returns a trailing `AstTypePack*` when the list
//! ends in a `...`/named pack, else null. `result`/`result_names` and the
//! optional comma / name-colon position vectors are filled in place for the
//! caller; raw-pointer position vectors are written only when non-null.

use crate::functions::should_parse_type_pack::should_parse_type_pack;
use crate::records::ast_name::AstName;
use crate::records::ast_type::AstType;
use crate::records::ast_type_pack::AstTypePack;
use crate::records::lexeme::Type;
use crate::records::parser::Parser;
use crate::records::position::Position;
use crate::records::temp_vector::TempVector;
use crate::type_aliases::ast_argument_name::AstArgumentName;

impl Parser {
    pub fn parse_type_list(
        &mut self,
        result: &mut TempVector<'_, *mut AstType>,
        result_names: &mut TempVector<'_, Option<AstArgumentName>>,
        comma_positions: *mut TempVector<'_, Position>,
        name_colon_positions: *mut TempVector<'_, Position>,
    ) -> *mut AstTypePack {
        loop {
            if should_parse_type_pack(&mut self.lexer) {
                return self.parse_type_pack();
            }

            if self.lexer.current().r#type == Type::Name
                && self.lexer.lookahead().r#type == Type(b':' as i32)
            {
                // Fill in previous argument names with empty slots
                while result_names.size() < result.size() {
                    result_names.push_back(None);
                }
                if !name_colon_positions.is_null() {
                    while unsafe { (*name_colon_positions).size() } < result.size() {
                        unsafe {
                            (*name_colon_positions).push_back(Position::missing());
                        }
                    }
                }

                let arg_name: AstArgumentName = (
                    AstName {
                        value: unsafe { self.lexer.current().data.name },
                    },
                    self.lexer.current().location,
                );
                result_names.push_back(Some(arg_name));
                self.next_lexeme();

                if !name_colon_positions.is_null() {
                    let begin = self.lexer.current().location.begin;
                    unsafe {
                        (*name_colon_positions).push_back(begin);
                    }
                }
                self.expect_and_consume_char(':', "");
            } else if !result_names.empty() {
                // If we have a type with named arguments, provide elements for all types
                result_names.push_back(None);
                if !name_colon_positions.is_null() {
                    unsafe {
                        (*name_colon_positions).push_back(Position::missing());
                    }
                }
            }

            let ty = self.parse_type_bool(false);
            result.push_back(ty);
            if self.lexer.current().r#type != Type(b',' as i32) {
                break;
            }

            if !comma_positions.is_null() {
                let begin = self.lexer.current().location.begin;
                unsafe {
                    (*comma_positions).push_back(begin);
                }
            }
            self.next_lexeme();

            if self.lexer.current().r#type == Type(b')' as i32) {
                let loc = self.lexer.current().location;
                self.report(
                    loc,
                    format_args!("Expected type after ',' but got ')' instead"),
                );
                break;
            }
        }

        core::ptr::null_mut()
    }
}
