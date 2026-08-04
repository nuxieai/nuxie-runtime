use crate::records::ast_local::AstLocal;
use crate::records::ast_name::AstName;
use crate::records::fragment_parse_resume_settings::FragmentParseResumeSettings;
use crate::records::position::Position;
use alloc::vec::Vec;
use luaur_common::records::dense_hash_map::DenseHashMap;

impl FragmentParseResumeSettings {
    pub fn new(
        local_map: DenseHashMap<AstName, *mut AstLocal>,
        local_stack: Vec<*mut AstLocal>,
        resume_position: Position,
    ) -> Self {
        Self {
            local_map,
            local_stack,
            resume_position,
        }
    }
}
