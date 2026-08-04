use crate::records::ast_local::AstLocal;
use crate::records::ast_name::AstName;
use crate::records::position::Position;
use alloc::vec::Vec;
use luaur_common::records::dense_hash_map::DenseHashMap;

#[derive(Debug, Clone)]
pub struct FragmentParseResumeSettings {
    pub(crate) local_map: DenseHashMap<AstName, *mut AstLocal>,
    pub(crate) local_stack: Vec<*mut AstLocal>,
    pub(crate) resume_position: Position,
}
