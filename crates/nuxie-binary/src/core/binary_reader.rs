use crate::StringValue;
use anyhow::{Context, Result, bail};

pub(crate) struct BinaryReader<'a> {
    pub(crate) bytes: &'a [u8],
    pub(crate) offset: usize,
    overflowed: bool,
    int_range_error: bool,
}

#[allow(dead_code)] // Complete pinned BinaryReader surface; runtime consumers use a subset.
impl<'a> BinaryReader<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            offset: 0,
            overflowed: false,
            int_range_error: false,
        }
    }

    pub(crate) fn reached_end(&self) -> bool {
        self.offset == self.bytes.len() || self.has_error()
    }

    pub(crate) fn length_in_bytes(&self) -> usize {
        self.bytes.len()
    }

    pub(crate) fn position(&self) -> usize {
        self.offset
    }

    pub(crate) fn did_overflow(&self) -> bool {
        self.overflowed
    }

    pub(crate) fn did_int_range_error(&self) -> bool {
        self.int_range_error
    }

    pub(crate) fn has_error(&self) -> bool {
        self.overflowed || self.int_range_error
    }

    fn overflow(&mut self) {
        self.overflowed = true;
        self.offset = self.bytes.len();
    }

    fn int_range_error(&mut self) {
        self.int_range_error = true;
        self.offset = self.bytes.len();
    }

    pub(crate) fn read_var_uint(&mut self) -> Result<u64> {
        let mut result = 0u64;
        let mut shift = 0u8;

        loop {
            let byte = self.read_byte()?;
            result |= u64::from(byte & 0x7f).wrapping_shl(u32::from(shift));

            if byte & 0x80 == 0 {
                return Ok(result);
            }

            shift = shift.wrapping_add(7);
        }
    }

    pub(crate) fn read_string_with_length(&mut self, length: usize) -> Result<StringValue> {
        let raw = self.read_bytes_exact(length)?.to_vec();
        let value = String::from_utf8(raw.clone()).ok();
        Ok(StringValue { value, raw })
    }

    pub(crate) fn read_string(&mut self) -> Result<StringValue> {
        let length =
            usize::try_from(self.read_var_uint()?).context("length does not fit in usize")?;
        self.read_string_with_length(length)
    }

    pub(crate) fn read_length_prefixed_bytes(&mut self) -> Result<&'a [u8]> {
        let length =
            usize::try_from(self.read_var_uint()?).context("length does not fit in usize")?;
        self.read_bytes_exact(length)
    }

    pub(crate) fn read_bytes_exact(&mut self, length: usize) -> Result<&'a [u8]> {
        let remaining = self.bytes.len().saturating_sub(self.offset);
        if length > remaining {
            let offset = self.offset;
            self.overflow();
            bail!("read {length} bytes past end at byte {offset}");
        }

        let end = self.offset + length;
        let bytes = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(bytes)
    }

    pub(crate) fn read_f32(&mut self) -> Result<f32> {
        let bytes: [u8; 4] = self.read_bytes_exact(4)?.try_into().unwrap();
        Ok(f32::from_le_bytes(bytes))
    }

    pub(crate) fn read_byte(&mut self) -> Result<u8> {
        let Some(&byte) = self.bytes.get(self.offset) else {
            let offset = self.offset;
            self.overflow();
            bail!("read past end at byte {offset}");
        };
        self.offset += 1;
        Ok(byte)
    }

    pub(crate) fn read_u16(&mut self) -> Result<u16> {
        let bytes: [u8; 2] = self.read_bytes_exact(2)?.try_into().unwrap();
        Ok(u16::from_le_bytes(bytes))
    }

    pub(crate) fn read_u32(&mut self) -> Result<u32> {
        let bytes: [u8; 4] = self.read_bytes_exact(4)?.try_into().unwrap();
        Ok(u32::from_le_bytes(bytes))
    }

    pub(crate) fn read_var_uint_u32(&mut self, label: &str) -> Result<u32> {
        let value = self.read_var_uint()?;
        let Ok(value) = u32::try_from(value) else {
            self.int_range_error();
            bail!("{label} {value} does not fit in C++ unsigned int");
        };
        Ok(value)
    }

    pub(crate) fn reset(&mut self) {
        self.offset = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::BinaryReader;

    #[test]
    fn read_bytes_overflows_on_a_length_past_the_end_of_the_buffer() {
        let storage = [1, 2, 3, 4];
        let mut reader = BinaryReader::new(&storage);

        let error = reader
            .read_bytes_exact(1000)
            .expect_err("an overlong file-controlled span must overflow");

        assert!(error.to_string().contains("past end"), "{error:#}");
        assert!(reader.reached_end());
    }

    #[test]
    fn read_bytes_returns_exactly_the_in_range_bytes_requested() {
        let storage = [1, 2, 3, 4];
        let mut reader = BinaryReader::new(&storage);

        let bytes = reader.read_bytes_exact(3).expect("in-range bytes");
        assert_eq!(bytes, [1, 2, 3]);
        assert!(!reader.reached_end());

        reader
            .read_bytes_exact(100)
            .expect_err("only one byte remains");
        assert!(reader.reached_end());
    }
}
