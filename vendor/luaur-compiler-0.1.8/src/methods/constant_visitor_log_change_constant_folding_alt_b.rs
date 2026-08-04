use crate::records::constant::Constant;
use crate::records::constant_visitor::ConstantVisitor;
use luaur_ast::records::ast_local::AstLocal;
use luaur_common::records::dense_hash_map::DenseHashMap;

impl<'a> ConstantVisitor<'a> {
    pub fn log_change_dense_hash_map_ast_local_constant_ast_local_constant(
        &mut self,
        map: &mut DenseHashMap<*mut AstLocal, Constant>,
        key: *mut AstLocal,
        existing: Option<&Constant>,
    ) {
        if self.local_change_log.is_null() {
            return;
        }

        let old = match existing {
            Some(existing_value) => Some(*existing_value),
            None => map.find(&key).copied(),
        };

        unsafe {
            (*self.local_change_log).push(crate::records::local_constant_change::LocalConstantChange {
                key,
                old_value: old.unwrap_or_default(),
                was_absent: old.is_none(),
            });
        }
    }
}
