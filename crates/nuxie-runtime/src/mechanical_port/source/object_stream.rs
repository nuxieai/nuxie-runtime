use std::{any::Any, collections::VecDeque};

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
    stream: VecDeque<Box<dyn Any + Send>>,
}

impl PodStream {
    pub fn empty(&self) -> bool {
        self.stream.is_empty()
    }

    pub fn write<T: Copy + Send + 'static>(&mut self, object: T) -> &mut Self {
        self.stream.push_back(Box::new(object));
        self
    }

    pub fn read<T: Copy + Send + 'static>(&mut self) -> T {
        let value = self
            .stream
            .pop_front()
            .expect("cannot read an empty PodStream")
            .downcast::<T>()
            .expect("PodStream producer and consumer types must match");
        *value
    }
}
