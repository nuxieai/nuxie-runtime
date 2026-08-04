use crate::enums::type_constant_folding::Type;
use crate::functions::sref_compiler::sref_ast_name;
use crate::functions::sref_compiler_alt_c::sref_ast_array_c_char;
use crate::records::compile_error::CompileError;
use crate::records::compiler::Compiler;
use crate::records::constant::Constant;
use crate::records::reg_scope::RegScope;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_constant_string::AstExprConstantString;
use luaur_ast::records::ast_expr_table::AstExprTable;
use luaur_ast::records::ast_expr_varargs::AstExprVarargs;
use luaur_ast::rtti;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;
use luaur_common::records::insertion_ordered_map::InsertionOrderedMap;

impl Compiler {
    pub fn compile_expr_table(&mut self, expr: *mut AstExprTable, target: u8, target_temp: bool) {
        unsafe {
            let expr_ref = &*expr;
            if expr_ref.items.size == 0 {
                // C++ `TableShape shape = tableShapes[expr];` — `operator[]`
                // default-constructs a zero (hash_size=0, array_size=0) shape when
                // the table had no predicted shape (an empty `{}` with no later
                // field writes). The model translated this as panic-on-miss.
                let shape = self.table_shapes.find(&expr).copied().unwrap_or_default();
                (*self.bytecode)
                    .add_debug_remark(format_args!("allocation: table hash {}", shape.hash_size));
                (*self.bytecode).emit_abc(
                    LuauOpcode::LOP_NEWTABLE,
                    target,
                    Self::encode_hash_size(shape.hash_size),
                    0,
                );
                (*self.bytecode).emit_aux(shape.array_size);
                return;
            }

            let mut array_size = 0;
            let mut hash_size = 0;
            let mut record_size = 0;
            for item in expr_ref.items.iter() {
                array_size +=
                    (item.kind == luaur_ast::records::ast_expr_table::ItemKind::List) as u32;
                hash_size +=
                    (item.kind != luaur_ast::records::ast_expr_table::ItemKind::List) as u32;
                record_size +=
                    (item.kind == luaur_ast::records::ast_expr_table::ItemKind::Record) as u32;
            }

            let mut index_size = 0;
            if array_size == 0 && hash_size > 0 {
                for item in expr_ref.items.iter() {
                    LUAU_ASSERT!(!item.key.is_null());
                    if let Some(ckey) = self.constants.find(&item.key) {
                        if ckey.r#type == Type::Type_Number {
                            let val = unsafe { ckey.data.value_number };
                            if val == (index_size + 1) as f64 {
                                index_size += 1;
                            }
                        }
                    }
                }
                if hash_size == record_size + index_size {
                    hash_size = record_size;
                } else {
                    index_size = 0;
                }
            }

            let encoded_hash_size = Self::encode_hash_size(hash_size);
            let _rs = self.reg_scope_compiler();
            // Optimization: if target is a temp register, compute the result directly into it.
            let reg = if target_temp {
                target
            } else {
                self.alloc_reg(expr as *mut _, 1)
            };

            use luaur_ast::records::ast_expr_table::ItemKind;
            type BcTableShape = luaur_bytecode::records::table_shape::TableShape;

            // flattening map for record fields (template-table path).
            let mut last_key_val: InsertionOrderedMap<i32, i32> = InsertionOrderedMap::new();

            // Optimization: when all items are record fields, use template tables (DUPTABLE).
            if array_size == 0
                && index_size == 0
                && hash_size == record_size
                && record_size >= 1
                && record_size <= BcTableShape::kMaxLength
            {
                let mut shape = BcTableShape::default();

                if luaur_common::FFlag::LuauCompileDuptableConstantPack2.get() {
                    for i in 0..expr_ref.items.size {
                        let item = &*expr_ref.items.data.add(i as usize);
                        LUAU_ASSERT!(item.kind == ItemKind::Record);
                        let ckey = rtti::ast_node_as::<AstExprConstantString>(item.key as *mut _);
                        LUAU_ASSERT!(!ckey.is_null());
                        let key_cid = (*self.bytecode)
                            .add_constant_string(sref_ast_array_c_char((*ckey).value));
                        if key_cid < 0 {
                            CompileError::raise(
                                &(*ckey).base.base.location,
                                format_args!(
                                    "Exceeded constant limit; simplify the code to compile"
                                ),
                            );
                        }
                        let value_cid = self.get_constant_index(item.value);
                        if let Some(existing) = last_key_val.get(&key_cid) {
                            if *existing == -1 {
                                continue;
                            }
                        }
                        // C++ `lastKeyVal[keyCid] = valueCid` — operator[] OVERWRITES an
                        // existing entry. InsertionOrderedMap::insert is a no-op when the key
                        // exists, so a duplicate key whose value later becomes -1 (non-constant,
                        // e.g. a closure) failed to downgrade and the table wrongly packed its
                        // constants. get_or_default is the operator[] equivalent.
                        *last_key_val.get_or_default(key_cid) = value_cid;
                    }

                    for (key_cid, value_cid) in last_key_val.iter() {
                        LUAU_ASSERT!(shape.length < BcTableShape::kMaxLength);
                        let idx = shape.length as usize;
                        shape.keys[idx] = *key_cid;
                        shape.constants[idx] = *value_cid;
                        if *value_cid >= 0 {
                            shape.hasConstants = true;
                        }
                        shape.length += 1;
                    }
                } else {
                    for i in 0..expr_ref.items.size {
                        let item = &*expr_ref.items.data.add(i as usize);
                        LUAU_ASSERT!(item.kind == ItemKind::Record);
                        let ckey = rtti::ast_node_as::<AstExprConstantString>(item.key as *mut _);
                        LUAU_ASSERT!(!ckey.is_null());
                        let cid = (*self.bytecode)
                            .add_constant_string(sref_ast_array_c_char((*ckey).value));
                        if cid < 0 {
                            CompileError::raise(
                                &(*ckey).base.base.location,
                                format_args!(
                                    "Exceeded constant limit; simplify the code to compile"
                                ),
                            );
                        }
                        LUAU_ASSERT!(shape.length < BcTableShape::kMaxLength);
                        shape.keys[shape.length as usize] = cid;
                        shape.length += 1;
                    }
                }

                let tid = (*self.bytecode).add_constant_table(&shape);
                if tid < 0 {
                    CompileError::raise(
                        &expr_ref.base.base.location,
                        format_args!("Exceeded constant limit; simplify the code to compile"),
                    );
                }
                (*self.bytecode)
                    .add_debug_remark(format_args!("allocation: table template {}", hash_size));

                if tid < 32768 {
                    (*self.bytecode).emit_ad(LuauOpcode::LOP_DUPTABLE, reg, tid as i16);
                } else {
                    // must disable duptable constant optimization here, as we default back to new table
                    if luaur_common::FFlag::LuauCompileDuptableConstantPack2.get() {
                        shape.hasConstants = false;
                        last_key_val.clear();
                    }
                    (*self.bytecode).emit_abc(LuauOpcode::LOP_NEWTABLE, reg, encoded_hash_size, 0);
                    (*self.bytecode).emit_aux(0);
                }
            } else {
                // Optimization: when the last element is `...`, let SETLIST allocate storage.
                let last: *const luaur_ast::records::ast_expr_table::Item =
                    if expr_ref.items.size > 0 {
                        &*expr_ref.items.data.add((expr_ref.items.size - 1) as usize)
                    } else {
                        core::ptr::null()
                    };
                let trailing_varargs = !last.is_null()
                    && (*last).kind == ItemKind::List
                    && !rtti::ast_node_as::<AstExprVarargs>((*last).value as *mut _).is_null();
                LUAU_ASSERT!(!trailing_varargs || array_size > 0);

                let array_allocation = array_size - (trailing_varargs as u32) + index_size;

                if hash_size == 0 {
                    (*self.bytecode).add_debug_remark(format_args!(
                        "allocation: table array {}",
                        array_allocation
                    ));
                } else if array_allocation == 0 {
                    (*self.bytecode)
                        .add_debug_remark(format_args!("allocation: table hash {}", hash_size));
                } else {
                    (*self.bytecode).add_debug_remark(format_args!(
                        "allocation: table hash {} array {}",
                        hash_size, array_allocation
                    ));
                }

                (*self.bytecode).emit_abc(LuauOpcode::LOP_NEWTABLE, reg, encoded_hash_size, 0);
                (*self.bytecode).emit_aux(array_allocation);
            }

            let array_chunk_size = core::cmp::min(16u32, array_size);
            let array_chunk_reg = self.alloc_reg(expr as *mut _, array_chunk_size);
            let mut array_chunk_current: u32 = 0;
            let mut array_index: u32 = 1;
            let mut mult_ret = false;

            for i in 0..expr_ref.items.size {
                let item = &*expr_ref.items.data.add(i as usize);
                let key = item.key;
                let value = item.value;

                if luaur_common::FFlag::LuauCompileDuptableConstantPack2.get()
                    && last_key_val.size() > 0
                    && !key.is_null()
                    && !rtti::ast_node_as::<AstExprConstantString>(key as *mut _).is_null()
                {
                    let ckey = rtti::ast_node_as::<AstExprConstantString>(item.key as *mut _);
                    LUAU_ASSERT!(!ckey.is_null());
                    let key_cid =
                        (*self.bytecode).add_constant_string(sref_ast_array_c_char((*ckey).value));
                    if let Some(value_cid) = last_key_val.get(&key_cid) {
                        // do not generate assignments for constants
                        if *value_cid >= 0 {
                            continue;
                        }
                    }
                }

                // some key/value pairs don't require compiling the expressions, set up line info here
                self.set_debug_line_ast_node(value as *mut luaur_ast::records::ast_node::AstNode);

                if self.options.coverage_level >= 2 {
                    (*self.bytecode).emit_abc(LuauOpcode::LOP_COVERAGE, 0, 0, 0);
                }

                // flush array chunk on overflow or before hash keys to maintain insertion order
                if array_chunk_current > 0
                    && (!key.is_null() || array_chunk_current == array_chunk_size)
                {
                    (*self.bytecode).emit_abc(
                        LuauOpcode::LOP_SETLIST,
                        reg,
                        array_chunk_reg,
                        (array_chunk_current + 1) as u8,
                    );
                    (*self.bytecode).emit_aux(array_index);
                    array_index += array_chunk_current;
                    array_chunk_current = 0;
                }

                if !key.is_null() {
                    // items with a key are set via SETTABLE/SETTABLEKS/SETTABLEN
                    let mut rsi = self.reg_scope_compiler();
                    let lv = self.compile_l_value_index(reg, key, &mut rsi);
                    let rv = self.compile_expr_auto(value, &mut rsi);
                    self.compile_assign(&lv, rv, core::ptr::null_mut());
                } else {
                    // items without a key are set via SETLIST to init large arrays quickly
                    let temp = (array_chunk_reg as u32 + array_chunk_current) as u8;
                    if i + 1 == expr_ref.items.size {
                        mult_ret = self.compile_expr_temp_mult_ret(value, temp);
                    } else {
                        self.compile_expr_temp_top(value, temp);
                    }
                    array_chunk_current += 1;
                }
            }

            // flush last array chunk; needs multret handling if the last expression was multret
            if array_chunk_current != 0 {
                (*self.bytecode).emit_abc(
                    LuauOpcode::LOP_SETLIST,
                    reg,
                    array_chunk_reg,
                    if mult_ret {
                        0
                    } else {
                        (array_chunk_current + 1) as u8
                    },
                );
                (*self.bytecode).emit_aux(array_index);
            }

            if target != reg {
                (*self.bytecode).emit_abc(LuauOpcode::LOP_MOVE, target, reg, 0);
            }
        }
    }
}
