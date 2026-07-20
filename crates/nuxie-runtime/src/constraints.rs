use std::sync::OnceLock;

use nuxie_binary::RuntimeFile;
use nuxie_graph::{ArtboardGraph, PathGeometryNode};

use crate::components::TransformComponents;
use crate::draw::{RuntimeLayoutBounds, RuntimePathMeasure, runtime_path_geometry_commands};
use crate::properties::property_key_for_name;
use crate::text::static_text_constraint_bounds;
use crate::{ArtboardInstance, Mat2D};

#[derive(Debug, Clone)]
pub(crate) struct RuntimeFollowPathConstraint {
    local_id: usize,
    target_local: usize,
    target_kind: RuntimeFollowPathTargetKind,
    paths: Vec<RuntimeFollowPathPath>,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeListFollowPathConstraint {
    list_local: usize,
    constraint: RuntimeFollowPathConstraint,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeScrollConstraint {
    local_id: usize,
    content_local: usize,
    layout_child_locals: Vec<usize>,
    intent_x: Option<RuntimeScrollAxisIntent>,
    intent_y: Option<RuntimeScrollAxisIntent>,
    layout_initialized: bool,
}

impl RuntimeScrollConstraint {
    fn intent(&self, axis: RuntimeScrollAxis) -> Option<RuntimeScrollAxisIntent> {
        match axis {
            RuntimeScrollAxis::X => self.intent_x,
            RuntimeScrollAxis::Y => self.intent_y,
        }
    }

    fn set_intent(&mut self, axis: RuntimeScrollAxis, intent: Option<RuntimeScrollAxisIntent>) {
        match axis {
            RuntimeScrollAxis::X => self.intent_x = intent,
            RuntimeScrollAxis::Y => self.intent_y = intent,
        }
    }

    fn clear_intent(&mut self, axis: RuntimeScrollAxis) -> bool {
        let had_intent = self.intent(axis).is_some();
        self.set_intent(axis, None);
        had_intent
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeScrollAxis {
    X,
    Y,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeScrollSpace {
    Percent,
    Index,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeScrollProperty {
    Offset(RuntimeScrollAxis),
    Percent(RuntimeScrollAxis),
    Index,
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

#[derive(Debug, Clone, Copy, PartialEq)]
struct RuntimeScrollAxisIntent {
    space: RuntimeScrollSpace,
    value: f32,
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

#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeComponentListVirtualization {
    pub(crate) constraint_local: usize,
    pub(crate) content_local: usize,
    pub(crate) viewport_local: usize,
    pub(crate) scroll_offset_x: f32,
    pub(crate) scroll_offset_y: f32,
    pub(crate) infinite: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RuntimeComponentListVirtualItem {
    pub(crate) logical_index: usize,
    pub(crate) position_x: f32,
    pub(crate) position_y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeFollowPathTargetKind {
    Shape,
    Path,
    Other,
}

#[derive(Debug, Clone)]
struct RuntimeFollowPathPath {
    local_id: usize,
    geometry: PathGeometryNode,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeIkConstraint {
    local_id: usize,
    target_local: usize,
    chain: Vec<RuntimeIkChainLink>,
}

#[derive(Debug, Clone, Copy)]
struct RuntimeIkChainLink {
    bone_local: usize,
}

#[derive(Debug, Clone, Copy)]
struct IkChainState {
    bone_index: usize,
    parent_world_inverse: Mat2D,
    transform_components: TransformComponents,
    angle: f32,
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

pub(crate) fn build_runtime_follow_path_constraints(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
) -> Vec<RuntimeFollowPathConstraint> {
    graph
        .local_objects
        .iter()
        .filter(|object| object.type_name == Some("FollowPathConstraint"))
        .filter_map(|object| build_runtime_follow_path_constraint(file, graph, object.local_id))
        .collect()
}

pub(crate) fn build_runtime_list_follow_path_constraints(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
) -> Vec<RuntimeListFollowPathConstraint> {
    graph
        .list_constraint_registrations
        .iter()
        .filter(|registration| registration.constraint_type_name == "ListFollowPathConstraint")
        .filter_map(|registration| {
            Some(RuntimeListFollowPathConstraint {
                list_local: registration.constrainable_list_local,
                constraint: build_runtime_follow_path_constraint(
                    file,
                    graph,
                    registration.constraint_local,
                )?,
            })
        })
        .collect()
}

pub(crate) fn build_runtime_scroll_constraints(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
) -> Vec<RuntimeScrollConstraint> {
    graph
        .local_objects
        .iter()
        .filter(|object| object.type_name == Some("ScrollConstraint"))
        .filter_map(|object| {
            let constraint = file.object(object.global_id as usize)?;
            let content_local = usize::try_from(constraint.uint_property("parentId")?).ok()?;
            let layout_child_locals = graph
                .layout_constraint_registrations
                .iter()
                .filter(|registration| registration.constraint_local == object.local_id)
                .map(|registration| registration.layout_provider_local)
                .collect::<Vec<_>>();
            Some(RuntimeScrollConstraint {
                local_id: object.local_id,
                content_local,
                layout_child_locals,
                intent_x: None,
                intent_y: None,
                layout_initialized: false,
            })
        })
        .collect()
}

pub(crate) fn runtime_scroll_double_property(
    artboard: &ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<f32> {
    if artboard.scroll_constraints.is_empty() {
        return None;
    }
    let property = runtime_scroll_property(property_key)?;
    let constraint = artboard
        .scroll_constraints
        .iter()
        .find(|constraint| constraint.local_id == local_id)?;
    match property {
        RuntimeScrollProperty::Offset(_) => None,
        RuntimeScrollProperty::Percent(axis) => {
            if let Some(value) = constraint
                .intent(axis)
                .and_then(|intent| intent.read(RuntimeScrollSpace::Percent))
            {
                return Some(value);
            }
            let metrics = runtime_scroll_layout_metrics(artboard, constraint, false);
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
                runtime_scroll_layout_metrics(artboard, constraint, true)
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
    if artboard.scroll_constraints.is_empty() {
        return None;
    }
    let property = runtime_scroll_property(property_key)?;
    if matches!(property, RuntimeScrollProperty::Offset(_)) {
        return None;
    }
    let constraint_index = artboard
        .scroll_constraints
        .iter()
        .position(|constraint| constraint.local_id == local_id)?;
    if runtime_scroll_double_property(artboard, local_id, property_key) == Some(value) {
        return Some(false);
    }

    let constraint = artboard.scroll_constraints[constraint_index].clone();
    let metrics = if constraint.layout_initialized {
        runtime_scroll_layout_metrics(
            artboard,
            &constraint,
            matches!(property, RuntimeScrollProperty::Index),
        )
    } else {
        Some(build_runtime_scroll_layout_metrics(
            artboard,
            &constraint,
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
    let axes = runtime_scroll_intent_axes(property, direction);

    for (axis, space) in axes {
        let intent = RuntimeScrollAxisIntent { space, value };
        let resolved = intent.resolve(axis, metrics.as_ref());
        artboard.scroll_constraints[constraint_index].set_intent(
            axis,
            if resolved.is_some() {
                None
            } else {
                Some(intent)
            },
        );
        if let Some(offset) = resolved
            && let Some(offset_key) = scroll_offset_property_key(axis)
        {
            artboard.set_double_property(local_id, offset_key, offset);
        }
    }
    Some(true)
}

pub(crate) fn clear_runtime_scroll_intent_for_direct_offset(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> bool {
    if artboard.scroll_constraints.is_empty() {
        return false;
    }
    let Some(RuntimeScrollProperty::Offset(axis)) = runtime_scroll_property(property_key) else {
        return false;
    };
    let Some(constraint) = artboard
        .scroll_constraints
        .iter_mut()
        .find(|constraint| constraint.local_id == local_id)
    else {
        return false;
    };
    constraint.clear_intent(axis)
}

fn raw_scroll_offset(artboard: &ArtboardInstance, local_id: usize, axis: RuntimeScrollAxis) -> f32 {
    scroll_offset_property_key(axis)
        .and_then(|key| artboard.double_property(local_id, key))
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
    let Some(constraint_index) = artboard
        .scroll_constraints
        .iter()
        .position(|constraint| constraint.local_id == constraint_local)
    else {
        return false;
    };
    let intents = [
        (
            RuntimeScrollAxis::X,
            artboard.scroll_constraints[constraint_index].intent_x,
        ),
        (
            RuntimeScrollAxis::Y,
            artboard.scroll_constraints[constraint_index].intent_y,
        ),
    ];
    let mut changed = false;
    for (axis, intent) in intents {
        let Some(intent) = intent else {
            continue;
        };
        let Some(offset) = intent.resolve(axis, Some(metrics)) else {
            continue;
        };
        artboard.scroll_constraints[constraint_index].set_intent(axis, None);
        if let Some(offset_key) = scroll_offset_property_key(axis) {
            changed |= artboard.set_double_property(constraint_local, offset_key, offset);
        }
    }
    changed
}

fn build_runtime_follow_path_constraint(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    constraint_local: usize,
) -> Option<RuntimeFollowPathConstraint> {
    let object = graph
        .local_objects
        .iter()
        .find(|object| object.local_id == constraint_local)?;
    if !matches!(
        object.type_name,
        Some("FollowPathConstraint" | "ListFollowPathConstraint")
    ) {
        return None;
    }

    let constraint = file.object(object.global_id as usize)?;
    let target_local = usize::try_from(constraint.uint_property("targetId")?).ok()?;
    let target_type = graph
        .local_objects
        .iter()
        .find(|object| object.local_id == target_local)
        .and_then(|object| object.type_name);

    let target_kind = if target_type == Some("Shape") {
        RuntimeFollowPathTargetKind::Shape
    } else if graph.paths.iter().any(|path| path.local_id == target_local) {
        RuntimeFollowPathTargetKind::Path
    } else {
        RuntimeFollowPathTargetKind::Other
    };

    let paths = match target_kind {
        RuntimeFollowPathTargetKind::Shape => graph
            .path_composers
            .iter()
            .find(|composer| composer.shape_local == target_local)
            .map(|composer| {
                composer
                    .paths
                    .iter()
                    .filter_map(|path_ref| {
                        graph
                            .paths
                            .iter()
                            .find(|path| path.local_id == path_ref.local_id)
                            .cloned()
                            .map(|geometry| RuntimeFollowPathPath {
                                local_id: path_ref.local_id,
                                geometry,
                            })
                    })
                    .collect()
            })
            .unwrap_or_default(),
        RuntimeFollowPathTargetKind::Path => graph
            .paths
            .iter()
            .find(|path| path.local_id == target_local)
            .cloned()
            .map(|geometry| {
                vec![RuntimeFollowPathPath {
                    local_id: target_local,
                    geometry,
                }]
            })
            .unwrap_or_default(),
        RuntimeFollowPathTargetKind::Other => Vec::new(),
    };

    Some(RuntimeFollowPathConstraint {
        local_id: constraint_local,
        target_local,
        target_kind,
        paths,
    })
}

pub(crate) fn build_runtime_ik_constraints(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
) -> Vec<RuntimeIkConstraint> {
    graph
        .local_objects
        .iter()
        .filter(|object| object.type_name == Some("IKConstraint"))
        .filter_map(|object| {
            let constraint = file.object(object.global_id as usize)?;
            let tip_local = usize::try_from(constraint.uint_property("parentId")?).ok()?;
            if !local_object_type(graph, tip_local).is_some_and(is_bone_type) {
                return None;
            }

            let target_local = usize::try_from(constraint.uint_property("targetId")?).ok()?;
            let mut reverse_chain = vec![tip_local];
            let mut current_local = tip_local;
            let mut remaining = constraint.uint_property("parentBoneCount").unwrap_or(0);
            while remaining > 0 {
                let Some(parent_local) = local_object_parent(file, graph, current_local) else {
                    break;
                };
                if !local_object_type(graph, parent_local).is_some_and(is_bone_type) {
                    break;
                }
                remaining -= 1;
                reverse_chain.push(parent_local);
                current_local = parent_local;
            }
            reverse_chain.reverse();

            Some(RuntimeIkConstraint {
                local_id: object.local_id,
                target_local,
                chain: reverse_chain
                    .into_iter()
                    .map(|bone_local| RuntimeIkChainLink { bone_local })
                    .collect(),
            })
        })
        .collect()
}

/// Runtime constraint application for the C++ `src/constraints/` path.
pub(crate) fn apply_constraints(artboard: &mut ArtboardInstance, component_index: usize) -> bool {
    let constraint_locals = artboard.components[component_index]
        .constraint_locals
        .clone();
    constraint_locals
        .into_iter()
        .fold(false, |changed, constraint_local| {
            changed | apply_constraint(artboard, component_index, constraint_local)
        })
}

pub(crate) fn apply_list_constraints(
    artboard: &mut ArtboardInstance,
    component_index: usize,
) -> bool {
    if artboard.components[component_index].type_name != "ArtboardComponentList" {
        return false;
    }

    let list_local = artboard.components[component_index].local_id;
    let Some(mut item_transforms) = artboard.component_list_item_transforms.remove(&list_local)
    else {
        return false;
    };
    let changed = constrain_component_list_item_transforms(
        artboard,
        list_local,
        component_index,
        &mut item_transforms,
    );
    artboard
        .component_list_item_transforms
        .insert(list_local, item_transforms);
    changed
}

/// Apply list constraints after the hosting layout has assigned each mounted
/// artboard its base transform. This mirrors C++
/// `ArtboardComponentList::updateArtboardsWorldTransform` followed by
/// `ArtboardComponentList::updateConstraints`.
pub(crate) fn constrain_component_list_item_transforms(
    artboard: &ArtboardInstance,
    list_local: usize,
    list_component_index: usize,
    item_transforms: &mut [Mat2D],
) -> bool {
    // C++ explicitly skips list constraints while the component list is
    // virtualized. The scroll virtualizer owns row positions in that mode.
    if component_list_virtualization(artboard, list_local).is_some() {
        return false;
    }

    artboard
        .list_follow_path_constraints
        .iter()
        .filter(|constraint| constraint.list_local == list_local)
        .fold(false, |changed, constraint| {
            changed
                | apply_list_follow_path_constraint_to_transforms(
                    artboard,
                    list_component_index,
                    &constraint.constraint,
                    item_transforms,
                )
        })
}

/// Resolve the live C++ `ArtboardComponentList::virtualizationEnabled`
/// relationship for one list. A `ScrollConstraint` can be animated or data
/// bound, so read its current instance properties instead of caching flags at
/// import time.
pub(crate) fn component_list_virtualization(
    artboard: &ArtboardInstance,
    list_local: usize,
) -> Option<RuntimeComponentListVirtualization> {
    let constraint = artboard.scroll_constraints.iter().find(|constraint| {
        constraint.layout_child_locals.contains(&list_local)
            && constraint_bool(
                artboard,
                constraint.local_id,
                "ScrollConstraint",
                "virtualize",
                false,
            )
    })?;
    let viewport_local = artboard.component(constraint.content_local)?.parent_local?;
    Some(RuntimeComponentListVirtualization {
        constraint_local: constraint.local_id,
        content_local: constraint.content_local,
        viewport_local,
        scroll_offset_x: constraint_double(
            artboard,
            constraint.local_id,
            "ScrollConstraint",
            "scrollOffsetX",
            0.0,
        ),
        scroll_offset_y: constraint_double(
            artboard,
            constraint.local_id,
            "ScrollConstraint",
            "scrollOffsetY",
            0.0,
        ),
        infinite: constraint_bool(
            artboard,
            constraint.local_id,
            "ScrollConstraint",
            "infinite",
            false,
        ),
    })
}

fn runtime_scroll_layout_metrics(
    artboard: &ArtboardInstance,
    constraint: &RuntimeScrollConstraint,
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
        constraint,
        layout_bounds,
        include_item_bounds,
    ))
}

fn build_runtime_scroll_layout_metrics(
    artboard: &ArtboardInstance,
    constraint: &RuntimeScrollConstraint,
    layout_bounds: Option<&std::collections::BTreeMap<usize, RuntimeLayoutBounds>>,
    include_item_bounds: bool,
) -> RuntimeScrollLayoutMetrics {
    let direction = constraint_uint(
        artboard,
        constraint.local_id,
        "DraggableConstraint",
        "directionValue",
        1,
    );
    let infinite = constraint_bool(
        artboard,
        constraint.local_id,
        "ScrollConstraint",
        "infinite",
        false,
    );
    let virtualize = constraint_bool(
        artboard,
        constraint.local_id,
        "ScrollConstraint",
        "virtualize",
        false,
    );
    let content_style_local = layout_component_style_local(artboard, constraint.content_local);
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
        .component(constraint.content_local)
        .and_then(|component| component.parent_local);
    let viewport_bounds = viewport_local.and_then(|local| layout_bounds?.get(&local).copied());
    let content_bounds = layout_bounds
        .and_then(|bounds| bounds.get(&constraint.content_local))
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
    constraint: &RuntimeScrollConstraint,
    layout_bounds: Option<&std::collections::BTreeMap<usize, RuntimeLayoutBounds>>,
    virtualize: bool,
    main_axis_is_horizontal: bool,
    gap_x: f32,
    gap_y: f32,
    content_bounds: Option<RuntimeLayoutBounds>,
) -> Vec<RuntimeLayoutBounds> {
    let has_component_list = constraint.layout_child_locals.iter().any(|local| {
        artboard
            .component(*local)
            .is_some_and(|component| component.type_name == "ArtboardComponentList")
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
    for provider_local in &constraint.layout_child_locals {
        let is_component_list = artboard
            .component(*provider_local)
            .is_some_and(|component| component.type_name == "ArtboardComponentList");
        if !is_component_list {
            if let Some(mut bounds) =
                layout_bounds.and_then(|bounds| bounds.get(provider_local).copied())
            {
                bounds.x -= content_origin.0;
                bounds.y -= content_origin.1;
                flat_bounds.push(bounds);
            }
            continue;
        }

        if !virtualize
            && let Some(bounds) = assigned_list_bounds.get(provider_local)
            && !bounds.is_empty()
        {
            flat_bounds.extend(bounds.iter().copied());
            continue;
        }
        let sizes = artboard
            .component_list_logical_items
            .get(provider_local)
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

/// Resolve the mounted logical rows and their virtualizer-owned positions for
/// one component list. C++ keeps every list item and its authored size in
/// `m_listItems` / `m_artboardSizes`, but only creates `ArtboardInstance`s for
/// the window selected by `ScrollVirtualizer`.
pub(crate) fn component_list_virtual_window(
    artboard: &ArtboardInstance,
    list_local: usize,
    item_sizes: &[(f32, f32)],
) -> Option<Vec<RuntimeComponentListVirtualItem>> {
    let virtualization = component_list_virtualization(artboard, list_local)?;
    let constraint = artboard
        .scroll_constraints
        .iter()
        .find(|constraint| constraint.local_id == virtualization.constraint_local)?;
    let style_local = layout_component_style_local(artboard, virtualization.content_local);
    let is_horizontal = style_local
        .and_then(|style_local| {
            property_key_for_name("LayoutComponentStyle", "flexDirectionValue")
                .and_then(|key| artboard.uint_property(style_local, key))
        })
        .map(|direction| matches!(direction, 2 | 3))
        .unwrap_or(true);
    let gap_property = if is_horizontal {
        "gapHorizontal"
    } else {
        "gapVertical"
    };
    let gap = style_local
        .and_then(|style_local| {
            property_key_for_name("LayoutComponentStyle", gap_property)
                .and_then(|key| artboard.double_property(style_local, key))
        })
        .unwrap_or(0.0);
    let computed_layout_bounds = artboard
        .runtime_graph()
        .and_then(|graph| artboard.runtime_taffy_layout_bounds(graph, artboard.runtime_file()));
    let layout_bounds = artboard
        .layout_constraint_bounds
        .as_deref()
        .or(computed_layout_bounds.as_ref());
    let viewport_axis_size = layout_component_axis_size(
        artboard,
        layout_bounds,
        virtualization.viewport_local,
        is_horizontal,
    );
    let direction = constraint_uint(
        artboard,
        virtualization.constraint_local,
        "DraggableConstraint",
        "directionValue",
        1,
    );
    let subtracts_content_origin = if is_horizontal {
        direction != 1
    } else {
        direction != 0
    };
    let content_axis_origin = if subtracts_content_origin {
        layout_component_axis_origin(layout_bounds, virtualization.content_local, is_horizontal)
            - layout_component_axis_origin(
                layout_bounds,
                virtualization.viewport_local,
                is_horizontal,
            )
    } else {
        0.0
    };
    // C++ `ScrollConstraint::viewportWidth/Height` removes the content's
    // layout origin on the constrained axis. This is observable when the
    // content has leading padding/alignment inside the viewport.
    let viewport_size = scroll_viewport_axis_size(viewport_axis_size, content_axis_origin);
    let provider_item_sizes = virtualized_provider_item_sizes(
        artboard,
        layout_bounds,
        constraint,
        Some((list_local, item_sizes)),
    );
    let content_size = virtualized_provider_content_size(
        &provider_item_sizes,
        is_horizontal,
        gap,
        virtualization.infinite,
    );
    let raw_scroll_offset = if is_horizontal {
        virtualization.scroll_offset_x
    } else {
        virtualization.scroll_offset_y
    };
    let trailing_padding = layout_style_axis_trailing_padding(
        artboard,
        layout_component_style_local(artboard, virtualization.viewport_local),
        is_horizontal,
    );
    let scroll_offset = clamped_scroll_offset(
        raw_scroll_offset,
        viewport_size,
        content_size,
        trailing_padding,
        virtualization.infinite,
    );

    let provider_index = constraint
        .layout_child_locals
        .iter()
        .position(|local| *local == list_local)?;
    Some(
        component_list_virtual_windows_for_provider_metrics(
            &provider_item_sizes,
            is_horizontal,
            gap,
            viewport_size,
            scroll_offset,
            virtualization.infinite,
            content_size,
        )
        .into_iter()
        .nth(provider_index)
        .unwrap_or_default(),
    )
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
    if artboard
        .component(layout_local)
        .is_some_and(|component| component.parent_local == Some(0))
    {
        if horizontal {
            artboard.width
        } else {
            artboard.height
        }
    } else {
        0.0
    }
}

fn layout_component_axis_origin(
    layout_bounds: Option<&std::collections::BTreeMap<usize, crate::draw::RuntimeLayoutBounds>>,
    layout_local: usize,
    horizontal: bool,
) -> f32 {
    layout_bounds
        .and_then(|bounds| bounds.get(&layout_local))
        .map(|bounds| if horizontal { bounds.x } else { bounds.y })
        .filter(|origin| origin.is_finite())
        .unwrap_or(0.0)
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

fn virtualized_provider_item_sizes(
    artboard: &ArtboardInstance,
    layout_bounds: Option<&std::collections::BTreeMap<usize, crate::draw::RuntimeLayoutBounds>>,
    constraint: &RuntimeScrollConstraint,
    current_list: Option<(usize, &[(f32, f32)])>,
) -> Vec<Vec<(f32, f32)>> {
    constraint
        .layout_child_locals
        .iter()
        .map(|provider_local| {
            if artboard
                .component(*provider_local)
                .is_some_and(|component| component.type_name == "ArtboardComponentList")
            {
                if current_list.is_some_and(|(list_local, _)| *provider_local == list_local) {
                    current_list
                        .map(|(_, item_sizes)| item_sizes.to_vec())
                        .unwrap_or_default()
                } else {
                    artboard
                        .component_list_logical_items
                        .get(provider_local)
                        .map(|items| items.iter().map(|item| item.size).collect())
                        .unwrap_or_default()
                }
            } else {
                vec![(
                    layout_component_axis_size(artboard, layout_bounds, *provider_local, true),
                    layout_component_axis_size(artboard, layout_bounds, *provider_local, false),
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
fn component_list_virtual_window_for_metrics(
    item_sizes: &[(f32, f32)],
    is_horizontal: bool,
    gap: f32,
    viewport_size: f32,
    scroll_offset: f32,
    infinite: bool,
) -> Vec<RuntimeComponentListVirtualItem> {
    component_list_virtual_windows_for_provider_metrics(
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

fn component_list_virtual_windows_for_provider_metrics(
    provider_item_sizes: &[Vec<(f32, f32)>],
    is_horizontal: bool,
    gap: f32,
    viewport_size: f32,
    scroll_offset: f32,
    infinite: bool,
    content_size: f32,
) -> Vec<Vec<RuntimeComponentListVirtualItem>> {
    let mut provider_windows = vec![Vec::new(); provider_item_sizes.len()];
    let flat_items = provider_item_sizes
        .iter()
        .enumerate()
        .flat_map(|(provider_index, items)| {
            items
                .iter()
                .copied()
                .enumerate()
                .map(move |(logical_index, size)| (provider_index, logical_index, size))
        })
        .collect::<Vec<_>>();
    let count = flat_items.len();
    if count == 0 || !viewport_size.is_finite() || viewport_size <= 0.0 {
        return provider_windows;
    }

    let axis_size = |index: usize| {
        let size = if is_horizontal {
            flat_items[index].2.0
        } else {
            flat_items[index].2.1
        };
        if size.is_finite() { size.max(0.0) } else { 0.0 }
    };
    if !content_size.is_finite() || content_size <= 0.0 {
        return provider_windows;
    }

    // Ported from `ScrollVirtualizer::constrain`: the virtualizer consumes the
    // clamped scroll transform (negative while moving forward) and normalizes
    // an infinite carousel into one logical content cycle.
    let normalized_offset = -scroll_offset;
    let offset = if scroll_offset > 0.0 {
        if infinite {
            let multiplier = (scroll_offset / content_size).floor() + 1.0;
            -(scroll_offset - multiplier * content_size)
        } else {
            -scroll_offset
        }
    } else {
        let multiplier = (normalized_offset / content_size).floor();
        if multiplier > 0.0 {
            normalized_offset % (multiplier * content_size)
        } else {
            normalized_offset
        }
    };

    let mut running_size = 0.0;
    let mut running_offset = 0.0;
    let mut visible_start = 0usize;
    let mut found_start = false;
    for index in 0..count {
        let size = axis_size(index);
        if running_size + size > offset {
            running_offset = running_size - offset;
            visible_start = index;
            found_start = true;
            break;
        }
        running_size += size;
        let next_index = if index + 1 == count { 0 } else { index + 1 };
        if running_size + gap > offset {
            running_size += gap;
            running_offset = running_size - offset;
            visible_start = next_index;
            found_start = true;
            break;
        }
        running_size += gap;
    }
    if !found_start {
        return provider_windows;
    }

    let max_virtual_rows = if infinite {
        count.saturating_mul(2)
    } else {
        count.saturating_sub(visible_start)
    };
    let mut mounted = Vec::new();
    let mut virtual_index = visible_start;
    for _ in 0..max_virtual_rows {
        let logical_index = virtual_index % count;
        let size = axis_size(logical_index);
        let (provider_index, logical_index, _) = flat_items[logical_index];
        mounted.push((
            provider_index,
            RuntimeComponentListVirtualItem {
                logical_index,
                position_x: if is_horizontal { running_offset } else { 0.0 },
                position_y: if is_horizontal { 0.0 } else { running_offset },
            },
        ));
        if running_size + size + gap >= offset + viewport_size {
            break;
        }
        running_size += size + gap;
        running_offset += size + gap;
        virtual_index += 1;
    }

    // C++ owns one mounted artboard per list-item pointer. An oversized
    // infinite viewport may encounter an item twice; the later virtual
    // occurrence updates the same mounted artboard's position.
    for (provider_index, item) in mounted {
        let provider_window = &mut provider_windows[provider_index];
        if let Some(existing) = provider_window
            .iter_mut()
            .find(|existing| existing.logical_index == item.logical_index)
        {
            *existing = item;
        } else {
            provider_window.push(item);
        }
    }
    provider_windows
}

fn apply_constraint(
    artboard: &mut ArtboardInstance,
    component_index: usize,
    constraint_local: usize,
) -> bool {
    let Some(slot) = artboard.slot(constraint_local) else {
        return false;
    };
    match slot.type_name {
        Some("DistanceConstraint") => {
            apply_distance_constraint(artboard, component_index, constraint_local)
        }
        Some("TranslationConstraint") => {
            apply_translation_constraint(artboard, component_index, constraint_local)
        }
        Some("RotationConstraint") => {
            apply_rotation_constraint(artboard, component_index, constraint_local)
        }
        Some("ScaleConstraint") => {
            apply_scale_constraint(artboard, component_index, constraint_local)
        }
        Some("TransformConstraint") => {
            apply_transform_constraint(artboard, component_index, constraint_local)
        }
        Some("FollowPathConstraint") => {
            apply_follow_path_constraint(artboard, component_index, constraint_local)
        }
        Some("ScrollConstraint") => {
            apply_scroll_constraint(artboard, component_index, constraint_local)
        }
        Some("IKConstraint") => apply_ik_constraint(artboard, component_index, constraint_local),
        _ => false,
    }
}

fn apply_scroll_constraint(
    artboard: &mut ArtboardInstance,
    component_index: usize,
    constraint_local: usize,
) -> bool {
    // Ported from C++ `src/constraints/scrolling/scroll_constraint.cpp`
    // `ScrollConstraint::constrain` / `constrainChild`.
    let Some(scroll_constraint) = artboard
        .scroll_constraints
        .iter()
        .find(|constraint| constraint.local_id == constraint_local)
        .cloned()
    else {
        return false;
    };
    if artboard.components[component_index].local_id != scroll_constraint.content_local {
        return false;
    }

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
    let content_style_local =
        layout_component_style_local(artboard, scroll_constraint.content_local);
    let main_axis_is_horizontal = content_style_local
        .and_then(|style_local| {
            property_key_for_name("LayoutComponentStyle", "flexDirectionValue")
                .and_then(|key| artboard.uint_property(style_local, key))
        })
        .map(|value| matches!(value, 2 | 3))
        .unwrap_or(true);
    let gap_horizontal = content_style_local
        .and_then(|style_local| {
            property_key_for_name("LayoutComponentStyle", "gapHorizontal")
                .and_then(|key| artboard.double_property(style_local, key))
        })
        .unwrap_or(0.0);
    let gap_vertical = content_style_local
        .and_then(|style_local| {
            property_key_for_name("LayoutComponentStyle", "gapVertical")
                .and_then(|key| artboard.double_property(style_local, key))
        })
        .unwrap_or(0.0);
    let computed_layout_bounds = artboard
        .runtime_graph()
        .and_then(|graph| artboard.runtime_taffy_layout_bounds(graph, artboard.runtime_file()));
    let retained_layout_bounds = artboard.layout_constraint_bounds.clone();
    let layout_bounds = retained_layout_bounds
        .as_deref()
        .or(computed_layout_bounds.as_ref());
    if let Some(constraint) = artboard
        .scroll_constraints
        .iter_mut()
        .find(|constraint| constraint.local_id == constraint_local)
    {
        constraint.layout_initialized = true;
    }
    let intent_changed =
        if scroll_constraint.intent_x.is_some() || scroll_constraint.intent_y.is_some() {
            let include_item_bounds = scroll_constraint
                .intent_x
                .into_iter()
                .chain(scroll_constraint.intent_y)
                .any(|intent| intent.space == RuntimeScrollSpace::Index);
            let scroll_metrics = build_runtime_scroll_layout_metrics(
                artboard,
                &scroll_constraint,
                layout_bounds,
                include_item_bounds,
            );
            resolve_runtime_scroll_intents(artboard, constraint_local, &scroll_metrics)
        } else {
            false
        };
    let provider_item_sizes =
        virtualized_provider_item_sizes(artboard, layout_bounds, &scroll_constraint, None);
    let viewport_local = artboard
        .component(scroll_constraint.content_local)
        .and_then(|component| component.parent_local);
    let clamped_axis_offset = |horizontal: bool, raw_offset: f32| {
        let Some(viewport_local) = viewport_local else {
            return raw_offset;
        };
        let viewport_axis_size =
            layout_component_axis_size(artboard, layout_bounds, viewport_local, horizontal);
        // C++ subtracts the content origin only on axes the constraint can
        // actually drag (`viewportWidth/Height`).
        let subtracts_content_origin = if horizontal {
            direction != 1
        } else {
            direction != 0
        };
        let content_axis_origin = if subtracts_content_origin {
            layout_component_axis_origin(layout_bounds, scroll_constraint.content_local, horizontal)
                - layout_component_axis_origin(layout_bounds, viewport_local, horizontal)
        } else {
            0.0
        };
        let viewport_size = scroll_viewport_axis_size(viewport_axis_size, content_axis_origin);
        let content_size = if virtualize && horizontal == main_axis_is_horizontal {
            virtualized_provider_content_size(
                &provider_item_sizes,
                horizontal,
                if horizontal {
                    gap_horizontal
                } else {
                    gap_vertical
                },
                infinite,
            )
        } else {
            layout_component_axis_size(
                artboard,
                layout_bounds,
                scroll_constraint.content_local,
                horizontal,
            )
        };
        let trailing_padding = layout_style_axis_trailing_padding(
            artboard,
            layout_component_style_local(artboard, viewport_local),
            horizontal,
        );
        clamped_scroll_offset(
            raw_offset,
            viewport_size,
            content_size,
            trailing_padding,
            infinite,
        )
    };
    let offset_x = if matches!(direction, 0 | 2) {
        clamped_axis_offset(
            true,
            constraint_double(
                artboard,
                constraint_local,
                "ScrollConstraint",
                "scrollOffsetX",
                0.0,
            ),
        )
    } else {
        0.0
    };
    let offset_y = if matches!(direction, 1 | 2) {
        clamped_axis_offset(
            false,
            constraint_double(
                artboard,
                constraint_local,
                "ScrollConstraint",
                "scrollOffsetY",
                0.0,
            ),
        )
    } else {
        0.0
    };
    let scroll_transform = Mat2D([1.0, 0.0, 0.0, 1.0, offset_x, offset_y]);
    let strength = constraint_double(artboard, constraint_local, "Constraint", "strength", 1.0);

    let mut changed = intent_changed;
    for child_local in scroll_constraint.layout_child_locals {
        let Some(child_index) = artboard.component_by_local.get(&child_local).copied() else {
            continue;
        };
        let current = artboard.components[child_index].transform.world_transform;
        let target = current.multiply(scroll_transform);
        changed |= constrain_world(artboard, child_index, current, target, strength);
    }
    changed
}

fn apply_distance_constraint(
    artboard: &mut ArtboardInstance,
    component_index: usize,
    constraint_local: usize,
) -> bool {
    // Ported from C++ `src/constraints/distance_constraint.cpp`.
    let Some(slot) = artboard.slot(constraint_local) else {
        return false;
    };
    if slot.type_name != Some("DistanceConstraint") {
        return false;
    }

    let Some(target_local) = targeted_constraint_target_local(artboard, constraint_local) else {
        return false;
    };
    if artboard
        .component(target_local)
        .is_some_and(|target| target.is_collapsed())
    {
        return false;
    }

    let Some(target_index) = artboard.component_by_local.get(&target_local).copied() else {
        return false;
    };
    let target_transform = artboard.components[target_index].transform.world_transform;
    let target_x = target_transform.0[4];
    let target_y = target_transform.0[5];

    let world = artboard.components[component_index]
        .transform
        .world_transform;
    let our_x = world.0[4];
    let our_y = world.0[5];
    let to_target_x = our_x - target_x;
    let to_target_y = our_y - target_y;
    let current_distance = to_target_x.hypot(to_target_y);
    let distance = constraint_double(
        artboard,
        constraint_local,
        "DistanceConstraint",
        "distance",
        100.0,
    );

    match constraint_uint(
        artboard,
        constraint_local,
        "DistanceConstraint",
        "modeValue",
        0,
    ) {
        0 if current_distance < distance => return false,
        1 if current_distance > distance => return false,
        _ => {}
    }
    if current_distance < 0.001 {
        return false;
    }

    let scale = distance / current_distance;
    let constrained_x = target_x + to_target_x * scale;
    let constrained_y = target_y + to_target_y * scale;
    let strength = constraint_double(artboard, constraint_local, "Constraint", "strength", 1.0);
    let new_x = our_x + (constrained_x - our_x) * strength;
    let new_y = our_y + (constrained_y - our_y) * strength;

    let world = &mut artboard.components[component_index]
        .transform
        .world_transform
        .0;
    if world[4] == new_x && world[5] == new_y {
        return false;
    }
    world[4] = new_x;
    world[5] = new_y;
    true
}

fn apply_rotation_constraint(
    artboard: &mut ArtboardInstance,
    component_index: usize,
    constraint_local: usize,
) -> bool {
    // Ported from C++ `src/constraints/rotation_constraint.cpp`.
    let target_index = targeted_constraint_target_local(artboard, constraint_local)
        .and_then(|target_local| artboard.component_by_local.get(&target_local).copied());
    if target_index.is_some_and(|index| artboard.components[index].is_collapsed()) {
        return false;
    }

    let transform_a = artboard.components[component_index]
        .transform
        .world_transform;
    let components_a = transform_a.decompose();
    let mut components_b = components_a;

    if let Some(target_index) = target_index {
        let mut transform_b = artboard.components[target_index].transform.world_transform;
        if transform_space(
            artboard,
            constraint_local,
            "TransformSpaceConstraint",
            "sourceSpaceValue",
        ) == TransformSpace::Local
        {
            let Some(inverse) = invert(parent_world_transform(artboard, target_index)) else {
                return false;
            };
            transform_b = inverse.multiply(transform_b);
        }

        components_b = transform_b.decompose();
        let dest_space = transform_space(
            artboard,
            constraint_local,
            "TransformSpaceConstraint",
            "destSpaceValue",
        );
        if !constraint_bool(
            artboard,
            constraint_local,
            "TransformComponentConstraint",
            "doesCopy",
            true,
        ) {
            components_b.rotation = if dest_space == TransformSpace::Local {
                0.0
            } else {
                components_a.rotation
            };
        } else {
            components_b.rotation *= constraint_double(
                artboard,
                constraint_local,
                "TransformComponentConstraint",
                "copyFactor",
                1.0,
            );
            if constraint_bool(
                artboard,
                constraint_local,
                "TransformComponentConstraint",
                "offset",
                false,
            ) {
                let authored =
                    artboard.authored_transform(artboard.components[component_index].local_id);
                components_b.rotation += authored.rotation;
            }
        }

        if dest_space == TransformSpace::Local {
            transform_b = parent_world_transform(artboard, component_index)
                .multiply(Mat2D::compose(components_b));
            components_b = transform_b.decompose();
        }
    }

    let clamp_local = transform_space(
        artboard,
        constraint_local,
        "TransformComponentConstraint",
        "minMaxSpaceValue",
    ) == TransformSpace::Local;
    if clamp_local {
        let transform_b = Mat2D::compose(components_b);
        let Some(inverse) = invert(parent_world_transform(artboard, component_index)) else {
            return false;
        };
        components_b = inverse.multiply(transform_b).decompose();
    }
    if constraint_bool(
        artboard,
        constraint_local,
        "TransformComponentConstraint",
        "max",
        false,
    ) && components_b.rotation
        > constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraint",
            "maxValue",
            0.0,
        )
    {
        components_b.rotation = constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraint",
            "maxValue",
            0.0,
        );
    }
    if constraint_bool(
        artboard,
        constraint_local,
        "TransformComponentConstraint",
        "min",
        false,
    ) && components_b.rotation
        < constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraint",
            "minValue",
            0.0,
        )
    {
        components_b.rotation = constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraint",
            "minValue",
            0.0,
        );
    }
    if clamp_local {
        let transform_b = parent_world_transform(artboard, component_index)
            .multiply(Mat2D::compose(components_b));
        components_b = transform_b.decompose();
    }

    components_b.rotation = interpolated_rotation(
        components_a.rotation,
        components_b.rotation,
        constraint_double(artboard, constraint_local, "Constraint", "strength", 1.0),
    );
    components_b.x = components_a.x;
    components_b.y = components_a.y;
    components_b.scale_x = components_a.scale_x;
    components_b.scale_y = components_a.scale_y;
    components_b.skew = components_a.skew;

    write_world_transform(artboard, component_index, Mat2D::compose(components_b))
}

fn apply_scale_constraint(
    artboard: &mut ArtboardInstance,
    component_index: usize,
    constraint_local: usize,
) -> bool {
    // Ported from C++ `src/constraints/scale_constraint.cpp`.
    let target_index = targeted_constraint_target_local(artboard, constraint_local)
        .and_then(|target_local| artboard.component_by_local.get(&target_local).copied());
    if target_index.is_some_and(|index| artboard.components[index].is_collapsed()) {
        return false;
    }

    let transform_a = artboard.components[component_index]
        .transform
        .world_transform;
    let components_a = transform_a.decompose();
    let mut components_b = components_a;

    if let Some(target_index) = target_index {
        let mut transform_b = artboard.components[target_index].transform.world_transform;
        if transform_space(
            artboard,
            constraint_local,
            "TransformSpaceConstraint",
            "sourceSpaceValue",
        ) == TransformSpace::Local
        {
            let Some(inverse) = invert(parent_world_transform(artboard, target_index)) else {
                return false;
            };
            transform_b = inverse.multiply(transform_b);
        }

        components_b = transform_b.decompose();
        let dest_space = transform_space(
            artboard,
            constraint_local,
            "TransformSpaceConstraint",
            "destSpaceValue",
        );
        let authored = artboard.authored_transform(artboard.components[component_index].local_id);
        if !constraint_bool(
            artboard,
            constraint_local,
            "TransformComponentConstraint",
            "doesCopy",
            true,
        ) {
            components_b.scale_x = if dest_space == TransformSpace::Local {
                1.0
            } else {
                components_a.scale_x
            };
        } else {
            components_b.scale_x *= constraint_double(
                artboard,
                constraint_local,
                "TransformComponentConstraint",
                "copyFactor",
                1.0,
            );
            if constraint_bool(
                artboard,
                constraint_local,
                "TransformComponentConstraint",
                "offset",
                false,
            ) {
                components_b.scale_x *= authored.scale_x;
            }
        }

        if !constraint_bool(
            artboard,
            constraint_local,
            "TransformComponentConstraintY",
            "doesCopyY",
            true,
        ) {
            components_b.scale_y = if dest_space == TransformSpace::Local {
                1.0
            } else {
                components_a.scale_y
            };
        } else {
            components_b.scale_y *= constraint_double(
                artboard,
                constraint_local,
                "TransformComponentConstraintY",
                "copyFactorY",
                1.0,
            );
            if constraint_bool(
                artboard,
                constraint_local,
                "TransformComponentConstraint",
                "offset",
                false,
            ) {
                components_b.scale_y *= authored.scale_y;
            }
        }

        if dest_space == TransformSpace::Local {
            let transform_b = parent_world_transform(artboard, component_index)
                .multiply(Mat2D::compose(components_b));
            components_b = transform_b.decompose();
        }
    }

    let clamp_local = transform_space(
        artboard,
        constraint_local,
        "TransformComponentConstraint",
        "minMaxSpaceValue",
    ) == TransformSpace::Local;
    if clamp_local {
        let transform_b = Mat2D::compose(components_b);
        let Some(inverse) = invert(parent_world_transform(artboard, component_index)) else {
            return false;
        };
        components_b = inverse.multiply(transform_b).decompose();
    }
    if constraint_bool(
        artboard,
        constraint_local,
        "TransformComponentConstraint",
        "max",
        false,
    ) && components_b.scale_x
        > constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraint",
            "maxValue",
            0.0,
        )
    {
        components_b.scale_x = constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraint",
            "maxValue",
            0.0,
        );
    }
    if constraint_bool(
        artboard,
        constraint_local,
        "TransformComponentConstraint",
        "min",
        false,
    ) && components_b.scale_x
        < constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraint",
            "minValue",
            0.0,
        )
    {
        components_b.scale_x = constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraint",
            "minValue",
            0.0,
        );
    }
    if constraint_bool(
        artboard,
        constraint_local,
        "TransformComponentConstraintY",
        "maxY",
        false,
    ) && components_b.scale_y
        > constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraintY",
            "maxValueY",
            0.0,
        )
    {
        components_b.scale_y = constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraintY",
            "maxValueY",
            0.0,
        );
    }
    if constraint_bool(
        artboard,
        constraint_local,
        "TransformComponentConstraintY",
        "minY",
        false,
    ) && components_b.scale_y
        < constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraintY",
            "minValueY",
            0.0,
        )
    {
        components_b.scale_y = constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraintY",
            "minValueY",
            0.0,
        );
    }
    if clamp_local {
        let transform_b = parent_world_transform(artboard, component_index)
            .multiply(Mat2D::compose(components_b));
        components_b = transform_b.decompose();
    }

    let t = constraint_double(artboard, constraint_local, "Constraint", "strength", 1.0);
    let ti = 1.0 - t;
    components_b.rotation = components_a.rotation;
    components_b.x = components_a.x;
    components_b.y = components_a.y;
    components_b.scale_x = components_a.scale_x * ti + components_b.scale_x * t;
    components_b.scale_y = components_a.scale_y * ti + components_b.scale_y * t;
    components_b.skew = components_a.skew;

    write_world_transform(artboard, component_index, Mat2D::compose(components_b))
}

fn apply_transform_constraint(
    artboard: &mut ArtboardInstance,
    component_index: usize,
    constraint_local: usize,
) -> bool {
    // Ported from C++ `src/constraints/transform_constraint.cpp`.
    let Some(target_index) = targeted_constraint_target_local(artboard, constraint_local)
        .and_then(|target_local| artboard.component_by_local.get(&target_local).copied())
    else {
        return false;
    };
    if artboard.components[target_index].is_collapsed() {
        return false;
    }

    let transform_a = artboard.components[component_index]
        .transform
        .world_transform;
    let mut transform_b =
        target_transform_for_transform_constraint(artboard, target_index, constraint_local);
    if transform_space(
        artboard,
        constraint_local,
        "TransformSpaceConstraint",
        "sourceSpaceValue",
    ) == TransformSpace::Local
    {
        let Some(inverse) = invert(parent_world_transform(artboard, target_index)) else {
            return false;
        };
        transform_b = inverse.multiply(transform_b);
    }
    if transform_space(
        artboard,
        constraint_local,
        "TransformSpaceConstraint",
        "destSpaceValue",
    ) == TransformSpace::Local
    {
        transform_b = parent_world_transform(artboard, component_index).multiply(transform_b);
    }

    constrain_world(
        artboard,
        component_index,
        transform_a,
        transform_b,
        constraint_double(artboard, constraint_local, "Constraint", "strength", 1.0),
    )
}

fn apply_follow_path_constraint(
    artboard: &mut ArtboardInstance,
    component_index: usize,
    constraint_local: usize,
) -> bool {
    // Ported from C++ `src/constraints/follow_path_constraint.cpp`.
    let Some(runtime) = follow_path_constraint(artboard, constraint_local).cloned() else {
        return false;
    };
    let Some(target_index) = artboard
        .component_by_local
        .get(&runtime.target_local)
        .copied()
    else {
        return false;
    };
    if artboard.components[target_index].is_collapsed() {
        return false;
    }

    let transform_b = target_transform_for_follow_path_constraint(
        artboard,
        &runtime,
        target_index,
        component_index,
    );
    let components = follow_path_constrain_components(
        artboard,
        constraint_local,
        target_index,
        artboard.components[component_index]
            .transform
            .world_transform,
        transform_b,
        parent_world_transform(artboard, component_index),
    );
    write_world_transform(artboard, component_index, Mat2D::compose(components))
}

fn target_transform_for_follow_path_constraint(
    artboard: &ArtboardInstance,
    runtime: &RuntimeFollowPathConstraint,
    target_index: usize,
    component_index: usize,
) -> Mat2D {
    let distance = constraint_double(
        artboard,
        runtime.local_id,
        "FollowPathConstraint",
        "distance",
        0.0,
    );
    target_transform_for_follow_path_constraint_at_distance(
        artboard,
        runtime,
        target_index,
        component_index,
        distance,
    )
}

fn target_transform_for_follow_path_constraint_at_distance(
    artboard: &ArtboardInstance,
    runtime: &RuntimeFollowPathConstraint,
    target_index: usize,
    offset_component_index: usize,
    distance: f32,
) -> Mat2D {
    match runtime.target_kind {
        RuntimeFollowPathTargetKind::Shape | RuntimeFollowPathTargetKind::Path => {
            let mut commands = Vec::new();
            for path in &runtime.paths {
                let Some(path_world) = artboard
                    .component(path.local_id)
                    .map(|component| component.transform.world_transform)
                else {
                    continue;
                };
                commands.extend(runtime_path_geometry_commands(
                    artboard,
                    &path.geometry,
                    path_world,
                ));
            }

            let sample = RuntimePathMeasure::from_commands(&commands).at_percentage(distance);
            let mut transform_b = artboard.components[target_index].transform.world_transform;

            if constraint_bool(
                artboard,
                runtime.local_id,
                "FollowPathConstraint",
                "orient",
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
                            * constraint_double(
                                artboard,
                                runtime.local_id,
                                "Constraint",
                                "strength",
                                1.0,
                            ),
                );
            }

            let offset_position = if constraint_bool(
                artboard,
                runtime.local_id,
                "FollowPathConstraint",
                "offset",
                false,
            ) {
                let local_transform = artboard.components[offset_component_index]
                    .transform
                    .local_transform
                    .0;
                (local_transform[4], local_transform[5])
            } else {
                (0.0, 0.0)
            };
            transform_b.0[4] = sample.pos.0 + offset_position.0;
            transform_b.0[5] = sample.pos.1 + offset_position.1;
            transform_b
        }
        RuntimeFollowPathTargetKind::Other => {
            artboard.components[target_index].transform.world_transform
        }
    }
}

fn apply_list_follow_path_constraint_to_transforms(
    artboard: &ArtboardInstance,
    list_component_index: usize,
    runtime: &RuntimeFollowPathConstraint,
    item_transforms: &mut [Mat2D],
) -> bool {
    // Ported from C++ `src/constraints/list_follow_path_constraint.cpp`.
    let Some(target_index) = artboard
        .component_by_local
        .get(&runtime.target_local)
        .copied()
    else {
        return false;
    };
    if artboard.components[target_index].is_collapsed() {
        return false;
    }

    let count = item_transforms.len();
    let distance = constraint_double(
        artboard,
        runtime.local_id,
        "FollowPathConstraint",
        "distance",
        0.0,
    );
    let distance_end = constraint_double(
        artboard,
        runtime.local_id,
        "ListFollowPathConstraint",
        "distanceEnd",
        1.0,
    );
    let distance_offset = constraint_double(
        artboard,
        runtime.local_id,
        "ListFollowPathConstraint",
        "distanceOffset",
        0.0,
    );
    let start_offset = distance_offset + distance;
    let start_to_end_distance = distance_end - distance;
    let offset_distance = if count <= 1 {
        0.0
    } else {
        start_to_end_distance / (count as f32 - 1.0)
    };
    let list_transform = artboard.components[list_component_index]
        .transform
        .world_transform;
    let mut changed = false;

    for (index, transform) in item_transforms.iter_mut().enumerate() {
        let transform_b = target_transform_for_follow_path_constraint_at_distance(
            artboard,
            runtime,
            target_index,
            list_component_index,
            start_offset + index as f32 * offset_distance,
        );
        let components = follow_path_constrain_components(
            artboard,
            runtime.local_id,
            target_index,
            *transform,
            transform_b,
            list_transform,
        );
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
    target_index: usize,
    component_transform: Mat2D,
    mut transform_b: Mat2D,
    component_parent_world: Mat2D,
) -> TransformComponents {
    if transform_space(
        artboard,
        constraint_local,
        "TransformSpaceConstraint",
        "sourceSpaceValue",
    ) == TransformSpace::Local
    {
        let target_parent_world = parent_world_transform(artboard, target_index);
        let Some(inverse) = invert(target_parent_world) else {
            return TransformComponents::default();
        };
        transform_b = inverse.multiply(transform_b);
    }
    if transform_space(
        artboard,
        constraint_local,
        "TransformSpaceConstraint",
        "destSpaceValue",
    ) == TransformSpace::Local
    {
        transform_b = component_parent_world.multiply(transform_b);
    }

    let components_a = component_transform.decompose();
    let mut components_b = transform_b.decompose();
    let t = constraint_double(artboard, constraint_local, "Constraint", "strength", 1.0);
    let ti = 1.0 - t;

    if !constraint_bool(
        artboard,
        constraint_local,
        "FollowPathConstraint",
        "orient",
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

fn follow_path_constraint(
    artboard: &ArtboardInstance,
    constraint_local: usize,
) -> Option<&RuntimeFollowPathConstraint> {
    artboard
        .follow_path_constraints
        .iter()
        .find(|constraint| constraint.local_id == constraint_local)
}

fn apply_ik_constraint(
    artboard: &mut ArtboardInstance,
    _component_index: usize,
    constraint_local: usize,
) -> bool {
    // Ported from C++ `src/constraints/ik_constraint.cpp`.
    let Some(runtime) = ik_constraint(artboard, constraint_local).cloned() else {
        return false;
    };
    let Some(target_index) = artboard
        .component_by_local
        .get(&runtime.target_local)
        .copied()
    else {
        return false;
    };
    if artboard.components[target_index].is_collapsed() {
        return false;
    }

    let invert_direction = constraint_bool(
        artboard,
        constraint_local,
        "IKConstraint",
        "invertDirection",
        false,
    );
    let world_target_translation =
        world_translation(artboard.components[target_index].transform.world_transform);
    let mut chain = Vec::new();
    let mut changed = false;
    for link in &runtime.chain {
        let Some(bone_index) = artboard.component_by_local.get(&link.bone_local).copied() else {
            continue;
        };
        let parent_world = parent_world_transform(artboard, bone_index);
        let parent_world_inverse = parent_world.invert_or_identity();
        let bone_transform = parent_world_inverse
            .multiply(artboard.components[bone_index].transform.world_transform);
        changed |= write_local_transform(artboard, bone_index, bone_transform);
        chain.push(IkChainState {
            bone_index,
            parent_world_inverse,
            transform_components: bone_transform.decompose(),
            angle: 0.0,
        });
    }

    match chain.len() {
        0 => return changed,
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
                    let bone_index = chain[child_index].bone_index;
                    chain[child_index].parent_world_inverse =
                        parent_world_transform(artboard, bone_index).invert_or_identity();
                }
            }
        }
    }

    let strength = constraint_double(artboard, constraint_local, "Constraint", "strength", 1.0);
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

    changed
}

fn solve_ik1(
    artboard: &mut ArtboardInstance,
    chain: &mut [IkChainState],
    index: usize,
    world_target_translation: (f32, f32),
) -> bool {
    let bone_index = chain[index].bone_index;
    let p_a = world_translation(artboard.components[bone_index].transform.world_transform);
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
    chain: &mut [IkChainState],
    fk1_index: usize,
    fk2_index: usize,
    world_target_translation: (f32, f32),
    invert_direction: bool,
) -> bool {
    let first_child_index = fk1_index + 1;
    let b1_index = chain[fk1_index].bone_index;
    let b2_index = chain[fk2_index].bone_index;
    let first_child_bone_index = chain[first_child_index].bone_index;
    let iworld = chain[fk1_index].parent_world_inverse;

    let mut p_a = world_translation(artboard.components[b1_index].transform.world_transform);
    let mut p_c = world_translation(
        artboard.components[first_child_bone_index]
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

    let (r1, r2) = if artboard.components[b2_index].parent_local
        != Some(artboard.components[b1_index].local_id)
    {
        let second_child_index = fk1_index + 2;
        let second_child_world_inverse = chain[second_child_index].parent_world_inverse;
        let p_c_world = world_translation(
            artboard.components[first_child_bone_index]
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
        let bone_index = chain[fk2_index].bone_index;
        let parent_world = parent_world_transform(artboard, bone_index);
        let local = artboard.components[bone_index].transform.local_transform;
        changed |= write_world_transform(artboard, bone_index, parent_world.multiply(local));
    }

    chain[fk1_index].angle = r1;
    chain[first_child_index].angle = r2;
    changed
}

fn constrain_ik_rotation(
    artboard: &mut ArtboardInstance,
    state: &IkChainState,
    rotation: f32,
) -> bool {
    let bone_index = state.bone_index;
    let mut components = state.transform_components;
    components.rotation = rotation;
    let local_transform = Mat2D::compose(components);
    let parent_world = parent_world_transform(artboard, bone_index);
    write_local_world_transform(
        artboard,
        bone_index,
        local_transform,
        parent_world.multiply(local_transform),
    )
}

fn ik_constraint(
    artboard: &ArtboardInstance,
    constraint_local: usize,
) -> Option<&RuntimeIkConstraint> {
    artboard
        .ik_constraints
        .iter()
        .find(|constraint| constraint.local_id == constraint_local)
}

fn target_transform_for_transform_constraint(
    artboard: &ArtboardInstance,
    target_index: usize,
    constraint_local: usize,
) -> Mat2D {
    let (left, top, width, height) = constraint_bounds(artboard, target_index);
    let origin_x = constraint_double(
        artboard,
        constraint_local,
        "TransformConstraint",
        "originX",
        0.0,
    );
    let origin_y = constraint_double(
        artboard,
        constraint_local,
        "TransformConstraint",
        "originY",
        0.0,
    );
    let component = &artboard.components[target_index];
    let target_world = if artboard.layout_constraint_bounds_enabled
        && component.type_name == "LayoutComponent"
        && let Some(graph) = artboard.runtime_graph()
    {
        artboard.runtime_component_world_transform_with_bounds(
            component.local_id,
            graph,
            artboard.layout_constraint_bounds.as_deref(),
        )
    } else {
        component.transform.world_transform
    };
    target_world.multiply(Mat2D([
        1.0,
        0.0,
        0.0,
        1.0,
        left + width * origin_x,
        top + height * origin_y,
    ]))
}

fn constraint_bounds(artboard: &ArtboardInstance, component_index: usize) -> (f32, f32, f32, f32) {
    let component = &artboard.components[component_index];
    if artboard.layout_constraint_bounds_enabled
        && component.type_name == "LayoutComponent"
        && let Some(graph) = artboard.runtime_graph()
    {
        let bounds = artboard
            .layout_constraint_bounds
            .as_deref()
            .and_then(|bounds| bounds.get(&component.local_id).copied())
            .unwrap_or_else(|| artboard.runtime_layout_component_bounds(component.local_id, graph));
        return (0.0, 0.0, bounds.width, bounds.height);
    }
    if component.type_name == "Text"
        && let (Some(runtime), Some(graph)) = (artboard.runtime_file(), artboard.runtime_graph())
        && let Some(bounds) =
            static_text_constraint_bounds(runtime, graph, artboard, component.local_id)
    {
        return bounds;
    }

    // C++ `TransformComponent::constraintBounds()` defaults to an empty AABB.
    // LayoutComponent overrides become available after their layout host has
    // supplied dimensions, matching LayoutComponent::localBounds().
    (0.0, 0.0, 0.0, 0.0)
}

fn constrain_world(
    artboard: &mut ArtboardInstance,
    component_index: usize,
    from: Mat2D,
    to: Mat2D,
    strength: f32,
) -> bool {
    let components_from = from.decompose();
    let mut components_to = to.decompose();
    let t = strength;
    let ti = 1.0 - t;

    components_to.rotation =
        interpolated_rotation(components_from.rotation, components_to.rotation, t);
    components_to.x = components_from.x * ti + components_to.x * t;
    components_to.y = components_from.y * ti + components_to.y * t;
    components_to.scale_x = components_from.scale_x * ti + components_to.scale_x * t;
    components_to.scale_y = components_from.scale_y * ti + components_to.scale_y * t;
    components_to.skew = components_from.skew * ti + components_to.skew * t;

    write_world_transform(artboard, component_index, Mat2D::compose(components_to))
}

fn apply_translation_constraint(
    artboard: &mut ArtboardInstance,
    component_index: usize,
    constraint_local: usize,
) -> bool {
    // Ported from C++ `src/constraints/translation_constraint.cpp`.
    let target_index = targeted_constraint_target_local(artboard, constraint_local)
        .and_then(|target_local| artboard.component_by_local.get(&target_local).copied());
    if target_index.is_some_and(|index| artboard.components[index].is_collapsed()) {
        return false;
    }

    let world = artboard.components[component_index]
        .transform
        .world_transform;
    let translation_a = (world.0[4], world.0[5]);
    let mut translation_b = translation_a;

    if let Some(target_index) = target_index {
        let mut transform_b = artboard.components[target_index].transform.world_transform;
        if transform_space(
            artboard,
            constraint_local,
            "TransformSpaceConstraint",
            "sourceSpaceValue",
        ) == TransformSpace::Local
        {
            let Some(inverse) = invert(parent_world_transform(artboard, target_index)) else {
                return false;
            };
            transform_b = inverse.multiply(transform_b);
        }
        translation_b = (transform_b.0[4], transform_b.0[5]);

        let dest_space = transform_space(
            artboard,
            constraint_local,
            "TransformSpaceConstraint",
            "destSpaceValue",
        );
        let authored = artboard.authored_transform(artboard.components[component_index].local_id);
        if !constraint_bool(
            artboard,
            constraint_local,
            "TransformComponentConstraint",
            "doesCopy",
            true,
        ) {
            translation_b.0 = if dest_space == TransformSpace::Local {
                0.0
            } else {
                translation_a.0
            };
        } else {
            translation_b.0 *= constraint_double(
                artboard,
                constraint_local,
                "TransformComponentConstraint",
                "copyFactor",
                1.0,
            );
            if constraint_bool(
                artboard,
                constraint_local,
                "TransformComponentConstraint",
                "offset",
                false,
            ) {
                translation_b.0 += authored.x;
            }
        }

        if !constraint_bool(
            artboard,
            constraint_local,
            "TransformComponentConstraintY",
            "doesCopyY",
            true,
        ) {
            translation_b.1 = if dest_space == TransformSpace::Local {
                0.0
            } else {
                translation_a.1
            };
        } else {
            translation_b.1 *= constraint_double(
                artboard,
                constraint_local,
                "TransformComponentConstraintY",
                "copyFactorY",
                1.0,
            );
            if constraint_bool(
                artboard,
                constraint_local,
                "TransformComponentConstraint",
                "offset",
                false,
            ) {
                translation_b.1 += authored.y;
            }
        }

        if dest_space == TransformSpace::Local {
            translation_b = parent_world_transform(artboard, component_index)
                .transform_point(translation_b.0, translation_b.1);
        }
    }

    let clamp_local = transform_space(
        artboard,
        constraint_local,
        "TransformComponentConstraint",
        "minMaxSpaceValue",
    ) == TransformSpace::Local;
    if clamp_local {
        let Some(inverse) = invert(parent_world_transform(artboard, component_index)) else {
            return false;
        };
        translation_b = inverse.transform_point(translation_b.0, translation_b.1);
    }
    if constraint_bool(
        artboard,
        constraint_local,
        "TransformComponentConstraint",
        "max",
        false,
    ) && translation_b.0
        > constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraint",
            "maxValue",
            0.0,
        )
    {
        translation_b.0 = constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraint",
            "maxValue",
            0.0,
        );
    }
    if constraint_bool(
        artboard,
        constraint_local,
        "TransformComponentConstraint",
        "min",
        false,
    ) && translation_b.0
        < constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraint",
            "minValue",
            0.0,
        )
    {
        translation_b.0 = constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraint",
            "minValue",
            0.0,
        );
    }
    if constraint_bool(
        artboard,
        constraint_local,
        "TransformComponentConstraintY",
        "maxY",
        false,
    ) && translation_b.1
        > constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraintY",
            "maxValueY",
            0.0,
        )
    {
        translation_b.1 = constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraintY",
            "maxValueY",
            0.0,
        );
    }
    if constraint_bool(
        artboard,
        constraint_local,
        "TransformComponentConstraintY",
        "minY",
        false,
    ) && translation_b.1
        < constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraintY",
            "minValueY",
            0.0,
        )
    {
        translation_b.1 = constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraintY",
            "minValueY",
            0.0,
        );
    }
    if clamp_local {
        translation_b = parent_world_transform(artboard, component_index)
            .transform_point(translation_b.0, translation_b.1);
    }

    let t = constraint_double(artboard, constraint_local, "Constraint", "strength", 1.0);
    let ti = 1.0 - t;
    let new_x = translation_a.0 * ti + translation_b.0 * t;
    let new_y = translation_a.1 * ti + translation_b.1 * t;

    let mut transform = artboard.components[component_index]
        .transform
        .world_transform;
    transform.0[4] = new_x;
    transform.0[5] = new_y;
    write_world_transform(artboard, component_index, transform)
}

fn write_world_transform(
    artboard: &mut ArtboardInstance,
    component_index: usize,
    transform: Mat2D,
) -> bool {
    let world = &mut artboard.components[component_index]
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
    component_index: usize,
    transform: Mat2D,
) -> bool {
    let local = &mut artboard.components[component_index]
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
    component_index: usize,
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

fn tip_world_translation(artboard: &ArtboardInstance, bone_index: usize) -> (f32, f32) {
    let bone = &artboard.components[bone_index];
    let length = artboard.bone_length(bone.local_id).unwrap_or(0.0);
    bone.transform.world_transform.transform_point(length, 0.0)
}

fn point_sub(left: (f32, f32), right: (f32, f32)) -> (f32, f32) {
    (left.0 - right.0, left.1 - right.1)
}

fn point_length(point: (f32, f32)) -> f32 {
    point.0.hypot(point.1)
}

fn point_atan2(point: (f32, f32)) -> f32 {
    point.1.atan2(point.0)
}

fn targeted_constraint_target_local(
    artboard: &ArtboardInstance,
    constraint_local: usize,
) -> Option<usize> {
    let target_key = property_key_for_name("TargetedConstraint", "targetId")?;
    let target_local =
        usize::try_from(artboard.uint_property(constraint_local, target_key)?).ok()?;
    artboard.slot(target_local).map(|slot| slot.local_id)
}

fn parent_world_transform(artboard: &ArtboardInstance, component_index: usize) -> Mat2D {
    let Some(parent_local) = artboard.components[component_index].parent_local else {
        return Mat2D::IDENTITY;
    };
    artboard
        .component(parent_local)
        .filter(|parent| parent.capabilities.world_transform)
        .map(|parent| parent.transform.world_transform)
        .unwrap_or(Mat2D::IDENTITY)
}

fn invert(transform: Mat2D) -> Option<Mat2D> {
    (transform.determinant() != 0.0).then(|| transform.invert_or_identity())
}

fn transform_space(
    artboard: &ArtboardInstance,
    local_id: usize,
    type_name: &str,
    property_name: &str,
) -> TransformSpace {
    TransformSpace::from_value(constraint_uint(
        artboard,
        local_id,
        type_name,
        property_name,
        0,
    ))
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

fn local_object_type(graph: &ArtboardGraph, local_id: usize) -> Option<&'static str> {
    graph
        .local_objects
        .iter()
        .find(|object| object.local_id == local_id)
        .and_then(|object| object.type_name)
}

fn local_object_parent(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    local_id: usize,
) -> Option<usize> {
    let global_id = graph
        .local_objects
        .iter()
        .find(|object| object.local_id == local_id)?
        .global_id;
    usize::try_from(file.object(global_id as usize)?.uint_property("parentId")?).ok()
}

fn is_bone_type(type_name: &'static str) -> bool {
    type_name == "Bone" || type_name == "RootBone"
}

#[cfg(test)]
mod tests {
    use nuxie_binary::read_runtime_file;
    use nuxie_graph::GraphFile;

    use crate::ArtboardInstance;
    use crate::draw::RuntimeLayoutBounds;
    use crate::properties::property_key_for_name;

    use super::{
        RuntimeComponentListVirtualItem, RuntimeScrollAxis, RuntimeScrollAxisIntent,
        RuntimeScrollConstraint, RuntimeScrollLayoutMetrics, RuntimeScrollProperty,
        RuntimeScrollSpace, clamped_scroll_offset, component_list_virtual_window_for_metrics,
        component_list_virtual_windows_for_provider_metrics, runtime_scroll_intent_axes,
        scroll_viewport_axis_size, virtualized_provider_content_size,
    };

    fn vertical_item(logical_index: usize, y: f32) -> RuntimeComponentListVirtualItem {
        RuntimeComponentListVirtualItem {
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
        let mut constraint = RuntimeScrollConstraint {
            local_id: 3,
            content_local: 2,
            layout_child_locals: vec![],
            intent_x: Some(intent),
            intent_y: Some(intent),
            layout_initialized: false,
        };
        assert!(constraint.clear_intent(RuntimeScrollAxis::X));
        assert_eq!(constraint.intent_x, None);
        assert_eq!(constraint.intent_y, Some(intent));
    }

    #[test]
    fn virtual_window_mounts_only_rows_intersecting_the_viewport() {
        let sizes = vec![(200.0, 50.0); 10];
        assert_eq!(
            component_list_virtual_window_for_metrics(&sizes, false, -10.0, 100.0, 0.0, true,),
            vec![
                vertical_item(0, 0.0),
                vertical_item(1, 40.0),
                vertical_item(2, 80.0),
            ]
        );
    }

    #[test]
    fn virtual_window_preserves_wrapped_infinite_order_and_positions() {
        let sizes = vec![(200.0, 50.0); 10];
        assert_eq!(
            component_list_virtual_window_for_metrics(&sizes, false, -10.0, 100.0, -360.0, true,),
            vec![
                vertical_item(8, -40.0),
                vertical_item(9, 0.0),
                vertical_item(0, 40.0),
                vertical_item(1, 80.0),
            ]
        );
    }

    #[test]
    fn virtual_window_does_not_wrap_a_finite_list() {
        let sizes = vec![(20.0, 30.0); 4];
        assert_eq!(
            component_list_virtual_window_for_metrics(&sizes, true, 5.0, 40.0, -70.0, false,),
            vec![RuntimeComponentListVirtualItem {
                logical_index: 3,
                position_x: 5.0,
                position_y: 0.0,
            }]
        );
    }

    #[test]
    fn virtual_window_flattens_ordinary_providers_and_multiple_lists() {
        let providers = vec![
            vec![(20.0, 10.0)],
            vec![(30.0, 10.0), (30.0, 10.0)],
            vec![(15.0, 10.0)],
            vec![(25.0, 10.0), (25.0, 10.0)],
        ];
        let content_size = virtualized_provider_content_size(&providers, true, 5.0, false);
        assert_eq!(content_size, 170.0);

        let windows = component_list_virtual_windows_for_provider_metrics(
            &providers,
            true,
            5.0,
            130.0,
            -25.0,
            false,
            content_size,
        );
        assert_eq!(
            windows[1],
            vec![
                RuntimeComponentListVirtualItem {
                    logical_index: 0,
                    position_x: 0.0,
                    position_y: 0.0,
                },
                RuntimeComponentListVirtualItem {
                    logical_index: 1,
                    position_x: 35.0,
                    position_y: 0.0,
                },
            ]
        );
        assert_eq!(
            windows[3],
            vec![
                RuntimeComponentListVirtualItem {
                    logical_index: 0,
                    position_x: 90.0,
                    position_y: 0.0,
                },
                RuntimeComponentListVirtualItem {
                    logical_index: 1,
                    position_x: 120.0,
                    position_y: 0.0,
                },
            ]
        );
    }

    #[test]
    fn virtual_window_uses_content_origin_and_clamped_offset() {
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
    fn virtual_window_applies_size_feedback_to_later_rows_without_remounting() {
        let initial_sizes = vec![(100.0, 20.0); 3];
        let initial = component_list_virtual_window_for_metrics(
            &initial_sizes,
            false,
            5.0,
            100.0,
            0.0,
            false,
        );
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
        let settled = component_list_virtual_window_for_metrics(
            &measured_sizes,
            false,
            5.0,
            100.0,
            0.0,
            false,
        );
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
