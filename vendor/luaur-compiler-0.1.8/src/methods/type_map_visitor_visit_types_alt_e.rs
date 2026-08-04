use crate::records::type_map_visitor::TypeMapVisitor;
use luaur_ast::records::ast_stat_local_function::AstStatLocalFunction;
use luaur_ast::records::ast_type_pack_explicit::AstTypePackExplicit;

pub fn visit_ast_stat_local_function(
    this: &mut TypeMapVisitor<'_>,
    node: *mut AstStatLocalFunction,
) -> bool {
    unsafe {
        if node.is_null() {
            return true;
        }

        let n = &*node;
        if !n.func.is_null() && !(*n.func).return_annotation.is_null() {
            let return_annotation = (*n.func).return_annotation;
            let type_pack = luaur_ast::rtti::ast_node_as::<AstTypePackExplicit>(
                return_annotation as *mut luaur_ast::records::ast_node::AstNode,
            );

            if !type_pack.is_null() {
                let type_list = &(*type_pack).type_list;
                let types = type_list.types.as_slice();
                if !types.is_empty() {
                    let first_type = types[0];
                    this.function_return_types.try_insert(n.name, first_type);
                }
            }
        }
    }

    true // Let generic visitor step into all expressions
}

impl TypeMapVisitor<'_> {
    pub fn visit_ast_stat_local_function(&mut self, node: *mut AstStatLocalFunction) -> bool {
        visit_ast_stat_local_function(self, node)
    }

    pub fn visit_ast_expr_function(
        &mut self,
        node: *mut luaur_ast::records::ast_expr_function::AstExprFunction,
    ) -> bool {
        let type_str = crate::functions::get_function_type::get_function_type(
            node,
            &self.type_aliases,
            self.host_vector_type,
            &self.userdata_types,
            &mut *self.bytecode,
        );
        if !type_str.is_empty() {
            *self.function_types.get_or_insert(node) = type_str;
        }
        true
    }
}

trait VisitAstStatLocalFunction {
    fn visit(&mut self, visitor: &mut TypeMapVisitor<'_>);
}

impl VisitAstStatLocalFunction for AstStatLocalFunction {
    fn visit(&mut self, visitor: &mut TypeMapVisitor<'_>) {
        visitor.visit_ast_stat_local_function(self as *mut AstStatLocalFunction);
    }
}
