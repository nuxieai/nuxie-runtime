use nuxie_binary::{BinaryDataReader, BinaryStream, BinaryWriter};

fn fits_in<T>(value: i64) -> bool
where
    T: TryFrom<i64>,
{
    T::try_from(value).is_ok()
}

fn assert_fits_in_boundaries<T>()
where
    T: TryFrom<i64> + BoundedI64,
{
    let min = T::MIN;
    let max = T::MAX;
    assert!(fits_in::<T>(max));
    assert!(fits_in::<T>(min));
    assert!(!fits_in::<T>(max + 1));
    assert!(!fits_in::<T>(min - 1));
}

trait BoundedI64 {
    const MIN: i64;
    const MAX: i64;
}

macro_rules! bounded_i64 {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl BoundedI64 for $ty {
                const MIN: i64 = <$ty>::MIN as i64;
                const MAX: i64 = <$ty>::MAX as i64;
            }
        )+
    };
}

bounded_i64!(i8, u8, i16, u16, i32, u32);

fn pack_var_uint(mut value: u64) -> Vec<u8> {
    let mut bytes = Vec::new();
    while value > 127 {
        bytes.push(0x80 | (value as u8 & 0x7f));
        value >>= 7;
    }
    bytes.push(value as u8);
    bytes
}

fn read_var_uint_as<T>(value: u64) -> Option<T>
where
    T: TryFrom<u64>,
{
    let bytes = pack_var_uint(value);
    let mut reader = BinaryDataReader::new(&bytes);
    let value = T::try_from(reader.read_var_uint()).ok()?;
    (!reader.did_overflow()).then_some(value)
}

#[test]
fn binary_reader_test_fits_in_checks_direct_port() {
    assert_fits_in_boundaries::<i8>();
    assert_fits_in_boundaries::<u8>();
    assert_fits_in_boundaries::<i16>();
    assert_fits_in_boundaries::<u16>();
    assert_fits_in_boundaries::<i32>();
    assert_fits_in_boundaries::<u32>();
}

#[test]
fn binary_reader_test_range_checks_direct_port() {
    assert_eq!(read_var_uint_as::<u8>(100), Some(100));
    assert_eq!(read_var_uint_as::<u16>(100), Some(100));
    assert_eq!(read_var_uint_as::<u32>(100), Some(100));

    assert_eq!(read_var_uint_as::<u8>(1_000), None);
    assert_eq!(read_var_uint_as::<u16>(1_000), Some(1_000));
    assert_eq!(read_var_uint_as::<u32>(1_000), Some(1_000));

    assert_eq!(read_var_uint_as::<u8>(100_000), None);
    assert_eq!(read_var_uint_as::<u16>(100_000), None);
    assert_eq!(read_var_uint_as::<u32>(100_000), Some(100_000));
}

#[test]
fn reader_test_uint_leb_decoder_direct_port() {
    let mut reader = BinaryDataReader::new(&[0x01]);
    assert_eq!(reader.read_var_uint(), 1);

    let mut reader = BinaryDataReader::new(&[0x0f]);
    assert_eq!(reader.read_var_uint(), 15);

    let mut reader = BinaryDataReader::new(&[0xe5, 0x8e, 0x26]);
    assert_eq!(reader.read_var_uint(), 624_485);
}

#[test]
fn reader_test_string_decoder_direct_port() {
    const STRING_BYTES: &[u8] = b"New Artboard";

    let mut encoded = vec![12];
    encoded.extend_from_slice(STRING_BYTES);
    let mut reader = BinaryDataReader::new(&encoded);
    assert_eq!(reader.read_string(), STRING_BYTES);
    assert_eq!(reader.position(), 13);

    let mut truncated = vec![12];
    truncated.extend_from_slice(&STRING_BYTES[..11]);
    let mut reader = BinaryDataReader::new(&truncated);
    assert!(reader.read_string().is_empty());
    assert!(reader.did_overflow());
}

#[test]
#[allow(clippy::approx_constant)] // Literal values from pinned reader_test.cpp.
fn reader_test_float_decoder_direct_port() {
    let mut reader = BinaryDataReader::new(&[0x00, 0x00, 0xc8, 0x42]);
    assert_eq!(reader.read_float32(), 100.0);
    assert_eq!(reader.position(), 4);

    let mut reader = BinaryDataReader::new(&[0xd0, 0x0f, 0x49, 0x40]);
    assert_eq!(reader.read_float32(), 3.14159);
    assert_eq!(reader.position(), 4);

    let mut reader = BinaryDataReader::new(&[0x51, 0xf8, 0x2d, 0xc0]);
    assert_eq!(reader.read_float32(), -2.718281);
    assert_eq!(reader.position(), 4);

    let mut reader = BinaryDataReader::new(&[0x51, 0xf8, 0x2d]);
    assert_eq!(reader.read_float32(), 0.0);
    assert!(reader.did_overflow());
}

#[test]
fn reader_test_byte_decoder_direct_port() {
    let bytes = [0x00, 0x00, 0xc8, 0x42];
    let mut reader = BinaryDataReader::new(&bytes);
    for expected in bytes {
        assert_eq!(reader.read_byte(), expected);
    }
    assert_eq!(reader.read_byte(), 0);
    assert!(reader.did_overflow());
}

#[derive(Default)]
struct RecordingStream {
    bytes: Vec<u8>,
    flushes: usize,
}

impl BinaryStream for RecordingStream {
    fn write(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    fn flush(&mut self) {
        self.flushes += 1;
    }

    fn clear(&mut self) {
        self.bytes.clear();
    }
}

#[test]
#[allow(clippy::approx_constant)] // Literal value in pinned binary_reader_test.cpp.
fn binary_reader_test_all_data_types_can_be_written_and_read_direct_port() {
    // Direct port of binary_reader_test.cpp's "all data types can be written
    // & read" case. BinaryDataReader has no uint16 method upstream, so its two
    // byte reads retain the exact little-endian wire without inventing one.
    let mut bytes = Vec::new();
    {
        let mut writer = BinaryWriter::new(&mut bytes);
        writer.write_var_uint32(34);
        writer.write_u16(22);
        writer.write_f32(3.14);
    }

    let mut reader = BinaryDataReader::new(&bytes);
    assert_eq!(reader.read_var_uint32(), 34);
    assert_eq!(
        u16::from_le_bytes([reader.read_byte(), reader.read_byte()]),
        22
    );
    assert_eq!(reader.read_float32(), 3.14);
    assert!(reader.is_eof());
    assert!(!reader.did_overflow());
}

#[test]
#[allow(clippy::approx_constant)] // Literal F14 wire-oracle value, not mathematical pi.
fn all_binary_data_reader_types_round_trip() {
    let mut stream = RecordingStream::default();
    {
        let mut writer = BinaryWriter::new(&mut stream);
        writer.write_var_uint32(34);
        writer.write_var_uint64(u64::MAX);
        writer.write_f32(3.14);
        writer.write_f64(-7.25);
        writer.write_u8(0xab);
        writer.write_u32(0x89ab_cdef);
        writer.write_string(b"Rive\0Rust");
    }
    assert_eq!(stream.flushes, 1, "C++ BinaryWriter flushes on destruction");

    let mut reader = BinaryDataReader::new(&stream.bytes);
    assert_eq!(reader.length_in_bytes(), stream.bytes.len());
    assert_eq!(reader.read_var_uint32(), 34);
    assert_eq!(reader.read_var_uint(), u64::MAX);
    assert_eq!(reader.read_float32(), 3.14);
    assert_eq!(reader.read_float64(), -7.25);
    assert_eq!(reader.read_byte(), 0xab);
    assert_eq!(reader.read_uint32(), 0x89ab_cdef);
    assert_eq!(reader.read_string(), b"Rive\0Rust");
    assert!(reader.is_eof());
    assert!(!reader.did_overflow());
}

#[test]
fn binary_writer_matches_the_pinned_little_endian_wire_format() {
    let mut stream = RecordingStream::default();
    {
        let mut writer = BinaryWriter::new(&mut stream);
        writer.write_f32(1.0);
        writer.write_float(-0.0);
        writer.write_f64(1.0);
        writer.write_double(-0.0);
        writer.write_var_uint32(0);
        writer.write_var_uint32(127);
        writer.write_var_uint32(128);
        writer.write_var_uint64(u64::MAX);
        writer.write_u8(0x12);
        writer.write_u16(0x3456);
        writer.write_u32(0x789a_bcde);
        writer.write_bytes(&[]);
        writer.write_bytes(&[0xfe, 0xdc]);
        writer.write_string(b"A\0B");
    }

    let mut expected = Vec::new();
    expected.extend_from_slice(&1.0f32.to_le_bytes());
    expected.extend_from_slice(&(-0.0f32).to_le_bytes());
    expected.extend_from_slice(&1.0f64.to_le_bytes());
    expected.extend_from_slice(&(-0.0f64).to_le_bytes());
    expected.extend_from_slice(&[0x00, 0x7f, 0x80, 0x01]);
    expected.extend_from_slice(&[0xff; 9]);
    expected.push(0x01);
    expected.push(0x12);
    expected.extend_from_slice(&0x3456u16.to_le_bytes());
    expected.extend_from_slice(&0x789a_bcdeu32.to_le_bytes());
    expected.extend_from_slice(&[0xfe, 0xdc, 0x03, b'A', 0, b'B']);
    assert_eq!(stream.bytes, expected);
}

#[test]
fn binary_data_reader_overflow_is_sticky_and_moves_to_eof() {
    let mut reader = BinaryDataReader::new(&[0x80]);
    assert_eq!(reader.read_var_uint(), 0);
    assert!(reader.did_overflow());
    assert!(reader.is_eof());
    assert_eq!(reader.position(), 1);
    assert_eq!(reader.read_byte(), 0);

    reader.complete(&[0x2a]);
    assert_eq!(reader.length_in_bytes(), 1);
    assert_eq!(reader.position(), 0);
    assert!(
        reader.did_overflow(),
        "C++ complete does not clear overflow"
    );
    assert_eq!(reader.read_byte(), 0x2a);
}

#[test]
fn binary_data_reader_reset_only_rewinds_the_position() {
    let mut reader = BinaryDataReader::new(&[1, 2]);
    assert_eq!(reader.read_byte(), 1);
    reader.reset();
    assert_eq!(reader.position(), 0);
    assert_eq!(reader.read_byte(), 1);
    assert_eq!(reader.length_in_bytes(), 2);
    assert!(!reader.did_overflow());
}

#[test]
fn writer_clear_delegates_to_the_stream() {
    let mut stream = RecordingStream::default();
    {
        let mut writer = BinaryWriter::new(&mut stream);
        writer.write_bytes(&[1, 2, 3]);
        writer.clear();
        writer.write_u8(4);
    }
    assert_eq!(stream.bytes, [4]);
}
