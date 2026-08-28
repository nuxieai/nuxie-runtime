use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    core::CoreHandle,
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
    clipping_shapes: Vec<CoreHandle>,
    pub(crate) flattened_draw_rules: Option<CoreHandle>,
    pub(crate) prev: Option<RuntimeDrawableWeakOccurrence>,
    pub(crate) next: Option<RuntimeDrawableWeakOccurrence>,
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

    pub fn draw(&mut self, _renderer: &mut Renderer) {
        panic!("abstract Drawable::draw");
    }

    pub fn hit_test(&mut self, _info: &mut HitInfo, _transform: &Mat2D) -> Option<CoreHandle> {
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
        let this = self.base.base.base.base.base.handle();
        if let Some(hittable) = self.hittable_component()
            && this.as_ref() != Some(&hittable)
        {
            return hittable
                .with_mut(|hittable| {
                    hittable
                        .component_hit_test_point(position, skip_on_unclipped, is_primary_hit)
                        .unwrap_or(false)
                })
                .unwrap_or(false);
        }
        self.base
            .base
            .base
            .base
            .base
            .hit_test_point(position, skip_on_unclipped, is_primary_hit)
    }

    pub fn add_clipping_shape(&mut self, shape: CoreHandle) {
        self.clipping_shapes.push(shape);
    }

    pub fn clipping_shapes(&self) -> &[CoreHandle] {
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

    pub fn is_child_of_layout(&self, layout: &CoreHandle) -> bool {
        let mut parent = self.base.base.base.base.base.parent_handle();
        while let Some(current) = parent {
            if &current == layout {
                return true;
            }
            parent = current
                .with(|current| current.component_parent_handle())
                .flatten();
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

    pub fn hittable_component(&self) -> Option<CoreHandle> {
        self.base.base.base.base.base.handle()
    }

    pub fn empty_clip_count(&self) -> i32 {
        0
    }

    pub fn next_drawable(&self) -> Option<RuntimeDrawableOccurrence> {
        self.next
            .as_ref()
            .and_then(RuntimeDrawableWeakOccurrence::upgrade)
    }
    pub fn prev_drawable(&self) -> Option<RuntimeDrawableOccurrence> {
        self.prev
            .as_ref()
            .and_then(RuntimeDrawableWeakOccurrence::upgrade)
    }
}

pub trait ProxyDrawing {
    fn draw_proxy(&mut self, renderer: &mut Renderer, needs_save_operation: bool);
    fn is_proxy_hidden(&self) -> bool;
    fn owner_handle(&self) -> CoreHandle;
    fn empty_clip_count(&mut self) -> i32 {
        0
    }
    fn is_clip_start(&self) -> bool {
        false
    }
    fn is_clip_end(&self) -> bool {
        false
    }
    fn will_clip(&self) -> bool {
        false
    }
}

#[derive(Clone)]
pub enum RuntimeDrawableOccurrence {
    Authored(CoreHandle),
    RuntimeProxy(Rc<RefCell<DrawableProxy>>),
}

#[derive(Clone)]
pub enum RuntimeDrawableWeakOccurrence {
    Authored(CoreHandle),
    RuntimeProxy(Weak<RefCell<DrawableProxy>>),
}

impl RuntimeDrawableOccurrence {
    pub fn with_component<R>(
        &self,
        use_component: impl FnOnce(&crate::mechanical_port::source::component::Component) -> R,
    ) -> Option<R> {
        match self {
            Self::Authored(handle) => {
                handle.with(|object| object.as_component().map(use_component))?
            }
            Self::RuntimeProxy(proxy) => Some(use_component(&proxy.borrow().base)),
        }
    }

    pub fn with_component_mut<R>(
        &self,
        use_component: impl FnOnce(&mut crate::mechanical_port::source::component::Component) -> R,
    ) -> Option<R> {
        match self {
            Self::Authored(handle) => {
                handle.with_mut(|object| object.as_component_mut().map(use_component))?
            }
            Self::RuntimeProxy(proxy) => Some(use_component(&mut proxy.borrow_mut().base)),
        }
    }

    pub fn hit_test_point(
        &self,
        position: &Vec2D,
        skip_on_unclipped: bool,
        is_primary_hit: bool,
    ) -> bool {
        match self {
            Self::Authored(handle) => handle
                .with_mut(|object| {
                    object.component_hit_test_point(position, skip_on_unclipped, is_primary_hit)
                })
                .flatten()
                .unwrap_or(false),
            Self::RuntimeProxy(proxy) => {
                let owner = proxy.borrow().hittable_component();
                owner
                    .with_mut(|owner| {
                        owner.component_hit_test_point(position, skip_on_unclipped, is_primary_hit)
                    })
                    .flatten()
                    .unwrap_or(false)
            }
        }
    }

    pub fn is_target_opaque(&self) -> bool {
        match self {
            Self::Authored(handle) => handle
                .with(|object| object.as_drawable().map(Drawable::is_target_opaque))
                .flatten()
                .unwrap_or(false),
            Self::RuntimeProxy(proxy) => proxy.borrow_mut().is_target_opaque(),
        }
    }

    pub fn authored(handle: CoreHandle) -> Self {
        Self::Authored(handle)
    }

    pub fn runtime_proxy(proxy: Rc<RefCell<DrawableProxy>>) -> Self {
        Self::RuntimeProxy(proxy)
    }

    pub fn authored_handle(&self) -> Option<CoreHandle> {
        match self {
            Self::Authored(handle) => Some(handle.clone()),
            Self::RuntimeProxy(_) => None,
        }
    }

    pub fn is_hidden(&self) -> bool {
        match self {
            Self::Authored(handle) => handle
                .with(|object| object.drawable_is_hidden())
                .unwrap_or(true),
            Self::RuntimeProxy(proxy) => proxy.borrow().is_hidden(),
        }
    }

    pub fn will_draw(&self) -> bool {
        match self {
            Self::Authored(handle) => handle
                .with(|object| object.drawable_will_draw())
                .unwrap_or(false),
            Self::RuntimeProxy(proxy) => !proxy.borrow().is_hidden(),
        }
    }

    pub fn draw(&self, renderer: &mut Renderer) -> bool {
        match self {
            Self::Authored(handle) => {
                crate::mechanical_port::source::generated::core_registry::drawable_draw_handle(
                    handle, renderer,
                )
            }
            Self::RuntimeProxy(proxy) => {
                proxy.borrow_mut().draw(renderer);
                true
            }
        }
    }

    pub fn add_to_render_path(
        &self,
        path: &mut crate::mechanical_port::source::renderer::RenderPath,
        transform: &Mat2D,
    ) -> bool {
        match self {
            Self::Authored(handle) => handle
                .with_mut(|object| object.drawable_add_to_render_path(path, transform))
                .unwrap_or(false),
            Self::RuntimeProxy(_) => false,
        }
    }

    pub fn add_to_raw_path(
        &self,
        path: &mut crate::mechanical_port::source::math::raw_path::RawPath,
        transform: Option<&Mat2D>,
    ) -> bool {
        match self {
            Self::Authored(handle) => handle
                .with_mut(|object| object.drawable_add_to_raw_path(path, transform))
                .unwrap_or(false),
            Self::RuntimeProxy(_) => false,
        }
    }

    pub fn hit_test(&self, info: &mut HitInfo, transform: &Mat2D) -> Option<CoreHandle> {
        match self {
            Self::Authored(handle) => handle
                .with_mut(|object| object.drawable_hit_test(info, transform))
                .flatten(),
            Self::RuntimeProxy(_) => None,
        }
    }

    pub fn downgrade(&self) -> RuntimeDrawableWeakOccurrence {
        match self {
            Self::Authored(handle) => RuntimeDrawableWeakOccurrence::Authored(handle.clone()),
            Self::RuntimeProxy(proxy) => {
                RuntimeDrawableWeakOccurrence::RuntimeProxy(Rc::downgrade(proxy))
            }
        }
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Authored(a), Self::Authored(b)) => a == b,
            (Self::RuntimeProxy(a), Self::RuntimeProxy(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }

    pub fn with<R>(&self, use_drawable: impl FnOnce(&Drawable) -> R) -> Option<R> {
        match self {
            Self::Authored(handle) => {
                handle.with(|object| object.as_drawable().map(use_drawable))?
            }
            Self::RuntimeProxy(proxy) => Some(use_drawable(&proxy.borrow().base)),
        }
    }

    pub fn with_mut<R>(&self, use_drawable: impl FnOnce(&mut Drawable) -> R) -> Option<R> {
        match self {
            Self::Authored(handle) => {
                handle.with_mut(|object| object.as_drawable_mut().map(use_drawable))?
            }
            Self::RuntimeProxy(proxy) => Some(use_drawable(&mut proxy.borrow_mut().base)),
        }
    }

    pub fn with_proxy<R>(&self, use_proxy: impl FnOnce(&DrawableProxy) -> R) -> Option<R> {
        match self {
            Self::RuntimeProxy(proxy) => Some(use_proxy(&proxy.borrow())),
            Self::Authored(_) => None,
        }
    }

    pub fn with_proxy_mut<R>(&self, use_proxy: impl FnOnce(&mut DrawableProxy) -> R) -> Option<R> {
        match self {
            Self::RuntimeProxy(proxy) => Some(use_proxy(&mut proxy.borrow_mut())),
            Self::Authored(_) => None,
        }
    }

    pub fn is_clip_start(&self) -> bool {
        match self {
            Self::Authored(handle) => handle
                .with(|object| object.drawable_is_clip_start())
                .unwrap_or(false),
            Self::RuntimeProxy(proxy) => proxy.borrow().is_clip_start(),
        }
    }

    pub fn is_clip_end(&self) -> bool {
        match self {
            Self::Authored(handle) => handle
                .with(|object| object.drawable_is_clip_end())
                .unwrap_or(false),
            Self::RuntimeProxy(proxy) => proxy.borrow().is_clip_end(),
        }
    }

    pub fn will_clip(&self) -> bool {
        match self {
            Self::Authored(handle) => handle
                .with(|object| object.drawable_will_clip())
                .unwrap_or(false),
            Self::RuntimeProxy(proxy) => proxy.borrow().will_clip(),
        }
    }

    pub fn empty_clip_count(&self) -> i32 {
        match self {
            Self::Authored(handle) => handle
                .with_mut(|object| object.drawable_empty_clip_count())
                .unwrap_or_default(),
            Self::RuntimeProxy(proxy) => proxy.borrow_mut().empty_clip_count(),
        }
    }
}

impl PartialEq for RuntimeDrawableOccurrence {
    fn eq(&self, other: &Self) -> bool {
        self.ptr_eq(other)
    }
}
impl Eq for RuntimeDrawableOccurrence {}
impl std::hash::Hash for RuntimeDrawableOccurrence {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use std::hash::Hash;
        match self {
            Self::Authored(handle) => {
                0u8.hash(state);
                handle.hash(state);
            }
            Self::RuntimeProxy(proxy) => {
                1u8.hash(state);
                Rc::as_ptr(proxy).hash(state);
            }
        }
    }
}

impl RuntimeDrawableWeakOccurrence {
    pub fn upgrade(&self) -> Option<RuntimeDrawableOccurrence> {
        match self {
            Self::Authored(handle) => handle
                .is_alive()
                .then(|| RuntimeDrawableOccurrence::Authored(handle.clone())),
            Self::RuntimeProxy(proxy) => {
                proxy.upgrade().map(RuntimeDrawableOccurrence::RuntimeProxy)
            }
        }
    }
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

    pub fn draw(&mut self, renderer: &mut Renderer) {
        self.proxy_drawing
            .draw_proxy(renderer, self.base.needs_save_operation());
    }
    pub fn is_hidden(&self) -> bool {
        self.proxy_drawing.is_proxy_hidden()
    }
    pub fn hittable_component(&self) -> CoreHandle {
        self.proxy_drawing.owner_handle()
    }
    pub fn is_target_opaque(&mut self) -> bool {
        self.hittable_component()
            .with(|hittable| {
                hittable
                    .as_drawable()
                    .is_some_and(Drawable::is_target_opaque)
            })
            .unwrap_or(false)
    }
    pub fn hit_test(&mut self, _info: &mut HitInfo, _transform: &Mat2D) -> Option<CoreHandle> {
        None
    }
    pub fn is_proxy(&self) -> bool {
        true
    }
    pub fn empty_clip_count(&mut self) -> i32 {
        self.proxy_drawing.empty_clip_count()
    }
    pub fn is_clip_start(&self) -> bool {
        self.proxy_drawing.is_clip_start()
    }
    pub fn is_clip_end(&self) -> bool {
        self.proxy_drawing.is_clip_end()
    }
    pub fn will_clip(&self) -> bool {
        self.proxy_drawing.will_clip()
    }
    pub fn proxy_drawing(&self) -> &dyn ProxyDrawing {
        &*self.proxy_drawing
    }
}

impl std::ops::Deref for Drawable {
    type Target = DrawableBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for Drawable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl std::ops::Deref for DrawableProxy {
    type Target = Drawable;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for DrawableProxy {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
use std::{
    cell::{Ref, RefCell, RefMut},
    rc::{Rc, Weak},
};
