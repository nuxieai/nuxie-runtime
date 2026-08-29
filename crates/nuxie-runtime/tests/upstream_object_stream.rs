//! All four pinned object-stream cases, exercising the translated production owners.

use nuxie_runtime::source::object_stream::{ObjectStream, PodStream};

#[test]
fn object_stream() {
    let mut stream = ObjectStream::default();
    assert!(stream.empty());
    stream.write("hello".to_owned());
    assert!(!stream.empty());
    stream.write("world".to_owned()).write("hi".to_owned());
    assert!(!stream.empty());
    stream.write("rive too".to_owned());
    assert!(!stream.empty());

    let string = stream.read();
    let string2 = stream.read();
    let string3 = stream.read();
    assert_eq!(string, "hello");
    assert_eq!(string2, "world");
    assert_eq!(string3, "hi");
    assert!(!stream.empty());
    let string = stream.read();
    assert_eq!(string, "rive too");
    assert!(stream.empty());
}

#[test]
fn object_stream_forces_deque_reallocation() {
    let mut stream = ObjectStream::default();
    for index in 0..100 {
        let values = (0..index)
            .map(|value| value + index * 123 + 257)
            .collect::<Vec<_>>();
        stream.write(values);
    }
    for index in 0..100 {
        let values = (0..index)
            .map(|value| value + index * 123 + 257)
            .collect::<Vec<_>>();
        let values2 = stream.read();
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

    stream.write(pair);
    assert!(!stream.empty());
    stream.write(boolean).write(f3d);
    assert!(!stream.empty());

    assert!(!stream.empty());
    let pair2 = stream.read::<(i32, f32)>();
    assert_eq!(pair, pair2);

    assert!(!stream.empty());
    stream.write([1.0f32, 2.0, 3.0, 4.0]).write([1, 2]);

    assert!(!stream.empty());
    let boolean2 = stream.read::<bool>();
    assert_eq!(boolean, boolean2);

    assert!(!stream.empty());
    let f3d2 = stream.read::<[f32; 3]>();
    let f4d2 = stream.read::<[f32; 4]>();
    assert_eq!(f3d, f3d2);
    assert_eq!(f4d2, [1.0, 2.0, 3.0, 4.0]);

    assert!(!stream.empty());
    let array2 = stream.read::<[i32; 2]>();
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
            .write(true)
            .write(false)
            .write(i8_value)
            .write(i16_value)
            .write(i32_value)
            .write(i64_value)
            .write(i32x2)
            .write(i32x4)
            .write(i32x4)
            .write(false)
            .write(i32_value)
            .write(i32x2)
            .write(true)
            .write(i64_value)
            .write(i8_value)
            .write(i16_value);
    }

    for expected in 0i16..1 << 12 {
        let b1 = stream.read::<bool>();
        let b2 = stream.read::<bool>();
        let i8_value = stream.read::<i8>();
        let i16_value = stream.read::<i16>();
        let i32_value = stream.read::<i32>();
        let i64_value = stream.read::<i64>();
        let i32x2 = stream.read::<[i32; 2]>();
        let i32x4 = stream.read::<[i32; 4]>();
        assert!(b1);
        assert!(!b2);
        assert_eq!(i8_value, (i16_value - 3) as i8);
        assert_eq!(i16_value, expected);
        assert_eq!(i32_value, i32::from(i16_value) + 12);
        assert_eq!(i64_value, i64::from(i32_value) << 16);
        assert_eq!(i32x2, [15, i32_value]);
        assert_eq!(i32x4, [17, 18, i32_value, 20]);

        let i32x4 = stream.read::<[i32; 4]>();
        let b1 = stream.read::<bool>();
        let i32_value = stream.read::<i32>();
        let i32x2 = stream.read::<[i32; 2]>();
        let b2 = stream.read::<bool>();
        let i64_value = stream.read::<i64>();
        let i8_value = stream.read::<i8>();
        let i16_value = stream.read::<i16>();
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
