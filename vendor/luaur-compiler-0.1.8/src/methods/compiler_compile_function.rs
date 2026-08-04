use crate::enums::type_compiler::Type as LoopJumpType;
use crate::functions::model_cost_cost_model::model_cost_ast_node_ast_local_usize_dense_hash_map_ast_expr_call_i32_dense_hash_map_ast_expr_constant;
use crate::records::compiler::Compiler;
use crate::records::return_visitor::ReturnVisitor;
use luaur_ast::records::ast_expr_function::AstExprFunction;
use luaur_ast::records::ast_node::AstNode;
use luaur_ast::records::ast_stat::AstStat;
use luaur_common::enums::luau_bytecode_type::LBC_TYPE_ANY;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;
use luaur_common::macros::luau_timetrace_argument::LUAU_TIMETRACE_ARGUMENT;
use luaur_common::macros::luau_timetrace_scope::LUAU_TIMETRACE_SCOPE;

impl Compiler {
    pub fn compile_function(&mut self, func: *mut AstExprFunction, protoflags: &mut u8) -> u32 {
        LUAU_TIMETRACE_SCOPE!("Compiler::compileFunction", "Compiler");
        unsafe {
            let func_ref = &*func;
            if !func_ref.debugname.value.is_null() {
                LUAU_TIMETRACE_ARGUMENT!("name", func_ref.debugname.value);
            }

            LUAU_ASSERT!(!self.functions.contains(&func));
            LUAU_ASSERT!(
                self.reg_top == 0
                    && self.stack_size == 0
                    && self.local_stack.is_empty()
                    && self.upvals.is_empty()
            );
            if luaur_common::FFlag::LuauExportValueSyntax.get() {
                self.current_function = func;
            }

            let mut rs = self.reg_scope_compiler();
            let self_ = if !func_ref.self_.is_null() { 1 } else { 0 };
            let fid = (*self.bytecode)
                .begin_function((self_ + func_ref.args.size) as u8, func_ref.vararg);

            self.set_debug_line_ast_node(func as *mut AstNode);

            if func_ref.vararg {
                (*self.bytecode).emit_abc(
                    LuauOpcode::LOP_PREPVARARGS,
                    (self_ + func_ref.args.size) as u8,
                    0,
                    0,
                );
            }

            let args = self.alloc_reg(func as *mut AstNode, (self_ + func_ref.args.size) as u32);
            if !func_ref.self_.is_null() {
                self.push_local(func_ref.self_, args, 0);
            }
            for i in 0..func_ref.args.size {
                self.push_local(*func_ref.args.data.add(i), args + self_ as u8 + i as u8, 0);
            }

            self.arg_count = self.local_stack.len();
            let stat = func_ref.body;
            let mut terminates_early = false;
            self.current_function = func;

            for i in 0..(*stat).body.size {
                let body_stat = *(*stat).body.data.add(i);
                self.compile_stat(body_stat);
                if self.always_terminates(body_stat) {
                    terminates_early = true;
                    break;
                }
            }

            if luaur_common::FFlag::LuauExportValueSyntax.get() {
                self.set_debug_line_end(stat as *mut AstNode);
                if (!self.exported_locals.is_empty() || !self.exported_classes.is_empty())
                    && self.at_top_level()
                {
                    self.compile_export_table();
                } else if !terminates_early {
                    self.close_locals(0);
                    (*self.bytecode).emit_abc(LuauOpcode::LOP_RETURN, 0, 1, 0);
                }
            } else if !terminates_early {
                self.set_debug_line_end(stat as *mut AstNode);
                self.close_locals(0);
                (*self.bytecode).emit_abc(LuauOpcode::LOP_RETURN, 0, 1, 0);
            }

            if self.options.optimization_level >= 1 && self.options.debug_level >= 2 {
                self.gather_const_upvals(func);
            }

            (*self.bytecode)
                .set_debug_function_line_defined(func_ref.base.base.location.begin.line as i32 + 1);

            if self.options.debug_level >= 1 && !func_ref.debugname.value.is_null() {
                (*self.bytecode).set_debug_function_name(
                    crate::functions::sref_compiler::sref_ast_name(func_ref.debugname),
                );
            }

            if self.options.debug_level >= 2 {
                for &l in &self.upvals {
                    (*self.bytecode).push_debug_upval(
                        crate::functions::sref_compiler::sref_ast_name((*l).name),
                    );
                }
            }

            if self.options.type_info_level >= 1 {
                for &l in &self.upvals {
                    let ty = self.local_types.find(&l).copied().unwrap_or(LBC_TYPE_ANY);
                    (*self.bytecode).push_upval_type_info(ty);
                }
            }

            if self.options.optimization_level >= 1 {
                (*self.bytecode).fold_jumps();
            }
            (*self.bytecode).expand_jumps();
            self.pop_locals(0);

            if (*self.bytecode).get_instruction_count() > 1000000 {
                crate::records::compile_error::CompileError::raise(
                    &func_ref.base.base.location,
                    format_args!("Exceeded function instruction limit; split the function into parts to compile"),
                );
            }

            if let Some(func_type) = self.function_types.find(&func) {
                (*self.bytecode).set_function_type_info(func_type.clone());
            }

            if func_ref.function_depth == 0 && !self.has_loops {
                *protoflags |= 2; // LPF_NATIVE_COLD = 1 << 1 (was wrongly 1)
            }
            if func_ref.has_native_attribute() {
                *protoflags |= 4; // LPF_NATIVE_FUNCTION = 1 << 2 (was wrongly 2)
            }

            // C: `isInlinable = !vararg && !getfenvUsed && !setfenvUsed;
            //     if (FFlag::LuauEmitCallFeedback && isInlinable && upvals.empty()) flags |= LPF_INLINABLE;`
            // This whole block was missing, so the Rust compiler never marked functions inlinable.
            let is_inlinable = !func_ref.vararg && !self.getfenv_used && !self.setfenv_used;
            if luaur_common::FFlag::LuauEmitCallFeedback.get()
                && is_inlinable
                && self.upvals.len() == 0
            {
                *protoflags |= 8; // LPF_INLINABLE = 1 << 3
            }

            (*self.bytecode).end_function(
                self.stack_size as u8,
                self.upvals.len() as u8,
                *protoflags,
            );

            {
                let f = self.functions.get_or_insert(func);
                f.id = fid;
                f.upvals = self.upvals.clone();
            }

            if self.options.optimization_level >= 2
                && !func_ref.vararg
                && func_ref.self_.is_null()
                && !self.getfenv_used
                && !self.setfenv_used
            {
                let cost_model = model_cost_ast_node_ast_local_usize_dense_hash_map_ast_expr_call_i32_dense_hash_map_ast_expr_constant(
                    func_ref.body as *mut AstNode,
                    func_ref.args.data as *const _,
                    func_ref.args.size,
                    &*self.builtins_fold,
                    &self.constants,
                );
                let returns_one = if self.always_terminates(func_ref.body as *mut AstStat) {
                    let mut rv = self.return_visitor_return_visitor();
                    luaur_ast::visit::ast_stat_block_visit(&*func_ref.body, &mut rv);
                    Some(rv.returns_one)
                } else {
                    None
                };

                let f = self.functions.get_or_insert(func);
                // C++: `f.canInline = !(DebugLuauNoInline && func->hasAttribute(DebugNoinline))`.
                f.can_inline = !(luaur_common::FFlag::DebugLuauNoInline.get()
                    && !func_ref
                        .get_attribute(luaur_ast::records::ast_attr::AstAttrType::DebugNoinline)
                        .is_null());
                f.stack_size = self.stack_size as u32;
                f.cost_model = cost_model;
                if let Some(returns_one) = returns_one {
                    f.returns_one = returns_one;
                }
            }

            self.upvals.clear();
            self.stack_size = 0;
            self.arg_count = 0;
            self.has_loops = false;
            self.current_function = core::ptr::null_mut();
            fid
        }
    }
}
