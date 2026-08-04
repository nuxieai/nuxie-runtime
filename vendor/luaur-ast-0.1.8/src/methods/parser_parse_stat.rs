use crate::functions::get_identifier::get_identifier;
use crate::records::ast_array::AstArray;
use crate::records::ast_attr::AstAttr;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_binary::AstExprBinaryOp;
use crate::records::ast_expr_call::AstExprCall;
use crate::records::ast_name::AstName;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_expr::AstStatExpr;
use crate::records::lexeme::Lexeme;
use crate::records::lexeme::Type;
use crate::records::location::Location;
use crate::records::parser::Parser;

impl Parser {
    pub fn parse_stat(&mut self) -> *mut AstStat {
        match self.lexer.current().r#type {
            Type::ReservedIf => return self.parse_if(),
            Type::ReservedWhile => return self.parse_while(),
            Type::ReservedDo => return self.parse_do(),
            Type::ReservedFor => return self.parse_for(),
            Type::ReservedRepeat => return self.parse_repeat(),
            Type::ReservedFunction => {
                return self.parse_function_stat(&AstArray {
                    data: core::ptr::null_mut(),
                    size: 0,
                }) as *mut AstStat;
            }
            Type::ReservedLocal => {
                if luaur_common::FFlag::LuauConst2.get() {
                    let start = self.lexer.current().location;
                    return self.parse_local(
                        start,
                        start.begin,
                        &AstArray {
                            data: core::ptr::null_mut(),
                            size: 0,
                        },
                        false,
                    );
                } else {
                    return self.parseLocal_DEPRECATED(&AstArray {
                        data: core::ptr::null_mut(),
                        size: 0,
                    });
                }
            }
            Type::ReservedReturn => return self.parse_return(),
            Type::ReservedBreak => return self.parser_parse_break(),
            Type::Attribute | Type::AttributeOpen => return self.parse_attribute_stat(),
            _ => {}
        }

        let start = self.lexer.current().location;
        let expr = self.parse_primary_expr(true);

        if unsafe {
            crate::rtti::ast_node_is::<AstExprCall>(unsafe { &*std::ptr::addr_of!((*expr).base) })
        } {
            return unsafe {
                (*self.allocator).alloc(AstStatExpr::new((*expr).base.location, expr))
                    as *mut AstStat
            };
        }

        let current_type = self.lexer.current().r#type;
        if current_type == Type(',' as i32) || current_type == Type('=' as i32) {
            return self.parse_assignment(expr);
        }

        if let Some(op) = self.parse_compound_op(self.lexer.current()) {
            return self.parse_compound_assignment(expr, op);
        }

        let ident = get_identifier(expr);

        if ident.operator_eq_c_char(c"type".as_ptr()) {
            return self.parse_type_alias(&unsafe { (*expr).base.location }, false, unsafe {
                (*expr).base.location.begin
            });
        }

        if luaur_common::FFlag::DebugLuauUserDefinedClasses.get()
            && ident.operator_eq_c_char(c"class".as_ptr())
        {
            return self.parse_class_stat(&start, false);
        }

        if ident.operator_eq_c_char(c"export".as_ptr()) {
            if luaur_common::FFlag::LuauConst2.get() {
                let current = self.lexer.current();
                let is_local = current.r#type == Type::ReservedLocal;
                let is_function = current.r#type == Type::ReservedFunction;
                let is_const = current.r#type == Type::Name
                    && AstName::ast_name_c_char(unsafe { current.data.name })
                        .operator_eq_c_char(c"const".as_ptr());
                let is_class = luaur_common::FFlag::DebugLuauUserDefinedClasses.get()
                    && current.r#type == Type::Name
                    && AstName::ast_name_c_char(unsafe { current.data.name })
                        .operator_eq_c_char(c"class".as_ptr());

                if is_local || is_function || is_const || is_class {
                    return self.parse_export_value(
                        &unsafe { (*expr).base.location },
                        unsafe { (*expr).base.location.begin },
                        &AstArray {
                            data: core::ptr::null_mut(),
                            size: 0,
                        },
                    );
                } else if current.r#type == Type::Name
                    && AstName::ast_name_c_char(unsafe { current.data.name })
                        .operator_eq_c_char(c"type".as_ptr())
                {
                    let type_keyword_position = current.location.begin;
                    self.next_lexeme();
                    return self.parse_type_alias(
                        &unsafe { (*expr).base.location },
                        true,
                        type_keyword_position,
                    );
                }
            } else if luaur_common::FFlag::DebugLuauUserDefinedClasses.get()
                && AstName::ast_name_c_char(unsafe { self.lexer.current().data.name })
                    .operator_eq_c_char(c"class".as_ptr())
            {
                self.next_lexeme();
                return self.parse_class_stat(&start, true);
            } else if self.lexer.current().r#type == Type::Name
                && AstName::ast_name_c_char(unsafe { self.lexer.current().data.name })
                    .operator_eq_c_char(c"type".as_ptr())
            {
                let type_keyword_position = self.lexer.current().location.begin;
                self.next_lexeme();
                return self.parse_type_alias(
                    &unsafe { (*expr).base.location },
                    true,
                    type_keyword_position,
                );
            }
        }

        if ident.operator_eq_c_char(c"continue".as_ptr()) {
            return self.parser_parse_continue(&unsafe { (*expr).base.location });
        }

        if luaur_common::FFlag::LuauConst2.get() && ident.operator_eq_c_char(c"const".as_ptr()) {
            return self.parse_local(
                unsafe { (*expr).base.location },
                unsafe { (*expr).base.location.begin },
                &AstArray {
                    data: core::ptr::null_mut(),
                    size: 0,
                },
                true,
            );
        }

        if self.options.allow_declaration_syntax {
            if ident.operator_eq_c_char(c"declare".as_ptr()) {
                return self.parse_declaration(
                    &unsafe { (*expr).base.location },
                    &AstArray {
                        data: core::ptr::null_mut(),
                        size: 0,
                    },
                );
            }
        }

        if start
            .begin
            .operator_eq(&self.lexer.current().location.begin)
        {
            self.next_lexeme();
        }

        let expr_location = unsafe { (*expr).base.location };
        let exprs = self.copy_initializer_list_t(&[expr]);

        self.report_stat_error(
            expr_location,
            exprs,
            AstArray {
                data: core::ptr::null_mut(),
                size: 0,
            },
            format_args!("Incomplete statement: expected assignment or a function call"),
        ) as *mut AstStat
    }
}
