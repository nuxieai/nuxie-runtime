//! Backend-neutral logical frame planning shared by GPU and null adapters.
//!
//! The interface deliberately stays at begin/draw/flush. All resource
//! accounting, retained shadow-buffer growth, typed writes, and rewind live
//! behind it so a backend cannot accidentally benchmark a shallower seam.

use std::borrow::Cow;
use std::mem::size_of;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use bytemuck::{Pod, Zeroable};
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

#[cfg(test)]
thread_local! {
    static PATH_DRAW_ADMISSION_EVALUATIONS: Cell<usize> = const { Cell::new(0) };
    static GRADIENT_BATCH_PREPARATIONS: Cell<usize> = const { Cell::new(0) };
}

#[cfg(test)]
pub(crate) fn take_path_draw_admission_evaluations() -> usize {
    PATH_DRAW_ADMISSION_EVALUATIONS.with(|evaluations| evaluations.replace(0))
}

#[cfg(test)]
pub(crate) fn take_gradient_batch_preparations() -> usize {
    GRADIENT_BATCH_PREPARATIONS.with(|preparations| preparations.replace(0))
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

#[derive(Clone)]
pub(crate) struct GradientBatch {
    pub(crate) spans: Vec<gpu::GradientSpan>,
    pub(crate) height: u32,
    pub(crate) draws: Vec<Option<PreparedGradient>>,
}

impl GradientBatch {
    pub(crate) fn draw(&self, index: usize) -> Option<PreparedGradient> {
        if self.draws.is_empty() {
            None
        } else {
            self.draws[index]
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.height == 0
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
    fn into_wgpu(self) -> super::WgpuShader {
        match self {
            Self::Linear {
                start,
                end,
                colors,
                stops,
            } => super::WgpuShader::Linear {
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
            } => super::WgpuShader::Radial {
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
    pub(crate) fn into_wgpu(self) -> LogicalPaint {
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
                super::WgpuShader::Linear { colors, stops, .. }
                | super::WgpuShader::Radial { colors, stops, .. } => (colors, stops),
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
pub struct LogicalFrameReport {
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
    next_occurrence_id: u64,
    pub(crate) msaa_schedule_flush_starts: Vec<usize>,
    finalized: bool,
    finalization_passes: usize,
}

struct PlannedDrawBatch {
    draws: Vec<logical_flush::ResourceCounters>,
    total: logical_flush::ResourceCounters,
}

impl LogicalFrame {
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
            msaa_schedule_flush_starts: Vec::new(),
            next_occurrence_id: 1,
            finalized: false,
            finalization_passes: 0,
        }
    }

    pub(crate) fn finalize(
        &mut self,
        board: &mut super::intersection_board::IntersectionBoard,
    ) -> Result<(), &'static str> {
        if self.finalized {
            return self.validate();
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
                let scheduled_resources = self.msaa_schedule[schedule_start..]
                    .iter()
                    .map(|entry| self.draw_resources[flush_start + entry.authored_order])
                    .collect::<Vec<_>>();
                super::apply_msaa_draw_schedule(
                    &mut self.draws[flush_start..flush_end],
                    &mut self.msaa_schedule[schedule_start..],
                );
                self.draw_resources[flush_start..flush_end].copy_from_slice(&scheduled_resources);
            }
        }
        self.finalized = true;
        self.finalization_passes = self.finalization_passes.saturating_add(1);
        self.validate()
    }

    pub(crate) fn is_finalized(&self) -> bool {
        self.finalized
    }

    pub(crate) fn validate(&self) -> Result<(), &'static str> {
        if self.draws.len() != self.draw_resources.len() {
            return Err("logical frame draw/resource plan length mismatch");
        }
        if self.finalized {
            let mut occurrence_ids = std::collections::HashSet::with_capacity(self.draws.len());
            if self.draws.iter().any(|draw| {
                draw.logical_occurrence_id == 0
                    || !occurrence_ids.insert(draw.logical_occurrence_id)
            }) {
                return Err("logical frame draw occurrence identities are not unique");
            }
        }
        if self
            .logical_flush_starts
            .iter()
            .any(|&start| start > self.draws.len())
        {
            return Err("logical frame flush layout exceeds draw plan");
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
            .collect::<Vec<_>>();
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
        self.draw_resources.extend_from_slice(&planned.draws);
        Ok(())
    }

    fn rollover_plan(
        &mut self,
        original_updates: &[SolidDraw],
        original: &PlannedDrawBatch,
        updates: &[SolidDraw],
    ) -> Result<PlannedDrawBatch, &'static str> {
        let mut reused = vec![false; original_updates.len()];
        let mut draws = Vec::with_capacity(updates.len() + 1);
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
                || (config.mode == RenderMode::ClockwiseAtomic
                    && draw::feather_requires_atlas(
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
        }
    }
}

impl LogicalDrawState {
    pub(crate) fn save(&mut self) {
        self.stack.push(self.state);
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
        }
        self.state.clip_stack_height = height + 1;
    }

    pub(crate) fn prepare_scheduled_clip_updates(
        &mut self,
        config: LogicalFrameConfig,
    ) -> Result<(Vec<SolidDraw>, u16), &'static str> {
        if config.mode == RenderMode::ClockwiseAtomic {
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
        if config.mode == RenderMode::ClockwiseAtomic {
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
    gradient_flushes: Vec<PreparedGradientFlush>,
    typed_address: usize,
    typed_inputs: Vec<u64>,
    typed_draws: Vec<Option<PreparedTypedDrawResources>>,
    typed_consumption: Arc<Vec<AtomicUsize>>,
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
}

pub(crate) struct PreparedTypedDrawSelection<'a> {
    resources: &'a PreparedLogicalFrameResources,
    indices: Vec<Option<usize>>,
}

impl PreparedTypedDrawSelection<'_> {
    pub(crate) fn draw(&self, index: usize) -> Option<&PreparedTypedDrawResources> {
        let resource_index = self.indices[index]?;
        let state = &self.resources.typed_consumption[resource_index];
        match state.compare_exchange(usize::MAX, 1, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => {}
            Err(1) => {
                state.store(2, Ordering::Relaxed);
            }
            Err(actual) => panic!(
                "typed logical output was consumed without a unique reservation (state {actual})"
            ),
        }
        self.resources.typed_draws[resource_index].as_ref()
    }
}

pub(crate) struct PreparedGradientSelection<'a> {
    batch: Cow<'a, GradientBatch>,
    draws: Option<Vec<Option<PreparedGradient>>>,
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
        self.draws
            .as_ref()
            .map_or_else(|| self.batch.draw(index), |draws| draws[index])
    }
}

struct PreparedGradientFlush {
    address: usize,
    draw_count: usize,
    inputs: Vec<u64>,
    batch: GradientBatch,
}

impl PreparedLogicalFrameResources {
    pub(crate) fn into_report(self) -> LogicalFrameReport {
        self.report_with_consumption()
    }

    pub(crate) fn gradient_batch<'a>(
        &'a self,
        draws: &[SolidDraw],
    ) -> PreparedGradientSelection<'a> {
        let address = draws.as_ptr() as usize;
        let copied_inputs = || {
            draws
                .iter()
                .map(gradient_input_fingerprint)
                .collect::<Vec<_>>()
        };
        let mut inputs = None;
        let matched = self.gradient_flushes.iter().find(|flush| {
            flush.draw_count == draws.len()
                && (flush.address == address
                    || flush.inputs == *inputs.get_or_insert_with(copied_inputs))
        });
        if let Some(flush) = matched {
            return PreparedGradientSelection {
                batch: Cow::Borrowed(&flush.batch),
                draws: None,
            };
        }

        let requested = inputs.get_or_insert_with(copied_inputs);
        for flush in &self.gradient_flushes {
            let mut mapped = Vec::with_capacity(requested.len());
            let mut available = vec![true; flush.inputs.len()];
            let mut complete = true;
            for input in requested.iter().copied() {
                let Some(index) = flush
                    .inputs
                    .iter()
                    .enumerate()
                    .find_map(|(index, candidate)| {
                        (available[index] && *candidate == input).then_some(index)
                    })
                else {
                    complete = false;
                    break;
                };
                available[index] = false;
                mapped.push(flush.batch.draw(index));
            }
            if complete {
                return PreparedGradientSelection {
                    batch: Cow::Borrowed(&flush.batch),
                    draws: Some(mapped),
                };
            }
        }

        panic!(
            "encoder requested gradient inputs outside the finalized logical frame; \
             production gradient normalization must run exactly once"
        )
    }

    pub(crate) fn typed_draws<'a>(&'a self, draws: &[SolidDraw]) -> PreparedTypedDrawSelection<'a> {
        if draws.as_ptr() as usize == self.typed_address && draws.len() == self.typed_draws.len() {
            let indices = self
                .typed_draws
                .iter()
                .enumerate()
                .map(|(index, draw)| draw.as_ref().map(|_| self.reserve_typed_draw(index)))
                .collect();
            return PreparedTypedDrawSelection {
                resources: self,
                indices,
            };
        }

        let requested = draws
            .iter()
            .map(|draw| draw.logical_occurrence_id)
            .collect::<Vec<_>>();
        let mut mapped = Vec::with_capacity(requested.len());
        for input in requested {
            let index = self
                .typed_inputs
                .iter()
                .enumerate()
                .find_map(|(index, candidate)| {
                    if *candidate != input {
                        return None;
                    }
                    if self.typed_draws[index].is_none() {
                        return Some(index);
                    }
                    self.typed_consumption[index]
                        .compare_exchange(0, usize::MAX, Ordering::Relaxed, Ordering::Relaxed)
                        .is_ok()
                        .then_some(index)
                })
                .expect("encoder requested typed resources outside the finalized logical frame");
            mapped.push(self.typed_draws[index].as_ref().map(|_| index));
        }
        PreparedTypedDrawSelection {
            resources: self,
            indices: mapped,
        }
    }

    fn reserve_typed_draw(&self, index: usize) -> usize {
        self.typed_consumption[index]
            .compare_exchange(0, usize::MAX, Ordering::Relaxed, Ordering::Relaxed)
            .unwrap_or_else(|state| {
                panic!("typed logical output occurrence {index} was selected twice (state {state})")
            });
        index
    }

    pub(crate) fn consume_all_for_null(&self) {
        for (index, draw) in self.typed_draws.iter().enumerate() {
            if draw.is_some() {
                self.typed_consumption[index]
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
        let mut report = self.report.clone();
        let mut consumed_once = 0;
        let mut exact = report.production_typed_output_eligible_draws != 0;
        for (draw, state) in self.typed_draws.iter().zip(self.typed_consumption.iter()) {
            if draw.is_none() {
                continue;
            }
            let state = state.load(Ordering::Relaxed);
            consumed_once += usize::from(state == 1);
            exact &= state == 1;
        }
        report.production_typed_output_consumed_draws = consumed_once;
        report.production_typed_output_consumed = exact;
        report
    }
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
        frame.validate()?;
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
        let gradient_batch = prepare_gradient_batch(&frame.draws);
        let atomic_batch_flags = production_atomic_batch_flags(frame);
        let mut typed_draws = Vec::with_capacity(frame.draws.len());
        for (flush_index, (&start, counts)) in starts.iter().zip(&flushes).enumerate() {
            let end = starts
                .get(flush_index + 1)
                .copied()
                .unwrap_or(frame.draws.len());
            let before_capacities = buffers.buffer_capacities();
            buffers.grow(*counts);
            let gradient_selection = PreparedGradientSelection {
                batch: Cow::Borrowed(&gradient_batch),
                draws: (start != 0 || end != frame.draws.len()).then(|| {
                    (start..end)
                        .map(|draw_index| gradient_batch.draw(draw_index))
                        .collect()
                }),
            };
            typed_draws.extend(write_resources(
                &mut buffers,
                config,
                flush_index,
                *counts,
                &frame.draws[start..end],
                &frame.draw_resources[start..end],
                &atomic_batch_flags[start..end],
                &gradient_selection,
            ));
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
        let report = LogicalFrameReport {
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
            production_typed_output_eligible_draws: typed_draws.iter().flatten().count(),
            production_typed_output_consumed_draws: 0,
            production_fallback_draws: typed_draws.iter().filter(|draw| draw.is_none()).count(),
            production_typed_output_consumed: false,
        };
        Ok(PreparedLogicalFrameResources {
            report,
            gradient_flushes: vec![PreparedGradientFlush {
                address: frame.draws.as_ptr() as usize,
                draw_count: frame.draws.len(),
                inputs: frame.draws.iter().map(gradient_input_fingerprint).collect(),
                batch: gradient_batch,
            }],
            typed_address: frame.draws.as_ptr() as usize,
            typed_inputs: frame
                .draws
                .iter()
                .map(|draw| draw.logical_occurrence_id)
                .collect(),
            typed_consumption: Arc::new(
                (0..typed_draws.len())
                    .map(|_| AtomicUsize::new(0))
                    .collect(),
            ),
            typed_draws,
        })
    }
}

fn gradient_input_fingerprint(draw: &SolidDraw) -> u64 {
    fn write(hash: &mut u64, bytes: &[u8]) {
        for &byte in bytes {
            *hash ^= u64::from(byte);
            *hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }

    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    write(&mut hash, &draw.state.opacity.to_bits().to_le_bytes());
    for value in draw.state.transform.0 {
        write(&mut hash, &value.to_bits().to_le_bytes());
    }
    match draw.paint.shader.as_ref() {
        None => write(&mut hash, &[0]),
        Some(super::WgpuShader::Linear {
            start,
            end,
            colors,
            stops,
        }) => {
            write(&mut hash, &[1]);
            for value in [start.0, start.1, end.0, end.1] {
                write(&mut hash, &value.to_bits().to_le_bytes());
            }
            for color in colors {
                write(&mut hash, &color.to_le_bytes());
            }
            for stop in stops {
                write(&mut hash, &stop.to_bits().to_le_bytes());
            }
        }
        Some(super::WgpuShader::Radial {
            center,
            radius,
            colors,
            stops,
        }) => {
            write(&mut hash, &[2]);
            for value in [center.0, center.1, *radius] {
                write(&mut hash, &value.to_bits().to_le_bytes());
            }
            for color in colors {
                write(&mut hash, &color.to_le_bytes());
            }
            for stop in stops {
                write(&mut hash, &stop.to_bits().to_le_bytes());
            }
        }
    }
    hash
}

pub(crate) fn prepare_gradient_batch(draws: &[SolidDraw]) -> GradientBatch {
    #[cfg(test)]
    GRADIENT_BATCH_PREPARATIONS.with(|preparations| preparations.set(preparations.get() + 1));

    const RAMPS_PER_SIMPLE_ROW: usize = gradient_pipeline::TEXTURE_WIDTH as usize / 2;
    const ONE_TEXEL_FIXED: u32 = 65_536 / gradient_pipeline::TEXTURE_WIDTH;
    const LEFT_BORDER: u32 = 0x8000_0000;
    const RIGHT_BORDER: u32 = 0x4000_0000;
    const COMPLEX_BORDER: u32 = 0x2000_0000;

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
            definitions.resize_with(draw_index, || None);
            definitions.push(normalize_gradient(shader, draw.state.opacity));
        } else {
            definitions
                .push(shader.and_then(|shader| normalize_gradient(shader, draw.state.opacity)));
        }
    }
    if definitions.is_empty() {
        return GradientBatch {
            spans: Vec::new(),
            height: 0,
            draws: Vec::new(),
        };
    }
    let is_simple = |gradient: &GradientDefinition| {
        gradient.stops.len() == 1
            || (gradient.stops.len() == 2 && gradient.stops[0] == 0.0 && gradient.stops[1] == 1.0)
    };
    let simple_count = definitions
        .iter()
        .flatten()
        .filter(|gradient| is_simple(gradient))
        .count();
    let complex_count = definitions
        .iter()
        .flatten()
        .filter(|gradient| !is_simple(gradient))
        .count();
    let simple_height = simple_count.div_ceil(RAMPS_PER_SIMPLE_ROW) as u32;
    let height = simple_height + complex_count as u32;
    let mut simple_index = 0usize;
    let mut complex_index = 0u32;
    let mut spans = Vec::new();
    let mut prepared = Vec::with_capacity(draws.len());
    for (draw, gradient) in draws.iter().zip(definitions) {
        let Some(gradient) = gradient else {
            prepared.push(None);
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
        let inverse = super::invert(draw.state.transform).unwrap_or(Mat2D([0.0; 6]));
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
        prepared.push(Some(PreparedGradient {
            paint_type: gradient.paint_type,
            texture_y: (row as f32 + 0.5) / height as f32,
            matrix: multiply(gradient_matrix, inverse),
            texture_span,
        }));
    }
    GradientBatch {
        spans,
        height,
        draws: prepared,
    }
}

pub(crate) fn normalize_gradient(
    shader: &super::WgpuShader,
    opacity: f32,
) -> Option<GradientDefinition> {
    const EPSILON: f32 = 1.0 / 4096.0;
    let (paint_type, mut colors, stops, coeffs) = match shader {
        super::WgpuShader::Linear {
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
        super::WgpuShader::Radial {
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

fn production_atomic_batch_flags(frame: &LogicalFrame) -> Vec<bool> {
    let mut flags = vec![false; frame.draws.len()];
    if frame.config.mode != RenderMode::ClockwiseAtomic {
        return flags;
    }
    let advanced_segments = frame
        .draws
        .iter()
        .any(super::draw_uses_advanced_blend)
        .then(|| super::AdvancedAtomicSegmentPlan::new(&frame.draws));
    let mut start = 0;
    let mut logical_flush_index = 0;
    while start < frame.draws.len() {
        let logical_flush_end = frame
            .logical_flush_starts
            .get(logical_flush_index + 1)
            .copied()
            .unwrap_or(frame.draws.len());
        let atomic = super::atomic_draw_is_eligible(&frame.draws[start]);
        let advanced_end = advanced_segments
            .as_ref()
            .and_then(|plan| plan.segment_end(start, logical_flush_end));
        let clockwise_atomic = super::WEBGPU_SUPPORTS_CLOCKWISE_ATOMIC_MODE
            && atomic
            && advanced_end.is_none()
            && super::draw_requires_clockwise_atomic(
                &frame.draws[start],
                frame.config.width,
                frame.config.height,
            );
        let end = advanced_end.unwrap_or_else(|| {
            super::atomic_strategy_run_end(&frame.draws, start, logical_flush_end)
        });
        flags[start..end].fill(clockwise_atomic);
        start = end;
        if start == logical_flush_end {
            logical_flush_index += 1;
        }
    }
    flags
}

fn write_resources(
    buffers: &mut ShadowBuffers,
    config: LogicalFrameConfig,
    flush_index: usize,
    counts: LogicalResourceCounts,
    draws: &[SolidDraw],
    draw_resources: &[logical_flush::ResourceCounters],
    atomic_batch_flags: &[bool],
    gradient_batch: &PreparedGradientSelection<'_>,
) -> Vec<Option<PreparedTypedDrawResources>> {
    let _ = (flush_index, counts);
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
    let mut prepared = Vec::with_capacity(draws.len());
    for (draw_index, ((draw, resources), &use_clockwise_atomic_batch)) in draws
        .iter()
        .zip(draw_resources)
        .zip(atomic_batch_flags)
        .enumerate()
    {
        let path_start = buffers.paths.len();
        let paint_start = buffers.paints.len();
        let contour_start = buffers.contours.len();
        let tessellation_start = buffers.tessellations.len();
        let triangle_start = buffers.triangles.len();
        write_typed_draw_resources(
            buffers,
            config,
            draw_index,
            draw,
            use_clockwise_atomic_batch,
            gradient_batch,
        );
        let backend_fallback =
            typed_draw_uses_backend_fallback(config, draw, use_clockwise_atomic_batch);
        prepared.push(
            (buffers.paths.len() != path_start && !backend_fallback).then(|| {
                debug_assert_eq!(buffers.paths.len(), path_start + 1);
                debug_assert_eq!(buffers.paints.len(), paint_start + 1);
                let paint = buffers.paints[paint_start];
                let metadata = typed_draw_metadata(draw, config, use_clockwise_atomic_batch);
                PreparedTypedDrawResources {
                    contour_base: u32::try_from(contour_start).expect("contour base overflow"),
                    path: buffers.paths[path_start],
                    paint: paint.paint,
                    paint_aux: paint.aux,
                    spans: buffers.tessellations[tessellation_start..].to_vec(),
                    contours: buffers.contours[contour_start..].to_vec(),
                    triangles: buffers.triangles[triangle_start..].to_vec(),
                    base_instance: metadata.base_instance,
                    instance_count: metadata.instance_count,
                    triangle_count: metadata.triangle_count,
                    borrowed_triangle_count: metadata.borrowed_triangle_count,
                    main_triangle_batches: metadata.main_triangle_batches,
                    has_interior_triangles: metadata.has_interior_triangles,
                    uses_interior: metadata.uses_interior,
                }
            }),
        );
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
        });
    }
    prepared
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
    if config.mode == RenderMode::ClockwiseAtomic
        && draw.paint.style == RenderPaintStyle::Fill
        && draw.paint.feather == 0.0
        && draw.authored_should_use_interior()
    {
        let clockwise_override = atomic_fill_clockwise_override(draw, true);
        if let Some(prepared) = draw.authored_atomic_interior_geometry(
            clockwise_override,
            use_clockwise_atomic_batch,
            use_clockwise_atomic_batch
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

    if config.mode == RenderMode::ClockwiseAtomic
        && draw.paint.style == RenderPaintStyle::Fill
        && draw.paint.feather == 0.0
    {
        if let Some(mut tessellation) = draw.authored_fill_tessellation() {
            let clockwise_override = atomic_fill_clockwise_override(draw, true);
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
    draw_index: usize,
    draw: &SolidDraw,
    use_clockwise_atomic_batch: bool,
    gradients: &PreparedGradientSelection<'_>,
) {
    if matches!(draw.role, DrawRole::ClipReset { .. }) || draw.image.is_some() {
        return;
    }

    let path_id = u16::try_from(draw_index + 1).expect("logical path ID overflow");
    let frame_clockwise_override = config.mode == RenderMode::ClockwiseAtomic;
    let clockwise_override = atomic_fill_clockwise_override(draw, frame_clockwise_override);
    let mut spans = Vec::new();
    let mut contours = Vec::new();
    let mut triangles = Vec::new();
    let mut path = gpu::PathData::zeroed();

    if draw.paint.feather != 0.0 {
        if let Some(prepared) = draw.prepared_feather(config.mode) {
            spans.clone_from(&prepared.tessellation.spans);
            contours.clone_from(&prepared.tessellation.contours);
            path = prepared.tessellation.path;
        }
    } else if draw.paint.style == RenderPaintStyle::Stroke {
        if let Some(stroke) = draw.prepared_stroke() {
            spans.clone_from(&stroke.tessellation.spans);
            contours.clone_from(&stroke.tessellation.contours);
            path = stroke.tessellation.path;
        }
    } else if config.mode == RenderMode::ClockwiseAtomic && draw.authored_should_use_interior() {
        let prepared = draw.authored_atomic_interior_geometry(
            clockwise_override,
            use_clockwise_atomic_batch,
            use_clockwise_atomic_batch
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
    } else if let Some(mut tessellation) =
        if config.mode == RenderMode::Msaa && draw.authored_msaa_fill_requires_reverse() {
            draw.prepared_fill()
                .and_then(|prepared| prepared.reversed_midpoint(&draw.path, draw.state.transform))
                .map(|prepared| prepared.tessellation.clone())
        } else {
            draw.authored_fill_tessellation()
        }
    {
        if config.mode == RenderMode::ClockwiseAtomic {
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
    }

    if spans.is_empty()
        && contours.is_empty()
        && triangles.is_empty()
        && draw.paint.style == RenderPaintStyle::Fill
    {
        if let Some(mut tessellation) = draw.authored_fill_tessellation() {
            if config.mode == RenderMode::ClockwiseAtomic {
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
        }
    }
    if spans.is_empty() && contours.is_empty() && triangles.is_empty() {
        return;
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
    buffers.contours.extend(contours);
    buffers.tessellations.extend(spans);
    buffers.triangles.extend(triangles);
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
            let mut paint = paint.into_wgpu();
            paint.shader = gradient.map(LogicalGradient::into_wgpu);
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
