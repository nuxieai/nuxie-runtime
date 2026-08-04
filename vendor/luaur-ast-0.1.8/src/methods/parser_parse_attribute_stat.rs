use crate::enums::type_lexer::Type;
use crate::records::ast_attr::AstAttr;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_name::AstName;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_error::AstStatError;
use crate::records::ast_stat_function::AstStatFunction;
use crate::records::location::Location;
use crate::records::parser::Parser;

impl Parser {
    pub fn parse_attribute_stat(&mut self) -> *mut AstStat {
        let attributes = self.parse_attributes();
        let current_type = self.lexer.current().r#type;

        match current_type {
            Type::ReservedFunction => self.parse_function_stat(&attributes) as *mut AstStat,
            Type::ReservedLocal => {
                if luaur_common::FFlag::LuauConst2.get() {
                    let attr_loc = if attributes.size > 0 {
                        unsafe { (**attributes.data.add(0)).base.location }
                    } else {
                        self.lexer.current().location
                    };

                    self.parse_local(
                        attr_loc,
                        self.lexer.current().location.begin,
                        &attributes,
                        false,
                    ) as *mut AstStat
                } else {
                    self.parseLocal_DEPRECATED(&attributes) as *mut AstStat
                }
            }
            Type::Name => {
                let current = self.lexer.current();
                let current_name = unsafe { current.data.name };

                if luaur_common::FFlag::LuauExportValueSyntax.get()
                    && luaur_common::FFlag::LuauConst2.get()
                    && unsafe {
                        AstName::operator_eq_c_char(
                            &AstName {
                                value: current_name,
                            },
                            c"export".as_ptr(),
                        )
                    }
                {
                    let keyword_loc = current.location;
                    self.next_lexeme();

                    let attr_loc = if attributes.size > 0 {
                        unsafe { (**attributes.data.add(0)).base.location }
                    } else {
                        keyword_loc
                    };

                    self.parse_export_value(&attr_loc, keyword_loc.begin, &attributes)
                        as *mut AstStat
                } else if luaur_common::FFlag::LuauConst2.get()
                    && unsafe {
                        AstName::operator_eq_c_char(
                            &AstName {
                                value: current_name,
                            },
                            c"const".as_ptr(),
                        )
                    }
                {
                    let keyword_loc = current.location;
                    self.next_lexeme();

                    let attr_loc = if attributes.size > 0 {
                        unsafe { (**attributes.data.add(0)).base.location }
                    } else {
                        keyword_loc
                    };

                    self.parse_local(attr_loc, keyword_loc.begin, &attributes, true) as *mut AstStat
                } else if self.options.allow_declaration_syntax
                    && unsafe {
                        AstName::operator_eq_c_char(
                            &AstName {
                                value: current_name,
                            },
                            c"declare".as_ptr(),
                        )
                    }
                {
                    let expr = self.parse_primary_expr(true);
                    self.parse_declaration(&unsafe { (*expr).base.location }, &attributes)
                        as *mut AstStat
                } else {
                    self.parse_attribute_stat_fallthrough_to_error(&attributes) as *mut AstStat
                }
            }
            _ => self.parse_attribute_stat_fallthrough_to_error(&attributes) as *mut AstStat,
        }
    }

    fn parse_attribute_stat_fallthrough_to_error(
        &mut self,
        _attributes: &crate::records::ast_array::AstArray<*mut AstAttr>,
    ) -> *mut AstStatError {
        let current = self.lexer.current();
        let loc = current.location;

        if luaur_common::FFlag::LuauConst2.get() {
            self.report_stat_error(
                loc,
                crate::records::ast_array::AstArray {
                    data: core::ptr::null_mut(),
                    size: 0,
                },
                crate::records::ast_array::AstArray {
                    data: core::ptr::null_mut(),
                    size: 0,
                },
                format_args!(
                    "Expected 'function', 'local function', 'const function', 'declare function' or a function type declaration after attribute, but got {} instead",
                    current.to_string()
                ),
            )
        } else {
            self.report_stat_error(
                loc,
                crate::records::ast_array::AstArray {
                    data: core::ptr::null_mut(),
                    size: 0,
                },
                crate::records::ast_array::AstArray {
                    data: core::ptr::null_mut(),
                    size: 0,
                },
                format_args!(
                    "Expected 'function', 'local function', 'declare function' or a function type declaration after attribute, but got {} instead",
                    current.to_string()
                ),
            )
        }
    }
}
