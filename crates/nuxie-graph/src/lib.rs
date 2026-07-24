use anyhow::{Context, Result};
use nuxie_binary::{
    FieldValue, RuntimeArtboardGeometry, RuntimeFile, RuntimeMesh, RuntimeNSlicerDetails,
    RuntimeObject, RuntimePath, RuntimeShape, RuntimeShapePaintContainer,
};
use nuxie_schema::{definition_by_type_key, object_supports_property};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize)]
pub struct GraphFile {
    pub file_assets: Vec<FileAssetNode>,
    pub view_models: Vec<ViewModelGraph>,
    pub enums: Vec<DataEnumGraph>,
    pub artboards: Vec<ArtboardGraph>,
}

impl GraphFile {
    pub fn from_runtime_file(file: &RuntimeFile) -> Result<Self> {
        let artboard_ranges = artboard_ranges(file);
        let artboards = artboard_ranges
            .into_iter()
            .enumerate()
            .map(|(artboard_index, range)| ArtboardGraph::build(file, artboard_index, range))
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            file_assets: file_assets(file),
            view_models: view_models(file),
            enums: data_enums(file),
            artboards,
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct ArtboardRange {
    start: usize,
    end: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArtboardGraph {
    pub name: Option<String>,
    pub global_id: u32,
    pub local_objects: Vec<LocalObject>,
    pub components: Vec<ComponentNode>,
    pub dependency_nodes: Vec<DependencyNode>,
    pub dependency_edges: Vec<DependencyEdge>,
    pub dependency_node_edges: Vec<DependencyNodeEdge>,
    pub dependency_cycles: Vec<DependencyCycle>,
    pub dependency_node_cycles: Vec<DependencyNodeCycle>,
    pub draw_targets: Vec<DrawTargetNode>,
    pub draw_rules: Vec<DrawRulesNode>,
    pub draw_target_dependency_edges: Vec<DrawTargetDependencyEdge>,
    pub draw_target_order: Vec<usize>,
    pub draw_target_cycles: Vec<DrawTargetCycle>,
    pub drawable_order: Vec<DrawableOrderNode>,
    pub sorted_drawable_order: Vec<SortedDrawableNode>,
    pub clipping_shapes: Vec<ClippingShapeNode>,
    pub path_composers: Vec<PathComposerNode>,
    pub meshes: Vec<MeshGeometryNode>,
    pub paths: Vec<PathGeometryNode>,
    pub shape_paint_containers: Vec<ShapePaintContainerNode>,
    pub n_slicer_details: Vec<NSlicerDetailsNode>,
    pub shape_deformers: Vec<ShapeDeformerNode>,
    pub skeletal_bones: Vec<SkeletalBoneNode>,
    pub skeletal_skins: Vec<SkeletalSkinNode>,
    pub text_variation_helpers: Vec<TextVariationHelperNode>,
    pub list_constraint_registrations: Vec<ListConstraintRegistrationNode>,
    pub layout_constraint_registrations: Vec<LayoutConstraintRegistrationNode>,
    pub nested_artboards: Vec<NestedArtboardNode>,
    pub component_lists: Vec<ComponentListNode>,
    pub artboard_hosts: Vec<ArtboardHostNode>,
    pub joysticks: Vec<JoystickNode>,
    pub joysticks_apply_before_update: bool,
    pub resetting_components: Vec<ResettingComponentNode>,
    pub advancing_components: Vec<AdvancingComponentNode>,
    pub data_binds: Vec<DataBindNode>,
    pub animations: Vec<AnimationGraph>,
    pub state_machines: Vec<StateMachineGraph>,
    pub dependency_order: Vec<usize>,
    pub dependency_insertion_order: Vec<usize>,
    /// Root-reachable C++ dependency traversal, including embedded nodes such
    /// as PathComposer and TextVariationHelper.
    pub runtime_dependency_node_order: Vec<usize>,
    /// Complete dependency-node order, including unattached components kept
    /// for diagnostics and graph inspection.
    pub dependency_node_order: Vec<usize>,
    pub diagnostics: Vec<GraphDiagnostic>,
    pub lifecycle: LifecycleSummary,
}

impl ArtboardGraph {
    fn build(file: &RuntimeFile, artboard_index: usize, range: ArtboardRange) -> Result<Self> {
        let mut local_objects = Vec::new();
        for global_id in range.start..range.end {
            let object = file
                .objects
                .get(global_id)
                .context("object range out of bounds")?;
            if !is_artboard_object(object.as_ref()) {
                continue;
            }

            local_objects.push(LocalObject {
                local_id: local_objects.len(),
                global_id: global_id as u32,
                type_name: object.as_ref().map(|object| object.type_name),
                name: object.as_ref().and_then(object_name),
            });
        }
        validate_local_objects(file, &mut local_objects);

        let mut component_by_local = BTreeMap::new();
        let mut components = Vec::new();

        for local_object in &local_objects {
            if local_object.type_name.is_none() {
                continue;
            }

            let object = file
                .objects
                .get(local_object.global_id as usize)
                .and_then(|object| object.as_ref());
            let Some(object) = object else {
                continue;
            };
            if !is_component(object) {
                continue;
            }

            let parent_local = if object.type_name == "Artboard" {
                None
            } else {
                object_parent_id(object).map(|parent| parent as usize)
            };

            let component_index = components.len();
            component_by_local.insert(local_object.local_id, component_index);
            let capabilities = capabilities(object);
            components.push(ComponentNode {
                local_id: local_object.local_id,
                global_id: local_object.global_id,
                type_name: object.type_name,
                name: object_name(object),
                capabilities,
                parent_local,
                parent_global: None,
                children: Vec::new(),
                constraint_locals: Vec::new(),
                dependent_locals: Vec::new(),
                graph_order: None,
                missing_parent: false,
            });
        }

        let mut lifecycle = LifecycleSummary::default();
        lifecycle.imported_slots = local_objects.len();
        lifecycle.on_added_dirty_resolved = resolve_parents(&mut components);
        lifecycle.on_added_clean_indexed =
            index_children(file, &local_objects, &mut components, &component_by_local);
        index_transform_constraints(file, &local_objects, &mut components, &component_by_local);
        let draw_targets = draw_targets(file, &local_objects);
        let draw_rules = draw_rules(file, &local_objects);
        let drawable_order = drawable_order(file, &local_objects);
        let draw_target_order =
            draw_target_order(file, &local_objects, &draw_targets, &drawable_order);
        let clipping_shapes =
            clipping_shapes(file, &local_objects, &components, &component_by_local);
        let sorted_drawable_order = sorted_drawable_order(
            &drawable_order,
            &draw_targets,
            &draw_rules,
            &draw_target_order.local_ids,
            &clipping_shapes,
        );
        lifecycle.post_build_dependencies_draw_target_edges =
            draw_target_order.dependency_edges.len();
        lifecycle.draw_target_cycles = draw_target_order.cycles.len();
        let RuntimeArtboardGeometry {
            meshes: runtime_meshes,
            paths: runtime_paths,
            shapes: runtime_shapes,
            shape_paint_containers: runtime_shape_paint_containers,
            n_slicer_details: runtime_n_slicer_details,
        } = file
            .artboard_geometry(artboard_index)
            .context("missing runtime artboard geometry")?;
        let path_composers = path_composers(runtime_shapes, &local_objects);
        let meshes = meshes(runtime_meshes, &local_objects);
        let paths = paths(runtime_paths, &local_objects);
        let shape_paint_containers =
            shape_paint_containers(file, runtime_shape_paint_containers, &local_objects);
        let n_slicer_details = n_slicer_details(runtime_n_slicer_details, &local_objects);
        let shape_deformers = shape_deformers(file, &local_objects);
        let skeletal_bones = skeletal_bones(file, &local_objects);
        let skeletal_skins = skeletal_skins(file, &local_objects);
        let text_variation_helpers = text_variation_helpers(file, &local_objects);
        let list_constraint_registrations = list_constraint_registrations(file, &local_objects);
        let layout_constraint_registrations =
            layout_constraint_registrations(file, &local_objects, &components);
        let nested_artboards = nested_artboards(file, &local_objects);
        let component_lists = component_lists(file, &local_objects);
        let artboard_hosts = artboard_hosts(file, &local_objects);
        let resetting_components = resetting_components(file, &local_objects);
        let advancing_components = advancing_components(file, &local_objects);
        let dependency_edges_in_insertion_order = build_dependency_edges(
            file,
            &local_objects,
            &components,
            &draw_targets,
            &draw_rules,
            &clipping_shapes,
        );
        let mut dependency_edges = dependency_edges_in_insertion_order.clone();
        sort_dependency_edges(&mut dependency_edges);
        lifecycle.build_dependencies_edges = dependency_edges.len();
        let dependency_nodes =
            build_dependency_nodes(&components, &path_composers, &text_variation_helpers);
        lifecycle.build_dependencies_nodes = dependency_nodes.len();
        let dependency_node_edges_in_insertion_order = build_dependency_node_edges(
            file,
            &local_objects,
            &dependency_nodes,
            &dependency_edges_in_insertion_order,
            &path_composers,
            &clipping_shapes,
            &text_variation_helpers,
        );
        let mut dependency_node_edges = dependency_node_edges_in_insertion_order.clone();
        sort_dependency_node_edges(&mut dependency_node_edges);
        lifecycle.build_dependencies_node_edges = dependency_node_edges.len();
        index_component_dependents(
            &mut components,
            &dependency_nodes,
            &dependency_node_edges,
            &draw_target_order.dependency_edges,
        );
        let mut insertion_components = components.clone();
        let dependency_insertion_order = build_dependency_order(
            &mut insertion_components,
            &component_by_local,
            &dependency_nodes,
            &dependency_node_edges_in_insertion_order,
        );
        let dependency_order = build_dependency_order(
            &mut components,
            &component_by_local,
            &dependency_nodes,
            &dependency_node_edges,
        );
        lifecycle.dependency_cycles = dependency_order.cycles.len();
        lifecycle.dependency_node_cycles = dependency_order.node_cycles.len();
        let diagnostics = graph_diagnostics(
            &components,
            &draw_targets,
            &draw_rules,
            &clipping_shapes,
            &dependency_order.cycles,
            &dependency_order.node_cycles,
            &draw_target_order.cycles,
        );

        let artboard = file.objects[range.start]
            .as_ref()
            .context("artboard range does not start with an artboard")?;

        let data_binds = artboard_data_binds(file, artboard_index);
        let animations = animations(file, range, &local_objects);
        let joysticks = joysticks(file, &local_objects, &animations);
        let joysticks_apply_before_update = joysticks
            .iter()
            .all(|joystick| joystick.can_apply_before_update);
        let mut state_machines = state_machines(file, artboard_index);
        if animations.is_empty() && state_machines.is_empty() {
            state_machines.push(StateMachineGraph::auto_generated());
        }

        Ok(Self {
            name: object_name(artboard),
            global_id: range.start as u32,
            local_objects,
            components,
            dependency_nodes,
            dependency_edges,
            dependency_node_edges,
            dependency_cycles: dependency_order.cycles,
            dependency_node_cycles: dependency_order.node_cycles,
            draw_targets,
            draw_rules,
            draw_target_dependency_edges: draw_target_order.dependency_edges,
            draw_target_order: draw_target_order.local_ids,
            draw_target_cycles: draw_target_order.cycles,
            drawable_order,
            sorted_drawable_order,
            clipping_shapes,
            path_composers,
            meshes,
            paths,
            shape_paint_containers,
            n_slicer_details,
            shape_deformers,
            skeletal_bones,
            skeletal_skins,
            text_variation_helpers,
            list_constraint_registrations,
            layout_constraint_registrations,
            nested_artboards,
            component_lists,
            artboard_hosts,
            joysticks,
            joysticks_apply_before_update,
            resetting_components,
            advancing_components,
            data_binds,
            animations,
            state_machines,
            dependency_order: dependency_order.component_order,
            dependency_insertion_order: dependency_insertion_order.component_order,
            runtime_dependency_node_order: dependency_order.runtime_node_order,
            dependency_node_order: dependency_order.node_order,
            diagnostics,
            lifecycle,
        })
    }

    pub fn component_named(&self, name: &str) -> Option<&ComponentNode> {
        self.components
            .iter()
            .find(|component| component.name.as_deref() == Some(name))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LocalObject {
    pub local_id: usize,
    pub global_id: u32,
    pub type_name: Option<&'static str>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComponentNode {
    pub local_id: usize,
    pub global_id: u32,
    pub type_name: &'static str,
    pub name: Option<String>,
    pub capabilities: ComponentCapabilities,
    pub parent_local: Option<usize>,
    pub parent_global: Option<u32>,
    pub children: Vec<usize>,
    pub constraint_locals: Vec<usize>,
    pub dependent_locals: Vec<usize>,
    pub graph_order: Option<usize>,
    pub missing_parent: bool,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct ComponentCapabilities {
    pub artboard: bool,
    pub world_transform: bool,
    pub transform: bool,
    pub drawable: bool,
    pub container: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DependencyNode {
    pub node_id: usize,
    pub kind: DependencyNodeKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GraphDiagnostic {
    MissingParent {
        component_local: usize,
        parent_local: usize,
    },
    UnresolvedDrawTargetDrawable {
        draw_target_local: usize,
        drawable_id: u64,
    },
    UnresolvedDrawRulesTarget {
        draw_rules_local: usize,
        draw_target_id: u64,
    },
    UnresolvedClippingSource {
        clipping_shape_local: usize,
        source_id: u64,
    },
    DependencyCycle {
        local_ids: Vec<usize>,
    },
    DependencyNodeCycle {
        node_ids: Vec<usize>,
    },
    DrawTargetCycle {
        local_ids: Vec<usize>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DependencyNodeKind {
    Component {
        local_id: usize,
        global_id: u32,
        type_name: &'static str,
        name: Option<String>,
    },
    PathComposer {
        shape_local: usize,
        shape_global: u32,
    },
    TextVariationHelper {
        text_style_local: usize,
        text_style_global: u32,
        text_local: usize,
        text_global: u32,
        artboard_local: usize,
        artboard_global: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyKind {
    ParentChild,
    TargetedConstraint,
    IkConstraintTarget,
    IkConstraintTipChild,
    DrawTargetDrawable,
    DrawRulesTarget,
    ClippingSource,
    SkinMesh,
    SkinPointsPath,
    SkinBone,
    SkinBoneConstraintParent,
    JoystickParent,
    JoystickHandleSource,
    ScrollBarConstraint,
    ScrollConstraintLayoutChild,
    PathComposerShape,
    PathComposerPath,
    ClippingShapePathComposer,
    FollowPathConstraintParent,
    FollowPathConstraintTargetPathComposer,
    FollowPathConstraintTargetPath,
    TextFollowPathModifierText,
    TextFollowPathModifierTargetPathComposer,
    TextFollowPathModifierTargetPath,
    StrokePathBuilder,
    FillPathBuilder,
    FeatherPathBuilder,
    GroupEffectParent,
    ScriptedPathEffectParent,
    LinearGradientPaintContainer,
    TextVariationHelperArtboard,
    TextVariationHelperText,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct DependencyEdge {
    pub source_local: usize,
    pub dependent_local: usize,
    pub kind: DependencyKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct DependencyNodeEdge {
    pub source_node: usize,
    pub dependent_node: usize,
    pub kind: DependencyKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DependencyCycle {
    pub local_ids: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DependencyNodeCycle {
    pub node_ids: Vec<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DrawTargetNode {
    pub local_id: usize,
    pub global_id: u32,
    pub drawable_id: u64,
    pub drawable_local: Option<usize>,
    pub placement_value: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DrawRulesNode {
    pub local_id: usize,
    pub global_id: u32,
    pub draw_target_id: u64,
    pub active_target_local: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DrawTargetDependencyKind {
    RootRuleTarget,
    FlattenedRuleTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct DrawTargetDependencyEdge {
    pub source_local: Option<usize>,
    pub dependent_local: usize,
    pub kind: DrawTargetDependencyKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DrawTargetCycle {
    pub local_ids: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DrawableOrderKind {
    Drawable,
    LayoutProxy,
    ClipStartProxy,
    ClipEndProxy,
}

#[derive(Debug, Clone, Serialize)]
pub struct DrawableOrderNode {
    pub kind: DrawableOrderKind,
    pub local_id: Option<usize>,
    pub global_id: Option<u32>,
    pub type_name: &'static str,
    pub is_hidden: bool,
    pub resolved_image_asset_global: Option<u32>,
    pub referenced_artboard_global: Option<u32>,
    pub layout_local: Option<usize>,
    pub layout_global: Option<u32>,
    pub flattened_draw_rules_local: Option<usize>,
    pub flattened_draw_rules_global: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SortedDrawableNode {
    pub kind: DrawableOrderKind,
    pub local_id: Option<usize>,
    pub global_id: Option<u32>,
    pub type_name: &'static str,
    pub is_hidden: bool,
    pub resolved_image_asset_global: Option<u32>,
    pub referenced_artboard_global: Option<u32>,
    pub layout_local: Option<usize>,
    pub layout_global: Option<u32>,
    pub draw_target_local: Option<usize>,
    pub clipping_shape_local: Option<usize>,
    pub clipping_shape_global: Option<u32>,
    pub needs_save_operation: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClippingShapeNode {
    pub local_id: usize,
    pub global_id: u32,
    pub source_id: u64,
    pub source_local: Option<usize>,
    pub fill_rule: u64,
    pub is_visible: bool,
    pub shape_locals: Vec<usize>,
    pub clipped_drawable_locals: Vec<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PathComposerNode {
    pub shape_local: usize,
    pub shape_global: u32,
    pub path_locals: Vec<usize>,
    pub path_globals: Vec<u32>,
    pub paths: Vec<PathComposerPathNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PathComposerPathNode {
    pub local_id: usize,
    pub global_id: u32,
    pub is_hidden: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct MeshGeometryNode {
    pub local_id: usize,
    pub global_id: u32,
    pub type_name: &'static str,
    pub vertices: Vec<MeshVertexNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MeshVertexNode {
    pub local_id: usize,
    pub global_id: u32,
    pub type_name: &'static str,
    pub weight_local: Option<usize>,
    pub weight_global: Option<u32>,
    pub weight_type_name: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PathGeometryNode {
    pub local_id: usize,
    pub global_id: u32,
    pub type_name: &'static str,
    pub is_closed: bool,
    pub is_hole: bool,
    pub is_clockwise: bool,
    pub parametric: Option<ParametricPathNode>,
    pub vertices: Vec<PathVertexNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ParametricPathNode {
    Ellipse {
        width: f32,
        height: f32,
        origin_x: f32,
        origin_y: f32,
    },
    Polygon {
        width: f32,
        height: f32,
        origin_x: f32,
        origin_y: f32,
        points: u32,
        corner_radius: f32,
    },
    Star {
        width: f32,
        height: f32,
        origin_x: f32,
        origin_y: f32,
        points: u32,
        corner_radius: f32,
        inner_radius: f32,
    },
    Triangle {
        width: f32,
        height: f32,
        origin_x: f32,
        origin_y: f32,
    },
    Rectangle {
        width: f32,
        height: f32,
        origin_x: f32,
        origin_y: f32,
        link_corner_radius: bool,
        corner_radius_tl: f32,
        corner_radius_tr: f32,
        corner_radius_bl: f32,
        corner_radius_br: f32,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct PathVertexNode {
    pub local_id: usize,
    pub global_id: u32,
    pub type_name: &'static str,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub rotation: f32,
    pub distance: f32,
    pub in_rotation: f32,
    pub in_distance: f32,
    pub out_rotation: f32,
    pub out_distance: f32,
    pub weight_local: Option<usize>,
    pub weight_global: Option<u32>,
    pub weight_type_name: Option<&'static str>,
    pub weight_values: Option<u32>,
    pub weight_indices: Option<u32>,
    pub weight_in_values: Option<u32>,
    pub weight_in_indices: Option<u32>,
    pub weight_out_values: Option<u32>,
    pub weight_out_indices: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShapePaintContainerNode {
    pub local_id: usize,
    pub global_id: u32,
    pub type_name: &'static str,
    pub blend_mode_value: u32,
    pub paints: Vec<ShapePaintNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShapePaintNode {
    pub local_id: usize,
    pub global_id: u32,
    pub type_name: &'static str,
    pub paint_type: ShapePaintKind,
    pub is_visible: bool,
    pub blend_mode_value: u32,
    pub fill_rule: u64,
    pub path_kind: Option<ShapePaintPathKind>,
    pub paint_state: Option<ShapePaintStateNode>,
    pub mutator_local: Option<usize>,
    pub mutator_global: Option<u32>,
    pub mutator_type_name: Option<&'static str>,
    pub feather: Option<FeatherNode>,
    pub feather_local: Option<usize>,
    pub feather_global: Option<u32>,
    pub feather_type_name: Option<&'static str>,
    pub effects: Vec<StrokeEffectNode>,
    pub gradient_stops: Vec<GradientStopNode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ShapePaintKind {
    Fill,
    Stroke,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ShapePaintPathKind {
    Local,
    LocalClockwise,
    World,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ShapePaintStateNode {
    SolidColor {
        color: u32,
    },
    LinearGradient {
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
        opacity: f32,
        stops: Vec<GradientStopNode>,
    },
    RadialGradient {
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
        opacity: f32,
        stops: Vec<GradientStopNode>,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct StrokeEffectNode {
    pub local_id: usize,
    pub global_id: u32,
    pub type_name: &'static str,
    pub trim_start: Option<f32>,
    pub trim_end: Option<f32>,
    pub trim_offset: Option<f32>,
    pub trim_mode_value: Option<u32>,
    pub dash_offset: Option<f32>,
    pub dash_offset_is_percentage: Option<bool>,
    pub dashes: Vec<DashNode>,
    pub target_group_effect_local: Option<usize>,
    pub target_group_effect_global: Option<u32>,
    pub target_group_effect_type_name: Option<&'static str>,
    pub group_effects: Vec<StrokeEffectNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashNode {
    pub local_id: usize,
    pub global_id: u32,
    pub length: f32,
    pub length_is_percentage: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GradientStopNode {
    pub local_id: usize,
    pub global_id: u32,
    pub type_name: &'static str,
    pub color: u32,
    pub position: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FeatherNode {
    pub local_id: usize,
    pub global_id: u32,
    pub type_name: &'static str,
    pub space_value: u32,
    pub strength: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub inner: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct NSlicerDetailsNode {
    pub local_id: usize,
    pub global_id: u32,
    pub type_name: &'static str,
    pub x_axes: Vec<NSlicerAxisNode>,
    pub y_axes: Vec<NSlicerAxisNode>,
    pub tile_modes: Vec<NSlicerTileModeNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NSlicerAxisNode {
    pub local_id: usize,
    pub global_id: u32,
    pub type_name: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct NSlicerTileModeNode {
    pub local_id: usize,
    pub global_id: u32,
    pub type_name: &'static str,
    pub patch_index: u64,
    pub style: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShapeDeformerNode {
    pub shape_local: usize,
    pub shape_global: u32,
    pub deformer_local: Option<usize>,
    pub deformer_global: Option<u32>,
    pub deformer_type_name: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkeletalBoneNode {
    pub local_id: usize,
    pub global_id: u32,
    pub type_name: &'static str,
    pub child_bone_locals: Vec<usize>,
    pub child_bone_globals: Vec<u32>,
    pub peer_constraint_locals: Vec<usize>,
    pub peer_constraint_globals: Vec<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkeletalSkinNode {
    pub skin_local: usize,
    pub skin_global: u32,
    pub world_transform: [f32; 6],
    pub skinnable_local: Option<usize>,
    pub skinnable_global: Option<u32>,
    pub skinnable_type_name: Option<&'static str>,
    pub tendons: Vec<SkeletalTendonNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkeletalTendonNode {
    pub tendon_local: usize,
    pub tendon_global: u32,
    pub bone_local: Option<usize>,
    pub bone_global: Option<u32>,
    pub bone_type_name: Option<&'static str>,
    pub inverse_bind: [f32; 6],
}

#[derive(Debug, Clone, Serialize)]
pub struct TextVariationHelperNode {
    pub text_style_local: usize,
    pub text_style_global: u32,
    pub text_local: usize,
    pub text_global: u32,
    pub artboard_local: usize,
    pub artboard_global: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListConstraintRegistrationNode {
    pub constrainable_list_local: usize,
    pub constrainable_list_global: u32,
    pub constraint_local: usize,
    pub constraint_global: u32,
    pub constraint_type_name: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct LayoutConstraintRegistrationNode {
    pub layout_provider_local: usize,
    pub layout_provider_global: u32,
    pub layout_provider_type_name: &'static str,
    pub constraint_local: usize,
    pub constraint_global: u32,
    pub constraint_type_name: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct NestedArtboardNode {
    pub local_id: usize,
    pub global_id: u32,
    pub type_name: &'static str,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComponentListNode {
    pub local_id: usize,
    pub global_id: u32,
    pub type_name: &'static str,
    pub name: Option<String>,
    pub map_rules: Vec<ComponentListMapRuleNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComponentListMapRuleNode {
    pub view_model_id: i64,
    pub artboard_id: i64,
    /// Explicit state-machine ordinals authored beneath this mapping rule.
    /// Empty retains the legacy component-list default-machine behavior.
    pub state_machine_ids: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtboardHostKind {
    NestedArtboard,
    ComponentList,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArtboardHostNode {
    pub local_id: usize,
    pub global_id: u32,
    pub type_name: &'static str,
    pub name: Option<String>,
    pub kind: ArtboardHostKind,
}

#[derive(Debug, Clone, Serialize)]
pub struct JoystickNode {
    pub local_id: usize,
    pub global_id: u32,
    pub name: Option<String>,
    pub handle_source_local: Option<usize>,
    pub handle_source_global: Option<u32>,
    pub can_apply_before_update: bool,
    pub x_animation_global: Option<u32>,
    pub y_animation_global: Option<u32>,
    pub nested_remap_dependents: Vec<JoystickNestedRemapDependentNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JoystickNestedRemapDependentNode {
    pub local_id: usize,
    pub global_id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResettingComponentKind {
    NestedArtboard,
    ArtboardComponentList,
    CustomPropertyTrigger,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResettingComponentNode {
    pub local_id: usize,
    pub global_id: u32,
    pub type_name: &'static str,
    pub name: Option<String>,
    pub kind: ResettingComponentKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdvancingComponentKind {
    Artboard,
    NestedArtboard,
    LayoutComponent,
    ArtboardComponentList,
    ScrollConstraint,
    TextInput,
    ScriptedDataConverter,
    ScriptedDrawable,
    ScriptedLayout,
    ScriptedPathEffect,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdvancingComponentNode {
    pub local_id: usize,
    pub global_id: u32,
    pub type_name: &'static str,
    pub name: Option<String>,
    pub kind: AdvancingComponentKind,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileAssetNode {
    pub global_id: u32,
    pub type_name: &'static str,
    pub name: Option<String>,
    pub asset_id: u64,
    pub cdn_base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ViewModelGraph {
    pub global_id: u32,
    pub type_name: &'static str,
    pub name: Option<String>,
    pub properties: Vec<ViewModelPropertyNode>,
    pub instances: Vec<ViewModelInstanceNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ViewModelPropertyNode {
    pub global_id: u32,
    pub type_name: &'static str,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ViewModelInstanceNode {
    pub global_id: u32,
    pub name: Option<String>,
    pub view_model_id: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DataEnumGraph {
    pub global_id: u32,
    pub type_name: &'static str,
    pub name: Option<String>,
    pub values: Vec<DataEnumValueNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DataEnumValueNode {
    pub global_id: u32,
    pub key: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnimationGraph {
    pub global_id: u32,
    pub name: Option<String>,
    pub fps: u64,
    pub duration: u64,
    pub speed: f32,
    pub loop_value: u64,
    pub work_start: u64,
    pub work_end: u64,
    pub enable_work_area: bool,
    pub quantize: bool,
    pub keyed_objects: Vec<KeyedObjectGraph>,
}

#[derive(Debug, Clone, Serialize)]
pub struct KeyedObjectGraph {
    pub global_id: u32,
    pub object_id: u64,
    pub keyed_properties: Vec<KeyedPropertyGraph>,
}

#[derive(Debug, Clone, Serialize)]
pub struct KeyedPropertyGraph {
    pub global_id: u32,
    pub property_key: u64,
    pub first_key_frame: Option<KeyFrameRef>,
}

#[derive(Debug, Clone, Serialize)]
pub struct KeyFrameRef {
    pub global_id: u32,
    pub type_name: &'static str,
    pub frame: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StateMachineGraph {
    pub global_id: Option<u32>,
    pub name: Option<String>,
    pub layers: Vec<StateMachineLayerGraph>,
    pub inputs: Vec<StateMachineInputNode>,
    pub listeners: Vec<StateMachineListenerGraph>,
    pub data_binds: Vec<DataBindNode>,
    pub scripted_objects: Vec<StateMachineScriptedObjectNode>,
}

impl StateMachineGraph {
    fn auto_generated() -> Self {
        Self {
            global_id: None,
            name: Some("Auto Generated State Machine".to_owned()),
            layers: Vec::new(),
            inputs: Vec::new(),
            listeners: Vec::new(),
            data_binds: Vec::new(),
            scripted_objects: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct StateMachineLayerGraph {
    pub global_id: u32,
    pub name: Option<String>,
    pub state_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct StateMachineInputNode {
    pub global_id: u32,
    pub type_name: &'static str,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StateMachineListenerGraph {
    pub global_id: u32,
    pub type_name: &'static str,
    pub name: Option<String>,
    pub target_id: u64,
    pub action_count: usize,
    pub listener_input_type_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct StateMachineScriptedObjectNode {
    pub global_id: u32,
    pub type_name: &'static str,
    pub inputs: Vec<StateMachineScriptInputNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StateMachineScriptInputNode {
    pub global_id: u32,
    pub type_name: &'static str,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DataBindNode {
    pub global_id: u32,
    pub type_name: &'static str,
    pub property_key: u64,
    pub flags: u64,
    pub converter_id: u64,
    pub converter_global: Option<u32>,
    pub converter_type_name: Option<&'static str>,
    pub converter_duration: Option<f32>,
    pub target_global: Option<u32>,
    pub target_type_name: Option<&'static str>,
    pub target_local: Option<usize>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct LifecycleSummary {
    pub imported_slots: usize,
    pub on_added_dirty_resolved: usize,
    pub on_added_clean_indexed: usize,
    pub build_dependencies_nodes: usize,
    pub build_dependencies_edges: usize,
    pub build_dependencies_node_edges: usize,
    pub dependency_cycles: usize,
    pub dependency_node_cycles: usize,
    pub post_build_dependencies_draw_target_edges: usize,
    pub draw_target_cycles: usize,
}

fn artboard_ranges(file: &RuntimeFile) -> Vec<ArtboardRange> {
    let starts = file
        .objects
        .iter()
        .enumerate()
        .filter_map(|(index, object)| match object {
            Some(object) if object.type_name == "Artboard" => Some(index),
            _ => None,
        })
        .collect::<Vec<_>>();

    starts
        .iter()
        .enumerate()
        .map(|(index, start)| ArtboardRange {
            start: *start,
            end: starts.get(index + 1).copied().unwrap_or(file.objects.len()),
        })
        .collect()
}

fn file_assets(file: &RuntimeFile) -> Vec<FileAssetNode> {
    file.file_assets()
        .into_iter()
        .map(|object| FileAssetNode {
            global_id: object.id,
            type_name: object.type_name,
            name: object_name(object),
            asset_id: object.uint_property("assetId").unwrap_or(0),
            cdn_base_url: object.string_property("cdnBaseUrl").map(ToOwned::to_owned),
        })
        .collect()
}

fn view_models(file: &RuntimeFile) -> Vec<ViewModelGraph> {
    let first_backboard_id = file
        .objects
        .iter()
        .filter_map(Option::as_ref)
        .find(|object| object.type_name == "Backboard")
        .map(|object| object.id);

    file.view_models()
        .into_iter()
        .map(|view_model| ViewModelGraph {
            global_id: view_model.object.id,
            type_name: view_model.object.type_name,
            name: object_name(view_model.object),
            properties: view_model
                .properties
                .into_iter()
                .map(|property| ViewModelPropertyNode {
                    global_id: property.id,
                    type_name: property.type_name,
                    name: object_name(property),
                })
                .collect(),
            instances: view_model
                .instances
                .into_iter()
                // RuntimeFile also reads publisher-era instances serialized before
                // the Backboard. GraphFile is the strict C++ projection, where
                // ViewModelInstance::import requires a BackboardImporter.
                .filter(|instance| {
                    first_backboard_id.is_some_and(|backboard_id| instance.object.id > backboard_id)
                })
                .map(|instance| ViewModelInstanceNode {
                    global_id: instance.object.id,
                    name: object_name(instance.object),
                    view_model_id: instance.object.uint_property("viewModelId").unwrap_or(0),
                })
                .collect(),
        })
        .collect()
}

fn data_enums(file: &RuntimeFile) -> Vec<DataEnumGraph> {
    file.data_enums()
        .into_iter()
        .map(|item| DataEnumGraph {
            global_id: item.object.id,
            type_name: item.object.type_name,
            name: object_name(item.object),
            values: item
                .values
                .into_iter()
                .map(|value| DataEnumValueNode {
                    global_id: value.id,
                    key: value.string_property("key").map(ToOwned::to_owned),
                    value: value.string_property("value").map(ToOwned::to_owned),
                })
                .collect(),
        })
        .collect()
}

fn animations(
    file: &RuntimeFile,
    range: ArtboardRange,
    local_objects: &[LocalObject],
) -> Vec<AnimationGraph> {
    let mut animations = Vec::<AnimationGraph>::new();
    let mut current_animation = None;
    let mut current_keyed_object = None;
    let mut current_keyed_property = None;

    for (global_id, object) in file.objects[range.start..range.end]
        .iter()
        .enumerate()
        .filter_map(|(offset, object)| Some((range.start + offset, object.as_ref()?)))
    {
        let Some(definition) = definition_by_type_key(object.type_key) else {
            continue;
        };

        if definition.name == "LinearAnimation" {
            animations.push(AnimationGraph {
                global_id: global_id as u32,
                name: object_name(object),
                fps: object.uint_property("fps").unwrap_or(0),
                duration: object.uint_property("duration").unwrap_or(0),
                speed: object.double_property("speed").unwrap_or(0.0),
                loop_value: object.uint_property("loopValue").unwrap_or(0),
                work_start: object.uint_property("workStart").unwrap_or(0),
                work_end: object.uint_property("workEnd").unwrap_or(0),
                enable_work_area: object.bool_property("enableWorkArea").unwrap_or(false),
                quantize: object.bool_property("quantize").unwrap_or(false),
                keyed_objects: Vec::new(),
            });
            current_animation = Some(animations.len() - 1);
            current_keyed_object = None;
            current_keyed_property = None;
            continue;
        }

        if definition.name == "StateMachine" {
            current_animation = None;
            current_keyed_object = None;
            current_keyed_property = None;
            continue;
        }

        let Some(animation_index) = current_animation else {
            continue;
        };

        if definition.name == "KeyedObject" {
            if keyed_object_target(file, local_objects, object).is_none() {
                current_keyed_object = None;
                current_keyed_property = None;
                continue;
            }

            animations[animation_index]
                .keyed_objects
                .push(KeyedObjectGraph {
                    global_id: global_id as u32,
                    object_id: object.uint_property("objectId").unwrap_or(0),
                    keyed_properties: Vec::new(),
                });
            current_keyed_object = Some(animations[animation_index].keyed_objects.len() - 1);
            current_keyed_property = None;
            continue;
        }

        if definition.name == "KeyedProperty" {
            let Some(keyed_object_index) = current_keyed_object else {
                continue;
            };
            if !keyed_object_supports_property(
                file,
                local_objects,
                &animations[animation_index].keyed_objects[keyed_object_index],
                object,
            ) {
                current_keyed_property = None;
                continue;
            }

            animations[animation_index].keyed_objects[keyed_object_index]
                .keyed_properties
                .push(KeyedPropertyGraph {
                    global_id: global_id as u32,
                    property_key: object.uint_property("propertyKey").unwrap_or(0),
                    first_key_frame: None,
                });
            current_keyed_property = Some((
                keyed_object_index,
                animations[animation_index].keyed_objects[keyed_object_index]
                    .keyed_properties
                    .len()
                    - 1,
            ));
            continue;
        }

        if definition.is_a("KeyFrame") {
            let Some((keyed_object_index, keyed_property_index)) = current_keyed_property else {
                continue;
            };
            let first_key_frame = &mut animations[animation_index].keyed_objects
                [keyed_object_index]
                .keyed_properties[keyed_property_index]
                .first_key_frame;
            if first_key_frame.is_none() {
                *first_key_frame = Some(KeyFrameRef {
                    global_id: global_id as u32,
                    type_name: object.type_name,
                    frame: object.uint_property("frame").unwrap_or(0),
                });
            }
        }
    }

    animations
}

fn keyed_object_target<'a>(
    file: &'a RuntimeFile,
    local_objects: &[LocalObject],
    keyed_object: &RuntimeObject,
) -> Option<&'a RuntimeObject> {
    local_object_reference(file, local_objects, keyed_object.uint_property("objectId"))
}

fn keyed_object_supports_property(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    keyed_object: &KeyedObjectGraph,
    keyed_property: &RuntimeObject,
) -> bool {
    let Some(property_key) = keyed_property.uint_property("propertyKey") else {
        return false;
    };
    let Ok(property_key) = u16::try_from(property_key) else {
        return false;
    };
    let Some(target) = local_object_reference(file, local_objects, Some(keyed_object.object_id))
    else {
        return false;
    };

    object_supports_property(target.type_key, property_key)
}

fn state_machines(file: &RuntimeFile, artboard_index: usize) -> Vec<StateMachineGraph> {
    file.artboard_state_machine_graphs(artboard_index)
        .into_iter()
        .map(|state_machine| StateMachineGraph {
            global_id: Some(state_machine.object.id),
            name: object_name(state_machine.object),
            layers: state_machine
                .layers
                .into_iter()
                .map(|layer| StateMachineLayerGraph {
                    global_id: layer.object.id,
                    name: object_name(layer.object),
                    state_count: layer.state_count,
                })
                .collect(),
            inputs: state_machine
                .inputs
                .into_iter()
                .map(|input| StateMachineInputNode {
                    global_id: input.id,
                    type_name: input.type_name,
                    name: object_name(input),
                })
                .collect(),
            listeners: state_machine
                .listeners
                .into_iter()
                .map(|listener| StateMachineListenerGraph {
                    global_id: listener.object.id,
                    type_name: listener.object.type_name,
                    name: object_name(listener.object),
                    target_id: listener.object.uint_property("targetId").unwrap_or(0),
                    action_count: listener.actions.len(),
                    listener_input_type_count: listener.listener_input_types.len(),
                })
                .collect(),
            data_binds: state_machine
                .data_binds
                .into_iter()
                .map(|data_bind| data_bind_node(file, data_bind, None, None))
                .collect(),
            scripted_objects: state_machine
                .scripted_objects
                .into_iter()
                .map(|scripted_object| StateMachineScriptedObjectNode {
                    global_id: scripted_object.object.id,
                    type_name: scripted_object.object.type_name,
                    inputs: scripted_object
                        .inputs
                        .into_iter()
                        .map(|input| StateMachineScriptInputNode {
                            global_id: input.id,
                            type_name: input.type_name,
                            name: object_name(input),
                        })
                        .collect(),
                })
                .collect(),
        })
        .collect()
}

fn artboard_data_binds(file: &RuntimeFile, artboard_index: usize) -> Vec<DataBindNode> {
    let data_binds = file.artboard_data_binds(artboard_index);
    let data_bind_ids = data_binds
        .iter()
        .map(|data_bind| data_bind.object.id as usize)
        .collect::<Vec<_>>();
    let sorted_ids = file
        .sorted_data_bind_ids(&data_bind_ids)
        .unwrap_or(data_bind_ids);

    sorted_ids
        .into_iter()
        .filter_map(|data_bind_id| {
            data_binds
                .iter()
                .find(|data_bind| data_bind.object.id as usize == data_bind_id)
        })
        .map(|data_bind| {
            data_bind_node(
                file,
                data_bind.object,
                data_bind.target,
                data_bind.target_local_id,
            )
        })
        .collect()
}

fn data_bind_node(
    file: &RuntimeFile,
    data_bind: &RuntimeObject,
    target: Option<&RuntimeObject>,
    target_local: Option<usize>,
) -> DataBindNode {
    let converter = file.resolved_data_converter_for_data_bind_object(data_bind);

    DataBindNode {
        global_id: data_bind.id,
        type_name: data_bind.type_name,
        property_key: data_bind.uint_property("propertyKey").unwrap_or(0),
        flags: data_bind.uint_property("flags").unwrap_or(0),
        converter_id: data_bind.uint_property("converterId").unwrap_or(0),
        converter_global: converter.map(|converter| converter.id),
        converter_type_name: converter.map(|converter| converter.type_name),
        converter_duration: converter.and_then(|converter| converter.double_property("duration")),
        target_global: target.map(|target| target.id),
        target_type_name: target.map(|target| target.type_name),
        target_local,
    }
}

fn draw_targets(file: &RuntimeFile, local_objects: &[LocalObject]) -> Vec<DrawTargetNode> {
    local_objects
        .iter()
        .filter_map(|local_object| {
            let object = runtime_object_for_local(file, local_objects, local_object.local_id)?;
            if object.type_name != "DrawTarget" {
                return None;
            }

            let drawable_id = object
                .uint_property("drawableId")
                .unwrap_or(u64::from(u32::MAX));
            let drawable_local =
                local_object_reference_with_local_id(file, local_objects, Some(drawable_id))
                    .and_then(|(local_id, target)| is_drawable(target).then_some(local_id));

            Some(DrawTargetNode {
                local_id: local_object.local_id,
                global_id: local_object.global_id,
                drawable_id,
                drawable_local,
                placement_value: object.uint_property("placementValue").unwrap_or(0),
            })
        })
        .collect()
}

fn draw_rules(file: &RuntimeFile, local_objects: &[LocalObject]) -> Vec<DrawRulesNode> {
    local_objects
        .iter()
        .filter_map(|local_object| {
            let object = runtime_object_for_local(file, local_objects, local_object.local_id)?;
            if object.type_name != "DrawRules" {
                return None;
            }

            let draw_target_id = object
                .uint_property("drawTargetId")
                .unwrap_or(u64::from(u32::MAX));
            let active_target_local =
                local_object_reference_with_local_id(file, local_objects, Some(draw_target_id))
                    .and_then(|(local_id, target)| {
                        (target.type_name == "DrawTarget").then_some(local_id)
                    });

            Some(DrawRulesNode {
                local_id: local_object.local_id,
                global_id: local_object.global_id,
                draw_target_id,
                active_target_local,
            })
        })
        .collect()
}

#[derive(Debug)]
struct DrawTargetOrder {
    dependency_edges: Vec<DrawTargetDependencyEdge>,
    local_ids: Vec<usize>,
    cycles: Vec<DrawTargetCycle>,
}

fn draw_target_order(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    draw_targets: &[DrawTargetNode],
    drawable_order: &[DrawableOrderNode],
) -> DrawTargetOrder {
    let component_draw_rules = component_draw_rules_by_parent(file, local_objects);
    let resolved_draw_target_locals = draw_targets
        .iter()
        .filter_map(|target| target.drawable_local.map(|_| target.local_id))
        .collect::<BTreeSet<_>>();
    let draw_target_locals_by_rules =
        draw_target_locals_by_parent_rules(file, local_objects, &resolved_draw_target_locals);
    let flattened_rules_by_drawable = flattened_rules_by_drawable(drawable_order);
    let draw_targets_by_local = draw_targets
        .iter()
        .map(|target| (target.local_id, target))
        .collect::<BTreeMap<_, _>>();
    let mut dependency_edges = Vec::new();

    for local_object in local_objects {
        let Some(rules_local) = component_draw_rules.get(&local_object.local_id).copied() else {
            continue;
        };
        let Some(rule_targets) = draw_target_locals_by_rules.get(&rules_local) else {
            continue;
        };

        for target_local in rule_targets {
            push_draw_target_dependency_edge(
                &mut dependency_edges,
                None,
                *target_local,
                DrawTargetDependencyKind::RootRuleTarget,
            );

            let Some(drawable_local) = draw_targets_by_local
                .get(target_local)
                .and_then(|target| target.drawable_local)
            else {
                continue;
            };
            let Some(dependent_rules_local) =
                flattened_rules_by_drawable.get(&drawable_local).copied()
            else {
                continue;
            };
            let Some(dependent_targets) = draw_target_locals_by_rules.get(&dependent_rules_local)
            else {
                continue;
            };

            for dependent_target_local in dependent_targets {
                push_draw_target_dependency_edge(
                    &mut dependency_edges,
                    Some(*dependent_target_local),
                    *target_local,
                    DrawTargetDependencyKind::FlattenedRuleTarget,
                );
            }
        }
    }

    let (local_ids, cycles) = sort_draw_target_order(&dependency_edges);
    DrawTargetOrder {
        dependency_edges,
        local_ids,
        cycles,
    }
}

fn draw_target_locals_by_parent_rules(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    resolved_draw_target_locals: &BTreeSet<usize>,
) -> BTreeMap<usize, Vec<usize>> {
    let mut by_parent = BTreeMap::<usize, Vec<usize>>::new();
    for local_object in local_objects {
        let Some(object) = runtime_object_for_local(file, local_objects, local_object.local_id)
        else {
            continue;
        };
        if object.type_name != "DrawTarget" {
            continue;
        }
        if !resolved_draw_target_locals.contains(&local_object.local_id) {
            continue;
        }
        let Some((parent_local, parent)) =
            local_object_reference_with_local_id(file, local_objects, object_parent_id(object))
        else {
            continue;
        };
        if parent.type_name == "DrawRules" {
            push_unique(
                by_parent.entry(parent_local).or_default(),
                local_object.local_id,
            );
        }
    }
    by_parent
}

fn flattened_rules_by_drawable(drawable_order: &[DrawableOrderNode]) -> BTreeMap<usize, usize> {
    drawable_order
        .iter()
        .filter_map(|node| Some((node.local_id?, node.flattened_draw_rules_local?)))
        .collect()
}

fn push_draw_target_dependency_edge(
    edges: &mut Vec<DrawTargetDependencyEdge>,
    source_local: Option<usize>,
    dependent_local: usize,
    kind: DrawTargetDependencyKind,
) {
    let edge = DrawTargetDependencyEdge {
        source_local,
        dependent_local,
        kind,
    };
    if !edges.contains(&edge) {
        edges.push(edge);
    }
}

fn sort_draw_target_order(
    edges: &[DrawTargetDependencyEdge],
) -> (Vec<usize>, Vec<DrawTargetCycle>) {
    const DRAW_TARGET_ROOT: usize = usize::MAX;

    let mut dependents_by_source = BTreeMap::<usize, Vec<usize>>::new();
    for edge in edges {
        let source = edge.source_local.unwrap_or(DRAW_TARGET_ROOT);
        push_unique(
            dependents_by_source.entry(source).or_default(),
            edge.dependent_local,
        );
    }

    let mut order = Vec::new();
    let mut cycles = Vec::new();
    visit_draw_target_order_node(
        DRAW_TARGET_ROOT,
        &dependents_by_source,
        &mut BTreeSet::new(),
        &mut BTreeSet::new(),
        &mut Vec::new(),
        &mut order,
        &mut cycles,
    );
    order.retain(|local_id| *local_id != DRAW_TARGET_ROOT);
    (order, cycles)
}

fn visit_draw_target_order_node(
    local_id: usize,
    dependents_by_source: &BTreeMap<usize, Vec<usize>>,
    permanent: &mut BTreeSet<usize>,
    temporary: &mut BTreeSet<usize>,
    visiting: &mut Vec<usize>,
    order: &mut Vec<usize>,
    cycles: &mut Vec<DrawTargetCycle>,
) {
    if permanent.contains(&local_id) {
        return;
    }
    if temporary.contains(&local_id) {
        if let Some(start) = visiting.iter().position(|visited| *visited == local_id) {
            let mut local_ids = visiting[start..].to_vec();
            local_ids.push(local_id);
            let cycle = DrawTargetCycle { local_ids };
            if !cycles.contains(&cycle) {
                cycles.push(cycle);
            }
        }
        return;
    }

    temporary.insert(local_id);
    visiting.push(local_id);

    if let Some(dependents) = dependents_by_source.get(&local_id) {
        for dependent in dependents {
            visit_draw_target_order_node(
                *dependent,
                dependents_by_source,
                permanent,
                temporary,
                visiting,
                order,
                cycles,
            );
        }
    }

    visiting.pop();
    temporary.remove(&local_id);
    permanent.insert(local_id);
    order.insert(0, local_id);
}

fn drawable_order(file: &RuntimeFile, local_objects: &[LocalObject]) -> Vec<DrawableOrderNode> {
    let component_draw_rules = component_draw_rules_by_parent(file, local_objects);
    let mut order = Vec::new();

    for local_object in local_objects {
        let Some(object) = runtime_object_for_local(file, local_objects, local_object.local_id)
        else {
            continue;
        };
        if object.type_name == "Artboard" || !is_drawable(object) {
            continue;
        }

        order.push(drawable_order_node(
            file,
            local_objects,
            local_object.local_id,
            object,
            &component_draw_rules,
        ));

        if object.type_name == "ForegroundLayoutDrawable" {
            move_foreground_layout_drawable_before_parent(
                &mut order,
                object_parent_id(object).and_then(|parent| usize::try_from(parent).ok()),
            );
        }
    }

    inject_layout_proxy_drawables(file, local_objects, &mut order);
    order
}

fn sorted_drawable_order(
    drawable_order: &[DrawableOrderNode],
    draw_targets: &[DrawTargetNode],
    draw_rules: &[DrawRulesNode],
    draw_target_order: &[usize],
    clipping_shapes: &[ClippingShapeNode],
) -> Vec<SortedDrawableNode> {
    let active_target_by_rules = draw_rules
        .iter()
        .filter_map(|rules| {
            rules
                .active_target_local
                .map(|target_local| (rules.local_id, target_local))
        })
        .collect::<BTreeMap<_, _>>();
    let draw_targets_by_local = draw_targets
        .iter()
        .map(|target| (target.local_id, target))
        .collect::<BTreeMap<_, _>>();

    let mut main = Vec::new();
    let mut grouped = BTreeMap::<usize, Vec<SortedDrawableNode>>::new();
    for drawable in drawable_order {
        let draw_target_local = drawable
            .flattened_draw_rules_local
            .and_then(|rules_local| active_target_by_rules.get(&rules_local).copied());
        let node = sorted_drawable_node(drawable, draw_target_local);
        if let Some(draw_target_local) = draw_target_local {
            grouped.entry(draw_target_local).or_default().push(node);
        } else {
            main.push(node);
        }
    }

    for draw_target_local in draw_target_order {
        let Some(group) = grouped.remove(draw_target_local) else {
            continue;
        };
        if group.is_empty() {
            continue;
        }
        let Some(target) = draw_targets_by_local.get(draw_target_local) else {
            continue;
        };
        let Some(target_drawable_local) = target.drawable_local else {
            continue;
        };
        let Some(target_position) = main
            .iter()
            .position(|drawable| drawable.local_id == Some(target_drawable_local))
        else {
            continue;
        };

        let insert_at = match target.placement_value {
            0 => target_position,
            1 => target_position + 1,
            _ => continue,
        };
        main.splice(insert_at..insert_at, group);
    }

    let mut sorted = interleave_clipping_proxy_drawables(main.into_iter().rev(), clipping_shapes);
    apply_save_operation_elision(&mut sorted, clipping_shapes);
    sorted
}

fn sorted_drawable_node(
    drawable: &DrawableOrderNode,
    draw_target_local: Option<usize>,
) -> SortedDrawableNode {
    SortedDrawableNode {
        kind: drawable.kind,
        local_id: drawable.local_id,
        global_id: drawable.global_id,
        type_name: drawable.type_name,
        is_hidden: drawable.is_hidden,
        resolved_image_asset_global: drawable.resolved_image_asset_global,
        referenced_artboard_global: drawable.referenced_artboard_global,
        layout_local: drawable.layout_local,
        layout_global: drawable.layout_global,
        draw_target_local,
        clipping_shape_local: None,
        clipping_shape_global: None,
        needs_save_operation: true,
    }
}

fn interleave_clipping_proxy_drawables(
    sorted_drawables: impl IntoIterator<Item = SortedDrawableNode>,
    clipping_shapes: &[ClippingShapeNode],
) -> Vec<SortedDrawableNode> {
    let mut order = Vec::new();
    let mut clipping_stack = Vec::<usize>::new();

    for drawable in sorted_drawables {
        let drawable_clipping_shapes = drawable_clipping_shape_locals(&drawable, clipping_shapes);
        let removing_index = clipping_stack
            .iter()
            .position(|clipping_shape_local| {
                !drawable_clipping_shapes.contains(clipping_shape_local)
            })
            .unwrap_or(clipping_stack.len());

        if removing_index < clipping_stack.len() {
            for clipping_shape_local in clipping_stack[removing_index..].iter().rev() {
                order.push(clipping_proxy_node(
                    clipping_shapes,
                    *clipping_shape_local,
                    DrawableOrderKind::ClipEndProxy,
                ));
            }
            clipping_stack.truncate(removing_index);
        }

        for clipping_shape_local in drawable_clipping_shapes {
            if !clipping_stack.contains(&clipping_shape_local) {
                order.push(clipping_proxy_node(
                    clipping_shapes,
                    clipping_shape_local,
                    DrawableOrderKind::ClipStartProxy,
                ));
                clipping_stack.push(clipping_shape_local);
            }
        }

        order.push(drawable);
    }

    for clipping_shape_local in clipping_stack.into_iter().rev() {
        order.push(clipping_proxy_node(
            clipping_shapes,
            clipping_shape_local,
            DrawableOrderKind::ClipEndProxy,
        ));
    }

    order
}

fn drawable_clipping_shape_locals(
    drawable: &SortedDrawableNode,
    clipping_shapes: &[ClippingShapeNode],
) -> Vec<usize> {
    let Some(drawable_local) = drawable.local_id else {
        return Vec::new();
    };

    clipping_shapes
        .iter()
        .filter_map(|clipping_shape| {
            clipping_shape
                .clipped_drawable_locals
                .contains(&drawable_local)
                .then_some(clipping_shape.local_id)
        })
        .collect()
}

fn clipping_proxy_node(
    clipping_shapes: &[ClippingShapeNode],
    clipping_shape_local: usize,
    kind: DrawableOrderKind,
) -> SortedDrawableNode {
    let clipping_shape_global = clipping_shapes
        .iter()
        .find(|clipping_shape| clipping_shape.local_id == clipping_shape_local)
        .map(|clipping_shape| clipping_shape.global_id);
    debug_assert!(matches!(
        kind,
        DrawableOrderKind::ClipStartProxy | DrawableOrderKind::ClipEndProxy
    ));

    SortedDrawableNode {
        kind,
        local_id: None,
        global_id: None,
        type_name: "ClippingShapeProxyDrawable",
        is_hidden: false,
        resolved_image_asset_global: None,
        referenced_artboard_global: None,
        layout_local: None,
        layout_global: None,
        draw_target_local: None,
        clipping_shape_local: Some(clipping_shape_local),
        clipping_shape_global,
        needs_save_operation: true,
    }
}

fn apply_save_operation_elision(
    sorted_drawables: &mut [SortedDrawableNode],
    clipping_shapes: &[ClippingShapeNode],
) {
    let clipping_visibility = clipping_shapes
        .iter()
        .map(|clipping_shape| (clipping_shape.local_id, clipping_shape.is_visible))
        .collect::<BTreeMap<_, _>>();
    let mut prev_applied_save = false;
    let mut applied_clipping_save_operations = Vec::<bool>::new();

    for index in 0..sorted_drawables.len() {
        sorted_drawables[index].needs_save_operation = true;
        if prev_applied_save {
            if sorted_drawables[index].kind == DrawableOrderKind::ClipStartProxy {
                applied_clipping_save_operations.push(false);
                sorted_drawables[index].needs_save_operation = false;
            } else if sorted_drawables[index].kind == DrawableOrderKind::ClipEndProxy {
                let operation_applied = applied_clipping_save_operations.pop().unwrap_or(true);
                sorted_drawables[index].needs_save_operation = operation_applied;
            } else if sorted_drawables
                .get(index + 1)
                .is_some_and(|next| next.kind == DrawableOrderKind::ClipEndProxy)
            {
                sorted_drawables[index].needs_save_operation = false;
            }
        } else if sorted_drawables[index].kind == DrawableOrderKind::ClipStartProxy {
            applied_clipping_save_operations.push(true);
        } else if sorted_drawables[index].kind == DrawableOrderKind::ClipEndProxy {
            let operation_applied = applied_clipping_save_operations.pop().unwrap_or(true);
            sorted_drawables[index].needs_save_operation = operation_applied;
        }

        prev_applied_save = sorted_drawables[index].kind == DrawableOrderKind::ClipStartProxy
            && (sorted_drawable_will_clip(&sorted_drawables[index], &clipping_visibility)
                || prev_applied_save);
    }

    debug_assert!(applied_clipping_save_operations.is_empty());
}

fn sorted_drawable_will_clip(
    drawable: &SortedDrawableNode,
    clipping_visibility: &BTreeMap<usize, bool>,
) -> bool {
    drawable
        .clipping_shape_local
        .and_then(|local_id| clipping_visibility.get(&local_id))
        .copied()
        .unwrap_or(false)
}

fn component_draw_rules_by_parent(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
) -> BTreeMap<usize, usize> {
    local_objects
        .iter()
        .filter_map(|local_object| {
            let object = runtime_object_for_local(file, local_objects, local_object.local_id)?;
            if object.type_name != "DrawRules" {
                return None;
            }
            let (parent_local, _) = local_object_reference_with_local_id(
                file,
                local_objects,
                object_parent_id(object),
            )?;
            Some((parent_local, local_object.local_id))
        })
        .collect()
}

fn drawable_order_node(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    local_id: usize,
    object: &RuntimeObject,
    component_draw_rules: &BTreeMap<usize, usize>,
) -> DrawableOrderNode {
    let flattened_draw_rules_local =
        flattened_draw_rules_local(file, local_objects, local_id, component_draw_rules);
    DrawableOrderNode {
        kind: DrawableOrderKind::Drawable,
        local_id: Some(local_id),
        global_id: local_object_global_id(local_objects, local_id),
        type_name: object.type_name,
        is_hidden: drawable_is_hidden(object),
        resolved_image_asset_global: resolved_image_asset_global(file, object),
        referenced_artboard_global: referenced_artboard_global(file, object),
        layout_local: None,
        layout_global: None,
        flattened_draw_rules_local,
        flattened_draw_rules_global: flattened_draw_rules_local
            .and_then(|rules_local| local_object_global_id(local_objects, rules_local)),
    }
}

fn flattened_draw_rules_local(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    drawable_local: usize,
    component_draw_rules: &BTreeMap<usize, usize>,
) -> Option<usize> {
    // Cycle guard: a malformed-but-accepted file can make the parent reference
    // chain cyclic (A -> B -> A, not just the self-loop the `parent_local ==
    // current_local` check below catches). C++ hangs on such input; its
    // Component::validate only checks that a parent resolves. We deliberately
    // DIVERGE and terminate the walk with a visited-set, mirroring C++'s own
    // cycle-guard idiom -- DependencySorter::visit's m_Perm/m_Temp visited sets
    // (src/dependency_sorter.cpp) -- so an embedded-SDK draw-order hang becomes
    // a graceful "no draw rule" result. Unreachable on any valid file.
    let mut current_local = drawable_local;
    let mut visited = BTreeSet::new();
    loop {
        if let Some(draw_rules_local) = component_draw_rules.get(&current_local).copied() {
            return Some(draw_rules_local);
        }
        if !visited.insert(current_local) {
            return None;
        }

        let current = runtime_object_for_local(file, local_objects, current_local)?;
        let (parent_local, _) =
            local_object_reference_with_local_id(file, local_objects, object_parent_id(current))?;
        if parent_local == current_local {
            return None;
        }
        current_local = parent_local;
    }
}

fn move_foreground_layout_drawable_before_parent(
    order: &mut [DrawableOrderNode],
    parent_local: Option<usize>,
) {
    if order.len() < 2 {
        return;
    }

    let mut index = order.len() - 1;
    while index >= 1 {
        let swapped_local = order[index - 1].local_id;
        order.swap(index - 1, index);
        if swapped_local == parent_local {
            break;
        }
        index -= 1;
    }
}

fn inject_layout_proxy_drawables(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    order: &mut Vec<DrawableOrderNode>,
) {
    let mut layouts = Vec::<usize>::new();
    let mut index = 0usize;
    while index < order.len() {
        let Some(drawable_local) = order[index].local_id else {
            index += 1;
            continue;
        };

        if let Some(mut current_layout) = layouts.last().copied() {
            if !drawable_is_child_of_layout(file, local_objects, drawable_local, current_layout) {
                loop {
                    order.insert(
                        index,
                        layout_proxy_node(file, local_objects, current_layout),
                    );
                    index += 1;
                    layouts.pop();
                    let Some(next_layout) = layouts.last().copied() else {
                        break;
                    };
                    current_layout = next_layout;
                    if drawable_is_child_of_layout(
                        file,
                        local_objects,
                        drawable_local,
                        current_layout,
                    ) {
                        break;
                    }
                }
            }
        }

        if runtime_object_for_local(file, local_objects, drawable_local)
            .is_some_and(|object| object.type_name == "LayoutComponent")
        {
            layouts.push(drawable_local);
        }

        index += 1;
    }

    while let Some(layout_local) = layouts.pop() {
        order.push(layout_proxy_node(file, local_objects, layout_local));
    }
}

fn layout_proxy_node(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    layout_local: usize,
) -> DrawableOrderNode {
    let is_hidden =
        runtime_object_for_local(file, local_objects, layout_local).is_some_and(drawable_is_hidden);
    DrawableOrderNode {
        kind: DrawableOrderKind::LayoutProxy,
        local_id: None,
        global_id: None,
        type_name: "DrawableProxy",
        is_hidden,
        resolved_image_asset_global: None,
        referenced_artboard_global: None,
        layout_local: Some(layout_local),
        layout_global: local_object_global_id(local_objects, layout_local),
        flattened_draw_rules_local: None,
        flattened_draw_rules_global: None,
    }
}

fn clipping_shapes(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    components: &[ComponentNode],
    component_by_local: &BTreeMap<usize, usize>,
) -> Vec<ClippingShapeNode> {
    local_objects
        .iter()
        .filter_map(|local_object| {
            let object = runtime_object_for_local(file, local_objects, local_object.local_id)?;
            if object.type_name != "ClippingShape" {
                return None;
            }

            let source_id = object
                .uint_property("sourceId")
                .unwrap_or(u64::from(u32::MAX));
            let source_local =
                local_object_reference_with_local_id(file, local_objects, Some(source_id))
                    .and_then(|(local_id, source)| is_node(source).then_some(local_id));
            let shape_locals = source_local
                .map(|local_id| {
                    descendant_component_locals_inclusive(local_id, components, component_by_local)
                        .into_iter()
                        .filter(|shape_local| {
                            runtime_object_for_local(file, local_objects, *shape_local)
                                .is_some_and(is_shape)
                        })
                        .collect()
                })
                .unwrap_or_default();
            let clipped_drawable_locals = object_parent_id(object)
                .and_then(|parent_local| usize::try_from(parent_local).ok())
                .map(|local_id| {
                    descendant_component_locals_inclusive(local_id, components, component_by_local)
                        .into_iter()
                        .filter(|drawable_local| {
                            runtime_object_for_local(file, local_objects, *drawable_local)
                                .is_some_and(is_drawable)
                        })
                        .collect()
                })
                .unwrap_or_default();

            Some(ClippingShapeNode {
                local_id: local_object.local_id,
                global_id: local_object.global_id,
                source_id,
                source_local,
                fill_rule: object.uint_property("fillRule").unwrap_or(0),
                is_visible: object.bool_property("isVisible").unwrap_or(true),
                shape_locals,
                clipped_drawable_locals,
            })
        })
        .collect()
}

fn path_composers(
    shapes: Vec<RuntimeShape<'_>>,
    local_objects: &[LocalObject],
) -> Vec<PathComposerNode> {
    shapes
        .into_iter()
        .filter_map(|shape| {
            let shape_global = local_object_global_id(local_objects, shape.local_id)?;
            let mut path_locals = Vec::new();
            let mut path_globals = Vec::new();
            let mut paths = Vec::new();

            for path in shape.paths {
                let Some(path_global) = local_object_global_id(local_objects, path.local_id) else {
                    continue;
                };
                let is_hidden = path.object.uint_property("pathFlags").unwrap_or(0) & 1 != 0;
                path_locals.push(path.local_id);
                path_globals.push(path_global);
                paths.push(PathComposerPathNode {
                    local_id: path.local_id,
                    global_id: path_global,
                    is_hidden,
                });
            }

            Some(PathComposerNode {
                shape_local: shape.local_id,
                shape_global,
                path_locals,
                path_globals,
                paths,
            })
        })
        .collect()
}

fn meshes(meshes: Vec<RuntimeMesh<'_>>, local_objects: &[LocalObject]) -> Vec<MeshGeometryNode> {
    meshes
        .into_iter()
        .filter_map(|mesh| {
            let global_id = local_object_global_id(local_objects, mesh.local_id)?;

            Some(MeshGeometryNode {
                local_id: mesh.local_id,
                global_id,
                type_name: mesh.object.type_name,
                vertices: mesh
                    .vertices
                    .into_iter()
                    .filter_map(|vertex| {
                        let global_id = local_object_global_id(local_objects, vertex.local_id)?;

                        Some(MeshVertexNode {
                            local_id: vertex.local_id,
                            global_id,
                            type_name: vertex.object.type_name,
                            weight_local: vertex.weight_local_id,
                            weight_global: vertex.weight_local_id.and_then(|local_id| {
                                local_object_global_id(local_objects, local_id)
                            }),
                            weight_type_name: vertex.weight.map(|weight| weight.type_name),
                        })
                    })
                    .collect(),
            })
        })
        .collect()
}

fn paths(paths: Vec<RuntimePath<'_>>, local_objects: &[LocalObject]) -> Vec<PathGeometryNode> {
    paths
        .into_iter()
        .filter_map(|path| {
            let global_id = local_object_global_id(local_objects, path.local_id)?;

            Some(PathGeometryNode {
                local_id: path.local_id,
                global_id,
                type_name: path.object.type_name,
                is_closed: path.object.bool_property("isClosed").unwrap_or(false),
                is_hole: path.object.bool_property("isHole").unwrap_or(false),
                is_clockwise: path_is_clockwise(&path.object),
                parametric: parametric_path(&path.object),
                vertices: path
                    .vertices
                    .into_iter()
                    .filter_map(|vertex| {
                        let global_id = local_object_global_id(local_objects, vertex.local_id)?;

                        Some(PathVertexNode {
                            local_id: vertex.local_id,
                            global_id,
                            type_name: vertex.object.type_name,
                            x: vertex.object.double_property("x").unwrap_or(0.0),
                            y: vertex.object.double_property("y").unwrap_or(0.0),
                            radius: vertex.object.double_property("radius").unwrap_or(0.0),
                            rotation: vertex.object.double_property("rotation").unwrap_or(0.0),
                            distance: vertex.object.double_property("distance").unwrap_or(0.0),
                            in_rotation: vertex.object.double_property("inRotation").unwrap_or(0.0),
                            in_distance: vertex.object.double_property("inDistance").unwrap_or(0.0),
                            out_rotation: vertex
                                .object
                                .double_property("outRotation")
                                .unwrap_or(0.0),
                            out_distance: vertex
                                .object
                                .double_property("outDistance")
                                .unwrap_or(0.0),
                            weight_local: vertex.weight_local_id,
                            weight_global: vertex.weight_local_id.and_then(|local_id| {
                                local_object_global_id(local_objects, local_id)
                            }),
                            weight_type_name: vertex.weight.map(|weight| weight.type_name),
                            weight_values: optional_u32_property(vertex.weight, "values"),
                            weight_indices: optional_u32_property(vertex.weight, "indices"),
                            weight_in_values: optional_u32_property(vertex.weight, "inValues"),
                            weight_in_indices: optional_u32_property(vertex.weight, "inIndices"),
                            weight_out_values: optional_u32_property(vertex.weight, "outValues"),
                            weight_out_indices: optional_u32_property(vertex.weight, "outIndices"),
                        })
                    })
                    .collect(),
            })
        })
        .collect()
}

fn shape_paint_containers(
    file: &RuntimeFile,
    containers: Vec<RuntimeShapePaintContainer<'_>>,
    local_objects: &[LocalObject],
) -> Vec<ShapePaintContainerNode> {
    containers
        .into_iter()
        .filter_map(|container| {
            let global_id = local_object_global_id(local_objects, container.local_id)?;

            Some(ShapePaintContainerNode {
                local_id: container.local_id,
                global_id,
                type_name: container.object.type_name,
                blend_mode_value: shape_paint_container_blend_mode_value(&container.object),
                paints: container
                    .paints
                    .into_iter()
                    .filter_map(|paint| {
                        let global_id = local_object_global_id(local_objects, paint.local_id)?;
                        let paint_type = shape_paint_kind(paint.object.type_name);
                        let gradient_stops = paint
                            .gradient_stops
                            .into_iter()
                            .filter_map(|stop| {
                                let global_id =
                                    local_object_global_id(local_objects, stop.local_id)?;

                                Some(GradientStopNode {
                                    local_id: stop.local_id,
                                    global_id,
                                    type_name: stop.object.type_name,
                                    color: stop
                                        .object
                                        .color_property("colorValue")
                                        .unwrap_or(0xffff_ffff),
                                    position: stop
                                        .object
                                        .double_property("position")
                                        .unwrap_or(0.0),
                                })
                            })
                            .collect::<Vec<_>>();

                        Some(ShapePaintNode {
                            local_id: paint.local_id,
                            global_id,
                            type_name: paint.object.type_name,
                            paint_type,
                            is_visible: shape_paint_is_visible(&paint.object, paint_type),
                            blend_mode_value: shape_paint_blend_mode_value(&paint.object),
                            fill_rule: shape_paint_fill_rule(&paint.object, paint_type),
                            path_kind: shape_paint_path_kind(&paint.object, paint_type),
                            paint_state: shape_paint_state(paint.mutator, &gradient_stops),
                            mutator_local: paint.mutator_local_id,
                            mutator_global: paint.mutator_local_id.and_then(|local_id| {
                                local_object_global_id(local_objects, local_id)
                            }),
                            mutator_type_name: paint.mutator.map(|object| object.type_name),
                            feather: shape_paint_feather(
                                paint.feather,
                                paint.feather_local_id,
                                local_objects,
                            ),
                            feather_local: paint.feather_local_id,
                            feather_global: paint.feather_local_id.and_then(|local_id| {
                                local_object_global_id(local_objects, local_id)
                            }),
                            feather_type_name: paint.feather.map(|object| object.type_name),
                            effects: paint
                                .effects
                                .into_iter()
                                .filter_map(|effect| {
                                    stroke_effect_node(file, local_objects, effect)
                                })
                                .collect(),
                            gradient_stops,
                        })
                    })
                    .collect(),
            })
        })
        .collect()
}

fn optional_u32_property(object: Option<&RuntimeObject>, property: &str) -> Option<u32> {
    object?
        .uint_property(property)
        .and_then(|value| u32::try_from(value).ok())
}

fn stroke_effect_node(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    effect: nuxie_binary::RuntimeStrokeEffect<'_>,
) -> Option<StrokeEffectNode> {
    Some(StrokeEffectNode {
        local_id: effect.local_id,
        global_id: local_object_global_id(local_objects, effect.local_id)?,
        type_name: effect.object.type_name,
        trim_start: trim_path_double(effect.object, "start"),
        trim_end: trim_path_double(effect.object, "end"),
        trim_offset: trim_path_double(effect.object, "offset"),
        trim_mode_value: if effect.object.type_name == "TrimPath" {
            optional_u32_property(Some(effect.object), "modeValue")
        } else {
            None
        },
        dash_offset: dash_path_double(effect.object, "offset"),
        dash_offset_is_percentage: dash_path_bool(effect.object, "offsetIsPercentage"),
        dashes: dash_path_dashes(file, local_objects, effect.local_id),
        target_group_effect_local: effect.target_group_effect_local_id,
        target_group_effect_global: effect
            .target_group_effect_local_id
            .and_then(|local_id| local_object_global_id(local_objects, local_id)),
        target_group_effect_type_name: effect.target_group_effect.map(|object| object.type_name),
        group_effects: effect
            .group_effects
            .into_iter()
            .filter_map(|effect| stroke_effect_node(file, local_objects, effect))
            .collect(),
    })
}

fn trim_path_double(object: &RuntimeObject, property: &str) -> Option<f32> {
    if object.type_name == "TrimPath" {
        Some(object.double_property(property).unwrap_or(0.0))
    } else {
        None
    }
}

fn dash_path_double(object: &RuntimeObject, property: &str) -> Option<f32> {
    if object.type_name == "DashPath" {
        Some(object.double_property(property).unwrap_or(0.0))
    } else {
        None
    }
}

fn dash_path_bool(object: &RuntimeObject, property: &str) -> Option<bool> {
    if object.type_name == "DashPath" {
        Some(object.bool_property(property).unwrap_or(false))
    } else {
        None
    }
}

fn dash_path_dashes(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    dash_path_local: usize,
) -> Vec<DashNode> {
    local_objects
        .iter()
        .filter_map(|local_object| {
            let object = file.object(local_object.global_id as usize)?;
            if object.type_name != "Dash"
                || object_parent_id(object) != Some(dash_path_local as u64)
            {
                return None;
            }

            Some(DashNode {
                local_id: local_object.local_id,
                global_id: local_object.global_id,
                length: object.double_property("length").unwrap_or(0.0),
                length_is_percentage: object.bool_property("lengthIsPercentage").unwrap_or(false),
            })
        })
        .collect()
}

fn mat2d_property_array(object: &RuntimeObject) -> [f32; 6] {
    [
        object.double_property("xx").unwrap_or(1.0),
        object.double_property("xy").unwrap_or(0.0),
        object.double_property("yx").unwrap_or(0.0),
        object.double_property("yy").unwrap_or(1.0),
        object.double_property("tx").unwrap_or(0.0),
        object.double_property("ty").unwrap_or(0.0),
    ]
}

fn invert_mat2d_or_identity(matrix: [f32; 6]) -> [f32; 6] {
    let [a, b, c, d, e, f] = matrix;
    let determinant = a * d - c * b;
    if determinant == 0.0 {
        return [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
    }

    // Ported from src/math/mat2d.cpp Mat2D::invert. C++ computes the
    // reciprocal once and multiplies each coefficient; tiny skinning residuals
    // in static goldens depend on that grouping.
    let determinant = 1.0 / determinant;
    [
        d * determinant,
        -b * determinant,
        -c * determinant,
        a * determinant,
        c.mul_add(f, -(d * e)) * determinant,
        b.mul_add(e, -(a * f)) * determinant,
    ]
}

fn parametric_path(path: &RuntimeObject) -> Option<ParametricPathNode> {
    match path.type_name {
        "Ellipse" => Some(ParametricPathNode::Ellipse {
            width: path.double_property("width").unwrap_or(0.0),
            height: path.double_property("height").unwrap_or(0.0),
            origin_x: path.double_property("originX").unwrap_or(0.5),
            origin_y: path.double_property("originY").unwrap_or(0.5),
        }),
        "Polygon" => Some(ParametricPathNode::Polygon {
            width: path.double_property("width").unwrap_or(0.0),
            height: path.double_property("height").unwrap_or(0.0),
            origin_x: path.double_property("originX").unwrap_or(0.5),
            origin_y: path.double_property("originY").unwrap_or(0.5),
            points: path.uint_property("points").unwrap_or(5) as u32,
            corner_radius: path.double_property("cornerRadius").unwrap_or(0.0),
        }),
        "Star" => Some(ParametricPathNode::Star {
            width: path.double_property("width").unwrap_or(0.0),
            height: path.double_property("height").unwrap_or(0.0),
            origin_x: path.double_property("originX").unwrap_or(0.5),
            origin_y: path.double_property("originY").unwrap_or(0.5),
            points: path.uint_property("points").unwrap_or(5) as u32,
            corner_radius: path.double_property("cornerRadius").unwrap_or(0.0),
            inner_radius: path.double_property("innerRadius").unwrap_or(0.5),
        }),
        "Triangle" => Some(ParametricPathNode::Triangle {
            width: path.double_property("width").unwrap_or(0.0),
            height: path.double_property("height").unwrap_or(0.0),
            origin_x: path.double_property("originX").unwrap_or(0.5),
            origin_y: path.double_property("originY").unwrap_or(0.5),
        }),
        "Rectangle" => Some(ParametricPathNode::Rectangle {
            width: path.double_property("width").unwrap_or(0.0),
            height: path.double_property("height").unwrap_or(0.0),
            origin_x: path.double_property("originX").unwrap_or(0.5),
            origin_y: path.double_property("originY").unwrap_or(0.5),
            link_corner_radius: path.bool_property("linkCornerRadius").unwrap_or(true),
            corner_radius_tl: path.double_property("cornerRadiusTL").unwrap_or(0.0),
            corner_radius_tr: path.double_property("cornerRadiusTR").unwrap_or(0.0),
            corner_radius_bl: path.double_property("cornerRadiusBL").unwrap_or(0.0),
            corner_radius_br: path.double_property("cornerRadiusBR").unwrap_or(0.0),
        }),
        _ => None,
    }
}

fn shape_paint_kind(type_name: &str) -> ShapePaintKind {
    match type_name {
        "Fill" => ShapePaintKind::Fill,
        "Stroke" => ShapePaintKind::Stroke,
        _ => ShapePaintKind::Unknown,
    }
}

fn shape_paint_is_visible(object: &RuntimeObject, paint_type: ShapePaintKind) -> bool {
    let base_visible = object.bool_property("isVisible").unwrap_or(true);
    match paint_type {
        ShapePaintKind::Stroke => {
            base_visible && object.double_property("thickness").unwrap_or(1.0) > 0.0
        }
        ShapePaintKind::Fill | ShapePaintKind::Unknown => base_visible,
    }
}

fn shape_paint_fill_rule(object: &RuntimeObject, paint_type: ShapePaintKind) -> u64 {
    match paint_type {
        ShapePaintKind::Fill => object.uint_property("fillRule").unwrap_or(0),
        ShapePaintKind::Stroke | ShapePaintKind::Unknown => 0,
    }
}

fn shape_paint_path_kind(
    object: &RuntimeObject,
    paint_type: ShapePaintKind,
) -> Option<ShapePaintPathKind> {
    match paint_type {
        ShapePaintKind::Fill => {
            if object.uint_property("fillRule").unwrap_or(0) == 2 {
                Some(ShapePaintPathKind::LocalClockwise)
            } else {
                Some(ShapePaintPathKind::Local)
            }
        }
        ShapePaintKind::Stroke => {
            if object
                .bool_property("transformAffectsStroke")
                .unwrap_or(true)
            {
                Some(ShapePaintPathKind::Local)
            } else {
                Some(ShapePaintPathKind::World)
            }
        }
        ShapePaintKind::Unknown => None,
    }
}

fn shape_paint_state(
    mutator: Option<&RuntimeObject>,
    gradient_stops: &[GradientStopNode],
) -> Option<ShapePaintStateNode> {
    let mutator = mutator?;
    match mutator.type_name {
        "SolidColor" => Some(ShapePaintStateNode::SolidColor {
            color: mutator.color_property("colorValue").unwrap_or(0xFF747474),
        }),
        "LinearGradient" => Some(ShapePaintStateNode::LinearGradient {
            start_x: mutator.double_property("startX").unwrap_or(0.0),
            start_y: mutator.double_property("startY").unwrap_or(0.0),
            end_x: mutator.double_property("endX").unwrap_or(0.0),
            end_y: mutator.double_property("endY").unwrap_or(0.0),
            opacity: mutator.double_property("opacity").unwrap_or(1.0),
            stops: gradient_stops.to_vec(),
        }),
        "RadialGradient" => Some(ShapePaintStateNode::RadialGradient {
            start_x: mutator.double_property("startX").unwrap_or(0.0),
            start_y: mutator.double_property("startY").unwrap_or(0.0),
            end_x: mutator.double_property("endX").unwrap_or(0.0),
            end_y: mutator.double_property("endY").unwrap_or(0.0),
            opacity: mutator.double_property("opacity").unwrap_or(1.0),
            stops: gradient_stops.to_vec(),
        }),
        _ => None,
    }
}

fn shape_paint_feather(
    feather: Option<&RuntimeObject>,
    feather_local_id: Option<usize>,
    local_objects: &[LocalObject],
) -> Option<FeatherNode> {
    let feather = feather?;
    Some(FeatherNode {
        local_id: feather_local_id?,
        global_id: feather_local_id
            .and_then(|local_id| local_object_global_id(local_objects, local_id))?,
        type_name: feather.type_name,
        space_value: feather.uint_property("spaceValue").unwrap_or(0) as u32,
        strength: feather.double_property("strength").unwrap_or(12.0),
        offset_x: feather.double_property("offsetX").unwrap_or(0.0),
        offset_y: feather.double_property("offsetY").unwrap_or(0.0),
        inner: feather.bool_property("inner").unwrap_or(false),
    })
}

fn shape_paint_container_blend_mode_value(container: &RuntimeObject) -> u32 {
    container.uint_property("blendModeValue").unwrap_or(3) as u32
}

fn shape_paint_blend_mode_value(paint: &RuntimeObject) -> u32 {
    paint.uint_property("blendModeValue").unwrap_or(127) as u32
}

fn n_slicer_details(
    details: Vec<RuntimeNSlicerDetails<'_>>,
    local_objects: &[LocalObject],
) -> Vec<NSlicerDetailsNode> {
    details
        .into_iter()
        .filter_map(|details| {
            let global_id = local_object_global_id(local_objects, details.local_id)?;

            Some(NSlicerDetailsNode {
                local_id: details.local_id,
                global_id,
                type_name: details.object.type_name,
                x_axes: details
                    .x_axes
                    .into_iter()
                    .filter_map(|axis| {
                        Some(NSlicerAxisNode {
                            local_id: axis.local_id,
                            global_id: local_object_global_id(local_objects, axis.local_id)?,
                            type_name: axis.object.type_name,
                        })
                    })
                    .collect(),
                y_axes: details
                    .y_axes
                    .into_iter()
                    .filter_map(|axis| {
                        Some(NSlicerAxisNode {
                            local_id: axis.local_id,
                            global_id: local_object_global_id(local_objects, axis.local_id)?,
                            type_name: axis.object.type_name,
                        })
                    })
                    .collect(),
                tile_modes: details
                    .tile_modes
                    .into_iter()
                    .filter_map(|tile_mode| {
                        Some(NSlicerTileModeNode {
                            local_id: tile_mode.local_id,
                            global_id: local_object_global_id(local_objects, tile_mode.local_id)?,
                            type_name: tile_mode.object.type_name,
                            patch_index: tile_mode.patch_index,
                            style: tile_mode.style,
                        })
                    })
                    .collect(),
            })
        })
        .collect()
}

fn shape_deformers(file: &RuntimeFile, local_objects: &[LocalObject]) -> Vec<ShapeDeformerNode> {
    local_objects
        .iter()
        .filter_map(|local_object| {
            let shape = runtime_object_for_local(file, local_objects, local_object.local_id)?;
            if !is_shape(shape) {
                return None;
            }

            let (deformer_local, deformer_type_name) =
                render_path_deformer_for_shape(file, local_objects, shape);
            Some(ShapeDeformerNode {
                shape_local: local_object.local_id,
                shape_global: local_object.global_id,
                deformer_local,
                deformer_global: deformer_local
                    .and_then(|local_id| local_object_global_id(local_objects, local_id)),
                deformer_type_name,
            })
        })
        .collect()
}

fn render_path_deformer_for_shape(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    shape: &RuntimeObject,
) -> (Option<usize>, Option<&'static str>) {
    let mut current_local = object_parent_id(shape).and_then(|parent| usize::try_from(parent).ok());
    let mut visited = Vec::new();
    while let Some(local_id) = current_local {
        if visited.contains(&local_id) {
            return (None, None);
        }
        visited.push(local_id);

        let Some(object) = runtime_object_for_local(file, local_objects, local_id) else {
            return (None, None);
        };
        if object.type_name == "NSlicedNode" {
            return (Some(local_id), Some(object.type_name));
        }

        current_local = object_parent_id(object).and_then(|parent| usize::try_from(parent).ok());
    }

    (None, None)
}

fn skeletal_bones(file: &RuntimeFile, local_objects: &[LocalObject]) -> Vec<SkeletalBoneNode> {
    let peer_constraints = ik_peer_constraints_by_bone(file, local_objects);

    local_objects
        .iter()
        .filter_map(|local_object| {
            let bone = runtime_object_for_local(file, local_objects, local_object.local_id)?;
            if !is_bone(bone) {
                return None;
            }

            let child_bone_locals = child_bone_locals(file, local_objects, local_object.local_id);
            let child_bone_globals = local_globals(local_objects, &child_bone_locals);
            let peer_constraint_locals = peer_constraints
                .get(&local_object.local_id)
                .cloned()
                .unwrap_or_default();
            let peer_constraint_globals = local_globals(local_objects, &peer_constraint_locals);

            Some(SkeletalBoneNode {
                local_id: local_object.local_id,
                global_id: local_object.global_id,
                type_name: bone.type_name,
                child_bone_locals,
                child_bone_globals,
                peer_constraint_locals,
                peer_constraint_globals,
            })
        })
        .collect()
}

fn child_bone_locals(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    bone_local: usize,
) -> Vec<usize> {
    local_objects
        .iter()
        .filter_map(|local_object| {
            let child = runtime_object_for_local(file, local_objects, local_object.local_id)?;
            if child.type_name == "Bone" && object_parent_id(child) == Some(bone_local as u64) {
                Some(local_object.local_id)
            } else {
                None
            }
        })
        .collect()
}

fn skeletal_skins(file: &RuntimeFile, local_objects: &[LocalObject]) -> Vec<SkeletalSkinNode> {
    local_objects
        .iter()
        .filter_map(|local_object| {
            let skin = runtime_object_for_local(file, local_objects, local_object.local_id)?;
            if skin.type_name != "Skin" {
                return None;
            }

            let (skinnable_local, skinnable_type_name) =
                local_object_reference_with_local_id(file, local_objects, object_parent_id(skin))
                    .and_then(|(local_id, object)| {
                        if is_cpp_skinnable(object) {
                            Some((Some(local_id), Some(object.type_name)))
                        } else {
                            None
                        }
                    })
                    .unwrap_or((None, None));

            Some(SkeletalSkinNode {
                skin_local: local_object.local_id,
                skin_global: local_object.global_id,
                world_transform: mat2d_property_array(skin),
                skinnable_local,
                skinnable_global: skinnable_local
                    .and_then(|local_id| local_object_global_id(local_objects, local_id)),
                skinnable_type_name,
                tendons: skeletal_skin_tendons(file, local_objects, local_object.local_id),
            })
        })
        .collect()
}

fn skeletal_skin_tendons(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    skin_local: usize,
) -> Vec<SkeletalTendonNode> {
    local_objects
        .iter()
        .filter_map(|local_object| {
            let tendon = runtime_object_for_local(file, local_objects, local_object.local_id)?;
            if tendon.type_name != "Tendon" || object_parent_id(tendon) != Some(skin_local as u64) {
                return None;
            }

            let (bone_local, bone_type_name) = local_object_reference_with_local_id(
                file,
                local_objects,
                tendon.uint_property("boneId"),
            )
            .and_then(|(local_id, object)| {
                if is_bone(object) {
                    Some((Some(local_id), Some(object.type_name)))
                } else {
                    None
                }
            })
            .unwrap_or((None, None));

            Some(SkeletalTendonNode {
                tendon_local: local_object.local_id,
                tendon_global: local_object.global_id,
                bone_local,
                bone_global: bone_local
                    .and_then(|local_id| local_object_global_id(local_objects, local_id)),
                bone_type_name,
                inverse_bind: invert_mat2d_or_identity(mat2d_property_array(tendon)),
            })
        })
        .collect()
}

fn local_globals(local_objects: &[LocalObject], local_ids: &[usize]) -> Vec<u32> {
    local_ids
        .iter()
        .filter_map(|local_id| local_object_global_id(local_objects, *local_id))
        .collect()
}

fn text_variation_helpers(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
) -> Vec<TextVariationHelperNode> {
    let Some(artboard_local) = local_objects.iter().find_map(|local_object| {
        let object = runtime_object_for_local(file, local_objects, local_object.local_id)?;
        (object.type_name == "Artboard").then_some(local_object.local_id)
    }) else {
        return Vec::new();
    };

    local_objects
        .iter()
        .filter_map(|local_object| {
            let text_style = runtime_object_for_local(file, local_objects, local_object.local_id)?;
            if !is_text_style(text_style)
                || !text_style_has_variation_children(file, local_objects, local_object.local_id)
            {
                return None;
            }

            let (text_local, text) = local_object_reference_with_local_id(
                file,
                local_objects,
                object_parent_id(text_style),
            )?;
            if !is_text_interface(text) {
                return None;
            }

            Some(TextVariationHelperNode {
                text_style_local: local_object.local_id,
                text_style_global: local_object.global_id,
                text_local,
                text_global: local_object_global_id(local_objects, text_local)?,
                artboard_local,
                artboard_global: local_object_global_id(local_objects, artboard_local)
                    .unwrap_or(artboard_local as u32),
            })
        })
        .collect()
}

fn text_style_has_variation_children(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    text_style_local: usize,
) -> bool {
    local_objects.iter().any(|local_object| {
        let Some(child) = runtime_object_for_local(file, local_objects, local_object.local_id)
        else {
            return false;
        };
        object_parent_id(child) == Some(text_style_local as u64)
            && matches!(child.type_name, "TextStyleAxis" | "TextStyleFeature")
    })
}

fn list_constraint_registrations(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
) -> Vec<ListConstraintRegistrationNode> {
    local_objects
        .iter()
        .filter_map(|local_object| {
            let constraint = runtime_object_for_local(file, local_objects, local_object.local_id)?;
            if !is_list_constraint(constraint) {
                return None;
            }

            let (constrainable_list_local, constrainable_list) =
                local_object_reference_with_local_id(
                    file,
                    local_objects,
                    object_parent_id(constraint),
                )?;
            if !is_constrainable_list(constrainable_list) {
                return None;
            }

            Some(ListConstraintRegistrationNode {
                constrainable_list_local,
                constrainable_list_global: local_object_global_id(
                    local_objects,
                    constrainable_list_local,
                )?,
                constraint_local: local_object.local_id,
                constraint_global: local_object.global_id,
                constraint_type_name: constraint.type_name,
            })
        })
        .collect()
}

fn layout_constraint_registrations(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    components: &[ComponentNode],
) -> Vec<LayoutConstraintRegistrationNode> {
    scroll_constraint_layout_child_dependencies(file, local_objects, components)
        .into_iter()
        .filter_map(|(constraint_local, layout_provider_local)| {
            let constraint = runtime_object_for_local(file, local_objects, constraint_local)?;
            let layout_provider =
                runtime_object_for_local(file, local_objects, layout_provider_local)?;
            Some(LayoutConstraintRegistrationNode {
                layout_provider_local,
                layout_provider_global: local_object_global_id(
                    local_objects,
                    layout_provider_local,
                )?,
                layout_provider_type_name: layout_provider.type_name,
                constraint_local,
                constraint_global: local_object_global_id(local_objects, constraint_local)?,
                constraint_type_name: constraint.type_name,
            })
        })
        .collect()
}

fn nested_artboards(file: &RuntimeFile, local_objects: &[LocalObject]) -> Vec<NestedArtboardNode> {
    local_objects
        .iter()
        .filter_map(|local_object| {
            let object = runtime_object_for_local(file, local_objects, local_object.local_id)?;
            if !is_exact_nested_artboard_host(object) {
                return None;
            }

            Some(NestedArtboardNode {
                local_id: local_object.local_id,
                global_id: local_object.global_id,
                type_name: object.type_name,
                name: object_name(object),
            })
        })
        .collect()
}

fn component_lists(file: &RuntimeFile, local_objects: &[LocalObject]) -> Vec<ComponentListNode> {
    local_objects
        .iter()
        .filter_map(|local_object| {
            let object = runtime_object_for_local(file, local_objects, local_object.local_id)?;
            if !is_artboard_component_list(object) {
                return None;
            }

            Some(ComponentListNode {
                local_id: local_object.local_id,
                global_id: local_object.global_id,
                type_name: object.type_name,
                name: object_name(object),
                map_rules: component_list_map_rules(file, local_objects, object),
            })
        })
        .collect()
}

fn component_list_map_rules(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    component_list: &RuntimeObject,
) -> Vec<ComponentListMapRuleNode> {
    let mut map_rules = BTreeMap::new();
    for rule in file.artboard_component_list_map_rules_for_object(component_list) {
        let state_machine_ids = local_objects
            .iter()
            .find(|local| local.global_id == rule.object.id)
            .map(|rule_local| {
                local_objects
                    .iter()
                    .filter_map(|local| {
                        let object = runtime_object_for_local(file, local_objects, local.local_id)?;
                        (object.type_name == "NestedStateMachine"
                            && object.uint_property("parentId") == Some(rule_local.local_id as u64))
                        .then(|| {
                            object
                                .uint_property("animationId")
                                .and_then(|id| usize::try_from(id).ok())
                        })
                        .flatten()
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        map_rules.insert(
            cpp_i32_from_runtime_uint(rule.view_model_id),
            (
                cpp_i32_from_runtime_uint(rule.artboard_id),
                state_machine_ids,
            ),
        );
    }

    map_rules
        .into_iter()
        .map(
            |(view_model_id, (artboard_id, state_machine_ids))| ComponentListMapRuleNode {
                view_model_id,
                artboard_id,
                state_machine_ids,
            },
        )
        .collect()
}

fn artboard_hosts(file: &RuntimeFile, local_objects: &[LocalObject]) -> Vec<ArtboardHostNode> {
    local_objects
        .iter()
        .filter_map(|local_object| {
            let object = runtime_object_for_local(file, local_objects, local_object.local_id)?;
            let kind = if is_exact_nested_artboard_host(object) {
                ArtboardHostKind::NestedArtboard
            } else if is_artboard_component_list(object) {
                ArtboardHostKind::ComponentList
            } else {
                return None;
            };

            Some(ArtboardHostNode {
                local_id: local_object.local_id,
                global_id: local_object.global_id,
                type_name: object.type_name,
                name: object_name(object),
                kind,
            })
        })
        .collect()
}

fn joysticks(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    animations: &[AnimationGraph],
) -> Vec<JoystickNode> {
    local_objects
        .iter()
        .filter_map(|local_object| {
            let joystick = runtime_object_for_local(file, local_objects, local_object.local_id)?;
            if joystick.type_name != "Joystick" {
                return None;
            }

            let handle_source_local = local_object_reference_with_local_id(
                file,
                local_objects,
                joystick.uint_property("handleSourceId"),
            )
            .filter(|(_, source)| is_transform_component(source))
            .map(|(local_id, _)| local_id);
            let handle_source_global = handle_source_local
                .and_then(|local_id| local_object_global_id(local_objects, local_id));
            let x_animation = joystick_axis_animation(joystick, "xId", animations);
            let y_animation = joystick_axis_animation(joystick, "yId", animations);

            let mut nested_remap_dependents = Vec::new();
            nested_remap_dependents.extend(joystick_nested_remap_dependents(
                file,
                local_objects,
                y_animation,
            ));
            nested_remap_dependents.extend(joystick_nested_remap_dependents(
                file,
                local_objects,
                x_animation,
            ));

            Some(JoystickNode {
                local_id: local_object.local_id,
                global_id: local_object.global_id,
                name: object_name(joystick),
                handle_source_local,
                handle_source_global,
                can_apply_before_update: handle_source_local.is_none(),
                x_animation_global: x_animation.map(|animation| animation.global_id),
                y_animation_global: y_animation.map(|animation| animation.global_id),
                nested_remap_dependents,
            })
        })
        .collect()
}

fn joystick_axis_animation<'a>(
    joystick: &RuntimeObject,
    property_name: &str,
    animations: &'a [AnimationGraph],
) -> Option<&'a AnimationGraph> {
    let index = usize::try_from(joystick.uint_property(property_name)?).ok()?;
    animations.get(index)
}

fn joystick_nested_remap_dependents(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    animation: Option<&AnimationGraph>,
) -> Vec<JoystickNestedRemapDependentNode> {
    let Some(animation) = animation else {
        return Vec::new();
    };

    animation
        .keyed_objects
        .iter()
        .filter_map(|keyed_object| {
            let (local_id, target) = local_object_reference_with_local_id(
                file,
                local_objects,
                Some(keyed_object.object_id),
            )?;
            if target.type_name != "NestedRemapAnimation" {
                return None;
            }

            Some(JoystickNestedRemapDependentNode {
                local_id,
                global_id: local_object_global_id(local_objects, local_id)?,
            })
        })
        .collect()
}

fn resetting_components(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
) -> Vec<ResettingComponentNode> {
    local_objects
        .iter()
        .filter_map(|local_object| {
            let object = runtime_object_for_local(file, local_objects, local_object.local_id)?;
            let kind = resetting_component_kind(object)?;
            Some(ResettingComponentNode {
                local_id: local_object.local_id,
                global_id: local_object.global_id,
                type_name: object.type_name,
                name: object_name(object),
                kind,
            })
        })
        .collect()
}

fn advancing_components(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
) -> Vec<AdvancingComponentNode> {
    local_objects
        .iter()
        .filter_map(|local_object| {
            let object = runtime_object_for_local(file, local_objects, local_object.local_id)?;
            let kind = advancing_component_kind(object)?;
            Some(AdvancingComponentNode {
                local_id: local_object.local_id,
                global_id: local_object.global_id,
                type_name: object.type_name,
                name: object_name(object),
                kind,
            })
        })
        .collect()
}

fn build_dependency_nodes(
    components: &[ComponentNode],
    path_composers: &[PathComposerNode],
    text_variation_helpers: &[TextVariationHelperNode],
) -> Vec<DependencyNode> {
    let mut nodes = Vec::new();

    for component in components {
        nodes.push(DependencyNode {
            node_id: nodes.len(),
            kind: DependencyNodeKind::Component {
                local_id: component.local_id,
                global_id: component.global_id,
                type_name: component.type_name,
                name: component.name.clone(),
            },
        });
    }

    for composer in path_composers {
        nodes.push(DependencyNode {
            node_id: nodes.len(),
            kind: DependencyNodeKind::PathComposer {
                shape_local: composer.shape_local,
                shape_global: composer.shape_global,
            },
        });
    }

    for helper in text_variation_helpers {
        nodes.push(DependencyNode {
            node_id: nodes.len(),
            kind: DependencyNodeKind::TextVariationHelper {
                text_style_local: helper.text_style_local,
                text_style_global: helper.text_style_global,
                text_local: helper.text_local,
                text_global: helper.text_global,
                artboard_local: helper.artboard_local,
                artboard_global: helper.artboard_global,
            },
        });
    }

    nodes
}

fn build_dependency_node_edges(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    dependency_nodes: &[DependencyNode],
    dependency_edges: &[DependencyEdge],
    path_composers: &[PathComposerNode],
    clipping_shapes: &[ClippingShapeNode],
    text_variation_helpers: &[TextVariationHelperNode],
) -> Vec<DependencyNodeEdge> {
    let component_node_by_local = component_dependency_node_by_local(dependency_nodes);
    let path_composer_node_by_shape = path_composer_dependency_node_by_shape(dependency_nodes);
    let text_variation_helper_node_by_style =
        text_variation_helper_dependency_node_by_style(dependency_nodes);
    let mut edges = Vec::new();

    for edge in dependency_edges {
        let Some(source_node) = component_node_by_local.get(&edge.source_local).copied() else {
            continue;
        };
        let Some(dependent_node) = component_node_by_local.get(&edge.dependent_local).copied()
        else {
            continue;
        };
        edges.push(DependencyNodeEdge {
            source_node,
            dependent_node,
            kind: edge.kind,
        });
    }

    for composer in path_composers {
        let Some(path_composer_node) = path_composer_node_by_shape
            .get(&composer.shape_local)
            .copied()
        else {
            continue;
        };
        if let Some(shape_node) = component_node_by_local.get(&composer.shape_local).copied() {
            edges.push(DependencyNodeEdge {
                source_node: shape_node,
                dependent_node: path_composer_node,
                kind: DependencyKind::PathComposerShape,
            });
        }
        for path_local in &composer.path_locals {
            let Some(path_node) = component_node_by_local.get(path_local).copied() else {
                continue;
            };
            edges.push(DependencyNodeEdge {
                source_node: path_node,
                dependent_node: path_composer_node,
                kind: DependencyKind::PathComposerPath,
            });
        }
    }

    for clipping_shape in clipping_shapes {
        let Some(clipping_shape_node) = component_node_by_local
            .get(&clipping_shape.local_id)
            .copied()
        else {
            continue;
        };
        for shape_local in &clipping_shape.shape_locals {
            let Some(path_composer_node) = path_composer_node_by_shape.get(shape_local).copied()
            else {
                continue;
            };
            edges.push(DependencyNodeEdge {
                source_node: path_composer_node,
                dependent_node: clipping_shape_node,
                kind: DependencyKind::ClippingShapePathComposer,
            });
        }
    }

    for helper in text_variation_helpers {
        let Some(helper_node) = text_variation_helper_node_by_style
            .get(&helper.text_style_local)
            .copied()
        else {
            continue;
        };
        if let Some(artboard_node) = component_node_by_local.get(&helper.artboard_local).copied() {
            edges.push(DependencyNodeEdge {
                source_node: artboard_node,
                dependent_node: helper_node,
                kind: DependencyKind::TextVariationHelperArtboard,
            });
        }
        if let Some(text_node) = component_node_by_local.get(&helper.text_local).copied() {
            edges.push(DependencyNodeEdge {
                source_node: helper_node,
                dependent_node: text_node,
                kind: DependencyKind::TextVariationHelperText,
            });
        }
    }

    for (source_node, constraint_node, kind) in follow_path_constraint_target_node_dependencies(
        file,
        local_objects,
        dependency_nodes,
        path_composers,
        &component_node_by_local,
        &path_composer_node_by_shape,
    ) {
        edges.push(DependencyNodeEdge {
            source_node,
            dependent_node: constraint_node,
            kind,
        });
    }

    for (source_node, modifier_node, kind) in text_follow_path_modifier_target_node_dependencies(
        file,
        local_objects,
        dependency_nodes,
        &component_node_by_local,
        &path_composer_node_by_shape,
    ) {
        edges.push(DependencyNodeEdge {
            source_node,
            dependent_node: modifier_node,
            kind,
        });
    }

    for (source_node, stroke_node) in stroke_path_builder_node_dependencies(
        file,
        local_objects,
        dependency_nodes,
        &component_node_by_local,
        &path_composer_node_by_shape,
    ) {
        edges.push(DependencyNodeEdge {
            source_node,
            dependent_node: stroke_node,
            kind: DependencyKind::StrokePathBuilder,
        });
    }

    for (source_node, fill_node) in fill_path_builder_node_dependencies(
        file,
        local_objects,
        dependency_nodes,
        &component_node_by_local,
        &path_composer_node_by_shape,
    ) {
        edges.push(DependencyNodeEdge {
            source_node,
            dependent_node: fill_node,
            kind: DependencyKind::FillPathBuilder,
        });
    }

    for (source_node, feather_node) in feather_path_builder_node_dependencies(
        file,
        local_objects,
        dependency_nodes,
        &component_node_by_local,
        &path_composer_node_by_shape,
    ) {
        edges.push(DependencyNodeEdge {
            source_node,
            dependent_node: feather_node,
            kind: DependencyKind::FeatherPathBuilder,
        });
    }

    for (source_node, gradient_node) in linear_gradient_paint_container_node_dependencies(
        file,
        local_objects,
        dependency_nodes,
        &component_node_by_local,
    ) {
        edges.push(DependencyNodeEdge {
            source_node,
            dependent_node: gradient_node,
            kind: DependencyKind::LinearGradientPaintContainer,
        });
    }

    dedup_preserving_order(&mut edges);
    edges
}

fn sort_dependency_node_edges(edges: &mut Vec<DependencyNodeEdge>) {
    edges.sort_by_key(|edge| {
        (
            edge.source_node,
            edge.dependent_node,
            dependency_kind_sort_key(edge.kind),
        )
    });
    edges.dedup();
}

fn build_dependency_edges(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    components: &[ComponentNode],
    draw_targets: &[DrawTargetNode],
    draw_rules: &[DrawRulesNode],
    clipping_shapes: &[ClippingShapeNode],
) -> Vec<DependencyEdge> {
    let mut edges = Vec::new();

    for component in components {
        for child in &component.children {
            if component_skips_parent_child_dependency(file, local_objects, *child) {
                continue;
            }
            edges.push(DependencyEdge {
                source_local: component.local_id,
                dependent_local: *child,
                kind: DependencyKind::ParentChild,
            });
        }
    }

    for local_object in local_objects {
        let Some(object) = runtime_object_for_local(file, local_objects, local_object.local_id)
        else {
            continue;
        };
        let Some(definition) = definition_by_type_key(object.type_key) else {
            continue;
        };
        if !definition.is_a("TargetedConstraint") {
            continue;
        }
        if definition.is_a("FollowPathConstraint") {
            continue;
        }

        let Some((target_local, target)) = local_object_reference_with_local_id(
            file,
            local_objects,
            object.uint_property("targetId"),
        ) else {
            continue;
        };
        let Some((parent_local, parent)) =
            local_object_reference_with_local_id(file, local_objects, object_parent_id(object))
        else {
            continue;
        };
        if is_transform_component(target) && is_transform_component(parent) {
            edges.push(DependencyEdge {
                source_local: target_local,
                dependent_local: parent_local,
                kind: DependencyKind::TargetedConstraint,
            });
        }
    }

    for (target_local, constraint_local) in ik_constraint_target_dependencies(file, local_objects) {
        edges.push(DependencyEdge {
            source_local: target_local,
            dependent_local: constraint_local,
            kind: DependencyKind::IkConstraintTarget,
        });
    }

    for (tip_local, child_local) in
        ik_constraint_tip_child_dependencies(file, local_objects, components)
    {
        edges.push(DependencyEdge {
            source_local: tip_local,
            dependent_local: child_local,
            kind: DependencyKind::IkConstraintTipChild,
        });
    }

    for (constraint_local, parent_local) in
        follow_path_constraint_parent_dependencies(file, local_objects)
    {
        edges.push(DependencyEdge {
            source_local: constraint_local,
            dependent_local: parent_local,
            kind: DependencyKind::FollowPathConstraintParent,
        });
    }

    for (modifier_local, text_local) in
        text_follow_path_modifier_text_dependencies(file, local_objects)
    {
        edges.push(DependencyEdge {
            source_local: modifier_local,
            dependent_local: text_local,
            kind: DependencyKind::TextFollowPathModifierText,
        });
    }

    for (parent_local, effect_local, kind) in group_effect_parent_dependencies(file, local_objects)
    {
        edges.push(DependencyEdge {
            source_local: parent_local,
            dependent_local: effect_local,
            kind,
        });
    }

    for target in draw_targets {
        if let Some(drawable_local) = target.drawable_local {
            edges.push(DependencyEdge {
                source_local: drawable_local,
                dependent_local: target.local_id,
                kind: DependencyKind::DrawTargetDrawable,
            });
        }
    }

    for rules in draw_rules {
        if let Some(target_local) = rules.active_target_local {
            edges.push(DependencyEdge {
                source_local: target_local,
                dependent_local: rules.local_id,
                kind: DependencyKind::DrawRulesTarget,
            });
        }
    }

    for clipping_shape in clipping_shapes {
        if let Some(source_local) = clipping_shape.source_local {
            edges.push(DependencyEdge {
                source_local,
                dependent_local: clipping_shape.local_id,
                kind: DependencyKind::ClippingSource,
            });
        }
    }

    for (skin_local, skinnable_local, kind) in skin_skinnable_dependencies(file, local_objects) {
        edges.push(DependencyEdge {
            source_local: skin_local,
            dependent_local: skinnable_local,
            kind,
        });
    }

    for (source_local, skin_local, kind) in skin_tendon_dependencies(file, local_objects) {
        edges.push(DependencyEdge {
            source_local,
            dependent_local: skin_local,
            kind,
        });
    }

    for (source_local, joystick_local, kind) in joystick_dependencies(file, local_objects) {
        edges.push(DependencyEdge {
            source_local,
            dependent_local: joystick_local,
            kind,
        });
    }

    for (scroll_constraint_local, scroll_bar_local) in
        scroll_bar_constraint_dependencies(file, local_objects)
    {
        edges.push(DependencyEdge {
            source_local: scroll_constraint_local,
            dependent_local: scroll_bar_local,
            kind: DependencyKind::ScrollBarConstraint,
        });
    }

    for (scroll_constraint_local, child_local) in
        scroll_constraint_layout_child_dependencies(file, local_objects, components)
    {
        edges.push(DependencyEdge {
            source_local: scroll_constraint_local,
            dependent_local: child_local,
            kind: DependencyKind::ScrollConstraintLayoutChild,
        });
    }

    dedup_preserving_order(&mut edges);
    edges
}

fn sort_dependency_edges(edges: &mut Vec<DependencyEdge>) {
    edges.sort_by_key(|edge| {
        (
            edge.source_local,
            edge.dependent_local,
            dependency_kind_sort_key(edge.kind),
        )
    });
    edges.dedup();
}

fn dedup_preserving_order<T: Copy + PartialEq>(items: &mut Vec<T>) {
    let mut deduped = Vec::with_capacity(items.len());
    for item in items.iter().copied() {
        if !deduped.contains(&item) {
            deduped.push(item);
        }
    }
    *items = deduped;
}

fn component_skips_parent_child_dependency(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    local_id: usize,
) -> bool {
    let Some(object) = runtime_object_for_local(file, local_objects, local_id) else {
        return false;
    };
    !component_has_parent_child_dependency(object)
}

fn component_has_parent_child_dependency(object: &RuntimeObject) -> bool {
    let Some(definition) = definition_by_type_key(object.type_key) else {
        return false;
    };

    if object.type_name == "Skin" {
        return false;
    }
    if object.type_name == "Joystick" {
        return false;
    }
    if object.type_name == "TextModifierGroup" {
        return false;
    }
    if object.type_name == "ClippingShape" {
        return false;
    }
    if paint_effect_skips_generic_parent_child_dependency(object) {
        return false;
    }
    if text_variation_child_skips_generic_parent_child_dependency(object) {
        return false;
    }
    if definition.is_a("TargetedConstraint") || definition.is_a("TextModifier") {
        return false;
    }
    if object.type_name == "Mesh" {
        return true;
    }

    definition.is_a("TransformComponent")
        || definition.is_a("Constraint")
        || definition.is_a("TextStyle")
        || matches!(object.type_name, "FocusData" | "SemanticData" | "NSlicer")
}

fn dependency_kind_sort_key(kind: DependencyKind) -> u8 {
    match kind {
        DependencyKind::ParentChild => 0,
        DependencyKind::TargetedConstraint => 1,
        DependencyKind::IkConstraintTarget => 2,
        DependencyKind::IkConstraintTipChild => 3,
        DependencyKind::DrawTargetDrawable => 4,
        DependencyKind::DrawRulesTarget => 5,
        DependencyKind::ClippingSource => 6,
        DependencyKind::SkinMesh => 7,
        DependencyKind::SkinPointsPath => 8,
        DependencyKind::SkinBone => 9,
        DependencyKind::SkinBoneConstraintParent => 10,
        DependencyKind::JoystickParent => 11,
        DependencyKind::JoystickHandleSource => 12,
        DependencyKind::ScrollBarConstraint => 13,
        DependencyKind::ScrollConstraintLayoutChild => 14,
        DependencyKind::PathComposerShape => 15,
        DependencyKind::PathComposerPath => 16,
        DependencyKind::ClippingShapePathComposer => 17,
        DependencyKind::FollowPathConstraintParent => 18,
        DependencyKind::FollowPathConstraintTargetPathComposer => 19,
        DependencyKind::FollowPathConstraintTargetPath => 20,
        DependencyKind::TextFollowPathModifierText => 21,
        DependencyKind::TextFollowPathModifierTargetPathComposer => 22,
        DependencyKind::TextFollowPathModifierTargetPath => 23,
        DependencyKind::StrokePathBuilder => 24,
        DependencyKind::FillPathBuilder => 25,
        DependencyKind::FeatherPathBuilder => 26,
        DependencyKind::GroupEffectParent => 27,
        DependencyKind::ScriptedPathEffectParent => 28,
        DependencyKind::LinearGradientPaintContainer => 29,
        DependencyKind::TextVariationHelperArtboard => 30,
        DependencyKind::TextVariationHelperText => 31,
    }
}

fn ik_constraint_target_dependencies(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
) -> Vec<(usize, usize)> {
    let mut edges = Vec::new();
    for local_object in local_objects {
        let Some(object) = runtime_object_for_local(file, local_objects, local_object.local_id)
        else {
            continue;
        };
        if object.type_name != "IKConstraint" {
            continue;
        }

        let Some((target_local, target)) = local_object_reference_with_local_id(
            file,
            local_objects,
            object.uint_property("targetId"),
        ) else {
            continue;
        };
        if is_transform_component(target) {
            edges.push((target_local, local_object.local_id));
        }
    }
    edges
}

fn ik_constraint_tip_child_dependencies(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    components: &[ComponentNode],
) -> Vec<(usize, usize)> {
    let component_by_local = components
        .iter()
        .enumerate()
        .map(|(index, component)| (component.local_id, index))
        .collect::<BTreeMap<_, _>>();
    let mut edges = Vec::new();

    for local_object in local_objects {
        let Some(constraint) = runtime_object_for_local(file, local_objects, local_object.local_id)
        else {
            continue;
        };
        if constraint.type_name != "IKConstraint" {
            continue;
        }

        let Some((tip_local, tip)) =
            local_object_reference_with_local_id(file, local_objects, object_parent_id(constraint))
        else {
            continue;
        };
        if !is_bone(tip) {
            continue;
        }

        let mut chain_locals = vec![tip_local];
        let mut current = tip;
        let mut remaining = constraint.uint_property("parentBoneCount").unwrap_or(0);
        while remaining > 0 {
            let Some((parent_local, parent)) = local_object_reference_with_local_id(
                file,
                local_objects,
                object_parent_id(current),
            ) else {
                break;
            };
            if !is_bone(parent) {
                break;
            }
            remaining -= 1;
            chain_locals.push(parent_local);
            current = parent;
        }

        for window in chain_locals.windows(2) {
            let chain_child_local = window[0];
            let ancestor_local = window[1];
            let Some(ancestor_index) = component_by_local.get(&ancestor_local).copied() else {
                continue;
            };

            for child_local in &components[ancestor_index].children {
                if *child_local == chain_child_local {
                    continue;
                }
                let Some(child) = runtime_object_for_local(file, local_objects, *child_local)
                else {
                    continue;
                };
                if is_transform_component(child) {
                    edges.push((tip_local, *child_local));
                }
            }
        }
    }

    edges
}

fn follow_path_constraint_parent_dependencies(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
) -> Vec<(usize, usize)> {
    let mut edges = Vec::new();
    for local_object in local_objects {
        let Some(constraint) = runtime_object_for_local(file, local_objects, local_object.local_id)
        else {
            continue;
        };
        if !is_follow_path_constraint(constraint) {
            continue;
        }

        let Some((parent_local, parent)) =
            local_object_reference_with_local_id(file, local_objects, object_parent_id(constraint))
        else {
            continue;
        };
        if is_transform_component(parent) {
            edges.push((local_object.local_id, parent_local));
        }
    }
    edges
}

fn follow_path_constraint_target_node_dependencies(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    dependency_nodes: &[DependencyNode],
    path_composers: &[PathComposerNode],
    component_node_by_local: &BTreeMap<usize, usize>,
    path_composer_node_by_shape: &BTreeMap<usize, usize>,
) -> Vec<(usize, usize, DependencyKind)> {
    let mut edges = Vec::new();
    for node in dependency_nodes {
        let DependencyNodeKind::Component { local_id, .. } = &node.kind else {
            continue;
        };
        let constraint_local = *local_id;
        let Some(constraint) = runtime_object_for_local(file, local_objects, constraint_local)
        else {
            continue;
        };
        if !is_follow_path_constraint(constraint) {
            continue;
        }

        let Some(constraint_node) = component_node_by_local.get(&constraint_local).copied() else {
            continue;
        };
        let Some((target_local, target)) = local_object_reference_with_local_id(
            file,
            local_objects,
            constraint.uint_property("targetId"),
        ) else {
            continue;
        };

        if is_shape(target) {
            if let Some(path_composer_node) =
                path_composer_node_by_shape.get(&target_local).copied()
            {
                edges.push((
                    path_composer_node,
                    constraint_node,
                    DependencyKind::FollowPathConstraintTargetPathComposer,
                ));
            }
            continue;
        }

        if is_path(target) {
            if let Some(path_composer_node) = shape_local_for_path(path_composers, target_local)
                .and_then(|shape_local| path_composer_node_by_shape.get(&shape_local).copied())
            {
                edges.push((
                    path_composer_node,
                    constraint_node,
                    DependencyKind::FollowPathConstraintTargetPathComposer,
                ));
            } else if let Some(path_node) = component_node_by_local.get(&target_local).copied() {
                edges.push((
                    path_node,
                    constraint_node,
                    DependencyKind::FollowPathConstraintTargetPath,
                ));
            }
        }
    }
    edges
}

fn text_follow_path_modifier_text_dependencies(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
) -> Vec<(usize, usize)> {
    let mut edges = Vec::new();
    for local_object in local_objects {
        let Some(modifier) = runtime_object_for_local(file, local_objects, local_object.local_id)
        else {
            continue;
        };
        if !is_text_follow_path_modifier(modifier) {
            continue;
        }

        if let Some(text_local) = text_component_local_for_modifier(file, local_objects, modifier) {
            edges.push((local_object.local_id, text_local));
        }
    }
    edges
}

fn text_follow_path_modifier_target_node_dependencies(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    dependency_nodes: &[DependencyNode],
    component_node_by_local: &BTreeMap<usize, usize>,
    path_composer_node_by_shape: &BTreeMap<usize, usize>,
) -> Vec<(usize, usize, DependencyKind)> {
    let mut edges = Vec::new();
    for node in dependency_nodes {
        let DependencyNodeKind::Component { local_id, .. } = &node.kind else {
            continue;
        };
        let modifier_local = *local_id;
        let Some(modifier) = runtime_object_for_local(file, local_objects, modifier_local) else {
            continue;
        };
        if !is_text_follow_path_modifier(modifier) {
            continue;
        }

        let Some(modifier_node) = component_node_by_local.get(&modifier_local).copied() else {
            continue;
        };
        let Some((target_local, target)) = local_object_reference_with_local_id(
            file,
            local_objects,
            modifier.uint_property("targetId"),
        ) else {
            continue;
        };

        if is_shape(target) {
            if let Some(path_composer_node) =
                path_composer_node_by_shape.get(&target_local).copied()
            {
                edges.push((
                    path_composer_node,
                    modifier_node,
                    DependencyKind::TextFollowPathModifierTargetPathComposer,
                ));
            }
            continue;
        }

        if is_path(target) {
            if let Some(path_node) = component_node_by_local.get(&target_local).copied() {
                edges.push((
                    path_node,
                    modifier_node,
                    DependencyKind::TextFollowPathModifierTargetPath,
                ));
            }
        }
    }
    edges
}

fn text_component_local_for_modifier(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    modifier: &RuntimeObject,
) -> Option<usize> {
    let (_, group) =
        local_object_reference_with_local_id(file, local_objects, object_parent_id(modifier))?;
    if group.type_name != "TextModifierGroup" {
        return None;
    }

    let (text_local, text) =
        local_object_reference_with_local_id(file, local_objects, object_parent_id(group))?;
    (text.type_name == "Text").then_some(text_local)
}

fn group_effect_parent_dependencies(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
) -> Vec<(usize, usize, DependencyKind)> {
    let mut edges = Vec::new();
    for local_object in local_objects {
        let Some(effect) = runtime_object_for_local(file, local_objects, local_object.local_id)
        else {
            continue;
        };
        let kind = match effect.type_name {
            "GroupEffect" => DependencyKind::GroupEffectParent,
            "ScriptedPathEffect" => DependencyKind::ScriptedPathEffectParent,
            _ => continue,
        };

        let Some((parent_local, _)) =
            local_object_reference_with_local_id(file, local_objects, object_parent_id(effect))
        else {
            continue;
        };
        edges.push((parent_local, local_object.local_id, kind));
    }
    edges
}

fn stroke_path_builder_node_dependencies(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    dependency_nodes: &[DependencyNode],
    component_node_by_local: &BTreeMap<usize, usize>,
    path_composer_node_by_shape: &BTreeMap<usize, usize>,
) -> Vec<(usize, usize)> {
    let mut edges = Vec::new();
    for node in dependency_nodes {
        let DependencyNodeKind::Component { local_id, .. } = &node.kind else {
            continue;
        };
        let stroke_local = *local_id;
        let Some(stroke) = runtime_object_for_local(file, local_objects, stroke_local) else {
            continue;
        };
        if stroke.type_name != "Stroke" {
            continue;
        }

        let Some(stroke_node) = component_node_by_local.get(&stroke_local).copied() else {
            continue;
        };
        let Some((container_local, container)) =
            local_object_reference_with_local_id(file, local_objects, object_parent_id(stroke))
        else {
            continue;
        };
        let Some(path_builder_node) = shape_paint_container_path_builder_node(
            file,
            local_objects,
            container_local,
            container,
            component_node_by_local,
            path_composer_node_by_shape,
        ) else {
            continue;
        };

        edges.push((path_builder_node, stroke_node));
    }
    edges
}

fn fill_path_builder_node_dependencies(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    dependency_nodes: &[DependencyNode],
    component_node_by_local: &BTreeMap<usize, usize>,
    path_composer_node_by_shape: &BTreeMap<usize, usize>,
) -> Vec<(usize, usize)> {
    let mut edges = Vec::new();
    for node in dependency_nodes {
        let DependencyNodeKind::Component { local_id, .. } = &node.kind else {
            continue;
        };
        let fill_local = *local_id;
        let Some(fill) = runtime_object_for_local(file, local_objects, fill_local) else {
            continue;
        };
        if fill.type_name != "Fill"
            || !shape_paint_has_registered_stroke_effect(file, local_objects, fill_local)
        {
            continue;
        }

        let Some(fill_node) = component_node_by_local.get(&fill_local).copied() else {
            continue;
        };
        let Some((container_local, container)) =
            local_object_reference_with_local_id(file, local_objects, object_parent_id(fill))
        else {
            continue;
        };
        let Some(path_builder_node) = shape_paint_container_path_builder_node(
            file,
            local_objects,
            container_local,
            container,
            component_node_by_local,
            path_composer_node_by_shape,
        ) else {
            continue;
        };

        edges.push((path_builder_node, fill_node));
    }
    edges
}

fn feather_path_builder_node_dependencies(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    dependency_nodes: &[DependencyNode],
    component_node_by_local: &BTreeMap<usize, usize>,
    path_composer_node_by_shape: &BTreeMap<usize, usize>,
) -> Vec<(usize, usize)> {
    let mut edges = Vec::new();
    for node in dependency_nodes {
        let DependencyNodeKind::Component { local_id, .. } = &node.kind else {
            continue;
        };
        let feather_local = *local_id;
        let Some(feather) = runtime_object_for_local(file, local_objects, feather_local) else {
            continue;
        };
        if feather.type_name != "Feather" {
            continue;
        }

        let Some(feather_node) = component_node_by_local.get(&feather_local).copied() else {
            continue;
        };
        let Some((_, shape_paint)) =
            local_object_reference_with_local_id(file, local_objects, object_parent_id(feather))
        else {
            continue;
        };
        if !is_shape_paint(shape_paint) {
            continue;
        }
        let Some((container_local, container)) = local_object_reference_with_local_id(
            file,
            local_objects,
            object_parent_id(shape_paint),
        ) else {
            continue;
        };
        let Some(path_builder_node) = shape_paint_container_dependency_node(
            container_local,
            container,
            component_node_by_local,
            path_composer_node_by_shape,
        ) else {
            continue;
        };

        edges.push((path_builder_node, feather_node));
    }
    edges
}

fn linear_gradient_paint_container_node_dependencies(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    dependency_nodes: &[DependencyNode],
    component_node_by_local: &BTreeMap<usize, usize>,
) -> Vec<(usize, usize)> {
    let mut edges = Vec::new();
    for node in dependency_nodes {
        let DependencyNodeKind::Component { local_id, .. } = &node.kind else {
            continue;
        };
        let gradient_local = *local_id;
        let Some(gradient) = runtime_object_for_local(file, local_objects, gradient_local) else {
            continue;
        };
        if !is_linear_gradient(gradient) {
            continue;
        }

        let Some(gradient_node) = component_node_by_local.get(&gradient_local).copied() else {
            continue;
        };
        let Some((_, shape_paint)) =
            local_object_reference_with_local_id(file, local_objects, object_parent_id(gradient))
        else {
            continue;
        };
        if !is_shape_paint(shape_paint) {
            continue;
        }
        let Some((container_local, container)) = local_object_reference_with_local_id(
            file,
            local_objects,
            object_parent_id(shape_paint),
        ) else {
            continue;
        };
        let Some(container_node) = linear_gradient_paint_container_dependency_node(
            file,
            local_objects,
            container_local,
            container,
            component_node_by_local,
        ) else {
            continue;
        };

        edges.push((container_node, gradient_node));
    }
    edges
}

fn linear_gradient_paint_container_dependency_node(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    container_local: usize,
    container: &RuntimeObject,
    component_node_by_local: &BTreeMap<usize, usize>,
) -> Option<usize> {
    let mut current_local = container_local;
    let mut current = container;

    // Cycle guard: a malformed-but-accepted file can make this parent reference
    // chain cyclic; C++ would hang. Terminate with a visited set, mirroring
    // C++'s own DependencySorter::visit cycle-guard idiom
    // (src/dependency_sorter.cpp). Falling out of the loop yields the same
    // container fallback as a chain that simply ends. No-op on valid files.
    let mut visited = BTreeSet::new();
    loop {
        if is_node(current) {
            return component_node_by_local.get(&current_local).copied();
        }
        if !visited.insert(current_local) {
            break;
        }

        let Some((parent_local, parent)) =
            local_object_reference_with_local_id(file, local_objects, object_parent_id(current))
        else {
            break;
        };
        current_local = parent_local;
        current = parent;
    }

    component_node_by_local.get(&container_local).copied()
}

fn shape_paint_container_path_builder_node(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    container_local: usize,
    container: &RuntimeObject,
    component_node_by_local: &BTreeMap<usize, usize>,
    path_composer_node_by_shape: &BTreeMap<usize, usize>,
) -> Option<usize> {
    if container.type_name == "Shape" {
        return path_composer_node_by_shape.get(&container_local).copied();
    }

    if matches!(container.type_name, "Artboard" | "LayoutComponent") {
        return component_node_by_local.get(&container_local).copied();
    }

    if shape_paint_container_path_builder_is_parent(container) {
        let (parent_local, _) =
            local_object_reference_with_local_id(file, local_objects, object_parent_id(container))?;
        return component_node_by_local.get(&parent_local).copied();
    }

    None
}

fn shape_paint_container_dependency_node(
    container_local: usize,
    container: &RuntimeObject,
    component_node_by_local: &BTreeMap<usize, usize>,
    path_composer_node_by_shape: &BTreeMap<usize, usize>,
) -> Option<usize> {
    if container.type_name == "Shape" {
        path_composer_node_by_shape.get(&container_local).copied()
    } else {
        component_node_by_local.get(&container_local).copied()
    }
}

fn shape_paint_container_path_builder_is_parent(container: &RuntimeObject) -> bool {
    matches!(
        container.type_name,
        "TextStylePaint"
            | "ForegroundLayoutDrawable"
            | "TextInputCursor"
            | "TextInputSelection"
            | "TextInputText"
            | "TextInputSelectedText"
    )
}

fn shape_paint_has_registered_stroke_effect(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    shape_paint_local: usize,
) -> bool {
    local_objects.iter().any(|local_object| {
        let Some(effect) = runtime_object_for_local(file, local_objects, local_object.local_id)
        else {
            return false;
        };
        object_parent_id(effect) == Some(shape_paint_local as u64)
            && is_registered_stroke_effect(effect)
    })
}

fn is_registered_stroke_effect(object: &RuntimeObject) -> bool {
    matches!(
        object.type_name,
        "DashPath" | "TargetEffect" | "TrimPath" | "ScriptedPathEffect"
    )
}

fn paint_effect_skips_generic_parent_child_dependency(object: &RuntimeObject) -> bool {
    if is_linear_gradient(object) {
        return true;
    }

    matches!(
        object.type_name,
        "Fill"
            | "Stroke"
            | "Feather"
            | "DashPath"
            | "TargetEffect"
            | "TrimPath"
            | "GroupEffect"
            | "ScriptedPathEffect"
    )
}

fn text_variation_child_skips_generic_parent_child_dependency(object: &RuntimeObject) -> bool {
    matches!(object.type_name, "TextStyleAxis" | "TextStyleFeature")
}

fn skin_skinnable_dependencies(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
) -> Vec<(usize, usize, DependencyKind)> {
    let mut edges = Vec::new();
    for local_object in local_objects {
        let Some(skin) = runtime_object_for_local(file, local_objects, local_object.local_id)
        else {
            continue;
        };
        if skin.type_name != "Skin" {
            continue;
        }

        let Some((parent_local, parent)) =
            local_object_reference_with_local_id(file, local_objects, object_parent_id(skin))
        else {
            continue;
        };
        let kind = match parent.type_name {
            "Mesh" => DependencyKind::SkinMesh,
            "PointsPath" => DependencyKind::SkinPointsPath,
            _ => continue,
        };
        edges.push((local_object.local_id, parent_local, kind));
    }
    edges
}

fn skin_tendon_dependencies(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
) -> Vec<(usize, usize, DependencyKind)> {
    let peer_constraints = ik_peer_constraints_by_bone(file, local_objects);
    let mut edges = Vec::new();

    for local_object in local_objects {
        let Some(tendon) = runtime_object_for_local(file, local_objects, local_object.local_id)
        else {
            continue;
        };
        if tendon.type_name != "Tendon" {
            continue;
        }

        let Some((skin_local, skin)) =
            local_object_reference_with_local_id(file, local_objects, object_parent_id(tendon))
        else {
            continue;
        };
        if skin.type_name != "Skin" {
            continue;
        }

        let Some((bone_local, bone)) = local_object_reference_with_local_id(
            file,
            local_objects,
            tendon.uint_property("boneId"),
        ) else {
            continue;
        };
        if !is_bone(bone) {
            continue;
        }

        edges.push((bone_local, skin_local, DependencyKind::SkinBone));

        for constraint_local in peer_constraints
            .get(&bone_local)
            .into_iter()
            .flatten()
            .copied()
        {
            let Some(constraint) = runtime_object_for_local(file, local_objects, constraint_local)
            else {
                continue;
            };
            let Some((parent_local, parent)) = local_object_reference_with_local_id(
                file,
                local_objects,
                object_parent_id(constraint),
            ) else {
                continue;
            };
            if is_transform_component(parent) {
                edges.push((
                    parent_local,
                    skin_local,
                    DependencyKind::SkinBoneConstraintParent,
                ));
            }
        }
    }

    edges
}

fn ik_peer_constraints_by_bone(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
) -> BTreeMap<usize, Vec<usize>> {
    let mut peers = BTreeMap::<usize, Vec<usize>>::new();

    for local_object in local_objects {
        let Some(constraint) = runtime_object_for_local(file, local_objects, local_object.local_id)
        else {
            continue;
        };
        if constraint.type_name != "IKConstraint" {
            continue;
        }

        let Some((_tip_local, mut bone)) =
            local_object_reference_with_local_id(file, local_objects, object_parent_id(constraint))
        else {
            continue;
        };
        if !is_bone(bone) {
            continue;
        }

        let mut remaining = constraint.uint_property("parentBoneCount").unwrap_or(0);
        while remaining > 0 {
            let Some((parent_local, parent)) =
                local_object_reference_with_local_id(file, local_objects, object_parent_id(bone))
            else {
                break;
            };
            if !is_bone(parent) {
                break;
            }

            remaining -= 1;
            bone = parent;
            let bone_local = parent_local;
            push_unique(peers.entry(bone_local).or_default(), local_object.local_id);
        }
    }

    peers
}

fn joystick_dependencies(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
) -> Vec<(usize, usize, DependencyKind)> {
    let mut edges = Vec::new();
    for local_object in local_objects {
        let Some(joystick) = runtime_object_for_local(file, local_objects, local_object.local_id)
        else {
            continue;
        };
        if joystick.type_name != "Joystick" {
            continue;
        }

        let Some((parent_local, _parent)) =
            local_object_reference_with_local_id(file, local_objects, object_parent_id(joystick))
        else {
            continue;
        };
        let Some((handle_source_local, _handle_source)) = local_object_reference_with_local_id(
            file,
            local_objects,
            joystick.uint_property("handleSourceId"),
        )
        .filter(|(_, source)| is_transform_component(source)) else {
            continue;
        };

        edges.push((
            parent_local,
            local_object.local_id,
            DependencyKind::JoystickParent,
        ));
        edges.push((
            handle_source_local,
            local_object.local_id,
            DependencyKind::JoystickHandleSource,
        ));
    }
    edges
}

fn scroll_bar_constraint_dependencies(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
) -> Vec<(usize, usize)> {
    let mut edges = Vec::new();
    for local_object in local_objects {
        let Some(scroll_bar) = runtime_object_for_local(file, local_objects, local_object.local_id)
        else {
            continue;
        };
        if scroll_bar.type_name != "ScrollBarConstraint" {
            continue;
        }

        let Some((scroll_constraint_local, scroll_constraint)) =
            local_object_reference_with_local_id(
                file,
                local_objects,
                scroll_bar.uint_property("scrollConstraintId"),
            )
        else {
            continue;
        };
        if is_scroll_constraint(scroll_constraint) {
            edges.push((scroll_constraint_local, local_object.local_id));
        }
    }
    edges
}

fn scroll_constraint_layout_child_dependencies(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    components: &[ComponentNode],
) -> Vec<(usize, usize)> {
    let component_by_local = components
        .iter()
        .map(|component| (component.local_id, component))
        .collect::<BTreeMap<_, _>>();
    let mut edges = Vec::new();
    for local_object in local_objects {
        let Some(scroll_constraint) =
            runtime_object_for_local(file, local_objects, local_object.local_id)
        else {
            continue;
        };
        if !is_scroll_constraint(scroll_constraint) {
            continue;
        }

        let Some(content_local) =
            object_parent_id(scroll_constraint).and_then(|parent| usize::try_from(parent).ok())
        else {
            continue;
        };
        let Some(content) = component_by_local.get(&content_local) else {
            continue;
        };

        for child_local in &content.children {
            let Some(child) = runtime_object_for_local(file, local_objects, *child_local) else {
                continue;
            };
            if is_layout_node_provider(child) {
                edges.push((local_object.local_id, *child_local));
            }
        }
    }
    edges
}

fn descendant_component_locals_inclusive(
    local_id: usize,
    components: &[ComponentNode],
    component_by_local: &BTreeMap<usize, usize>,
) -> Vec<usize> {
    let mut locals = Vec::new();
    let mut visited = BTreeSet::new();
    collect_descendant_component_locals(
        local_id,
        components,
        component_by_local,
        &mut locals,
        &mut visited,
    );
    locals
}

fn collect_descendant_component_locals(
    local_id: usize,
    components: &[ComponentNode],
    component_by_local: &BTreeMap<usize, usize>,
    locals: &mut Vec<usize>,
    visited: &mut BTreeSet<usize>,
) {
    // Cycle guard: a malformed-but-accepted file can make the parent/child
    // component graph cyclic, which turns this descendant walk into unbounded
    // recursion -> stack overflow. C++ hangs/overflows on the same input; we
    // deliberately DIVERGE and terminate, mirroring C++'s own visited-set
    // cycle-guard idiom (DependencySorter::visit's m_Perm/m_Temp sets,
    // src/dependency_sorter.cpp). A node is collected at most once, which is
    // identical behavior on any valid (acyclic) tree.
    if !visited.insert(local_id) {
        return;
    }
    locals.push(local_id);

    let Some(index) = component_by_local.get(&local_id) else {
        return;
    };
    for child in &components[*index].children {
        collect_descendant_component_locals(
            *child,
            components,
            component_by_local,
            locals,
            visited,
        );
    }
}

fn drawable_is_child_of_layout(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    drawable_local: usize,
    layout_local: usize,
) -> bool {
    let mut current_local = Some(drawable_local);
    // Cycle guard: the `parent_local == Some(local_id)` check below only stops
    // self-loops; a longer malformed parent cycle (A -> B -> A) would loop
    // forever. Terminate with a visited set, mirroring C++'s
    // DependencySorter::visit cycle-guard idiom (src/dependency_sorter.cpp).
    // Treated as "not a child of the layout", same as a chain that ends.
    let mut visited = BTreeSet::new();
    while let Some(local_id) = current_local {
        if !visited.insert(local_id) {
            return false;
        }
        let Some(object) = runtime_object_for_local(file, local_objects, local_id) else {
            return false;
        };
        if object.type_name == "LayoutComponent" && local_id == layout_local {
            return true;
        }
        let parent_local = object_parent_id(object).and_then(|parent| usize::try_from(parent).ok());
        if parent_local == Some(local_id) {
            return false;
        }
        current_local = parent_local;
    }
    false
}

fn resolve_parents(components: &mut [ComponentNode]) -> usize {
    let local_to_global = components
        .iter()
        .map(|component| (component.local_id, component.global_id))
        .collect::<BTreeMap<_, _>>();
    let mut resolved = 0;
    for component in components {
        let Some(parent_local) = component.parent_local else {
            continue;
        };
        match local_to_global.get(&parent_local) {
            Some(parent_global) => {
                component.parent_global = Some(*parent_global);
                resolved += 1;
            }
            None => component.missing_parent = true,
        }
    }
    resolved
}

fn index_children(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    components: &mut [ComponentNode],
    component_by_local: &BTreeMap<usize, usize>,
) -> usize {
    for component in components.iter_mut() {
        component.children.clear();
        if let Some(parent_local) = component.parent_local {
            if !component_by_local.contains_key(&parent_local) {
                component.missing_parent = true;
            }
        }
    }

    let mut edge_count = 0;
    let component_locals = components
        .iter()
        .map(|component| component.local_id)
        .collect::<Vec<_>>();

    for local_id in component_locals {
        let Some(component_index) = component_by_local.get(&local_id).copied() else {
            continue;
        };
        let parent_local = components[component_index].parent_local;
        if let Some(parent_local) = parent_local {
            if let Some(parent_index) = component_by_local.get(&parent_local).copied() {
                components[parent_index].children.push(local_id);
                edge_count += 1;
            }
        }

        let Some(object) = runtime_object_for_local(file, local_objects, local_id) else {
            continue;
        };
        if !definition_by_type_key(object.type_key)
            .is_some_and(|definition| definition.is_a("LayoutComponent"))
        {
            continue;
        }
        let Some((style_local, style)) = local_object_reference_with_local_id(
            file,
            local_objects,
            object.uint_property("styleId"),
        ) else {
            continue;
        };
        if style.type_name == "LayoutComponentStyle"
            && component_by_local.contains_key(&style_local)
        {
            components[component_index].children.push(style_local);
            edge_count += 1;
        }
    }

    edge_count
}

fn index_transform_constraints(
    file: &RuntimeFile,
    local_objects: &[LocalObject],
    components: &mut [ComponentNode],
    component_by_local: &BTreeMap<usize, usize>,
) -> usize {
    for component in components.iter_mut() {
        component.constraint_locals.clear();
    }

    let component_locals = components
        .iter()
        .map(|component| component.local_id)
        .collect::<Vec<_>>();
    let mut registration_count = 0;

    for local_id in component_locals {
        let Some(object) = runtime_object_for_local(file, local_objects, local_id) else {
            continue;
        };
        let Some(definition) = definition_by_type_key(object.type_key) else {
            continue;
        };
        if !definition.is_a("Constraint") {
            continue;
        }

        let Some(parent_local) =
            object_parent_id(object).and_then(|parent| usize::try_from(parent).ok())
        else {
            continue;
        };
        let Some(parent_index) = component_by_local.get(&parent_local).copied() else {
            continue;
        };
        let Some(parent) = runtime_object_for_local(file, local_objects, parent_local) else {
            continue;
        };
        if !is_transform_component(parent) {
            continue;
        }

        components[parent_index].constraint_locals.push(local_id);
        registration_count += 1;
    }

    registration_count
}

fn index_component_dependents(
    components: &mut [ComponentNode],
    dependency_nodes: &[DependencyNode],
    dependency_node_edges: &[DependencyNodeEdge],
    draw_target_dependency_edges: &[DrawTargetDependencyEdge],
) {
    for component in components.iter_mut() {
        component.dependent_locals.clear();
    }

    let component_by_local = components
        .iter()
        .enumerate()
        .map(|(index, component)| (component.local_id, index))
        .collect::<BTreeMap<_, _>>();
    let component_local_by_node = component_local_by_dependency_node(dependency_nodes);

    for edge in dependency_node_edges {
        if !dependency_kind_is_component_dependent(edge.kind) {
            continue;
        }
        let Some(source_local) = component_local_by_node.get(&edge.source_node).copied() else {
            continue;
        };
        let Some(dependent_local) = component_local_by_node.get(&edge.dependent_node).copied()
        else {
            continue;
        };
        let Some(source_index) = component_by_local.get(&source_local).copied() else {
            continue;
        };
        if !component_by_local.contains_key(&dependent_local) {
            continue;
        }
        push_unique(
            &mut components[source_index].dependent_locals,
            dependent_local,
        );
    }

    for edge in draw_target_dependency_edges {
        let Some(source_local) = edge.source_local else {
            continue;
        };
        let Some(source_index) = component_by_local.get(&source_local).copied() else {
            continue;
        };
        if !component_by_local.contains_key(&edge.dependent_local) {
            continue;
        }
        push_unique(
            &mut components[source_index].dependent_locals,
            edge.dependent_local,
        );
    }
}

fn dependency_kind_is_component_dependent(kind: DependencyKind) -> bool {
    !matches!(
        kind,
        DependencyKind::DrawTargetDrawable
            | DependencyKind::DrawRulesTarget
            | DependencyKind::ClippingSource
            | DependencyKind::PathComposerShape
            | DependencyKind::PathComposerPath
            | DependencyKind::ClippingShapePathComposer
            | DependencyKind::FollowPathConstraintTargetPathComposer
            | DependencyKind::TextFollowPathModifierTargetPathComposer
            | DependencyKind::TextVariationHelperArtboard
            | DependencyKind::TextVariationHelperText
    )
}

fn graph_diagnostics(
    components: &[ComponentNode],
    draw_targets: &[DrawTargetNode],
    draw_rules: &[DrawRulesNode],
    clipping_shapes: &[ClippingShapeNode],
    dependency_cycles: &[DependencyCycle],
    dependency_node_cycles: &[DependencyNodeCycle],
    draw_target_cycles: &[DrawTargetCycle],
) -> Vec<GraphDiagnostic> {
    let mut diagnostics = Vec::new();

    for component in components {
        if let (true, Some(parent_local)) = (component.missing_parent, component.parent_local) {
            diagnostics.push(GraphDiagnostic::MissingParent {
                component_local: component.local_id,
                parent_local,
            });
        }
    }

    for target in draw_targets {
        if target.drawable_id != 0 && target.drawable_local.is_none() {
            diagnostics.push(GraphDiagnostic::UnresolvedDrawTargetDrawable {
                draw_target_local: target.local_id,
                drawable_id: target.drawable_id,
            });
        }
    }

    for rules in draw_rules {
        if rules.draw_target_id != 0 && rules.active_target_local.is_none() {
            diagnostics.push(GraphDiagnostic::UnresolvedDrawRulesTarget {
                draw_rules_local: rules.local_id,
                draw_target_id: rules.draw_target_id,
            });
        }
    }

    for clipping_shape in clipping_shapes {
        if clipping_shape.source_id != 0 && clipping_shape.source_local.is_none() {
            diagnostics.push(GraphDiagnostic::UnresolvedClippingSource {
                clipping_shape_local: clipping_shape.local_id,
                source_id: clipping_shape.source_id,
            });
        }
    }

    diagnostics.extend(
        dependency_cycles
            .iter()
            .map(|cycle| GraphDiagnostic::DependencyCycle {
                local_ids: cycle.local_ids.clone(),
            }),
    );
    diagnostics.extend(dependency_node_cycles.iter().map(|cycle| {
        GraphDiagnostic::DependencyNodeCycle {
            node_ids: cycle.node_ids.clone(),
        }
    }));
    diagnostics.extend(
        draw_target_cycles
            .iter()
            .map(|cycle| GraphDiagnostic::DrawTargetCycle {
                local_ids: cycle.local_ids.clone(),
            }),
    );

    diagnostics
}

fn component_dependency_node_by_local(
    dependency_nodes: &[DependencyNode],
) -> BTreeMap<usize, usize> {
    dependency_nodes
        .iter()
        .filter_map(|node| match &node.kind {
            DependencyNodeKind::Component { local_id, .. } => Some((*local_id, node.node_id)),
            DependencyNodeKind::PathComposer { .. }
            | DependencyNodeKind::TextVariationHelper { .. } => None,
        })
        .collect()
}

fn component_local_by_dependency_node(
    dependency_nodes: &[DependencyNode],
) -> BTreeMap<usize, usize> {
    dependency_nodes
        .iter()
        .filter_map(|node| match &node.kind {
            DependencyNodeKind::Component { local_id, .. } => Some((node.node_id, *local_id)),
            DependencyNodeKind::PathComposer { .. }
            | DependencyNodeKind::TextVariationHelper { .. } => None,
        })
        .collect()
}

fn path_composer_dependency_node_by_shape(
    dependency_nodes: &[DependencyNode],
) -> BTreeMap<usize, usize> {
    dependency_nodes
        .iter()
        .filter_map(|node| match &node.kind {
            DependencyNodeKind::PathComposer { shape_local, .. } => {
                Some((*shape_local, node.node_id))
            }
            DependencyNodeKind::Component { .. }
            | DependencyNodeKind::TextVariationHelper { .. } => None,
        })
        .collect()
}

fn text_variation_helper_dependency_node_by_style(
    dependency_nodes: &[DependencyNode],
) -> BTreeMap<usize, usize> {
    dependency_nodes
        .iter()
        .filter_map(|node| match &node.kind {
            DependencyNodeKind::TextVariationHelper {
                text_style_local, ..
            } => Some((*text_style_local, node.node_id)),
            DependencyNodeKind::Component { .. } | DependencyNodeKind::PathComposer { .. } => None,
        })
        .collect()
}

fn shape_local_for_path(path_composers: &[PathComposerNode], path_local: usize) -> Option<usize> {
    path_composers
        .iter()
        .find(|composer| composer.path_locals.contains(&path_local))
        .map(|composer| composer.shape_local)
}

fn dependency_component_cycles(
    node_cycles: &[DependencyNodeCycle],
    component_local_by_node: &BTreeMap<usize, usize>,
) -> Vec<DependencyCycle> {
    let mut cycles = Vec::new();
    for node_cycle in node_cycles {
        let mut local_ids = Vec::new();
        for node_id in &node_cycle.node_ids {
            let Some(local_id) = component_local_by_node.get(node_id).copied() else {
                local_ids.clear();
                break;
            };
            local_ids.push(local_id);
        }
        if !local_ids.is_empty() {
            let cycle = DependencyCycle { local_ids };
            if !cycles.contains(&cycle) {
                cycles.push(cycle);
            }
        }
    }
    cycles
}

struct DependencyOrder {
    component_order: Vec<usize>,
    runtime_node_order: Vec<usize>,
    node_order: Vec<usize>,
    cycles: Vec<DependencyCycle>,
    node_cycles: Vec<DependencyNodeCycle>,
}

fn build_dependency_order(
    components: &mut [ComponentNode],
    component_by_local: &BTreeMap<usize, usize>,
    dependency_nodes: &[DependencyNode],
    dependency_node_edges: &[DependencyNodeEdge],
) -> DependencyOrder {
    let mut complete_node_order = Vec::new();
    let mut node_cycles = Vec::new();
    let mut permanent = BTreeSet::new();
    let mut temporary = BTreeSet::new();
    let mut visiting = Vec::new();
    let component_node_by_local = component_dependency_node_by_local(dependency_nodes);
    let component_local_by_node = component_local_by_dependency_node(dependency_nodes);
    let roots = components
        .iter()
        .filter(|component| component.parent_local.is_none() || component.missing_parent)
        .filter_map(|component| component_node_by_local.get(&component.local_id).copied())
        .collect::<Vec<_>>();
    let mut dependents_by_source = BTreeMap::<usize, Vec<usize>>::new();
    let mut graph_order_dependents_by_source = BTreeMap::<usize, Vec<usize>>::new();

    for edge in dependency_node_edges {
        push_unique(
            dependents_by_source.entry(edge.source_node).or_default(),
            edge.dependent_node,
        );
        if dependency_kind_affects_component_graph_order(edge.kind) {
            push_unique(
                graph_order_dependents_by_source
                    .entry(edge.source_node)
                    .or_default(),
                edge.dependent_node,
            );
        }
    }

    let mut graph_node_order = Vec::new();
    let mut graph_permanent = BTreeSet::new();
    let mut graph_temporary = BTreeSet::new();
    let mut graph_visiting = Vec::new();
    let mut graph_node_cycles = Vec::new();
    for root in roots {
        visit_dependency_node(
            root,
            &graph_order_dependents_by_source,
            &mut graph_permanent,
            &mut graph_temporary,
            &mut graph_visiting,
            &mut graph_node_order,
            &mut graph_node_cycles,
        );
    }

    for node in dependency_nodes {
        visit_dependency_node(
            node.node_id,
            &dependents_by_source,
            &mut permanent,
            &mut temporary,
            &mut visiting,
            &mut complete_node_order,
            &mut node_cycles,
        );
    }

    let component_order = complete_node_order
        .iter()
        .filter_map(|node_id| component_local_by_node.get(node_id).copied())
        .collect::<Vec<_>>();

    for component in components.iter_mut() {
        component.graph_order = None;
    }
    for (graph_order, node_id) in graph_node_order.iter().enumerate() {
        if let Some(local_id) = component_local_by_node.get(node_id) {
            if let Some(index) = component_by_local.get(local_id) {
                components[*index].graph_order = Some(graph_order);
            }
        }
    }

    let cycles = dependency_component_cycles(&node_cycles, &component_local_by_node);

    DependencyOrder {
        component_order,
        runtime_node_order: graph_node_order,
        node_order: complete_node_order,
        cycles,
        node_cycles,
    }
}

fn dependency_kind_affects_component_graph_order(kind: DependencyKind) -> bool {
    !matches!(
        kind,
        DependencyKind::DrawTargetDrawable
            | DependencyKind::DrawRulesTarget
            | DependencyKind::ClippingSource
    )
}

fn visit_dependency_node(
    node_id: usize,
    dependents_by_source: &BTreeMap<usize, Vec<usize>>,
    permanent: &mut BTreeSet<usize>,
    temporary: &mut BTreeSet<usize>,
    visiting: &mut Vec<usize>,
    order: &mut Vec<usize>,
    cycles: &mut Vec<DependencyNodeCycle>,
) {
    if permanent.contains(&node_id) {
        return;
    }
    if temporary.contains(&node_id) {
        if let Some(start) = visiting.iter().position(|visited| *visited == node_id) {
            let mut node_ids = visiting[start..].to_vec();
            node_ids.push(node_id);
            let cycle = DependencyNodeCycle { node_ids };
            if !cycles.contains(&cycle) {
                cycles.push(cycle);
            }
        }
        return;
    }

    temporary.insert(node_id);
    visiting.push(node_id);

    if let Some(dependents) = dependents_by_source.get(&node_id) {
        for dependent in dependents {
            visit_dependency_node(
                *dependent,
                dependents_by_source,
                permanent,
                temporary,
                visiting,
                order,
                cycles,
            );
        }
    }

    visiting.pop();
    temporary.remove(&node_id);
    permanent.insert(node_id);
    order.insert(0, node_id);
}

fn push_unique(values: &mut Vec<usize>, value: usize) {
    if !values.contains(&value) {
        values.push(value);
    }
}

fn is_component(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key).is_some_and(|definition| definition.is_a("Component"))
}

fn validate_local_objects(file: &RuntimeFile, local_objects: &mut [LocalObject]) {
    loop {
        let mut changed = false;
        for index in 1..local_objects.len() {
            if local_objects[index].type_name.is_none() {
                continue;
            }
            if !local_object_is_valid(file, local_objects, index) {
                local_objects[index].type_name = None;
                local_objects[index].name = None;
                changed = true;
            }
        }

        if !changed {
            break;
        }
    }
}

fn local_object_is_valid(file: &RuntimeFile, local_objects: &[LocalObject], index: usize) -> bool {
    let Some(object) = runtime_object_for_local(file, local_objects, index) else {
        return false;
    };
    let Some(definition) = definition_by_type_key(object.type_key) else {
        return false;
    };

    if definition.name == "Artboard" {
        return true;
    }

    if definition.is_a("Component") {
        let Some(parent) = local_object_reference(file, local_objects, object_parent_id(object))
        else {
            return false;
        };
        let component_list_machine_selection =
            definition.is_a("NestedAnimation") && parent.type_name == "ArtboardListMapRule";
        if !is_container_component(parent) && !component_list_machine_selection {
            return false;
        }
    }

    if definition.is_a("TargetedConstraint") {
        match local_object_reference(file, local_objects, object.uint_property("targetId")) {
            Some(target) => return is_transform_component(target),
            None => return !targeted_constraint_requires_target(definition.name),
        }
    }

    if definition.is_a("NestedAnimation") {
        let Some(parent) = local_object_reference(file, local_objects, object_parent_id(object))
        else {
            return false;
        };
        return is_nested_artboard(parent) || parent.type_name == "ArtboardListMapRule";
    }

    if definition.name == "TextStyle" {
        let Some(parent) = local_object_reference(file, local_objects, object_parent_id(object))
        else {
            return false;
        };
        return is_text_interface(parent);
    }

    if definition.name == "ScrollBarConstraint" {
        let Some(scroll_constraint) = local_object_reference(
            file,
            local_objects,
            object.uint_property("scrollConstraintId"),
        ) else {
            return false;
        };
        return is_scroll_constraint(scroll_constraint);
    }

    if definition.name == "Feather" {
        let Some(parent) = local_object_reference(file, local_objects, object_parent_id(object))
        else {
            return false;
        };
        return is_shape_paint(parent);
    }

    true
}

fn runtime_object_for_local<'a>(
    file: &'a RuntimeFile,
    local_objects: &[LocalObject],
    index: usize,
) -> Option<&'a RuntimeObject> {
    let local_object = local_objects.get(index)?;
    local_object.type_name?;
    file.objects
        .get(local_object.global_id as usize)
        .and_then(|object| object.as_ref())
}

fn local_object_global_id(local_objects: &[LocalObject], local_id: usize) -> Option<u32> {
    local_objects
        .get(local_id)
        .and_then(|local_object| local_object.type_name.map(|_| local_object.global_id))
}

fn cpp_i32_from_runtime_uint(value: u64) -> i64 {
    i64::from(value as u32 as i32)
}

fn local_object_reference<'a>(
    file: &'a RuntimeFile,
    local_objects: &[LocalObject],
    id: Option<u64>,
) -> Option<&'a RuntimeObject> {
    local_object_reference_with_local_id(file, local_objects, id).map(|(_, object)| object)
}

fn local_object_reference_with_local_id<'a>(
    file: &'a RuntimeFile,
    local_objects: &[LocalObject],
    id: Option<u64>,
) -> Option<(usize, &'a RuntimeObject)> {
    let id = usize::try_from(id?).ok()?;
    let object = runtime_object_for_local(file, local_objects, id)?;
    Some((id, object))
}

fn is_container_component(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key)
        .is_some_and(|definition| definition.is_a("ContainerComponent"))
}

fn is_transform_component(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key)
        .is_some_and(|definition| definition.is_a("TransformComponent"))
}

fn is_nested_artboard(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key)
        .is_some_and(|definition| definition.is_a("NestedArtboard"))
}

fn is_exact_nested_artboard_host(object: &RuntimeObject) -> bool {
    matches!(
        object.type_name,
        "NestedArtboard" | "NestedArtboardLeaf" | "NestedArtboardLayout"
    )
}

fn is_text_interface(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key)
        .is_some_and(|definition| matches!(definition.name, "Text" | "TextInput"))
}

fn is_text_style(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key).is_some_and(|definition| definition.is_a("TextStyle"))
}

fn is_list_constraint(object: &RuntimeObject) -> bool {
    matches!(object.type_name, "ListFollowPathConstraint")
}

fn is_constrainable_list(object: &RuntimeObject) -> bool {
    matches!(object.type_name, "ArtboardComponentList")
}

fn is_artboard_component_list(object: &RuntimeObject) -> bool {
    matches!(object.type_name, "ArtboardComponentList")
}

fn resetting_component_kind(object: &RuntimeObject) -> Option<ResettingComponentKind> {
    match object.type_name {
        "NestedArtboard" | "NestedArtboardLeaf" | "NestedArtboardLayout" => {
            Some(ResettingComponentKind::NestedArtboard)
        }
        "ArtboardComponentList" => Some(ResettingComponentKind::ArtboardComponentList),
        "CustomPropertyTrigger" => Some(ResettingComponentKind::CustomPropertyTrigger),
        _ => None,
    }
}

fn advancing_component_kind(object: &RuntimeObject) -> Option<AdvancingComponentKind> {
    match object.type_name {
        "Artboard" => Some(AdvancingComponentKind::Artboard),
        "NestedArtboard" | "NestedArtboardLeaf" | "NestedArtboardLayout" => {
            Some(AdvancingComponentKind::NestedArtboard)
        }
        "LayoutComponent" => Some(AdvancingComponentKind::LayoutComponent),
        "ArtboardComponentList" => Some(AdvancingComponentKind::ArtboardComponentList),
        "ScrollConstraint" => Some(AdvancingComponentKind::ScrollConstraint),
        "TextInput" => Some(AdvancingComponentKind::TextInput),
        "ScriptedDataConverter" => Some(AdvancingComponentKind::ScriptedDataConverter),
        "ScriptedDrawable" => Some(AdvancingComponentKind::ScriptedDrawable),
        "ScriptedLayout" => Some(AdvancingComponentKind::ScriptedLayout),
        "ScriptedPathEffect" => Some(AdvancingComponentKind::ScriptedPathEffect),
        _ => None,
    }
}

fn is_scroll_constraint(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key)
        .is_some_and(|definition| definition.name == "ScrollConstraint")
}

fn is_layout_node_provider(object: &RuntimeObject) -> bool {
    matches!(
        object.type_name,
        "LayoutComponent" | "NestedArtboardLayout" | "ArtboardComponentList"
    )
}

fn is_shape_paint(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key).is_some_and(|definition| definition.is_a("ShapePaint"))
}

fn is_linear_gradient(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key)
        .is_some_and(|definition| definition.is_a("LinearGradient"))
}

fn is_drawable(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key).is_some_and(|definition| definition.is_a("Drawable"))
}

fn drawable_is_hidden(object: &RuntimeObject) -> bool {
    object.uint_property("drawableFlags").unwrap_or(0) & 1 != 0
}

fn path_is_clockwise(object: &RuntimeObject) -> bool {
    object.uint_property("pathFlags").unwrap_or(0) & (1 << 1) == 0
}

fn resolved_image_asset_global(file: &RuntimeFile, object: &RuntimeObject) -> Option<u32> {
    if object.type_name != "Image" {
        return None;
    }
    file.resolved_file_asset_for_referencer(object)
        .map(|asset| asset.id)
}

fn referenced_artboard_global(file: &RuntimeFile, object: &RuntimeObject) -> Option<u32> {
    if !is_exact_nested_artboard_host(object) {
        return None;
    }
    file.resolved_artboard_for_referencer_object(object)
        .map(|artboard| artboard.id)
}

fn is_node(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key).is_some_and(|definition| definition.is_a("Node"))
}

fn is_shape(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key).is_some_and(|definition| definition.is_a("Shape"))
}

fn is_path(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key).is_some_and(|definition| definition.is_a("Path"))
}

fn is_follow_path_constraint(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key)
        .is_some_and(|definition| definition.is_a("FollowPathConstraint"))
}

fn is_text_follow_path_modifier(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key)
        .is_some_and(|definition| definition.name == "TextFollowPathModifier")
}

fn is_bone(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key).is_some_and(|definition| definition.is_a("Bone"))
}

fn is_cpp_skinnable(object: &RuntimeObject) -> bool {
    matches!(object.type_name, "Mesh" | "PointsPath")
}

fn targeted_constraint_requires_target(type_name: &str) -> bool {
    !matches!(
        type_name,
        "RotationConstraint" | "ScaleConstraint" | "TranslationConstraint"
    )
}

fn is_artboard_object(object: Option<&RuntimeObject>) -> bool {
    match object {
        // C++ preserves null object slots in Artboard::objects(); abstract
        // BindableProperty in dependency_test.riv is the first known example.
        None => true,
        Some(object) => definition_by_type_key(object.type_key).is_some_and(|definition| {
            // Component-owned ScriptInputs occupy C++ artboard slots; inputs
            // owned by non-components are removed by parent validation.
            (definition.is_a("Component") && !definition.is_a("ScrollPhysics"))
                || definition.is_a("KeyFrameInterpolator")
                || definition.is_a("UserInput")
        }),
    }
}

fn capabilities(object: &RuntimeObject) -> ComponentCapabilities {
    let Some(definition) = definition_by_type_key(object.type_key) else {
        return ComponentCapabilities::default();
    };

    ComponentCapabilities {
        artboard: definition.name == "Artboard",
        world_transform: definition.is_a("WorldTransformComponent"),
        transform: definition.is_a("TransformComponent"),
        drawable: definition.is_a("Drawable"),
        container: definition.is_a("ContainerComponent"),
    }
}

fn object_name(object: &RuntimeObject) -> Option<String> {
    object.properties.iter().find_map(|property| {
        if property.name == "name" {
            if let FieldValue::String(value) = &property.value {
                return value.as_str().map(ToOwned::to_owned);
            }
        }
        None
    })
}

fn object_parent_id(object: &RuntimeObject) -> Option<u64> {
    object.uint_property("parentId")
}
