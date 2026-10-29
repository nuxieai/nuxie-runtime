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
pub struct DSIndexRangeChunker<'a> {
    firstRange: *const ElementRange,
    endRange: *const ElementRange,
    maxPatchesPerDraw: u32,
    indexCountPerPatch: u32,
    patchStrideLog2: u32,
    vertexFlags: i32,
    passFlags: [i32; 2],
    passCount: u32,
    lifetime: core::marker::PhantomData<&'a ElementRange>,
}
#[derive(Clone, Copy, Debug)]
pub struct DSIndexRangeIterator<'a> {
    chunker: DSIndexRangeChunker<'a>,
    chunkerIdentity: *const (),
    range: *const ElementRange,
    pass: u32,
    basePatch: u32,
}
impl DSIndexRangeIterator<'_> {
    fn skipEmptyRanges(&mut self) {
        while self.range != self.chunker.endRange && unsafe { (*self.range).count } == 0 {
            self.range = unsafe { (*self.range).next }.map_or(core::ptr::null(), |p| p.as_ptr());
        }
    }
    fn chunkPatchCount(&self) -> u32 {
        unsafe { (*self.range).count }
            .wrapping_sub(self.basePatch)
            .min(self.chunker.maxPatchesPerDraw)
    }
    pub fn value(&self) -> Draw {
        assert!(
            self.range != self.chunker.endRange,
            "cannot dereference the end iterator"
        );
        Draw {
            indexCount: self
                .chunkPatchCount()
                .wrapping_mul(self.chunker.indexCountPerPatch),
            baseVertex: ((unsafe { (*self.range).base } as i32)
                | self.chunker.vertexFlags
                | self.chunker.passFlags[self.pass as usize])
                .wrapping_add((self.basePatch as i32) << self.chunker.patchStrideLog2),
        }
    }
    pub fn advance(&mut self) {
        assert!(
            self.range != self.chunker.endRange,
            "cannot advance the end iterator"
        );
        self.basePatch += self.chunkPatchCount();
        if self.basePatch == unsafe { (*self.range).count } {
            self.basePatch = 0;
            self.pass += 1;
            if self.pass == self.chunker.passCount {
                self.pass = 0;
                self.range =
                    unsafe { (*self.range).next }.map_or(core::ptr::null(), |p| p.as_ptr());
                self.skipEmptyRanges();
            }
        }
    }
}
impl PartialEq for DSIndexRangeIterator<'_> {
    fn eq(&self, other: &Self) -> bool {
        debug_assert_eq!(self.chunkerIdentity, other.chunkerIdentity);
        self.range == other.range && self.pass == other.pass && self.basePatch == other.basePatch
    }
}
impl Eq for DSIndexRangeIterator<'_> {}
impl Iterator for DSIndexRangeIterator<'_> {
    type Item = Draw;
    fn next(&mut self) -> Option<Draw> {
        if self.range == self.chunker.endRange {
            return None;
        }
        let value = self.value();
        self.advance();
        Some(value)
    }
}
impl<'a> DSIndexRangeChunker<'a> {
    /// # Safety
    /// The batch's linked ranges must remain valid and immutable for 'a,
    /// and the chain must terminate at null.
    pub unsafe fn drawsForBatch(batch: &'a DrawBatch) -> Self {
        debug_assert!(!drawTypeHasPipelineDynamicState(batch.drawType));
        Self::new(batch, &batch.elementRange, core::ptr::null(), 0)
    }
    /// # Safety
    /// The supplied range and its linked end pointer must belong to the batch's
    /// valid, immutable range chain, whose nodes remain live for 'a.
    pub unsafe fn drawsForDynamicFillSubpass(
        batch: &'a DrawBatch,
        range: &'a ElementRange,
        state: &PipelineState,
    ) -> Self {
        debug_assert!(drawTypeHasPipelineDynamicState(batch.drawType));
        Self::new(
            batch,
            range,
            range.next.map_or(core::ptr::null(), |p| p.as_ptr()),
            if state.colorWriteEnabled {
                0
            } else {
                DSVertexFlag_DisableColorWrite
            },
        )
    }
    fn new(
        batch: &'a DrawBatch,
        firstRange: &'a ElementRange,
        endRange: *const ElementRange,
        vertexFlags: i32,
    ) -> Self {
        debug_assert!(
            batch.elementRange.next.is_none()
                || drawTypeIsDepthStencilPolarStroke(batch.drawType)
                || batch.drawType == DrawType::depthAAOuterHairline
                || drawTypeHasPipelineDynamicState(batch.drawType)
        );
        let stride = dsPatchStrideLog2(batch.drawType);
        #[cfg(debug_assertions)]
        for range in unsafe { super::next_ptr_range_hpp::NextPtrRange::new(firstRange, endRange) } {
            debug_assert!(
                (range.base & ((1 << DSVertexFlagsShift) - 1)) + (range.count << stride)
                    <= 1 << DSVertexFlagsShift
            );
        }
        let aa = drawTypeIsDepthAAStroke(batch.drawType);
        Self {
            firstRange,
            endRange,
            maxPatchesPerDraw: dsPatchMaxReps(batch.drawType),
            indexCountPerPatch: batch.indexCountPerInstance,
            patchStrideLog2: stride,
            vertexFlags,
            passFlags: if aa {
                [
                    DSVertexFlag_StrokeDepthPass | DSVertexFlag_DisableColorWrite,
                    0,
                ]
            } else {
                [0, 0]
            },
            passCount: if aa { 2 } else { 1 },
            lifetime: core::marker::PhantomData,
        }
    }
    pub fn begin(&self) -> DSIndexRangeIterator<'a> {
        let mut it = DSIndexRangeIterator {
            chunker: *self,
            chunkerIdentity: self as *const Self as *const (),
            range: self.firstRange,
            pass: 0,
            basePatch: 0,
        };
        it.skipEmptyRanges();
        it
    }
    pub fn end(&self) -> DSIndexRangeIterator<'a> {
        DSIndexRangeIterator {
            chunker: *self,
            chunkerIdentity: self as *const Self as *const (),
            range: self.endRange,
            pass: 0,
            basePatch: 0,
        }
    }
}
impl<'a> IntoIterator for DSIndexRangeChunker<'a> {
    type Item = Draw;
    type IntoIter = DSIndexRangeIterator<'a>;
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
            DrawType::stencilMidpointFans | DrawType::stencilDynamicMidpointFans => {
                DSMidpointFanFillPatchIndexCount
            }
            DrawType::stencilOuterCubics | DrawType::stencilDynamicOuterCubics => {
                DSOuterCubicFillPatchIndexCount
            }
            DrawType::depthStrokes | DrawType::depthAAOuterHairline => DSStrokePatchIndexCount,
            DrawType::depthAAStrokes => DSAAStrokePatchIndexCount,
            _ => unreachable!(),
        }
    }
    fn patch_flags(ty: DrawType) -> i32 {
        match ty {
            DrawType::stencilOuterCubics | DrawType::stencilDynamicOuterCubics => {
                DSVertexFlag_OuterCubicFill
            }
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
            let batch = make_batch(ty, 3, 7);
            // SAFETY: the retained local batch and linked ranges stay live and immutable through iteration.
            let c = unsafe { DSIndexRangeChunker::drawsForBatch(&batch) };
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
            let batch = make_batch(ty, 1, 0);
            // SAFETY: the retained local batch and linked ranges stay live and immutable through iteration.
            let c = unsafe { DSIndexRangeChunker::drawsForBatch(&batch) };
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
            let batch = make_batch(ty, 0, 4);
            // SAFETY: the retained local batch and linked ranges stay live and immutable through iteration.
            let c = unsafe { DSIndexRangeChunker::drawsForBatch(&batch) };
            assert_eq!(c.begin(), c.end());
        }
    }
    #[test]
    fn splits_at_rep_count() {
        for ty in DS_DRAW_TYPES {
            let max = dsPatchMaxReps(ty);
            let batch = make_batch(ty, max + 13, 0);
            // SAFETY: the retained local batch and linked ranges stay live and immutable through iteration.
            let c = unsafe { DSIndexRangeChunker::drawsForBatch(&batch) };
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
    fn dynamic_fill_subpasses_carry_color_write_flag() {
        for ty in [
            DrawType::stencilDynamicMidpointFans,
            DrawType::stencilDynamicOuterCubics,
        ] {
            let batch = make_batch(ty, 2, 5);
            for enabled in [false, true] {
                let state = PipelineState {
                    colorWriteEnabled: enabled,
                    ..PipelineState::new()
                };
                // SAFETY: this local batch owns its only range for the entire subpass traversal.
                let c = unsafe {
                    DSIndexRangeChunker::drawsForDynamicFillSubpass(
                        &batch,
                        &batch.elementRange,
                        &state,
                    )
                };
                let mut it = c.begin();
                assert_ne!(it, c.end());
                check_draw(
                    it.value(),
                    ty,
                    2,
                    5,
                    if enabled {
                        0
                    } else {
                        DSVertexFlag_DisableColorWrite
                    },
                );
                it.advance();
                assert_eq!(it, c.end());
            }
        }
    }
    #[test]
    fn aa_strokes_draw_depth_before_color() {
        let batch = make_batch(DrawType::depthAAStrokes, 4, 2);
        // SAFETY: the retained local batch and linked ranges stay live and immutable through iteration.
        let c = unsafe { DSIndexRangeChunker::drawsForBatch(&batch) };
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
            let batch = make_batch(ty, count, 0);
            // SAFETY: batch stays live and immutable throughout the loop.
            for draw in unsafe { DSIndexRangeChunker::drawsForBatch(&batch) } {
                total += draw.indexCount;
                draws += 1;
            }
            assert_eq!(draws, 3 * pass_count);
            assert_eq!(total, pass_count * count * patch_index_count(ty));
        }
    }

    fn make_range(ty: DrawType, count: u32, first: u32) -> ElementRange {
        ElementRange::new(
            count,
            (first << dsPatchStrideLog2(ty)) | patch_flags(ty) as u32,
            None,
        )
    }

    #[test]
    fn each_range_runs_every_pass_before_next_range() {
        for ty in [DrawType::depthStrokes, DrawType::depthAAStrokes] {
            let max = dsPatchMaxReps(ty);
            let spec = [(0, 1), (3, 10), (0, 50), (max + 5, 100), (2, 7), (0, 9)];
            let mut batch = make_batch(ty, spec[0].0, spec[0].1);
            let mut ranges: Vec<_> = spec[1..]
                .iter()
                .map(|&(count, first)| make_range(ty, count, first))
                .collect();
            // Allocation is fixed before linking; all ranges outlive the chunker.
            for i in 0..ranges.len() - 1 {
                ranges[i].next = Some(core::ptr::NonNull::from(&mut ranges[i + 1]));
            }
            batch.elementRange.next = Some(core::ptr::NonNull::from(&mut ranges[0]));
            // SAFETY: the retained local batch and linked ranges stay live and immutable through iteration.
            let c = unsafe { DSIndexRangeChunker::drawsForBatch(&batch) };
            let mut it = c.begin();
            for (count, first) in spec {
                for flags in pass_flags(ty) {
                    let mut done = 0;
                    while done < count {
                        let chunk = (count - done).min(max);
                        assert_ne!(it, c.end());
                        check_draw(it.value(), ty, chunk, first + done, flags);
                        it.advance();
                        done += chunk;
                    }
                }
            }
            assert_eq!(it, c.end());
        }
    }

    #[test]
    fn dynamic_fill_subpass_stops_at_next_range() {
        for ty in [
            DrawType::stencilDynamicMidpointFans,
            DrawType::stencilDynamicOuterCubics,
        ] {
            let max = dsPatchMaxReps(ty);
            let mut batch = make_batch(ty, 2, 5);
            let mut second = make_range(ty, max + 1, 40);
            batch.elementRange.next = Some(core::ptr::NonNull::from(&mut second));
            let state = PipelineState::new();
            // SAFETY: batch and second remain live and immutable; the first subpass ends before second.
            let first = unsafe {
                DSIndexRangeChunker::drawsForDynamicFillSubpass(&batch, &batch.elementRange, &state)
            };
            let mut it = first.begin();
            assert_ne!(it, first.end());
            check_draw(it.value(), ty, 2, 5, 0);
            it.advance();
            assert_eq!(it, first.end());
            // SAFETY: second is a live local terminal range retained throughout iteration.
            let rest =
                unsafe { DSIndexRangeChunker::drawsForDynamicFillSubpass(&batch, &second, &state) };
            let mut it = rest.begin();
            assert_ne!(it, rest.end());
            check_draw(it.value(), ty, max, 40, 0);
            it.advance();
            assert_ne!(it, rest.end());
            check_draw(it.value(), ty, 1, 40 + max, 0);
            it.advance();
            assert_eq!(it, rest.end());
        }
    }
}
