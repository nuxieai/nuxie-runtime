use crate::enums::type_constant_folding::Type;
use crate::functions::compute_cost::compute_cost;
use crate::functions::get_trip_count::get_trip_count;
use crate::functions::model_cost_cost_model::model_cost_ast_node_ast_local_usize_dense_hash_map_ast_expr_call_i32_dense_hash_map_ast_expr_constant;
use crate::records::compiler::Compiler;
use crate::records::constant::Constant;
use crate::records::variable::Variable;
use luaur_ast::records::ast_stat_for::AstStatFor;

impl Compiler {
    pub fn try_compile_unrolled_for(
        &mut self,
        stat: *mut AstStatFor,
        threshold_base: i32,
        threshold_max_boost: i32,
    ) -> bool {
        let stat_ref = unsafe { &*stat };

        let one = Constant {
            r#type: Type::Type_Number,
            string_length: 0,
            data: unsafe { core::mem::zeroed() },
        };
        let mut one_data = unsafe { core::mem::zeroed::<crate::records::constant::ConstantData>() };
        unsafe { one_data.value_number = 1.0 };
        let one = Constant {
            r#type: Type::Type_Number,
            string_length: 0,
            data: one_data,
        };

        let fromc = self.get_constant(stat_ref.from);
        let toc = self.get_constant(stat_ref.to);
        let stepc = if !stat_ref.step.is_null() {
            self.get_constant(stat_ref.step)
        } else {
            one
        };

        let trip_count = if fromc.r#type == Type::Type_Number
            && toc.r#type == Type::Type_Number
            && stepc.r#type == Type::Type_Number
        {
            get_trip_count(
                unsafe { fromc.data.value_number },
                unsafe { toc.data.value_number },
                unsafe { stepc.data.value_number },
            )
        } else {
            -1
        };

        if trip_count < 0 {
            unsafe {
                (*self.bytecode)
                    .add_debug_remark(format_args!("loop unroll failed: invalid iteration count"));
            }
            return false;
        }

        if trip_count > threshold_base {
            unsafe {
                (*self.bytecode).add_debug_remark(format_args!(
                    "loop unroll failed: too many iterations ({})",
                    trip_count
                ));
            }
            return false;
        }

        if let Some(lv) = self.variables.find(&stat_ref.var) {
            if lv.written {
                unsafe {
                    (*self.bytecode).add_debug_remark(format_args!(
                        "loop unroll failed: mutable loop variable"
                    ));
                }
                return false;
            }
        }

        let mut var = stat_ref.var;
        let cost_model = model_cost_ast_node_ast_local_usize_dense_hash_map_ast_expr_call_i32_dense_hash_map_ast_expr_constant(
            stat_ref.body as *mut luaur_ast::records::ast_node::AstNode,
            &var,
            1,
            unsafe { &*self.builtins_fold },
            &self.constants,
        );

        let varc = true;
        let unrolled_cost = compute_cost(cost_model, &varc as *const bool, 1) * trip_count;
        let baseline_cost = (compute_cost(cost_model, core::ptr::null(), 0) + 1) * trip_count;
        let unroll_profit = if unrolled_cost == 0 {
            threshold_max_boost
        } else {
            threshold_max_boost.min(100 * baseline_cost / unrolled_cost)
        };

        let threshold = threshold_base * unroll_profit / 100;

        if unrolled_cost > threshold {
            unsafe {
                (*self.bytecode).add_debug_remark(format_args!(
                    "loop unroll failed: too expensive (iterations {}, cost {}, profit {:.2}x)",
                    trip_count,
                    unrolled_cost,
                    unroll_profit as f64 / 100.0
                ));
            }
            return false;
        }

        unsafe {
            (*self.bytecode).add_debug_remark(format_args!(
                "loop unroll succeeded (iterations {}, cost {}, profit {:.2}x)",
                trip_count,
                unrolled_cost,
                unroll_profit as f64 / 100.0
            ));
        }

        self.compile_unrolled_for(
            stat,
            trip_count,
            unsafe { fromc.data.value_number },
            unsafe { stepc.data.value_number },
        );
        true
    }
}
