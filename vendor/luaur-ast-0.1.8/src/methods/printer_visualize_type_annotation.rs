use crate::enums::ast_table_access::AstTableAccess;
use crate::records::ast_array::AstArray;
use crate::records::ast_node::AstNode;
use crate::records::ast_type::AstType;
use crate::records::ast_type_error::AstTypeError;
use crate::records::ast_type_function::AstTypeFunction;
use crate::records::ast_type_group::AstTypeGroup;
use crate::records::ast_type_intersection::AstTypeIntersection;
use crate::records::ast_type_optional::AstTypeOptional;
use crate::records::ast_type_reference::AstTypeReference;
use crate::records::ast_type_singleton_bool::AstTypeSingletonBool;
use crate::records::ast_type_singleton_string::AstTypeSingletonString;
use crate::records::ast_type_table::AstTypeTable;
use crate::records::ast_type_typeof::AstTypeTypeof;
use crate::records::ast_type_union::AstTypeUnion;
use crate::records::comma_separator_inserter::CommaSeparatorInserter;
use crate::records::cst_expr_table::CstExprTable;
use crate::records::cst_generic_type_pack::CstGenericTypePack;
use crate::records::cst_type_function::CstTypeFunction;
use crate::records::cst_type_group::CstTypeGroup;
use crate::records::cst_type_intersection::CstTypeIntersection;
use crate::records::cst_type_reference::CstTypeReference;
use crate::records::cst_type_singleton_string::CstTypeSingletonString;
use crate::records::cst_type_table::CstTypeTable;
use crate::records::cst_type_typeof::CstTypeTypeof;
use crate::records::cst_type_union::CstTypeUnion;
use crate::records::position::Position;
use crate::records::printer::Printer;
use crate::rtti::{ast_node_as, ast_node_is, CstNodeClass};
use luaur_common::macros::luau_assert::LUAU_ASSERT;

pub trait IntoAstTypeMut {
    unsafe fn into_ast_type_mut(self) -> *mut AstType;
}

impl IntoAstTypeMut for *mut AstType {
    unsafe fn into_ast_type_mut(self) -> *mut AstType {
        self
    }
}

impl IntoAstTypeMut for &*mut AstType {
    unsafe fn into_ast_type_mut(self) -> *mut AstType {
        *self
    }
}

impl IntoAstTypeMut for &mut AstType {
    unsafe fn into_ast_type_mut(self) -> *mut AstType {
        self
    }
}

impl<'a> Printer<'a> {
    pub fn visualize_type_annotation<T: IntoAstTypeMut>(&mut self, type_annotation: T) {
        let type_annotation = unsafe { &mut *type_annotation.into_ast_type_mut() };
        self.advance(&type_annotation.base.location.begin);

        if let Some(a) = unsafe {
            ast_node_as::<AstTypeReference>(type_annotation as *mut AstType as *mut AstNode)
                .as_mut()
        } {
            let cst_node = self.lookup_cst_node_impl::<CstTypeReference>(
                a as *mut AstTypeReference as *mut AstNode,
            );

            if let Some(prefix) = a.prefix {
                let name_str = unsafe { core::ffi::CStr::from_ptr(prefix.value).to_string_lossy() };
                self.writer.write(&name_str);
                if !cst_node.is_null() {
                    self.advance(unsafe { &(*cst_node).prefix_point_position });
                }
                self.writer.symbol(".");
            }

            self.advance(&a.name_location.begin);
            let name_str = unsafe { core::ffi::CStr::from_ptr(a.name.value).to_string_lossy() };
            self.writer.write(&name_str);

            if a.parameters.size > 0 || a.has_parameter_list {
                let comma_pos_ptr = if !cst_node.is_null() {
                    unsafe { (*cst_node).parameters_comma_positions.begin() }
                } else {
                    core::ptr::null()
                };

                if !cst_node.is_null() {
                    self.advance(unsafe { &(*cst_node).open_parameters_position });
                }
                self.writer.symbol("<");

                let mut comma = CommaSeparatorInserter::new(self.writer, comma_pos_ptr);
                for i in 0..a.parameters.size {
                    comma.operator_call(self.writer);

                    let o = unsafe { &mut *a.parameters.data.add(i) };
                    if !o.r#type.is_null() {
                        self.visualize_type_annotation(unsafe { &mut *o.r#type });
                    } else {
                        self.visualize_type_pack_annotation(
                            unsafe { &mut *o.type_pack },
                            false,
                            false,
                            false,
                        );
                    }
                }

                if !cst_node.is_null() {
                    let pos = unsafe { (*cst_node).close_parameters_position };
                    self.maybe_advance_and_write(&pos, ">", false);
                } else {
                    self.writer.symbol(">");
                }
            }
        } else if let Some(a) = unsafe {
            ast_node_as::<AstTypeFunction>(type_annotation as *mut AstType as *mut AstNode).as_mut()
        } {
            let cst_node = self
                .lookup_cst_node_impl::<CstTypeFunction>(a as *mut AstTypeFunction as *mut AstNode);

            if a.generics.size > 0 || a.generic_packs.size > 0 {
                let comma_pos_ptr = if !cst_node.is_null() {
                    unsafe { (*cst_node).generics_comma_positions.begin() }
                } else {
                    core::ptr::null()
                };

                if !cst_node.is_null() {
                    self.advance(unsafe { &(*cst_node).open_generics_position });
                }
                self.writer.symbol("<");

                let mut comma = CommaSeparatorInserter::new(self.writer, comma_pos_ptr);
                for i in 0..a.generics.size {
                    comma.operator_call(self.writer);

                    let o = unsafe { &mut **a.generics.data.add(i) };
                    self.writer.advance(&o.base.location.begin);
                    let name_str =
                        unsafe { core::ffi::CStr::from_ptr(o.name.value).to_string_lossy() };
                    self.writer.identifier(&name_str);
                }

                for i in 0..a.generic_packs.size {
                    comma.operator_call(self.writer);

                    let o = unsafe { &mut **a.generic_packs.data.add(i) };
                    self.writer.advance(&o.base.location.begin);
                    let name_str =
                        unsafe { core::ffi::CStr::from_ptr(o.name.value).to_string_lossy() };
                    self.writer.identifier(&name_str);

                    let generic_type_pack_cst_node = self
                        .lookup_cst_node_impl::<CstGenericTypePack>(o as *mut _ as *mut AstNode);
                    if !generic_type_pack_cst_node.is_null() {
                        self.advance(unsafe { &(*generic_type_pack_cst_node).ellipsis_position });
                    }
                    self.writer.symbol("...");
                }

                if !cst_node.is_null() {
                    let pos = unsafe { (*cst_node).close_generics_position };
                    self.maybe_advance_and_write(&pos, ">", false);
                } else {
                    self.writer.symbol(">");
                }
            }

            let open_args = if !cst_node.is_null() {
                unsafe { (*cst_node).open_args_position }
            } else {
                Position::missing()
            };
            let close_args = if !cst_node.is_null() {
                unsafe { (*cst_node).close_args_position }
            } else {
                Position::missing()
            };
            let comma_pos = if !cst_node.is_null() {
                unsafe { &(*cst_node).arguments_comma_positions }
            } else {
                &AstArray::default()
            };
            let colon_pos = if !cst_node.is_null() {
                unsafe { &(*cst_node).argument_name_colon_positions }
            } else {
                &AstArray::default()
            };

            self.visualize_named_type_list(
                &a.arg_types,
                cst_node.is_null(),
                open_args,
                close_args,
                comma_pos,
                &a.arg_names,
                colon_pos,
            );

            if !cst_node.is_null() {
                self.advance(unsafe { &(*cst_node).return_arrow_position });
            }
            self.writer.symbol("->");

            self.visualize_type_pack_annotation(
                unsafe { &mut *a.return_types },
                false,
                cst_node.is_null(),
                false,
            );
        } else if let Some(a) = unsafe {
            ast_node_as::<AstTypeTable>(type_annotation as *mut AstType as *mut AstNode).as_mut()
        } {
            let index_type = if !a.indexer.is_null() {
                unsafe { ast_node_as::<AstTypeReference>((*a.indexer).index_type as *mut AstNode) }
            } else {
                core::ptr::null_mut()
            };

            self.writer.symbol("{");

            let cst_node =
                self.lookup_cst_node_impl::<CstTypeTable>(a as *mut AstTypeTable as *mut AstNode);
            if !cst_node.is_null() {
                if unsafe { (*cst_node).is_array } {
                    LUAU_ASSERT!(
                        a.props.size == 0
                            && !index_type.is_null()
                            && unsafe {
                                core::ffi::CStr::from_ptr((*index_type).name.value)
                                    .to_string_lossy()
                                    == "number"
                            }
                    );
                    unsafe {
                        if let Some(loc) = (*a.indexer).access_location {
                            self.advance(&loc.begin);
                            self.writer
                                .keyword(if (*a.indexer).access == AstTableAccess::Read {
                                    "read"
                                } else {
                                    "write"
                                });
                        }
                        self.visualize_type_annotation(&mut *(*a.indexer).result_type);
                    }
                } else {
                    let mut prop_idx = 0;
                    let items = unsafe { (*cst_node).items.as_slice() };

                    for item in items {
                        if item.kind
                            == crate::records::cst_type_table::CstTypeTable_Item_Kind::Indexer
                        {
                            LUAU_ASSERT!(!a.indexer.is_null());
                            unsafe {
                                if let Some(loc) = (*a.indexer).access_location {
                                    self.advance(&loc.begin);
                                    self.writer.keyword(
                                        if (*a.indexer).access == AstTableAccess::Read {
                                            "read"
                                        } else {
                                            "write"
                                        },
                                    );
                                }
                                self.advance(&item.indexer_open_position);
                                self.writer.symbol("[");
                                self.visualize_type_annotation(&mut *(*a.indexer).index_type);
                                self.maybe_advance_and_write(
                                    &item.indexer_close_position,
                                    "]",
                                    false,
                                );
                                self.maybe_advance_and_write(&item.colon_position, ":", false);
                                self.visualize_type_annotation(&mut *(*a.indexer).result_type);
                            }
                        } else {
                            let prop = unsafe { &mut *a.props.data.add(prop_idx) };
                            if let Some(loc) = prop.access_location {
                                self.advance(&loc.begin);
                                self.writer.keyword(if prop.access == AstTableAccess::Read {
                                    "read"
                                } else {
                                    "write"
                                });
                            }

                            if item.kind == crate::records::cst_type_table::CstTypeTable_Item_Kind::StringProperty {
                                if item.indexer_open_position.has_value() {
                                    self.maybe_advance_and_write(&item.indexer_open_position, "[", false);
                                }
                                self.advance(&item.string_position);
                                unsafe {
                                    let s_ptr = (*item.string_info).source_string.data as *const u8;
                                    let s_len = (*item.string_info).source_string.size;
                                    let s_str = core::str::from_utf8_unchecked(core::slice::from_raw_parts(s_ptr, s_len));
                                    self.writer.source_string(s_str, (*item.string_info).quote_style, (*item.string_info).block_depth);
                                }
                                if item.indexer_close_position.has_value() {
                                    self.maybe_advance_and_write(&item.indexer_close_position, "]", false);
                                }
                            } else {
                                self.advance(&prop.location.begin);
                                let name_str = unsafe { core::ffi::CStr::from_ptr(prop.name.value).to_string_lossy() };
                                self.writer.identifier(&name_str);
                            }

                            self.maybe_advance_and_write(&item.colon_position, ":", false);
                            self.visualize_type_annotation(unsafe { &mut *prop.r#type });
                            prop_idx += 1;
                        }

                        if item.separator
                            != crate::records::cst_expr_table::CstExprTableSeparator::Missing
                        {
                            LUAU_ASSERT!(item.separator_position.has_value());
                            self.maybe_advance_and_write(
                                &item.separator_position,
                                if item.separator
                                    == crate::records::cst_expr_table::CstExprTableSeparator::Comma
                                {
                                    ","
                                } else {
                                    ";"
                                },
                                true,
                            );
                        }
                    }
                }
            } else {
                if a.props.size == 0
                    && !index_type.is_null()
                    && unsafe {
                        core::ffi::CStr::from_ptr((*index_type).name.value).to_string_lossy()
                            == "number"
                    }
                {
                    self.visualize_type_annotation(unsafe { &mut *(*a.indexer).result_type });
                } else {
                    let mut comma = CommaSeparatorInserter::new(self.writer, core::ptr::null());

                    for i in 0..a.props.size {
                        comma.operator_call(self.writer);

                        let prop = unsafe { &mut *a.props.data.add(i) };
                        self.advance(&prop.location.begin);
                        let name_str =
                            unsafe { core::ffi::CStr::from_ptr(prop.name.value).to_string_lossy() };
                        self.writer.identifier(&name_str);
                        if !prop.r#type.is_null() {
                            self.writer.symbol(":");
                            self.visualize_type_annotation(unsafe { &mut *prop.r#type });
                        }
                    }
                    if !a.indexer.is_null() {
                        comma.operator_call(self.writer);

                        self.writer.symbol("[");
                        self.visualize_type_annotation(unsafe { &mut *(*a.indexer).index_type });
                        self.writer.symbol("]");
                        self.writer.symbol(":");
                        self.visualize_type_annotation(unsafe { &mut *(*a.indexer).result_type });
                    }
                }
            }

            let mut end_pos = type_annotation.base.location.end;
            if end_pos.column > 0 {
                end_pos.column -= 1;
            }
            self.advance(&end_pos);
            self.writer.symbol("}");
        } else if let Some(a) = unsafe {
            ast_node_as::<AstTypeTypeof>(type_annotation as *mut AstType as *mut AstNode).as_mut()
        } {
            self.writer.keyword("typeof");
            let cst_node =
                self.lookup_cst_node_impl::<CstTypeTypeof>(a as *mut AstTypeTypeof as *mut AstNode);
            if !cst_node.is_null() {
                self.maybe_advance_and_write(unsafe { &(*cst_node).open_position }, "(", false);
                self.visualize_ast_expr(unsafe { &mut *a.expr });
                self.maybe_advance_and_write(unsafe { &(*cst_node).close_position }, ")", false);
            } else {
                self.writer.symbol("(");
                self.visualize_ast_expr(unsafe { &mut *a.expr });
                self.writer.symbol(")");
            }
        } else if let Some(a) = unsafe {
            ast_node_as::<AstTypeUnion>(type_annotation as *mut AstType as *mut AstNode).as_mut()
        } {
            let cst_node =
                self.lookup_cst_node_impl::<CstTypeUnion>(a as *mut AstTypeUnion as *mut AstNode);

            if cst_node.is_null() && a.types.size == 2 {
                let mut l = unsafe { *a.types.data.add(0) };
                let mut r = unsafe { *a.types.data.add(1) };

                let lta = unsafe { ast_node_as::<AstTypeReference>(l as *mut AstNode) };
                if !lta.is_null()
                    && unsafe {
                        core::ffi::CStr::from_ptr((*lta).name.value).to_string_lossy() == "nil"
                    }
                    && !ast_node_is::<AstTypeOptional>(unsafe { &*(r as *mut AstNode) })
                {
                    core::mem::swap(&mut l, &mut r);
                }

                let rta = unsafe { ast_node_as::<AstTypeReference>(r as *mut AstNode) };
                if !rta.is_null()
                    && unsafe {
                        core::ffi::CStr::from_ptr((*rta).name.value).to_string_lossy() == "nil"
                    }
                {
                    let wrap = ast_node_is::<AstTypeIntersection>(unsafe { &*(l as *mut AstNode) })
                        || ast_node_is::<AstTypeFunction>(unsafe { &*(l as *mut AstNode) });
                    if wrap {
                        self.writer.symbol("(");
                    }
                    self.visualize_type_annotation(unsafe { &mut *l });
                    if wrap {
                        self.writer.symbol(")");
                    }
                    self.writer.symbol("?");
                    return;
                }
            }

            if !cst_node.is_null() {
                self.maybe_advance_and_write(unsafe { &(*cst_node).leading_position }, "|", false);
            }

            let mut separator_index = 0;
            for i in 0..a.types.size {
                let t = unsafe { &mut **a.types.data.add(i) };
                if let Some(optional) = unsafe {
                    ast_node_as::<AstTypeOptional>(t as *mut AstType as *mut AstNode).as_mut()
                } {
                    self.advance(&optional.base.base.location.begin);
                    self.writer.symbol("?");
                    continue;
                }

                if i > 0 {
                    if !cst_node.is_null() {
                        self.advance(unsafe {
                            &*(*cst_node).separator_positions.data.add(separator_index)
                        });
                        separator_index += 1;
                    } else {
                        self.writer.maybe_space(&t.base.location.begin, 2);
                    }
                    self.writer.symbol("|");
                }

                let wrap = cst_node.is_null()
                    && (ast_node_is::<AstTypeIntersection>(unsafe {
                        &*(t as *mut AstType as *mut AstNode)
                    }) || ast_node_is::<AstTypeFunction>(unsafe {
                        &*(t as *mut AstType as *mut AstNode)
                    }));
                if wrap {
                    self.writer.symbol("(");
                }
                self.visualize_type_annotation(t);
                if wrap {
                    self.writer.symbol(")");
                }
            }
        } else if let Some(a) = unsafe {
            ast_node_as::<AstTypeIntersection>(type_annotation as *mut AstType as *mut AstNode)
                .as_mut()
        } {
            let cst_node = self.lookup_cst_node_impl::<CstTypeIntersection>(
                a as *mut AstTypeIntersection as *mut AstNode,
            );

            if !cst_node.is_null() {
                self.maybe_advance_and_write(unsafe { &(*cst_node).leading_position }, "&", false);
            }

            for i in 0..a.types.size {
                let t = unsafe { &mut **a.types.data.add(i) };
                if i > 0 {
                    if !cst_node.is_null() {
                        self.advance(unsafe { &*(*cst_node).separator_positions.data.add(i - 1) });
                    } else {
                        self.writer.maybe_space(&t.base.location.begin, 2);
                    }
                    self.writer.symbol("&");
                }

                let wrap = cst_node.is_null()
                    && (ast_node_is::<AstTypeUnion>(unsafe {
                        &*(t as *mut AstType as *mut AstNode)
                    }) || ast_node_is::<AstTypeFunction>(unsafe {
                        &*(t as *mut AstType as *mut AstNode)
                    }));
                if wrap {
                    self.writer.symbol("(");
                }
                self.visualize_type_annotation(t);
                if wrap {
                    self.writer.symbol(")");
                }
            }
        } else if let Some(a) = unsafe {
            ast_node_as::<AstTypeGroup>(type_annotation as *mut AstType as *mut AstNode).as_mut()
        } {
            self.writer.symbol("(");
            self.visualize_type_annotation(unsafe { &mut *a.type_ });

            if luaur_common::FFlag::LuauCstTypeGroup.get() {
                let cst_node = self
                    .lookup_cst_node_impl::<CstTypeGroup>(a as *mut AstTypeGroup as *mut AstNode);
                if !cst_node.is_null() {
                    self.maybe_advance_and_write(
                        unsafe { &(*cst_node).close_position },
                        ")",
                        false,
                    );
                } else {
                    self.advance_before(type_annotation.base.location.end, 1);
                    self.writer.symbol(")");
                }
            } else {
                self.advance_before(type_annotation.base.location.end, 1);
                self.writer.symbol(")");
            }
        } else if let Some(a) = unsafe {
            ast_node_as::<AstTypeSingletonBool>(type_annotation as *mut AstType as *mut AstNode)
                .as_mut()
        } {
            self.writer.keyword(if a.value { "true" } else { "false" });
        } else if let Some(a) = unsafe {
            ast_node_as::<AstTypeSingletonString>(type_annotation as *mut AstType as *mut AstNode)
                .as_mut()
        } {
            let cst_node = self.lookup_cst_node_impl::<CstTypeSingletonString>(
                a as *mut AstTypeSingletonString as *mut AstNode,
            );
            if !cst_node.is_null() {
                unsafe {
                    let s_ptr = (*cst_node).source_string.data as *const u8;
                    let s_len = (*cst_node).source_string.size;
                    let s_str =
                        core::str::from_utf8_unchecked(core::slice::from_raw_parts(s_ptr, s_len));
                    self.writer.source_string(
                        s_str,
                        (*cst_node).quote_style,
                        (*cst_node).block_depth,
                    );
                }
            } else {
                let s_ptr = a.value.data as *const u8;
                let s_len = a.value.size;
                let s_str = unsafe {
                    core::str::from_utf8_unchecked(core::slice::from_raw_parts(s_ptr, s_len))
                };
                self.writer.write(s_str);
            }
        } else if ast_node_is::<AstTypeError>(unsafe {
            &*(type_annotation as *mut AstType as *mut AstNode)
        }) {
            self.writer.symbol("%error-type%");
        } else {
            LUAU_ASSERT!(false);
        }
    }
}
