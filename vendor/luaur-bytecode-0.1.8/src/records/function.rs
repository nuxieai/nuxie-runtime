extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Function {
    pub(crate) data: String,
    pub(crate) maxstacksize: u8,
    pub(crate) numparams: u8,
    pub(crate) numupvalues: u8,
    pub(crate) isvararg: bool,
    pub(crate) debugname: u32,
    pub(crate) debuglinedefined: i32,
    pub(crate) dump: String,
    pub(crate) dumpname: String,
    pub(crate) dumpinstoffs: Vec<i32>,
    pub(crate) typeinfo: String,
}
