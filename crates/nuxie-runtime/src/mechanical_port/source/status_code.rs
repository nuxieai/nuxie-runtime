#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StatusCode {
    Ok,
    MissingObject,
    InvalidObject,
    FailedInversion,
}
