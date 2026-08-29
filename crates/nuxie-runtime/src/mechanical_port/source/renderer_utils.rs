pub struct AutoStArray<T, const N: usize> {
    values: Vec<T>,
}

impl<T: Default, const N: usize> AutoStArray<T, N> {
    pub fn new(count: usize) -> Self {
        let mut values = Vec::with_capacity(if count > N { count } else { N });
        values.resize_with(count, T::default);
        Self { values }
    }
}

impl<T, const N: usize> AutoStArray<T, N> {
    pub fn size(&self) -> usize {
        self.values.len()
    }

    pub fn count(&self) -> i32 {
        i32::try_from(self.values.len()).expect("AutoSTArray count must fit in i32")
    }

    pub fn data(&mut self) -> &mut [T] {
        &mut self.values
    }
}

impl<T, const N: usize> std::ops::Index<usize> for AutoStArray<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.values.len());
        &self.values[index]
    }
}

impl<T, const N: usize> std::ops::IndexMut<usize> for AutoStArray<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.values.len());
        &mut self.values[index]
    }
}

pub const fn make_tag(a: u8, b: u8, c: u8, d: u8) -> u32 {
    ((a as u32) << 24) | ((b as u32) << 16) | ((c as u32) << 8) | d as u32
}

pub fn tag_to_string(tag: u32) -> String {
    let bytes = [
        ((tag >> 24) & 0xff) as u8,
        ((tag >> 16) & 0xff) as u8,
        ((tag >> 8) & 0xff) as u8,
        (tag & 0xff) as u8,
    ];
    bytes.into_iter().map(char::from).collect()
}
