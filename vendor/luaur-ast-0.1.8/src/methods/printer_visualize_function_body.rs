use crate::records::ast_expr_function::AstExprFunction;
use crate::records::ast_generic_type::AstGenericType;
use crate::records::ast_generic_type_pack::AstGenericTypePack;
use crate::records::ast_local::AstLocal;
use crate::records::ast_stat_block::AstStatBlock;
use crate::records::comma_separator_inserter::CommaSeparatorInserter;
use crate::records::cst_expr_function::CstExprFunction;
use crate::records::cst_generic_type_pack::CstGenericTypePack;
use crate::records::printer::Printer;

pub trait IntoAstExprFunctionMut {
    unsafe fn into_ast_expr_function_mut(self) -> *mut AstExprFunction;
}

impl IntoAstExprFunctionMut for *mut AstExprFunction {
    unsafe fn into_ast_expr_function_mut(self) -> *mut AstExprFunction {
        self
    }
}

impl IntoAstExprFunctionMut for &mut AstExprFunction {
    unsafe fn into_ast_expr_function_mut(self) -> *mut AstExprFunction {
        self
    }
}

impl<'a> Printer<'a> {
    pub fn visualize_function_body<F: IntoAstExprFunctionMut>(&mut self, func: F) {
        let func = unsafe { &mut *func.into_ast_expr_function_mut() };
        let cst_node =
            self.lookup_cst_node_impl::<CstExprFunction>(func as *mut AstExprFunction as *mut _);

        if func.generics.size > 0 || func.generic_packs.size > 0 {
            let comma_position = if cst_node.is_null() {
                core::ptr::null()
            } else {
                unsafe { (*cst_node).generics_comma_positions.data }
            };

            let mut comma = CommaSeparatorInserter::new(self.writer, core::ptr::null());
            comma.comma_position = comma_position;

            if !cst_node.is_null() {
                let open_pos = unsafe { &(*cst_node).open_generics_position };
                self.maybe_advance_and_write(open_pos, "<", false);
            } else {
                self.writer.symbol("<");
            }

            for i in 0..func.generics.size {
                comma.call(self.writer);

                let generic_ty = unsafe { *func.generics.data.add(i) };
                unsafe {
                    self.writer.advance(&(*generic_ty).base.location.begin);
                    let name_val = (*generic_ty).name.value;
                    let name_str = core::ffi::CStr::from_ptr(name_val).to_string_lossy();
                    self.writer.identifier(&name_str);
                }
            }

            for i in 0..func.generic_packs.size {
                comma.call(self.writer);

                let pack = unsafe { *func.generic_packs.data.add(i) };
                unsafe {
                    self.writer.advance(&(*pack).base.location.begin);
                    let name_val = (*pack).name.value;
                    let name_str = core::ffi::CStr::from_ptr(name_val).to_string_lossy();
                    self.writer.identifier(&name_str);

                    let generic_type_pack_cst_node =
                        self.lookup_cst_node_impl::<CstGenericTypePack>(pack as *mut _);
                    if !generic_type_pack_cst_node.is_null() {
                        let ellipsis_pos = &(*generic_type_pack_cst_node).ellipsis_position;
                        self.advance(ellipsis_pos);
                    }

                    self.writer.symbol("...");
                }
            }

            if !cst_node.is_null() {
                let close_pos = unsafe { &(*cst_node).close_generics_position };
                self.maybe_advance_and_write(close_pos, ">", false);
            } else {
                self.writer.symbol(">");
            }
        }

        if let Some(arg_location) = func.arg_location.as_ref() {
            self.advance(&arg_location.begin);
        }
        self.writer.symbol("(");

        let args_comma_pos = if cst_node.is_null() {
            core::ptr::null()
        } else {
            unsafe { (*cst_node).args_comma_positions.data }
        };

        let mut comma = CommaSeparatorInserter::new(self.writer, core::ptr::null());
        comma.comma_position = args_comma_pos;

        for i in 0..func.args.size {
            let local = unsafe { *func.args.data.add(i) };
            comma.call(self.writer);

            unsafe {
                self.advance(&(*local).location.begin);
                let name_val = (*local).name.value;
                let name_str = core::ffi::CStr::from_ptr(name_val).to_string_lossy();
                self.writer.identifier(&name_str);

                if self.write_types && !(*local).annotation.is_null() {
                    if !cst_node.is_null() {
                        let colon_pos =
                            unsafe { (*cst_node).args_annotation_colon_positions.data.add(i) };
                        self.maybe_advance_and_write(&*colon_pos, ":", false);
                    } else {
                        self.writer.symbol(":");
                    }

                    self.visualize_type_annotation(&mut *(*local).annotation);
                }
            }
        }

        if func.vararg {
            comma.call(self.writer);

            self.advance(&func.vararg_location.begin);
            self.writer.symbol("...");

            if self.write_types && !func.vararg_annotation.is_null() {
                if !cst_node.is_null() {
                    unsafe {
                        self.maybe_advance_and_write(
                            &(*cst_node).vararg_annotation_colon_position,
                            ":",
                            false,
                        );
                    }
                } else {
                    self.writer.symbol(":");
                }

                unsafe {
                    self.visualize_type_pack_annotation(
                        &mut *func.vararg_annotation,
                        true,
                        false,
                        false,
                    );
                }
            }
        }

        if let Some(arg_location) = func.arg_location.as_ref() {
            self.advance_before(arg_location.end, 1);
        }
        self.writer.symbol(")");

        if self.write_types && !func.return_annotation.is_null() {
            if !cst_node.is_null() {
                unsafe {
                    self.maybe_advance_and_write(
                        &(*cst_node).return_specifier_position,
                        ":",
                        false,
                    );
                }
            } else {
                self.writer.symbol(":");
            }

            if cst_node.is_null() {
                self.writer.space();
            }

            unsafe {
                self.visualize_type_pack_annotation(
                    &mut *func.return_annotation,
                    false,
                    false,
                    true,
                );
            }
        }

        unsafe {
            self.visualize_block_ast_stat_block(&mut *func.body);
            self.advance(&(*func.body).base.base.location.end);
        }
        self.writer.keyword("end");
    }
}
