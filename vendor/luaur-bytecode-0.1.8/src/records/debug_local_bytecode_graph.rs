#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DebugLocal<'a> {
    pub(crate) varname: &'a str,
    pub(crate) reg: u8,
    pub(crate) startpc: u32,
    pub(crate) endpc: u32,
}

impl Default for DebugLocal<'static> {
    fn default() -> Self {
        Self {
            varname: "",
            reg: 0,
            startpc: 0,
            endpc: 0,
        }
    }
}
