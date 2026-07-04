use anyhow::{Context, Result};
use rive_binary::{
    RuntimeFile, RuntimeObject, RuntimeViewModel, RuntimeViewModelInstance,
    RuntimeViewModelInstanceReference,
};
use rive_graph::{
    ArtboardGraph, ClippingShapeNode, DashNode, DrawableOrderKind, FeatherNode, GradientStopNode,
    ParametricPathNode, PathComposerNode, PathComposerPathNode, PathGeometryNode, PathVertexNode,
    ShapePaintContainerNode, ShapePaintKind, ShapePaintNode, ShapePaintPathKind,
    ShapePaintStateNode, SortedDrawableNode, StrokeEffectNode,
};
use rive_render_api::{
    BlendMode as RenderBlendMode, Factory as RenderFactory, FillRule as RenderFillRule,
    Mat2D as RenderMat2D, RenderPaint, RenderPaintStyle, RenderPath, RenderShader, Renderer,
    StrokeCap as RenderStrokeCap, StrokeJoin as RenderStrokeJoin,
};
use rive_schema::{FieldKind, definition_by_name, definition_by_type_key};
use std::collections::{BTreeMap, BTreeSet, btree_map::Entry};

mod animation;
mod artboard_data_bind;
mod components;
mod data_bind_graph;
mod objects;
mod state_machine;
mod view_model;

pub use animation::{
    LinearAnimationInstance, RuntimeKeyFrameBool, RuntimeKeyFrameCallback, RuntimeKeyFrameColor,
    RuntimeKeyFrameDouble, RuntimeKeyFrameString, RuntimeKeyFrameUint, RuntimeKeyedObject,
    RuntimeKeyedProperty, RuntimeLinearAnimation,
};
use animation::{RuntimeJoystick, build_linear_animations, build_runtime_joysticks};
use artboard_data_bind::{
    RuntimeArtboardCustomPropertyBindingInstance, RuntimeArtboardListBindingInstance,
    RuntimeArtboardSoloBindingInstance, build_artboard_custom_property_bindings,
    build_artboard_default_view_model_values, build_artboard_list_bindings,
    build_artboard_solo_bindings,
};
use components::{
    AuthoredTransform, RuntimeSolo, apply_initial_solo_collapses, build_runtime_solos,
};
pub use components::{
    ComponentDirt, Mat2D, RuntimeComponent, RuntimeComponentCapabilities, TransformProperty,
    TransformRuntimeState, UpdateComponentsReport,
};
use data_bind_graph::{
    RuntimeDataBindGraph, RuntimeDataBindGraphApplyPhase, RuntimeDataBindGraphConverter,
    RuntimeDataBindGraphTargetsMut, RuntimeDataBindGraphValue,
    data_bind_flags_apply_source_to_target, data_bind_flags_apply_target_to_source,
};
use objects::InstanceObjectArena;
pub use objects::InstanceSlot;
pub use state_machine::{
    RuntimeLayerState, RuntimeStateMachine, RuntimeStateMachineInput, RuntimeStateMachineLayer,
    StateMachineInputInstance, StateMachineInputKind, StateMachineInstance,
    StateMachineReportedEvent,
};
use state_machine::{
    RuntimeTransitionInterpolator, StateMachineBindableArtboardInstance,
    StateMachineBindableAssetInstance, StateMachineBindableBooleanInstance,
    StateMachineBindableColorInstance, StateMachineBindableEnumInstance,
    StateMachineBindableIntegerInstance, StateMachineBindableListInstance,
    StateMachineBindableNumberInstance, StateMachineBindableStringInstance,
    StateMachineBindableTriggerInstance, StateMachineBindableViewModelInstance,
    build_state_machines,
};
pub use view_model::{
    RuntimeDefaultViewModelArtboardSourceHandle, RuntimeDefaultViewModelAssetSourceHandle,
    RuntimeDefaultViewModelBooleanSourceHandle, RuntimeDefaultViewModelColorSourceHandle,
    RuntimeDefaultViewModelEnumSourceHandle, RuntimeDefaultViewModelListSourceHandle,
    RuntimeDefaultViewModelNumberSourceHandle, RuntimeDefaultViewModelStringSourceHandle,
    RuntimeDefaultViewModelSymbolListIndexSourceHandle, RuntimeDefaultViewModelTriggerSourceHandle,
    RuntimeDefaultViewModelViewModelSourceHandle, RuntimeImportedViewModelArtboardSourceHandle,
    RuntimeImportedViewModelAssetSourceHandle, RuntimeImportedViewModelBooleanSourceHandle,
    RuntimeImportedViewModelColorSourceHandle, RuntimeImportedViewModelEnumSourceHandle,
    RuntimeImportedViewModelInstanceContext, RuntimeImportedViewModelListSourceHandle,
    RuntimeImportedViewModelNumberSourceHandle, RuntimeImportedViewModelStringSourceHandle,
    RuntimeImportedViewModelSymbolListIndexSourceHandle,
    RuntimeImportedViewModelTriggerSourceHandle, RuntimeImportedViewModelViewModelSourceHandle,
};

#[derive(Debug, Clone)]
pub struct ArtboardInstance {
    width: f32,
    height: f32,
    origin_x: f32,
    origin_y: f32,
    clip: bool,
    slots: Vec<InstanceSlot>,
    objects: InstanceObjectArena,
    components: Vec<RuntimeComponent>,
    component_by_local: BTreeMap<usize, usize>,
    solos: Vec<RuntimeSolo>,
    joysticks: Vec<RuntimeJoystick>,
    joysticks_apply_before_update: bool,
    update_order: Vec<usize>,
    linear_animations: Vec<RuntimeLinearAnimation>,
    state_machines: Vec<RuntimeStateMachine>,
    artboard_data_bind_values: BTreeMap<Vec<u32>, RuntimeDataBindGraphValue>,
    artboard_custom_property_bindings: Vec<RuntimeArtboardCustomPropertyBindingInstance>,
    artboard_solo_bindings: Vec<RuntimeArtboardSoloBindingInstance>,
    artboard_list_bindings: Vec<RuntimeArtboardListBindingInstance>,
    dirt: ComponentDirt,
    dirt_depth: usize,
    did_change: bool,
}

impl ArtboardInstance {
    pub fn from_graph(file: &RuntimeFile, graph: &ArtboardGraph) -> Result<Self> {
        let dimensions =
            RuntimeArtboardDimensions::from_object(file.object(graph.global_id as usize));
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
        let objects = InstanceObjectArena::from_slots(file, &slots);

        let mut component_by_local = BTreeMap::new();
        let mut components = Vec::new();

        for component in &graph.components {
            file.object(component.global_id as usize).with_context(|| {
                format!("component global id {} is missing", component.global_id)
            })?;

            component_by_local.insert(component.local_id, components.len());
            components.push(RuntimeComponent::from_graph_component(component));
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
        let solos = build_runtime_solos(file, graph);
        let linear_animations = build_linear_animations(file, graph, &slots);
        let joysticks = build_runtime_joysticks(graph, &linear_animations);
        let state_machines = build_state_machines(file, graph, &linear_animations);
        let artboard_data_bind_values = build_artboard_default_view_model_values(file, graph);
        let artboard_custom_property_bindings =
            build_artboard_custom_property_bindings(file, graph);
        let artboard_solo_bindings = build_artboard_solo_bindings(file, graph);
        let artboard_list_bindings = build_artboard_list_bindings(file, graph);
        apply_initial_solo_collapses(&objects, &solos, &mut components, &component_by_local);

        Ok(Self {
            width: dimensions.width,
            height: dimensions.height,
            origin_x: dimensions.origin_x,
            origin_y: dimensions.origin_y,
            clip: dimensions.clip,
            slots,
            objects,
            components,
            component_by_local,
            solos,
            joysticks,
            joysticks_apply_before_update: graph.joysticks_apply_before_update,
            update_order,
            linear_animations,
            state_machines,
            artboard_data_bind_values,
            artboard_custom_property_bindings,
            artboard_solo_bindings,
            artboard_list_bindings,
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

    pub fn draw_static_artboard(
        &self,
        runtime: &RuntimeFile,
        graph: &ArtboardGraph,
        factory: &mut dyn RenderFactory,
        renderer: &mut dyn Renderer,
        paint_by_global: &mut BTreeMap<u32, Box<dyn RenderPaint>>,
    ) -> Result<()> {
        let mut render_cache = RuntimeRenderPathCache::default();
        self.prepare_static_artboard_paints(
            runtime,
            graph,
            factory,
            paint_by_global,
            &mut render_cache,
        )?;
        self.draw_prepared_static_artboard_with_path_cache(
            runtime,
            graph,
            factory,
            renderer,
            paint_by_global,
            &mut render_cache,
        )
    }

    pub fn prepare_static_artboard_paints(
        &self,
        runtime: &RuntimeFile,
        graph: &ArtboardGraph,
        factory: &mut dyn RenderFactory,
        paint_by_global: &mut BTreeMap<u32, Box<dyn RenderPaint>>,
        render_cache: &mut RuntimeRenderPathCache,
    ) -> Result<()> {
        let mut paints_by_mutator =
            BTreeMap::<usize, Vec<(&ShapePaintContainerNode, &ShapePaintNode)>>::new();
        for container in &graph.shape_paint_containers {
            for paint in &container.paints {
                if !matches!(
                    paint.paint_state,
                    Some(
                        ShapePaintStateNode::LinearGradient { .. }
                            | ShapePaintStateNode::RadialGradient { .. }
                    )
                ) {
                    continue;
                }
                if let Some(mutator_local) = paint.mutator_local {
                    paints_by_mutator
                        .entry(mutator_local)
                        .or_default()
                        .push((container, paint));
                }
            }
        }

        let mut prepared = BTreeSet::new();
        // C++ Artboard::advance updates gradient mutators in dependency order before
        // drawing, and the recording factory observes shader creation there.
        for local_id in graph
            .dependency_order
            .iter()
            .copied()
            .chain(graph.local_objects.iter().map(|object| object.local_id))
        {
            let Some(paints) = paints_by_mutator.get(&local_id) else {
                continue;
            };
            for (container, paint) in paints {
                if !prepared.insert(paint.global_id) {
                    continue;
                }
                let object = runtime
                    .object(paint.global_id as usize)
                    .with_context(|| format!("missing paint global {}", paint.global_id))?;
                let runtime_paint = runtime_prepare_gradient_paint_command(self, container, paint);
                let mut gradient_resources = RuntimeGradientShaderResources {
                    factory,
                    shaders: &mut render_cache.gradient_shaders,
                };
                runtime_configure_paint(
                    paint_by_global
                        .get_mut(&paint.global_id)
                        .with_context(|| {
                            format!("missing render paint for global {}", paint.global_id)
                        })?
                        .as_mut(),
                    object,
                    &runtime_paint,
                    Some(&mut gradient_resources),
                )?;
            }
        }

        Ok(())
    }

    pub fn draw_prepared_static_artboard(
        &self,
        runtime: &RuntimeFile,
        graph: &ArtboardGraph,
        factory: &mut dyn RenderFactory,
        renderer: &mut dyn Renderer,
        paint_by_global: &mut BTreeMap<u32, Box<dyn RenderPaint>>,
    ) -> Result<()> {
        let mut path_cache = RuntimeRenderPathCache::default();
        self.draw_prepared_static_artboard_with_path_cache(
            runtime,
            graph,
            factory,
            renderer,
            paint_by_global,
            &mut path_cache,
        )
    }

    pub fn draw_prepared_static_artboard_with_path_cache(
        &self,
        runtime: &RuntimeFile,
        graph: &ArtboardGraph,
        factory: &mut dyn RenderFactory,
        renderer: &mut dyn Renderer,
        paint_by_global: &mut BTreeMap<u32, Box<dyn RenderPaint>>,
        path_cache: &mut RuntimeRenderPathCache,
    ) -> Result<()> {
        renderer.save();
        if self.clip {
            let clip = path_cache.artboard_clip_path(
                factory,
                &runtime_rect_commands(0.0, 0.0, self.width, self.height),
                RenderFillRule::Clockwise,
            );
            renderer.clip_path(clip.as_ref());
        }
        renderer.transform(self.artboard_origin_transform());

        if let Some(background) = graph
            .shape_paint_containers
            .iter()
            .find(|container| container.local_id == 0)
        {
            runtime_draw_background(
                runtime,
                self,
                background,
                self.width,
                self.height,
                self.origin_x,
                self.origin_y,
                factory,
                renderer,
                paint_by_global,
                path_cache,
            )?;
        }

        let local_to_global = graph
            .local_objects
            .iter()
            .map(|object| (object.local_id, object.global_id))
            .collect::<BTreeMap<_, _>>();

        for command in self.draw_commands(graph) {
            runtime_draw_command(
                runtime,
                self,
                graph,
                &local_to_global,
                command,
                factory,
                renderer,
                paint_by_global,
                path_cache,
            )?;
        }

        renderer.restore();
        Ok(())
    }

    fn artboard_origin_transform(&self) -> RenderMat2D {
        RenderMat2D([
            1.0,
            0.0,
            0.0,
            1.0,
            self.width * self.origin_x,
            self.height * self.origin_y,
        ])
    }

    fn runtime_will_draw(&self, drawable: &SortedDrawableNode) -> bool {
        match drawable.kind {
            DrawableOrderKind::ClipStartProxy | DrawableOrderKind::ClipEndProxy => true,
            DrawableOrderKind::Drawable | DrawableOrderKind::LayoutProxy => {
                if drawable.is_hidden {
                    return false;
                }
                if drawable
                    .local_id
                    .and_then(|local_id| self.component(local_id))
                    .is_some_and(RuntimeComponent::is_collapsed)
                {
                    return false;
                }

                if drawable.type_name == "Image" {
                    return drawable.resolved_image_asset_global.is_some()
                        && self
                            .runtime_drawable_render_opacity(drawable)
                            .is_some_and(|opacity| opacity != 0.0);
                }

                if sorted_drawable_is_nested_artboard(drawable.type_name) {
                    return drawable.referenced_artboard_global.is_some();
                }

                if sorted_drawable_uses_render_opacity(drawable.type_name) {
                    return self
                        .runtime_drawable_render_opacity(drawable)
                        .is_some_and(|opacity| opacity != 0.0);
                }

                true
            }
        }
    }

    fn runtime_drawable_render_opacity(&self, drawable: &SortedDrawableNode) -> Option<f32> {
        let local_id = drawable.local_id?;
        self.component(local_id)
            .map(|component| component.transform.render_opacity)
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
        let shape_world = self
            .component(shape_local)
            .map(|component| component.transform.world_transform)
            .unwrap_or(Mat2D::IDENTITY);

        container
            .paints
            .iter()
            .filter_map(|paint| {
                let path_commands =
                    self.runtime_shape_paint_path_commands(shape_local, paint, graph);
                runtime_shape_paint_command(
                    self,
                    paint,
                    container.blend_mode_value,
                    needs_save_operation,
                    render_opacity,
                    shape_world,
                    path_commands,
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
            let weighted_context = self.runtime_weighted_path_context(path, graph);
            let path_transform = if weighted_context.is_some() {
                Mat2D::IDENTITY
            } else {
                path_world
            };
            let transform = match path_kind {
                ShapePaintPathKind::World => path_transform,
                ShapePaintPathKind::Local | ShapePaintPathKind::LocalClockwise => {
                    if mat2d_has_visible_skew_or_rotation(shape_world)
                        || mat2d_has_visible_skew_or_rotation(path_transform)
                    {
                        inverse_shape_world.multiply_path_local_fused(path_transform)
                    } else {
                        inverse_shape_world.multiply(path_transform)
                    }
                }
            };

            let mut path_commands =
                path_commands(path, path_kind, transform, weighted_context.as_ref());
            prune_empty_path_segments(&mut path_commands);
            commands.extend(path_commands);
        }

        commands
    }

    fn runtime_clipping_shape_path_commands(
        &self,
        clipping_shape: &ClippingShapeNode,
        graph: &ArtboardGraph,
    ) -> Vec<RuntimePathCommand> {
        let mut commands = Vec::new();
        for shape_local in &clipping_shape.shape_locals {
            let Some(composer) = graph
                .path_composers
                .iter()
                .find(|composer| composer.shape_local == *shape_local)
            else {
                continue;
            };

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
                let weighted_context = self.runtime_weighted_path_context(path, graph);
                let path_transform = if weighted_context.is_some() {
                    Mat2D::IDENTITY
                } else {
                    path_world
                };
                let mut path_commands = path_commands(
                    path,
                    ShapePaintPathKind::World,
                    path_transform,
                    weighted_context.as_ref(),
                );
                prune_empty_path_segments(&mut path_commands);
                commands.extend(path_commands);
            }
        }

        commands
    }

    fn runtime_weighted_path_context(
        &self,
        path: &PathGeometryNode,
        graph: &ArtboardGraph,
    ) -> Option<WeightedPathContext> {
        let skin = graph
            .skeletal_skins
            .iter()
            .find(|skin| skin.skinnable_local == Some(path.local_id))?;
        let mut bone_transforms = vec![Mat2D::IDENTITY];
        for tendon in &skin.tendons {
            let bone = self.component(tendon.bone_local?)?;
            bone_transforms.push(
                bone.transform
                    .world_transform
                    .multiply(Mat2D(tendon.inverse_bind)),
            );
        }

        Some(WeightedPathContext {
            skin_world_transform: Mat2D(skin.world_transform),
            bone_transforms,
        })
    }

    pub fn state_machine(&self, index: usize) -> Option<&RuntimeStateMachine> {
        self.state_machines.get(index)
    }

    pub fn state_machines(&self) -> &[RuntimeStateMachine] {
        &self.state_machines
    }

    pub fn set_artboard_dimensions(&mut self, width: f32, height: f32) -> bool {
        if self.width == width && self.height == height {
            return false;
        }
        self.width = width;
        self.height = height;
        self.did_change = true;
        true
    }

    fn artboard_property_value(&self, property_type: u64) -> f32 {
        match property_type {
            0 => self.width,
            1 => self.height,
            2 => self.width / self.height,
            _ => 0.0,
        }
    }

    fn color_property(&self, local_id: usize, property_key: u16) -> Option<u32> {
        self.objects.color_property(local_id, property_key)
    }

    fn set_color_property(&mut self, local_id: usize, property_key: u16, value: u32) -> bool {
        if !self
            .objects
            .set_color_property(local_id, property_key, value)
        {
            return false;
        }
        self.did_change = true;
        true
    }

    fn bool_property(&self, local_id: usize, property_key: u16) -> Option<bool> {
        self.objects.bool_property(local_id, property_key)
    }

    fn set_bool_property(&mut self, local_id: usize, property_key: u16, value: bool) -> bool {
        if !self
            .objects
            .set_bool_property(local_id, property_key, value)
        {
            return false;
        }
        self.did_change = true;
        true
    }

    fn uint_property(&self, local_id: usize, property_key: u16) -> Option<u64> {
        self.objects.uint_property(local_id, property_key)
    }

    fn double_property(&self, local_id: usize, property_key: u16) -> Option<f32> {
        self.objects.double_property(local_id, property_key)
    }

    fn set_double_property(&mut self, local_id: usize, property_key: u16, value: f32) -> bool {
        if !self
            .objects
            .set_double_property(local_id, property_key, value)
        {
            return false;
        }
        self.did_change = true;
        true
    }

    fn set_uint_property(&mut self, local_id: usize, property_key: u16, value: u64) -> bool {
        if !self
            .objects
            .set_uint_property(local_id, property_key, value)
        {
            return false;
        }
        self.did_change = true;
        self.apply_uint_property_changed(local_id, property_key);
        true
    }

    fn string_property(&self, local_id: usize, property_key: u16) -> Option<&[u8]> {
        self.objects.string_property(local_id, property_key)
    }

    fn set_string_property(&mut self, local_id: usize, property_key: u16, value: Vec<u8>) -> bool {
        if !self
            .objects
            .set_string_property(local_id, property_key, value)
        {
            return false;
        }
        self.did_change = true;
        true
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

    pub fn advance_linear_animation_instance_with_events(
        &self,
        instance: &mut LinearAnimationInstance,
        elapsed_seconds: f32,
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) -> bool {
        let Some(animation) = self.linear_animation(instance.animation_index) else {
            return false;
        };
        instance.advance_with_events(animation, elapsed_seconds, reported_events)
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
        let Some(state_machine) = self.state_machine(instance.state_machine_index()).cloned()
        else {
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

        let Some(current) = self.transform_property(local_id, property) else {
            return false;
        };
        if current == value {
            return false;
        }
        if !self
            .objects
            .set_double_property_by_name(local_id, property.property_name(), value)
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

    pub fn transform_property(&self, local_id: usize, property: TransformProperty) -> Option<f32> {
        self.component(local_id)
            .filter(|component| component.capabilities.transform)?;
        self.objects
            .double_property_by_name(local_id, property.property_name())
            .or_else(|| Some(property.default_value()))
    }

    fn authored_transform(&self, local_id: usize) -> AuthoredTransform {
        AuthoredTransform {
            x: self
                .transform_property(local_id, TransformProperty::X)
                .unwrap_or_else(|| TransformProperty::X.default_value()),
            y: self
                .transform_property(local_id, TransformProperty::Y)
                .unwrap_or_else(|| TransformProperty::Y.default_value()),
            rotation: self
                .transform_property(local_id, TransformProperty::Rotation)
                .unwrap_or_else(|| TransformProperty::Rotation.default_value()),
            scale_x: self
                .transform_property(local_id, TransformProperty::ScaleX)
                .unwrap_or_else(|| TransformProperty::ScaleX.default_value()),
            scale_y: self
                .transform_property(local_id, TransformProperty::ScaleY)
                .unwrap_or_else(|| TransformProperty::ScaleY.default_value()),
            opacity: self
                .transform_property(local_id, TransformProperty::Opacity)
                .unwrap_or_else(|| TransformProperty::Opacity.default_value()),
        }
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
        self.apply_component_collapse_changed(local_id);
        true
    }

    pub fn update_components(&mut self) -> UpdateComponentsReport {
        self.update_components_with_hook(|_, _, _| {})
    }

    pub fn update_pass(&mut self) -> bool {
        let mut did_update = false;
        if self.joysticks_apply_before_update {
            did_update |= self.apply_joysticks(true);
        }
        if self.update_components().did_update {
            did_update = true;
        }
        if !self.joysticks_apply_before_update {
            let joysticks = self.joysticks.clone();
            for joystick in joysticks {
                if !joystick.can_apply_before_update && self.update_components().did_update {
                    did_update = true;
                }
                did_update |= self.apply_joystick(&joystick);
            }
            if self.update_components().did_update {
                did_update = true;
            }
        }
        did_update
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
        let local_id = self.components[component_index].local_id;
        if dirt.contains(ComponentDirt::TRANSFORM) {
            let authored = self.authored_transform(local_id);
            self.components[component_index].update_transform(authored);
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
            let opacity = self.authored_transform(local_id).opacity;
            let parent_opacity = self.components[component_index]
                .parent_local
                .and_then(|parent_local| self.component(parent_local))
                .filter(|parent| parent.capabilities.world_transform)
                .map(|parent| parent.transform.render_opacity)
                .unwrap_or(1.0);
            self.components[component_index].update_render_opacity(opacity, parent_opacity);
        }
    }

    fn apply_joysticks(&mut self, can_apply_before_update: bool) -> bool {
        let joysticks = self.joysticks.clone();
        joysticks
            .iter()
            .filter(|joystick| joystick.can_apply_before_update == can_apply_before_update)
            .fold(false, |changed, joystick| {
                changed | self.apply_joystick(joystick)
            })
    }

    fn apply_joystick(&mut self, joystick: &RuntimeJoystick) -> bool {
        let mut changed = false;
        if let Some(animation_index) = joystick.x_animation_index {
            if let Some(seconds) =
                self.joystick_axis_seconds(joystick.local_id, animation_index, true)
            {
                changed |= self.apply_linear_animation(animation_index, seconds, 1.0);
            }
        }
        if let Some(animation_index) = joystick.y_animation_index {
            if let Some(seconds) =
                self.joystick_axis_seconds(joystick.local_id, animation_index, false)
            {
                changed |= self.apply_linear_animation(animation_index, seconds, 1.0);
            }
        }
        changed
    }

    fn joystick_axis_seconds(
        &self,
        local_id: usize,
        animation_index: usize,
        is_x_axis: bool,
    ) -> Option<f32> {
        let animation = self.linear_animation(animation_index)?;
        let axis_key = if is_x_axis {
            joystick_x_property_key()
        } else {
            joystick_y_property_key()
        }?;
        let flag = if is_x_axis {
            JOYSTICK_FLAG_INVERT_X
        } else {
            JOYSTICK_FLAG_INVERT_Y
        };
        let mut axis = self.double_property(local_id, axis_key).unwrap_or(0.0);
        let flags = joystick_flags_property_key()
            .and_then(|key| self.uint_property(local_id, key))
            .unwrap_or(0);
        if flags & flag != 0 {
            axis = -axis;
        }
        Some(((axis + 1.0) / 2.0) * animation.duration_seconds())
    }

    fn apply_uint_property_changed(&mut self, local_id: usize, property_key: u16) -> bool {
        if solo_active_component_id_property_key() != Some(property_key) {
            return false;
        }
        self.propagate_solo_collapse(local_id)
    }

    fn apply_component_collapse_changed(&mut self, local_id: usize) -> bool {
        self.propagate_solo_collapse(local_id)
    }

    fn set_solo_active_child_by_index(&mut self, solo_local_id: usize, value: f32) -> bool {
        let rounded = value.round();
        if rounded < 0.0 || !rounded.is_finite() {
            return false;
        }
        let Some(solo) = self
            .solos
            .iter()
            .find(|solo| solo.local_id == solo_local_id)
            .cloned()
        else {
            return false;
        };
        let Some(child) = solo.children.get(rounded as usize) else {
            return false;
        };
        self.set_solo_active_child(&solo, child.local_id)
    }

    fn set_solo_active_child_by_name(&mut self, solo_local_id: usize, value: &[u8]) -> bool {
        let Some(solo) = self
            .solos
            .iter()
            .find(|solo| solo.local_id == solo_local_id)
            .cloned()
        else {
            return false;
        };
        let Some(child) = solo.children.iter().find(|child| {
            self.slot(child.local_id)
                .and_then(|slot| slot.name.as_deref())
                .is_some_and(|name| name.as_bytes() == value)
        }) else {
            return false;
        };
        self.set_solo_active_child(&solo, child.local_id)
    }

    fn set_solo_active_child(&mut self, solo: &RuntimeSolo, child_local_id: usize) -> bool {
        let Some(cpp_local_id) =
            solo.runtime_local_by_cpp_local
                .iter()
                .find_map(|(cpp_local_id, runtime_local_id)| {
                    (*runtime_local_id == child_local_id).then_some(*cpp_local_id)
                })
        else {
            return false;
        };
        let Ok(cpp_local_id) = u64::try_from(cpp_local_id) else {
            return false;
        };
        self.set_uint_property(
            solo.local_id,
            solo.active_component_property_key,
            cpp_local_id,
        )
    }

    fn propagate_solo_collapse(&mut self, solo_local_id: usize) -> bool {
        let Some(solo) = self
            .solos
            .iter()
            .find(|solo| solo.local_id == solo_local_id)
            .cloned()
        else {
            return false;
        };

        let solo_collapsed = self
            .component(solo.local_id)
            .is_some_and(RuntimeComponent::is_collapsed);
        let active_local = self
            .uint_property(solo.local_id, solo.active_component_property_key)
            .and_then(|id| usize::try_from(id).ok())
            .and_then(|id| solo.runtime_local_by_cpp_local.get(&id).copied());

        let mut changed = false;
        for child in solo.children {
            let collapsed = if child.participates {
                solo_collapsed || Some(child.local_id) != active_local
            } else {
                solo_collapsed
            };
            changed |= self.collapse_component_tree(child.local_id, collapsed);
        }
        changed
    }

    fn collapse_component_tree(&mut self, local_id: usize, collapsed: bool) -> bool {
        self.collapse_component_tree_with_ancestor(local_id, collapsed, false)
    }

    fn collapse_component_tree_with_ancestor(
        &mut self,
        local_id: usize,
        collapsed: bool,
        ancestor_changed: bool,
    ) -> bool {
        let changed_here = self.collapse_component(local_id, collapsed);
        let mut changed = changed_here;
        if ancestor_changed && !collapsed {
            changed |= self.add_dirt(local_id, ComponentDirt::FILTHY, false);
        }
        let children = self
            .components
            .iter()
            .filter(|component| component.parent_local == Some(local_id))
            .map(|component| component.local_id)
            .collect::<Vec<_>>();
        for child in children {
            changed |= self.collapse_component_tree_with_ancestor(
                child,
                collapsed,
                ancestor_changed || changed_here,
            );
        }
        changed
    }
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
    pub effect_path_commands: Vec<RuntimePathCommand>,
    pub has_effect_path: bool,
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

#[derive(Default)]
pub struct RuntimeRenderPathCache {
    artboard_clip: Option<Box<dyn RenderPath>>,
    background_paths: BTreeMap<usize, Box<dyn RenderPath>>,
    clip_paths: BTreeMap<usize, Box<dyn RenderPath>>,
    draw_paths: BTreeMap<RuntimeDrawPathCacheKey, Box<dyn RenderPath>>,
    gradient_shaders: BTreeMap<u32, RuntimeGradientShaderCacheEntry>,
}

struct RuntimeGradientShaderCacheEntry {
    state: RuntimeShapePaintState,
    shader: Box<dyn RenderShader>,
}

struct RuntimeGradientShaderResources<'a> {
    factory: &'a mut dyn RenderFactory,
    shaders: &'a mut BTreeMap<u32, RuntimeGradientShaderCacheEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct RuntimeDrawPathCacheKey {
    local_id: Option<usize>,
    path_index: usize,
}

impl RuntimeRenderPathCache {
    fn artboard_clip_path(
        &mut self,
        factory: &mut dyn RenderFactory,
        commands: &[RuntimePathCommand],
        fill_rule: RenderFillRule,
    ) -> &mut Box<dyn RenderPath> {
        runtime_cached_render_path(&mut self.artboard_clip, factory, commands, fill_rule)
    }

    fn clipping_shape_path(
        &mut self,
        local_id: usize,
        factory: &mut dyn RenderFactory,
        commands: &[RuntimePathCommand],
        fill_rule: RenderFillRule,
    ) -> &mut Box<dyn RenderPath> {
        let path = match self.clip_paths.entry(local_id) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(runtime_make_path(factory, commands, fill_rule)),
        };
        runtime_rebuild_path(path.as_mut(), commands, fill_rule);
        path
    }

    fn background_path(
        &mut self,
        local_id: usize,
        factory: &mut dyn RenderFactory,
        commands: &[RuntimePathCommand],
        fill_rule: RenderFillRule,
    ) -> &mut Box<dyn RenderPath> {
        let path = match self.background_paths.entry(local_id) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(runtime_make_path(factory, commands, fill_rule)),
        };
        runtime_rebuild_path(path.as_mut(), commands, fill_rule);
        path
    }

    fn draw_path(
        &mut self,
        key: RuntimeDrawPathCacheKey,
        factory: &mut dyn RenderFactory,
        commands: &[RuntimePathCommand],
        fill_rule: RenderFillRule,
    ) -> &mut Box<dyn RenderPath> {
        match self.draw_paths.entry(key) {
            Entry::Occupied(entry) => {
                let path = entry.into_mut();
                runtime_rebuild_path_preserving_fill_rule(path.as_mut(), commands);
                path
            }
            Entry::Vacant(entry) => entry.insert(runtime_make_path(factory, commands, fill_rule)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeFeatherState {
    pub feather_local: usize,
    pub space_value: u32,
    pub strength: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub inner: bool,
    pub inner_path_commands: Vec<RuntimePathCommand>,
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

pub fn preallocate_render_paints(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    factory: &mut dyn RenderFactory,
) -> BTreeMap<u32, Box<dyn RenderPaint>> {
    let _source_artboard_paints = preallocate_render_paint_batch(runtime, factory);
    preallocate_artboard_render_paint_batch(runtime, graph, factory)
}

fn preallocate_render_paint_batch(
    runtime: &RuntimeFile,
    factory: &mut dyn RenderFactory,
) -> BTreeMap<u32, Box<dyn RenderPaint>> {
    let mut paints = BTreeMap::new();
    for object in runtime.objects.iter().flatten() {
        if matches!(object.type_name, "Fill" | "Stroke") {
            paints.insert(object.id, factory.make_render_paint());
        }
    }
    paints
}

fn preallocate_artboard_render_paint_batch(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    factory: &mut dyn RenderFactory,
) -> BTreeMap<u32, Box<dyn RenderPaint>> {
    let mut paints = BTreeMap::new();
    let mut allocated = BTreeSet::new();
    let paint_by_mutator = graph
        .shape_paint_containers
        .iter()
        .flat_map(|container| container.paints.iter())
        .map(|paint| (paint.mutator_global, paint.global_id))
        .collect::<BTreeMap<_, _>>();

    for local_object in &graph.local_objects {
        if let Some(paint_global_id) = paint_by_mutator.get(&Some(local_object.global_id)) {
            preallocate_render_paint(*paint_global_id, factory, &mut paints, &mut allocated);
        }
    }

    for local_object in &graph.local_objects {
        let Some(object) = runtime.object(local_object.global_id as usize) else {
            continue;
        };
        if matches!(object.type_name, "Fill" | "Stroke") {
            preallocate_render_paint(object.id, factory, &mut paints, &mut allocated);
        }
    }
    paints
}

fn preallocate_render_paint(
    global_id: u32,
    factory: &mut dyn RenderFactory,
    paints: &mut BTreeMap<u32, Box<dyn RenderPaint>>,
    allocated: &mut BTreeSet<u32>,
) {
    if allocated.insert(global_id) {
        paints.insert(global_id, factory.make_render_paint());
    }
}

fn runtime_draw_background(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    container: &ShapePaintContainerNode,
    width: f32,
    height: f32,
    origin_x: f32,
    origin_y: f32,
    factory: &mut dyn RenderFactory,
    renderer: &mut dyn Renderer,
    paint_by_global: &mut BTreeMap<u32, Box<dyn RenderPaint>>,
    path_cache: &mut RuntimeRenderPathCache,
) -> Result<()> {
    let left = -origin_x * width;
    let top = -origin_y * height;
    let commands = runtime_rect_commands(left, top, left + width, top + height);
    for paint in &container.paints {
        let object = runtime
            .object(paint.global_id as usize)
            .with_context(|| format!("missing paint global {}", paint.global_id))?;
        let runtime_paint =
            runtime_background_shape_paint_command(instance, paint, commands.clone());
        runtime_configure_paint(
            paint_by_global
                .get_mut(&paint.global_id)
                .with_context(|| format!("missing render paint for global {}", paint.global_id))?
                .as_mut(),
            object,
            &runtime_paint,
            None,
        )?;
        renderer.save();
        renderer.transform(RenderMat2D::IDENTITY);
        let path = path_cache.background_path(
            container.local_id,
            factory,
            &commands,
            runtime_fill_rule_for_object(object),
        );
        runtime_configure_fill_rule(path.as_mut(), object);
        renderer.draw_path(
            path.as_ref(),
            paint_by_global
                .get(&paint.global_id)
                .with_context(|| format!("missing render paint for global {}", paint.global_id))?
                .as_ref(),
        );
        renderer.restore();
    }
    Ok(())
}

fn runtime_background_shape_paint_command(
    instance: &ArtboardInstance,
    paint: &ShapePaintNode,
    path_commands: Vec<RuntimePathCommand>,
) -> RuntimeShapePaintCommand {
    RuntimeShapePaintCommand {
        paint_local: paint.local_id,
        mutator_local: paint.mutator_local,
        paint_type: runtime_shape_paint_kind(paint.paint_type),
        path_kind: RuntimeShapePaintPathKind::Local,
        blend_mode_value: paint.blend_mode_value,
        render_blend_mode_value: 3,
        paint_state: runtime_shape_paint_state(instance, paint, 1.0),
        feather_state: None,
        path_commands,
        effect_path_commands: Vec::new(),
        has_effect_path: false,
        needs_save_operation: true,
    }
}

fn runtime_prepare_gradient_paint_command(
    instance: &ArtboardInstance,
    container: &ShapePaintContainerNode,
    paint: &ShapePaintNode,
) -> RuntimeShapePaintCommand {
    let render_opacity = instance
        .component(container.local_id)
        .map(|component| component.transform.render_opacity)
        .unwrap_or(1.0);
    RuntimeShapePaintCommand {
        paint_local: paint.local_id,
        mutator_local: paint.mutator_local,
        paint_type: runtime_shape_paint_kind(paint.paint_type),
        path_kind: paint
            .path_kind
            .and_then(runtime_shape_paint_path_kind)
            .unwrap_or(RuntimeShapePaintPathKind::Local),
        blend_mode_value: paint.blend_mode_value,
        render_blend_mode_value: runtime_shape_paint_blend_mode_value(
            paint.blend_mode_value,
            container.blend_mode_value,
        ),
        paint_state: runtime_shape_paint_state(instance, paint, render_opacity),
        feather_state: None,
        path_commands: Vec::new(),
        effect_path_commands: Vec::new(),
        has_effect_path: false,
        needs_save_operation: false,
    }
}

fn runtime_draw_command(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    graph: &ArtboardGraph,
    local_to_global: &BTreeMap<usize, u32>,
    command: RuntimeDrawCommand,
    factory: &mut dyn RenderFactory,
    renderer: &mut dyn Renderer,
    paint_by_global: &mut BTreeMap<u32, Box<dyn RenderPaint>>,
    path_cache: &mut RuntimeRenderPathCache,
) -> Result<()> {
    match command.kind {
        RuntimeDrawCommandKind::ClipStart => {
            runtime_draw_clip_start(instance, graph, command, factory, renderer, path_cache);
            return Ok(());
        }
        RuntimeDrawCommandKind::ClipEnd => {
            runtime_draw_clip_end(graph, command, renderer);
            return Ok(());
        }
        RuntimeDrawCommandKind::Draw => {}
    }

    let shape_world = command
        .local_id
        .and_then(|local_id| instance.component(local_id))
        .map(|component| component.transform.world_transform)
        .unwrap_or(Mat2D::IDENTITY);

    let mut draw_path_slots = Vec::<Vec<RuntimePathCommand>>::new();
    for paint in &command.shape_paints {
        let global_id = *local_to_global
            .get(&paint.paint_local)
            .with_context(|| format!("missing paint local {}", paint.paint_local))?;
        let object = runtime
            .object(global_id as usize)
            .with_context(|| format!("missing paint global {global_id}"))?;
        let path_commands = if paint.has_effect_path {
            &paint.effect_path_commands
        } else {
            &paint.path_commands
        };
        runtime_configure_paint(
            paint_by_global
                .get_mut(&global_id)
                .with_context(|| format!("missing render paint for global {global_id}"))?
                .as_mut(),
            object,
            paint,
            None,
        )?;
        let path_index = runtime_cached_path_slot_index(&mut draw_path_slots, path_commands);
        let mut saved = !paint.needs_save_operation;
        if matches!(
            paint.path_kind,
            RuntimeShapePaintPathKind::Local | RuntimeShapePaintPathKind::LocalClockwise
        ) {
            if !saved {
                saved = true;
                renderer.save();
            }
            renderer.transform(runtime_render_mat(shape_world));
        }
        let path = path_cache.draw_path(
            RuntimeDrawPathCacheKey {
                local_id: command.local_id,
                path_index,
            },
            factory,
            path_commands,
            RenderFillRule::Clockwise,
        );
        runtime_configure_fill_rule(path.as_mut(), object);
        renderer.draw_path(
            path.as_ref(),
            paint_by_global
                .get(&global_id)
                .with_context(|| format!("missing render paint for global {global_id}"))?
                .as_ref(),
        );
        if saved && paint.needs_save_operation {
            renderer.restore();
        }
    }

    Ok(())
}

fn runtime_draw_clip_start(
    instance: &ArtboardInstance,
    graph: &ArtboardGraph,
    command: RuntimeDrawCommand,
    factory: &mut dyn RenderFactory,
    renderer: &mut dyn Renderer,
    path_cache: &mut RuntimeRenderPathCache,
) {
    let Some(clipping_shape) = command
        .clipping_shape_local
        .and_then(|local_id| runtime_clipping_shape(graph, local_id))
    else {
        return;
    };
    if !clipping_shape.is_visible {
        return;
    }
    if command.needs_save_operation {
        renderer.save();
    }
    let path_commands = instance.runtime_clipping_shape_path_commands(clipping_shape, graph);
    if path_commands.is_empty() {
        return;
    }
    // Ported from src/shapes/clipping_shape.cpp: each ClippingShape owns a
    // cached clip path, so repeated clip-start proxies reuse the same RenderPath.
    let path = path_cache.clipping_shape_path(
        clipping_shape.local_id,
        factory,
        &path_commands,
        runtime_fill_rule_for_value(clipping_shape.fill_rule),
    );
    renderer.clip_path(path.as_ref());
}

fn runtime_draw_clip_end(
    graph: &ArtboardGraph,
    command: RuntimeDrawCommand,
    renderer: &mut dyn Renderer,
) {
    let Some(clipping_shape) = command
        .clipping_shape_local
        .and_then(|local_id| runtime_clipping_shape(graph, local_id))
    else {
        return;
    };
    if clipping_shape.is_visible && command.needs_save_operation {
        renderer.restore();
    }
}

fn runtime_clipping_shape(graph: &ArtboardGraph, local_id: usize) -> Option<&ClippingShapeNode> {
    graph
        .clipping_shapes
        .iter()
        .find(|clipping_shape| clipping_shape.local_id == local_id)
}

fn runtime_cached_path_slot_index(
    cache: &mut Vec<Vec<RuntimePathCommand>>,
    commands: &[RuntimePathCommand],
) -> usize {
    if let Some(index) = cache
        .iter()
        .position(|candidate| candidate.as_slice() == commands)
    {
        return index;
    }
    cache.push(commands.to_vec());
    cache.len() - 1
}

fn runtime_configure_paint(
    render_paint: &mut dyn RenderPaint,
    object: &RuntimeObject,
    paint: &RuntimeShapePaintCommand,
    gradient_resources: Option<&mut RuntimeGradientShaderResources<'_>>,
) -> Result<()> {
    match paint.paint_type {
        RuntimeShapePaintKind::Fill => {
            render_paint.style(RenderPaintStyle::Fill);
        }
        RuntimeShapePaintKind::Stroke => {
            render_paint.style(RenderPaintStyle::Stroke);
            render_paint.thickness(object.double_property("thickness").unwrap_or(1.0));
            render_paint.cap(runtime_stroke_cap(
                object.uint_property("cap").unwrap_or(0),
            )?);
            render_paint.join(runtime_stroke_join(
                object.uint_property("join").unwrap_or(0),
            )?);
        }
        RuntimeShapePaintKind::Unknown => anyhow::bail!("unsupported: unknown shape paint"),
    }
    render_paint.blend_mode(runtime_blend_mode(paint.render_blend_mode_value)?);
    match paint.paint_state.as_ref() {
        Some(RuntimeShapePaintState::SolidColor { render_color, .. }) => {
            render_paint.shader(None);
            render_paint.color(*render_color);
        }
        Some(
            state @ RuntimeShapePaintState::LinearGradient {
                start_x,
                start_y,
                end_x,
                end_y,
                stops,
                ..
            },
        ) => {
            if let Some(resources) = gradient_resources {
                runtime_configure_linear_gradient(
                    render_paint,
                    resources,
                    object.id,
                    state.clone(),
                    *start_x,
                    *start_y,
                    *end_x,
                    *end_y,
                    stops,
                );
            }
        }
        Some(
            state @ RuntimeShapePaintState::RadialGradient {
                start_x,
                start_y,
                end_x,
                end_y,
                stops,
                ..
            },
        ) => {
            if let Some(resources) = gradient_resources {
                runtime_configure_radial_gradient(
                    render_paint,
                    resources,
                    object.id,
                    state.clone(),
                    *start_x,
                    *start_y,
                    *end_x,
                    *end_y,
                    stops,
                );
            }
        }
        None => {
            render_paint.shader(None);
        }
    }
    Ok(())
}

// Ported from src/shapes/paint/linear_gradient.cpp and radial_gradient.cpp.
fn runtime_configure_linear_gradient(
    render_paint: &mut dyn RenderPaint,
    resources: &mut RuntimeGradientShaderResources<'_>,
    paint_global_id: u32,
    state: RuntimeShapePaintState,
    start_x: f32,
    start_y: f32,
    end_x: f32,
    end_y: f32,
    stops: &[RuntimeGradientStop],
) {
    let colors = stops
        .iter()
        .map(|stop| stop.render_color)
        .collect::<Vec<_>>();
    let positions = stops.iter().map(|stop| stop.position).collect::<Vec<_>>();
    let shader = runtime_cached_gradient_shader(resources, paint_global_id, state, |factory| {
        factory.make_linear_gradient(start_x, start_y, end_x, end_y, &colors, &positions)
    });
    render_paint.shader(Some(shader));
}

fn runtime_configure_radial_gradient(
    render_paint: &mut dyn RenderPaint,
    resources: &mut RuntimeGradientShaderResources<'_>,
    paint_global_id: u32,
    state: RuntimeShapePaintState,
    start_x: f32,
    start_y: f32,
    end_x: f32,
    end_y: f32,
    stops: &[RuntimeGradientStop],
) {
    let colors = stops
        .iter()
        .map(|stop| stop.render_color)
        .collect::<Vec<_>>();
    let positions = stops.iter().map(|stop| stop.position).collect::<Vec<_>>();
    let dx = end_x - start_x;
    let dy = end_y - start_y;
    let radius = (dx * dx + dy * dy).sqrt();
    let shader = runtime_cached_gradient_shader(resources, paint_global_id, state, |factory| {
        factory.make_radial_gradient(start_x, start_y, radius, &colors, &positions)
    });
    render_paint.shader(Some(shader));
}

fn runtime_cached_gradient_shader<'a>(
    resources: &'a mut RuntimeGradientShaderResources<'_>,
    paint_global_id: u32,
    state: RuntimeShapePaintState,
    create: impl FnOnce(&mut dyn RenderFactory) -> Box<dyn RenderShader>,
) -> &'a dyn RenderShader {
    let needs_shader = resources
        .shaders
        .get(&paint_global_id)
        .is_none_or(|entry| entry.state != state);
    if needs_shader {
        let shader = create(resources.factory);
        resources.shaders.insert(
            paint_global_id,
            RuntimeGradientShaderCacheEntry { state, shader },
        );
    }
    resources
        .shaders
        .get(&paint_global_id)
        .expect("gradient shader cache entry was inserted")
        .shader
        .as_ref()
}

fn runtime_configure_fill_rule(path: &mut dyn RenderPath, object: &RuntimeObject) {
    if object.type_name == "Fill" {
        path.fill_rule(runtime_fill_rule_for_object(object));
    }
}

fn runtime_fill_rule_for_object(object: &RuntimeObject) -> RenderFillRule {
    runtime_fill_rule_for_value(object.uint_property("fillRule").unwrap_or(0))
}

fn runtime_fill_rule_for_value(value: u64) -> RenderFillRule {
    match value {
        1 => RenderFillRule::EvenOdd,
        2 => RenderFillRule::Clockwise,
        _ => RenderFillRule::NonZero,
    }
}

fn runtime_make_path(
    factory: &mut dyn RenderFactory,
    commands: &[RuntimePathCommand],
    fill_rule: RenderFillRule,
) -> Box<dyn RenderPath> {
    let mut path = factory.make_empty_render_path();
    runtime_rebuild_path(path.as_mut(), commands, fill_rule);
    path
}

fn runtime_cached_render_path<'a>(
    slot: &'a mut Option<Box<dyn RenderPath>>,
    factory: &mut dyn RenderFactory,
    commands: &[RuntimePathCommand],
    fill_rule: RenderFillRule,
) -> &'a mut Box<dyn RenderPath> {
    let path = match slot {
        Some(path) => path,
        None => slot.insert(runtime_make_path(factory, commands, fill_rule)),
    };
    runtime_rebuild_path(path.as_mut(), commands, fill_rule);
    path
}

fn runtime_rebuild_path(
    path: &mut dyn RenderPath,
    commands: &[RuntimePathCommand],
    fill_rule: RenderFillRule,
) {
    path.rewind();
    path.fill_rule(fill_rule);
    runtime_append_path_commands(path, commands);
}

fn runtime_rebuild_path_preserving_fill_rule(
    path: &mut dyn RenderPath,
    commands: &[RuntimePathCommand],
) {
    path.rewind();
    runtime_append_path_commands(path, commands);
}

fn runtime_append_path_commands(path: &mut dyn RenderPath, commands: &[RuntimePathCommand]) {
    for command in commands {
        match *command {
            RuntimePathCommand::Move { x, y } => path.move_to(x, y),
            RuntimePathCommand::Line { x, y } => path.line_to(x, y),
            RuntimePathCommand::Cubic {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
            } => path.cubic_to(x1, y1, x2, y2, x3, y3),
            RuntimePathCommand::Close => path.close(),
        }
    }
}

fn runtime_rect_commands(left: f32, top: f32, right: f32, bottom: f32) -> Vec<RuntimePathCommand> {
    vec![
        RuntimePathCommand::Move { x: left, y: top },
        RuntimePathCommand::Line { x: right, y: top },
        RuntimePathCommand::Line {
            x: right,
            y: bottom,
        },
        RuntimePathCommand::Line { x: left, y: bottom },
        RuntimePathCommand::Close,
    ]
}

fn runtime_render_mat(mat: Mat2D) -> RenderMat2D {
    RenderMat2D(mat.0)
}

fn mat2d_has_visible_skew_or_rotation(mat: Mat2D) -> bool {
    // Keep near-axis-aligned cancellation on the regular multiply path; real
    // off-axis local path composition needs the fused C++ residual.
    const MATRIX_AXIS_EPSILON: f32 = 5.0e-2;
    mat.0[1].abs() > MATRIX_AXIS_EPSILON || mat.0[2].abs() > MATRIX_AXIS_EPSILON
}

fn runtime_blend_mode(value: u32) -> Result<RenderBlendMode> {
    Ok(match value {
        3 => RenderBlendMode::SrcOver,
        14 => RenderBlendMode::Screen,
        15 => RenderBlendMode::Overlay,
        16 => RenderBlendMode::Darken,
        17 => RenderBlendMode::Lighten,
        18 => RenderBlendMode::ColorDodge,
        19 => RenderBlendMode::ColorBurn,
        20 => RenderBlendMode::HardLight,
        21 => RenderBlendMode::SoftLight,
        22 => RenderBlendMode::Difference,
        23 => RenderBlendMode::Exclusion,
        24 => RenderBlendMode::Multiply,
        25 => RenderBlendMode::Hue,
        26 => RenderBlendMode::Saturation,
        27 => RenderBlendMode::Color,
        28 => RenderBlendMode::Luminosity,
        _ => anyhow::bail!("unsupported blend mode {value}"),
    })
}

fn runtime_stroke_cap(value: u64) -> Result<RenderStrokeCap> {
    Ok(match value {
        0 => RenderStrokeCap::Butt,
        1 => RenderStrokeCap::Round,
        2 => RenderStrokeCap::Square,
        _ => anyhow::bail!("unsupported stroke cap {value}"),
    })
}

fn runtime_stroke_join(value: u64) -> Result<RenderStrokeJoin> {
    Ok(match value {
        0 => RenderStrokeJoin::Miter,
        1 => RenderStrokeJoin::Round,
        2 => RenderStrokeJoin::Bevel,
        _ => anyhow::bail!("unsupported stroke join {value}"),
    })
}

fn runtime_shape_paint_command(
    artboard: &ArtboardInstance,
    paint: &ShapePaintNode,
    container_blend_mode_value: u32,
    needs_save_operation: bool,
    render_opacity: f32,
    shape_world: Mat2D,
    path_commands: Vec<RuntimePathCommand>,
) -> Option<RuntimeShapePaintCommand> {
    if !runtime_shape_paint_is_visible(artboard, paint) {
        return None;
    }
    let effect_path_commands = runtime_effect_path_commands(artboard, paint, &path_commands);
    let has_effect_path = effect_path_commands.is_some();
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
        paint_state: runtime_shape_paint_state(artboard, paint, render_opacity),
        feather_state: runtime_feather_state(paint.feather.as_ref(), &path_commands, shape_world),
        path_commands,
        effect_path_commands: effect_path_commands.unwrap_or_default(),
        has_effect_path,
        needs_save_operation,
    })
}

fn runtime_shape_paint_is_visible(artboard: &ArtboardInstance, paint: &ShapePaintNode) -> bool {
    let Some(property_key) = shape_paint_is_visible_property_key() else {
        return paint.is_visible;
    };
    match paint.paint_type {
        ShapePaintKind::Fill | ShapePaintKind::Unknown => artboard
            .bool_property(paint.local_id, property_key)
            .unwrap_or(paint.is_visible),
        ShapePaintKind::Stroke => artboard
            .bool_property(paint.local_id, property_key)
            .map(|visible| paint.is_visible && visible)
            .unwrap_or(paint.is_visible),
    }
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
    artboard: &ArtboardInstance,
    paint: &ShapePaintNode,
    render_opacity: f32,
) -> Option<RuntimeShapePaintState> {
    match paint.paint_state.clone()? {
        ShapePaintStateNode::SolidColor { color } => {
            let color = paint
                .mutator_local
                .zip(solid_color_value_property_key())
                .and_then(|(local_id, property_key)| {
                    artboard.color_property(local_id, property_key)
                })
                .unwrap_or(color);
            Some(RuntimeShapePaintState::SolidColor {
                color,
                render_color: color_modulate_opacity(color, render_opacity),
            })
        }
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

fn runtime_feather_state(
    feather: Option<&FeatherNode>,
    path_commands: &[RuntimePathCommand],
    shape_world: Mat2D,
) -> Option<RuntimeFeatherState> {
    let feather = feather?;
    let inner_path_commands = inner_feather_path_commands(feather, path_commands, shape_world);
    Some(RuntimeFeatherState {
        feather_local: feather.local_id,
        space_value: feather.space_value,
        strength: feather.strength,
        offset_x: feather.offset_x,
        offset_y: feather.offset_y,
        inner: feather.inner,
        inner_path_commands,
    })
}

fn inner_feather_path_commands(
    feather: &FeatherNode,
    source: &[RuntimePathCommand],
    shape_world: Mat2D,
) -> Vec<RuntimePathCommand> {
    if !feather.inner || source.is_empty() {
        return Vec::new();
    }
    let Some(bounds) = path_command_bounds(source) else {
        return Vec::new();
    };
    let pad = feather.strength * 1.5;
    let mut commands = rect_commands(bounds.pad(pad));
    let mut reversed_source = path_commands_backwards(source);
    let mut offset = (feather.offset_x, feather.offset_y);
    if feather.space_value == 0 {
        offset = shape_world
            .invert_or_identity()
            .transform_direction(offset.0, offset.1);
    }
    translate_path_commands(&mut reversed_source, offset);
    commands.extend(reversed_source);
    commands
}

#[derive(Debug, Clone, Copy)]
struct PathBounds {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
}

impl PathBounds {
    fn from_point(x: f32, y: f32) -> Self {
        Self {
            min_x: x,
            min_y: y,
            max_x: x,
            max_y: y,
        }
    }

    fn include(&mut self, x: f32, y: f32) {
        self.min_x = self.min_x.min(x);
        self.min_y = self.min_y.min(y);
        self.max_x = self.max_x.max(x);
        self.max_y = self.max_y.max(y);
    }

    fn pad(self, amount: f32) -> Self {
        Self {
            min_x: self.min_x - amount,
            min_y: self.min_y - amount,
            max_x: self.max_x + amount,
            max_y: self.max_y + amount,
        }
    }
}

fn path_command_bounds(commands: &[RuntimePathCommand]) -> Option<PathBounds> {
    let mut bounds: Option<PathBounds> = None;
    for command in commands {
        for (x, y) in command_points(command) {
            match &mut bounds {
                Some(bounds) => bounds.include(x, y),
                None => bounds = Some(PathBounds::from_point(x, y)),
            }
        }
    }
    bounds
}

fn rect_commands(bounds: PathBounds) -> Vec<RuntimePathCommand> {
    vec![
        RuntimePathCommand::Move {
            x: bounds.min_x,
            y: bounds.min_y,
        },
        RuntimePathCommand::Line {
            x: bounds.max_x,
            y: bounds.min_y,
        },
        RuntimePathCommand::Line {
            x: bounds.max_x,
            y: bounds.max_y,
        },
        RuntimePathCommand::Line {
            x: bounds.min_x,
            y: bounds.max_y,
        },
        RuntimePathCommand::Close,
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RawPathVerb {
    Move,
    Line,
    Cubic,
    Close,
}

fn path_commands_backwards(commands: &[RuntimePathCommand]) -> Vec<RuntimePathCommand> {
    let (verbs, points) = raw_path_parts(commands);
    if verbs.is_empty() {
        return Vec::new();
    }

    let reversed_points = points.into_iter().rev().collect::<Vec<_>>();
    let mut reversed_verbs = Vec::with_capacity(verbs.len());
    reversed_verbs.push(RawPathVerb::Move);
    let mut closed = false;
    for index in (0..verbs.len()).rev() {
        let verb = verbs[index];
        if verb == RawPathVerb::Close {
            closed = true;
            continue;
        }
        if verb == RawPathVerb::Move && closed {
            reversed_verbs.push(RawPathVerb::Close);
            closed = false;
        }
        if index != 0 {
            reversed_verbs.push(verb);
        } else {
            break;
        }
    }

    raw_path_parts_to_commands(&reversed_verbs, &reversed_points)
}

// Coarsely translated from:
// /Users/levi/dev/oss/rive-runtime/src/math/raw_path.cpp RawPath::pruneEmptySegments
fn prune_empty_path_segments(commands: &mut Vec<RuntimePathCommand>) {
    let mut pruned = Vec::with_capacity(commands.len());
    let mut current = None::<(f32, f32)>;
    for command in commands.drain(..) {
        match command {
            RuntimePathCommand::Move { x, y } => {
                current = Some((x, y));
                pruned.push(RuntimePathCommand::Move { x, y });
            }
            RuntimePathCommand::Line { x, y } => {
                if current != Some((x, y)) {
                    pruned.push(RuntimePathCommand::Line { x, y });
                }
                current = Some((x, y));
            }
            RuntimePathCommand::Cubic {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
            } => {
                if current != Some((x1, y1)) || (x1, y1) != (x2, y2) || (x2, y2) != (x3, y3) {
                    pruned.push(RuntimePathCommand::Cubic {
                        x1,
                        y1,
                        x2,
                        y2,
                        x3,
                        y3,
                    });
                }
                current = Some((x3, y3));
            }
            RuntimePathCommand::Close => pruned.push(RuntimePathCommand::Close),
        }
    }
    *commands = pruned;
}

fn raw_path_parts(commands: &[RuntimePathCommand]) -> (Vec<RawPathVerb>, Vec<(f32, f32)>) {
    let mut verbs = Vec::with_capacity(commands.len());
    let mut points = Vec::new();
    for command in commands {
        match *command {
            RuntimePathCommand::Move { x, y } => {
                verbs.push(RawPathVerb::Move);
                points.push((x, y));
            }
            RuntimePathCommand::Line { x, y } => {
                verbs.push(RawPathVerb::Line);
                points.push((x, y));
            }
            RuntimePathCommand::Cubic {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
            } => {
                verbs.push(RawPathVerb::Cubic);
                points.push((x1, y1));
                points.push((x2, y2));
                points.push((x3, y3));
            }
            RuntimePathCommand::Close => verbs.push(RawPathVerb::Close),
        }
    }
    (verbs, points)
}

fn raw_path_parts_to_commands(
    verbs: &[RawPathVerb],
    points: &[(f32, f32)],
) -> Vec<RuntimePathCommand> {
    let mut commands = Vec::with_capacity(verbs.len());
    let mut point_index = 0;
    for verb in verbs {
        match *verb {
            RawPathVerb::Move => {
                let Some((x, y)) = points.get(point_index).copied() else {
                    return Vec::new();
                };
                point_index += 1;
                commands.push(RuntimePathCommand::Move { x, y });
            }
            RawPathVerb::Line => {
                let Some((x, y)) = points.get(point_index).copied() else {
                    return Vec::new();
                };
                point_index += 1;
                commands.push(RuntimePathCommand::Line { x, y });
            }
            RawPathVerb::Cubic => {
                let Some((x1, y1)) = points.get(point_index).copied() else {
                    return Vec::new();
                };
                let Some((x2, y2)) = points.get(point_index + 1).copied() else {
                    return Vec::new();
                };
                let Some((x3, y3)) = points.get(point_index + 2).copied() else {
                    return Vec::new();
                };
                point_index += 3;
                commands.push(RuntimePathCommand::Cubic {
                    x1,
                    y1,
                    x2,
                    y2,
                    x3,
                    y3,
                });
            }
            RawPathVerb::Close => commands.push(RuntimePathCommand::Close),
        }
    }
    commands
}

fn translate_path_commands(commands: &mut [RuntimePathCommand], offset: (f32, f32)) {
    for command in commands {
        match command {
            RuntimePathCommand::Move { x, y } | RuntimePathCommand::Line { x, y } => {
                *x += offset.0;
                *y += offset.1;
            }
            RuntimePathCommand::Cubic {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
            } => {
                *x1 += offset.0;
                *y1 += offset.1;
                *x2 += offset.0;
                *y2 += offset.1;
                *x3 += offset.0;
                *y3 += offset.1;
            }
            RuntimePathCommand::Close => {}
        }
    }
}

fn command_points(command: &RuntimePathCommand) -> Vec<(f32, f32)> {
    match *command {
        RuntimePathCommand::Move { x, y } | RuntimePathCommand::Line { x, y } => {
            vec![(x, y)]
        }
        RuntimePathCommand::Cubic {
            x1,
            y1,
            x2,
            y2,
            x3,
            y3,
        } => vec![(x1, y1), (x2, y2), (x3, y3)],
        RuntimePathCommand::Close => Vec::new(),
    }
}

fn runtime_effect_path_commands(
    artboard: &ArtboardInstance,
    paint: &ShapePaintNode,
    source: &[RuntimePathCommand],
) -> Option<Vec<RuntimePathCommand>> {
    if paint.effects.is_empty() {
        return None;
    }

    let mut current = source.to_vec();
    let mut has_supported_effect = false;
    for effect in &paint.effects {
        let Some(effect_path) =
            runtime_stroke_effect_path_commands(artboard, effect, paint, &current)
        else {
            continue;
        };
        current = effect_path;
        has_supported_effect = true;
    }

    has_supported_effect.then_some(current)
}

fn runtime_stroke_effect_path_commands(
    artboard: &ArtboardInstance,
    effect: &StrokeEffectNode,
    paint: &ShapePaintNode,
    source: &[RuntimePathCommand],
) -> Option<Vec<RuntimePathCommand>> {
    match effect.type_name {
        "DashPath" => runtime_dash_path_effect_commands(artboard, effect, paint, source),
        "TrimPath" => runtime_trim_path_line_effect_commands(artboard, effect, paint, source),
        _ => None,
    }
}

// Coarsely translated from:
// /Users/levi/dev/oss/rive-runtime/src/shapes/paint/dash_path.cpp DashPath::updateEffect
// and PathDasher::applyDash.
fn runtime_dash_path_effect_commands(
    artboard: &ArtboardInstance,
    effect: &StrokeEffectNode,
    paint: &ShapePaintNode,
    source: &[RuntimePathCommand],
) -> Option<Vec<RuntimePathCommand>> {
    let offset = runtime_dash_path_double_property(artboard, effect, "offset", effect.dash_offset)?;
    let offset_is_percentage = runtime_dash_path_bool_property(
        artboard,
        effect,
        "offsetIsPercentage",
        effect.dash_offset_is_percentage,
    )?;
    if paint.paint_type == ShapePaintKind::Fill {
        return Some(source.to_vec());
    }

    Some(dash_path_apply_dash(
        artboard,
        source,
        offset,
        offset_is_percentage,
        &effect.dashes,
    ))
}

fn runtime_dash_path_double_property(
    artboard: &ArtboardInstance,
    effect: &StrokeEffectNode,
    property_name: &str,
    fallback: Option<f32>,
) -> Option<f32> {
    property_key_for_name("DashPath", property_name)
        .and_then(|key| artboard.double_property(effect.local_id, key))
        .or(fallback)
}

fn runtime_dash_path_bool_property(
    artboard: &ArtboardInstance,
    effect: &StrokeEffectNode,
    property_name: &str,
    fallback: Option<bool>,
) -> Option<bool> {
    property_key_for_name("DashPath", property_name)
        .and_then(|key| artboard.bool_property(effect.local_id, key))
        .or(fallback)
}

fn dash_path_apply_dash(
    artboard: &ArtboardInstance,
    source: &[RuntimePathCommand],
    offset: f32,
    offset_is_percentage: bool,
    dashes: &[DashNode],
) -> Vec<RuntimePathCommand> {
    let path_measure = RuntimePathMeasure::from_commands(source);
    if path_measure.length <= 0.0
        || !dashes.iter().any(|dash| {
            dash_normalized_length(
                runtime_dash_length(artboard, dash),
                runtime_dash_length_is_percentage(artboard, dash),
                path_measure.length,
                false,
            ) > 0.0
        })
    {
        return Vec::new();
    }

    let mut commands = Vec::new();
    let mut dash_index = 0usize;
    let mut dashed = 0.0;
    let mut distance =
        dash_normalized_length(offset, offset_is_percentage, path_measure.length, true);
    let mut draw = true;

    while dashed < path_measure.length {
        let dash = &dashes[dash_index % dashes.len()];
        dash_index += 1;

        let mut dash_length = dash_normalized_length(
            runtime_dash_length(artboard, dash),
            runtime_dash_length_is_percentage(artboard, dash),
            path_measure.length,
            false,
        );
        if dash_length > path_measure.length {
            dash_length = path_measure.length;
        }

        let mut end_length = distance + dash_length;
        if end_length > path_measure.length {
            end_length -= path_measure.length;
            if draw {
                if distance < path_measure.length {
                    path_measure.get_segment(distance, path_measure.length, &mut commands, true);
                    path_measure.get_segment(
                        0.0,
                        end_length,
                        &mut commands,
                        !path_measure.is_closed(),
                    );
                } else {
                    path_measure.get_segment(0.0, end_length, &mut commands, true);
                }
            }

            distance = end_length - dash_length;
        } else if draw {
            path_measure.get_segment(distance, end_length, &mut commands, true);
        }

        distance += dash_length;
        dashed += dash_length;
        draw = !draw;
    }

    commands
}

fn runtime_dash_length(artboard: &ArtboardInstance, dash: &DashNode) -> f32 {
    property_key_for_name("Dash", "length")
        .and_then(|key| artboard.double_property(dash.local_id, key))
        .unwrap_or(dash.length)
}

fn runtime_dash_length_is_percentage(artboard: &ArtboardInstance, dash: &DashNode) -> bool {
    property_key_for_name("Dash", "lengthIsPercentage")
        .and_then(|key| artboard.bool_property(dash.local_id, key))
        .unwrap_or(dash.length_is_percentage)
}

fn dash_normalized_length(
    value: f32,
    is_percentage: bool,
    contour_length: f32,
    wraps: bool,
) -> f32 {
    let mut p = value;
    if wraps {
        let right = if is_percentage { 1.0 } else { contour_length };
        if right != 0.0 {
            p = value % right;
            if p < 0.0 {
                p += right;
            }
        }
    }
    if is_percentage { p * contour_length } else { p }
}

fn runtime_trim_path_mode_value(
    artboard: &ArtboardInstance,
    effect: &StrokeEffectNode,
) -> Option<u32> {
    property_key_for_name("TrimPath", "modeValue")
        .and_then(|key| artboard.uint_property(effect.local_id, key))
        .and_then(|value| u32::try_from(value).ok())
        .or(effect.trim_mode_value)
}

fn runtime_trim_path_double_property(
    artboard: &ArtboardInstance,
    effect: &StrokeEffectNode,
    property_name: &str,
    fallback: Option<f32>,
) -> f32 {
    property_key_for_name("TrimPath", property_name)
        .and_then(|key| artboard.double_property(effect.local_id, key))
        .or(fallback)
        .unwrap_or(0.0)
}

// Coarsely translated from:
// /Users/levi/dev/oss/rive-runtime/src/shapes/paint/trim_path.cpp TrimPath::trimPath
fn runtime_trim_path_line_effect_commands(
    artboard: &ArtboardInstance,
    effect: &StrokeEffectNode,
    paint: &ShapePaintNode,
    source: &[RuntimePathCommand],
) -> Option<Vec<RuntimePathCommand>> {
    let mode = runtime_trim_path_mode_value(artboard, effect)?;
    if !matches!(mode, 1 | 2) {
        return None;
    }
    let contours = TrimContour::from_commands(source);
    if contours.is_empty() {
        return Some(Vec::new());
    }

    let render_offset = positive_unit_mod(runtime_trim_path_double_property(
        artboard,
        effect,
        "offset",
        effect.trim_offset,
    ));
    let trim_start =
        runtime_trim_path_double_property(artboard, effect, "start", effect.trim_start);
    let trim_end = runtime_trim_path_double_property(artboard, effect, "end", effect.trim_end);
    let close_shape = paint.paint_type == ShapePaintKind::Fill;
    match mode {
        1 => Some(trim_path_sequential(
            &contours,
            trim_start,
            trim_end,
            render_offset,
            close_shape,
        )),
        2 => Some(trim_path_synchronized(
            &contours,
            trim_start,
            trim_end,
            render_offset,
            close_shape,
        )),
        _ => None,
    }
}

fn positive_unit_mod(value: f32) -> f32 {
    value.rem_euclid(1.0)
}

#[derive(Debug, Clone)]
struct TrimContour {
    segments: Vec<TrimMeasuredSegment>,
    length: f32,
    is_closed: bool,
}

#[derive(Debug, Clone)]
struct RuntimePathMeasure {
    contours: Vec<TrimContour>,
    length: f32,
}

#[derive(Debug, Clone)]
struct TrimMeasuredSegment {
    original_index: usize,
    kind: TrimSegmentKind,
    distance: f32,
    t: f32,
}

#[derive(Debug, Clone, Copy)]
enum TrimSegmentKind {
    Line {
        from: (f32, f32),
        to: (f32, f32),
    },
    Cubic {
        p0: (f32, f32),
        p1: (f32, f32),
        p2: (f32, f32),
        p3: (f32, f32),
    },
}

impl TrimContour {
    fn from_commands(commands: &[RuntimePathCommand]) -> Vec<Self> {
        let mut contours = Vec::new();
        let mut raw_segments = Vec::<TrimSegmentKind>::new();
        let mut start = None::<(f32, f32)>;
        let mut current = None::<(f32, f32)>;
        let mut is_closed = false;

        let finish_contour = |contours: &mut Vec<Self>,
                              raw_segments: &mut Vec<TrimSegmentKind>,
                              is_closed: &mut bool| {
            if let Some(contour) = Self::from_raw_segments(raw_segments, *is_closed) {
                contours.push(contour);
            }
            raw_segments.clear();
            *is_closed = false;
        };

        for command in commands {
            match *command {
                RuntimePathCommand::Move { x, y } => {
                    if !raw_segments.is_empty() {
                        finish_contour(&mut contours, &mut raw_segments, &mut is_closed);
                    }
                    start = Some((x, y));
                    current = Some((x, y));
                }
                RuntimePathCommand::Line { x, y } => {
                    let Some(from) = current else {
                        continue;
                    };
                    let to = (x, y);
                    if distance_squared(from, to) > 0.0 {
                        raw_segments.push(TrimSegmentKind::Line { from, to });
                    }
                    current = Some(to);
                }
                RuntimePathCommand::Cubic {
                    x1,
                    y1,
                    x2,
                    y2,
                    x3,
                    y3,
                } => {
                    let Some(p0) = current else {
                        continue;
                    };
                    let p1 = (x1, y1);
                    let p2 = (x2, y2);
                    let p3 = (x3, y3);
                    raw_segments.push(TrimSegmentKind::Cubic { p0, p1, p2, p3 });
                    current = Some(p3);
                }
                RuntimePathCommand::Close => {
                    if let (Some(from), Some(to)) = (current, start) {
                        if distance_squared(from, to) > 0.0 {
                            raw_segments.push(TrimSegmentKind::Line { from, to });
                        }
                        current = Some(to);
                    }
                    is_closed = true;
                }
            }
        }

        if !raw_segments.is_empty() {
            finish_contour(&mut contours, &mut raw_segments, &mut is_closed);
        }
        contours
    }

    fn from_raw_segments(raw_segments: &[TrimSegmentKind], is_closed: bool) -> Option<Self> {
        let mut segments = Vec::new();
        let mut distance_so_far = 0.0;

        for (original_index, segment) in raw_segments.iter().copied().enumerate() {
            match segment {
                TrimSegmentKind::Line { from, to } => {
                    let length = distance(from, to);
                    if length == 0.0 {
                        continue;
                    }
                    distance_so_far += length;
                    segments.push(TrimMeasuredSegment {
                        original_index,
                        kind: segment,
                        distance: distance_so_far,
                        t: 1.0,
                    });
                }
                TrimSegmentKind::Cubic { p0, p1, p2, p3 } => {
                    let segment_count = cubic_measure_segment_count([p0, p1, p2, p3]);
                    if segment_count == 0 {
                        continue;
                    }
                    let dt = 1.0 / segment_count as f32;
                    let mut t = dt;
                    let mut previous = p0;
                    for _ in 1..segment_count {
                        let next = eval_cubic([p0, p1, p2, p3], t);
                        distance_so_far += distance(previous, next);
                        segments.push(TrimMeasuredSegment {
                            original_index,
                            kind: segment,
                            distance: distance_so_far,
                            t: trim_contour_dot30_t(t),
                        });
                        previous = next;
                        t += dt;
                    }
                    distance_so_far += distance(previous, p3);
                    segments.push(TrimMeasuredSegment {
                        original_index,
                        kind: segment,
                        distance: distance_so_far,
                        t: 1.0,
                    });
                }
            }
        }

        (distance_so_far > 0.0 && !segments.is_empty()).then_some(Self {
            segments,
            length: distance_so_far,
            is_closed,
        })
    }

    fn get_segment(
        &self,
        mut start_distance: f32,
        mut end_distance: f32,
        commands: &mut Vec<RuntimePathCommand>,
        start_with_move: bool,
    ) {
        start_distance = start_distance.max(0.0);
        end_distance = end_distance.min(self.length);
        if start_distance >= end_distance {
            return;
        }

        let mut start_index = self.find_segment(start_distance);
        let end_index = self.find_segment(end_distance);
        let mut start_t = self.compute_t(start_index, start_distance);
        let end_t = self.compute_t(end_index, end_distance);

        if 1.0 - start_t < TRIM_CONTOUR_EPSILON && start_index < end_index {
            start_index += 1;
            start_t = 0.0;
        }

        let start = &self.segments[start_index];
        let end = &self.segments[end_index];
        if start.original_index == end.original_index {
            start
                .kind
                .extract(commands, start_t, end_t, start_with_move);
            return;
        }

        start.kind.extract(commands, start_t, 1.0, start_with_move);

        let mut original_index = start.original_index + 1;
        while original_index < end.original_index {
            if let Some(segment) = self.first_segment_for_original(original_index) {
                segment.kind.extract_full(commands);
            }
            original_index += 1;
        }

        end.kind.extract(commands, 0.0, end_t, false);
    }

    fn find_segment(&self, distance: f32) -> usize {
        self.segments
            .iter()
            .position(|segment| segment.distance >= distance)
            .unwrap_or_else(|| self.segments.len() - 1)
    }

    fn compute_t(&self, index: usize, distance: f32) -> f32 {
        let segment = &self.segments[index];
        let mut previous_distance = 0.0;
        let mut previous_t = 0.0;
        if index > 0 {
            let previous = &self.segments[index - 1];
            previous_distance = previous.distance;
            if previous.original_index == segment.original_index {
                previous_t = previous.t;
            }
        }

        let denominator = segment.distance - previous_distance;
        if denominator == 0.0 {
            return previous_t;
        }
        let ratio = (distance - previous_distance) / denominator;
        (previous_t + (segment.t - previous_t) * ratio).clamp(previous_t, segment.t)
    }

    fn first_segment_for_original(&self, original_index: usize) -> Option<&TrimMeasuredSegment> {
        self.segments
            .iter()
            .find(|segment| segment.original_index == original_index)
    }
}

impl RuntimePathMeasure {
    fn from_commands(commands: &[RuntimePathCommand]) -> Self {
        let contours = TrimContour::from_commands(commands);
        let length = contours.iter().map(|contour| contour.length).sum();
        Self { contours, length }
    }

    fn get_segment(
        &self,
        start_distance: f32,
        end_distance: f32,
        commands: &mut Vec<RuntimePathCommand>,
        start_with_move: bool,
    ) {
        if self.contours.is_empty() {
            return;
        }

        let start_distance = start_distance.clamp(0.0, self.length);
        let end_distance = end_distance.clamp(0.0, self.length);
        if start_distance >= end_distance {
            return;
        }

        let mut current_distance = 0.0;
        let mut is_first_segment = true;
        for contour in &self.contours {
            let contour_length = contour.length;
            let contour_start = current_distance;
            let contour_end = current_distance + contour_length;

            if contour_end > start_distance && contour_start < end_distance {
                let local_start = (start_distance - contour_start).max(0.0);
                let local_end = (end_distance - contour_start).min(contour_length);
                contour.get_segment(
                    local_start,
                    local_end,
                    commands,
                    !is_first_segment || start_with_move,
                );
                is_first_segment = false;
            }

            current_distance += contour_length;
            if current_distance >= end_distance {
                break;
            }
        }
    }

    fn is_closed(&self) -> bool {
        self.contours.len() == 1 && self.contours[0].is_closed
    }
}

impl TrimSegmentKind {
    fn extract(
        self,
        commands: &mut Vec<RuntimePathCommand>,
        start_t: f32,
        end_t: f32,
        move_to: bool,
    ) {
        match self {
            Self::Line { from, to } => {
                let start = lerp_point(from, to, start_t);
                let end = lerp_point(from, to, end_t);
                if move_to {
                    commands.push(RuntimePathCommand::Move {
                        x: start.0,
                        y: start.1,
                    });
                }
                commands.push(RuntimePathCommand::Line { x: end.0, y: end.1 });
            }
            Self::Cubic { p0, p1, p2, p3 } => {
                let [start, control_1, control_2, end] =
                    cubic_extract([p0, p1, p2, p3], start_t, end_t);
                if move_to {
                    commands.push(RuntimePathCommand::Move {
                        x: start.0,
                        y: start.1,
                    });
                }
                commands.push(RuntimePathCommand::Cubic {
                    x1: control_1.0,
                    y1: control_1.1,
                    x2: control_2.0,
                    y2: control_2.1,
                    x3: end.0,
                    y3: end.1,
                });
            }
        }
    }

    fn extract_full(self, commands: &mut Vec<RuntimePathCommand>) {
        match self {
            Self::Line { to, .. } => commands.push(RuntimePathCommand::Line { x: to.0, y: to.1 }),
            Self::Cubic { p1, p2, p3, .. } => commands.push(RuntimePathCommand::Cubic {
                x1: p1.0,
                y1: p1.1,
                x2: p2.0,
                y2: p2.1,
                x3: p3.0,
                y3: p3.1,
            }),
        }
    }
}

fn trim_path_sequential(
    contours: &[TrimContour],
    trim_start: f32,
    trim_end: f32,
    render_offset: f32,
    close_shape: bool,
) -> Vec<RuntimePathCommand> {
    let total_length = contours.iter().map(|contour| contour.length).sum::<f32>();
    if total_length == 0.0 {
        return Vec::new();
    }

    let mut start_length = total_length * (trim_start + render_offset);
    let mut end_length = total_length * (trim_end + render_offset);
    if end_length < start_length {
        std::mem::swap(&mut start_length, &mut end_length);
    }
    if start_length > total_length {
        start_length -= total_length;
        end_length -= total_length;
    }

    let mut indices = Vec::new();
    let mut lengths = Vec::new();
    let mut contour_index = 0usize;
    while end_length > 0.0 {
        let current_index = contour_index % contours.len();
        let contour = &contours[current_index];
        if start_length < contour.length {
            indices.push(current_index);
            lengths.push(start_length);
            lengths.push(end_length);
            end_length -= contour.length;
            start_length = 0.0;
        } else {
            start_length -= contour.length;
            end_length -= contour.length;
        }
        contour_index += 1;
    }

    let mut commands = Vec::new();
    let mut starting_index = 0isize;
    let mut index_count = 0usize;
    let mut previous_contour_index = None::<usize>;
    while index_count < indices.len() {
        let index = starting_index.rem_euclid(indices.len() as isize) as usize;
        let contour_index = indices[index];
        let contour = &contours[contour_index];
        let length_index = index * 2;
        let start_length = lengths[length_index];
        let end_length = lengths[length_index + 1];
        contour.get_segment(
            start_length,
            end_length,
            &mut commands,
            previous_contour_index != Some(contour_index) || !contour.is_closed,
        );
        if (start_length == 0.0 && end_length - start_length >= contour.length && contour.is_closed)
            || close_shape
        {
            commands.push(RuntimePathCommand::Close);
        }
        previous_contour_index = Some(contour_index);
        index_count += 1;
        starting_index -= 1;
    }
    commands
}

fn trim_path_synchronized(
    contours: &[TrimContour],
    trim_start: f32,
    trim_end: f32,
    render_offset: f32,
    close_shape: bool,
) -> Vec<RuntimePathCommand> {
    let mut commands = Vec::new();
    for contour in contours {
        let mut start_length = contour.length * (trim_start + render_offset);
        let mut end_length = contour.length * (trim_end + render_offset);
        if end_length < start_length {
            std::mem::swap(&mut start_length, &mut end_length);
        }

        if start_length >= contour.length {
            start_length -= contour.length;
            end_length -= contour.length;
        }
        contour.get_segment(start_length, end_length, &mut commands, true);
        while end_length > contour.length {
            start_length = 0.0;
            end_length -= contour.length;
            contour.get_segment(start_length, end_length, &mut commands, !contour.is_closed);
        }

        if (trim_start == 0.0 && trim_end == 1.0 && contour.is_closed) || close_shape {
            commands.push(RuntimePathCommand::Close);
        }
    }
    commands
}

const TRIM_CONTOUR_EPSILON: f32 = 1.0 / 4096.0;
const TRIM_CONTOUR_INV_TOLERANCE: f32 = 2.0;
const TRIM_CONTOUR_MAX_SEGMENTS: u32 = 100;
const TRIM_CONTOUR_DOT30_SCALE: f32 = (1u32 << 30) as f32;
const TRIM_CONTOUR_MAX_DOT30: u32 = (1u32 << 30) - 1;
const TRIM_CONTOUR_INV_MAX_DOT30: f32 = 1.0 / TRIM_CONTOUR_MAX_DOT30 as f32;

fn trim_contour_dot30_t(t: f32) -> f32 {
    ((t * TRIM_CONTOUR_DOT30_SCALE) as u32) as f32 * TRIM_CONTOUR_INV_MAX_DOT30
}

fn cubic_measure_segment_count(points: [(f32, f32); 4]) -> u32 {
    wangs_cubic(points, TRIM_CONTOUR_INV_TOLERANCE)
        .ceil()
        .ceil()
        .min(TRIM_CONTOUR_MAX_SEGMENTS as f32) as u32
}

fn wangs_cubic(points: [(f32, f32); 4], precision: f32) -> f32 {
    let first = vector_length_squared((
        points[0].0 - 2.0 * points[1].0 + points[2].0,
        points[0].1 - 2.0 * points[1].1 + points[2].1,
    ));
    let second = vector_length_squared((
        points[1].0 - 2.0 * points[2].0 + points[3].0,
        points[1].1 - 2.0 * points[2].1 + points[3].1,
    ));
    let length_term_pow2 = 9.0 * 4.0 / 64.0 * precision * precision;
    (first.max(second) * length_term_pow2).sqrt().sqrt()
}

fn eval_cubic(points: [(f32, f32); 4], t: f32) -> (f32, f32) {
    let a = (
        points[3].0 + 3.0 * (points[1].0 - points[2].0) - points[0].0,
        points[3].1 + 3.0 * (points[1].1 - points[2].1) - points[0].1,
    );
    let b = (
        3.0 * (points[2].0 - 2.0 * points[1].0 + points[0].0),
        3.0 * (points[2].1 - 2.0 * points[1].1 + points[0].1),
    );
    let c = (
        3.0 * (points[1].0 - points[0].0),
        3.0 * (points[1].1 - points[0].1),
    );
    (
        ((a.0 * t + b.0) * t + c.0) * t + points[0].0,
        ((a.1 * t + b.1) * t + c.1) * t + points[0].1,
    )
}

fn cubic_extract(points: [(f32, f32); 4], start_t: f32, end_t: f32) -> [(f32, f32); 4] {
    if start_t == 0.0 && end_t == 1.0 {
        points
    } else if start_t == 0.0 {
        let chopped = cubic_subdivide(points, end_t);
        [chopped[0], chopped[1], chopped[2], chopped[3]]
    } else if end_t == 1.0 {
        let chopped = cubic_subdivide(points, start_t);
        [chopped[3], chopped[4], chopped[5], chopped[6]]
    } else {
        let chopped = cubic_subdivide(points, end_t);
        let chopped_again = cubic_subdivide(
            [chopped[0], chopped[1], chopped[2], chopped[3]],
            start_t / end_t,
        );
        [
            chopped_again[3],
            chopped_again[4],
            chopped_again[5],
            chopped_again[6],
        ]
    }
}

fn cubic_subdivide(points: [(f32, f32); 4], t: f32) -> [(f32, f32); 7] {
    let ab = lerp_point(points[0], points[1], t);
    let bc = lerp_point(points[1], points[2], t);
    let cd = lerp_point(points[2], points[3], t);
    let abc = lerp_point(ab, bc, t);
    let bcd = lerp_point(bc, cd, t);
    [
        points[0],
        ab,
        abc,
        lerp_point(abc, bcd, t),
        bcd,
        cd,
        points[3],
    ]
}

fn distance(from: (f32, f32), to: (f32, f32)) -> f32 {
    let x = to.0 - from.0;
    let y = to.1 - from.1;
    (x * x + y * y).sqrt()
}

fn distance_squared(from: (f32, f32), to: (f32, f32)) -> f32 {
    vector_length_squared((to.0 - from.0, to.1 - from.1))
}

fn vector_length_squared(vector: (f32, f32)) -> f32 {
    vector.0 * vector.0 + vector.1 * vector.1
}

fn lerp_point(from: (f32, f32), to: (f32, f32), t: f32) -> (f32, f32) {
    if t == 0.0 {
        return from;
    }
    if t == 1.0 {
        return to;
    }
    (from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t)
}

fn color_modulate_opacity(color: u32, opacity: f32) -> u32 {
    let alpha = ((color >> 24) & 0xff) as f32 / 255.0;
    let alpha = opacity_to_alpha(alpha * opacity);
    (color & 0x00ff_ffff) | (u32::from(alpha) << 24)
}

fn color_lerp(from: u32, to: u32, mix: f32) -> u32 {
    let channel = |shift: u32| {
        let from = ((from >> shift) & 0xff) as f32;
        let to = ((to >> shift) & 0xff) as f32;
        ((from * (1.0 - mix) + to * mix).clamp(0.0, 255.0)).round() as u32
    };
    (channel(24) << 24) | (channel(16) << 16) | (channel(8) << 8) | channel(0)
}

fn opacity_to_alpha(opacity: f32) -> u8 {
    (255.0 * opacity.clamp(0.0, 1.0)).round() as u8
}

fn path_commands(
    path: &PathGeometryNode,
    path_kind: ShapePaintPathKind,
    transform: Mat2D,
    weighted_context: Option<&WeightedPathContext>,
) -> Vec<RuntimePathCommand> {
    match path.type_name {
        "Ellipse" => ellipse_path_commands(path, path_kind, transform),
        "PointsPath" => points_path_commands(path, path_kind, transform, weighted_context),
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
    weighted_context: Option<&WeightedPathContext>,
) -> Vec<RuntimePathCommand> {
    if path.type_name != "PointsPath" || path.vertices.len() < 2 {
        return Vec::new();
    }
    if path
        .vertices
        .iter()
        .any(|vertex| vertex.weight_local.is_some())
    {
        let Some(weighted_context) = weighted_context else {
            return Vec::new();
        };
        let Some(deformed_path) = deformed_points_path(path, weighted_context) else {
            return Vec::new();
        };
        return points_path_commands(&deformed_path, path_kind, transform, None);
    }
    if path
        .vertices
        .iter()
        .any(|vertex| !is_supported_point_path_vertex(vertex))
    {
        return Vec::new();
    }
    let reverse_for_clockwise_fill = path_kind == ShapePaintPathKind::LocalClockwise
        && path_needs_clockwise_reversal(path, transform);

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

    if reverse_for_clockwise_fill {
        path_commands_backwards(&commands)
    } else {
        commands
    }
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

    points_path_commands(&virtual_path, path_kind, transform, None)
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
    let reverse_for_clockwise_fill = path_kind == ShapePaintPathKind::LocalClockwise
        && path_needs_clockwise_reversal(path, transform);

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
    if reverse_for_clockwise_fill {
        path_commands_backwards(&commands)
    } else {
        commands
    }
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

    points_path_commands(&virtual_path, path_kind, transform, None)
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
    match vertex.type_name {
        "StraightVertex" => true,
        "CubicDetachedVertex" | "CubicAsymmetricVertex" | "CubicMirroredVertex" => true,
        _ => false,
    }
}

#[derive(Debug, Clone)]
struct WeightedPathContext {
    skin_world_transform: Mat2D,
    bone_transforms: Vec<Mat2D>,
}

impl WeightedPathContext {
    fn deform_point(&self, point: (f32, f32), indices: u32, weights: u32) -> Option<(f32, f32)> {
        let mut blended = [0.0; 6];
        for index in 0..4 {
            let shift = index * 8;
            let weight = ((weights >> shift) & 0xff) as u8;
            if weight == 0 {
                continue;
            }
            let bone_index = ((indices >> shift) & 0xff) as usize;
            let bone_transform = self.bone_transforms.get(bone_index)?;
            let normalized_weight = f32::from(weight) / 255.0;
            for (target, value) in blended.iter_mut().zip(bone_transform.0) {
                *target += value * normalized_weight;
            }
        }

        let (x, y) = self.skin_world_transform.transform_point(point.0, point.1);
        Some(Mat2D(blended).transform_point(x, y))
    }
}

fn deformed_points_path(
    path: &PathGeometryNode,
    weighted_context: &WeightedPathContext,
) -> Option<PathGeometryNode> {
    let mut deformed_path = path.clone();
    for vertex in &mut deformed_path.vertices {
        if vertex.weight_local.is_none() {
            continue;
        }

        let original = vertex.clone();
        if !is_supported_point_path_vertex(&original) {
            return None;
        }

        let translation = weighted_context.deform_point(
            vertex_translation(&original),
            original.weight_indices.unwrap_or(1),
            original.weight_values.unwrap_or(255),
        )?;
        vertex.x = translation.0;
        vertex.y = translation.1;

        if let Some((in_point, out_point)) = cubic_vertex_points(&original) {
            let in_point = weighted_context.deform_point(
                in_point,
                original.weight_in_indices.unwrap_or(1),
                original.weight_in_values.unwrap_or(255),
            )?;
            let out_point = weighted_context.deform_point(
                out_point,
                original.weight_out_indices.unwrap_or(1),
                original.weight_out_values.unwrap_or(255),
            )?;
            set_detached_cubic_points(vertex, translation, in_point, out_point);
        }

        clear_vertex_weight(vertex);
    }

    Some(deformed_path)
}

fn set_detached_cubic_points(
    vertex: &mut PathVertexNode,
    translation: (f32, f32),
    in_point: (f32, f32),
    out_point: (f32, f32),
) {
    let in_vector = subtract_point(in_point, translation);
    let out_vector = subtract_point(out_point, translation);
    vertex.type_name = "CubicDetachedVertex";
    vertex.radius = 0.0;
    vertex.rotation = 0.0;
    vertex.distance = 0.0;
    vertex.in_rotation = in_vector.1.atan2(in_vector.0);
    vertex.in_distance = vector_length(in_vector);
    vertex.out_rotation = out_vector.1.atan2(out_vector.0);
    vertex.out_distance = vector_length(out_vector);
}

fn clear_vertex_weight(vertex: &mut PathVertexNode) {
    vertex.weight_local = None;
    vertex.weight_global = None;
    vertex.weight_type_name = None;
    vertex.weight_values = None;
    vertex.weight_indices = None;
    vertex.weight_in_values = None;
    vertex.weight_in_indices = None;
    vertex.weight_out_values = None;
    vertex.weight_out_indices = None;
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
    let length = vector_length(vector);
    if length > 0.0 {
        ((vector.0 / length, vector.1 / length), length)
    } else {
        (vector, length)
    }
}

fn vector_length(vector: (f32, f32)) -> f32 {
    (vector.0 * vector.0 + vector.1 * vector.1).sqrt()
}

fn scale_and_add_point(point: (f32, f32), vector: (f32, f32), scale: f32) -> (f32, f32) {
    // Mirrors C++ Vec2D::scaleAndAdd after compiler contraction; rounded
    // midpoint pruning can depend on the one-ulp split this preserves.
    (
        vector.0.mul_add(scale, point.0),
        vector.1.mul_add(scale, point.1),
    )
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
    let (x, y) = transform.map_point(point.0, point.1);
    commands.push(RuntimePathCommand::Move { x, y });
}

fn push_line(commands: &mut Vec<RuntimePathCommand>, transform: Mat2D, point: (f32, f32)) {
    let (x, y) = transform.map_point(point.0, point.1);
    commands.push(RuntimePathCommand::Line { x, y });
}

fn push_cubic(
    commands: &mut Vec<RuntimePathCommand>,
    transform: Mat2D,
    point1: (f32, f32),
    point2: (f32, f32),
    point3: (f32, f32),
) {
    let (x1, y1) = transform.map_point(point1.0, point1.1);
    let (x2, y2) = transform.map_point(point2.0, point2.1);
    let (x3, y3) = transform.map_point(point3.0, point3.1);
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
        definition.is_a("Shape") || matches!(definition.name, "TextInputDrawable")
    })
}

fn sorted_drawable_is_nested_artboard(type_name: &str) -> bool {
    definition_by_name(type_name).is_some_and(|definition| definition.is_a("NestedArtboard"))
}

#[derive(Debug, Clone)]
pub struct RuntimeOwnedViewModelInstance {
    view_model_index: usize,
    property_names: Vec<(String, usize)>,
    numbers: Vec<RuntimeOwnedViewModelNumber>,
    booleans: Vec<RuntimeOwnedViewModelBoolean>,
    strings: Vec<RuntimeOwnedViewModelString>,
    colors: Vec<RuntimeOwnedViewModelColor>,
    enums: Vec<RuntimeOwnedViewModelEnum>,
    symbol_list_indices: Vec<RuntimeOwnedViewModelSymbolListIndex>,
    lists: Vec<RuntimeOwnedViewModelList>,
    assets: Vec<RuntimeOwnedViewModelAsset>,
    artboards: Vec<RuntimeOwnedViewModelArtboard>,
    triggers: Vec<RuntimeOwnedViewModelTrigger>,
    view_models: Vec<RuntimeOwnedViewModelViewModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelNumberSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelNumberSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelBooleanSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelBooleanSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelStringSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelStringSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelColorSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelColorSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelEnumSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelEnumSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelSymbolListIndexSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelSymbolListIndexSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelAssetSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelAssetSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelArtboardSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelArtboardSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelTriggerSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelTriggerSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelListSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelListSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelViewModelSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelViewModelSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone)]
struct RuntimeOwnedViewModelNumber {
    property_index: usize,
    value: f32,
}

#[derive(Debug, Clone)]
struct RuntimeOwnedViewModelBoolean {
    property_index: usize,
    value: bool,
}

#[derive(Debug, Clone)]
struct RuntimeOwnedViewModelString {
    property_index: usize,
    value: Vec<u8>,
}

#[derive(Debug, Clone)]
struct RuntimeOwnedViewModelColor {
    property_index: usize,
    value: u32,
}

#[derive(Debug, Clone)]
struct RuntimeOwnedViewModelEnum {
    property_index: usize,
    value: u64,
}

#[derive(Debug, Clone)]
struct RuntimeOwnedViewModelSymbolListIndex {
    property_index: usize,
    value: u64,
}

#[derive(Debug, Clone)]
struct RuntimeOwnedViewModelList {
    property_index: usize,
    item_count: usize,
}

#[derive(Debug, Clone)]
struct RuntimeOwnedViewModelAsset {
    property_index: usize,
    value: u64,
}

#[derive(Debug, Clone)]
struct RuntimeOwnedViewModelArtboard {
    property_index: usize,
    value: u64,
}

#[derive(Debug, Clone)]
struct RuntimeOwnedViewModelTrigger {
    property_index: usize,
    value: u64,
}

#[derive(Debug, Clone)]
struct RuntimeOwnedViewModelViewModel {
    property_index: usize,
    property_name: String,
    value: RuntimeViewModelPointer,
    property_names: Vec<(String, usize)>,
    numbers: Vec<RuntimeOwnedViewModelNumber>,
    imported_numbers: BTreeMap<u32, Vec<RuntimeOwnedViewModelNumber>>,
    booleans: Vec<RuntimeOwnedViewModelBoolean>,
    imported_booleans: BTreeMap<u32, Vec<RuntimeOwnedViewModelBoolean>>,
    strings: Vec<RuntimeOwnedViewModelString>,
    imported_strings: BTreeMap<u32, Vec<RuntimeOwnedViewModelString>>,
    colors: Vec<RuntimeOwnedViewModelColor>,
    imported_colors: BTreeMap<u32, Vec<RuntimeOwnedViewModelColor>>,
    enums: Vec<RuntimeOwnedViewModelEnum>,
    imported_enums: BTreeMap<u32, Vec<RuntimeOwnedViewModelEnum>>,
    symbol_list_indices: Vec<RuntimeOwnedViewModelSymbolListIndex>,
    imported_symbol_list_indices: BTreeMap<u32, Vec<RuntimeOwnedViewModelSymbolListIndex>>,
    lists: Vec<RuntimeOwnedViewModelList>,
    imported_lists: BTreeMap<u32, Vec<RuntimeOwnedViewModelList>>,
    assets: Vec<RuntimeOwnedViewModelAsset>,
    imported_assets: BTreeMap<u32, Vec<RuntimeOwnedViewModelAsset>>,
    artboards: Vec<RuntimeOwnedViewModelArtboard>,
    imported_artboards: BTreeMap<u32, Vec<RuntimeOwnedViewModelArtboard>>,
    triggers: Vec<RuntimeOwnedViewModelTrigger>,
    imported_triggers: BTreeMap<u32, Vec<RuntimeOwnedViewModelTrigger>>,
    view_model_instance_ids: Vec<u32>,
    children: Vec<RuntimeOwnedViewModelViewModel>,
    imported_children: BTreeMap<u32, Vec<RuntimeOwnedViewModelViewModel>>,
}

impl RuntimeOwnedViewModelViewModel {
    fn active_children(&self) -> Option<&[RuntimeOwnedViewModelViewModel]> {
        match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => Some(self.children.as_slice()),
            RuntimeViewModelPointer::Imported { object_id } => {
                self.imported_children.get(&object_id).map(Vec::as_slice)
            }
            _ => None,
        }
    }

    fn generated_children_mut(&mut self) -> Option<&mut Vec<RuntimeOwnedViewModelViewModel>> {
        match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => Some(&mut self.children),
            _ => None,
        }
    }

    fn property_index_by_name(&self, property_name: &str) -> Option<usize> {
        runtime_owned_view_model_property_index_by_name(&self.property_names, property_name)
    }

    fn number_value_by_property_index(&self, property_index: usize) -> Option<f32> {
        self.numbers
            .iter()
            .find(|number| number.property_index == property_index)
            .map(|number| number.value)
    }

    fn active_number_value_by_property_index(&self, property_index: usize) -> Option<f32> {
        match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                self.number_value_by_property_index(property_index)
            }
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_numbers
                .get(&object_id)
                .and_then(|numbers| {
                    numbers
                        .iter()
                        .find(|number| number.property_index == property_index)
                })
                .map(|number| number.value),
            _ => None,
        }
    }

    fn boolean_value_by_property_index(&self, property_index: usize) -> Option<bool> {
        self.booleans
            .iter()
            .find(|boolean| boolean.property_index == property_index)
            .map(|boolean| boolean.value)
    }

    fn active_boolean_value_by_property_index(&self, property_index: usize) -> Option<bool> {
        match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                self.boolean_value_by_property_index(property_index)
            }
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_booleans
                .get(&object_id)
                .and_then(|booleans| {
                    booleans
                        .iter()
                        .find(|boolean| boolean.property_index == property_index)
                })
                .map(|boolean| boolean.value),
            _ => None,
        }
    }

    fn string_value_by_property_index(&self, property_index: usize) -> Option<&[u8]> {
        self.strings
            .iter()
            .find(|string| string.property_index == property_index)
            .map(|string| string.value.as_slice())
    }

    fn active_string_value_by_property_index(&self, property_index: usize) -> Option<&[u8]> {
        match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                self.string_value_by_property_index(property_index)
            }
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_strings
                .get(&object_id)
                .and_then(|strings| {
                    strings
                        .iter()
                        .find(|string| string.property_index == property_index)
                })
                .map(|string| string.value.as_slice()),
            _ => None,
        }
    }

    fn color_value_by_property_index(&self, property_index: usize) -> Option<u32> {
        self.colors
            .iter()
            .find(|color| color.property_index == property_index)
            .map(|color| color.value)
    }

    fn active_color_value_by_property_index(&self, property_index: usize) -> Option<u32> {
        match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                self.color_value_by_property_index(property_index)
            }
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_colors
                .get(&object_id)
                .and_then(|colors| {
                    colors
                        .iter()
                        .find(|color| color.property_index == property_index)
                })
                .map(|color| color.value),
            _ => None,
        }
    }

    fn enum_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        self.enums
            .iter()
            .find(|enum_value| enum_value.property_index == property_index)
            .map(|enum_value| enum_value.value)
    }

    fn active_enum_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                self.enum_value_by_property_index(property_index)
            }
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_enums
                .get(&object_id)
                .and_then(|enums| {
                    enums
                        .iter()
                        .find(|enum_value| enum_value.property_index == property_index)
                })
                .map(|enum_value| enum_value.value),
            _ => None,
        }
    }

    fn symbol_list_index_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        self.symbol_list_indices
            .iter()
            .find(|symbol_list_index| symbol_list_index.property_index == property_index)
            .map(|symbol_list_index| symbol_list_index.value)
    }

    fn active_symbol_list_index_value_by_property_index(
        &self,
        property_index: usize,
    ) -> Option<u64> {
        match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                self.symbol_list_index_value_by_property_index(property_index)
            }
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_symbol_list_indices
                .get(&object_id)
                .and_then(|symbol_list_indices| {
                    symbol_list_indices.iter().find(|symbol_list_index| {
                        symbol_list_index.property_index == property_index
                    })
                })
                .map(|symbol_list_index| symbol_list_index.value),
            _ => None,
        }
    }

    fn list_item_count_by_property_index(&self, property_index: usize) -> Option<usize> {
        self.lists
            .iter()
            .find(|list| list.property_index == property_index)
            .map(|list| list.item_count)
    }

    fn active_list_item_count_by_property_index(&self, property_index: usize) -> Option<usize> {
        match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                self.list_item_count_by_property_index(property_index)
            }
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_lists
                .get(&object_id)
                .and_then(|lists| {
                    lists
                        .iter()
                        .find(|list| list.property_index == property_index)
                })
                .map(|list| list.item_count),
            _ => None,
        }
    }

    fn asset_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        self.assets
            .iter()
            .find(|asset| asset.property_index == property_index)
            .map(|asset| asset.value)
    }

    fn active_asset_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                self.asset_value_by_property_index(property_index)
            }
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_assets
                .get(&object_id)
                .and_then(|assets| {
                    assets
                        .iter()
                        .find(|asset| asset.property_index == property_index)
                })
                .map(|asset| asset.value),
            _ => None,
        }
    }

    fn artboard_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        self.artboards
            .iter()
            .find(|artboard| artboard.property_index == property_index)
            .map(|artboard| artboard.value)
    }

    fn active_artboard_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                self.artboard_value_by_property_index(property_index)
            }
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_artboards
                .get(&object_id)
                .and_then(|artboards| {
                    artboards
                        .iter()
                        .find(|artboard| artboard.property_index == property_index)
                })
                .map(|artboard| artboard.value),
            _ => None,
        }
    }

    fn trigger_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        self.triggers
            .iter()
            .find(|trigger| trigger.property_index == property_index)
            .map(|trigger| trigger.value)
    }

    fn active_trigger_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                self.trigger_value_by_property_index(property_index)
            }
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_triggers
                .get(&object_id)
                .and_then(|triggers| {
                    triggers
                        .iter()
                        .find(|trigger| trigger.property_index == property_index)
                })
                .map(|trigger| trigger.value),
            _ => None,
        }
    }

    fn set_number_by_property_name(&mut self, property_name: &str, value: f32) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_number_by_property_index(property_index, value)
    }

    fn set_number_by_property_index(&mut self, property_index: usize, value: f32) -> bool {
        let Some(number) = self
            .numbers
            .iter_mut()
            .find(|number| number.property_index == property_index)
        else {
            return false;
        };
        if number.value == value {
            return false;
        }
        number.value = value;
        true
    }

    fn set_boolean_by_property_name(&mut self, property_name: &str, value: bool) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_boolean_by_property_index(property_index, value)
    }

    fn set_boolean_by_property_index(&mut self, property_index: usize, value: bool) -> bool {
        let Some(boolean) = self
            .booleans
            .iter_mut()
            .find(|boolean| boolean.property_index == property_index)
        else {
            return false;
        };
        if boolean.value == value {
            return false;
        }
        boolean.value = value;
        true
    }

    fn set_string_by_property_name(&mut self, property_name: &str, value: &[u8]) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_string_by_property_index(property_index, value)
    }

    fn set_string_by_property_index(&mut self, property_index: usize, value: &[u8]) -> bool {
        let Some(string) = self
            .strings
            .iter_mut()
            .find(|string| string.property_index == property_index)
        else {
            return false;
        };
        if string.value == value {
            return false;
        }
        string.value = value.to_vec();
        true
    }

    fn set_color_by_property_name(&mut self, property_name: &str, value: u32) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_color_by_property_index(property_index, value)
    }

    fn set_color_by_property_index(&mut self, property_index: usize, value: u32) -> bool {
        let Some(color) = self
            .colors
            .iter_mut()
            .find(|color| color.property_index == property_index)
        else {
            return false;
        };
        if color.value == value {
            return false;
        }
        color.value = value;
        true
    }

    fn set_enum_by_property_name(&mut self, property_name: &str, value: u64) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_enum_by_property_index(property_index, value)
    }

    fn set_enum_by_property_index(&mut self, property_index: usize, value: u64) -> bool {
        let Some(enum_value) = self
            .enums
            .iter_mut()
            .find(|enum_value| enum_value.property_index == property_index)
        else {
            return false;
        };
        if enum_value.value == value {
            return false;
        }
        enum_value.value = value;
        true
    }

    fn set_symbol_list_index_by_property_name(&mut self, property_name: &str, value: u64) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_symbol_list_index_by_property_index(property_index, value)
    }

    fn set_symbol_list_index_by_property_index(
        &mut self,
        property_index: usize,
        value: u64,
    ) -> bool {
        let Some(symbol_list_index) = self
            .symbol_list_indices
            .iter_mut()
            .find(|symbol_list_index| symbol_list_index.property_index == property_index)
        else {
            return false;
        };
        if symbol_list_index.value == value {
            return false;
        }
        symbol_list_index.value = value;
        true
    }

    fn set_list_item_count_by_property_name(
        &mut self,
        property_name: &str,
        item_count: usize,
    ) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_list_item_count_by_property_index(property_index, item_count)
    }

    fn set_list_item_count_by_property_index(
        &mut self,
        property_index: usize,
        item_count: usize,
    ) -> bool {
        let Some(list) = self
            .lists
            .iter_mut()
            .find(|list| list.property_index == property_index)
        else {
            return false;
        };
        if list.item_count == item_count {
            return false;
        }
        list.item_count = item_count;
        true
    }

    fn set_asset_by_property_name(&mut self, property_name: &str, value: u64) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_asset_by_property_index(property_index, value)
    }

    fn set_asset_by_property_index(&mut self, property_index: usize, value: u64) -> bool {
        let Some(asset) = self
            .assets
            .iter_mut()
            .find(|asset| asset.property_index == property_index)
        else {
            return false;
        };
        if asset.value == value {
            return false;
        }
        asset.value = value;
        true
    }

    fn set_artboard_by_property_name(&mut self, property_name: &str, value: u64) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_artboard_by_property_index(property_index, value)
    }

    fn set_artboard_by_property_index(&mut self, property_index: usize, value: u64) -> bool {
        let Some(artboard) = self
            .artboards
            .iter_mut()
            .find(|artboard| artboard.property_index == property_index)
        else {
            return false;
        };
        if artboard.value == value {
            return false;
        }
        artboard.value = value;
        true
    }

    fn set_trigger_by_property_name(&mut self, property_name: &str, value: u64) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_trigger_by_property_index(property_index, value)
    }

    fn set_trigger_by_property_index(&mut self, property_index: usize, value: u64) -> bool {
        let Some(trigger) = self
            .triggers
            .iter_mut()
            .find(|trigger| trigger.property_index == property_index)
        else {
            return false;
        };
        if trigger.value == value {
            return false;
        }
        trigger.value = value;
        true
    }
}

fn runtime_owned_view_model_path_key(path: &[usize]) -> u64 {
    let mut key = 0xcbf29ce484222325u64;
    for segment in path {
        key ^= *segment as u64;
        key = key.wrapping_mul(0x100000001b3);
    }
    key
}

fn runtime_owned_view_model_property_index_by_name(
    property_names: &[(String, usize)],
    property_name: &str,
) -> Option<usize> {
    property_names
        .iter()
        .find_map(|(name, index)| (name == property_name).then_some(*index))
}

fn runtime_owned_view_model_property_names(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<(String, usize)> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .map(|(property_index, property)| {
                    (
                        property
                            .string_property("name")
                            .unwrap_or_default()
                            .to_owned(),
                        property_index,
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_imported_view_model_number_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name(
        file,
        view_model_index,
        property_name,
        "ViewModelPropertyNumber",
    )
}

fn runtime_imported_view_model_number_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &["ViewModelPropertyNumber"],
    )
}

fn runtime_default_view_model_number_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_number_property_path_for_name(file, 0, property_name)
}

fn runtime_default_view_model_number_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_number_property_path_for_name_path(file, 0, property_path)
}

fn runtime_imported_view_model_boolean_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name(
        file,
        view_model_index,
        property_name,
        "ViewModelPropertyBoolean",
    )
}

fn runtime_imported_view_model_boolean_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &["ViewModelPropertyBoolean"],
    )
}

fn runtime_default_view_model_boolean_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_boolean_property_path_for_name(file, 0, property_name)
}

fn runtime_default_view_model_boolean_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_boolean_property_path_for_name_path(file, 0, property_path)
}

fn runtime_imported_view_model_string_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name(
        file,
        view_model_index,
        property_name,
        "ViewModelPropertyString",
    )
}

fn runtime_imported_view_model_string_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &["ViewModelPropertyString"],
    )
}

fn runtime_default_view_model_string_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_string_property_path_for_name(file, 0, property_name)
}

fn runtime_default_view_model_string_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_string_property_path_for_name_path(file, 0, property_path)
}

fn runtime_imported_view_model_color_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name(
        file,
        view_model_index,
        property_name,
        "ViewModelPropertyColor",
    )
}

fn runtime_imported_view_model_color_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &["ViewModelPropertyColor"],
    )
}

fn runtime_default_view_model_color_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_color_property_path_for_name(file, 0, property_name)
}

fn runtime_default_view_model_color_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_color_property_path_for_name_path(file, 0, property_path)
}

fn runtime_imported_view_model_enum_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_type_names(
        file,
        view_model_index,
        property_name,
        &[
            "ViewModelPropertyEnum",
            "ViewModelPropertyEnumCustom",
            "ViewModelPropertyEnumSystem",
        ],
    )
}

fn runtime_imported_view_model_enum_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &[
            "ViewModelPropertyEnum",
            "ViewModelPropertyEnumCustom",
            "ViewModelPropertyEnumSystem",
        ],
    )
}

fn runtime_default_view_model_enum_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_enum_property_path_for_name(file, 0, property_name)
}

fn runtime_default_view_model_enum_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_enum_property_path_for_name_path(file, 0, property_path)
}

fn runtime_imported_view_model_symbol_list_index_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name(
        file,
        view_model_index,
        property_name,
        "ViewModelPropertySymbolListIndex",
    )
}

fn runtime_imported_view_model_symbol_list_index_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &["ViewModelPropertySymbolListIndex"],
    )
}

fn runtime_default_view_model_symbol_list_index_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_symbol_list_index_property_path_for_name(file, 0, property_name)
}

fn runtime_default_view_model_symbol_list_index_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_symbol_list_index_property_path_for_name_path(
        file,
        0,
        property_path,
    )
}

fn runtime_imported_view_model_asset_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_type_names(
        file,
        view_model_index,
        property_name,
        &["ViewModelPropertyAsset", "ViewModelPropertyAssetImage"],
    )
}

fn runtime_imported_view_model_asset_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &["ViewModelPropertyAsset", "ViewModelPropertyAssetImage"],
    )
}

fn runtime_default_view_model_asset_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_asset_property_path_for_name(file, 0, property_name)
}

fn runtime_default_view_model_asset_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_asset_property_path_for_name_path(file, 0, property_path)
}

fn runtime_imported_view_model_artboard_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name(
        file,
        view_model_index,
        property_name,
        "ViewModelPropertyArtboard",
    )
}

fn runtime_imported_view_model_artboard_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &["ViewModelPropertyArtboard"],
    )
}

fn runtime_default_view_model_artboard_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_artboard_property_path_for_name(file, 0, property_name)
}

fn runtime_default_view_model_artboard_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_artboard_property_path_for_name_path(file, 0, property_path)
}

fn runtime_imported_view_model_trigger_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name(
        file,
        view_model_index,
        property_name,
        "ViewModelPropertyTrigger",
    )
}

fn runtime_imported_view_model_trigger_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &["ViewModelPropertyTrigger"],
    )
}

fn runtime_default_view_model_trigger_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_trigger_property_path_for_name(file, 0, property_name)
}

fn runtime_default_view_model_trigger_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_trigger_property_path_for_name_path(file, 0, property_path)
}

fn runtime_imported_view_model_list_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name(
        file,
        view_model_index,
        property_name,
        "ViewModelPropertyList",
    )
}

fn runtime_imported_view_model_list_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &["ViewModelPropertyList"],
    )
}

fn runtime_default_view_model_list_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_list_property_path_for_name(file, 0, property_name)
}

fn runtime_default_view_model_list_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_list_property_path_for_name_path(file, 0, property_path)
}

fn runtime_imported_view_model_view_model_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name(
        file,
        view_model_index,
        property_name,
        "ViewModelPropertyViewModel",
    )
}

fn runtime_imported_view_model_view_model_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &["ViewModelPropertyViewModel"],
    )
}

fn runtime_imported_view_model_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
    property_type_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_type_names(
        file,
        view_model_index,
        property_name,
        &[property_type_name],
    )
}

fn runtime_imported_view_model_property_path_for_type_names(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
    property_type_names: &[&str],
) -> Option<Vec<u32>> {
    if property_name.is_empty() {
        return None;
    }
    let view_model = file.view_model(view_model_index)?;
    view_model
        .properties
        .into_iter()
        .enumerate()
        .find_map(|(property_index, property)| {
            if !property_type_names.contains(&property.type_name) {
                return None;
            }
            if property.string_property("name")? != property_name {
                return None;
            }
            Some(vec![
                u32::try_from(view_model_index).ok()?,
                u32::try_from(property_index).ok()?,
            ])
        })
}

fn runtime_imported_view_model_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
    property_type_names: &[&str],
) -> Option<Vec<u32>> {
    let property_names = property_path.split('/').collect::<Vec<_>>();
    if property_names.is_empty() || property_names.iter().any(|segment| segment.is_empty()) {
        return None;
    }

    let mut current_view_model_index = view_model_index;
    let mut path = vec![u32::try_from(view_model_index).ok()?];
    for (property_name_index, property_name) in property_names.iter().enumerate() {
        let view_model = file.view_model(current_view_model_index)?;
        let (property_index, property) = view_model
            .properties
            .into_iter()
            .enumerate()
            .find(|(_, property)| property.string_property("name") == Some(*property_name))?;
        path.push(u32::try_from(property_index).ok()?);
        if property_name_index + 1 == property_names.len() {
            return property_type_names
                .contains(&property.type_name)
                .then_some(path);
        }
        if property.type_name != "ViewModelPropertyViewModel" {
            return None;
        }
        current_view_model_index =
            usize::try_from(property.uint_property("viewModelReferenceId")?).ok()?;
    }

    None
}

fn runtime_view_model_reference_index_for_property_path(
    file: &RuntimeFile,
    property_path: &[u32],
) -> Option<usize> {
    let mut current_view_model_index = usize::try_from(*property_path.first()?).ok()?;
    let property_indices = property_path.get(1..)?;
    if property_indices.is_empty() {
        return None;
    }

    for (segment_index, property_index) in property_indices.iter().enumerate() {
        let view_model = file.view_model(current_view_model_index)?;
        let property = view_model
            .properties
            .into_iter()
            .nth(usize::try_from(*property_index).ok()?)?;
        if property.type_name != "ViewModelPropertyViewModel" {
            return None;
        }
        let referenced_view_model_index =
            usize::try_from(property.uint_property("viewModelReferenceId")?).ok()?;
        if segment_index + 1 == property_indices.len() {
            return Some(referenced_view_model_index);
        }
        current_view_model_index = referenced_view_model_index;
    }

    None
}

fn runtime_owned_view_model_numbers(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelNumber> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    (property.type_name == "ViewModelPropertyNumber").then_some(
                        RuntimeOwnedViewModelNumber {
                            property_index,
                            value: 0.0,
                        },
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_numbers_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelNumber> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    if property.type_name != "ViewModelPropertyNumber" {
                        return None;
                    }
                    let path = [
                        u32::try_from(view_model_index).ok()?,
                        u32::try_from(property_index).ok()?,
                    ];
                    let source = file.data_context_view_model_property_for_instance(
                        view_model_instance,
                        &path,
                    )?;
                    let value = file.view_model_instance_number_value_for_object(source)?;
                    Some(RuntimeOwnedViewModelNumber {
                        property_index,
                        value,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_imported_numbers(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelNumber>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_numbers_for_instance(
                            file,
                            view_model_index,
                            instance.object,
                        ),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_booleans(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelBoolean> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    (property.type_name == "ViewModelPropertyBoolean").then_some(
                        RuntimeOwnedViewModelBoolean {
                            property_index,
                            value: false,
                        },
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_booleans_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelBoolean> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    if property.type_name != "ViewModelPropertyBoolean" {
                        return None;
                    }
                    let path = [
                        u32::try_from(view_model_index).ok()?,
                        u32::try_from(property_index).ok()?,
                    ];
                    let source = file.data_context_view_model_property_for_instance(
                        view_model_instance,
                        &path,
                    )?;
                    let value = file.view_model_instance_boolean_value_for_object(source)?;
                    Some(RuntimeOwnedViewModelBoolean {
                        property_index,
                        value,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_imported_booleans(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelBoolean>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_booleans_for_instance(
                            file,
                            view_model_index,
                            instance.object,
                        ),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_strings(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelString> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    (property.type_name == "ViewModelPropertyString").then_some(
                        RuntimeOwnedViewModelString {
                            property_index,
                            value: Vec::new(),
                        },
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_strings_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelString> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    if property.type_name != "ViewModelPropertyString" {
                        return None;
                    }
                    let path = [
                        u32::try_from(view_model_index).ok()?,
                        u32::try_from(property_index).ok()?,
                    ];
                    let source = file.data_context_view_model_property_for_instance(
                        view_model_instance,
                        &path,
                    )?;
                    let value = file.view_model_instance_string_value_for_object(source)?;
                    Some(RuntimeOwnedViewModelString {
                        property_index,
                        value: value.as_bytes().to_vec(),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_imported_strings(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelString>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_strings_for_instance(
                            file,
                            view_model_index,
                            instance.object,
                        ),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_colors(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelColor> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    (property.type_name == "ViewModelPropertyColor").then_some(
                        RuntimeOwnedViewModelColor {
                            property_index,
                            value: 0,
                        },
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_colors_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelColor> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    if property.type_name != "ViewModelPropertyColor" {
                        return None;
                    }
                    let path = [
                        u32::try_from(view_model_index).ok()?,
                        u32::try_from(property_index).ok()?,
                    ];
                    let source = file.data_context_view_model_property_for_instance(
                        view_model_instance,
                        &path,
                    )?;
                    let value = file.view_model_instance_color_value_for_object(source)?;
                    Some(RuntimeOwnedViewModelColor {
                        property_index,
                        value,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_imported_colors(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelColor>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_colors_for_instance(
                            file,
                            view_model_index,
                            instance.object,
                        ),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_enums(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelEnum> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    matches!(
                        property.type_name,
                        "ViewModelPropertyEnum"
                            | "ViewModelPropertyEnumCustom"
                            | "ViewModelPropertyEnumSystem"
                    )
                    .then_some(RuntimeOwnedViewModelEnum {
                        property_index,
                        value: 0,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_enums_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelEnum> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    if property.type_name != "ViewModelPropertyEnumCustom" {
                        return None;
                    }
                    let path = [
                        u32::try_from(view_model_index).ok()?,
                        u32::try_from(property_index).ok()?,
                    ];
                    let source = file.data_context_view_model_property_for_instance(
                        view_model_instance,
                        &path,
                    )?;
                    let value = file.view_model_instance_enum_value_index_for_object(source)?;
                    Some(RuntimeOwnedViewModelEnum {
                        property_index,
                        value: u64::try_from(value).ok()?,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_imported_enums(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelEnum>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_enums_for_instance(
                            file,
                            view_model_index,
                            instance.object,
                        ),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_symbol_list_indices(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelSymbolListIndex> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    (property.type_name == "ViewModelPropertySymbolListIndex").then_some(
                        RuntimeOwnedViewModelSymbolListIndex {
                            property_index,
                            value: 0,
                        },
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_symbol_list_indices_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelSymbolListIndex> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    if property.type_name != "ViewModelPropertySymbolListIndex" {
                        return None;
                    }
                    let path = [
                        u32::try_from(view_model_index).ok()?,
                        u32::try_from(property_index).ok()?,
                    ];
                    let source = file.data_context_view_model_property_for_instance(
                        view_model_instance,
                        &path,
                    )?;
                    let value =
                        file.view_model_instance_symbol_list_index_value_for_object(source)?;
                    Some(RuntimeOwnedViewModelSymbolListIndex {
                        property_index,
                        value,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_imported_symbol_list_indices(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelSymbolListIndex>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_symbol_list_indices_for_instance(
                            file,
                            view_model_index,
                            instance.object,
                        ),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_lists(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelList> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    (property.type_name == "ViewModelPropertyList").then_some(
                        RuntimeOwnedViewModelList {
                            property_index,
                            item_count: 0,
                        },
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_lists_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelList> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    if property.type_name != "ViewModelPropertyList" {
                        return None;
                    }
                    let path = [
                        u32::try_from(view_model_index).ok()?,
                        u32::try_from(property_index).ok()?,
                    ];
                    let source = file.data_context_view_model_property_for_instance(
                        view_model_instance,
                        &path,
                    )?;
                    let item_count = file.view_model_instance_list_size_for_object(source)?;
                    Some(RuntimeOwnedViewModelList {
                        property_index,
                        item_count,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_imported_lists(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelList>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_lists_for_instance(
                            file,
                            view_model_index,
                            instance.object,
                        ),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_assets(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelAsset> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    matches!(
                        property.type_name,
                        "ViewModelPropertyAsset" | "ViewModelPropertyAssetImage"
                    )
                    .then_some(RuntimeOwnedViewModelAsset {
                        property_index,
                        value: 0,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_assets_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelAsset> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    if !matches!(
                        property.type_name,
                        "ViewModelPropertyAsset" | "ViewModelPropertyAssetImage"
                    ) {
                        return None;
                    }
                    let path = [
                        u32::try_from(view_model_index).ok()?,
                        u32::try_from(property_index).ok()?,
                    ];
                    let source = file.data_context_view_model_property_for_instance(
                        view_model_instance,
                        &path,
                    )?;
                    let value = file.view_model_instance_asset_index_for_object(source)?;
                    Some(RuntimeOwnedViewModelAsset {
                        property_index,
                        value,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_imported_assets(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelAsset>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_assets_for_instance(
                            file,
                            view_model_index,
                            instance.object,
                        ),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_artboards(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelArtboard> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    (property.type_name == "ViewModelPropertyArtboard").then_some(
                        RuntimeOwnedViewModelArtboard {
                            property_index,
                            value: 0,
                        },
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_artboards_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelArtboard> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    if property.type_name != "ViewModelPropertyArtboard" {
                        return None;
                    }
                    let path = [
                        u32::try_from(view_model_index).ok()?,
                        u32::try_from(property_index).ok()?,
                    ];
                    let source = file.data_context_view_model_property_for_instance(
                        view_model_instance,
                        &path,
                    )?;
                    let value = file.view_model_instance_artboard_index_for_object(source)?;
                    Some(RuntimeOwnedViewModelArtboard {
                        property_index,
                        value,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_imported_artboards(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelArtboard>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_artboards_for_instance(
                            file,
                            view_model_index,
                            instance.object,
                        ),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_triggers(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelTrigger> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    (property.type_name == "ViewModelPropertyTrigger").then_some(
                        RuntimeOwnedViewModelTrigger {
                            property_index,
                            value: 0,
                        },
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_triggers_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelTrigger> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    if property.type_name != "ViewModelPropertyTrigger" {
                        return None;
                    }
                    let path = [
                        u32::try_from(view_model_index).ok()?,
                        u32::try_from(property_index).ok()?,
                    ];
                    let source = file.data_context_view_model_property_for_instance(
                        view_model_instance,
                        &path,
                    )?;
                    let value = file.view_model_instance_trigger_count_for_object(source)?;
                    Some(RuntimeOwnedViewModelTrigger {
                        property_index,
                        value,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_imported_triggers(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelTrigger>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_triggers_for_instance(
                            file,
                            view_model_index,
                            instance.object,
                        ),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_view_model_view_model_property_path_for_names(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &[&str],
) -> Option<Vec<u32>> {
    if property_path.is_empty() {
        return None;
    }

    let mut current_view_model_index = view_model_index;
    let mut path = vec![u32::try_from(view_model_index).ok()?];
    for (segment_index, property_name) in property_path.iter().enumerate() {
        if property_name.is_empty() {
            return None;
        }
        let is_last = segment_index + 1 == property_path.len();
        let view_model = file.view_model(current_view_model_index)?;
        let (property_index, property) =
            view_model
                .properties
                .into_iter()
                .enumerate()
                .find(|(_, property)| {
                    property.type_name == "ViewModelPropertyViewModel"
                        && property.string_property("name") == Some(*property_name)
                })?;
        path.push(u32::try_from(property_index).ok()?);

        if !is_last {
            current_view_model_index = property.uint_property("viewModelReferenceId").and_then(
                |view_model_reference_id| usize::try_from(view_model_reference_id).ok(),
            )?;
        }
    }
    Some(path)
}

fn runtime_view_model_view_model_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    let property_path = property_path.split('/').collect::<Vec<_>>();
    runtime_view_model_view_model_property_path_for_names(file, view_model_index, &property_path)
}

fn runtime_default_view_model_view_model_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_view_model_view_model_property_path_for_names(file, 0, &[property_name])
}

fn runtime_default_view_model_view_model_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_view_model_view_model_property_path_for_name_path(file, 0, property_path)
}

fn runtime_owned_view_model_view_model_children(
    file: &RuntimeFile,
    view_model_index: usize,
    parent_path: &[usize],
    ancestor_view_model_indices: &[usize],
) -> Vec<RuntimeOwnedViewModelViewModel> {
    if ancestor_view_model_indices.contains(&view_model_index) {
        return Vec::new();
    }
    if file.view_model(view_model_index).is_none() {
        return Vec::new();
    }
    let mut child_ancestors = ancestor_view_model_indices.to_vec();
    child_ancestors.push(view_model_index);

    runtime_owned_view_model_property_children(
        file,
        view_model_index,
        None,
        parent_path,
        &child_ancestors,
        true,
    )
}

fn runtime_owned_view_model_view_model_children_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
    parent_path: &[usize],
    ancestor_view_model_indices: &[usize],
) -> Vec<RuntimeOwnedViewModelViewModel> {
    if ancestor_view_model_indices.contains(&view_model_index) {
        return Vec::new();
    }
    let mut child_ancestors = ancestor_view_model_indices.to_vec();
    child_ancestors.push(view_model_index);

    runtime_owned_view_model_property_children(
        file,
        view_model_index,
        Some(view_model_instance),
        parent_path,
        &child_ancestors,
        false,
    )
}

fn runtime_owned_view_model_property_children(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: Option<&RuntimeObject>,
    parent_path: &[usize],
    child_ancestors: &[usize],
    use_generated_defaults: bool,
) -> Vec<RuntimeOwnedViewModelViewModel> {
    let Some(view_model) = file.view_model(view_model_index) else {
        return Vec::new();
    };

    view_model
        .properties
        .into_iter()
        .enumerate()
        .filter_map(|(property_index, property)| {
            if property.type_name != "ViewModelPropertyViewModel" {
                return None;
            }
            let referenced_view_model_index = property
                .uint_property("viewModelReferenceId")
                .and_then(|view_model_reference_id| usize::try_from(view_model_reference_id).ok());
            let referenced_view_model = referenced_view_model_index
                .and_then(|view_model_index| file.view_model(view_model_index));
            let mut path = parent_path.to_vec();
            path.push(property_index);
            let imported_value = view_model_instance
                .and_then(|view_model_instance| {
                    let path = [
                        u32::try_from(view_model_index).ok()?,
                        u32::try_from(property_index).ok()?,
                    ];
                    file.data_context_view_model_instance_for_instance(view_model_instance, &path)
                })
                .map(|reference| RuntimeViewModelPointer::Imported {
                    object_id: reference.object.id,
                });
            let value = if let Some(value) = imported_value {
                value
            } else if use_generated_defaults && referenced_view_model.is_some() {
                RuntimeViewModelPointer::OwnedGenerated {
                    view_model_index,
                    property_index,
                    path_key: runtime_owned_view_model_path_key(&path),
                }
            } else {
                RuntimeViewModelPointer::Null
            };
            let has_referenced_view_model = referenced_view_model.is_some();
            let view_model_instance_ids = referenced_view_model
                .map(|view_model| {
                    view_model
                        .instances
                        .into_iter()
                        .map(|instance| instance.object.id)
                        .collect()
                })
                .unwrap_or_default();
            let children = if has_referenced_view_model {
                referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_view_model_children(
                            file,
                            view_model_index,
                            &path,
                            &child_ancestors,
                        )
                    })
                    .unwrap_or_default()
            } else {
                Vec::new()
            };
            let imported_children = referenced_view_model_index
                .and_then(|referenced_view_model_index| {
                    file.view_model(referenced_view_model_index)
                        .map(|view_model| (referenced_view_model_index, view_model))
                })
                .map(|(referenced_view_model_index, view_model)| {
                    view_model
                        .instances
                        .into_iter()
                        .map(|instance| {
                            (
                                instance.object.id,
                                runtime_owned_view_model_view_model_children_for_instance(
                                    file,
                                    referenced_view_model_index,
                                    instance.object,
                                    &path,
                                    child_ancestors,
                                ),
                            )
                        })
                        .collect()
                })
                .unwrap_or_default();
            Some(RuntimeOwnedViewModelViewModel {
                property_index,
                property_name: property
                    .string_property("name")
                    .unwrap_or_default()
                    .to_owned(),
                value,
                property_names: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_property_names(file, view_model_index)
                    })
                    .unwrap_or_default(),
                numbers: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_numbers(file, view_model_index)
                    })
                    .unwrap_or_default(),
                imported_numbers: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_imported_numbers(file, view_model_index)
                    })
                    .unwrap_or_default(),
                booleans: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_booleans(file, view_model_index)
                    })
                    .unwrap_or_default(),
                imported_booleans: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_imported_booleans(file, view_model_index)
                    })
                    .unwrap_or_default(),
                strings: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_strings(file, view_model_index)
                    })
                    .unwrap_or_default(),
                imported_strings: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_imported_strings(file, view_model_index)
                    })
                    .unwrap_or_default(),
                colors: referenced_view_model_index
                    .map(|view_model_index| runtime_owned_view_model_colors(file, view_model_index))
                    .unwrap_or_default(),
                imported_colors: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_imported_colors(file, view_model_index)
                    })
                    .unwrap_or_default(),
                enums: referenced_view_model_index
                    .map(|view_model_index| runtime_owned_view_model_enums(file, view_model_index))
                    .unwrap_or_default(),
                imported_enums: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_imported_enums(file, view_model_index)
                    })
                    .unwrap_or_default(),
                symbol_list_indices: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_symbol_list_indices(file, view_model_index)
                    })
                    .unwrap_or_default(),
                imported_symbol_list_indices: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_imported_symbol_list_indices(
                            file,
                            view_model_index,
                        )
                    })
                    .unwrap_or_default(),
                lists: referenced_view_model_index
                    .map(|view_model_index| runtime_owned_view_model_lists(file, view_model_index))
                    .unwrap_or_default(),
                imported_lists: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_imported_lists(file, view_model_index)
                    })
                    .unwrap_or_default(),
                assets: referenced_view_model_index
                    .map(|view_model_index| runtime_owned_view_model_assets(file, view_model_index))
                    .unwrap_or_default(),
                imported_assets: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_imported_assets(file, view_model_index)
                    })
                    .unwrap_or_default(),
                artboards: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_artboards(file, view_model_index)
                    })
                    .unwrap_or_default(),
                imported_artboards: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_imported_artboards(file, view_model_index)
                    })
                    .unwrap_or_default(),
                triggers: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_triggers(file, view_model_index)
                    })
                    .unwrap_or_default(),
                imported_triggers: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_imported_triggers(file, view_model_index)
                    })
                    .unwrap_or_default(),
                view_model_instance_ids,
                children,
                imported_children,
            })
        })
        .collect()
}

impl RuntimeOwnedViewModelInstance {
    pub fn new(file: &RuntimeFile, view_model_index: usize) -> Option<Self> {
        let view_model = file.view_model(view_model_index)?;
        let mut property_names = Vec::new();
        let mut numbers = Vec::new();
        let mut booleans = Vec::new();
        let mut strings = Vec::new();
        let mut colors = Vec::new();
        let mut enums = Vec::new();
        let mut symbol_list_indices = Vec::new();
        let mut lists = Vec::new();
        let mut assets = Vec::new();
        let mut artboards = Vec::new();
        let mut triggers = Vec::new();
        let mut view_models = Vec::new();
        for (property_index, property) in view_model.properties.into_iter().enumerate() {
            property_names.push((
                property
                    .string_property("name")
                    .unwrap_or_default()
                    .to_owned(),
                property_index,
            ));
            match property.type_name {
                "ViewModelPropertyNumber" => numbers.push(RuntimeOwnedViewModelNumber {
                    property_index,
                    value: 0.0,
                }),
                "ViewModelPropertyBoolean" => booleans.push(RuntimeOwnedViewModelBoolean {
                    property_index,
                    value: false,
                }),
                "ViewModelPropertyString" => strings.push(RuntimeOwnedViewModelString {
                    property_index,
                    value: Vec::new(),
                }),
                "ViewModelPropertyColor" => colors.push(RuntimeOwnedViewModelColor {
                    property_index,
                    value: 0,
                }),
                "ViewModelPropertyEnum"
                | "ViewModelPropertyEnumCustom"
                | "ViewModelPropertyEnumSystem" => enums.push(RuntimeOwnedViewModelEnum {
                    property_index,
                    value: 0,
                }),
                "ViewModelPropertySymbolListIndex" => {
                    symbol_list_indices.push(RuntimeOwnedViewModelSymbolListIndex {
                        property_index,
                        value: 0,
                    })
                }
                "ViewModelPropertyList" => lists.push(RuntimeOwnedViewModelList {
                    property_index,
                    item_count: 0,
                }),
                "ViewModelPropertyAsset" | "ViewModelPropertyAssetImage" => {
                    assets.push(RuntimeOwnedViewModelAsset {
                        property_index,
                        value: 0,
                    })
                }
                "ViewModelPropertyArtboard" => artboards.push(RuntimeOwnedViewModelArtboard {
                    property_index,
                    value: 0,
                }),
                "ViewModelPropertyTrigger" => triggers.push(RuntimeOwnedViewModelTrigger {
                    property_index,
                    value: 0,
                }),
                "ViewModelPropertyViewModel" => {
                    let referenced_view_model_index =
                        property.uint_property("viewModelReferenceId").and_then(
                            |view_model_reference_id| usize::try_from(view_model_reference_id).ok(),
                        );
                    let referenced_view_model = referenced_view_model_index
                        .and_then(|view_model_index| file.view_model(view_model_index));
                    let path = [view_model_index, property_index];
                    let value = if referenced_view_model.is_some() {
                        RuntimeViewModelPointer::OwnedGenerated {
                            view_model_index,
                            property_index,
                            path_key: runtime_owned_view_model_path_key(&path),
                        }
                    } else {
                        RuntimeViewModelPointer::Null
                    };
                    let children = if referenced_view_model.is_some() {
                        referenced_view_model_index
                            .map(|referenced_view_model_index| {
                                runtime_owned_view_model_view_model_children(
                                    file,
                                    referenced_view_model_index,
                                    &path,
                                    &[view_model_index],
                                )
                            })
                            .unwrap_or_default()
                    } else {
                        Vec::new()
                    };
                    let view_model_instance_ids = referenced_view_model
                        .map(|view_model| {
                            view_model
                                .instances
                                .into_iter()
                                .map(|instance| instance.object.id)
                                .collect()
                        })
                        .unwrap_or_default();
                    view_models.push(RuntimeOwnedViewModelViewModel {
                        property_index,
                        property_name: property.string_property("name").unwrap_or_default().to_owned(),
                        value,
                        property_names: referenced_view_model_index
                            .map(|view_model_index| {
                                runtime_owned_view_model_property_names(file, view_model_index)
                            })
                            .unwrap_or_default(),
                        numbers: referenced_view_model_index
                            .map(|view_model_index| {
                                runtime_owned_view_model_numbers(file, view_model_index)
                            })
                            .unwrap_or_default(),
                        imported_numbers: referenced_view_model_index
                            .map(|view_model_index| {
                                runtime_owned_view_model_imported_numbers(file, view_model_index)
                            })
                            .unwrap_or_default(),
                        booleans: referenced_view_model_index
                            .map(|view_model_index| {
                                runtime_owned_view_model_booleans(file, view_model_index)
                            })
                            .unwrap_or_default(),
                        imported_booleans: referenced_view_model_index
                            .map(|view_model_index| {
                                runtime_owned_view_model_imported_booleans(file, view_model_index)
                            })
                            .unwrap_or_default(),
                        strings: referenced_view_model_index
                            .map(|view_model_index| {
                                runtime_owned_view_model_strings(file, view_model_index)
                            })
                            .unwrap_or_default(),
                        imported_strings: referenced_view_model_index
                            .map(|view_model_index| {
                                runtime_owned_view_model_imported_strings(file, view_model_index)
                            })
                            .unwrap_or_default(),
                        colors: referenced_view_model_index
                            .map(|view_model_index| {
                                runtime_owned_view_model_colors(file, view_model_index)
                            })
                            .unwrap_or_default(),
                        imported_colors: referenced_view_model_index
                            .map(|view_model_index| {
                                runtime_owned_view_model_imported_colors(file, view_model_index)
                            })
                            .unwrap_or_default(),
                        enums: referenced_view_model_index
                            .map(|view_model_index| {
                                runtime_owned_view_model_enums(file, view_model_index)
                            })
                            .unwrap_or_default(),
                        imported_enums: referenced_view_model_index
                            .map(|view_model_index| {
                                runtime_owned_view_model_imported_enums(file, view_model_index)
                            })
                            .unwrap_or_default(),
                        symbol_list_indices: referenced_view_model_index
                            .map(|view_model_index| {
                                runtime_owned_view_model_symbol_list_indices(
                                    file,
                                    view_model_index,
                                )
                            })
                            .unwrap_or_default(),
                        imported_symbol_list_indices: referenced_view_model_index
                            .map(|view_model_index| {
                                runtime_owned_view_model_imported_symbol_list_indices(
                                    file,
                                    view_model_index,
                                )
                            })
                            .unwrap_or_default(),
                        lists: referenced_view_model_index
                            .map(|view_model_index| {
                                runtime_owned_view_model_lists(file, view_model_index)
                            })
                            .unwrap_or_default(),
                        imported_lists: referenced_view_model_index
                            .map(|view_model_index| {
                                runtime_owned_view_model_imported_lists(file, view_model_index)
                            })
                            .unwrap_or_default(),
                        assets: referenced_view_model_index
                            .map(|view_model_index| {
                                runtime_owned_view_model_assets(file, view_model_index)
                            })
                            .unwrap_or_default(),
                        imported_assets: referenced_view_model_index
                            .map(|view_model_index| {
                                runtime_owned_view_model_imported_assets(file, view_model_index)
                            })
                            .unwrap_or_default(),
                        artboards: referenced_view_model_index
                            .map(|view_model_index| {
                                runtime_owned_view_model_artboards(file, view_model_index)
                            })
                            .unwrap_or_default(),
                        imported_artboards: referenced_view_model_index
                            .map(|view_model_index| {
                                runtime_owned_view_model_imported_artboards(file, view_model_index)
                            })
                            .unwrap_or_default(),
                        triggers: referenced_view_model_index
                            .map(|view_model_index| {
                                runtime_owned_view_model_triggers(file, view_model_index)
                            })
                            .unwrap_or_default(),
                        imported_triggers: referenced_view_model_index
                            .map(|view_model_index| {
                                runtime_owned_view_model_imported_triggers(file, view_model_index)
                            })
                            .unwrap_or_default(),
                        view_model_instance_ids,
                        children,
                        imported_children: referenced_view_model_index
                            .and_then(|referenced_view_model_index| {
                                file.view_model(referenced_view_model_index)
                                    .map(|view_model| {
                                        view_model
                                            .instances
                                            .into_iter()
                                            .map(|instance| {
                                                (
                                                    instance.object.id,
                                                    runtime_owned_view_model_view_model_children_for_instance(
                                                        file,
                                                        referenced_view_model_index,
                                                        instance.object,
                                                        &path,
                                                        &[view_model_index],
                                                    ),
                                                )
                                            })
                                            .collect()
                                    })
                            })
                            .unwrap_or_default(),
                    });
                }
                _ => {}
            }
        }
        Some(Self {
            view_model_index,
            property_names,
            numbers,
            booleans,
            strings,
            colors,
            enums,
            symbol_list_indices,
            lists,
            assets,
            artboards,
            triggers,
            view_models,
        })
    }

    fn property_index_by_name(&self, property_name: &str) -> Option<usize> {
        runtime_owned_view_model_property_index_by_name(&self.property_names, property_name)
    }

    pub fn set_number_by_property_index(&mut self, property_index: usize, value: f32) -> bool {
        let Some(number) = self
            .numbers
            .iter_mut()
            .find(|number| number.property_index == property_index)
        else {
            return false;
        };
        if number.value == value {
            return false;
        }
        number.value = value;
        true
    }

    pub fn set_number_by_property_name(&mut self, property_name: &str, value: f32) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_number_by_property_index(property_index, value)
    }

    pub fn number_source_handle_by_property_name(
        &self,
        property_name: &str,
    ) -> Option<RuntimeOwnedViewModelNumberSourceHandle> {
        let property_index = self.property_index_by_name(property_name)?;
        self.numbers
            .iter()
            .any(|number| number.property_index == property_index)
            .then_some(RuntimeOwnedViewModelNumberSourceHandle {
                property_path: vec![property_index],
            })
    }

    pub fn number_source_handle_by_property_name_path(
        &self,
        property_path: &str,
    ) -> Option<RuntimeOwnedViewModelNumberSourceHandle> {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return None;
        }
        let property_path = self.number_property_path_by_names(&property_path)?;
        Some(RuntimeOwnedViewModelNumberSourceHandle { property_path })
    }

    pub fn set_number_by_source_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelNumberSourceHandle,
        value: f32,
    ) -> bool {
        self.set_number_by_property_path(&handle.property_path, value)
    }

    pub fn set_number_by_property_name_path(&mut self, property_path: &str, value: f32) -> bool {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return false;
        }
        self.set_number_by_property_names(&property_path, value)
    }

    pub fn set_number_by_property_names(&mut self, property_path: &[&str], value: f32) -> bool {
        if property_path.len() == 1 {
            return self.set_number_by_property_name(property_path[0], value);
        }
        let Some((number_name, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_names_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        view_model.set_number_by_property_name(number_name, value)
    }

    fn set_number_by_property_path(&mut self, property_path: &[usize], value: f32) -> bool {
        if property_path.len() == 1 {
            return self.set_number_by_property_index(property_path[0], value);
        }
        let Some((number_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        view_model.set_number_by_property_index(*number_index, value)
    }

    fn number_property_path_by_names(&self, property_path: &[&str]) -> Option<Vec<usize>> {
        if property_path.len() == 1 {
            let property_index = self.property_index_by_name(property_path[0])?;
            return self
                .numbers
                .iter()
                .any(|number| number.property_index == property_index)
                .then_some(vec![property_index]);
        }

        let (number_name, view_model_names) = property_path.split_last()?;
        let (view_model_path, view_model) =
            self.view_model_property_path_by_names(view_model_names)?;
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return None;
        }
        let property_index = view_model.property_index_by_name(number_name)?;
        if !view_model
            .numbers
            .iter()
            .any(|number| number.property_index == property_index)
        {
            return None;
        }
        let mut property_path = view_model_path;
        property_path.push(property_index);
        Some(property_path)
    }

    pub fn set_boolean_by_property_index(&mut self, property_index: usize, value: bool) -> bool {
        let Some(boolean) = self
            .booleans
            .iter_mut()
            .find(|boolean| boolean.property_index == property_index)
        else {
            return false;
        };
        if boolean.value == value {
            return false;
        }
        boolean.value = value;
        true
    }

    pub fn set_boolean_by_property_name(&mut self, property_name: &str, value: bool) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_boolean_by_property_index(property_index, value)
    }

    pub fn boolean_source_handle_by_property_name(
        &self,
        property_name: &str,
    ) -> Option<RuntimeOwnedViewModelBooleanSourceHandle> {
        let property_index = self.property_index_by_name(property_name)?;
        self.booleans
            .iter()
            .any(|boolean| boolean.property_index == property_index)
            .then_some(RuntimeOwnedViewModelBooleanSourceHandle {
                property_path: vec![property_index],
            })
    }

    pub fn boolean_source_handle_by_property_name_path(
        &self,
        property_path: &str,
    ) -> Option<RuntimeOwnedViewModelBooleanSourceHandle> {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return None;
        }
        let property_path = self.boolean_property_path_by_names(&property_path)?;
        Some(RuntimeOwnedViewModelBooleanSourceHandle { property_path })
    }

    pub fn set_boolean_by_source_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelBooleanSourceHandle,
        value: bool,
    ) -> bool {
        self.set_boolean_by_property_path(&handle.property_path, value)
    }

    pub fn set_boolean_by_property_name_path(&mut self, property_path: &str, value: bool) -> bool {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return false;
        }
        self.set_boolean_by_property_names(&property_path, value)
    }

    pub fn set_boolean_by_property_names(&mut self, property_path: &[&str], value: bool) -> bool {
        if property_path.len() == 1 {
            return self.set_boolean_by_property_name(property_path[0], value);
        }
        let Some((boolean_name, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_names_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        view_model.set_boolean_by_property_name(boolean_name, value)
    }

    fn set_boolean_by_property_path(&mut self, property_path: &[usize], value: bool) -> bool {
        if property_path.len() == 1 {
            return self.set_boolean_by_property_index(property_path[0], value);
        }
        let Some((boolean_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        view_model.set_boolean_by_property_index(*boolean_index, value)
    }

    fn boolean_property_path_by_names(&self, property_path: &[&str]) -> Option<Vec<usize>> {
        if property_path.len() == 1 {
            let property_index = self.property_index_by_name(property_path[0])?;
            return self
                .booleans
                .iter()
                .any(|boolean| boolean.property_index == property_index)
                .then_some(vec![property_index]);
        }

        let (boolean_name, view_model_names) = property_path.split_last()?;
        let (view_model_path, view_model) =
            self.view_model_property_path_by_names(view_model_names)?;
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return None;
        }
        let property_index = view_model.property_index_by_name(boolean_name)?;
        if !view_model
            .booleans
            .iter()
            .any(|boolean| boolean.property_index == property_index)
        {
            return None;
        }
        let mut property_path = view_model_path;
        property_path.push(property_index);
        Some(property_path)
    }

    pub fn set_string_by_property_index(&mut self, property_index: usize, value: &[u8]) -> bool {
        let Some(string) = self
            .strings
            .iter_mut()
            .find(|string| string.property_index == property_index)
        else {
            return false;
        };
        if string.value == value {
            return false;
        }
        string.value = value.to_vec();
        true
    }

    pub fn set_string_by_property_name(&mut self, property_name: &str, value: &[u8]) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_string_by_property_index(property_index, value)
    }

    pub fn string_source_handle_by_property_name(
        &self,
        property_name: &str,
    ) -> Option<RuntimeOwnedViewModelStringSourceHandle> {
        let property_index = self.property_index_by_name(property_name)?;
        self.strings
            .iter()
            .any(|string| string.property_index == property_index)
            .then_some(RuntimeOwnedViewModelStringSourceHandle {
                property_path: vec![property_index],
            })
    }

    pub fn string_source_handle_by_property_name_path(
        &self,
        property_path: &str,
    ) -> Option<RuntimeOwnedViewModelStringSourceHandle> {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return None;
        }
        let property_path = self.string_property_path_by_names(&property_path)?;
        Some(RuntimeOwnedViewModelStringSourceHandle { property_path })
    }

    pub fn set_string_by_source_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelStringSourceHandle,
        value: &[u8],
    ) -> bool {
        self.set_string_by_property_path(&handle.property_path, value)
    }

    pub fn set_string_by_property_name_path(&mut self, property_path: &str, value: &[u8]) -> bool {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return false;
        }
        self.set_string_by_property_names(&property_path, value)
    }

    pub fn set_string_by_property_names(&mut self, property_path: &[&str], value: &[u8]) -> bool {
        if property_path.len() == 1 {
            return self.set_string_by_property_name(property_path[0], value);
        }
        let Some((string_name, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_names_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        view_model.set_string_by_property_name(string_name, value)
    }

    fn set_string_by_property_path(&mut self, property_path: &[usize], value: &[u8]) -> bool {
        if property_path.len() == 1 {
            return self.set_string_by_property_index(property_path[0], value);
        }
        let Some((string_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        view_model.set_string_by_property_index(*string_index, value)
    }

    fn string_property_path_by_names(&self, property_path: &[&str]) -> Option<Vec<usize>> {
        if property_path.len() == 1 {
            let property_index = self.property_index_by_name(property_path[0])?;
            return self
                .strings
                .iter()
                .any(|string| string.property_index == property_index)
                .then_some(vec![property_index]);
        }

        let (string_name, view_model_names) = property_path.split_last()?;
        let (view_model_path, view_model) =
            self.view_model_property_path_by_names(view_model_names)?;
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return None;
        }
        let property_index = view_model.property_index_by_name(string_name)?;
        if !view_model
            .strings
            .iter()
            .any(|string| string.property_index == property_index)
        {
            return None;
        }
        let mut property_path = view_model_path;
        property_path.push(property_index);
        Some(property_path)
    }

    pub fn set_color_by_property_index(&mut self, property_index: usize, value: u32) -> bool {
        let Some(color) = self
            .colors
            .iter_mut()
            .find(|color| color.property_index == property_index)
        else {
            return false;
        };
        if color.value == value {
            return false;
        }
        color.value = value;
        true
    }

    pub fn set_color_by_property_name(&mut self, property_name: &str, value: u32) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_color_by_property_index(property_index, value)
    }

    pub fn color_source_handle_by_property_name(
        &self,
        property_name: &str,
    ) -> Option<RuntimeOwnedViewModelColorSourceHandle> {
        let property_index = self.property_index_by_name(property_name)?;
        self.colors
            .iter()
            .any(|color| color.property_index == property_index)
            .then_some(RuntimeOwnedViewModelColorSourceHandle {
                property_path: vec![property_index],
            })
    }

    pub fn color_source_handle_by_property_name_path(
        &self,
        property_path: &str,
    ) -> Option<RuntimeOwnedViewModelColorSourceHandle> {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return None;
        }
        let property_path = self.color_property_path_by_names(&property_path)?;
        Some(RuntimeOwnedViewModelColorSourceHandle { property_path })
    }

    pub fn set_color_by_source_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelColorSourceHandle,
        value: u32,
    ) -> bool {
        self.set_color_by_property_path(&handle.property_path, value)
    }

    pub fn set_color_by_property_name_path(&mut self, property_path: &str, value: u32) -> bool {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return false;
        }
        self.set_color_by_property_names(&property_path, value)
    }

    pub fn set_color_by_property_names(&mut self, property_path: &[&str], value: u32) -> bool {
        if property_path.len() == 1 {
            return self.set_color_by_property_name(property_path[0], value);
        }
        let Some((color_name, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_names_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        view_model.set_color_by_property_name(color_name, value)
    }

    fn set_color_by_property_path(&mut self, property_path: &[usize], value: u32) -> bool {
        if property_path.len() == 1 {
            return self.set_color_by_property_index(property_path[0], value);
        }
        let Some((color_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        view_model.set_color_by_property_index(*color_index, value)
    }

    fn color_property_path_by_names(&self, property_path: &[&str]) -> Option<Vec<usize>> {
        if property_path.len() == 1 {
            let property_index = self.property_index_by_name(property_path[0])?;
            return self
                .colors
                .iter()
                .any(|color| color.property_index == property_index)
                .then_some(vec![property_index]);
        }

        let (color_name, view_model_names) = property_path.split_last()?;
        let (view_model_path, view_model) =
            self.view_model_property_path_by_names(view_model_names)?;
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return None;
        }
        let property_index = view_model.property_index_by_name(color_name)?;
        if !view_model
            .colors
            .iter()
            .any(|color| color.property_index == property_index)
        {
            return None;
        }
        let mut property_path = view_model_path;
        property_path.push(property_index);
        Some(property_path)
    }

    pub fn set_enum_by_property_index(&mut self, property_index: usize, value: u64) -> bool {
        let Some(enum_value) = self
            .enums
            .iter_mut()
            .find(|enum_value| enum_value.property_index == property_index)
        else {
            return false;
        };
        if enum_value.value == value {
            return false;
        }
        enum_value.value = value;
        true
    }

    pub fn set_enum_by_property_name(&mut self, property_name: &str, value: u64) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_enum_by_property_index(property_index, value)
    }

    pub fn enum_source_handle_by_property_name(
        &self,
        property_name: &str,
    ) -> Option<RuntimeOwnedViewModelEnumSourceHandle> {
        let property_index = self.property_index_by_name(property_name)?;
        self.enums
            .iter()
            .any(|enum_value| enum_value.property_index == property_index)
            .then_some(RuntimeOwnedViewModelEnumSourceHandle {
                property_path: vec![property_index],
            })
    }

    pub fn enum_source_handle_by_property_name_path(
        &self,
        property_path: &str,
    ) -> Option<RuntimeOwnedViewModelEnumSourceHandle> {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return None;
        }
        let property_path = self.enum_property_path_by_names(&property_path)?;
        Some(RuntimeOwnedViewModelEnumSourceHandle { property_path })
    }

    pub fn set_enum_by_source_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelEnumSourceHandle,
        value: u64,
    ) -> bool {
        self.set_enum_by_property_path(&handle.property_path, value)
    }

    pub fn set_enum_by_property_name_path(&mut self, property_path: &str, value: u64) -> bool {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return false;
        }
        self.set_enum_by_property_names(&property_path, value)
    }

    pub fn set_enum_by_property_names(&mut self, property_path: &[&str], value: u64) -> bool {
        if property_path.len() == 1 {
            return self.set_enum_by_property_name(property_path[0], value);
        }
        let Some((enum_name, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_names_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        view_model.set_enum_by_property_name(enum_name, value)
    }

    fn set_enum_by_property_path(&mut self, property_path: &[usize], value: u64) -> bool {
        if property_path.len() == 1 {
            return self.set_enum_by_property_index(property_path[0], value);
        }
        let Some((enum_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        view_model.set_enum_by_property_index(*enum_index, value)
    }

    fn enum_property_path_by_names(&self, property_path: &[&str]) -> Option<Vec<usize>> {
        if property_path.len() == 1 {
            let property_index = self.property_index_by_name(property_path[0])?;
            return self
                .enums
                .iter()
                .any(|enum_value| enum_value.property_index == property_index)
                .then_some(vec![property_index]);
        }

        let (enum_name, view_model_names) = property_path.split_last()?;
        let (view_model_path, view_model) =
            self.view_model_property_path_by_names(view_model_names)?;
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return None;
        }
        let property_index = view_model.property_index_by_name(enum_name)?;
        if !view_model
            .enums
            .iter()
            .any(|enum_value| enum_value.property_index == property_index)
        {
            return None;
        }
        let mut property_path = view_model_path;
        property_path.push(property_index);
        Some(property_path)
    }

    pub fn set_symbol_list_index_by_property_index(
        &mut self,
        property_index: usize,
        value: u64,
    ) -> bool {
        let Some(symbol_list_index) = self
            .symbol_list_indices
            .iter_mut()
            .find(|symbol_list_index| symbol_list_index.property_index == property_index)
        else {
            return false;
        };
        if symbol_list_index.value == value {
            return false;
        }
        symbol_list_index.value = value;
        true
    }

    pub fn set_symbol_list_index_by_property_name(
        &mut self,
        property_name: &str,
        value: u64,
    ) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_symbol_list_index_by_property_index(property_index, value)
    }

    pub fn symbol_list_index_source_handle_by_property_name(
        &self,
        property_name: &str,
    ) -> Option<RuntimeOwnedViewModelSymbolListIndexSourceHandle> {
        let property_index = self.property_index_by_name(property_name)?;
        self.symbol_list_indices
            .iter()
            .any(|symbol_list_index| symbol_list_index.property_index == property_index)
            .then_some(RuntimeOwnedViewModelSymbolListIndexSourceHandle {
                property_path: vec![property_index],
            })
    }

    pub fn symbol_list_index_source_handle_by_property_name_path(
        &self,
        property_path: &str,
    ) -> Option<RuntimeOwnedViewModelSymbolListIndexSourceHandle> {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return None;
        }
        let property_path = self.symbol_list_index_property_path_by_names(&property_path)?;
        Some(RuntimeOwnedViewModelSymbolListIndexSourceHandle { property_path })
    }

    pub fn set_symbol_list_index_by_source_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelSymbolListIndexSourceHandle,
        value: u64,
    ) -> bool {
        self.set_symbol_list_index_by_property_path(&handle.property_path, value)
    }

    pub fn set_symbol_list_index_by_property_name_path(
        &mut self,
        property_path: &str,
        value: u64,
    ) -> bool {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return false;
        }
        self.set_symbol_list_index_by_property_names(&property_path, value)
    }

    pub fn set_symbol_list_index_by_property_names(
        &mut self,
        property_path: &[&str],
        value: u64,
    ) -> bool {
        if property_path.len() == 1 {
            return self.set_symbol_list_index_by_property_name(property_path[0], value);
        }
        let Some((symbol_list_index_name, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_names_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        view_model.set_symbol_list_index_by_property_name(symbol_list_index_name, value)
    }

    fn set_symbol_list_index_by_property_path(
        &mut self,
        property_path: &[usize],
        value: u64,
    ) -> bool {
        if property_path.len() == 1 {
            return self.set_symbol_list_index_by_property_index(property_path[0], value);
        }
        let Some((symbol_list_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        view_model.set_symbol_list_index_by_property_index(*symbol_list_index, value)
    }

    fn symbol_list_index_property_path_by_names(
        &self,
        property_path: &[&str],
    ) -> Option<Vec<usize>> {
        if property_path.len() == 1 {
            let property_index = self.property_index_by_name(property_path[0])?;
            return self
                .symbol_list_indices
                .iter()
                .any(|symbol_list_index| symbol_list_index.property_index == property_index)
                .then_some(vec![property_index]);
        }

        let (symbol_list_index_name, view_model_names) = property_path.split_last()?;
        let (view_model_path, view_model) =
            self.view_model_property_path_by_names(view_model_names)?;
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return None;
        }
        let property_index = view_model.property_index_by_name(symbol_list_index_name)?;
        if !view_model
            .symbol_list_indices
            .iter()
            .any(|symbol_list_index| symbol_list_index.property_index == property_index)
        {
            return None;
        }
        let mut property_path = view_model_path;
        property_path.push(property_index);
        Some(property_path)
    }

    pub fn set_list_item_count_by_property_index(
        &mut self,
        property_index: usize,
        item_count: usize,
    ) -> bool {
        let Some(list) = self
            .lists
            .iter_mut()
            .find(|list| list.property_index == property_index)
        else {
            return false;
        };
        if list.item_count == item_count {
            return false;
        }
        list.item_count = item_count;
        true
    }

    pub fn set_list_item_count_by_property_name(
        &mut self,
        property_name: &str,
        item_count: usize,
    ) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_list_item_count_by_property_index(property_index, item_count)
    }

    pub fn list_source_handle_by_property_name(
        &self,
        property_name: &str,
    ) -> Option<RuntimeOwnedViewModelListSourceHandle> {
        let property_index = self.property_index_by_name(property_name)?;
        self.lists
            .iter()
            .any(|list| list.property_index == property_index)
            .then_some(RuntimeOwnedViewModelListSourceHandle {
                property_path: vec![property_index],
            })
    }

    pub fn list_source_handle_by_property_name_path(
        &self,
        property_path: &str,
    ) -> Option<RuntimeOwnedViewModelListSourceHandle> {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return None;
        }
        let property_path = self.list_property_path_by_names(&property_path)?;
        Some(RuntimeOwnedViewModelListSourceHandle { property_path })
    }

    pub fn set_list_item_count_by_source_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelListSourceHandle,
        item_count: usize,
    ) -> bool {
        self.set_list_item_count_by_property_path(&handle.property_path, item_count)
    }

    pub fn set_list_item_count_by_property_name_path(
        &mut self,
        property_path: &str,
        item_count: usize,
    ) -> bool {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return false;
        }
        self.set_list_item_count_by_property_names(&property_path, item_count)
    }

    pub fn set_list_item_count_by_property_names(
        &mut self,
        property_path: &[&str],
        item_count: usize,
    ) -> bool {
        if property_path.len() == 1 {
            return self.set_list_item_count_by_property_name(property_path[0], item_count);
        }
        let Some((list_name, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_names_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        view_model.set_list_item_count_by_property_name(list_name, item_count)
    }

    fn set_list_item_count_by_property_path(
        &mut self,
        property_path: &[usize],
        item_count: usize,
    ) -> bool {
        if property_path.len() == 1 {
            return self.set_list_item_count_by_property_index(property_path[0], item_count);
        }
        let Some((list_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        view_model.set_list_item_count_by_property_index(*list_index, item_count)
    }

    fn list_property_path_by_names(&self, property_path: &[&str]) -> Option<Vec<usize>> {
        if property_path.len() == 1 {
            let property_index = self.property_index_by_name(property_path[0])?;
            return self
                .lists
                .iter()
                .any(|list| list.property_index == property_index)
                .then_some(vec![property_index]);
        }

        let (list_name, view_model_names) = property_path.split_last()?;
        let (view_model_path, view_model) =
            self.view_model_property_path_by_names(view_model_names)?;
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return None;
        }
        let property_index = view_model.property_index_by_name(list_name)?;
        if !view_model
            .lists
            .iter()
            .any(|list| list.property_index == property_index)
        {
            return None;
        }
        let mut property_path = view_model_path;
        property_path.push(property_index);
        Some(property_path)
    }

    pub fn set_asset_by_property_index(&mut self, property_index: usize, value: u64) -> bool {
        let Some(asset) = self
            .assets
            .iter_mut()
            .find(|asset| asset.property_index == property_index)
        else {
            return false;
        };
        if asset.value == value {
            return false;
        }
        asset.value = value;
        true
    }

    pub fn set_asset_by_property_name(&mut self, property_name: &str, value: u64) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_asset_by_property_index(property_index, value)
    }

    pub fn asset_source_handle_by_property_name(
        &self,
        property_name: &str,
    ) -> Option<RuntimeOwnedViewModelAssetSourceHandle> {
        let property_index = self.property_index_by_name(property_name)?;
        self.assets
            .iter()
            .any(|asset| asset.property_index == property_index)
            .then_some(RuntimeOwnedViewModelAssetSourceHandle {
                property_path: vec![property_index],
            })
    }

    pub fn asset_source_handle_by_property_name_path(
        &self,
        property_path: &str,
    ) -> Option<RuntimeOwnedViewModelAssetSourceHandle> {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return None;
        }
        let property_path = self.asset_property_path_by_names(&property_path)?;
        Some(RuntimeOwnedViewModelAssetSourceHandle { property_path })
    }

    pub fn set_asset_by_source_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelAssetSourceHandle,
        value: u64,
    ) -> bool {
        self.set_asset_by_property_path(&handle.property_path, value)
    }

    pub fn set_asset_by_property_name_path(&mut self, property_path: &str, value: u64) -> bool {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return false;
        }
        self.set_asset_by_property_names(&property_path, value)
    }

    pub fn set_asset_by_property_names(&mut self, property_path: &[&str], value: u64) -> bool {
        if property_path.len() == 1 {
            return self.set_asset_by_property_name(property_path[0], value);
        }
        let Some((asset_name, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_names_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        view_model.set_asset_by_property_name(asset_name, value)
    }

    fn set_asset_by_property_path(&mut self, property_path: &[usize], value: u64) -> bool {
        if property_path.len() == 1 {
            return self.set_asset_by_property_index(property_path[0], value);
        }
        let Some((asset_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        view_model.set_asset_by_property_index(*asset_index, value)
    }

    fn asset_property_path_by_names(&self, property_path: &[&str]) -> Option<Vec<usize>> {
        if property_path.len() == 1 {
            let property_index = self.property_index_by_name(property_path[0])?;
            return self
                .assets
                .iter()
                .any(|asset| asset.property_index == property_index)
                .then_some(vec![property_index]);
        }

        let (asset_name, view_model_names) = property_path.split_last()?;
        let (view_model_path, view_model) =
            self.view_model_property_path_by_names(view_model_names)?;
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return None;
        }
        let property_index = view_model.property_index_by_name(asset_name)?;
        if !view_model
            .assets
            .iter()
            .any(|asset| asset.property_index == property_index)
        {
            return None;
        }
        let mut property_path = view_model_path;
        property_path.push(property_index);
        Some(property_path)
    }

    pub fn set_artboard_by_property_index(&mut self, property_index: usize, value: u64) -> bool {
        let Some(artboard) = self
            .artboards
            .iter_mut()
            .find(|artboard| artboard.property_index == property_index)
        else {
            return false;
        };
        if artboard.value == value {
            return false;
        }
        artboard.value = value;
        true
    }

    pub fn set_artboard_by_property_name(&mut self, property_name: &str, value: u64) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_artboard_by_property_index(property_index, value)
    }

    pub fn artboard_source_handle_by_property_name(
        &self,
        property_name: &str,
    ) -> Option<RuntimeOwnedViewModelArtboardSourceHandle> {
        let property_index = self.property_index_by_name(property_name)?;
        self.artboards
            .iter()
            .any(|artboard| artboard.property_index == property_index)
            .then_some(RuntimeOwnedViewModelArtboardSourceHandle {
                property_path: vec![property_index],
            })
    }

    pub fn artboard_source_handle_by_property_name_path(
        &self,
        property_path: &str,
    ) -> Option<RuntimeOwnedViewModelArtboardSourceHandle> {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return None;
        }
        let property_path = self.artboard_property_path_by_names(&property_path)?;
        Some(RuntimeOwnedViewModelArtboardSourceHandle { property_path })
    }

    pub fn set_artboard_by_source_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelArtboardSourceHandle,
        value: u64,
    ) -> bool {
        self.set_artboard_by_property_path(&handle.property_path, value)
    }

    pub fn set_artboard_by_property_name_path(&mut self, property_path: &str, value: u64) -> bool {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return false;
        }
        self.set_artboard_by_property_names(&property_path, value)
    }

    pub fn set_artboard_by_property_names(&mut self, property_path: &[&str], value: u64) -> bool {
        if property_path.len() == 1 {
            return self.set_artboard_by_property_name(property_path[0], value);
        }
        let Some((artboard_name, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_names_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        view_model.set_artboard_by_property_name(artboard_name, value)
    }

    fn set_artboard_by_property_path(&mut self, property_path: &[usize], value: u64) -> bool {
        if property_path.len() == 1 {
            return self.set_artboard_by_property_index(property_path[0], value);
        }
        let Some((artboard_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        view_model.set_artboard_by_property_index(*artboard_index, value)
    }

    fn artboard_property_path_by_names(&self, property_path: &[&str]) -> Option<Vec<usize>> {
        if property_path.len() == 1 {
            let property_index = self.property_index_by_name(property_path[0])?;
            return self
                .artboards
                .iter()
                .any(|artboard| artboard.property_index == property_index)
                .then_some(vec![property_index]);
        }

        let (artboard_name, view_model_names) = property_path.split_last()?;
        let (view_model_path, view_model) =
            self.view_model_property_path_by_names(view_model_names)?;
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return None;
        }
        let property_index = view_model.property_index_by_name(artboard_name)?;
        if !view_model
            .artboards
            .iter()
            .any(|artboard| artboard.property_index == property_index)
        {
            return None;
        }
        let mut property_path = view_model_path;
        property_path.push(property_index);
        Some(property_path)
    }

    pub fn set_trigger_by_property_index(&mut self, property_index: usize, value: u64) -> bool {
        let Some(trigger) = self
            .triggers
            .iter_mut()
            .find(|trigger| trigger.property_index == property_index)
        else {
            return false;
        };
        if trigger.value == value {
            return false;
        }
        trigger.value = value;
        true
    }

    pub fn set_trigger_by_property_name(&mut self, property_name: &str, value: u64) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_trigger_by_property_index(property_index, value)
    }

    pub fn trigger_source_handle_by_property_name(
        &self,
        property_name: &str,
    ) -> Option<RuntimeOwnedViewModelTriggerSourceHandle> {
        let property_index = self.property_index_by_name(property_name)?;
        self.triggers
            .iter()
            .any(|trigger| trigger.property_index == property_index)
            .then_some(RuntimeOwnedViewModelTriggerSourceHandle {
                property_path: vec![property_index],
            })
    }

    pub fn trigger_source_handle_by_property_name_path(
        &self,
        property_path: &str,
    ) -> Option<RuntimeOwnedViewModelTriggerSourceHandle> {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return None;
        }
        let property_path = self.trigger_property_path_by_names(&property_path)?;
        Some(RuntimeOwnedViewModelTriggerSourceHandle { property_path })
    }

    pub fn set_trigger_by_source_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelTriggerSourceHandle,
        value: u64,
    ) -> bool {
        self.set_trigger_by_property_path(&handle.property_path, value)
    }

    pub fn set_trigger_by_property_name_path(&mut self, property_path: &str, value: u64) -> bool {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return false;
        }
        self.set_trigger_by_property_names(&property_path, value)
    }

    pub fn set_trigger_by_property_names(&mut self, property_path: &[&str], value: u64) -> bool {
        if property_path.len() == 1 {
            return self.set_trigger_by_property_name(property_path[0], value);
        }
        let Some((trigger_name, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_names_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        view_model.set_trigger_by_property_name(trigger_name, value)
    }

    fn set_trigger_by_property_path(&mut self, property_path: &[usize], value: u64) -> bool {
        if property_path.len() == 1 {
            return self.set_trigger_by_property_index(property_path[0], value);
        }
        let Some((trigger_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        view_model.set_trigger_by_property_index(*trigger_index, value)
    }

    fn trigger_property_path_by_names(&self, property_path: &[&str]) -> Option<Vec<usize>> {
        if property_path.len() == 1 {
            let property_index = self.property_index_by_name(property_path[0])?;
            return self
                .triggers
                .iter()
                .any(|trigger| trigger.property_index == property_index)
                .then_some(vec![property_index]);
        }

        let (trigger_name, view_model_names) = property_path.split_last()?;
        let (view_model_path, view_model) =
            self.view_model_property_path_by_names(view_model_names)?;
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return None;
        }
        let property_index = view_model.property_index_by_name(trigger_name)?;
        if !view_model
            .triggers
            .iter()
            .any(|trigger| trigger.property_index == property_index)
        {
            return None;
        }
        let mut property_path = view_model_path;
        property_path.push(property_index);
        Some(property_path)
    }

    pub fn set_view_model_by_property_index(
        &mut self,
        property_index: usize,
        instance_index: usize,
    ) -> bool {
        self.set_view_model_by_property_path(&[property_index], instance_index)
    }

    pub fn view_model_source_handle_by_property_name(
        &self,
        property_name: &str,
    ) -> Option<RuntimeOwnedViewModelViewModelSourceHandle> {
        let property_index = self.property_index_by_name(property_name)?;
        self.view_models
            .iter()
            .any(|view_model| view_model.property_index == property_index)
            .then_some(RuntimeOwnedViewModelViewModelSourceHandle {
                property_path: vec![property_index],
            })
    }

    pub fn view_model_source_handle_by_property_name_path(
        &self,
        property_path: &str,
    ) -> Option<RuntimeOwnedViewModelViewModelSourceHandle> {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return None;
        }
        let (property_path, _) = self.view_model_property_path_by_names(&property_path)?;
        Some(RuntimeOwnedViewModelViewModelSourceHandle { property_path })
    }

    pub fn set_view_model_by_source_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelViewModelSourceHandle,
        instance_index: usize,
    ) -> bool {
        self.set_view_model_by_property_path(&handle.property_path, instance_index)
    }

    pub fn set_view_model_by_property_path(
        &mut self,
        property_path: &[usize],
        instance_index: usize,
    ) -> bool {
        let Some(view_model) = self.view_model_by_property_path_mut(property_path) else {
            return false;
        };
        let Some(object_id) = view_model
            .view_model_instance_ids
            .get(instance_index)
            .copied()
        else {
            return false;
        };
        let value = RuntimeViewModelPointer::Imported { object_id };
        if view_model.value == value {
            return false;
        }
        view_model.value = value;
        true
    }

    pub fn set_view_model_by_property_name_path(
        &mut self,
        property_path: &str,
        instance_index: usize,
    ) -> bool {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return false;
        }
        self.set_view_model_by_property_names(&property_path, instance_index)
    }

    pub fn set_view_model_by_property_names(
        &mut self,
        property_path: &[&str],
        instance_index: usize,
    ) -> bool {
        let Some(view_model) = self.view_model_by_property_names_mut(property_path) else {
            return false;
        };
        let Some(object_id) = view_model
            .view_model_instance_ids
            .get(instance_index)
            .copied()
        else {
            return false;
        };
        let value = RuntimeViewModelPointer::Imported { object_id };
        if view_model.value == value {
            return false;
        }
        view_model.value = value;
        true
    }

    fn view_model_by_property_path(
        &self,
        property_path: &[usize],
    ) -> Option<&RuntimeOwnedViewModelViewModel> {
        let (property_index, rest) = property_path.split_first()?;
        let mut view_model = self
            .view_models
            .iter()
            .find(|view_model| view_model.property_index == *property_index)?;
        for property_index in rest {
            view_model = view_model
                .active_children()?
                .iter()
                .find(|view_model| view_model.property_index == *property_index)?;
        }
        Some(view_model)
    }

    fn view_model_by_property_path_mut(
        &mut self,
        property_path: &[usize],
    ) -> Option<&mut RuntimeOwnedViewModelViewModel> {
        let (property_index, rest) = property_path.split_first()?;
        let mut view_model = self
            .view_models
            .iter_mut()
            .find(|view_model| view_model.property_index == *property_index)?;
        for property_index in rest {
            view_model = view_model
                .generated_children_mut()?
                .iter_mut()
                .find(|view_model| view_model.property_index == *property_index)?;
        }
        Some(view_model)
    }

    fn view_model_by_property_names_mut(
        &mut self,
        property_path: &[&str],
    ) -> Option<&mut RuntimeOwnedViewModelViewModel> {
        let (property_name, rest) = property_path.split_first()?;
        let mut view_model = self
            .view_models
            .iter_mut()
            .find(|view_model| view_model.property_name == *property_name)?;
        for property_name in rest {
            view_model = view_model
                .generated_children_mut()?
                .iter_mut()
                .find(|view_model| view_model.property_name == *property_name)?;
        }
        Some(view_model)
    }

    fn view_model_property_path_by_names(
        &self,
        property_path: &[&str],
    ) -> Option<(Vec<usize>, &RuntimeOwnedViewModelViewModel)> {
        let (property_name, rest) = property_path.split_first()?;
        let mut path = Vec::new();
        let mut view_model = self
            .view_models
            .iter()
            .find(|view_model| view_model.property_name == *property_name)?;
        path.push(view_model.property_index);
        for property_name in rest {
            if !matches!(
                view_model.value,
                RuntimeViewModelPointer::OwnedGenerated { .. }
            ) {
                return None;
            }
            view_model = view_model
                .children
                .iter()
                .find(|view_model| view_model.property_name == *property_name)?;
            path.push(view_model.property_index);
        }
        Some((path, view_model))
    }

    fn number_value_by_property_index(&self, property_index: usize) -> Option<f32> {
        self.numbers
            .iter()
            .find(|number| number.property_index == property_index)
            .map(|number| number.value)
    }

    fn number_value_by_property_path(&self, property_path: &[usize]) -> Option<f32> {
        if property_path.len() == 1 {
            return self.number_value_by_property_index(property_path[0]);
        }
        let (property_index, view_model_path) = property_path.split_last()?;
        let view_model = self.view_model_by_property_path(view_model_path)?;
        view_model.active_number_value_by_property_index(*property_index)
    }

    fn boolean_value_by_property_index(&self, property_index: usize) -> Option<bool> {
        self.booleans
            .iter()
            .find(|boolean| boolean.property_index == property_index)
            .map(|boolean| boolean.value)
    }

    fn boolean_value_by_property_path(&self, property_path: &[usize]) -> Option<bool> {
        if property_path.len() == 1 {
            return self.boolean_value_by_property_index(property_path[0]);
        }
        let (property_index, view_model_path) = property_path.split_last()?;
        let view_model = self.view_model_by_property_path(view_model_path)?;
        view_model.active_boolean_value_by_property_index(*property_index)
    }

    fn string_value_by_property_index(&self, property_index: usize) -> Option<&[u8]> {
        self.strings
            .iter()
            .find(|string| string.property_index == property_index)
            .map(|string| string.value.as_slice())
    }

    fn string_value_by_property_path(&self, property_path: &[usize]) -> Option<&[u8]> {
        if property_path.len() == 1 {
            return self.string_value_by_property_index(property_path[0]);
        }
        let (property_index, view_model_path) = property_path.split_last()?;
        let view_model = self.view_model_by_property_path(view_model_path)?;
        view_model.active_string_value_by_property_index(*property_index)
    }

    fn color_value_by_property_index(&self, property_index: usize) -> Option<u32> {
        self.colors
            .iter()
            .find(|color| color.property_index == property_index)
            .map(|color| color.value)
    }

    fn color_value_by_property_path(&self, property_path: &[usize]) -> Option<u32> {
        if property_path.len() == 1 {
            return self.color_value_by_property_index(property_path[0]);
        }
        let (property_index, view_model_path) = property_path.split_last()?;
        let view_model = self.view_model_by_property_path(view_model_path)?;
        view_model.active_color_value_by_property_index(*property_index)
    }

    fn enum_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        self.enums
            .iter()
            .find(|enum_value| enum_value.property_index == property_index)
            .map(|enum_value| enum_value.value)
    }

    fn enum_value_by_property_path(&self, property_path: &[usize]) -> Option<u64> {
        if property_path.len() == 1 {
            return self.enum_value_by_property_index(property_path[0]);
        }
        let (property_index, view_model_path) = property_path.split_last()?;
        let view_model = self.view_model_by_property_path(view_model_path)?;
        view_model.active_enum_value_by_property_index(*property_index)
    }

    fn symbol_list_index_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        self.symbol_list_indices
            .iter()
            .find(|symbol_list_index| symbol_list_index.property_index == property_index)
            .map(|symbol_list_index| symbol_list_index.value)
    }

    fn symbol_list_index_value_by_property_path(&self, property_path: &[usize]) -> Option<u64> {
        if property_path.len() == 1 {
            return self.symbol_list_index_value_by_property_index(property_path[0]);
        }
        let (property_index, view_model_path) = property_path.split_last()?;
        let view_model = self.view_model_by_property_path(view_model_path)?;
        view_model.active_symbol_list_index_value_by_property_index(*property_index)
    }

    fn list_item_count_by_property_index(&self, property_index: usize) -> Option<usize> {
        self.lists
            .iter()
            .find(|list| list.property_index == property_index)
            .map(|list| list.item_count)
    }

    fn list_item_count_by_property_path(&self, property_path: &[usize]) -> Option<usize> {
        if property_path.len() == 1 {
            return self.list_item_count_by_property_index(property_path[0]);
        }
        let (property_index, view_model_path) = property_path.split_last()?;
        let view_model = self.view_model_by_property_path(view_model_path)?;
        view_model.active_list_item_count_by_property_index(*property_index)
    }

    fn asset_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        self.assets
            .iter()
            .find(|asset| asset.property_index == property_index)
            .map(|asset| asset.value)
    }

    fn asset_value_by_property_path(&self, property_path: &[usize]) -> Option<u64> {
        if property_path.len() == 1 {
            return self.asset_value_by_property_index(property_path[0]);
        }
        let (property_index, view_model_path) = property_path.split_last()?;
        let view_model = self.view_model_by_property_path(view_model_path)?;
        view_model.active_asset_value_by_property_index(*property_index)
    }

    fn artboard_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        self.artboards
            .iter()
            .find(|artboard| artboard.property_index == property_index)
            .map(|artboard| artboard.value)
    }

    fn artboard_value_by_property_path(&self, property_path: &[usize]) -> Option<u64> {
        if property_path.len() == 1 {
            return self.artboard_value_by_property_index(property_path[0]);
        }
        let (property_index, view_model_path) = property_path.split_last()?;
        let view_model = self.view_model_by_property_path(view_model_path)?;
        view_model.active_artboard_value_by_property_index(*property_index)
    }

    fn trigger_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        self.triggers
            .iter()
            .find(|trigger| trigger.property_index == property_index)
            .map(|trigger| trigger.value)
    }

    fn trigger_value_by_property_path(&self, property_path: &[usize]) -> Option<u64> {
        if property_path.len() == 1 {
            return self.trigger_value_by_property_index(property_path[0]);
        }
        let (property_index, view_model_path) = property_path.split_last()?;
        let view_model = self.view_model_by_property_path(view_model_path)?;
        view_model.active_trigger_value_by_property_index(*property_index)
    }

    fn view_model_value_by_property_path(
        &self,
        property_path: &[usize],
    ) -> Option<RuntimeViewModelPointer> {
        self.view_model_by_property_path(property_path)
            .map(|view_model| view_model.value)
    }
}

#[derive(Debug, Clone, Copy)]
struct RuntimeArtboardDimensions {
    width: f32,
    height: f32,
    origin_x: f32,
    origin_y: f32,
    clip: bool,
}

impl RuntimeArtboardDimensions {
    fn from_object(object: Option<&RuntimeObject>) -> Self {
        let width = object
            .and_then(|object| object.double_property("width"))
            .unwrap_or(0.0);
        let height = object
            .and_then(|object| object.double_property("height"))
            .unwrap_or(0.0);
        let origin_x = object
            .and_then(|object| object.double_property("originX"))
            .unwrap_or(0.0);
        let origin_y = object
            .and_then(|object| object.double_property("originY"))
            .unwrap_or(0.0);
        let clip = object
            .and_then(|object| object.bool_property("clip"))
            .unwrap_or(true);
        Self {
            width,
            height,
            origin_x,
            origin_y,
            clip,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeViewModelPointer {
    Null,
    DataContextRoot,
    OwnedGenerated {
        view_model_index: usize,
        property_index: usize,
        path_key: u64,
    },
    Imported {
        object_id: u32,
    },
}

fn artboard_index_for_graph(file: &RuntimeFile, graph: &ArtboardGraph) -> Option<usize> {
    file.artboards()
        .into_iter()
        .position(|artboard| artboard.id == graph.global_id)
}

fn runtime_property_name_by_key(object: &RuntimeObject, property_key: u16) -> Option<&'static str> {
    definition_by_type_key(object.type_key)?
        .property_by_key_in_hierarchy(property_key)
        .map(|property| property.name)
}

fn runtime_object_field_kind_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<FieldKind> {
    definition_by_type_key(object.type_key)?
        .property_by_key_in_hierarchy(property_key)
        .map(|property| property.runtime_type)
}

fn runtime_object_double_property_by_key(object: &RuntimeObject, property_key: u16) -> Option<f32> {
    object.double_property(runtime_property_name_by_key(object, property_key)?)
}

fn runtime_object_uint_property_by_key(object: &RuntimeObject, property_key: u16) -> Option<u64> {
    object.uint_property(runtime_property_name_by_key(object, property_key)?)
}

fn runtime_object_bool_property_by_key(object: &RuntimeObject, property_key: u16) -> Option<bool> {
    object.bool_property(runtime_property_name_by_key(object, property_key)?)
}

fn runtime_object_color_property_by_key(object: &RuntimeObject, property_key: u16) -> Option<u32> {
    object.color_property(runtime_property_name_by_key(object, property_key)?)
}

fn runtime_object_string_property_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<Vec<u8>> {
    runtime_object_string_property_bytes_by_key(object, property_key).map(|value| value.to_vec())
}

fn runtime_object_string_property_bytes_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<&[u8]> {
    let property =
        definition_by_type_key(object.type_key)?.property_by_key_in_hierarchy(property_key)?;
    match property.runtime_type {
        FieldKind::String => object.string_property_bytes(property.name),
        FieldKind::Bytes => object.bytes_property(property.name),
        _ => None,
    }
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

fn solid_color_value_property_key() -> Option<u16> {
    property_key_for_name("SolidColor", "colorValue")
}

fn shape_paint_is_visible_property_key() -> Option<u16> {
    property_key_for_name("ShapePaint", "isVisible")
}

fn solo_active_component_id_property_key() -> Option<u16> {
    property_key_for_name("Solo", "activeComponentId")
}

const JOYSTICK_FLAG_INVERT_X: u64 = 1 << 0;
const JOYSTICK_FLAG_INVERT_Y: u64 = 1 << 1;

fn joystick_x_property_key() -> Option<u16> {
    property_key_for_name("Joystick", "x")
}

fn joystick_y_property_key() -> Option<u16> {
    property_key_for_name("Joystick", "y")
}

fn joystick_flags_property_key() -> Option<u16> {
    property_key_for_name("Joystick", "joystickFlags")
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

#[derive(Debug, Clone)]
pub struct RuntimeDataContext<'a> {
    file: &'a RuntimeFile,
    current_instance: &'a RuntimeObject,
    parent_instances: Vec<&'a RuntimeObject>,
}

impl<'a> RuntimeDataContext<'a> {
    pub fn new(
        file: &'a RuntimeFile,
        view_model_index: usize,
        instance_index: usize,
    ) -> Option<Self> {
        let view_model = file.view_model(view_model_index)?;
        let instance = view_model.instances.get(instance_index)?;
        Self::from_instance_object(file, instance.object)
    }

    pub fn from_instance_reference(
        file: &'a RuntimeFile,
        instance: RuntimeViewModelInstanceReference<'a>,
    ) -> Option<Self> {
        Self::from_instance_object(file, instance.object)
    }

    pub fn from_instance_object(
        file: &'a RuntimeFile,
        instance: &'a RuntimeObject,
    ) -> Option<Self> {
        (instance.type_name == "ViewModelInstance").then_some(Self {
            file,
            current_instance: instance,
            parent_instances: Vec::new(),
        })
    }

    pub fn with_parent(mut self, parent: &RuntimeDataContext<'a>) -> Self {
        self.parent_instances.push(parent.current_instance);
        self.parent_instances
            .extend(parent.parent_instances.iter().copied());
        self
    }

    pub fn current_instance(&self) -> &'a RuntimeObject {
        self.current_instance
    }

    pub fn parent_instances(&self) -> &[&'a RuntimeObject] {
        &self.parent_instances
    }

    pub fn absolute_property(&self, path: &[u32]) -> Option<&'a RuntimeObject> {
        let chain = self.instance_chain();
        self.file
            .data_context_view_model_property_for_instance_chain(&chain, path)
    }

    pub fn absolute_property_ref(&self, path: &[u32]) -> Option<RuntimeDataContextValueRef> {
        let view_models = self.file.view_models();
        self.absolute_property(path)
            .and_then(|value| runtime_data_context_value_ref(self.file, &view_models, value))
    }

    pub fn absolute_instance(&self, path: &[u32]) -> Option<RuntimeViewModelInstanceReference<'a>> {
        let chain = self.instance_chain();
        self.file
            .data_context_view_model_instance_for_instance_chain(&chain, path)
    }

    pub fn absolute_instance_ref(&self, path: &[u32]) -> Option<RuntimeDataContextInstanceRef> {
        let view_models = self.file.view_models();
        self.absolute_instance(path)
            .and_then(|instance| runtime_data_context_instance_ref(&view_models, instance))
    }

    pub fn property_from_path(&self, path: &[u32]) -> Option<&'a RuntimeObject> {
        self.file
            .view_model_instance_property_from_path_for_object(self.current_instance, path)
    }

    pub fn property_from_path_ref(&self, path: &[u32]) -> Option<RuntimeDataContextValueRef> {
        let view_models = self.file.view_models();
        self.property_from_path(path)
            .and_then(|value| runtime_data_context_value_ref(self.file, &view_models, value))
    }

    pub fn relative_property(&self, path: &[u32]) -> Option<&'a RuntimeObject> {
        let chain = self.instance_chain();
        self.file
            .data_context_relative_view_model_property_for_instance_chain(&chain, path)
    }

    pub fn relative_property_ref(&self, path: &[u32]) -> Option<RuntimeDataContextValueRef> {
        let view_models = self.file.view_models();
        self.relative_property(path)
            .and_then(|value| runtime_data_context_value_ref(self.file, &view_models, value))
    }

    pub fn relative_instance(&self, path: &[u32]) -> Option<RuntimeViewModelInstanceReference<'a>> {
        let chain = self.instance_chain();
        self.file
            .data_context_relative_view_model_instance_for_instance_chain(&chain, path)
    }

    pub fn relative_instance_ref(&self, path: &[u32]) -> Option<RuntimeDataContextInstanceRef> {
        let view_models = self.file.view_models();
        self.relative_instance(path)
            .and_then(|instance| runtime_data_context_instance_ref(&view_models, instance))
    }

    fn instance_chain(&self) -> Vec<&'a RuntimeObject> {
        let mut chain = Vec::with_capacity(self.parent_instances.len() + 1);
        chain.push(self.current_instance);
        chain.extend(self.parent_instances.iter().copied());
        chain
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDataContextLookupReport {
    pub kind: RuntimeDataContextLookupKind,
    pub current_view_model_index: usize,
    pub current_instance_index: usize,
    pub parent_view_model_index: Option<usize>,
    pub parent_instance_index: Option<usize>,
    pub path: Vec<u32>,
    pub value: Option<RuntimeDataContextValueRef>,
    pub instance: Option<RuntimeDataContextInstanceRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeDataContextLookupKind {
    AbsoluteInstance,
    AbsoluteProperty,
    PropertyFromPath,
    RelativeProperty,
    RelativeInstance,
    AbsolutePropertyParentFallback,
    RelativePropertyParentFallback,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDataContextValueRef {
    pub view_model_index: usize,
    pub instance_index: usize,
    pub value_index: usize,
    pub core_type: u32,
    pub view_model_property_id: u32,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDataContextInstanceRef {
    pub view_model_index: usize,
    pub instance_index: usize,
    pub core_type: u32,
    pub name: String,
    pub view_model_id: u32,
}

pub fn runtime_data_context_lookup_reports(
    file: &RuntimeFile,
) -> Vec<RuntimeDataContextLookupReport> {
    let view_models = file.view_models();
    let manifest_name_ids = runtime_data_context_manifest_name_ids(file);
    let mut reports = Vec::new();

    for (view_model_index, view_model) in view_models.iter().enumerate() {
        for (instance_index, instance) in view_model.instances.iter().enumerate() {
            let Some(context) = RuntimeDataContext::from_instance_object(file, instance.object)
            else {
                continue;
            };
            let absolute_path = vec![runtime_object_u32_property(
                context.current_instance(),
                "viewModelId",
            )];
            collect_runtime_data_context_absolute_lookups(
                file,
                &view_models,
                &mut reports,
                &context,
                view_model_index,
                instance_index,
                instance,
                absolute_path,
                0,
            );
            collect_runtime_data_context_property_from_path_lookups(
                file,
                &view_models,
                &mut reports,
                &context,
                view_model_index,
                instance_index,
                instance,
                Vec::new(),
                0,
            );
            collect_runtime_data_context_relative_lookups(
                file,
                &view_models,
                &manifest_name_ids,
                &mut reports,
                &context,
                view_model_index,
                instance_index,
                instance,
                Vec::new(),
                0,
            );
        }
    }
    collect_runtime_data_context_parent_fallback_lookups(
        file,
        &view_models,
        &manifest_name_ids,
        &mut reports,
    );

    reports
}

fn collect_runtime_data_context_absolute_lookups<'a>(
    file: &'a RuntimeFile,
    view_models: &[RuntimeViewModel<'a>],
    reports: &mut Vec<RuntimeDataContextLookupReport>,
    context: &RuntimeDataContext<'a>,
    root_view_model_index: usize,
    root_instance_index: usize,
    instance: &RuntimeViewModelInstance<'a>,
    path: Vec<u32>,
    depth: usize,
) {
    if depth > 8 {
        return;
    }

    reports.push(RuntimeDataContextLookupReport {
        kind: RuntimeDataContextLookupKind::AbsoluteInstance,
        current_view_model_index: root_view_model_index,
        current_instance_index: root_instance_index,
        parent_view_model_index: None,
        parent_instance_index: None,
        path: path.clone(),
        value: None,
        instance: context.absolute_instance_ref(&path),
    });

    for value in &instance.values {
        let mut value_path = path.clone();
        value_path.push(runtime_object_u32_property(
            value.object,
            "viewModelPropertyId",
        ));
        reports.push(RuntimeDataContextLookupReport {
            kind: RuntimeDataContextLookupKind::AbsoluteProperty,
            current_view_model_index: root_view_model_index,
            current_instance_index: root_instance_index,
            parent_view_model_index: None,
            parent_instance_index: None,
            path: value_path.clone(),
            value: context.absolute_property_ref(&value_path),
            instance: None,
        });

        if value.object.type_name != "ViewModelInstanceViewModel" {
            continue;
        }
        let Some(reference) = file.referenced_view_model_instance_for_value_object(value.object)
        else {
            continue;
        };
        reports.push(RuntimeDataContextLookupReport {
            kind: RuntimeDataContextLookupKind::AbsoluteInstance,
            current_view_model_index: root_view_model_index,
            current_instance_index: root_instance_index,
            parent_view_model_index: None,
            parent_instance_index: None,
            path: value_path.clone(),
            value: None,
            instance: context.absolute_instance_ref(&value_path),
        });

        if let Some(referenced_instance) = runtime_view_model_instance_from_reference(
            view_models,
            reference.view_model_index,
            reference.instance_index,
        ) {
            collect_runtime_data_context_absolute_lookups(
                file,
                view_models,
                reports,
                context,
                root_view_model_index,
                root_instance_index,
                referenced_instance,
                value_path,
                depth + 1,
            );
        }
    }
}

fn collect_runtime_data_context_property_from_path_lookups<'a>(
    file: &'a RuntimeFile,
    view_models: &[RuntimeViewModel<'a>],
    reports: &mut Vec<RuntimeDataContextLookupReport>,
    context: &RuntimeDataContext<'a>,
    root_view_model_index: usize,
    root_instance_index: usize,
    instance: &RuntimeViewModelInstance<'a>,
    path: Vec<u32>,
    depth: usize,
) {
    if depth > 8 {
        return;
    }

    for value in &instance.values {
        let mut value_path = path.clone();
        value_path.push(runtime_object_u32_property(
            value.object,
            "viewModelPropertyId",
        ));
        reports.push(RuntimeDataContextLookupReport {
            kind: RuntimeDataContextLookupKind::PropertyFromPath,
            current_view_model_index: root_view_model_index,
            current_instance_index: root_instance_index,
            parent_view_model_index: None,
            parent_instance_index: None,
            path: value_path.clone(),
            value: context.property_from_path_ref(&value_path),
            instance: None,
        });

        if value.object.type_name != "ViewModelInstanceViewModel" {
            continue;
        }
        let Some(reference) = file.referenced_view_model_instance_for_value_object(value.object)
        else {
            continue;
        };
        if let Some(referenced_instance) = runtime_view_model_instance_from_reference(
            view_models,
            reference.view_model_index,
            reference.instance_index,
        ) {
            collect_runtime_data_context_property_from_path_lookups(
                file,
                view_models,
                reports,
                context,
                root_view_model_index,
                root_instance_index,
                referenced_instance,
                value_path,
                depth + 1,
            );
        }
    }
}

fn collect_runtime_data_context_relative_lookups<'a>(
    file: &'a RuntimeFile,
    view_models: &[RuntimeViewModel<'a>],
    manifest_name_ids: &[(Vec<u8>, u32)],
    reports: &mut Vec<RuntimeDataContextLookupReport>,
    context: &RuntimeDataContext<'a>,
    root_view_model_index: usize,
    root_instance_index: usize,
    instance: &RuntimeViewModelInstance<'a>,
    path: Vec<u32>,
    depth: usize,
) {
    if depth > 8 || manifest_name_ids.is_empty() {
        return;
    }

    for value in &instance.values {
        let Some(name) = file.view_model_instance_value_name_for_object(value.object) else {
            continue;
        };
        let Some(name_id) = runtime_data_context_name_id(manifest_name_ids, name.as_bytes()) else {
            continue;
        };

        let mut value_path = path.clone();
        value_path.push(name_id);
        reports.push(RuntimeDataContextLookupReport {
            kind: RuntimeDataContextLookupKind::RelativeProperty,
            current_view_model_index: root_view_model_index,
            current_instance_index: root_instance_index,
            parent_view_model_index: None,
            parent_instance_index: None,
            path: value_path.clone(),
            value: context.relative_property_ref(&value_path),
            instance: None,
        });

        if value.object.type_name != "ViewModelInstanceViewModel" {
            continue;
        }
        let Some(reference) = file.referenced_view_model_instance_for_value_object(value.object)
        else {
            continue;
        };
        reports.push(RuntimeDataContextLookupReport {
            kind: RuntimeDataContextLookupKind::RelativeInstance,
            current_view_model_index: root_view_model_index,
            current_instance_index: root_instance_index,
            parent_view_model_index: None,
            parent_instance_index: None,
            path: value_path.clone(),
            value: None,
            instance: context.relative_instance_ref(&value_path),
        });

        if let Some(referenced_instance) = runtime_view_model_instance_from_reference(
            view_models,
            reference.view_model_index,
            reference.instance_index,
        ) {
            collect_runtime_data_context_relative_lookups(
                file,
                view_models,
                manifest_name_ids,
                reports,
                context,
                root_view_model_index,
                root_instance_index,
                referenced_instance,
                value_path,
                depth + 1,
            );
        }
    }
}

fn collect_runtime_data_context_parent_fallback_lookups<'a>(
    file: &'a RuntimeFile,
    view_models: &[RuntimeViewModel<'a>],
    manifest_name_ids: &[(Vec<u8>, u32)],
    reports: &mut Vec<RuntimeDataContextLookupReport>,
) {
    if view_models.len() < 2 {
        return;
    }

    for (current_view_model_index, current_view_model) in view_models.iter().enumerate() {
        let Some(current_instance) = current_view_model.instances.first() else {
            continue;
        };
        for (parent_view_model_index, parent_view_model) in view_models.iter().enumerate() {
            if parent_view_model_index == current_view_model_index {
                continue;
            }
            let Some(parent_instance) = parent_view_model.instances.first() else {
                continue;
            };
            let Some(parent_value) = parent_instance.values.first() else {
                continue;
            };
            let Some(context) =
                RuntimeDataContext::from_instance_object(file, current_instance.object)
            else {
                continue;
            };
            let Some(parent_context) =
                RuntimeDataContext::from_instance_object(file, parent_instance.object)
            else {
                continue;
            };
            let context = context.with_parent(&parent_context);

            let absolute_path = vec![
                runtime_object_u32_property(parent_instance.object, "viewModelId"),
                runtime_object_u32_property(parent_value.object, "viewModelPropertyId"),
            ];
            reports.push(RuntimeDataContextLookupReport {
                kind: RuntimeDataContextLookupKind::AbsolutePropertyParentFallback,
                current_view_model_index,
                current_instance_index: 0,
                parent_view_model_index: Some(parent_view_model_index),
                parent_instance_index: Some(0),
                path: absolute_path.clone(),
                value: context.absolute_property_ref(&absolute_path),
                instance: None,
            });

            if let Some(name_id) = file
                .view_model_instance_value_name_for_object(parent_value.object)
                .and_then(|name| runtime_data_context_name_id(manifest_name_ids, name.as_bytes()))
            {
                let relative_path = vec![name_id];
                reports.push(RuntimeDataContextLookupReport {
                    kind: RuntimeDataContextLookupKind::RelativePropertyParentFallback,
                    current_view_model_index,
                    current_instance_index: 0,
                    parent_view_model_index: Some(parent_view_model_index),
                    parent_instance_index: Some(0),
                    path: relative_path.clone(),
                    value: context.relative_property_ref(&relative_path),
                    instance: None,
                });
            }
            return;
        }
    }
}

fn runtime_data_context_manifest_name_ids(file: &RuntimeFile) -> Vec<(Vec<u8>, u32)> {
    file.manifest()
        .map(|manifest| {
            manifest
                .names
                .iter()
                .filter_map(|(id, name)| {
                    u32::try_from(*id)
                        .ok()
                        .map(|id| (name.as_bytes().to_vec(), id))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_data_context_name_id(names: &[(Vec<u8>, u32)], name: &[u8]) -> Option<u32> {
    names
        .iter()
        .find_map(|(candidate, id)| (candidate.as_slice() == name).then_some(*id))
}

fn runtime_view_model_instance_from_reference<'models, 'file>(
    view_models: &'models [RuntimeViewModel<'file>],
    view_model_index: usize,
    instance_index: usize,
) -> Option<&'models RuntimeViewModelInstance<'file>> {
    view_models
        .get(view_model_index)?
        .instances
        .get(instance_index)
}

fn runtime_data_context_value_ref(
    file: &RuntimeFile,
    view_models: &[RuntimeViewModel<'_>],
    value: &RuntimeObject,
) -> Option<RuntimeDataContextValueRef> {
    for (view_model_index, view_model) in view_models.iter().enumerate() {
        for (instance_index, instance) in view_model.instances.iter().enumerate() {
            for (value_index, candidate) in instance.values.iter().enumerate() {
                if candidate.object.id != value.id {
                    continue;
                }
                return Some(RuntimeDataContextValueRef {
                    view_model_index,
                    instance_index,
                    value_index,
                    core_type: u32::from(value.type_key),
                    view_model_property_id: runtime_object_u32_property(
                        value,
                        "viewModelPropertyId",
                    ),
                    name: file
                        .view_model_instance_value_name_for_object(value)
                        .unwrap_or_default()
                        .to_owned(),
                });
            }
        }
    }

    None
}

fn runtime_data_context_instance_ref(
    view_models: &[RuntimeViewModel<'_>],
    reference: RuntimeViewModelInstanceReference<'_>,
) -> Option<RuntimeDataContextInstanceRef> {
    let instance = view_models
        .get(reference.view_model_index)?
        .instances
        .get(reference.instance_index)?;
    Some(RuntimeDataContextInstanceRef {
        view_model_index: reference.view_model_index,
        instance_index: reference.instance_index,
        core_type: u32::from(instance.object.type_key),
        name: instance
            .object
            .string_property("name")
            .unwrap_or_default()
            .to_owned(),
        view_model_id: runtime_object_u32_property(instance.object, "viewModelId"),
    })
}

fn runtime_object_u32_property(object: &RuntimeObject, property: &str) -> u32 {
    object
        .uint_property(property)
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_bind_graph::runtime_data_bind_graph_reverse_convert_value;
    use rive_binary::{BytesValue, FieldValue, RuntimeProperty, read_runtime_file};
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

        let slots = components
            .iter()
            .enumerate()
            .map(|(index, component)| InstanceSlot {
                local_id: component.local_id,
                source_global_id: component.global_id,
                type_name: Some(component.type_name),
                name: None,
                component_index: Some(index),
            })
            .collect::<Vec<_>>();
        let mut runtime_objects = vec![None; slots.len()];
        for component in &components {
            if component.local_id >= runtime_objects.len() {
                runtime_objects.resize(component.local_id + 1, None);
            }
            runtime_objects[component.local_id] = Some(synthetic_runtime_object(
                component.global_id,
                component.type_name,
                Vec::new(),
            ));
        }
        let objects = InstanceObjectArena::from_runtime_objects(runtime_objects);

        ArtboardInstance {
            width: 0.0,
            height: 0.0,
            origin_x: 0.0,
            origin_y: 0.0,
            clip: true,
            slots,
            objects,
            components,
            component_by_local,
            solos: Vec::new(),
            joysticks: Vec::new(),
            joysticks_apply_before_update: true,
            update_order,
            linear_animations: Vec::new(),
            state_machines: Vec::new(),
            artboard_data_bind_values: BTreeMap::new(),
            artboard_custom_property_bindings: Vec::new(),
            artboard_solo_bindings: Vec::new(),
            artboard_list_bindings: Vec::new(),
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
    fn range_mapper_reverse_conversion_swaps_input_and_output_ranges() {
        let converter = RuntimeDataBindGraphConverter::RangeMapper {
            min_input: 0.0,
            max_input: 10.0,
            min_output: 100.0,
            max_output: 200.0,
            flags: 0,
            interpolation_type: 1,
            interpolator: None,
        };

        let Some(RuntimeDataBindGraphValue::Number(value)) =
            runtime_data_bind_graph_reverse_convert_value(
                &converter,
                &RuntimeDataBindGraphValue::Number(160.0),
            )
        else {
            panic!("range mapper reverse conversion did not return a number");
        };

        assert!(
            (value - 6.0).abs() <= 0.0001,
            "range mapper reverse conversion mismatch: expected 6, got {value}"
        );
    }

    #[test]
    fn range_mapper_reverse_conversion_preserves_reverse_flag() {
        let converter = RuntimeDataBindGraphConverter::RangeMapper {
            min_input: 0.0,
            max_input: 10.0,
            min_output: 100.0,
            max_output: 200.0,
            flags: 1 << 3,
            interpolation_type: 1,
            interpolator: None,
        };

        let Some(RuntimeDataBindGraphValue::Number(value)) =
            runtime_data_bind_graph_reverse_convert_value(
                &converter,
                &RuntimeDataBindGraphValue::Number(160.0),
            )
        else {
            panic!("range mapper reverse conversion did not return a number");
        };

        assert!(
            (value - 4.0).abs() <= 0.0001,
            "range mapper reverse conversion mismatch: expected 4, got {value}"
        );
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

    fn synthetic_runtime_object(
        id: u32,
        type_name: &'static str,
        properties: Vec<RuntimeProperty>,
    ) -> RuntimeObject {
        let definition = definition_by_name(type_name).expect("synthetic runtime object type");
        RuntimeObject {
            id,
            type_key: definition.type_key.int,
            type_name: definition.name,
            rust_variant: definition.rust_variant,
            properties,
            skipped_properties: Vec::new(),
        }
    }

    #[test]
    fn instance_object_arena_uses_generated_core_registry_setter_families() {
        let node_x_key = property_key_for_name("Node", "x").expect("Node.x key");
        let bytes_key =
            property_key_for_name("FileAssetContents", "bytes").expect("FileAssetContents.bytes");
        let mut arena = InstanceObjectArena::from_runtime_objects(vec![
            Some(synthetic_runtime_object(0, "Node", Vec::new())),
            Some(synthetic_runtime_object(1, "FileAssetContents", Vec::new())),
        ]);

        assert!(arena.set_double_property(0, node_x_key, 12.5));
        assert_eq!(arena.double_property(0, node_x_key), Some(12.5));

        assert!(!arena.set_uint_property(0, node_x_key, 12));
        assert_eq!(arena.double_property(0, node_x_key), Some(12.5));

        assert!(!arena.set_string_property(1, bytes_key, vec![1, 2, 3]));
        assert_eq!(arena.string_property(1, bytes_key), None);
    }

    #[test]
    fn instance_object_arena_keeps_mutable_properties_in_instance_storage() {
        let node_x_key = property_key_for_name("Node", "x").expect("Node.x key");
        let source = synthetic_runtime_object(0, "Node", Vec::new());
        let mut arena = InstanceObjectArena::from_runtime_objects(vec![Some(source.clone())]);

        assert!(arena.set_double_property(0, node_x_key, 42.0));

        assert!(source.properties.is_empty());
        assert_eq!(arena.double_property(0, node_x_key), Some(42.0));
    }

    #[test]
    fn instance_object_arena_reads_generated_defaults_and_imported_fields() {
        let node_x_key = property_key_for_name("Node", "x").expect("Node.x key");
        let artboard_clip_key = property_key_for_name("Artboard", "clip").expect("Artboard.clip");
        let bytes_key =
            property_key_for_name("FileAssetContents", "bytes").expect("FileAssetContents.bytes");
        let arena = InstanceObjectArena::from_runtime_objects(vec![
            Some(synthetic_runtime_object(
                0,
                "Node",
                vec![RuntimeProperty {
                    key: node_x_key,
                    name: "x",
                    owner: "Node",
                    value: FieldValue::Double(7.5),
                }],
            )),
            Some(synthetic_runtime_object(1, "Artboard", Vec::new())),
            Some(synthetic_runtime_object(
                2,
                "FileAssetContents",
                vec![RuntimeProperty {
                    key: bytes_key,
                    name: "bytes",
                    owner: "FileAssetContents",
                    value: FieldValue::Bytes(BytesValue::new(vec![1, 2, 3])),
                }],
            )),
        ]);

        assert_eq!(arena.double_property(0, node_x_key), Some(7.5));
        assert_eq!(arena.bool_property(1, artboard_clip_key), Some(true));
        assert_eq!(arena.string_property(2, bytes_key), Some(&[1, 2, 3][..]));
    }

    #[test]
    fn update_transform_reads_generated_instance_storage() {
        let node_x_key = property_key_for_name("Node", "x").expect("Node.x key");
        let node_scale_x_key = property_key_for_name("Node", "scaleX").expect("Node.scaleX key");
        let mut component = synthetic_component(0, 0);
        component.dirt = ComponentDirt::TRANSFORM;
        let mut instance = synthetic_instance(vec![component], vec![0]);

        assert!(instance.objects.set_double_property_by_name(0, "x", 8.0));
        assert!(
            instance
                .objects
                .set_double_property_by_name(0, "scaleX", 2.5)
        );

        let report = instance.update_components();

        assert_eq!(report.updated_locals, vec![0]);
        assert_eq!(instance.double_property(0, node_x_key), Some(8.0));
        assert_eq!(instance.double_property(0, node_scale_x_key), Some(2.5));
        assert_eq!(
            instance.component(0).unwrap().transform.local_transform,
            Mat2D([2.5, 0.0, -0.0, 1.0, 8.0, 0.0])
        );
    }

    #[test]
    fn transform_update_matches_basic_cpp_order() {
        let mut root = synthetic_component(0, 0);
        root.type_name = "Artboard";
        root.transform.render_opacity = 0.5;
        let mut child = synthetic_component(1, 1);
        child.parent_local = Some(0);
        child.dirt = ComponentDirt::TRANSFORM
            | ComponentDirt::WORLD_TRANSFORM
            | ComponentDirt::RENDER_OPACITY;
        let mut instance = synthetic_instance(vec![root, child], vec![0, 1]);
        assert!(instance.objects.set_double_property_by_name(1, "x", 2.0));
        assert!(instance.objects.set_double_property_by_name(1, "y", 3.0));
        assert!(
            instance
                .objects
                .set_double_property_by_name(1, "scaleX", 4.0)
        );
        assert!(
            instance
                .objects
                .set_double_property_by_name(1, "scaleY", 5.0)
        );
        assert!(
            instance
                .objects
                .set_double_property_by_name(1, "opacity", 0.25)
        );

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
        let node_x_key = property_key_for_name("Node", "x").expect("Node.x key");
        let mut component = synthetic_component(0, 0);
        component.dirt = ComponentDirt::NONE;
        let mut instance = synthetic_instance(vec![component], vec![0]);
        instance.dirt = ComponentDirt::NONE;
        instance.did_change = false;

        assert!(instance.set_transform_property(0, TransformProperty::X, 12.0));
        let component = instance.component(0).unwrap();
        assert_eq!(
            instance.transform_property(0, TransformProperty::X),
            Some(12.0)
        );
        assert_eq!(instance.double_property(0, node_x_key), Some(12.0));
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
    fn transform_property_mutation_writes_generated_storage_by_concrete_type() {
        let node_x_key = property_key_for_name("Node", "x").expect("Node.x key");
        let vertex_x_key = property_key_for_name("StraightVertex", "x").expect("StraightVertex.x");
        let mut vertex = synthetic_component(0, 0);
        vertex.type_name = "StraightVertex";
        let mut instance = synthetic_instance(vec![vertex], vec![0]);

        assert!(instance.set_transform_property(0, TransformProperty::X, 14.0));

        assert_eq!(
            instance.transform_property(0, TransformProperty::X),
            Some(14.0)
        );
        assert_eq!(instance.double_property(0, vertex_x_key), Some(14.0));
        assert_eq!(instance.double_property(0, node_x_key), None);
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
        assert_eq!(
            instance.transform_property(0, TransformProperty::Opacity),
            Some(0.35)
        );
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
