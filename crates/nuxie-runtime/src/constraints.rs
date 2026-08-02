use std::collections::BTreeSet;
use std::sync::OnceLock;

use nuxie_binary::RuntimeFile;
use nuxie_graph::ArtboardGraph;
use nuxie_render_api::Mat2D as RenderMat2D;

use crate::components::{
    ComponentHandle, RuntimeConstraintBoundsKind, RuntimeConstraintKind, RuntimeConstraintScratch,
    RuntimeConstraintState, RuntimeIkChainLink, RuntimeScrollAxis, RuntimeScrollAxisIntent,
    RuntimeScrollConstraintState, RuntimeScrollPhysicsState, RuntimeScrollSpace,
    RuntimeScrollVirtualizerState, TransformComponents, TransformProperty,
};
use crate::draw::{RuntimeLayoutBounds, RuntimePathMeasure};
use crate::objects::InstanceObjectArena;
use crate::properties::property_key_for_name;
use crate::text::static_text_constraint_bounds;
use crate::{ArtboardInstance, Mat2D};

mod distance_constraint;
mod rotation_constraint;
mod scale_constraint;
mod transform_constraint;
mod translation_constraint;

/// Read-only state of one imported `ScrollConstraint` in one concrete
/// [`ArtboardInstance`] occurrence.
///
/// `lower_bound`/`upper_bound` are the legal offset interval. They correspond
/// to pinned C++ `maxOffsetX/Y()` and `minOffsetX/Y()` respectively; the C++
/// names describe scroll extent rather than numeric ordering. `clamped_offset`
/// is the live `clampedOffsetX/Y()` result, including elastic physics while it
/// is enabled. `physics_running` is exactly `physics()->isRunning()`, not the
/// broader artboard advancing state.
///
/// Pinned C++: `scroll_constraint.hpp:99-130`,
/// `scroll_constraint.cpp:93-167`, and `scroll_physics.hpp:40-41` at
/// `d788e8ec6e8b598526607d6a1e8818e8b637b60c`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RuntimeScrollConstraintSnapshot {
    /// Local identity of this concrete constraint occurrence.
    pub constraint_local_id: usize,
    /// File-global authored identity from which this occurrence was cloned.
    pub constraint_authored_id: u32,
    /// Local identity of the constrained content component in this occurrence.
    pub content_local_id: usize,
    /// File-global authored identity of the constrained content component.
    pub content_authored_id: u32,
    /// Unclamped live `offsetX()`/`offsetY()` values.
    pub offset: (f32, f32),
    /// Numerically lower legal offset bound on each axis.
    pub lower_bound: (f32, f32),
    /// Numerically upper legal offset bound on each axis.
    pub upper_bound: (f32, f32),
    /// Physics-aware `clampedOffsetX()`/`clampedOffsetY()` values.
    pub clamped_offset: (f32, f32),
    /// Whether this constraint occurrence owns imported scroll physics.
    pub physics_present: bool,
    /// The owned physics occurrence's `isRunning()` value.
    pub physics_running: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeScrollProperty {
    Offset(RuntimeScrollAxis),
    Percent(RuntimeScrollAxis),
    Index,
}

/// One proxy/listener-group pair constructed by a concrete C++
/// `DraggableConstraint::listenerGroups` call. Every StateMachineInstance owns
/// a fresh set; the constraint and hittable remain non-owning occurrence
/// handles (`draggable_constraint.cpp:8-28`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeDraggableProxyKind {
    Viewport,
    Thumb,
    Track,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeDraggableProxy {
    pub(crate) constraint: ComponentHandle,
    pub(crate) hittable: ComponentHandle,
    pub(crate) kind: RuntimeDraggableProxyKind,
    pub(crate) opaque: bool,
    pub(crate) last_position: (f32, f32),
    pub(crate) viewport_is_dragging: bool,
    pub(crate) active_pointers: Vec<i32>,
    pub(crate) has_scrolled: bool,
}

impl RuntimeDraggableProxy {
    fn new(
        constraint: ComponentHandle,
        hittable: ComponentHandle,
        kind: RuntimeDraggableProxyKind,
        opaque: bool,
    ) -> Self {
        Self {
            constraint,
            hittable,
            kind,
            opaque,
            last_position: (0.0, 0.0),
            viewport_is_dragging: false,
            active_pointers: Vec::new(),
            has_scrolled: false,
        }
    }

    pub(crate) fn clone_cold(&self) -> Self {
        Self::new(self.constraint, self.hittable, self.kind, self.opaque)
    }
}

fn runtime_scroll_property(property_key: u16) -> Option<RuntimeScrollProperty> {
    let [offset_x, offset_y, percent_x, percent_y, index] = *runtime_scroll_property_keys();
    if offset_x == Some(property_key) {
        Some(RuntimeScrollProperty::Offset(RuntimeScrollAxis::X))
    } else if offset_y == Some(property_key) {
        Some(RuntimeScrollProperty::Offset(RuntimeScrollAxis::Y))
    } else if percent_x == Some(property_key) {
        Some(RuntimeScrollProperty::Percent(RuntimeScrollAxis::X))
    } else if percent_y == Some(property_key) {
        Some(RuntimeScrollProperty::Percent(RuntimeScrollAxis::Y))
    } else if index == Some(property_key) {
        Some(RuntimeScrollProperty::Index)
    } else {
        None
    }
}

fn runtime_scroll_property_keys() -> &'static [Option<u16>; 5] {
    static KEYS: OnceLock<[Option<u16>; 5]> = OnceLock::new();
    KEYS.get_or_init(|| {
        [
            property_key_for_name("ScrollConstraint", "scrollOffsetX"),
            property_key_for_name("ScrollConstraint", "scrollOffsetY"),
            property_key_for_name("ScrollConstraint", "scrollPercentX"),
            property_key_for_name("ScrollConstraint", "scrollPercentY"),
            property_key_for_name("ScrollConstraint", "scrollIndex"),
        ]
    })
}

fn runtime_scroll_intent_axes(
    property: RuntimeScrollProperty,
    direction: u64,
) -> Vec<(RuntimeScrollAxis, RuntimeScrollSpace)> {
    match property {
        RuntimeScrollProperty::Percent(axis) => vec![(axis, RuntimeScrollSpace::Percent)],
        RuntimeScrollProperty::Index => {
            let mut axes = Vec::with_capacity(2);
            if matches!(direction, 0 | 2) {
                axes.push((RuntimeScrollAxis::X, RuntimeScrollSpace::Index));
            }
            if matches!(direction, 1 | 2) {
                axes.push((RuntimeScrollAxis::Y, RuntimeScrollSpace::Index));
            }
            axes
        }
        RuntimeScrollProperty::Offset(_) => Vec::new(),
    }
}

impl RuntimeScrollAxisIntent {
    fn read(self, space: RuntimeScrollSpace) -> Option<f32> {
        (self.space == space).then_some(self.value)
    }

    fn resolve(
        self,
        axis: RuntimeScrollAxis,
        metrics: Option<&RuntimeScrollLayoutMetrics>,
    ) -> Option<f32> {
        if self.space == RuntimeScrollSpace::Index
            && (self.value.is_nan()
                || (metrics.is_some_and(|metrics| metrics.infinite) && !self.value.is_finite()))
        {
            return Some(0.0);
        }
        let metrics = metrics?;
        if !metrics.layout_resolvable(axis) {
            return None;
        }
        match self.space {
            RuntimeScrollSpace::Percent => {
                let content_size = metrics.content_size(axis);
                if content_size <= 0.0 {
                    return None;
                }
                Some(
                    metrics.clamp_resolved_offset(
                        self.value * metrics.max_offset_for_percent(axis),
                        axis,
                    ),
                )
            }
            RuntimeScrollSpace::Index => {
                let position = metrics.position_at_index(self.value)?;
                let offset = match axis {
                    RuntimeScrollAxis::X => position.0,
                    RuntimeScrollAxis::Y => position.1,
                };
                Some(metrics.clamp_resolved_offset(offset, axis))
            }
        }
    }
}

#[derive(Debug, Clone)]
struct RuntimeScrollLayoutMetrics {
    direction: u64,
    infinite: bool,
    main_axis_horizontal: bool,
    viewport_layout_width: f32,
    viewport_layout_height: f32,
    viewport_width: f32,
    viewport_height: f32,
    content_width: f32,
    content_height: f32,
    trailing_padding_x: f32,
    trailing_padding_y: f32,
    gap_x: f32,
    gap_y: f32,
    item_bounds: Vec<RuntimeLayoutBounds>,
}

impl RuntimeScrollLayoutMetrics {
    fn layout_resolvable(&self, axis: RuntimeScrollAxis) -> bool {
        match axis {
            RuntimeScrollAxis::X => self.viewport_layout_width > 0.0,
            RuntimeScrollAxis::Y => self.viewport_layout_height > 0.0,
        }
    }

    fn viewport_size(&self, axis: RuntimeScrollAxis) -> f32 {
        match axis {
            RuntimeScrollAxis::X => self.viewport_width,
            RuntimeScrollAxis::Y => self.viewport_height,
        }
    }

    fn content_size(&self, axis: RuntimeScrollAxis) -> f32 {
        match axis {
            RuntimeScrollAxis::X => self.content_width,
            RuntimeScrollAxis::Y => self.content_height,
        }
    }

    fn trailing_padding(&self, axis: RuntimeScrollAxis) -> f32 {
        match axis {
            RuntimeScrollAxis::X => self.trailing_padding_x,
            RuntimeScrollAxis::Y => self.trailing_padding_y,
        }
    }

    fn max_offset(&self, axis: RuntimeScrollAxis) -> f32 {
        if self.infinite && self.main_axis() == axis {
            return f32::NEG_INFINITY;
        }
        (self.viewport_size(axis) - self.content_size(axis) - self.trailing_padding(axis)).min(0.0)
    }

    fn max_offset_for_percent(&self, axis: RuntimeScrollAxis) -> f32 {
        if self.infinite {
            self.content_size(axis)
        } else {
            self.max_offset(axis)
        }
    }

    fn clamp_resolved_offset(&self, value: f32, axis: RuntimeScrollAxis) -> f32 {
        if self.infinite {
            value
        } else {
            value.clamp(self.max_offset(axis), 0.0)
        }
    }

    fn main_axis(&self) -> RuntimeScrollAxis {
        if self.main_axis_horizontal {
            RuntimeScrollAxis::X
        } else {
            RuntimeScrollAxis::Y
        }
    }

    fn constrains_horizontal(&self) -> bool {
        matches!(self.direction, 0 | 2)
    }

    fn constrains_vertical(&self) -> bool {
        matches!(self.direction, 1 | 2)
    }

    fn gap(&self, axis: RuntimeScrollAxis) -> f32 {
        match axis {
            RuntimeScrollAxis::X => self.gap_x,
            RuntimeScrollAxis::Y => self.gap_y,
        }
    }

    fn bounds_collapsed(&self, bounds: RuntimeLayoutBounds) -> bool {
        (self.constrains_horizontal() && bounds.width <= 0.0)
            || (self.constrains_vertical() && bounds.height <= 0.0)
    }

    fn position_at_index(&self, index: f32) -> Option<(f32, f32)> {
        if index.is_nan() || (self.infinite && !index.is_finite()) {
            return Some((0.0, 0.0));
        }
        let count = self.item_bounds.len();
        if count == 0 {
            return None;
        }

        let normalized_index = if self.infinite {
            let mut normalized = index % count as f32;
            if normalized < 0.0 {
                normalized += count as f32;
            }
            normalized
        } else {
            let normalized = index.max(0.0);
            if normalized >= count as f32 {
                if self.content_width <= 0.0 && self.content_height <= 0.0 {
                    return None;
                }
                return Some((-self.content_width, -self.content_height));
            }
            normalized
        };

        let floor_index = normalized_index.floor();
        let fractional = normalized_index - floor_index;
        let target_index = floor_index as usize;
        let target = self.item_bounds[target_index];
        if !self.bounds_collapsed(target) {
            return Some((
                -target.x - (target.width + self.gap(RuntimeScrollAxis::X)) * fractional,
                -target.y - (target.height + self.gap(RuntimeScrollAxis::Y)) * fractional,
            ));
        }

        if let Some(bounds) = self
            .item_bounds
            .iter()
            .skip(target_index + 1)
            .copied()
            .find(|bounds| !self.bounds_collapsed(*bounds))
        {
            return Some((-bounds.x, -bounds.y));
        }
        if self.infinite
            && let Some(bounds) = self
                .item_bounds
                .iter()
                .take(target_index)
                .copied()
                .find(|bounds| !self.bounds_collapsed(*bounds))
        {
            return Some((-bounds.x, -bounds.y));
        }
        if !self.infinite
            && let Some(bounds) = self
                .item_bounds
                .iter()
                .take(target_index)
                .rev()
                .copied()
                .find(|bounds| !self.bounds_collapsed(*bounds))
        {
            return Some((-bounds.x, -bounds.y));
        }
        None
    }

    fn index_at_position(&self, position: (f32, f32)) -> f32 {
        let axis = if self.constrains_horizontal() {
            RuntimeScrollAxis::X
        } else if self.constrains_vertical() {
            RuntimeScrollAxis::Y
        } else {
            return 0.0;
        };
        let position = match axis {
            RuntimeScrollAxis::X => position.0,
            RuntimeScrollAxis::Y => position.1,
        };
        let gap = self.gap(axis);
        for (index, bounds) in self.item_bounds.iter().enumerate() {
            let (origin, size) = match axis {
                RuntimeScrollAxis::X => (bounds.x, bounds.width),
                RuntimeScrollAxis::Y => (bounds.y, bounds.height),
            };
            let step = size + gap;
            if position > -origin - step {
                return if step != 0.0 {
                    index as f32 + (-position - origin) / step
                } else {
                    index as f32
                };
            }
        }
        self.item_bounds.len() as f32
    }

    #[cfg(test)]
    fn vertical_for_test(
        viewport_height: f32,
        content_height: f32,
        gap_y: f32,
        item_bounds: Vec<RuntimeLayoutBounds>,
    ) -> Self {
        Self {
            direction: 1,
            infinite: false,
            main_axis_horizontal: false,
            viewport_layout_width: 0.0,
            viewport_layout_height: viewport_height,
            viewport_width: 0.0,
            viewport_height,
            content_width: 0.0,
            content_height,
            trailing_padding_x: 0.0,
            trailing_padding_y: 0.0,
            gap_x: 0.0,
            gap_y,
            item_bounds,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransformSpace {
    World,
    Local,
}

impl TransformSpace {
    fn from_value(value: u64) -> Self {
        match value {
            1 => Self::Local,
            _ => Self::World,
        }
    }
}

/// C++ `ScrollConstraint::clampedOffsetX/Y` first honors infinite scrolling,
/// then delegates finite overscroll to the retained physics owner while it is
/// enabled, and uses a plain bounds clamp otherwise
/// (`scroll_constraint.cpp:129-167`).
fn clamped_scroll_constraint_offsets(
    artboard: &ArtboardInstance,
    constraint: ComponentHandle,
    metrics: &RuntimeScrollLayoutMetrics,
) -> (f32, f32) {
    let scroll = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.concrete.scroll.as_ref())
        .expect("ScrollConstraint occurrence retains its concrete state");
    let raw = (scroll.offset_x, scroll.offset_y);
    if metrics.infinite {
        return raw;
    }
    let range_min = (
        metrics.max_offset(RuntimeScrollAxis::X),
        metrics.max_offset(RuntimeScrollAxis::Y),
    );
    if let Some(physics) = scroll.physics.as_ref().filter(|physics| physics.enabled()) {
        return physics.clamp(range_min, (0.0, 0.0), raw);
    }
    (raw.0.clamp(range_min.0, 0.0), raw.1.clamp(range_min.1, 0.0))
}

impl ArtboardInstance {
    /// Snapshot every imported ScrollConstraint owned by this exact artboard
    /// occurrence, preserving authored/local order.
    pub fn scroll_constraint_occurrences(&self) -> Vec<RuntimeScrollConstraintSnapshot> {
        self.slots
            .iter()
            .filter(|slot| slot.type_name == Some("ScrollConstraint"))
            .filter_map(|slot| self.scroll_constraint_snapshot(slot.local_id))
            .collect()
    }

    /// Find the ScrollConstraint occurrence that constrains `content_local_id`.
    pub fn scroll_constraint_for_content(
        &self,
        content_local_id: usize,
    ) -> Option<RuntimeScrollConstraintSnapshot> {
        self.slots
            .iter()
            .filter(|slot| slot.type_name == Some("ScrollConstraint"))
            .find_map(|slot| {
                self.scroll_constraint_snapshot(slot.local_id)
                    .filter(|snapshot| snapshot.content_local_id == content_local_id)
            })
    }

    /// Find the occurrence cloned from one authored ScrollConstraint object.
    pub fn scroll_constraint_for_authored_id(
        &self,
        constraint_authored_id: u32,
    ) -> Option<RuntimeScrollConstraintSnapshot> {
        self.slots
            .iter()
            .find(|slot| {
                slot.type_name == Some("ScrollConstraint")
                    && slot.source_global_id == constraint_authored_id
            })
            .and_then(|slot| self.scroll_constraint_snapshot(slot.local_id))
    }

    /// Find the ScrollConstraint occurrence whose constrained component was
    /// cloned from `content_authored_id`.
    pub fn scroll_constraint_for_content_authored_id(
        &self,
        content_authored_id: u32,
    ) -> Option<RuntimeScrollConstraintSnapshot> {
        self.slots
            .iter()
            .filter(|slot| slot.type_name == Some("ScrollConstraint"))
            .find_map(|slot| {
                self.scroll_constraint_snapshot(slot.local_id)
                    .filter(|snapshot| snapshot.content_authored_id == content_authored_id)
            })
    }

    fn scroll_constraint_snapshot(
        &self,
        constraint_local_id: usize,
    ) -> Option<RuntimeScrollConstraintSnapshot> {
        let constraint_handle = self.component_handle(constraint_local_id)?;
        let constraint = self
            .objects
            .component(constraint_handle)?
            .concrete
            .scroll
            .as_ref()?;
        let content_handle = constraint.content?;
        let content_local_id = self.objects.component_local_id(content_handle)?;
        let metrics = runtime_scroll_layout_metrics(self, constraint_handle, constraint, false)
            .unwrap_or_else(|| {
                build_runtime_scroll_layout_metrics(
                    self,
                    constraint_handle,
                    constraint,
                    None,
                    false,
                )
            });
        let lower_bound = (
            metrics.max_offset(RuntimeScrollAxis::X),
            metrics.max_offset(RuntimeScrollAxis::Y),
        );
        let upper_bound = if metrics.infinite {
            match metrics.main_axis() {
                RuntimeScrollAxis::X => (f32::INFINITY, 0.0),
                RuntimeScrollAxis::Y => (0.0, f32::INFINITY),
            }
        } else {
            (0.0, 0.0)
        };
        let physics = constraint.physics.as_ref();
        Some(RuntimeScrollConstraintSnapshot {
            constraint_local_id,
            constraint_authored_id: self.slot(constraint_local_id)?.source_global_id,
            content_local_id,
            content_authored_id: self.slot(content_local_id)?.source_global_id,
            offset: (constraint.offset_x, constraint.offset_y),
            lower_bound,
            upper_bound,
            clamped_offset: clamped_scroll_constraint_offsets(self, constraint_handle, &metrics),
            physics_present: physics.is_some(),
            physics_running: physics.is_some_and(RuntimeScrollPhysicsState::is_running),
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct RuntimeConstraintPropertyKeys {
    strength: u16,
    target_id: u16,
    source_space: u16,
    dest_space: u16,
    min_max_space: u16,
    copy_factor: u16,
    min_value: u16,
    max_value: u16,
    offset: u16,
    does_copy: u16,
    min: u16,
    max: u16,
    copy_factor_y: u16,
    min_value_y: u16,
    max_value_y: u16,
    does_copy_y: u16,
    min_y: u16,
    max_y: u16,
    distance: u16,
    mode: u16,
    origin_x: u16,
    origin_y: u16,
}

// Pinned generated property keys from the same schema that emits the C++
// generated constraint members. Keep these compile-time constants in the
// frame loop: C++ leaf getters are direct member reads, not lazy schema
// lookups.
const RUNTIME_CONSTRAINT_PROPERTY_KEYS: RuntimeConstraintPropertyKeys =
    RuntimeConstraintPropertyKeys {
        strength: 172,
        target_id: 173,
        source_space: 179,
        dest_space: 180,
        min_max_space: 195,
        copy_factor: 182,
        min_value: 183,
        max_value: 184,
        offset: 188,
        does_copy: 189,
        min: 190,
        max: 191,
        copy_factor_y: 185,
        min_value_y: 186,
        max_value_y: 187,
        does_copy_y: 192,
        min_y: 193,
        max_y: 194,
        distance: 177,
        mode: 178,
        origin_x: 372,
        origin_y: 373,
    };
const BONE_LENGTH_PROPERTY_KEY: u16 = 89;
pub(crate) const IK_INVERT_DIRECTION_PROPERTY_KEY: u16 = 174;
pub(crate) const IK_PARENT_BONE_COUNT_PROPERTY_KEY: u16 = 175;
const FOLLOW_PATH_DISTANCE_PROPERTY_KEY: u16 = 363;
const FOLLOW_PATH_ORIENT_PROPERTY_KEY: u16 = 364;
const FOLLOW_PATH_OFFSET_PROPERTY_KEY: u16 = 365;
const LIST_FOLLOW_PATH_DISTANCE_END_PROPERTY_KEY: u16 = 888;
const LIST_FOLLOW_PATH_DISTANCE_OFFSET_PROPERTY_KEY: u16 = 889;

impl RuntimeScrollPhysicsState {
    fn enabled(&self) -> bool {
        match &self.kind {
            crate::components::RuntimeScrollPhysicsKind::Clamped { .. } => self.is_running,
            crate::components::RuntimeScrollPhysicsKind::Elastic { x, y } => {
                x.is_some() || y.is_some()
            }
        }
    }

    fn is_running(&self) -> bool {
        match &self.kind {
            crate::components::RuntimeScrollPhysicsKind::Clamped { .. } => self.is_running,
            crate::components::RuntimeScrollPhysicsKind::Elastic { x, y } => {
                x.as_ref().is_some_and(|helper| helper.is_running)
                    || y.as_ref().is_some_and(|helper| helper.is_running)
            }
        }
    }

    fn stop(&mut self) {
        self.is_running = false;
        self.speed = (0.0, 0.0);
    }

    fn reset(&mut self) {
        self.last_time_micros = 0;
        self.speed = (0.0, 0.0);
        self.acceleration = (0.0, 0.0);
        self.stop();
        if let crate::components::RuntimeScrollPhysicsKind::Elastic { x, y } = &mut self.kind {
            *x = None;
            *y = None;
        }
    }

    fn prepare(&mut self, direction: u64) {
        self.reset();
        self.direction = direction;
        if let crate::components::RuntimeScrollPhysicsKind::Elastic { x, y } = &mut self.kind {
            if matches!(direction, 0 | 2) {
                *x = Some(crate::components::RuntimeElasticScrollPhysicsHelper::new(
                    self.friction,
                    self.speed_multiplier,
                    self.elastic_factor,
                ));
            }
            if matches!(direction, 1 | 2) {
                *y = Some(crate::components::RuntimeElasticScrollPhysicsHelper::new(
                    self.friction,
                    self.speed_multiplier,
                    self.elastic_factor,
                ));
            }
        }
    }

    fn clear_velocity(&mut self) {
        self.speed = (0.0, 0.0);
    }

    fn accumulate(&mut self, delta: (f32, f32), timestamp: f32) {
        // Canonical runtime/probe execution uses C++ deterministicMode: the
        // pointer timestamp is the clock and reset seeds zero
        // (`scroll_physics.cpp:8-34,36-51`).
        let elapsed_seconds = timestamp - self.last_time_micros as f32;
        self.last_time_micros = timestamp as i64;
        if elapsed_seconds > 0.0 {
            let last_speed = self.speed;
            self.speed = (delta.0 / elapsed_seconds, delta.1 / elapsed_seconds);
            self.acceleration = (
                (last_speed.0 + self.speed.0) / elapsed_seconds,
                (last_speed.1 + self.speed.1) / elapsed_seconds,
            );
        }
    }

    fn clamp(&self, range_min: (f32, f32), range_max: (f32, f32), value: (f32, f32)) -> (f32, f32) {
        match &self.kind {
            crate::components::RuntimeScrollPhysicsKind::Clamped { .. } => (
                value.0.clamp(range_min.0, range_max.0),
                value.1.clamp(range_min.1, range_max.1),
            ),
            crate::components::RuntimeScrollPhysicsKind::Elastic { x, y } => (
                x.as_ref().map_or(0.0, |helper| {
                    helper.clamp(range_min.0, range_max.0, value.0)
                }),
                y.as_ref().map_or(0.0, |helper| {
                    helper.clamp(range_min.1, range_max.1, value.1)
                }),
            ),
        }
    }

    fn run(
        &mut self,
        range_min: (f32, f32),
        range_max: (f32, f32),
        value: (f32, f32),
        snapping_points: &[(f32, f32)],
        content_size: f32,
        viewport_size: f32,
    ) {
        self.is_running = true;
        match &mut self.kind {
            crate::components::RuntimeScrollPhysicsKind::Clamped { value: retained } => {
                *retained = (
                    value.0.clamp(range_min.0, range_max.0),
                    value.1.clamp(range_min.1, range_max.1),
                );
            }
            crate::components::RuntimeScrollPhysicsKind::Elastic { x, y } => {
                let x_points = snapping_points
                    .iter()
                    .map(|point| point.0)
                    .collect::<Vec<_>>();
                let y_points = snapping_points
                    .iter()
                    .map(|point| point.1)
                    .collect::<Vec<_>>();
                if let Some(helper) = x {
                    helper.run(
                        self.acceleration.0,
                        range_min.0,
                        range_max.0,
                        value.0,
                        &x_points,
                        content_size,
                        viewport_size,
                    );
                }
                if let Some(helper) = y {
                    helper.run(
                        self.acceleration.1,
                        range_min.1,
                        range_max.1,
                        value.1,
                        &y_points,
                        content_size,
                        viewport_size,
                    );
                }
            }
        }
    }

    fn advance(&mut self, elapsed_seconds: f32) -> (f32, f32) {
        match &mut self.kind {
            crate::components::RuntimeScrollPhysicsKind::Clamped { value } => {
                let result = *value;
                self.stop();
                result
            }
            crate::components::RuntimeScrollPhysicsKind::Elastic { x, y } => {
                let previous = (
                    x.as_ref().map_or(0.0, |helper| helper.current),
                    y.as_ref().map_or(0.0, |helper| helper.current),
                );
                let result = (
                    x.as_mut()
                        .map_or(0.0, |helper| helper.advance(elapsed_seconds)),
                    y.as_mut()
                        .map_or(0.0, |helper| helper.advance(elapsed_seconds)),
                );
                if elapsed_seconds > 0.0 {
                    self.speed = (
                        (result.0 - previous.0) / elapsed_seconds,
                        (result.1 - previous.1) / elapsed_seconds,
                    );
                }
                let running = x.as_ref().is_some_and(|helper| helper.is_running)
                    || y.as_ref().is_some_and(|helper| helper.is_running);
                if !running {
                    self.reset();
                }
                result
            }
        }
    }
}

impl crate::components::RuntimeElasticScrollPhysicsHelper {
    fn advance(&mut self, elapsed_seconds: f32) -> f32 {
        if self.speed != 0.0 {
            self.current += self.speed * elapsed_seconds;
            if self.current < self.run_range_min {
                self.friction *= 4.0;
            } else if self.current > self.run_range_max {
                self.friction *= 4.0;
            }
            self.speed += -self.speed * (elapsed_seconds * self.friction).min(1.0);
            if self.speed.abs() < 5.0 {
                self.speed = 0.0;
                self.target = if self.current < self.run_range_min {
                    self.run_range_min
                } else if self.current > self.run_range_max {
                    self.run_range_max
                } else {
                    self.current
                };
            }
            return self.current;
        }
        let diff = self.target - self.current;
        if diff.abs() < 0.1 {
            self.current = if self.snap_target.is_nan() {
                self.target
            } else {
                self.snap_target
            };
            self.is_running = false;
        } else {
            self.current += diff * (elapsed_seconds * 15.0).min(1.0);
        }
        self.current
    }

    fn clamp(&self, range_min: f32, range_max: f32, value: f32) -> f32 {
        if value < range_min {
            range_min - (-(value - range_min)).powf(self.elastic_factor)
        } else if value > range_max {
            // Preserve pinned C++'s literal `value + rangeMax`, including its
            // asymmetric behavior for non-zero maxima.
            range_max + (value + range_max).powf(self.elastic_factor)
        } else {
            value
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn run(
        &mut self,
        acceleration: f32,
        range_min: f32,
        range_max: f32,
        value: f32,
        snapping_points: &[f32],
        content_size: f32,
        viewport_size: f32,
    ) {
        let _ = viewport_size;
        self.is_running = true;
        self.run_range_min = range_min;
        self.run_range_max = range_max;
        self.speed = if acceleration.abs() > 100.0 {
            acceleration * 0.16 * 0.16 * 0.1 * self.speed_multiplier
        } else {
            0.0
        };
        self.target = value.clamp(range_min, range_max);
        self.current = value;
        if snapping_points.is_empty() {
            self.snap_target = f32::NAN;
            return;
        }
        let end_target = -(self.current + self.speed / self.friction);
        let section_size = if content_size != 0.0 {
            content_size
        } else {
            1.0
        };
        let multiple = if range_max == f32::INFINITY {
            (end_target / section_size).floor()
        } else {
            0.0
        };
        let mod_end_target = if range_max == f32::INFINITY {
            ((end_target % section_size) + section_size) % section_size
        } else {
            end_target
        };
        let max_target = if range_max == f32::INFINITY {
            f32::INFINITY
        } else {
            -range_min
        };
        let mut closest = f32::MAX;
        let mut snap_target = 0.0;
        for snap in snapping_points {
            let diff = (*snap - mod_end_target).abs();
            if diff < closest {
                closest = diff;
                snap_target = *snap + multiple * section_size;
            }
        }
        if max_target != f32::INFINITY {
            let diff = (max_target - mod_end_target).abs();
            if diff < closest {
                snap_target = max_target;
            }
        }
        snap_target = snap_target.min(max_target);
        self.speed = -(snap_target + self.current) * self.friction;
        self.snap_target = -snap_target;
    }
}

pub(crate) fn targeted_constraint_target_id_property_key() -> Option<u16> {
    Some(RUNTIME_CONSTRAINT_PROPERTY_KEYS.target_id)
}

pub(crate) fn constraint_double_change_marks_parent_dirty(
    kind: RuntimeConstraintKind,
    property_key: u16,
) -> bool {
    let keys = RUNTIME_CONSTRAINT_PROPERTY_KEYS;
    (keys.strength == property_key && kind != RuntimeConstraintKind::Ik)
        || (kind == RuntimeConstraintKind::Distance && keys.distance == property_key)
        || (matches!(
            kind,
            RuntimeConstraintKind::FollowPath | RuntimeConstraintKind::ListFollowPath
        ) && matches!(
            property_key,
            FOLLOW_PATH_DISTANCE_PROPERTY_KEY
                | LIST_FOLLOW_PATH_DISTANCE_END_PROPERTY_KEY
                | LIST_FOLLOW_PATH_DISTANCE_OFFSET_PROPERTY_KEY
        ))
        || (kind == RuntimeConstraintKind::Transform
            && (keys.origin_x == property_key || keys.origin_y == property_key))
}

pub(crate) fn constraint_is_ik_strength_property(
    kind: RuntimeConstraintKind,
    property_key: u16,
) -> bool {
    kind == RuntimeConstraintKind::Ik && property_key == RUNTIME_CONSTRAINT_PROPERTY_KEYS.strength
}

pub(crate) fn follow_path_orient_property_key() -> u16 {
    FOLLOW_PATH_ORIENT_PROPERTY_KEY
}

pub(crate) fn constraint_uint_change_marks_parent_dirty(
    kind: RuntimeConstraintKind,
    property_key: u16,
) -> bool {
    kind == RuntimeConstraintKind::Distance && RUNTIME_CONSTRAINT_PROPERTY_KEYS.mode == property_key
}

pub(crate) fn retain_runtime_scroll_constraints(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    objects: &mut InstanceObjectArena,
) -> Vec<ComponentHandle> {
    let physics = file
        .objects
        .iter()
        .flatten()
        .filter_map(|object| match object.type_name {
            "ClampedScrollPhysics" => Some(RuntimeScrollPhysicsState::clamped()),
            "ElasticScrollPhysics" => Some(RuntimeScrollPhysicsState::elastic(
                object.double_property("friction").unwrap_or(8.0),
                object.double_property("speedMultiplier").unwrap_or(1.0),
                object.double_property("elasticFactor").unwrap_or(0.66),
            )),
            _ => None,
        })
        .collect::<Vec<_>>();
    let physics_id_key = property_key_for_name("ScrollConstraint", "physicsId");
    let virtualize_key = property_key_for_name("ScrollConstraint", "virtualize");
    let offset_x_key = property_key_for_name("ScrollConstraint", "scrollOffsetX");
    let offset_y_key = property_key_for_name("ScrollConstraint", "scrollOffsetY");
    let mut handles = Vec::new();
    for object in graph
        .local_objects
        .iter()
        .filter(|object| object.type_name == Some("ScrollConstraint"))
    {
        let Some(handle) = objects.component_handle(object.local_id) else {
            continue;
        };
        let content = objects
            .component(handle)
            .and_then(|component| component.parent);
        let layout_children = graph
            .layout_constraint_registrations
            .iter()
            .filter(|registration| registration.constraint_local == object.local_id)
            .filter_map(|registration| objects.component_handle(registration.layout_provider_local))
            .collect::<Vec<_>>();
        for child in &layout_children {
            if let Some(list) = objects
                .component_mut(*child)
                .and_then(|component| component.concrete.constrainable_list.as_mut())
            {
                list.layout_constraints.push(handle);
            } else if let Some(layout) = objects
                .component_mut(*child)
                .and_then(|component| component.concrete.layout.as_mut())
            {
                layout.layout_constraints.push(handle);
            }
        }
        let has_list_children = layout_children.iter().any(|child| {
            objects
                .component(*child)
                .is_some_and(|component| component.concrete.constrainable_list.is_some())
        });
        let owned_physics = physics_id_key
            .and_then(|key| objects.uint_property(object.local_id, key))
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|index| physics.get(index))
            .map(RuntimeScrollPhysicsState::clone_for_constraint);
        let virtualizer = virtualize_key
            .and_then(|key| objects.bool_property(object.local_id, key))
            .unwrap_or(false)
            .then(RuntimeScrollVirtualizerState::default);
        let offset_x = offset_x_key
            .and_then(|key| objects.double_property(object.local_id, key))
            .unwrap_or(0.0);
        let offset_y = offset_y_key
            .and_then(|key| objects.double_property(object.local_id, key))
            .unwrap_or(0.0);
        let Some(scroll) = objects
            .component_mut(handle)
            .and_then(|component| component.concrete.scroll.as_mut())
        else {
            continue;
        };
        scroll.content = content;
        scroll.layout_children = layout_children;
        scroll.has_list_children = has_list_children;
        scroll.physics = owned_physics;
        scroll.virtualizer = virtualizer;
        scroll.offset_x = offset_x;
        scroll.offset_y = offset_y;
        handles.push(handle);
    }
    let scroll_constraint_id_key =
        property_key_for_name("ScrollBarConstraint", "scrollConstraintId");
    for object in graph
        .local_objects
        .iter()
        .filter(|object| object.type_name == Some("ScrollBarConstraint"))
    {
        let Some(handle) = objects.component_handle(object.local_id) else {
            continue;
        };
        let target = scroll_constraint_id_key
            .and_then(|key| objects.uint_property(object.local_id, key))
            .and_then(|local| usize::try_from(local).ok())
            .and_then(|local| objects.component_handle(local))
            .filter(|target| {
                objects
                    .component(*target)
                    .is_some_and(|component| component.concrete.scroll.is_some())
            });
        if let Some(scroll_bar) = objects
            .component_mut(handle)
            .and_then(|component| component.concrete.scroll_bar.as_mut())
        {
            // Pinned `onAddedDirty` rejects an unresolved/non-Scroll target;
            // validated graph occurrences therefore retain exactly this one
            // handle (`scroll_bar_constraint.cpp:126-140,234-239`).
            scroll_bar.scroll_constraint = target;
        }
    }
    handles
}

/// Construct the exact component-provided draggable groups in authored
/// occurrence order for one StateMachineInstance
/// (`state_machine_instance.cpp:1969-2013`;
/// `draggable_constraint.cpp:8-28`).
pub(crate) fn runtime_draggable_proxies(artboard: &ArtboardInstance) -> Vec<RuntimeDraggableProxy> {
    let mut proxies = Vec::new();
    for component in artboard.components().iter() {
        let Some(handle) = artboard.component_handle(component.local_id) else {
            continue;
        };
        if let Some(scroll) = component.concrete.scroll.as_ref()
            && let Some(viewport) = scroll
                .content
                .and_then(|content| artboard.objects.component(content)?.parent)
        {
            proxies.push(RuntimeDraggableProxy::new(
                handle,
                viewport,
                RuntimeDraggableProxyKind::Viewport,
                false,
            ));
        }
        if component.concrete.scroll_bar.is_some()
            && let Some(thumb) = component.parent
            && let Some(track) = artboard
                .objects
                .component(thumb)
                .and_then(|thumb| thumb.parent)
        {
            proxies.push(RuntimeDraggableProxy::new(
                handle,
                thumb,
                RuntimeDraggableProxyKind::Thumb,
                true,
            ));
            proxies.push(RuntimeDraggableProxy::new(
                handle,
                track,
                RuntimeDraggableProxyKind::Track,
                false,
            ));
        }
    }
    let hit_order = artboard.runtime_hit_component_order();
    proxies.sort_by_key(|proxy| {
        hit_order
            .iter()
            .position(|component| *component == proxy.hittable)
            .unwrap_or(usize::MAX)
    });
    proxies
}

pub(crate) fn runtime_draggable_proxy_hit_test(
    artboard: &ArtboardInstance,
    proxy: &RuntimeDraggableProxy,
    position: (f32, f32),
) -> bool {
    // Each C++ draggable retains a LayoutComponent::proxy(). HitLayout starts
    // the exact virtual chain with skipOnUnclipped=false/isPrimaryHit=true;
    // Layout then preserves both flags through Drawable and Component parent
    // fallback (`state_machine_instance.cpp:890-902`;
    // `layout_component.cpp:49-80`; `drawable.cpp:62-77`).
    artboard.component_hit_test_point(proxy.hittable, position, false, true)
}

pub(crate) fn runtime_draggable_proxy_start(
    artboard: &mut ArtboardInstance,
    proxy: &mut RuntimeDraggableProxy,
    position: (f32, f32),
    timestamp: f32,
) {
    proxy.has_scrolled = false;
    match proxy.kind {
        RuntimeDraggableProxyKind::Viewport => {
            proxy.viewport_is_dragging = false;
            let local = artboard.component_at(proxy.constraint).local_id;
            if !constraint_bool(artboard, local, "ScrollConstraint", "interactive", true) {
                return;
            }
            proxy.last_position = position;
            let direction =
                constraint_uint(artboard, local, "DraggableConstraint", "directionValue", 1);
            if let Some(scroll) = artboard
                .objects
                .component_mut(proxy.constraint)
                .and_then(|component| component.concrete.scroll.as_mut())
            {
                scroll.is_dragging = true;
                scroll.intent_x = None;
                scroll.intent_y = None;
                scroll.last_frame_offset_x = scroll.offset_x;
                scroll.last_frame_offset_y = scroll.offset_y;
                if let Some(physics) = scroll.physics.as_mut() {
                    physics.prepare(direction);
                }
            }
        }
        RuntimeDraggableProxyKind::Thumb => {
            proxy.last_position = position;
            let Some(scroll_constraint) = artboard
                .objects
                .component(proxy.constraint)
                .and_then(|component| component.concrete.scroll_bar.as_ref())
                .and_then(|bar| bar.scroll_constraint)
            else {
                return;
            };
            if let Some(scroll) = artboard
                .objects
                .component_mut(scroll_constraint)
                .and_then(|component| component.concrete.scroll.as_mut())
            {
                if !scroll.is_scroll_bar_dragging {
                    scroll.intent_x = None;
                    scroll.intent_y = None;
                    scroll.last_frame_offset_x = scroll.offset_x;
                    scroll.last_frame_offset_y = scroll.offset_y;
                }
                scroll.is_scroll_bar_dragging = true;
                if let Some(physics) = scroll.physics.as_mut() {
                    physics.accumulate((0.0, 0.0), timestamp);
                }
            }
        }
        RuntimeDraggableProxyKind::Track => {
            scroll_bar_hit_track(artboard, proxy.constraint, position);
        }
    }
}

pub(crate) fn runtime_draggable_proxy_drag(
    artboard: &mut ArtboardInstance,
    proxy: &mut RuntimeDraggableProxy,
    position: (f32, f32),
    timestamp: f32,
) -> bool {
    let delta = (
        position.0 - proxy.last_position.0,
        position.1 - proxy.last_position.1,
    );
    match proxy.kind {
        RuntimeDraggableProxyKind::Viewport => {
            let local = artboard.component_at(proxy.constraint).local_id;
            if !constraint_bool(artboard, local, "ScrollConstraint", "interactive", true) {
                return false;
            }
            if !proxy.viewport_is_dragging {
                let direction =
                    constraint_uint(artboard, local, "DraggableConstraint", "directionValue", 1);
                let threshold =
                    constraint_double(artboard, local, "ScrollConstraint", "threshold", 0.0);
                let crossed = match direction {
                    0 => delta.0.abs() > threshold,
                    1 => delta.1.abs() > threshold,
                    2 => delta.0.hypot(delta.1) > threshold,
                    _ => false,
                };
                if !crossed {
                    return false;
                }
                proxy.viewport_is_dragging = true;
            }
            scroll_constraint_drag_view(artboard, proxy.constraint, delta, timestamp);
            proxy.last_position = position;
            true
        }
        RuntimeDraggableProxyKind::Thumb => {
            scroll_bar_drag_thumb(artboard, proxy.constraint, delta, timestamp);
            proxy.last_position = position;
            true
        }
        RuntimeDraggableProxyKind::Track => true,
    }
}

pub(crate) fn runtime_draggable_proxy_end(
    artboard: &mut ArtboardInstance,
    proxy: &mut RuntimeDraggableProxy,
) {
    match proxy.kind {
        RuntimeDraggableProxyKind::Viewport => {
            let local = artboard.component_at(proxy.constraint).local_id;
            if !constraint_bool(artboard, local, "ScrollConstraint", "interactive", true) {
                return;
            }
            scroll_constraint_run_physics(artboard, proxy.constraint);
        }
        RuntimeDraggableProxyKind::Thumb => {
            let Some(scroll_constraint) = artboard
                .objects
                .component(proxy.constraint)
                .and_then(|component| component.concrete.scroll_bar.as_ref())
                .and_then(|bar| bar.scroll_constraint)
            else {
                return;
            };
            if let Some(scroll) = artboard
                .objects
                .component_mut(scroll_constraint)
                .and_then(|component| component.concrete.scroll.as_mut())
            {
                scroll.is_scroll_bar_dragging = false;
                if let Some(physics) = scroll.physics.as_mut() {
                    physics.clear_velocity();
                }
            }
        }
        RuntimeDraggableProxyKind::Track => {}
    }
}

fn set_scroll_offset(
    artboard: &mut ArtboardInstance,
    constraint: ComponentHandle,
    axis: RuntimeScrollAxis,
    value: f32,
) {
    let local = artboard.component_at(constraint).local_id;
    let property = match axis {
        RuntimeScrollAxis::X => "scrollOffsetX",
        RuntimeScrollAxis::Y => "scrollOffsetY",
    };
    if let Some(key) = property_key_for_name("ScrollConstraint", property) {
        let _ = artboard.set_double_property(local, key, value);
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeTextInputScrollViewport {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) offset_x: f32,
    pub(crate) offset_y: f32,
    pub(crate) constrains_horizontal: bool,
    pub(crate) constrains_vertical: bool,
}

/// Read-only bridge for `TextInput::worldToLocalWithViewport`.
pub(crate) fn text_input_scroll_viewport(
    artboard: &ArtboardInstance,
    constraint: ComponentHandle,
) -> Option<RuntimeTextInputScrollViewport> {
    let scroll = artboard
        .objects
        .component(constraint)?
        .concrete
        .scroll
        .as_ref()?;
    let metrics = runtime_scroll_layout_metrics(artboard, constraint, scroll, false)
        .unwrap_or_else(|| {
            build_runtime_scroll_layout_metrics(artboard, constraint, scroll, None, false)
        });
    Some(RuntimeTextInputScrollViewport {
        width: metrics.viewport_width,
        height: metrics.viewport_height,
        offset_x: scroll.offset_x,
        offset_y: scroll.offset_y,
        constrains_horizontal: metrics.constrains_horizontal(),
        constrains_vertical: metrics.constrains_vertical(),
    })
}

/// Pinned `TextInput::updateMultiline` clears the scroll axis that no longer
/// participates when switching between single-line and multiline modes.
pub(crate) fn reset_text_input_cross_axis_scroll(
    artboard: &mut ArtboardInstance,
    constraint: ComponentHandle,
    multiline: bool,
) {
    let axis = if multiline {
        RuntimeScrollAxis::X
    } else {
        RuntimeScrollAxis::Y
    };
    let offset = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.concrete.scroll.as_ref())
        .map(|scroll| match axis {
            RuntimeScrollAxis::X => scroll.offset_x,
            RuntimeScrollAxis::Y => scroll.offset_y,
        })
        .unwrap_or(0.0);
    if offset == 0.0 {
        return;
    }
    if let Some(physics) = artboard
        .objects
        .component_mut(constraint)
        .and_then(|component| component.concrete.scroll.as_mut())
        .and_then(|scroll| scroll.physics.as_mut())
    {
        physics.stop();
    }
    set_scroll_offset(artboard, constraint, axis, 0.0);
}

pub(crate) fn scroll_text_input_caret_into_view(
    artboard: &mut ArtboardInstance,
    constraint: ComponentHandle,
    multiline: bool,
    cursor_x: f32,
    cursor_top: f32,
    cursor_bottom: f32,
) -> bool {
    let Some(viewport) = text_input_scroll_viewport(artboard, constraint) else {
        return false;
    };
    let next = if !multiline && viewport.constrains_horizontal {
        let viewport_x = cursor_x + viewport.offset_x;
        if viewport_x < 0.0 {
            Some((RuntimeScrollAxis::X, viewport.offset_x - viewport_x))
        } else if viewport_x > viewport.width - 1.0 {
            Some((
                RuntimeScrollAxis::X,
                viewport.offset_x - (viewport_x - viewport.width + 1.0),
            ))
        } else {
            None
        }
    } else if multiline && viewport.constrains_vertical {
        let viewport_top = cursor_top + viewport.offset_y;
        let viewport_bottom = cursor_bottom + viewport.offset_y;
        if viewport_top < 0.0 {
            Some((RuntimeScrollAxis::Y, viewport.offset_y - viewport_top))
        } else if viewport_bottom > viewport.height {
            Some((
                RuntimeScrollAxis::Y,
                viewport.offset_y - (viewport_bottom - viewport.height),
            ))
        } else {
            None
        }
    } else {
        None
    };
    let Some((axis, value)) = next else {
        return false;
    };
    if let Some(physics) = artboard
        .objects
        .component_mut(constraint)
        .and_then(|component| component.concrete.scroll.as_mut())
        .and_then(|scroll| scroll.physics.as_mut())
    {
        physics.stop();
    }
    set_scroll_offset(artboard, constraint, axis, value);
    true
}

pub(crate) fn advance_text_input_scroll(
    artboard: &mut ArtboardInstance,
    constraint: ComponentHandle,
    scroll_x: f32,
    scroll_y: f32,
    elapsed_seconds: f32,
) -> bool {
    let Some((metrics, previous)) = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.concrete.scroll.as_ref())
        .and_then(|scroll| {
            Some((
                runtime_scroll_layout_metrics(artboard, constraint, scroll, false)?,
                (scroll.offset_x, scroll.offset_y),
            ))
        })
    else {
        return false;
    };
    if let Some(physics) = artboard
        .objects
        .component_mut(constraint)
        .and_then(|component| component.concrete.scroll.as_mut())
        .and_then(|scroll| scroll.physics.as_mut())
    {
        physics.stop();
    }

    let mut changed = false;
    if scroll_x != 0.0 {
        let value = previous.0 + scroll_x * elapsed_seconds;
        let value = if metrics.infinite {
            value
        } else {
            value.clamp(metrics.max_offset(RuntimeScrollAxis::X), 0.0)
        };
        if value != previous.0 {
            set_scroll_offset(artboard, constraint, RuntimeScrollAxis::X, value);
            changed = true;
        }
    }
    if scroll_y != 0.0 {
        let value = previous.1 + scroll_y * elapsed_seconds;
        let value = if metrics.infinite {
            value
        } else {
            value.clamp(metrics.max_offset(RuntimeScrollAxis::Y), 0.0)
        };
        if value != previous.1 {
            set_scroll_offset(artboard, constraint, RuntimeScrollAxis::Y, value);
            changed = true;
        }
    }
    changed
}

fn scroll_constraint_drag_view(
    artboard: &mut ArtboardInstance,
    constraint: ComponentHandle,
    delta: (f32, f32),
    timestamp: f32,
) {
    let local = artboard.component_at(constraint).local_id;
    let multiplier = constraint_double(artboard, local, "ScrollConstraint", "dragMultiplier", 1.0);
    let scaled = (delta.0 * multiplier, delta.1 * multiplier);
    let Some((offset_x, offset_y)) = artboard
        .objects
        .component_mut(constraint)
        .and_then(|component| component.concrete.scroll.as_mut())
        .map(|scroll| {
            if let Some(physics) = scroll.physics.as_mut() {
                physics.accumulate(scaled, timestamp);
            }
            (scroll.offset_x + scaled.0, scroll.offset_y + scaled.1)
        })
    else {
        return;
    };
    set_scroll_offset(artboard, constraint, RuntimeScrollAxis::X, offset_x);
    set_scroll_offset(artboard, constraint, RuntimeScrollAxis::Y, offset_y);
}

fn scroll_constraint_run_physics(artboard: &mut ArtboardInstance, constraint: ComponentHandle) {
    let local = artboard.component_at(constraint).local_id;
    let Some(scroll) = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.concrete.scroll.as_ref())
    else {
        return;
    };
    let snap = constraint_bool(artboard, local, "ScrollConstraint", "snap", false);
    let metrics =
        runtime_scroll_layout_metrics(artboard, constraint, scroll, snap).unwrap_or_else(|| {
            build_runtime_scroll_layout_metrics(artboard, constraint, scroll, None, snap)
        });
    let snapping_points = if snap {
        metrics
            .item_bounds
            .iter()
            .filter(|bounds| !metrics.bounds_collapsed(**bounds))
            .map(|bounds| (bounds.x, bounds.y))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let range_min = (
        metrics.max_offset(RuntimeScrollAxis::X),
        metrics.max_offset(RuntimeScrollAxis::Y),
    );
    let content_size = if metrics.main_axis_horizontal {
        metrics.content_width
    } else {
        metrics.content_height
    };
    let viewport_size = if metrics.main_axis_horizontal {
        metrics.viewport_width
    } else {
        metrics.viewport_height
    };
    if let Some(scroll) = artboard
        .objects
        .component_mut(constraint)
        .and_then(|component| component.concrete.scroll.as_mut())
    {
        scroll.is_dragging = false;
        if let Some(physics) = scroll.physics.as_mut() {
            physics.run(
                range_min,
                (0.0, 0.0),
                (scroll.offset_x, scroll.offset_y),
                &snapping_points,
                content_size,
                viewport_size,
            );
        }
    }
}

fn computed_scroll_bar_thumb_size(
    artboard: &ArtboardInstance,
    constraint: ComponentHandle,
    scroll_constraint: ComponentHandle,
    thumb: ComponentHandle,
    track: ComponentHandle,
) -> (f32, f32) {
    let (_, _, authored_width, authored_height) = constraint_bounds(artboard, thumb);
    let constraint_local = artboard.component_at(constraint).local_id;
    if !constraint_bool(
        artboard,
        constraint_local,
        "ScrollBarConstraint",
        "autoSize",
        true,
    ) {
        return (authored_width, authored_height);
    }
    let Some(scroll) = artboard
        .objects
        .component(scroll_constraint)
        .and_then(|component| component.concrete.scroll.as_ref())
    else {
        return (authored_width, authored_height);
    };
    let metrics = runtime_scroll_layout_metrics(artboard, scroll_constraint, scroll, false)
        .unwrap_or_else(|| {
            build_runtime_scroll_layout_metrics(artboard, scroll_constraint, scroll, None, false)
        });
    let (_, _, track_width, track_height) = constraint_bounds(artboard, track);
    let track_style = layout_component_style_local(artboard, artboard.component_at(track).local_id);
    let inner_width = track_width
        - layout_style_axis_leading_padding(artboard, track_style, true)
        - layout_style_axis_trailing_padding(artboard, track_style, true);
    let inner_height = track_height
        - layout_style_axis_leading_padding(artboard, track_style, false)
        - layout_style_axis_trailing_padding(artboard, track_style, false);
    let visible_width_ratio = if metrics.content_width == 0.0 {
        1.0
    } else {
        (metrics.viewport_width / metrics.content_width).min(1.0)
    };
    let visible_height_ratio = if metrics.content_height == 0.0 {
        1.0
    } else {
        (metrics.viewport_height / metrics.content_height).min(1.0)
    };
    (
        inner_width * visible_width_ratio,
        inner_height * visible_height_ratio,
    )
}

fn scroll_bar_hit_track(
    artboard: &mut ArtboardInstance,
    constraint: ComponentHandle,
    world_position: (f32, f32),
) {
    let Some((scroll_constraint, thumb, track)) = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.concrete.scroll_bar.as_ref())
        .and_then(|bar| {
            let thumb = artboard.objects.component(constraint)?.parent?;
            let track = artboard.objects.component(thumb)?.parent?;
            Some((bar.scroll_constraint?, thumb, track))
        })
    else {
        return;
    };
    let local = artboard.component_at(constraint).local_id;
    let direction = constraint_uint(artboard, local, "DraggableConstraint", "directionValue", 1);
    let Some(scroll) = artboard
        .objects
        .component(scroll_constraint)
        .and_then(|component| component.concrete.scroll.as_ref())
    else {
        return;
    };
    let metrics = runtime_scroll_layout_metrics(artboard, scroll_constraint, scroll, false)
        .unwrap_or_else(|| {
            build_runtime_scroll_layout_metrics(artboard, scroll_constraint, scroll, None, false)
        });
    let track_world = artboard.component_at(track).transform.world_transform;
    if track_world.determinant() == 0.0 {
        return;
    }
    let inverse = track_world.invert_or_identity();
    let mut local_position = inverse.transform_point(world_position.0, world_position.1);
    let track_local = artboard.component_at(track).local_id;
    let style = layout_component_style_local(artboard, track_local);
    local_position.0 -= layout_style_axis_leading_padding(artboard, style, true);
    local_position.1 -= layout_style_axis_leading_padding(artboard, style, false);
    let (_, _, track_width, track_height) = constraint_bounds(artboard, track);
    let (thumb_width, thumb_height) =
        computed_scroll_bar_thumb_size(artboard, constraint, scroll_constraint, thumb, track);
    if matches!(direction, 0 | 2) {
        let track_range = track_width
            - layout_style_axis_leading_padding(artboard, style, true)
            - layout_style_axis_trailing_padding(artboard, style, true)
            - thumb_width;
        let max_offset = metrics.max_offset(RuntimeScrollAxis::X);
        set_scroll_offset(
            artboard,
            scroll_constraint,
            RuntimeScrollAxis::X,
            (local_position.0 / track_range * max_offset).clamp(max_offset, 0.0),
        );
    }
    if matches!(direction, 1 | 2) {
        let track_range = track_height
            - layout_style_axis_leading_padding(artboard, style, false)
            - layout_style_axis_trailing_padding(artboard, style, false)
            - thumb_height;
        let max_offset = metrics.max_offset(RuntimeScrollAxis::Y);
        set_scroll_offset(
            artboard,
            scroll_constraint,
            RuntimeScrollAxis::Y,
            (local_position.1 / track_range * max_offset).clamp(max_offset, 0.0),
        );
    }
}

fn scroll_bar_drag_thumb(
    artboard: &mut ArtboardInstance,
    constraint: ComponentHandle,
    delta: (f32, f32),
    timestamp: f32,
) {
    let Some((scroll_constraint, thumb, track)) = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.concrete.scroll_bar.as_ref())
        .and_then(|bar| {
            let thumb = artboard.objects.component(constraint)?.parent?;
            let track = artboard.objects.component(thumb)?.parent?;
            Some((bar.scroll_constraint?, thumb, track))
        })
    else {
        return;
    };
    let local = artboard.component_at(constraint).local_id;
    let direction = constraint_uint(artboard, local, "DraggableConstraint", "directionValue", 1);
    let Some(scroll) = artboard
        .objects
        .component(scroll_constraint)
        .and_then(|component| component.concrete.scroll.as_ref())
    else {
        return;
    };
    let previous = (scroll.offset_x, scroll.offset_y);
    let metrics = runtime_scroll_layout_metrics(artboard, scroll_constraint, scroll, false)
        .unwrap_or_else(|| {
            build_runtime_scroll_layout_metrics(artboard, scroll_constraint, scroll, None, false)
        });
    let (_, _, track_width, track_height) = constraint_bounds(artboard, track);
    let (thumb_width, thumb_height) =
        computed_scroll_bar_thumb_size(artboard, constraint, scroll_constraint, thumb, track);
    if constraint_bool(artboard, local, "ScrollBarConstraint", "autoSize", true)
        && let Some(layout) = artboard
            .objects
            .component(thumb)
            .and_then(|component| component.concrete.layout.as_ref())
    {
        if matches!(direction, 0 | 2) {
            layout.forced_width(thumb_width);
        }
        if matches!(direction, 1 | 2) {
            layout.forced_height(thumb_height);
        }
    }
    let style = layout_component_style_local(artboard, artboard.component_at(track).local_id);
    if matches!(direction, 0 | 2) {
        let track_range = track_width
            - layout_style_axis_leading_padding(artboard, style, true)
            - layout_style_axis_trailing_padding(artboard, style, true)
            - thumb_width;
        let max_offset = metrics.max_offset(RuntimeScrollAxis::X);
        let thumb_offset = previous.0 / max_offset * track_range + delta.0;
        set_scroll_offset(
            artboard,
            scroll_constraint,
            RuntimeScrollAxis::X,
            (thumb_offset / track_range * max_offset).clamp(max_offset, 0.0),
        );
    }
    if matches!(direction, 1 | 2) {
        let track_range = track_height
            - layout_style_axis_leading_padding(artboard, style, false)
            - layout_style_axis_trailing_padding(artboard, style, false)
            - thumb_height;
        let max_offset = metrics.max_offset(RuntimeScrollAxis::Y);
        let thumb_offset = previous.1 / max_offset * track_range + delta.1;
        set_scroll_offset(
            artboard,
            scroll_constraint,
            RuntimeScrollAxis::Y,
            (thumb_offset / track_range * max_offset).clamp(max_offset, 0.0),
        );
    }
    if let Some(scroll) = artboard
        .objects
        .component_mut(scroll_constraint)
        .and_then(|component| component.concrete.scroll.as_mut())
        && let Some(physics) = scroll.physics.as_mut()
    {
        physics.accumulate(
            (scroll.offset_x - previous.0, scroll.offset_y - previous.1),
            timestamp,
        );
    }
}

fn runtime_scroll_constraint(
    artboard: &ArtboardInstance,
    local_id: usize,
) -> Option<(ComponentHandle, &RuntimeScrollConstraintState)> {
    let handle = artboard.component_handle(local_id)?;
    let state = artboard
        .objects
        .component(handle)?
        .concrete
        .scroll
        .as_ref()?;
    Some((handle, state))
}

pub(crate) fn runtime_scroll_double_property(
    artboard: &ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<f32> {
    let property = runtime_scroll_property(property_key)?;
    let (constraint_handle, constraint) = runtime_scroll_constraint(artboard, local_id)?;
    match property {
        RuntimeScrollProperty::Offset(_) => None,
        RuntimeScrollProperty::Percent(axis) => {
            if let Some(value) = constraint
                .intent(axis)
                .and_then(|intent| intent.read(RuntimeScrollSpace::Percent))
            {
                return Some(value);
            }
            let metrics =
                runtime_scroll_layout_metrics(artboard, constraint_handle, constraint, false);
            let max_offset = metrics
                .as_ref()
                .map(|metrics| metrics.max_offset(axis))
                .unwrap_or(0.0);
            if max_offset == 0.0 {
                return Some(0.0);
            }
            let offset = raw_scroll_offset(artboard, local_id, axis);
            Some(
                offset
                    / metrics
                        .as_ref()
                        .map(|metrics| metrics.max_offset_for_percent(axis))
                        .unwrap_or(1.0),
            )
        }
        RuntimeScrollProperty::Index => {
            let direction = constraint_uint(
                artboard,
                local_id,
                "DraggableConstraint",
                "directionValue",
                1,
            );
            let axis = if matches!(direction, 0 | 2) {
                Some(RuntimeScrollAxis::X)
            } else if direction == 1 {
                Some(RuntimeScrollAxis::Y)
            } else {
                None
            };
            if let Some(value) = axis
                .and_then(|axis| constraint.intent(axis))
                .and_then(|intent| intent.read(RuntimeScrollSpace::Index))
            {
                return Some(value);
            }
            Some(
                runtime_scroll_layout_metrics(artboard, constraint_handle, constraint, true)
                    .map(|metrics| {
                        metrics.index_at_position((
                            raw_scroll_offset(artboard, local_id, RuntimeScrollAxis::X),
                            raw_scroll_offset(artboard, local_id, RuntimeScrollAxis::Y),
                        ))
                    })
                    .unwrap_or(0.0),
            )
        }
    }
}

pub(crate) fn set_runtime_scroll_double_property(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
    value: f32,
) -> Option<bool> {
    let property = runtime_scroll_property(property_key)?;
    if matches!(property, RuntimeScrollProperty::Offset(_)) {
        return None;
    }
    let (constraint_handle, constraint) = runtime_scroll_constraint(artboard, local_id)?;
    if runtime_scroll_double_property(artboard, local_id, property_key) == Some(value) {
        return Some(false);
    }
    let layout_initialized = constraint.layout_initialized;
    if constraint.is_dragging {
        // The generated computed-property wrapper still publishes its
        // changed notification after the concrete setter returns, but the
        // retained ScrollConstraint owner itself does not mutate while a drag
        // is active (`scroll_constraint.cpp:497-532`).
        return Some(true);
    }
    if let Some(physics) = artboard
        .objects
        .component_mut(constraint_handle)?
        .concrete
        .scroll
        .as_mut()?
        .physics
        .as_mut()
    {
        physics.reset();
    }

    let constraint = artboard
        .objects
        .component(constraint_handle)?
        .concrete
        .scroll
        .as_ref()?;
    let metrics = if layout_initialized {
        runtime_scroll_layout_metrics(
            artboard,
            constraint_handle,
            constraint,
            matches!(property, RuntimeScrollProperty::Index),
        )
    } else {
        Some(build_runtime_scroll_layout_metrics(
            artboard,
            constraint_handle,
            constraint,
            None,
            matches!(property, RuntimeScrollProperty::Index),
        ))
    };
    let direction = constraint_uint(
        artboard,
        local_id,
        "DraggableConstraint",
        "directionValue",
        1,
    );
    match property {
        RuntimeScrollProperty::Percent(axis) => apply_scroll_axis_intent(
            artboard,
            local_id,
            constraint_handle,
            axis,
            RuntimeScrollAxisIntent {
                space: RuntimeScrollSpace::Percent,
                value,
            },
            metrics.as_ref(),
        )?,
        RuntimeScrollProperty::Index => {
            if matches!(direction, 0 | 2) {
                apply_scroll_axis_intent(
                    artboard,
                    local_id,
                    constraint_handle,
                    RuntimeScrollAxis::X,
                    RuntimeScrollAxisIntent {
                        space: RuntimeScrollSpace::Index,
                        value,
                    },
                    metrics.as_ref(),
                )?;
            }
            if matches!(direction, 1 | 2) {
                apply_scroll_axis_intent(
                    artboard,
                    local_id,
                    constraint_handle,
                    RuntimeScrollAxis::Y,
                    RuntimeScrollAxisIntent {
                        space: RuntimeScrollSpace::Index,
                        value,
                    },
                    metrics.as_ref(),
                )?;
            }
        }
        RuntimeScrollProperty::Offset(_) => unreachable!("offsets use generated storage"),
    }
    Some(true)
}

fn apply_scroll_axis_intent(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    constraint_handle: ComponentHandle,
    axis: RuntimeScrollAxis,
    intent: RuntimeScrollAxisIntent,
    metrics: Option<&RuntimeScrollLayoutMetrics>,
) -> Option<()> {
    let resolved = intent.resolve(axis, metrics);
    artboard
        .objects
        .component_mut(constraint_handle)?
        .concrete
        .scroll
        .as_mut()?
        .set_intent(axis, resolved.is_none().then_some(intent));
    if let Some(offset) = resolved
        && let Some(offset_key) = scroll_offset_property_key(axis)
    {
        artboard.set_double_property(local_id, offset_key, offset);
    }
    Some(())
}

pub(crate) fn apply_scroll_offset_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
    value: f32,
) -> Option<bool> {
    let axis = match runtime_scroll_property(property_key)? {
        RuntimeScrollProperty::Offset(axis) => axis,
        _ => return None,
    };
    let handle = artboard.component_handle(local_id)?;
    let content = artboard
        .objects
        .component(handle)?
        .concrete
        .scroll
        .as_ref()?
        .content?;
    {
        let scroll = artboard
            .objects
            .component_mut(handle)?
            .concrete
            .scroll
            .as_mut()?;
        let retained = match axis {
            RuntimeScrollAxis::X => &mut scroll.offset_x,
            RuntimeScrollAxis::Y => &mut scroll.offset_y,
        };
        if *retained == value {
            return Some(false);
        }
        *retained = value;
    }
    let content_local = artboard.objects.component_local_id(content)?;
    Some(artboard.add_dirt(
        content_local,
        crate::components::ComponentDirt::WORLD_TRANSFORM,
        true,
    ))
}

fn raw_scroll_offset(artboard: &ArtboardInstance, local_id: usize, axis: RuntimeScrollAxis) -> f32 {
    runtime_scroll_constraint(artboard, local_id)
        .map(|(_, scroll)| match axis {
            RuntimeScrollAxis::X => scroll.offset_x,
            RuntimeScrollAxis::Y => scroll.offset_y,
        })
        .unwrap_or(0.0)
}

fn scroll_offset_property_key(axis: RuntimeScrollAxis) -> Option<u16> {
    let keys = runtime_scroll_property_keys();
    keys[match axis {
        RuntimeScrollAxis::X => 0,
        RuntimeScrollAxis::Y => 1,
    }]
}

fn resolve_runtime_scroll_intents(
    artboard: &mut ArtboardInstance,
    constraint_local: usize,
    metrics: &RuntimeScrollLayoutMetrics,
) -> bool {
    let Some(constraint_handle) = artboard.component_handle(constraint_local) else {
        return false;
    };
    let Some(scroll) = artboard
        .objects
        .component(constraint_handle)
        .and_then(|component| component.concrete.scroll.as_ref())
    else {
        return false;
    };
    let intents = [
        (RuntimeScrollAxis::X, scroll.intent_x),
        (RuntimeScrollAxis::Y, scroll.intent_y),
    ];
    let mut changed = false;
    for (axis, intent) in intents {
        let Some(intent) = intent else {
            continue;
        };
        let Some(offset) = intent.resolve(axis, Some(metrics)) else {
            continue;
        };
        artboard
            .objects
            .component_mut(constraint_handle)
            .and_then(|component| component.concrete.scroll.as_mut())
            .expect("ScrollConstraint occurrence retains its concrete state")
            .set_intent(axis, None);
        if let Some(offset_key) = scroll_offset_property_key(axis) {
            changed |= artboard.set_double_property(constraint_local, offset_key, offset);
        }
    }
    changed
}

/// Runtime constraint application for the C++ `src/constraints/` path.
pub(crate) fn apply_constraints(
    artboard: &mut ArtboardInstance,
    component_index: ComponentHandle,
) -> bool {
    let mut changed = false;
    let constraint_count = artboard.objects.constraint_len(component_index);
    for index in 0..constraint_count {
        let Some(constraint) = artboard.objects.constraint_at(component_index, index) else {
            continue;
        };
        if artboard
            .objects
            .component(component_index)
            .is_some_and(|component| component.concrete.constrainable_list.is_some())
            && artboard
                .objects
                .component(constraint)
                .and_then(|component| component.concrete.constraint)
                .is_some_and(|state| state.kind == RuntimeConstraintKind::ListFollowPath)
        {
            continue;
        }
        changed |= apply_constraint(artboard, component_index, constraint);
    }
    changed
}

pub(crate) fn apply_list_constraints(
    artboard: &mut ArtboardInstance,
    component_index: ComponentHandle,
) -> bool {
    if artboard
        .objects
        .component(component_index)
        .is_none_or(|component| component.concrete.constrainable_list.is_none())
    {
        return false;
    }

    let list_local = artboard.component_at(component_index).local_id;
    let Some(mut item_transforms) = artboard
        .component_list_state_mut(list_local)
        .map(|list| std::mem::take(&mut list.item_transforms))
    else {
        return false;
    };
    let changed = constrain_component_list_item_transforms(
        artboard,
        list_local,
        component_index,
        &mut item_transforms,
    );
    if let Some(list) = artboard.component_list_state_mut(list_local) {
        list.item_transforms = item_transforms;
    }
    changed
}

pub(crate) fn apply_parent_layout_constraints(
    artboard: &mut ArtboardInstance,
    component: ComponentHandle,
) -> bool {
    let count = artboard
        .objects
        .component(component)
        .map_or(0, |component| {
            if component.concrete.constrainable_list.is_some() {
                component
                    .concrete
                    .constrainable_list
                    .as_ref()
                    .map_or(0, |list| list.layout_constraints.len())
            } else {
                component
                    .concrete
                    .layout
                    .as_ref()
                    .map_or(0, |layout| layout.layout_constraints.len())
            }
        });
    let mut changed = false;
    for index in 0..count {
        let constraint = artboard.objects.component(component).and_then(|component| {
            if component.concrete.constrainable_list.is_some() {
                component
                    .concrete
                    .constrainable_list
                    .as_ref()
                    .and_then(|list| list.layout_constraints.get(index))
                    .copied()
            } else {
                component
                    .concrete
                    .layout
                    .as_ref()
                    .and_then(|layout| layout.layout_constraints.get(index))
                    .copied()
            }
        });
        if let Some(constraint) = constraint {
            changed |= apply_scroll_constraint_child(artboard, component, constraint);
        }
    }
    changed
}

fn apply_scroll_constraint_child(
    artboard: &mut ArtboardInstance,
    child: ComponentHandle,
    constraint: ComponentHandle,
) -> bool {
    if !artboard.component_at(child).capabilities.transform {
        // `LayoutNodeProvider::transformComponent()` returned null: C++
        // returns without incrementing the rendezvous count
        // (`scroll_constraint.cpp:203-209`).
        return false;
    }
    let Some(scroll_transform) = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.concrete.scroll.as_ref())
        .map(|scroll| scroll.scroll_transform)
    else {
        return false;
    };
    let constraint_local = artboard.component_at(constraint).local_id;
    let strength = constraint_double(artboard, constraint_local, "Constraint", "strength", 1.0);
    let current = artboard.component_at(child).transform.world_transform;
    let target = current.multiply(scroll_transform);
    let components_a = current.decompose();
    let mut components_b = target.decompose();
    let inverse_strength = 1.0 - strength;
    components_b.rotation = interpolated_rotation_from_modded_base(
        components_a.rotation,
        components_b.rotation,
        strength,
    );
    components_b.x = components_a.x * inverse_strength + components_b.x * strength;
    components_b.y = components_a.y * inverse_strength + components_b.y * strength;
    components_b.scale_x =
        components_a.scale_x * inverse_strength + components_b.scale_x * strength;
    components_b.scale_y =
        components_a.scale_y * inverse_strength + components_b.scale_y * strength;
    components_b.skew = components_a.skew * inverse_strength + components_b.skew * strength;
    let changed = write_world_transform(artboard, child, Mat2D::compose(components_b));

    // C++ applies the transform before incrementing and only then tests the
    // virtualizer rendezvous (`scroll_constraint.cpp:203-237`).
    let scroll = artboard
        .objects
        .component_mut(constraint)
        .and_then(|component| component.concrete.scroll.as_mut())
        .expect("registered ScrollConstraint remains live");
    scroll.components_a = components_a;
    scroll.components_b = components_b;
    scroll.child_constraint_applied_count += 1;
    // C++ calls `constrainVirtualized()` after every successful child. That
    // owner performs the live `virtualize()` and rendezvous gates itself
    // (`scroll_constraint.cpp:203-237`).
    constrain_scroll_virtualizer(artboard, constraint, false);
    changed
}

pub(crate) fn constrain_scroll_virtualizer(
    artboard: &mut ArtboardInstance,
    constraint: ComponentHandle,
    force: bool,
) -> bool {
    let Some((applied, child_count, has_virtualizer)) = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.concrete.scroll.as_ref())
        .map(|scroll| {
            (
                scroll.child_constraint_applied_count,
                scroll.layout_children.len(),
                scroll.virtualizer.is_some(),
            )
        })
    else {
        return false;
    };
    let constraint_local = artboard.component_at(constraint).local_id;
    let virtualize = constraint_bool(
        artboard,
        constraint_local,
        "ScrollConstraint",
        "virtualize",
        false,
    );
    if !virtualize || !has_virtualizer || (!force && applied < child_count) {
        return false;
    }
    let computed_layout_bounds = artboard
        .runtime_graph()
        .and_then(|graph| artboard.runtime_taffy_layout_bounds(graph, artboard.runtime_file()));
    let retained_layout_bounds = artboard.layout_constraint_bounds.clone();
    let layout_bounds = retained_layout_bounds
        .as_deref()
        .or(computed_layout_bounds.as_ref());
    let metrics = {
        let scroll = artboard
            .objects
            .component(constraint)
            .and_then(|component| component.concrete.scroll.as_ref())
            .expect("ScrollConstraint remains live");
        build_runtime_scroll_layout_metrics(artboard, constraint, scroll, layout_bounds, false)
    };
    let direction = if metrics.main_axis_horizontal {
        RuntimeScrollAxis::X
    } else {
        RuntimeScrollAxis::Y
    };
    let (clamped_x, clamped_y) = clamped_scroll_constraint_offsets(artboard, constraint, &metrics);
    let offset = match direction {
        RuntimeScrollAxis::X => clamped_x,
        RuntimeScrollAxis::Y => clamped_y,
    };
    let viewport_size = metrics.viewport_size(direction);
    let infinite = metrics.infinite;
    let content_size = match direction {
        RuntimeScrollAxis::X => metrics.content_width,
        RuntimeScrollAxis::Y => metrics.content_height,
    };
    // Pinned `ScrollVirtualizer::constrain` returns true but leaves every
    // retained field untouched when content size is non-positive.
    if content_size <= 0.0 {
        return true;
    }
    let provider_item_sizes = {
        let scroll = artboard
            .objects
            .component(constraint)
            .and_then(|component| component.concrete.scroll.as_ref())
            .expect("ScrollConstraint remains live");
        virtualized_provider_item_sizes(artboard, layout_bounds, scroll, None)
    };
    let gap = match direction {
        RuntimeScrollAxis::X => metrics.gap_x,
        RuntimeScrollAxis::Y => metrics.gap_y,
    };
    let range = exact_scroll_virtualizer_range(
        &provider_item_sizes,
        direction == RuntimeScrollAxis::X,
        gap,
        viewport_size,
        offset,
        infinite,
        content_size,
    );
    let (last_visible_start, last_visible_end) = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.concrete.scroll.as_ref())
        .and_then(|scroll| scroll.virtualizer.as_ref())
        .map(|virtualizer| (virtualizer.visible_start, virtualizer.visible_end))
        .unwrap_or((0, 0));
    {
        let virtualizer = artboard
            .objects
            .component_mut(constraint)
            .and_then(|component| component.concrete.scroll.as_mut())
            .and_then(|scroll| scroll.virtualizer.as_mut())
            .expect("virtualized ScrollConstraint owns its virtualizer");
        virtualizer.offset = normalized_scroll_virtualizer_offset(offset, infinite, content_size);
        virtualizer.infinite = infinite;
        virtualizer.viewport_size = viewport_size;
        virtualizer.direction = direction;
        virtualizer.visible_start = range.visible_start;
        virtualizer.visible_end = range.visible_end;
    }

    let providers = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.concrete.scroll.as_ref())
        .map(|scroll| scroll.layout_children.clone())
        .unwrap_or_default();
    let provider_locals = providers
        .iter()
        .map(|provider| artboard.objects.component_local_id(*provider))
        .collect::<Vec<_>>();
    for provider_local in provider_locals.iter().flatten().copied() {
        if artboard.component_list_state(provider_local).is_some() {
            artboard.set_component_list_visible_indices(provider_local, -1, -1);
        }
    }

    let total_item_count = provider_item_sizes.iter().map(Vec::len).sum::<usize>();
    if total_item_count == 0 {
        return true;
    }
    let actual_start = if infinite {
        range.visible_start.rem_euclid(total_item_count as i32)
    } else {
        range.visible_start
    };
    let actual_end = if infinite {
        range.visible_end.rem_euclid(total_item_count as i32)
    } else {
        range.visible_end
    };
    let mut used_indices = BTreeSet::new();
    if actual_start <= actual_end {
        used_indices.extend(actual_start..=actual_end);
    } else {
        used_indices.extend(actual_start..total_item_count as i32);
        used_indices.extend(0..=actual_end);
    }
    let last_start = if infinite {
        last_visible_start.rem_euclid(total_item_count as i32)
    } else {
        last_visible_start
    };
    let last_end = if infinite {
        last_visible_end.rem_euclid(total_item_count as i32)
    } else {
        last_visible_end
    };
    let mut indices_to_recycle = Vec::new();
    let mut consider_previous = |index: i32| {
        if index >= 0 && !used_indices.contains(&index) {
            indices_to_recycle.push(index as usize);
        }
    };
    if last_start <= last_end {
        for index in last_start..=last_end {
            consider_previous(index);
        }
    } else {
        for index in last_start..total_item_count as i32 {
            consider_previous(index);
        }
        for index in 0..=last_end {
            consider_previous(index);
        }
    }
    indices_to_recycle.sort_unstable();

    let locate_item = |actual_index: usize| {
        let mut running_total = 0usize;
        for (provider_index, child) in provider_item_sizes.iter().enumerate() {
            let start = running_total;
            let end = start + child.len();
            if start < end && actual_index >= start && actual_index < end {
                return Some((provider_index, actual_index - start));
            }
            running_total = end;
        }
        None
    };
    for actual_index in indices_to_recycle {
        let Some((provider_index, logical_index)) = locate_item(actual_index) else {
            continue;
        };
        let Some(provider_local) = provider_locals.get(provider_index).copied().flatten() else {
            continue;
        };
        artboard.remove_component_list_virtualizable(provider_local, logical_index);
    }

    let Some(file) = artboard.runtime_file_arc() else {
        return true;
    };
    let mut visible_indices = vec![(-1_i32, -1_i32); providers.len()];
    let mut changed_providers = BTreeSet::new();
    let mut running_offset = range.running_offset;
    for global_index in range.visible_start..=range.visible_end {
        let actual_index = if infinite {
            global_index.rem_euclid(total_item_count as i32) as usize
        } else {
            global_index as usize
        };
        let Some((provider_index, logical_index)) = locate_item(actual_index) else {
            continue;
        };
        let Some(provider_local) = provider_locals.get(provider_index).copied().flatten() else {
            continue;
        };
        if artboard.component_list_state(provider_local).is_none() {
            continue;
        }
        let visible = &mut visible_indices[provider_index];
        if visible.0 == -1 {
            visible.0 = logical_index as i32;
        }
        visible.1 = logical_index as i32;
        if !artboard.virtualizing_component_has_item(provider_local, logical_index)
            && artboard.add_component_list_virtualizable(&file, provider_local, logical_index)
        {
            changed_providers.insert(provider_local);
        }
        if artboard.virtualizing_component_has_item(provider_local, logical_index) {
            let layout_position = artboard
                .component_list_virtualizable_layout_position(provider_local, logical_index);
            // The pinned virtualizer replaces only the main-axis coordinate.
            // The cross axis stays on the mounted Artboard root's transferred
            // Yoga node (`scroll_virtualizer.cpp:269-291`).
            let position = if direction == RuntimeScrollAxis::X {
                (running_offset, layout_position.1)
            } else {
                (layout_position.0, running_offset)
            };
            artboard.set_component_list_virtualizable_position(
                provider_local,
                logical_index,
                position,
            );
        }
        let size = provider_item_sizes[provider_index][logical_index];
        running_offset += if direction == RuntimeScrollAxis::X {
            size.0
        } else {
            size.1
        } + gap;
    }
    for (provider_index, provider_local) in provider_locals.into_iter().enumerate() {
        let Some(provider_local) = provider_local else {
            continue;
        };
        if artboard.component_list_state(provider_local).is_some() {
            let visible = visible_indices[provider_index];
            artboard.set_component_list_visible_indices(provider_local, visible.0, visible.1);
        }
    }
    for provider_local in changed_providers {
        artboard.component_list_virtualizable_changed(provider_local);
    }
    true
}

/// Pinned `ScrollConstraint::advanceComponent`.
///
/// The retained advancing schedule supplies the two flags explicitly. The
/// physics owner is advanced before the NewFrame paused-drag velocity check,
/// and the return keeps the component enrolled while physics or either drag
/// owner remains active (`scroll_constraint.cpp:299-336`).
pub(crate) fn advance_scroll_constraint(
    artboard: &mut ArtboardInstance,
    constraint: ComponentHandle,
    elapsed_seconds: f32,
    advance_nested: bool,
    new_frame: bool,
) -> bool {
    if !advance_nested || artboard.component_at(constraint).is_collapsed() {
        return false;
    }
    let Some((physics_running, offset_x_key, offset_y_key)) = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.concrete.scroll.as_ref())
        .and_then(|scroll| {
            let physics = scroll.physics.as_ref()?;
            Some((
                physics.is_running(),
                property_key_for_name("ScrollConstraint", "scrollOffsetX"),
                property_key_for_name("ScrollConstraint", "scrollOffsetY"),
            ))
        })
    else {
        return false;
    };
    let local_id = artboard.component_at(constraint).local_id;
    if physics_running {
        let offset = artboard
            .objects
            .component_mut(constraint)
            .and_then(|component| component.concrete.scroll.as_mut())
            .and_then(|scroll| scroll.physics.as_mut())
            .expect("running ScrollPhysics remains retained")
            .advance(elapsed_seconds);
        if let Some(key) = offset_x_key {
            artboard.set_double_property(local_id, key, offset.0);
        }
        if let Some(key) = offset_y_key {
            artboard.set_double_property(local_id, key, offset.1);
        }
    }

    let scroll = artboard
        .objects
        .component_mut(constraint)
        .and_then(|component| component.concrete.scroll.as_mut())
        .expect("scheduled ScrollConstraint remains retained");
    if new_frame {
        let moved = scroll.offset_x != scroll.last_frame_offset_x
            || scroll.offset_y != scroll.last_frame_offset_y;
        if (scroll.is_scroll_bar_dragging || scroll.is_dragging) && !moved {
            scroll
                .physics
                .as_mut()
                .expect("scheduled ScrollConstraint retains physics")
                .clear_velocity();
        }
        scroll.last_frame_offset_x = scroll.offset_x;
        scroll.last_frame_offset_y = scroll.offset_y;
    }
    scroll
        .physics
        .as_ref()
        .is_some_and(RuntimeScrollPhysicsState::enabled)
        || scroll.is_scroll_bar_dragging
        || scroll.is_dragging
}

/// Apply list constraints after the hosting layout has assigned each mounted
/// artboard its base transform. This mirrors C++
/// `ArtboardComponentList::updateArtboardsWorldTransform` followed by
/// `ArtboardComponentList::updateConstraints`.
pub(crate) fn constrain_component_list_item_transforms(
    artboard: &ArtboardInstance,
    list_local: usize,
    list_component_index: ComponentHandle,
    item_transforms: &mut [Mat2D],
) -> bool {
    // C++ explicitly skips list constraints while the component list is
    // virtualized. The scroll virtualizer owns row positions in that mode.
    if component_list_virtualization(artboard, list_local).is_some() {
        return false;
    }

    let constraint_count = artboard
        .objects
        .component(list_component_index)
        .and_then(|component| component.concrete.constrainable_list.as_ref())
        .map_or(0, |list| list.constraints.len());
    let mut changed = false;
    for index in 0..constraint_count {
        let Some(constraint) = artboard
            .objects
            .component(list_component_index)
            .and_then(|component| component.concrete.constrainable_list.as_ref())
            .and_then(|list| list.constraints.get(index))
            .copied()
        else {
            continue;
        };
        changed |= apply_list_follow_path_constraint_to_transforms(
            artboard,
            list_component_index,
            constraint,
            item_transforms,
        );
    }
    changed
}

/// Resolve the live C++ `ArtboardComponentList::virtualizationEnabled`
/// relationship for one list. A `ScrollConstraint` can be animated or data
/// bound, so read its current instance properties instead of caching flags at
/// import time.
pub(crate) fn component_list_virtualization(
    artboard: &ArtboardInstance,
    list_local: usize,
) -> Option<()> {
    let list = artboard.component_handle(list_local)?;
    let constraint = artboard
        .objects
        .component(list)?
        .concrete
        .constrainable_list
        .as_ref()?
        .layout_constraints
        .iter()
        .copied()
        .find(|constraint| {
            artboard
                .objects
                .component(*constraint)
                .and_then(|component| component.concrete.scroll.as_ref())
                .is_some()
        })?;
    let constraint_local = artboard.objects.component_local_id(constraint)?;
    if !constraint_bool(
        artboard,
        constraint_local,
        "ScrollConstraint",
        "virtualize",
        false,
    ) {
        return None;
    }
    Some(())
}

fn runtime_scroll_layout_metrics(
    artboard: &ArtboardInstance,
    constraint_handle: ComponentHandle,
    constraint: &RuntimeScrollConstraintState,
    include_item_bounds: bool,
) -> Option<RuntimeScrollLayoutMetrics> {
    if !constraint.layout_initialized {
        return None;
    }
    let computed_layout_bounds = artboard
        .runtime_graph()
        .and_then(|graph| artboard.runtime_taffy_layout_bounds(graph, artboard.runtime_file()));
    let retained_layout_bounds = artboard.layout_constraint_bounds.clone();
    let layout_bounds = retained_layout_bounds
        .as_deref()
        .or(computed_layout_bounds.as_ref());
    Some(build_runtime_scroll_layout_metrics(
        artboard,
        constraint_handle,
        constraint,
        layout_bounds,
        include_item_bounds,
    ))
}

fn build_runtime_scroll_layout_metrics(
    artboard: &ArtboardInstance,
    constraint_handle: ComponentHandle,
    constraint: &RuntimeScrollConstraintState,
    layout_bounds: Option<&std::collections::BTreeMap<usize, RuntimeLayoutBounds>>,
    include_item_bounds: bool,
) -> RuntimeScrollLayoutMetrics {
    let constraint_local = artboard
        .objects
        .component_local_id(constraint_handle)
        .expect("ScrollConstraint handle belongs to this occurrence");
    let content = constraint
        .content
        .expect("ScrollConstraint retains its content parent");
    let content_local = artboard
        .objects
        .component_local_id(content)
        .expect("ScrollConstraint content belongs to this occurrence");
    let direction = constraint_uint(
        artboard,
        constraint_local,
        "DraggableConstraint",
        "directionValue",
        1,
    );
    let infinite = constraint_bool(
        artboard,
        constraint_local,
        "ScrollConstraint",
        "infinite",
        false,
    );
    let virtualize = constraint_bool(
        artboard,
        constraint_local,
        "ScrollConstraint",
        "virtualize",
        false,
    );
    let content_style_local = layout_component_style_local(artboard, content_local);
    let main_axis_is_horizontal = content_style_local
        .and_then(|style_local| {
            property_key_for_name("LayoutComponentStyle", "flexDirectionValue")
                .and_then(|key| artboard.uint_property(style_local, key))
        })
        .map(|value| matches!(value, 2 | 3))
        .unwrap_or(true);
    let gap_x = content_style_local
        .and_then(|style_local| {
            property_key_for_name("LayoutComponentStyle", "gapHorizontal")
                .and_then(|key| artboard.double_property(style_local, key))
        })
        .unwrap_or(0.0);
    let gap_y = content_style_local
        .and_then(|style_local| {
            property_key_for_name("LayoutComponentStyle", "gapVertical")
                .and_then(|key| artboard.double_property(style_local, key))
        })
        .unwrap_or(0.0);
    let provider_item_sizes = if virtualize {
        virtualized_provider_item_sizes(artboard, layout_bounds, constraint, None)
    } else {
        Vec::new()
    };
    let viewport_local = artboard
        .objects
        .component(content)
        .and_then(|component| component.parent)
        .and_then(|viewport| artboard.objects.component_local_id(viewport));
    let viewport_bounds = viewport_local.and_then(|local| layout_bounds?.get(&local).copied());
    let content_bounds = layout_bounds
        .and_then(|bounds| bounds.get(&content_local))
        .copied();
    let viewport_layout_width = viewport_bounds.map(|bounds| bounds.width).unwrap_or(0.0);
    let viewport_layout_height = viewport_bounds.map(|bounds| bounds.height).unwrap_or(0.0);
    let content_origin_x = match (content_bounds, viewport_bounds) {
        (Some(content), Some(viewport)) => content.x - viewport.x,
        _ => 0.0,
    };
    let content_origin_y = match (content_bounds, viewport_bounds) {
        (Some(content), Some(viewport)) => content.y - viewport.y,
        _ => 0.0,
    };
    let viewport_width = if direction == 1 {
        viewport_layout_width
    } else {
        scroll_viewport_axis_size(viewport_layout_width, content_origin_x)
    };
    let viewport_height = if direction == 0 {
        viewport_layout_height
    } else {
        scroll_viewport_axis_size(viewport_layout_height, content_origin_y)
    };
    let content_width = if virtualize && main_axis_is_horizontal {
        virtualized_provider_content_size(&provider_item_sizes, true, gap_x, infinite)
    } else {
        content_bounds.map(|bounds| bounds.width).unwrap_or(0.0)
    };
    let content_height = if virtualize && !main_axis_is_horizontal {
        virtualized_provider_content_size(&provider_item_sizes, false, gap_y, infinite)
    } else {
        content_bounds.map(|bounds| bounds.height).unwrap_or(0.0)
    };
    let trailing_padding_x = viewport_local
        .map(|local| {
            layout_style_axis_trailing_padding(
                artboard,
                layout_component_style_local(artboard, local),
                true,
            )
        })
        .unwrap_or(0.0);
    let trailing_padding_y = viewport_local
        .map(|local| {
            layout_style_axis_trailing_padding(
                artboard,
                layout_component_style_local(artboard, local),
                false,
            )
        })
        .unwrap_or(0.0);
    let item_bounds = if include_item_bounds {
        runtime_scroll_item_bounds(
            artboard,
            constraint,
            layout_bounds,
            virtualize,
            main_axis_is_horizontal,
            gap_x,
            gap_y,
            content_bounds,
        )
    } else {
        Vec::new()
    };

    RuntimeScrollLayoutMetrics {
        direction,
        infinite,
        main_axis_horizontal: main_axis_is_horizontal,
        viewport_layout_width,
        viewport_layout_height,
        viewport_width,
        viewport_height,
        content_width,
        content_height,
        trailing_padding_x,
        trailing_padding_y,
        gap_x,
        gap_y,
        item_bounds,
    }
}

#[allow(clippy::too_many_arguments)]
fn runtime_scroll_item_bounds(
    artboard: &ArtboardInstance,
    constraint: &RuntimeScrollConstraintState,
    layout_bounds: Option<&std::collections::BTreeMap<usize, RuntimeLayoutBounds>>,
    virtualize: bool,
    main_axis_is_horizontal: bool,
    gap_x: f32,
    gap_y: f32,
    content_bounds: Option<RuntimeLayoutBounds>,
) -> Vec<RuntimeLayoutBounds> {
    let has_component_list = constraint.layout_children.iter().any(|handle| {
        artboard
            .objects
            .component(*handle)
            .is_some_and(|component| component.concrete.constrainable_list.is_some())
    });
    let assigned_list_bounds = if layout_bounds.is_some() && has_component_list && !virtualize {
        artboard.runtime_component_list_assigned_layout_bounds()
    } else {
        Default::default()
    };
    let content_origin = content_bounds
        .map(|bounds| (bounds.x, bounds.y))
        .unwrap_or((0.0, 0.0));
    let mut flat_bounds = Vec::new();
    for provider in &constraint.layout_children {
        let Some(provider_local) = artboard.objects.component_local_id(*provider) else {
            continue;
        };
        let is_component_list = artboard
            .objects
            .component(*provider)
            .is_some_and(|component| component.concrete.constrainable_list.is_some());
        if !is_component_list {
            if let Some(mut bounds) =
                layout_bounds.and_then(|bounds| bounds.get(&provider_local).copied())
            {
                bounds.x -= content_origin.0;
                bounds.y -= content_origin.1;
                flat_bounds.push(bounds);
            }
            continue;
        }

        if !virtualize
            && let Some(bounds) = assigned_list_bounds.get(&provider_local)
            && !bounds.is_empty()
        {
            flat_bounds.extend(bounds.iter().copied());
            continue;
        }
        let sizes = artboard
            .component_list_state(provider_local)
            .map(|list| &list.logical_items)
            .map(|items| items.iter().map(|item| item.size).collect::<Vec<_>>())
            .unwrap_or_default();
        let mut running = 0.0;
        for (width, height) in sizes {
            flat_bounds.push(RuntimeLayoutBounds {
                x: if main_axis_is_horizontal {
                    running
                } else {
                    0.0
                },
                y: if main_axis_is_horizontal {
                    0.0
                } else {
                    running
                },
                width,
                height,
            });
            running += if main_axis_is_horizontal {
                width + gap_x
            } else {
                height + gap_y
            };
        }
    }
    flat_bounds
}

fn layout_component_style_local(artboard: &ArtboardInstance, layout_local: usize) -> Option<usize> {
    property_key_for_name("LayoutComponent", "styleId")
        .and_then(|key| artboard.uint_property(layout_local, key))
        .and_then(|style_local| usize::try_from(style_local).ok())
}

fn layout_component_axis_size(
    artboard: &ArtboardInstance,
    layout_bounds: Option<&std::collections::BTreeMap<usize, crate::draw::RuntimeLayoutBounds>>,
    layout_local: usize,
    horizontal: bool,
) -> f32 {
    if let Some(size) = layout_bounds
        .and_then(|bounds| bounds.get(&layout_local).copied())
        .map(|bounds| {
            if horizontal {
                bounds.width
            } else {
                bounds.height
            }
        })
        .filter(|size| size.is_finite() && *size > 0.0)
    {
        return size;
    }
    let property_name = if horizontal { "width" } else { "height" };
    let authored_size = property_key_for_name("LayoutComponent", property_name)
        .and_then(|key| artboard.double_property(layout_local, key))
        .filter(|size| size.is_finite() && *size > 0.0);
    if let Some(size) = authored_size {
        return size;
    }

    // A root-hosted zero-sized layout fills the artboard in C++/Yoga. This is
    // the common viewport shape and lets virtualization settle before the
    // first render-layout cache has been built.
    if artboard.component_parent_local(layout_local) == Some(0) {
        if horizontal {
            artboard.width
        } else {
            artboard.height
        }
    } else {
        0.0
    }
}

fn layout_style_axis_trailing_padding(
    artboard: &ArtboardInstance,
    style_local: Option<usize>,
    horizontal: bool,
) -> f32 {
    let property = if horizontal {
        "paddingRight"
    } else {
        "paddingBottom"
    };
    style_local
        .and_then(|style_local| {
            property_key_for_name("LayoutComponentStyle", property)
                .and_then(|key| artboard.double_property(style_local, key))
        })
        .filter(|padding| padding.is_finite())
        .unwrap_or(0.0)
}

fn layout_style_axis_leading_padding(
    artboard: &ArtboardInstance,
    style_local: Option<usize>,
    horizontal: bool,
) -> f32 {
    let property = if horizontal {
        "paddingLeft"
    } else {
        "paddingTop"
    };
    style_local
        .and_then(|style_local| {
            property_key_for_name("LayoutComponentStyle", property)
                .and_then(|key| artboard.double_property(style_local, key))
        })
        .filter(|padding| padding.is_finite())
        .unwrap_or(0.0)
}

fn virtualized_provider_item_sizes(
    artboard: &ArtboardInstance,
    layout_bounds: Option<&std::collections::BTreeMap<usize, crate::draw::RuntimeLayoutBounds>>,
    constraint: &RuntimeScrollConstraintState,
    current_list: Option<(usize, &[(f32, f32)])>,
) -> Vec<Vec<(f32, f32)>> {
    constraint
        .layout_children
        .iter()
        .map(|provider| {
            let Some(provider_local) = artboard.objects.component_local_id(*provider) else {
                return Vec::new();
            };
            if artboard
                .objects
                .component(*provider)
                .is_some_and(|component| component.concrete.constrainable_list.is_some())
            {
                if current_list.is_some_and(|(list_local, _)| provider_local == list_local) {
                    current_list
                        .map(|(_, item_sizes)| item_sizes.to_vec())
                        .unwrap_or_default()
                } else {
                    artboard
                        .component_list_state(provider_local)
                        .map(|list| &list.logical_items)
                        .map(|items| items.iter().map(|item| item.size).collect())
                        .unwrap_or_default()
                }
            } else {
                vec![(
                    layout_component_axis_size(artboard, layout_bounds, provider_local, true),
                    layout_component_axis_size(artboard, layout_bounds, provider_local, false),
                )]
            }
        })
        .collect()
}

fn virtualized_provider_content_size(
    provider_item_sizes: &[Vec<(f32, f32)>],
    is_horizontal: bool,
    gap: f32,
    infinite: bool,
) -> f32 {
    // This intentionally follows `ScrollConstraint::contentWidth/Height`, not
    // merely the flattened node count. Each provider contributes its aggregate
    // layout bounds, then the content layout contributes the inter-provider
    // gaps. For non-empty providers this is algebraically the same as one gap
    // between every flat node; retaining the two levels also matches C++ for an
    // empty list provider.
    let providers_extent = provider_item_sizes
        .iter()
        .map(|items| {
            let item_extent = items
                .iter()
                .map(|size| {
                    let value = if is_horizontal { size.0 } else { size.1 };
                    if value.is_finite() {
                        value.max(0.0)
                    } else {
                        0.0
                    }
                })
                .sum::<f32>();
            item_extent + gap * items.len().saturating_sub(1) as f32
        })
        .sum::<f32>();
    let inter_provider_gap_count = if infinite {
        provider_item_sizes.len()
    } else {
        provider_item_sizes.len().saturating_sub(1)
    };
    providers_extent + gap * inter_provider_gap_count as f32
}

#[cfg(test)]
fn clamped_scroll_offset(
    raw_offset: f32,
    viewport_size: f32,
    content_size: f32,
    trailing_padding: f32,
    infinite: bool,
) -> f32 {
    if infinite || !raw_offset.is_finite() {
        return raw_offset;
    }
    let max_offset = (viewport_size - content_size - trailing_padding).min(0.0);
    raw_offset.clamp(max_offset, 0.0)
}

fn scroll_viewport_axis_size(viewport_size: f32, content_origin: f32) -> f32 {
    (viewport_size - content_origin).max(0.0)
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq)]
struct TestVirtualizerPlacement {
    logical_index: usize,
    position_x: f32,
    position_y: f32,
}

#[cfg(test)]
fn test_virtualizer_placements_for_metrics(
    item_sizes: &[(f32, f32)],
    is_horizontal: bool,
    gap: f32,
    viewport_size: f32,
    scroll_offset: f32,
    infinite: bool,
) -> Vec<TestVirtualizerPlacement> {
    test_virtualizer_placements_for_providers(
        &[item_sizes.to_vec()],
        is_horizontal,
        gap,
        viewport_size,
        scroll_offset,
        infinite,
        virtualized_provider_content_size(&[item_sizes.to_vec()], is_horizontal, gap, infinite),
    )
    .pop()
    .unwrap_or_default()
}

#[cfg(test)]
fn test_virtualizer_placements_for_providers(
    provider_item_sizes: &[Vec<(f32, f32)>],
    is_horizontal: bool,
    gap: f32,
    viewport_size: f32,
    scroll_offset: f32,
    infinite: bool,
    content_size: f32,
) -> Vec<Vec<TestVirtualizerPlacement>> {
    let range = exact_scroll_virtualizer_range(
        provider_item_sizes,
        is_horizontal,
        gap,
        viewport_size,
        scroll_offset,
        infinite,
        content_size,
    );
    let total_item_count = provider_item_sizes.iter().map(Vec::len).sum::<usize>();
    let mut placements = vec![Vec::new(); provider_item_sizes.len()];
    if total_item_count == 0 {
        return placements;
    }
    let mut running_offset = range.running_offset;
    for global_index in range.visible_start..=range.visible_end {
        let actual_index = if infinite {
            global_index.rem_euclid(total_item_count as i32) as usize
        } else {
            global_index as usize
        };
        let mut running_total = 0usize;
        for (provider_index, child) in provider_item_sizes.iter().enumerate() {
            let start = running_total;
            let end = start + child.len();
            if start < end && actual_index >= start && actual_index < end {
                let logical_index = actual_index - start;
                let item = TestVirtualizerPlacement {
                    logical_index,
                    position_x: if is_horizontal { running_offset } else { 0.0 },
                    position_y: if is_horizontal { 0.0 } else { running_offset },
                };
                if let Some(existing) = placements[provider_index]
                    .iter_mut()
                    .find(|existing| existing.logical_index == logical_index)
                {
                    *existing = item;
                } else {
                    placements[provider_index].push(item);
                }
                let size = provider_item_sizes[provider_index][logical_index];
                running_offset += (if is_horizontal { size.0 } else { size.1 }) + gap;
                break;
            }
            running_total = end;
        }
    }
    placements
}

fn normalized_scroll_virtualizer_offset(offset: f32, infinite: bool, content_size: f32) -> f32 {
    let normalized_offset = -offset;
    if offset > 0.0 {
        if infinite {
            let offset_multiplier = (offset / content_size).floor() as i32 + 1;
            -1.0 * (offset - offset_multiplier as f32 * content_size)
        } else {
            -offset
        }
    } else {
        let offset_multiplier = (normalized_offset / content_size).floor() as i32;
        if offset_multiplier > 0 {
            normalized_offset % (offset_multiplier as f32 * content_size)
        } else {
            normalized_offset
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct RuntimeScrollVirtualizerRange {
    visible_start: i32,
    visible_end: i32,
    running_offset: f32,
}

/// Literal range-selection prefix of pinned
/// `ScrollVirtualizer::virtualize`.
///
/// The odd `currentChildIndex` comparisons and the unchanged `childIndex`
/// inside the visible-end loop are intentional pin behavior, not cleanups
/// (`scroll_virtualizer.cpp:54-153`). Recycling and interface calls remain in
/// `constrain_scroll_virtualizer` so production never materializes a Rust-only
/// provider window.
fn exact_scroll_virtualizer_range(
    provider_item_sizes: &[Vec<(f32, f32)>],
    is_horizontal: bool,
    gap: f32,
    viewport_size: f32,
    scroll_offset: f32,
    infinite: bool,
    content_size: f32,
) -> RuntimeScrollVirtualizerRange {
    let total_item_count = provider_item_sizes.iter().map(Vec::len).sum::<usize>();
    if provider_item_sizes.is_empty() || total_item_count == 0 || content_size <= 0.0 {
        return RuntimeScrollVirtualizerRange {
            visible_start: 0,
            visible_end: total_item_count as i32 - 1,
            running_offset: 0.0,
        };
    }
    let item_size = |provider: usize, index: usize| {
        let size = provider_item_sizes[provider][index];
        if is_horizontal { size.0 } else { size.1 }
    };
    let offset = normalized_scroll_virtualizer_offset(scroll_offset, infinite, content_size);

    let mut running_size = 0.0;
    let mut running_offset = 0.0;
    let mut running_index = 0usize;
    let mut child_index = 0usize;
    let mut current_child_index = 0usize;
    let mut visible_start = 0usize;
    let mut visible_end = total_item_count - 1;

    'find_start: for (i, child) in provider_item_sizes.iter().enumerate() {
        for j in 0..child.len() {
            let size = item_size(i, j);
            if running_size + size > offset {
                running_offset = running_size - offset;
                visible_start = running_index;
                if current_child_index == provider_item_sizes.len() - 1 {
                    child_index += 1;
                    current_child_index = 0;
                } else {
                    current_child_index += 1;
                }
                break 'find_start;
            }
            running_size += size;
            current_child_index = j;
            running_index += 1;
            if running_size + gap > offset {
                if running_index == total_item_count {
                    running_index = 0;
                }
                if current_child_index == provider_item_sizes.len() - 1 {
                    child_index += 1;
                    current_child_index = 0;
                } else {
                    current_child_index += 1;
                }
                running_size += gap;
                running_offset = running_size - offset;
                visible_start = running_index;
                break 'find_start;
            }
            running_size += gap;
        }
        child_index += 1;
    }

    child_index %= provider_item_sizes.len();
    let mut i = visible_start as i32;
    let mut wrapped = false;
    let mut cycle_count = 0;
    'find_end: while i < total_item_count as i32 && cycle_count < 2 {
        let child = &provider_item_sizes[child_index];
        for j in current_child_index..child.len() {
            let size = item_size(child_index, j);
            if running_size + size + gap >= offset + viewport_size {
                visible_end = if infinite && wrapped {
                    i as usize + total_item_count
                } else {
                    i as usize
                };
                break 'find_end;
            }
            running_size += size + gap;
            running_index += 1;
            if infinite && i == total_item_count as i32 - 1 {
                wrapped = true;
                i = -1;
                cycle_count += 1;
            }
            i += 1;
        }
        // Pinned C++ increments `runningIndex` in this loop even though the
        // visible-end result does not subsequently consult it
        // (`scroll_virtualizer.cpp:107-153`). Keep the literal translation
        // while making that deliberate dead write explicit to Rust.
        let _ = running_index;
        current_child_index = 0;
    }

    RuntimeScrollVirtualizerRange {
        visible_start: visible_start as i32,
        visible_end: visible_end as i32,
        running_offset,
    }
}

fn apply_constraint(
    artboard: &mut ArtboardInstance,
    component_index: ComponentHandle,
    constraint: ComponentHandle,
) -> bool {
    let Some(state) = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.concrete.constraint)
    else {
        return false;
    };
    match state.kind {
        RuntimeConstraintKind::Distance => {
            distance_constraint::apply(artboard, component_index, constraint, state)
        }
        RuntimeConstraintKind::Translation => {
            translation_constraint::apply(artboard, component_index, constraint, state)
        }
        RuntimeConstraintKind::Rotation => {
            rotation_constraint::apply(artboard, component_index, constraint, state)
        }
        RuntimeConstraintKind::Scale => {
            scale_constraint::apply(artboard, component_index, constraint, state)
        }
        RuntimeConstraintKind::Transform => {
            transform_constraint::apply(artboard, component_index, constraint, state)
        }
        RuntimeConstraintKind::FollowPath | RuntimeConstraintKind::ListFollowPath => {
            apply_follow_path_constraint(artboard, component_index, constraint)
        }
        RuntimeConstraintKind::Scroll => {
            apply_scroll_constraint(artboard, component_index, constraint)
        }
        RuntimeConstraintKind::ScrollBar => {
            apply_scroll_bar_constraint(artboard, component_index, constraint)
        }
        RuntimeConstraintKind::Ik => apply_ik_constraint(artboard, component_index, constraint),
        _ => false,
    }
}

fn apply_scroll_bar_constraint(
    artboard: &mut ArtboardInstance,
    component_index: ComponentHandle,
    constraint_handle: ComponentHandle,
) -> bool {
    // Literal owner translation of
    // `ScrollBarConstraint::{computedThumbWidth,computedThumbHeight,constrain}`
    // (`scroll_bar_constraint.cpp:12-118`).
    let constraint_local = artboard.component_at(constraint_handle).local_id;
    let Some((scroll_constraint, thumb, track)) = artboard
        .objects
        .component(constraint_handle)
        .and_then(|component| component.concrete.scroll_bar.as_ref())
        .and_then(|scroll_bar| {
            let thumb = artboard.objects.component(constraint_handle)?.parent?;
            let track = artboard.objects.component(thumb)?.parent?;
            Some((scroll_bar.scroll_constraint?, thumb, track))
        })
    else {
        return false;
    };
    if component_index != thumb {
        return false;
    }
    let Some(scroll) = artboard
        .objects
        .component(scroll_constraint)
        .and_then(|component| component.concrete.scroll.as_ref())
    else {
        return false;
    };
    let metrics = runtime_scroll_layout_metrics(artboard, scroll_constraint, scroll, false)
        .unwrap_or_else(|| {
            build_runtime_scroll_layout_metrics(artboard, scroll_constraint, scroll, None, false)
        });
    let (_, _, track_width, track_height) = constraint_bounds(artboard, track);
    let (_, _, authored_thumb_width, authored_thumb_height) = constraint_bounds(artboard, thumb);
    let padding_left = layout_style_axis_leading_padding(
        artboard,
        layout_component_style_local(artboard, artboard.component_at(track).local_id),
        true,
    );
    let padding_right = layout_style_axis_trailing_padding(
        artboard,
        layout_component_style_local(artboard, artboard.component_at(track).local_id),
        true,
    );
    let padding_top = layout_style_axis_leading_padding(
        artboard,
        layout_component_style_local(artboard, artboard.component_at(track).local_id),
        false,
    );
    let padding_bottom = layout_style_axis_trailing_padding(
        artboard,
        layout_component_style_local(artboard, artboard.component_at(track).local_id),
        false,
    );
    let inner_width = track_width - padding_left - padding_right;
    let inner_height = track_height - padding_top - padding_bottom;
    let auto_size = constraint_bool(
        artboard,
        constraint_local,
        "ScrollBarConstraint",
        "autoSize",
        true,
    );
    let direction = constraint_uint(
        artboard,
        constraint_local,
        "DraggableConstraint",
        "directionValue",
        1,
    );
    let constrains_horizontal = matches!(direction, 0 | 2);
    let constrains_vertical = matches!(direction, 1 | 2);
    let mut thumb_offset_x = 0.0;
    let mut thumb_offset_y = 0.0;
    if constrains_horizontal {
        let mut thumb_width = if auto_size {
            inner_width
                * if metrics.content_width == 0.0 {
                    1.0
                } else {
                    (metrics.viewport_width / metrics.content_width).min(1.0)
                }
        } else {
            authored_thumb_width
        };
        let max_thumb_offset = inner_width - thumb_width;
        let max_offset = metrics.max_offset(RuntimeScrollAxis::X);
        let clamped = clamped_scroll_constraint_offsets(artboard, scroll_constraint, &metrics).0;
        thumb_offset_x = if max_offset == 0.0 {
            0.0
        } else {
            clamped / max_offset * max_thumb_offset
        };
        if thumb_offset_x < 0.0 {
            thumb_width += thumb_offset_x;
            thumb_offset_x = 0.0;
        } else if thumb_offset_x > max_thumb_offset {
            thumb_width -= thumb_offset_x - max_thumb_offset;
            if !auto_size {
                thumb_offset_x = max_thumb_offset;
            }
        }
        if auto_size
            && let Some(layout) = artboard
                .objects
                .component(thumb)
                .and_then(|component| component.concrete.layout.as_ref())
        {
            layout.forced_width(thumb_width);
        }
    }
    if constrains_vertical {
        let mut thumb_height = if auto_size {
            inner_height
                * if metrics.content_height == 0.0 {
                    1.0
                } else {
                    (metrics.viewport_height / metrics.content_height).min(1.0)
                }
        } else {
            authored_thumb_height
        };
        let max_thumb_offset = inner_height - thumb_height;
        let max_offset = metrics.max_offset(RuntimeScrollAxis::Y);
        let clamped = clamped_scroll_constraint_offsets(artboard, scroll_constraint, &metrics).1;
        thumb_offset_y = if max_offset == 0.0 {
            0.0
        } else {
            clamped / max_offset * max_thumb_offset
        };
        if thumb_offset_y < 0.0 {
            thumb_height += thumb_offset_y;
            thumb_offset_y = 0.0;
        } else if thumb_offset_y > max_thumb_offset {
            thumb_height -= thumb_offset_y - max_thumb_offset;
            if !auto_size {
                thumb_offset_y = max_thumb_offset;
            }
        }
        if auto_size
            && let Some(layout) = artboard
                .objects
                .component(thumb)
                .and_then(|component| component.concrete.layout.as_ref())
        {
            layout.forced_height(thumb_height);
        }
    }
    let world = artboard
        .component_at(component_index)
        .transform
        .world_transform;
    let target = world.multiply(Mat2D([1.0, 0.0, 0.0, 1.0, thumb_offset_x, thumb_offset_y]));
    let strength = retained_constraint_double(
        artboard,
        constraint_local,
        RUNTIME_CONSTRAINT_PROPERTY_KEYS.strength,
        1.0,
    );
    let (constrained, components_a, components_b) =
        constrained_world_transform(world, target, strength);
    if let Some(scroll_bar) = artboard
        .objects
        .component_mut(constraint_handle)
        .and_then(|component| component.concrete.scroll_bar.as_mut())
    {
        scroll_bar.components_a = components_a;
        scroll_bar.components_b = components_b;
    }
    write_world_transform(artboard, component_index, constrained)
}

fn apply_scroll_constraint(
    artboard: &mut ArtboardInstance,
    component_index: ComponentHandle,
    constraint_handle: ComponentHandle,
) -> bool {
    // Ported from C++ `src/constraints/scrolling/scroll_constraint.cpp`
    // `ScrollConstraint::constrain` / `constrainChild`.
    let constraint_local = artboard.component_at(constraint_handle).local_id;
    let Some(content) = artboard
        .objects
        .component(constraint_handle)
        .and_then(|component| component.concrete.scroll.as_ref())
        .and_then(|scroll| scroll.content)
    else {
        return false;
    };
    if component_index != content {
        return false;
    }
    let computed_layout_bounds = artboard
        .runtime_graph()
        .and_then(|graph| artboard.runtime_taffy_layout_bounds(graph, artboard.runtime_file()));
    let retained_layout_bounds = artboard.layout_constraint_bounds.clone();
    let layout_bounds = retained_layout_bounds
        .as_deref()
        .or(computed_layout_bounds.as_ref());
    artboard
        .objects
        .component_mut(constraint_handle)
        .and_then(|component| component.concrete.scroll.as_mut())
        .expect("ScrollConstraint occurrence retains its concrete state")
        .layout_initialized = true;
    let intent_changed = {
        let scroll_constraint = artboard
            .objects
            .component(constraint_handle)
            .and_then(|component| component.concrete.scroll.as_ref())
            .expect("ScrollConstraint occurrence retains its concrete state");
        if scroll_constraint.intent_x.is_some() || scroll_constraint.intent_y.is_some() {
            let include_item_bounds = scroll_constraint
                .intent_x
                .into_iter()
                .chain(scroll_constraint.intent_y)
                .any(|intent| intent.space == RuntimeScrollSpace::Index);
            let scroll_metrics = build_runtime_scroll_layout_metrics(
                artboard,
                constraint_handle,
                scroll_constraint,
                layout_bounds,
                include_item_bounds,
            );
            resolve_runtime_scroll_intents(artboard, constraint_local, &scroll_metrics)
        } else {
            false
        }
    };
    let scroll_constraint = artboard
        .objects
        .component(constraint_handle)
        .and_then(|component| component.concrete.scroll.as_ref())
        .expect("ScrollConstraint occurrence retains its concrete state");
    let metrics = build_runtime_scroll_layout_metrics(
        artboard,
        constraint_handle,
        scroll_constraint,
        layout_bounds,
        false,
    );
    let (clamped_x, clamped_y) =
        clamped_scroll_constraint_offsets(artboard, constraint_handle, &metrics);
    let offset_x = if metrics.constrains_horizontal() {
        clamped_x
    } else {
        0.0
    };
    let offset_y = if metrics.constrains_vertical() {
        clamped_y
    } else {
        0.0
    };
    let scroll_transform = Mat2D([1.0, 0.0, 0.0, 1.0, offset_x, offset_y]);
    let scroll = artboard
        .objects
        .component_mut(constraint_handle)
        .and_then(|component| component.concrete.scroll.as_mut())
        .expect("ScrollConstraint occurrence retains its concrete state");
    scroll.scroll_transform = scroll_transform;
    scroll.child_constraint_applied_count = 0;
    intent_changed
}

pub(crate) fn update_follow_path_constraint(
    artboard: &mut ArtboardInstance,
    constraint: ComponentHandle,
) -> bool {
    let Some(target) = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.concrete.constraint)
        .and_then(|constraint| constraint.target)
    else {
        return false;
    };
    let path_handles = artboard
        .objects
        .component(target)
        .and_then(|component| component.concrete.shape.as_ref())
        .map(|shape| shape.paths.clone())
        .or_else(|| {
            artboard
                .objects
                .component(target)
                .and_then(|component| component.concrete.path.as_ref())
                .map(|_| vec![target])
        })
        .unwrap_or_default();

    // C++ preserves the previous RawPath/PathMeasure when a Shape currently
    // has no paths (`follow_path_constraint.cpp:122-147`).
    if path_handles.is_empty() {
        return false;
    }

    // C++ materializes only a local vector of retained Path pointers, then
    // rewinds and appends their RawPaths directly into the constraint owner.
    // Arc clones are the Rust pointer references; geometry is never lowered
    // into a temporary command buffer (`follow_path_constraint.cpp:122-145`).
    let mut sources = Vec::with_capacity(path_handles.len());
    for path_handle in path_handles {
        let Some(path_local) = artboard.objects.component_local_id(path_handle) else {
            continue;
        };
        let Some((raw_path, has_weighted_context)) = artboard
            .runtime_shapes
            .retained_follow_path_source(path_local)
        else {
            continue;
        };
        let transform = if has_weighted_context {
            Mat2D::IDENTITY
        } else {
            artboard.component_at(path_handle).transform.world_transform
        };
        sources.push((raw_path, transform));
    }
    let retained = artboard
        .objects
        .component_mut(constraint)
        .and_then(|component| component.concrete.follow_path.as_mut())
        .expect("FollowPathConstraint update requires its concrete owner");
    let verb_count = sources.iter().map(|(path, _)| path.verbs().len()).sum();
    let point_count = sources.iter().map(|(path, _)| path.points().len()).sum();
    retained.raw_path.rewind();
    retained.raw_path.reserve(verb_count, point_count);
    for (source, transform) in &sources {
        retained.raw_path.add_path(source, RenderMat2D(transform.0));
    }
    retained.path_measure = RuntimePathMeasure::from_raw_path(&retained.raw_path);
    #[cfg(test)]
    {
        retained.measure_rebuilds += 1;
    }
    true
}

fn apply_follow_path_constraint(
    artboard: &mut ArtboardInstance,
    component_index: ComponentHandle,
    constraint: ComponentHandle,
) -> bool {
    let Some(target) = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.concrete.constraint)
        .and_then(|constraint| constraint.target)
    else {
        return false;
    };
    if artboard.component_at(target).is_collapsed() {
        return false;
    }
    let constraint_local = artboard.component_at(constraint).local_id;
    let distance = retained_constraint_double(
        artboard,
        constraint_local,
        FOLLOW_PATH_DISTANCE_PROPERTY_KEY,
        0.0,
    );
    let transform_b = target_transform_for_follow_path_constraint_at_distance(
        artboard,
        constraint,
        target,
        component_index,
        distance,
    );
    let components = follow_path_constrain_components(
        artboard,
        constraint_local,
        target,
        artboard
            .component_at(component_index)
            .transform
            .world_transform,
        transform_b,
        parent_world_transform(artboard, component_index),
    );
    write_world_transform(artboard, component_index, Mat2D::compose(components))
}

fn target_transform_for_follow_path_constraint_at_distance(
    artboard: &ArtboardInstance,
    constraint: ComponentHandle,
    target: ComponentHandle,
    offset_component: ComponentHandle,
    distance: f32,
) -> Mat2D {
    let constraint_local = artboard.component_at(constraint).local_id;
    let target_component = artboard.component_at(target);
    if target_component.concrete.shape.is_none() && target_component.concrete.path.is_none() {
        return target_component.transform.world_transform;
    }

    let sample = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.concrete.follow_path.as_ref())
        .expect("FollowPathConstraint targetTransform requires retained measure")
        .path_measure
        .at_percentage(distance);
    let mut transform_b = target_component.transform.world_transform;

    if retained_constraint_bool(
        artboard,
        constraint_local,
        FOLLOW_PATH_ORIENT_PROPERTY_KEY,
        true,
    ) {
        let components_b = transform_b.decompose();
        let tangent_rotation = sample.tan.1.atan2(sample.tan.0);
        let two_pi = std::f32::consts::PI * 2.0;
        let angle_b = components_b.rotation % two_pi;
        let mut diff = tangent_rotation - angle_b;
        if diff > std::f32::consts::PI {
            diff -= two_pi;
        } else if diff < -std::f32::consts::PI {
            diff += two_pi;
        }
        transform_b = Mat2D::from_rotation(
            angle_b
                + diff
                    * retained_constraint_double(
                        artboard,
                        constraint_local,
                        RUNTIME_CONSTRAINT_PROPERTY_KEYS.strength,
                        1.0,
                    ),
        );
    }
    let offset_position = if retained_constraint_bool(
        artboard,
        constraint_local,
        FOLLOW_PATH_OFFSET_PROPERTY_KEY,
        false,
    ) {
        let local = artboard
            .component_at(offset_component)
            .transform
            .local_transform
            .0;
        (local[4], local[5])
    } else {
        (0.0, 0.0)
    };
    transform_b.0[4] = sample.pos.0 + offset_position.0;
    transform_b.0[5] = sample.pos.1 + offset_position.1;
    transform_b
}

fn apply_list_follow_path_constraint_to_transforms(
    artboard: &ArtboardInstance,
    list_component_index: ComponentHandle,
    constraint: ComponentHandle,
    item_transforms: &mut [Mat2D],
) -> bool {
    // Ported from C++ `src/constraints/list_follow_path_constraint.cpp`.
    let constraint_local = artboard.component_at(constraint).local_id;
    let target = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.concrete.constraint)
        .and_then(|constraint| constraint.target);
    let count = item_transforms.len();
    let distance = retained_constraint_double(
        artboard,
        constraint_local,
        FOLLOW_PATH_DISTANCE_PROPERTY_KEY,
        0.0,
    );
    let distance_end = retained_constraint_double(
        artboard,
        constraint_local,
        LIST_FOLLOW_PATH_DISTANCE_END_PROPERTY_KEY,
        1.0,
    );
    let distance_offset = retained_constraint_double(
        artboard,
        constraint_local,
        LIST_FOLLOW_PATH_DISTANCE_OFFSET_PROPERTY_KEY,
        0.0,
    );
    let start_offset = distance_offset + distance;
    let start_to_end_distance = distance_end - distance;
    let offset_distance = if count <= 1 {
        0.0
    } else {
        start_to_end_distance / (count as f32 - 1.0)
    };
    let list_transform = artboard
        .component_at(list_component_index)
        .transform
        .world_transform;
    let mut changed = false;

    for (index, transform) in item_transforms.iter_mut().enumerate() {
        let components = if let Some(target) =
            target.filter(|target| !artboard.component_at(*target).is_collapsed())
        {
            let transform_b = target_transform_for_follow_path_constraint_at_distance(
                artboard,
                constraint,
                target,
                list_component_index,
                start_offset + index as f32 * offset_distance,
            );
            follow_path_constrain_components(
                artboard,
                constraint_local,
                target,
                *transform,
                transform_b,
                list_transform,
            )
        } else {
            TransformComponents::default()
        };
        let next = Mat2D::compose(components);
        if *transform != next {
            *transform = next;
            changed = true;
        }
    }

    changed
}

fn follow_path_constrain_components(
    artboard: &ArtboardInstance,
    constraint_local: usize,
    target_index: ComponentHandle,
    component_transform: Mat2D,
    mut transform_b: Mat2D,
    component_parent_world: Mat2D,
) -> TransformComponents {
    if retained_constraint_space(
        artboard,
        constraint_local,
        RUNTIME_CONSTRAINT_PROPERTY_KEYS.source_space,
    ) == TransformSpace::Local
    {
        let target_parent_world = parent_world_transform(artboard, target_index);
        let Some(inverse) = invert(target_parent_world) else {
            return TransformComponents::default();
        };
        transform_b = inverse.multiply(transform_b);
    }
    if retained_constraint_space(
        artboard,
        constraint_local,
        RUNTIME_CONSTRAINT_PROPERTY_KEYS.dest_space,
    ) == TransformSpace::Local
    {
        transform_b = component_parent_world.multiply(transform_b);
    }

    let components_a = component_transform.decompose();
    let mut components_b = transform_b.decompose();
    let t = retained_constraint_double(
        artboard,
        constraint_local,
        RUNTIME_CONSTRAINT_PROPERTY_KEYS.strength,
        1.0,
    );
    let ti = 1.0 - t;

    if !retained_constraint_bool(
        artboard,
        constraint_local,
        FOLLOW_PATH_ORIENT_PROPERTY_KEY,
        true,
    ) {
        components_b.rotation = components_a.rotation % (std::f32::consts::PI * 2.0);
    }
    components_b.x = components_a.x * ti + components_b.x * t;
    components_b.y = components_a.y * ti + components_b.y * t;
    components_b.scale_x = components_a.scale_x;
    components_b.scale_y = components_a.scale_y;
    components_b.skew = components_a.skew;
    components_b
}

fn apply_ik_constraint(
    artboard: &mut ArtboardInstance,
    _component_index: ComponentHandle,
    constraint: ComponentHandle,
) -> bool {
    // Ported from C++ `src/constraints/ik_constraint.cpp`.
    let constraint_local = artboard.component_at(constraint).local_id;
    let Some(target_index) = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.concrete.constraint)
        .and_then(|constraint| constraint.target)
    else {
        return false;
    };
    if artboard.component_at(target_index).is_collapsed() {
        return false;
    }

    let invert_direction = retained_constraint_bool(
        artboard,
        constraint_local,
        IK_INVERT_DIRECTION_PROPERTY_KEY,
        false,
    );
    let world_target_translation = world_translation(
        artboard
            .component_at(target_index)
            .transform
            .world_transform,
    );
    let mut chain = std::mem::take(
        &mut artboard
            .objects
            .component_mut(constraint)
            .and_then(|component| component.concrete.ik.as_mut())
            .expect("IKConstraint apply requires its concrete owner")
            .chain,
    );
    let mut changed = false;
    for link in &mut chain {
        let bone_index = link.bone;
        let parent_world = parent_world_transform(artboard, bone_index);
        link.parent_world_inverse = parent_world.invert_or_identity();
        let bone_transform = link
            .parent_world_inverse
            .multiply(artboard.component_at(bone_index).transform.world_transform);
        changed |= write_local_transform(artboard, bone_index, bone_transform);
        link.transform_components = bone_transform.decompose();
    }

    match chain.len() {
        0 => {}
        1 => {
            changed |= solve_ik1(artboard, &mut chain, 0, world_target_translation);
        }
        2 => {
            changed |= solve_ik2(
                artboard,
                &mut chain,
                0,
                1,
                world_target_translation,
                invert_direction,
            );
        }
        count => {
            let tip_index = count - 1;
            for index in 0..tip_index {
                changed |= solve_ik2(
                    artboard,
                    &mut chain,
                    index,
                    tip_index,
                    world_target_translation,
                    invert_direction,
                );
                for child_index in (index + 1)..tip_index {
                    let bone_index = chain[child_index].bone;
                    chain[child_index].parent_world_inverse =
                        parent_world_transform(artboard, bone_index).invert_or_identity();
                }
            }
        }
    }

    let strength = retained_constraint_double(
        artboard,
        constraint_local,
        RUNTIME_CONSTRAINT_PROPERTY_KEYS.strength,
        1.0,
    );
    if strength != 1.0 {
        for index in 0..chain.len() {
            let from_angle =
                chain[index].transform_components.rotation % (std::f32::consts::PI * 2.0);
            let to_angle = chain[index].angle % (std::f32::consts::PI * 2.0);
            let mut diff = to_angle - from_angle;
            if diff > std::f32::consts::PI {
                diff -= std::f32::consts::PI * 2.0;
            } else if diff < -std::f32::consts::PI {
                diff += std::f32::consts::PI * 2.0;
            }
            changed |= constrain_ik_rotation(artboard, &chain[index], from_angle + diff * strength);
        }
    }

    artboard
        .objects
        .component_mut(constraint)
        .and_then(|component| component.concrete.ik.as_mut())
        .expect("IKConstraint owner remained live during apply")
        .chain = chain;
    changed
}

fn solve_ik1(
    artboard: &mut ArtboardInstance,
    chain: &mut [RuntimeIkChainLink],
    index: usize,
    world_target_translation: (f32, f32),
) -> bool {
    let bone_index = chain[index].bone;
    let p_a = world_translation(artboard.component_at(bone_index).transform.world_transform);
    let to_target = (
        world_target_translation.0 - p_a.0,
        world_target_translation.1 - p_a.1,
    );
    let to_target_local = chain[index]
        .parent_world_inverse
        .transform_direction(to_target.0, to_target.1);
    let rotation = point_atan2(to_target_local);
    chain[index].angle = rotation;
    constrain_ik_rotation(artboard, &chain[index], rotation)
}

fn solve_ik2(
    artboard: &mut ArtboardInstance,
    chain: &mut [RuntimeIkChainLink],
    fk1_index: usize,
    fk2_index: usize,
    world_target_translation: (f32, f32),
    invert_direction: bool,
) -> bool {
    let first_child_index = chain[fk1_index].index + 1;
    let b1_index = chain[fk1_index].bone;
    let b2_index = chain[fk2_index].bone;
    let first_child_bone_index = chain[first_child_index].bone;
    let iworld = chain[fk1_index].parent_world_inverse;

    let mut p_a = world_translation(artboard.component_at(b1_index).transform.world_transform);
    let mut p_c = world_translation(
        artboard
            .component_at(first_child_bone_index)
            .transform
            .world_transform,
    );
    let mut p_b = tip_world_translation(artboard, b2_index);
    let mut p_bt = world_target_translation;

    p_a = iworld.transform_point(p_a.0, p_a.1);
    p_c = iworld.transform_point(p_c.0, p_c.1);
    p_b = iworld.transform_point(p_b.0, p_b.1);
    p_bt = iworld.transform_point(p_bt.0, p_bt.1);

    let av = point_sub(p_b, p_c);
    let bv = point_sub(p_c, p_a);
    let cv = point_sub(p_bt, p_a);
    let a = point_length(av);
    let b = point_length(bv);
    let c = point_length(cv);

    let angle_a = ((-a * a + b * b + c * c) / (2.0 * b * c))
        .clamp(-1.0, 1.0)
        .acos();
    let angle_c = ((a * a + b * b - c * c) / (2.0 * a * b))
        .clamp(-1.0, 1.0)
        .acos();

    let (r1, r2) = if artboard.component_parent_handle(b2_index) != Some(b1_index) {
        let second_child_index = fk1_index + 2;
        let second_child_world_inverse = chain[second_child_index].parent_world_inverse;
        let p_c_world = world_translation(
            artboard
                .component_at(first_child_bone_index)
                .transform
                .world_transform,
        );
        let p_b_world = tip_world_translation(artboard, b2_index);
        let av_local = second_child_world_inverse
            .transform_direction(p_b_world.0 - p_c_world.0, p_b_world.1 - p_c_world.1);
        let angle_correction = -point_atan2(av_local);
        if invert_direction {
            (
                point_atan2(cv) - angle_a,
                -angle_c + std::f32::consts::PI + angle_correction,
            )
        } else {
            (
                angle_a + point_atan2(cv),
                angle_c - std::f32::consts::PI + angle_correction,
            )
        }
    } else if invert_direction {
        (point_atan2(cv) - angle_a, -angle_c + std::f32::consts::PI)
    } else {
        (angle_a + point_atan2(cv), angle_c - std::f32::consts::PI)
    };

    let mut changed = false;
    changed |= constrain_ik_rotation(artboard, &chain[fk1_index], r1);
    changed |= constrain_ik_rotation(artboard, &chain[first_child_index], r2);
    if first_child_index != fk2_index {
        let bone_index = chain[fk2_index].bone;
        let parent_world = parent_world_transform(artboard, bone_index);
        let local = artboard.component_at(bone_index).transform.local_transform;
        changed |= write_world_transform(artboard, bone_index, parent_world.multiply(local));
    }

    chain[fk1_index].angle = r1;
    chain[first_child_index].angle = r2;
    changed
}

fn constrain_ik_rotation(
    artboard: &mut ArtboardInstance,
    state: &RuntimeIkChainLink,
    rotation: f32,
) -> bool {
    let bone_index = state.bone;
    let components = state.transform_components;
    let mut local_transform = Mat2D::from_rotation(rotation);
    local_transform.0[4] = components.x;
    local_transform.0[5] = components.y;
    local_transform.0[0] *= components.scale_x;
    local_transform.0[1] *= components.scale_x;
    local_transform.0[2] *= components.scale_y;
    local_transform.0[3] *= components.scale_y;
    if components.skew != 0.0 {
        local_transform.0[2] = local_transform.0[0] * components.skew + local_transform.0[2];
        local_transform.0[3] = local_transform.0[1] * components.skew + local_transform.0[3];
    }
    let parent_world = parent_world_transform(artboard, bone_index);
    write_local_world_transform(
        artboard,
        bone_index,
        local_transform,
        parent_world.multiply(local_transform),
    )
}

fn target_transform_for_transform_constraint(
    artboard: &ArtboardInstance,
    target_index: ComponentHandle,
    origin_x: f32,
    origin_y: f32,
) -> Mat2D {
    let (left, top, width, height) = constraint_bounds(artboard, target_index);
    let component = artboard.component_at(target_index);
    component.transform.world_transform.multiply(Mat2D([
        1.0,
        0.0,
        0.0,
        1.0,
        left + width * origin_x,
        top + height * origin_y,
    ]))
}

fn constraint_bounds(
    artboard: &ArtboardInstance,
    component_index: ComponentHandle,
) -> (f32, f32, f32, f32) {
    let component = artboard.component_at(component_index);
    match component.concrete.constraint_bounds {
        RuntimeConstraintBoundsKind::Layout => {
            if let Some(layout) = component.concrete.layout.as_ref() {
                return layout.constraint_bounds();
            }
        }
        RuntimeConstraintBoundsKind::Text => {
            if let (Some(runtime), Some(graph)) =
                (artboard.runtime_file(), artboard.runtime_graph())
            {
                if let Some(bounds) =
                    static_text_constraint_bounds(runtime, graph, artboard, component.local_id)
                {
                    return bounds;
                }
            }
        }
        RuntimeConstraintBoundsKind::Default => {}
    }

    // C++ `TransformComponent::constraintBounds()` defaults to an empty AABB.
    // Concrete LayoutComponent/Text overrides read their retained owner state.
    (0.0, 0.0, 0.0, 0.0)
}

fn constrained_world_transform(
    from: Mat2D,
    to: Mat2D,
    strength: f32,
) -> (Mat2D, TransformComponents, TransformComponents) {
    let components_from = from.decompose();
    let mut components_to = to.decompose();
    let t = strength;
    let ti = 1.0 - t;

    components_to.rotation =
        interpolated_rotation_from_modded_base(components_from.rotation, components_to.rotation, t);
    components_to.x = components_from.x * ti + components_to.x * t;
    components_to.y = components_from.y * ti + components_to.y * t;
    components_to.scale_x = components_from.scale_x * ti + components_to.scale_x * t;
    components_to.scale_y = components_from.scale_y * ti + components_to.scale_y * t;
    components_to.skew = components_from.skew * ti + components_to.skew * t;

    (
        Mat2D::compose(components_to),
        components_from,
        components_to,
    )
}

fn write_world_transform(
    artboard: &mut ArtboardInstance,
    component_index: ComponentHandle,
    transform: Mat2D,
) -> bool {
    let world = &mut artboard
        .component_at_mut(component_index)
        .transform
        .world_transform
        .0;
    if *world == transform.0 {
        return false;
    }
    *world = transform.0;
    true
}

fn write_local_transform(
    artboard: &mut ArtboardInstance,
    component_index: ComponentHandle,
    transform: Mat2D,
) -> bool {
    let local = &mut artboard
        .component_at_mut(component_index)
        .transform
        .local_transform
        .0;
    if *local == transform.0 {
        return false;
    }
    *local = transform.0;
    true
}

fn write_local_world_transform(
    artboard: &mut ArtboardInstance,
    component_index: ComponentHandle,
    local_transform: Mat2D,
    world_transform: Mat2D,
) -> bool {
    let local_changed = write_local_transform(artboard, component_index, local_transform);
    let world_changed = write_world_transform(artboard, component_index, world_transform);
    local_changed || world_changed
}

fn world_translation(transform: Mat2D) -> (f32, f32) {
    (transform.0[4], transform.0[5])
}

fn tip_world_translation(artboard: &ArtboardInstance, bone_index: ComponentHandle) -> (f32, f32) {
    let bone = artboard.component_at(bone_index);
    let length = artboard.bone_length(bone.local_id).unwrap_or(0.0);
    bone.transform.world_transform.transform_point(length, 0.0)
}

fn point_sub(left: (f32, f32), right: (f32, f32)) -> (f32, f32) {
    (left.0 - right.0, left.1 - right.1)
}

fn point_length(point: (f32, f32)) -> f32 {
    (point.0 * point.0 + point.1 * point.1).sqrt()
}

fn point_atan2(point: (f32, f32)) -> f32 {
    point.1.atan2(point.0)
}

fn parent_world_transform(artboard: &ArtboardInstance, component_index: ComponentHandle) -> Mat2D {
    let Some(parent) = artboard.component_parent_handle(component_index) else {
        return Mat2D::IDENTITY;
    };
    Some(artboard.component_at(parent))
        .filter(|parent| parent.capabilities.world_transform)
        .map(|parent| parent.transform.world_transform)
        .unwrap_or(Mat2D::IDENTITY)
}

fn invert(transform: Mat2D) -> Option<Mat2D> {
    (transform.determinant() != 0.0).then(|| transform.invert_or_identity())
}

fn retained_constraint_double(
    artboard: &ArtboardInstance,
    local_id: usize,
    property_key: u16,
    default: f32,
) -> f32 {
    artboard
        .objects
        .double_property(local_id, property_key)
        .unwrap_or(default)
}

fn retained_constraint_bool(
    artboard: &ArtboardInstance,
    local_id: usize,
    property_key: u16,
    default: bool,
) -> bool {
    artboard
        .objects
        .bool_property(local_id, property_key)
        .unwrap_or(default)
}

fn retained_constraint_uint(
    artboard: &ArtboardInstance,
    local_id: usize,
    property_key: u16,
    default: u64,
) -> u64 {
    artboard
        .objects
        .uint_property(local_id, property_key)
        .unwrap_or(default)
}

fn retained_constraint_space(
    artboard: &ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> TransformSpace {
    TransformSpace::from_value(retained_constraint_uint(
        artboard,
        local_id,
        property_key,
        0,
    ))
}

fn retained_authored_transform_property(
    artboard: &ArtboardInstance,
    component: ComponentHandle,
    property: TransformProperty,
) -> f32 {
    let component = artboard.component_at(component);
    if let Some(bone) = component.concrete.bone.as_ref()
        && !bone.is_root
    {
        return match property {
            TransformProperty::X => component
                .parent
                .and_then(|parent| artboard.objects.component_local_id(parent))
                .and_then(|parent_local| {
                    artboard
                        .objects
                        .double_property(parent_local, BONE_LENGTH_PROPERTY_KEY)
                })
                .unwrap_or(0.0),
            TransformProperty::Y => 0.0,
            _ => retained_generated_transform_property(artboard, component, property),
        };
    }
    if let Some(layout) = component.concrete.layout.as_ref()
        && let Some(value) = layout.transform_property(property)
    {
        return value;
    }
    retained_generated_transform_property(artboard, component, property)
}

fn retained_generated_transform_property(
    artboard: &ArtboardInstance,
    component: &crate::components::RuntimeComponent,
    property: TransformProperty,
) -> f32 {
    component
        .transform_property_key(property)
        .and_then(|key| artboard.objects.double_property(component.local_id, key))
        .unwrap_or_else(|| property.default_value())
}

fn retain_constraint_component_a(
    artboard: &mut ArtboardInstance,
    constraint: ComponentHandle,
    components: TransformComponents,
) {
    let Some(state) = artboard
        .objects
        .component_mut(constraint)
        .and_then(|component| component.concrete.constraint.as_mut())
    else {
        return;
    };
    match &mut state.scratch {
        RuntimeConstraintScratch::Rotation {
            components_a: retained_a,
            ..
        }
        | RuntimeConstraintScratch::Scale {
            components_a: retained_a,
            ..
        } => {
            *retained_a = components;
        }
        RuntimeConstraintScratch::None | RuntimeConstraintScratch::Transform { .. } => {}
    }
}

fn retain_constraint_component_b(
    artboard: &mut ArtboardInstance,
    constraint: ComponentHandle,
    components: TransformComponents,
) {
    let Some(state) = artboard
        .objects
        .component_mut(constraint)
        .and_then(|component| component.concrete.constraint.as_mut())
    else {
        return;
    };
    match &mut state.scratch {
        RuntimeConstraintScratch::Rotation {
            components_b: retained_b,
            ..
        }
        | RuntimeConstraintScratch::Scale {
            components_b: retained_b,
            ..
        } => {
            *retained_b = components;
        }
        RuntimeConstraintScratch::None | RuntimeConstraintScratch::Transform { .. } => {}
    }
}

fn retain_transform_constraint_components(
    artboard: &mut ArtboardInstance,
    constraint: ComponentHandle,
    components_a: TransformComponents,
    components_b: TransformComponents,
) {
    let Some(state) = artboard
        .objects
        .component_mut(constraint)
        .and_then(|component| component.concrete.constraint.as_mut())
    else {
        return;
    };
    if let RuntimeConstraintScratch::Transform {
        components_a: retained_a,
        components_b: retained_b,
    } = &mut state.scratch
    {
        *retained_a = components_a;
        *retained_b = components_b;
    }
}

fn interpolated_rotation(from: f32, to: f32, strength: f32) -> f32 {
    let two_pi = std::f32::consts::PI * 2.0;
    let angle_a = from % two_pi;
    let angle_b = to % two_pi;
    let mut diff = angle_b - angle_a;
    if diff > std::f32::consts::PI {
        diff -= two_pi;
    } else if diff < -std::f32::consts::PI {
        diff += two_pi;
    }
    from + diff * strength
}

fn interpolated_rotation_from_modded_base(from: f32, to: f32, strength: f32) -> f32 {
    let two_pi = std::f32::consts::PI * 2.0;
    let angle_a = from % two_pi;
    let angle_b = to % two_pi;
    let mut diff = angle_b - angle_a;
    if diff > std::f32::consts::PI {
        diff -= two_pi;
    } else if diff < -std::f32::consts::PI {
        diff += two_pi;
    }
    angle_a + diff * strength
}

fn constraint_double(
    artboard: &ArtboardInstance,
    local_id: usize,
    type_name: &str,
    property_name: &str,
    default: f32,
) -> f32 {
    property_key_for_name(type_name, property_name)
        .and_then(|key| artboard.double_property(local_id, key))
        .unwrap_or(default)
}

fn constraint_bool(
    artboard: &ArtboardInstance,
    local_id: usize,
    type_name: &str,
    property_name: &str,
    default: bool,
) -> bool {
    property_key_for_name(type_name, property_name)
        .and_then(|key| artboard.bool_property(local_id, key))
        .unwrap_or(default)
}

fn constraint_uint(
    artboard: &ArtboardInstance,
    local_id: usize,
    type_name: &str,
    property_name: &str,
    default: u64,
) -> u64 {
    property_key_for_name(type_name, property_name)
        .and_then(|key| artboard.uint_property(local_id, key))
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use nuxie_binary::read_runtime_file;
    use nuxie_graph::GraphFile;

    use crate::draw::RuntimeLayoutBounds;
    use crate::properties::property_key_for_name;
    use crate::{ArtboardInstance, TransformProperty};

    use super::{
        BONE_LENGTH_PROPERTY_KEY, FOLLOW_PATH_DISTANCE_PROPERTY_KEY,
        FOLLOW_PATH_OFFSET_PROPERTY_KEY, FOLLOW_PATH_ORIENT_PROPERTY_KEY,
        IK_INVERT_DIRECTION_PROPERTY_KEY, IK_PARENT_BONE_COUNT_PROPERTY_KEY,
        LIST_FOLLOW_PATH_DISTANCE_END_PROPERTY_KEY, LIST_FOLLOW_PATH_DISTANCE_OFFSET_PROPERTY_KEY,
        RUNTIME_CONSTRAINT_PROPERTY_KEYS, RuntimeDraggableProxyKind, RuntimeScrollAxis,
        RuntimeScrollAxisIntent, RuntimeScrollConstraintState, RuntimeScrollLayoutMetrics,
        RuntimeScrollProperty, RuntimeScrollSpace, TestVirtualizerPlacement, clamped_scroll_offset,
        interpolated_rotation, interpolated_rotation_from_modded_base, point_length,
        runtime_draggable_proxies, runtime_draggable_proxy_drag, runtime_draggable_proxy_end,
        runtime_draggable_proxy_start, runtime_scroll_intent_axes, scroll_viewport_axis_size,
        test_virtualizer_placements_for_metrics, test_virtualizer_placements_for_providers,
        virtualized_provider_content_size,
    };

    #[test]
    fn pinned_constraint_property_constants_match_generated_schema() {
        let keys = RUNTIME_CONSTRAINT_PROPERTY_KEYS;
        for (type_name, property_name, actual) in [
            ("Constraint", "strength", keys.strength),
            ("TargetedConstraint", "targetId", keys.target_id),
            (
                "TransformSpaceConstraint",
                "sourceSpaceValue",
                keys.source_space,
            ),
            (
                "TransformSpaceConstraint",
                "destSpaceValue",
                keys.dest_space,
            ),
            (
                "TransformComponentConstraint",
                "minMaxSpaceValue",
                keys.min_max_space,
            ),
            (
                "TransformComponentConstraint",
                "copyFactor",
                keys.copy_factor,
            ),
            ("TransformComponentConstraint", "minValue", keys.min_value),
            ("TransformComponentConstraint", "maxValue", keys.max_value),
            ("TransformComponentConstraint", "offset", keys.offset),
            ("TransformComponentConstraint", "doesCopy", keys.does_copy),
            ("TransformComponentConstraint", "min", keys.min),
            ("TransformComponentConstraint", "max", keys.max),
            (
                "TransformComponentConstraintY",
                "copyFactorY",
                keys.copy_factor_y,
            ),
            (
                "TransformComponentConstraintY",
                "minValueY",
                keys.min_value_y,
            ),
            (
                "TransformComponentConstraintY",
                "maxValueY",
                keys.max_value_y,
            ),
            (
                "TransformComponentConstraintY",
                "doesCopyY",
                keys.does_copy_y,
            ),
            ("TransformComponentConstraintY", "minY", keys.min_y),
            ("TransformComponentConstraintY", "maxY", keys.max_y),
            ("DistanceConstraint", "distance", keys.distance),
            ("DistanceConstraint", "modeValue", keys.mode),
            ("TransformConstraint", "originX", keys.origin_x),
            ("TransformConstraint", "originY", keys.origin_y),
            (
                "FollowPathConstraint",
                "distance",
                FOLLOW_PATH_DISTANCE_PROPERTY_KEY,
            ),
            (
                "FollowPathConstraint",
                "orient",
                FOLLOW_PATH_ORIENT_PROPERTY_KEY,
            ),
            (
                "FollowPathConstraint",
                "offset",
                FOLLOW_PATH_OFFSET_PROPERTY_KEY,
            ),
            (
                "ListFollowPathConstraint",
                "distanceEnd",
                LIST_FOLLOW_PATH_DISTANCE_END_PROPERTY_KEY,
            ),
            (
                "ListFollowPathConstraint",
                "distanceOffset",
                LIST_FOLLOW_PATH_DISTANCE_OFFSET_PROPERTY_KEY,
            ),
            (
                "IKConstraint",
                "invertDirection",
                IK_INVERT_DIRECTION_PROPERTY_KEY,
            ),
            (
                "IKConstraint",
                "parentBoneCount",
                IK_PARENT_BONE_COUNT_PROPERTY_KEY,
            ),
            ("Bone", "length", BONE_LENGTH_PROPERTY_KEY),
        ] {
            assert_eq!(
                property_key_for_name(type_name, property_name),
                Some(actual),
                "{type_name}.{property_name}"
            );
        }
    }

    #[test]
    fn constraint_rotation_helpers_preserve_distinct_cpp_base_angles() {
        let from = std::f32::consts::TAU + 0.2;
        let to = 0.4;
        let strength = 0.5;
        let rotation = interpolated_rotation(from, to, strength);
        let transform = interpolated_rotation_from_modded_base(from, to, strength);

        assert!((rotation - (std::f32::consts::TAU + 0.3)).abs() < 1e-5);
        assert!((transform - 0.3).abs() < 1e-5);
    }

    #[test]
    fn distance_constraint_length_preserves_literal_cpp_operation_order() {
        let coordinate = f32::MAX / 2.0;
        assert!(point_length((coordinate, coordinate)).is_infinite());
        assert!(coordinate.hypot(coordinate).is_finite());
    }

    fn vertical_item(logical_index: usize, y: f32) -> TestVirtualizerPlacement {
        TestVirtualizerPlacement {
            logical_index,
            position_x: 0.0,
            position_y: y,
        }
    }

    fn push_var_uint(bytes: &mut Vec<u8>, mut value: u64) {
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            bytes.push(byte);
            if value == 0 {
                break;
            }
        }
    }

    fn schema_property_key(type_name: &str, property_name: &str) -> u64 {
        let definition = nuxie_schema::definition_by_name(type_name)
            .unwrap_or_else(|| panic!("missing schema definition {type_name}"));
        definition
            .properties
            .iter()
            .find(|property| property.name == property_name)
            .map(|property| property.key.int)
            .or_else(|| {
                definition.ancestors.iter().find_map(|ancestor| {
                    nuxie_schema::definition_by_name(ancestor).and_then(|ancestor| {
                        ancestor
                            .properties
                            .iter()
                            .find(|property| property.name == property_name)
                            .map(|property| property.key.int)
                    })
                })
            })
            .unwrap_or_else(|| panic!("missing property {type_name}.{property_name}"))
            .into()
    }

    fn push_object(bytes: &mut Vec<u8>, type_name: &str, properties: impl FnOnce(&mut Vec<u8>)) {
        let type_key = nuxie_schema::definition_by_name(type_name)
            .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
            .type_key
            .int;
        push_var_uint(bytes, u64::from(type_key));
        properties(bytes);
        push_var_uint(bytes, 0);
    }

    fn push_uint(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: u64) {
        push_var_uint(bytes, schema_property_key(type_name, property_name));
        push_var_uint(bytes, value);
    }

    fn push_f32(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: f32) {
        push_var_uint(bytes, schema_property_key(type_name, property_name));
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn instance_from_objects(file_id: u64, objects: impl FnOnce(&mut Vec<u8>)) -> ArtboardInstance {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIVE");
        push_var_uint(&mut bytes, 7);
        push_var_uint(&mut bytes, 0);
        push_var_uint(&mut bytes, file_id);
        push_var_uint(&mut bytes, 0);
        objects(&mut bytes);
        let file = read_runtime_file(&bytes).expect("synthetic constraint fixture imports");
        let graphs =
            GraphFile::from_runtime_file(&file).expect("synthetic constraint fixture graphs");
        let graph = graphs
            .artboards
            .first()
            .expect("constraint fixture has an artboard");
        ArtboardInstance::from_graph(&file, graph).expect("constraint fixture instance builds")
    }

    #[test]
    fn translation_offset_reads_bone_virtual_x_y_from_parent_length() {
        let mut instance = instance_from_objects(9_703, |bytes| {
            push_object(bytes, "Backboard", |_| {});
            push_object(bytes, "Artboard", |_| {});
            push_object(bytes, "RootBone", |bytes| {
                push_uint(bytes, "RootBone", "parentId", 0);
                push_f32(bytes, "RootBone", "length", 10.0);
            });
            push_object(bytes, "Bone", |bytes| {
                push_uint(bytes, "Bone", "parentId", 1);
            });
            push_object(bytes, "Node", |bytes| {
                push_uint(bytes, "Node", "parentId", 0);
                push_f32(bytes, "Node", "x", 20.0);
                push_f32(bytes, "Node", "y", 7.0);
            });
            push_object(bytes, "TranslationConstraint", |bytes| {
                push_uint(bytes, "TranslationConstraint", "parentId", 2);
                push_uint(bytes, "TranslationConstraint", "targetId", 3);
                push_uint(bytes, "TranslationConstraint", "offset", 1);
            });
        });

        assert!(instance.update_components().did_update);
        let world = instance
            .component(2)
            .expect("constrained Bone")
            .transform
            .world_transform;
        assert!((world.0[4] - 30.0).abs() < 1e-5);
        assert!((world.0[5] - 7.0).abs() < 1e-5);
    }

    #[test]
    fn translation_offset_reads_nested_layout_parent_local_x_y() {
        let mut instance = instance_from_objects(9_704, |bytes| {
            push_object(bytes, "Backboard", |_| {});
            push_object(bytes, "Artboard", |bytes| {
                push_f32(bytes, "Artboard", "width", 300.0);
                push_f32(bytes, "Artboard", "height", 200.0);
            });
            push_object(bytes, "LayoutComponent", |bytes| {
                push_uint(bytes, "LayoutComponent", "parentId", 0);
                push_uint(bytes, "LayoutComponent", "styleId", 2);
                push_f32(bytes, "LayoutComponent", "width", 100.0);
                push_f32(bytes, "LayoutComponent", "height", 80.0);
            });
            push_object(bytes, "LayoutComponentStyle", |bytes| {
                push_uint(bytes, "LayoutComponentStyle", "positionTypeValue", 2);
                push_f32(bytes, "LayoutComponentStyle", "positionLeft", 11.0);
                push_uint(bytes, "LayoutComponentStyle", "positionLeftUnitsValue", 1);
                push_f32(bytes, "LayoutComponentStyle", "positionTop", 19.0);
                push_uint(bytes, "LayoutComponentStyle", "positionTopUnitsValue", 1);
            });
            push_object(bytes, "LayoutComponent", |bytes| {
                push_uint(bytes, "LayoutComponent", "parentId", 1);
                push_uint(bytes, "LayoutComponent", "styleId", 4);
                push_f32(bytes, "LayoutComponent", "width", 40.0);
                push_f32(bytes, "LayoutComponent", "height", 30.0);
            });
            push_object(bytes, "LayoutComponentStyle", |bytes| {
                push_uint(bytes, "LayoutComponentStyle", "positionTypeValue", 2);
                push_f32(bytes, "LayoutComponentStyle", "positionLeft", 17.0);
                push_uint(bytes, "LayoutComponentStyle", "positionLeftUnitsValue", 1);
                push_f32(bytes, "LayoutComponentStyle", "positionTop", 23.0);
                push_uint(bytes, "LayoutComponentStyle", "positionTopUnitsValue", 1);
            });
            push_object(bytes, "Node", |bytes| {
                push_uint(bytes, "Node", "parentId", 0);
                push_f32(bytes, "Node", "x", 50.0);
                push_f32(bytes, "Node", "y", 60.0);
            });
            push_object(bytes, "TranslationConstraint", |bytes| {
                push_uint(bytes, "TranslationConstraint", "parentId", 3);
                push_uint(bytes, "TranslationConstraint", "targetId", 5);
                push_uint(bytes, "TranslationConstraint", "offset", 1);
            });
        });

        assert!(instance.update_components().did_update);
        let retained_layout = instance
            .component(3)
            .and_then(|component| component.concrete.layout.as_ref())
            .expect("retained nested layout");
        assert_eq!(
            retained_layout.transform_property(TransformProperty::X),
            Some(17.0)
        );
        assert_eq!(
            retained_layout.transform_property(TransformProperty::Y),
            Some(23.0)
        );
        let world = instance
            .component(3)
            .expect("constrained nested LayoutComponent")
            .transform
            .world_transform;
        assert!((world.0[4] - 67.0).abs() < 1e-5, "{world:?}");
        assert!((world.0[5] - 83.0).abs() < 1e-5, "{world:?}");
    }

    fn scroll_intent_fixture() -> (ArtboardInstance, usize) {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIVE");
        push_var_uint(&mut bytes, 7);
        push_var_uint(&mut bytes, 0);
        push_var_uint(&mut bytes, 9_702);
        push_var_uint(&mut bytes, 0);
        push_object(&mut bytes, "Backboard", |_| {});
        push_object(&mut bytes, "Artboard", |bytes| {
            push_f32(bytes, "LayoutComponent", "width", 500.0);
            push_f32(bytes, "LayoutComponent", "height", 500.0);
        });
        push_object(&mut bytes, "LayoutComponent", |bytes| {
            push_uint(bytes, "Node", "parentId", 0);
            push_f32(bytes, "LayoutComponent", "width", 500.0);
            push_f32(bytes, "LayoutComponent", "height", 500.0);
            push_uint(bytes, "LayoutComponent", "styleId", 2);
        });
        push_object(&mut bytes, "LayoutComponentStyle", |_| {});
        push_object(&mut bytes, "LayoutComponent", |bytes| {
            push_uint(bytes, "Node", "parentId", 1);
            push_f32(bytes, "LayoutComponent", "width", 500.0);
            push_f32(bytes, "LayoutComponent", "height", 1_110.0);
            push_uint(bytes, "LayoutComponent", "styleId", 4);
        });
        push_object(&mut bytes, "LayoutComponentStyle", |bytes| {
            push_f32(bytes, "LayoutComponentStyle", "gapVertical", 10.0);
            push_uint(bytes, "LayoutComponentStyle", "flexDirectionValue", 0);
        });
        for index in 0..10 {
            let local_id = 5 + index * 2;
            push_object(&mut bytes, "LayoutComponent", |bytes| {
                push_uint(bytes, "Node", "parentId", 3);
                push_f32(bytes, "LayoutComponent", "width", 500.0);
                push_f32(bytes, "LayoutComponent", "height", 100.0);
                push_uint(bytes, "LayoutComponent", "styleId", local_id + 1);
            });
            push_object(&mut bytes, "LayoutComponentStyle", |_| {});
        }
        let constraint_local = 25;
        push_object(&mut bytes, "ScrollConstraint", |bytes| {
            push_uint(bytes, "Component", "parentId", 3);
        });

        let file = read_runtime_file(&bytes).expect("synthetic scroll intent fixture imports");
        let graphs = GraphFile::from_runtime_file(&file).expect("synthetic scroll fixture graphs");
        let graph = graphs.artboards.first().expect("fixture has an artboard");
        let instance = ArtboardInstance::from_graph(&file, graph).expect("fixture instance builds");
        (instance, constraint_local)
    }

    fn scroll_bar_proxy_fixture() -> (ArtboardInstance, usize, usize) {
        let mut instance = instance_from_objects(9_705, |bytes| {
            push_object(bytes, "Backboard", |_| {});
            push_object(bytes, "Artboard", |bytes| {
                push_f32(bytes, "Artboard", "width", 200.0);
                push_f32(bytes, "Artboard", "height", 200.0);
            });
            push_object(bytes, "LayoutComponent", |bytes| {
                push_uint(bytes, "LayoutComponent", "parentId", 0);
                push_uint(bytes, "LayoutComponent", "styleId", 2);
                push_f32(bytes, "LayoutComponent", "width", 100.0);
                push_f32(bytes, "LayoutComponent", "height", 100.0);
            });
            push_object(bytes, "LayoutComponentStyle", |_| {});
            push_object(bytes, "LayoutComponent", |bytes| {
                push_uint(bytes, "LayoutComponent", "parentId", 1);
                push_uint(bytes, "LayoutComponent", "styleId", 4);
                push_f32(bytes, "LayoutComponent", "width", 100.0);
                push_f32(bytes, "LayoutComponent", "height", 300.0);
            });
            push_object(bytes, "LayoutComponentStyle", |_| {});
            push_object(bytes, "ScrollConstraint", |bytes| {
                push_uint(bytes, "ScrollConstraint", "parentId", 3);
                push_f32(bytes, "ScrollConstraint", "threshold", 5.0);
                push_f32(bytes, "ScrollConstraint", "dragMultiplier", 2.0);
            });
            push_object(bytes, "LayoutComponent", |bytes| {
                push_uint(bytes, "LayoutComponent", "parentId", 0);
                push_uint(bytes, "LayoutComponent", "styleId", 7);
                push_f32(bytes, "LayoutComponent", "width", 20.0);
                push_f32(bytes, "LayoutComponent", "height", 100.0);
            });
            push_object(bytes, "LayoutComponentStyle", |_| {});
            push_object(bytes, "LayoutComponent", |bytes| {
                push_uint(bytes, "LayoutComponent", "parentId", 6);
                push_uint(bytes, "LayoutComponent", "styleId", 9);
                push_f32(bytes, "LayoutComponent", "width", 20.0);
                push_f32(bytes, "LayoutComponent", "height", 20.0);
            });
            push_object(bytes, "LayoutComponentStyle", |_| {});
            push_object(bytes, "ScrollBarConstraint", |bytes| {
                push_uint(bytes, "ScrollBarConstraint", "parentId", 8);
                push_uint(bytes, "ScrollBarConstraint", "scrollConstraintId", 5);
            });
        });
        instance.update_pass();
        let scroll = instance
            .component_handle(5)
            .expect("ScrollConstraint handle");
        instance
            .objects
            .component_mut(scroll)
            .and_then(|component| component.concrete.scroll.as_mut())
            .expect("retained ScrollConstraint")
            .physics = Some(crate::components::RuntimeScrollPhysicsState::clamped());
        (instance, 5, 10)
    }

    #[test]
    fn draggable_proxy_lifecycle_matches_cpp_owner_state() {
        let (mut instance, scroll_local, scroll_bar_local) = scroll_bar_proxy_fixture();
        let mut proxies = runtime_draggable_proxies(&instance);
        assert_eq!(proxies.len(), 3);
        assert!(
            proxies.iter().any(|proxy| {
                proxy.kind == RuntimeDraggableProxyKind::Viewport && !proxy.opaque
            })
        );
        assert!(
            proxies
                .iter()
                .any(|proxy| { proxy.kind == RuntimeDraggableProxyKind::Thumb && proxy.opaque })
        );
        assert!(
            proxies
                .iter()
                .any(|proxy| proxy.kind == RuntimeDraggableProxyKind::Track && !proxy.opaque)
        );

        let viewport_index = proxies
            .iter()
            .position(|proxy| proxy.kind == RuntimeDraggableProxyKind::Viewport)
            .unwrap();
        runtime_draggable_proxy_start(
            &mut instance,
            &mut proxies[viewport_index],
            (10.0, 10.0),
            1.0,
        );
        let scroll = instance
            .component(scroll_local)
            .and_then(|component| component.concrete.scroll.as_ref())
            .unwrap();
        assert!(scroll.is_dragging);
        assert!(!runtime_draggable_proxy_drag(
            &mut instance,
            &mut proxies[viewport_index],
            (10.0, 14.0),
            1.1,
        ));
        assert!(runtime_draggable_proxy_drag(
            &mut instance,
            &mut proxies[viewport_index],
            (10.0, 20.0),
            1.2,
        ));
        assert_eq!(
            instance
                .component(scroll_local)
                .and_then(|component| component.concrete.scroll.as_ref())
                .map(|scroll| scroll.offset_y),
            Some(20.0)
        );
        runtime_draggable_proxy_end(&mut instance, &mut proxies[viewport_index]);
        assert!(
            !instance
                .component(scroll_local)
                .and_then(|component| component.concrete.scroll.as_ref())
                .unwrap()
                .is_dragging
        );

        let thumb_index = proxies
            .iter()
            .position(|proxy| proxy.kind == RuntimeDraggableProxyKind::Thumb)
            .unwrap();
        runtime_draggable_proxy_start(&mut instance, &mut proxies[thumb_index], (10.0, 10.0), 2.0);
        assert!(
            instance
                .component(scroll_local)
                .and_then(|component| component.concrete.scroll.as_ref())
                .unwrap()
                .is_scroll_bar_dragging
        );
        runtime_draggable_proxy_end(&mut instance, &mut proxies[thumb_index]);
        let scroll = instance
            .component(scroll_local)
            .and_then(|component| component.concrete.scroll.as_ref())
            .unwrap();
        assert!(!scroll.is_scroll_bar_dragging);
        assert_eq!(
            scroll.physics.as_ref().map(|physics| physics.speed),
            Some((0.0, 0.0))
        );

        assert!(
            instance
                .component(scroll_bar_local)
                .and_then(|component| component.concrete.scroll_bar.as_ref())
                .is_some_and(|scroll_bar| scroll_bar.scroll_constraint.is_some())
        );
        let mut dirty = proxies[thumb_index].clone();
        dirty.active_pointers.push(42);
        dirty.has_scrolled = true;
        dirty.viewport_is_dragging = true;
        let cold = dirty.clone_cold();
        assert!(cold.active_pointers.is_empty());
        assert!(!cold.has_scrolled);
        assert!(!cold.viewport_is_dragging);
    }

    #[test]
    fn scroll_drag_dirties_and_reconstrains_retained_layout_children() {
        let (mut instance, constraint_local) = scroll_intent_fixture();
        instance.update_pass();
        let constraint = instance
            .component_handle(constraint_local)
            .expect("ScrollConstraint handle");
        let child = instance
            .objects
            .component(constraint)
            .and_then(|component| component.concrete.scroll.as_ref())
            .and_then(|scroll| scroll.layout_children.first())
            .copied()
            .expect("ScrollConstraint retains a layout child");
        let initial_y = instance.component_at(child).transform.world_transform.0[5];
        let mut proxy = runtime_draggable_proxies(&instance)
            .into_iter()
            .find(|proxy| proxy.kind == RuntimeDraggableProxyKind::Viewport)
            .expect("ScrollConstraint owns a viewport draggable proxy");

        runtime_draggable_proxy_start(&mut instance, &mut proxy, (250.0, 250.0), 0.1);
        assert!(runtime_draggable_proxy_drag(
            &mut instance,
            &mut proxy,
            (250.0, 200.0),
            0.116,
        ));
        instance.update_pass();

        assert_eq!(
            instance
                .objects
                .component(constraint)
                .and_then(|component| component.concrete.scroll.as_ref())
                .map(|scroll| scroll.offset_y),
            Some(-50.0)
        );
        assert_eq!(
            instance.component_at(child).transform.world_transform.0[5],
            initial_y - 50.0,
            "ScrollConstraint::dragView writes scrollOffsetY, whose changed \
             callback marks content world-transform dirty; constrain and \
             constrainChild apply the retained translation in the same \
             update pass (`scroll_constraint.cpp:170-208,244-255`)"
        );
    }

    #[test]
    fn public_scroll_observation_finds_the_occurrence_and_reports_live_state() {
        let (mut instance, constraint_local) = scroll_intent_fixture();
        instance.update_pass();
        let content_local = instance
            .scroll_constraint_occurrences()
            .first()
            .expect("imported ScrollConstraint occurrence")
            .content_local_id;
        let constraint_source_id = instance
            .slot(constraint_local)
            .expect("authored ScrollConstraint slot")
            .source_global_id;

        let offset_y = property_key_for_name("ScrollConstraint", "scrollOffsetY").unwrap();
        assert!(instance.set_double_property(constraint_local, offset_y, -700.0));
        instance.update_pass();
        let constraint = instance
            .component_handle(constraint_local)
            .expect("ScrollConstraint handle");
        let scroll = instance
            .objects
            .component_mut(constraint)
            .and_then(|component| component.concrete.scroll.as_mut())
            .expect("retained ScrollConstraint");
        let mut physics = crate::components::RuntimeScrollPhysicsState::clamped();
        physics.run(
            (0.0, -610.0),
            (0.0, 0.0),
            (0.0, scroll.offset_y),
            &[],
            1_110.0,
            500.0,
        );
        scroll.physics = Some(physics);

        let by_content = instance
            .scroll_constraint_for_content(content_local)
            .expect("query by constrained component");
        let by_authored = instance
            .scroll_constraint_for_authored_id(constraint_source_id)
            .expect("query by authored identity");
        let by_content_authored = instance
            .scroll_constraint_for_content_authored_id(by_content.content_authored_id)
            .expect("query by constrained component authored identity");

        assert_eq!(by_content, by_authored);
        assert_eq!(by_content, by_content_authored);
        assert_eq!(by_content.constraint_local_id, constraint_local);
        assert_eq!(by_content.constraint_authored_id, constraint_source_id);
        assert_eq!(by_content.content_local_id, content_local);
        assert_eq!(by_content.offset, (0.0, -700.0));
        assert_eq!(by_content.lower_bound, (0.0, -610.0));
        assert_eq!(by_content.upper_bound, (0.0, 0.0));
        assert_eq!(by_content.clamped_offset, (0.0, -610.0));
        assert!(by_content.physics_present);
        assert!(by_content.physics_running);
    }

    #[test]
    fn typed_scroll_properties_hold_then_resolve_against_live_layout() {
        let (mut instance, constraint_local) = scroll_intent_fixture();
        let index_key = property_key_for_name("ScrollConstraint", "scrollIndex").unwrap();
        let percent_y_key = property_key_for_name("ScrollConstraint", "scrollPercentY").unwrap();
        let offset_y_key = property_key_for_name("ScrollConstraint", "scrollOffsetY").unwrap();

        assert!(instance.set_double_property(constraint_local, index_key, 2.0));
        assert_eq!(
            instance.double_property(constraint_local, index_key),
            Some(2.0)
        );
        assert_eq!(
            instance.double_property(constraint_local, offset_y_key),
            Some(0.0)
        );

        instance.update_pass();
        assert_eq!(
            instance.double_property(constraint_local, offset_y_key),
            Some(-220.0)
        );
        assert_eq!(
            instance.double_property(constraint_local, index_key),
            Some(2.0)
        );

        assert!(instance.set_double_property(constraint_local, index_key, 99.0));
        assert_eq!(
            instance.double_property(constraint_local, offset_y_key),
            Some(-610.0)
        );
        let resolved_index = instance
            .double_property(constraint_local, index_key)
            .expect("resolved index reads from the clamped offset");
        assert!((resolved_index - 5.545_454_5).abs() < 1.0e-5);

        assert!(instance.set_double_property(constraint_local, index_key, -5.0));
        assert_eq!(
            instance.double_property(constraint_local, offset_y_key),
            Some(0.0)
        );
        assert!(instance.set_double_property(constraint_local, percent_y_key, 0.5));
        assert_eq!(
            instance.double_property(constraint_local, offset_y_key),
            Some(-305.0)
        );
        assert_eq!(
            instance.double_property(constraint_local, percent_y_key),
            Some(0.5)
        );
    }

    #[test]
    fn index_intent_survives_a_hidden_layout_and_resolves_when_shown() {
        let (mut instance, constraint_local) = scroll_intent_fixture();
        let index_key = property_key_for_name("ScrollConstraint", "scrollIndex").unwrap();
        let offset_y_key = property_key_for_name("ScrollConstraint", "scrollOffsetY").unwrap();
        let display_key = property_key_for_name("LayoutComponentStyle", "displayValue").unwrap();

        instance.update_pass();
        assert!(instance.set_double_property(constraint_local, index_key, 2.0));
        assert_eq!(
            instance.double_property(constraint_local, offset_y_key),
            Some(-220.0)
        );

        assert!(instance.set_uint_property(2, display_key, 1));
        instance.update_pass();
        assert!(instance.set_double_property(constraint_local, index_key, 4.0));
        instance.update_pass();
        assert_eq!(
            instance.double_property(constraint_local, index_key),
            Some(4.0)
        );
        assert_eq!(
            instance.double_property(constraint_local, offset_y_key),
            Some(-220.0)
        );

        assert!(instance.set_uint_property(2, display_key, 0));
        instance.update_pass();
        assert_eq!(
            instance.double_property(constraint_local, offset_y_key),
            Some(-440.0)
        );
        assert_eq!(
            instance.double_property(constraint_local, index_key),
            Some(4.0)
        );
    }

    #[test]
    fn unresolved_intent_survives_a_direct_offset_write_until_layout_resolves() {
        let (mut instance, constraint_local) = scroll_intent_fixture();
        let index_key = property_key_for_name("ScrollConstraint", "scrollIndex").unwrap();
        let offset_y_key = property_key_for_name("ScrollConstraint", "scrollOffsetY").unwrap();
        let display_key = property_key_for_name("LayoutComponentStyle", "displayValue").unwrap();

        instance.update_pass();
        assert!(instance.set_uint_property(2, display_key, 1));
        instance.update_pass();
        assert!(instance.set_double_property(constraint_local, index_key, 4.0));
        assert!(instance.set_double_property(constraint_local, offset_y_key, -50.0));
        assert_eq!(
            instance.double_property(constraint_local, offset_y_key),
            Some(-50.0)
        );

        assert!(instance.set_uint_property(2, display_key, 0));
        instance.update_pass();
        assert_eq!(
            instance.double_property(constraint_local, offset_y_key),
            Some(-440.0)
        );
        assert_eq!(
            instance.double_property(constraint_local, index_key),
            Some(4.0)
        );
    }

    #[test]
    fn computed_scroll_setters_preserve_drag_and_stop_active_physics_otherwise() {
        let (mut instance, constraint_local) = scroll_intent_fixture();
        let index_key = property_key_for_name("ScrollConstraint", "scrollIndex").unwrap();
        let offset_y_key = property_key_for_name("ScrollConstraint", "scrollOffsetY").unwrap();
        instance.update_pass();
        let handle = instance
            .component_handle(constraint_local)
            .expect("ScrollConstraint handle");
        {
            let scroll = instance
                .objects
                .component_mut(handle)
                .and_then(|component| component.concrete.scroll.as_mut())
                .expect("retained ScrollConstraint");
            let mut physics = crate::components::RuntimeScrollPhysicsState::clamped();
            physics.is_running = true;
            physics.speed = (12.0, 34.0);
            scroll.physics = Some(physics);
            scroll.is_dragging = true;
        }

        assert!(instance.set_double_property(constraint_local, index_key, 3.0));
        assert_eq!(
            instance.double_property(constraint_local, offset_y_key),
            Some(0.0)
        );
        let scroll = instance
            .objects
            .component(handle)
            .and_then(|component| component.concrete.scroll.as_ref())
            .expect("retained ScrollConstraint");
        assert!(
            scroll
                .physics
                .as_ref()
                .is_some_and(|physics| physics.is_running)
        );
        assert_eq!(
            scroll.physics.as_ref().map(|physics| physics.speed),
            Some((12.0, 34.0))
        );

        instance
            .objects
            .component_mut(handle)
            .and_then(|component| component.concrete.scroll.as_mut())
            .expect("retained ScrollConstraint")
            .is_dragging = false;
        assert!(instance.set_double_property(constraint_local, index_key, 3.0));
        assert_eq!(
            instance.double_property(constraint_local, offset_y_key),
            Some(-330.0)
        );
        let physics = instance
            .objects
            .component(handle)
            .and_then(|component| component.concrete.scroll.as_ref())
            .and_then(|scroll| scroll.physics.as_ref())
            .expect("retained ScrollPhysics");
        assert!(!physics.is_running);
        assert_eq!(physics.speed, (0.0, 0.0));
    }

    #[test]
    fn percent_intent_reads_verbatim_until_layout_resolves() {
        let intent = RuntimeScrollAxisIntent {
            space: RuntimeScrollSpace::Percent,
            value: 0.5,
        };
        assert_eq!(intent.read(RuntimeScrollSpace::Percent), Some(0.5));
        assert_eq!(intent.resolve(RuntimeScrollAxis::Y, None), None);

        let metrics = RuntimeScrollLayoutMetrics::vertical_for_test(500.0, 1_100.0, 0.0, vec![]);
        assert_eq!(
            intent.resolve(RuntimeScrollAxis::Y, Some(&metrics)),
            Some(-300.0)
        );
    }

    #[test]
    fn index_intent_reads_verbatim_until_layout_resolves() {
        let intent = RuntimeScrollAxisIntent {
            space: RuntimeScrollSpace::Index,
            value: 2.0,
        };
        assert_eq!(intent.read(RuntimeScrollSpace::Index), Some(2.0));
        assert_eq!(intent.resolve(RuntimeScrollAxis::Y, None), None);

        let item_bounds = (0..10)
            .map(|index| RuntimeLayoutBounds {
                x: 0.0,
                y: index as f32 * 110.0,
                width: 500.0,
                height: 100.0,
            })
            .collect();
        let metrics =
            RuntimeScrollLayoutMetrics::vertical_for_test(500.0, 1_110.0, 10.0, item_bounds);
        assert_eq!(
            intent.resolve(RuntimeScrollAxis::Y, Some(&metrics)),
            Some(-220.0)
        );
    }

    #[test]
    fn hidden_layout_keeps_percent_and_index_intents_unresolved() {
        let bounds = RuntimeLayoutBounds {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let mut metrics =
            RuntimeScrollLayoutMetrics::vertical_for_test(500.0, 1_100.0, 10.0, vec![bounds]);
        metrics.viewport_layout_height = 0.0;

        for intent in [
            RuntimeScrollAxisIntent {
                space: RuntimeScrollSpace::Percent,
                value: 0.5,
            },
            RuntimeScrollAxisIntent {
                space: RuntimeScrollSpace::Index,
                value: 0.0,
            },
        ] {
            assert_eq!(intent.resolve(RuntimeScrollAxis::Y, Some(&metrics)), None);
            assert_eq!(intent.read(intent.space), Some(intent.value));
        }
    }

    #[test]
    fn finite_index_intents_clamp_to_the_scrollable_ends() {
        let item_bounds = (0..10)
            .map(|index| RuntimeLayoutBounds {
                x: 0.0,
                y: index as f32 * 110.0,
                width: 500.0,
                height: 100.0,
            })
            .collect();
        let metrics =
            RuntimeScrollLayoutMetrics::vertical_for_test(500.0, 1_110.0, 10.0, item_bounds);

        for (value, expected) in [(99.0, -610.0), (f32::INFINITY, -610.0), (-5.0, 0.0)] {
            let intent = RuntimeScrollAxisIntent {
                space: RuntimeScrollSpace::Index,
                value,
            };
            assert_eq!(
                intent.resolve(RuntimeScrollAxis::Y, Some(&metrics)),
                Some(expected)
            );
        }
        assert!((metrics.index_at_position((0.0, -610.0)) - 5.545_454_5).abs() < 1.0e-5);
    }

    #[test]
    fn infinite_index_intents_wrap_in_both_directions() {
        let item_bounds = (0..10)
            .map(|index| RuntimeLayoutBounds {
                x: 0.0,
                y: index as f32 * 110.0,
                width: 500.0,
                height: 100.0,
            })
            .collect();
        let mut metrics =
            RuntimeScrollLayoutMetrics::vertical_for_test(500.0, 1_100.0, 10.0, item_bounds);
        metrics.infinite = true;

        for (value, expected) in [(11.0, -110.0), (-1.0, -990.0), (f32::INFINITY, 0.0)] {
            let intent = RuntimeScrollAxisIntent {
                space: RuntimeScrollSpace::Index,
                value,
            };
            assert_eq!(
                intent.resolve(RuntimeScrollAxis::Y, Some(&metrics)),
                Some(expected)
            );
        }
    }

    #[test]
    fn two_dimensional_index_intent_resolves_both_axes() {
        let mut metrics = RuntimeScrollLayoutMetrics::vertical_for_test(
            50.0,
            200.0,
            10.0,
            vec![
                RuntimeLayoutBounds {
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 50.0,
                },
                RuntimeLayoutBounds {
                    x: 100.0,
                    y: 50.0,
                    width: 100.0,
                    height: 50.0,
                },
            ],
        );
        metrics.direction = 2;
        metrics.viewport_layout_width = 50.0;
        metrics.viewport_width = 50.0;
        metrics.content_width = 200.0;
        let intent = RuntimeScrollAxisIntent {
            space: RuntimeScrollSpace::Index,
            value: 1.0,
        };

        assert_eq!(
            intent.resolve(RuntimeScrollAxis::X, Some(&metrics)),
            Some(-100.0)
        );
        assert_eq!(
            intent.resolve(RuntimeScrollAxis::Y, Some(&metrics)),
            Some(-50.0)
        );
    }

    #[test]
    fn two_dimensional_index_writes_both_axes_and_direct_offsets_clear_per_axis() {
        assert_eq!(
            runtime_scroll_intent_axes(RuntimeScrollProperty::Index, 2),
            vec![
                (RuntimeScrollAxis::X, RuntimeScrollSpace::Index),
                (RuntimeScrollAxis::Y, RuntimeScrollSpace::Index),
            ]
        );

        let intent = RuntimeScrollAxisIntent {
            space: RuntimeScrollSpace::Index,
            value: 4.0,
        };
        let mut constraint = RuntimeScrollConstraintState {
            intent_x: Some(intent),
            intent_y: Some(intent),
            ..RuntimeScrollConstraintState::default()
        };
        assert!(constraint.clear_intent(RuntimeScrollAxis::X));
        assert_eq!(constraint.intent_x, None);
        assert_eq!(constraint.intent_y, Some(intent));
    }

    #[test]
    fn virtualizer_places_only_rows_intersecting_the_viewport() {
        let sizes = vec![(200.0, 50.0); 10];
        assert_eq!(
            test_virtualizer_placements_for_metrics(&sizes, false, -10.0, 100.0, 0.0, true,),
            vec![
                vertical_item(0, 0.0),
                vertical_item(1, 40.0),
                vertical_item(2, 80.0),
            ]
        );
    }

    #[test]
    fn virtualizer_preserves_wrapped_infinite_order_and_positions() {
        let sizes = vec![(200.0, 50.0); 10];
        assert_eq!(
            test_virtualizer_placements_for_metrics(&sizes, false, -10.0, 100.0, -360.0, true,),
            vec![
                vertical_item(8, -40.0),
                vertical_item(9, 0.0),
                vertical_item(0, 40.0),
                vertical_item(1, 80.0),
            ]
        );
    }

    #[test]
    fn virtualizer_does_not_wrap_a_finite_list() {
        let sizes = vec![(20.0, 30.0); 4];
        assert_eq!(
            test_virtualizer_placements_for_metrics(&sizes, true, 5.0, 40.0, -70.0, false,),
            vec![TestVirtualizerPlacement {
                logical_index: 3,
                position_x: 5.0,
                position_y: 0.0,
            }]
        );
    }

    #[test]
    fn virtualizer_flattens_provider_metrics_in_source_order() {
        let providers = vec![
            vec![(20.0, 10.0)],
            vec![(30.0, 10.0), (30.0, 10.0)],
            vec![(15.0, 10.0)],
            vec![(25.0, 10.0), (25.0, 10.0)],
        ];
        let content_size = virtualized_provider_content_size(&providers, true, 5.0, false);
        assert_eq!(content_size, 170.0);

        let placements = test_virtualizer_placements_for_providers(
            &providers,
            true,
            5.0,
            130.0,
            -25.0,
            false,
            content_size,
        );
        assert_eq!(
            placements[1],
            vec![
                TestVirtualizerPlacement {
                    logical_index: 0,
                    position_x: 0.0,
                    position_y: 0.0,
                },
                TestVirtualizerPlacement {
                    logical_index: 1,
                    position_x: 35.0,
                    position_y: 0.0,
                },
            ]
        );
        assert_eq!(
            placements[3],
            vec![TestVirtualizerPlacement {
                logical_index: 0,
                position_x: 90.0,
                position_y: 0.0,
            }],
            // Pinned C++ intentionally does not advance `childIndex` in the
            // visible-end while loop. With four providers this ends at global
            // index 4, before this provider's second item
            // (`scroll_virtualizer.cpp:107-153`).
        );
    }

    #[test]
    fn virtualizer_uses_content_origin_and_clamped_offset() {
        let viewport_size = scroll_viewport_axis_size(150.0, 20.0);
        assert_eq!(viewport_size, 130.0);
        assert_eq!(
            clamped_scroll_offset(-500.0, viewport_size, 170.0, 10.0, false),
            -50.0
        );
        assert_eq!(
            clamped_scroll_offset(25.0, viewport_size, 170.0, 10.0, false),
            0.0
        );
        assert_eq!(
            clamped_scroll_offset(-500.0, viewport_size, 170.0, 10.0, true),
            -500.0
        );
    }

    #[test]
    fn virtualizer_applies_size_feedback_to_later_rows_without_remounting() {
        let initial_sizes = vec![(100.0, 20.0); 3];
        let initial =
            test_virtualizer_placements_for_metrics(&initial_sizes, false, 5.0, 100.0, 0.0, false);
        assert_eq!(
            initial,
            vec![
                vertical_item(0, 0.0),
                vertical_item(1, 25.0),
                vertical_item(2, 50.0),
            ]
        );

        // The parent assigns row 0 a larger intrinsic height. The visible
        // topology stays [0, 1, 2], but C++'s same-pass
        // updateLayoutBounds/constrainVirtualized(true) feedback moves both
        // later rows before draw.
        let measured_sizes = vec![(100.0, 40.0), (100.0, 20.0), (100.0, 20.0)];
        let settled =
            test_virtualizer_placements_for_metrics(&measured_sizes, false, 5.0, 100.0, 0.0, false);
        assert_eq!(
            settled,
            vec![
                vertical_item(0, 0.0),
                vertical_item(1, 45.0),
                vertical_item(2, 70.0),
            ]
        );
        assert_eq!(
            initial
                .iter()
                .map(|item| item.logical_index)
                .collect::<Vec<_>>(),
            settled
                .iter()
                .map(|item| item.logical_index)
                .collect::<Vec<_>>()
        );
    }
}
