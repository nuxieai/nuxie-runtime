//! Source-paired translation of rive/layer_mask.hpp and src/layer_mask.cpp, 8398db31.
use crate::mechanical_port::source::{
    artboard::Artboard,
    component_dirt::ComponentDirt,
    container_component::ContainerComponent,
    core::CoreHandle,
    core_context::CoreContext,
    drawable::{
        DrawableProxy, ProxyDrawing, RuntimeDrawableOccurrence, RuntimeDrawableWeakOccurrence,
    },
    generated::{
        component_base::ComponentBaseCallbacks,
        layer_mask_base::{LayerMaskBase, LayerMaskBaseCallbacks},
        node_base::NodeBase,
    },
    math::aabb::Aabb,
    renderer::Renderer,
    status_code::StatusCode,
};
use nuxie_render_api::RenderCanvasHandle;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaskMode {
    Alpha = 0,
    InvertedAlpha = 1,
    Luminance = 2,
    InvertedLuminance = 3,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerMaskOp {
    MaskStart,
    MaskEnd,
    SourceStart,
    SourceEnd,
}

pub struct LayerMaskProxyDrawable {
    pub mask: CoreHandle,
    pub op: LayerMaskOp,
    pub paired_end: Option<RuntimeDrawableWeakOccurrence>,
}
struct MaskDrawing(Rc<RefCell<LayerMaskProxyDrawable>>);
impl ProxyDrawing for MaskDrawing {
    fn hittable_component(&self) -> Option<CoreHandle> { None }
    fn draw_proxy(&mut self, _: &mut Renderer, _: bool) {}
    fn is_proxy_hidden(&self) -> bool {
        false
    }
    fn owner_handle(&self) -> CoreHandle {
        self.0.borrow().mask.clone()
    }
    fn layer_mask_marker(&self) -> Option<Rc<RefCell<LayerMaskProxyDrawable>>> {
        Some(self.0.clone())
    }
    fn empty_clip_count(&mut self) -> i32 {
        let marker = self.0.borrow();
        if !matches!(marker.op, LayerMaskOp::SourceStart | LayerMaskOp::SourceEnd) {
            return 0;
        }
        marker
            .mask
            .with_downcast::<LayerMask, _>(|mask| {
                if mask.source_draws() || !mask.is_visible() || mask.is_self_referential {
                    0
                } else if marker.op == LayerMaskOp::SourceStart {
                    1
                } else {
                    -1
                }
            })
            .unwrap_or(0)
    }
}
struct MaskProxy {
    drawable: Rc<RefCell<DrawableProxy>>,
    marker: Rc<RefCell<LayerMaskProxyDrawable>>,
}

pub struct LayerMask {
    pub base: LayerMaskBase,
    source: Option<CoreHandle>,
    source_drawables: Vec<CoreHandle>,
    proxy_drawables: Vec<MaskProxy>,
    pooled_proxy_drawables: Vec<MaskProxy>,
    pub(crate) source_start: Option<RuntimeDrawableOccurrence>,
    pub(crate) source_end: Option<RuntimeDrawableOccurrence>,
    pub(crate) is_self_referential: bool,
    pub(crate) is_drawing: bool,
    pub(crate) content_canvas: Option<RenderCanvasHandle>,
    pub(crate) mask_canvas: Option<RenderCanvasHandle>,
    pub(crate) width_px: u32,
    pub(crate) height_px: u32,
    pub(crate) raster_scale: f32,
    pub(crate) dirty: bool,
    pub(crate) tight_box: Aabb,
    pub(crate) raster_box: Aabb,
    pub(crate) shrink_streak: u8,
    #[cfg(any(test, feature = "testing"))]
    pub test_allocations: i32,
}
impl Default for LayerMask {
    fn default() -> Self {
        Self {
            base: LayerMaskBase::default(),
            source: None,
            source_drawables: Vec::new(),
            proxy_drawables: Vec::new(),
            pooled_proxy_drawables: Vec::new(),
            source_start: None,
            source_end: None,
            is_self_referential: false,
            is_drawing: false,
            content_canvas: None,
            mask_canvas: None,
            width_px: 0,
            height_px: 0,
            raster_scale: 1.0,
            dirty: true,
            tight_box: Aabb::default(),
            raster_box: Aabb::default(),
            shrink_streak: 0,
            #[cfg(any(test, feature = "testing"))]
            test_allocations: 0,
        }
    }
}
impl LayerMask {
    pub const TYPE_KEY: u16 = LayerMaskBase::TYPE_KEY;
    pub fn mask_mode(&self) -> MaskMode {
        match self.base.mask_mode_value() {
            0 => MaskMode::Alpha,
            1 => MaskMode::InvertedAlpha,
            2 => MaskMode::Luminance,
            3 => MaskMode::InvertedLuminance,
            // Unknown bound modes fall back to alpha, as in the editor.
            _ => MaskMode::Alpha,
        }
    }
    pub fn source(&self) -> Option<CoreHandle> {
        self.source.clone()
    }
    pub fn custom_bounds(&self) -> Aabb {
        Aabb::from_ltwh(
            self.bounds_x(),
            self.bounds_y(),
            self.bounds_width(),
            self.bounds_height(),
        )
    }
    pub fn source_drawables(&self) -> &[CoreHandle] {
        &self.source_drawables
    }
    pub fn is_self_referential(&self) -> bool {
        self.is_self_referential
    }
    pub fn source_start(&self) -> Option<RuntimeDrawableOccurrence> {
        self.source_start.clone()
    }
    pub fn source_end(&self) -> Option<RuntimeDrawableOccurrence> {
        self.source_end.clone()
    }
    pub fn source_bracket(
        &mut self,
        start: RuntimeDrawableOccurrence,
        end: RuntimeDrawableOccurrence,
    ) {
        self.source_start = Some(start);
        self.source_end = Some(end);
    }
    pub fn encloses(container: Option<&CoreHandle>, inner: Option<&CoreHandle>) -> bool {
        let Some(container) = container else {
            return false;
        };
        let mut current = inner.cloned();
        while let Some(c) = current {
            if &c == container {
                return true;
            }
            current = c.with(|o| o.component_parent_handle()).flatten();
        }
        false
    }
    pub fn subtrees_overlap(source: Option<&CoreHandle>, masked: Option<&CoreHandle>) -> bool {
        Self::encloses(source, masked) || Self::encloses(masked, source)
    }
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        self.source = context
            .resolve_handle(self.source_id())
            .filter(|h| h.is_type_of(NodeBase::TYPE_KEY));
        if self.source.is_some() {
            StatusCode::Ok
        } else {
            StatusCode::MissingObject
        }
    }
    pub fn on_added_clean(&mut self, _: &mut dyn CoreContext) -> StatusCode {
        let parent = self.parent_handle();
        self.is_self_referential = Self::subtrees_overlap(self.source.as_ref(), parent.as_ref());
        if self.source.is_none() || self.is_self_referential {
            return StatusCode::Ok;
        }
        let this = self.handle().expect("live LayerMask");
        if let Some(parent) = parent {
            ContainerComponent::for_all(&parent, |h| {
                if h == this {
                    return true;
                }
                h.with_mut(|o| {
                    if let Some(d) = o.as_drawable_mut() {
                        d.add_layer_mask(this.clone());
                    }
                });
                true
            });
        }
        ContainerComponent::for_all(self.source.as_ref().unwrap(), |h| {
            let drawable = h
                .with_mut(|o| {
                    if let Some(d) = o.as_drawable_mut() {
                        d.add_source_of_layer_mask(this.clone());
                        true
                    } else {
                        false
                    }
                })
                .unwrap_or(false);
            if drawable {
                self.source_drawables.push(h);
            }
            true
        });
        StatusCode::Ok
    }
    pub fn build_dependencies(&mut self) {
        if self.is_self_referential {
            return;
        }
        self.depend_on_subtree(self.parent_handle());
        self.depend_on_subtree(self.source.clone());
    }
    fn depend_on_subtree(&self, root: Option<CoreHandle>) {
        let Some(root) = root else {
            return;
        };
        let this = self.handle().expect("live LayerMask");
        ContainerComponent::for_all(&root, |h| {
            if h != this && !h.is_type_of(Self::TYPE_KEY) {
                h.with_mut(|o| {
                    if let Some(c) = o.as_component_mut() {
                        c.add_dependent(this.clone());
                    }
                });
            }
            true
        });
    }
    pub fn update(&mut self, _: ComponentDirt) {
        self.invalidate();
    }
    pub fn mark_raster_dirty(&mut self) {
        self.invalidate();
    }
    fn invalidate(&mut self) {
        self.dirty = true;
        self.shrink_streak = 0;
        if let Some(owner) = self.artboard_handle() {
            let ours = self.parent_handle();
            let this = self.handle();
            let masks = owner
                .with_downcast::<Artboard, _>(|a| a.all_layer_masks().to_vec())
                .unwrap_or_default();
            for other in masks {
                if Some(&other) == this.as_ref() {
                    continue;
                }
                other.with_downcast_mut::<LayerMask, _>(|m| {
                    if !m.dirty
                        && (Self::encloses(m.parent_handle().as_ref(), ours.as_ref())
                            || Self::encloses(m.source.as_ref(), ours.as_ref()))
                    {
                        m.dirty = true;
                        m.shrink_streak = 0;
                    }
                });
            }
            if let Some(dirty) = owner.artboard_dirty_handle() {
                dirty.changed();
            }
        }
    }
    pub(crate) fn release_canvases(&mut self) {
        self.content_canvas = None;
        self.mask_canvas = None;
        self.width_px = 0;
        self.height_px = 0;
        self.tight_box = Aabb::default();
        self.raster_box = Aabb::default();
        self.shrink_streak = 0;
    }
    pub fn resolution_changed(&mut self) {
        self.invalidate();
    }
    pub fn bounds_x_changed(&mut self) {
        self.invalidate();
    }
    pub fn bounds_y_changed(&mut self) {
        self.invalidate();
    }
    pub fn bounds_width_changed(&mut self) {
        self.invalidate();
    }
    pub fn bounds_height_changed(&mut self) {
        self.invalidate();
    }
    pub fn mask_flags_changed(&mut self) {
        self.invalidate();
        if !self.is_visible() {
            self.release_canvases();
        }
    }
    pub fn reset_drawables(&mut self) {
        self.pooled_proxy_drawables
            .append(&mut self.proxy_drawables);
        self.source_start = None;
        self.source_end = None;
    }
    pub fn create_proxy_drawable(&mut self, op: LayerMaskOp) -> RuntimeDrawableOccurrence {
        let owner = self.handle().expect("live LayerMask");
        let proxy = if let Some(proxy) = self.pooled_proxy_drawables.pop() {
            {
                let mut m = proxy.marker.borrow_mut();
                m.mask = owner;
                m.op = op;
                m.paired_end = None;
            }
            proxy.drawable.borrow_mut().set_needs_save_operation(true);
            proxy
        } else {
            let marker = Rc::new(RefCell::new(LayerMaskProxyDrawable {
                mask: owner,
                op,
                paired_end: None,
            }));
            let drawable = Rc::new(RefCell::new(DrawableProxy::new(Box::new(MaskDrawing(
                marker.clone(),
            )))));
            MaskProxy { drawable, marker }
        };
        let result = RuntimeDrawableOccurrence::runtime_proxy(proxy.drawable.clone());
        self.proxy_drawables.push(proxy);
        result
    }
    #[cfg(any(test, feature = "testing"))]
    pub fn test_width_px(&self) -> u32 {
        self.width_px
    }
    #[cfg(any(test, feature = "testing"))]
    pub fn test_height_px(&self) -> u32 {
        self.height_px
    }
    #[cfg(any(test, feature = "testing"))]
    pub fn test_raster_scale(&self) -> f32 {
        self.raster_scale
    }
    #[cfg(any(test, feature = "testing"))]
    pub fn test_dirty(&self) -> bool {
        self.dirty
    }
    #[cfg(any(test, feature = "testing"))]
    pub fn test_box(&self) -> Aabb {
        self.raster_box
    }
}
impl LayerMaskBaseCallbacks for LayerMask {
    fn notify_property_changed(&mut self, key: u16) {
        self.base.base.base.base.notify_property_changed(key);
    }
    fn mask_flags_changed(&mut self) {
        LayerMask::mask_flags_changed(self);
    }
    fn resolution_changed(&mut self) {
        LayerMask::resolution_changed(self);
    }
    fn bounds_x_changed(&mut self) {
        LayerMask::bounds_x_changed(self);
    }
    fn bounds_y_changed(&mut self) {
        LayerMask::bounds_y_changed(self);
    }
    fn bounds_width_changed(&mut self) {
        LayerMask::bounds_width_changed(self);
    }
    fn bounds_height_changed(&mut self) {
        LayerMask::bounds_height_changed(self);
    }
}
impl ComponentBaseCallbacks for LayerMask {
    fn notify_property_changed(&mut self, key: u16) {
        self.base.base.base.base.notify_property_changed(key);
    }
}
impl std::ops::Deref for LayerMask {
    type Target = LayerMaskBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for LayerMask {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
