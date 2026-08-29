use crate::core::binary_reader::BinaryReader;
use anyhow::{Context, Result};
use nuxie_schema::IntStorage;

/// Header-defined `CoreIntType::zigzagEncode`.
#[allow(dead_code)]
pub(super) const fn zigzag_encode(value: i32) -> u32 {
    ((value as u32) << 1) ^ ((value >> 31) as u32)
}

/// Header-defined `CoreIntType::zigzagDecode`.
pub(super) const fn zigzag_decode(value: u32) -> i32 {
    ((value >> 1) as i32) ^ -((value & 1) as i32)
}

pub(super) fn deserialize(
    reader: &mut BinaryReader<'_>,
    storage: IntStorage,
    label: &str,
) -> Result<i32> {
    let encoded = u32::try_from(reader.read_var_uint()?)
        .with_context(|| format!("{label} zigzag value does not fit in u32"))?;
    let value = zigzag_decode(encoded);
    if storage == IntStorage::Int16 {
        i16::try_from(value)
            .map(i32::from)
            .with_context(|| format!("{label} {value} does not fit in C++ int16_t"))
    } else {
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::deserialize;
    use crate::core::binary_reader::BinaryReader;
    use nuxie_schema::IntStorage;

    #[test]
    fn zigzag_decodes_positive_and_negative_values() {
        for (bytes, expected) in [
            (&[0_u8][..], 0),
            (&[1][..], -1),
            (&[2][..], 1),
            (&[3][..], -2),
        ] {
            assert_eq!(
                deserialize(&mut BinaryReader::new(bytes), IntStorage::Int32, "int")
                    .expect("valid zigzag int"),
                expected
            );
        }
    }

    #[test]
    fn int16_storage_rejects_an_out_of_range_zigzag_value() {
        let error = deserialize(
            &mut BinaryReader::new(&[0x80, 0x80, 0x04]),
            IntStorage::Int16,
            "int16",
        )
        .expect_err("32768 cannot be stored in int16");
        assert!(error.to_string().contains("int16_t"), "{error:#}");
    }
}
