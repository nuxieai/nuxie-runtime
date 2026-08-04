use crate::records::compile_error::CompileError;
use crate::records::compiler::Compiler;
use crate::records::undefined_local_visitor::UndefinedLocalVisitor;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_stat::AstStat;
use luaur_ast::records::ast_stat_block::AstStatBlock;
use luaur_ast::records::ast_stat_local::AstStatLocal;
use luaur_ast::records::ast_stat_local_function::AstStatLocalFunction;
use luaur_ast::rtti;

impl Compiler {
    pub fn validate_continue_until(
        &mut self,
        cont: *mut AstStat,
        condition: *mut AstExpr,
        body: *mut AstStatBlock,
        start: usize,
    ) {
        let mut visitor = UndefinedLocalVisitor {
            self_: self as *mut Compiler,
            undef: core::ptr::null_mut(),
            locals: luaur_common::records::dense_hash_set::DenseHashSet::new(core::ptr::null_mut()),
        };

        unsafe {
            let body_ref = &*body;
            for i in start..body_ref.body.size {
                let stat = *body_ref.body.data.add(i);
                let local_stat = rtti::ast_node_as::<AstStatLocal>(stat as *mut _);
                if !local_stat.is_null() {
                    for &var in (*local_stat).vars.iter() {
                        visitor.locals.insert(var);
                    }
                } else {
                    let func_stat = rtti::ast_node_as::<AstStatLocalFunction>(stat as *mut _);
                    if !func_stat.is_null() {
                        visitor.locals.insert((*func_stat).name);
                    }
                }
            }

            luaur_ast::visit::ast_expr_visit(condition, &mut visitor);

            if !visitor.undef.is_null() {
                CompileError::raise(
                    &(*condition).base.location,
                    format_args!(
                        "Local {} used in the repeat..until condition is undefined because continue statement on line {} jumps over it",
                        core::ffi::CStr::from_ptr((*visitor.undef).name.value).to_string_lossy(),
                        (*cont).base.location.begin.line + 1
                    )
                );
            }
        }
    }
}
