#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Loop {
    OneShot = 0,
    Loop = 1,
    PingPong = 2,
}
