//! Resource rollover translated from `renderer/src/render_context.cpp`.

use crate::gpu;

const LARGEST_FP16_BEFORE_EXPONENT_ALL_ONES: usize = (0x1f << 10) - 1;
const LARGEST_DENORMALIZED_FP16: usize = 1023;
const CLEAR_COLOR_PATH_COUNT: usize = 1;

pub(crate) const MAX_PATH_COUNT: usize =
    LARGEST_FP16_BEFORE_EXPONENT_ALL_ONES - LARGEST_DENORMALIZED_FP16 - CLEAR_COLOR_PATH_COUNT;
pub(crate) const MAX_CONTOUR_COUNT: usize = gpu::CONTOUR_ID_MASK as usize;
pub(crate) const MAX_TESSELLATION_VERTEX_COUNT: usize = 2048 * gpu::TESS_TEXTURE_WIDTH as usize
    - (gpu::MIDPOINT_FAN_PATCH_SEGMENT_SPAN + (gpu::OUTER_CURVE_PATCH_SEGMENT_SPAN - 1) + 1);
pub(crate) const MAX_REORDERED_DRAW_PASS_COUNT: usize = i16::MAX as usize;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ResourceCounters {
    pub midpoint_fan_tess_vertex_count: usize,
    pub outer_cubic_tess_vertex_count: usize,
    pub path_count: usize,
    pub contour_count: usize,
    pub max_tessellated_segment_count: usize,
    pub max_triangle_vertex_count: usize,
    pub image_draw_count: usize,
    pub draw_pass_count: usize,
}

impl ResourceCounters {
    fn checked_add(self, rhs: Self) -> Option<Self> {
        Some(Self {
            midpoint_fan_tess_vertex_count: self
                .midpoint_fan_tess_vertex_count
                .checked_add(rhs.midpoint_fan_tess_vertex_count)?,
            outer_cubic_tess_vertex_count: self
                .outer_cubic_tess_vertex_count
                .checked_add(rhs.outer_cubic_tess_vertex_count)?,
            path_count: self.path_count.checked_add(rhs.path_count)?,
            contour_count: self.contour_count.checked_add(rhs.contour_count)?,
            max_tessellated_segment_count: self
                .max_tessellated_segment_count
                .checked_add(rhs.max_tessellated_segment_count)?,
            max_triangle_vertex_count: self
                .max_triangle_vertex_count
                .checked_add(rhs.max_triangle_vertex_count)?,
            image_draw_count: self.image_draw_count.checked_add(rhs.image_draw_count)?,
            draw_pass_count: self.draw_pass_count.checked_add(rhs.draw_pass_count)?,
        })
    }

    fn fits(self) -> bool {
        // Rust's atomic encoder assigns a path-index slot to standalone image
        // draws as well. C++ stores those in a separate image buffer, so keep
        // the translated counters intact but enforce the Rust indexing limit
        // across both record classes.
        let path_indices_fit = self
            .path_count
            .checked_add(self.image_draw_count)
            .is_some_and(|count| count <= MAX_PATH_COUNT);
        self.path_count <= MAX_PATH_COUNT
            && path_indices_fit
            && self.contour_count <= MAX_CONTOUR_COUNT
            && self
                .midpoint_fan_tess_vertex_count
                .checked_add(self.outer_cubic_tess_vertex_count)
                .is_some_and(|count| count <= MAX_TESSELLATION_VERTEX_COUNT)
            && self.draw_pass_count <= MAX_REORDERED_DRAW_PASS_COUNT
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct LogicalFlush {
    counters: ResourceCounters,
}

impl LogicalFlush {
    pub(crate) fn push_draws(&mut self, batch: ResourceCounters) -> bool {
        let Some(combined) = self.counters.checked_add(batch) else {
            return false;
        };
        if !combined.fits() {
            return false;
        }
        self.counters = combined;
        true
    }

    pub(crate) fn rewind(&mut self) {
        self.counters = ResourceCounters::default();
    }

    #[cfg(test)]
    pub(crate) fn counters(self) -> ResourceCounters {
        self.counters
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_rolls_over_at_limit(field: fn(&mut ResourceCounters) -> &mut usize, limit: usize) {
        let mut flush = LogicalFlush::default();
        let mut at_limit = ResourceCounters::default();
        *field(&mut at_limit) = limit;
        assert!(flush.push_draws(at_limit));

        let mut one_more = ResourceCounters::default();
        *field(&mut one_more) = 1;
        assert!(!flush.push_draws(one_more));
        assert_eq!(flush.counters(), at_limit);

        flush.rewind();
        assert!(flush.push_draws(one_more));
        assert_eq!(flush.counters(), one_more);
    }

    #[test]
    fn path_budget_matches_cpp_fp16_id_reservation() {
        assert_eq!(MAX_PATH_COUNT, 30_719);
        assert_rolls_over_at_limit(|counts| &mut counts.path_count, MAX_PATH_COUNT);
    }

    #[test]
    fn image_draws_share_rusts_atomic_path_index_budget() {
        let mut flush = LogicalFlush::default();
        assert!(flush.push_draws(ResourceCounters {
            path_count: MAX_PATH_COUNT - 1,
            image_draw_count: 1,
            ..Default::default()
        }));
        assert!(!flush.push_draws(ResourceCounters {
            image_draw_count: 1,
            ..Default::default()
        }));
    }

    #[test]
    fn contour_budget_matches_cpp_u16_id_limit() {
        assert_rolls_over_at_limit(|counts| &mut counts.contour_count, MAX_CONTOUR_COUNT);
    }

    #[test]
    fn tessellation_budget_combines_both_patch_sections() {
        let mut flush = LogicalFlush::default();
        assert!(flush.push_draws(ResourceCounters {
            midpoint_fan_tess_vertex_count: MAX_TESSELLATION_VERTEX_COUNT - 17,
            outer_cubic_tess_vertex_count: 17,
            ..Default::default()
        }));
        assert!(!flush.push_draws(ResourceCounters {
            midpoint_fan_tess_vertex_count: 1,
            ..Default::default()
        }));
    }

    #[test]
    fn draw_pass_budget_matches_cpp_signed_sort_key() {
        assert_rolls_over_at_limit(
            |counts| &mut counts.draw_pass_count,
            MAX_REORDERED_DRAW_PASS_COUNT,
        );
    }

    #[test]
    fn overflow_fails_without_mutating_the_flush() {
        let mut flush = LogicalFlush::default();
        let accepted = ResourceCounters {
            path_count: 1,
            ..Default::default()
        };
        assert!(flush.push_draws(accepted));
        assert!(!flush.push_draws(ResourceCounters {
            path_count: usize::MAX,
            ..Default::default()
        }));
        assert_eq!(flush.counters(), accepted);
    }
}
