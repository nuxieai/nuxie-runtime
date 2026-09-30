use crate::mechanical_port::source::{
    artboard::Artboard,
    bones::skinnable::SkinnableBehavior,
    component::{Component, ComponentDirt, ComponentOccurrenceHandle, has_dirt},
    core::{Core, CoreHandle},
    core_context::CoreContext,
    drawable_flag::DrawableFlag,
    generated::{core_registry::CoreCapabilities, shapes::shape_base::ShapeBase},
    hit_info::HitInfo,
    hittest_command_path::HitTestCommandPath,
    layout::{
        layout_enums::{LayoutDirection, LayoutScaleType},
        layout_measure_mode::LayoutMeasureMode,
        layout_participant::LayoutParticipant,
    },
    math::{
        aabb::Aabb, contour_measure::ContourMeasureIter, mat2d::Mat2D, raw_path::RawPath,
        vec2d::Vec2D,
    },
    renderer::{RenderPath, Renderer},
    shapes::{
        deformer::render_path_deformer_from,
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
    local_bounds: std::cell::Cell<Aabb>,
    local_bounds_clean: std::cell::Cell<bool>,
    world_length: f32,
    want_difference_path: bool,
    has_layout_participant: bool,
    deformer: Option<CoreHandle>,
}

impl Shape {
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
            local_bounds: std::cell::Cell::new(Aabb::default()),
            local_bounds_clean: std::cell::Cell::new(false),
            world_length: -1.0,
            want_difference_path: false,
            has_layout_participant: false,
            deformer: None,
        }
    }

    pub fn add_path(&mut self, path: CoreHandle) {
        let has_skin = path
            .with(|object| {
                object
                    .as_points_path()
                    .is_some_and(|points| points.skin().is_some())
            })
            .unwrap_or(false);
        self.add_path_with_skin(path, has_skin);
    }
    /// Registration can run while the path's arena slot is already borrowed.
    /// The caller supplies its current skin state before entering this Shape.
    pub(crate) fn add_path_with_skin(&mut self, path: CoreHandle, has_skin: bool) {
        assert!(!self.paths.contains(&path));
        self.paths.push(path);
        if has_skin {
            self.add_flags(PathFlags::NEVER_DEFER_UPDATE);
        }
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
        self.base.render_opacity() == 0.0
            && !self.is_flagged(PathFlags::CLIPPING | PathFlags::NEVER_DEFER_UPDATE)
    }
    pub(crate) fn can_defer_path_update_with_active_path(
        &self,
        _active_points_path: Option<(&CoreHandle, bool)>,
    ) -> bool {
        self.can_defer_path_update()
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
                    .with(|object| {
                        let path = object.as_path()?;
                        let dirty = path.base.has_dirt(
                            ComponentDirt::PATH
                                | ComponentDirt::WORLD_TRANSFORM
                                | ComponentDirt::N_SLICER,
                        );
                        let mut temporary = RawPath::default();
                        let base = if dirty {
                            path.build_path_from_shape(
                                &mut temporary,
                                Path::is_path_closed_for(object),
                                self,
                            );
                            &temporary
                        } else {
                            path.raw_path()
                        };
                        let source = base.transform(Path::path_transform_for(object));
                        let mut length = 0.0;
                        let mut iter = ContourMeasureIter::new(&source, 0.5);
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
        // Geometry and path transforms invalidate this even when a transparent
        // shape defers the composer update and therefore mark_bounds_dirty.
        self.local_bounds_clean.set(false);
        self.invalidate_intrinsic_bounds();
        for constraint in self.base.constraints().to_vec() {
            crate::mechanical_port::source::component::ComponentOccurrenceHandle::Authored(
                constraint,
            )
            .add_dirt_from_shape(self, ComponentDirt::PATH, false);
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
                if !behavior.is_visible() {
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
            path.with(|object| {
                if let Some(path) = object.as_path()
                    && !path.base.is_collapsed()
                {
                    tester.set_xform(Path::path_transform_for(object));
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
            let Some((translucent, visible, flags)) = paint
                .with_mut(|object| {
                    object.as_shape_paint_behavior_mut().map(|paint| {
                        (
                            paint.is_translucent(),
                            paint.is_visible(),
                            paint.path_flags(),
                        )
                    })
                })
                .flatten()
            else {
                continue;
            };
            if translucent || !visible {
                continue;
            }
            let paint_local = !(flags & (PathFlags::LOCAL | PathFlags::LOCAL_CLOCKWISE)).is_empty();
            let matrix = if paint_local {
                xform * *self.base.world_transform()
            } else {
                xform
            };
            let mut tester = HitTestCommandPath::new(hinfo.area);
            for path in self.paths() {
                path.with(|object| {
                    let Some(path) = object.as_path() else {
                        return;
                    };
                    let path_transform = Path::path_transform_for(object);
                    tester.set_xform(if shape_local {
                        xform * path_transform
                    } else {
                        matrix * path_transform
                    });
                    path.raw_path().add_to(&mut tester);
                });
            }
            if tester.was_hit() {
                return Some(self);
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
                    .map(|paint| paint.blend_mode(blend.into()))
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
                .with(|current| render_path_deformer_from(current).is_some())
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
        self.local_bounds_clean.set(false);
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
        if self.base.drawable_flags() & DrawableFlag::WORLD_BOUNDS_CLEAN.0 == 0 {
            self.set_drawable_flags(
                self.base.drawable_flags() | DrawableFlag::WORLD_BOUNDS_CLEAN.0,
            );
            self.world_bounds = self.compute_world_bounds(None);
        }
        self.world_bounds
    }
    pub fn mark_bounds_dirty(&mut self) {
        self.set_drawable_flags(self.base.drawable_flags() & !DrawableFlag::WORLD_BOUNDS_CLEAN.0);
        self.local_bounds_clean.set(false);
        self.world_length = -1.0;
        if let Some(participant) = self.layout_participant() {
            participant.with_downcast_mut::<LayoutParticipant, _>(|participant| {
                participant.mark_layout_node_dirty_from_host(self, false)
            });
        }
    }

    /// Combined path control-point hulls in world space. The optional transform
    /// is applied after each path's world transform.
    pub fn compute_world_bounds(&self, xform: Option<Mat2D>) -> Aabb {
        let mut result = Aabb::for_expansion();
        let mut first = true;
        for path in self.paths() {
            path.with(|object| {
                let Some(path) = object.as_path() else {
                    return;
                };
                if path.base.is_collapsed() {
                    return;
                }
                let path_transform = Path::path_transform_for(object);
                let matrix = xform.map(|x| x * path_transform).unwrap_or(path_transform);
                let bounds = matrix.map_bounding_box_points(path.raw_path().points());
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
        if !self.local_bounds_clean.get() {
            self.local_bounds_clean.set(true);
            self.local_bounds.set(self.compute_local_bounds());
        }
        self.local_bounds.get()
    }

    pub fn compute_intrinsic_bounds(&self) -> Aabb {
        let participant = self.layout_participant();
        if let Some(bounds) = participant.as_ref().and_then(|participant| {
            participant
                .with_downcast::<LayoutParticipant, _>(|participant| {
                    participant
                        .host_bounds_valid()
                        .then(|| *participant.host_bounds())
                })
                .flatten()
        }) {
            return bounds;
        }
        let mut first = true;
        let mut result = Aabb::for_expansion();
        let mut used_pending = false;
        for path in self.paths() {
            path.with(|object| {
                let Some(path) = object.as_path() else {
                    return;
                };
                if path.base.is_collapsed() {
                    return;
                }
                let bounds = if !path.needs_path_build() {
                    path.raw_path()
                        .precise_bounds_with_transform(*path.base.transform())
                } else {
                    let mut property = Aabb::default();
                    used_pending = true;
                    let has_property_bounds = if let Some(parametric) = object.as_parametric_path()
                    {
                        parametric.try_property_bounds(&mut property)
                    } else {
                        path.try_property_bounds(&mut property)
                    };
                    if has_property_bounds {
                        path.base.transform().map_bounding_box(property)
                    } else {
                        let mut pending = RawPath::default();
                        path.build_path_from_shape(
                            &mut pending,
                            Path::is_path_closed_for(object),
                            self,
                        );
                        pending.precise_bounds_with_transform(*path.base.transform())
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
            participant.with_downcast_mut::<LayoutParticipant, _>(|participant| {
                participant.set_host_bounds(bounds, !used_pending)
            });
        }
        bounds
    }

    fn invalidate_intrinsic_bounds(&mut self) {
        if let Some(participant) = self.layout_participant() {
            participant.with_downcast_mut::<LayoutParticipant, _>(
                LayoutParticipant::invalidate_host_bounds,
            );
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
            let available_width = if width_mode == LayoutMeasureMode::Undefined {
                f32::MAX
            } else {
                width
            };
            let available_height = if height_mode == LayoutMeasureMode::Undefined {
                f32::MAX
            } else {
                height
            };
            // std::min keeps its first operand when the comparison is false,
            // including NaN; f32::min does not have that selection behavior.
            return Vec2D::new(
                if bounds.width() < available_width {
                    bounds.width()
                } else {
                    available_width
                },
                if bounds.height() < available_height {
                    bounds.height()
                } else {
                    available_height
                },
            );
        }
        self.paths().iter().fold(Vec2D::default(), |size, path| {
            let measured = path
                .with_mut(|path| {
                    path.as_intrinsically_sizeable_mut()
                        .map(|path| path.measure_layout(width, width_mode, height, height_mode))
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
        if let Some(path) = self.prepare_control_size(size) {
            path.with_mut(|path| {
                path.as_parametric_path_mut()
                    .expect("firstParametricPath selects a ParametricPath")
                    .control_size(size, width, height, direction);
            });
        }
    }
    pub fn control_size_occurrence(
        owner: &CoreHandle,
        size: Vec2D,
        width: LayoutScaleType,
        height: LayoutScaleType,
        direction: LayoutDirection,
    ) {
        let path = owner
            .with_downcast_mut::<Shape, _>(|shape| shape.prepare_control_size(size))
            .flatten();
        // ParametricPath's width/height callbacks synchronously call back into
        // this Shape. Its owning arena borrow must end before that virtual call.
        if let Some(path) = path {
            path.with_mut(|path| {
                path.as_parametric_path_mut()
                    .expect("firstParametricPath selects a ParametricPath")
                    .control_size(size, width, height, direction);
            });
        }
    }
    fn prepare_control_size(&mut self, size: Vec2D) -> Option<CoreHandle> {
        if self.is_participating_in_layout() {
            self.update_layout_scale(size);
            return None;
        }
        self.paths()
            .iter()
            .find(|path| path.is_type_of(crate::mechanical_port::source::generated::shapes::parametric_path_base::ParametricPathBase::TYPE_KEY))
            .cloned()
    }
    fn update_layout_scale(&mut self, size: Vec2D) {
        let bounds = self.compute_intrinsic_bounds();
        let (width, height) = (bounds.width(), bounds.height());
        let (sx, sy) = (
            if width > 0.0 { size.x / width } else { 1.0 },
            if height > 0.0 { size.y / height } else { 1.0 },
        );
        let Some(participant) = self.layout_participant() else {
            return;
        };
        let changed = participant
            .with_downcast_mut::<LayoutParticipant, _>(|participant| {
                if sx != participant.host_scale_x() || sy != participant.host_scale_y() {
                    participant.set_host_scale(sx, sy);
                    true
                } else {
                    false
                }
            })
            .unwrap_or(false);
        if changed {
            CoreCapabilities::world_transform_mark_dirty(self);
        }
    }
    pub fn add_child(&mut self, child: CoreHandle) {
        let is_participant = child.is_type_of(LayoutParticipant::TYPE_KEY);
        self.base.add_child(child);
        if is_participant {
            self.has_layout_participant = true;
        }
    }
    pub fn layout_participant(&self) -> Option<CoreHandle> {
        // Runtime children are fixed; WITH_RIVE_EDITOR's rescan is not a
        // tools-build behavior.
        if !self.has_layout_participant {
            return None;
        }
        self.base
            .children()
            .iter()
            .find(|child| {
                child.is_type_of(crate::mechanical_port::source::generated::layout::layout_participant_base::LayoutParticipantBase::TYPE_KEY)
            })
            .cloned()
    }
    fn set_drawable_flags(&mut self, value: u16) {
        if self.base.set_drawable_flags_value(value) {
            use crate::mechanical_port::source::generated::drawable_base::{
                DrawableBase, DrawableBaseCallbacks,
            };
            DrawableBaseCallbacks::drawable_flags_changed(self);
            DrawableBaseCallbacks::notify_property_changed(
                self,
                DrawableBase::DRAWABLE_FLAGS_PROPERTY_KEY,
            );
        }
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
            let transform = *self.base.transform();
            self.base
                .set_world_transform(parent_world * base * transform * Mat2D::from_scale(sx, sy));
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
