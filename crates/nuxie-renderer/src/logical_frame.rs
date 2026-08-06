//! Backend-neutral logical frame planning shared by GPU and null adapters.
//!
//! The interface deliberately stays at begin/draw/flush. All resource
//! accounting, retained shadow-buffer growth, typed writes, and rewind live
//! behind it so a backend cannot accidentally benchmark a shallower seam.

use std::sync::{Arc, Mutex};

use nuxie_render_api::{
    BlendMode, ColorInt, FillRule, RawPath, RenderPaintStyle, StrokeCap, StrokeJoin,
};

use super::{
    apply_clip_rect, draw, feather_atlas_placement, gradient_pipeline, intersect_pixel_bounds,
    logical_flush, multiply, normalize_gradient, pack_logical_feather_atlas_for_cpp, path_aabb,
    path_draw_has_valid_parameters, path_draw_pixel_bounds, pixel_bounds_are_empty,
    pixel_bounds_are_outside_frame, prepare_path_draw_with_pixel_bounds, ClipElement, DrawRole,
    DrawState, LogicalPaint, LogicalPath, MsaaClipResetAction, PathDrawPreparation,
    PreparedFillGeometry, RenderMode, SolidDraw, FEATHER_ATLAS_PADDING,
};

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
}

impl LogicalResourceCounts {
    fn from_counters(counters: logical_flush::ResourceCounters) -> Self {
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
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogicalFrameReport {
    pub draw_count: usize,
    pub resource_planning_passes: usize,
    pub logical_flushes: Vec<LogicalResourceCounts>,
    pub retained_capacity: LogicalResourceCounts,
    pub written: LogicalResourceCounts,
    pub shadow_fingerprint: u64,
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
        }
    }

    pub(crate) fn validate(&self) -> Result<(), &'static str> {
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
        Ok(())
    }

    pub(crate) fn try_plan_draws<'a>(
        &mut self,
        draws: impl Clone + Iterator<Item = &'a SolidDraw>,
    ) -> Result<(), &'static str> {
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
        let resources = planned
            .iter()
            .try_fold(logical_flush::ResourceCounters::default(), |total, draw| {
                total.checked_add(*draw)
            })
            .ok_or("draw batch overflows logical flush resource accounting")?;
        let allocations = self.logical_flush_allocations.with_draws(config, draws)?;
        if !self.logical_flush.push_draws(resources) {
            return Err("draw batch exceeds logical flush resource counters");
        }
        self.logical_flush_allocations = allocations;
        self.draw_resources.extend(planned);
        Ok(())
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
        clip_updates: Vec<SolidDraw>,
        content: SolidDraw,
    ) -> Result<(), &'static str> {
        let config = self.config;
        let uses_generic_atomic_plane =
            config.mode == RenderMode::ClockwiseAtomic && super::atomic_draw_is_eligible(&content);
        let content_clip_id = match content.role {
            DrawRole::Content { clip_id } => clip_id,
            DrawRole::ClipUpdate { .. } | DrawRole::ClipReset { .. } => {
                unreachable!("content batch must end in a content draw")
            }
        };
        let batch = clip_updates.iter().chain(std::iter::once(&content));
        if self.try_plan_draws(batch).is_ok() {
            self.draws.extend(clip_updates);
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
        let (clip_updates, clip_id) = self.logical_state.prepare_scheduled_clip_updates(config)?;
        match &mut content.role {
            DrawRole::Content { clip_id: id } => *id = clip_id,
            DrawRole::ClipUpdate { .. } | DrawRole::ClipReset { .. } => {
                unreachable!("content batch must end in a content draw")
            }
        }
        let batch = clip_updates.iter().chain(std::iter::once(&content));
        self.try_plan_draws(batch)?;
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

#[derive(Default)]
struct ShadowBuffers {
    paths: Vec<u8>,
    paints: Vec<u8>,
    contours: Vec<u8>,
    tessellations: Vec<u8>,
    triangles: Vec<u8>,
    images: Vec<u8>,
    draws: Vec<u8>,
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
        };
        reserve_records(&mut self.paths, self.capacity.path_records, 64);
        reserve_records(&mut self.paints, self.capacity.paint_records, 32);
        reserve_records(&mut self.contours, self.capacity.contour_records, 8);
        reserve_records(
            &mut self.tessellations,
            self.capacity.tessellation_records,
            8,
        );
        reserve_records(&mut self.triangles, self.capacity.triangle_records, 8);
        reserve_records(&mut self.images, self.capacity.image_records, 16);
        reserve_records(&mut self.draws, self.capacity.draw_records, 16);
    }

    fn rewind(&mut self) {
        self.paths.clear();
        self.paints.clear();
        self.contours.clear();
        self.tessellations.clear();
        self.triangles.clear();
        self.images.clear();
        self.draws.clear();
    }

    fn fingerprint(&self) -> u64 {
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for bytes in [
            self.paths.as_slice(),
            self.paints.as_slice(),
            self.contours.as_slice(),
            self.tessellations.as_slice(),
            self.triangles.as_slice(),
            self.images.as_slice(),
            self.draws.as_slice(),
        ] {
            for &byte in bytes {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        hash
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

#[derive(Clone, Default)]
pub(crate) struct LogicalResourceStore {
    buffers: Arc<Mutex<ShadowBuffers>>,
}

impl LogicalResourceStore {
    pub(crate) fn prepare(&self, frame: &LogicalFrame) -> Result<LogicalFrameReport, &'static str> {
        frame.validate()?;
        let config = frame.config;
        let mut flushes = Vec::with_capacity(frame.logical_flush_starts.len().max(1));
        let starts = if frame.logical_flush_starts.is_empty() {
            &[0][..]
        } else {
            &frame.logical_flush_starts
        };
        let mut required = LogicalResourceCounts::default();
        for (index, &start) in starts.iter().enumerate() {
            let end = starts.get(index + 1).copied().unwrap_or(frame.draws.len());
            let counters = frame.draw_resources[start..end]
                .iter()
                .try_fold(logical_flush::ResourceCounters::default(), |total, draw| {
                    total.checked_add(*draw)
                })
                .ok_or("logical frame resource accounting overflow")?;
            let counts = LogicalResourceCounts::from_counters(counters);
            required = required
                .checked_add(counts)
                .ok_or("logical frame resource layout overflow")?;
            flushes.push(counts);
        }

        let mut buffers = self
            .buffers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        buffers.rewind();
        buffers.grow(required);
        write_resources(
            &mut buffers,
            config,
            &frame.draws,
            &frame.draw_resources,
            &flushes,
        );
        let report = LogicalFrameReport {
            draw_count: frame.draws.len(),
            resource_planning_passes: frame.resource_planning_evaluations,
            logical_flushes: flushes,
            retained_capacity: buffers.capacity,
            written: required,
            shadow_fingerprint: buffers.fingerprint(),
        };
        buffers.rewind();
        Ok(report)
    }
}

fn write_resources(
    buffers: &mut ShadowBuffers,
    config: LogicalFrameConfig,
    draws: &[SolidDraw],
    draw_resources: &[logical_flush::ResourceCounters],
    flushes: &[LogicalResourceCounts],
) {
    push_u32(&mut buffers.draws, config.width);
    push_u32(&mut buffers.draws, config.height);
    push_u8(&mut buffers.draws, mode_tag(config.mode));
    for counts in flushes {
        for value in [
            counts.path_records,
            counts.paint_records,
            counts.contour_records,
            counts.tessellation_records,
            counts.triangle_records,
            counts.image_records,
            counts.draw_records,
        ] {
            push_usize(&mut buffers.draws, value);
        }
    }
    for (draw, resources) in draws.iter().zip(draw_resources) {
        push_u8(&mut buffers.paths, fill_rule_tag(draw.path.fill_rule));
        for value in draw.state.transform.0 {
            push_u32(&mut buffers.paths, value.to_bits());
        }
        for verb in draw.path.raw_path.verbs() {
            push_u8(&mut buffers.paths, path_verb_tag(*verb));
        }
        for point in draw.path.raw_path.points() {
            push_u32(&mut buffers.paths, point.x.to_bits());
            push_u32(&mut buffers.paths, point.y.to_bits());
        }

        push_u8(&mut buffers.paints, paint_style_tag(draw.paint.style));
        push_u32(&mut buffers.paints, draw.paint.color);
        push_u32(&mut buffers.paints, draw.paint.thickness.to_bits());
        push_u8(&mut buffers.paints, stroke_join_tag(draw.paint.join));
        push_u8(&mut buffers.paints, stroke_cap_tag(draw.paint.cap));
        push_u32(&mut buffers.paints, draw.paint.feather.to_bits());
        push_u32(&mut buffers.paints, draw.state.opacity.to_bits());
        push_u8(&mut buffers.paints, draw.paint.blend_mode as u8);
        for value in [
            resources.path_count,
            resources.contour_count,
            resources.midpoint_fan_tess_vertex_count,
            resources.outer_cubic_tess_vertex_count,
            resources.max_tessellated_segment_count,
            resources.max_triangle_vertex_count,
        ] {
            push_usize(&mut buffers.tessellations, value);
        }
        push_usize(&mut buffers.contours, resources.contour_count);
        push_usize(&mut buffers.triangles, resources.max_triangle_vertex_count);
        push_usize(&mut buffers.images, resources.image_draw_count);
        push_usize(&mut buffers.draws, resources.draw_pass_count);
        match draw.role {
            DrawRole::Content { clip_id } => {
                push_u8(&mut buffers.draws, 0);
                push_u16(&mut buffers.draws, clip_id);
            }
            DrawRole::ClipUpdate {
                replacement_id,
                parent_id,
            } => {
                push_u8(&mut buffers.draws, 1);
                push_u16(&mut buffers.draws, replacement_id);
                push_u16(&mut buffers.draws, parent_id);
            }
            DrawRole::ClipReset { .. } => push_u8(&mut buffers.draws, 2),
        }
    }
}

fn push_u8(buffer: &mut Vec<u8>, value: u8) {
    buffer.push(value);
}

fn push_u16(buffer: &mut Vec<u8>, value: u16) {
    buffer.extend_from_slice(&value.to_le_bytes());
}

fn push_u32(buffer: &mut Vec<u8>, value: u32) {
    buffer.extend_from_slice(&value.to_le_bytes());
}

fn push_usize(buffer: &mut Vec<u8>, value: usize) {
    buffer.extend_from_slice(&(value as u64).to_le_bytes());
}

fn mode_tag(mode: RenderMode) -> u8 {
    match mode {
        RenderMode::Msaa => 0,
        RenderMode::ClockwiseAtomic => 1,
    }
}

fn fill_rule_tag(rule: FillRule) -> u8 {
    match rule {
        FillRule::NonZero => 0,
        FillRule::EvenOdd => 1,
        FillRule::Clockwise => 2,
    }
}

fn paint_style_tag(style: RenderPaintStyle) -> u8 {
    match style {
        RenderPaintStyle::Fill => 0,
        RenderPaintStyle::Stroke => 1,
    }
}

fn stroke_join_tag(join: StrokeJoin) -> u8 {
    match join {
        StrokeJoin::Miter => 0,
        StrokeJoin::Round => 1,
        StrokeJoin::Bevel => 2,
    }
}

fn stroke_cap_tag(cap: StrokeCap) -> u8 {
    match cap {
        StrokeCap::Butt => 0,
        StrokeCap::Round => 1,
        StrokeCap::Square => 2,
    }
}

fn path_verb_tag(verb: nuxie_render_api::PathVerb) -> u8 {
    match verb {
        nuxie_render_api::PathVerb::Move => 0,
        nuxie_render_api::PathVerb::Line => 1,
        nuxie_render_api::PathVerb::Quad => 2,
        nuxie_render_api::PathVerb::Cubic => 3,
        nuxie_render_api::PathVerb::Close => 4,
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
    scratch: super::draw::StrokePreparationScratch,
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
    frame: Option<NullFrame>,
}

impl NullLogicalRenderer {
    pub fn new(config: LogicalFrameConfig) -> Self {
        Self {
            config,
            resources: LogicalResourceStore::default(),
            frame: None,
        }
    }

    pub fn begin_frame(&mut self) {
        assert!(self.frame.is_none(), "null logical frame already active");
        self.frame = Some(NullFrame {
            logical: LogicalFrame::new(self.config),
            scratch: super::draw::StrokePreparationScratch::default(),
        });
    }

    pub fn save(&mut self) {
        self.active_frame().logical_state.save();
    }

    pub fn restore(&mut self) {
        self.active_frame().logical_state.restore();
    }

    pub fn transform(&mut self, transform: nuxie_render_api::Mat2D) {
        self.active_frame().logical_state.transform(transform);
    }

    pub fn clip_path(
        &mut self,
        raw_path: &RawPath,
        fill_rule: FillRule,
    ) -> Result<(), &'static str> {
        let config = self.config;
        let path = LogicalPath {
            raw_path: Arc::new(raw_path.clone()),
            fill_rule,
            valid: true,
        };
        self.active_frame().logical_state.clip_path(config, &path)
    }

    pub fn draw_path(
        &mut self,
        raw_path: &RawPath,
        fill_rule: FillRule,
        paint: LogicalPathPaint,
    ) -> Result<(), &'static str> {
        let config = self.config;
        let frame = self.active_frame();
        let path = LogicalPath {
            raw_path: Arc::new(raw_path.clone()),
            fill_rule,
            valid: true,
        };
        let paint = paint.into_wgpu();
        let Some(admitted) = admit_path_draw(config, frame.logical_state.state, &path, &paint)?
        else {
            return Ok(());
        };
        let (clip_updates, clip_id) = frame.logical_state.prepare_scheduled_clip_updates(config)?;
        let content = admitted.finish(clip_id, &mut frame.scratch)?;
        frame.logical.push_content_batch(clip_updates, content)
    }

    pub fn flush(&mut self) -> Result<LogicalFrameReport, &'static str> {
        let mut frame = self
            .frame
            .take()
            .expect("null logical frame must begin before flush");
        let report = self.resources.prepare(&frame.logical)?;
        frame.draws.clear();
        frame.logical_flush_starts.clear();
        frame.scratch.reset_for_reuse();
        Ok(report)
    }

    fn active_frame(&mut self) -> &mut NullFrame {
        self.frame
            .as_mut()
            .expect("null logical frame must begin before operation")
    }
}
