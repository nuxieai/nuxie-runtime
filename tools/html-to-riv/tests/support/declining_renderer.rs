use nuxie_render_api::*;

/// Stateful backend that accepts some exclusions and then declines. Every
/// subsequent draw checks that the caller unwound any partially installed clip.
pub struct DecliningRenderer<'a> {
    pub inner: &'a mut dyn Renderer,
    pub remaining: usize,
    pub declined: bool,
    active: usize,
    stack: Vec<usize>,
}
impl<'a> DecliningRenderer<'a> {
    pub fn new(inner: &'a mut dyn Renderer, remaining: usize) -> Self {
        Self {
            inner,
            remaining,
            declined: false,
            active: 0,
            stack: Vec::new(),
        }
    }
}
impl Renderer for DecliningRenderer<'_> {
    fn save(&mut self) {
        self.stack.push(self.active);
        self.inner.save();
    }
    fn restore(&mut self) {
        self.active = self.stack.pop().expect("balanced renderer restore");
        self.inner.restore();
    }
    fn transform(&mut self, m: Mat2D) {
        self.inner.transform(m);
    }
    fn draw_path(&mut self, p: &dyn RenderPath, c: &dyn RenderPaint) {
        if self.declined {
            assert_eq!(
                self.active, 0,
                "partial hard clips leaked into fallback drawing"
            );
        }
        self.inner.draw_path(p, c);
    }
    fn clip_path(&mut self, p: &dyn RenderPath) {
        self.inner.clip_path(p);
    }
    fn clip_out_rect(&mut self, rect: Aabb) -> bool {
        if self.remaining == 0 {
            self.declined = true;
            return false;
        }
        self.remaining -= 1;
        assert!(self.inner.clip_out_rect(rect));
        self.active += 1;
        true
    }
    fn draw_image(&mut self, i: Option<&dyn RenderImage>, s: ImageSampler, b: BlendMode, o: f32) {
        self.inner.draw_image(i, s, b, o);
    }
    fn draw_image_mesh(
        &mut self,
        i: Option<&dyn RenderImage>,
        s: ImageSampler,
        v: Option<&dyn RenderBuffer>,
        u: Option<&dyn RenderBuffer>,
        indices: Option<&dyn RenderBuffer>,
        vc: u32,
        ic: u32,
        b: BlendMode,
        o: f32,
    ) {
        self.inner
            .draw_image_mesh(i, s, v, u, indices, vc, ic, b, o);
    }
    fn modulate_opacity(&mut self, o: f32) {
        self.inner.modulate_opacity(o);
    }
}
