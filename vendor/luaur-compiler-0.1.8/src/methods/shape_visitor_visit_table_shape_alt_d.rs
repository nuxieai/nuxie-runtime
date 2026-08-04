use luaur_ast::records::ast_stat_for::AstStatFor;
use luaur_ast::records::ast_expr_constant_number::AstExprConstantNumber;
use crate::records::shape_visitor::ShapeVisitor;

// TableShape.cpp: `static const int kMaxLoopBound = 16;`
const K_MAX_LOOP_BOUND: i32 = 16;

pub fn visit_ast_stat_for(this: &mut ShapeVisitor<'_>, node: *mut AstStatFor) -> bool {
    unsafe {
        if node.is_null() {
            return true;
        }

        let stat_for = &*node;
        let from = stat_for.from as *mut AstExprConstantNumber;
        let to = stat_for.to as *mut AstExprConstantNumber;

        let from_val = if !from.is_null() {
            let expr = &*from;
            Some(expr.value)
        } else {
            None
        };

        let to_val = if !to.is_null() {
            let expr = &*to;
            Some(expr.value)
        } else {
            None
        };

        if let (Some(from_v), Some(to_v)) = (from_val, to_val) {
            if from_v == 1.0
                && to_v >= 1.0
                && to_v <= K_MAX_LOOP_BOUND as f64
                && stat_for.step.is_null()
            {
                this.loops.try_insert(stat_for.var, to_v as core::ffi::c_uint);
            }
        }
    }

    true
}

impl<'a> ShapeVisitor<'a> {
    pub fn visit_ast_stat_for(&mut self, node: *mut AstStatFor) -> bool {
        visit_ast_stat_for(self, node)
    }
}
