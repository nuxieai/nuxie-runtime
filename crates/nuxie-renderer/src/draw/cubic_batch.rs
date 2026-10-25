//! RenderContext::CubicBatch from renderer/render_context.hpp. Kept outside the
//! feature-gated context module so CPU path preparation uses the same storage.
use nuxie_render_api::Vec2D;

#[derive(Default)]
pub(crate) struct CubicBatch {
    pub points: Vec<[f32; 4]>,
    pub chop_keys: Vec<u8>,
    pub chop_candidates: Vec<u32>,
    pub chop_t0: Vec<f32>,
    pub chop_t1: Vec<f32>,
    // Safe owned originals: the generic host also normalizes quadratics, whose
    // cubic control points are not a contiguous slice in the input RawPath.
    pub raw_path_cubic_points: Vec<[Vec2D; 4]>,
    // Retained Rust counterparts of the source's per-frame arrays and traversal
    // cursors. They map combined contour records back to the main/extra slots.
    pub candidate_slots: Vec<usize>,
    pub extra_curve_indices: Vec<usize>,
    pub authored_join_slots: Vec<usize>,
    pub cusp_join_slots: Vec<usize>,
    pub parametric_counts: Vec<u32>,
    pub tangent_pairs: Vec<[Vec2D; 2]>,
    pub polar_counts: Vec<u32>,
    pub rotation_slots: Vec<u32>,
    pub cusp_joins: Vec<[u32; 3]>,
}
impl CubicBatch {
    pub fn resize(&mut self, cubic_count: usize, max_curve_count: usize, is_stroke: bool) {
        let block_float4_count = (max_curve_count + 3) / 4 * 8;
        if self.points.len() < block_float4_count {
            self.points.resize(block_float4_count, [0.0; 4]);
        }
        if is_stroke && self.rotation_slots.len() < max_curve_count {
            self.rotation_slots.resize(max_curve_count, 0);
        }
        if is_stroke && self.raw_path_cubic_points.len() < cubic_count {
            self.raw_path_cubic_points
                .resize(cubic_count, [Vec2D::new(0.0, 0.0); 4]);
        }
    }
    pub fn set_cubic(&mut self, index: usize, points: [Vec2D; 4]) {
        let lane = index / 4 * 8 + index % 4;
        self.points[lane] = [points[0].x, points[0].y, points[1].x, points[1].y];
        self.points[lane + 4] = [points[2].x, points[2].y, points[3].x, points[3].y];
    }
    pub fn cubic(&self, index: usize) -> [Vec2D; 4] {
        let lane = index / 4 * 8 + index % 4;
        let a = self.points[lane];
        let b = self.points[lane + 4];
        [
            Vec2D::new(a[0], a[1]),
            Vec2D::new(a[2], a[3]),
            Vec2D::new(b[0], b[1]),
            Vec2D::new(b[2], b[3]),
        ]
    }
}
