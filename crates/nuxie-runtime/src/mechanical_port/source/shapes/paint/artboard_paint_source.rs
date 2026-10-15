//! Mechanical translation of artboard_paint_source.{hpp,cpp}.
use crate::source::{
    advance_flags::AdvanceFlags,
    animation::{
        linear_animation_instance::LinearAnimationInstance,
        state_machine_instance::RuntimeStateMachineInstanceHandle,
    },
    artboard::{Artboard, RuntimeArtboardInstanceHandle},
    component::ComponentDirt,
    core::CoreHandle,
    data_bind::data_context::RuntimeDataContextHandle,
    factory::RuntimeFactoryHandle,
    math::{aabb::Aabb, mat2d::Mat2D},
    offscreen_raster,
    shapes::paint::image_sampler::ImageSampler,
};
use nuxie_render_api::{DeferredCanvasHostHandle, RenderCanvasHandle, RenderPaint, Renderer};
use std::cell::Cell;

thread_local! { static NESTING: Cell<u32> = const { Cell::new(0) }; }
pub struct ModulatedDrawTarget {
    /// None means the original renderer. A fresh renderer owns its scoped host proxy.
    pub renderer: Option<Box<dyn Renderer>>,
    pub transform: Mat2D,
    pub opacity: f32,
}
impl Default for ModulatedDrawTarget {
    fn default() -> Self {
        Self {
            renderer: None,
            transform: Mat2D::default(),
            opacity: 1.0,
        }
    }
}
pub struct ArtboardPaintSource {
    // Rust drops in declaration order: scenes must be destroyed before their artboard.
    canvas: Option<RenderCanvasHandle>,
    animation: Option<Box<LinearAnimationInstance>>,
    state_machine: Option<RuntimeStateMachineInstanceHandle>,
    artboard: RuntimeArtboardInstanceHandle,
    width_px: u32,
    height_px: u32,
    shrink_streak: u8,
    dirty: bool,
    raster_count: u32,
}
impl ArtboardPaintSource {
    pub fn make(
        source: Option<&CoreHandle>,
        factory: Option<RuntimeFactoryHandle>,
        host: Option<CoreHandle>,
    ) -> Option<Self> {
        let source = source?;
        if NESTING.with(|n| n.get() >= 16) {
            return None;
        }
        struct NestingGuard;
        impl Drop for NestingGuard {
            fn drop(&mut self) {
                NESTING.with(|n| n.set(n.get() - 1));
            }
        }
        NESTING.with(|n| n.set(n.get() + 1));
        let guard = NestingGuard;
        let instance = Artboard::instance_from_handle_with_factory(source, factory);
        drop(guard);
        let artboard = instance?;
        if let Some(host) = host {
            let parent = host
                .with(|h| h.as_artboard_host().and_then(|h| h.parent_artboard()))
                .flatten();
            Artboard::set_host_occurrence(&artboard.core_handle(), Some(host), parent);
        }
        let state_machine = artboard
            .default_state_machine_handle()
            .or_else(|| artboard.state_machine_instance_handle(0));
        let animation = if state_machine.is_none() {
            artboard.animation_at(0)
        } else {
            None
        };
        let mut result = Self {
            state_machine,
            animation,
            canvas: None,
            artboard,
            width_px: 0,
            height_px: 0,
            shrink_streak: 0,
            dirty: true,
            raster_count: 0,
        };
        result.advance(
            0.0,
            AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
        );
        result.update();
        Some(result)
    }
    pub fn artboard(&self) -> RuntimeArtboardInstanceHandle {
        self.artboard.clone()
    }
    pub fn advance(&mut self, seconds: f32, flags: AdvanceFlags) -> bool {
        let mut keep = false;
        let new_frame = flags.contains(AdvanceFlags::NEW_FRAME);
        if flags.contains(AdvanceFlags::ADVANCE_NESTED) {
            if let Some(machine) = &self.state_machine {
                machine.with_instance_mut(|machine| {
                    if new_frame || machine.try_change_state() {
                        keep = machine.advance(seconds, new_frame);
                    }
                });
            } else if let Some(animation) = &mut self.animation {
                if new_frame {
                    keep = animation.advance(seconds, None);
                    animation.apply(1.0);
                }
            }
        }
        if self
            .artboard
            .advance_internal(seconds, flags & !AdvanceFlags::IS_ROOT)
        {
            keep = true;
        }
        keep
    }
    pub fn needs_update(&self) -> bool {
        self.artboard
            .with_artboard(|a| a.base.has_dirt(ComponentDirt::COMPONENTS))
    }
    pub fn update(&mut self) {
        self.artboard.update_pass(false);
    }
    pub fn bounds(&self) -> Aabb {
        self.artboard.with_artboard(|a| a.base.bounds())
    }
    pub fn data_context(&mut self, context: Option<RuntimeDataContextHandle>) {
        let Some(context) = context else {
            self.clear_data_context();
            return;
        };
        self.artboard.internal_data_context(context.clone());
        if let Some(machine) = &self.state_machine {
            machine.with_instance_mut(|m| m.set_data_context(context));
        }
    }
    pub fn relink_data_context(&mut self, instance: Option<CoreHandle>) {
        if let Some(context) = self.artboard.data_context() {
            if context.with_context(|c| c.main_view_model_instance()) != instance {
                context.set_view_model_instance(instance);
            }
        }
        self.artboard.relink_data_context();
    }
    pub fn clear_data_context(&mut self) {
        self.artboard.clear_data_context();
        if let Some(m) = &self.state_machine {
            m.with_instance_mut(|m| m.clear_data_context());
        }
    }
    pub fn unbind(&mut self) {
        self.artboard.unbind();
    }
    pub fn update_data_binds(&mut self) {
        self.artboard.update_data_binds(true);
    }
    fn rasterize(&mut self, host: &DeferredCanvasHostHandle) -> bool {
        let canvas = self.canvas.as_ref().unwrap().clone();
        let renderer = host.borrow_mut().begin_canvas_content(canvas.clone(), 0);
        let Some(mut renderer) = renderer else {
            self.canvas = None;
            self.width_px = 0;
            self.height_px = 0;
            self.dirty = true;
            return false;
        };
        let bounds = self.bounds();
        renderer.save();
        renderer.transform(nuxie_render_api::Mat2D(
            *Mat2D::from_scale(
                self.width_px as f32 / bounds.width(),
                self.height_px as f32 / bounds.height(),
            )
            .values(),
        ));
        renderer.transform(nuxie_render_api::Mat2D(
            *Mat2D::from_translate(-bounds.left(), -bounds.top()).values(),
        ));
        self.artboard.draw_content(renderer.as_mut());
        renderer.restore();
        drop(renderer);
        host.borrow_mut().end_canvas_content(&canvas);
        self.dirty = false;
        self.raster_count += 1;
        true
    }
    pub fn apply_to(
        &mut self,
        renderer: &mut dyn Renderer,
        host: Option<DeferredCanvasHostHandle>,
        paint: &mut dyn RenderPaint,
        sampler: ImageSampler,
        image_transform: Mat2D,
        target: &mut ModulatedDrawTarget,
    ) -> bool {
        target.renderer = None;
        let Some(host) = host else {
            return false;
        };
        let bounds = self.bounds();
        if !(bounds.width() > 0.0) || !(bounds.height() > 0.0) {
            return false;
        }
        let ctm = renderer.current_transform();
        let to_device = ctm
            .map(|m| Mat2D::new(m.0[0], m.0[1], m.0[2], m.0[3], m.0[4], m.0[5]) * image_transform)
            .unwrap_or(image_transform);
        let m = to_device.values();
        let mut w = texels_for((m[0] * m[0] + m[1] * m[1]).sqrt(), bounds.width());
        let mut h = texels_for((m[2] * m[2] + m[3] * m[3]).sqrt(), bounds.height());
        let cap = offscreen_raster::K_MAX_DIM as f32;
        let largest = w.max(h);
        if largest > cap {
            let shrink = cap / largest;
            w = (w * shrink).min(cap);
            h = (h * shrink).min(cap);
        }
        let resize = offscreen_raster::decide_raster_resize(
            (w as u32).max(1),
            (h as u32).max(1),
            if self.canvas.is_some() {
                self.width_px
            } else {
                0
            },
            if self.canvas.is_some() {
                self.height_px
            } else {
                0
            },
            &mut self.shrink_streak,
        );
        if resize.reallocate || self.canvas.is_none() {
            self.canvas = host
                .borrow_mut()
                .make_content_canvas(resize.width_px, resize.height_px);
            self.width_px = if self.canvas.is_some() {
                resize.width_px
            } else {
                0
            };
            self.height_px = if self.canvas.is_some() {
                resize.height_px
            } else {
                0
            };
            self.dirty = true;
        }
        if self.canvas.is_none() {
            return false;
        }
        if (self.dirty || self.artboard.with_artboard(|a| a.base.did_change()))
            && !self.rasterize(&host)
        {
            return false;
        }
        let Some(image) = host
            .borrow_mut()
            .content_canvas_image(self.canvas.as_ref().unwrap())
        else {
            return false;
        };
        if let Some(ctm) = ctm {
            if let Some(opacity) = renderer.current_modulated_opacity() {
                if let Some(fresh) = host.borrow_mut().composite_renderer() {
                    target.renderer = Some(fresh);
                    target.transform =
                        Mat2D::new(ctm.0[0], ctm.0[1], ctm.0[2], ctm.0[3], ctm.0[4], ctm.0[5]);
                    target.opacity = opacity;
                }
            }
        }
        paint.modulated_image(
            Some(image.as_ref()),
            sampler.into(),
            nuxie_render_api::Mat2D(*image_transform.values()),
        );
        true
    }
    pub fn raster_width(&self) -> u32 {
        if self.canvas.is_some() {
            self.width_px
        } else {
            0
        }
    }
    pub fn raster_height(&self) -> u32 {
        if self.canvas.is_some() {
            self.height_px
        } else {
            0
        }
    }
    pub fn raster_count(&self) -> u32 {
        self.raster_count
    }
    pub fn has_state_machine(&self) -> bool {
        self.state_machine.is_some()
    }
    pub fn has_animation(&self) -> bool {
        self.animation.is_some()
    }
}
fn texels_for(span: f32, extent: f32) -> f32 {
    if !(span > 0.0) || !span.is_finite() {
        return 1.0;
    }
    let per_unit = (span / extent * 16.0).ceil() / 16.0;
    let texels = (extent * per_unit).ceil();
    let texels = if 1.0 < texels { texels } else { 1.0 };
    if f32::MAX < texels { f32::MAX } else { texels }
}
