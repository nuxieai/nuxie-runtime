use crate::records::ast_array::AstArray;
use crate::records::ast_attr::AstAttr;
use crate::records::ast_local::AstLocal;
use crate::records::ast_name::AstName;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_local::AstStatLocal;
use crate::records::lexeme::Type;
use crate::records::location::Location;
use crate::records::name::Name;
use crate::records::parser::Parser;
use crate::records::position::Position;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Parser {
    fn check_duplicate_export_value(&mut self, name: AstName, location: Location) -> bool {
        if self.declared_export_bindings.find(&name).is_some() {
            return false;
        }

        *self.declared_export_bindings.get_or_insert(name) = location;
        true
    }

    fn export_local_stat_value(
        &mut self,
        stat: *mut AstStat,
        keyword_location: Location,
    ) -> *mut AstStat {
        let local_stat = unsafe {
            crate::rtti::ast_node_as::<AstStatLocal>(stat as *mut crate::records::ast_node::AstNode)
        };
        if !local_stat.is_null() {
            unsafe {
                (*local_stat).is_exported = true;
            }

            let vars = unsafe { (*local_stat).vars };
            for i in 0..vars.size {
                let local = unsafe { *vars.data.add(i) };
                if !self.check_duplicate_export_value(unsafe { (*local).name }, unsafe {
                    (*local).location
                }) {
                    self.report_location_c_char_item(
                        unsafe { (*local).location },
                        format_args!("Duplicate exported identifier '{}'", unsafe {
                            core::ffi::CStr::from_ptr((*local).name.value).to_string_lossy()
                        }),
                    );
                    continue;
                }

                unsafe {
                    (*local).is_exported = true;
                }
            }

            unsafe {
                (*local_stat).keyword_location = Some(keyword_location);
            }
        } else {
            LUAU_ASSERT!(
                false,
                "Expected export local/const to parse as AstStatLocal"
            );
        }

        stat
    }

    pub fn parse_export_value(
        &mut self,
        start: &Location,
        keyword_position: Position,
        attributes: &AstArray<*mut AstAttr>,
    ) -> *mut AstStat {
        if self.function_stack.len() != 1 || self.recursion_counter != 1 {
            self.report_location_c_char_item(
                *start,
                format_args!("'export' may only be applied to top-level statements"),
            );
        }

        if self.has_module_return {
            self.report_location_c_char_item(
                *start,
                format_args!("Exporting values is not compatible with top-level return (export/return conflict)"),
            );
        }

        if attributes.size != 0 && self.lexer.current().r#type != Type::ReservedFunction {
            self.report_location_c_char_item(
                self.lexer.current().location,
                format_args!(
                    "Expected 'function' after export declaration with attribute, but got {} instead",
                    self.lexer.current().to_string()
                ),
            );
        }

        if self.lexer.current().r#type == Type::ReservedLocal {
            let local_keyword_location = self.lexer.current().location;

            if self.lexer.lookahead().r#type == Type::ReservedFunction {
                self.report_location_c_char_item(
                    *start,
                    format_args!("'export' must be followed by an identifier or 'function'; try removing 'local'"),
                );
                return self.parse_local(
                    *start,
                    local_keyword_location.begin,
                    &AstArray {
                        data: core::ptr::null_mut(),
                        size: 0,
                    },
                    true,
                );
            }

            let stat = self.parse_local(
                *start,
                keyword_position,
                &AstArray {
                    data: core::ptr::null_mut(),
                    size: 0,
                },
                false,
            );
            return self.export_local_stat_value(stat, local_keyword_location);
        } else if self.lexer.current().r#type == Type::ReservedFunction {
            let func_stat = self.parse_local(*start, keyword_position, attributes, true);
            if !crate::rtti::ast_node_is::<
                crate::records::ast_stat_local_function::AstStatLocalFunction,
            >(func_stat as *mut crate::records::ast_node::AstNode)
            {
                return func_stat;
            }

            let func = unsafe {
                func_stat as *mut crate::records::ast_stat_local_function::AstStatLocalFunction
            };
            let name = unsafe { (*func).name };
            if !self
                .check_duplicate_export_value(unsafe { (*name).name }, unsafe { (*name).location })
            {
                self.report_location_c_char_item(
                    unsafe { (*name).location },
                    format_args!("Duplicate exported identifier '{}'", unsafe {
                        core::ffi::CStr::from_ptr((*name).name.value).to_string_lossy()
                    }),
                );
            }

            unsafe {
                (*name).is_exported = true;
                (*name).is_const = true;
            }
            func_stat
        } else if self.lexer.current().r#type == Type::Name
            && unsafe { AstName::ast_name_c_char(self.lexer.current().data.name) }
                .operator_eq_c_char(b"const\0".as_ptr() as *const core::ffi::c_char)
        {
            let const_keyword_location = self.lexer.current().location;
            self.next_lexeme();

            if self.lexer.current().r#type == Type::ReservedFunction {
                self.report_location_c_char_item(
                    *start,
                    format_args!("'export' must be followed by an identifier or 'function'"),
                );
                return self.parse_local(
                    *start,
                    const_keyword_location.begin,
                    &AstArray {
                        data: core::ptr::null_mut(),
                        size: 0,
                    },
                    true,
                );
            }

            let stat = self.parse_local(
                *start,
                const_keyword_location.begin,
                &AstArray {
                    data: core::ptr::null_mut(),
                    size: 0,
                },
                true,
            );
            return self.export_local_stat_value(stat, const_keyword_location);
        } else if luaur_common::FFlag::DebugLuauUserDefinedClasses.get()
            && self.lexer.current().r#type == Type::Name
            && unsafe { AstName::ast_name_c_char(self.lexer.current().data.name) }
                .operator_eq_c_char(b"class\0".as_ptr() as *const core::ffi::c_char)
        {
            self.next_lexeme();
            let stat = self.parse_class_stat(start, true);
            let class_stat = unsafe { stat as *mut crate::records::ast_stat_class::AstStatClass };
            if !class_stat.is_null() {
                let name = unsafe { (*class_stat).name };
                if !self.check_duplicate_export_value(unsafe { (*name).name }, unsafe {
                    (*name).location
                }) {
                    self.report_location_c_char_item(
                        unsafe { (*name).location },
                        format_args!("Duplicate exported class '{}'", unsafe {
                            core::ffi::CStr::from_ptr((*name).name.value).to_string_lossy()
                        }),
                    );
                }

                unsafe {
                    (*name).is_exported = true;
                }
            }
            stat
        } else {
            self.report_stat_error(
                *start,
                AstArray {
                    data: core::ptr::null_mut(),
                    size: 0,
                },
                AstArray {
                    data: core::ptr::null_mut(),
                    size: 0,
                },
                format_args!("'export' must be followed by an identifier or 'function'"),
            ) as *mut AstStat
        }
    }
}
