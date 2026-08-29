#[derive(Clone, Copy, Debug)]
pub struct BitFieldLoc {
    start: u32,
    count: u32,
    mask: u32,
}

impl BitFieldLoc {
    pub fn new(start: u32, end: u32) -> Self {
        assert!(end >= start);
        assert!(end < 32);
        let count = end - start + 1;
        let mask = ((1u32 << count) - 1) << start;
        Self { start, count, mask }
    }

    pub fn read(self, bits: u32) -> u32 {
        (bits & self.mask) >> self.start
    }

    pub fn write(self, bits: u32, value: u32) -> u32 {
        (bits & !self.mask) | ((value << self.start) & self.mask)
    }

    pub fn count(self) -> u32 {
        self.count
    }
}
