use std::ops::Index;

/// Stores the glyph index representing the code point at each index.
#[derive(Debug, Clone, Default)]
pub(crate) struct GlyphLookup {
    glyph_indices: Vec<u32>,
}

impl GlyphLookup {
    /// Rust shaping retains the flattened `GlyphRun::textIndices` sequence on
    /// each styled glyph, so the caller supplies that sequence directly.
    pub(crate) fn compute(&mut self, code_point_count: usize, text_indices: &[u32]) {
        self.glyph_indices.resize(code_point_count + 1, 0);

        let mut glyph_index = 0u32;
        let mut last_text_index = 0u32;
        for &text_index in text_indices {
            for index in last_text_index..text_index {
                assert!(glyph_index != 0);
                self.glyph_indices[index as usize] = glyph_index.wrapping_sub(1);
            }
            last_text_index = text_index;
            glyph_index = glyph_index.wrapping_add(1);
        }
        for index in last_text_index as usize..code_point_count {
            self.glyph_indices[index] = glyph_index.wrapping_sub(1);
        }

        self.glyph_indices[code_point_count] = if code_point_count == 0 {
            0
        } else {
            self.glyph_indices[code_point_count - 1].wrapping_add(1)
        };
    }

    pub(crate) fn count(&self, mut index: u32) -> u32 {
        assert!(index < self.glyph_indices.len() as u32);

        let value = self.glyph_indices[index as usize];
        let mut count = 1u32;
        let size = self.glyph_indices.len() as u32;
        index = index.wrapping_add(1);
        while index < size && self.glyph_indices[index as usize] == value {
            count = count.wrapping_add(1);
            index = index.wrapping_add(1);
        }
        count
    }

    pub(crate) fn glyph_start(&self, mut index: u32) -> u32 {
        if index == 0 || index >= self.glyph_indices.len() as u32 {
            return index;
        }
        let value = self.glyph_indices[index as usize];
        while index > 0 && self.glyph_indices[index as usize - 1] == value {
            index -= 1;
        }
        index
    }

    pub(crate) fn is_glyph_boundary(&self, index: u32) -> bool {
        if index == 0 || index >= self.glyph_indices.len() as u32 {
            return true;
        }
        self.glyph_indices[index as usize] != self.glyph_indices[index as usize - 1]
    }

    pub(crate) fn size(&self) -> usize {
        self.glyph_indices.len()
    }
}

impl Index<u32> for GlyphLookup {
    type Output = u32;

    fn index(&self, code_point_index: u32) -> &Self::Output {
        assert!(code_point_index < self.glyph_indices.len() as u32);
        &self.glyph_indices[code_point_index as usize]
    }
}

impl GlyphLookup {
    pub(crate) fn last_code_point_index(&self) -> u32 {
        if self.glyph_indices.is_empty() {
            0
        } else {
            (self.glyph_indices.len() - 1) as u32
        }
    }

    pub(crate) fn empty(&self) -> bool {
        self.glyph_indices.is_empty()
    }

    pub(crate) fn clear(&mut self) {
        self.glyph_indices.clear();
    }

    pub(crate) fn advance_factor(&self, code_point_index: i32, inv: bool) -> f32 {
        if code_point_index < 0 || code_point_index >= self.glyph_indices.len() as i32 {
            return 0.0;
        }
        let glyph_index = self[code_point_index as u32];
        let mut start = code_point_index;
        while start > 0 {
            if self[(start - 1) as u32] != glyph_index {
                break;
            }
            start -= 1;
        }
        let mut end = code_point_index;
        while (end as usize) < self.glyph_indices.len() - 1 {
            if self[(end + 1) as u32] != glyph_index {
                break;
            }
            end += 1;
        }

        let factor = (code_point_index - start) as f32 / (end - start + 1) as f32;
        if inv { 1.0 - factor } else { factor }
    }
}
