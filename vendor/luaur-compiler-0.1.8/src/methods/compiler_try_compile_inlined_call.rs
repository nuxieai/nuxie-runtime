use crate::functions::compute_cost::compute_cost;
use crate::records::compiler::Compiler;
use luaur_ast::records::ast_expr_call::AstExprCall;
use luaur_ast::records::ast_expr_function::AstExprFunction;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Compiler {
    pub fn try_compile_inlined_call(
        &mut self,
        expr: *mut AstExprCall,
        func: *mut AstExprFunction,
        target: u8,
        target_count: u8,
        mult_ret: bool,
        threshold_base: i32,
        threshold_max_boost: i32,
        depth_limit: i32,
    ) -> bool {
        unsafe {
            let fi = self.functions.find(&func);
            LUAU_ASSERT!(fi.is_some());
            let fi = fi.unwrap();
            let fi_stack_size = fi.stack_size;
            let mut call_cost_model = fi.cost_model;
            let fi_cost_model = fi.cost_model;

            if self.reg_top > 128 || fi_stack_size > 32 {
                (*self.bytecode)
                    .add_debug_remark(format_args!("inlining failed: high register pressure"));
                return false;
            }

            if self.inline_frames.len() as i32 >= depth_limit {
                (*self.bytecode)
                    .add_debug_remark(format_args!("inlining failed: too many inlined frames"));
                return false;
            }

            for frame in &self.inline_frames {
                if frame.func == func {
                    (*self.bytecode).add_debug_remark(format_args!(
                        "inlining failed: can't inline recursive calls"
                    ));
                    return false;
                }
            }

            if mult_ret {
                (*self.bytecode).add_debug_remark(format_args!(
                    "inlining failed: can't convert fixed returns to multret"
                ));
                return false;
            }

            let mut varc = [false; 8];
            let mut has_constant = false;

            let mut i = 0usize;
            while i < (*func).args.size && i < (*expr).args.size && i < 8 {
                if self.is_constant(*(*expr).args.data.add(i)) {
                    varc[i] = true;
                    has_constant = true;
                }
                i += 1;
            }

            if (*expr).args.size != 0
                && !self.is_expr_mult_ret(*(*expr).args.data.add((*expr).args.size - 1))
            {
                i = (*expr).args.size;
                while i < (*func).args.size && i < 8 {
                    varc[i] = true;
                    has_constant = true;
                    i += 1;
                }
            }

            if has_constant {
                call_cost_model = self.cost_model_inlined_call(expr, func);
            }

            let inlined_cost = compute_cost(
                call_cost_model,
                varc.as_ptr(),
                core::cmp::min((*func).args.size, 8),
            );
            let baseline_cost = compute_cost(fi_cost_model, core::ptr::null(), 0) + 3;
            let inline_profit = if inlined_cost == 0 {
                threshold_max_boost
            } else {
                core::cmp::min(threshold_max_boost, 100 * baseline_cost / inlined_cost)
            };

            let threshold = threshold_base * inline_profit / 100;

            if inlined_cost > threshold {
                (*self.bytecode).add_debug_remark(format_args!(
                    "inlining failed: too expensive (cost {}, profit {:.2}x)",
                    inlined_cost,
                    inline_profit as f64 / 100.0
                ));
                return false;
            }

            (*self.bytecode).add_debug_remark(format_args!(
                "inlining succeeded (cost {}, profit {:.2}x, depth {})",
                inlined_cost,
                inline_profit as f64 / 100.0,
                self.inline_frames.len() as i32
            ));

            self.compile_inlined_call(expr, func, target, target_count);
            true
        }
    }
}
