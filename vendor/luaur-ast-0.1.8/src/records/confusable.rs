#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Confusable {
    pub(crate) codepoint: u32,
    pub(crate) text: [i8; 5],
}
