use crate::records::type_map_visitor::TypeMapVisitor;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_type::AstType;
use luaur_common::enums::luau_bytecode_type::LuauBytecodeType;
use luaur_common::records::dense_hash_set::DenseHashSet;

impl<'a> TypeMapVisitor<'a> {
    pub fn record_resolved_type_ast_expr_ast_type(
        &mut self,
        expr: *mut AstExpr,
        ty: *const AstType,
    ) -> LuauBytecodeType {
        unsafe {
            let ty = self.resolve_aliases_deprecated(ty);

            *self.resolved_exprs.get_or_insert(expr) = ty;

            let mut seen_aliases: DenseHashSet<luaur_ast::records::ast_name::AstName> =
                DenseHashSet::new(luaur_ast::records::ast_name::AstName::new());

            let bty = crate::functions::get_type::get_type(
                ty,
                Default::default(),
                &self.type_aliases,
                true,
                self.host_vector_type,
                self.userdata_types,
                self.bytecode,
                &mut seen_aliases,
            );

            *self.expr_types.get_or_insert(expr) = bty;
            bty
        }
    }
}
