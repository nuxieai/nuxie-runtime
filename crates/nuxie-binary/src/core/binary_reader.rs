use crate::StringValue;
use anyhow::{Context, Result, bail};

pub(crate) struct BinaryReader<'a> {
    pub(crate) bytes: &'a [u8],
    pub(crate) offset: usize,
}

impl<'a> BinaryReader<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    pub(crate) fn reached_end(&self) -> bool {
        self.offset == self.bytes.len()
    }

    pub(crate) fn read_byte(&mut self) -> Result<u8> {
        let byte = *self
            .bytes
            .get(self.offset)
            .with_context(|| format!("read past end at byte {}", self.offset))?;
        self.offset += 1;
        Ok(byte)
    }

    pub(crate) fn read_bytes_exact(&mut self, len: usize) -> Result<&'a [u8]> {
        // Lengths come directly from the file. Reject a corrupt or truncated
        // span before advancing so no sub-reader can point beyond the buffer.
        let remaining = self.bytes.len().saturating_sub(self.offset);
        if len > remaining {
            let offset = self.offset;
            self.offset = self.bytes.len();
            bail!("read {len} bytes past end at byte {offset}");
        }

        let end = self.offset + len;
        let bytes = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(bytes)
    }

    pub(crate) fn read_length_prefixed_bytes(&mut self) -> Result<&'a [u8]> {
        let len = usize::try_from(self.read_var_uint()?).context("length does not fit in usize")?;
        self.read_bytes_exact(len)
    }

    pub(crate) fn read_string(&mut self) -> Result<StringValue> {
        let bytes = self.read_length_prefixed_bytes()?;
        let raw = bytes.to_vec();
        let value = String::from_utf8(raw.clone()).ok();
        Ok(StringValue { value, raw })
    }

    pub(crate) fn read_f32(&mut self) -> Result<f32> {
        let bytes: [u8; 4] = self.read_bytes_exact(4)?.try_into().unwrap();
        Ok(f32::from_le_bytes(bytes))
    }

    pub(crate) fn read_u32(&mut self) -> Result<u32> {
        let bytes: [u8; 4] = self.read_bytes_exact(4)?.try_into().unwrap();
        Ok(u32::from_le_bytes(bytes))
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
