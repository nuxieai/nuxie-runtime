//! Opt-in support seam for the pinned upstream microbenchmarks.
//!
//! This module deliberately exposes a small, feature-gated API into the
//! production-compiled tessellation and intersection implementations. Keeping
//! this seam stable is a permanent maintenance cost, but it avoids compiling
//! test-only counters or copied private modules into benchmark binaries.

use nuxie_render_api::{FillRule, RawPath, RenderPaintStyle, StrokeCap, StrokeJoin};

use crate::intersection_board::{
    FindResult, GroupingType, IntersectionBoard, IntersectionTile, Rect,
};

pub struct IntersectionTileWorkload {
    grouping_type: GroupingType,
    tile: IntersectionTile,
}

impl IntersectionTileWorkload {
    pub fn disjoint() -> Self {
        Self::new(GroupingType::Disjoint)
    }

    pub fn overlap_allowed() -> Self {
        Self::new(GroupingType::OverlapAllowed)
    }

    fn new(grouping_type: GroupingType) -> Self {
        let mut workload = Self {
            grouping_type,
            tile: IntersectionTile::default(),
        };
        workload.run();
        workload
    }

    pub fn run(&mut self) -> i16 {
        reset_cpp_rand();
        self.tile.reset(0, 0, 0, 0);
        let mut result = 0;
        for _ in 0..10_000 {
            let values = [cpp_rand(), cpp_rand(), cpp_rand(), cpp_rand()].map(|value| value & 0xff);
            let left = values[0].min(values[2]).min(254);
            let top = values[1].min(values[3]).min(254);
            let right = values[0].max(values[2]).min(254) + 1;
            let bottom = values[1].max(values[3]).min(254) + 1;
            let rect = Rect::new(left, top, right, bottom);
            let found = self.tile.find_max_intersecting_group_index(
                self.grouping_type,
                rect,
                FindResult::default(),
            );
            result = found.max_group_indices.into_iter().max().unwrap_or(0) + 1;
            self.tile.add_rectangle(self.grouping_type, rect, result, 0);
        }
        result
    }
}

pub struct IntersectionBoardWorkload {
    board: IntersectionBoard,
    boxes: Vec<Rect>,
}

impl IntersectionBoardWorkload {
    pub fn from_i32le(bytes: &[u8]) -> Self {
        let boxes = bytes
            .chunks_exact(16)
            .map(|row| {
                let value = |offset| {
                    i32::from_le_bytes(row[offset..offset + 4].try_into().expect("bbox row"))
                };
                Rect::new(value(0), value(4), value(8), value(12))
            })
            .collect();
        let mut workload = Self {
            board: IntersectionBoard::new(GroupingType::Disjoint),
            boxes,
        };
        workload.run();
        workload
    }

    pub fn run(&mut self) -> i16 {
        self.board.resize_and_reset(3456, 2102);
        self.boxes.iter().fold(0, |maximum, &rect| {
            maximum.max(self.board.add_rectangle(rect, 1))
        })
    }
}

#[derive(Clone, Debug)]
pub struct CapturedPathPaint {
    pub path: RawPath,
    pub fill_rule: FillRule,
    pub style: RenderPaintStyle,
    pub thickness: f32,
    pub join: StrokeJoin,
    pub cap: StrokeCap,
    pub feather: f32,
}

#[derive(Clone, Copy)]
pub enum Preparation {
    Authored,
    Strokes(StrokeJoin),
    Feather(f32),
}

#[cfg(test)]
fn forced_feather_stroke_style() -> (StrokeJoin, StrokeCap) {
    (StrokeJoin::Round, StrokeCap::Round)
}

pub struct NullFrameWorkload {
    _paths: Vec<CapturedPathPaint>,
    _preparation: Preparation,
}

impl NullFrameWorkload {
    pub fn new(paths: Vec<CapturedPathPaint>, preparation: Preparation) -> Self {
        Self {
            _paths: paths,
            _preparation: preparation,
        }
    }

    pub fn run(&mut self) -> usize {
        panic!(
            "backend-neutral logical frame is not implemented; refusing to time direct tessellation"
        )
    }
}

fn reset_cpp_rand() {
    // SAFETY: C's process-global PRNG accepts every unsigned seed. The
    // benchmark runs these workloads serially.
    unsafe { libc::srand(0) };
}

fn cpp_rand() -> i32 {
    // SAFETY: `rand` has no preconditions. Benchmark execution is serial.
    unsafe { libc::rand() }
}

#[cfg(test)]
mod tests {
    use nuxie_render_api::{FillRule, RawPath, RenderPaintStyle, StrokeCap, StrokeJoin};

    #[test]
    fn forced_feather_uses_upstream_round_stroke_style() {
        assert_eq!(
            super::forced_feather_stroke_style(),
            (StrokeJoin::Round, StrokeCap::Round)
        );
    }

    #[test]
    #[should_panic(expected = "backend-neutral logical frame is not implemented")]
    fn draw_workload_refuses_to_time_tessellation_without_logical_flush() {
        let mut path = RawPath::new();
        path.move_to(0.0, 0.0);
        path.line_to(10.0, 10.0);
        let mut workload = super::NullFrameWorkload::new(
            vec![super::CapturedPathPaint {
                path,
                fill_rule: FillRule::EvenOdd,
                style: RenderPaintStyle::Fill,
                thickness: 1.0,
                join: StrokeJoin::Miter,
                cap: StrokeCap::Butt,
                feather: 0.0,
            }],
            super::Preparation::Authored,
        );

        workload.run();
    }
}
