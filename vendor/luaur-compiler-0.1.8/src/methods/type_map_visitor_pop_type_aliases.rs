use crate::records::type_map_visitor::TypeMapVisitor;
use luaur_ast::records::ast_name::AstName;
use luaur_ast::records::ast_stat_type_alias::AstStatTypeAlias;
use luaur_common::records::dense_hash_map::DenseHashMap;

impl<'a> TypeMapVisitor<'a> {
    pub fn pop_type_aliases(&mut self, alias_stack_top: usize) {
        while self.type_alias_stack.len() > alias_stack_top {
            let top = self.type_alias_stack.pop().unwrap();
            // C++ `typeAliases[top.first] = top.second` — operator[] OVERWRITES, restoring
            // the previous binding (often null) on block exit. try_insert was a no-op when
            // the key still existed, so a block-scoped alias (e.g. `type Part = number`
            // inside a do..end) leaked into the enclosing scope.
            *self.type_aliases.get_or_insert(top.0) = top.1;
        }
    }
}
