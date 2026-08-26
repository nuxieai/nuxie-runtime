//! Literal retained-owner ports of pinned `reader_test.cpp`.

use nuxie_binary::BinaryDataReader;

#[test]
fn wave_c8_reader_001_uint_leb_decoder() {
    let mut reader = BinaryDataReader::new(&[0x01]);
    assert_eq!(reader.read_var_uint(), 1);

    let mut reader = BinaryDataReader::new(&[0x0f]);
    assert_eq!(reader.read_var_uint(), 15);

    let mut reader = BinaryDataReader::new(&[0xe5, 0x8e, 0x26]);
    assert_eq!(reader.read_var_uint(), 624_485);
}

#[test]
fn wave_c8_reader_002_string_decoder() {
    let string_bytes = [
        0x4e, 0x65, 0x77, 0x20, 0x41, 0x72, 0x74, 0x62, 0x6f, 0x61, 0x72, 0x64,
    ];
    let mut encoded = vec![12];
    encoded.extend_from_slice(&string_bytes);
    let mut reader = BinaryDataReader::new(&encoded);
    let decoded_string = reader.read_string();
    assert_eq!(decoded_string, b"New Artboard");
    assert_eq!(reader.position() - 1, 12);

    let mut truncated = vec![12];
    truncated.extend_from_slice(&string_bytes[..11]);
    let mut reader = BinaryDataReader::new(&truncated);
    let _ = reader.read_string();
    assert!(reader.did_overflow());
}

#[test]
#[allow(clippy::approx_constant)] // Literal values from pinned reader_test.cpp.
fn wave_c8_reader_003_float_decoder() {
    let mut reader = BinaryDataReader::new(&[0x00, 0x00, 0xc8, 0x42]);
    let decoded_number = reader.read_float32();
    assert_eq!(decoded_number, 100.0);
    assert_eq!(reader.position(), 4);

    let mut reader = BinaryDataReader::new(&[0xd0, 0x0f, 0x49, 0x40]);
    let decoded_number = reader.read_float32();
    assert_eq!(decoded_number, 3.14159);
    assert_eq!(reader.position(), 4);

    let mut reader = BinaryDataReader::new(&[0x51, 0xf8, 0x2d, 0xc0]);
    let decoded_number = reader.read_float32();
    assert_eq!(decoded_number, -2.718281);
    assert_eq!(reader.position(), 4);

    let mut reader = BinaryDataReader::new(&[0x51, 0xf8, 0x2d]);
    let _ = reader.read_float32();
    assert!(reader.did_overflow());
}

#[test]
fn wave_c8_reader_004_byte_decoder() {
    let bytes = [0x00, 0x00, 0xc8, 0x42];
    let mut reader = BinaryDataReader::new(&bytes);

    let decoded_byte = reader.read_byte();
    assert_eq!(decoded_byte, bytes[reader.position() - 1]);
    let decoded_byte = reader.read_byte();
    assert_eq!(decoded_byte, bytes[reader.position() - 1]);
    let decoded_byte = reader.read_byte();
    assert_eq!(decoded_byte, bytes[reader.position() - 1]);
    let decoded_byte = reader.read_byte();
    assert_eq!(decoded_byte, bytes[reader.position() - 1]);
    assert_eq!(reader.read_byte(), 0);
}
