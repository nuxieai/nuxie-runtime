//! renderer/include/rive/renderer/range_chunker.hpp at c14cb2510071bd4cfa08d52ba5cd44d98c362237.
use super::gpu_hpp::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chunk {
    pub count: u32,
    pub first: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct RangeIterator {
    end: u32,
    maxPerDrawCommand: u32,
    current: u32,
}
impl RangeIterator {
    pub fn currentChunkCount(&self) -> u32 {
        self.end
            .wrapping_sub(self.current)
            .min(self.maxPerDrawCommand)
    }
    pub fn value(&self) -> Chunk {
        Chunk {
            count: self.currentChunkCount(),
            first: self.current,
        }
    }
    pub fn advance(&mut self) {
        self.current = self.current.wrapping_add(self.currentChunkCount());
    }
}
impl PartialEq for RangeIterator {
    fn eq(&self, other: &Self) -> bool {
        debug_assert_eq!(self.end, other.end);
        debug_assert_eq!(self.maxPerDrawCommand, other.maxPerDrawCommand);
        self.current == other.current
    }
}
impl Eq for RangeIterator {}
impl Iterator for RangeIterator {
    type Item = Chunk;
    fn next(&mut self) -> Option<Chunk> {
        if self.current == self.end {
            return None;
        }
        let value = self.value();
        self.advance();
        Some(value)
    }
}
pub struct RangeChunker {
    first: u32,
    end: u32,
    maxPerDrawCommand: u32,
}
impl RangeChunker {
    pub fn new(count: u32, first: u32, maxPerDrawCommand: u32) -> Self {
        Self {
            first,
            end: first.wrapping_add(count),
            maxPerDrawCommand,
        }
    }
    pub fn begin(&self) -> RangeIterator {
        RangeIterator {
            end: self.end,
            maxPerDrawCommand: self.maxPerDrawCommand,
            current: self.first,
        }
    }
    pub fn end(&self) -> RangeIterator {
        RangeIterator {
            end: self.end,
            maxPerDrawCommand: self.maxPerDrawCommand,
            current: self.end,
        }
    }
}
impl IntoIterator for RangeChunker {
    type Item = Chunk;
    type IntoIter = RangeIterator;
    fn into_iter(self) -> Self::IntoIter {
        self.begin()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Draw {
    pub indexCount: u32,
    pub baseVertex: i32,
}
#[derive(Clone, Copy, Debug)]
pub struct DSIndexRangeIterator {
    indexCountPerPatch: u32,
    patchStrideLog2: u32,
    vertexFlags: i32,
    chunk: RangeIterator,
}
impl DSIndexRangeIterator {
    pub fn value(&self) -> Draw {
        let Chunk { count, first } = self.chunk.value();
        Draw {
            indexCount: count.wrapping_mul(self.indexCountPerPatch),
            baseVertex: ((first as i32) << self.patchStrideLog2) | self.vertexFlags,
        }
    }
    pub fn advance(&mut self) {
        self.chunk.advance();
    }
}
impl PartialEq for DSIndexRangeIterator {
    fn eq(&self, other: &Self) -> bool {
        debug_assert_eq!(self.indexCountPerPatch, other.indexCountPerPatch);
        debug_assert_eq!(self.patchStrideLog2, other.patchStrideLog2);
        debug_assert_eq!(self.vertexFlags, other.vertexFlags);
        self.chunk == other.chunk
    }
}
impl Eq for DSIndexRangeIterator {}
impl Iterator for DSIndexRangeIterator {
    type Item = Draw;
    fn next(&mut self) -> Option<Draw> {
        if self.chunk.current == self.chunk.end {
            return None;
        }
        let value = self.value();
        self.advance();
        Some(value)
    }
}
pub struct DSIndexRangeChunker {
    chunker: RangeChunker,
    indexCountPerPatch: u32,
    patchStrideLog2: u32,
    vertexFlags: i32,
}
impl DSIndexRangeChunker {
    pub fn new(drawType: DrawType, patchCount: u32, firstPatch: u32, vertexFlags: i32) -> Self {
        let outer = drawTypeSubmitsOuterCubicPatches(drawType);
        let stride = if outer {
            DSOuterCubicFillPatchStrideLog2
        } else {
            DSMidpointFanFillPatchStrideLog2
        };
        debug_assert!(
            (firstPatch.wrapping_add(patchCount) << stride) <= (1u32 << DSFillVertexFlagsShift)
        );
        Self {
            chunker: RangeChunker::new(patchCount, firstPatch, dsFillPatchMaxReps(outer)),
            indexCountPerPatch: dsFillPatchIndexCount(outer),
            patchStrideLog2: stride,
            vertexFlags: if outer {
                vertexFlags | DSFillVertexFlagOuterCubic
            } else {
                vertexFlags
            },
        }
    }
    pub fn begin(&self) -> DSIndexRangeIterator {
        DSIndexRangeIterator {
            indexCountPerPatch: self.indexCountPerPatch,
            patchStrideLog2: self.patchStrideLog2,
            vertexFlags: self.vertexFlags,
            chunk: self.chunker.begin(),
        }
    }
    pub fn end(&self) -> DSIndexRangeIterator {
        DSIndexRangeIterator {
            indexCountPerPatch: self.indexCountPerPatch,
            patchStrideLog2: self.patchStrideLog2,
            vertexFlags: self.vertexFlags,
            chunk: self.chunker.end(),
        }
    }
}
impl IntoIterator for DSIndexRangeChunker {
    type Item = Draw;
    type IntoIter = DSIndexRangeIterator;
    fn into_iter(self) -> Self::IntoIter {
        self.begin()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanical_port::source::renderer::src::shaders::constants_glsl::{
        DS_PATCH_STRIDE_LOG2, VERTEX_FLAG_DISABLE_COLOR_WRITE, VERTEX_FLAG_OUTER_CUBIC,
    };
    #[test]
    fn max_two_odd() {
        let c = RangeChunker::new(5, 0, 2);
        let mut it = c.begin();
        for (first, count) in [(0, 2), (2, 2), (4, 1)] {
            assert_eq!(it.value(), Chunk { count, first });
            assert_ne!(it, c.end());
            it.advance();
        }
        for _ in 0..10 {
            assert_eq!(it.value(), Chunk { count: 0, first: 5 });
            assert_eq!(it, c.end());
            it.advance();
        }
    }
    #[test]
    fn max_two_even() {
        let c = RangeChunker::new(6, 0, 2);
        let mut it = c.begin();
        for (first, count) in [(0, 2), (2, 2), (4, 2)] {
            assert_eq!(it.value(), Chunk { count, first });
            assert_ne!(it, c.end());
            it.advance();
        }
        for _ in 0..10 {
            assert_eq!(it.value(), Chunk { count: 0, first: 6 });
            assert_eq!(it, c.end());
            it.advance();
        }
    }
    #[test]
    fn max_uint32() {
        let c = RangeChunker::new(5, 0, u32::MAX);
        let mut it = c.begin();
        assert_eq!(it.value(), Chunk { count: 5, first: 0 });
        assert_ne!(it, c.end());
        for _ in 0..10 {
            it.advance();
            assert_eq!(it.value(), Chunk { count: 0, first: 5 });
            assert_eq!(it, c.end());
        }
    }
    #[test]
    fn max_zero() {
        let c = RangeChunker::new(5, 0, 0);
        let mut it = c.begin();
        for _ in 0..10 {
            assert_ne!(it, c.end());
            assert_eq!(it.value(), Chunk { count: 0, first: 0 });
            it.advance();
        }
    }
    #[test]
    fn empty() {
        let c = RangeChunker::new(0, 0, 5);
        let mut it = c.begin();
        for _ in 0..10 {
            assert_eq!(it, c.end());
            it.advance();
        }
    }
    #[test]
    fn empty_max_zero() {
        let c = RangeChunker::new(0, 0, 0);
        let mut it = c.begin();
        for _ in 0..10 {
            assert_eq!(it, c.end());
            it.advance();
        }
    }
    fn check_draw(draw: Draw, outer: bool, count: u32, first: u32, flags: i32) {
        assert_eq!(draw.indexCount, count * dsFillPatchIndexCount(outer));
        assert_eq!(
            draw.baseVertex,
            ((first as i32) << DS_PATCH_STRIDE_LOG2(outer))
                | flags
                | if outer { VERTEX_FLAG_OUTER_CUBIC } else { 0 }
        );
    }
    fn draw_type(outer: bool) -> DrawType {
        if outer {
            DrawType::stencilOuterCubics
        } else {
            DrawType::stencilMidpointFans
        }
    }
    #[test]
    fn one_chunk() {
        for outer in [false, true] {
            let c = DSIndexRangeChunker::new(draw_type(outer), 3, 7, 0);
            let mut it = c.begin();
            assert_ne!(it, c.end());
            check_draw(it.value(), outer, 3, 7, 0);
            it.advance();
            assert_eq!(it, c.end());
        }
    }
    #[test]
    fn base_patch_zero() {
        for outer in [false, true] {
            let c = DSIndexRangeChunker::new(draw_type(outer), 1, 0, 0);
            let it = c.begin();
            assert_ne!(it, c.end());
            assert_eq!(
                it.value().baseVertex,
                if outer { VERTEX_FLAG_OUTER_CUBIC } else { 0 }
            );
            check_draw(it.value(), outer, 1, 0, 0);
        }
    }
    #[test]
    fn empty_batch() {
        for outer in [false, true] {
            let c = DSIndexRangeChunker::new(draw_type(outer), 0, 4, 0);
            assert_eq!(c.begin(), c.end());
        }
    }
    #[test]
    fn splits_at_rep_count() {
        for outer in [false, true] {
            let max = dsFillPatchMaxReps(outer);
            let c = DSIndexRangeChunker::new(draw_type(outer), max + 13, 0, 0);
            let mut it = c.begin();
            assert_ne!(it, c.end());
            check_draw(it.value(), outer, max, 0, 0);
            it.advance();
            assert_ne!(it, c.end());
            check_draw(it.value(), outer, 13, max, 0);
            it.advance();
            assert_eq!(it, c.end());
        }
    }
    #[test]
    fn carries_color_write_flag() {
        for outer in [false, true] {
            let c =
                DSIndexRangeChunker::new(draw_type(outer), 2, 5, VERTEX_FLAG_DISABLE_COLOR_WRITE);
            let it = c.begin();
            assert_ne!(it, c.end());
            check_draw(it.value(), outer, 2, 5, VERTEX_FLAG_DISABLE_COLOR_WRITE);
            assert_ne!(it.value().baseVertex & VERTEX_FLAG_DISABLE_COLOR_WRITE, 0);
        }
    }
    #[test]
    fn covers_every_patch_once() {
        for outer in [false, true] {
            let count = dsFillPatchMaxReps(outer) * 2 + 3;
            let mut total = 0;
            let mut draws = 0;
            for draw in DSIndexRangeChunker::new(draw_type(outer), count, 0, 0) {
                total += draw.indexCount;
                draws += 1;
            }
            assert_eq!(draws, 3);
            assert_eq!(total, count * dsFillPatchIndexCount(outer));
        }
    }
}
