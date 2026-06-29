use anyhow::{Context, Result};
use rive_binary::{RuntimeFile, RuntimeImportStatus, RuntimeObject};
use rive_graph::{
    ArtboardGraph, ClippingShapeNode, ComponentNode, DrawableOrderKind, FeatherNode,
    GradientStopNode, ParametricPathNode, PathComposerNode, PathComposerPathNode, PathGeometryNode,
    PathVertexNode, ShapePaintKind, ShapePaintNode, ShapePaintPathKind, ShapePaintStateNode,
    SortedDrawableNode,
};
use rive_schema::{definition_by_name, object_supports_property};
use std::collections::BTreeMap;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

#[derive(Debug, Clone)]
pub struct ArtboardInstance {
    slots: Vec<InstanceSlot>,
    components: Vec<RuntimeComponent>,
    component_by_local: BTreeMap<usize, usize>,
    update_order: Vec<usize>,
    linear_animations: Vec<RuntimeLinearAnimation>,
    state_machines: Vec<RuntimeStateMachine>,
    dirt: ComponentDirt,
    dirt_depth: usize,
    did_change: bool,
}

impl ArtboardInstance {
    pub fn from_graph(file: &RuntimeFile, graph: &ArtboardGraph) -> Result<Self> {
        let mut slots = Vec::new();
        for local_object in &graph.local_objects {
            let object = file.object(local_object.global_id as usize);
            if local_object.type_name.is_some() && object.is_none() {
                anyhow::bail!(
                    "local object {} global id {} is missing",
                    local_object.local_id,
                    local_object.global_id
                );
            }
            slots.push(InstanceSlot {
                local_id: local_object.local_id,
                source_global_id: local_object.global_id,
                type_name: local_object.type_name,
                name: local_object.name.clone(),
                component_index: None,
            });
        }

        let mut component_by_local = BTreeMap::new();
        let mut components = Vec::new();

        for component in &graph.components {
            let object = file.object(component.global_id as usize).with_context(|| {
                format!("component global id {} is missing", component.global_id)
            })?;

            component_by_local.insert(component.local_id, components.len());
            components.push(RuntimeComponent::from_graph_component(component, object));
            let slot = slots
                .get_mut(component.local_id)
                .with_context(|| format!("component local id {} is missing", component.local_id))?;
            slot.component_index = Some(components.len() - 1);
        }

        let mut update_order = graph
            .components
            .iter()
            .filter_map(|component| {
                component
                    .graph_order
                    .map(|order| (order, component.local_id))
            })
            .collect::<Vec<_>>();
        update_order.sort_by_key(|(order, local_id)| (*order, *local_id));
        let update_order = update_order
            .into_iter()
            .map(|(_, local_id)| local_id)
            .collect::<Vec<_>>();
        let linear_animations = build_linear_animations(file, graph, &slots);
        let state_machines = build_state_machines(file, graph, &linear_animations);

        Ok(Self {
            slots,
            components,
            component_by_local,
            update_order,
            linear_animations,
            state_machines,
            dirt: ComponentDirt::COMPONENTS,
            dirt_depth: 0,
            did_change: true,
        })
    }

    pub fn component(&self, local_id: usize) -> Option<&RuntimeComponent> {
        self.component_by_local
            .get(&local_id)
            .map(|index| &self.components[*index])
    }

    pub fn slot(&self, local_id: usize) -> Option<&InstanceSlot> {
        self.slots.get(local_id)
    }

    pub fn slots(&self) -> &[InstanceSlot] {
        &self.slots
    }

    pub fn component_mut(&mut self, local_id: usize) -> Option<&mut RuntimeComponent> {
        let index = *self.component_by_local.get(&local_id)?;
        Some(&mut self.components[index])
    }

    pub fn components(&self) -> &[RuntimeComponent] {
        &self.components
    }

    pub fn update_order(&self) -> &[usize] {
        &self.update_order
    }

    pub fn linear_animation(&self, index: usize) -> Option<&RuntimeLinearAnimation> {
        self.linear_animations.get(index)
    }

    pub fn linear_animations(&self) -> &[RuntimeLinearAnimation] {
        &self.linear_animations
    }

    pub fn draw_commands(&self, graph: &ArtboardGraph) -> Vec<RuntimeDrawCommand> {
        let mut commands = Vec::new();
        let mut pending_clip_operations = Vec::<&SortedDrawableNode>::new();
        let mut empty_clips = 0i32;

        for drawable in &graph.sorted_drawable_order {
            let prev_clips = empty_clips;
            empty_clips += self.runtime_empty_clip_count(drawable, graph);
            if !self.runtime_will_draw(drawable) || empty_clips != prev_clips || empty_clips > 0 {
                continue;
            }

            if drawable.kind == DrawableOrderKind::ClipStartProxy {
                pending_clip_operations.push(drawable);
                continue;
            } else if !pending_clip_operations.is_empty() {
                if drawable.kind == DrawableOrderKind::ClipEndProxy {
                    pending_clip_operations.pop();
                    continue;
                }

                commands.extend(
                    pending_clip_operations.drain(..).map(|pending_clip| {
                        self.runtime_draw_command_for_node(pending_clip, graph)
                    }),
                );
            }

            commands.push(self.runtime_draw_command_for_node(drawable, graph));
        }

        commands
    }

    fn runtime_will_draw(&self, drawable: &SortedDrawableNode) -> bool {
        match drawable.kind {
            DrawableOrderKind::ClipStartProxy | DrawableOrderKind::ClipEndProxy => true,
            DrawableOrderKind::Drawable | DrawableOrderKind::LayoutProxy => {
                if drawable.is_hidden {
                    return false;
                }

                if sorted_drawable_uses_render_opacity(drawable.type_name) {
                    let Some(local_id) = drawable.local_id else {
                        return false;
                    };
                    return self
                        .component(local_id)
                        .is_some_and(|component| component.transform.render_opacity != 0.0);
                }

                true
            }
        }
    }

    fn runtime_empty_clip_count(
        &self,
        drawable: &SortedDrawableNode,
        graph: &ArtboardGraph,
    ) -> i32 {
        let empty_delta = match drawable.kind {
            DrawableOrderKind::ClipStartProxy => 1,
            DrawableOrderKind::ClipEndProxy => -1,
            DrawableOrderKind::Drawable | DrawableOrderKind::LayoutProxy => return 0,
        };
        let Some(clipping_shape_local) = drawable.clipping_shape_local else {
            return 0;
        };
        let Some(clipping_shape) = graph
            .clipping_shapes
            .iter()
            .find(|clipping_shape| clipping_shape.local_id == clipping_shape_local)
        else {
            return 0;
        };

        if clipping_shape.is_visible
            && !self.clipping_shape_has_runtime_clip_path(clipping_shape, &graph.path_composers)
        {
            empty_delta
        } else {
            0
        }
    }

    fn clipping_shape_has_runtime_clip_path(
        &self,
        clipping_shape: &ClippingShapeNode,
        path_composers: &[PathComposerNode],
    ) -> bool {
        clipping_shape.shape_locals.iter().any(|shape_local| {
            path_composers
                .iter()
                .find(|composer| composer.shape_local == *shape_local)
                .is_some_and(|composer| {
                    composer
                        .paths
                        .iter()
                        .any(|path| self.runtime_path_counts_for_clip(path))
                })
        })
    }

    fn runtime_path_counts_for_clip(&self, path: &PathComposerPathNode) -> bool {
        !path.is_hidden
            && !self
                .component(path.local_id)
                .is_some_and(|component| component.is_collapsed())
    }

    fn runtime_draw_command_for_node(
        &self,
        drawable: &SortedDrawableNode,
        graph: &ArtboardGraph,
    ) -> RuntimeDrawCommand {
        RuntimeDrawCommand {
            kind: match drawable.kind {
                DrawableOrderKind::ClipStartProxy => RuntimeDrawCommandKind::ClipStart,
                DrawableOrderKind::ClipEndProxy => RuntimeDrawCommandKind::ClipEnd,
                DrawableOrderKind::Drawable | DrawableOrderKind::LayoutProxy => {
                    RuntimeDrawCommandKind::Draw
                }
            },
            local_id: drawable.local_id,
            clipping_shape_local: drawable.clipping_shape_local,
            needs_save_operation: drawable.needs_save_operation,
            shape_paints: self.runtime_shape_paint_commands(drawable, graph),
        }
    }

    fn runtime_shape_paint_commands(
        &self,
        drawable: &SortedDrawableNode,
        graph: &ArtboardGraph,
    ) -> Vec<RuntimeShapePaintCommand> {
        if drawable.kind != DrawableOrderKind::Drawable || drawable.type_name != "Shape" {
            return Vec::new();
        }
        let Some(shape_local) = drawable.local_id else {
            return Vec::new();
        };
        let Some(container) = graph
            .shape_paint_containers
            .iter()
            .find(|container| container.local_id == shape_local)
        else {
            return Vec::new();
        };
        let needs_save_operation = drawable.needs_save_operation || container.paints.len() > 1;
        let render_opacity = self
            .component(shape_local)
            .map(|component| component.transform.render_opacity)
            .unwrap_or(1.0);

        container
            .paints
            .iter()
            .filter_map(|paint| {
                runtime_shape_paint_command(
                    paint,
                    container.blend_mode_value,
                    needs_save_operation,
                    render_opacity,
                    self.runtime_shape_paint_path_commands(shape_local, paint, graph),
                )
            })
            .collect()
    }

    fn runtime_shape_paint_path_commands(
        &self,
        shape_local: usize,
        paint: &ShapePaintNode,
        graph: &ArtboardGraph,
    ) -> Vec<RuntimePathCommand> {
        let Some(path_kind) = paint.path_kind else {
            return Vec::new();
        };
        let Some(composer) = graph
            .path_composers
            .iter()
            .find(|composer| composer.shape_local == shape_local)
        else {
            return Vec::new();
        };

        let shape_world = self
            .component(shape_local)
            .map(|component| component.transform.world_transform)
            .unwrap_or(Mat2D::IDENTITY);
        let inverse_shape_world = shape_world.invert_or_identity();
        let mut commands = Vec::new();

        for path_ref in &composer.paths {
            if !self.runtime_path_counts_for_clip(path_ref) {
                continue;
            }
            let Some(path) = graph
                .paths
                .iter()
                .find(|path| path.local_id == path_ref.local_id)
            else {
                continue;
            };
            let Some(path_world) = self
                .component(path.local_id)
                .map(|component| component.transform.world_transform)
            else {
                continue;
            };
            let transform = match path_kind {
                ShapePaintPathKind::World => path_world,
                ShapePaintPathKind::Local | ShapePaintPathKind::LocalClockwise => {
                    inverse_shape_world.multiply(path_world)
                }
            };

            commands.extend(path_commands(path, path_kind, transform));
        }

        commands
    }

    pub fn state_machine(&self, index: usize) -> Option<&RuntimeStateMachine> {
        self.state_machines.get(index)
    }

    pub fn state_machines(&self) -> &[RuntimeStateMachine] {
        &self.state_machines
    }

    pub fn apply_linear_animation(&mut self, index: usize, seconds: f32, mix: f32) -> bool {
        let Some(animation) = self.linear_animations.get(index).cloned() else {
            return false;
        };
        animation.apply(self, seconds, mix)
    }

    pub fn linear_animation_instance(&self, index: usize) -> Option<LinearAnimationInstance> {
        self.linear_animation_instance_with_speed(index, 1.0)
    }

    pub fn linear_animation_instance_with_speed(
        &self,
        index: usize,
        speed_multiplier: f32,
    ) -> Option<LinearAnimationInstance> {
        let animation = self.linear_animation(index)?;
        Some(LinearAnimationInstance::new(
            index,
            animation,
            speed_multiplier,
        ))
    }

    pub fn advance_linear_animation_instance(
        &self,
        instance: &mut LinearAnimationInstance,
        elapsed_seconds: f32,
    ) -> bool {
        let Some(animation) = self.linear_animation(instance.animation_index) else {
            return false;
        };
        instance.advance(animation, elapsed_seconds)
    }

    pub fn apply_linear_animation_instance(
        &mut self,
        instance: &LinearAnimationInstance,
        mix: f32,
    ) -> bool {
        self.apply_linear_animation(instance.animation_index, instance.time, mix)
    }

    pub fn linear_animation_instance_keep_going(&self, instance: &LinearAnimationInstance) -> bool {
        let Some(animation) = self.linear_animation(instance.animation_index) else {
            return false;
        };
        instance.keep_going(animation)
    }

    pub fn state_machine_instance(&self, index: usize) -> Option<StateMachineInstance> {
        let state_machine = self.state_machine(index)?;
        Some(StateMachineInstance::new(index, state_machine, self))
    }

    pub fn advance_state_machine_instance(
        &mut self,
        instance: &mut StateMachineInstance,
        elapsed_seconds: f32,
    ) -> bool {
        let Some(state_machine) = self.state_machine(instance.state_machine_index).cloned() else {
            return false;
        };
        instance.advance(self, &state_machine, elapsed_seconds)
    }

    pub fn set_transform_property(
        &mut self,
        local_id: usize,
        property: TransformProperty,
        value: f32,
    ) -> bool {
        let Some(index) = self.component_by_local.get(&local_id).copied() else {
            return false;
        };
        if !self.components[index].capabilities.transform {
            return false;
        }

        if !self.components[index]
            .transform
            .set_property(property, value)
        {
            return false;
        }

        match property {
            TransformProperty::Opacity => {
                self.add_dirt(local_id, ComponentDirt::RENDER_OPACITY, true);
            }
            TransformProperty::X
            | TransformProperty::Y
            | TransformProperty::Rotation
            | TransformProperty::ScaleX
            | TransformProperty::ScaleY => {
                self.add_dirt(local_id, ComponentDirt::TRANSFORM, false);
                self.add_dirt(local_id, ComponentDirt::WORLD_TRANSFORM, true);
            }
        }
        true
    }

    fn transform_property(&self, local_id: usize, property: TransformProperty) -> Option<f32> {
        self.component(local_id)
            .filter(|component| component.capabilities.transform)
            .map(|component| component.transform.property(property))
    }

    pub fn has_dirt(&self, dirt: ComponentDirt) -> bool {
        self.dirt.contains(dirt)
    }

    pub fn did_change(&self) -> bool {
        self.did_change
    }

    pub fn clear_component_dirt(&mut self, local_id: usize) {
        if let Some(component) = self.component_mut(local_id) {
            component.dirt = ComponentDirt::NONE;
        }
    }

    pub fn add_dirt(&mut self, local_id: usize, dirt: ComponentDirt, recurse: bool) -> bool {
        if dirt.is_empty() {
            return false;
        }

        let Some(index) = self.component_by_local.get(&local_id).copied() else {
            return false;
        };

        if self.components[index].dirt.contains(dirt) {
            return false;
        }

        self.components[index].dirt |= dirt;
        self.on_component_dirty(local_id);

        if recurse {
            let dependents = self.components[index].dependent_locals.clone();
            for dependent in dependents {
                self.add_dirt(dependent, dirt, true);
            }
        }

        true
    }

    pub fn collapse_component(&mut self, local_id: usize, collapsed: bool) -> bool {
        let Some(index) = self.component_by_local.get(&local_id).copied() else {
            return false;
        };

        if self.components[index].is_collapsed() == collapsed {
            return false;
        }

        if collapsed {
            self.components[index].dirt |= ComponentDirt::COLLAPSED;
        } else {
            self.components[index].dirt &= !ComponentDirt::COLLAPSED;
        }
        self.on_component_dirty(local_id);
        true
    }

    pub fn update_components(&mut self) -> UpdateComponentsReport {
        self.update_components_with_hook(|_, _, _| {})
    }

    pub fn update_components_with_hook<F>(&mut self, mut hook: F) -> UpdateComponentsReport
    where
        F: FnMut(&mut Self, usize, ComponentDirt),
    {
        let mut report = UpdateComponentsReport::default();
        if !self.has_dirt(ComponentDirt::COMPONENTS) {
            return report;
        }

        report.did_update = true;
        let max_steps = 100;
        let update_order = self.update_order.clone();

        while self.has_dirt(ComponentDirt::COMPONENTS) && report.steps < max_steps {
            self.dirt &= !ComponentDirt::COMPONENTS;

            for (order_index, local_id) in update_order.iter().copied().enumerate() {
                self.dirt_depth = order_index;
                let Some(component_index) = self.component_by_local.get(&local_id).copied() else {
                    continue;
                };
                let dirt = self.components[component_index].dirt;
                if dirt.is_empty() || dirt.contains(ComponentDirt::COLLAPSED) {
                    continue;
                }

                self.components[component_index].dirt = ComponentDirt::NONE;
                self.update_component(component_index, dirt);
                report.updated_locals.push(local_id);
                hook(self, local_id, dirt);

                if self.dirt_depth < order_index {
                    break;
                }
            }

            report.steps += 1;
        }

        report.max_steps_reached = self.has_dirt(ComponentDirt::COMPONENTS);
        report
    }

    fn on_component_dirty(&mut self, local_id: usize) {
        self.did_change = true;
        self.dirt |= ComponentDirt::COMPONENTS;

        let Some(component) = self.component(local_id) else {
            return;
        };
        if component.graph_order < self.dirt_depth {
            self.dirt_depth = component.graph_order;
        }
    }

    fn update_component(&mut self, component_index: usize, dirt: ComponentDirt) {
        if dirt.contains(ComponentDirt::TRANSFORM) {
            self.components[component_index].update_transform();
        }
        if dirt.contains(ComponentDirt::WORLD_TRANSFORM) {
            let parent_world = self.components[component_index]
                .parent_local
                .and_then(|parent_local| self.component(parent_local))
                .filter(|parent| parent.capabilities.world_transform)
                .map(|parent| parent.transform.world_transform);
            self.components[component_index].update_world_transform(parent_world);
        }
        if dirt.contains(ComponentDirt::RENDER_OPACITY) {
            let parent_opacity = self.components[component_index]
                .parent_local
                .and_then(|parent_local| self.component(parent_local))
                .filter(|parent| parent.capabilities.world_transform)
                .map(|parent| parent.transform.render_opacity)
                .unwrap_or(1.0);
            self.components[component_index].update_render_opacity(parent_opacity);
        }
    }
}

#[derive(Debug, Clone)]
pub struct InstanceSlot {
    pub local_id: usize,
    pub source_global_id: u32,
    pub type_name: Option<&'static str>,
    pub name: Option<String>,
    pub component_index: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeDrawCommandKind {
    Draw,
    ClipStart,
    ClipEnd,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeDrawCommand {
    pub kind: RuntimeDrawCommandKind,
    pub local_id: Option<usize>,
    pub clipping_shape_local: Option<usize>,
    pub needs_save_operation: bool,
    pub shape_paints: Vec<RuntimeShapePaintCommand>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeShapePaintKind {
    Fill,
    Stroke,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeShapePaintPathKind {
    Local,
    LocalClockwise,
    World,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeShapePaintCommand {
    pub paint_local: usize,
    pub mutator_local: Option<usize>,
    pub paint_type: RuntimeShapePaintKind,
    pub path_kind: RuntimeShapePaintPathKind,
    pub blend_mode_value: u32,
    pub render_blend_mode_value: u32,
    pub paint_state: Option<RuntimeShapePaintState>,
    pub feather_state: Option<RuntimeFeatherState>,
    pub path_commands: Vec<RuntimePathCommand>,
    pub needs_save_operation: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeShapePaintState {
    SolidColor {
        color: u32,
        render_color: u32,
    },
    LinearGradient {
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
        opacity: f32,
        render_opacity: f32,
        stops: Vec<RuntimeGradientStop>,
    },
    RadialGradient {
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
        opacity: f32,
        render_opacity: f32,
        stops: Vec<RuntimeGradientStop>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RuntimeGradientStop {
    pub color: u32,
    pub render_color: u32,
    pub position: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RuntimeFeatherState {
    pub feather_local: usize,
    pub space_value: u32,
    pub strength: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub inner: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RuntimePathCommand {
    Move {
        x: f32,
        y: f32,
    },
    Line {
        x: f32,
        y: f32,
    },
    Cubic {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
    },
    Close,
}

fn runtime_shape_paint_command(
    paint: &ShapePaintNode,
    container_blend_mode_value: u32,
    needs_save_operation: bool,
    render_opacity: f32,
    path_commands: Vec<RuntimePathCommand>,
) -> Option<RuntimeShapePaintCommand> {
    if !paint.is_visible {
        return None;
    }
    Some(RuntimeShapePaintCommand {
        paint_local: paint.local_id,
        mutator_local: paint.mutator_local,
        paint_type: runtime_shape_paint_kind(paint.paint_type),
        path_kind: runtime_shape_paint_path_kind(paint.path_kind?)?,
        blend_mode_value: paint.blend_mode_value,
        render_blend_mode_value: runtime_shape_paint_blend_mode_value(
            paint.blend_mode_value,
            container_blend_mode_value,
        ),
        paint_state: runtime_shape_paint_state(paint.paint_state.clone(), render_opacity),
        feather_state: runtime_feather_state(paint.feather.as_ref()),
        path_commands,
        needs_save_operation,
    })
}

fn runtime_shape_paint_blend_mode_value(
    paint_blend_mode_value: u32,
    container_blend_mode_value: u32,
) -> u32 {
    if paint_blend_mode_value == 127 {
        container_blend_mode_value
    } else {
        paint_blend_mode_value
    }
}

fn runtime_shape_paint_kind(kind: ShapePaintKind) -> RuntimeShapePaintKind {
    match kind {
        ShapePaintKind::Fill => RuntimeShapePaintKind::Fill,
        ShapePaintKind::Stroke => RuntimeShapePaintKind::Stroke,
        ShapePaintKind::Unknown => RuntimeShapePaintKind::Unknown,
    }
}

fn runtime_shape_paint_path_kind(kind: ShapePaintPathKind) -> Option<RuntimeShapePaintPathKind> {
    match kind {
        ShapePaintPathKind::Local => Some(RuntimeShapePaintPathKind::Local),
        ShapePaintPathKind::LocalClockwise => Some(RuntimeShapePaintPathKind::LocalClockwise),
        ShapePaintPathKind::World => Some(RuntimeShapePaintPathKind::World),
    }
}

fn runtime_shape_paint_state(
    state: Option<ShapePaintStateNode>,
    render_opacity: f32,
) -> Option<RuntimeShapePaintState> {
    match state? {
        ShapePaintStateNode::SolidColor { color } => Some(RuntimeShapePaintState::SolidColor {
            color,
            render_color: color_modulate_opacity(color, render_opacity),
        }),
        ShapePaintStateNode::LinearGradient {
            start_x,
            start_y,
            end_x,
            end_y,
            opacity,
            stops,
        } => Some(RuntimeShapePaintState::LinearGradient {
            start_x,
            start_y,
            end_x,
            end_y,
            opacity,
            render_opacity,
            stops: runtime_gradient_stops(stops, opacity * render_opacity),
        }),
        ShapePaintStateNode::RadialGradient {
            start_x,
            start_y,
            end_x,
            end_y,
            opacity,
            stops,
        } => Some(RuntimeShapePaintState::RadialGradient {
            start_x,
            start_y,
            end_x,
            end_y,
            opacity,
            render_opacity,
            stops: runtime_gradient_stops(stops, opacity * render_opacity),
        }),
    }
}

fn runtime_gradient_stops(
    mut stops: Vec<GradientStopNode>,
    opacity: f32,
) -> Vec<RuntimeGradientStop> {
    stops.sort_by(|left, right| left.position.total_cmp(&right.position));
    stops
        .into_iter()
        .map(|stop| RuntimeGradientStop {
            color: stop.color,
            render_color: color_modulate_opacity(stop.color, opacity),
            position: stop.position.clamp(0.0, 1.0),
        })
        .collect()
}

fn runtime_feather_state(feather: Option<&FeatherNode>) -> Option<RuntimeFeatherState> {
    let feather = feather?;
    Some(RuntimeFeatherState {
        feather_local: feather.local_id,
        space_value: feather.space_value,
        strength: feather.strength,
        offset_x: feather.offset_x,
        offset_y: feather.offset_y,
        inner: feather.inner,
    })
}

fn color_modulate_opacity(color: u32, opacity: f32) -> u32 {
    let alpha = ((color >> 24) & 0xff) as f32 / 255.0;
    let alpha = opacity_to_alpha(alpha * opacity);
    (color & 0x00ff_ffff) | (u32::from(alpha) << 24)
}

fn opacity_to_alpha(opacity: f32) -> u8 {
    (255.0 * opacity.clamp(0.0, 1.0)).round() as u8
}

fn path_commands(
    path: &PathGeometryNode,
    path_kind: ShapePaintPathKind,
    transform: Mat2D,
) -> Vec<RuntimePathCommand> {
    match path.type_name {
        "Ellipse" => ellipse_path_commands(path, path_kind, transform),
        "PointsPath" => points_path_commands(path, path_kind, transform),
        "Polygon" => polygon_path_commands(path, path_kind, transform),
        "Rectangle" => rectangle_path_commands(path, path_kind, transform),
        "Star" => star_path_commands(path, path_kind, transform),
        "Triangle" => triangle_path_commands(path, path_kind, transform),
        _ => Vec::new(),
    }
}

fn points_path_commands(
    path: &PathGeometryNode,
    path_kind: ShapePaintPathKind,
    transform: Mat2D,
) -> Vec<RuntimePathCommand> {
    if path.type_name != "PointsPath" || path.vertices.len() < 2 {
        return Vec::new();
    }
    if path
        .vertices
        .iter()
        .any(|vertex| !is_supported_point_path_vertex(vertex))
    {
        return Vec::new();
    }
    if path_kind == ShapePaintPathKind::LocalClockwise
        && path_needs_clockwise_reversal(path, transform)
    {
        return Vec::new();
    }

    let mut commands = Vec::new();
    let first = &path.vertices[0];

    let (start, start_in, mut out, start_is_cubic, mut prev_is_cubic) =
        if let Some((in_point, out_point)) = cubic_vertex_points(first) {
            let start = vertex_translation(first);
            push_move(&mut commands, transform, start);
            (start, in_point, out_point, true, true)
        } else if first.radius != 0.0 {
            let prev = path
                .vertices
                .last()
                .expect("path has at least two vertices");
            let next = &path.vertices[1];
            let (start, mut out_point, mut in_point, out_after) =
                rounded_straight_vertex_points(first, prev, next);
            if first.radius < 0.0 {
                rotate_rounded_points(
                    out_after,
                    start,
                    vertex_translation(first),
                    &mut out_point,
                    &mut in_point,
                );
            }
            push_move(&mut commands, transform, start);
            push_cubic(&mut commands, transform, out_point, in_point, out_after);
            (start, start, out_after, false, false)
        } else {
            let start = vertex_translation(first);
            push_move(&mut commands, transform, start);
            (start, start, start, false, false)
        };

    for (index, vertex) in path.vertices.iter().enumerate().skip(1) {
        if let Some((in_point, out_point)) = cubic_vertex_points(vertex) {
            let translation = vertex_translation(vertex);
            push_cubic(&mut commands, transform, out, in_point, translation);
            prev_is_cubic = true;
            out = out_point;
        } else {
            let position = vertex_translation(vertex);
            if vertex.radius != 0.0 {
                let prev = &path.vertices[index - 1];
                let next = &path.vertices[(index + 1) % path.vertices.len()];
                let (translation, mut out_point, mut in_point, out_after) =
                    rounded_straight_vertex_points(vertex, prev, next);
                if prev_is_cubic {
                    push_cubic(&mut commands, transform, out, translation, translation);
                } else {
                    push_line(&mut commands, transform, translation);
                }
                if vertex.radius < 0.0 {
                    rotate_rounded_points(
                        out_after,
                        translation,
                        position,
                        &mut out_point,
                        &mut in_point,
                    );
                }
                push_cubic(&mut commands, transform, out_point, in_point, out_after);
                prev_is_cubic = false;
                out = out_after;
            } else if prev_is_cubic {
                push_cubic(&mut commands, transform, out, position, position);
                prev_is_cubic = false;
                out = position;
            } else {
                push_line(&mut commands, transform, position);
                out = position;
            }
        }
    }

    if path.is_closed {
        if prev_is_cubic || start_is_cubic {
            push_cubic(&mut commands, transform, out, start_in, start);
        } else {
            push_line(&mut commands, transform, start);
        }
        commands.push(RuntimePathCommand::Close);
    }

    commands
}

fn rectangle_path_commands(
    path: &PathGeometryNode,
    path_kind: ShapePaintPathKind,
    transform: Mat2D,
) -> Vec<RuntimePathCommand> {
    let Some(ParametricPathNode::Rectangle {
        width,
        height,
        origin_x,
        origin_y,
        link_corner_radius,
        corner_radius_tl,
        corner_radius_tr,
        corner_radius_bl,
        corner_radius_br,
    }) = path.parametric.as_ref()
    else {
        return Vec::new();
    };

    let width = *width;
    let height = *height;
    let origin_x = *origin_x;
    let origin_y = *origin_y;
    let link_corner_radius = *link_corner_radius;
    let left = -origin_x * width;
    let top = -origin_y * height;
    let right = left + width;
    let bottom = top + height;
    let top_left_radius = *corner_radius_tl;
    let top_right_radius = if link_corner_radius {
        top_left_radius
    } else {
        *corner_radius_tr
    };
    let bottom_right_radius = if link_corner_radius {
        top_left_radius
    } else {
        *corner_radius_br
    };
    let bottom_left_radius = if link_corner_radius {
        top_left_radius
    } else {
        *corner_radius_bl
    };

    let virtual_path = PathGeometryNode {
        local_id: path.local_id,
        global_id: path.global_id,
        type_name: "PointsPath",
        is_closed: true,
        is_hole: path.is_hole,
        is_clockwise: true,
        parametric: None,
        vertices: vec![
            virtual_straight_vertex(left, top, top_left_radius),
            virtual_straight_vertex(right, top, top_right_radius),
            virtual_straight_vertex(right, bottom, bottom_right_radius),
            virtual_straight_vertex(left, bottom, bottom_left_radius),
        ],
    };

    points_path_commands(&virtual_path, path_kind, transform)
}

const CIRCLE_CONSTANT: f32 = 0.552_284_8;

fn ellipse_path_commands(
    path: &PathGeometryNode,
    path_kind: ShapePaintPathKind,
    transform: Mat2D,
) -> Vec<RuntimePathCommand> {
    let Some(ParametricPathNode::Ellipse {
        width,
        height,
        origin_x,
        origin_y,
    }) = path.parametric.as_ref()
    else {
        return Vec::new();
    };
    if path_kind == ShapePaintPathKind::LocalClockwise
        && path_needs_clockwise_reversal(path, transform)
    {
        return Vec::new();
    }

    let radius_x = *width / 2.0;
    let radius_y = *height / 2.0;
    let ox = -*origin_x * *width + radius_x;
    let oy = -*origin_y * *height + radius_y;
    let top = (ox, oy - radius_y);
    let right = (ox + radius_x, oy);
    let bottom = (ox, oy + radius_y);
    let left = (ox - radius_x, oy);

    let mut commands = Vec::new();
    push_move(&mut commands, transform, top);
    push_cubic(
        &mut commands,
        transform,
        (ox + radius_x * CIRCLE_CONSTANT, oy - radius_y),
        (ox + radius_x, oy - CIRCLE_CONSTANT * radius_y),
        right,
    );
    push_cubic(
        &mut commands,
        transform,
        (ox + radius_x, oy + CIRCLE_CONSTANT * radius_y),
        (ox + radius_x * CIRCLE_CONSTANT, oy + radius_y),
        bottom,
    );
    push_cubic(
        &mut commands,
        transform,
        (ox - radius_x * CIRCLE_CONSTANT, oy + radius_y),
        (ox - radius_x, oy + radius_y * CIRCLE_CONSTANT),
        left,
    );
    push_cubic(
        &mut commands,
        transform,
        (ox - radius_x, oy - radius_y * CIRCLE_CONSTANT),
        (ox - radius_x * CIRCLE_CONSTANT, oy - radius_y),
        top,
    );
    commands.push(RuntimePathCommand::Close);
    commands
}

fn polygon_path_commands(
    path: &PathGeometryNode,
    path_kind: ShapePaintPathKind,
    transform: Mat2D,
) -> Vec<RuntimePathCommand> {
    let Some(ParametricPathNode::Polygon {
        width,
        height,
        origin_x,
        origin_y,
        points,
        corner_radius,
    }) = path.parametric.as_ref()
    else {
        return Vec::new();
    };

    let Ok(count) = usize::try_from(*points) else {
        return Vec::new();
    };
    if count < 2 {
        return Vec::new();
    }

    let half_width = *width / 2.0;
    let half_height = *height / 2.0;
    let ox = -*origin_x * *width + half_width;
    let oy = -*origin_y * *height + half_height;
    let mut angle = -std::f32::consts::FRAC_PI_2;
    let inc = 2.0 * std::f32::consts::PI / *points as f32;
    let mut vertices = Vec::with_capacity(count);
    for _ in 0..count {
        vertices.push(virtual_straight_vertex(
            ox + angle.cos() * half_width,
            oy + angle.sin() * half_height,
            *corner_radius,
        ));
        angle += inc;
    }

    closed_straight_vertices_path_commands(path, path_kind, transform, vertices)
}

fn star_path_commands(
    path: &PathGeometryNode,
    path_kind: ShapePaintPathKind,
    transform: Mat2D,
) -> Vec<RuntimePathCommand> {
    let Some(ParametricPathNode::Star {
        width,
        height,
        origin_x,
        origin_y,
        points,
        corner_radius,
        inner_radius,
    }) = path.parametric.as_ref()
    else {
        return Vec::new();
    };

    let Ok(point_count) = usize::try_from(*points) else {
        return Vec::new();
    };
    let Some(count) = point_count.checked_mul(2) else {
        return Vec::new();
    };
    if count < 2 {
        return Vec::new();
    }

    let half_width = *width / 2.0;
    let half_height = *height / 2.0;
    let inner_half_width = *width * *inner_radius / 2.0;
    let inner_half_height = *height * *inner_radius / 2.0;
    let ox = -*origin_x * *width + half_width;
    let oy = -*origin_y * *height + half_height;
    let mut angle = -std::f32::consts::FRAC_PI_2;
    let inc = 2.0 * std::f32::consts::PI / count as f32;
    let mut vertices = Vec::with_capacity(count);
    for _ in 0..point_count {
        vertices.push(virtual_straight_vertex(
            ox + angle.cos() * half_width,
            oy + angle.sin() * half_height,
            *corner_radius,
        ));
        angle += inc;
        vertices.push(virtual_straight_vertex(
            ox + angle.cos() * inner_half_width,
            oy + angle.sin() * inner_half_height,
            *corner_radius,
        ));
        angle += inc;
    }

    closed_straight_vertices_path_commands(path, path_kind, transform, vertices)
}

fn closed_straight_vertices_path_commands(
    path: &PathGeometryNode,
    path_kind: ShapePaintPathKind,
    transform: Mat2D,
    vertices: Vec<PathVertexNode>,
) -> Vec<RuntimePathCommand> {
    let virtual_path = PathGeometryNode {
        local_id: path.local_id,
        global_id: path.global_id,
        type_name: "PointsPath",
        is_closed: true,
        is_hole: path.is_hole,
        is_clockwise: true,
        parametric: None,
        vertices,
    };

    points_path_commands(&virtual_path, path_kind, transform)
}

fn triangle_path_commands(
    path: &PathGeometryNode,
    path_kind: ShapePaintPathKind,
    transform: Mat2D,
) -> Vec<RuntimePathCommand> {
    let Some(ParametricPathNode::Triangle {
        width,
        height,
        origin_x,
        origin_y,
    }) = path.parametric.as_ref()
    else {
        return Vec::new();
    };

    let ox = -*origin_x * *width;
    let oy = -*origin_y * *height;
    let vertices = vec![
        virtual_straight_vertex(ox + *width / 2.0, oy, 0.0),
        virtual_straight_vertex(ox + *width, oy + *height, 0.0),
        virtual_straight_vertex(ox, oy + *height, 0.0),
    ];

    closed_straight_vertices_path_commands(path, path_kind, transform, vertices)
}

fn virtual_straight_vertex(x: f32, y: f32, radius: f32) -> PathVertexNode {
    PathVertexNode {
        local_id: 0,
        global_id: 0,
        type_name: "StraightVertex",
        x,
        y,
        radius,
        rotation: 0.0,
        distance: 0.0,
        in_rotation: 0.0,
        in_distance: 0.0,
        out_rotation: 0.0,
        out_distance: 0.0,
        weight_local: None,
        weight_global: None,
        weight_type_name: None,
        weight_values: None,
        weight_indices: None,
        weight_in_values: None,
        weight_in_indices: None,
        weight_out_values: None,
        weight_out_indices: None,
    }
}

fn is_supported_point_path_vertex(vertex: &PathVertexNode) -> bool {
    if vertex.weight_local.is_some() {
        return false;
    }
    match vertex.type_name {
        "StraightVertex" => true,
        "CubicDetachedVertex" | "CubicAsymmetricVertex" | "CubicMirroredVertex" => true,
        _ => false,
    }
}

fn vertex_translation(vertex: &PathVertexNode) -> (f32, f32) {
    (vertex.x, vertex.y)
}

fn cubic_vertex_points(vertex: &PathVertexNode) -> Option<((f32, f32), (f32, f32))> {
    let point = vertex_translation(vertex);
    match vertex.type_name {
        "CubicDetachedVertex" => Some((
            add_point(point, angle_vector(vertex.in_rotation, vertex.in_distance)),
            add_point(
                point,
                angle_vector(vertex.out_rotation, vertex.out_distance),
            ),
        )),
        "CubicAsymmetricVertex" => Some((
            subtract_point(point, angle_vector(vertex.rotation, vertex.in_distance)),
            add_point(point, angle_vector(vertex.rotation, vertex.out_distance)),
        )),
        "CubicMirroredVertex" => Some((
            subtract_point(point, angle_vector(vertex.rotation, vertex.distance)),
            add_point(point, angle_vector(vertex.rotation, vertex.distance)),
        )),
        _ => None,
    }
}

fn angle_vector(rotation: f32, distance: f32) -> (f32, f32) {
    (rotation.cos() * distance, rotation.sin() * distance)
}

fn add_point(point: (f32, f32), vector: (f32, f32)) -> (f32, f32) {
    (point.0 + vector.0, point.1 + vector.1)
}

fn subtract_point(point: (f32, f32), vector: (f32, f32)) -> (f32, f32) {
    (point.0 - vector.0, point.1 - vector.1)
}

fn rounded_straight_vertex_points(
    vertex: &PathVertexNode,
    prev: &PathVertexNode,
    next: &PathVertexNode,
) -> ((f32, f32), (f32, f32), (f32, f32), (f32, f32)) {
    let position = vertex_translation(vertex);
    let (to_prev, to_prev_length) =
        normalize_vector(subtract_point(vertex_render_out_point(prev), position));
    let (to_next, to_next_length) =
        normalize_vector(subtract_point(vertex_render_in_point(next), position));
    let render_radius = (to_prev_length / 2.0)
        .min(to_next_length / 2.0)
        .min(vertex.radius.abs());
    let ideal_distance = compute_ideal_control_point_distance(to_prev, to_next, render_radius);
    let translation = scale_and_add_point(position, to_prev, render_radius);
    let out_point = scale_and_add_point(position, to_prev, render_radius - ideal_distance);
    let in_point = scale_and_add_point(position, to_next, render_radius - ideal_distance);
    let out_after = scale_and_add_point(position, to_next, render_radius);

    (translation, out_point, in_point, out_after)
}

fn vertex_render_in_point(vertex: &PathVertexNode) -> (f32, f32) {
    cubic_vertex_points(vertex)
        .map(|(in_point, _)| in_point)
        .unwrap_or_else(|| vertex_translation(vertex))
}

fn vertex_render_out_point(vertex: &PathVertexNode) -> (f32, f32) {
    cubic_vertex_points(vertex)
        .map(|(_, out_point)| out_point)
        .unwrap_or_else(|| vertex_translation(vertex))
}

fn normalize_vector(vector: (f32, f32)) -> ((f32, f32), f32) {
    let length = (vector.0 * vector.0 + vector.1 * vector.1).sqrt();
    if length > 0.0 {
        ((vector.0 / length, vector.1 / length), length)
    } else {
        (vector, length)
    }
}

fn scale_and_add_point(point: (f32, f32), vector: (f32, f32), scale: f32) -> (f32, f32) {
    (point.0 + vector.0 * scale, point.1 + vector.1 * scale)
}

fn compute_ideal_control_point_distance(
    to_prev: (f32, f32),
    to_next: (f32, f32),
    radius: f32,
) -> f32 {
    let angle = cross_point(to_prev, to_next)
        .atan2(dot_point(to_prev, to_next))
        .abs();
    let natural_rounding = (4.0 / 3.0)
        * (std::f32::consts::PI / (2.0 * ((2.0 * std::f32::consts::PI) / angle))).tan()
        * radius
        * if angle < std::f32::consts::FRAC_PI_2 {
            1.0 + angle.cos()
        } else {
            2.0 - angle.sin()
        };

    radius.min(natural_rounding)
}

fn rotate_rounded_points(
    next_point: (f32, f32),
    prev_point: (f32, f32),
    point: (f32, f32),
    out_point: &mut (f32, f32),
    in_point: &mut (f32, f32),
) {
    let v1 = subtract_point(prev_point, next_point);
    let v2 = subtract_point(point, next_point);
    let angle = cross_point(v1, v2).atan2(dot_point(v1, v2));
    *out_point = rotate_point_around(*out_point, prev_point, angle * 2.0);
    *in_point = rotate_point_around(*in_point, next_point, -angle * 2.0);
}

fn rotate_point_around(point: (f32, f32), origin: (f32, f32), angle: f32) -> (f32, f32) {
    let sin = angle.sin();
    let cos = angle.cos();
    let x = point.0 - origin.0;
    let y = point.1 - origin.1;

    (x * cos - y * sin + origin.0, x * sin + y * cos + origin.1)
}

fn dot_point(left: (f32, f32), right: (f32, f32)) -> f32 {
    left.0 * right.0 + left.1 * right.1
}

fn cross_point(left: (f32, f32), right: (f32, f32)) -> f32 {
    left.0 * right.1 - left.1 * right.0
}

fn push_move(commands: &mut Vec<RuntimePathCommand>, transform: Mat2D, point: (f32, f32)) {
    let (x, y) = transform.transform_point(point.0, point.1);
    commands.push(RuntimePathCommand::Move { x, y });
}

fn push_line(commands: &mut Vec<RuntimePathCommand>, transform: Mat2D, point: (f32, f32)) {
    let (x, y) = transform.transform_point(point.0, point.1);
    commands.push(RuntimePathCommand::Line { x, y });
}

fn push_cubic(
    commands: &mut Vec<RuntimePathCommand>,
    transform: Mat2D,
    point1: (f32, f32),
    point2: (f32, f32),
    point3: (f32, f32),
) {
    let (x1, y1) = transform.transform_point(point1.0, point1.1);
    let (x2, y2) = transform.transform_point(point2.0, point2.1);
    let (x3, y3) = transform.transform_point(point3.0, point3.1);
    commands.push(RuntimePathCommand::Cubic {
        x1,
        y1,
        x2,
        y2,
        x3,
        y3,
    });
}

fn path_needs_clockwise_reversal(path: &PathGeometryNode, transform: Mat2D) -> bool {
    let designed_clockwise = if path.is_clockwise { 1.0 } else { -1.0 };
    let is_not_clockwise = transform.determinant() * designed_clockwise < 0.0;
    is_not_clockwise != path.is_hole
}

fn sorted_drawable_uses_render_opacity(type_name: &str) -> bool {
    definition_by_name(type_name).is_some_and(|definition| {
        definition.is_a("Shape") || matches!(definition.name, "Image" | "TextInputDrawable")
    })
}

#[derive(Debug, Clone)]
pub struct RuntimeLinearAnimation {
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
    pub keyed_objects: Vec<RuntimeKeyedObject>,
}

impl RuntimeLinearAnimation {
    fn apply(&self, instance: &mut ArtboardInstance, seconds: f32, mix: f32) -> bool {
        let seconds = if self.quantize && self.fps != 0 {
            let fps = self.fps as f32;
            (seconds * fps).floor() / fps
        } else {
            seconds
        };

        let mut changed = false;
        for keyed_object in &self.keyed_objects {
            for keyed_property in &keyed_object.keyed_properties {
                let Some(property) = keyed_property.transform_property else {
                    continue;
                };
                let Some(current) =
                    instance.transform_property(keyed_object.target_local_id, property)
                else {
                    continue;
                };
                let Some(value) = keyed_property.value_at(seconds, self.fps, current, mix) else {
                    continue;
                };
                changed |=
                    instance.set_transform_property(keyed_object.target_local_id, property, value);
            }
        }
        changed
    }

    fn start_seconds(&self) -> f32 {
        self.frame_to_seconds(self.start_frame())
    }

    fn end_seconds(&self) -> f32 {
        self.frame_to_seconds(self.end_frame())
    }

    fn duration_seconds(&self) -> f32 {
        self.frame_to_seconds(self.duration as f32)
    }

    fn start_time_with_speed(&self, speed_multiplier: f32) -> f32 {
        if self.speed * speed_multiplier >= 0.0 {
            self.start_seconds()
        } else {
            self.end_seconds()
        }
    }

    fn fps_as_f32(&self) -> f32 {
        self.fps as f32
    }

    fn start_frame(&self) -> f32 {
        if self.enable_work_area {
            self.work_start as f32
        } else {
            0.0
        }
    }

    fn end_frame(&self) -> f32 {
        if self.enable_work_area {
            self.work_end as f32
        } else {
            self.duration as f32
        }
    }

    fn frame_to_seconds(&self, frame: f32) -> f32 {
        if self.fps == 0 {
            return 0.0;
        }
        frame / self.fps_as_f32()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnimationLoop {
    OneShot,
    Loop,
    PingPong,
}

impl AnimationLoop {
    fn from_loop_value(value: u64) -> Self {
        match value {
            1 => Self::Loop,
            2 => Self::PingPong,
            _ => Self::OneShot,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LinearAnimationInstance {
    animation_index: usize,
    time: f32,
    speed_direction: f32,
    total_time: f32,
    last_total_time: f32,
    spilled_time: f32,
    direction: f32,
    did_loop: bool,
    loop_value: Option<u64>,
}

impl LinearAnimationInstance {
    fn new(
        animation_index: usize,
        animation: &RuntimeLinearAnimation,
        speed_multiplier: f32,
    ) -> Self {
        Self {
            animation_index,
            time: animation.start_time_with_speed(speed_multiplier),
            speed_direction: if speed_multiplier >= 0.0 { 1.0 } else { -1.0 },
            total_time: 0.0,
            last_total_time: 0.0,
            spilled_time: 0.0,
            direction: 1.0,
            did_loop: false,
            loop_value: None,
        }
    }

    pub fn animation_index(&self) -> usize {
        self.animation_index
    }

    pub fn time(&self) -> f32 {
        self.time
    }

    pub fn speed_direction(&self) -> f32 {
        self.speed_direction
    }

    pub fn total_time(&self) -> f32 {
        self.total_time
    }

    pub fn last_total_time(&self) -> f32 {
        self.last_total_time
    }

    pub fn spilled_time(&self) -> f32 {
        self.spilled_time
    }

    pub fn direction(&self) -> f32 {
        self.direction
    }

    pub fn did_loop(&self) -> bool {
        self.did_loop
    }

    pub fn loop_value(&self) -> Option<u64> {
        self.loop_value
    }

    fn set_time(&mut self, animation: &RuntimeLinearAnimation, value: f32) {
        if self.time == value {
            return;
        }
        self.time = value;
        let diff = self.total_time - self.last_total_time;
        self.total_time = value - animation.start_seconds();
        self.last_total_time = self.total_time - diff;
        self.direction = 1.0;
    }

    pub fn directed_speed(&self, animation: &RuntimeLinearAnimation) -> f32 {
        self.direction * animation.speed
    }

    fn resolved_loop_kind(&self, animation: &RuntimeLinearAnimation) -> AnimationLoop {
        AnimationLoop::from_loop_value(self.loop_value.unwrap_or(animation.loop_value))
    }

    fn keep_going(&self, animation: &RuntimeLinearAnimation) -> bool {
        self.resolved_loop_kind(animation) != AnimationLoop::OneShot
            || (self.directed_speed(animation) > 0.0 && self.time < animation.end_seconds())
            || (self.directed_speed(animation) < 0.0 && self.time > animation.start_seconds())
    }

    fn keep_going_with_speed_multiplier(
        &self,
        animation: &RuntimeLinearAnimation,
        speed_multiplier: f32,
    ) -> bool {
        self.resolved_loop_kind(animation) != AnimationLoop::OneShot
            || (self.directed_speed(animation) * speed_multiplier > 0.0
                && self.time < animation.end_seconds())
            || (self.directed_speed(animation) * speed_multiplier < 0.0
                && self.time > animation.start_seconds())
    }

    fn advance(&mut self, animation: &RuntimeLinearAnimation, elapsed_seconds: f32) -> bool {
        let delta_seconds = elapsed_seconds * animation.speed * self.direction;
        self.spilled_time = 0.0;
        if delta_seconds == 0.0 {
            self.did_loop = false;
            return false;
        }

        self.last_total_time = self.total_time;
        self.total_time += delta_seconds.abs();
        let kill_spilled_time = !self.keep_going_with_speed_multiplier(animation, elapsed_seconds);

        self.time += delta_seconds;
        let fps = animation.fps_as_f32();
        if fps == 0.0 {
            self.did_loop = false;
            return self.keep_going_with_speed_multiplier(animation, elapsed_seconds);
        }

        let mut frames = self.time * fps;
        let start = animation.start_frame();
        let end = animation.end_frame();
        let range = end - start;
        let mut did_loop = false;
        let mut direction = if delta_seconds < 0.0 { -1 } else { 1 };

        match self.resolved_loop_kind(animation) {
            AnimationLoop::OneShot => {
                if direction == 1 && frames > end {
                    let delta_frames = delta_seconds * fps;
                    let spilled_frames_ratio = (frames - end) / delta_frames;
                    self.spilled_time = spilled_frames_ratio * elapsed_seconds;
                    frames = end;
                    self.time = frames / fps;
                    did_loop = true;
                } else if direction == -1 && frames < start {
                    let delta_frames = (delta_seconds * fps).abs();
                    let spilled_frames_ratio = (start - frames) / delta_frames;
                    self.spilled_time = spilled_frames_ratio * elapsed_seconds;
                    frames = start;
                    self.time = frames / fps;
                    did_loop = true;
                }
            }
            AnimationLoop::Loop => {
                if range != 0.0 && direction == 1 && frames >= end {
                    let delta_frames = delta_seconds * fps;
                    let remainder = (frames - start) % range;
                    let spilled_frames_ratio = remainder / delta_frames;
                    self.spilled_time = spilled_frames_ratio * elapsed_seconds;
                    frames = start + remainder;
                    self.time = frames / fps;
                    did_loop = true;
                } else if range != 0.0 && direction == -1 && frames <= start {
                    let delta_frames = delta_seconds * fps;
                    let remainder = ((start - frames) % range).abs();
                    let spilled_frames_ratio = (remainder / delta_frames).abs();
                    self.spilled_time = spilled_frames_ratio * elapsed_seconds;
                    frames = end - remainder;
                    self.time = frames / fps;
                    did_loop = true;
                }
            }
            AnimationLoop::PingPong => loop {
                if direction == 1 && frames >= end {
                    self.spilled_time = (frames - end) / fps;
                    frames = end + (end - frames);
                } else if direction == -1 && frames < start {
                    self.spilled_time = (start - frames) / fps;
                    frames = start + (start - frames);
                } else {
                    break;
                }
                self.time = frames / fps;
                self.direction *= -1.0;
                direction *= -1;
                did_loop = true;
            },
        }

        if kill_spilled_time {
            self.spilled_time = 0.0;
        }
        self.did_loop = did_loop;
        self.keep_going_with_speed_multiplier(animation, elapsed_seconds)
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeStateMachine {
    pub global_id: u32,
    pub name: Option<String>,
    pub inputs: Vec<RuntimeStateMachineInput>,
    pub layers: Vec<RuntimeStateMachineLayer>,
    bindable_numbers: Vec<RuntimeBindableNumber>,
}

#[derive(Debug, Clone)]
pub struct RuntimeStateMachineInput {
    pub global_id: u32,
    pub name: Option<String>,
    pub kind: StateMachineInputKind,
    value: StateMachineInputValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateMachineInputKind {
    Bool,
    Number,
    Trigger,
}

#[derive(Debug, Clone)]
enum StateMachineInputValue {
    Bool(bool),
    Number(f32),
    Trigger {
        fired: bool,
        used_layers: Vec<usize>,
    },
}

#[derive(Debug, Clone)]
struct RuntimeBindableNumber {
    global_id: u32,
    data_bind_indices: Vec<usize>,
    value: f32,
}

#[derive(Debug, Clone)]
pub struct RuntimeStateMachineLayer {
    pub global_id: u32,
    pub name: Option<String>,
    pub states: Vec<RuntimeLayerState>,
    entry_state_index: Option<usize>,
    any_state_index: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct RuntimeLayerState {
    pub global_id: Option<u32>,
    pub type_name: Option<&'static str>,
    animation_index: Option<usize>,
    blend_state_1d: Option<RuntimeBlendState1D>,
    blend_state_direct: Option<RuntimeBlendStateDirect>,
    speed: f32,
    flags: u64,
    fire_actions: Vec<RuntimeStateMachineFireEvent>,
    listener_actions: Vec<RuntimeScheduledListenerAction>,
    transitions: Vec<RuntimeStateTransition>,
}

impl RuntimeLayerState {
    const RANDOM: u64 = 1 << 0;

    fn uses_random_transition_selection(&self) -> bool {
        self.flags & Self::RANDOM == Self::RANDOM
    }

    fn fire_events(
        &self,
        occurrence: StateMachineFireOccurrence,
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) {
        fire_state_machine_events(&self.fire_actions, occurrence, reported_events);
    }

    fn perform_listener_actions(
        &self,
        occurrence: StateMachineFireOccurrence,
        inputs: &mut [StateMachineInputInstance],
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) -> bool {
        perform_scheduled_listener_actions(
            &self.listener_actions,
            occurrence,
            inputs,
            reported_events,
        )
    }
}

#[derive(Debug, Clone)]
struct RuntimeBlendState1D {
    source: RuntimeBlendState1DSource,
    animations: Vec<RuntimeBlendAnimation1D>,
}

impl RuntimeBlendState1D {
    fn from_imported(
        file: &RuntimeFile,
        state: &rive_binary::RuntimeLayerState<'_>,
        animation_index_by_global: &BTreeMap<u32, usize>,
    ) -> Option<Self> {
        let object = state.object?;
        let source = match object.type_name {
            "BlendState1DInput" => RuntimeBlendState1DSource::Input {
                input_index: object
                    .uint_property("inputId")
                    .filter(|input_id| *input_id != u64::from(u32::MAX))
                    .and_then(|input_id| usize::try_from(input_id).ok()),
            },
            "BlendState1DViewModel" => RuntimeBlendState1DSource::BindableProperty {
                global_id: file.latest_bindable_property_for_object(object)?.id as u32,
            },
            _ => return None,
        };
        let animations = state
            .blend_animations
            .iter()
            .filter_map(|animation| {
                let animation_index = animation
                    .animation
                    .and_then(|animation| animation_index_by_global.get(&animation.id).copied())?;
                Some(RuntimeBlendAnimation1D {
                    animation_index,
                    value: animation.object.double_property("value").unwrap_or(0.0),
                })
            })
            .collect::<Vec<_>>();
        (!animations.is_empty()).then_some(Self { source, animations })
    }
}

#[derive(Debug, Clone)]
enum RuntimeBlendState1DSource {
    Input { input_index: Option<usize> },
    BindableProperty { global_id: u32 },
}

#[derive(Debug, Clone)]
struct RuntimeBlendAnimation1D {
    animation_index: usize,
    value: f32,
}

#[derive(Debug, Clone)]
struct RuntimeBlendStateDirect {
    animations: Vec<RuntimeBlendAnimationDirect>,
}

impl RuntimeBlendStateDirect {
    fn from_imported(
        file: &RuntimeFile,
        state: &rive_binary::RuntimeLayerState<'_>,
        animation_index_by_global: &BTreeMap<u32, usize>,
    ) -> Option<Self> {
        let object = state.object?;
        if object.type_name != "BlendStateDirect" {
            return None;
        }
        let animations = state
            .blend_animations
            .iter()
            .filter_map(|animation| {
                if animation.object.type_name != "BlendAnimationDirect" {
                    return None;
                }
                let animation_index = animation
                    .animation
                    .and_then(|animation| animation_index_by_global.get(&animation.id).copied())?;
                Some(RuntimeBlendAnimationDirect {
                    animation_index,
                    source: RuntimeDirectBlendSource::from_object(file, animation.object)?,
                })
            })
            .collect::<Vec<_>>();
        (!animations.is_empty()).then_some(Self { animations })
    }
}

#[derive(Debug, Clone)]
struct RuntimeBlendAnimationDirect {
    animation_index: usize,
    source: RuntimeDirectBlendSource,
}

#[derive(Debug, Clone)]
enum RuntimeDirectBlendSource {
    Input { input_index: usize },
    MixValue { value: f32 },
    BindableProperty { global_id: u32 },
}

impl RuntimeDirectBlendSource {
    fn from_object(file: &RuntimeFile, object: &RuntimeObject) -> Option<Self> {
        match object.uint_property("blendSource").unwrap_or(0) {
            0 => Some(Self::Input {
                input_index: usize::try_from(object.uint_property("inputId")?).ok()?,
            }),
            1 => Some(Self::MixValue {
                value: object.double_property("mixValue").unwrap_or(100.0),
            }),
            2 => Some(Self::BindableProperty {
                global_id: file.latest_bindable_property_for_object(object)?.id as u32,
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
struct RuntimeStateTransition {
    state_to_index: Option<usize>,
    exit_blend_animation_index: Option<usize>,
    duration: u64,
    exit_time: u64,
    flags: u64,
    random_weight: u64,
    condition_count: usize,
    conditions: Vec<RuntimeTransitionCondition>,
    fire_actions: Vec<RuntimeStateMachineFireEvent>,
    listener_actions: Vec<RuntimeScheduledListenerAction>,
    interpolator: Option<RuntimeTransitionInterpolator>,
    has_unsupported_interpolator: bool,
}

#[derive(Debug, Clone, Copy)]
struct RuntimeTransitionAnimationRef<'a> {
    instance: &'a LinearAnimationInstance,
    animation: &'a RuntimeLinearAnimation,
}

impl RuntimeStateTransition {
    const DISABLED: u64 = 1 << 0;
    const DURATION_IS_PERCENTAGE: u64 = 1 << 1;
    const ENABLE_EXIT_TIME: u64 = 1 << 2;
    const EXIT_TIME_IS_PERCENTAGE: u64 = 1 << 3;
    const PAUSE_ON_EXIT: u64 = 1 << 4;
    const ENABLE_EARLY_EXIT: u64 = 1 << 5;

    fn is_simple_supported(&self) -> bool {
        self.state_to_index.is_some()
            && self.condition_count == self.conditions.len()
            && !self.has_unsupported_interpolator
            && self.flags & Self::DISABLED == 0
    }

    fn allow(
        &self,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        data_context_present: bool,
        layer_index: usize,
        animation_from: Option<RuntimeTransitionAnimationRef<'_>>,
    ) -> TransitionAllowance {
        if !self.conditions.iter().all(|condition| {
            condition.evaluate(inputs, bindable_numbers, data_context_present, layer_index)
        }) {
            return TransitionAllowance::No;
        }

        if self.flags & Self::ENABLE_EXIT_TIME == Self::ENABLE_EXIT_TIME
            && let Some(animation_from) = animation_from
        {
            let mut exit_time = self.exit_time_seconds(Some(animation_from.animation), false);
            let duration = animation_from.animation.duration_seconds();
            if exit_time <= duration
                && AnimationLoop::from_loop_value(animation_from.animation.loop_value)
                    != AnimationLoop::OneShot
                && duration != 0.0
            {
                exit_time +=
                    (animation_from.instance.last_total_time / duration).floor() * duration;
            }
            if animation_from.instance.total_time < exit_time {
                return TransitionAllowance::WaitingForExit;
            }
        }

        TransitionAllowance::Yes
    }

    fn has_exit_time(&self) -> bool {
        self.flags & Self::ENABLE_EXIT_TIME == Self::ENABLE_EXIT_TIME
    }

    fn pause_on_exit(&self) -> bool {
        self.flags & Self::PAUSE_ON_EXIT == Self::PAUSE_ON_EXIT
    }

    fn enable_early_exit(&self) -> bool {
        self.flags & Self::ENABLE_EARLY_EXIT == Self::ENABLE_EARLY_EXIT
    }

    fn fire_events(
        &self,
        occurrence: StateMachineFireOccurrence,
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) {
        fire_state_machine_events(&self.fire_actions, occurrence, reported_events);
    }

    fn perform_listener_actions(
        &self,
        occurrence: StateMachineFireOccurrence,
        inputs: &mut [StateMachineInputInstance],
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) -> bool {
        perform_scheduled_listener_actions(
            &self.listener_actions,
            occurrence,
            inputs,
            reported_events,
        )
    }

    fn transition_duration_seconds(&self, animation_from: Option<&RuntimeLinearAnimation>) -> f32 {
        if self.duration == 0 {
            return 0.0;
        }
        if self.flags & Self::DURATION_IS_PERCENTAGE == Self::DURATION_IS_PERCENTAGE {
            return animation_from
                .map(|animation| self.duration as f32 / 100.0 * animation.duration_seconds())
                .unwrap_or(0.0);
        }
        self.duration as f32 / 1000.0
    }

    fn exit_time_seconds(
        &self,
        animation_from: Option<&RuntimeLinearAnimation>,
        absolute: bool,
    ) -> f32 {
        if self.flags & Self::EXIT_TIME_IS_PERCENTAGE == Self::EXIT_TIME_IS_PERCENTAGE {
            return animation_from
                .map(|animation| {
                    let start = if absolute {
                        animation.start_seconds()
                    } else {
                        0.0
                    };
                    start + self.exit_time as f32 / 100.0 * animation.duration_seconds()
                })
                .unwrap_or(0.0);
        }
        self.exit_time as f32 / 1000.0
    }

    fn use_inputs(&self, inputs: &mut [StateMachineInputInstance], layer_index: usize) {
        for condition in &self.conditions {
            condition.use_input(inputs, layer_index);
        }
    }
}

#[derive(Debug, Clone)]
struct RuntimeStateMachineFireEvent {
    occurs_value: u64,
    event: StateMachineReportedEvent,
}

#[derive(Debug, Clone)]
enum RuntimeScheduledListenerAction {
    FireEvent {
        flags: u64,
        event: StateMachineReportedEvent,
    },
    BoolChange {
        flags: u64,
        input_index: usize,
        value: u64,
    },
    NumberChange {
        flags: u64,
        input_index: usize,
        value: f32,
    },
    TriggerChange {
        flags: u64,
        input_index: usize,
    },
}

impl RuntimeScheduledListenerAction {
    fn from_imported(action: &rive_binary::RuntimeListenerAction<'_>) -> Option<Self> {
        let flags = action.object.uint_property("flags").unwrap_or(0);
        match action.object.type_name {
            "ListenerFireEvent" => {
                let event = action.event?;
                Some(Self::FireEvent {
                    flags,
                    event: StateMachineReportedEvent {
                        event_local_index: action.event_local_index?,
                        event_core_type: u32::from(event.type_key),
                        name: event.string_property("name").map(ToOwned::to_owned),
                        seconds_delay: 0.0,
                    },
                })
            }
            "ListenerBoolChange" => Some(Self::BoolChange {
                flags,
                input_index: listener_action_input_index(action)?,
                value: action.object.uint_property("value").unwrap_or(1),
            }),
            "ListenerNumberChange" => Some(Self::NumberChange {
                flags,
                input_index: listener_action_input_index(action)?,
                value: action.object.double_property("value").unwrap_or(0.0),
            }),
            "ListenerTriggerChange" => Some(Self::TriggerChange {
                flags,
                input_index: listener_action_input_index(action)?,
            }),
            _ => None,
        }
    }
}

fn listener_action_input_index(action: &rive_binary::RuntimeListenerAction<'_>) -> Option<usize> {
    if action
        .object
        .uint_property("nestedInputId")
        .is_some_and(|nested_input_id| nested_input_id != u64::from(u32::MAX))
    {
        return None;
    }
    usize::try_from(action.object.uint_property("inputId")?).ok()
}

impl RuntimeStateMachineFireEvent {
    fn from_imported(action: &rive_binary::RuntimeStateMachineFireAction<'_>) -> Option<Self> {
        let event = action.event?;
        Some(Self {
            occurs_value: action.object.uint_property("occursValue").unwrap_or(0),
            event: StateMachineReportedEvent {
                event_local_index: action.event_local_index?,
                event_core_type: u32::from(event.type_key),
                name: event.string_property("name").map(ToOwned::to_owned),
                seconds_delay: 0.0,
            },
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StateMachineFireOccurrence {
    AtStart,
    AtEnd,
}

impl StateMachineFireOccurrence {
    fn value(self) -> u64 {
        match self {
            Self::AtStart => 0,
            Self::AtEnd => 1,
        }
    }
}

fn fire_state_machine_events(
    fire_actions: &[RuntimeStateMachineFireEvent],
    occurrence: StateMachineFireOccurrence,
    reported_events: &mut Vec<StateMachineReportedEvent>,
) {
    for action in fire_actions {
        if action.occurs_value == occurrence.value() {
            reported_events.push(action.event.clone());
        }
    }
}

fn perform_scheduled_listener_actions(
    listener_actions: &[RuntimeScheduledListenerAction],
    occurrence: StateMachineFireOccurrence,
    inputs: &mut [StateMachineInputInstance],
    reported_events: &mut Vec<StateMachineReportedEvent>,
) -> bool {
    let mut changed_input = false;
    for action in listener_actions {
        let flags = match action {
            RuntimeScheduledListenerAction::FireEvent { flags, .. }
            | RuntimeScheduledListenerAction::BoolChange { flags, .. }
            | RuntimeScheduledListenerAction::NumberChange { flags, .. }
            | RuntimeScheduledListenerAction::TriggerChange { flags, .. } => *flags,
        };
        if flags & 1 != occurrence.value() {
            continue;
        }
        match action {
            RuntimeScheduledListenerAction::FireEvent { event, .. } => {
                reported_events.push(event.clone());
            }
            RuntimeScheduledListenerAction::BoolChange {
                input_index, value, ..
            } => {
                if let Some(input) = inputs.get_mut(*input_index) {
                    changed_input |= input.apply_listener_bool_change(*value);
                }
            }
            RuntimeScheduledListenerAction::NumberChange {
                input_index, value, ..
            } => {
                if let Some(input) = inputs.get_mut(*input_index) {
                    changed_input |= input.set_number(*value);
                }
            }
            RuntimeScheduledListenerAction::TriggerChange { input_index, .. } => {
                if let Some(input) = inputs.get_mut(*input_index) {
                    changed_input |= input.fire_trigger();
                }
            }
        }
    }
    changed_input
}

#[derive(Debug, Clone, Copy)]
enum RuntimeTransitionInterpolator {
    CubicEase {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    },
    Elastic {
        amplitude: f32,
        period: f32,
        easing_value: u64,
    },
}

impl RuntimeTransitionInterpolator {
    fn from_object(object: &RuntimeObject) -> Option<Self> {
        match object.type_name {
            "CubicEaseInterpolator" => Some(Self::CubicEase {
                x1: object.double_property("x1").unwrap_or(0.42),
                y1: object.double_property("y1").unwrap_or(0.0),
                x2: object.double_property("x2").unwrap_or(0.58),
                y2: object.double_property("y2").unwrap_or(1.0),
            }),
            "ElasticInterpolator" => Some(Self::Elastic {
                amplitude: object.double_property("amplitude").unwrap_or(1.0),
                period: object.double_property("period").unwrap_or(1.0),
                easing_value: object.uint_property("easingValue").unwrap_or(1),
            }),
            _ => None,
        }
    }

    fn transform(self, factor: f32) -> f32 {
        match self {
            Self::CubicEase { x1, y1, x2, y2 } => {
                let t = cubic_interpolator_get_t(factor, x1, x2);
                cubic_interpolator_calc_bezier(t, y1, y2)
            }
            Self::Elastic {
                amplitude,
                period,
                easing_value,
            } => elastic_interpolator_transform(factor, amplitude, period, easing_value),
        }
    }
}

fn cubic_interpolator_calc_bezier(t: f32, a1: f32, a2: f32) -> f32 {
    (((1.0 - 3.0 * a2 + 3.0 * a1) * t + (3.0 * a2 - 6.0 * a1)) * t + (3.0 * a1)) * t
}

fn cubic_interpolator_slope(t: f32, a1: f32, a2: f32) -> f32 {
    3.0 * (1.0 - 3.0 * a2 + 3.0 * a1) * t * t + 2.0 * (3.0 * a2 - 6.0 * a1) * t + (3.0 * a1)
}

fn cubic_interpolator_get_t(x: f32, x1: f32, x2: f32) -> f32 {
    const SPLINE_TABLE_SIZE: usize = 11;
    const SAMPLE_STEP_SIZE: f32 = 1.0 / (SPLINE_TABLE_SIZE as f32 - 1.0);
    const NEWTON_ITERATIONS: usize = 4;
    const NEWTON_MIN_SLOPE: f32 = 0.001;
    const SUBDIVISION_PRECISION: f32 = 0.0000001;
    const SUBDIVISION_MAX_ITERATIONS: usize = 10;

    let mut values = [0.0; SPLINE_TABLE_SIZE];
    for (i, value) in values.iter_mut().enumerate() {
        *value = cubic_interpolator_calc_bezier(i as f32 * SAMPLE_STEP_SIZE, x1, x2);
    }

    let mut interval_start = 0.0;
    let mut current_sample = 1;
    let last_sample = SPLINE_TABLE_SIZE - 1;
    while current_sample != last_sample && values[current_sample] <= x {
        interval_start += SAMPLE_STEP_SIZE;
        current_sample += 1;
    }
    current_sample -= 1;

    let dist = (x - values[current_sample]) / (values[current_sample + 1] - values[current_sample]);
    let mut guess_for_t = interval_start + dist * SAMPLE_STEP_SIZE;
    let initial_slope = cubic_interpolator_slope(guess_for_t, x1, x2);
    if initial_slope >= NEWTON_MIN_SLOPE {
        for _ in 0..NEWTON_ITERATIONS {
            let current_slope = cubic_interpolator_slope(guess_for_t, x1, x2);
            if current_slope == 0.0 {
                return guess_for_t;
            }
            let current_x = cubic_interpolator_calc_bezier(guess_for_t, x1, x2) - x;
            guess_for_t -= current_x / current_slope;
        }
        guess_for_t
    } else if initial_slope == 0.0 {
        guess_for_t
    } else {
        let mut upper_bound = interval_start + SAMPLE_STEP_SIZE;
        let mut iterations = 0;
        loop {
            let current_t = interval_start + (upper_bound - interval_start) / 2.0;
            let current_x = cubic_interpolator_calc_bezier(current_t, x1, x2) - x;
            if current_x > 0.0 {
                upper_bound = current_t;
            } else {
                interval_start = current_t;
            }
            iterations += 1;
            if current_x.abs() <= SUBDIVISION_PRECISION || iterations >= SUBDIVISION_MAX_ITERATIONS
            {
                return current_t;
            }
        }
    }
}

fn elastic_interpolator_transform(
    factor: f32,
    amplitude: f32,
    serialized_period: f32,
    easing_value: u64,
) -> f32 {
    let period = if serialized_period == 0.0 {
        0.5
    } else {
        serialized_period
    };
    let shift = if amplitude < 1.0 {
        period / 4.0
    } else {
        period / (2.0 * std::f32::consts::PI) * (1.0 / amplitude).asin()
    };

    match easing_value {
        0 => elastic_ease_in(factor, amplitude, period, shift),
        1 => elastic_ease_out(factor, amplitude, period, shift),
        2 => elastic_ease_in_out(factor, amplitude, period, shift),
        _ => factor,
    }
}

fn elastic_actual_amplitude(time: f32, amplitude: f32, shift: f32) -> f32 {
    if amplitude < 1.0 {
        let shift_abs = shift.abs();
        let time_abs = time.abs();
        if time_abs < shift_abs {
            let l = time_abs / shift_abs;
            return (amplitude * l) + (1.0 - l);
        }
    }

    amplitude
}

fn elastic_ease_out(factor: f32, amplitude: f32, period: f32, shift: f32) -> f32 {
    let time = factor;
    let actual_amplitude = elastic_actual_amplitude(time, amplitude, shift);
    actual_amplitude
        * 2.0_f32.powf(10.0 * -time)
        * ((time - shift) * (2.0 * std::f32::consts::PI) / period).sin()
        + 1.0
}

fn elastic_ease_in(factor: f32, amplitude: f32, period: f32, shift: f32) -> f32 {
    let time = factor - 1.0;
    let actual_amplitude = elastic_actual_amplitude(time, amplitude, shift);
    -(actual_amplitude
        * 2.0_f32.powf(10.0 * time)
        * ((-time - shift) * (2.0 * std::f32::consts::PI) / period).sin())
}

fn elastic_ease_in_out(factor: f32, amplitude: f32, period: f32, shift: f32) -> f32 {
    let time = factor * 2.0 - 1.0;
    let actual_amplitude = elastic_actual_amplitude(time, amplitude, shift);
    if time < 0.0 {
        -0.5 * actual_amplitude
            * 2.0_f32.powf(10.0 * time)
            * ((-time - shift) * (2.0 * std::f32::consts::PI) / period).sin()
    } else {
        0.5 * (actual_amplitude
            * 2.0_f32.powf(10.0 * -time)
            * ((time - shift) * (2.0 * std::f32::consts::PI) / period).sin())
            + 1.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransitionAllowance {
    No,
    WaitingForExit,
    Yes,
}

#[derive(Debug, Clone)]
enum RuntimeTransitionCondition {
    Bool {
        input_index: usize,
        op: TransitionConditionOp,
    },
    Number {
        input_index: usize,
        op: TransitionConditionOp,
        value: f32,
    },
    Trigger {
        input_index: usize,
    },
    ViewModelNumber {
        bindable_global_id: u32,
        op: TransitionConditionOp,
        value: f32,
    },
}

impl RuntimeTransitionCondition {
    fn from_object(file: &RuntimeFile, object: &RuntimeObject) -> Option<Self> {
        match object.type_name {
            "TransitionBoolCondition" => {
                let input_index = usize::try_from(object.uint_property("inputId")?).ok()?;
                Some(Self::Bool {
                    input_index,
                    op: TransitionConditionOp::from_value(
                        object.uint_property("opValue").unwrap_or(0),
                    ),
                })
            }
            "TransitionNumberCondition" => {
                let input_index = usize::try_from(object.uint_property("inputId")?).ok()?;
                Some(Self::Number {
                    input_index,
                    op: TransitionConditionOp::from_value(
                        object.uint_property("opValue").unwrap_or(0),
                    ),
                    value: object.double_property("value").unwrap_or(0.0),
                })
            }
            "TransitionTriggerCondition" => {
                let input_index = usize::try_from(object.uint_property("inputId")?).ok()?;
                Some(Self::Trigger { input_index })
            }
            "TransitionViewModelCondition" => {
                let comparators = file.transition_view_model_condition_comparators(object)?;
                let left = comparators.left?;
                let right = comparators.right?;
                if left.type_name != "TransitionPropertyViewModelComparator"
                    || right.type_name != "TransitionValueNumberComparator"
                {
                    return None;
                }
                let bindable = file.latest_bindable_property_for_object(left)?;
                if bindable.type_name != "BindablePropertyNumber" {
                    return None;
                }
                Some(Self::ViewModelNumber {
                    bindable_global_id: bindable.id,
                    op: TransitionConditionOp::from_value(
                        object.uint_property("opValue").unwrap_or(0),
                    ),
                    value: right.double_property("value").unwrap_or(0.0),
                })
            }
            _ => None,
        }
    }

    fn evaluate(
        &self,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        data_context_present: bool,
        layer_index: usize,
    ) -> bool {
        match self {
            Self::Bool { input_index, op } => {
                let Some(value) = inputs
                    .get(*input_index)
                    .and_then(StateMachineInputInstance::bool_value)
                else {
                    return true;
                };
                (value && *op == TransitionConditionOp::Equal)
                    || (!value && *op == TransitionConditionOp::NotEqual)
            }
            Self::Number {
                input_index,
                op,
                value,
            } => {
                let Some(input_value) = inputs
                    .get(*input_index)
                    .and_then(StateMachineInputInstance::number_value)
                else {
                    return true;
                };
                op.compare(input_value, *value)
            }
            Self::Trigger { input_index } => {
                let Some(input) = inputs.get(*input_index) else {
                    return true;
                };
                input
                    .trigger_is_fireable_for_layer(layer_index)
                    .unwrap_or(true)
            }
            Self::ViewModelNumber {
                bindable_global_id,
                op,
                value,
            } => {
                if !data_context_present {
                    return false;
                }
                let input_value =
                    bindable_number_value(bindable_numbers, *bindable_global_id).unwrap_or(0.0);
                op.compare(input_value, *value)
            }
        }
    }

    fn use_input(&self, inputs: &mut [StateMachineInputInstance], layer_index: usize) {
        if let Self::Trigger { input_index } = self
            && let Some(input) = inputs.get_mut(*input_index)
        {
            input.use_trigger_in_layer(layer_index);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransitionConditionOp {
    Equal,
    NotEqual,
    LessThanOrEqual,
    GreaterThanOrEqual,
    LessThan,
    GreaterThan,
}

impl TransitionConditionOp {
    fn from_value(value: u64) -> Self {
        match value {
            1 => Self::NotEqual,
            2 => Self::LessThanOrEqual,
            3 => Self::GreaterThanOrEqual,
            4 => Self::LessThan,
            5 => Self::GreaterThan,
            _ => Self::Equal,
        }
    }

    fn compare(self, input_value: f32, value: f32) -> bool {
        match self {
            Self::Equal => input_value == value,
            Self::NotEqual => input_value != value,
            Self::LessThanOrEqual => input_value <= value,
            Self::GreaterThanOrEqual => input_value >= value,
            Self::LessThan => input_value < value,
            Self::GreaterThan => input_value > value,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StateMachineInstance {
    state_machine_index: usize,
    inputs: Vec<StateMachineInputInstance>,
    bindable_numbers: Vec<StateMachineBindableNumberInstance>,
    layers: Vec<StateMachineLayerInstance>,
    reported_events: Vec<StateMachineReportedEvent>,
    changed_state_count: usize,
    needs_advance: bool,
    data_context_present: bool,
}

#[derive(Debug, Clone)]
pub struct StateMachineReportedEvent {
    event_local_index: usize,
    event_core_type: u32,
    name: Option<String>,
    seconds_delay: f32,
}

impl StateMachineReportedEvent {
    pub fn event_local_index(&self) -> usize {
        self.event_local_index
    }

    pub fn event_core_type(&self) -> u32 {
        self.event_core_type
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn seconds_delay(&self) -> f32 {
        self.seconds_delay
    }
}

impl StateMachineInstance {
    fn new(
        state_machine_index: usize,
        state_machine: &RuntimeStateMachine,
        artboard: &ArtboardInstance,
    ) -> Self {
        let inputs = state_machine
            .inputs
            .iter()
            .enumerate()
            .map(|(index, input)| StateMachineInputInstance::new(index, input))
            .collect::<Vec<_>>();
        let bindable_numbers = state_machine
            .bindable_numbers
            .iter()
            .map(StateMachineBindableNumberInstance::new)
            .collect::<Vec<_>>();
        let layers = state_machine
            .layers
            .iter()
            .map(|layer| {
                StateMachineLayerInstance::new(layer, artboard, &inputs, &bindable_numbers)
            })
            .collect();
        Self {
            state_machine_index,
            inputs,
            bindable_numbers,
            layers,
            reported_events: Vec::new(),
            changed_state_count: 0,
            needs_advance: false,
            data_context_present: false,
        }
    }

    pub fn state_machine_index(&self) -> usize {
        self.state_machine_index
    }

    pub fn changed_state_count(&self) -> usize {
        self.changed_state_count
    }

    pub fn needs_advance(&self) -> bool {
        self.needs_advance
    }

    pub fn input_count(&self) -> usize {
        self.inputs.len()
    }

    pub fn input(&self, index: usize) -> Option<&StateMachineInputInstance> {
        self.inputs.get(index)
    }

    pub fn input_named(&self, name: &str) -> Option<&StateMachineInputInstance> {
        self.inputs
            .iter()
            .find(|input| input.name.as_deref() == Some(name))
    }

    pub fn input_index_named(&self, name: &str) -> Option<usize> {
        self.inputs
            .iter()
            .position(|input| input.name.as_deref() == Some(name))
    }

    pub fn set_bool(&mut self, index: usize, value: bool) -> bool {
        let Some(input) = self.inputs.get_mut(index) else {
            return false;
        };
        if !input.set_bool(value) {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_number(&mut self, index: usize, value: f32) -> bool {
        let Some(input) = self.inputs.get_mut(index) else {
            return false;
        };
        if !input.set_number(value) {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn fire_trigger(&mut self, index: usize) -> bool {
        let Some(input) = self.inputs.get_mut(index) else {
            return false;
        };
        if !input.fire_trigger() {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_bindable_number_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: f32,
    ) -> bool {
        let Some(bindable_number) = self
            .bindable_numbers
            .iter_mut()
            .find(|bindable_number| bindable_number.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_number.set_value(value) {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn bind_empty_data_context(&mut self) -> bool {
        if self.data_context_present {
            return false;
        }
        self.data_context_present = true;
        self.needs_advance = true;
        true
    }

    pub fn current_animation_count(&self) -> usize {
        self.layers
            .iter()
            .filter(|layer| layer.current_animation.is_some())
            .count()
    }

    pub fn current_animation(&self, index: usize) -> Option<&LinearAnimationInstance> {
        self.layers
            .iter()
            .filter_map(|layer| layer.current_animation.as_ref())
            .nth(index)
    }

    pub fn reported_event_count(&self) -> usize {
        self.reported_events.len()
    }

    pub fn reported_event(&self, index: usize) -> Option<&StateMachineReportedEvent> {
        self.reported_events.get(index)
    }

    fn advance(
        &mut self,
        artboard: &mut ArtboardInstance,
        state_machine: &RuntimeStateMachine,
        elapsed_seconds: f32,
    ) -> bool {
        self.reported_events.clear();
        self.changed_state_count = 0;
        self.needs_advance = false;
        let mut keep_going = false;
        for (layer_index, (layer_instance, layer)) in self
            .layers
            .iter_mut()
            .zip(&state_machine.layers)
            .enumerate()
        {
            let layer_result = layer_instance.advance(
                artboard,
                layer,
                elapsed_seconds,
                layer_index,
                &mut self.inputs,
                &self.bindable_numbers,
                self.data_context_present,
                &mut self.reported_events,
            );
            if layer_result.changed_state {
                self.changed_state_count += 1;
            }
            keep_going |= layer_result.keep_going;
        }
        for input in &mut self.inputs {
            input.advanced();
        }
        self.needs_advance = keep_going || !self.reported_events.is_empty();
        self.needs_advance
    }
}

#[derive(Debug, Clone)]
pub struct StateMachineInputInstance {
    index: usize,
    global_id: u32,
    name: Option<String>,
    kind: StateMachineInputKind,
    value: StateMachineInputValue,
}

impl StateMachineInputInstance {
    fn new(index: usize, input: &RuntimeStateMachineInput) -> Self {
        Self {
            index,
            global_id: input.global_id,
            name: input.name.clone(),
            kind: input.kind,
            value: input.value.clone(),
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn global_id(&self) -> u32 {
        self.global_id
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn kind(&self) -> StateMachineInputKind {
        self.kind
    }

    pub fn bool_value(&self) -> Option<bool> {
        match self.value {
            StateMachineInputValue::Bool(value) => Some(value),
            _ => None,
        }
    }

    pub fn number_value(&self) -> Option<f32> {
        match self.value {
            StateMachineInputValue::Number(value) => Some(value),
            _ => None,
        }
    }

    pub fn trigger_fired(&self) -> Option<bool> {
        match self.value {
            StateMachineInputValue::Trigger { fired, .. } => Some(fired),
            _ => None,
        }
    }

    fn set_bool(&mut self, value: bool) -> bool {
        match &mut self.value {
            StateMachineInputValue::Bool(current) => {
                if *current == value {
                    return false;
                }
                *current = value;
                true
            }
            _ => false,
        }
    }

    fn set_number(&mut self, value: f32) -> bool {
        match &mut self.value {
            StateMachineInputValue::Number(current) => {
                if *current == value {
                    return false;
                }
                *current = value;
                true
            }
            _ => false,
        }
    }

    fn apply_listener_bool_change(&mut self, value: u64) -> bool {
        match &mut self.value {
            StateMachineInputValue::Bool(current) => {
                let next = match value {
                    0 => false,
                    1 => true,
                    _ => !*current,
                };
                if *current == next {
                    return false;
                }
                *current = next;
                true
            }
            _ => false,
        }
    }

    fn fire_trigger(&mut self) -> bool {
        match &mut self.value {
            StateMachineInputValue::Trigger { fired, .. } => {
                if *fired {
                    return false;
                }
                *fired = true;
                true
            }
            _ => false,
        }
    }

    fn trigger_is_fireable_for_layer(&self, layer_index: usize) -> Option<bool> {
        match &self.value {
            StateMachineInputValue::Trigger { fired, used_layers } => {
                Some(*fired && !used_layers.contains(&layer_index))
            }
            _ => None,
        }
    }

    fn use_trigger_in_layer(&mut self, layer_index: usize) {
        if let StateMachineInputValue::Trigger { used_layers, .. } = &mut self.value
            && !used_layers.contains(&layer_index)
        {
            used_layers.push(layer_index);
        }
    }

    fn advanced(&mut self) {
        if let StateMachineInputValue::Trigger { fired, used_layers } = &mut self.value {
            *fired = false;
            used_layers.clear();
        }
    }
}

#[derive(Debug, Clone)]
struct StateMachineBindableNumberInstance {
    global_id: u32,
    data_bind_indices: Vec<usize>,
    value: f32,
}

impl StateMachineBindableNumberInstance {
    fn new(bindable_number: &RuntimeBindableNumber) -> Self {
        Self {
            global_id: bindable_number.global_id,
            data_bind_indices: bindable_number.data_bind_indices.clone(),
            value: bindable_number.value,
        }
    }

    fn has_data_bind_index(&self, data_bind_index: usize) -> bool {
        self.data_bind_indices.contains(&data_bind_index)
    }

    fn set_value(&mut self, value: f32) -> bool {
        if self.value == value {
            return false;
        }
        self.value = value;
        true
    }
}

fn bindable_number_value(
    bindable_numbers: &[StateMachineBindableNumberInstance],
    global_id: u32,
) -> Option<f32> {
    bindable_numbers
        .iter()
        .find(|bindable_number| bindable_number.global_id == global_id)
        .map(|bindable_number| bindable_number.value)
}

#[derive(Debug, Clone)]
struct StateMachineLayerInstance {
    current_state_index: Option<usize>,
    current_animation: Option<LinearAnimationInstance>,
    current_blend_state_1d: Option<BlendState1DInstance>,
    current_blend_state_direct: Option<BlendStateDirectInstance>,
    current_animation_keep_going: bool,
    transition_source_state_index: Option<usize>,
    transition_source_animation: Option<LinearAnimationInstance>,
    transition_source_blend_state_1d: Option<BlendState1DInstance>,
    transition_source_blend_state_direct: Option<BlendStateDirectInstance>,
    transition_duration_seconds: f32,
    transition_mix: f32,
    transition_mix_from: f32,
    transition_source_paused: bool,
    transition_interpolator: Option<RuntimeTransitionInterpolator>,
    transition_enable_early_exit: bool,
    transition_fire_actions: Vec<RuntimeStateMachineFireEvent>,
    transition_listener_actions: Vec<RuntimeScheduledListenerAction>,
    transition_completed: bool,
    transition_animation_reset: Option<TransitionAnimationReset>,
    waiting_for_exit: bool,
}

#[derive(Debug, Clone, Copy)]
struct StateMachineLayerAdvance {
    changed_state: bool,
    keep_going: bool,
}

#[derive(Debug, Clone)]
struct TransitionAnimationReset {
    entries: Vec<TransitionAnimationResetEntry>,
}

#[derive(Debug, Clone)]
struct TransitionAnimationResetEntry {
    local_id: usize,
    property: TransformProperty,
    value: f32,
}

impl TransitionAnimationReset {
    fn from_animation_indices(
        artboard: &ArtboardInstance,
        animation_indices: &[usize],
    ) -> Option<Self> {
        let mut entries = Vec::new();
        let mut seen = Vec::new();

        for animation_index in animation_indices {
            let Some(animation) = artboard.linear_animation(*animation_index) else {
                continue;
            };
            for keyed_object in &animation.keyed_objects {
                for keyed_property in &keyed_object.keyed_properties {
                    let Some(property) = keyed_property.transform_property else {
                        continue;
                    };
                    let key = (keyed_object.target_local_id, property);
                    if seen.contains(&key) {
                        continue;
                    }
                    let Some(value) =
                        artboard.transform_property(keyed_object.target_local_id, property)
                    else {
                        continue;
                    };
                    seen.push(key);
                    entries.push(TransitionAnimationResetEntry {
                        local_id: keyed_object.target_local_id,
                        property,
                        value,
                    });
                }
            }
        }

        if entries.is_empty() {
            None
        } else {
            Some(Self { entries })
        }
    }

    fn apply(&self, artboard: &mut ArtboardInstance) -> bool {
        let mut changed = false;
        for entry in &self.entries {
            changed |= artboard.set_transform_property(entry.local_id, entry.property, entry.value);
        }
        changed
    }
}

impl StateMachineLayerInstance {
    fn new(
        layer: &RuntimeStateMachineLayer,
        artboard: &ArtboardInstance,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
    ) -> Self {
        let mut instance = Self {
            current_state_index: layer.entry_state_index,
            current_animation: None,
            current_blend_state_1d: None,
            current_blend_state_direct: None,
            current_animation_keep_going: false,
            transition_source_state_index: None,
            transition_source_animation: None,
            transition_source_blend_state_1d: None,
            transition_source_blend_state_direct: None,
            transition_duration_seconds: 0.0,
            transition_mix: 1.0,
            transition_mix_from: 1.0,
            transition_source_paused: false,
            transition_interpolator: None,
            transition_enable_early_exit: false,
            transition_fire_actions: Vec::new(),
            transition_listener_actions: Vec::new(),
            transition_completed: false,
            transition_animation_reset: None,
            waiting_for_exit: false,
        };
        instance.refresh_current_animation(artboard, layer, inputs, bindable_numbers);
        instance
    }

    fn advance(
        &mut self,
        artboard: &mut ArtboardInstance,
        layer: &RuntimeStateMachineLayer,
        elapsed_seconds: f32,
        layer_index: usize,
        inputs: &mut [StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        data_context_present: bool,
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) -> StateMachineLayerAdvance {
        self.advance_current_animation(artboard, layer, elapsed_seconds, inputs, bindable_numbers);
        let input_changed = self.update_transition_mix(elapsed_seconds, inputs, reported_events);
        self.advance_transition_source_animation(
            artboard,
            layer,
            elapsed_seconds,
            inputs,
            bindable_numbers,
        );
        self.apply_animations(artboard);

        let mut changed_state = false;
        for _ in 0..100 {
            if !self.update_state(
                artboard,
                layer,
                layer_index,
                inputs,
                bindable_numbers,
                data_context_present,
                reported_events,
            ) {
                break;
            }
            changed_state = true;
            self.apply_animations(artboard);
        }

        StateMachineLayerAdvance {
            changed_state,
            keep_going: changed_state
                || input_changed
                || self.is_transitioning()
                || self.waiting_for_exit
                || self.current_animation_keep_going,
        }
    }

    fn update_state(
        &mut self,
        artboard: &ArtboardInstance,
        layer: &RuntimeStateMachineLayer,
        layer_index: usize,
        inputs: &mut [StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        data_context_present: bool,
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) -> bool {
        if self.is_transitioning() && !self.transition_enable_early_exit {
            return false;
        }
        self.waiting_for_exit = false;
        if self.try_change_state(
            artboard,
            layer,
            layer.any_state_index,
            layer_index,
            inputs,
            bindable_numbers,
            data_context_present,
            reported_events,
        ) {
            return true;
        }
        self.try_change_state(
            artboard,
            layer,
            self.current_state_index,
            layer_index,
            inputs,
            bindable_numbers,
            data_context_present,
            reported_events,
        )
    }

    fn try_change_state(
        &mut self,
        artboard: &ArtboardInstance,
        layer: &RuntimeStateMachineLayer,
        state_index: Option<usize>,
        layer_index: usize,
        inputs: &mut [StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        data_context_present: bool,
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) -> bool {
        let Some(state_index) = state_index else {
            return false;
        };
        let Some(state) = layer.states.get(state_index) else {
            return false;
        };

        if state.uses_random_transition_selection() {
            let Some((transition_index, state_to_index)) = self.find_random_transition(
                artboard,
                state,
                state_index,
                layer_index,
                inputs,
                bindable_numbers,
                data_context_present,
            ) else {
                return false;
            };
            let transition = &state.transitions[transition_index];
            transition.use_inputs(inputs, layer_index);
            self.change_state(
                artboard,
                layer,
                transition,
                state_to_index,
                inputs,
                bindable_numbers,
                reported_events,
            );
            return true;
        }

        for transition in &state.transitions {
            if !transition.is_simple_supported() {
                continue;
            }
            let Some(state_to_index) = transition.state_to_index else {
                continue;
            };
            if self.current_state_index == Some(state_to_index) {
                continue;
            }
            let animation_from = self.current_transition_animation(
                artboard,
                transition,
                self.current_state_index == Some(state_index),
            );
            match transition.allow(
                inputs,
                bindable_numbers,
                data_context_present,
                layer_index,
                animation_from,
            ) {
                TransitionAllowance::No => continue,
                TransitionAllowance::WaitingForExit => {
                    self.waiting_for_exit = true;
                    continue;
                }
                TransitionAllowance::Yes => {
                    self.waiting_for_exit = false;
                }
            }
            transition.use_inputs(inputs, layer_index);
            self.change_state(
                artboard,
                layer,
                transition,
                state_to_index,
                inputs,
                bindable_numbers,
                reported_events,
            );
            return true;
        }
        false
    }

    fn find_random_transition(
        &mut self,
        artboard: &ArtboardInstance,
        state: &RuntimeLayerState,
        state_index: usize,
        layer_index: usize,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        data_context_present: bool,
    ) -> Option<(usize, usize)> {
        let mut weighted_candidates = Vec::new();
        let mut total_weight = 0_u64;
        let mut waiting_for_exit = false;

        for (transition_index, transition) in state.transitions.iter().enumerate() {
            if !transition.is_simple_supported() {
                continue;
            }
            let Some(state_to_index) = transition.state_to_index else {
                continue;
            };
            if self.current_state_index == Some(state_to_index) {
                continue;
            }

            let animation_from = self.current_transition_animation(
                artboard,
                transition,
                self.current_state_index == Some(state_index),
            );
            match transition.allow(
                inputs,
                bindable_numbers,
                data_context_present,
                layer_index,
                animation_from,
            ) {
                TransitionAllowance::No => {}
                TransitionAllowance::WaitingForExit => {
                    waiting_for_exit = true;
                }
                TransitionAllowance::Yes => {
                    total_weight = total_weight.saturating_add(transition.random_weight);
                    weighted_candidates.push((
                        transition_index,
                        state_to_index,
                        transition.random_weight,
                    ));
                }
            }
        }

        if total_weight == 0 {
            self.waiting_for_exit = waiting_for_exit;
            return None;
        }

        let random_weight = Self::random_transition_value() * total_weight as f64;
        let mut current_weight = 0.0_f64;
        for (transition_index, state_to_index, transition_weight) in weighted_candidates {
            current_weight += transition_weight as f64;
            if current_weight > random_weight {
                self.waiting_for_exit = false;
                return Some((transition_index, state_to_index));
            }
        }

        self.waiting_for_exit = waiting_for_exit;
        None
    }

    fn random_transition_value() -> f64 {
        0.0
    }

    fn current_transition_animation<'a>(
        &'a self,
        artboard: &'a ArtboardInstance,
        transition: &RuntimeStateTransition,
        is_current_state: bool,
    ) -> Option<RuntimeTransitionAnimationRef<'a>> {
        if !is_current_state {
            return None;
        }

        if let Some(animation_instance) = self.current_animation.as_ref() {
            let animation = artboard.linear_animation(animation_instance.animation_index)?;
            return Some(RuntimeTransitionAnimationRef {
                instance: animation_instance,
                animation,
            });
        }

        let blend_animation_index = transition.exit_blend_animation_index?;
        let animation_instance = self
            .current_blend_state_1d
            .as_ref()
            .and_then(|blend_state| blend_state.animation_instance(blend_animation_index))
            .or_else(|| {
                self.current_blend_state_direct
                    .as_ref()
                    .and_then(|blend_state| blend_state.animation_instance(blend_animation_index))
            })?;
        let animation = artboard.linear_animation(animation_instance.animation_index)?;
        Some(RuntimeTransitionAnimationRef {
            instance: animation_instance,
            animation,
        })
    }

    fn change_state(
        &mut self,
        artboard: &ArtboardInstance,
        layer: &RuntimeStateMachineLayer,
        transition: &RuntimeStateTransition,
        state_to_index: usize,
        inputs: &mut [StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) {
        let previous_state_index = self.current_state_index;
        let mut previous_animation = self.current_animation.clone();
        let previous_blend_state_1d = self.current_blend_state_1d.clone();
        let previous_blend_state_direct = self.current_blend_state_direct.clone();
        let previous_spilled_time = previous_animation
            .as_ref()
            .map(LinearAnimationInstance::spilled_time)
            .unwrap_or(0.0);
        let previous_mix = self.transition_mix;
        if let Some(previous_state) =
            previous_state_index.and_then(|state_index| layer.states.get(state_index))
        {
            previous_state.fire_events(StateMachineFireOccurrence::AtEnd, reported_events);
            previous_state.perform_listener_actions(
                StateMachineFireOccurrence::AtEnd,
                inputs,
                reported_events,
            );
        }

        if transition.pause_on_exit()
            && transition.has_exit_time()
            && let Some(animation_instance) = previous_animation.as_mut()
            && let Some(animation) = artboard.linear_animation(animation_instance.animation_index)
        {
            animation_instance.set_time(
                animation,
                transition.exit_time_seconds(Some(animation), true),
            );
        }
        let previous_runtime_animation =
            previous_animation.as_ref().and_then(|animation_instance| {
                artboard.linear_animation(animation_instance.animation_index)
            });
        let transition_duration_seconds =
            transition.transition_duration_seconds(previous_runtime_animation);

        self.current_state_index = Some(state_to_index);
        self.refresh_current_animation(artboard, layer, inputs, bindable_numbers);
        if let Some(current_state) = layer.states.get(state_to_index) {
            current_state.fire_events(StateMachineFireOccurrence::AtStart, reported_events);
            current_state.perform_listener_actions(
                StateMachineFireOccurrence::AtStart,
                inputs,
                reported_events,
            );
        }
        transition.fire_events(StateMachineFireOccurrence::AtStart, reported_events);
        transition.perform_listener_actions(
            StateMachineFireOccurrence::AtStart,
            inputs,
            reported_events,
        );
        if previous_spilled_time != 0.0 {
            self.advance_current_animation(
                artboard,
                layer,
                previous_spilled_time,
                inputs,
                bindable_numbers,
            );
        }

        if transition_duration_seconds == 0.0 {
            transition.fire_events(StateMachineFireOccurrence::AtEnd, reported_events);
            transition.perform_listener_actions(
                StateMachineFireOccurrence::AtEnd,
                inputs,
                reported_events,
            );
            self.clear_transition_source();
            return;
        }

        let has_transition_source = previous_animation.is_some()
            || previous_blend_state_1d.is_some()
            || previous_blend_state_direct.is_some();
        let mut reset_animation_indices = Vec::new();
        if let Some(animation) = previous_animation.as_ref() {
            reset_animation_indices.push(animation.animation_index);
        }
        if let Some(animation) = self.current_animation.as_ref() {
            reset_animation_indices.push(animation.animation_index);
        }
        let transition_animation_reset =
            TransitionAnimationReset::from_animation_indices(artboard, &reset_animation_indices);

        if let Some(source_state_index) = previous_state_index
            && has_transition_source
        {
            self.transition_source_state_index = Some(source_state_index);
            self.transition_source_animation = previous_animation;
            self.transition_source_blend_state_1d = previous_blend_state_1d;
            self.transition_source_blend_state_direct = previous_blend_state_direct;
            self.transition_duration_seconds = transition_duration_seconds;
            self.transition_mix_from = previous_mix;
            self.transition_mix = 0.0;
            self.transition_source_paused = transition.pause_on_exit();
            self.transition_interpolator = transition.interpolator;
            self.transition_enable_early_exit = transition.enable_early_exit();
            self.transition_fire_actions = transition.fire_actions.clone();
            self.transition_listener_actions = transition.listener_actions.clone();
            self.transition_completed = false;
            self.transition_animation_reset = transition_animation_reset;
            self.update_transition_mix(0.0, inputs, reported_events);
        } else {
            self.clear_transition_source();
        }
    }

    fn clear_transition_source(&mut self) {
        self.transition_source_state_index = None;
        self.transition_source_animation = None;
        self.transition_source_blend_state_1d = None;
        self.transition_source_blend_state_direct = None;
        self.transition_duration_seconds = 0.0;
        self.transition_mix = 1.0;
        self.transition_mix_from = 1.0;
        self.transition_source_paused = false;
        self.transition_interpolator = None;
        self.transition_enable_early_exit = false;
        self.transition_fire_actions.clear();
        self.transition_listener_actions.clear();
        self.transition_completed = false;
        self.transition_animation_reset = None;
    }

    fn is_transitioning(&self) -> bool {
        self.has_transition_source()
            && self.transition_duration_seconds != 0.0
            && self.transition_mix < 1.0
    }

    fn has_transition_source(&self) -> bool {
        self.transition_source_animation.is_some()
            || self.transition_source_blend_state_1d.is_some()
            || self.transition_source_blend_state_direct.is_some()
    }

    fn update_transition_mix(
        &mut self,
        elapsed_seconds: f32,
        inputs: &mut [StateMachineInputInstance],
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) -> bool {
        if !self.has_transition_source() || self.transition_duration_seconds == 0.0 {
            self.transition_mix = 1.0;
            return false;
        }
        self.transition_mix = (self.transition_mix
            + elapsed_seconds / self.transition_duration_seconds)
            .clamp(0.0, 1.0);
        if self.transition_mix == 1.0 && !self.transition_completed {
            self.transition_completed = true;
            self.transition_animation_reset = None;
            fire_state_machine_events(
                &self.transition_fire_actions,
                StateMachineFireOccurrence::AtEnd,
                reported_events,
            );
            perform_scheduled_listener_actions(
                &self.transition_listener_actions,
                StateMachineFireOccurrence::AtEnd,
                inputs,
                reported_events,
            )
        } else {
            false
        }
    }

    fn refresh_current_animation(
        &mut self,
        artboard: &ArtboardInstance,
        layer: &RuntimeStateMachineLayer,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
    ) {
        let Some(state) = self
            .current_state_index
            .and_then(|index| layer.states.get(index))
        else {
            self.current_animation = None;
            self.current_blend_state_1d = None;
            self.current_blend_state_direct = None;
            self.current_animation_keep_going = false;
            return;
        };
        if let Some(blend_state) = state.blend_state_1d.as_ref() {
            self.current_animation = None;
            let mut blend_instance = BlendState1DInstance::new(blend_state, artboard);
            blend_instance.advance(artboard, inputs, bindable_numbers, 0.0);
            self.current_blend_state_1d = Some(blend_instance);
            self.current_blend_state_direct = None;
            self.current_animation_keep_going = true;
            return;
        }
        if let Some(blend_state) = state.blend_state_direct.as_ref() {
            self.current_animation = None;
            self.current_blend_state_1d = None;
            let mut blend_instance = BlendStateDirectInstance::new(blend_state, artboard);
            blend_instance.advance(artboard, inputs, bindable_numbers, 0.0);
            self.current_blend_state_direct = Some(blend_instance);
            self.current_animation_keep_going = true;
            return;
        }
        self.current_blend_state_1d = None;
        self.current_blend_state_direct = None;
        let Some(animation_index) = state.animation_index else {
            self.current_animation = None;
            self.current_animation_keep_going = false;
            return;
        };
        let Some(animation) = artboard.linear_animation(animation_index) else {
            self.current_animation = None;
            self.current_animation_keep_going = false;
            return;
        };
        let mut animation_instance =
            LinearAnimationInstance::new(animation_index, animation, state.speed);
        self.current_animation_keep_going = animation_instance.advance(animation, 0.0);
        self.current_animation = Some(animation_instance);
    }

    fn advance_current_animation(
        &mut self,
        artboard: &ArtboardInstance,
        layer: &RuntimeStateMachineLayer,
        elapsed_seconds: f32,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
    ) -> bool {
        if let Some(blend_state) = self.current_blend_state_1d.as_mut() {
            self.current_animation_keep_going =
                blend_state.advance(artboard, inputs, bindable_numbers, elapsed_seconds);
            return self.current_animation_keep_going;
        }
        if let Some(blend_state) = self.current_blend_state_direct.as_mut() {
            self.current_animation_keep_going =
                blend_state.advance(artboard, inputs, bindable_numbers, elapsed_seconds);
            return self.current_animation_keep_going;
        }
        let Some(animation_instance) = self.current_animation.as_mut() else {
            self.current_animation_keep_going = false;
            return false;
        };
        let Some(state) = self
            .current_state_index
            .and_then(|index| layer.states.get(index))
        else {
            self.current_animation_keep_going = false;
            return false;
        };
        let Some(animation) = artboard.linear_animation(animation_instance.animation_index) else {
            self.current_animation_keep_going = false;
            return false;
        };
        self.current_animation_keep_going =
            animation_instance.advance(animation, elapsed_seconds * state.speed);
        self.current_animation_keep_going
    }

    fn advance_transition_source_animation(
        &mut self,
        artboard: &ArtboardInstance,
        layer: &RuntimeStateMachineLayer,
        elapsed_seconds: f32,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
    ) -> bool {
        if !self.is_transitioning() {
            return false;
        }
        if self.transition_source_paused {
            return false;
        }
        if let Some(blend_state) = self.transition_source_blend_state_1d.as_mut() {
            return blend_state.advance(artboard, inputs, bindable_numbers, elapsed_seconds);
        }
        if let Some(blend_state) = self.transition_source_blend_state_direct.as_mut() {
            return blend_state.advance(artboard, inputs, bindable_numbers, elapsed_seconds);
        }
        let Some(animation_instance) = self.transition_source_animation.as_mut() else {
            return false;
        };
        let Some(state) = self
            .transition_source_state_index
            .and_then(|index| layer.states.get(index))
        else {
            return false;
        };
        let Some(animation) = artboard.linear_animation(animation_instance.animation_index) else {
            return false;
        };
        animation_instance.advance(animation, elapsed_seconds * state.speed)
    }

    fn apply_animations(&self, artboard: &mut ArtboardInstance) -> bool {
        let mut changed = false;
        if self.is_transitioning() {
            if let Some(reset) = self.transition_animation_reset.as_ref() {
                changed |= reset.apply(artboard);
            }
            let mix_from = self
                .transition_interpolator
                .map(|interpolator| interpolator.transform(self.transition_mix_from))
                .unwrap_or(self.transition_mix_from);
            if let Some(source_animation) = self.transition_source_animation.as_ref() {
                changed |= artboard.apply_linear_animation_instance(source_animation, mix_from);
            }
            if let Some(source_blend_state) = self.transition_source_blend_state_1d.as_ref() {
                changed |= source_blend_state.apply(artboard, mix_from);
            }
            if let Some(source_blend_state) = self.transition_source_blend_state_direct.as_ref() {
                changed |= source_blend_state.apply(artboard, mix_from);
            }
        }
        let Some(animation_instance) = self.current_animation.as_ref() else {
            if let Some(blend_state) = self.current_blend_state_1d.as_ref() {
                let mix = if self.has_transition_source() {
                    self.transition_interpolator
                        .map(|interpolator| interpolator.transform(self.transition_mix))
                        .unwrap_or(self.transition_mix)
                } else {
                    1.0
                };
                return changed | blend_state.apply(artboard, mix);
            }
            if let Some(blend_state) = self.current_blend_state_direct.as_ref() {
                let mix = if self.has_transition_source() {
                    self.transition_interpolator
                        .map(|interpolator| interpolator.transform(self.transition_mix))
                        .unwrap_or(self.transition_mix)
                } else {
                    1.0
                };
                return changed | blend_state.apply(artboard, mix);
            }
            return changed;
        };
        let mix = if self.has_transition_source() {
            self.transition_interpolator
                .map(|interpolator| interpolator.transform(self.transition_mix))
                .unwrap_or(self.transition_mix)
        } else {
            1.0
        };
        changed | artboard.apply_linear_animation_instance(animation_instance, mix)
    }
}

#[derive(Debug, Clone)]
struct BlendState1DInstance {
    source: RuntimeBlendState1DSource,
    animations: Vec<BlendAnimation1DInstance>,
}

impl BlendState1DInstance {
    fn new(blend_state: &RuntimeBlendState1D, artboard: &ArtboardInstance) -> Self {
        let animations = blend_state
            .animations
            .iter()
            .filter_map(|animation| {
                let linear_animation = artboard.linear_animation(animation.animation_index)?;
                Some(BlendAnimation1DInstance {
                    value: animation.value,
                    animation: LinearAnimationInstance::new(
                        animation.animation_index,
                        linear_animation,
                        1.0,
                    ),
                    mix: 0.0,
                })
            })
            .collect();

        Self {
            source: blend_state.source.clone(),
            animations,
        }
    }

    fn advance(
        &mut self,
        artboard: &ArtboardInstance,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        elapsed_seconds: f32,
    ) -> bool {
        for animation in &mut self.animations {
            if artboard.linear_animation_instance_keep_going(&animation.animation) {
                artboard
                    .advance_linear_animation_instance(&mut animation.animation, elapsed_seconds);
            }
        }

        self.update_mix_values(inputs, bindable_numbers);
        true
    }

    fn update_mix_values(
        &mut self,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
    ) {
        if self.animations.is_empty() {
            return;
        }

        let value = match self.source {
            RuntimeBlendState1DSource::Input { input_index } => input_index
                .and_then(|input_index| inputs.get(input_index))
                .and_then(StateMachineInputInstance::number_value)
                .unwrap_or(0.0),
            RuntimeBlendState1DSource::BindableProperty { global_id } => {
                bindable_number_value(bindable_numbers, global_id).unwrap_or(0.0)
            }
        };

        let animation_count = self.animations.len();
        let to_index = self.animation_index(value);
        let from_index = to_index.checked_sub(1);
        let to_value = self
            .animations
            .get(to_index)
            .map(|animation| animation.value)
            .unwrap_or(0.0);
        let from_value = from_index
            .and_then(|index| self.animations.get(index))
            .map(|animation| animation.value)
            .unwrap_or(0.0);
        let (mix, mix_from) =
            if to_index >= animation_count || from_index.is_none() || to_value == from_value {
                (1.0, 1.0)
            } else {
                let mix = (value - from_value) / (to_value - from_value);
                (mix, 1.0 - mix)
            };

        for animation in &mut self.animations {
            if to_index < animation_count && animation.value == to_value {
                animation.mix = mix;
            } else if from_index.is_some() && animation.value == from_value {
                animation.mix = mix_from;
            } else {
                animation.mix = 0.0;
            }
        }
    }

    fn animation_index(&self, value: f32) -> usize {
        let mut index = 0_usize;
        let mut start = 0_isize;
        let mut end = self.animations.len() as isize - 1;

        while start <= end {
            let mid = (start + end) >> 1;
            let closest_value = self.animations[mid as usize].value;
            if closest_value < value {
                start = mid + 1;
            } else if closest_value > value {
                end = mid - 1;
            } else {
                index = mid as usize;
                break;
            }

            index = start as usize;
        }

        index
    }

    fn animation_instance(&self, index: usize) -> Option<&LinearAnimationInstance> {
        self.animations
            .get(index)
            .map(|animation| &animation.animation)
    }

    fn apply(&self, artboard: &mut ArtboardInstance, mix: f32) -> bool {
        let mut changed = false;
        for animation in &self.animations {
            let animation_mix = mix * animation.mix;
            if animation_mix == 0.0 {
                continue;
            }
            changed |=
                artboard.apply_linear_animation_instance(&animation.animation, animation_mix);
        }
        changed
    }
}

#[derive(Debug, Clone)]
struct BlendAnimation1DInstance {
    value: f32,
    animation: LinearAnimationInstance,
    mix: f32,
}

#[derive(Debug, Clone)]
struct BlendStateDirectInstance {
    animations: Vec<BlendAnimationDirectInstance>,
}

impl BlendStateDirectInstance {
    fn new(blend_state: &RuntimeBlendStateDirect, artboard: &ArtboardInstance) -> Self {
        let animations = blend_state
            .animations
            .iter()
            .filter_map(|animation| {
                let linear_animation = artboard.linear_animation(animation.animation_index)?;
                Some(BlendAnimationDirectInstance {
                    source: animation.source.clone(),
                    animation: LinearAnimationInstance::new(
                        animation.animation_index,
                        linear_animation,
                        1.0,
                    ),
                    mix: 0.0,
                })
            })
            .collect();

        Self { animations }
    }

    fn advance(
        &mut self,
        artboard: &ArtboardInstance,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        elapsed_seconds: f32,
    ) -> bool {
        for animation in &mut self.animations {
            if artboard.linear_animation_instance_keep_going(&animation.animation) {
                artboard
                    .advance_linear_animation_instance(&mut animation.animation, elapsed_seconds);
            }
        }

        self.update_mix_values(inputs, bindable_numbers);
        true
    }

    fn update_mix_values(
        &mut self,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
    ) {
        for animation in &mut self.animations {
            let value = match animation.source {
                RuntimeDirectBlendSource::Input { input_index } => inputs
                    .get(input_index)
                    .and_then(StateMachineInputInstance::number_value)
                    .unwrap_or(0.0),
                RuntimeDirectBlendSource::MixValue { value } => value,
                RuntimeDirectBlendSource::BindableProperty { global_id } => {
                    bindable_number_value(bindable_numbers, global_id).unwrap_or(0.0)
                }
            };
            animation.mix = (value / 100.0).clamp(0.0, 1.0);
        }
    }

    fn animation_instance(&self, index: usize) -> Option<&LinearAnimationInstance> {
        self.animations
            .get(index)
            .map(|animation| &animation.animation)
    }

    fn apply(&self, artboard: &mut ArtboardInstance, mix: f32) -> bool {
        let mut changed = false;
        for animation in &self.animations {
            let animation_mix = mix * animation.mix;
            if animation_mix == 0.0 {
                continue;
            }
            changed |=
                artboard.apply_linear_animation_instance(&animation.animation, animation_mix);
        }
        changed
    }
}

#[derive(Debug, Clone)]
struct BlendAnimationDirectInstance {
    source: RuntimeDirectBlendSource,
    animation: LinearAnimationInstance,
    mix: f32,
}

#[derive(Debug, Clone)]
pub struct RuntimeKeyedObject {
    pub global_id: u32,
    pub object_id: usize,
    pub target_local_id: usize,
    pub keyed_properties: Vec<RuntimeKeyedProperty>,
}

#[derive(Debug, Clone)]
pub struct RuntimeKeyedProperty {
    pub global_id: u32,
    pub property_key: u16,
    pub transform_property: Option<TransformProperty>,
    pub key_frames: Vec<RuntimeKeyFrameDouble>,
}

impl RuntimeKeyedProperty {
    fn value_at(&self, seconds: f32, fps: u64, current: f32, mix: f32) -> Option<f32> {
        if self.key_frames.is_empty() {
            return None;
        }

        let idx = self.closest_frame_index(seconds, fps);
        let value = if idx == 0 {
            self.key_frames[0].value
        } else if idx < self.key_frames.len() {
            let from = &self.key_frames[idx - 1];
            let to = &self.key_frames[idx];
            if seconds == to.seconds(fps) {
                to.value
            } else if from.interpolation_type == 0 {
                from.value
            } else if from.interpolator_id.is_some() {
                return None;
            } else {
                let from_seconds = from.seconds(fps);
                let to_seconds = to.seconds(fps);
                let frame_mix = if to_seconds == from_seconds {
                    1.0
                } else {
                    (seconds - from_seconds) / (to_seconds - from_seconds)
                };
                from.value + (to.value - from.value) * frame_mix
            }
        } else {
            self.key_frames.last()?.value
        };

        Some(mix_value(current, value, mix))
    }

    fn closest_frame_index(&self, seconds: f32, fps: u64) -> usize {
        let last = self.key_frames.len() - 1;
        if seconds > self.key_frames[last].seconds(fps) {
            return self.key_frames.len();
        }

        let mut start = 0;
        let mut end = last;
        while start <= end {
            let mid = (start + end) >> 1;
            let closest = self.key_frames[mid].seconds(fps);
            if closest < seconds {
                start = mid + 1;
            } else if closest > seconds {
                if mid == 0 {
                    break;
                }
                end = mid - 1;
            } else {
                return mid;
            }
        }
        start
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeKeyFrameDouble {
    pub global_id: u32,
    pub frame: u64,
    pub interpolation_type: u64,
    pub interpolator_id: Option<u64>,
    pub value: f32,
}

impl RuntimeKeyFrameDouble {
    fn seconds(&self, fps: u64) -> f32 {
        if fps == 0 {
            return 0.0;
        }
        self.frame as f32 / fps as f32
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeComponent {
    pub local_id: usize,
    pub global_id: u32,
    pub type_name: &'static str,
    pub capabilities: RuntimeComponentCapabilities,
    pub parent_local: Option<usize>,
    pub dependent_locals: Vec<usize>,
    pub graph_order: usize,
    pub dirt: ComponentDirt,
    pub transform: TransformRuntimeState,
}

impl RuntimeComponent {
    fn from_graph_component(component: &ComponentNode, object: &RuntimeObject) -> Self {
        Self {
            local_id: component.local_id,
            global_id: component.global_id,
            type_name: component.type_name,
            capabilities: RuntimeComponentCapabilities {
                world_transform: component.capabilities.world_transform,
                transform: component.capabilities.transform,
            },
            parent_local: component.parent_local,
            dependent_locals: component.dependent_locals.clone(),
            graph_order: component.graph_order.unwrap_or(0),
            dirt: ComponentDirt::FILTHY,
            transform: TransformRuntimeState::from_object(object),
        }
    }

    pub fn is_collapsed(&self) -> bool {
        self.dirt.contains(ComponentDirt::COLLAPSED)
    }
}

fn build_linear_animations(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    slots: &[InstanceSlot],
) -> Vec<RuntimeLinearAnimation> {
    let Some((start, end)) = artboard_object_range(file, graph.global_id as usize) else {
        return Vec::new();
    };

    let mut animations = Vec::<RuntimeLinearAnimation>::new();
    let mut current_animation = None;
    let mut current_keyed_object = None;
    let mut current_keyed_property = None;

    for global_id in start..end {
        let Some(object) = file.object(global_id) else {
            continue;
        };
        if file.import_status(global_id) != Some(RuntimeImportStatus::Imported) {
            continue;
        }

        if object.type_name == "LinearAnimation" {
            animations.push(RuntimeLinearAnimation {
                global_id: global_id as u32,
                name: object.string_property("name").map(ToOwned::to_owned),
                fps: object.uint_property("fps").unwrap_or(60),
                duration: object.uint_property("duration").unwrap_or(60),
                speed: object.double_property("speed").unwrap_or(1.0),
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

        let Some(animation_index) = current_animation else {
            continue;
        };

        if object.type_name == "KeyedObject" {
            let Some((object_id, target_local_id, _target)) =
                keyed_object_target(file, slots, object)
            else {
                current_keyed_object = None;
                current_keyed_property = None;
                continue;
            };

            animations[animation_index]
                .keyed_objects
                .push(RuntimeKeyedObject {
                    global_id: global_id as u32,
                    object_id,
                    target_local_id,
                    keyed_properties: Vec::new(),
                });
            current_keyed_object = Some(animations[animation_index].keyed_objects.len() - 1);
            current_keyed_property = None;
            continue;
        }

        if object.type_name == "KeyedProperty" {
            let Some(keyed_object_index) = current_keyed_object else {
                continue;
            };
            let Some(property_key) = object
                .uint_property("propertyKey")
                .and_then(|key| u16::try_from(key).ok())
            else {
                current_keyed_property = None;
                continue;
            };
            let target_local_id =
                animations[animation_index].keyed_objects[keyed_object_index].target_local_id;
            let Some(target) = slots
                .get(target_local_id)
                .and_then(|slot| file.object(slot.source_global_id as usize))
            else {
                current_keyed_property = None;
                continue;
            };
            if !object_supports_property(target.type_key, property_key) {
                current_keyed_property = None;
                continue;
            }

            animations[animation_index].keyed_objects[keyed_object_index]
                .keyed_properties
                .push(RuntimeKeyedProperty {
                    global_id: global_id as u32,
                    property_key,
                    transform_property: transform_property_for_key(property_key),
                    key_frames: Vec::new(),
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

        if object.type_name == "KeyFrameDouble" {
            let Some((keyed_object_index, keyed_property_index)) = current_keyed_property else {
                continue;
            };
            animations[animation_index].keyed_objects[keyed_object_index].keyed_properties
                [keyed_property_index]
                .key_frames
                .push(RuntimeKeyFrameDouble {
                    global_id: global_id as u32,
                    frame: object.uint_property("frame").unwrap_or(0),
                    interpolation_type: object.uint_property("interpolationType").unwrap_or(0),
                    interpolator_id: normalized_interpolator_id(object),
                    value: object.double_property("value").unwrap_or(0.0),
                });
        }
    }

    animations
}

fn build_state_machines(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    linear_animations: &[RuntimeLinearAnimation],
) -> Vec<RuntimeStateMachine> {
    let Some(artboard_index) = artboard_index_for_graph(file, graph) else {
        return Vec::new();
    };
    let animation_index_by_global = linear_animations
        .iter()
        .enumerate()
        .map(|(index, animation)| (animation.global_id, index))
        .collect::<BTreeMap<_, _>>();

    file.artboard_state_machine_graphs(artboard_index)
        .into_iter()
        .map(|state_machine| {
            let bindable_numbers = runtime_bindable_numbers(file, &state_machine);
            RuntimeStateMachine {
                global_id: state_machine.object.id,
                name: state_machine
                    .object
                    .string_property("name")
                    .map(ToOwned::to_owned),
                inputs: state_machine
                    .inputs
                    .into_iter()
                    .filter_map(runtime_state_machine_input)
                    .collect(),
                bindable_numbers,
                layers: state_machine
                    .layers
                    .into_iter()
                    .map(|layer| {
                        let states = layer
                            .states
                            .into_iter()
                            .map(|state| {
                                let animation_index = state.animation.and_then(|animation| {
                                    animation_index_by_global.get(&animation.id).copied()
                                });
                                let blend_state_1d = RuntimeBlendState1D::from_imported(
                                    file,
                                    &state,
                                    &animation_index_by_global,
                                );
                                let blend_state_direct = RuntimeBlendStateDirect::from_imported(
                                    file,
                                    &state,
                                    &animation_index_by_global,
                                );
                                RuntimeLayerState {
                                    global_id: state.object.map(|object| object.id),
                                    type_name: state.object.map(|object| object.type_name),
                                    animation_index,
                                    blend_state_1d,
                                    blend_state_direct,
                                    speed: state
                                        .object
                                        .and_then(|object| object.double_property("speed"))
                                        .unwrap_or(1.0),
                                    flags: state
                                        .object
                                        .and_then(|object| object.uint_property("flags"))
                                        .unwrap_or(0),
                                    fire_actions: state
                                        .fire_actions
                                        .iter()
                                        .filter_map(RuntimeStateMachineFireEvent::from_imported)
                                        .collect(),
                                    listener_actions: state
                                        .listener_actions
                                        .iter()
                                        .filter_map(RuntimeScheduledListenerAction::from_imported)
                                        .collect(),
                                    transitions: state
                                        .transitions
                                        .into_iter()
                                        .map(|transition| {
                                            let interpolator = transition.interpolator.and_then(
                                                RuntimeTransitionInterpolator::from_object,
                                            );
                                            RuntimeStateTransition {
                                                state_to_index: transition.state_to_index,
                                                exit_blend_animation_index: transition
                                                    .exit_blend_animation_index,
                                                duration: transition
                                                    .object
                                                    .uint_property("duration")
                                                    .unwrap_or(0),
                                                exit_time: transition
                                                    .object
                                                    .uint_property("exitTime")
                                                    .unwrap_or(0),
                                                flags: transition
                                                    .object
                                                    .uint_property("flags")
                                                    .unwrap_or(0),
                                                random_weight: transition
                                                    .object
                                                    .uint_property("randomWeight")
                                                    .unwrap_or(1),
                                                condition_count: transition.conditions.len(),
                                                conditions: transition
                                                    .conditions
                                                    .iter()
                                                    .filter_map(|condition| {
                                                        RuntimeTransitionCondition::from_object(
                                                            file,
                                                            condition,
                                                        )
                                                    })
                                                    .collect(),
                                                fire_actions: transition
                                                    .fire_actions
                                                    .iter()
                                                    .filter_map(
                                                        RuntimeStateMachineFireEvent::from_imported,
                                                    )
                                                    .collect(),
                                                listener_actions: transition
                                                    .listener_actions
                                                    .iter()
                                                    .filter_map(
                                                        RuntimeScheduledListenerAction::from_imported,
                                                    )
                                                    .collect(),
                                                interpolator,
                                                has_unsupported_interpolator: transition
                                                    .interpolator
                                                    .is_some()
                                                    && interpolator.is_none(),
                                            }
                                        })
                                        .collect(),
                                }
                            })
                            .collect::<Vec<_>>();
                        let entry_state_index = states
                            .iter()
                            .position(|state| state.type_name == Some("EntryState"));
                        let any_state_index = states
                            .iter()
                            .position(|state| state.type_name == Some("AnyState"));
                        RuntimeStateMachineLayer {
                            global_id: layer.object.id,
                            name: layer.object.string_property("name").map(ToOwned::to_owned),
                            states,
                            entry_state_index,
                            any_state_index,
                        }
                    })
                    .collect(),
            }
        })
        .collect()
}

fn runtime_bindable_numbers(
    file: &RuntimeFile,
    state_machine: &rive_binary::RuntimeStateMachine<'_>,
) -> Vec<RuntimeBindableNumber> {
    let mut values = BTreeMap::<u32, RuntimeBindableNumber>::new();
    for (data_bind_index, data_bind) in state_machine.data_binds.iter().enumerate() {
        let Some(target) = file.data_bind_target_for_object(data_bind) else {
            continue;
        };
        if target.type_name != "BindablePropertyNumber" {
            continue;
        }
        values
            .entry(target.id)
            .and_modify(|bindable_number| bindable_number.data_bind_indices.push(data_bind_index))
            .or_insert_with(|| RuntimeBindableNumber {
                global_id: target.id,
                data_bind_indices: vec![data_bind_index],
                value: target.double_property("propertyValue").unwrap_or(0.0),
            });
    }

    values.into_values().collect()
}

fn runtime_state_machine_input(object: &RuntimeObject) -> Option<RuntimeStateMachineInput> {
    let (kind, value) = match object.type_name {
        "StateMachineBool" => (
            StateMachineInputKind::Bool,
            StateMachineInputValue::Bool(object.bool_property("value").unwrap_or(false)),
        ),
        "StateMachineNumber" => (
            StateMachineInputKind::Number,
            StateMachineInputValue::Number(object.double_property("value").unwrap_or(0.0)),
        ),
        "StateMachineTrigger" => (
            StateMachineInputKind::Trigger,
            StateMachineInputValue::Trigger {
                fired: false,
                used_layers: Vec::new(),
            },
        ),
        _ => return None,
    };

    Some(RuntimeStateMachineInput {
        global_id: object.id,
        name: object.string_property("name").map(ToOwned::to_owned),
        kind,
        value,
    })
}

fn artboard_index_for_graph(file: &RuntimeFile, graph: &ArtboardGraph) -> Option<usize> {
    file.artboards()
        .into_iter()
        .position(|artboard| artboard.id == graph.global_id)
}

fn artboard_object_range(file: &RuntimeFile, start: usize) -> Option<(usize, usize)> {
    let artboard = file.object(start)?;
    if artboard.type_name != "Artboard" {
        return None;
    }
    let end = ((start + 1)..file.objects.len())
        .find(|index| {
            file.object(*index)
                .is_some_and(|object| object.type_name == "Artboard")
        })
        .unwrap_or(file.objects.len());
    Some((start, end))
}

fn keyed_object_target<'a>(
    file: &'a RuntimeFile,
    slots: &[InstanceSlot],
    keyed_object: &RuntimeObject,
) -> Option<(usize, usize, &'a RuntimeObject)> {
    let object_id = usize::try_from(keyed_object.uint_property("objectId")?).ok()?;
    let slot = slots.get(object_id)?;
    let target = file.object(slot.source_global_id as usize)?;
    Some((object_id, slot.local_id, target))
}

fn transform_property_for_key(property_key: u16) -> Option<TransformProperty> {
    [
        ("Node", "x", TransformProperty::X),
        ("Node", "y", TransformProperty::Y),
        ("Node", "rotation", TransformProperty::Rotation),
        ("Node", "scaleX", TransformProperty::ScaleX),
        ("Node", "scaleY", TransformProperty::ScaleY),
        ("Node", "opacity", TransformProperty::Opacity),
    ]
    .into_iter()
    .find_map(|(type_name, property_name, property)| {
        (property_key_for_name(type_name, property_name) == Some(property_key)).then_some(property)
    })
}

fn property_key_for_name(type_name: &str, property_name: &str) -> Option<u16> {
    let definition = definition_by_name(type_name)?;
    if let Some(property) = definition
        .properties
        .iter()
        .find(|property| property.name == property_name)
    {
        return Some(property.key.int);
    }

    for ancestor in definition.ancestors {
        let ancestor = definition_by_name(ancestor)?;
        if let Some(property) = ancestor
            .properties
            .iter()
            .find(|property| property.name == property_name)
        {
            return Some(property.key.int);
        }
    }

    None
}

fn mix_value(current: f32, value: f32, mix: f32) -> f32 {
    if mix == 1.0 {
        value
    } else {
        current * (1.0 - mix) + value * mix
    }
}

fn normalized_interpolator_id(object: &RuntimeObject) -> Option<u64> {
    object
        .uint_property("interpolatorId")
        .filter(|id| *id != u64::from(u32::MAX) && *id != u64::MAX)
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

#[derive(Debug, Clone, Copy, Default)]
pub struct RuntimeComponentCapabilities {
    pub world_transform: bool,
    pub transform: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransformRuntimeState {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub opacity: f32,
    pub local_transform: Mat2D,
    pub world_transform: Mat2D,
    pub render_opacity: f32,
}

impl TransformRuntimeState {
    fn from_object(object: &RuntimeObject) -> Self {
        Self {
            x: object.double_property("x").unwrap_or(0.0),
            y: object.double_property("y").unwrap_or(0.0),
            rotation: object.double_property("rotation").unwrap_or(0.0),
            scale_x: object.double_property("scaleX").unwrap_or(1.0),
            scale_y: object.double_property("scaleY").unwrap_or(1.0),
            opacity: object.double_property("opacity").unwrap_or(1.0),
            local_transform: Mat2D::IDENTITY,
            world_transform: Mat2D::IDENTITY,
            render_opacity: 0.0,
        }
    }

    fn property(&self, property: TransformProperty) -> f32 {
        match property {
            TransformProperty::X => self.x,
            TransformProperty::Y => self.y,
            TransformProperty::Rotation => self.rotation,
            TransformProperty::ScaleX => self.scale_x,
            TransformProperty::ScaleY => self.scale_y,
            TransformProperty::Opacity => self.opacity,
        }
    }

    fn set_property(&mut self, property: TransformProperty, value: f32) -> bool {
        let current = match property {
            TransformProperty::X => &mut self.x,
            TransformProperty::Y => &mut self.y,
            TransformProperty::Rotation => &mut self.rotation,
            TransformProperty::ScaleX => &mut self.scale_x,
            TransformProperty::ScaleY => &mut self.scale_y,
            TransformProperty::Opacity => &mut self.opacity,
        };
        if *current == value {
            return false;
        }
        *current = value;
        true
    }
}

impl Default for TransformRuntimeState {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            opacity: 1.0,
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
            a[0] * b[0] + a[2] * b[1],
            a[1] * b[0] + a[3] * b[1],
            a[0] * b[2] + a[2] * b[3],
            a[1] * b[2] + a[3] * b[3],
            a[0] * b[4] + a[2] * b[5] + a[4],
            a[1] * b[4] + a[3] * b[5] + a[5],
        ])
    }

    pub fn scale_by_values(&mut self, scale_x: f32, scale_y: f32) {
        self.0[0] *= scale_x;
        self.0[1] *= scale_x;
        self.0[2] *= scale_y;
        self.0[3] *= scale_y;
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
        Self([
            d / determinant,
            -b / determinant,
            -c / determinant,
            a / determinant,
            (c * f - d * e) / determinant,
            (b * e - a * f) / determinant,
        ])
    }

    pub fn transform_point(self, x: f32, y: f32) -> (f32, f32) {
        (
            self.0[0] * x + self.0[2] * y + self.0[4],
            self.0[1] * x + self.0[3] * y + self.0[5],
        )
    }
}

impl Default for Mat2D {
    fn default() -> Self {
        Self::IDENTITY
    }
}

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

impl RuntimeComponent {
    fn update_transform(&mut self) {
        if !self.capabilities.transform {
            return;
        }

        let mut transform = Mat2D::from_rotation(self.transform.rotation);
        transform.0[4] = self.transform.x;
        transform.0[5] = self.transform.y;
        transform.scale_by_values(self.transform.scale_x, self.transform.scale_y);
        self.transform.local_transform = transform;
    }

    fn update_world_transform(&mut self, parent_world: Option<Mat2D>) {
        if self.type_name == "Artboard" || !self.capabilities.transform {
            return;
        }

        self.transform.world_transform = match parent_world {
            Some(parent_world) => parent_world.multiply(self.transform.local_transform),
            None => self.transform.local_transform,
        };
    }

    fn update_render_opacity(&mut self, parent_opacity: f32) {
        if !self.capabilities.transform {
            return;
        }

        self.transform.render_opacity = self.transform.opacity * parent_opacity;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rive_binary::read_runtime_file;
    use rive_graph::GraphFile;

    fn synthetic_instance(
        components: Vec<RuntimeComponent>,
        update_order: Vec<usize>,
    ) -> ArtboardInstance {
        let component_by_local = components
            .iter()
            .enumerate()
            .map(|(index, component)| (component.local_id, index))
            .collect::<BTreeMap<_, _>>();

        ArtboardInstance {
            slots: components
                .iter()
                .enumerate()
                .map(|(index, component)| InstanceSlot {
                    local_id: component.local_id,
                    source_global_id: component.global_id,
                    type_name: Some(component.type_name),
                    name: None,
                    component_index: Some(index),
                })
                .collect(),
            components,
            component_by_local,
            update_order,
            linear_animations: Vec::new(),
            state_machines: Vec::new(),
            dirt: ComponentDirt::COMPONENTS,
            dirt_depth: 0,
            did_change: true,
        }
    }

    fn synthetic_component(local_id: usize, graph_order: usize) -> RuntimeComponent {
        RuntimeComponent {
            local_id,
            global_id: local_id as u32,
            type_name: "Node",
            capabilities: RuntimeComponentCapabilities {
                world_transform: true,
                transform: true,
            },
            parent_local: None,
            dependent_locals: Vec::new(),
            graph_order,
            dirt: ComponentDirt::NONE,
            transform: TransformRuntimeState::default(),
        }
    }

    #[test]
    fn component_dirt_bits_match_cpp_layout() {
        assert_eq!(ComponentDirt::NONE.0, 0);
        assert_eq!(ComponentDirt::COLLAPSED.0, 1 << 0);
        assert_eq!(ComponentDirt::DEPENDENTS.0, 1 << 1);
        assert_eq!(ComponentDirt::COMPONENTS.0, 1 << 2);
        assert_eq!(ComponentDirt::DRAW_ORDER.0, 1 << 3);
        assert_eq!(ComponentDirt::PATH.0, 1 << 4);
        assert_eq!(ComponentDirt::TEXT_SHAPE.0, ComponentDirt::PATH.0);
        assert_eq!(ComponentDirt::SKIN.0, ComponentDirt::PATH.0);
        assert_eq!(ComponentDirt::VERTICES.0, 1 << 5);
        assert_eq!(ComponentDirt::TEXT_COVERAGE.0, ComponentDirt::VERTICES.0);
        assert_eq!(ComponentDirt::TRANSFORM.0, 1 << 6);
        assert_eq!(ComponentDirt::WORLD_TRANSFORM.0, 1 << 7);
        assert_eq!(ComponentDirt::RENDER_OPACITY.0, 1 << 8);
        assert_eq!(ComponentDirt::PAINT.0, 1 << 9);
        assert_eq!(ComponentDirt::STOPS.0, 1 << 10);
        assert_eq!(ComponentDirt::LAYOUT_STYLE.0, 1 << 11);
        assert_eq!(ComponentDirt::BINDINGS.0, 1 << 12);
        assert_eq!(ComponentDirt::N_SLICER.0, 1 << 13);
        assert_eq!(ComponentDirt::SCRIPT_UPDATE.0, 1 << 14);
        assert_eq!(ComponentDirt::CLIPPING.0, 1 << 15);
        assert_eq!(ComponentDirt::FILTHY.0, 0xFFFE);
    }

    #[test]
    fn add_dirt_recurses_to_graph_dependents() {
        let mut source = synthetic_component(0, 0);
        source.dependent_locals.push(1);
        let dependent = synthetic_component(1, 1);
        let mut instance = synthetic_instance(vec![source, dependent], vec![0, 1]);

        assert!(instance.add_dirt(0, ComponentDirt::PATH, true));
        assert!(
            instance
                .component(0)
                .unwrap()
                .dirt
                .contains(ComponentDirt::PATH)
        );
        assert!(
            instance
                .component(1)
                .unwrap()
                .dirt
                .contains(ComponentDirt::PATH)
        );
        assert!(instance.has_dirt(ComponentDirt::COMPONENTS));

        assert!(!instance.add_dirt(0, ComponentDirt::PATH, true));
    }

    #[test]
    fn update_components_skips_collapsed_components_without_clearing_dirt() {
        let mut first = synthetic_component(0, 0);
        first.dirt = ComponentDirt::PATH;
        let mut second = synthetic_component(1, 1);
        second.dirt = ComponentDirt::PATH | ComponentDirt::COLLAPSED;
        let mut instance = synthetic_instance(vec![first, second], vec![0, 1]);

        let report = instance.update_components();

        assert!(report.did_update);
        assert_eq!(report.updated_locals, vec![0]);
        assert_eq!(instance.component(0).unwrap().dirt, ComponentDirt::NONE);
        assert!(
            instance
                .component(1)
                .unwrap()
                .dirt
                .contains(ComponentDirt::PATH)
        );
        assert!(instance.component(1).unwrap().is_collapsed());
    }

    #[test]
    fn update_components_restarts_when_update_dirties_earlier_graph_order() {
        let first = synthetic_component(0, 0);
        let mut second = synthetic_component(1, 1);
        second.dirt = ComponentDirt::PATH;
        let mut instance = synthetic_instance(vec![first, second], vec![0, 1]);
        let mut dirtied_earlier = false;

        let report = instance.update_components_with_hook(|instance, local_id, _| {
            if local_id == 1 && !dirtied_earlier {
                dirtied_earlier = true;
                instance.add_dirt(0, ComponentDirt::PATH, false);
            }
        });

        assert_eq!(report.steps, 2);
        assert_eq!(report.updated_locals, vec![1, 0]);
        assert!(!report.max_steps_reached);
    }

    #[test]
    fn update_components_surfaces_cpp_max_pass_guard() {
        let mut component = synthetic_component(0, 0);
        component.dirt = ComponentDirt::PATH;
        let mut instance = synthetic_instance(vec![component], vec![0]);

        let report = instance.update_components_with_hook(|instance, local_id, _| {
            instance.add_dirt(local_id, ComponentDirt::PATH, false);
        });

        assert_eq!(report.steps, 100);
        assert_eq!(report.updated_locals.len(), 100);
        assert!(report.max_steps_reached);
        assert!(instance.has_dirt(ComponentDirt::COMPONENTS));
    }

    #[test]
    fn transform_update_matches_basic_cpp_order() {
        let mut root = synthetic_component(0, 0);
        root.type_name = "Artboard";
        root.transform.render_opacity = 0.5;
        let mut child = synthetic_component(1, 1);
        child.parent_local = Some(0);
        child.transform.x = 2.0;
        child.transform.y = 3.0;
        child.transform.scale_x = 4.0;
        child.transform.scale_y = 5.0;
        child.transform.opacity = 0.25;
        child.dirt = ComponentDirt::TRANSFORM
            | ComponentDirt::WORLD_TRANSFORM
            | ComponentDirt::RENDER_OPACITY;
        let mut instance = synthetic_instance(vec![root, child], vec![0, 1]);

        let report = instance.update_components();

        assert_eq!(report.updated_locals, vec![1]);
        let child = instance.component(1).unwrap();
        assert_eq!(
            child.transform.local_transform,
            Mat2D([4.0, 0.0, -0.0, 5.0, 2.0, 3.0])
        );
        assert_eq!(
            child.transform.world_transform,
            child.transform.local_transform
        );
        assert_eq!(child.transform.render_opacity, 0.125);
    }

    #[test]
    fn transform_property_mutation_marks_instance_dirty() {
        let mut component = synthetic_component(0, 0);
        component.dirt = ComponentDirt::NONE;
        let mut instance = synthetic_instance(vec![component], vec![0]);
        instance.dirt = ComponentDirt::NONE;
        instance.did_change = false;

        assert!(instance.set_transform_property(0, TransformProperty::X, 12.0));
        let component = instance.component(0).unwrap();
        assert_eq!(component.transform.x, 12.0);
        assert!(
            component
                .dirt
                .contains(ComponentDirt::TRANSFORM | ComponentDirt::WORLD_TRANSFORM)
        );
        assert!(instance.has_dirt(ComponentDirt::COMPONENTS));
        assert!(instance.did_change());

        assert!(!instance.set_transform_property(0, TransformProperty::X, 12.0));
    }

    #[test]
    fn transform_property_mutation_only_recurses_world_transform_to_dependents() {
        let mut source = synthetic_component(0, 0);
        source.dependent_locals.push(1);
        let dependent = synthetic_component(1, 1);
        let mut instance = synthetic_instance(vec![source, dependent], vec![0, 1]);
        instance.clear_component_dirt(0);
        instance.clear_component_dirt(1);
        instance.dirt = ComponentDirt::NONE;

        assert!(instance.set_transform_property(0, TransformProperty::X, 12.0));

        let source = instance.component(0).unwrap();
        assert!(source.dirt.contains(ComponentDirt::TRANSFORM));
        assert!(source.dirt.contains(ComponentDirt::WORLD_TRANSFORM));
        let dependent = instance.component(1).unwrap();
        assert!(!dependent.dirt.contains(ComponentDirt::TRANSFORM));
        assert!(dependent.dirt.contains(ComponentDirt::WORLD_TRANSFORM));
    }

    #[test]
    fn opacity_mutation_marks_render_opacity_dirty() {
        let mut component = synthetic_component(0, 0);
        component.dirt = ComponentDirt::NONE;
        let mut instance = synthetic_instance(vec![component], vec![0]);
        instance.dirt = ComponentDirt::NONE;

        assert!(instance.set_transform_property(0, TransformProperty::Opacity, 0.35));
        let component = instance.component(0).unwrap();
        assert_eq!(component.transform.opacity, 0.35);
        assert!(component.dirt.contains(ComponentDirt::RENDER_OPACITY));
        assert!(!component.dirt.contains(ComponentDirt::TRANSFORM));
    }

    #[test]
    fn update_reads_mutated_instance_transform_state() {
        let mut component = synthetic_component(0, 0);
        component.dirt = ComponentDirt::NONE;
        let mut instance = synthetic_instance(vec![component], vec![0]);
        instance.dirt = ComponentDirt::NONE;

        assert!(instance.set_transform_property(0, TransformProperty::X, 9.0));
        assert!(instance.set_transform_property(0, TransformProperty::Y, 4.0));

        let report = instance.update_components();

        assert_eq!(report.updated_locals, vec![0]);
        assert_eq!(
            instance.component(0).unwrap().transform.local_transform,
            Mat2D([1.0, 0.0, -0.0, 1.0, 9.0, 4.0])
        );
    }

    #[test]
    fn builds_instance_from_graph_fixture() {
        let bytes = include_bytes!("../../../fixtures/graph/dependency_test.riv");
        let file = read_runtime_file(bytes).expect("fixture should import");
        let graph = GraphFile::from_runtime_file(&file).expect("fixture should graph");
        let artboard = graph.artboards.first().expect("fixture has artboard");
        let instance = ArtboardInstance::from_graph(&file, artboard).expect("instance builds");

        assert_eq!(instance.slots().len(), artboard.local_objects.len());
        assert_eq!(
            instance
                .slots()
                .iter()
                .map(|slot| (slot.local_id, slot.source_global_id, slot.type_name))
                .collect::<Vec<_>>(),
            artboard
                .local_objects
                .iter()
                .map(|object| (object.local_id, object.global_id, object.type_name))
                .collect::<Vec<_>>()
        );
        assert_eq!(instance.components().len(), artboard.components.len());
        let graph_ordered_components = artboard
            .components
            .iter()
            .filter(|component| component.graph_order.is_some())
            .count();
        assert_eq!(instance.update_order().len(), graph_ordered_components);
        assert!(instance.has_dirt(ComponentDirt::COMPONENTS));
        assert!(
            instance
                .components()
                .iter()
                .all(|component| component.dirt == ComponentDirt::FILTHY)
        );
    }
}
