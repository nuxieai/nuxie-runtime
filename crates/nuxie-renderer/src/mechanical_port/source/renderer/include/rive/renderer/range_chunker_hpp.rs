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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DSIndexRangeChunker {
    patchCount: u32,
    maxPatchesPerDraw: u32,
    indexCountPerPatch: u32,
    patchStrideLog2: u32,
    baseVertex: i32,
    passFlags: [i32; 2],
    passCount: u32,
}
#[derive(Clone, Copy, Debug)]
pub struct DSIndexRangeIterator {
    chunker: DSIndexRangeChunker,
    pass: u32,
    basePatch: u32,
}
impl DSIndexRangeIterator {
    fn chunkPatchCount(&self) -> u32 {
        self.chunker
            .patchCount
            .wrapping_sub(self.basePatch)
            .min(self.chunker.maxPatchesPerDraw)
    }
    pub fn value(&self) -> Draw {
        Draw {
            indexCount: self
                .chunkPatchCount()
                .wrapping_mul(self.chunker.indexCountPerPatch),
            baseVertex: (self.chunker.baseVertex | self.chunker.passFlags[self.pass as usize])
                .wrapping_add((self.basePatch as i32) << self.chunker.patchStrideLog2),
        }
    }
    pub fn advance(&mut self) {
        self.basePatch += self.chunkPatchCount();
        if self.basePatch == self.chunker.patchCount {
            self.pass += 1;
            self.basePatch = 0;
        }
    }
}
impl PartialEq for DSIndexRangeIterator {
    fn eq(&self, other: &Self) -> bool {
        debug_assert_eq!(self.chunker, other.chunker);
        self.pass == other.pass && self.basePatch == other.basePatch
    }
}
impl Eq for DSIndexRangeIterator {}
impl Iterator for DSIndexRangeIterator {
    type Item = Draw;
    fn next(&mut self) -> Option<Draw> {
        if self.pass == self.chunker.passCount {
            return None;
        }
        let value = self.value();
        self.advance();
        Some(value)
    }
}
impl DSIndexRangeChunker {
    pub fn new(batch: &DrawBatch, vertexFlags: i32) -> Self {
        let stride = dsPatchStrideLog2(batch.drawType);
        debug_assert!(
            (batch.baseElement & ((1 << DSVertexFlagsShift) - 1)) + (batch.elementCount << stride)
                <= 1 << DSVertexFlagsShift
        );
        let aa = drawTypeIsDepthAAStroke(batch.drawType);
        Self {
            patchCount: batch.elementCount,
            maxPatchesPerDraw: dsPatchMaxReps(batch.drawType),
            indexCountPerPatch: batch.indexCountPerInstance,
            patchStrideLog2: stride,
            baseVertex: batch.baseElement as i32 | vertexFlags,
            passFlags: if aa {
                [
                    DSVertexFlag_StrokeDepthPass | DSVertexFlag_DisableColorWrite,
                    0,
                ]
            } else {
                [0, 0]
            },
            passCount: if aa { 2 } else { 1 },
        }
    }
    pub fn begin(&self) -> DSIndexRangeIterator {
        if self.patchCount == 0 {
            self.end()
        } else {
            DSIndexRangeIterator {
                chunker: *self,
                pass: 0,
                basePatch: 0,
            }
        }
    }
    pub fn end(&self) -> DSIndexRangeIterator {
        DSIndexRangeIterator {
            chunker: *self,
            pass: self.passCount,
            basePatch: 0,
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
    const DS_DRAW_TYPES: [DrawType; 5] = [
        DrawType::stencilMidpointFans,
        DrawType::stencilOuterCubics,
        DrawType::depthStrokes,
        DrawType::depthAAStrokes,
        DrawType::depthAAOuterHairline,
    ];
    fn patch_index_count(ty: DrawType) -> u32 {
        match ty {
            DrawType::stencilMidpointFans => DSMidpointFanFillPatchIndexCount,
            DrawType::stencilOuterCubics => DSOuterCubicFillPatchIndexCount,
            DrawType::depthStrokes | DrawType::depthAAOuterHairline => DSStrokePatchIndexCount,
            DrawType::depthAAStrokes => DSAAStrokePatchIndexCount,
            _ => unreachable!(),
        }
    }
    fn patch_flags(ty: DrawType) -> i32 {
        match ty {
            DrawType::stencilOuterCubics => DSVertexFlag_OuterCubicFill,
            DrawType::depthAAStrokes => DSVertexFlag_AAPolarStroke,
            _ => 0,
        }
    }
    fn pass_flags(ty: DrawType) -> Vec<i32> {
        if matches!(
            ty,
            DrawType::depthAAStrokes | DrawType::depthAAOuterHairline
        ) {
            vec![
                DSVertexFlag_StrokeDepthPass | DSVertexFlag_DisableColorWrite,
                0,
            ]
        } else {
            vec![0]
        }
    }
    fn make_batch(ty: DrawType, count: u32, first: u32) -> DrawBatch {
        let mut batch = DrawBatch::new(
            ty,
            ShaderMiscFlags::none,
            DrawContents::none,
            count,
            (first << dsPatchStrideLog2(ty)) | patch_flags(ty) as u32,
            nuxie_render_api::BlendMode::SrcOver,
            ImageSampler::LinearClamp(),
            BarrierFlags::none,
        );
        batch.indexCountPerInstance = patch_index_count(ty);
        batch
    }
    fn check_draw(draw: Draw, ty: DrawType, count: u32, first: u32, flags: i32) {
        assert_eq!(draw.indexCount, count * patch_index_count(ty));
        assert_eq!(
            draw.baseVertex,
            ((first as i32) << dsPatchStrideLog2(ty)) | patch_flags(ty) | flags
        );
    }
    #[test]
    fn one_chunk() {
        for ty in DS_DRAW_TYPES {
            let c = DSIndexRangeChunker::new(&make_batch(ty, 3, 7), 0);
            let mut it = c.begin();
            for flags in pass_flags(ty) {
                assert_ne!(it, c.end());
                check_draw(it.value(), ty, 3, 7, flags);
                it.advance();
            }
            assert_eq!(it, c.end());
        }
    }
    #[test]
    fn base_patch_zero() {
        for ty in DS_DRAW_TYPES {
            let c = DSIndexRangeChunker::new(&make_batch(ty, 1, 0), 0);
            let mut it = c.begin();
            for flags in pass_flags(ty) {
                assert_ne!(it, c.end());
                assert_eq!(it.value().baseVertex, patch_flags(ty) | flags);
                check_draw(it.value(), ty, 1, 0, flags);
                it.advance();
            }
            assert_eq!(it, c.end());
        }
    }
    #[test]
    fn empty_batch() {
        for ty in DS_DRAW_TYPES {
            let c = DSIndexRangeChunker::new(&make_batch(ty, 0, 4), 0);
            assert_eq!(c.begin(), c.end());
        }
    }
    #[test]
    fn splits_at_rep_count() {
        for ty in DS_DRAW_TYPES {
            let max = dsPatchMaxReps(ty);
            let c = DSIndexRangeChunker::new(&make_batch(ty, max + 13, 0), 0);
            let mut it = c.begin();
            for flags in pass_flags(ty) {
                assert_ne!(it, c.end());
                check_draw(it.value(), ty, max, 0, flags);
                it.advance();
                assert_ne!(it, c.end());
                check_draw(it.value(), ty, 13, max, flags);
                it.advance();
            }
            assert_eq!(it, c.end());
        }
    }
    #[test]
    fn carries_color_write_flag() {
        for ty in DS_DRAW_TYPES {
            let c = DSIndexRangeChunker::new(&make_batch(ty, 2, 5), DSVertexFlag_DisableColorWrite);
            let mut it = c.begin();
            for flags in pass_flags(ty) {
                assert_ne!(it, c.end());
                check_draw(it.value(), ty, 2, 5, flags | DSVertexFlag_DisableColorWrite);
                assert_ne!(it.value().baseVertex & DSVertexFlag_DisableColorWrite, 0);
                it.advance();
            }
            assert_eq!(it, c.end());
        }
    }
    #[test]
    fn aa_strokes_draw_depth_before_color() {
        let c = DSIndexRangeChunker::new(&make_batch(DrawType::depthAAStrokes, 4, 2), 0);
        let mut it = c.begin();
        assert_ne!(it, c.end());
        assert_ne!(it.value().baseVertex & DSVertexFlag_StrokeDepthPass, 0);
        assert_ne!(it.value().baseVertex & DSVertexFlag_DisableColorWrite, 0);
        it.advance();
        assert_ne!(it, c.end());
        assert_eq!(it.value().baseVertex & DSVertexFlag_StrokeDepthPass, 0);
        assert_eq!(it.value().baseVertex & DSVertexFlag_DisableColorWrite, 0);
        it.advance();
        assert_eq!(it, c.end());
    }
    #[test]
    fn covers_every_patch_once_per_pass() {
        for ty in DS_DRAW_TYPES {
            let count = dsPatchMaxReps(ty) * 2 + 3;
            let pass_count = pass_flags(ty).len() as u32;
            let mut total = 0;
            let mut draws = 0;
            for draw in DSIndexRangeChunker::new(&make_batch(ty, count, 0), 0) {
                total += draw.indexCount;
                draws += 1;
            }
            assert_eq!(draws, 3 * pass_count);
            assert_eq!(total, pass_count * count * patch_index_count(ty));
        }
    }
}
