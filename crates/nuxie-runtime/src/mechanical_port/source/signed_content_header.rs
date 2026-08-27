pub const SIGNATURE_SIZE: usize = 64;

pub struct SignedContentHeader<'a> {
    data: &'a [u8],
    flags: u8,
}

impl<'a> SignedContentHeader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        let flags = data.first().copied().unwrap_or(0);
        Self { data, flags }
    }

    pub fn is_signed(&self) -> bool {
        self.flags & 0x80 != 0
    }

    pub fn version(&self) -> u8 {
        self.flags & 0x7f
    }

    pub fn is_valid(&self) -> bool {
        self.data.len() >= self.content_offset()
    }

    pub fn content_offset(&self) -> usize {
        if self.is_signed() {
            1 + SIGNATURE_SIZE
        } else {
            1
        }
    }

    pub fn signature(&self) -> &[u8] {
        if !self.is_signed() || self.data.len() < 1 + SIGNATURE_SIZE {
            return &[];
        }
        &self.data[1..1 + SIGNATURE_SIZE]
    }

    pub fn content(&self) -> &[u8] {
        let offset = self.content_offset();
        if self.data.len() < offset {
            return &[];
        }
        &self.data[offset..]
    }
}
