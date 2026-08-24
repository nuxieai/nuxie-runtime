//! Safe-Rust container adaptations of all four cases in pinned
//! `tests/unit_tests/runtime/object_stream_test.cpp`.

use std::collections::VecDeque;

struct ObjectStream<T>(VecDeque<T>);

impl<T> ObjectStream<T> {
    fn new() -> Self {
        Self(VecDeque::new())
    }

    fn empty(&self) -> bool {
        self.0.is_empty()
    }

    fn push(&mut self, value: T) -> &mut Self {
        self.0.push_back(value);
        self
    }

    fn pop(&mut self) -> T {
        self.0.pop_front().expect("stream value")
    }
}

#[derive(Default)]
struct PodStream(VecDeque<u8>);

impl PodStream {
    fn empty(&self) -> bool {
        self.0.is_empty()
    }

    fn push_bytes(&mut self, bytes: &[u8]) -> &mut Self {
        self.0.extend(bytes.iter().copied());
        self
    }

    fn pop_bytes<const N: usize>(&mut self) -> [u8; N] {
        std::array::from_fn(|_| self.0.pop_front().expect("POD stream byte"))
    }

    fn push_bool(&mut self, value: bool) -> &mut Self {
        self.push_bytes(&[u8::from(value)])
    }

    fn pop_bool(&mut self) -> bool {
        self.pop_bytes::<1>()[0] != 0
    }

    fn push_i8(&mut self, value: i8) -> &mut Self {
        self.push_bytes(&value.to_ne_bytes())
    }

    fn pop_i8(&mut self) -> i8 {
        i8::from_ne_bytes(self.pop_bytes())
    }

    fn push_i16(&mut self, value: i16) -> &mut Self {
        self.push_bytes(&value.to_ne_bytes())
    }

    fn pop_i16(&mut self) -> i16 {
        i16::from_ne_bytes(self.pop_bytes())
    }

    fn push_i32(&mut self, value: i32) -> &mut Self {
        self.push_bytes(&value.to_ne_bytes())
    }

    fn pop_i32(&mut self) -> i32 {
        i32::from_ne_bytes(self.pop_bytes())
    }

    fn push_i64(&mut self, value: i64) -> &mut Self {
        self.push_bytes(&value.to_ne_bytes())
    }

    fn pop_i64(&mut self) -> i64 {
        i64::from_ne_bytes(self.pop_bytes())
    }

    fn push_f32(&mut self, value: f32) -> &mut Self {
        self.push_bytes(&value.to_ne_bytes())
    }

    fn pop_f32(&mut self) -> f32 {
        f32::from_ne_bytes(self.pop_bytes())
    }

    fn push_i32_array<const N: usize>(&mut self, value: [i32; N]) -> &mut Self {
        for item in value {
            self.push_i32(item);
        }
        self
    }

    fn pop_i32_array<const N: usize>(&mut self) -> [i32; N] {
        std::array::from_fn(|_| self.pop_i32())
    }

    fn push_f32_array<const N: usize>(&mut self, value: [f32; N]) -> &mut Self {
        for item in value {
            self.push_f32(item);
        }
        self
    }

    fn pop_f32_array<const N: usize>(&mut self) -> [f32; N] {
        std::array::from_fn(|_| self.pop_f32())
    }
}

#[test]
fn object_stream() {
    let mut stream = ObjectStream::new();
    assert!(stream.empty());
    stream.push("hello".to_owned());
    assert!(!stream.empty());
    stream.push("world".to_owned()).push("hi".to_owned());
    assert!(!stream.empty());
    stream.push("rive too".to_owned());
    assert!(!stream.empty());

    let string = stream.pop();
    let string2 = stream.pop();
    let string3 = stream.pop();
    assert_eq!(string, "hello");
    assert_eq!(string2, "world");
    assert_eq!(string3, "hi");
    assert!(!stream.empty());
    let string = stream.pop();
    assert_eq!(string, "rive too");
    assert!(stream.empty());
}

#[test]
fn object_stream_forces_deque_reallocation() {
    let mut stream = ObjectStream::new();
    for index in 0..100 {
        let values = (0..index)
            .map(|value| value + index * 123 + 257)
            .collect::<Vec<_>>();
        stream.push(values);
    }
    for index in 0..100 {
        let values = (0..index)
            .map(|value| value + index * 123 + 257)
            .collect::<Vec<_>>();
        let values2 = stream.pop();
        assert_eq!(values2.len(), index as usize);
        assert_eq!(values, values2);
    }
    assert!(stream.empty());
}

#[test]
fn pod_stream() {
    let mut stream = PodStream::default();
    assert!(stream.empty());

    let pair = (1i32, 3.0f32);
    let boolean = true;
    let f3d = [3.0f32, 2.0, 1.0];

    stream.push_i32(pair.0).push_f32(pair.1);
    assert!(!stream.empty());
    stream.push_bool(boolean).push_f32_array(f3d);
    assert!(!stream.empty());

    assert!(!stream.empty());
    let pair2 = (stream.pop_i32(), stream.pop_f32());
    assert_eq!(pair, pair2);

    assert!(!stream.empty());
    stream
        .push_f32_array([1.0, 2.0, 3.0, 4.0])
        .push_i32_array([1, 2]);

    assert!(!stream.empty());
    let boolean2 = stream.pop_bool();
    assert_eq!(boolean, boolean2);

    assert!(!stream.empty());
    let f3d2 = stream.pop_f32_array::<3>();
    let f4d2 = stream.pop_f32_array::<4>();
    assert_eq!(f3d, f3d2);
    assert_eq!(f4d2, [1.0, 2.0, 3.0, 4.0]);

    assert!(!stream.empty());
    let array2 = stream.pop_i32_array::<2>();
    assert_eq!(array2, [1, 2]);

    assert!(stream.empty());
}

#[test]
fn pod_stream_forces_deque_reallocation() {
    let mut stream = PodStream::default();
    for i16_value in 0i16..1 << 12 {
        let i8_value = (i16_value - 3) as i8;
        let i32_value = i32::from(i16_value) + 12;
        let i64_value = i64::from(i32_value) << 16;
        let i32x2 = [15, i32_value];
        let i32x4 = [17, 18, i32_value, 20];
        stream
            .push_bool(true)
            .push_bool(false)
            .push_i8(i8_value)
            .push_i16(i16_value)
            .push_i32(i32_value)
            .push_i64(i64_value)
            .push_i32_array(i32x2)
            .push_i32_array(i32x4)
            .push_i32_array(i32x4)
            .push_bool(false)
            .push_i32(i32_value)
            .push_i32_array(i32x2)
            .push_bool(true)
            .push_i64(i64_value)
            .push_i8(i8_value)
            .push_i16(i16_value);
    }

    for expected in 0i16..1 << 12 {
        let b1 = stream.pop_bool();
        let b2 = stream.pop_bool();
        let i8_value = stream.pop_i8();
        let i16_value = stream.pop_i16();
        let i32_value = stream.pop_i32();
        let i64_value = stream.pop_i64();
        let i32x2 = stream.pop_i32_array::<2>();
        let i32x4 = stream.pop_i32_array::<4>();
        assert!(b1);
        assert!(!b2);
        assert_eq!(i8_value, (i16_value - 3) as i8);
        assert_eq!(i16_value, expected);
        assert_eq!(i32_value, i32::from(i16_value) + 12);
        assert_eq!(i64_value, i64::from(i32_value) << 16);
        assert_eq!(i32x2, [15, i32_value]);
        assert_eq!(i32x4, [17, 18, i32_value, 20]);

        let i32x4 = stream.pop_i32_array::<4>();
        let b1 = stream.pop_bool();
        let i32_value = stream.pop_i32();
        let i32x2 = stream.pop_i32_array::<2>();
        let b2 = stream.pop_bool();
        let i64_value = stream.pop_i64();
        let i8_value = stream.pop_i8();
        let i16_value = stream.pop_i16();
        assert!(!b1);
        assert!(b2);
        assert_eq!(i8_value, (i16_value - 3) as i8);
        assert_eq!(i16_value, expected);
        assert_eq!(i32_value, i32::from(i16_value) + 12);
        assert_eq!(i64_value, i64::from(i32_value) << 16);
        assert_eq!(i32x2, [15, i32_value]);
        assert_eq!(i32x4, [17, 18, i32_value, 20]);
    }
    assert!(stream.empty());
}
