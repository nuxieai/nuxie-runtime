//! Translation of offscreen_raster.{hpp,cpp}, upstream 8398db31.
use super::math::{aabb::Aabb, mat2d::Mat2D, vec2d::Vec2D};
use nuxie_render_api::{DeferredCanvasHostHandle, RenderCanvasHandle, Renderer};

pub const K_MAX_DIM: u32 = 2048;
pub const K_SHRINK_DRAWS: u8 = 30;

#[derive(Clone, Copy, Debug)]
pub struct RasterPlan {
    pub ctm: Mat2D,
    pub have_ctm: bool,
    pub modulated_opacity: f32,
    pub have_opacity: bool,
    pub bounds: Aabb,
    pub raster_scale: f32,
    pub width_px: u32,
    pub height_px: u32,
}
impl Default for RasterPlan {
    fn default() -> Self {
        Self {
            ctm: Mat2D::default(),
            have_ctm: false,
            modulated_opacity: 1.0,
            have_opacity: false,
            bounds: Aabb::default(),
            raster_scale: 1.0,
            width_px: 0,
            height_px: 0,
        }
    }
}
impl RasterPlan {
    pub fn valid(&self) -> bool {
        self.width_px != 0 && self.height_px != 0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RasterFit {
    Exact,
    StableGrid,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RasterResize {
    pub reallocate: bool,
    pub width_px: u32,
    pub height_px: u32,
}

// std::min/max preserve their first operand on unordered comparison. Rust's
// f32::min/max instead discard NaN, so spell out the source comparison.
fn cpp_min(a: f32, b: f32) -> f32 {
    if b < a { b } else { a }
}
fn cpp_max(a: f32, b: f32) -> f32 {
    if a < b { b } else { a }
}

pub fn plan_raster_scale(renderer: &dyn Renderer, resolution: f32, out: &mut RasterPlan) -> bool {
    let ctm = renderer.current_transform();
    out.have_ctm = ctm.is_some();
    if let Some(ctm) = ctm {
        let m = ctm.0;
        out.ctm = Mat2D::new(m[0], m[1], m[2], m[3], m[4], m[5]);
    }
    let opacity = renderer.current_modulated_opacity();
    out.have_opacity = opacity.is_some();
    if let Some(opacity) = opacity {
        out.modulated_opacity = opacity;
    }
    let mut device_scale = 1.0;
    if out.have_ctm {
        let s = out.ctm.find_max_scale();
        if s.is_finite() && s > 0.0 {
            device_scale = (s * 16.0).ceil() / 16.0;
        }
    }
    let res = cpp_min(cpp_max(resolution, 0.01), 8.0);
    let scale = res * device_scale;
    if !(scale > 0.0) || !scale.is_finite() {
        return false;
    }
    out.raster_scale = scale;
    true
}
pub fn bucketed_raster_dim(n: u32) -> u32 {
    if n >= K_MAX_DIM {
        return K_MAX_DIM;
    }
    (((n + 63) / 64) * 64).min(K_MAX_DIM)
}
pub fn decide_raster_resize(
    need_w: u32,
    need_h: u32,
    have_w: u32,
    have_h: u32,
    streak: &mut u8,
) -> RasterResize {
    let want_w = bucketed_raster_dim(need_w);
    let want_h = bucketed_raster_dim(need_h);
    if have_w == 0 || have_h == 0 {
        *streak = 0;
        return RasterResize {
            reallocate: true,
            width_px: want_w,
            height_px: want_h,
        };
    }
    if need_w > have_w || need_h > have_h {
        *streak = 0;
        return RasterResize {
            reallocate: true,
            width_px: if need_w > have_w { want_w } else { have_w },
            height_px: if need_h > have_h { want_h } else { have_h },
        };
    }
    if want_w < have_w || want_h < have_h {
        *streak = streak.wrapping_add(1);
        if *streak < K_SHRINK_DRAWS {
            return RasterResize {
                reallocate: false,
                width_px: have_w,
                height_px: have_h,
            };
        }
        *streak = 0;
        return RasterResize {
            reallocate: true,
            width_px: want_w,
            height_px: want_h,
        };
    }
    *streak = 0;
    RasterResize {
        reallocate: false,
        width_px: have_w,
        height_px: have_h,
    }
}
pub fn fit_raster_to_box(
    bounds: &Aabb,
    guard_texels: u32,
    fit: RasterFit,
    io: &mut RasterPlan,
) -> bool {
    debug_assert!(fit == RasterFit::StableGrid || guard_texels == 0);
    let w = bounds.width();
    let h = bounds.height();
    if !(w > 0.0) || !(h > 0.0) {
        return false;
    }
    let reserve = if fit == RasterFit::StableGrid {
        guard_texels.wrapping_mul(2).wrapping_add(2)
    } else {
        0
    };
    let usable = if K_MAX_DIM > reserve {
        K_MAX_DIM - reserve
    } else {
        1
    } as f32;
    let scale = cpp_min(io.raster_scale, cpp_min(usable / w, usable / h));
    if !(scale > 0.0) || !scale.is_finite() {
        return false;
    }
    io.raster_scale = scale;
    if fit == RasterFit::Exact {
        io.bounds = *bounds;
        io.width_px = cpp_min(cpp_max((w * scale).ceil(), 1.0), K_MAX_DIM as f32) as u32;
        io.height_px = cpp_min(cpp_max((h * scale).ceil(), 1.0), K_MAX_DIM as f32) as u32;
        return true;
    }
    let guard = guard_texels as f32;
    let left = (bounds.left() * scale).floor() - guard;
    let top = (bounds.top() * scale).floor() - guard;
    let right = (bounds.right() * scale).ceil() + guard;
    let bottom = (bounds.bottom() * scale).ceil() + guard;
    if !left.is_finite() || !top.is_finite() || !right.is_finite() || !bottom.is_finite() {
        return false;
    }
    let width = bucketed_raster_dim(cpp_min(cpp_max(right - left, 1.0), K_MAX_DIM as f32) as u32);
    let height = bucketed_raster_dim(cpp_min(cpp_max(bottom - top, 1.0), K_MAX_DIM as f32) as u32);
    io.width_px = width;
    io.height_px = height;
    io.bounds = Aabb::new(
        left / scale,
        top / scale,
        (left + width as f32) / scale,
        (top + height as f32) / scale,
    );
    true
}
pub fn hold_raster_allocation(width: u32, height: u32, io: &mut RasterPlan) {
    if width <= io.width_px && height <= io.height_px {
        return;
    }
    let w = width.max(io.width_px);
    let h = height.max(io.height_px);
    io.bounds = Aabb::new(
        io.bounds.left(),
        io.bounds.top(),
        io.bounds.left() + w as f32 / io.raster_scale,
        io.bounds.top() + h as f32 / io.raster_scale,
    );
    io.width_px = w;
    io.height_px = h;
}
pub fn plan_raster(
    renderer: &dyn Renderer,
    bounds: &Aabb,
    resolution: f32,
    out: &mut RasterPlan,
) -> bool {
    let mut plan = RasterPlan::default();
    if !plan_raster_scale(renderer, resolution, &mut plan)
        || !fit_raster_to_box(bounds, 0, RasterFit::Exact, &mut plan)
    {
        return false;
    }
    *out = plan;
    true
}

pub struct CanvasContentScope {
    host: Option<DeferredCanvasHostHandle>,
    canvas: Option<RenderCanvasHandle>,
    renderer: Option<Box<dyn Renderer>>,
}
impl CanvasContentScope {
    pub fn new(
        host: Option<&DeferredCanvasHostHandle>,
        canvas: Option<&RenderCanvasHandle>,
        plan: &RasterPlan,
        clear_color: u32,
    ) -> Self {
        let mut scope = Self {
            host: host.cloned(),
            canvas: canvas.cloned(),
            renderer: None,
        };
        if let (Some(host), Some(canvas)) = (host, canvas) {
            scope.renderer = host
                .borrow_mut()
                .begin_canvas_content(canvas.clone(), clear_color);
            if let Some(renderer) = scope.renderer.as_mut() {
                renderer.save();
                renderer.transform(nuxie_render_api::Mat2D([
                    plan.raster_scale,
                    0.0,
                    0.0,
                    plan.raster_scale,
                    0.0,
                    0.0,
                ]));
                renderer.translate(-plan.bounds.left(), -plan.bounds.top());
            }
        }
        scope
    }
    pub fn renderer(&mut self) -> Option<&mut (dyn Renderer + 'static)> {
        self.renderer.as_deref_mut()
    }
}
impl Drop for CanvasContentScope {
    fn drop(&mut self) {
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.restore();
            self.host
                .as_ref()
                .unwrap()
                .borrow_mut()
                .end_canvas_content(self.canvas.as_ref().unwrap());
        }
    }
}

enum CompositeRenderer<'a> {
    Original(&'a mut dyn Renderer),
    Fresh(Box<dyn Renderer>),
}
pub struct CompositePlacement<'a> {
    renderer: CompositeRenderer<'a>,
    pub opacity: f32,
}
impl CompositePlacement<'_> {
    pub fn renderer(&mut self) -> &mut dyn Renderer {
        match &mut self.renderer {
            CompositeRenderer::Original(r) => *r,
            CompositeRenderer::Fresh(r) => r.as_mut(),
        }
    }
}
pub fn begin_composite<'a>(
    renderer: &'a mut dyn Renderer,
    host: Option<&DeferredCanvasHostHandle>,
    plan: &RasterPlan,
    raster_scale: f32,
) -> CompositePlacement<'a> {
    begin_composite_for_box(renderer, host, plan, raster_scale, &plan.bounds)
}
pub fn begin_composite_for_box<'a>(
    renderer: &'a mut dyn Renderer,
    host: Option<&DeferredCanvasHostHandle>,
    plan: &RasterPlan,
    raster_scale: f32,
    bounds: &Aabb,
) -> CompositePlacement<'a> {
    let fresh = host.and_then(|host| host.borrow_mut().composite_renderer());
    let mut placement = if let Some(fresh) = fresh.filter(|_| plan.have_ctm && plan.have_opacity) {
        CompositePlacement {
            renderer: CompositeRenderer::Fresh(fresh),
            opacity: plan.modulated_opacity,
        }
    } else {
        CompositePlacement {
            renderer: CompositeRenderer::Original(renderer),
            opacity: 1.0,
        }
    };
    let is_fresh = matches!(placement.renderer, CompositeRenderer::Fresh(_));
    let r = placement.renderer();
    r.save();
    if is_fresh {
        r.transform(nuxie_render_api::Mat2D(*plan.ctm.values()));
    }
    if plan.have_ctm
        && plan.ctm.xy() == 0.0
        && plan.ctm.yx() == 0.0
        && plan.ctm.xx() != 0.0
        && plan.ctm.yy() != 0.0
    {
        let origin = plan.ctm * Vec2D::new(bounds.left(), bounds.top());
        r.translate(
            (origin.x.round() - origin.x) / plan.ctm.xx(),
            (origin.y.round() - origin.y) / plan.ctm.yy(),
        );
    }
    r.translate(bounds.left(), bounds.top());
    let inv = 1.0 / raster_scale;
    r.transform(nuxie_render_api::Mat2D([inv, 0.0, 0.0, inv, 0.0, 0.0]));
    placement
}
pub fn end_composite(mut placement: CompositePlacement<'_>) {
    placement.renderer().restore();
}
