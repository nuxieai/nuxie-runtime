use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_binary::{AstExprBinary, AstExprBinary_Op};
use crate::records::ast_expr_call::AstExprCall;
use crate::records::ast_expr_constant_bool::AstExprConstantBool;
use crate::records::ast_expr_constant_integer::AstExprConstantInteger;
use crate::records::ast_expr_constant_nil::AstExprConstantNil;
use crate::records::ast_expr_constant_number::AstExprConstantNumber;
use crate::records::ast_expr_constant_string::AstExprConstantString;
use crate::records::ast_expr_error::AstExprError;
use crate::records::ast_expr_function::AstExprFunction;
use crate::records::ast_expr_global::AstExprGlobal;
use crate::records::ast_expr_group::AstExprGroup;
use crate::records::ast_expr_if_else::AstExprIfElse;
use crate::records::ast_expr_index_expr::AstExprIndexExpr;
use crate::records::ast_expr_index_name::AstExprIndexName;
use crate::records::ast_expr_instantiate::AstExprInstantiate;
use crate::records::ast_expr_interp_string::AstExprInterpString;
use crate::records::ast_expr_local::AstExprLocal;
use crate::records::ast_expr_table::{AstExprTable, ItemKind};
use crate::records::ast_expr_type_assertion::AstExprTypeAssertion;
use crate::records::ast_expr_unary::{AstExprUnary, AstExprUnaryOp};
use crate::records::ast_expr_varargs::AstExprVarargs;
use crate::records::ast_node::AstNode;
use crate::records::comma_separator_inserter::CommaSeparatorInserter;
use crate::records::cst_expr_call::CstExprCall;
use crate::records::cst_expr_constant_integer::CstExprConstantInteger;
use crate::records::cst_expr_constant_number::CstExprConstantNumber;
use crate::records::cst_expr_constant_string::CstExprConstantString;
use crate::records::cst_expr_explicit_type_instantiation::CstExprExplicitTypeInstantiation;
use crate::records::cst_expr_group::CstExprGroup;
use crate::records::cst_expr_index_expr::CstExprIndexExpr;
use crate::records::cst_expr_interp_string::CstExprInterpString;
use crate::records::cst_expr_op::CstExprOp;
use crate::records::cst_expr_table::{CstExprTable, CstExprTableSeparator};
use crate::records::cst_expr_type_assertion::CstExprTypeAssertion;
use crate::records::printer::Printer;
use crate::rtti::{ast_node_as, ast_node_is};

pub trait IntoAstExprMut {
    unsafe fn into_ast_expr_mut(self) -> *mut AstExpr;
}

impl IntoAstExprMut for *mut AstExpr {
    unsafe fn into_ast_expr_mut(self) -> *mut AstExpr {
        self
    }
}

impl IntoAstExprMut for &*mut AstExpr {
    unsafe fn into_ast_expr_mut(self) -> *mut AstExpr {
        *self
    }
}

impl IntoAstExprMut for &mut AstExpr {
    unsafe fn into_ast_expr_mut(self) -> *mut AstExpr {
        self
    }
}

impl<'a> Printer<'a> {
    pub fn visualize_ast_expr<E: IntoAstExprMut>(&mut self, expr: E) {
        let expr = unsafe { expr.into_ast_expr_mut() };
        if expr.is_null() {
            return;
        }

        let node = expr as *mut AstNode;
        let expr_ref = unsafe { &mut *expr };
        self.advance(&expr_ref.base.location.begin);

        if let Some(a) = unsafe { ast_node_as::<AstExprGroup>(node).as_mut() } {
            self.writer.symbol("(");
            self.visualize_ast_expr(a.expr);

            let cst_node = self.lookup_cst_node_impl::<CstExprGroup>(node);
            if !cst_node.is_null() {
                self.maybe_advance_and_write(unsafe { &(*cst_node).close_position }, ")", false);
            } else {
                self.advance_before(a.base.base.location.end, 1);
                self.writer.symbol(")");
            }
        } else if unsafe { ast_node_is::<AstExprConstantNil>(node) } {
            self.writer.keyword("nil");
        } else if let Some(a) = unsafe { ast_node_as::<AstExprConstantBool>(node).as_mut() } {
            self.writer.keyword(if a.value { "true" } else { "false" });
        } else if let Some(a) = unsafe { ast_node_as::<AstExprConstantNumber>(node).as_mut() } {
            let cst_node = self.lookup_cst_node_impl::<CstExprConstantNumber>(node);
            if !cst_node.is_null() {
                let value = unsafe {
                    core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                        (*cst_node).value.data as *const u8,
                        (*cst_node).value.size,
                    ))
                };
                self.writer.literal(value);
            } else if a.value.is_infinite() {
                self.writer.literal(if a.value.is_sign_positive() {
                    "1e500"
                } else {
                    "-1e500"
                });
            } else if a.value.is_nan() {
                self.writer.literal("0/0");
            } else if Printer::printer_is_integerish(a.value) {
                self.writer.literal(&(a.value as i32).to_string());
            } else {
                self.writer
                    .literal(&luaur_common::functions::format_g::format_g(a.value, 17));
            }
        } else if let Some(a) = unsafe { ast_node_as::<AstExprConstantInteger>(node).as_mut() } {
            let cst_node = self.lookup_cst_node_impl::<CstExprConstantInteger>(node);
            if !cst_node.is_null() {
                let value = unsafe {
                    core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                        (*cst_node).value.data as *const u8,
                        (*cst_node).value.size,
                    ))
                };
                self.writer.literal(value);
            } else if a.value >= 0 {
                self.writer.literal(&format!("{}i", a.value));
            } else {
                self.writer.literal(&format!("0x{:x}i", a.value as u64));
            }
        } else if let Some(a) = unsafe { ast_node_as::<AstExprConstantString>(node).as_mut() } {
            let cst_node = self.lookup_cst_node_impl::<CstExprConstantString>(node);
            if !cst_node.is_null() {
                let source = unsafe {
                    core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                        (*cst_node).source_string.data as *const u8,
                        (*cst_node).source_string.size,
                    ))
                };
                self.writer
                    .source_string(source, unsafe { (*cst_node).quote_style }, unsafe {
                        (*cst_node).block_depth
                    });
            } else {
                let value = unsafe {
                    core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                        a.value.data as *const u8,
                        a.value.size,
                    ))
                };
                self.writer.string(value);
            }
        } else if let Some(a) = unsafe { ast_node_as::<AstExprLocal>(node).as_mut() } {
            let name =
                unsafe { core::ffi::CStr::from_ptr((*a.local).name.value).to_string_lossy() };
            self.writer.identifier(&name);
        } else if let Some(a) = unsafe { ast_node_as::<AstExprGlobal>(node).as_mut() } {
            let name = unsafe { core::ffi::CStr::from_ptr(a.name.value).to_string_lossy() };
            self.writer.identifier(&name);
        } else if unsafe { ast_node_is::<AstExprVarargs>(node) } {
            self.writer.symbol("...");
        } else if let Some(a) = unsafe { ast_node_as::<AstExprCall>(node).as_mut() } {
            self.visualize_ast_expr(a.func);

            let cst_node = self.lookup_cst_node_impl::<CstExprCall>(node);

            if self.write_types
                && (a.type_arguments.size > 0
                    || (!cst_node.is_null() && unsafe { !(*cst_node).explicit_types.is_null() }))
            {
                self.visualize_explicit_type_instantiation(
                    a.type_arguments,
                    if !cst_node.is_null() {
                        unsafe { (*cst_node).explicit_types }
                    } else {
                        core::ptr::null_mut()
                    },
                );
            }

            if !cst_node.is_null() {
                self.maybe_advance_and_write(unsafe { &(*cst_node).open_parens }, "(", false);
            } else {
                self.writer.symbol("(");
            }

            let mut comma = CommaSeparatorInserter::new(
                self.writer,
                if !cst_node.is_null() {
                    unsafe { (*cst_node).comma_positions.data }
                } else {
                    core::ptr::null()
                },
            );
            for arg in a.args.iter() {
                comma.operator_call(self.writer);
                self.visualize_ast_expr(*arg);
            }

            if !cst_node.is_null() {
                self.maybe_advance_and_write(unsafe { &(*cst_node).close_parens }, ")", false);
            } else {
                self.writer.symbol(")");
            }
        } else if let Some(a) = unsafe { ast_node_as::<AstExprIndexName>(node).as_mut() } {
            self.visualize_ast_expr(a.expr);
            self.advance(&a.op_position);
            let op = (a.op as u8 as char).to_string();
            self.writer.symbol(&op);
            self.advance(&a.index_location.begin);
            let index = unsafe { core::ffi::CStr::from_ptr(a.index.value).to_string_lossy() };
            self.writer.write(&index);
        } else if let Some(a) = unsafe { ast_node_as::<AstExprIndexExpr>(node).as_mut() } {
            let cst_node = self.lookup_cst_node_impl::<CstExprIndexExpr>(node);
            self.visualize_ast_expr(a.expr);

            if !cst_node.is_null() {
                self.maybe_advance_and_write(
                    unsafe { &(*cst_node).open_bracket_position },
                    "[",
                    false,
                );
            } else {
                self.writer.symbol("[");
            }

            self.visualize_ast_expr(a.index);

            if !cst_node.is_null() {
                self.maybe_advance_and_write(
                    unsafe { &(*cst_node).close_bracket_position },
                    "]",
                    false,
                );
            } else {
                self.writer.symbol("]");
            }
        } else if let Some(a) = unsafe { ast_node_as::<AstExprFunction>(node).as_mut() } {
            for attr in a.attributes.iter() {
                self.visualize_attribute(unsafe { &mut **attr });
            }
            self.writer.keyword("function");
            self.visualize_function_body(a);
        } else if let Some(a) = unsafe { ast_node_as::<AstExprTable>(node).as_mut() } {
            self.writer.symbol("{");

            let cst_node = self.lookup_cst_node_impl::<CstExprTable>(node);
            let mut cst_item = if !cst_node.is_null() {
                luaur_common::LUAU_ASSERT!(unsafe { (*cst_node).items.size == a.items.size });
                unsafe { (*cst_node).items.data }
            } else {
                core::ptr::null()
            };
            let mut first = true;

            for item in a.items.iter() {
                if cst_item.is_null() {
                    if first {
                        first = false;
                    } else {
                        self.writer.symbol(",");
                    }
                }

                match item.kind {
                    ItemKind::List => {}
                    ItemKind::Record => {
                        let key = unsafe {
                            ast_node_as::<AstExprConstantString>(
                                item.key as *mut crate::records::ast_node::AstNode,
                            )
                        };
                        luaur_common::LUAU_ASSERT!(!key.is_null());
                        let key = unsafe { &mut *key };
                        self.advance(&key.base.base.location.begin);
                        let value = unsafe {
                            core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                                key.value.data as *const u8,
                                key.value.size,
                            ))
                        };
                        self.writer.identifier(value);

                        if !cst_item.is_null() {
                            self.advance(unsafe { (*cst_item).equals_position });
                        } else {
                            unsafe {
                                self.writer
                                    .maybe_space(&(*item.value).base.location.begin, 1);
                            }
                        }
                        self.writer.symbol("=");
                    }
                    ItemKind::General => {
                        if !cst_item.is_null() {
                            luaur_common::LUAU_ASSERT!(unsafe {
                                (*cst_item).indexer_open_position.has_value()
                            });
                            self.maybe_advance_and_write(
                                unsafe { &(*cst_item).indexer_open_position },
                                "[",
                                true,
                            );
                            self.visualize_ast_expr(item.key);
                            self.maybe_advance_and_write(
                                unsafe { &(*cst_item).indexer_close_position },
                                "]",
                                false,
                            );
                            self.maybe_advance_and_write(
                                unsafe { &(*cst_item).equals_position },
                                "=",
                                false,
                            );
                        } else {
                            self.writer.symbol("[");
                            self.visualize_ast_expr(item.key);
                            self.writer.symbol("]");
                            unsafe {
                                self.writer
                                    .maybe_space(&(*item.value).base.location.begin, 1);
                            }
                            self.writer.symbol("=");
                        }
                    }
                }

                unsafe {
                    self.advance(&(*item.value).base.location.begin);
                }
                self.visualize_ast_expr(item.value);

                if !cst_item.is_null() {
                    let separator = unsafe { (*cst_item).separator };
                    if separator != CstExprTableSeparator::Missing {
                        luaur_common::LUAU_ASSERT!(unsafe {
                            (*cst_item).separator_position.has_value()
                        });
                        self.maybe_advance_and_write(
                            unsafe { &(*cst_item).separator_position },
                            if separator == CstExprTableSeparator::Comma {
                                ","
                            } else {
                                ";"
                            },
                            true,
                        );
                    }
                    cst_item = unsafe { cst_item.add(1) };
                }
            }

            let mut end_pos = expr_ref.base.location.end;
            if end_pos.column > 0 {
                end_pos.column -= 1;
            }
            self.advance(end_pos);
            self.writer.symbol("}");
            self.advance(expr_ref.base.location.end);
        } else if let Some(a) = unsafe { ast_node_as::<AstExprUnary>(node).as_mut() } {
            let cst_node = self.lookup_cst_node_impl::<CstExprOp>(node);
            if !cst_node.is_null() {
                self.advance(unsafe { (*cst_node).op_position });
            }

            match a.op {
                AstExprUnaryOp::Not => self.writer.keyword("not"),
                AstExprUnaryOp::Minus => self.writer.symbol("-"),
                AstExprUnaryOp::Len => self.writer.symbol("#"),
            }
            self.visualize_ast_expr(a.expr);
        } else if let Some(a) = unsafe { ast_node_as::<AstExprBinary>(node).as_mut() } {
            self.visualize_ast_expr(a.left);

            let cst_node = self.lookup_cst_node_impl::<CstExprOp>(node);
            if !cst_node.is_null() {
                self.advance(unsafe { (*cst_node).op_position });
            } else {
                match a.op {
                    AstExprBinary_Op::Add
                    | AstExprBinary_Op::Sub
                    | AstExprBinary_Op::Mul
                    | AstExprBinary_Op::Div
                    | AstExprBinary_Op::FloorDiv
                    | AstExprBinary_Op::Mod
                    | AstExprBinary_Op::Pow
                    | AstExprBinary_Op::CompareLt
                    | AstExprBinary_Op::CompareGt => unsafe {
                        self.writer.maybe_space(&(*a.right).base.location.begin, 2);
                    },
                    AstExprBinary_Op::Concat
                    | AstExprBinary_Op::CompareNe
                    | AstExprBinary_Op::CompareEq
                    | AstExprBinary_Op::CompareLe
                    | AstExprBinary_Op::CompareGe
                    | AstExprBinary_Op::Or => unsafe {
                        self.writer.maybe_space(&(*a.right).base.location.begin, 3);
                    },
                    AstExprBinary_Op::And => unsafe {
                        self.writer.maybe_space(&(*a.right).base.location.begin, 4);
                    },
                    AstExprBinary_Op::Op__Count => luaur_common::LUAU_ASSERT!(false),
                }
            }

            let op = crate::functions::to_string_ast_alt_b::to_string(a.op);
            self.writer.symbol(&op);
            self.visualize_ast_expr(a.right);
        } else if let Some(a) = unsafe { ast_node_as::<AstExprTypeAssertion>(node).as_mut() } {
            self.visualize_ast_expr(a.expr);

            if self.write_types {
                let cst_node = self.lookup_cst_node_impl::<CstExprTypeAssertion>(node);
                if !cst_node.is_null() {
                    self.advance(unsafe { (*cst_node).op_position });
                } else {
                    unsafe {
                        self.writer
                            .maybe_space(&(*a.annotation).base.location.begin, 2);
                    }
                }
                self.writer.symbol("::");
                unsafe {
                    self.visualize_type_annotation(&mut *a.annotation);
                }
            }
        } else if let Some(a) = unsafe { ast_node_as::<AstExprIfElse>(node).as_mut() } {
            self.writer.keyword("if");
            self.visualize_else_if_expr(a);
        } else if let Some(a) = unsafe { ast_node_as::<AstExprInterpString>(node).as_mut() } {
            let cst_node = self.lookup_cst_node_impl::<CstExprInterpString>(node);

            self.writer.symbol("`");

            for index in 0..a.strings.size {
                if !cst_node.is_null() {
                    if index > 0 {
                        self.advance(unsafe { *(*cst_node).string_positions.data.add(index) });
                        self.writer.symbol("}");
                    }

                    let source_string = unsafe { *(*cst_node).source_strings.data.add(index) };
                    let source = unsafe {
                        core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                            source_string.data as *const u8,
                            source_string.size,
                        ))
                    };
                    self.writer.write_multiline(source);
                } else {
                    let string = unsafe { *a.strings.data.add(index) };
                    let value = unsafe {
                        core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                            string.data as *const u8,
                            string.size,
                        ))
                    };
                    self.writer
                        .write(&luaur_common::functions::escape::escape(value, true));
                }

                if index < a.expressions.size {
                    self.writer.symbol("{");
                    self.visualize_ast_expr(unsafe { *a.expressions.data.add(index) });
                    if cst_node.is_null() {
                        self.writer.symbol("}");
                    }
                }
            }

            self.writer.symbol("`");
        } else if let Some(a) = unsafe { ast_node_as::<AstExprError>(node).as_mut() } {
            self.writer.symbol("(error-expr");

            for i in 0..a.expressions.size {
                self.writer.symbol(if i == 0 { ": " } else { ", " });
                self.visualize_ast_expr(unsafe { *a.expressions.data.add(i as usize) });
            }

            self.writer.symbol(")");
        } else if let Some(a) = unsafe { ast_node_as::<AstExprInstantiate>(node).as_mut() } {
            self.visualize_ast_expr(a.expr);

            if self.write_types {
                let cst_expr_node =
                    self.lookup_cst_node_impl::<CstExprExplicitTypeInstantiation>(node);
                self.visualize_explicit_type_instantiation(
                    a.type_arguments,
                    if !cst_expr_node.is_null() {
                        unsafe { &(*cst_expr_node).instantiation }
                    } else {
                        core::ptr::null()
                    },
                );
            }
        } else {
            luaur_common::LUAU_ASSERT!(false);
        }
    }
}
