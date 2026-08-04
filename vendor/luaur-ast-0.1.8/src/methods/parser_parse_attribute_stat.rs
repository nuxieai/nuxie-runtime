use crate::enums::type_lexer::Type;
use crate::records::ast_attr::AstAttr;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_name::AstName;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_error::AstStatError;
use crate::records::ast_stat_function::AstStatFunction;
use crate::records::cst_attr_list::CstAttrList;
use crate::records::location::Location;
use crate::records::parser::Parser;
use crate::records::temp_vector::TempVector;

impl Parser {
    pub fn parse_attribute_stat(&mut self) -> *mut AstStat {
        let start_location = self.lexer.current().location;
        let mut cst_attr_lists =
            TempVector::<*mut CstAttrList>::new(&mut self.scratch_cst_attr_list);
        let cst_attr_lists_ptr = if luaur_common::FFlag::LuauCstAttr.get() {
            &mut cst_attr_lists
        } else {
            core::ptr::null_mut()
        };
        let attributes = self.parse_attributes(cst_attr_lists_ptr);
        let current_type = self.lexer.current().r#type;

        match current_type {
            Type::ReservedFunction => {
                self.parse_function_stat(&attributes, cst_attr_lists_ptr) as *mut AstStat
            }
            Type::ReservedLocal => {
                let attr_loc = if luaur_common::FFlag::LuauCstAttr.get() {
                    self.get_attribute_start_location(&attributes, &cst_attr_lists, start_location)
                } else if attributes.size > 0 {
                    unsafe { (**attributes.data.add(0)).base.location }
                } else {
                    self.lexer.current().location
                };

                self.parse_local(
                    attr_loc,
                    self.lexer.current().location.begin,
                    &attributes,
                    false,
                    cst_attr_lists_ptr,
                ) as *mut AstStat
            }
            Type::Name => {
                let current = self.lexer.current();
                let current_name = unsafe { current.data.name };

                if luaur_common::FFlag::LuauExportValueSyntax.get()
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

                    let attr_loc = if luaur_common::FFlag::LuauCstAttr.get() {
                        self.get_attribute_start_location(
                            &attributes,
                            &cst_attr_lists,
                            start_location,
                        )
                    } else if attributes.size > 0 {
                        unsafe { (**attributes.data.add(0)).base.location }
                    } else {
                        keyword_loc
                    };

                    self.parse_export_value(
                        &attr_loc,
                        keyword_loc.begin,
                        &attributes,
                        cst_attr_lists_ptr,
                    ) as *mut AstStat
                } else if unsafe {
                    AstName::operator_eq_c_char(
                        &AstName {
                            value: current_name,
                        },
                        c"const".as_ptr(),
                    )
                } {
                    let keyword_loc = current.location;
                    self.next_lexeme();

                    let attr_loc = if luaur_common::FFlag::LuauCstAttr.get() {
                        self.get_attribute_start_location(
                            &attributes,
                            &cst_attr_lists,
                            start_location,
                        )
                    } else if attributes.size > 0 {
                        unsafe { (**attributes.data.add(0)).base.location }
                    } else {
                        keyword_loc
                    };

                    self.parse_local(
                        attr_loc,
                        keyword_loc.begin,
                        &attributes,
                        true,
                        cst_attr_lists_ptr,
                    ) as *mut AstStat
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
    }
}
