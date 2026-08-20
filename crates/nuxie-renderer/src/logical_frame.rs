//! Backend-neutral logical frame planning shared by GPU and null adapters.
//!
//! The interface deliberately stays at begin/draw/flush. All resource
//! accounting, retained shadow-buffer growth, typed writes, and rewind live
//! behind it so a backend cannot accidentally benchmark a shallower seam.

use std::borrow::Cow;
use std::collections::HashSet;
use std::mem::size_of;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use bytemuck::{Pod, Zeroable};
use smallvec::SmallVec;
#[cfg(test)]
use std::cell::Cell;

use nuxie_render_api::{
    BlendMode, ColorInt, FillRule, Mat2D, RawPath, RenderPaintStyle, StrokeCap, StrokeJoin,
};

use super::{
    apply_clip_rect, atomic_fill_clockwise_override, atomic_paint_fill_rule, clip_rect_paint_aux,
    draw, feather_atlas_placement, gpu, gradient_paint_aux, gradient_pipeline,
    intersect_pixel_bounds, logical_flush, modulate_color_alpha, multiply,
    pack_logical_feather_atlas_for_cpp, path_aabb, path_draw_has_valid_parameters,
    path_draw_pixel_bounds, pixel_bounds_are_empty, pixel_bounds_are_outside_frame,
    prepare_path_draw_with_pixel_bounds, ClipElement, DrawRole, DrawState, LogicalPaint,
    LogicalPath, MsaaClipResetAction, PathDrawPreparation, PreparedFillGeometry, RenderMode,
    SolidDraw, FEATHER_ATLAS_PADDING,
};

#[cfg(any(test, feature = "native-metal-experimental"))]
use super::PreparedFeatherGeometry;

#[cfg(test)]
thread_local! {
    static PATH_DRAW_ADMISSION_EVALUATIONS: Cell<usize> = const { Cell::new(0) };
    static GRADIENT_BATCH_PREPARATIONS: Cell<usize> = const { Cell::new(0) };
    static PREPARED_TYPED_TESSELLATION_VECTOR_COPIES: Cell<usize> = const { Cell::new(0) };
    static PRODUCTION_DIRECT_MSAA_LOGICAL_WRITER_TESSELLATION_COPIES: Cell<usize> = const { Cell::new(0) };
    static LOGICAL_FEATHER_ATLAS_PLACEMENT_RECORDS: Cell<usize> = const { Cell::new(0) };
    static PLANNED_DRAW_BATCH_HEAP_BACKINGS: Cell<usize> = const { Cell::new(0) };
    static PREPARED_LOGICAL_FRAME_AUXILIARY_HEAP_BACKINGS: Cell<usize> = const { Cell::new(0) };
}

#[cfg(test)]
pub(crate) fn take_path_draw_admission_evaluations() -> usize {
    PATH_DRAW_ADMISSION_EVALUATIONS.with(|evaluations| evaluations.replace(0))
}

#[cfg(test)]
pub(crate) fn take_gradient_batch_preparations() -> usize {
    GRADIENT_BATCH_PREPARATIONS.with(|preparations| preparations.replace(0))
}

#[cfg(test)]
pub(crate) fn reset_prepared_typed_tessellation_vector_copies() {
    PREPARED_TYPED_TESSELLATION_VECTOR_COPIES.with(|copies| copies.set(0));
}

#[cfg(test)]
pub(crate) fn prepared_typed_tessellation_vector_copies() -> usize {
    PREPARED_TYPED_TESSELLATION_VECTOR_COPIES.with(Cell::get)
}

#[cfg(test)]
pub(crate) fn reset_production_direct_msaa_logical_writer_tessellation_copies() {
    PRODUCTION_DIRECT_MSAA_LOGICAL_WRITER_TESSELLATION_COPIES.with(|copies| copies.set(0));
}

#[cfg(test)]
pub(crate) fn production_direct_msaa_logical_writer_tessellation_copies() -> usize {
    PRODUCTION_DIRECT_MSAA_LOGICAL_WRITER_TESSELLATION_COPIES.with(Cell::get)
}

#[cfg(test)]
pub(crate) fn reset_logical_feather_atlas_placement_records() {
    LOGICAL_FEATHER_ATLAS_PLACEMENT_RECORDS.with(|records| records.set(0));
}

#[cfg(test)]
pub(crate) fn logical_feather_atlas_placement_records() -> usize {
    LOGICAL_FEATHER_ATLAS_PLACEMENT_RECORDS.with(Cell::get)
}

#[cfg(test)]
pub(crate) fn reset_planned_draw_batch_heap_backings() {
    PLANNED_DRAW_BATCH_HEAP_BACKINGS.with(|backings| backings.set(0));
}

#[cfg(test)]
pub(crate) fn planned_draw_batch_heap_backings() -> usize {
    PLANNED_DRAW_BATCH_HEAP_BACKINGS.with(Cell::get)
}

#[cfg(test)]
pub(crate) fn reset_prepared_logical_frame_auxiliary_heap_backings() {
    PREPARED_LOGICAL_FRAME_AUXILIARY_HEAP_BACKINGS.with(|backings| backings.set(0));
}

#[cfg(test)]
pub(crate) fn prepared_logical_frame_auxiliary_heap_backings() -> usize {
    PREPARED_LOGICAL_FRAME_AUXILIARY_HEAP_BACKINGS.with(Cell::get)
}

#[derive(Clone)]
pub(crate) struct GradientDefinition {
    pub(crate) paint_type: gpu::PaintType,
    pub(crate) colors: Vec<ColorInt>,
    pub(crate) stops: Vec<f32>,
    pub(crate) coeffs: [f32; 3],
}

#[derive(Clone, Copy)]
pub(crate) struct PreparedGradient {
    pub(crate) paint_type: gpu::PaintType,
    pub(crate) texture_y: f32,
    pub(crate) matrix: Mat2D,
    pub(crate) texture_span: [f32; 2],
}

#[derive(Clone, Copy)]
struct PreparedGradientDraw {
    occurrence_id: u64,
    gradient: Option<PreparedGradient>,
}

#[derive(Clone)]
pub(crate) struct GradientBatch {
    pub(crate) spans: Vec<gpu::GradientSpan>,
    pub(crate) height: u32,
    draws: Vec<PreparedGradientDraw>,
}

impl GradientBatch {
    pub(crate) fn draw(&self, index: usize) -> Option<PreparedGradient> {
        if self.draws.is_empty() {
            None
        } else {
            self.draws[index].gradient
        }
    }

    fn draw_for_occurrence(&self, occurrence_id: u64) -> Option<PreparedGradient> {
        self.draws
            .iter()
            .find(|draw| draw.occurrence_id == occurrence_id)
            .and_then(|draw| draw.gradient)
    }

    fn contains_occurrence(&self, occurrence_id: u64) -> bool {
        self.draws
            .iter()
            .any(|draw| draw.occurrence_id == occurrence_id)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.height == 0
    }

    #[cfg(test)]
    pub(crate) fn draw_table_is_empty(&self) -> bool {
        self.draws.is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LogicalFrameConfig {
    pub width: u32,
    pub height: u32,
    pub mode: RenderMode,
    pub max_texture_dimension_2d: u32,
    pub msaa_atlas_supports_clip_rect: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct LogicalPathPaint {
    pub style: RenderPaintStyle,
    pub color: ColorInt,
    pub thickness: f32,
    pub join: StrokeJoin,
    pub cap: StrokeCap,
    pub feather: f32,
    pub blend_mode: BlendMode,
}

/// Immutable backend-neutral path storage prepared once and shared by every
/// frame/draw that references it.
#[derive(Clone, Debug)]
pub struct LogicalPathHandle {
    path: LogicalPath,
}

impl LogicalPathHandle {
    pub fn new(raw_path: &RawPath, fill_rule: FillRule) -> Self {
        let mut raw_path = raw_path.clone();
        raw_path.renew_mutation_id();
        Self {
            path: LogicalPath {
                raw_path: Arc::new(raw_path),
                fill_rule,
                valid: true,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum LogicalGradient {
    Linear {
        start: (f32, f32),
        end: (f32, f32),
        colors: Vec<ColorInt>,
        stops: Vec<f32>,
    },
    Radial {
        center: (f32, f32),
        radius: f32,
        colors: Vec<ColorInt>,
        stops: Vec<f32>,
    },
}

impl LogicalGradient {
    fn into_logical_shader(self) -> super::LogicalShader {
        match self {
            Self::Linear {
                start,
                end,
                colors,
                stops,
            } => super::LogicalShader::Linear {
                start,
                end,
                colors,
                stops,
            },
            Self::Radial {
                center,
                radius,
                colors,
                stops,
            } => super::LogicalShader::Radial {
                center,
                radius,
                colors,
                stops,
            },
        }
    }
}

impl Default for LogicalPathPaint {
    fn default() -> Self {
        Self {
            style: RenderPaintStyle::Fill,
            color: 0xff00_0000,
            thickness: 1.0,
            join: StrokeJoin::Miter,
            cap: StrokeCap::Butt,
            feather: 0.0,
            blend_mode: BlendMode::SrcOver,
        }
    }
}

impl LogicalPathPaint {
    pub(crate) fn into_logical_paint(self) -> LogicalPaint {
        LogicalPaint {
            style: self.style,
            color: self.color,
            thickness: self.thickness.abs(),
            join: self.join,
            cap: self.cap,
            feather: self.feather.abs(),
            blend_mode: self.blend_mode,
            ..Default::default()
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LogicalResourceCounts {
    pub path_records: usize,
    pub paint_records: usize,
    pub contour_records: usize,
    pub tessellation_records: usize,
    pub triangle_records: usize,
    pub image_records: usize,
    pub draw_records: usize,
    pub gradient_records: usize,
    pub gradient_color_records: usize,
    pub gradient_stop_records: usize,
}

impl LogicalResourceCounts {
    fn from_flush(counters: logical_flush::ResourceCounters, draws: &[SolidDraw]) -> Self {
        let mut gradient_records = 0usize;
        let mut gradient_color_records = 0usize;
        let mut gradient_stop_records = 0usize;
        for shader in draws.iter().filter_map(|draw| draw.paint.shader.as_ref()) {
            let (colors, stops) = match shader {
                super::LogicalShader::Linear { colors, stops, .. }
                | super::LogicalShader::Radial { colors, stops, .. } => (colors, stops),
            };
            gradient_records = gradient_records.saturating_add(1);
            gradient_color_records = gradient_color_records.saturating_add(colors.len());
            gradient_stop_records = gradient_stop_records.saturating_add(stops.len());
        }
        Self {
            path_records: counters.path_count,
            paint_records: counters.path_count,
            contour_records: counters.contour_count,
            tessellation_records: counters
                .midpoint_fan_tess_vertex_count
                .saturating_add(counters.outer_cubic_tess_vertex_count),
            triangle_records: counters.max_triangle_vertex_count,
            image_records: counters.image_draw_count,
            draw_records: counters.draw_pass_count,
            gradient_records,
            gradient_color_records,
            gradient_stop_records,
        }
    }

    fn checked_add(self, rhs: Self) -> Option<Self> {
        Some(Self {
            path_records: self.path_records.checked_add(rhs.path_records)?,
            paint_records: self.paint_records.checked_add(rhs.paint_records)?,
            contour_records: self.contour_records.checked_add(rhs.contour_records)?,
            tessellation_records: self
                .tessellation_records
                .checked_add(rhs.tessellation_records)?,
            triangle_records: self.triangle_records.checked_add(rhs.triangle_records)?,
            image_records: self.image_records.checked_add(rhs.image_records)?,
            draw_records: self.draw_records.checked_add(rhs.draw_records)?,
            gradient_records: self.gradient_records.checked_add(rhs.gradient_records)?,
            gradient_color_records: self
                .gradient_color_records
                .checked_add(rhs.gradient_color_records)?,
            gradient_stop_records: self
                .gradient_stop_records
                .checked_add(rhs.gradient_stop_records)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RasterOrderingPlan {
    /// Non-atlased path coverage is accumulated in the transient PLS coverage
    /// plane.
    pub pixel_local_storage_coverage: bool,
    pub pixel_local_storage_draws: usize,
    pub feather_atlas_draws: usize,
    /// Raster-ordering consumes draw passes in authored order instead of
    /// using the overlap sorter required by atomic and MSAA modes.
    pub authored_draw_order_preserved: bool,
    /// Raster ordering provides the interlock, so adjacent draw passes do not
    /// require explicit PLS barriers.
    pub interlock_barriers: usize,
    /// C++ allocates clip, scratch-color, and coverage transient PLS planes.
    pub transient_backing_planes: usize,
    pub draw_passes: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogicalFrameReport {
    pub mode: RenderMode,
    pub raster_ordering: Option<RasterOrderingPlan>,
    pub draw_count: usize,
    pub resource_planning_passes: usize,
    pub plan_finalization_passes: usize,
    pub logical_flushes: Vec<LogicalResourceCounts>,
    pub retained_capacity: LogicalResourceCounts,
    pub written: LogicalResourceCounts,
    pub allocation_growths: usize,
    pub buffer_write_operations: usize,
    pub buffer_rewinds: usize,
    pub written_bytes: usize,
    pub shadow_fingerprint: u64,
    /// Draws for which the shared logical writer emitted a typed path/paint
    /// resource bundle that the production backend must consume exactly once.
    pub production_typed_output_eligible_draws: usize,
    /// Eligible typed bundles consumed exactly once by the production backend.
    pub production_typed_output_consumed_draws: usize,
    /// Draws intentionally handled by a backend-specific non-path fallback
    /// (for example images and MSAA clip resets).
    pub production_fallback_draws: usize,
    pub production_typed_output_consumed: bool,
}

pub struct LogicalFrame {
    pub(crate) config: LogicalFrameConfig,
    pub(crate) logical_state: LogicalDrawState,
    pub(crate) draws: Vec<SolidDraw>,
    pub(crate) draw_resources: Vec<logical_flush::ResourceCounters>,
    pub(crate) resource_planning_evaluations: usize,
    pub(crate) logical_flush: logical_flush::LogicalFlush,
    pub(crate) logical_flush_allocations: LogicalFlushAllocations,
    pub(crate) logical_flush_starts: Vec<usize>,
    pub(crate) msaa_schedule: Vec<super::MsaaDrawSchedule>,
    msaa_scheduled_resources_scratch: Vec<logical_flush::ResourceCounters>,
    atomic_batch_flags: Vec<bool>,
    occurrence_validation_scratch: HashSet<u64>,
    next_occurrence_id: u64,
    pub(crate) msaa_schedule_flush_starts: Vec<usize>,
    retain_in_pool: bool,
    finalized: bool,
    finalization_passes: usize,
}

const MAX_RETAINED_LOGICAL_FRAMES: usize = 3;
const MAX_RETAINED_LOGICAL_FRAME_DRAWS: usize = 16 * 1024;

#[derive(Default)]
pub(crate) struct LogicalFramePool {
    cached: Mutex<Vec<LogicalFrame>>,
}

impl LogicalFramePool {
    pub(crate) fn checkout(self: &Arc<Self>, config: LogicalFrameConfig) -> LogicalFrameLease {
        let mut frame = self
            .cached
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .pop()
            .unwrap_or_else(|| LogicalFrame::new(config));
        frame.reset(config);
        LogicalFrameLease {
            frame: Some(frame),
            pool: Arc::clone(self),
        }
    }
}

pub(crate) struct LogicalFrameLease {
    frame: Option<LogicalFrame>,
    pool: Arc<LogicalFramePool>,
}

impl std::ops::Deref for LogicalFrameLease {
    type Target = LogicalFrame;

    fn deref(&self) -> &Self::Target {
        self.frame
            .as_ref()
            .expect("logical frame lease must own a frame")
    }
}

impl std::ops::DerefMut for LogicalFrameLease {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.frame
            .as_mut()
            .expect("logical frame lease must own a frame")
    }
}

impl Drop for LogicalFrameLease {
    fn drop(&mut self) {
        let Some(mut frame) = self.frame.take() else {
            return;
        };
        let retain_in_pool =
            frame.retain_in_pool && frame.logical_state.retained_backings_within_limit();
        frame.reset(frame.config);
        let mut cached = self
            .pool
            .cached
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if retain_in_pool && cached.len() < MAX_RETAINED_LOGICAL_FRAMES {
            cached.push(frame);
        }
    }
}

struct PlannedDrawBatch {
    draws: SmallVec<[logical_flush::ResourceCounters; 1]>,
    total: logical_flush::ResourceCounters,
}

impl LogicalFrame {
    fn note_retained_frame_growth(&mut self) {
        if self.draw_resources.len() > MAX_RETAINED_LOGICAL_FRAME_DRAWS {
            self.retain_in_pool = false;
        }
    }

    pub(crate) fn new(config: LogicalFrameConfig) -> Self {
        Self {
            config,
            logical_state: LogicalDrawState::default(),
            draws: Vec::new(),
            draw_resources: Vec::new(),
            resource_planning_evaluations: 0,
            logical_flush: logical_flush::LogicalFlush::default(),
            logical_flush_allocations: LogicalFlushAllocations::default(),
            logical_flush_starts: vec![0],
            msaa_schedule: Vec::new(),
            msaa_scheduled_resources_scratch: Vec::new(),
            atomic_batch_flags: Vec::new(),
            occurrence_validation_scratch: HashSet::new(),
            msaa_schedule_flush_starts: Vec::new(),
            next_occurrence_id: 1,
            retain_in_pool: true,
            finalized: false,
            finalization_passes: 0,
        }
    }

    fn reset(&mut self, config: LogicalFrameConfig) {
        self.config = config;
        self.logical_state.state = DrawState::default();
        self.logical_state.stack.clear();
        self.logical_state.clips.clear();
        self.logical_state.next_clip_id = 1;
        self.logical_state.msaa_path_clips.clear();
        self.logical_state.msaa_path_clip_id = 0;
        self.logical_state.generic_atomic_path_clip_id = 0;
        self.logical_state.oversized_backing = false;
        self.draws.clear();
        self.draw_resources.clear();
        self.resource_planning_evaluations = 0;
        self.logical_flush.rewind();
        self.logical_flush_allocations.simple_gradient_count = 0;
        self.logical_flush_allocations.complex_gradient_count = 0;
        self.logical_flush_allocations.atlas_draw_sizes.clear();
        self.logical_flush_starts.clear();
        self.logical_flush_starts.push(0);
        self.msaa_schedule.clear();
        self.msaa_scheduled_resources_scratch.clear();
        self.atomic_batch_flags.clear();
        self.occurrence_validation_scratch.clear();
        self.next_occurrence_id = 1;
        self.msaa_schedule_flush_starts.clear();
        self.retain_in_pool = true;
        self.finalized = false;
        self.finalization_passes = 0;
    }

    pub(crate) fn finalize(
        &mut self,
        board: &mut super::intersection_board::IntersectionBoard,
    ) -> Result<(), &'static str> {
        self.finalize_impl(board, true)
    }

    pub(crate) fn finalize_for_production(
        &mut self,
        board: &mut super::intersection_board::IntersectionBoard,
    ) -> Result<(), &'static str> {
        self.finalize_impl(board, false)
    }

    fn finalize_impl(
        &mut self,
        board: &mut super::intersection_board::IntersectionBoard,
        validate_occurrences: bool,
    ) -> Result<(), &'static str> {
        if self.finalized {
            return if validate_occurrences {
                self.validate()
            } else {
                self.validate_structure()
            };
        }
        // Focused tests and internal replay helpers can construct SolidDraws
        // directly. Give those occurrences the same stable identity that the
        // normal admission path assigns before any scheduler clones/reorders.
        for draw in &mut self.draws {
            if draw.logical_occurrence_id == 0 {
                draw.logical_occurrence_id = self.next_occurrence_id;
                self.next_occurrence_id = self
                    .next_occurrence_id
                    .checked_add(1)
                    .ok_or("logical draw occurrence identity overflow")?;
            }
        }
        self.validate()?;
        if self.config.mode == RenderMode::Msaa {
            self.msaa_schedule.reserve(self.draws.len());
            self.msaa_schedule_flush_starts
                .reserve(self.logical_flush_starts.len());
            for (flush_index, &flush_start) in self.logical_flush_starts.iter().enumerate() {
                let flush_end = self
                    .logical_flush_starts
                    .get(flush_index + 1)
                    .copied()
                    .unwrap_or(self.draws.len());
                let schedule_start = self.msaa_schedule.len();
                self.msaa_schedule_flush_starts.push(schedule_start);
                super::ordered_msaa_draws_with_board(
                    &self.draws[flush_start..flush_end],
                    self.config.width,
                    self.config.height,
                    board,
                    &mut self.msaa_schedule,
                );
                self.msaa_scheduled_resources_scratch.clear();
                self.msaa_scheduled_resources_scratch.extend(
                    self.msaa_schedule[schedule_start..]
                        .iter()
                        .map(|entry| self.draw_resources[flush_start + entry.authored_order]),
                );
                super::apply_msaa_draw_schedule(
                    &mut self.draws[flush_start..flush_end],
                    &mut self.msaa_schedule[schedule_start..],
                );
                self.draw_resources[flush_start..flush_end]
                    .copy_from_slice(&self.msaa_scheduled_resources_scratch);
            }
        }
        populate_production_atomic_batch_flags(
            self.config,
            &self.draws,
            &self.logical_flush_starts,
            &mut self.atomic_batch_flags,
        );
        self.finalized = true;
        self.finalization_passes = self.finalization_passes.saturating_add(1);
        if validate_occurrences {
            self.validate()
        } else {
            self.validate_structure()
        }
    }

    pub(crate) fn is_finalized(&self) -> bool {
        self.finalized
    }

    pub(crate) fn validate(&mut self) -> Result<(), &'static str> {
        if self.finalized {
            self.occurrence_validation_scratch.clear();
            self.occurrence_validation_scratch.reserve(self.draws.len());
            if self.draws.iter().any(|draw| {
                draw.logical_occurrence_id == 0
                    || !self
                        .occurrence_validation_scratch
                        .insert(draw.logical_occurrence_id)
            }) {
                return Err("logical frame draw occurrence identities are not unique");
            }
        }
        self.validate_structure()
    }

    fn validate_structure(&self) -> Result<(), &'static str> {
        if self.draws.len() != self.draw_resources.len() {
            return Err("logical frame draw/resource plan length mismatch");
        }
        if self
            .logical_flush_starts
            .iter()
            .any(|&start| start > self.draws.len())
        {
            return Err("logical frame flush layout exceeds draw plan");
        }
        if self.finalized && self.atomic_batch_flags.len() != self.draws.len() {
            return Err("finalized logical frame atomic flags do not match draw plan");
        }
        if self.finalized
            && self.config.mode == RenderMode::Msaa
            && (self.msaa_schedule.len() != self.draws.len()
                || self.msaa_schedule_flush_starts.len() != self.logical_flush_starts.len())
        {
            return Err("finalized MSAA frame schedule does not match draw plan");
        }
        Ok(())
    }

    fn plan_draws<'a>(
        &mut self,
        draws: impl Clone + Iterator<Item = &'a SolidDraw>,
    ) -> Result<PlannedDrawBatch, &'static str> {
        let config = self.config;
        let planned = draws
            .clone()
            .map(|draw| {
                super::logical_flush_draw_resources(
                    draw,
                    config.mode,
                    config.width,
                    config.height,
                    config.mode == RenderMode::ClockwiseAtomic,
                )
            })
            .collect::<SmallVec<[_; 1]>>();
        #[cfg(test)]
        if planned.spilled() {
            PLANNED_DRAW_BATCH_HEAP_BACKINGS.with(|backings| backings.set(backings.get() + 1));
        }
        self.resource_planning_evaluations = self
            .resource_planning_evaluations
            .saturating_add(planned.len());
        let total = planned
            .iter()
            .try_fold(logical_flush::ResourceCounters::default(), |total, draw| {
                total.checked_add(*draw)
            })
            .ok_or("draw batch overflows logical flush resource accounting")?;
        Ok(PlannedDrawBatch {
            draws: planned,
            total,
        })
    }

    fn try_commit_planned_draws<'a>(
        &mut self,
        planned: &PlannedDrawBatch,
        draws: impl IntoIterator<Item = &'a SolidDraw>,
    ) -> Result<(), &'static str> {
        let config = self.config;
        let allocations = self.logical_flush_allocations.with_draws(config, draws)?;
        if !self.logical_flush.push_draws(planned.total) {
            return Err("draw batch exceeds logical flush resource counters");
        }
        self.logical_flush_allocations = allocations;
        self.draw_resources
            .extend_from_slice(planned.draws.as_slice());
        self.note_retained_frame_growth();
        Ok(())
    }

    fn rollover_plan(
        &mut self,
        original_updates: &[SolidDraw],
        original: &PlannedDrawBatch,
        updates: &[SolidDraw],
    ) -> Result<PlannedDrawBatch, &'static str> {
        let mut reused = vec![false; original_updates.len()];
        let mut draws = SmallVec::<[logical_flush::ResourceCounters; 1]>::new();
        draws.reserve(updates.len() + 1);
        for update in updates {
            let cached = original_updates
                .iter()
                .enumerate()
                .find_map(|(index, candidate)| {
                    (!reused[index] && draw_resource_shape_is_equivalent(candidate, update))
                        .then_some(index)
                });
            if let Some(index) = cached {
                reused[index] = true;
                draws.push(original.draws[index]);
            } else {
                draws.push(self.plan_draws(std::iter::once(update))?.draws[0]);
            }
        }
        draws.push(
            *original
                .draws
                .last()
                .expect("content resource plan must contain its content draw"),
        );
        #[cfg(test)]
        if draws.spilled() {
            PLANNED_DRAW_BATCH_HEAP_BACKINGS.with(|backings| backings.set(backings.get() + 1));
        }
        let total = draws
            .iter()
            .try_fold(logical_flush::ResourceCounters::default(), |total, draw| {
                total.checked_add(*draw)
            })
            .ok_or("draw batch overflows logical flush resource accounting")?;
        Ok(PlannedDrawBatch { draws, total })
    }

    pub(crate) fn begin_logical_flush(&mut self) {
        debug_assert_ne!(self.logical_flush_starts.last(), Some(&self.draws.len()));
        self.logical_flush_starts.push(self.draws.len());
        self.logical_flush.rewind();
        self.logical_flush_allocations = LogicalFlushAllocations::default();
        self.logical_state.reset_for_logical_flush();
    }

    pub(crate) fn push_content_batch(
        &mut self,
        mut initial_clip_updates: Vec<SolidDraw>,
        mut content: SolidDraw,
    ) -> Result<(), &'static str> {
        if self.finalized {
            return Err("cannot append draws after logical frame finalization");
        }
        for draw in initial_clip_updates
            .iter_mut()
            .chain(std::iter::once(&mut content))
        {
            draw.logical_occurrence_id = self.next_occurrence_id;
            self.next_occurrence_id = self
                .next_occurrence_id
                .checked_add(1)
                .ok_or("logical draw occurrence identity overflow")?;
        }
        let config = self.config;
        let uses_generic_atomic_plane =
            config.mode == RenderMode::ClockwiseAtomic && super::atomic_draw_is_eligible(&content);
        let content_clip_id = match content.role {
            DrawRole::Content { clip_id } => clip_id,
            DrawRole::ClipUpdate { .. } | DrawRole::ClipReset { .. } => {
                unreachable!("content batch must end in a content draw")
            }
        };
        let batch = initial_clip_updates.iter().chain(std::iter::once(&content));
        let planned = self.plan_draws(batch.clone())?;
        if self.try_commit_planned_draws(&planned, batch).is_ok() {
            self.draws.extend(initial_clip_updates);
            self.draws.push(content);
            self.logical_state.commit_generic_atomic_path_clip(
                config,
                content_clip_id,
                uses_generic_atomic_plane,
            );
            return Ok(());
        }
        if self.logical_flush_starts.last() == Some(&self.draws.len()) {
            return Err("draw batch exceeds logical flush resource limits");
        }

        let mut content = content;
        self.begin_logical_flush();
        let (mut clip_updates, clip_id) =
            self.logical_state.prepare_scheduled_clip_updates(config)?;
        for draw in &mut clip_updates {
            draw.logical_occurrence_id = self.next_occurrence_id;
            self.next_occurrence_id = self
                .next_occurrence_id
                .checked_add(1)
                .ok_or("logical draw occurrence identity overflow")?;
        }
        match &mut content.role {
            DrawRole::Content { clip_id: id } => *id = clip_id,
            DrawRole::ClipUpdate { .. } | DrawRole::ClipReset { .. } => {
                unreachable!("content batch must end in a content draw")
            }
        }
        let rollover_plan = self.rollover_plan(&initial_clip_updates, &planned, &clip_updates)?;
        let batch = clip_updates.iter().chain(std::iter::once(&content));
        self.try_commit_planned_draws(&rollover_plan, batch)?;
        self.draws.extend(clip_updates);
        self.draws.push(content);
        self.logical_state.commit_generic_atomic_path_clip(
            config,
            clip_id,
            uses_generic_atomic_plane,
        );
        Ok(())
    }
}

fn draw_resource_shape_is_equivalent(left: &SolidDraw, right: &SolidDraw) -> bool {
    match (left.role, right.role) {
        (DrawRole::ClipReset { .. }, DrawRole::ClipReset { .. }) => true,
        (
            DrawRole::ClipUpdate {
                parent_id: left_parent,
                ..
            },
            DrawRole::ClipUpdate {
                parent_id: right_parent,
                ..
            },
        ) => {
            (left_parent == 0) == (right_parent == 0)
                && Arc::ptr_eq(&left.path.raw_path, &right.path.raw_path)
                && left.path.fill_rule == right.path.fill_rule
                && left.paint.style == right.paint.style
                && left.paint.feather == right.paint.feather
                && left.state.transform == right.state.transform
                && left.prepared_pixel_bounds == right.prepared_pixel_bounds
        }
        _ => false,
    }
}

impl std::ops::Deref for LogicalFrame {
    type Target = LogicalDrawState;

    fn deref(&self) -> &Self::Target {
        &self.logical_state
    }
}

impl std::ops::DerefMut for LogicalFrame {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.logical_state
    }
}

pub struct LogicalDrawState {
    pub(crate) state: DrawState,
    pub(crate) stack: Vec<DrawState>,
    pub(crate) clips: Vec<ClipElement>,
    pub(crate) next_clip_id: u32,
    pub(crate) msaa_path_clips: Vec<ClipElement>,
    pub(crate) msaa_path_clip_id: u16,
    pub(crate) generic_atomic_path_clip_id: u16,
    oversized_backing: bool,
}

#[derive(Clone, Default)]
pub(crate) struct LogicalFlushAllocations {
    pub(crate) simple_gradient_count: usize,
    pub(crate) complex_gradient_count: usize,
    pub(crate) atlas_draw_sizes: Vec<(u32, u32)>,
}

impl LogicalFlushAllocations {
    #[cfg(test)]
    pub(crate) fn with_batch(
        &self,
        config: LogicalFrameConfig,
        draws: &[SolidDraw],
    ) -> Result<Self, &'static str> {
        self.with_draws(config, draws)
    }

    pub(crate) fn with_draws<'a>(
        &self,
        config: LogicalFrameConfig,
        draws: impl IntoIterator<Item = &'a SolidDraw>,
    ) -> Result<Self, &'static str> {
        const MAX_GRADIENT_HEIGHT: usize = 2048;
        const RAMPS_PER_SIMPLE_ROW: usize = gradient_pipeline::TEXTURE_WIDTH as usize / 2;

        let mut next = self.clone();
        for draw in draws {
            if let Some(gradient) = draw
                .paint
                .shader
                .as_ref()
                .and_then(|shader| normalize_gradient(shader, draw.state.opacity))
            {
                let simple = gradient.stops.len() == 1
                    || (gradient.stops.len() == 2
                        && gradient.stops[0] == 0.0
                        && gradient.stops[1] == 1.0);
                if simple {
                    next.simple_gradient_count = next
                        .simple_gradient_count
                        .checked_add(1)
                        .ok_or("logical flush gradient count overflow")?;
                } else {
                    next.complex_gradient_count = next
                        .complex_gradient_count
                        .checked_add(1)
                        .ok_or("logical flush gradient count overflow")?;
                }
            }

            let uses_feather_atlas = config.mode == RenderMode::Msaa
                || (matches!(
                    config.mode,
                    RenderMode::RasterOrdering | RenderMode::ClockwiseAtomic
                ) && draw::feather_requires_atlas(
                    draw.paint.feather,
                    draw.state.transform,
                    false,
                ));
            if draw.paint.feather != 0.0 && uses_feather_atlas {
                let placement = feather_atlas_placement(
                    &draw.path.raw_path,
                    draw.state.transform,
                    draw.paint.feather,
                    draw.paint.effective_stroke(),
                    config.width,
                    config.height,
                )
                .ok_or("draw has invalid feather atlas placement")?;
                next.atlas_draw_sizes.push((
                    placement.width - FEATHER_ATLAS_PADDING * 2,
                    placement.height - FEATHER_ATLAS_PADDING * 2,
                ));
            }
        }

        let gradient_height = next
            .simple_gradient_count
            .div_ceil(RAMPS_PER_SIMPLE_ROW)
            .checked_add(next.complex_gradient_count)
            .ok_or("logical flush gradient height overflow")?;
        if gradient_height > MAX_GRADIENT_HEIGHT.min(config.max_texture_dimension_2d as usize) {
            return Err("draw batch exceeds logical flush gradient texture limit");
        }
        if !next.atlas_draw_sizes.is_empty() {
            pack_logical_feather_atlas_for_cpp(
                config.max_texture_dimension_2d,
                &next.atlas_draw_sizes,
            )
            .map_err(|_| "draw batch exceeds logical flush feather atlas texture limit")?;
        }
        Ok(next)
    }
}

impl Default for LogicalDrawState {
    fn default() -> Self {
        Self {
            state: DrawState::default(),
            stack: Vec::new(),
            clips: Vec::new(),
            next_clip_id: 1,
            msaa_path_clips: Vec::new(),
            msaa_path_clip_id: 0,
            generic_atomic_path_clip_id: 0,
            oversized_backing: false,
        }
    }
}

impl LogicalDrawState {
    fn retained_backings_within_limit(&self) -> bool {
        !self.oversized_backing
    }

    fn note_stack_backing_growth(&mut self) {
        self.oversized_backing |= self.stack.capacity() > MAX_RETAINED_LOGICAL_FRAME_DRAWS;
    }

    fn note_clip_backing_growth(&mut self) {
        self.oversized_backing |= self.clips.capacity() > MAX_RETAINED_LOGICAL_FRAME_DRAWS;
    }

    fn note_msaa_path_clip_backing_growth(&mut self) {
        self.oversized_backing |=
            self.msaa_path_clips.capacity() > MAX_RETAINED_LOGICAL_FRAME_DRAWS;
    }

    pub(crate) fn save(&mut self) {
        self.stack.push(self.state);
        self.note_stack_backing_growth();
    }

    pub(crate) fn restore(&mut self) {
        if let Some(state) = self.stack.pop() {
            self.state = state;
        }
    }

    pub(crate) fn transform(&mut self, transform: nuxie_render_api::Mat2D) {
        self.state.transform = multiply(self.state.transform, transform);
    }

    pub(crate) fn clip_path(
        &mut self,
        config: LogicalFrameConfig,
        path: &LogicalPath,
    ) -> Result<(), &'static str> {
        if pixel_bounds_are_empty(self.state.overall_clip_pixel_bounds) {
            return Ok(());
        }
        if !path.valid {
            return Err("clip path contains resources from another renderer backend");
        }
        if path.raw_path.verbs().is_empty() {
            self.state.overall_clip_pixel_bounds = [0; 4];
            return Ok(());
        }
        if config.mode != RenderMode::Msaa {
            if let Some(rect) = path_aabb(&path.raw_path) {
                if apply_clip_rect(&mut self.state, rect) {
                    return Ok(());
                }
            }
        }
        self.push_clip_path(path);
        Ok(())
    }

    pub(crate) fn push_clip_path(&mut self, path: &LogicalPath) {
        let height = self.state.clip_stack_height;
        let needs_new_element = self
            .clips
            .get(height)
            .is_none_or(|clip| !clip.is_equivalent(self.state.transform, path));
        let pixel_bounds = if needs_new_element {
            let Some(pixel_bounds) = draw::path_pixel_bounds(&path.raw_path, self.state.transform)
            else {
                self.state.overall_clip_pixel_bounds = [0; 4];
                return;
            };
            pixel_bounds
        } else {
            let Some(pixel_bounds) = self.clips[height].pixel_bounds else {
                self.state.overall_clip_pixel_bounds = [0; 4];
                return;
            };
            pixel_bounds
        };
        self.state.overall_clip_pixel_bounds =
            intersect_pixel_bounds(self.state.overall_clip_pixel_bounds, pixel_bounds);
        if pixel_bounds_are_empty(self.state.overall_clip_pixel_bounds) {
            return;
        }
        if needs_new_element {
            self.clips.truncate(height);
            self.clips.push(ClipElement {
                path: path.clone(),
                matrix: self.state.transform,
                pixel_bounds: Some(pixel_bounds),
                prepared_fill: Arc::new(PreparedFillGeometry::new(path, self.state.transform)),
                clip_id: 0,
            });
            self.note_clip_backing_growth();
        }
        self.state.clip_stack_height = height + 1;
    }

    pub(crate) fn prepare_scheduled_clip_updates(
        &mut self,
        config: LogicalFrameConfig,
    ) -> Result<(Vec<SolidDraw>, u16), &'static str> {
        if matches!(
            config.mode,
            RenderMode::RasterOrdering | RenderMode::ClockwiseAtomic
        ) {
            let height = self.state.clip_stack_height;
            if height == 0 {
                return Ok((Vec::new(), 0));
            }
            let active_index = (self.generic_atomic_path_clip_id != 0)
                .then(|| {
                    self.clips[..height]
                        .iter()
                        .rposition(|clip| clip.clip_id == self.generic_atomic_path_clip_id)
                })
                .flatten();
            let parent_id = active_index
                .map(|index| self.clips[index].clip_id)
                .unwrap_or(0);
            let update_start = active_index.map_or(0, |index| index + 1);
            let (updates, clip_id) = if update_start == height {
                (Vec::new(), parent_id)
            } else {
                self.prepare_clip_updates_from(config, update_start, parent_id)?
            };
            return Ok((updates, clip_id));
        }

        let height = self.state.clip_stack_height;
        let current_clips = self.clips[..height].to_vec();
        let previous_active = self.msaa_path_clips.last().cloned();
        if current_clips.is_empty() {
            return Ok((Vec::new(), 0));
        }
        let active_index = if self.msaa_path_clip_id == 0 {
            None
        } else {
            self.clips[..height]
                .iter()
                .rposition(|clip| clip.clip_id == self.msaa_path_clip_id)
        };
        let parent_id = active_index
            .map(|index| self.clips[index].clip_id)
            .unwrap_or(0);
        let update_start = active_index.map_or(0, |index| index + 1);
        let (updates, clip_id) = if update_start == height {
            (Vec::new(), parent_id)
        } else {
            self.prepare_clip_updates_from(config, update_start, parent_id)?
        };

        let mut scheduled = Vec::with_capacity(updates.len() * 2 + 1);
        if self.msaa_path_clip_id != 0 && active_index.is_none() {
            if let Some(active) = previous_active.as_ref() {
                scheduled.push(Self::msaa_clip_reset_draw(
                    config,
                    active,
                    MsaaClipResetAction::ClearPrevious,
                ));
            }
        }
        for (offset, update) in updates.into_iter().enumerate() {
            scheduled.push(update);
            let clip_index = update_start + offset;
            if clip_index != 0 {
                let action = match current_clips[clip_index].path.fill_rule {
                    FillRule::NonZero => MsaaClipResetAction::IntersectPreviousNonZero,
                    FillRule::EvenOdd => MsaaClipResetAction::IntersectPreviousEvenOdd,
                    FillRule::Clockwise => MsaaClipResetAction::IntersectPreviousClockwise,
                };
                scheduled.push(Self::msaa_clip_reset_draw(
                    config,
                    &current_clips[clip_index - 1],
                    action,
                ));
            }
        }
        self.msaa_path_clips = current_clips;
        self.note_msaa_path_clip_backing_growth();
        self.msaa_path_clip_id = clip_id;
        Ok((scheduled, clip_id))
    }

    fn msaa_clip_reset_draw(
        config: LogicalFrameConfig,
        clip: &ClipElement,
        action: MsaaClipResetAction,
    ) -> SolidDraw {
        let bounds = clip
            .pixel_bounds
            .unwrap_or([0, 0, config.width as i32, config.height as i32]);
        let [left, top, right, bottom] = bounds;
        let bounds = [
            left.clamp(0, config.width as i32) as f32,
            top.clamp(0, config.height as i32) as f32,
            right.clamp(0, config.width as i32) as f32,
            bottom.clamp(0, config.height as i32) as f32,
        ];
        SolidDraw::new(
            clip.path.clone(),
            LogicalPaint::default(),
            DrawState::default(),
            DrawRole::ClipReset { bounds, action },
            None,
        )
    }

    pub(crate) fn prepare_clip_updates_from(
        &mut self,
        config: LogicalFrameConfig,
        start: usize,
        initial_parent_id: u16,
    ) -> Result<(Vec<SolidDraw>, u16), &'static str> {
        let height = self.state.clip_stack_height;
        if height == 0 {
            return Ok((Vec::new(), 0));
        }
        debug_assert!(start < height);
        let update_count = u32::try_from(height - start)
            .map_err(|_| "more than 65535 clip updates in one frame")?;
        let end = self
            .next_clip_id
            .checked_add(update_count)
            .ok_or("more than 65535 clip updates in one frame")?;
        if end > u16::MAX as u32 + 1 {
            return Err("more than 65535 clip updates in one frame");
        }
        let mut updates = Vec::with_capacity(height - start);
        let mut parent_id = initial_parent_id;
        let next_clip_id = self.next_clip_id;
        let state = self.state;
        for (offset, clip) in self.clips[start..height].iter_mut().enumerate() {
            let replacement_id = (next_clip_id + offset as u32) as u16;
            clip.clip_id = replacement_id;
            let prepared_pixel_bounds =
                if config.mode == RenderMode::ClockwiseAtomic && parent_id != 0 {
                    None
                } else {
                    clip.pixel_bounds
                };
            updates.push(SolidDraw::new_with_prepared_fill_and_pixel_bounds(
                clip.path.clone(),
                LogicalPaint::default(),
                DrawState {
                    transform: clip.matrix,
                    clip_rect: None,
                    clip_stack_height: 0,
                    ..state
                },
                DrawRole::ClipUpdate {
                    replacement_id,
                    parent_id,
                },
                None,
                Some(Arc::clone(&clip.prepared_fill)),
                None,
                prepared_pixel_bounds,
            ));
            parent_id = replacement_id;
        }
        self.next_clip_id = end;
        Ok((updates, parent_id))
    }

    pub(crate) fn commit_generic_atomic_path_clip(
        &mut self,
        config: LogicalFrameConfig,
        clip_id: u16,
        uses_generic_atomic_plane: bool,
    ) {
        if config.mode == RenderMode::RasterOrdering {
            if clip_id != 0 {
                self.generic_atomic_path_clip_id = clip_id;
            }
        } else if config.mode == RenderMode::ClockwiseAtomic {
            if !uses_generic_atomic_plane {
                self.generic_atomic_path_clip_id = 0;
            } else if clip_id != 0 {
                self.generic_atomic_path_clip_id = clip_id;
            }
        }
    }

    pub(crate) fn reset_for_logical_flush(&mut self) {
        self.next_clip_id = 1;
        self.msaa_path_clips.clear();
        self.msaa_path_clip_id = 0;
        self.generic_atomic_path_clip_id = 0;
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct TypedPaintRecord {
    paint: gpu::PaintData,
    aux: gpu::PaintAuxData,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct TypedDrawRecord {
    role: u32,
    primary_id: u32,
    secondary_id: u32,
    draw_pass_count: u32,
    bounds: [f32; 4],
    transform: [f32; 6],
    fill_rule: u32,
    local_contour_ids_are_dense: u32,
}

#[derive(Default)]
struct ShadowBuffers {
    paths: Vec<gpu::PathData>,
    paints: Vec<TypedPaintRecord>,
    contours: Vec<gpu::ContourData>,
    tessellations: Vec<gpu::TessVertexSpan>,
    triangles: Vec<gpu::TriangleVertex>,
    images: Vec<u64>,
    draws: Vec<TypedDrawRecord>,
    gradients: Vec<u8>,
    capacity: LogicalResourceCounts,
}

impl ShadowBuffers {
    fn grow(&mut self, required: LogicalResourceCounts) {
        self.capacity = LogicalResourceCounts {
            path_records: grow_count(self.capacity.path_records, required.path_records),
            paint_records: grow_count(self.capacity.paint_records, required.paint_records),
            contour_records: grow_count(self.capacity.contour_records, required.contour_records),
            tessellation_records: grow_count(
                self.capacity.tessellation_records,
                required.tessellation_records,
            ),
            triangle_records: grow_count(self.capacity.triangle_records, required.triangle_records),
            image_records: grow_count(self.capacity.image_records, required.image_records),
            draw_records: grow_count(self.capacity.draw_records, required.draw_records),
            gradient_records: grow_count(self.capacity.gradient_records, required.gradient_records),
            gradient_color_records: grow_count(
                self.capacity.gradient_color_records,
                required.gradient_color_records,
            ),
            gradient_stop_records: grow_count(
                self.capacity.gradient_stop_records,
                required.gradient_stop_records,
            ),
        };
        reserve_typed(&mut self.paths, self.capacity.path_records);
        reserve_typed(&mut self.paints, self.capacity.paint_records);
        reserve_typed(&mut self.contours, self.capacity.contour_records);
        reserve_typed(&mut self.tessellations, self.capacity.tessellation_records);
        reserve_typed(&mut self.triangles, self.capacity.triangle_records);
        reserve_typed(&mut self.images, self.capacity.image_records);
        reserve_typed(&mut self.draws, self.capacity.draw_records);
        let gradient_values = self
            .capacity
            .gradient_records
            .saturating_mul(6)
            .saturating_add(self.capacity.gradient_color_records)
            .saturating_add(self.capacity.gradient_stop_records);
        reserve_records(&mut self.gradients, gradient_values, 4);
    }

    fn buffer_capacities(&self) -> [usize; 8] {
        [
            self.paths
                .capacity()
                .saturating_mul(size_of::<gpu::PathData>()),
            self.paints
                .capacity()
                .saturating_mul(size_of::<TypedPaintRecord>()),
            self.contours
                .capacity()
                .saturating_mul(size_of::<gpu::ContourData>()),
            self.tessellations
                .capacity()
                .saturating_mul(size_of::<gpu::TessVertexSpan>()),
            self.triangles
                .capacity()
                .saturating_mul(size_of::<gpu::TriangleVertex>()),
            self.images.capacity().saturating_mul(size_of::<u64>()),
            self.draws
                .capacity()
                .saturating_mul(size_of::<TypedDrawRecord>()),
            self.gradients.capacity(),
        ]
    }

    fn rewind(&mut self) {
        self.paths.clear();
        self.paints.clear();
        self.contours.clear();
        self.tessellations.clear();
        self.triangles.clear();
        self.images.clear();
        self.draws.clear();
        self.gradients.clear();
    }

    fn fingerprint_into(&self, mut hash: u64) -> u64 {
        for bytes in [
            bytemuck::cast_slice(self.paths.as_slice()),
            bytemuck::cast_slice(self.paints.as_slice()),
            bytemuck::cast_slice(self.contours.as_slice()),
            bytemuck::cast_slice(self.tessellations.as_slice()),
            bytemuck::cast_slice(self.triangles.as_slice()),
            bytemuck::cast_slice(self.images.as_slice()),
            bytemuck::cast_slice(self.draws.as_slice()),
            self.gradients.as_slice(),
        ] {
            for &byte in bytes {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        hash
    }

    fn written_bytes(&self) -> usize {
        self.paths.len().saturating_mul(size_of::<gpu::PathData>())
            + self
                .paints
                .len()
                .saturating_mul(size_of::<TypedPaintRecord>())
            + self
                .contours
                .len()
                .saturating_mul(size_of::<gpu::ContourData>())
            + self
                .tessellations
                .len()
                .saturating_mul(size_of::<gpu::TessVertexSpan>())
            + self
                .triangles
                .len()
                .saturating_mul(size_of::<gpu::TriangleVertex>())
            + self.images.len().saturating_mul(size_of::<u64>())
            + self
                .draws
                .len()
                .saturating_mul(size_of::<TypedDrawRecord>())
            + self.gradients.len()
    }

    fn nonempty_buffer_count(&self) -> usize {
        [
            self.paths.is_empty(),
            self.paints.is_empty(),
            self.contours.is_empty(),
            self.tessellations.is_empty(),
            self.triangles.is_empty(),
            self.images.is_empty(),
            self.draws.is_empty(),
            self.gradients.is_empty(),
        ]
        .into_iter()
        .filter(|is_empty| !*is_empty)
        .count()
    }
}

fn grow_count(current: usize, required: usize) -> usize {
    if required <= current {
        current
    } else {
        required.saturating_mul(5).div_ceil(4)
    }
}

fn reserve_records(buffer: &mut Vec<u8>, records: usize, stride: usize) {
    let required = records.saturating_mul(stride);
    if buffer.capacity() < required {
        // Vec::reserve_exact is relative to length, not capacity. Logical
        // buffers are rewound before growth, so reserve the complete required
        // range instead of only the difference from retained capacity.
        buffer.reserve_exact(required.saturating_sub(buffer.len()));
    }
}

fn reserve_typed<T>(buffer: &mut Vec<T>, records: usize) {
    if buffer.capacity() < records {
        buffer.reserve_exact(records.saturating_sub(buffer.len()));
    }
}

#[derive(Clone, Default)]
pub(crate) struct LogicalResourceStore {
    buffers: Arc<Mutex<ShadowBuffers>>,
    production_frame_writes: Arc<AtomicUsize>,
}

pub(crate) struct PreparedLogicalFrameResources {
    report: LogicalFrameReport,
    gradient_flushes: SmallVec<[PreparedGradientFlush; 1]>,
    typed_address: usize,
    typed_draws: Vec<PreparedTypedDrawSlot>,
}

struct PreparedTypedDrawSlot {
    occurrence_id: u64,
    resources: Option<PreparedTypedDrawResources>,
    consumption: AtomicUsize,
}

#[derive(Clone)]
pub(crate) struct PreparedTypedDrawResources {
    pub(crate) contour_base: u32,
    pub(crate) path: gpu::PathData,
    pub(crate) paint: gpu::PaintData,
    pub(crate) paint_aux: gpu::PaintAuxData,
    pub(crate) spans: Vec<gpu::TessVertexSpan>,
    pub(crate) contours: Vec<gpu::ContourData>,
    pub(crate) triangles: Vec<gpu::TriangleVertex>,
    pub(crate) base_instance: u32,
    pub(crate) instance_count: u32,
    pub(crate) triangle_count: usize,
    pub(crate) borrowed_triangle_count: usize,
    pub(crate) main_triangle_batches:
        Vec<super::clockwise_atomic_pipeline::ClockwiseAtomicTriangleBatch>,
    pub(crate) has_interior_triangles: bool,
    pub(crate) uses_interior: bool,
    // Whether the local contour IDs behind `spans`/`contours` are exactly
    // 1..=contours.len(). Stroke tessellation skips empty butt-cap contours,
    // so consumers must not infer density from the contour count.
    pub(crate) local_contour_ids_are_dense: bool,
}

#[derive(Clone, Copy)]
#[cfg(any(test, feature = "native-metal-experimental"))]
pub(crate) struct RasterOrderingAtlasInput<'a> {
    pub(crate) path: &'a LogicalPath,
    pub(crate) paint: &'a LogicalPaint,
    pub(crate) state: DrawState,
}

#[cfg(any(test, feature = "native-metal-experimental"))]
pub(crate) struct PreparedRasterOrderingAtlasDraw {
    /// Index in the caller's authored input slice. Atlas resource emission can
    /// reorder draws while path/paint records and final blits stay authored.
    #[cfg(test)]
    pub(crate) input_index: usize,
    #[cfg(test)]
    pub(crate) path_id: u16,
    #[cfg(test)]
    pub(crate) atlas_placement: super::AtlasPlacement,
    pub(crate) is_stroke: bool,
    #[cfg(test)]
    pub(crate) scissor: [u16; 4],
    #[cfg(test)]
    pub(crate) base_patch: u32,
    #[cfg(test)]
    pub(crate) patch_count: u32,
    #[cfg(test)]
    pub(crate) blit_vertex_range: std::ops::Range<usize>,
}

#[cfg(any(test, feature = "native-metal-experimental"))]
pub(crate) struct PreparedRasterOrderingAtlasFlush {
    pub(crate) paths: Vec<gpu::PathData>,
    pub(crate) paints: Vec<gpu::PaintData>,
    pub(crate) paint_aux: Vec<gpu::PaintAuxData>,
    pub(crate) spans: Vec<gpu::TessVertexSpan>,
    pub(crate) contours: Vec<gpu::ContourData>,
    pub(crate) triangles: Vec<gpu::TriangleVertex>,
    /// C++ atlas-pass order: fill before stroke, and unscissored before
    /// scissored within each style. Authored identity is retained explicitly.
    pub(crate) draws: Vec<PreparedRasterOrderingAtlasDraw>,
    pub(crate) fill_batches: Vec<gpu::AtlasDrawBatch>,
    pub(crate) stroke_batches: Vec<gpu::AtlasDrawBatch>,
    pub(crate) content_extent: [u32; 2],
    pub(crate) physical_extent: [u32; 2],
}

pub(crate) struct PreparedTypedDrawSelection<'a> {
    resources: &'a PreparedLogicalFrameResources,
    requested: Option<&'a [SolidDraw]>,
}

impl PreparedTypedDrawSelection<'_> {
    pub(crate) fn draw(&self, index: usize) -> Option<&PreparedTypedDrawResources> {
        let resource_index = match self.requested {
            None => index,
            Some(draws) => {
                let occurrence_id = draws[index].logical_occurrence_id;
                self.resources
                    .typed_draws
                    .iter()
                    .position(|draw| draw.occurrence_id == occurrence_id)
                    .expect("encoder requested typed resources outside the finalized logical frame")
            }
        };
        let resources = self.resources.typed_draws[resource_index]
            .resources
            .as_ref()?;
        let state = &self.resources.typed_draws[resource_index].consumption;
        match state.compare_exchange(0, 1, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => {}
            Err(1) => state.store(2, Ordering::Relaxed),
            Err(2) => {}
            Err(actual) => {
                panic!("typed logical output has an invalid consumption state {actual}")
            }
        }
        Some(resources)
    }
}

pub(crate) struct PreparedGradientSelection<'a> {
    batch: Cow<'a, GradientBatch>,
    requested: Option<&'a [SolidDraw]>,
}

impl PreparedGradientSelection<'_> {
    pub(crate) fn spans(&self) -> &[gpu::GradientSpan] {
        &self.batch.spans
    }

    pub(crate) fn height(&self) -> u32 {
        self.batch.height
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.batch.is_empty()
    }

    pub(crate) fn draw(&self, index: usize) -> Option<PreparedGradient> {
        self.requested.map_or_else(
            || self.batch.draw(index),
            |draws| {
                self.batch
                    .draw_for_occurrence(draws[index].logical_occurrence_id)
            },
        )
    }
}

struct PreparedGradientFlush {
    address: usize,
    draw_count: usize,
    batch: GradientBatch,
}

impl PreparedLogicalFrameResources {
    pub(crate) fn into_report(mut self) -> LogicalFrameReport {
        apply_typed_consumption_to_report(&mut self.report, &self.typed_draws);
        self.report
    }

    pub(crate) fn gradient_batch<'a>(
        &'a self,
        draws: &'a [SolidDraw],
    ) -> PreparedGradientSelection<'a> {
        let address = draws.as_ptr() as usize;
        let matched = self
            .gradient_flushes
            .iter()
            .find(|flush| flush.draw_count == draws.len() && flush.address == address);
        if let Some(flush) = matched {
            return PreparedGradientSelection {
                batch: Cow::Borrowed(&flush.batch),
                requested: None,
            };
        }

        // Solid-only flushes have no per-draw gradient table and are
        // interchangeable: every requested draw resolves to no gradient.
        if draws.iter().all(|draw| draw.paint.shader.is_none()) {
            if let Some(flush) = self
                .gradient_flushes
                .iter()
                .find(|flush| flush.batch.draws.is_empty())
            {
                return PreparedGradientSelection {
                    batch: Cow::Borrowed(&flush.batch),
                    requested: None,
                };
            }
        }

        for flush in &self.gradient_flushes {
            if draws
                .iter()
                .all(|draw| flush.batch.contains_occurrence(draw.logical_occurrence_id))
            {
                return PreparedGradientSelection {
                    batch: Cow::Borrowed(&flush.batch),
                    requested: Some(draws),
                };
            }
        }

        panic!(
            "encoder requested gradient inputs outside one finalized logical flush; \
             production gradient normalization must run exactly once per flush"
        )
    }

    pub(crate) fn typed_draws<'a>(
        &'a self,
        draws: &'a [SolidDraw],
    ) -> PreparedTypedDrawSelection<'a> {
        if draws.as_ptr() as usize == self.typed_address && draws.len() == self.typed_draws.len() {
            return PreparedTypedDrawSelection {
                resources: self,
                requested: None,
            };
        }

        assert!(
            draws.iter().all(|requested| self
                .typed_draws
                .iter()
                .any(|draw| draw.occurrence_id == requested.logical_occurrence_id)),
            "encoder requested typed resources outside the finalized logical frame"
        );
        PreparedTypedDrawSelection {
            resources: self,
            requested: Some(draws),
        }
    }

    pub(crate) fn consume_all_for_null(&self) {
        for (index, draw) in self.typed_draws.iter().enumerate() {
            if draw.resources.is_some() {
                draw.consumption
                    .compare_exchange(0, 1, Ordering::Relaxed, Ordering::Relaxed)
                    .unwrap_or_else(|state| {
                        panic!(
                            "null adapter consumed typed logical output occurrence {index} twice (state {state})"
                        )
                    });
            }
        }
    }

    pub(crate) fn report_with_consumption(&self) -> LogicalFrameReport {
        #[cfg(test)]
        PREPARED_LOGICAL_FRAME_AUXILIARY_HEAP_BACKINGS
            .with(|backings| backings.set(backings.get() + 1));
        let mut report = self.report.clone();
        apply_typed_consumption_to_report(&mut report, &self.typed_draws);
        report
    }
}

fn apply_typed_consumption_to_report(
    report: &mut LogicalFrameReport,
    typed_draws: &[PreparedTypedDrawSlot],
) {
    let mut consumed_once = 0;
    let mut exact = report.production_typed_output_eligible_draws != 0;
    for draw in typed_draws {
        if draw.resources.is_none() {
            continue;
        }
        let state = draw.consumption.load(Ordering::Relaxed);
        consumed_once += usize::from(state == 1);
        exact &= state == 1;
    }
    report.production_typed_output_consumed_draws = consumed_once;
    report.production_typed_output_consumed = exact;
}

impl LogicalResourceStore {
    pub(crate) fn prepare(&self, frame: &LogicalFrame) -> Result<LogicalFrameReport, &'static str> {
        self.prepare_frame(frame, false)
            .map(|prepared| prepared.report)
    }

    pub(crate) fn prepare_for_production(
        &self,
        frame: &LogicalFrame,
    ) -> Result<PreparedLogicalFrameResources, &'static str> {
        if !frame.is_finalized() {
            return Err("production logical frame must be finalized");
        }
        let prepared = self.prepare_frame(frame, true)?;
        self.production_frame_writes.fetch_add(1, Ordering::Relaxed);
        Ok(prepared)
    }

    pub(crate) fn prepare_for_production_with_diagnostics(
        &self,
        frame: &LogicalFrame,
    ) -> Result<PreparedLogicalFrameResources, &'static str> {
        if !frame.is_finalized() {
            return Err("production logical frame must be finalized");
        }
        let prepared = self.prepare_frame(frame, false)?;
        self.production_frame_writes.fetch_add(1, Ordering::Relaxed);
        Ok(prepared)
    }

    #[cfg(test)]
    pub(crate) fn production_frame_write_count(&self) -> usize {
        self.production_frame_writes.load(Ordering::Relaxed)
    }

    fn prepare_frame(
        &self,
        frame: &LogicalFrame,
        production: bool,
    ) -> Result<PreparedLogicalFrameResources, &'static str> {
        frame.validate_structure()?;
        let config = frame.config;
        let mut flushes = Vec::with_capacity(frame.logical_flush_starts.len().max(1));
        let starts = if frame.logical_flush_starts.is_empty() {
            &[0][..]
        } else {
            &frame.logical_flush_starts
        };
        let mut written = LogicalResourceCounts::default();
        for (index, &start) in starts.iter().enumerate() {
            let end = starts.get(index + 1).copied().unwrap_or(frame.draws.len());
            let counters = frame.draw_resources[start..end]
                .iter()
                .try_fold(logical_flush::ResourceCounters::default(), |total, draw| {
                    total.checked_add(*draw)
                })
                .ok_or("logical frame resource accounting overflow")?;
            let counts = LogicalResourceCounts::from_flush(counters, &frame.draws[start..end]);
            written = written
                .checked_add(counts)
                .ok_or("logical frame resource layout overflow")?;
            flushes.push(counts);
        }

        let mut buffers = self
            .buffers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        buffers.rewind();
        let mut allocation_growths = 0;
        let mut buffer_write_operations = 0;
        let mut written_bytes = 0;
        let mut fingerprint = 0xcbf2_9ce4_8422_2325_u64;
        // Production frames have already materialized their atomic admission
        // decisions during finalization. Diagnostic callers are also allowed
        // to inspect an unfinalized frame, so derive the same decisions here
        // instead of indexing the intentionally empty retained vector.
        let diagnostic_atomic_batch_flags = (!frame.is_finalized()).then(|| {
            let mut flags = Vec::new();
            populate_production_atomic_batch_flags(
                config,
                &frame.draws,
                &frame.logical_flush_starts,
                &mut flags,
            );
            flags
        });
        let atomic_batch_flags = diagnostic_atomic_batch_flags
            .as_deref()
            .unwrap_or(&frame.atomic_batch_flags);
        let mut gradient_flushes = SmallVec::<[PreparedGradientFlush; 1]>::new();
        let mut typed_draws = Vec::with_capacity(frame.draws.len());
        for (flush_index, (&start, counts)) in starts.iter().zip(&flushes).enumerate() {
            let end = starts
                .get(flush_index + 1)
                .copied()
                .unwrap_or(frame.draws.len());
            let before_capacities = buffers.buffer_capacities();
            buffers.grow(*counts);
            let draws = &frame.draws[start..end];
            let gradient_batch = prepare_gradient_batch(draws);
            let gradient_selection = PreparedGradientSelection {
                batch: Cow::Borrowed(&gradient_batch),
                requested: None,
            };
            write_resources(
                &mut buffers,
                config,
                production,
                flush_index,
                *counts,
                &frame.draws[start..end],
                &frame.draw_resources[start..end],
                &atomic_batch_flags[start..end],
                &gradient_selection,
                &mut typed_draws,
            );
            gradient_flushes.push(PreparedGradientFlush {
                address: draws.as_ptr() as usize,
                draw_count: draws.len(),
                batch: gradient_batch,
            });
            written_bytes += buffers.written_bytes();
            buffer_write_operations += buffers.nonempty_buffer_count();
            allocation_growths += before_capacities
                .into_iter()
                .zip(buffers.buffer_capacities())
                .filter(|(before, after)| before != after)
                .count();
            // Fingerprinting is a correctness diagnostic, not production CPU
            // resource preparation. Keep it out of Null benchmark timings and
            // WGPU finish while retaining the exact same typed writes.
            if !production {
                fingerprint = buffers.fingerprint_into(fingerprint);
            }
            buffers.rewind();
        }
        #[cfg(test)]
        if gradient_flushes.spilled() {
            PREPARED_LOGICAL_FRAME_AUXILIARY_HEAP_BACKINGS
                .with(|backings| backings.set(backings.get() + 1));
        }
        let raster_ordering = (config.mode == RenderMode::RasterOrdering).then(|| {
            let feather_atlas_draws = frame
                .draws
                .iter()
                .filter(|draw| {
                    draw.paint.feather != 0.0
                        && draw::feather_requires_atlas(
                            draw.paint.feather,
                            draw.state.transform,
                            false,
                        )
                })
                .count();
            let path_draws = frame
                .draws
                .iter()
                .filter(|draw| draw.image.is_none())
                .count();
            RasterOrderingPlan {
                pixel_local_storage_coverage: true,
                pixel_local_storage_draws: path_draws.saturating_sub(feather_atlas_draws),
                feather_atlas_draws,
                authored_draw_order_preserved: true,
                interlock_barriers: 0,
                transient_backing_planes: 3,
                draw_passes: written.draw_records,
            }
        });
        let report = LogicalFrameReport {
            mode: config.mode,
            raster_ordering,
            draw_count: frame.draws.len(),
            resource_planning_passes: frame.resource_planning_evaluations,
            plan_finalization_passes: frame.finalization_passes,
            logical_flushes: flushes,
            retained_capacity: buffers.capacity,
            written,
            allocation_growths,
            buffer_write_operations,
            buffer_rewinds: starts.len(),
            written_bytes,
            shadow_fingerprint: (!production).then_some(fingerprint).unwrap_or(0),
            production_typed_output_eligible_draws: typed_draws
                .iter()
                .filter(|draw| draw.resources.is_some())
                .count(),
            production_typed_output_consumed_draws: 0,
            production_fallback_draws: typed_draws
                .iter()
                .filter(|draw| draw.resources.is_none())
                .count(),
            production_typed_output_consumed: false,
        };
        Ok(PreparedLogicalFrameResources {
            report,
            gradient_flushes,
            typed_address: frame.draws.as_ptr() as usize,
            typed_draws,
        })
    }
}

pub(crate) fn prepare_gradient_batch(draws: &[SolidDraw]) -> GradientBatch {
    #[cfg(test)]
    GRADIENT_BATCH_PREPARATIONS.with(|preparations| preparations.set(preparations.get() + 1));

    // Preserve C++'s sparse solid-paint shape: allocate the per-draw table
    // only after the first authored gradient appears.
    let mut definitions = Vec::new();
    for (draw_index, draw) in draws.iter().enumerate() {
        let shader = draw.paint.shader.as_ref();
        if definitions.is_empty() {
            let Some(shader) = shader else {
                continue;
            };
            definitions.reserve(draws.len());
            definitions.extend(
                draws[..draw_index]
                    .iter()
                    .map(|prior| (prior.logical_occurrence_id, prior.state.transform, None)),
            );
            definitions.push((
                draw.logical_occurrence_id,
                draw.state.transform,
                normalize_gradient(shader, draw.state.opacity),
            ));
        } else {
            definitions.push((
                draw.logical_occurrence_id,
                draw.state.transform,
                shader.and_then(|shader| normalize_gradient(shader, draw.state.opacity)),
            ));
        }
    }
    if definitions.is_empty() {
        return GradientBatch {
            spans: Vec::new(),
            height: 0,
            draws: Vec::new(),
        };
    }
    prepare_normalized_gradient_batch(definitions)
}

#[cfg(any(test, feature = "native-metal-experimental"))]
pub(crate) fn prepare_single_gradient_batch(
    shader: &super::LogicalShader,
    opacity: f32,
    transform: Mat2D,
) -> Option<GradientBatch> {
    let gradient = normalize_gradient(shader, opacity)?;
    Some(prepare_normalized_gradient_batch(vec![(
        0,
        transform,
        Some(gradient),
    )]))
}

fn prepare_normalized_gradient_batch(
    definitions: Vec<(u64, Mat2D, Option<GradientDefinition>)>,
) -> GradientBatch {
    const RAMPS_PER_SIMPLE_ROW: usize = gradient_pipeline::TEXTURE_WIDTH as usize / 2;
    const ONE_TEXEL_FIXED: u32 = 65_536 / gradient_pipeline::TEXTURE_WIDTH;
    const LEFT_BORDER: u32 = 0x8000_0000;
    const RIGHT_BORDER: u32 = 0x4000_0000;
    const COMPLEX_BORDER: u32 = 0x2000_0000;

    let is_simple = |gradient: &GradientDefinition| {
        gradient.stops.len() == 1
            || (gradient.stops.len() == 2 && gradient.stops[0] == 0.0 && gradient.stops[1] == 1.0)
    };
    let simple_count = definitions
        .iter()
        .filter_map(|(_, _, gradient)| gradient.as_ref())
        .filter(|gradient| is_simple(gradient))
        .count();
    let complex_count = definitions
        .iter()
        .filter_map(|(_, _, gradient)| gradient.as_ref())
        .filter(|gradient| !is_simple(gradient))
        .count();
    let simple_height = simple_count.div_ceil(RAMPS_PER_SIMPLE_ROW) as u32;
    let height = simple_height + complex_count as u32;
    let mut simple_index = 0usize;
    let mut complex_index = 0u32;
    let mut spans = Vec::new();
    let mut prepared = Vec::with_capacity(definitions.len());
    for (occurrence_id, transform, gradient) in definitions {
        let Some(gradient) = gradient else {
            prepared.push(PreparedGradientDraw {
                occurrence_id,
                gradient: None,
            });
            continue;
        };
        let (row, texture_span) = if is_simple(&gradient) {
            let row = (simple_index / RAMPS_PER_SIMPLE_ROW) as u32;
            let left = ((simple_index % RAMPS_PER_SIMPLE_ROW) * 2) as u32;
            let center_fixed = (left + 1) * ONE_TEXEL_FIXED;
            let color0 = gradient.colors[0];
            let color1 = gradient.colors.get(1).copied().unwrap_or(color0);
            spans.push(gpu::GradientSpan::new(
                center_fixed,
                center_fixed,
                row,
                LEFT_BORDER | RIGHT_BORDER,
                color0,
                color1,
            ));
            simple_index += 1;
            (
                row,
                [
                    1.0 / gradient_pipeline::TEXTURE_WIDTH as f32,
                    (left as f32 + 0.5) / gradient_pipeline::TEXTURE_WIDTH as f32,
                ],
            )
        } else {
            let row = simple_height + complex_index;
            let scale = (gradient_pipeline::TEXTURE_WIDTH - 1) as f32 * ONE_TEXEL_FIXED as f32;
            let bias = 0.5 * ONE_TEXEL_FIXED as f32;
            let mut last_x = (gradient.stops[0] * scale + bias) as u32;
            let mut last_color = gradient.colors[0];
            for index in 1..gradient.stops.len() {
                let x = (gradient.stops[index] * scale + bias) as u32;
                let mut flags = COMPLEX_BORDER;
                if index == 1 {
                    flags |= LEFT_BORDER;
                }
                if index + 1 == gradient.stops.len() {
                    flags |= RIGHT_BORDER;
                }
                spans.push(gpu::GradientSpan::new(
                    last_x,
                    x,
                    row,
                    flags,
                    last_color,
                    gradient.colors[index],
                ));
                last_x = x;
                last_color = gradient.colors[index];
            }
            complex_index += 1;
            (
                row,
                [
                    (gradient_pipeline::TEXTURE_WIDTH - 1) as f32
                        / gradient_pipeline::TEXTURE_WIDTH as f32,
                    0.5 / gradient_pipeline::TEXTURE_WIDTH as f32,
                ],
            )
        };
        let inverse = super::invert(transform).unwrap_or(Mat2D([0.0; 6]));
        let gradient_matrix = match gradient.paint_type {
            gpu::PaintType::LinearGradient => Mat2D([
                gradient.coeffs[0],
                0.0,
                gradient.coeffs[1],
                0.0,
                gradient.coeffs[2],
                0.0,
            ]),
            gpu::PaintType::RadialGradient => {
                let inverse_radius = gradient.coeffs[2].recip();
                Mat2D([
                    inverse_radius,
                    0.0,
                    0.0,
                    inverse_radius,
                    -gradient.coeffs[0] * inverse_radius,
                    -gradient.coeffs[1] * inverse_radius,
                ])
            }
            _ => unreachable!(),
        };
        prepared.push(PreparedGradientDraw {
            occurrence_id,
            gradient: Some(PreparedGradient {
                paint_type: gradient.paint_type,
                texture_y: (row as f32 + 0.5) / height as f32,
                matrix: multiply(gradient_matrix, inverse),
                texture_span,
            }),
        });
    }
    GradientBatch {
        spans,
        height,
        draws: prepared,
    }
}

pub(crate) fn normalize_gradient(
    shader: &super::LogicalShader,
    opacity: f32,
) -> Option<GradientDefinition> {
    const EPSILON: f32 = 1.0 / 4096.0;
    let (paint_type, mut colors, stops, coeffs) = match shader {
        super::LogicalShader::Linear {
            start,
            end,
            colors,
            stops,
        } => {
            let mut start = *start;
            let mut end = *end;
            let mut stops = stops.clone();
            validate_gradient(colors, &stops)?;
            let first = stops[0];
            let last = *stops.last()?;
            if (first != 0.0 || last != 1.0) && last - first > EPSILON {
                let original_start = start;
                let original_end = end;
                start = (
                    original_start.0 + (original_end.0 - original_start.0) * first,
                    original_start.1 + (original_end.1 - original_start.1) * first,
                );
                end = (
                    original_start.0 + (original_end.0 - original_start.0) * last,
                    original_start.1 + (original_end.1 - original_start.1) * last,
                );
                let inverse_range = (last - first).recip();
                for stop in &mut stops {
                    *stop = (*stop - first) * inverse_range;
                }
                stops[0] = 0.0;
                *stops.last_mut().unwrap() = 1.0;
                let final_index = stops.len() - 1;
                for index in 1..final_index {
                    stops[index] = stops[index].max(stops[index - 1]);
                }
                for index in (1..final_index).rev() {
                    stops[index] = stops[index].min(stops[index + 1]);
                }
            }
            let dx = end.0 - start.0;
            let dy = end.1 - start.1;
            let inverse_length_squared = (dx * dx + dy * dy).recip();
            let vx = dx * inverse_length_squared;
            let vy = dy * inverse_length_squared;
            (
                gpu::PaintType::LinearGradient,
                colors.clone(),
                stops,
                [vx, vy, -(vx * start.0 + vy * start.1)],
            )
        }
        super::LogicalShader::Radial {
            center,
            radius,
            colors,
            stops,
        } => {
            let mut radius = *radius;
            let mut stops = stops.clone();
            validate_gradient(colors, &stops)?;
            let last = *stops.last()?;
            if last != 1.0 && last > EPSILON {
                radius *= last;
                let inverse_last = last.recip();
                let final_index = stops.len() - 1;
                for stop in &mut stops[..final_index] {
                    *stop *= inverse_last;
                }
                *stops.last_mut().unwrap() = 1.0;
                stops[0] = stops[0].max(0.0);
                for index in 1..final_index {
                    stops[index] = stops[index].max(stops[index - 1]);
                }
                for index in (0..final_index).rev() {
                    stops[index] = stops[index].min(stops[index + 1]);
                }
            }
            (
                gpu::PaintType::RadialGradient,
                colors.clone(),
                stops,
                [center.0, center.1, radius],
            )
        }
    };
    for color in &mut colors {
        *color = super::modulate_color_alpha(*color, opacity);
    }
    Some(GradientDefinition {
        paint_type,
        colors,
        stops,
        coeffs,
    })
}

fn validate_gradient(colors: &[ColorInt], stops: &[f32]) -> Option<()> {
    if colors.len() != stops.len()
        || stops.is_empty()
        || stops
            .iter()
            .any(|stop| !stop.is_finite() || !(0.0..=1.0).contains(stop))
        || stops.windows(2).any(|pair| pair[0] > pair[1])
    {
        return None;
    }
    Some(())
}

fn populate_production_atomic_batch_flags(
    config: LogicalFrameConfig,
    draws: &[SolidDraw],
    logical_flush_starts: &[usize],
    flags: &mut Vec<bool>,
) {
    flags.clear();
    flags.resize(draws.len(), false);
    if config.mode != RenderMode::ClockwiseAtomic {
        return;
    }
    let advanced_segments = draws
        .iter()
        .any(super::draw_uses_advanced_blend)
        .then(|| super::AdvancedAtomicSegmentPlan::new(draws));
    let mut start = 0;
    let mut logical_flush_index = 0;
    while start < draws.len() {
        let logical_flush_end = logical_flush_starts
            .get(logical_flush_index + 1)
            .copied()
            .unwrap_or(draws.len());
        let atomic = super::atomic_draw_is_eligible(&draws[start]);
        let advanced_end = advanced_segments
            .as_ref()
            .and_then(|plan| plan.segment_end(start, logical_flush_end));
        let clockwise_atomic = super::WEBGPU_SUPPORTS_CLOCKWISE_ATOMIC_MODE
            && atomic
            && advanced_end.is_none()
            && super::draw_requires_clockwise_atomic(&draws[start], config.width, config.height);
        let end = advanced_end
            .unwrap_or_else(|| super::atomic_strategy_run_end(draws, start, logical_flush_end));
        flags[start..end].fill(clockwise_atomic);
        start = end;
        if start == logical_flush_end {
            logical_flush_index += 1;
        }
    }
}

fn write_resources(
    buffers: &mut ShadowBuffers,
    config: LogicalFrameConfig,
    production: bool,
    flush_index: usize,
    counts: LogicalResourceCounts,
    draws: &[SolidDraw],
    draw_resources: &[logical_flush::ResourceCounters],
    atomic_batch_flags: &[bool],
    gradient_batch: &PreparedGradientSelection<'_>,
    prepared: &mut Vec<PreparedTypedDrawSlot>,
) {
    let _ = (flush_index, counts);
    let raster_ordering_atlas_placements = raster_ordering_feather_atlas_placements(config, draws);
    push_u32(&mut buffers.gradients, gradient_batch.height());
    push_usize(&mut buffers.gradients, gradient_batch.spans().len());
    for span in gradient_batch.spans() {
        for value in [
            span.horizontal_span,
            span.y_with_flags,
            span.color0,
            span.color1,
        ] {
            push_u32(&mut buffers.gradients, value);
        }
    }
    for (draw_index, (((draw, resources), &use_clockwise_atomic_batch), atlas_placement)) in draws
        .iter()
        .zip(draw_resources)
        .zip(atomic_batch_flags)
        .zip(
            raster_ordering_atlas_placements
                .into_iter()
                .chain(std::iter::repeat(None))
                .take(draws.len()),
        )
        .enumerate()
    {
        let path_start = buffers.paths.len();
        let paint_start = buffers.paints.len();
        let contour_start = buffers.contours.len();
        let tessellation_start = buffers.tessellations.len();
        let triangle_start = buffers.triangles.len();
        let (local_contour_ids_are_dense, direct_msaa_tessellation_selected) =
            write_typed_draw_resources(
                buffers,
                config,
                production,
                draw_index,
                draw,
                use_clockwise_atomic_batch,
                gradient_batch,
                atlas_placement,
            );
        let backend_fallback =
            typed_draw_uses_backend_fallback(config, draw, use_clockwise_atomic_batch);
        let prepared_resources =
            (buffers.paths.len() != path_start && !backend_fallback).then(|| {
                debug_assert_eq!(buffers.paths.len(), path_start + 1);
                debug_assert_eq!(buffers.paints.len(), paint_start + 1);
                let paint = buffers.paints[paint_start];
                let metadata = typed_draw_metadata(draw, config, use_clockwise_atomic_batch);
                let borrow_direct_msaa_tessellation = direct_msaa_tessellation_selected
                    && production_msaa_uses_retained_tessellation(production, config, draw);
                #[cfg(test)]
                if !borrow_direct_msaa_tessellation
                    && (buffers.tessellations.len() != tessellation_start
                        || buffers.contours.len() != contour_start
                        || buffers.triangles.len() != triangle_start)
                {
                    PREPARED_TYPED_TESSELLATION_VECTOR_COPIES
                        .with(|copies| copies.set(copies.get() + 1));
                }
                PreparedTypedDrawResources {
                    contour_base: u32::try_from(contour_start).expect("contour base overflow"),
                    path: buffers.paths[path_start],
                    paint: paint.paint,
                    paint_aux: paint.aux,
                    spans: (!borrow_direct_msaa_tessellation)
                        .then(|| buffers.tessellations[tessellation_start..].to_vec())
                        .unwrap_or_default(),
                    contours: (!borrow_direct_msaa_tessellation)
                        .then(|| buffers.contours[contour_start..].to_vec())
                        .unwrap_or_default(),
                    triangles: (!borrow_direct_msaa_tessellation)
                        .then(|| buffers.triangles[triangle_start..].to_vec())
                        .unwrap_or_default(),
                    base_instance: metadata.base_instance,
                    instance_count: metadata.instance_count,
                    triangle_count: metadata.triangle_count,
                    borrowed_triangle_count: metadata.borrowed_triangle_count,
                    main_triangle_batches: metadata.main_triangle_batches,
                    has_interior_triangles: metadata.has_interior_triangles,
                    uses_interior: metadata.uses_interior,
                    local_contour_ids_are_dense,
                }
            });
        prepared.push(PreparedTypedDrawSlot {
            occurrence_id: draw.logical_occurrence_id,
            resources: prepared_resources,
            consumption: AtomicUsize::new(0),
        });
        match gradient_batch.draw(draw_index) {
            None => push_u8(&mut buffers.gradients, 0),
            Some(gradient) => {
                push_u8(&mut buffers.gradients, 1);
                push_u8(&mut buffers.gradients, paint_type_tag(gradient.paint_type));
                push_u32(&mut buffers.gradients, gradient.texture_y.to_bits());
                for value in gradient.matrix.0.into_iter().chain(gradient.texture_span) {
                    push_u32(&mut buffers.gradients, value.to_bits());
                }
            }
        }
        buffers
            .images
            .extend(std::iter::repeat_n(1, resources.image_draw_count));
        let (role, primary_id, secondary_id, bounds) = match draw.role {
            DrawRole::Content { clip_id } => (0, u32::from(clip_id), 0, [0.0; 4]),
            DrawRole::ClipUpdate {
                replacement_id,
                parent_id,
            } => (1, u32::from(replacement_id), u32::from(parent_id), [0.0; 4]),
            DrawRole::ClipReset { bounds, action } => {
                (2, u32::from(clip_reset_action_tag(action)), 0, bounds)
            }
        };
        buffers.draws.push(TypedDrawRecord {
            role,
            primary_id,
            secondary_id,
            draw_pass_count: u32::try_from(resources.draw_pass_count)
                .expect("draw pass count overflow"),
            bounds,
            transform: draw.state.transform.0,
            fill_rule: match draw.path.fill_rule {
                FillRule::NonZero => 0,
                FillRule::EvenOdd => 1,
                FillRule::Clockwise => 2,
            },
            local_contour_ids_are_dense: u32::from(local_contour_ids_are_dense),
        });
    }
}

fn raster_ordering_feather_atlas_placements(
    config: LogicalFrameConfig,
    draws: &[SolidDraw],
) -> Vec<Option<super::AtlasPlacement>> {
    if config.mode != RenderMode::RasterOrdering {
        return Vec::new();
    }
    let placements = draws
        .iter()
        .map(|draw| {
            (draw.paint.feather != 0.0
                && draw::feather_requires_atlas(draw.paint.feather, draw.state.transform, false))
            .then(|| {
                feather_atlas_placement(
                    &draw.path.raw_path,
                    draw.state.transform,
                    draw.paint.feather,
                    draw.paint.effective_stroke(),
                    config.width,
                    config.height,
                )
                .expect("logical frame admission already validated feather atlas placement")
            })
        })
        .collect::<Vec<_>>();
    #[cfg(test)]
    LOGICAL_FEATHER_ATLAS_PLACEMENT_RECORDS
        .with(|records| records.set(records.get().saturating_add(placements.len())));
    pack_raster_ordering_feather_atlas_placements(config, placements)
}

fn pack_raster_ordering_feather_atlas_placements(
    config: LogicalFrameConfig,
    mut placements: Vec<Option<super::AtlasPlacement>>,
) -> Vec<Option<super::AtlasPlacement>> {
    let draw_sizes = placements
        .iter()
        .filter_map(|placement| {
            placement.map(|placement| {
                (
                    placement.width - FEATHER_ATLAS_PADDING * 2,
                    placement.height - FEATHER_ATLAS_PADDING * 2,
                )
            })
        })
        .collect::<Vec<_>>();
    if draw_sizes.is_empty() {
        return placements;
    }

    let atlas = pack_logical_feather_atlas_for_cpp(config.max_texture_dimension_2d, &draw_sizes)
        .expect("logical frame allocation already validated feather atlas packing");
    let mut packed_regions = atlas.origins().iter().copied().zip(&atlas.region_sizes);
    for placement in placements.iter_mut().flatten() {
        let (origin, &(region_width, region_height)) = packed_regions
            .next()
            .expect("packed feather atlas must include every draw");
        let raw_width = placement.width - FEATHER_ATLAS_PADDING * 2;
        let raw_height = placement.height - FEATHER_ATLAS_PADDING * 2;
        let horizontal_padding = (region_width - raw_width) / 2;
        let vertical_padding = (region_height - raw_height) / 2;
        placement.origin = origin;
        placement.translate[0] += origin[0] as f32;
        placement.translate[1] += origin[1] as f32;
        if horizontal_padding != FEATHER_ATLAS_PADDING {
            placement.translate[0] += horizontal_padding as f32 - FEATHER_ATLAS_PADDING as f32;
        }
        if vertical_padding != FEATHER_ATLAS_PADDING {
            placement.translate[1] += vertical_padding as f32 - FEATHER_ATLAS_PADDING as f32;
        }
        placement.width = region_width;
        placement.height = region_height;
    }
    debug_assert!(packed_regions.next().is_none());
    placements
}

pub(crate) fn typed_draw_uses_backend_fallback(
    config: LogicalFrameConfig,
    draw: &SolidDraw,
    use_clockwise_atomic_batch: bool,
) -> bool {
    matches!(draw.role, DrawRole::ClipReset { .. })
        || draw.image.is_some()
        || (config.mode == RenderMode::ClockwiseAtomic
            && use_clockwise_atomic_batch
            && matches!(draw.role, DrawRole::ClipUpdate { parent_id, .. } if parent_id != 0))
}

struct TypedDrawMetadata {
    base_instance: u32,
    instance_count: u32,
    triangle_count: usize,
    borrowed_triangle_count: usize,
    main_triangle_batches: Vec<super::clockwise_atomic_pipeline::ClockwiseAtomicTriangleBatch>,
    has_interior_triangles: bool,
    uses_interior: bool,
}

fn typed_draw_metadata(
    draw: &SolidDraw,
    config: LogicalFrameConfig,
    use_clockwise_atomic_batch: bool,
) -> TypedDrawMetadata {
    if matches!(
        config.mode,
        RenderMode::RasterOrdering | RenderMode::ClockwiseAtomic
    ) && draw.paint.style == RenderPaintStyle::Fill
        && draw.paint.feather == 0.0
        && draw.authored_should_use_interior()
    {
        let frame_clockwise_override = config.mode == RenderMode::ClockwiseAtomic;
        let clockwise_override = atomic_fill_clockwise_override(draw, frame_clockwise_override);
        if let Some(prepared) = draw.authored_atomic_interior_geometry(
            clockwise_override,
            config.mode == RenderMode::ClockwiseAtomic && use_clockwise_atomic_batch,
            config.mode == RenderMode::ClockwiseAtomic
                && use_clockwise_atomic_batch
                && matches!(draw.role, DrawRole::Content { clip_id: 0 })
                && super::clockwise_atomic_clip_is_inactive(draw),
            1,
        ) {
            return TypedDrawMetadata {
                base_instance: prepared.base_instance,
                instance_count: prepared.instance_count,
                triangle_count: prepared.triangles.len(),
                borrowed_triangle_count: prepared.borrowed_triangles.len(),
                main_triangle_batches: prepared.main_triangles.batches,
                has_interior_triangles: prepared.has_interior_triangles,
                uses_interior: true,
            };
        }
    }

    if matches!(
        config.mode,
        RenderMode::RasterOrdering | RenderMode::ClockwiseAtomic
    ) && draw.paint.style == RenderPaintStyle::Fill
        && draw.paint.feather == 0.0
    {
        if let Some(mut tessellation) = draw.authored_fill_tessellation() {
            let frame_clockwise_override = config.mode == RenderMode::ClockwiseAtomic;
            let clockwise_override = atomic_fill_clockwise_override(draw, frame_clockwise_override);
            tessellation.make_double_sided_with_direction(
                draw.authored_clockwise_atomic_negate_coverage(
                    draw.path.fill_rule,
                    clockwise_override,
                ),
            );
            return TypedDrawMetadata {
                base_instance: tessellation.base_instance,
                instance_count: tessellation.instance_count,
                triangle_count: 0,
                borrowed_triangle_count: 0,
                main_triangle_batches: Vec::new(),
                has_interior_triangles: false,
                uses_interior: false,
            };
        }
    }

    let tessellation = if draw.paint.feather != 0.0 {
        draw.prepared_feather(config.mode)
            .map(|prepared| &prepared.tessellation)
    } else if draw.paint.style == RenderPaintStyle::Stroke {
        draw.prepared_stroke()
            .map(|prepared| &prepared.tessellation)
    } else if config.mode == RenderMode::Msaa && draw.authored_msaa_fill_requires_reverse() {
        draw.prepared_fill().and_then(|prepared| {
            prepared
                .reversed_midpoint(&draw.path, draw.state.transform)
                .map(|prepared| &prepared.tessellation)
        })
    } else {
        draw.prepared_fill().and_then(|prepared| {
            prepared
                .midpoint(&draw.path, draw.state.transform)
                .map(|prepared| &prepared.tessellation)
        })
    };
    let (base_instance, instance_count) = tessellation.map_or((1, 0), |tessellation| {
        (tessellation.base_instance, tessellation.instance_count)
    });
    TypedDrawMetadata {
        base_instance,
        instance_count,
        triangle_count: 0,
        borrowed_triangle_count: 0,
        main_triangle_batches: Vec::new(),
        has_interior_triangles: false,
        uses_interior: false,
    }
}

fn write_typed_draw_resources(
    buffers: &mut ShadowBuffers,
    config: LogicalFrameConfig,
    production: bool,
    draw_index: usize,
    draw: &SolidDraw,
    use_clockwise_atomic_batch: bool,
    gradients: &PreparedGradientSelection<'_>,
    atlas_placement: Option<super::AtlasPlacement>,
) -> (bool, bool) {
    if matches!(draw.role, DrawRole::ClipReset { .. }) || draw.image.is_some() {
        return (false, false);
    }
    let path_id = u16::try_from(draw_index + 1).expect("logical path ID overflow");
    let frame_clockwise_override = config.mode == RenderMode::ClockwiseAtomic;
    let clockwise_override = atomic_fill_clockwise_override(draw, frame_clockwise_override);
    let mut spans = Vec::new();
    let mut contours = Vec::new();
    let mut triangles = Vec::new();
    let mut path = gpu::PathData::zeroed();
    let mut local_contour_ids_are_dense = false;
    let mut direct_msaa_tessellation: Option<(&draw::FillTessellation, bool)> = None;

    if draw.paint.feather != 0.0 {
        if let Some(prepared) = draw.prepared_feather(config.mode) {
            spans.clone_from(&prepared.tessellation.spans);
            contours.clone_from(&prepared.tessellation.contours);
            path = prepared.tessellation.path;
            local_contour_ids_are_dense = prepared.local_contour_ids_are_dense;
        }
    } else if draw.paint.style == RenderPaintStyle::Stroke {
        if let Some(stroke) = draw.prepared_stroke() {
            if config.mode == RenderMode::Msaa {
                direct_msaa_tessellation =
                    Some((&stroke.tessellation, stroke.local_contour_ids_are_dense));
            } else {
                spans.clone_from(&stroke.tessellation.spans);
                contours.clone_from(&stroke.tessellation.contours);
            }
            path = stroke.tessellation.path;
            // Stroke tessellation skips empty butt-cap contours; only the
            // builder knows whether the surviving local IDs stayed dense.
            local_contour_ids_are_dense = stroke.local_contour_ids_are_dense;
        }
    } else if matches!(
        config.mode,
        RenderMode::RasterOrdering | RenderMode::ClockwiseAtomic
    ) && draw.authored_should_use_interior()
    {
        let prepared = draw.authored_atomic_interior_geometry(
            clockwise_override,
            config.mode == RenderMode::ClockwiseAtomic && use_clockwise_atomic_batch,
            config.mode == RenderMode::ClockwiseAtomic
                && use_clockwise_atomic_batch
                && matches!(draw.role, DrawRole::Content { clip_id: 0 })
                && super::clockwise_atomic_clip_is_inactive(draw),
            path_id,
        );
        if let Some(mut prepared) = prepared {
            spans = prepared.spans;
            contours = prepared.contours;
            path = prepared.path;
            triangles = prepared.triangles;
            triangles.append(&mut prepared.borrowed_triangles);
            triangles.append(&mut prepared.main_triangles.vertices);
        }
    } else if config.mode == RenderMode::Msaa {
        let tessellation = if draw.authored_msaa_fill_requires_reverse() {
            draw.prepared_fill()
                .and_then(|prepared| prepared.reversed_midpoint(&draw.path, draw.state.transform))
                .map(|prepared| &prepared.tessellation)
        } else {
            draw.prepared_fill()
                .and_then(|prepared| prepared.midpoint(&draw.path, draw.state.transform))
                .map(|prepared| &prepared.tessellation)
        };
        if let Some(tessellation) = tessellation {
            path = tessellation.path;
            local_contour_ids_are_dense =
                tessellation.contours.len() <= gpu::CONTOUR_ID_MASK as usize;
            direct_msaa_tessellation = Some((tessellation, local_contour_ids_are_dense));
        }
    } else if let Some(mut tessellation) = draw.authored_fill_tessellation() {
        if matches!(
            config.mode,
            RenderMode::RasterOrdering | RenderMode::ClockwiseAtomic
        ) {
            tessellation.make_double_sided_with_direction(
                draw.authored_clockwise_atomic_negate_coverage(
                    draw.path.fill_rule,
                    clockwise_override,
                ),
            );
        }
        spans = tessellation.spans;
        contours = tessellation.contours;
        path = tessellation.path;
        // Fill tessellation emits every contour record with spans in order.
        local_contour_ids_are_dense = contours.len() <= gpu::CONTOUR_ID_MASK as usize;
    }
    if let Some(placement) = atlas_placement {
        path.atlas_transform = gpu::AtlasTransform {
            scale_factor: placement.scale,
            translate_x: placement.translate[0],
            translate_y: placement.translate[1],
        };
        let [left, top, right, bottom] = placement.bounds;
        triangles.extend([
            gpu::TriangleVertex::new([left, bottom], 1, path_id),
            gpu::TriangleVertex::new([left, top], 1, path_id),
            gpu::TriangleVertex::new([right, bottom], 1, path_id),
            gpu::TriangleVertex::new([right, bottom], 1, path_id),
            gpu::TriangleVertex::new([left, top], 1, path_id),
            gpu::TriangleVertex::new([right, top], 1, path_id),
        ]);
    }

    let direct_msaa_is_empty = direct_msaa_tessellation.is_none_or(|(tessellation, _)| {
        tessellation.spans.is_empty() && tessellation.contours.is_empty()
    });
    if spans.is_empty()
        && contours.is_empty()
        && triangles.is_empty()
        && direct_msaa_is_empty
        && draw.paint.style == RenderPaintStyle::Fill
    {
        if let Some(mut tessellation) = draw.authored_fill_tessellation() {
            if matches!(
                config.mode,
                RenderMode::RasterOrdering | RenderMode::ClockwiseAtomic
            ) {
                tessellation.make_double_sided_with_direction(
                    draw.authored_clockwise_atomic_negate_coverage(
                        draw.path.fill_rule,
                        clockwise_override,
                    ),
                );
            }
            spans = tessellation.spans;
            contours = tessellation.contours;
            path = tessellation.path;
            local_contour_ids_are_dense = contours.len() <= gpu::CONTOUR_ID_MASK as usize;
            direct_msaa_tessellation = None;
        }
    }
    let direct_msaa_is_empty = direct_msaa_tessellation.is_none_or(|(tessellation, _)| {
        tessellation.spans.is_empty() && tessellation.contours.is_empty()
    });
    if spans.is_empty() && contours.is_empty() && triangles.is_empty() && direct_msaa_is_empty {
        return (false, false);
    }
    path.z_index = if config.mode == RenderMode::Msaa {
        path_id.into()
    } else {
        0
    };
    if config.mode == RenderMode::ClockwiseAtomic {
        path.coverage_buffer_range.pitch = config.width.div_ceil(32) * 32;
    }
    let contour_offset = u32::try_from(buffers.contours.len()).expect("contour offset overflow");
    for span in &mut spans {
        let local_id = span.contour_id_with_flags & gpu::CONTOUR_ID_MASK;
        if local_id != 0 {
            span.contour_id_with_flags = (span.contour_id_with_flags & !gpu::CONTOUR_ID_MASK)
                | contour_offset.saturating_add(local_id);
        }
    }
    for contour in &mut contours {
        contour.path_id = path_id.into();
    }
    for triangle in &mut triangles {
        triangle.weight_path_id = (triangle.weight_path_id & !0xffff) | i32::from(path_id);
    }

    let fill_rule = if config.mode == RenderMode::ClockwiseAtomic {
        atomic_paint_fill_rule(draw.path.fill_rule, clockwise_override)
    } else {
        draw.path.fill_rule
    };
    let gradient = gradients.draw(draw_index);
    let mut paint = match draw.role {
        DrawRole::ClipUpdate {
            replacement_id,
            parent_id,
        } => {
            if config.mode == RenderMode::Msaa {
                gpu::PaintData::solid(0, draw.path.fill_rule, BlendMode::SrcOver)
            } else {
                gpu::PaintData::clip_update(replacement_id, parent_id, fill_rule)
            }
        }
        DrawRole::Content { clip_id } => {
            let paint = if let Some(gradient) = gradient {
                if draw.paint.style == RenderPaintStyle::Stroke {
                    gpu::PaintData::gradient_stroke(
                        gradient.paint_type,
                        gradient.texture_y,
                        draw.paint.blend_mode,
                    )
                } else {
                    gpu::PaintData::gradient(
                        gradient.paint_type,
                        gradient.texture_y,
                        fill_rule,
                        draw.paint.blend_mode,
                    )
                }
            } else if draw.paint.style == RenderPaintStyle::Stroke {
                gpu::PaintData::solid_stroke(
                    modulate_color_alpha(draw.paint.color, draw.state.opacity),
                    draw.paint.blend_mode,
                )
            } else {
                gpu::PaintData::solid(
                    modulate_color_alpha(draw.paint.color, draw.state.opacity),
                    fill_rule,
                    draw.paint.blend_mode,
                )
            };
            paint.with_clip_id(clip_id)
        }
        DrawRole::ClipReset { .. } => unreachable!(),
    };
    if draw.state.clip_rect.is_some() {
        paint = paint.with_clip_rect();
    }
    if config.mode == RenderMode::ClockwiseAtomic
        && !use_clockwise_atomic_batch
        && draw.paint.style == RenderPaintStyle::Fill
        && clockwise_override
    {
        paint = paint.with_generic_clockwise_fill();
    }
    let aux = gradient.map_or_else(
        || clip_rect_paint_aux(draw.state.clip_rect),
        |gradient| gradient_paint_aux(draw.state.clip_rect, gradient),
    );

    buffers.paths.push(path);
    buffers.paints.push(TypedPaintRecord { paint, aux });
    if let Some((tessellation, _)) = direct_msaa_tessellation {
        if !production_msaa_uses_retained_tessellation(production, config, draw) {
            append_direct_msaa_tessellation_to_shadow_buffers(
                buffers,
                tessellation,
                path_id,
                contour_offset,
                production,
            );
        }
    } else {
        buffers.contours.extend(contours);
        buffers.tessellations.extend(spans);
    }
    buffers.triangles.extend(triangles);
    (
        local_contour_ids_are_dense,
        direct_msaa_tessellation.is_some(),
    )
}

#[cfg(any(test, feature = "native-metal-experimental"))]
fn write_raster_ordering_atlas_resources(
    buffers: &mut ShadowBuffers,
    draw_index: usize,
    path_source: &LogicalPath,
    paint_source: &LogicalPaint,
    state: DrawState,
    prepared: &PreparedFeatherGeometry,
    placement: super::AtlasPlacement,
) -> bool {
    debug_assert_eq!(prepared.mode, RenderMode::RasterOrdering);
    let path_id = u16::try_from(draw_index + 1).expect("logical path ID overflow");
    let contour_offset = u32::try_from(buffers.contours.len()).expect("contour offset overflow");
    let mut spans = prepared.tessellation.spans.clone();
    let mut contours = prepared.tessellation.contours.clone();
    let mut path = prepared.tessellation.path;
    path.atlas_transform = gpu::AtlasTransform {
        scale_factor: placement.scale,
        translate_x: placement.translate[0],
        translate_y: placement.translate[1],
    };
    let [left, top, right, bottom] = placement.bounds;
    let mut triangles = vec![
        gpu::TriangleVertex::new([left, bottom], 1, path_id),
        gpu::TriangleVertex::new([left, top], 1, path_id),
        gpu::TriangleVertex::new([right, bottom], 1, path_id),
        gpu::TriangleVertex::new([right, bottom], 1, path_id),
        gpu::TriangleVertex::new([left, top], 1, path_id),
        gpu::TriangleVertex::new([right, top], 1, path_id),
    ];
    for span in &mut spans {
        let local_id = span.contour_id_with_flags & gpu::CONTOUR_ID_MASK;
        if local_id != 0 {
            span.contour_id_with_flags = (span.contour_id_with_flags & !gpu::CONTOUR_ID_MASK)
                | contour_offset.saturating_add(local_id);
        }
    }
    for contour in &mut contours {
        contour.path_id = u32::from(path_id);
    }
    for triangle in &mut triangles {
        triangle.weight_path_id = (triangle.weight_path_id & !0xffff) | i32::from(path_id);
    }
    path.z_index = 0;
    let mut paint = if paint_source.style == RenderPaintStyle::Stroke {
        gpu::PaintData::solid_stroke(
            modulate_color_alpha(paint_source.color, state.opacity),
            paint_source.blend_mode,
        )
    } else {
        gpu::PaintData::solid(
            modulate_color_alpha(paint_source.color, state.opacity),
            path_source.fill_rule,
            paint_source.blend_mode,
        )
    }
    .with_clip_id(0);
    if state.clip_rect.is_some() {
        paint = paint.with_clip_rect();
    }
    buffers.paths.push(path);
    buffers.paints.push(TypedPaintRecord {
        paint,
        aux: clip_rect_paint_aux(state.clip_rect),
    });
    buffers.contours.extend(contours);
    buffers.tessellations.extend(spans);
    buffers.triangles.extend(triangles);
    prepared.local_contour_ids_are_dense
}

fn append_direct_msaa_tessellation_to_shadow_buffers(
    buffers: &mut ShadowBuffers,
    tessellation: &draw::FillTessellation,
    path_id: u16,
    contour_offset: u32,
    production: bool,
) {
    #[cfg(not(test))]
    let _ = production;
    #[cfg(test)]
    if production {
        PRODUCTION_DIRECT_MSAA_LOGICAL_WRITER_TESSELLATION_COPIES
            .with(|copies| copies.set(copies.get() + 1));
    }
    buffers
        .contours
        .extend(tessellation.contours.iter().copied().map(|mut contour| {
            contour.path_id = path_id.into();
            contour
        }));
    buffers
        .tessellations
        .extend(tessellation.spans.iter().copied().map(|mut span| {
            let local_id = span.contour_id_with_flags & gpu::CONTOUR_ID_MASK;
            if local_id != 0 {
                span.contour_id_with_flags = (span.contour_id_with_flags & !gpu::CONTOUR_ID_MASK)
                    | contour_offset.saturating_add(local_id);
            }
            span
        }));
}

fn production_msaa_uses_retained_tessellation(
    production: bool,
    config: LogicalFrameConfig,
    draw: &SolidDraw,
) -> bool {
    production
        && config.mode == RenderMode::Msaa
        && draw.paint.feather == 0.0
        && draw.image.is_none()
        && matches!(
            draw.paint.style,
            RenderPaintStyle::Fill | RenderPaintStyle::Stroke
        )
}

fn push_u8(buffer: &mut Vec<u8>, value: u8) {
    buffer.push(value);
}

fn push_u32(buffer: &mut Vec<u8>, value: u32) {
    buffer.extend_from_slice(&value.to_le_bytes());
}

fn push_usize(buffer: &mut Vec<u8>, value: usize) {
    buffer.extend_from_slice(&(value as u64).to_le_bytes());
}

fn paint_type_tag(paint_type: gpu::PaintType) -> u8 {
    match paint_type {
        gpu::PaintType::ClipUpdate => 0,
        gpu::PaintType::SolidColor => 1,
        gpu::PaintType::LinearGradient => 2,
        gpu::PaintType::RadialGradient => 3,
        gpu::PaintType::Image => 4,
    }
}

fn clip_reset_action_tag(action: MsaaClipResetAction) -> u8 {
    match action {
        MsaaClipResetAction::ClearPrevious => 0,
        MsaaClipResetAction::IntersectPreviousNonZero => 1,
        MsaaClipResetAction::IntersectPreviousEvenOdd => 2,
        MsaaClipResetAction::IntersectPreviousClockwise => 3,
    }
}

pub(crate) struct AdmittedPathDraw {
    path: LogicalPath,
    paint: LogicalPaint,
    state: DrawState,
    preparation: PathDrawPreparation,
    msaa_feather_atlas: bool,
}

impl AdmittedPathDraw {
    pub(crate) fn finish(
        self,
        clip_id: u16,
        scratch: &mut super::draw::StrokePreparationScratch,
    ) -> Result<SolidDraw, &'static str> {
        let content = SolidDraw::new_with_preparation_using_stroke_scratch(
            self.path,
            self.paint,
            self.state,
            DrawRole::Content { clip_id },
            None,
            self.preparation,
            scratch,
        );
        if clip_id != 0 && !self.msaa_feather_atlas && !super::atomic_draw_is_eligible(&content) {
            return Err("non-rectangular clips on fallback draws");
        }
        Ok(content)
    }
}

pub(crate) fn admit_path_draw(
    config: LogicalFrameConfig,
    state: DrawState,
    path: &LogicalPath,
    paint: &LogicalPaint,
) -> Result<Option<AdmittedPathDraw>, &'static str> {
    #[cfg(test)]
    PATH_DRAW_ADMISSION_EVALUATIONS.with(|evaluations| {
        evaluations.set(evaluations.get().saturating_add(1));
    });
    if !path_draw_has_valid_parameters(path, paint) {
        return Ok(None);
    }
    let Some(pixel_bounds) = path_draw_pixel_bounds(path, paint, state.transform) else {
        return Ok(None);
    };
    let clipped_pixel_bounds =
        intersect_pixel_bounds(pixel_bounds, state.overall_clip_pixel_bounds);
    if pixel_bounds_are_outside_frame(clipped_pixel_bounds, config.width, config.height) {
        return Ok(None);
    }
    let Some(preparation) = prepare_path_draw_with_pixel_bounds(
        path,
        paint,
        state,
        Some(pixel_bounds),
        config.mode,
        config.width,
        config.height,
    ) else {
        return Ok(None);
    };
    let msaa_feather_atlas = config.mode == RenderMode::Msaa && paint.feather != 0.0;
    if msaa_feather_atlas && state.clip_rect.is_some() && !config.msaa_atlas_supports_clip_rect {
        return Err("clip rectangles on msaa feather atlas draws");
    }
    Ok(Some(AdmittedPathDraw {
        path: path.clone(),
        paint: paint.clone(),
        state,
        preparation,
        msaa_feather_atlas,
    }))
}

/// Lightweight canonical-equivalent resources for one native-Metal
/// generic-atomic path draw.
#[cfg(any(test, feature = "native-metal-experimental"))]
pub(crate) struct PreparedSingleAtomicPathDraw {
    pub(crate) resources: PreparedTypedDrawResources,
    pub(crate) gradient_batch: Option<GradientBatch>,
}

/// Prepares one unclipped solid or linear-gradient path through the canonical
/// production logical writer contract for the bounded native-Metal
/// generic-atomic tracer.
///
/// This materializes a lightweight admitted midpoint-fill record without
/// constructing the backend-bearing `SolidDraw` envelope. A cfg(test)
/// field-by-field oracle proves it matches the canonical production prepared
/// slot, gradient batch, and exactly-once consumption report for the supported
/// single-draw fixtures.
/// Scheduling, clipping, multiple draws, and multiple logical flushes
/// deliberately remain outside this helper.
#[cfg(any(test, feature = "native-metal-experimental"))]
pub(crate) fn prepare_single_atomic_path_draw(
    config: LogicalFrameConfig,
    path: &LogicalPath,
    paint: &LogicalPaint,
    state: DrawState,
) -> Result<Option<PreparedSingleAtomicPathDraw>, &'static str> {
    if config.mode != RenderMode::ClockwiseAtomic {
        return Err("single atomic path preparation requires clockwise-atomic mode");
    }
    let Some(admitted) = admit_path_draw(config, state, path, paint)? else {
        return Ok(None);
    };
    if admitted.paint.style != RenderPaintStyle::Fill
        || admitted.paint.feather != 0.0
        || admitted.state.clip_rect.is_some()
    {
        return Err("single atomic path preparation requires an unclipped fill");
    }
    if let Some(shader) = admitted.paint.shader.as_ref() {
        match shader {
            super::LogicalShader::Linear { colors, stops, .. }
                if colors.len() == 2 && stops.as_slice() == [0.0, 1.0] => {}
            super::LogicalShader::Linear { .. } => {
                return Err("single atomic path preparation only supports simple linear gradients");
            }
            super::LogicalShader::Radial { .. } => {
                return Err("single atomic path preparation only supports linear gradients");
            }
        }
    }
    let prepared = admitted
        .preparation
        .prepared_fill
        .as_deref()
        .ok_or("single atomic path omitted prepared fill geometry")?;
    if prepared.should_use_interior(&admitted.path, admitted.state.transform) {
        return Err("single atomic path preparation requires midpoint geometry");
    }
    let mut tessellation = prepared
        .midpoint(&admitted.path, admitted.state.transform)
        .map(|prepared| prepared.tessellation.clone())
        .ok_or("single atomic path omitted midpoint geometry")?;
    let [xx, yx, xy, yy, _, _] = admitted.state.transform.0;
    tessellation.make_double_sided_with_direction(
        super::draw::clockwise_atomic_negate_coverage_from_area(
            super::draw::path_coarse_area(&admitted.path.raw_path),
            xx * yy - xy * yx,
            admitted.path.fill_rule,
            true,
        ),
    );
    let mut path_data = tessellation.path;
    path_data.z_index = 0;
    path_data.coverage_buffer_range.pitch = config.width.div_ceil(32) * 32;
    let mut contours = tessellation.contours;
    for contour in &mut contours {
        contour.path_id = 1;
    }
    let local_contour_ids_are_dense = contours.len() <= gpu::CONTOUR_ID_MASK as usize;
    let gradient_batch = admitted
        .paint
        .shader
        .as_ref()
        .map(|shader| {
            prepare_single_gradient_batch(shader, admitted.state.opacity, admitted.state.transform)
                .ok_or("single atomic path has invalid gradient parameters")
        })
        .transpose()?;
    let gradient = gradient_batch.as_ref().and_then(|batch| batch.draw(0));
    let paint_data = gradient
        .map_or_else(
            || {
                gpu::PaintData::solid(
                    modulate_color_alpha(admitted.paint.color, admitted.state.opacity),
                    atomic_paint_fill_rule(admitted.path.fill_rule, true),
                    admitted.paint.blend_mode,
                )
            },
            |gradient| {
                gpu::PaintData::gradient(
                    gradient.paint_type,
                    gradient.texture_y,
                    atomic_paint_fill_rule(admitted.path.fill_rule, true),
                    admitted.paint.blend_mode,
                )
            },
        )
        .with_clip_id(0)
        .with_generic_clockwise_fill();
    let paint_aux = gradient.map_or_else(
        || clip_rect_paint_aux(None),
        |gradient| gradient_paint_aux(None, gradient),
    );
    Ok(Some(PreparedSingleAtomicPathDraw {
        resources: PreparedTypedDrawResources {
            contour_base: 0,
            path: path_data,
            paint: paint_data,
            paint_aux,
            spans: tessellation.spans,
            contours,
            triangles: Vec::new(),
            base_instance: tessellation.base_instance,
            instance_count: tessellation.instance_count,
            triangle_count: 0,
            borrowed_triangle_count: 0,
            main_triangle_batches: Vec::new(),
            has_interior_triangles: false,
            uses_interior: false,
            local_contour_ids_are_dense,
        },
        gradient_batch,
    }))
}

/// Admit and materialize one same-logical-flush set of solid
/// raster-ordering feather-atlas draws without constructing `SolidDraw`.
///
/// Path, paint, and final atlas-blit records remain in authored order. Atlas
/// tessellation follows pinned C++ `LogicalFlush::writeResources`: fills then
/// strokes, with unscissored draws before scissored draws in each style. The
/// midpoint tessellation is compacted into one flush-wide patch range with a
/// single pre-padding and tail-padding sequence.
///
/// Pinned upstream source: `renderer/src/render_context.cpp:1412-1439,
/// 2239-2329` at `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
#[cfg(any(test, feature = "native-metal-experimental"))]
pub(crate) fn prepare_raster_ordering_atlas_flush(
    config: LogicalFrameConfig,
    inputs: &[RasterOrderingAtlasInput<'_>],
) -> Result<Option<PreparedRasterOrderingAtlasFlush>, &'static str> {
    if config.mode != RenderMode::RasterOrdering {
        return Err("feather-atlas flush preparation requires raster ordering");
    }

    struct PendingAtlasDraw {
        #[cfg(test)]
        input_index: usize,
        path_id: u16,
        placement: super::AtlasPlacement,
        is_stroke: bool,
        scissored: bool,
        buffers: ShadowBuffers,
        base_patch: u32,
        patch_count: u32,
        local_contour_ids_are_dense: bool,
    }

    let frame_bounds = [
        0,
        0,
        i32::try_from(config.width).unwrap_or(i32::MAX),
        i32::try_from(config.height).unwrap_or(i32::MAX),
    ];
    let mut admitted = Vec::with_capacity(inputs.len());
    let mut placements = Vec::with_capacity(inputs.len());
    for (input_index, input) in inputs.iter().copied().enumerate() {
        if input.paint.shader.is_some() || input.paint.invalid_shader {
            return Err("feather-atlas flush preparation requires solid paints");
        }
        if input.paint.feather == 0.0
            || !draw::feather_requires_atlas(input.paint.feather, input.state.transform, false)
        {
            return Err("feather-atlas flush input does not route through the atlas");
        }
        if !input.path.valid || !path_draw_has_valid_parameters(input.path, input.paint) {
            continue;
        }
        let Some(pixel_bounds) =
            path_draw_pixel_bounds(input.path, input.paint, input.state.transform)
        else {
            continue;
        };
        let clipped_pixel_bounds =
            intersect_pixel_bounds(pixel_bounds, input.state.overall_clip_pixel_bounds);
        if pixel_bounds_are_outside_frame(clipped_pixel_bounds, config.width, config.height) {
            continue;
        }
        let Some(preparation) = prepare_path_draw_with_pixel_bounds(
            input.path,
            input.paint,
            input.state,
            Some(pixel_bounds),
            config.mode,
            config.width,
            config.height,
        ) else {
            continue;
        };
        let prepared_feather = preparation
            .prepared_feather
            .ok_or("logical preparation omitted feather geometry")?;
        let placement = feather_atlas_placement(
            &input.path.raw_path,
            input.state.transform,
            input.paint.feather,
            input.paint.effective_stroke(),
            config.width,
            config.height,
        )
        .ok_or("logical preparation omitted feather-atlas placement")?;
        let admitted_index = admitted.len();
        let path_id = u16::try_from(admitted_index + 1).map_err(|_| "logical path ID overflow")?;
        let is_stroke = input.paint.style == RenderPaintStyle::Stroke;
        let scissored = intersect_pixel_bounds(pixel_bounds, frame_bounds) != pixel_bounds;
        admitted.push((
            input_index,
            path_id,
            input,
            prepared_feather,
            is_stroke,
            scissored,
        ));
        placements.push(Some(placement));
    }
    if admitted.is_empty() {
        return Ok(None);
    }

    let packed = pack_raster_ordering_feather_atlas_placements(config, placements);
    let mut pending = Vec::with_capacity(admitted.len());
    let mut paths = Vec::with_capacity(admitted.len());
    let mut paints = Vec::with_capacity(admitted.len());
    let mut paint_aux = Vec::with_capacity(admitted.len());
    let mut triangles = Vec::with_capacity(admitted.len().saturating_mul(6));
    for (
        admitted_index,
        ((input_index, path_id, input, prepared_feather, is_stroke, scissored), placement),
    ) in admitted.into_iter().zip(packed).enumerate()
    {
        #[cfg(not(test))]
        let _ = input_index;
        let placement = placement.ok_or("packed feather atlas omitted an admitted draw")?;
        let mut buffers = ShadowBuffers::default();
        write_raster_ordering_atlas_resources(
            &mut buffers,
            admitted_index,
            input.path,
            input.paint,
            input.state,
            &prepared_feather,
            placement,
        );
        if buffers.paths.len() != 1 || buffers.paints.len() != 1 || buffers.triangles.len() != 6 {
            return Err("logical atlas serializer omitted canonical draw resources");
        }
        paths.push(buffers.paths[0]);
        paints.push(buffers.paints[0].paint);
        paint_aux.push(buffers.paints[0].aux);
        triangles.extend_from_slice(&buffers.triangles);
        pending.push(PendingAtlasDraw {
            #[cfg(test)]
            input_index,
            path_id,
            placement,
            is_stroke,
            scissored,
            base_patch: prepared_feather.tessellation.base_instance,
            patch_count: prepared_feather.tessellation.instance_count,
            local_contour_ids_are_dense: prepared_feather.local_contour_ids_are_dense,
            buffers,
        });
    }

    let content_extent = pending
        .iter()
        .try_fold([0_u32; 2], |extent, draw| {
            Some([
                extent[0].max(draw.placement.origin[0].checked_add(draw.placement.width)?),
                extent[1].max(draw.placement.origin[1].checked_add(draw.placement.height)?),
            ])
        })
        .ok_or("feather atlas content extent overflow")?;
    let physical_extent = super::cpp_webgpu_atlas_physical_size(
        content_extent,
        [config.width, config.height],
        config.max_texture_dimension_2d,
    );
    let full_scissor = [
        0,
        0,
        u16::try_from(content_extent[0]).map_err(|_| "feather atlas width exceeds UInt16")?,
        u16::try_from(content_extent[1]).map_err(|_| "feather atlas height exceeds UInt16")?,
    ];

    let midpoint_span = gpu::MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32;
    let mut spans = Vec::new();
    super::append_tessellation_padding_span(&mut spans, 0, midpoint_span);
    let mut contours = Vec::new();
    let mut local_contour_ids = Vec::new();
    let mut draws = Vec::with_capacity(pending.len());
    let mut fill_batches = Vec::new();
    let mut stroke_batches = Vec::new();
    let mut next_base_patch = 1_u32;
    for is_stroke in [false, true] {
        for scissored in [false, true] {
            for draw in pending
                .iter_mut()
                .filter(|draw| draw.is_stroke == is_stroke && draw.scissored == scissored)
            {
                let mut draw_spans = std::mem::take(&mut draw.buffers.tessellations);
                draw_spans.retain(|span| span.contour_id_with_flags & gpu::CONTOUR_ID_MASK != 0);
                let mut base_patch = draw.base_patch;
                let relocation = next_base_patch
                    .checked_sub(base_patch)
                    .and_then(|patches| patches.checked_mul(midpoint_span))
                    .ok_or("feather atlas tessellation relocation overflow")?;
                let contour_offset = super::append_relocated_midpoint_contours_to_flush(
                    &draw_spans,
                    &draw.buffers.contours,
                    0,
                    draw.local_contour_ids_are_dense,
                    u32::from(draw.path_id),
                    relocation,
                    &mut contours,
                    &mut local_contour_ids,
                );
                for span in &mut draw_spans {
                    super::globalize_midpoint_span_contour_id(span, 0, contour_offset);
                }
                super::relocate_tessellation_logically(
                    &mut draw_spans,
                    &mut base_patch,
                    &mut [],
                    next_base_patch,
                    midpoint_span,
                );
                #[cfg(test)]
                let blit_vertex_range = {
                    let start = draw
                        .path_id
                        .checked_sub(1)
                        .and_then(|index| usize::from(index).checked_mul(6))
                        .ok_or("atlas blit vertex range overflow")?;
                    let end = start
                        .checked_add(6)
                        .ok_or("atlas blit vertex range overflow")?;
                    start..end
                };
                let scissor = if scissored {
                    let right = draw.placement.origin[0]
                        .checked_add(draw.placement.width)
                        .ok_or("feather atlas scissor overflow")?;
                    let bottom = draw.placement.origin[1]
                        .checked_add(draw.placement.height)
                        .ok_or("feather atlas scissor overflow")?;
                    [
                        u16::try_from(draw.placement.origin[0])
                            .map_err(|_| "feather atlas scissor exceeds UInt16")?,
                        u16::try_from(draw.placement.origin[1])
                            .map_err(|_| "feather atlas scissor exceeds UInt16")?,
                        u16::try_from(right).map_err(|_| "feather atlas scissor exceeds UInt16")?,
                        u16::try_from(bottom)
                            .map_err(|_| "feather atlas scissor exceeds UInt16")?,
                    ]
                } else {
                    full_scissor
                };
                spans.append(&mut draw_spans);
                let batches = if is_stroke {
                    &mut stroke_batches
                } else {
                    &mut fill_batches
                };
                if scissored {
                    batches.push(gpu::AtlasDrawBatch {
                        scissor,
                        base_patch,
                        patch_count: draw.patch_count,
                    });
                } else if let Some(batch) = batches.last_mut() {
                    debug_assert_eq!(batch.scissor, scissor);
                    debug_assert_eq!(batch.base_patch + batch.patch_count, base_patch);
                    batch.patch_count = batch
                        .patch_count
                        .checked_add(draw.patch_count)
                        .ok_or("feather atlas batch patch count overflow")?;
                } else {
                    batches.push(gpu::AtlasDrawBatch {
                        scissor,
                        base_patch,
                        patch_count: draw.patch_count,
                    });
                }
                draws.push(PreparedRasterOrderingAtlasDraw {
                    #[cfg(test)]
                    input_index: draw.input_index,
                    #[cfg(test)]
                    path_id: draw.path_id,
                    #[cfg(test)]
                    atlas_placement: draw.placement,
                    is_stroke,
                    #[cfg(test)]
                    scissor,
                    #[cfg(test)]
                    base_patch,
                    #[cfg(test)]
                    patch_count: draw.patch_count,
                    #[cfg(test)]
                    blit_vertex_range,
                });
                next_base_patch = next_base_patch
                    .checked_add(draw.patch_count)
                    .ok_or("feather atlas patch range overflow")?;
            }
        }
    }

    let geometry_end = next_base_patch
        .checked_mul(midpoint_span)
        .ok_or("feather atlas tessellation range overflow")?;
    let outer_start = super::align_to(geometry_end, gpu::OUTER_CURVE_PATCH_SEGMENT_SPAN as u32);
    super::append_tessellation_padding_span(&mut spans, geometry_end, outer_start);
    super::append_tessellation_padding_span(
        &mut spans,
        outer_start,
        outer_start
            .checked_add(1)
            .ok_or("feather atlas tessellation range overflow")?,
    );
    let tessellation_height = outer_start
        .checked_add(1)
        .ok_or("feather atlas tessellation range overflow")?
        .div_ceil(gpu::TESS_TEXTURE_WIDTH as u32);
    if tessellation_height > config.max_texture_dimension_2d {
        return Err("feather atlas tessellation texture exceeds device limit");
    }

    Ok(Some(PreparedRasterOrderingAtlasFlush {
        paths,
        paints,
        paint_aux,
        spans,
        contours,
        triangles,
        draws,
        fill_batches,
        stroke_batches,
        content_extent,
        physical_extent,
    }))
}

struct NullFrame {
    logical: LogicalFrame,
    scratch: super::StrokePreparationScratchLease,
    error: Option<&'static str>,
}

impl std::ops::Deref for NullFrame {
    type Target = LogicalFrame;

    fn deref(&self) -> &Self::Target {
        &self.logical
    }
}

impl std::ops::DerefMut for NullFrame {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.logical
    }
}

/// GPU-free adapter for the production logical frame planner.
pub struct NullLogicalRenderer {
    config: LogicalFrameConfig,
    resources: LogicalResourceStore,
    intersection_board: super::intersection_board::IntersectionBoard,
    scratch_pool: Arc<super::StrokePreparationScratchPool>,
    frame: Option<NullFrame>,
}

impl NullLogicalRenderer {
    pub fn new(config: LogicalFrameConfig) -> Self {
        Self {
            config,
            resources: LogicalResourceStore::default(),
            intersection_board: super::intersection_board::IntersectionBoard::new(
                super::intersection_board::GroupingType::Disjoint,
            ),
            scratch_pool: Arc::new(super::StrokePreparationScratchPool::default()),
            frame: None,
        }
    }

    pub fn begin_frame(&mut self) {
        assert!(self.frame.is_none(), "null logical frame already active");
        let scratch = self.scratch_pool.checkout();
        self.frame = Some(NullFrame {
            logical: LogicalFrame::new(self.config),
            scratch,
            error: None,
        });
    }

    pub fn prepare_path(&self, raw_path: &RawPath, fill_rule: FillRule) -> LogicalPathHandle {
        LogicalPathHandle::new(raw_path, fill_rule)
    }

    pub fn save(&mut self) {
        let frame = self.active_frame();
        if frame.error.is_none() {
            frame.logical_state.save();
        }
    }

    pub fn restore(&mut self) {
        let frame = self.active_frame();
        if frame.error.is_none() {
            frame.logical_state.restore();
        }
    }

    pub fn transform(&mut self, transform: nuxie_render_api::Mat2D) {
        let frame = self.active_frame();
        if frame.error.is_none() {
            frame.logical_state.transform(transform);
        }
    }

    pub fn clip_path(&mut self, path: &LogicalPathHandle) -> Result<(), &'static str> {
        let config = self.config;
        let frame = self.active_frame();
        if let Some(error) = frame.error {
            return Err(error);
        }
        let result = frame.logical_state.clip_path(config, &path.path);
        if let Err(error) = result {
            frame.error = Some(error);
        }
        result
    }

    pub fn draw_path(
        &mut self,
        path: &LogicalPathHandle,
        paint: LogicalPathPaint,
    ) -> Result<(), &'static str> {
        self.draw_path_internal(path, paint, None)
    }

    pub fn draw_path_with_gradient(
        &mut self,
        path: &LogicalPathHandle,
        paint: LogicalPathPaint,
        gradient: LogicalGradient,
    ) -> Result<(), &'static str> {
        self.draw_path_internal(path, paint, Some(gradient))
    }

    fn draw_path_internal(
        &mut self,
        path: &LogicalPathHandle,
        paint: LogicalPathPaint,
        gradient: Option<LogicalGradient>,
    ) -> Result<(), &'static str> {
        let config = self.config;
        let frame = self.active_frame();
        if let Some(error) = frame.error {
            return Err(error);
        }
        if pixel_bounds_are_empty(frame.logical_state.state.overall_clip_pixel_bounds) {
            return Ok(());
        }
        let result = (|| {
            let mut paint = paint.into_logical_paint();
            paint.shader = gradient.map(LogicalGradient::into_logical_shader);
            let Some(admitted) =
                admit_path_draw(config, frame.logical_state.state, &path.path, &paint)?
            else {
                return Ok(());
            };
            let (clip_updates, clip_id) =
                frame.logical_state.prepare_scheduled_clip_updates(config)?;
            let content = admitted.finish(clip_id, &mut frame.scratch)?;
            frame.logical.push_content_batch(clip_updates, content)
        })();
        if let Err(error) = result {
            frame.error = Some(error);
        }
        result
    }

    pub fn flush(&mut self) -> Result<LogicalFrameReport, &'static str> {
        self.flush_internal(false)
    }

    /// Flushes the production CPU path and additionally fingerprints every
    /// retained buffer for differential diagnostics. Do not use this method
    /// for benchmark timing.
    pub fn flush_with_diagnostics(&mut self) -> Result<LogicalFrameReport, &'static str> {
        self.flush_internal(true)
    }

    fn flush_internal(&mut self, diagnostics: bool) -> Result<LogicalFrameReport, &'static str> {
        let mut frame = self
            .frame
            .take()
            .expect("null logical frame must begin before flush");
        if let Some(error) = frame.error {
            return Err(error);
        }
        frame.logical.finalize(&mut self.intersection_board)?;
        let prepared = if diagnostics {
            self.resources
                .prepare_for_production_with_diagnostics(&frame.logical)?
        } else {
            self.resources.prepare_for_production(&frame.logical)?
        };
        // The null adapter's retained shadow writer is the terminal consumer:
        // there is deliberately no GPU-side encoder after this boundary.
        prepared.consume_all_for_null();
        let report = prepared.into_report();
        drop(frame);
        Ok(report)
    }

    pub fn retained_scratch_slots(&self) -> usize {
        self.scratch_pool.cached_len()
    }

    pub fn retained_scratch_capacity_bytes(&self) -> usize {
        self.scratch_pool.retained_capacity_bytes()
    }

    fn active_frame(&mut self) -> &mut NullFrame {
        self.frame
            .as_mut()
            .expect("null logical frame must begin before operation")
    }
}

#[cfg(test)]
mod frame_pool_tests {
    use super::*;
    use crate::MAXIMAL_PIXEL_BOUNDS;
    use std::sync::Arc;

    #[test]
    fn logical_frame_pool_reuses_storage_and_resets_frame_state() {
        let pool = Arc::new(LogicalFramePool::default());
        let config = LogicalFrameConfig {
            width: 1024,
            height: 1024,
            mode: RenderMode::ClockwiseAtomic,
            max_texture_dimension_2d: 8192,
            msaa_atlas_supports_clip_rect: true,
        };

        {
            let mut frame = pool.checkout(config);
            frame.draws.reserve(32);
            frame.draw_resources.reserve(32);
            frame.logical_flush_starts.reserve(8);
            frame.msaa_schedule.reserve(32);
            frame.msaa_schedule_flush_starts.reserve(8);
            frame.msaa_scheduled_resources_scratch.reserve(32);
            frame.atomic_batch_flags.reserve(32);
            frame.occurrence_validation_scratch.reserve(32);
            frame.logical_state.stack.reserve(8);
            frame.logical_state.clips.reserve(8);
            frame.logical_state.msaa_path_clips.reserve(8);
            frame.logical_state.state.opacity = 0.5;
            frame.finalized = true;
        }

        let frame = pool.checkout(config);
        assert!(frame.draws.capacity() >= 32);
        assert!(frame.draw_resources.capacity() >= 32);
        assert!(frame.logical_flush_starts.capacity() >= 8);
        assert!(frame.msaa_schedule.capacity() >= 32);
        assert!(frame.msaa_schedule_flush_starts.capacity() >= 8);
        assert!(frame.msaa_scheduled_resources_scratch.capacity() >= 32);
        assert!(frame.atomic_batch_flags.capacity() >= 32);
        assert!(frame.occurrence_validation_scratch.capacity() >= 32);
        assert!(frame.logical_state.stack.capacity() >= 8);
        assert!(frame.logical_state.clips.capacity() >= 8);
        assert!(frame.logical_state.msaa_path_clips.capacity() >= 8);
        assert!(frame.draws.is_empty());
        assert!(frame.msaa_scheduled_resources_scratch.is_empty());
        assert!(frame.atomic_batch_flags.is_empty());
        assert!(frame.occurrence_validation_scratch.is_empty());
        assert_eq!(frame.logical_flush_starts, [0]);
        assert_eq!(frame.logical_state.state.transform, Mat2D::IDENTITY);
        assert_eq!(frame.logical_state.state.opacity, 1.0);
        assert!(frame.logical_state.state.clip_rect.is_none());
        assert_eq!(
            frame.logical_state.state.overall_clip_pixel_bounds,
            MAXIMAL_PIXEL_BOUNDS
        );
        assert_eq!(frame.logical_state.state.clip_stack_height, 0);
        assert!(!frame.finalized);
    }

    #[test]
    fn logical_frame_pool_drops_oversized_backings() {
        let pool = Arc::new(LogicalFramePool::default());
        let config = LogicalFrameConfig {
            width: 1024,
            height: 1024,
            mode: RenderMode::Msaa,
            max_texture_dimension_2d: 8192,
            msaa_atlas_supports_clip_rect: true,
        };

        {
            let mut frame = pool.checkout(config);
            frame.draw_resources.resize(
                MAX_RETAINED_LOGICAL_FRAME_DRAWS + 1,
                logical_flush::ResourceCounters::default(),
            );
            frame.note_retained_frame_growth();
            assert!(!frame.retain_in_pool);
        }

        assert!(pool.cached.lock().unwrap().is_empty());
    }

    #[test]
    fn logical_frame_pool_drops_oversized_state_backings() {
        let config = LogicalFrameConfig {
            width: 1024,
            height: 1024,
            mode: RenderMode::Msaa,
            max_texture_dimension_2d: 8192,
            msaa_atlas_supports_clip_rect: true,
        };

        for backing in 0..3 {
            let pool = Arc::new(LogicalFramePool::default());
            {
                let mut frame = pool.checkout(config);
                match backing {
                    0 => {
                        frame
                            .logical_state
                            .stack
                            .reserve(MAX_RETAINED_LOGICAL_FRAME_DRAWS + 1);
                        frame.logical_state.note_stack_backing_growth();
                    }
                    1 => {
                        frame
                            .logical_state
                            .clips
                            .reserve(MAX_RETAINED_LOGICAL_FRAME_DRAWS + 1);
                        frame.logical_state.note_clip_backing_growth();
                    }
                    2 => {
                        frame
                            .logical_state
                            .msaa_path_clips
                            .reserve(MAX_RETAINED_LOGICAL_FRAME_DRAWS + 1);
                        frame.logical_state.note_msaa_path_clip_backing_growth();
                    }
                    _ => unreachable!(),
                }
            }
            assert!(pool.cached.lock().unwrap().is_empty());
        }
    }
}
