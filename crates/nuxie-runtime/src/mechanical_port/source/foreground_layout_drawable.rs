use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    core::CoreHandle,
    generated::foreground_layout_drawable_base::ForegroundLayoutDrawableBase,
    hit_info::HitInfo,
    math::mat2d::Mat2D,
    renderer::Renderer,
    shapes::{
        paint::{shape_paint::ShapePaintPathKind, shape_paint_path::ShapePaintPath},
        shape_paint_container::ShapePaintContainer,
    },
};

#[derive(Default)]
pub struct ForegroundLayoutDrawable {
    pub base: ForegroundLayoutDrawableBase,
    paint_container: ShapePaintContainer,
}

impl ForegroundLayoutDrawable {
    pub fn shape_paint_container(&self) -> &ShapePaintContainer {
        &self.paint_container
    }
    pub fn shape_paint_container_mut(&mut self) -> &mut ShapePaintContainer {
        &mut self.paint_container
    }

    pub fn build_dependencies(&mut self) {
        self.base.build_dependencies();
        let Some(parent) = self.base.parent_handle() else {
            return;
        };
        let blend = parent
            .with_mut(|parent| {
                parent.as_layout_component_mut().map(|parent| {
                    parent.register_foreground_drawable();
                    parent.base.base.blend_mode()
                })
            })
            .flatten();
        if let Some(blend) = blend {
            for paint in self.paint_container.shape_paints().iter().cloned() {
                paint.with_mut(|paint| {
                    if let Some(paint) = paint.as_shape_paint_mut() {
                        paint.blend_mode(blend);
                    }
                });
            }
        }
    }

    pub(crate) fn update_after_transform_super(&mut self, value: ComponentDirt) {
        let Some(parent) = self.base.parent_handle() else {
            return;
        };
        let opacity = parent
            .with(|parent| {
                parent.as_layout_component()?;
                parent.world_transform_child_opacity()
            })
            .flatten();
        let Some(opacity) = opacity else {
            return;
        };
        if value.contains(ComponentDirt::RENDER_OPACITY) {
            self.paint_container.propagate_opacity(opacity);
        }
        if !(value & (ComponentDirt::PATH | ComponentDirt::WORLD_TRANSFORM)).is_empty() {
            self.paint_container.invalidate_stroke_effects();
        }
    }

    pub fn draw(&mut self, renderer: &mut Renderer) {
        let Some(parent) = self.base.parent_handle() else {
            return;
        };
        for paint in self.paint_container.shape_paints().iter().cloned() {
            paint.with_mut(|paint| {
                let Some(paint) = paint.as_shape_paint_behavior_mut() else {
                    return;
                };
                if !paint.shape_paint().should_draw() {
                    return;
                }
                let kind = paint.pick_path_kind();
                parent.with_mut(|parent| {
                    let Some(parent) = parent.as_layout_component_mut() else {
                        return;
                    };
                    let world = parent.shape_world_transform();
                    let path = match kind {
                        ShapePaintPathKind::World => parent.world_path(),
                        ShapePaintPathKind::Local => parent.local_path(),
                        ShapePaintPathKind::LocalClockwise => parent.local_clockwise_path(),
                    };
                    paint
                        .shape_paint_mut()
                        .draw(renderer, path, world, false, None, true);
                });
            });
        }
    }

    pub fn hit_test(&mut self, _info: &mut HitInfo, _transform: &Mat2D) -> Option<CoreHandle> {
        None
    }
    pub fn shape_world_transform(&self) -> Mat2D {
        *self.base.world_transform()
    }
    pub fn path_builder(&self) -> Option<CoreHandle> {
        self.base.parent_handle()
    }

    /// Paths remain owned by the parent LayoutComponent; no reference is
    /// allowed to escape the parent's CoreHandle borrow.
    pub fn with_path_mut(
        &self,
        kind: ShapePaintPathKind,
        use_path: &mut dyn FnMut(&mut ShapePaintPath),
    ) -> bool {
        self.base
            .parent_handle()
            .and_then(|parent| {
                parent
                    .with_mut(|parent| {
                        let parent = parent.as_layout_component_mut()?;
                        let path = match kind {
                            ShapePaintPathKind::World => parent.world_path(),
                            ShapePaintPathKind::Local => parent.local_path(),
                            ShapePaintPathKind::LocalClockwise => parent.local_clockwise_path(),
                        };
                        use_path(path);
                        Some(())
                    })
                    .flatten()
            })
            .is_some()
    }
}
impl std::ops::Deref for ForegroundLayoutDrawable {
    type Target = ForegroundLayoutDrawableBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for ForegroundLayoutDrawable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
