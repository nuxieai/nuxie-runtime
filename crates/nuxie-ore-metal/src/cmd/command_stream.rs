//! renderer/cmd/command_stream.hpp at e949498e.
//! Explicit codecs avoid reading uninitialized C-layout padding in Rust.
pub trait WirePod: Default + Sized {
    const SIZE: usize;
    fn encode(&self, bytes: &mut Vec<u8>);
    fn decode(bytes: &[u8]) -> Self;
}
macro_rules! primitive {
    ($($ty:ty),*) => {$ (
        impl WirePod for $ty {
            const SIZE: usize = std::mem::size_of::<Self>();
            fn encode(&self, bytes: &mut Vec<u8>) { bytes.extend_from_slice(&self.to_ne_bytes()); }
            fn decode(bytes: &[u8]) -> Self { Self::from_ne_bytes(bytes.try_into().expect("wire scalar width")) }
        }
    )*};
}
primitive!(u8, u16, u32, u64, i32, f32);

impl WirePod for bool {
    const SIZE: usize = 1;
    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(u8::from(*self));
    }
    fn decode(bytes: &[u8]) -> Self {
        bytes[0] != 0
    }
}
impl<T: WirePod, const N: usize> WirePod for [T; N]
where
    [T; N]: Default,
{
    const SIZE: usize = T::SIZE * N;
    fn encode(&self, bytes: &mut Vec<u8>) {
        for value in self {
            value.encode(bytes);
        }
    }
    fn decode(bytes: &[u8]) -> Self {
        std::array::from_fn(|i| T::decode(&bytes[i * T::SIZE..(i + 1) * T::SIZE]))
    }
}

/// Encode an existing repr(C) POD field-by-field, preserving ABI offsets
/// without ever reading its potentially uninitialized padding.
#[macro_export]
macro_rules! impl_wire_pod {
    ($name:ty { $($field:ident : $ty:ty),* $(,)? }) => {
        impl $crate::cmd::command_stream::WirePod for $name {
            const SIZE: usize = std::mem::size_of::<Self>();
            fn encode(&self, bytes: &mut Vec<u8>) {
                use $crate::cmd::command_stream::WirePod;
                let start = bytes.len();
                $(let offset = std::mem::offset_of!(Self, $field);
                  bytes.resize(start + offset, 0);
                  self.$field.encode(bytes);)*
                bytes.resize(start + Self::SIZE, 0);
            }
            fn decode(bytes: &[u8]) -> Self {
                use $crate::cmd::command_stream::WirePod;
                Self {
                    $($field: <$ty>::decode({
                        let offset = std::mem::offset_of!(Self, $field);
                        &bytes[offset..offset + <$ty>::SIZE]
                    })),*
                }
            }
        }
    };
}

#[derive(Default, Clone)]
pub struct CommandByteStream {
    commands: Vec<u8>,
    blobs: Vec<u8>,
}
impl CommandByteStream {
    pub fn append_blob(&mut self, data: &[u8]) -> u64 {
        assert!(data.len() <= u32::MAX as usize);
        self.blobs.resize((self.blobs.len() + 7) & !7, 0);
        let offset = self.blobs.len() as u64;
        self.blobs.extend_from_slice(data);
        offset
    }
    pub fn empty(&self) -> bool {
        self.commands.is_empty()
    }
    pub fn command_bytes(&self) -> &[u8] {
        &self.commands
    }
    pub fn blob_bytes(&self) -> &[u8] {
        &self.blobs
    }
    pub fn write_raw(&mut self, bytes: &[u8]) {
        self.commands.extend_from_slice(bytes);
    }
    pub fn write<P: WirePod>(&mut self, value: &P) {
        value.encode(&mut self.commands);
    }
    pub fn clear_bytes(&mut self) {
        self.commands.clear();
        self.blobs.clear();
    }
}

pub struct CommandReader<'a> {
    commands: &'a [u8],
    blobs: &'a [u8],
    pos: usize,
    overrun: bool,
}
impl<'a> CommandReader<'a> {
    pub fn new(commands: &'a [u8], blobs: &'a [u8]) -> Self {
        Self {
            commands,
            blobs,
            pos: 0,
            overrun: false,
        }
    }
    pub fn next<P: WirePod>(&mut self) -> Option<P> {
        let remaining = self.commands.len() - self.pos;
        if self.overrun || remaining < P::SIZE {
            if remaining != 0 {
                self.overrun = true;
            }
            return None;
        }
        Some(self.read())
    }
    pub fn next_u8(&mut self) -> Option<u8> {
        self.next()
    }
    pub fn read<P: WirePod>(&mut self) -> P {
        if self.commands.len() - self.pos < P::SIZE {
            self.overrun = true;
            return P::default();
        }
        let pod = P::decode(&self.commands[self.pos..self.pos + P::SIZE]);
        self.pos += P::SIZE;
        pod
    }
    pub fn skip(&mut self, bytes: usize) {
        if self.commands.len() - self.pos < bytes {
            self.overrun = true;
            return;
        }
        self.pos += bytes;
    }
    pub fn blob_at(&self, offset: u64, size: u32) -> &'a [u8] {
        let Some(end) = offset.checked_add(u64::from(size)) else {
            return &[];
        };
        if end > self.blobs.len() as u64 {
            return &[];
        }
        &self.blobs[offset as usize..end as usize]
    }
    pub fn position(&self) -> usize {
        self.pos
    }
    pub fn overrun(&self) -> bool {
        self.overrun
    }
}

/// Declare a C-POD wire owner with explicit field alignment and trailing padding.
/// All supported producers/consumers use the same primitive ABI widths.
#[macro_export]
macro_rules! wire_pod {
    ($name:ident { $($field:ident : $ty:ty),* $(,)? }) => {
        #[repr(C)]
        #[derive(Clone, Copy, Debug, Default)]
        pub struct $name { $(pub $field: $ty),* }
        impl $crate::cmd::command_stream::WirePod for $name {
            const SIZE: usize = {
                let mut size = 0usize;
                let mut alignment = 1usize;
                $(let a = std::mem::align_of::<$ty>();
                  if a > alignment { alignment = a; }
                  size = (size + a - 1) & !(a - 1);
                  size += std::mem::size_of::<$ty>();)*
                (size + alignment - 1) & !(alignment - 1)
            };
            fn encode(&self, bytes: &mut Vec<u8>) {
                use $crate::cmd::command_stream::WirePod;
                let start = bytes.len();
                $(let a = std::mem::align_of::<$ty>();
                  let offset = (bytes.len() - start + a - 1) & !(a - 1);
                  bytes.resize(start + offset, 0);
                  self.$field.encode(bytes);)*
                bytes.resize(start + Self::SIZE, 0);
            }
            fn decode(bytes: &[u8]) -> Self {
                use $crate::cmd::command_stream::WirePod;
                let mut offset = 0usize;
                $(let a = std::mem::align_of::<$ty>();
                  offset = (offset + a - 1) & !(a - 1);
                  let $field = <$ty>::decode(&bytes[offset..offset + <$ty>::SIZE]);
                  offset += <$ty>::SIZE;)*
                let _ = offset;
                Self { $($field),* }
            }
        }
    };
}
pub use crate::wire_pod;
