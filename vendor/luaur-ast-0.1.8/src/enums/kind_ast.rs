#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum Kind {
    List,    // foo, in which case key is a nullptr
    Record,  // foo=bar, in which case key is a AstExprConstantString
    General, // [foo]=bar
}
