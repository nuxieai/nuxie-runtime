use crate::mechanical_port::source::{
    artboard::Artboard,
    bones::skinnable::SkinnableBehavior,
    component::{Component, ComponentDirt, ComponentOccurrenceHandle, has_dirt},
    core::{Core, CoreHandle},
    core_context::CoreContext,
    drawable::DrawableFlag,
    generated::shapes::shape_base::ShapeBase,
    hit_info::HitInfo,
    hittest_command_path::HitTestCommandPath,
    layout::{
        LayoutDirection, LayoutMeasureMode, LayoutScaleType, layout_participant::LayoutParticipant,
    },
    math::{
        aabb::Aabb, contour_measure::ContourMeasureIter, mat2d::Mat2D, raw_path::RawPath,
        vec2d::Vec2D,
    },
    renderer::{RenderPath, Renderer},
    shapes::{
        deformer::RenderPathDeformer,
        paint::{shape_paint::ShapePaintPathKind, shape_paint_path::ShapePaintPath},
        parametric_path::ParametricPath,
        path::Path,
        path_composer::RuntimePathComposerHandle,
        path_flags::PathFlags,
        shape_paint_container::ShapePaintContainer,
    },
    status_code::StatusCode,
};

pub struct Shape {
    pub base: ShapeBase,
    pub paint_container: ShapePaintContainer,
    path_composer: RuntimePathComposerHandle,
    paths: Vec<CoreHandle>,
    world_bounds: Aabb,
    world_length: f32,
    want_difference_path: bool,
    deformer: Option<CoreHandle>,
}

impl Shape {
    fn get_artboard(&self) -> &Artboard {
        self.base.artboard()
    }
    pub fn shape_world_transform(&self) -> &Mat2D {
        self.base.world_transform()
    }
    pub fn new() -> Self {
        Self {
            base: ShapeBase::default(),
            paint_container: ShapePaintContainer::default(),
            path_composer: RuntimePathComposerHandle::new(),
            paths: Vec::new(),
            world_bounds: Aabb::default(),
            world_length: -1.0,
            want_difference_path: false,
            deformer: None,
        }
    }

    pub fn add_path(&mut self, path: CoreHandle) {
        assert!(!self.paths.contains(&path));
        self.paths.push(path);
        self.invalidate_intrinsic_bounds();
    }
    pub fn paths(&self) -> Vec<CoreHandle> {
        self.paths.clone()
    }
    pub fn add_flags(&mut self, flags: PathFlags) {
        self.paint_container.add_path_flags(flags);
    }
    pub fn is_flagged(&self, flags: PathFlags) -> bool {
        !(self.paint_container.path_flags() & flags).is_empty()
    }
    pub fn want_difference_path(&self) -> bool {
        self.want_difference_path
    }
    pub fn deformer(&self) -> Option<CoreHandle> {
        self.deformer.clone()
    }

    pub fn can_defer_path_update(&self) -> bool {
        let can_defer = self.base.render_opacity() == 0.0
            && !self.is_flagged(PathFlags::CLIPPING | PathFlags::NEVER_DEFER_UPDATE);
        if can_defer
            && self.base.dependents().iter().any(|d| {
                d.authored()
                    .and_then(|handle| {
                        handle.with(|object| {
                            object.as_points_path().is_some_and(|p| p.skin().is_some())
                        })
                    })
                    .unwrap_or(false)
            })
        {
            return false;
        }
        can_defer
    }

    pub(crate) fn update_after_transform_super(&mut self, value: ComponentDirt) {
        if has_dirt(value, ComponentDirt::RENDER_OPACITY) {
            self.paint_container
                .propagate_opacity(self.base.render_opacity());
        }
    }
    pub fn collapse(&mut self, value: bool) -> bool {
        if !self.base.collapse(value) {
            return false;
        }
        self.collapse_after_super(value);
        true
    }

    pub(crate) fn collapse_after_super(&mut self, value: bool) {
        self.path_composer.clone().collapse_from_shape(self, value);
        self.invalidate_intrinsic_bounds();
    }

    pub fn length(&mut self) -> f32 {
        if self.world_length < 0.0 {
            let mut length = 0.0;
            for path in self.paths() {
                length += path
                    .with(|path| {
                        let path = path.as_path()?;
                        let dirty = path.base.has_dirt(
                            ComponentDirt::PATH
                                | ComponentDirt::WORLD_TRANSFORM
                                | ComponentDirt::N_SLICER,
                        );
                        let mut temporary = RawPath::default();
                        let base = if dirty {
                            path.build_path(&mut temporary);
                            &temporary
                        } else {
                            path.raw_path()
                        };
                        let source = base.transform(path.path_transform());
                        let mut length = 0.0;
                        let mut iter = ContourMeasureIter::new(&source);
                        while let Some(contour) = iter.next() {
                            length += contour.length();
                        }
                        Some(length)
                    })
                    .flatten()
                    .unwrap_or_default();
            }
            self.world_length = length;
        }
        self.world_length
    }

    pub fn set_length(&mut self, _value: f32) {}

    pub fn path_changed(&mut self) {
        self.path_composer
            .clone()
            .add_dirt_from_shape(self, ComponentDirt::PATH, true);
        self.world_length = -1.0;
        self.invalidate_intrinsic_bounds();
        for constraint in self.base.constraints().to_vec() {
            constraint
                .with_mut(|constraint| constraint.component_add_dirt(ComponentDirt::PATH, false));
        }
        self.paint_container.invalidate_stroke_effects();
    }

    pub fn add_to_render_path(&mut self, path: &mut RenderPath, transform: Mat2D) {
        let factory = self
            .base
            .with_artboard(Artboard::factory)
            .flatten()
            .expect("Shape renderer factory");
        if self.is_flagged(PathFlags::LOCAL) {
            let transform = transform * *self.base.world_transform();
            self.with_path_mut(ShapePaintPathKind::Local, |source| {
                path.add_render_path(
                    source.render_path(&factory),
                    nuxie_render_api::Mat2D(*transform.values()),
                );
            });
        } else {
            self.with_path_mut(ShapePaintPathKind::World, |source| {
                path.add_render_path(
                    source.render_path(&factory),
                    nuxie_render_api::Mat2D(*transform.values()),
                );
            });
        }
    }

    pub fn add_to_raw_path(&mut self, path: &mut RawPath, transform: Option<Mat2D>) {
        if self.is_flagged(PathFlags::LOCAL) {
            let matrix = transform
                .map(|v| v * *self.base.world_transform())
                .unwrap_or_else(|| *self.base.world_transform());
            self.with_path_mut(ShapePaintPathKind::Local, |source| {
                path.add_path(source.raw_path(), Some(&matrix));
            });
        } else {
            self.with_path_mut(ShapePaintPathKind::World, |source| {
                path.add_path(source.raw_path(), transform.as_ref());
            });
        }
    }

    pub fn draw(&mut self, renderer: &mut Renderer) {
        let needs_save =
            self.base.needs_save_operation() || self.paint_container.shape_paints().len() > 1;
        let transform = *self.base.world_transform();
        for paint in self.paint_container.shape_paints() {
            paint.with_mut(|object| {
                let Some(behavior) = object.as_shape_paint_behavior_mut() else {
                    return;
                };
                if !behavior.shape_paint().base.is_visible() {
                    return;
                }
                let kind = behavior.pick_path_kind();
                let fill_rule = behavior.fill_rule();
                self.with_path_mut(kind, |path| {
                    behavior.shape_paint_mut().draw_with_fill_rule(
                        renderer, path, transform, false, None, needs_save, fill_rule,
                    );
                });
            });
        }
    }

    pub fn hit_test_aabb(&mut self, position: Vec2D) -> bool {
        self.world_bounds().contains(position)
    }
    pub fn hit_test_hi_fi(&self, position: Vec2D, radius: f32) -> bool {
        let area = Aabb::new(
            position.x - radius,
            position.y - radius,
            position.x + radius,
            position.y + radius,
        )
        .round();
        let mut tester = HitTestCommandPath::new(area);
        for path in self.paths() {
            path.with(|path| {
                if let Some(path) = path.as_path()
                    && !path.base.is_collapsed()
                {
                    tester.set_xform(path.path_transform());
                    path.raw_path().add_to(&mut tester);
                }
            });
        }
        tester.was_hit()
    }

    pub fn hit_test<'a>(&'a self, hinfo: &HitInfo, xform: Mat2D) -> Option<&'a Core> {
        if self.base.render_opacity() == 0.0 {
            return None;
        }
        let shape_local = self.is_flagged(PathFlags::LOCAL | PathFlags::LOCAL_CLOCKWISE);
        for paint in self.paint_container.shape_paints().iter().rev() {
            if paint.is_translucent() || !paint.base.is_visible() {
                continue;
            }
            let paint_local = paint.is_flagged(PathFlags::LOCAL | PathFlags::LOCAL_CLOCKWISE);
            let matrix = if paint_local {
                xform * self.base.world_transform()
            } else {
                xform
            };
            let mut tester = HitTestCommandPath::new(hinfo.area());
            for path in self.paths() {
                path.with(|path| {
                    let Some(path) = path.as_path() else {
                        return;
                    };
                    tester.set_xform(if shape_local {
                        xform * path.path_transform()
                    } else {
                        matrix * path.path_transform()
                    });
                    path.raw_path().add_to(&mut tester);
                });
            }
            if tester.was_hit() {
                return Some(self.base.as_core());
            }
        }
        None
    }

    pub fn hit_test_point(
        &mut self,
        position: Vec2D,
        skip_on_unclipped: bool,
        primary: bool,
    ) -> bool {
        if !primary {
            return crate::mechanical_port::source::component::Component::hit_test_point(
                &self.base.base.base.base.base.base,
                &position,
                skip_on_unclipped,
                primary,
            );
        }
        self.hit_test_aabb(position)
            && crate::mechanical_port::source::component::Component::hit_test_point(
                &self.base.base.base.base.base.base,
                &position,
                skip_on_unclipped,
                primary,
            )
            && self.hit_test_hi_fi(position, 2.0)
    }

    pub fn build_dependencies(&mut self) {
        let helper = self.path_composer.occurrence();
        self.base.add_dependent(helper);
        let paths = self.paths();
        self.path_composer
            .with_mut(|helper| helper.build_path_dependencies(&paths));
        self.base.build_dependencies();
        let blend = self.base.blend_mode();
        for paint in self.paint_container.shape_paints() {
            paint.with_mut(|paint| {
                paint
                    .as_shape_paint_mut()
                    .map(|paint| paint.blend_mode(blend))
            });
        }
    }
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let Some(this) = self.base.handle() else {
            return StatusCode::MissingObject;
        };
        self.path_composer.bind_shape(this);
        self.path_composer
            .with_mut(|helper| helper.component.on_added_dirty_runtime(context))
    }
    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.on_added_clean(context);
        if code != StatusCode::Ok {
            return code;
        }
        self.deformer = None;
        let mut parent = self.base.parent_handle();
        while let Some(current) = parent {
            if current
                .with(|current| {
                    current
                        .as_component()
                        .and_then(render_path_deformer_from)
                        .is_some()
                })
                .unwrap_or(false)
            {
                self.deformer = Some(current);
                return StatusCode::Ok;
            }
            parent = current
                .with(|current| current.component_parent_handle())
                .flatten();
        }
        StatusCode::Ok
    }
    pub fn is_empty(&self) -> bool {
        self.paths().iter().all(|path| {
            path.with(|path| {
                path.as_path()
                    .is_none_or(|path| path.is_hidden() || path.base.is_collapsed())
            })
            .unwrap_or(true)
        })
    }
    pub fn will_draw(&self) -> bool {
        self.base.will_draw() && self.base.render_opacity() != 0.0
    }
    pub fn path_collapse_changed(&mut self) {
        let helper = self.path_composer.occurrence();
        self.path_composer
            .clone()
            .add_dirt_from_shape(self, ComponentDirt::PATH, false);
        for dependent in helper
            .with_component(Component::dependents_snapshot)
            .unwrap_or_default()
        {
            dependent.add_dirt(ComponentDirt::PATH, true);
        }
    }

    pub fn world_bounds(&mut self) -> Aabb {
        if self.base.drawable_flags() & DrawableFlag::WORLD_BOUNDS_CLEAN.bits() == 0 {
            self.base.set_drawable_flags(
                self.base.drawable_flags() | DrawableFlag::WORLD_BOUNDS_CLEAN.bits(),
            );
            self.world_bounds = self.compute_world_bounds(None);
        }
        self.world_bounds
    }
    pub fn mark_bounds_dirty(&mut self) {
        self.base.set_drawable_flags(
            self.base.drawable_flags() & !DrawableFlag::WORLD_BOUNDS_CLEAN.bits(),
        );
        self.world_length = -1.0;
        if let Some(participant) = self.layout_participant_mut() {
            participant.mark_layout_node_dirty();
        }
    }

    pub fn compute_world_bounds(&self, xform: Option<Mat2D>) -> Aabb {
        let mut result = Aabb::for_expansion();
        let mut first = true;
        for path in self.paths() {
            path.with(|path| {
                let Some(path) = path.as_path() else {
                    return;
                };
                if path.base.is_collapsed() {
                    return;
                }
                let mut raw = path.raw_path().clone();
                let matrix = xform
                    .map(|x| path.path_transform() * x)
                    .unwrap_or_else(|| path.path_transform());
                raw.transform_in_place(matrix);
                let bounds = raw.bounds();
                if first {
                    first = false;
                    result = bounds;
                } else {
                    result.expand(bounds);
                }
            });
        }
        result
    }
    pub fn compute_local_bounds(&self) -> Aabb {
        self.compute_world_bounds(Some(self.base.world_transform().invert_or_identity()))
    }
    pub fn local_bounds(&self) -> Aabb {
        self.compute_local_bounds()
    }

    pub fn compute_intrinsic_bounds(&self) -> Aabb {
        let participant = self.layout_participant();
        if participant.is_some_and(LayoutParticipant::host_bounds_valid) {
            return participant.unwrap().host_bounds();
        }
        let mut first = true;
        let mut result = Aabb::for_expansion();
        let mut used_pending = false;
        for path in self.paths() {
            path.with(|path| {
                let Some(path) = path.as_path() else {
                    return;
                };
                if path.base.is_collapsed() {
                    return;
                }
                let bounds = if !path.needs_path_build() {
                    let mut raw = path.raw_path().clone();
                    raw.transform_in_place(path.base.transform());
                    raw.precise_bounds()
                } else {
                    let mut property = Aabb::default();
                    used_pending = true;
                    if path.try_property_bounds(&mut property) {
                        path.base.transform().map_bounding_box(property)
                    } else {
                        let mut pending = RawPath::default();
                        path.build_path(&mut pending);
                        pending.transform_in_place(path.base.transform());
                        pending.precise_bounds()
                    }
                };
                if !(bounds.width() >= 0.0 && bounds.height() >= 0.0) {
                    return;
                }
                if first {
                    first = false;
                    result = bounds;
                } else {
                    result.expand(bounds);
                }
            });
        }
        let bounds = if first { Aabb::default() } else { result };
        if let Some(participant) = participant {
            participant.set_host_bounds(bounds, !used_pending);
        }
        bounds
    }

    fn invalidate_intrinsic_bounds(&mut self) {
        if let Some(participant) = self.layout_participant_mut() {
            participant.invalidate_host_bounds();
        }
    }
    pub fn measure_layout(
        &self,
        width: f32,
        width_mode: LayoutMeasureMode,
        height: f32,
        height_mode: LayoutMeasureMode,
    ) -> Vec2D {
        if self.is_participating_in_layout() {
            let bounds = self.compute_intrinsic_bounds();
            return Vec2D::new(bounds.width(), bounds.height());
        }
        self.paths().iter().fold(Vec2D::default(), |size, path| {
            let measured = path
                .with(|path| {
                    path.as_path().map(|path| {
                        path.base
                            .measure_layout(width, width_mode, height, height_mode)
                    })
                })
                .flatten()
                .unwrap_or_default();
            Vec2D::new(size.x.max(measured.x), size.y.max(measured.y))
        })
    }
    pub fn control_size(
        &mut self,
        size: Vec2D,
        width: LayoutScaleType,
        height: LayoutScaleType,
        direction: LayoutDirection,
    ) {
        if self.is_participating_in_layout() {
            self.update_layout_scale(size);
            return;
        }
        for path in self.paths() {
            let controlled = path
                .with_mut(|path| {
                    let Some(path) = path.as_parametric_path_mut() else {
                        return false;
                    };
                    path.control_size(size, width, height, direction);
                    true
                })
                .unwrap_or(false);
            if controlled {
                break;
            }
        }
    }
    fn update_layout_scale(&mut self, size: Vec2D) {
        let bounds = self.compute_intrinsic_bounds();
        let (width, height) = (bounds.width(), bounds.height());
        let (sx, sy) = (
            if width > 0.0 { size.x / width } else { 1.0 },
            if height > 0.0 { size.y / height } else { 1.0 },
        );
        let Some(participant) = self.layout_participant_mut() else {
            return;
        };
        if sx != participant.host_scale_x() || sy != participant.host_scale_y() {
            participant.set_host_scale(sx, sy);
            self.base.mark_world_transform_dirty();
        }
    }
    pub fn layout_participant(&self) -> Option<&LayoutParticipant> {
        self.base
            .children()
            .iter()
            .find_map(Core::as_layout_participant)
    }
    pub fn layout_participant_mut(&mut self) -> Option<&mut LayoutParticipant> {
        self.base
            .children_mut()
            .iter_mut()
            .find_map(Core::as_layout_participant_mut)
    }
    pub fn is_participating_in_layout(&self) -> bool {
        self.layout_participant().is_some()
    }
    pub(crate) fn try_compose_world_transform_override(&mut self) -> bool {
        let participant = self.base.children().iter().find_map(|child| {
            child
                .with(|child| {
                    child.as_any().downcast_ref::<crate::mechanical_port::source::layout::layout_participant::LayoutParticipant>().map(|participant| {
                        (
                            participant.resolved_left(),
                            participant.resolved_top(),
                            participant.host_scale_x(),
                            participant.host_scale_y(),
                        )
                    })
                })
                .flatten()
        });
        let parent_world = self.base.parent_transform_component().and_then(|parent| {
            parent
                .with(|parent| {
                    parent
                        .as_world_transform_component()
                        .map(|parent| *parent.world_transform())
                })
                .flatten()
        });
        if let (Some((left, top, sx, sy)), Some(parent_world)) = (participant, parent_world) {
            let intrinsic = self.compute_intrinsic_bounds();
            let base = Mat2D::from_translation(Vec2D::new(
                left - intrinsic.left() * sx,
                top - intrinsic.top() * sy,
            ));
            self.base.set_world_transform(
                parent_world * base * *self.base.transform() * Mat2D::from_scale(sx, sy),
            );
            return true;
        }
        false
    }

    pub fn with_path_mut<R>(
        &self,
        kind: ShapePaintPathKind,
        f: impl FnOnce(&mut ShapePaintPath) -> R,
    ) -> R {
        self.path_composer.with_mut(|helper| {
            f(match kind {
                ShapePaintPathKind::Local => helper.local_path(),
                ShapePaintPathKind::LocalClockwise => helper.local_clockwise_path(),
                ShapePaintPathKind::World => helper.world_path(),
            })
        })
    }
    pub fn path_builder(&self) -> ComponentOccurrenceHandle {
        self.path_composer.occurrence()
    }
    pub fn path_composer_mut(&mut self) -> &RuntimePathComposerHandle {
        &self.path_composer
    }
}

impl Default for Shape {
    fn default() -> Self {
        Self::new()
    }
}
impl std::ops::Deref for Shape {
    type Target = ShapeBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for Shape {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
