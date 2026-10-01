// Copyright (c) Meta Platforms, Inc. and affiliates.
// This source code is licensed under the MIT license found in the LICENSE
// file in the root directory of this source tree.
//! Rive's pinned Yoga measured-node cache, separate from generic CSS layout.
use crate::geometry::{Rect, Size};
use crate::tree::{LayoutOutput, RunMode};

/// Live runtime state needed before attempting completed-measurement reuse.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RiveMeasureMetadata {
    /// The layout owner's direction (zero denotes the root's inherited direction).
    pub owner_direction: u8,
    /// Runtime measurement-mode normalization flags, not measurement results.
    pub normalization: u8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MeasureGeometry {
    pub min: Size<Option<f32>>,
    pub max: Size<Option<f32>>,
    pub inset: Rect<f32>,
    pub padding: Rect<f32>,
    pub border: Rect<f32>,
    pub aspect: Option<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MeasureRequest {
    // Yoga enum order: Undefined, Exactly, AtMost.
    pub modes: Size<u8>,
    pub available: Size<f32>,
    pub margin: Size<f32>,
    pub geometry: MeasureGeometry,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Entry {
    request: MeasureRequest,
    output: LayoutOutput,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RiveMeasureCache {
    final_layout: Option<Entry>,
    measurements: [Option<Entry>; 8],
    len: usize,
    metadata: Option<RiveMeasureMetadata>,
    generation: u64,
    dirty: bool,
}

fn float_equal(a: f32, b: f32) -> bool {
    (a.is_nan() && b.is_nan()) || (a - b).abs() < 0.0001
}

fn compatible(
    mode: u8,
    available: f32,
    old_mode: u8,
    old_available: f32,
    computed: f32,
    margin: f32,
) -> bool {
    let inner = available - margin;
    (mode == old_mode && float_equal(available, old_available))
        || (mode == 1 && float_equal(inner, computed))
        || (mode == 2 && old_mode == 0 && (inner >= computed || float_equal(inner, computed)))
        || (mode == 2
            && old_mode == 2
            && old_available > inner
            && (computed <= inner || float_equal(inner, computed)))
}

impl RiveMeasureCache {
    pub const fn new() -> Self {
        Self {
            final_layout: None,
            measurements: [None; 8],
            len: 0,
            metadata: None,
            generation: 0,
            dirty: true,
        }
    }

    pub fn mark_dirty(&mut self) -> bool {
        let changed = !self.dirty;
        self.dirty = true;
        changed
    }

    pub fn clear(&mut self) {
        self.final_layout = None;
        self.len = 0;
        self.metadata = None;
        self.dirty = true;
    }

    pub fn invalidate_final(&mut self) {
        self.final_layout = None;
    }

    pub fn get(
        &mut self,
        request: MeasureRequest,
        metadata: RiveMeasureMetadata,
        generation: u64,
    ) -> Option<LayoutOutput> {
        // Yoga dirt raised again in the same calculation does not invalidate.
        if (self.dirty && self.generation != generation) || self.metadata != Some(metadata) {
            self.final_layout = None;
            self.len = 0;
        }
        self.metadata = Some(metadata);
        self.final_layout
            .iter()
            .chain(self.measurements[..self.len].iter().flatten())
            .find_map(|entry| {
                let old = entry.request;
                let size = entry.output.size;
                // These resolved box properties can differ under Taffy's sizing
                // modes without a style mutation. Parent-size identity alone is
                // deliberately not a key: only its resolved geometry matters.
                (request.geometry == old.geometry
                    && !(size.width < 0.0 || size.height < 0.0)
                    && compatible(
                        request.modes.width,
                        request.available.width,
                        old.modes.width,
                        old.available.width,
                        size.width,
                        request.margin.width,
                    )
                    && compatible(
                        request.modes.height,
                        request.available.height,
                        old.modes.height,
                        old.available.height,
                        size.height,
                        request.margin.height,
                    ))
                .then_some(entry.output)
            })
    }

    pub fn finish(
        &mut self,
        request: MeasureRequest,
        output: LayoutOutput,
        run: RunMode,
        generation: u64,
        hit: bool,
    ) {
        if !hit {
            if self.len == self.measurements.len() {
                self.len = 0;
            }
            let entry = Some(Entry { request, output });
            if run == RunMode::PerformLayout {
                self.final_layout = entry;
            } else {
                self.measurements[self.len] = entry;
                self.len += 1;
            }
        }
        if run == RunMode::PerformLayout {
            self.dirty = false;
        }
        self.generation = generation;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(width: f32) -> MeasureRequest {
        MeasureRequest {
            modes: Size {
                width: 2,
                height: 0,
            },
            available: Size {
                width,
                height: f32::NAN,
            },
            margin: Size::ZERO,
            geometry: MeasureGeometry {
                min: Size::NONE,
                max: Size::NONE,
                inset: Rect::ZERO,
                padding: Rect::ZERO,
                border: Rect::ZERO,
                aspect: None,
            },
        }
    }

    fn output(width: f32) -> LayoutOutput {
        LayoutOutput {
            size: Size {
                width,
                height: 10.0,
            },
            ..LayoutOutput::HIDDEN
        }
    }

    #[test]
    fn rive_cache_direction_normalization_and_generation() {
        let mut cache = RiveMeasureCache::new();
        let key = request(100.0);
        let a = RiveMeasureMetadata::default();
        assert!(cache.get(key, a, 1).is_none());
        cache.finish(key, output(10.0), RunMode::PerformLayout, 1, false);
        cache.mark_dirty();
        // Yoga only re-visits dirt once per generation, even if callbacks
        // mutate content again. It must re-visit on the next calculation.
        assert!(cache.get(key, a, 1).is_some());
        assert!(cache.get(key, a, 2).is_none());
        cache.finish(key, output(20.0), RunMode::PerformLayout, 2, false);
        assert!(cache.get(key, a, 3).is_some());
        let b = RiveMeasureMetadata {
            owner_direction: 1,
            ..a
        };
        assert!(cache.get(key, b, 3).is_none());
        assert!(cache.get(key, a, 3).is_none());
        cache.finish(key, output(20.0), RunMode::PerformLayout, 3, false);
        assert!(cache
            .get(
                key,
                RiveMeasureMetadata {
                    normalization: 1,
                    ..a
                },
                3
            )
            .is_none());
    }

    #[test]
    fn rive_cache_final_precedence_probe_invalidation_and_prefix_rollover() {
        let mut cache = RiveMeasureCache::new();
        let meta = RiveMeasureMetadata::default();
        let key = request(100.0);
        cache.get(key, meta, 1);
        cache.finish(key, output(10.0), RunMode::ComputeSize, 1, false);
        cache.finish(key, output(20.0), RunMode::PerformLayout, 1, false);
        assert_eq!(cache.get(key, meta, 1).unwrap().size.width, 20.0);
        cache.invalidate_final();
        assert_eq!(cache.get(key, meta, 1).unwrap().size.width, 10.0);
        for index in 1..8 {
            cache.finish(
                request(100.0 + index as f32),
                output(10.0),
                RunMode::ComputeSize,
                1,
                false,
            );
        }
        assert_eq!(cache.len, 8);
        cache.finish(
            request(200.0),
            output(10.0),
            RunMode::PerformLayout,
            1,
            false,
        );
        assert_eq!(cache.len, 0);
        cache.invalidate_final();
        assert!(cache.get(key, meta, 1).is_none());
    }

    #[test]
    fn rive_cache_compatibility_keeps_margin_and_float_equality_contract() {
        assert!(compatible(1, 12.0, 0, f32::NAN, 10.0, 2.0));
        assert!(!compatible(1, 12.0, 0, f32::NAN, 10.0, 0.0));
        assert!(compatible(2, 20.0, 0, f32::NAN, 10.0, 0.0));
        assert!(compatible(2, 20.0, 2, 100.0, 10.0, 0.0));
        assert!(!compatible(2, 5.0, 2, 100.0, 10.0, 0.0));
        assert!(compatible(1, 10.00001, 2, 100.0, 10.0, 0.0));
        let mut cache = RiveMeasureCache::new();
        let key = request(100.0);
        cache.get(key, RiveMeasureMetadata::default(), 1);
        cache.finish(key, output(-1.0), RunMode::ComputeSize, 1, false);
        assert!(cache.get(key, RiveMeasureMetadata::default(), 1).is_none());
    }
}
