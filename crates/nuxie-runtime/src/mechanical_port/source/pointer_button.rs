/// Upstream's int-backed enum, retaining values outside its named enumerators.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PointerButton(pub i32);

#[allow(non_upper_case_globals)]
impl PointerButton {
    pub const Primary: Self = Self(0);
    pub const Secondary: Self = Self(1);
    pub const Middle: Self = Self(2);
}
