use crate::records::type_map_visitor::TypeMapVisitor;
use luaur_ast::records::ast_local::AstLocal;
use luaur_ast::records::ast_name::AstName;
use luaur_ast::records::ast_type::AstType;
use luaur_common::enums::luau_bytecode_type::{LuauBytecodeType, LBC_TYPE_ANY};
use luaur_common::records::dense_hash_set::DenseHashSet;

impl<'a> TypeMapVisitor<'a> {
    pub fn record_resolved_type_ast_local_ast_type(
        &mut self,
        local: *mut AstLocal,
        ty: *const AstType,
    ) -> LuauBytecodeType {
        let ty_resolved = self.resolve_aliases_deprecated(ty);

        *self.resolved_locals.get_or_insert(local) = ty_resolved;

        let mut seen_aliases = DenseHashSet::new(AstName::new());
        let bty = crate::functions::get_type::get_type(
            ty_resolved,
            Default::default(),
            &self.type_aliases,
            true,
            self.host_vector_type,
            self.userdata_types,
            self.bytecode,
            &mut seen_aliases,
        );

        if bty != LBC_TYPE_ANY {
            *self.local_types.get_or_insert(local) = bty;
        }

        bty
    }
}
