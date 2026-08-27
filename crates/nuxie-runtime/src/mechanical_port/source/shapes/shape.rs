use crate::mechanical_port::source::{
    component::{Component, ComponentDirt, has_dirt},
    core::{Core, CoreContext, StatusCode},
    drawable::DrawableFlag,
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
        deformer::RenderPathDeformer, parametric_path::ParametricPath, path::Path,
        path_composer::PathComposer, path_flags::PathFlags,
        shape_paint_container::ShapePaintContainer, shape_paint_path::ShapePaintPath,
    },
};

pub struct Shape {
    pub base: ShapeBase,
    pub paint_container: ShapePaintContainer,
    path_composer: Box<PathComposer>,
    paths: Vec<*mut Path>,
    world_bounds: Aabb,
    world_length: f32,
    want_difference_path: bool,
    deformer: Option<*mut dyn RenderPathDeformer>,
}

impl Shape {
    pub fn new() -> Self {
        let mut value = Self {
            base: ShapeBase::default(),
            paint_container: ShapePaintContainer::default(),
            path_composer: Box::new(PathComposer::unbound()),
            paths: Vec::new(),
            world_bounds: Aabb::default(),
            world_length: -1.0,
            want_difference_path: false,
            deformer: None,
        };
        value.path_composer = Box::new(PathComposer::new(value.shallow_handle()));
        value
    }

    pub fn add_path(&mut self, path: &mut Path) {
        let pointer = path as *mut Path;
        assert!(!self.paths.contains(&pointer));
        self.paths.push(pointer);
        self.invalidate_intrinsic_bounds();
    }
    pub fn paths(&self) -> impl Iterator<Item = &Path> {
        self.paths.iter().map(|p| unsafe { &**p })
    }
    pub fn paths_mut(&mut self) -> impl Iterator<Item = &mut Path> {
        self.paths.iter().map(|p| unsafe { &mut **p })
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
    pub fn deformer(&self) -> Option<&dyn RenderPathDeformer> {
        self.deformer.map(|p| unsafe { &*p })
    }

    pub fn can_defer_path_update(&self) -> bool {
        let can_defer = self.base.render_opacity() == 0.0
            && !self.is_flagged(PathFlags::CLIPPING | PathFlags::NEVER_DEFER_UPDATE);
        if can_defer
            && self
                .base
                .dependents()
                .iter()
                .any(|d| d.as_points_path().is_some_and(|p| p.skin().is_some()))
        {
            return false;
        }
        can_defer
    }

    pub fn update(&mut self, value: ComponentDirt) {
        self.base.update(value);
        if has_dirt(value, ComponentDirt::RENDER_OPACITY) {
            self.paint_container
                .propagate_opacity(self.base.render_opacity());
        }
    }
    pub fn collapse(&mut self, value: bool) -> bool {
        if !self.base.collapse(value) {
            return false;
        }
        self.path_composer.component.collapse(value);
        self.invalidate_intrinsic_bounds();
        true
    }

    pub fn length(&mut self) -> f32 {
        if self.world_length < 0.0 {
            let mut length = 0.0;
            for path in self.paths() {
                let dirty = path.base.has_dirt(
                    ComponentDirt::PATH | ComponentDirt::WORLD_TRANSFORM | ComponentDirt::N_SLICER,
                );
                let mut temporary = RawPath::default();
                let base = if dirty {
                    path.build_path(&mut temporary);
                    &temporary
                } else {
                    path.raw_path()
                };
                let source = base.transform(path.path_transform());
                let mut iter = ContourMeasureIter::new(&source);
                while let Some(contour) = iter.next() {
                    length += contour.length();
                }
            }
            self.world_length = length;
        }
        self.world_length
    }

    pub fn set_length(&mut self, _value: f32) {}

    pub fn path_changed(&mut self) {
        self.path_composer
            .component
            .add_dirt_with_recurse(ComponentDirt::PATH, true);
        self.world_length = -1.0;
        self.invalidate_intrinsic_bounds();
        for constraint in self.base.constraints_mut() {
            constraint.add_dirt(ComponentDirt::PATH);
        }
        self.paint_container.invalidate_stroke_effects();
    }

    pub fn add_to_render_path(&mut self, path: &mut RenderPath, transform: Mat2D) {
        if self.is_flagged(PathFlags::LOCAL) {
            let render = self.path_composer.local_path().render_path(self);
            path.add_path(render, transform * self.base.world_transform());
        } else {
            let render = self.path_composer.world_path().render_path(self);
            path.add_path(render, transform);
        }
    }

    pub fn add_to_raw_path(&mut self, path: &mut RawPath, transform: Option<Mat2D>) {
        if self.is_flagged(PathFlags::LOCAL) {
            let matrix = transform
                .map(|v| v * self.base.world_transform())
                .unwrap_or_else(|| self.base.world_transform());
            path.add_path(self.path_composer.local_path().raw_path(), Some(matrix));
        } else {
            path.add_path(self.path_composer.world_path().raw_path(), transform);
        }
    }

    pub fn draw(&mut self, renderer: &mut Renderer) {
        let needs_save =
            self.base.needs_save_operation() || self.paint_container.shape_paints().len() > 1;
        for paint in self.paint_container.shape_paints_mut() {
            if !paint.base.is_visible() {
                continue;
            }
            let Some(path) = paint.pick_path_option(self.paint_container()) else {
                continue;
            };
            paint.draw(
                renderer,
                path,
                self.base.world_transform(),
                false,
                None,
                needs_save,
            );
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
            if !path.base.is_collapsed() {
                tester.set_xform(path.path_transform());
                path.raw_path().add_to(&mut tester);
            }
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
                tester.set_xform(if shape_local {
                    xform * path.path_transform()
                } else {
                    matrix * path.path_transform()
                });
                path.raw_path().add_to(&mut tester);
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
            return self
                .base
                .hit_test_point(position, skip_on_unclipped, primary);
        }
        self.hit_test_aabb(position)
            && self
                .base
                .hit_test_point(position, skip_on_unclipped, primary)
            && self.hit_test_hi_fi(position, 2.0)
    }

    pub fn build_dependencies(&mut self) {
        self.path_composer.build_dependencies();
        self.base.build_dependencies();
        let blend = self.base.blend_mode();
        for paint in self.paint_container.shape_paints_mut() {
            paint.blend_mode(blend);
        }
    }
    pub fn on_added_dirty(&mut self, context: &mut CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        self.path_composer.component.on_added_dirty(context)
    }
    pub fn on_added_clean(&mut self, context: &mut CoreContext) -> StatusCode {
        let code = self.base.on_added_clean(context);
        if code != StatusCode::Ok {
            return code;
        }
        self.deformer = None;
        let mut parent = self.base.parent_mut();
        while let Some(current) = parent {
            if let Some(deformer) = render_path_deformer_from(current) {
                self.deformer = Some(deformer as *mut _);
                return StatusCode::Ok;
            }
            parent = current.parent_mut();
        }
        StatusCode::Ok
    }
    pub fn is_empty(&self) -> bool {
        self.paths()
            .all(|path| path.is_hidden() || path.base.is_collapsed())
    }
    pub fn will_draw(&self) -> bool {
        self.base.will_draw() && self.base.render_opacity() != 0.0
    }
    pub fn path_collapse_changed(&mut self) {
        self.path_composer.path_collapse_changed();
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
        #[cfg(feature = "rive_layout")]
        if let Some(participant) = self.layout_participant_mut() {
            participant.mark_layout_node_dirty();
        }
    }

    pub fn compute_world_bounds(&self, xform: Option<Mat2D>) -> Aabb {
        let mut result = Aabb::for_expansion();
        let mut first = true;
        for path in self.paths() {
            if path.base.is_collapsed() {
                continue;
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
        }
        result
    }
    pub fn compute_local_bounds(&self) -> Aabb {
        self.compute_world_bounds(Some(self.base.world_transform().invert_or_identity()))
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
            if path.base.is_collapsed() {
                continue;
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
                continue;
            }
            if first {
                first = false;
                result = bounds;
            } else {
                result.expand(bounds);
            }
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
        #[cfg(feature = "rive_layout")]
        if self.is_participating_in_layout() {
            let bounds = self.compute_intrinsic_bounds();
            return Vec2D::new(bounds.width(), bounds.height());
        }
        self.paths().fold(Vec2D::default(), |size, path| {
            let measured = path
                .base
                .measure_layout(width, width_mode, height, height_mode);
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
        #[cfg(feature = "rive_layout")]
        if self.is_participating_in_layout() {
            self.update_layout_scale(size);
            return;
        }
        if let Some(path) = self.paths_mut().find_map(Path::as_parametric_path_mut) {
            path.control_size(size, width, height, direction);
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
    pub fn compose_world_transform(&mut self) {
        #[cfg(feature = "rive_layout")]
        if let (Some(participant), Some(parent)) = (
            self.layout_participant(),
            self.base.parent_transform_component(),
        ) {
            let intrinsic = self.compute_intrinsic_bounds();
            let (sx, sy) = (participant.host_scale_x(), participant.host_scale_y());
            let base = Mat2D::from_translation(Vec2D::new(
                participant.resolved_left() - intrinsic.left() * sx,
                participant.resolved_top() - intrinsic.top() * sy,
            ));
            self.base.set_world_transform(
                parent.world_transform() * base * self.base.transform() * Mat2D::from_scale(sx, sy),
            );
            return;
        }
        self.base.compose_world_transform();
    }

    pub fn world_path(&mut self) -> &mut ShapePaintPath {
        self.path_composer.world_path()
    }
    pub fn local_path(&mut self) -> &mut ShapePaintPath {
        self.path_composer.local_path()
    }
    pub fn local_clockwise_path(&mut self) -> &mut ShapePaintPath {
        self.path_composer.local_clockwise_path()
    }
    pub fn path_builder(&mut self) -> &mut Component {
        &mut self.path_composer.component
    }
    pub fn path_composer_mut(&mut self) -> &mut PathComposer {
        &mut self.path_composer
    }
}
