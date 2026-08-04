use crate::functions::is_matching_global::is_matching_global;
use crate::records::type_map_visitor::TypeMapVisitor;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_call::AstExprCall;
use luaur_ast::records::ast_local::AstLocal;
use luaur_ast::records::ast_node::AstNode;
use luaur_ast::records::ast_stat_for_in::AstStatForIn;
use luaur_ast::records::ast_table_indexer::AstTableIndexer;
use luaur_ast::records::ast_type::AstType;

pub fn visit_ast_stat_for_in(this: &mut TypeMapVisitor<'_>, node: *mut AstStatForIn) -> bool {
    unsafe {
        if node.is_null() {
            return false;
        }

        let node_ref = &*node;

        for &expr_ptr in node_ref.values.as_slice() {
            if !expr_ptr.is_null() {
                luaur_ast::visit::ast_expr_visit(expr_ptr, this);
            }
        }

        // This is similar to how Compiler matches builtin iteration, but we also handle generalized iteration case
        if node_ref.vars.len() == 2 && node_ref.values.len() == 1 {
            let value_ptr = node_ref.values.as_slice()[0];
            let call = luaur_ast::rtti::ast_node_as::<AstExprCall>(value_ptr as *mut AstNode);

            if !call.is_null() && (*call).args.len() == 1 {
                let func = (*call).func;
                let arg = (*call).args.as_slice()[0];

                if is_matching_global(this.globals, func, c"ipairs".as_ptr()) {
                    let indexer = this.try_get_table_indexer(arg);
                    if !indexer.is_null() {
                        this.record_resolved_type_ast_local_ast_type(
                            node_ref.vars.as_slice()[0],
                            &(*this.builtin_types).number_type as *const _ as *const AstType,
                        );
                        this.record_resolved_type_ast_local_ast_type(
                            node_ref.vars.as_slice()[1],
                            (*indexer).result_type,
                        );
                    }
                } else if is_matching_global(this.globals, func, c"pairs".as_ptr()) {
                    let indexer = this.try_get_table_indexer(arg);
                    if !indexer.is_null() {
                        this.record_resolved_type_ast_local_ast_type(
                            node_ref.vars.as_slice()[0],
                            (*indexer).index_type,
                        );
                        this.record_resolved_type_ast_local_ast_type(
                            node_ref.vars.as_slice()[1],
                            (*indexer).result_type,
                        );
                    }
                }
            } else {
                let indexer = this.try_get_table_indexer(value_ptr);
                if !indexer.is_null() {
                    this.record_resolved_type_ast_local_ast_type(
                        node_ref.vars.as_slice()[0],
                        (*indexer).index_type,
                    );
                    this.record_resolved_type_ast_local_ast_type(
                        node_ref.vars.as_slice()[1],
                        (*indexer).result_type,
                    );
                }
            }
        }

        for i in 0..node_ref.vars.len() {
            let var_ptr = node_ref.vars.as_slice()[i];
            let var = &mut *var_ptr;

            if !var.annotation.is_null() {
                this.record_resolved_type_ast_local_ast_type(var_ptr, var.annotation);
            }
        }

        if !node_ref.body.is_null() {
            luaur_ast::visit::ast_stat_visit(
                node_ref.body as *mut luaur_ast::records::ast_stat::AstStat,
                this,
            );
        }
    }

    false
}

impl<'a> TypeMapVisitor<'a> {
    pub fn visit_ast_stat_for_in(&mut self, node: *mut AstStatForIn) -> bool {
        visit_ast_stat_for_in(self, node)
    }
}

trait AstExprVisit {
    fn visit(&mut self, visitor: &mut TypeMapVisitor<'_>);
}

impl AstExprVisit for AstExpr {
    fn visit(&mut self, visitor: &mut TypeMapVisitor<'_>) {
        unsafe {
            luaur_ast::visit::ast_expr_visit(self as *mut AstExpr, visitor);
        }
    }
}
