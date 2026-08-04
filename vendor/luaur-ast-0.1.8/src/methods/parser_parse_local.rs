use crate::enums::type_lexer::Type;
use crate::functions::is_enough_values::is_enough_values;
use crate::records::ast_array::AstArray;
use crate::records::ast_attr::AstAttr;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_local::AstLocal;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_local::AstStatLocal;
use crate::records::ast_stat_local_function::AstStatLocalFunction;
use crate::records::binding::Binding;
use crate::records::cst_stat_local::CstStatLocal;
use crate::records::cst_stat_local_function::CstStatLocalFunction;
use crate::records::lexeme::Lexeme;
use crate::records::location::Location;
use crate::records::parser::Parser;
use crate::records::position::Position;
use crate::records::temp_vector::TempVector;

impl Parser {
    pub fn parse_local(
        &mut self,
        start: Location,
        keyword_position: Position,
        attributes: &AstArray<*mut AstAttr>,
        is_const: bool,
    ) -> *mut AstStat {
        if !is_const {
            self.next_lexeme();
        }

        if self.lexer.current().r#type == Type::ReservedFunction {
            let mut match_function = *self.lexer.current();
            self.next_lexeme();

            // C++ patches matchFunction's column to where `local` starts so that
            // `local function` and a column-0 `end` align for the missed-indentation
            // "did you forget to close X" suspect heuristic. The port patched only a
            // COPY (function_keyword_position) but passed the UNPATCHED match_function
            // to parse_function_body, so the end-match compared the `function` keyword
            // column (6) against `end` (0) and recorded the wrong suspect.
            let function_keyword_position = match_function.location.begin;
            if match_function.location.begin.line == start.begin.line {
                match_function.location.begin.column = start.begin.column;
            }

            let name = self.parse_name("variable name");

            self.match_recovery_stop_on_token[Type::ReservedEnd.0 as usize] += 1;

            let (body, var) = self.parse_function_body(
                false,
                &match_function,
                &name.name,
                Some(&name),
                attributes,
                is_const,
            );

            self.match_recovery_stop_on_token[Type::ReservedEnd.0 as usize] -= 1;

            let location = Location::new(start.begin, unsafe { (*body).base.base.location.end });

            let node = unsafe {
                (*self.allocator).alloc(AstStatLocalFunction::new(location, var, body, is_const))
            };

            if self.options.store_cst_data {
                let cst_node = unsafe {
                    (*self.allocator).alloc(CstStatLocalFunction::new(
                        keyword_position,
                        function_keyword_position,
                    ))
                };
                self.cst_node_map.try_insert(
                    node as *mut crate::records::ast_node::AstNode,
                    cst_node as *mut crate::records::cst_node::CstNode,
                );
            }

            return node as *mut AstStat;
        } else {
            if attributes.size != 0 {
                return self.report_stat_error(
                    self.lexer.current().location,
                    AstArray {
                        data: core::ptr::null_mut(),
                        size: 0,
                    },
                    AstArray {
                        data: core::ptr::null_mut(),
                        size: 0,
                    },
                    format_args!(
                        "Expected 'function' after local declaration with attribute, but got {} instead",
                        self.lexer.current().to_string()
                    ),
                ) as *mut AstStat;
            }

            self.match_recovery_stop_on_token[Type::Operator.0 as usize] += 1;

            let mut names = TempVector::new(&mut self.scratch_binding);
            let mut vars_comma_positions = AstArray {
                data: core::ptr::null_mut(),
                size: 0,
            };

            if self.options.store_cst_data {
                self.parse_binding_list(
                    &mut names,
                    false,
                    &mut vars_comma_positions,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    is_const,
                );
            } else {
                self.parse_binding_list(
                    &mut names,
                    false,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    is_const,
                );
            }

            self.match_recovery_stop_on_token[Type::Operator.0 as usize] -= 1;

            let mut vars = TempVector::new(&mut self.scratch_local);
            let mut values = TempVector::new(&mut self.scratch_expr);
            let mut values_comma_positions = TempVector::new(&mut self.scratch_position);

            let mut equals_sign_location = None;

            if self.lexer.current().r#type == Type::Operator {
                equals_sign_location = Some(self.lexer.current().location);
                self.next_lexeme();

                if self.options.store_cst_data {
                    self.parse_expr_list(&mut values, Some(&mut values_comma_positions));
                } else {
                    self.parse_expr_list(&mut values, None);
                }
            }

            for i in 0..names.size() {
                vars.push_back(self.push_local(names.operator_index(i)));
            }

            let end = if values.empty() {
                *self.lexer.previous_location()
            } else {
                unsafe { (**values.back()).base.location }
            };

            if luaur_common::FFlag::LuauConstJustReportErrorForUnderfill.get() {
                let node = unsafe {
                    (*self.allocator).alloc(AstStatLocal::new(
                        Location::new(start.begin, end.end),
                        self.copy_temp_vector_t(&vars),
                        self.copy_temp_vector_t(&values),
                        equals_sign_location,
                        is_const,
                    ))
                };

                if self.options.store_cst_data {
                    let cst_node = unsafe {
                        (*self.allocator).alloc(CstStatLocal::new(
                            self.extract_annotation_colon_positions(&names),
                            vars_comma_positions,
                            self.copy_temp_vector_t(&values_comma_positions),
                        ))
                    };
                    self.cst_node_map.try_insert(
                        node as *mut crate::records::ast_node::AstNode,
                        cst_node as *mut crate::records::cst_node::CstNode,
                    );
                }

                if is_const && !is_enough_values(&mut values, vars.size()) {
                    self.report(
                        unsafe { (*node).base.base.location },
                        format_args!("Missing initializer in const declaration"),
                    );
                }

                return node as *mut AstStat;
            } else {
                if is_const && !is_enough_values(&mut values, vars.size()) {
                    return self.report_stat_error(
                        Location::new(start.begin, end.end),
                        AstArray {
                            data: core::ptr::null_mut(),
                            size: 0,
                        },
                        AstArray {
                            data: core::ptr::null_mut(),
                            size: 0,
                        },
                        format_args!("Missing initializer in const declaration"),
                    ) as *mut AstStat;
                }

                let node = unsafe {
                    (*self.allocator).alloc(AstStatLocal::new(
                        Location::new(start.begin, end.end),
                        self.copy_temp_vector_t(&vars),
                        self.copy_temp_vector_t(&values),
                        equals_sign_location,
                        is_const,
                    ))
                };

                if self.options.store_cst_data {
                    let cst_node = unsafe {
                        (*self.allocator).alloc(CstStatLocal::new(
                            self.extract_annotation_colon_positions(&names),
                            vars_comma_positions,
                            self.copy_temp_vector_t(&values_comma_positions),
                        ))
                    };
                    self.cst_node_map.try_insert(
                        node as *mut crate::records::ast_node::AstNode,
                        cst_node as *mut crate::records::cst_node::CstNode,
                    );
                }

                return node as *mut AstStat;
            }
        }
    }
}
