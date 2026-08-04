use crate::records::ast_stat::AstStat;

impl AstStat {
    pub fn as_stat(&mut self) -> *mut AstStat {
        self as *mut Self
    }
}
