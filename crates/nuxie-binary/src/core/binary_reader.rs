use crate::StringValue;
use anyhow::{Context, Result};

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
        let end = self
            .offset
            .checked_add(len)
            .context("byte offset overflow")?;
        let bytes = self
            .bytes
            .get(self.offset..end)
            .with_context(|| format!("read {len} bytes past end at byte {}", self.offset))?;
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
