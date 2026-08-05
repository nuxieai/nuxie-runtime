use alloc::vec::Vec;
use luaur_ast::records::ast_local::AstLocal;
use luaur_common::records::dense_hash_map::DenseHashMap;
use luaur_common::records::dense_hash_set::DenseHashSet;

#[derive(Debug)]
pub struct Exports {
    pub export_table_local: AstLocal,
    pub exported_classes: DenseHashMap<*mut AstLocal, u8>,
    pub exported_functions: DenseHashSet<*mut AstLocal>,
    pub exported_variables: Vec<*mut AstLocal>,
    pub exported_table_cid: i32,
    pub has_exports: bool,
}

impl Exports {
    pub fn new(export_table_local: AstLocal) -> Self {
        Self {
            export_table_local,
            exported_classes: DenseHashMap::new(core::ptr::null_mut()),
            exported_functions: DenseHashSet::new(core::ptr::null_mut()),
            exported_variables: Vec::new(),
            exported_table_cid: -1,
            has_exports: false,
        }
    }

    pub fn is_empty(&self) -> bool {
        !self.has_exports
            && self.exports_classes_empty()
            && self.exported_functions.empty()
            && self.exported_variables.is_empty()
    }

    fn exports_classes_empty(&self) -> bool {
        self.exported_classes.size() == 0
    }
}
