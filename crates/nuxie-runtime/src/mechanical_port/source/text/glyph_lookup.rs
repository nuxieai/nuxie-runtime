use crate::mechanical_port::source::text_engine::Paragraph;
#[derive(Default)]
pub struct GlyphLookup {
    glyph_indices: Vec<u32>,
}
impl GlyphLookup {
    pub fn compute(&mut self, text: &[u32], shape: &[Paragraph]) {
        let count = text.len();
        self.glyph_indices.resize(count + 1, 0);
        let mut glyph_index = 0;
        let mut last_text_index = 0;
        for paragraph in shape {
            for run in &paragraph.runs {
                for i in 0..run.glyphs.len() {
                    let text_index = run.text_indices[i];
                    for j in last_text_index..text_index {
                        assert_ne!(glyph_index, 0);
                        self.glyph_indices[j as usize] = glyph_index - 1;
                    }
                    last_text_index = text_index;
                    glyph_index += 1;
                }
            }
        }
        for i in last_text_index as usize..count {
            self.glyph_indices[i] = glyph_index - 1;
        }
        self.glyph_indices[count] = if count == 0 {
            0
        } else {
            self.glyph_indices[count - 1] + 1
        };
    }
    pub fn count(&self, mut index: u32) -> u32 {
        assert!((index as usize) < self.glyph_indices.len());
        let value = self.glyph_indices[index as usize];
        let mut count = 1;
        index += 1;
        while (index as usize) < self.glyph_indices.len()
            && self.glyph_indices[index as usize] == value
        {
            count += 1;
            index += 1;
        }
        count
    }
    pub fn glyph_start(&self, mut index: u32) -> u32 {
        if index == 0 || index as usize >= self.glyph_indices.len() {
            return index;
        }
        let value = self.glyph_indices[index as usize];
        while index > 0 && self.glyph_indices[index as usize - 1] == value {
            index -= 1;
        }
        index
    }
    pub fn is_glyph_boundary(&self, index: u32) -> bool {
        index == 0
            || index as usize >= self.glyph_indices.len()
            || self.glyph_indices[index as usize] != self.glyph_indices[index as usize - 1]
    }
    pub fn size(&self) -> usize {
        self.glyph_indices.len()
    }
    pub fn get(&self, index: u32) -> u32 {
        assert!((index as usize) < self.glyph_indices.len());
        self.glyph_indices[index as usize]
    }
    pub fn last_code_point_index(&self) -> u32 {
        self.glyph_indices.len().saturating_sub(1) as u32
    }
    pub fn empty(&self) -> bool {
        self.glyph_indices.is_empty()
    }
    pub fn clear(&mut self) {
        self.glyph_indices.clear();
    }
    pub fn advance_factor(&self, index: i32, inverse: bool) -> f32 {
        if index < 0 || index as usize >= self.glyph_indices.len() {
            return 0.0;
        }
        let glyph = self.glyph_indices[index as usize];
        let mut start = index;
        while start > 0 && self.glyph_indices[start as usize - 1] == glyph {
            start -= 1;
        }
        let mut end = index;
        while (end as usize) < self.glyph_indices.len() - 1
            && self.glyph_indices[end as usize + 1] == glyph
        {
            end += 1;
        }
        let factor = (index - start) as f32 / (end - start + 1) as f32;
        if inverse { 1.0 - factor } else { factor }
    }
}
