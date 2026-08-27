use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    core::Core,
    core_context::CoreContext,
    draw_rules::DrawRules,
    drawable_flag::DrawableFlag,
    generated::drawable_base::{DrawableBase, DrawableBaseCallbacks},
    hit_info::HitInfo,
    layout_component::LayoutComponent,
    math::{mat2d::Mat2D, vec2d::Vec2D},
    renderer::Renderer,
    shapes::{clipping_shape::ClippingShape, paint::blend_mode::BlendMode},
    status_code::StatusCode,
};

pub struct Drawable {
    pub base: DrawableBase,
    clipping_shapes: Vec<*mut ClippingShape>,
    pub(crate) flattened_draw_rules: Option<*mut DrawRules>,
    pub(crate) prev: Option<*mut Drawable>,
    pub(crate) next: Option<*mut Drawable>,
    needs_save_operation: bool,
}

impl Default for Drawable {
    fn default() -> Self {
        Self {
            base: DrawableBase::default(),
            clipping_shapes: Vec::new(),
            flattened_draw_rules: None,
            prev: None,
            next: None,
            needs_save_operation: true,
        }
    }
}

impl DrawableBaseCallbacks for Drawable {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .base
            .base
            .base
            .base
            .base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }
}

impl Drawable {
    pub fn blend_mode(&self) -> BlendMode {
        match self.base.blend_mode_value() {
            3 => BlendMode::SrcOver,
            14 => BlendMode::Screen,
            15 => BlendMode::Overlay,
            16 => BlendMode::Darken,
            17 => BlendMode::Lighten,
            18 => BlendMode::ColorDodge,
            19 => BlendMode::ColorBurn,
            20 => BlendMode::HardLight,
            21 => BlendMode::SoftLight,
            22 => BlendMode::Difference,
            23 => BlendMode::Exclusion,
            24 => BlendMode::Multiply,
            25 => BlendMode::Hue,
            26 => BlendMode::Saturation,
            27 => BlendMode::Color,
            28 => BlendMode::Luminosity,
            value => panic!("invalid blend mode {value}"),
        }
    }

    pub fn draw(&mut self, _renderer: &mut dyn Renderer) {
        panic!("abstract Drawable::draw");
    }

    pub fn hit_test(&mut self, _info: &mut HitInfo, _transform: &Mat2D) -> Option<&mut Core> {
        panic!("abstract Drawable::hit_test");
    }

    pub fn hit_test_point(
        &mut self,
        position: &Vec2D,
        skip_on_unclipped: bool,
        is_primary_hit: bool,
    ) -> bool {
        if self.is_hidden() {
            return false;
        }
        let this = self as *mut Drawable;
        if let Some(hittable) = self.hittable_component()
            && hittable != this
        {
            return unsafe { &mut *hittable }.hit_test_point(
                position,
                skip_on_unclipped,
                is_primary_hit,
            );
        }
        self.base
            .base
            .base
            .base
            .base
            .hit_test_point(position, skip_on_unclipped, is_primary_hit)
    }

    pub fn add_clipping_shape(&mut self, shape: *mut ClippingShape) {
        self.clipping_shapes.push(shape);
    }

    pub fn clipping_shapes(&self) -> &[*mut ClippingShape] {
        &self.clipping_shapes
    }

    pub fn is_hidden(&self) -> bool {
        self.base.drawable_flags() as u16 & DrawableFlag::HIDDEN.0 == DrawableFlag::HIDDEN.0
            || self
                .base
                .base
                .base
                .base
                .base
                .has_dirt(ComponentDirt::COLLAPSED)
    }

    pub fn is_target_opaque(&self) -> bool {
        self.base.drawable_flags() as u16 & DrawableFlag::OPAQUE.0 == DrawableFlag::OPAQUE.0
    }

    pub fn is_proxy(&self) -> bool {
        false
    }
    pub fn is_clip_start(&self) -> bool {
        false
    }
    pub fn is_clip_end(&self) -> bool {
        false
    }
    pub fn will_clip(&self) -> bool {
        false
    }
    pub fn will_draw(&self) -> bool {
        !self.is_hidden()
    }
    pub fn set_needs_save_operation(&mut self, value: bool) {
        self.needs_save_operation = value;
    }
    pub fn needs_save_operation(&self) -> bool {
        self.needs_save_operation
    }

    pub fn is_child_of_layout(&mut self, layout: *mut LayoutComponent) -> bool {
        let mut parent = Some(self.base.base.base.base.base.parent_mut());
        while let Some(Some(current)) = parent {
            if current
                .as_layout_component_mut()
                .is_some_and(|candidate| std::ptr::eq(candidate, layout))
            {
                return true;
            }
            parent = Some(current.base.base.parent_mut());
        }
        false
    }

    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.base.base.base.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        match self.blend_mode() {
            BlendMode::SrcOver
            | BlendMode::Screen
            | BlendMode::Overlay
            | BlendMode::Darken
            | BlendMode::Lighten
            | BlendMode::ColorDodge
            | BlendMode::ColorBurn
            | BlendMode::HardLight
            | BlendMode::SoftLight
            | BlendMode::Difference
            | BlendMode::Exclusion
            | BlendMode::Multiply
            | BlendMode::Hue
            | BlendMode::Saturation
            | BlendMode::Color
            | BlendMode::Luminosity => StatusCode::Ok,
        }
    }

    pub fn hittable_component(&mut self) -> Option<*mut Drawable> {
        Some(self as *mut Drawable)
    }

    pub fn empty_clip_count(&self) -> i32 {
        0
    }

    pub fn next_drawable(&self) -> Option<*mut Drawable> {
        self.next
    }
    pub fn prev_drawable(&self) -> Option<*mut Drawable> {
        self.prev
    }
}

pub trait ProxyDrawing {
    fn draw_proxy(&mut self, renderer: &mut dyn Renderer);
    fn is_proxy_hidden(&self) -> bool;
    fn as_layout_component_mut(&mut self) -> &mut LayoutComponent;
}

pub struct DrawableProxy {
    pub base: Drawable,
    proxy_drawing: Box<dyn ProxyDrawing>,
}

impl DrawableProxy {
    pub fn new(proxy_drawing: Box<dyn ProxyDrawing>) -> Self {
        Self {
            base: Drawable::default(),
            proxy_drawing,
        }
    }

    pub fn draw(&mut self, renderer: &mut dyn Renderer) {
        self.proxy_drawing.draw_proxy(renderer);
    }
    pub fn is_hidden(&self) -> bool {
        self.proxy_drawing.is_proxy_hidden()
    }
    pub fn hittable_component(&mut self) -> *mut Drawable {
        self.proxy_drawing
            .as_layout_component_mut()
            .as_drawable_mut()
    }
    pub fn is_target_opaque(&mut self) -> bool {
        unsafe { &mut *self.hittable_component() }.is_target_opaque()
    }
    pub fn hit_test(&mut self, _info: &mut HitInfo, _transform: &Mat2D) -> Option<&mut Core> {
        None
    }
    pub fn is_proxy(&self) -> bool {
        true
    }
    pub fn proxy_drawing(&self) -> &dyn ProxyDrawing {
        &*self.proxy_drawing
    }
}
