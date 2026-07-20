#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct overloaded<Ts> {
    pub Ts: Ts,
}
