use crate::records::type_map_visitor::TypeMapVisitor;

use luaur_ast::records::ast_stat::AstStat;
use luaur_ast::records::ast_stat_block::AstStatBlock;
use luaur_ast::records::ast_stat_type_alias::AstStatTypeAlias;

pub fn type_map_visitor_push_type_aliases<'a>(
    this: &mut TypeMapVisitor<'a>,
    block: *mut AstStatBlock,
) -> usize {
    let alias_stack_top = this.type_alias_stack.len();

    unsafe {
        let body = &(*block).body;

        for stat_ptr in body.as_slice() {
            let stat = &mut **stat_ptr;

            let stat_as_node = stat as *mut AstStat as *mut luaur_ast::records::ast_node::AstNode;

            if luaur_ast::rtti::ast_node_is::<AstStatTypeAlias>(&*stat_as_node) {
                let alias_ptr = luaur_ast::rtti::ast_node_as::<AstStatTypeAlias>(stat_as_node);
                if alias_ptr.is_null() {
                    continue;
                }

                let alias_ref: &mut AstStatTypeAlias = &mut *alias_ptr;

                let prev_alias =
                    if let Some(alias_ptr) = this.type_aliases.find(&alias_ref.name).map(|p| *p) {
                        alias_ptr
                    } else {
                        core::ptr::null_mut()
                    };

                this.type_alias_stack.push((alias_ref.name, prev_alias));

                // C++ `prevAlias = alias` overwrites typeAliases[name]; try_insert was a
                // no-op when a same-named alias already existed (a nested redefinition),
                // leaving the outer alias in scope.
                *this.type_aliases.get_or_insert(alias_ref.name) = alias_ptr;
            }
        }
    }

    alias_stack_top
}
