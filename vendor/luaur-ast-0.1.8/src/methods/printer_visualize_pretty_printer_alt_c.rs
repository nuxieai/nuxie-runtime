use crate::records::ast_array::AstArray;
use crate::records::ast_attr::AstAttr;
use crate::records::ast_class_method::AstClassMethod;
use crate::records::ast_class_property::AstClassProperty;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_binary::AstExprBinary;
use crate::records::ast_expr_binary::AstExprBinary_Op;
use crate::records::ast_local::AstLocal;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_assign::AstStatAssign;
use crate::records::ast_stat_block::AstStatBlock;
use crate::records::ast_stat_break::AstStatBreak;
use crate::records::ast_stat_class::AstStatClass;
use crate::records::ast_stat_compound_assign::AstStatCompoundAssign;
use crate::records::ast_stat_continue::AstStatContinue;
use crate::records::ast_stat_declare_global::AstStatDeclareGlobal;
use crate::records::ast_stat_error::AstStatError;
use crate::records::ast_stat_expr::AstStatExpr;
use crate::records::ast_stat_for::AstStatFor;
use crate::records::ast_stat_for_in::AstStatForIn;
use crate::records::ast_stat_function::AstStatFunction;
use crate::records::ast_stat_if::AstStatIf;
use crate::records::ast_stat_local::AstStatLocal;
use crate::records::ast_stat_local_function::AstStatLocalFunction;
use crate::records::ast_stat_repeat::AstStatRepeat;
use crate::records::ast_stat_return::AstStatReturn;
use crate::records::ast_stat_type_alias::AstStatTypeAlias;
use crate::records::ast_stat_type_function::AstStatTypeFunction;
use crate::records::ast_stat_while::AstStatWhile;
use crate::records::comma_separator_inserter::CommaSeparatorInserter;
use crate::records::cst_generic_type::CstGenericType;
use crate::records::cst_generic_type_pack::CstGenericTypePack;
use crate::records::cst_stat_assign::CstStatAssign;
use crate::records::cst_stat_compound_assign::CstStatCompoundAssign;
use crate::records::cst_stat_do::CstStatDo;
use crate::records::cst_stat_for::CstStatFor;
use crate::records::cst_stat_for_in::CstStatForIn;
use crate::records::cst_stat_function::CstStatFunction;
use crate::records::cst_stat_local::CstStatLocal;
use crate::records::cst_stat_local_function::CstStatLocalFunction;
use crate::records::cst_stat_repeat::CstStatRepeat;
use crate::records::cst_stat_return::CstStatReturn;
use crate::records::cst_stat_type_alias::CstStatTypeAlias;
use crate::records::cst_stat_type_function::CstStatTypeFunction;
use crate::records::position::Position;
use crate::records::printer::Printer;
use crate::records::string_writer::StringWriter;
use crate::records::writer::Writer;
use crate::rtti::CstNodeClass;
use crate::visit::ast_expr_visit;
use crate::visit::ast_stat_visit;
use luaur_common::functions::visit_variant::visit;
use luaur_common::records::overloaded;
use luaur_common::records::variant::Variant2;
use luaur_common::FFlag;
use luaur_common::LUAU_ASSERT;

pub trait IntoAstStatPrinter {
    unsafe fn into_ast_stat_mut(self) -> *mut AstStat;
}

impl IntoAstStatPrinter for &mut AstStat {
    unsafe fn into_ast_stat_mut(self) -> *mut AstStat {
        self
    }
}

impl IntoAstStatPrinter for *mut AstStat {
    unsafe fn into_ast_stat_mut(self) -> *mut AstStat {
        self
    }
}

impl IntoAstStatPrinter for &*mut AstStat {
    unsafe fn into_ast_stat_mut(self) -> *mut AstStat {
        *self
    }
}

impl<'a> Printer<'a> {
    #[allow(non_snake_case)]
    pub fn visualize_ast_stat<S: IntoAstStatPrinter>(&mut self, program: S) {
        let program = unsafe { &mut *program.into_ast_stat_mut() };
        self.advance(&program.base.location.begin);

        if let Some(block) = unsafe {
            crate::rtti::ast_node_as::<AstStatBlock>(
                program as *mut AstStat as *mut crate::records::ast_node::AstNode,
            )
            .as_mut()
        } {
            let cst_node = self.lookup_cst_node::<CstStatDo>(
                program as *mut AstStat as *mut crate::records::ast_node::AstNode,
            );
            if !cst_node.is_null() {
                self.writer.keyword("do");
                self.advance(&unsafe { (*cst_node).stats_start_position });
                for s in unsafe { crate::records::ast_array::AstArray::iter(&block.body) } {
                    self.visualize_ast_stat(s);
                }
                self.maybe_advance_and_write(unsafe { &(*cst_node).end_position }, "end", false);
            } else {
                for s in unsafe { crate::records::ast_array::AstArray::iter(&block.body) } {
                    self.visualize_ast_stat(s);
                }
                self.advance(&block.base.base.location.end);
                self.write_end(&program.base.location);
            }
        } else if let Some(a) = unsafe {
            crate::rtti::ast_node_as::<AstStatIf>(
                program as *mut AstStat as *mut crate::records::ast_node::AstNode,
            )
            .as_mut()
        } {
            self.writer.keyword("if");
            self.visualize_else_if(a);
        } else if let Some(a) = unsafe {
            crate::rtti::ast_node_as::<AstStatWhile>(
                program as *mut AstStat as *mut crate::records::ast_node::AstNode,
            )
            .as_mut()
        } {
            self.writer.keyword("while");
            self.visualize_ast_expr(a.condition);
            self.advance(&a.do_location.begin);
            self.writer.keyword("do");
            self.visualize_block_ast_stat_block(a.body);
            self.advance(&unsafe { (*a.body).base.base.location.end });
            self.writer.keyword("end");
        } else if let Some(a) = unsafe {
            crate::rtti::ast_node_as::<AstStatRepeat>(
                program as *mut AstStat as *mut crate::records::ast_node::AstNode,
            )
            .as_mut()
        } {
            self.writer.keyword("repeat");
            self.visualize_block_ast_stat_block(a.body);
            let cst_node = self.lookup_cst_node::<CstStatRepeat>(
                program as *mut AstStat as *mut crate::records::ast_node::AstNode,
            );
            if !cst_node.is_null() {
                self.maybe_advance_and_write(
                    unsafe { &(*cst_node).until_position },
                    "until",
                    false,
                );
            } else {
                self.advance_before(unsafe { (*a.condition).base.location.begin }, 6);
                self.writer.keyword("until");
            }
            self.visualize_ast_expr(a.condition);
        } else if unsafe {
            crate::rtti::ast_node_is::<AstStatBreak>(
                program as *mut AstStat as *mut crate::records::ast_node::AstNode,
            )
        } {
            self.writer.keyword("break");
        } else if unsafe {
            crate::rtti::ast_node_is::<AstStatContinue>(
                program as *mut AstStat as *mut crate::records::ast_node::AstNode,
            )
        } {
            self.writer.keyword("continue");
        } else if let Some(a) = unsafe {
            crate::rtti::ast_node_as::<AstStatReturn>(
                program as *mut AstStat as *mut crate::records::ast_node::AstNode,
            )
            .as_mut()
        } {
            let cst_node = self.lookup_cst_node::<CstStatReturn>(
                program as *mut AstStat as *mut crate::records::ast_node::AstNode,
            );
            self.writer.keyword("return");
            let mut comma = CommaSeparatorInserter::new(
                self.writer,
                if !cst_node.is_null() {
                    unsafe { (*cst_node).comma_positions.data }
                } else {
                    core::ptr::null()
                },
            );
            for expr in unsafe { crate::records::ast_array::AstArray::iter(&a.list) } {
                comma.operator_call(self.writer);
                self.visualize_ast_expr(expr);
            }
        } else if let Some(a) = unsafe {
            crate::rtti::ast_node_as::<AstStatExpr>(
                program as *mut AstStat as *mut crate::records::ast_node::AstNode,
            )
            .as_mut()
        } {
            self.visualize_ast_expr(a.expr);
        } else if let Some(a) = unsafe {
            crate::rtti::ast_node_as::<AstStatLocal>(
                program as *mut AstStat as *mut crate::records::ast_node::AstNode,
            )
            .as_mut()
        } {
            let cst_node = self.lookup_cst_node::<CstStatLocal>(
                program as *mut AstStat as *mut crate::records::ast_node::AstNode,
            );
            if FFlag::LuauExportValueSyntax.get() && FFlag::LuauConst2.get() && a.is_exported {
                self.writer.keyword("export");
                if !cst_node.is_null() {
                    self.advance(unsafe { (*cst_node).declaration_keyword_position });
                }
                self.writer
                    .keyword(if a.is_const { "const" } else { "local" });
            } else if FFlag::LuauConst2.get() && a.is_const {
                self.writer.keyword("const");
            } else {
                self.writer.keyword("local");
            }
            let mut var_comma = CommaSeparatorInserter::new(
                self.writer,
                if !cst_node.is_null() {
                    unsafe { (*cst_node).vars_comma_positions.data }
                } else {
                    core::ptr::null()
                },
            );
            for i in 0..a.vars.size {
                var_comma.operator_call(self.writer);
                if !cst_node.is_null() {
                    LUAU_ASSERT!(unsafe { (*cst_node).vars_annotation_colon_positions.size > i });
                    self.visualize_ast_local_position(
                        unsafe { &*(*a.vars.data.add(i as usize)) },
                        unsafe {
                            *(*cst_node)
                                .vars_annotation_colon_positions
                                .data
                                .add(i as usize)
                        },
                    );
                } else {
                    self.visualize_ast_local_position(
                        unsafe { &*(*a.vars.data.add(i as usize)) },
                        Position::missing(),
                    );
                }
            }
            if let Some(loc) = a.equals_sign_location {
                self.advance(&loc.begin);
                self.writer.symbol("=");
            }
            let mut value_comma = CommaSeparatorInserter::new(
                self.writer,
                if !cst_node.is_null() {
                    unsafe { (*cst_node).values_comma_positions.data }
                } else {
                    core::ptr::null()
                },
            );
            for value in unsafe { crate::records::ast_array::AstArray::iter(&a.values) } {
                value_comma.operator_call(self.writer);
                self.visualize_ast_expr(value);
            }
        } else if let Some(a) = unsafe {
            crate::rtti::ast_node_as::<AstStatFor>(
                program as *mut AstStat as *mut crate::records::ast_node::AstNode,
            )
            .as_mut()
        } {
            let cst_node = self.lookup_cst_node::<CstStatFor>(
                program as *mut AstStat as *mut crate::records::ast_node::AstNode,
            );
            self.writer.keyword("for");
            self.visualize_ast_local_position(
                unsafe { &*a.var },
                if !cst_node.is_null() {
                    unsafe { (*cst_node).annotation_colon_position }
                } else {
                    Position::missing()
                },
            );
            if !cst_node.is_null() {
                self.advance(&unsafe { (*cst_node).equals_position });
            }
            self.writer.symbol("=");
            self.visualize_ast_expr(a.from);
            if !cst_node.is_null() {
                self.maybe_advance_and_write(
                    &unsafe { (*cst_node).end_comma_position },
                    ",",
                    false,
                );
            } else {
                self.writer.symbol(",");
            }
            self.visualize_ast_expr(a.to);
            if !a.step.is_null() {
                if !cst_node.is_null() {
                    self.advance(unsafe { (*cst_node).step_comma_position });
                }
                self.writer.symbol(",");
                self.visualize_ast_expr(a.step);
            }
            self.advance(&a.do_location.begin);
            self.writer.keyword("do");
            self.visualize_block_ast_stat_block(a.body);
            self.advance(&unsafe { (*a.body).base.base.location.end });
            self.writer.keyword("end");
        } else if let Some(a) = unsafe {
            crate::rtti::ast_node_as::<AstStatForIn>(
                program as *mut AstStat as *mut crate::records::ast_node::AstNode,
            )
            .as_mut()
        } {
            let cst_node = self.lookup_cst_node::<CstStatForIn>(
                program as *mut AstStat as *mut crate::records::ast_node::AstNode,
            );
            self.writer.keyword("for");
            let mut var_comma = CommaSeparatorInserter::new(
                self.writer,
                if !cst_node.is_null() {
                    unsafe { (*cst_node).vars_comma_positions.data }
                } else {
                    core::ptr::null()
                },
            );
            for i in 0..a.vars.size {
                var_comma.operator_call(self.writer);
                if !cst_node.is_null() {
                    LUAU_ASSERT!(unsafe { (*cst_node).vars_annotation_colon_positions.size > i });
                    self.visualize_ast_local_position(
                        unsafe { &*(*a.vars.data.add(i as usize)) },
                        unsafe {
                            *(*cst_node)
                                .vars_annotation_colon_positions
                                .data
                                .add(i as usize)
                        },
                    );
                } else {
                    self.visualize_ast_local_position(
                        unsafe { &*(*a.vars.data.add(i as usize)) },
                        Position::missing(),
                    );
                }
            }
            self.advance(&a.in_location.begin);
            self.writer.keyword("in");
            let mut val_comma = CommaSeparatorInserter::new(
                self.writer,
                if !cst_node.is_null() {
                    unsafe { (*cst_node).values_comma_positions.data }
                } else {
                    core::ptr::null()
                },
            );
            for val in unsafe { crate::records::ast_array::AstArray::iter(&a.values) } {
                val_comma.operator_call(self.writer);
                self.visualize_ast_expr(val);
            }
            self.advance(&a.do_location.begin);
            self.writer.keyword("do");
            self.visualize_block_ast_stat_block(a.body);
            self.advance(&unsafe { (*a.body).base.base.location.end });
            self.writer.keyword("end");
        } else if let Some(a) = unsafe {
            crate::rtti::ast_node_as::<AstStatAssign>(
                program as *mut AstStat as *mut crate::records::ast_node::AstNode,
            )
            .as_mut()
        } {
            let cst_node = self.lookup_cst_node::<CstStatAssign>(
                program as *mut AstStat as *mut crate::records::ast_node::AstNode,
            );
            let mut var_comma = CommaSeparatorInserter::new(
                self.writer,
                if !cst_node.is_null() {
                    unsafe { (*cst_node).vars_comma_positions.data }
                } else {
                    core::ptr::null()
                },
            );
            for var in unsafe { crate::records::ast_array::AstArray::iter(&a.vars) } {
                var_comma.operator_call(self.writer);
                self.visualize_ast_expr(var);
            }
            if !cst_node.is_null() {
                self.maybe_advance_and_write(&unsafe { (*cst_node).equals_position }, "=", false);
            } else {
                self.writer.space();
                self.writer.symbol("=");
            }
            let mut value_comma = CommaSeparatorInserter::new(
                self.writer,
                if !cst_node.is_null() {
                    unsafe { (*cst_node).values_comma_positions.data }
                } else {
                    core::ptr::null()
                },
            );
            for value in unsafe { crate::records::ast_array::AstArray::iter(&a.values) } {
                value_comma.operator_call(self.writer);
                self.visualize_ast_expr(value);
            }
        } else if let Some(a) = unsafe {
            crate::rtti::ast_node_as::<AstStatCompoundAssign>(
                program as *mut AstStat as *mut crate::records::ast_node::AstNode,
            )
            .as_mut()
        } {
            let cst_node = self.lookup_cst_node::<CstStatCompoundAssign>(
                program as *mut AstStat as *mut crate::records::ast_node::AstNode,
            );
            self.visualize_ast_expr(a.var);
            if !cst_node.is_null() {
                self.advance(unsafe { (*cst_node).op_position });
            }
            match a.op {
                AstExprBinary_Op::Add => {
                    if cst_node.is_null() {
                        self.writer
                            .maybe_space(&unsafe { (*a.value).base.location.begin }, 2);
                    }
                    self.writer.symbol("+=");
                }
                AstExprBinary_Op::Sub => {
                    if cst_node.is_null() {
                        self.writer
                            .maybe_space(&unsafe { (*a.value).base.location.begin }, 2);
                    }
                    self.writer.symbol("-=");
                }
                AstExprBinary_Op::Mul => {
                    if cst_node.is_null() {
                        self.writer
                            .maybe_space(&unsafe { (*a.value).base.location.begin }, 2);
                    }
                    self.writer.symbol("*=");
                }
                AstExprBinary_Op::Div => {
                    if cst_node.is_null() {
                        self.writer
                            .maybe_space(&unsafe { (*a.value).base.location.begin }, 2);
                    }
                    self.writer.symbol("/=");
                }
                AstExprBinary_Op::FloorDiv => {
                    if cst_node.is_null() {
                        self.writer
                            .maybe_space(&unsafe { (*a.value).base.location.begin }, 3);
                    }
                    self.writer.symbol("//=");
                }
                AstExprBinary_Op::Mod => {
                    if cst_node.is_null() {
                        self.writer
                            .maybe_space(&unsafe { (*a.value).base.location.begin }, 2);
                    }
                    self.writer.symbol("%=");
                }
                AstExprBinary_Op::Pow => {
                    if cst_node.is_null() {
                        self.writer
                            .maybe_space(&unsafe { (*a.value).base.location.begin }, 2);
                    }
                    self.writer.symbol("^=");
                }
                AstExprBinary_Op::Concat => {
                    if cst_node.is_null() {
                        self.writer
                            .maybe_space(&unsafe { (*a.value).base.location.begin }, 3);
                    }
                    self.writer.symbol("..=");
                }
                _ => {
                    LUAU_ASSERT!(false);
                }
            }
            self.visualize_ast_expr(a.value);
        } else if let Some(a) = unsafe {
            crate::rtti::ast_node_as::<AstStatFunction>(
                program as *mut AstStat as *mut crate::records::ast_node::AstNode,
            )
            .as_mut()
        } {
            for attr in unsafe { crate::records::ast_array::AstArray::iter(&(*a.func).attributes) }
            {
                self.visualize_attribute(unsafe { &mut **attr });
            }
            let cst_node = self.lookup_cst_node::<CstStatFunction>(
                program as *mut AstStat as *mut crate::records::ast_node::AstNode,
            );
            if !cst_node.is_null() {
                self.advance(unsafe { (*cst_node).function_keyword_position });
            }
            self.writer.keyword("function");
            self.visualize_ast_expr(a.name);
            self.visualize_function_body(a.func);
        } else if let Some(a) = unsafe {
            crate::rtti::ast_node_as::<AstStatLocalFunction>(
                program as *mut AstStat as *mut crate::records::ast_node::AstNode,
            )
            .as_mut()
        } {
            for attr in unsafe { crate::records::ast_array::AstArray::iter(&(*a.func).attributes) }
            {
                self.visualize_attribute(unsafe { &mut **attr });
            }
            let cst_node = self.lookup_cst_node::<CstStatLocalFunction>(
                program as *mut AstStat as *mut crate::records::ast_node::AstNode,
            );
            if !cst_node.is_null() {
                self.advance(unsafe { (*cst_node).local_keyword_position });
            }
            if FFlag::LuauExportValueSyntax.get()
                && FFlag::LuauConst2.get()
                && unsafe { (*a.name).is_exported }
            {
                self.writer.keyword("export");
            } else if FFlag::LuauConst2.get() && unsafe { (*a.name).is_const } {
                self.writer.keyword("const");
            } else {
                self.writer.keyword("local");
            }
            if !cst_node.is_null() {
                self.advance(unsafe { (*cst_node).function_keyword_position });
            } else {
                self.writer.space();
            }
            self.writer.keyword("function");
            self.advance(unsafe { (*a.name).location.begin });
            let name = unsafe { core::ffi::CStr::from_ptr((*a.name).name.value).to_string_lossy() };
            self.writer.identifier(&name);
            self.visualize_function_body(a.func);
        } else if let Some(a) = unsafe {
            crate::rtti::ast_node_as::<AstStatTypeAlias>(
                program as *mut AstStat as *mut crate::records::ast_node::AstNode,
            )
            .as_mut()
        } {
            if self.write_types {
                let cst_node = self.lookup_cst_node::<CstStatTypeAlias>(
                    program as *mut AstStat as *mut crate::records::ast_node::AstNode,
                );
                if a.exported {
                    self.writer.keyword("export");
                }
                if !cst_node.is_null() {
                    self.advance(unsafe { (*cst_node).type_keyword_position });
                }
                self.writer.keyword("type");
                self.advance(a.name_location.begin);
                let name = unsafe { core::ffi::CStr::from_ptr(a.name.value).to_string_lossy() };
                self.writer.identifier(&name);
                if a.generics.size > 0 || a.generic_packs.size > 0 {
                    if !cst_node.is_null() {
                        self.advance(unsafe { (*cst_node).generics_open_position });
                    }
                    self.writer.symbol("<");
                    let mut comma = CommaSeparatorInserter::new(
                        self.writer,
                        if !cst_node.is_null() {
                            unsafe { (*cst_node).generics_comma_positions.data }
                        } else {
                            core::ptr::null()
                        },
                    );
                    for o in unsafe { crate::records::ast_array::AstArray::iter(&a.generics) } {
                        let o = *o;
                        comma.operator_call(self.writer);
                        self.writer.advance(unsafe { &(*o).base.location.begin });
                        let name =
                            unsafe { core::ffi::CStr::from_ptr((*o).name.value).to_string_lossy() };
                        self.writer.identifier(&name);
                        if !unsafe { (*o).default_value.is_null() } {
                            let generic_type_cst_node = self.lookup_cst_node::<CstGenericType>(
                                o as *mut crate::records::ast_node::AstNode,
                            );
                            if !generic_type_cst_node.is_null() {
                                self.advance(unsafe {
                                    (*generic_type_cst_node).default_equals_position
                                });
                            } else {
                                self.writer.maybe_space(
                                    unsafe { &(*(*o).default_value).base.location.begin },
                                    2,
                                );
                            }
                            self.writer.symbol("=");
                            self.visualize_type_annotation(unsafe { (*o).default_value });
                        }
                    }
                    for o in unsafe { crate::records::ast_array::AstArray::iter(&a.generic_packs) }
                    {
                        let o = *o;
                        comma.operator_call(self.writer);
                        let generic_type_pack_cst_node = self
                            .lookup_cst_node::<CstGenericTypePack>(
                                o as *mut crate::records::ast_node::AstNode,
                            );
                        self.writer.advance(unsafe { &(*o).base.location.begin });
                        let name =
                            unsafe { core::ffi::CStr::from_ptr((*o).name.value).to_string_lossy() };
                        self.writer.identifier(&name);
                        if !generic_type_pack_cst_node.is_null() {
                            self.maybe_advance_and_write(
                                &unsafe { (*generic_type_pack_cst_node).ellipsis_position },
                                "...",
                                false,
                            );
                        } else {
                            self.writer.symbol("...");
                        }
                        if !unsafe { (*o).default_value.is_null() } {
                            if !cst_node.is_null() {
                                self.advance(unsafe {
                                    (*generic_type_pack_cst_node).default_equals_position
                                });
                            } else {
                                self.writer.maybe_space(
                                    unsafe { &(*(*o).default_value).base.location.begin },
                                    2,
                                );
                            }
                            self.writer.symbol("=");
                            self.visualize_type_pack_annotation(
                                unsafe { &mut *(*o).default_value },
                                false,
                                false,
                                false,
                            );
                        }
                    }
                    if !cst_node.is_null() {
                        self.maybe_advance_and_write(
                            &unsafe { (*cst_node).generics_close_position },
                            ">",
                            false,
                        );
                    } else {
                        self.writer.symbol(">");
                    }
                }
                if !cst_node.is_null() {
                    self.maybe_advance_and_write(
                        &unsafe { (*cst_node).equals_position },
                        "=",
                        false,
                    );
                } else {
                    self.writer
                        .maybe_space(unsafe { &(*a.type_ptr).base.location.begin }, 2);
                    self.writer.symbol("=");
                }
                self.visualize_type_annotation(a.type_ptr);
            }
        } else if let Some(t) = unsafe {
            crate::rtti::ast_node_as::<AstStatTypeFunction>(
                program as *mut AstStat as *mut crate::records::ast_node::AstNode,
            )
            .as_mut()
        } {
            if self.write_types {
                let cst_node = self.lookup_cst_node::<CstStatTypeFunction>(
                    program as *mut AstStat as *mut crate::records::ast_node::AstNode,
                );
                if t.exported {
                    self.writer.keyword("export");
                }
                if !cst_node.is_null() {
                    self.advance(unsafe { (*cst_node).type_keyword_position });
                } else {
                    self.writer.space();
                }
                self.writer.keyword("type");
                if !cst_node.is_null() {
                    self.advance(unsafe { (*cst_node).function_keyword_position });
                } else {
                    self.writer.space();
                }
                self.writer.keyword("function");
                self.advance(t.name_location.begin);
                let name = unsafe { core::ffi::CStr::from_ptr(t.name.value).to_string_lossy() };
                self.writer.identifier(&name);
                self.visualize_function_body(t.body);
            }
        } else if let Some(a) = unsafe {
            crate::rtti::ast_node_as::<AstStatError>(
                program as *mut AstStat as *mut crate::records::ast_node::AstNode,
            )
            .as_mut()
        } {
            self.writer.symbol("(error-stat");
            for i in 0..a.expressions.size {
                self.writer.symbol(if i == 0 && a.statements.size == 0 {
                    ": "
                } else {
                    ", "
                });
                self.visualize_ast_expr(unsafe { *a.expressions.data.add(i as usize) });
            }
            for i in 0..a.statements.size {
                self.writer.symbol(if i == 0 && a.expressions.size == 0 {
                    ": "
                } else {
                    ", "
                });
                self.visualize_ast_stat(unsafe { *a.statements.data.add(i as usize) });
            }
            self.writer.symbol(")");
        } else if let Some(a) = unsafe {
            crate::rtti::ast_node_as::<AstStatDeclareGlobal>(
                program as *mut AstStat as *mut crate::records::ast_node::AstNode,
            )
            .as_mut()
        } {
            self.writer.keyword("declare");
            self.advance(&a.name_location.begin);
            let name = unsafe { core::ffi::CStr::from_ptr(a.name.value).to_string_lossy() };
            self.writer.identifier(&name);
            self.writer.symbol(":");
            self.visualize_type_annotation(a.type_);
        } else if let Some(c) = unsafe {
            crate::rtti::ast_node_as::<AstStatClass>(
                program as *mut AstStat as *mut crate::records::ast_node::AstNode,
            )
            .as_mut()
        } {
            if FFlag::DebugLuauUserDefinedClasses.get() {
                self.writer.keyword("class");
                self.advance(unsafe { (*c.name).location.begin });
                let name =
                    unsafe { core::ffi::CStr::from_ptr((*c.name).name.value).to_string_lossy() };
                self.writer.identifier(&name);
                for member in unsafe { crate::records::ast_array::AstArray::iter(&c.members) } {
                    match member {
                        Variant2::V0(prop) => {
                            let prop: &AstClassProperty = prop;
                            self.advance(prop.qualifier_location.begin);
                            self.writer.keyword("public");
                            self.advance(prop.name_location.begin);
                            let name = unsafe {
                                core::ffi::CStr::from_ptr(prop.name.value).to_string_lossy()
                            };
                            self.writer.identifier(&name);
                            if self.write_types && !prop.ty.is_null() {
                                LUAU_ASSERT!(prop.type_colon_location.is_some());
                                self.advance(prop.type_colon_location.unwrap().begin);
                                self.writer.symbol(":");
                                self.visualize_type_annotation(prop.ty);
                            }
                        }
                        Variant2::V1(method) => {
                            let method: &AstClassMethod = method;
                            if let Some(qualifier_location) = method.qualifier_location {
                                self.advance(&qualifier_location.begin);
                                self.writer.keyword("public");
                            }
                            self.advance(method.keyword_location.begin);
                            self.writer.keyword("function");
                            self.advance(method.name_location.begin);
                            let name = unsafe {
                                core::ffi::CStr::from_ptr(method.function_name.value)
                                    .to_string_lossy()
                            };
                            self.writer.identifier(&name);
                            self.visualize_function_body(method.function);
                        }
                    }
                }
                self.writer.newline();
                self.writer.keyword("end");
                self.writer.newline();
            }
        } else {
            LUAU_ASSERT!(false);
        }

        if program.has_semicolon {
            self.advance_before(program.base.location.end, 1);
            self.writer.symbol(";");
        }
    }
}
