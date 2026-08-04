use crate::enums::type_constant_folding::Type;
use crate::records::constant::Constant;
use crate::records::constant_visitor::ConstantVisitor;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_local::AstLocal;
use luaur_common::records::dense_hash_map::DenseHashMap;

impl<'a> ConstantVisitor<'a> {
    pub fn record_constant<T>(&mut self, map: &mut DenseHashMap<T, Constant>, key: T, value: &Constant)
    where
        T: Clone + Eq + core::hash::Hash + luaur_common::records::dense_hash_table::DenseDefault + Default + 'static,
    {
        let fold_optimize = luaur_common::FFlag::LuauCompileFoldOptimize.get();
        let propagate_table = luaur_common::FFlag::LuauCompilePropagateTableProps2.get();

        if fold_optimize && propagate_table {
            if value.r#type == Type::Type_Table {
                // Table constants are recorded in a separate map
            } else if value.r#type != Type::Type_Unknown {
                self.log_change_internal(map, key.clone(), None);
                *map.get_or_insert(key) = *value;
            } else if self.was_empty {
                // No need to clear out entries if we started with empty maps
            } else if let Some(old) = map.find(&key) {
                let old_val = *old;
                self.log_change_internal(map, key.clone(), Some(&old_val));
                if let Some(old_mut) = map.find_mut(&key) {
                    old_mut.r#type = Type::Type_Unknown;
                }
            }
        } else {
            if value.r#type != Type::Type_Unknown {
                *map.get_or_insert(key) = *value;
            } else if self.was_empty && !propagate_table {
                // No-op
            } else if let Some(old_mut) = map.find_mut(&key) {
                old_mut.r#type = Type::Type_Unknown;
            }
        }
    }

    /// Helper to dispatch to the correct log_change overload based on the map key type.
    fn log_change_internal<T>(
        &mut self,
        map: &mut DenseHashMap<T, Constant>,
        key: T,
        existing: Option<&Constant>,
    ) where
        T: Clone + Eq + core::hash::Hash + luaur_common::records::dense_hash_table::DenseDefault + Default + 'static,
    {
        // We use pointer comparison of the type ID to determine which log_change to call.
        // In Luau's ConstantFolding, T is either *mut AstExpr or *mut AstLocal.
        let key_any = &key as &dyn core::any::Any;

        if let Some(expr_key) = key_any.downcast_ref::<*mut AstExpr>() {
            let m = unsafe {
                core::mem::transmute::<&mut DenseHashMap<T, Constant>, &mut DenseHashMap<*mut AstExpr, Constant>>(
                    map,
                )
            };
            self.log_change_dense_hash_map_ast_expr_constant_ast_expr_constant(m, *expr_key, existing);
        } else if let Some(local_key) = key_any.downcast_ref::<*mut AstLocal>() {
            let m = unsafe {
                core::mem::transmute::<&mut DenseHashMap<T, Constant>, &mut DenseHashMap<*mut AstLocal, Constant>>(
                    map,
                )
            };
            self.log_change_dense_hash_map_ast_local_constant_ast_local_constant(m, *local_key, existing);
        }
    }
}
