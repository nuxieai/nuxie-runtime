use crate::objects::InstanceObjectArena;
use crate::properties::{artboard_index_for_graph, property_key_for_name};
use rive_binary::RuntimeFile;
use rive_graph::{ArtboardGraph, ComponentNode};
use rive_schema::definition_by_name;
use std::collections::BTreeMap;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

/// Runtime component dirt and transform state.
///
/// Ported from C++ `src/component.cpp`, `src/transform_component.cpp`, and
/// `src/math/mat2d.cpp` for the update-order and matrix semantics that M2
/// exercises.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComponentDirt(pub u16);

impl ComponentDirt {
    pub const NONE: Self = Self(0);
    pub const COLLAPSED: Self = Self(1 << 0);
    pub const DEPENDENTS: Self = Self(1 << 1);
    pub const COMPONENTS: Self = Self(1 << 2);
    pub const DRAW_ORDER: Self = Self(1 << 3);
    pub const PATH: Self = Self(1 << 4);
    pub const TEXT_SHAPE: Self = Self(1 << 4);
    pub const SKIN: Self = Self(1 << 4);
    pub const VERTICES: Self = Self(1 << 5);
    pub const TEXT_COVERAGE: Self = Self(1 << 5);
    pub const TRANSFORM: Self = Self(1 << 6);
    pub const WORLD_TRANSFORM: Self = Self(1 << 7);
    pub const RENDER_OPACITY: Self = Self(1 << 8);
    pub const PAINT: Self = Self(1 << 9);
    pub const STOPS: Self = Self(1 << 10);
    pub const LAYOUT_STYLE: Self = Self(1 << 11);
    pub const BINDINGS: Self = Self(1 << 12);
    pub const N_SLICER: Self = Self(1 << 13);
    pub const SCRIPT_UPDATE: Self = Self(1 << 14);
    pub const CLIPPING: Self = Self(1 << 15);
    pub const FILTHY: Self = Self(0xFFFE);

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn contains(self, flag: Self) -> bool {
        (self.0 & flag.0) == flag.0
    }
}

impl Default for ComponentDirt {
    fn default() -> Self {
        Self::NONE
    }
}

impl BitOr for ComponentDirt {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for ComponentDirt {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for ComponentDirt {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for ComponentDirt {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl Not for ComponentDirt {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UpdateComponentsReport {
    pub did_update: bool,
    pub steps: usize,
    pub updated_locals: Vec<usize>,
    pub max_steps_reached: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformProperty {
    X,
    Y,
    Rotation,
    ScaleX,
    ScaleY,
    Opacity,
}

impl TransformProperty {
    pub(crate) fn property_name(self) -> &'static str {
        match self {
            Self::X => "x",
            Self::Y => "y",
            Self::Rotation => "rotation",
            Self::ScaleX => "scaleX",
            Self::ScaleY => "scaleY",
            Self::Opacity => "opacity",
        }
    }

    pub(crate) fn default_value(self) -> f32 {
        match self {
            Self::X | Self::Y | Self::Rotation => 0.0,
            Self::ScaleX | Self::ScaleY | Self::Opacity => 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RuntimeComponentCapabilities {
    pub world_transform: bool,
    pub transform: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct AuthoredTransform {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) rotation: f32,
    pub(crate) scale_x: f32,
    pub(crate) scale_y: f32,
    pub(crate) opacity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransformRuntimeState {
    pub local_transform: Mat2D,
    pub world_transform: Mat2D,
    pub render_opacity: f32,
}

impl Default for TransformRuntimeState {
    fn default() -> Self {
        Self {
            local_transform: Mat2D::IDENTITY,
            world_transform: Mat2D::IDENTITY,
            render_opacity: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat2D(pub [f32; 6]);

impl Mat2D {
    pub const IDENTITY: Self = Self([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);

    pub fn from_rotation(radians: f32) -> Self {
        let (sin, cos) = if radians == 0.0 {
            (0.0, 1.0)
        } else {
            radians.sin_cos()
        };
        Self([cos, sin, -sin, cos, 0.0, 0.0])
    }

    pub fn multiply(self, rhs: Self) -> Self {
        let a = self.0;
        let b = rhs.0;
        Self([
            a[0].mul_add(b[0], a[2] * b[1]),
            a[1].mul_add(b[0], a[3] * b[1]),
            a[0].mul_add(b[2], a[2] * b[3]),
            a[1].mul_add(b[2], a[3] * b[3]),
            a[0].mul_add(b[4], a[2] * b[5]) + a[4],
            a[1].mul_add(b[4], a[3] * b[5]) + a[5],
        ])
    }

    pub(crate) fn multiply_path_local_fused(self, rhs: Self) -> Self {
        let a = self.0;
        let b = rhs.0;
        Self([
            a[0].mul_add(b[0], a[2] * b[1]),
            a[1].mul_add(b[0], a[3] * b[1]),
            a[0].mul_add(b[2], a[2] * b[3]),
            a[1].mul_add(b[2], a[3] * b[3]),
            a[0].mul_add(b[4], a[2].mul_add(b[5], a[4])),
            a[1].mul_add(b[4], a[3].mul_add(b[5], a[5])),
        ])
    }

    pub(crate) fn multiply_path_local_contracted(self, rhs: Self) -> Self {
        let a = self.0;
        let b = rhs.0;
        Self([
            a[0].mul_add(b[0], a[2] * b[1]),
            a[1].mul_add(b[0], a[3] * b[1]),
            a[0].mul_add(b[2], a[2] * b[3]),
            a[1].mul_add(b[2], a[3] * b[3]),
            a[0].mul_add(b[4], a[2] * b[5]) + a[4],
            a[1].mul_add(b[4], a[3] * b[5]) + a[5],
        ])
    }

    pub fn scale_by_values(&mut self, scale_x: f32, scale_y: f32) {
        self.0[0] *= scale_x;
        self.0[1] *= scale_x;
        self.0[2] *= scale_y;
        self.0[3] *= scale_y;
    }

    pub(crate) fn decompose(self) -> TransformComponents {
        // Ported from C++ `src/math/mat2d.cpp`.
        let [m0, m1, m2, m3, x, y] = self.0;
        let rotation = m1.atan2(m0);
        let denom = m0 * m0 + m1 * m1;
        let scale_x = denom.sqrt();
        let scale_y = if scale_x == 0.0 {
            0.0
        } else {
            (m0 * m3 - m2 * m1) / scale_x
        };
        let skew = (m0 * m2 + m1 * m3).atan2(denom);
        TransformComponents {
            x,
            y,
            scale_x,
            scale_y,
            rotation,
            skew,
        }
    }

    pub(crate) fn compose(components: TransformComponents) -> Self {
        // Ported from C++ `src/math/mat2d.cpp`.
        let mut result = Self::from_rotation(components.rotation);
        result.0[4] = components.x;
        result.0[5] = components.y;
        result.scale_by_values(components.scale_x, components.scale_y);

        if components.skew != 0.0 {
            result.0[2] = result.0[0] * components.skew + result.0[2];
            result.0[3] = result.0[1] * components.skew + result.0[3];
        }
        result
    }

    pub fn determinant(self) -> f32 {
        self.0[0] * self.0[3] - self.0[1] * self.0[2]
    }

    pub fn invert_or_identity(self) -> Self {
        let determinant = self.determinant();
        if determinant == 0.0 {
            return Self::IDENTITY;
        }

        let [a, b, c, d, e, f] = self.0;
        let determinant = 1.0 / determinant;
        Self([
            d * determinant,
            -b * determinant,
            -c * determinant,
            a * determinant,
            (c * f - d * e) * determinant,
            (b * e - a * f) * determinant,
        ])
    }

    pub fn transform_point(self, x: f32, y: f32) -> (f32, f32) {
        (
            self.0[0] * x + self.0[2] * y + self.0[4],
            self.0[1] * x + self.0[3] * y + self.0[5],
        )
    }

    pub fn map_point(self, x: f32, y: f32) -> (f32, f32) {
        let [a, b, c, d, e, f] = self.0;
        // Ported from src/math/mat2d.cpp Mat2D::mapPoints. The grouping matters
        // for cancellation-heavy local path composition.
        if b == 0.0 && c == 0.0 {
            (a.mul_add(x, e), d.mul_add(y, f))
        } else {
            (a.mul_add(x, c.mul_add(y, e)), d.mul_add(y, b.mul_add(x, f)))
        }
    }

    pub fn transform_direction(self, x: f32, y: f32) -> (f32, f32) {
        (self.0[0] * x + self.0[2] * y, self.0[1] * x + self.0[3] * y)
    }
}

impl Default for Mat2D {
    fn default() -> Self {
        Self::IDENTITY
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct TransformComponents {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) scale_x: f32,
    pub(crate) scale_y: f32,
    pub(crate) rotation: f32,
    pub(crate) skew: f32,
}

impl Default for TransformComponents {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
            skew: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeComponent {
    pub local_id: usize,
    pub global_id: u32,
    pub type_name: &'static str,
    pub capabilities: RuntimeComponentCapabilities,
    pub parent_local: Option<usize>,
    pub constraint_locals: Vec<usize>,
    pub dependent_locals: Vec<usize>,
    pub layout_chain_has_layout_component: bool,
    pub constrained_layout_ancestor: Option<usize>,
    pub graph_order: usize,
    pub dirt: ComponentDirt,
    pub transform: TransformRuntimeState,
}

impl RuntimeComponent {
    pub(crate) fn from_graph_component(component: &ComponentNode) -> Self {
        Self {
            local_id: component.local_id,
            global_id: component.global_id,
            type_name: component.type_name,
            capabilities: RuntimeComponentCapabilities {
                world_transform: component.capabilities.world_transform,
                transform: component.capabilities.transform,
            },
            parent_local: component.parent_local,
            constraint_locals: component.constraint_locals.clone(),
            dependent_locals: component.dependent_locals.clone(),
            layout_chain_has_layout_component: false,
            constrained_layout_ancestor: None,
            graph_order: component.graph_order.unwrap_or(0),
            dirt: ComponentDirt::FILTHY,
            transform: TransformRuntimeState::default(),
        }
    }

    pub fn is_collapsed(&self) -> bool {
        self.dirt.contains(ComponentDirt::COLLAPSED)
    }

    pub(crate) fn update_transform(&mut self, authored: AuthoredTransform) {
        if !self.capabilities.transform {
            return;
        }

        let mut transform = Mat2D::from_rotation(authored.rotation);
        transform.0[4] = authored.x;
        transform.0[5] = authored.y;
        transform.scale_by_values(authored.scale_x, authored.scale_y);
        self.transform.local_transform = transform;
    }

    pub(crate) fn update_world_transform(&mut self, parent_world: Option<Mat2D>) {
        if self.type_name == "Artboard" || !self.capabilities.transform {
            return;
        }

        self.transform.world_transform = match parent_world {
            Some(parent_world) => parent_world.multiply(self.transform.local_transform),
            None => self.transform.local_transform,
        };
    }

    pub(crate) fn update_render_opacity(&mut self, opacity: f32, parent_opacity: f32) {
        if !self.capabilities.transform {
            return;
        }

        self.transform.render_opacity = opacity * parent_opacity;
    }
}

pub(crate) fn retain_runtime_component_layout_topology(
    components: &mut [RuntimeComponent],
    component_by_local: &BTreeMap<usize, usize>,
) {
    for index in 0..components.len() {
        let local_id = components[index].local_id;
        components[index].layout_chain_has_layout_component =
            runtime_layout_chain_has_layout_component(local_id, components, component_by_local);
        components[index].constrained_layout_ancestor =
            runtime_constrained_layout_ancestor(local_id, components, component_by_local);
    }
}

fn runtime_layout_chain_has_layout_component(
    mut local_id: usize,
    components: &[RuntimeComponent],
    component_by_local: &BTreeMap<usize, usize>,
) -> bool {
    while let Some(component) = component_by_local
        .get(&local_id)
        .and_then(|index| components.get(*index))
    {
        if component.type_name == "LayoutComponent" {
            return true;
        }
        let Some(parent_local) = component.parent_local else {
            return false;
        };
        local_id = parent_local;
    }
    false
}

fn runtime_constrained_layout_ancestor(
    mut local_id: usize,
    components: &[RuntimeComponent],
    component_by_local: &BTreeMap<usize, usize>,
) -> Option<usize> {
    let mut saw_constraint = false;
    while let Some(component) = component_by_local
        .get(&local_id)
        .and_then(|index| components.get(*index))
    {
        if component.type_name == "LayoutComponent" {
            return saw_constraint.then_some(local_id);
        }
        saw_constraint |= !component.constraint_locals.is_empty();
        let Some(parent_local) = component.parent_local else {
            return None;
        };
        local_id = parent_local;
    }
    None
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeSolo {
    pub(crate) local_id: usize,
    pub(crate) active_component_property_key: u16,
    pub(crate) runtime_local_by_cpp_local: BTreeMap<usize, usize>,
    pub(crate) children: Vec<RuntimeSoloChild>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeSoloChild {
    pub(crate) local_id: usize,
    pub(crate) participates: bool,
}

pub(crate) fn build_runtime_solos(file: &RuntimeFile, graph: &ArtboardGraph) -> Vec<RuntimeSolo> {
    let Some(active_component_property_key) = property_key_for_name("Solo", "activeComponentId")
    else {
        return Vec::new();
    };
    let runtime_local_by_cpp_local = artboard_index_for_graph(file, graph)
        .map(|artboard_index| runtime_local_by_cpp_artboard_local(file, graph, artboard_index))
        .unwrap_or_default();

    graph
        .components
        .iter()
        .filter(|component| component.type_name == "Solo")
        .map(|solo| RuntimeSolo {
            local_id: solo.local_id,
            active_component_property_key,
            runtime_local_by_cpp_local: runtime_local_by_cpp_local.clone(),
            children: solo
                .children
                .iter()
                .map(|child_local| RuntimeSoloChild {
                    local_id: *child_local,
                    participates: runtime_solo_child_participates(graph, *child_local),
                })
                .collect(),
        })
        .collect()
}

// Mirrors src/solo.cpp Solo::propagateCollapse for the imported static state.
pub(crate) fn apply_initial_solo_collapses(
    objects: &InstanceObjectArena,
    solos: &[RuntimeSolo],
    components: &mut [RuntimeComponent],
    component_by_local: &BTreeMap<usize, usize>,
) {
    for solo in solos {
        let solo_collapsed = component_by_local
            .get(&solo.local_id)
            .is_some_and(|index| components[*index].is_collapsed());
        let active_local = objects
            .uint_property(solo.local_id, solo.active_component_property_key)
            .and_then(|id| usize::try_from(id).ok())
            .and_then(|active_index| solo.runtime_local_by_cpp_local.get(&active_index).copied());

        for child_local in &solo.children {
            let collapsed = if child_local.participates {
                solo_collapsed || Some(child_local.local_id) != active_local
            } else {
                solo_collapsed
            };
            if let Some(index) = component_by_local.get(&child_local.local_id).copied() {
                set_runtime_component_collapsed(&mut components[index], collapsed);
            }
        }
    }
}

fn runtime_solo_child_participates(graph: &ArtboardGraph, child_local: usize) -> bool {
    let Some(child) = graph
        .components
        .iter()
        .find(|component| component.local_id == child_local)
    else {
        return true;
    };
    let Some(definition) = definition_by_name(child.type_name) else {
        return true;
    };
    !definition.is_a("Constraint") && !definition.is_a("ClippingShape")
}

fn runtime_graph_local_for_cpp_artboard_local(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    artboard_index: usize,
    local_index: usize,
) -> Option<usize> {
    let object = file.artboard_local_object(artboard_index, local_index)?;
    graph
        .local_objects
        .iter()
        .find(|local_object| local_object.global_id == object.id)
        .map(|local_object| local_object.local_id)
}

fn runtime_local_by_cpp_artboard_local(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    artboard_index: usize,
) -> BTreeMap<usize, usize> {
    (0..graph.local_objects.len())
        .filter_map(|local_index| {
            runtime_graph_local_for_cpp_artboard_local(file, graph, artboard_index, local_index)
                .map(|runtime_local| (local_index, runtime_local))
        })
        .collect()
}

fn set_runtime_component_collapsed(component: &mut RuntimeComponent, collapsed: bool) {
    if collapsed {
        component.dirt |= ComponentDirt::COLLAPSED;
    } else {
        component.dirt &= !ComponentDirt::COLLAPSED;
    }
}
