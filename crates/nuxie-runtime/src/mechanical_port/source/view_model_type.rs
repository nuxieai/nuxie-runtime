#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ViewModelType {
    Standard = 0,
    TextInput = 1,
    Global = 2,
}
