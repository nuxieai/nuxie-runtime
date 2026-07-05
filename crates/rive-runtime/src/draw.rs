use anyhow::{Context, Result};
use rive_binary::{RuntimeFile, RuntimeObject};
use rive_graph::{
    ArtboardGraph, ClippingShapeNode, DashNode, DrawableOrderKind, DrawableOrderNode, FeatherNode,
    GradientStopNode, ParametricPathNode, PathComposerNode, PathComposerPathNode, PathGeometryNode,
    PathVertexNode, ShapePaintContainerNode, ShapePaintKind, ShapePaintNode, ShapePaintPathKind,
    ShapePaintStateNode, SortedDrawableNode, StrokeEffectNode,
};
use rive_render_api::{
    BlendMode as RenderBlendMode, Factory as RenderFactory, FillRule as RenderFillRule,
    Mat2D as RenderMat2D, RenderPaint, RenderPaintStyle, RenderPath, RenderShader, Renderer,
    StrokeCap as RenderStrokeCap, StrokeJoin as RenderStrokeJoin,
};
use rive_schema::definition_by_name;
use std::collections::{BTreeMap, BTreeSet, btree_map::Entry};

use crate::properties::{
    property_key_for_name, shape_paint_is_visible_property_key, solid_color_value_property_key,
};
use crate::text::runtime_text_shape_paint_commands;
use crate::{ArtboardInstance, Mat2D, RuntimeComponent};

impl ArtboardInstance {
    pub fn draw_commands(&self, graph: &ArtboardGraph) -> Vec<RuntimeDrawCommand> {
        let mut commands = Vec::new();
        let mut pending_clip_operations = Vec::<&SortedDrawableNode>::new();
        let mut empty_clips = 0i32;

        for drawable in &self.runtime_sorted_drawable_order(graph) {
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

    fn runtime_sorted_drawable_order(&self, graph: &ArtboardGraph) -> Vec<SortedDrawableNode> {
        let Some(draw_target_id_key) = property_key_for_name("DrawRules", "drawTargetId") else {
            return graph.sorted_drawable_order.clone();
        };
        let Some(placement_value_key) = property_key_for_name("DrawTarget", "placementValue")
        else {
            return graph.sorted_drawable_order.clone();
        };

        let draw_targets_by_local = graph
            .draw_targets
            .iter()
            .map(|target| (target.local_id, target))
            .collect::<BTreeMap<_, _>>();
        let active_target_by_rules = graph
            .draw_rules
            .iter()
            .filter_map(|rules| {
                let target_local =
                    usize::try_from(self.uint_property(rules.local_id, draw_target_id_key)?)
                        .ok()?;
                draw_targets_by_local
                    .contains_key(&target_local)
                    .then_some((rules.local_id, target_local))
            })
            .collect::<BTreeMap<_, _>>();
        let mut main = Vec::new();
        let mut grouped = BTreeMap::<usize, Vec<SortedDrawableNode>>::new();
        for drawable in &graph.drawable_order {
            let draw_target_local = drawable
                .flattened_draw_rules_local
                .and_then(|rules_local| active_target_by_rules.get(&rules_local).copied());
            let node = runtime_sorted_drawable_node(drawable, draw_target_local);
            if let Some(draw_target_local) = draw_target_local {
                grouped.entry(draw_target_local).or_default().push(node);
            } else {
                main.push(node);
            }
        }

        for draw_target_local in &graph.draw_target_order {
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
            let placement_value = self
                .uint_property(target.local_id, placement_value_key)
                .unwrap_or(target.placement_value);
            let insert_at = match placement_value {
                0 => target_position,
                1 => target_position + 1,
                _ => continue,
            };
            main.splice(insert_at..insert_at, group);
        }

        let mut sorted = runtime_interleave_clipping_proxy_drawables(
            main.into_iter().rev(),
            &graph.clipping_shapes,
        );
        runtime_apply_save_operation_elision(&mut sorted, &graph.clipping_shapes);
        sorted
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
            std::slice::from_ref(graph),
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
                if self.runtime_gradient_paint_is_collapsed(container, paint) {
                    continue;
                }
                if !prepared.insert(paint.global_id) {
                    continue;
                }
                let object = runtime
                    .object(paint.global_id as usize)
                    .with_context(|| format!("missing paint global {}", paint.global_id))?;
                let opacity_local = shape_paint_container_opacity_local(graph, container.local_id);
                let runtime_paint =
                    runtime_prepare_gradient_paint_command(self, opacity_local, container, paint);
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

    pub fn prepare_static_artboard_tree_paints(
        &self,
        runtime: &RuntimeFile,
        graph: &ArtboardGraph,
        artboards: &[ArtboardGraph],
        factory: &mut dyn RenderFactory,
        paint_cache: &mut RuntimeRenderPaintCache,
        render_cache: &mut RuntimeRenderPathCache,
    ) -> Result<()> {
        self.prepare_static_artboard_tree_paints_internal(
            runtime,
            graph,
            artboards,
            factory,
            &mut paint_cache.paints,
            Some(&mut paint_cache.nested_artboards),
            render_cache,
        )
    }

    fn prepare_static_artboard_tree_paints_internal(
        &self,
        runtime: &RuntimeFile,
        graph: &ArtboardGraph,
        artboards: &[ArtboardGraph],
        factory: &mut dyn RenderFactory,
        paint_by_global: &mut BTreeMap<u32, Box<dyn RenderPaint>>,
        mut nested_paint_caches: Option<&mut BTreeMap<u32, RuntimeRenderPaintCache>>,
        render_cache: &mut RuntimeRenderPathCache,
    ) -> Result<()> {
        self.prepare_static_artboard_paints(
            runtime,
            graph,
            factory,
            paint_by_global,
            render_cache,
        )?;

        for command in self.draw_commands(graph) {
            if !sorted_drawable_is_nested_artboard(command.type_name) {
                continue;
            }

            let referenced_artboard_global = command
                .referenced_artboard_global
                .context("nested artboard missing referenced artboard")?;
            let child_graph = artboards
                .iter()
                .find(|graph| graph.global_id == referenced_artboard_global)
                .with_context(|| {
                    format!("missing nested artboard graph for global {referenced_artboard_global}")
                })?;
            let cache_key = nested_render_cache_key(
                command.global_id,
                command.local_id,
                referenced_artboard_global,
            );
            let child_cache = render_cache.nested_artboards.entry(cache_key).or_default();
            let child_paint_caches = match &mut nested_paint_caches {
                Some(caches) => {
                    let child_paint_cache = caches.entry(cache_key).or_default();
                    Some((
                        &mut child_paint_cache.paints,
                        Some(&mut child_paint_cache.nested_artboards),
                    ))
                }
                None => None,
            };

            if let Some(child) = command
                .local_id
                .and_then(|local_id| self.nested_artboards.get(&local_id))
                .map(|nested| nested.child.as_ref())
            {
                if let Some((child_paints, child_nested_paints)) = child_paint_caches {
                    child.prepare_static_artboard_tree_paints_internal(
                        runtime,
                        child_graph,
                        artboards,
                        factory,
                        child_paints,
                        child_nested_paints,
                        child_cache,
                    )?;
                } else {
                    child.prepare_static_artboard_tree_paints_internal(
                        runtime,
                        child_graph,
                        artboards,
                        factory,
                        paint_by_global,
                        None,
                        child_cache,
                    )?;
                }
                continue;
            }

            let mut child = ArtboardInstance::from_graph(runtime, child_graph)?;
            let host_opacity = command
                .local_id
                .and_then(|local_id| self.component(local_id))
                .map(|component| component.transform.render_opacity)
                .unwrap_or(1.0);
            if let Some(opacity_key) = property_key_for_name("Artboard", "opacity") {
                child.set_double_property(0, opacity_key, host_opacity);
            }
            child.update_pass();
            if let Some((child_paints, child_nested_paints)) = child_paint_caches {
                child.prepare_static_artboard_tree_paints_internal(
                    runtime,
                    child_graph,
                    artboards,
                    factory,
                    child_paints,
                    child_nested_paints,
                    child_cache,
                )?;
            } else {
                child.prepare_static_artboard_tree_paints_internal(
                    runtime,
                    child_graph,
                    artboards,
                    factory,
                    paint_by_global,
                    None,
                    child_cache,
                )?;
            }
        }

        Ok(())
    }

    fn runtime_gradient_paint_is_collapsed(
        &self,
        container: &ShapePaintContainerNode,
        paint: &ShapePaintNode,
    ) -> bool {
        [
            Some(container.local_id),
            Some(paint.local_id),
            paint.mutator_local,
        ]
        .into_iter()
        .flatten()
        .any(|local_id| {
            self.component(local_id)
                .is_some_and(RuntimeComponent::is_collapsed)
        })
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
            std::slice::from_ref(graph),
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
        artboards: &[ArtboardGraph],
        factory: &mut dyn RenderFactory,
        renderer: &mut dyn Renderer,
        paint_by_global: &mut BTreeMap<u32, Box<dyn RenderPaint>>,
        path_cache: &mut RuntimeRenderPathCache,
    ) -> Result<()> {
        self.draw_prepared_static_artboard_internal_with_path_cache(
            runtime,
            graph,
            artboards,
            factory,
            renderer,
            paint_by_global,
            path_cache,
            None,
            true,
        )
    }

    pub fn draw_prepared_static_artboard_with_render_cache(
        &self,
        runtime: &RuntimeFile,
        graph: &ArtboardGraph,
        artboards: &[ArtboardGraph],
        factory: &mut dyn RenderFactory,
        renderer: &mut dyn Renderer,
        paint_cache: &mut RuntimeRenderPaintCache,
        path_cache: &mut RuntimeRenderPathCache,
    ) -> Result<()> {
        self.draw_prepared_static_artboard_internal_with_path_cache(
            runtime,
            graph,
            artboards,
            factory,
            renderer,
            &mut paint_cache.paints,
            path_cache,
            Some(&mut paint_cache.nested_artboards),
            true,
        )
    }

    fn draw_prepared_static_artboard_internal_with_path_cache(
        &self,
        runtime: &RuntimeFile,
        graph: &ArtboardGraph,
        artboards: &[ArtboardGraph],
        factory: &mut dyn RenderFactory,
        renderer: &mut dyn Renderer,
        paint_by_global: &mut BTreeMap<u32, Box<dyn RenderPaint>>,
        path_cache: &mut RuntimeRenderPathCache,
        mut nested_paint_caches: Option<&mut BTreeMap<u32, RuntimeRenderPaintCache>>,
        apply_origin_transform: bool,
    ) -> Result<()> {
        if self
            .component(0)
            .is_some_and(|component| component.transform.render_opacity == 0.0)
        {
            return Ok(());
        }

        let needs_save = self.clip || apply_origin_transform;
        if needs_save {
            renderer.save();
        }
        if self.clip {
            let (clip_left, clip_top) = if apply_origin_transform {
                (0.0, 0.0)
            } else {
                (-self.origin_x * self.width, -self.origin_y * self.height)
            };
            let clip = path_cache.artboard_clip_path(
                factory,
                &runtime_rect_commands(
                    clip_left,
                    clip_top,
                    clip_left + self.width,
                    clip_top + self.height,
                ),
                RenderFillRule::Clockwise,
            );
            renderer.clip_path(clip.as_ref());
        }
        if apply_origin_transform {
            renderer.transform(self.artboard_origin_transform());
        }

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
                artboards,
                &local_to_global,
                command,
                factory,
                renderer,
                paint_by_global,
                path_cache,
                match &mut nested_paint_caches {
                    Some(caches) => Some(&mut **caches),
                    None => None,
                },
            )?;
        }

        if needs_save {
            renderer.restore();
        }
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
            local_id: match drawable.kind {
                DrawableOrderKind::LayoutProxy => drawable.layout_local,
                _ => drawable.local_id,
            },
            global_id: drawable.global_id,
            type_name: drawable.type_name,
            referenced_artboard_global: drawable.referenced_artboard_global,
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
        let shape_local = match drawable.kind {
            DrawableOrderKind::Drawable if drawable.type_name == "Shape" => drawable.local_id,
            DrawableOrderKind::LayoutProxy => drawable.layout_local,
            _ => None,
        };
        let Some(shape_local) = shape_local else {
            return Vec::new();
        };
        let Some(container) = graph
            .shape_paint_containers
            .iter()
            .find(|container| container.local_id == shape_local)
        else {
            return Vec::new();
        };
        if !matches!(container.type_name, "Shape" | "LayoutComponent") {
            return Vec::new();
        }
        let needs_save_operation = drawable.needs_save_operation || container.paints.len() > 1;
        let render_opacity = self
            .component(shape_local)
            .map(|component| component.transform.render_opacity)
            .unwrap_or(1.0);
        let shape_world = if container.type_name == "LayoutComponent" {
            self.runtime_layout_component_world_transform(shape_local, graph)
        } else {
            self.component(shape_local)
                .map(|component| component.transform.world_transform)
                .unwrap_or(Mat2D::IDENTITY)
        };

        container
            .paints
            .iter()
            .filter_map(|paint| {
                let path_commands = match container.type_name {
                    "Shape" => self.runtime_shape_paint_path_commands(shape_local, paint, graph),
                    "LayoutComponent" => {
                        self.runtime_layout_component_paint_path_commands(shape_local, paint, graph)
                    }
                    _ => Vec::new(),
                };
                let mut command = runtime_shape_paint_command(
                    self,
                    paint,
                    container.blend_mode_value,
                    needs_save_operation,
                    render_opacity,
                    shape_world,
                    path_commands,
                )?;
                if drawable.kind == DrawableOrderKind::LayoutProxy {
                    command.shape_world_override = Some(shape_world);
                }
                Some(command)
            })
            .collect()
    }

    fn runtime_layout_component_paint_path_commands(
        &self,
        layout_local: usize,
        paint: &ShapePaintNode,
        graph: &ArtboardGraph,
    ) -> Vec<RuntimePathCommand> {
        if paint.path_kind.is_none() {
            return Vec::new();
        }
        // Ported from C++ `src/layout_component.cpp`
        // `LayoutComponent::drawProxy/updateRenderPath`: layout components draw
        // shape paints against a background rectangle built from computed
        // layout bounds. TODO(golden): route all layout components through the
        // M6 layout engine.
        let bounds = self.runtime_layout_component_bounds(layout_local, graph);
        let corners = self.runtime_layout_component_corners(layout_local);
        runtime_layout_rect_path_commands(bounds, corners)
    }

    fn runtime_layout_component_clip_path_commands(
        &self,
        layout_local: usize,
        graph: &ArtboardGraph,
    ) -> Vec<RuntimePathCommand> {
        let bounds = self.runtime_layout_component_bounds(layout_local, graph);
        let corners = self.runtime_layout_component_corners(layout_local);
        runtime_layout_rect_path_commands(bounds, corners)
    }

    fn runtime_layout_component_clip_enabled(&self, layout_local: usize) -> bool {
        property_key_for_name("LayoutComponent", "clip")
            .and_then(|key| self.bool_property(layout_local, key))
            .unwrap_or(false)
    }

    fn runtime_layout_component_world_transform(
        &self,
        layout_local: usize,
        graph: &ArtboardGraph,
    ) -> Mat2D {
        let bounds = self
            .runtime_simple_root_row_fill_layout_bounds(layout_local, graph)
            .or_else(|| self.runtime_nested_fill_hug_layout_bounds(layout_local, graph));
        if let Some(bounds) = bounds {
            return Mat2D([1.0, 0.0, 0.0, 1.0, bounds.x, bounds.y]);
        }
        self.component(layout_local)
            .map(|component| component.transform.world_transform)
            .unwrap_or(Mat2D::IDENTITY)
    }

    fn runtime_layout_component_bounds(
        &self,
        layout_local: usize,
        graph: &ArtboardGraph,
    ) -> RuntimeLayoutBounds {
        if let Some(bounds) = self.runtime_simple_root_row_fill_layout_bounds(layout_local, graph) {
            return bounds;
        }
        if let Some(bounds) = self.runtime_nested_fill_hug_layout_bounds(layout_local, graph) {
            return bounds;
        }
        let width = self.runtime_layout_component_dimension(layout_local, "width");
        let height = self.runtime_layout_component_dimension(layout_local, "height");
        if self
            .component(layout_local)
            .is_some_and(|component| component.parent_local == Some(0))
        {
            return RuntimeLayoutBounds {
                x: 0.0,
                y: 0.0,
                width: if width == 0.0 { self.width } else { width },
                height: if height == 0.0 { self.height } else { height },
            };
        }
        RuntimeLayoutBounds {
            x: 0.0,
            y: 0.0,
            width,
            height,
        }
    }

    fn runtime_nested_fill_hug_layout_bounds(
        &self,
        layout_local: usize,
        graph: &ArtboardGraph,
    ) -> Option<RuntimeLayoutBounds> {
        let component = graph
            .components
            .iter()
            .find(|component| component.local_id == layout_local)?;
        if component.type_name != "LayoutComponent" {
            return None;
        }
        let parent_local = component.parent_local?;
        let parent = graph
            .components
            .iter()
            .find(|component| component.local_id == parent_local)?;
        if parent.type_name != "LayoutComponent" {
            return None;
        }
        let style_local = self.runtime_layout_component_style_local(layout_local)?;

        const LAYOUT_SCALE_TYPE_FILL: u64 = 1;
        const LAYOUT_SCALE_TYPE_HUG: u64 = 2;
        let width_scale = self.runtime_layout_style_uint(style_local, "layoutWidthScaleType")?;
        let height_scale = self.runtime_layout_style_uint(style_local, "layoutHeightScaleType")?;
        if !matches!(width_scale, LAYOUT_SCALE_TYPE_FILL | LAYOUT_SCALE_TYPE_HUG)
            || !matches!(height_scale, LAYOUT_SCALE_TYPE_FILL | LAYOUT_SCALE_TYPE_HUG)
        {
            return None;
        }

        let parent_bounds = self.runtime_layout_component_bounds(parent_local, graph);
        let authored_width = self.runtime_layout_component_dimension(layout_local, "width");
        let authored_height = self.runtime_layout_component_dimension(layout_local, "height");
        let width = match width_scale {
            LAYOUT_SCALE_TYPE_FILL => parent_bounds.width,
            LAYOUT_SCALE_TYPE_HUG => {
                self.runtime_layout_component_hug_size(layout_local, graph, true)?
            }
            _ => authored_width,
        };
        let height = match height_scale {
            LAYOUT_SCALE_TYPE_FILL => parent_bounds.height,
            LAYOUT_SCALE_TYPE_HUG => {
                self.runtime_layout_component_hug_size(layout_local, graph, false)?
            }
            _ => authored_height,
        };

        Some(RuntimeLayoutBounds {
            x: 0.0,
            y: 0.0,
            width,
            height,
        })
    }

    fn runtime_layout_component_hug_size(
        &self,
        layout_local: usize,
        graph: &ArtboardGraph,
        width_axis: bool,
    ) -> Option<f32> {
        let style_local = self.runtime_layout_component_style_local(layout_local)?;
        let is_row = self.runtime_layout_style_is_row(style_local);
        let gap = if is_row {
            self.runtime_layout_style_double(style_local, "gapHorizontal")
        } else {
            self.runtime_layout_style_double(style_local, "gapVertical")
        }
        .unwrap_or(0.0);
        let component = graph
            .components
            .iter()
            .find(|component| component.local_id == layout_local)?;
        let mut child_sizes = Vec::new();
        for child_local in &component.children {
            let Some(child) = graph
                .components
                .iter()
                .find(|component| component.local_id == *child_local)
            else {
                continue;
            };
            match child.type_name {
                "LayoutComponent" => {
                    let bounds = self.runtime_layout_component_bounds(*child_local, graph);
                    child_sizes.push((bounds.width, bounds.height));
                }
                "ArtboardComponentList" => {
                    // Ported from C++ `src/artboard_component_list.cpp`
                    // `ArtboardComponentList::layoutBounds`: non-virtualized
                    // lists with no instantiated items retain the default
                    // zero layout size, which makes hug-sized parents collapse.
                    child_sizes.push((0.0, 0.0));
                }
                "Fill"
                | "LinearGradient"
                | "RadialGradient"
                | "SolidColor"
                | "Stroke"
                | "LayoutComponentStyle" => {}
                _ => return None,
            }
        }
        Some(runtime_layout_hug_size(
            &child_sizes,
            gap,
            is_row,
            width_axis,
        ))
    }

    fn runtime_layout_style_double(&self, style_local: usize, name: &str) -> Option<f32> {
        property_key_for_name("LayoutComponentStyle", name)
            .and_then(|key| self.double_property(style_local, key))
    }

    fn runtime_layout_style_bool(&self, style_local: usize, name: &str) -> Option<bool> {
        property_key_for_name("LayoutComponentStyle", name)
            .and_then(|key| self.bool_property(style_local, key))
    }

    fn runtime_layout_component_corners(&self, layout_local: usize) -> RuntimeLayoutCorners {
        let Some(style_local) = self.runtime_layout_component_style_local(layout_local) else {
            return RuntimeLayoutCorners::default();
        };
        let linked_value = self
            .runtime_layout_style_double(style_local, "cornerRadiusTL")
            .unwrap_or(0.0);
        let linked = self
            .runtime_layout_style_bool(style_local, "linkCornerRadius")
            .unwrap_or(true);
        if linked {
            return RuntimeLayoutCorners {
                top_left: linked_value,
                top_right: linked_value,
                bottom_right: linked_value,
                bottom_left: linked_value,
            };
        }
        RuntimeLayoutCorners {
            top_left: linked_value,
            top_right: self
                .runtime_layout_style_double(style_local, "cornerRadiusTR")
                .unwrap_or(0.0),
            bottom_right: self
                .runtime_layout_style_double(style_local, "cornerRadiusBR")
                .unwrap_or(0.0),
            bottom_left: self
                .runtime_layout_style_double(style_local, "cornerRadiusBL")
                .unwrap_or(0.0),
        }
    }

    fn runtime_simple_root_row_fill_layout_bounds(
        &self,
        layout_local: usize,
        graph: &ArtboardGraph,
    ) -> Option<RuntimeLayoutBounds> {
        if !self.runtime_layout_component_is_root_row_fill_child(layout_local) {
            return None;
        }
        if !self.runtime_root_layout_style_is_row() {
            return None;
        }
        let root = graph
            .components
            .iter()
            .find(|component| component.local_id == 0)?;
        let layout_children = root
            .children
            .iter()
            .copied()
            .filter(|child_local| {
                self.runtime_layout_component_is_root_row_fill_child(*child_local)
            })
            .collect::<Vec<_>>();
        if layout_children.len() < 2 {
            return None;
        }
        let child_index = layout_children
            .iter()
            .position(|child_local| *child_local == layout_local)?;
        let width = self.width / layout_children.len() as f32;
        Some(RuntimeLayoutBounds {
            x: width * child_index as f32,
            y: 0.0,
            width,
            height: self.height,
        })
    }

    fn runtime_layout_component_is_root_row_fill_child(&self, layout_local: usize) -> bool {
        let Some(component) = self.component(layout_local) else {
            return false;
        };
        if component.type_name != "LayoutComponent" || component.parent_local != Some(0) {
            return false;
        }
        let Some(style_local) = self.runtime_layout_component_style_local(layout_local) else {
            return false;
        };

        const LAYOUT_SCALE_TYPE_FILL: u64 = 1;
        self.runtime_layout_style_uint(style_local, "layoutWidthScaleType")
            == Some(LAYOUT_SCALE_TYPE_FILL)
            && self.runtime_layout_style_uint(style_local, "layoutHeightScaleType")
                == Some(LAYOUT_SCALE_TYPE_FILL)
    }

    fn runtime_root_layout_style_is_row(&self) -> bool {
        self.runtime_layout_component_style_local(0)
            .is_none_or(|style_local| self.runtime_layout_style_is_row(style_local))
    }

    fn runtime_layout_style_is_row(&self, style_local: usize) -> bool {
        // Ported from C++ `src/layout_component.cpp` `mainAxisIsRow`.
        matches!(
            self.runtime_layout_style_uint(style_local, "flexDirectionValue")
                .unwrap_or(2),
            2 | 3
        )
    }

    fn runtime_layout_component_style_local(&self, layout_local: usize) -> Option<usize> {
        property_key_for_name("LayoutComponent", "styleId")
            .and_then(|key| self.uint_property(layout_local, key))
            .and_then(|style| usize::try_from(style).ok())
    }

    fn runtime_layout_style_uint(&self, style_local: usize, name: &str) -> Option<u64> {
        property_key_for_name("LayoutComponentStyle", name)
            .and_then(|key| self.uint_property(style_local, key))
    }

    fn runtime_layout_component_dimension(&self, layout_local: usize, name: &str) -> f32 {
        property_key_for_name("LayoutComponent", name)
            .and_then(|key| self.double_property(layout_local, key))
            .unwrap_or(0.0)
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
            let runtime_path = runtime_path_geometry(self, path);
            let weighted_context = self.runtime_weighted_path_context(&runtime_path, graph);
            let path_transform = if weighted_context.is_some() {
                Mat2D::IDENTITY
            } else {
                path_world
            };
            let transform = match path_kind {
                ShapePaintPathKind::World => path_transform,
                ShapePaintPathKind::Local | ShapePaintPathKind::LocalClockwise => {
                    if shape_world == path_transform
                        && mat2d_has_visible_skew_or_rotation(shape_world)
                    {
                        Mat2D::IDENTITY
                    } else if mat2d_has_visible_skew_or_rotation(shape_world)
                        || mat2d_has_visible_skew_or_rotation(path_transform)
                    {
                        inverse_shape_world.multiply_path_local_fused(path_transform)
                    } else if mat2d_has_any_skew_or_rotation(shape_world)
                        || mat2d_has_any_skew_or_rotation(path_transform)
                    {
                        inverse_shape_world.multiply_path_local_contracted(path_transform)
                    } else {
                        inverse_shape_world.multiply(path_transform)
                    }
                }
            };

            let mut path_commands = path_commands(
                &runtime_path,
                path_kind,
                transform,
                weighted_context.as_ref(),
            );
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
                let runtime_path = runtime_path_geometry(self, path);
                let weighted_context = self.runtime_weighted_path_context(&runtime_path, graph);
                let path_transform = if weighted_context.is_some() {
                    Mat2D::IDENTITY
                } else {
                    path_world
                };
                let mut path_commands = path_commands(
                    &runtime_path,
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
}

fn runtime_sorted_drawable_node(
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

fn runtime_interleave_clipping_proxy_drawables(
    sorted_drawables: impl IntoIterator<Item = SortedDrawableNode>,
    clipping_shapes: &[ClippingShapeNode],
) -> Vec<SortedDrawableNode> {
    let mut order = Vec::new();
    let mut clipping_stack = Vec::<usize>::new();

    for drawable in sorted_drawables {
        let drawable_clipping_shapes =
            runtime_drawable_clipping_shape_locals(&drawable, clipping_shapes);
        let removing_index = clipping_stack
            .iter()
            .position(|clipping_shape_local| {
                !drawable_clipping_shapes.contains(clipping_shape_local)
            })
            .unwrap_or(clipping_stack.len());

        if removing_index < clipping_stack.len() {
            for clipping_shape_local in clipping_stack[removing_index..].iter().rev() {
                order.push(runtime_clipping_proxy_node(
                    clipping_shapes,
                    *clipping_shape_local,
                    DrawableOrderKind::ClipEndProxy,
                ));
            }
            clipping_stack.truncate(removing_index);
        }

        for clipping_shape_local in drawable_clipping_shapes {
            if !clipping_stack.contains(&clipping_shape_local) {
                order.push(runtime_clipping_proxy_node(
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
        order.push(runtime_clipping_proxy_node(
            clipping_shapes,
            clipping_shape_local,
            DrawableOrderKind::ClipEndProxy,
        ));
    }

    order
}

fn runtime_drawable_clipping_shape_locals(
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

fn runtime_clipping_proxy_node(
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

fn runtime_apply_save_operation_elision(
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
            && (runtime_sorted_drawable_will_clip(&sorted_drawables[index], &clipping_visibility)
                || prev_applied_save);
    }

    debug_assert!(applied_clipping_save_operations.is_empty());
}

fn runtime_sorted_drawable_will_clip(
    drawable: &SortedDrawableNode,
    clipping_visibility: &BTreeMap<usize, bool>,
) -> bool {
    drawable
        .clipping_shape_local
        .and_then(|local_id| clipping_visibility.get(&local_id))
        .copied()
        .unwrap_or(false)
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
    pub global_id: Option<u32>,
    pub type_name: &'static str,
    pub referenced_artboard_global: Option<u32>,
    pub clipping_shape_local: Option<usize>,
    pub needs_save_operation: bool,
    pub shape_paints: Vec<RuntimeShapePaintCommand>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct RuntimeLayoutBounds {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct RuntimeLayoutCorners {
    top_left: f32,
    top_right: f32,
    bottom_right: f32,
    bottom_left: f32,
}

impl RuntimeLayoutCorners {
    fn is_square(self) -> bool {
        self.top_left.abs() <= f32::EPSILON
            && self.top_right.abs() <= f32::EPSILON
            && self.bottom_right.abs() <= f32::EPSILON
            && self.bottom_left.abs() <= f32::EPSILON
    }
}

fn runtime_layout_rect_path_commands(
    bounds: RuntimeLayoutBounds,
    corners: RuntimeLayoutCorners,
) -> Vec<RuntimePathCommand> {
    let width_is_zero = bounds.width.abs() <= f32::EPSILON;
    let height_is_zero = bounds.height.abs() <= f32::EPSILON;
    if height_is_zero {
        return vec![
            RuntimePathCommand::Move { x: 0.0, y: 0.0 },
            RuntimePathCommand::Line {
                x: bounds.width,
                y: 0.0,
            },
            RuntimePathCommand::Line { x: 0.0, y: 0.0 },
            RuntimePathCommand::Close,
        ];
    }
    if width_is_zero {
        return vec![
            RuntimePathCommand::Move { x: 0.0, y: 0.0 },
            RuntimePathCommand::Line {
                x: 0.0,
                y: bounds.height,
            },
            RuntimePathCommand::Line { x: 0.0, y: 0.0 },
            RuntimePathCommand::Close,
        ];
    }
    if !corners.is_square() {
        return runtime_rounded_layout_rect_path_commands(bounds, corners);
    }
    vec![
        RuntimePathCommand::Move { x: 0.0, y: 0.0 },
        RuntimePathCommand::Line {
            x: bounds.width,
            y: 0.0,
        },
        RuntimePathCommand::Line {
            x: bounds.width,
            y: bounds.height,
        },
        RuntimePathCommand::Line {
            x: 0.0,
            y: bounds.height,
        },
        RuntimePathCommand::Line { x: 0.0, y: 0.0 },
        RuntimePathCommand::Close,
    ]
}

fn runtime_rounded_layout_rect_path_commands(
    bounds: RuntimeLayoutBounds,
    corners: RuntimeLayoutCorners,
) -> Vec<RuntimePathCommand> {
    let virtual_path = PathGeometryNode {
        local_id: 0,
        global_id: 0,
        type_name: "PointsPath",
        is_closed: true,
        is_hole: false,
        is_clockwise: true,
        parametric: None,
        vertices: vec![
            virtual_straight_vertex(0.0, 0.0, corners.top_left),
            virtual_straight_vertex(bounds.width, 0.0, corners.top_right),
            virtual_straight_vertex(bounds.width, bounds.height, corners.bottom_right),
            virtual_straight_vertex(0.0, bounds.height, corners.bottom_left),
        ],
    };
    points_path_commands(
        &virtual_path,
        ShapePaintPathKind::Local,
        Mat2D::IDENTITY,
        None,
    )
}

fn runtime_layout_hug_size(
    child_sizes: &[(f32, f32)],
    gap: f32,
    is_row: bool,
    width_axis: bool,
) -> f32 {
    if child_sizes.is_empty() {
        return 0.0;
    }
    let gap_total = gap * child_sizes.len().saturating_sub(1) as f32;
    match (is_row, width_axis) {
        (true, true) => child_sizes.iter().map(|(width, _)| *width).sum::<f32>() + gap_total,
        (true, false) => child_sizes
            .iter()
            .map(|(_, height)| *height)
            .fold(0.0, f32::max),
        (false, true) => child_sizes
            .iter()
            .map(|(width, _)| *width)
            .fold(0.0, f32::max),
        (false, false) => child_sizes.iter().map(|(_, height)| *height).sum::<f32>() + gap_total,
    }
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
    pub shape_world_override: Option<Mat2D>,
    pub uses_temporary_paint: bool,
    pub allocates_text_paint_pool: bool,
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
pub struct RuntimeRenderPaintCache {
    paints: BTreeMap<u32, Box<dyn RenderPaint>>,
    nested_artboards: BTreeMap<u32, RuntimeRenderPaintCache>,
}

impl RuntimeRenderPaintCache {
    pub fn root_paints_mut(&mut self) -> &mut BTreeMap<u32, Box<dyn RenderPaint>> {
        &mut self.paints
    }
}

#[derive(Default)]
pub struct RuntimeRenderPathCache {
    artboard_clip: Option<Box<dyn RenderPath>>,
    background_paths: BTreeMap<usize, Box<dyn RenderPath>>,
    clip_paths: BTreeMap<usize, Box<dyn RenderPath>>,
    layout_clip_paths: BTreeMap<usize, Box<dyn RenderPath>>,
    draw_paths: BTreeMap<RuntimeDrawPathCacheKey, Box<dyn RenderPath>>,
    gradient_shaders: BTreeMap<u32, RuntimeGradientShaderCacheEntry>,
    nested_artboards: BTreeMap<u32, RuntimeRenderPathCache>,
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

    fn layout_clip_path(
        &mut self,
        local_id: usize,
        factory: &mut dyn RenderFactory,
        commands: &[RuntimePathCommand],
        fill_rule: RenderFillRule,
    ) -> &mut Box<dyn RenderPath> {
        let path = match self.layout_clip_paths.entry(local_id) {
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

pub fn preallocate_render_paints_for_artboard_tree(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    factory: &mut dyn RenderFactory,
) -> BTreeMap<u32, Box<dyn RenderPaint>> {
    preallocate_render_paint_cache_for_artboard_tree(runtime, graph, artboards, factory).paints
}

pub fn preallocate_render_paint_cache_for_artboard_tree(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    factory: &mut dyn RenderFactory,
) -> RuntimeRenderPaintCache {
    let _source_artboard_paints = preallocate_render_paint_batch(runtime, factory);
    preallocate_artboard_render_paint_tree_batch(runtime, graph, artboards, factory)
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

fn preallocate_artboard_render_paint_tree_batch(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    factory: &mut dyn RenderFactory,
) -> RuntimeRenderPaintCache {
    let mut cache = RuntimeRenderPaintCache::default();
    let mut visiting = BTreeSet::new();
    preallocate_artboard_render_paint_tree_batch_into(
        runtime,
        graph,
        artboards,
        factory,
        &mut cache,
        &mut visiting,
    );
    cache
}

fn preallocate_artboard_render_paint_tree_batch_into(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    factory: &mut dyn RenderFactory,
    cache: &mut RuntimeRenderPaintCache,
    visiting: &mut BTreeSet<u32>,
) {
    if !visiting.insert(graph.global_id) {
        return;
    }

    let mut allocated_for_instance = BTreeSet::new();
    let paint_by_mutator = graph
        .shape_paint_containers
        .iter()
        .flat_map(|container| container.paints.iter())
        .map(|paint| (paint.mutator_global, paint.global_id))
        .collect::<BTreeMap<_, _>>();

    for local_object in &graph.local_objects {
        if let Some(child_graph) =
            referenced_artboard_graph_for_local_object(runtime, artboards, local_object.global_id)
        {
            let cache_key = nested_render_cache_key(
                Some(local_object.global_id),
                Some(local_object.local_id),
                child_graph.global_id,
            );
            let child_cache = cache.nested_artboards.entry(cache_key).or_default();
            preallocate_artboard_render_paint_tree_batch_into(
                runtime,
                child_graph,
                artboards,
                factory,
                child_cache,
                visiting,
            );
        }
    }

    for local_object in &graph.local_objects {
        if let Some(paint_global_id) = paint_by_mutator.get(&Some(local_object.global_id)) {
            preallocate_render_paint_for_instance(
                *paint_global_id,
                factory,
                &mut cache.paints,
                &mut allocated_for_instance,
            );
        }
    }

    for local_object in &graph.local_objects {
        let Some(object) = runtime.object(local_object.global_id as usize) else {
            continue;
        };
        if matches!(object.type_name, "Fill" | "Stroke") {
            preallocate_render_paint_for_instance(
                object.id,
                factory,
                &mut cache.paints,
                &mut allocated_for_instance,
            );
        }
    }

    visiting.remove(&graph.global_id);
}

fn referenced_artboard_graph_for_local_object<'a>(
    runtime: &RuntimeFile,
    artboards: &'a [ArtboardGraph],
    global_id: u32,
) -> Option<&'a ArtboardGraph> {
    let object = runtime.object(global_id as usize)?;
    let referenced = runtime.resolved_artboard_for_referencer_object(object)?;
    artboards
        .iter()
        .find(|graph| graph.global_id == referenced.id)
}

fn nested_render_cache_key(
    global_id: Option<u32>,
    local_id: Option<usize>,
    referenced_artboard_global: u32,
) -> u32 {
    global_id
        .or_else(|| local_id.and_then(|local_id| u32::try_from(local_id).ok()))
        .unwrap_or(referenced_artboard_global)
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

fn preallocate_render_paint_for_instance(
    global_id: u32,
    factory: &mut dyn RenderFactory,
    paints: &mut BTreeMap<u32, Box<dyn RenderPaint>>,
    allocated_for_instance: &mut BTreeSet<u32>,
) {
    if !allocated_for_instance.insert(global_id) {
        return;
    }

    let render_paint = factory.make_render_paint();
    if let Entry::Vacant(entry) = paints.entry(global_id) {
        entry.insert(render_paint);
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
        let Some(runtime_paint) =
            runtime_background_shape_paint_command(instance, paint, commands.clone())
        else {
            continue;
        };
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
) -> Option<RuntimeShapePaintCommand> {
    let render_opacity = instance
        .component(0)
        .map(|component| component.transform.render_opacity)
        .unwrap_or(1.0);
    let paint_state = runtime_shape_paint_state(instance, paint, render_opacity);
    if !runtime_shape_paint_state_is_visible(&paint_state) {
        return None;
    }
    Some(RuntimeShapePaintCommand {
        paint_local: paint.local_id,
        mutator_local: paint.mutator_local,
        paint_type: runtime_shape_paint_kind(paint.paint_type),
        path_kind: RuntimeShapePaintPathKind::Local,
        blend_mode_value: paint.blend_mode_value,
        render_blend_mode_value: 3,
        paint_state,
        feather_state: None,
        path_commands,
        effect_path_commands: Vec::new(),
        has_effect_path: false,
        needs_save_operation: true,
        shape_world_override: None,
        uses_temporary_paint: false,
        allocates_text_paint_pool: false,
    })
}

fn runtime_prepare_gradient_paint_command(
    instance: &ArtboardInstance,
    opacity_local: usize,
    container: &ShapePaintContainerNode,
    paint: &ShapePaintNode,
) -> RuntimeShapePaintCommand {
    let render_opacity = instance
        .component(opacity_local)
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
        shape_world_override: None,
        uses_temporary_paint: false,
        allocates_text_paint_pool: false,
    }
}

fn runtime_draw_command(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    graph: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    local_to_global: &BTreeMap<usize, u32>,
    command: RuntimeDrawCommand,
    factory: &mut dyn RenderFactory,
    renderer: &mut dyn Renderer,
    paint_by_global: &mut BTreeMap<u32, Box<dyn RenderPaint>>,
    path_cache: &mut RuntimeRenderPathCache,
    nested_paint_caches: Option<&mut BTreeMap<u32, RuntimeRenderPaintCache>>,
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

    if command.type_name == "DrawableProxy"
        && let Some(layout_local) = command.local_id
        && instance.runtime_layout_component_clip_enabled(layout_local)
    {
        renderer.save();
        let path_commands =
            instance.runtime_layout_component_clip_path_commands(layout_local, graph);
        if !path_commands.is_empty() {
            let path = path_cache.layout_clip_path(
                layout_local,
                factory,
                &path_commands,
                RenderFillRule::Clockwise,
            );
            renderer.clip_path(path.as_ref());
        }
    }

    if command.type_name == "LayoutComponent"
        && let Some(layout_local) = command.local_id
        && instance.runtime_layout_component_clip_enabled(layout_local)
    {
        renderer.restore();
        return Ok(());
    }

    if sorted_drawable_is_nested_artboard(command.type_name) {
        return runtime_draw_nested_artboard(
            runtime,
            instance,
            artboards,
            command,
            factory,
            renderer,
            paint_by_global,
            path_cache,
            nested_paint_caches,
        );
    }

    let shape_world = command
        .local_id
        .and_then(|local_id| instance.component(local_id))
        .map(|component| component.transform.world_transform)
        .unwrap_or(Mat2D::IDENTITY);

    let draws_text = command.type_name == "Text";
    let text_shape_paints = if draws_text {
        runtime_text_shape_paint_commands(runtime, instance, graph, &command)?
    } else {
        Vec::new()
    };
    let shape_paints = if draws_text {
        text_shape_paints.as_slice()
    } else {
        command.shape_paints.as_slice()
    };
    if draws_text && command.needs_save_operation {
        renderer.save();
    }

    let mut text_temporary_paints = Vec::new();
    if draws_text {
        text_temporary_paints.reserve(
            shape_paints
                .iter()
                .filter(|paint| paint.uses_temporary_paint)
                .count(),
        );
        for _ in shape_paints
            .iter()
            .filter(|paint| paint.uses_temporary_paint)
        {
            text_temporary_paints.push(factory.make_render_paint());
        }
    }
    let mut text_temporary_paint_index = 0;
    let mut draw_path_slots = Vec::<Vec<RuntimePathCommand>>::new();
    for paint in shape_paints {
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
        let temporary_paint_index = if draws_text && paint.uses_temporary_paint {
            let render_paint = text_temporary_paints
                .get_mut(text_temporary_paint_index)
                .context("missing temporary text render paint")?;
            let index = text_temporary_paint_index;
            text_temporary_paint_index += 1;
            runtime_configure_paint(render_paint.as_mut(), object, paint, None)?;
            Some(index)
        } else {
            runtime_configure_paint(
                paint_by_global
                    .get_mut(&global_id)
                    .with_context(|| format!("missing render paint for global {global_id}"))?
                    .as_mut(),
                object,
                paint,
                None,
            )?;
            None
        };
        let path_index = runtime_cached_path_slot_index(&mut draw_path_slots, path_commands);
        let mut saved = !paint.needs_save_operation;
        let paint_shape_world = paint.shape_world_override.unwrap_or(shape_world);
        if matches!(
            paint.path_kind,
            RuntimeShapePaintPathKind::Local | RuntimeShapePaintPathKind::LocalClockwise
        ) {
            if !saved {
                saved = true;
                renderer.save();
            }
            renderer.transform(runtime_render_mat(paint_shape_world));
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
        if !draws_text {
            runtime_configure_fill_rule(path.as_mut(), object);
        }
        let render_paint = if let Some(index) = temporary_paint_index {
            text_temporary_paints[index].as_ref()
        } else {
            paint_by_global
                .get(&global_id)
                .with_context(|| format!("missing render paint for global {global_id}"))?
                .as_ref()
        };
        renderer.draw_path(path.as_ref(), render_paint);
        if saved && paint.needs_save_operation {
            renderer.restore();
        }
        if draws_text && paint.allocates_text_paint_pool {
            let _ = factory.make_render_paint();
        }
    }
    if draws_text && command.needs_save_operation {
        renderer.restore();
    }

    Ok(())
}

fn runtime_draw_nested_artboard(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    artboards: &[ArtboardGraph],
    command: RuntimeDrawCommand,
    factory: &mut dyn RenderFactory,
    renderer: &mut dyn Renderer,
    paint_by_global: &mut BTreeMap<u32, Box<dyn RenderPaint>>,
    path_cache: &mut RuntimeRenderPathCache,
    nested_paint_caches: Option<&mut BTreeMap<u32, RuntimeRenderPaintCache>>,
) -> Result<()> {
    if let Some(local_id) = command.local_id
        && let Some(artboard_id_key) = property_key_for_name("NestedArtboard", "artboardId")
        && instance.uint_property(local_id, artboard_id_key) == Some(u64::from(u32::MAX))
    {
        return Ok(());
    }

    let referenced_artboard_global = command
        .referenced_artboard_global
        .context("nested artboard missing referenced artboard")?;
    let child_graph = artboards
        .iter()
        .find(|graph| graph.global_id == referenced_artboard_global)
        .with_context(|| {
            format!("missing nested artboard graph for global {referenced_artboard_global}")
        })?;
    let host_component = command
        .local_id
        .and_then(|local_id| instance.component(local_id))
        .context("nested artboard host component is missing")?;
    let host_world = host_component.transform.world_transform;
    let host_opacity = host_component.transform.render_opacity;

    if command.needs_save_operation {
        renderer.save();
    }
    renderer.transform(runtime_render_mat(host_world));

    let cache_key = nested_render_cache_key(
        command.global_id,
        command.local_id,
        referenced_artboard_global,
    );
    let child_cache = path_cache.nested_artboards.entry(cache_key).or_default();

    if let Some(child) = command
        .local_id
        .and_then(|local_id| instance.nested_artboards.get(&local_id))
        .map(|nested| nested.child.as_ref())
    {
        if let Some(caches) = nested_paint_caches {
            let child_paint_cache = caches.entry(cache_key).or_default();
            child.prepare_static_artboard_tree_paints_internal(
                runtime,
                child_graph,
                artboards,
                factory,
                &mut child_paint_cache.paints,
                Some(&mut child_paint_cache.nested_artboards),
                child_cache,
            )?;
            child.draw_prepared_static_artboard_internal_with_path_cache(
                runtime,
                child_graph,
                artboards,
                factory,
                renderer,
                &mut child_paint_cache.paints,
                child_cache,
                Some(&mut child_paint_cache.nested_artboards),
                false,
            )?;
        } else {
            child.prepare_static_artboard_tree_paints_internal(
                runtime,
                child_graph,
                artboards,
                factory,
                paint_by_global,
                None,
                child_cache,
            )?;
            child.draw_prepared_static_artboard_internal_with_path_cache(
                runtime,
                child_graph,
                artboards,
                factory,
                renderer,
                paint_by_global,
                child_cache,
                None,
                false,
            )?;
        }

        if command.needs_save_operation {
            renderer.restore();
        }
        return Ok(());
    }

    let mut child = ArtboardInstance::from_graph(runtime, child_graph)?;
    if let Some(opacity_key) = property_key_for_name("Artboard", "opacity") {
        child.set_double_property(0, opacity_key, host_opacity);
    }
    child.update_pass();
    if let Some(caches) = nested_paint_caches {
        let child_paint_cache = caches.entry(cache_key).or_default();
        child.prepare_static_artboard_tree_paints_internal(
            runtime,
            child_graph,
            artboards,
            factory,
            &mut child_paint_cache.paints,
            Some(&mut child_paint_cache.nested_artboards),
            child_cache,
        )?;
        child.draw_prepared_static_artboard_internal_with_path_cache(
            runtime,
            child_graph,
            artboards,
            factory,
            renderer,
            &mut child_paint_cache.paints,
            child_cache,
            Some(&mut child_paint_cache.nested_artboards),
            false,
        )?;
    } else {
        child.prepare_static_artboard_tree_paints_internal(
            runtime,
            child_graph,
            artboards,
            factory,
            paint_by_global,
            None,
            child_cache,
        )?;
        child.draw_prepared_static_artboard_internal_with_path_cache(
            runtime,
            child_graph,
            artboards,
            factory,
            renderer,
            paint_by_global,
            child_cache,
            None,
            false,
        )?;
    }

    if command.needs_save_operation {
        renderer.restore();
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

fn mat2d_has_any_skew_or_rotation(mat: Mat2D) -> bool {
    mat.0[1] != 0.0 || mat.0[2] != 0.0
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

pub(crate) fn runtime_shape_paint_command(
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
        shape_world_override: None,
        uses_temporary_paint: false,
        allocates_text_paint_pool: false,
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

fn runtime_shape_paint_state_is_visible(state: &Option<RuntimeShapePaintState>) -> bool {
    // C++ skips authored-transparent Backboard/background draws, while
    // render-opacity-transparent backgrounds can still appear in the stream.
    !matches!(
        state,
        Some(RuntimeShapePaintState::SolidColor { color, .. }) if (color >> 24) == 0
    )
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
        } => {
            let (start_x, start_y, end_x, end_y) = runtime_gradient_endpoints(
                artboard,
                paint.mutator_local,
                "LinearGradient",
                start_x,
                start_y,
                end_x,
                end_y,
            );
            Some(RuntimeShapePaintState::LinearGradient {
                start_x,
                start_y,
                end_x,
                end_y,
                opacity,
                render_opacity,
                stops: runtime_gradient_stops(stops, opacity * render_opacity),
            })
        }
        ShapePaintStateNode::RadialGradient {
            start_x,
            start_y,
            end_x,
            end_y,
            opacity,
            stops,
        } => {
            let (start_x, start_y, end_x, end_y) = runtime_gradient_endpoints(
                artboard,
                paint.mutator_local,
                "RadialGradient",
                start_x,
                start_y,
                end_x,
                end_y,
            );
            Some(RuntimeShapePaintState::RadialGradient {
                start_x,
                start_y,
                end_x,
                end_y,
                opacity,
                render_opacity,
                stops: runtime_gradient_stops(stops, opacity * render_opacity),
            })
        }
    }
}

fn shape_paint_container_opacity_local(graph: &ArtboardGraph, container_local: usize) -> usize {
    let mut current = Some(container_local);
    while let Some(local_id) = current {
        let Some(component) = graph
            .components
            .iter()
            .find(|component| component.local_id == local_id)
        else {
            break;
        };
        if component.capabilities.transform || component.capabilities.artboard {
            return local_id;
        }
        current = component.parent_local;
    }
    container_local
}

fn runtime_gradient_endpoints(
    artboard: &ArtboardInstance,
    mutator_local: Option<usize>,
    type_name: &str,
    start_x: f32,
    start_y: f32,
    end_x: f32,
    end_y: f32,
) -> (f32, f32, f32, f32) {
    let Some(mutator_local) = mutator_local else {
        return (start_x, start_y, end_x, end_y);
    };
    (
        runtime_gradient_double_property(artboard, mutator_local, type_name, "startX", start_x),
        runtime_gradient_double_property(artboard, mutator_local, type_name, "startY", start_y),
        runtime_gradient_double_property(artboard, mutator_local, type_name, "endX", end_x),
        runtime_gradient_double_property(artboard, mutator_local, type_name, "endY", end_y),
    )
}

fn runtime_gradient_double_property(
    artboard: &ArtboardInstance,
    local_id: usize,
    type_name: &str,
    property_name: &str,
    fallback: f32,
) -> f32 {
    property_key_for_name(type_name, property_name)
        .and_then(|property_key| artboard.double_property(local_id, property_key))
        .unwrap_or(fallback)
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
pub(crate) struct RuntimePathMeasure {
    contours: Vec<TrimContour>,
    length: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct RuntimePathSample {
    pub(crate) pos: (f32, f32),
    pub(crate) tan: (f32, f32),
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

    fn position_tangent_at_distance(&self, distance: f32) -> ((f32, f32), (f32, f32)) {
        let distance = distance.clamp(0.0, self.length);
        let index = self.find_segment(distance);
        let segment = &self.segments[index];

        match segment.kind {
            TrimSegmentKind::Line { from, to } => {
                let mut previous_distance = 0.0;
                if index > 0 {
                    previous_distance = self.segments[index - 1].distance;
                }
                let denominator = segment.distance - previous_distance;
                let rel_d = if denominator == 0.0 {
                    0.0
                } else {
                    (distance - previous_distance) / denominator
                };
                let (tan, _) = normalize_vector((to.0 - from.0, to.1 - from.1));
                (lerp_point(from, to, rel_d), tan)
            }
            TrimSegmentKind::Cubic { p0, p1, p2, p3 } => {
                cubic_position_tangent([p0, p1, p2, p3], self.compute_t(index, distance))
            }
        }
    }
}

impl RuntimePathMeasure {
    pub(crate) fn from_commands(commands: &[RuntimePathCommand]) -> Self {
        let contours = TrimContour::from_commands(commands);
        let length = contours.iter().map(|contour| contour.length).sum();
        Self { contours, length }
    }

    pub(crate) fn at_percentage(&self, percentage_distance: f32) -> RuntimePathSample {
        let mut in_range_percentage = percentage_distance % 1.0;
        if in_range_percentage < 0.0 {
            in_range_percentage += 1.0;
        }
        if percentage_distance != 0.0 && in_range_percentage == 0.0 {
            in_range_percentage = 1.0;
        }
        self.at_distance(self.length * in_range_percentage)
    }

    fn at_distance(&self, distance: f32) -> RuntimePathSample {
        let mut current_distance = distance;
        for contour in &self.contours {
            let contour_length = contour.length;
            if current_distance - contour_length <= 0.0 {
                let (pos, tan) = contour.position_tangent_at_distance(current_distance);
                return RuntimePathSample { pos, tan };
            }
            current_distance -= contour_length;
        }
        RuntimePathSample::default()
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

fn cubic_position_tangent(points: [(f32, f32); 4], t: f32) -> ((f32, f32), (f32, f32)) {
    if t == 0.0 {
        let tangent_to = if points[0] != points[1] {
            points[1]
        } else if points[1] != points[2] {
            points[2]
        } else {
            points[3]
        };
        return (
            points[0],
            (tangent_to.0 - points[0].0, tangent_to.1 - points[0].1),
        );
    }
    if t == 1.0 {
        let tangent_from = if points[3] != points[2] {
            points[2]
        } else if points[2] != points[1] {
            points[1]
        } else {
            points[0]
        };
        return (
            points[3],
            (points[3].0 - tangent_from.0, points[3].1 - tangent_from.1),
        );
    }

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
    let (tan, _) = normalize_vector((
        (3.0 * a.0 * t + 2.0 * b.0) * t + c.0,
        (3.0 * a.1 * t + 2.0 * b.1) * t + c.1,
    ));
    (eval_cubic(points, t), tan)
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

pub(crate) fn color_lerp(from: u32, to: u32, mix: f32) -> u32 {
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

pub(crate) fn runtime_path_geometry_commands(
    artboard: &ArtboardInstance,
    path: &PathGeometryNode,
    transform: Mat2D,
) -> Vec<RuntimePathCommand> {
    let path = runtime_path_geometry(artboard, path);
    let mut commands = path_commands(&path, ShapePaintPathKind::World, transform, None);
    prune_empty_path_segments(&mut commands);
    commands
}

fn runtime_path_geometry(artboard: &ArtboardInstance, path: &PathGeometryNode) -> PathGeometryNode {
    let mut path = path.clone();
    path.is_closed = runtime_path_bool_property(
        artboard,
        path.local_id,
        path.type_name,
        "isClosed",
        path.is_closed,
    );
    path.is_hole = runtime_path_bool_property(
        artboard,
        path.local_id,
        path.type_name,
        "isHole",
        path.is_hole,
    );
    path.is_clockwise = runtime_path_bool_property(
        artboard,
        path.local_id,
        path.type_name,
        "isClockwise",
        path.is_clockwise,
    );
    path.parametric = runtime_parametric_path(artboard, &path);

    for vertex in &mut path.vertices {
        vertex.x = runtime_path_double_property(
            artboard,
            vertex.local_id,
            vertex.type_name,
            "x",
            vertex.x,
        );
        vertex.y = runtime_path_double_property(
            artboard,
            vertex.local_id,
            vertex.type_name,
            "y",
            vertex.y,
        );
        vertex.radius = runtime_path_double_property(
            artboard,
            vertex.local_id,
            vertex.type_name,
            "radius",
            vertex.radius,
        );
        vertex.rotation = runtime_path_double_property(
            artboard,
            vertex.local_id,
            vertex.type_name,
            "rotation",
            vertex.rotation,
        );
        vertex.distance = runtime_path_double_property(
            artboard,
            vertex.local_id,
            vertex.type_name,
            "distance",
            vertex.distance,
        );
        vertex.in_rotation = runtime_path_double_property(
            artboard,
            vertex.local_id,
            vertex.type_name,
            "inRotation",
            vertex.in_rotation,
        );
        vertex.in_distance = runtime_path_double_property(
            artboard,
            vertex.local_id,
            vertex.type_name,
            "inDistance",
            vertex.in_distance,
        );
        vertex.out_rotation = runtime_path_double_property(
            artboard,
            vertex.local_id,
            vertex.type_name,
            "outRotation",
            vertex.out_rotation,
        );
        vertex.out_distance = runtime_path_double_property(
            artboard,
            vertex.local_id,
            vertex.type_name,
            "outDistance",
            vertex.out_distance,
        );
    }

    path
}

fn runtime_parametric_path(
    artboard: &ArtboardInstance,
    path: &PathGeometryNode,
) -> Option<ParametricPathNode> {
    match path.parametric.as_ref()? {
        ParametricPathNode::Ellipse {
            width,
            height,
            origin_x,
            origin_y,
        } => Some(ParametricPathNode::Ellipse {
            width: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "width",
                *width,
            ),
            height: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "height",
                *height,
            ),
            origin_x: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "originX",
                *origin_x,
            ),
            origin_y: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "originY",
                *origin_y,
            ),
        }),
        ParametricPathNode::Polygon {
            width,
            height,
            origin_x,
            origin_y,
            points,
            corner_radius,
        } => Some(ParametricPathNode::Polygon {
            width: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "width",
                *width,
            ),
            height: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "height",
                *height,
            ),
            origin_x: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "originX",
                *origin_x,
            ),
            origin_y: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "originY",
                *origin_y,
            ),
            points: runtime_path_uint_property(
                artboard,
                path.local_id,
                path.type_name,
                "points",
                *points as u64,
            ) as u32,
            corner_radius: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "cornerRadius",
                *corner_radius,
            ),
        }),
        ParametricPathNode::Star {
            width,
            height,
            origin_x,
            origin_y,
            points,
            corner_radius,
            inner_radius,
        } => Some(ParametricPathNode::Star {
            width: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "width",
                *width,
            ),
            height: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "height",
                *height,
            ),
            origin_x: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "originX",
                *origin_x,
            ),
            origin_y: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "originY",
                *origin_y,
            ),
            points: runtime_path_uint_property(
                artboard,
                path.local_id,
                path.type_name,
                "points",
                *points as u64,
            ) as u32,
            corner_radius: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "cornerRadius",
                *corner_radius,
            ),
            inner_radius: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "innerRadius",
                *inner_radius,
            ),
        }),
        ParametricPathNode::Rectangle {
            width,
            height,
            origin_x,
            origin_y,
            link_corner_radius,
            corner_radius_tl,
            corner_radius_tr,
            corner_radius_bl,
            corner_radius_br,
        } => Some(ParametricPathNode::Rectangle {
            width: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "width",
                *width,
            ),
            height: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "height",
                *height,
            ),
            origin_x: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "originX",
                *origin_x,
            ),
            origin_y: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "originY",
                *origin_y,
            ),
            link_corner_radius: runtime_path_bool_property(
                artboard,
                path.local_id,
                path.type_name,
                "linkCornerRadius",
                *link_corner_radius,
            ),
            corner_radius_tl: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "cornerRadiusTL",
                *corner_radius_tl,
            ),
            corner_radius_tr: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "cornerRadiusTR",
                *corner_radius_tr,
            ),
            corner_radius_bl: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "cornerRadiusBL",
                *corner_radius_bl,
            ),
            corner_radius_br: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "cornerRadiusBR",
                *corner_radius_br,
            ),
        }),
        ParametricPathNode::Triangle {
            width,
            height,
            origin_x,
            origin_y,
        } => Some(ParametricPathNode::Triangle {
            width: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "width",
                *width,
            ),
            height: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "height",
                *height,
            ),
            origin_x: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "originX",
                *origin_x,
            ),
            origin_y: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "originY",
                *origin_y,
            ),
        }),
    }
}

fn runtime_path_double_property(
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

fn runtime_path_bool_property(
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

fn runtime_path_uint_property(
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
