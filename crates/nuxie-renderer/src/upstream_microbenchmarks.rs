//! Opt-in support seam for the pinned upstream microbenchmarks.
//!
//! This module deliberately exposes a small, feature-gated API into the
//! production-compiled tessellation and intersection implementations. Keeping
//! this seam stable is a permanent maintenance cost, but it avoids compiling
//! test-only counters or copied private modules into benchmark binaries.

use nuxie_render_api::{
    BlendMode, ColorInt, FillRule, RawPath, RenderPaintStyle, StrokeCap, StrokeJoin,
};

use crate::intersection_board::{
    FindResult, GroupingType, IntersectionBoard, IntersectionTile, Rect,
};
use crate::{
    LogicalFrameConfig, LogicalGradient, LogicalPathHandle, LogicalPathPaint, NullLogicalRenderer,
    RenderMode,
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
    pub color: ColorInt,
    pub blend_mode: BlendMode,
    pub gradient: Option<LogicalGradient>,
}

#[derive(Clone, Copy)]
pub enum Preparation {
    Authored,
    Strokes(StrokeJoin),
    Feather(f32),
}

struct PreparedPathPaint {
    path: LogicalPathHandle,
    paint: LogicalPathPaint,
    gradient: Option<LogicalGradient>,
}

pub struct NullFrameWorkload {
    renderer: NullLogicalRenderer,
    draws: Vec<PreparedPathPaint>,
    completed_frames: u64,
}

impl NullFrameWorkload {
    pub fn new(mut paths: Vec<CapturedPathPaint>, preparation: Preparation) -> Self {
        for draw in &mut paths {
            match preparation {
                Preparation::Authored => {}
                Preparation::Strokes(join) => {
                    draw.style = RenderPaintStyle::Stroke;
                    draw.thickness = 2.0;
                    draw.join = join;
                }
                Preparation::Feather(feather) => {
                    draw.feather = feather;
                    draw.fill_rule = FillRule::Clockwise;
                }
            }
        }
        let renderer = NullLogicalRenderer::new(LogicalFrameConfig {
            width: 1600,
            height: 1600,
            mode: RenderMode::RasterOrdering,
            max_texture_dimension_2d: 8192,
            msaa_atlas_supports_clip_rect: true,
        });
        let draws = paths
            .into_iter()
            .map(|draw| PreparedPathPaint {
                path: renderer.prepare_path(&draw.path, draw.fill_rule),
                paint: LogicalPathPaint {
                    style: draw.style,
                    color: draw.color,
                    thickness: draw.thickness,
                    join: draw.join,
                    cap: draw.cap,
                    feather: draw.feather,
                    blend_mode: draw.blend_mode,
                },
                gradient: draw.gradient,
            })
            .collect();
        Self {
            renderer,
            draws,
            completed_frames: 0,
        }
    }

    pub fn run(&mut self) -> usize {
        let mut written_bytes = 0usize;
        for _ in 0..10 {
            written_bytes = written_bytes.wrapping_add(self.run_frame().written_bytes);
        }
        written_bytes
    }

    pub fn run_frame(&mut self) -> crate::LogicalFrameReport {
        self.renderer.begin_frame();
        for draw in &self.draws {
            match &draw.gradient {
                Some(gradient) => {
                    self.renderer
                        .draw_path_with_gradient(&draw.path, draw.paint, gradient.clone())
                }
                None => self.renderer.draw_path(&draw.path, draw.paint),
            }
            .expect("pinned upstream path is supported by the production logical frame");
        }
        let report = self
            .renderer
            .flush()
            .expect("pinned upstream logical frame flushes through the null adapter");
        self.completed_frames = self.completed_frames.saturating_add(1);
        report
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
    use nuxie_render_api::{BlendMode, FillRule, RawPath, RenderPaintStyle, StrokeCap, StrokeJoin};

    #[test]
    fn draw_workload_runs_ten_production_logical_frames() {
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
                color: 0xff00_0000,
                blend_mode: BlendMode::SrcOver,
                gradient: None,
            }],
            super::Preparation::Authored,
        );

        let report = workload.run_frame();

        assert_eq!(report.mode, super::RenderMode::RasterOrdering);
        assert!(report.written_bytes > 0);
        assert_eq!(workload.completed_frames, 1);

        let written_bytes = workload.run();

        assert!(written_bytes > 0);
        assert_eq!(workload.completed_frames, 11);
    }
}
