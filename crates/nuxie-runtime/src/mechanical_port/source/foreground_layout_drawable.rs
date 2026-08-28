use crate::mechanical_port::source::{
    artboard::Artboard,
    component::Component,
    component_dirt::ComponentDirt,
    core::Core,
    generated::foreground_layout_drawable_base::ForegroundLayoutDrawableBase,
    hit_info::HitInfo,
    math::mat2d::Mat2D,
    renderer::Renderer,
    shapes::{shape_paint_container::ShapePaintContainer, shape_paint_path::ShapePaintPath},
};

#[derive(Default)]
pub struct ForegroundLayoutDrawable {
    pub base: ForegroundLayoutDrawableBase,
    paint_container: ShapePaintContainer,
}

impl ForegroundLayoutDrawable {
    pub fn build_dependencies(&mut self) {
        self.base.base.base.base.base.base.build_dependencies();
        let Some(parent_layout) = self
            .base
            .base
            .base
            .base
            .base
            .base
            .parent_mut()
            .and_then(|parent| parent.as_layout_component_mut())
        else {
            return;
        };
        parent_layout.register_foreground_drawable();
        let blend_mode = parent_layout.blend_mode();
        for paint in self.paint_container.shape_paints_mut() {
            paint.blend_mode(blend_mode);
        }
    }

    pub fn update(&mut self, value: ComponentDirt) {
        self.base.base.base.base.base.base.update(value);
        let Some(parent_layout) = self
            .base
            .base
            .base
            .base
            .base
            .base
            .parent_mut()
            .and_then(|parent| parent.as_layout_component_mut())
        else {
            return;
        };
        if value.contains(ComponentDirt::RENDER_OPACITY) {
            self.paint_container
                .propagate_opacity(parent_layout.child_opacity());
        }
        if value.contains(ComponentDirt::PATH | ComponentDirt::WORLD_TRANSFORM) {
            self.paint_container.invalidate_stroke_effects();
        }
    }

    pub fn draw(&mut self, renderer: &mut Renderer) {
        let Some(parent_layout) = self
            .base
            .base
            .base
            .base
            .base
            .base
            .parent_mut()
            .and_then(|parent| parent.as_layout_component_mut())
        else {
            return;
        };
        for paint in self.paint_container.shape_paints_mut() {
            if !paint.should_draw() {
                continue;
            }
            let path = paint.pick_path(parent_layout.shape_paint_container());
            paint.draw(
                renderer,
                path,
                *parent_layout.world_transform(),
                true,
                None,
                self.base.base.needs_save_operation(),
            );
        }
    }

    pub fn hit_test(&mut self, _info: &mut HitInfo, _transform: &Mat2D) -> Option<&mut Core> {
        None
    }

    pub fn get_artboard(&mut self) -> Option<&mut Artboard> {
        self.base.base.base.base.base.base.artboard_mut()
    }

    pub fn shape_world_transform(&self) -> &Mat2D {
        self.base.base.base.base.base.world_transform()
    }

    pub fn path_builder(&mut self) -> Option<&mut Component> {
        self.base
            .base
            .base
            .base
            .base
            .base
            .parent_mut()
            .map(|parent| &mut parent.base.base)
    }

    pub fn world_path(&mut self) -> Option<&mut ShapePaintPath> {
        self.base
            .base
            .base
            .base
            .base
            .base
            .parent_mut()?
            .as_layout_component_mut()?
            .world_path()
    }

    pub fn local_path(&mut self) -> Option<&mut ShapePaintPath> {
        self.base
            .base
            .base
            .base
            .base
            .base
            .parent_mut()?
            .as_layout_component_mut()?
            .local_path()
    }

    pub fn local_clockwise_path(&mut self) -> Option<&mut ShapePaintPath> {
        self.base
            .base
            .base
            .base
            .base
            .base
            .parent_mut()?
            .as_layout_component_mut()?
            .local_clockwise_path()
    }
}
