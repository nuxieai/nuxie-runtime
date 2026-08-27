use std::{collections::VecDeque, mem::size_of, ptr::NonNull};

use crate::mechanical_port::source::refcnt::{Rcp, RefCounted};

#[derive(Default)]
pub struct ObjectStream<T> {
    stream: VecDeque<T>,
}

impl<T> ObjectStream<T> {
    pub fn empty(&self) -> bool {
        self.stream.is_empty()
    }

    pub fn write(&mut self, object: T) -> &mut Self {
        self.stream.push_back(object);
        self
    }

    pub fn read(&mut self) -> T {
        assert!(!self.empty());
        self.stream
            .pop_front()
            .expect("non-empty ObjectStream must have a front object")
    }
}

#[derive(Default)]
pub struct PodStream {
    byte_stream: VecDeque<u8>,
}

impl PodStream {
    pub fn empty(&self) -> bool {
        self.byte_stream.is_empty()
    }

    pub fn write<T: Copy>(&mut self, object: T) -> &mut Self {
        let bytes = unsafe {
            std::slice::from_raw_parts((&object as *const T).cast::<u8>(), size_of::<T>())
        };
        self.byte_stream.extend(bytes.iter().copied());
        self
    }

    pub fn read<T: Copy>(&mut self, destination: &mut T) -> &mut Self {
        assert!(self.byte_stream.len() >= size_of::<T>());
        let destination_bytes = unsafe {
            std::slice::from_raw_parts_mut((destination as *mut T).cast::<u8>(), size_of::<T>())
        };
        for byte in destination_bytes {
            *byte = self
                .byte_stream
                .pop_front()
                .expect("PODStream size assertion guarantees a byte");
        }
        self
    }

    pub fn write_rcp<T: RefCounted>(&mut self, mut object: Rcp<T>) -> &mut Self {
        self.write(object.release())
    }

    pub fn read_rcp<T: RefCounted>(&mut self, object: &mut Rcp<T>) -> &mut Self {
        let mut raw = std::ptr::null_mut();
        self.read(&mut raw);
        object.reset(NonNull::new(raw));
        self
    }
}
