//! Explicit glyph renderer installation with a factory-owned bounded LRU.
use crate::glyph_rasterizer::rasterize;
use nuxie_render_api::*;
use std::{collections::VecDeque, sync::Arc};

#[derive(Default, Clone, Copy, Debug)]
pub struct GlyphCacheStats {
    pub hits: u64,
    pub misses: u64,
    pub entries: usize,
    /// Conservative retained pixel, font and key bytes, not GPU allocator size.
    pub retained_bytes: usize,
}
#[derive(PartialEq)]
struct Key {
    font: usize,
    size: u32,
    color: u32,
    matrix: [u32; 6],
    glyphs: Vec<[u32; 3]>,
}
struct Entry {
    key: Key,
    // Keeps the pointer identity alive, so allocation reuse cannot alias fonts.
    _font: Arc<[u8]>,
    image: Option<Box<dyn RenderImage>>,
    x: i32,
    y: i32,
    bytes: usize,
}

/// A cache belongs to exactly one image factory/device. Clone a
/// PersistentFactory into it when the scene also needs that factory. Retain the
/// cache across frames and wrap each fresh renderer before any transforms.
/// Logical retention is limited to 32 MiB and 256 entries, including font bytes
/// conservatively counted for every entry. A single mask is limited to 16 MiB
/// by the rasterizer. Eviction releases both images and retained font sources.
pub struct GlyphCache<F: Factory> {
    factory: F,
    entries: VecDeque<Entry>,
    stats: GlyphCacheStats,
}
impl<F: Factory> GlyphCache<F> {
    pub fn new(factory: F) -> Self {
        Self {
            factory,
            entries: VecDeque::new(),
            stats: GlyphCacheStats::default(),
        }
    }
    pub fn stats(&self) -> GlyphCacheStats {
        self.stats
    }
    pub fn clear(&mut self) {
        self.entries.clear();
        self.stats.entries = 0;
        self.stats.retained_bytes = 0;
    }
    /// `inner` must use this cache's factory/device and start at identity CTM
    /// and unit modulated opacity (a fresh renderer frame).
    /// Route every state change through the returned wrapper. Its scoped image
    /// draw restores the original CTM and leaves the active clip intact.
    pub fn wrap<'a>(&'a mut self, inner: &'a mut dyn Renderer) -> GlyphRenderer<'a, F> {
        GlyphRenderer {
            cache: self,
            inner,
            transform: Mat2D::IDENTITY,
            opacity: 1.0,
            stack: Vec::new(),
            groups: Vec::new(),
        }
    }
    fn prepare(&mut self, run: &RenderGlyphRun<'_>, transform: Mat2D) -> Option<(usize, i32, i32)> {
        if run.font.face_index != 0
            || !run.font.variations.is_empty()
            || run.blend_mode != BlendMode::SrcOver
            || run.glyphs.len() > 16_384
            || !transform.0.iter().all(|v| v.is_finite() && v.abs() <= 1e7)
        {
            return None;
        }
        let mut matrix = transform.0;
        let ix = matrix[4].floor() as i32;
        let iy = matrix[5].floor() as i32;
        matrix[4] -= ix as f32;
        matrix[5] -= iy as f32;
        let key = Key {
            font: run.font.bytes.as_ptr() as usize,
            size: run.font_size.to_bits(),
            color: run.color,
            matrix: matrix.map(f32::to_bits),
            glyphs: run
                .glyphs
                .iter()
                .map(|g| [u32::from(g.id), g.x.to_bits(), g.y.to_bits()])
                .collect(),
        };
        if let Some(index) = self.entries.iter().position(|e| e.key == key) {
            let entry = self.entries.remove(index)?;
            self.entries.push_back(entry);
            self.stats.hits += 1;
            return Some((self.entries.len() - 1, ix, iy));
        }
        self.stats.misses += 1;
        let mask = rasterize(run, Mat2D(matrix))?;
        let bytes = mask
            .rgba
            .len()
            .checked_add(run.font.bytes.len())?
            .checked_add(key.glyphs.len() * 12)?;
        const LIMIT: usize = 32 * 1024 * 1024;
        if bytes > LIMIT {
            return None;
        }
        let image = if mask.rgba.is_empty() {
            None
        } else {
            let mut png = Vec::new();
            let mut encoder = png::Encoder::new(&mut png, mask.width, mask.height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .ok()?
                .write_image_data(&mask.rgba)
                .ok()?;
            Some(self.factory.decode_image(&png).ok()?)
        };
        while self.entries.len() >= 256 || self.stats.retained_bytes + bytes > LIMIT {
            self.stats.retained_bytes -= self.entries.pop_front()?.bytes;
        }
        self.entries.push_back(Entry {
            key,
            _font: Arc::clone(run.font.bytes),
            image,
            x: mask.x,
            y: mask.y,
            bytes,
        });
        self.stats.retained_bytes += bytes;
        self.stats.entries = self.entries.len();
        Some((self.entries.len() - 1, ix, iy))
    }
}

pub struct GlyphRenderer<'a, F: Factory> {
    cache: &'a mut GlyphCache<F>,
    inner: &'a mut dyn Renderer,
    transform: Mat2D,
    opacity: f32,
    stack: Vec<(Mat2D, f32)>,
    groups: Vec<(Mat2D, f32, usize)>,
}
fn concat(a: Mat2D, b: Mat2D) -> Mat2D {
    let a = a.0;
    let b = b.0;
    Mat2D([
        a[0] * b[0] + a[2] * b[1],
        a[1] * b[0] + a[3] * b[1],
        a[0] * b[2] + a[2] * b[3],
        a[1] * b[2] + a[3] * b[3],
        a[0] * b[4] + a[2] * b[5] + a[4],
        a[1] * b[4] + a[3] * b[5] + a[5],
    ])
}
impl<F: Factory> Renderer for GlyphRenderer<'_, F> {
    fn opacity_group_capacity(&self) -> usize { self.inner.opacity_group_capacity() }
    fn begin_opacity_group(&mut self, alpha: f32) -> bool {
        if !self.inner.begin_opacity_group(alpha) { return false; }
        self.groups.push((self.transform, self.opacity, self.stack.len()));
        true
    }
    fn end_opacity_group(&mut self) -> bool {
        let Some(&(transform, opacity, saves)) = self.groups.last() else { return false; };
        if saves != self.stack.len() || !self.inner.end_opacity_group() { return false; }
        self.groups.pop();
        self.transform = transform;
        self.opacity = opacity;
        true
    }

    fn supports_glyph_runs(&self) -> bool {
        true
    }
    fn draw_glyph_run(&mut self, run: &RenderGlyphRun<'_>) -> bool {
        if self.opacity == 0.0 {
            return true;
        }
        let inverse_opacity = 1.0 / self.opacity;
        if !self.opacity.is_finite() || !inverse_opacity.is_finite() {
            return false;
        }
        let Some(inverse) = self
            .transform
            .invert()
            .filter(|m| m.0.iter().all(|v| v.is_finite()))
        else {
            return false;
        };
        let Some((index, ix, iy)) = self.cache.prepare(run, self.transform) else {
            return false;
        };
        let entry = &self.cache.entries[index];
        if let Some(image) = entry.image.as_ref() {
            // No destination state changes occur until rasterization and image
            // decoding succeed for the entire run. No partial fallback draws.
            self.inner.save();
            // Express glyph opacity once as an explicit image opacity. The
            // exact renderer's image-as-path branch otherwise applies its
            // modulated state both in drawImage and again in drawPath. Scoped
            // normalization works with both image branches and leaves all
            // surrounding vector/image state unchanged.
            self.inner.modulate_opacity(inverse_opacity);
            self.inner.transform(inverse);
            self.inner
                .translate((entry.x + ix) as f32, (entry.y + iy) as f32);
            self.inner.draw_image(
                Some(image.as_ref()),
                ImageSampler::default(),
                run.blend_mode,
                self.opacity,
            );
            self.inner.restore();
        }
        true
    }
    fn save(&mut self) {
        self.stack.push((self.transform, self.opacity));
        self.inner.save();
    }
    fn restore(&mut self) {
        if let Some((m, opacity)) = self.stack.pop() {
            self.transform = m;
            self.opacity = opacity;
        }
        self.inner.restore();
    }
    fn transform(&mut self, m: Mat2D) {
        self.transform = concat(self.transform, m);
        self.inner.transform(m);
    }
    fn draw_path(&mut self, p: &dyn RenderPath, c: &dyn RenderPaint) {
        self.inner.draw_path(p, c);
    }
    fn clip_out_rect(&mut self, rect: nuxie_render_api::Aabb) -> bool {
        self.inner.clip_out_rect(rect)
    }
    fn clip_axis(&mut self, horizontal: bool, min: f32, max: f32) -> bool {
        self.inner.clip_axis(horizontal, min, max)
    }

    fn clip_axis_transformed(&mut self, horizontal: bool, min: f32, max: f32, local: nuxie_render_api::Mat2D) -> bool {
        self.inner.clip_axis_transformed(horizontal, min, max, local)
    }
    fn clip_path(&mut self, p: &dyn RenderPath) {
        self.inner.clip_path(p);
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
        self.opacity = (self.opacity * o).max(0.0);
        self.inner.modulate_opacity(o);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nuxie_render_api::{NullRenderer, RecordingFactory};

    #[test]
    fn opacity_group_restores_glyph_transform_and_modulation() {
        let factory = RecordingFactory::new();
        let mut inner = factory.make_renderer();
        let mut cache = GlyphCache::new(RecordingFactory::new());
        let mut renderer = cache.wrap(&mut inner);
        renderer.translate(4., 5.);
        renderer.modulate_opacity(0.75);
        assert_eq!(renderer.opacity_group_capacity(), 64);
        assert!(renderer.begin_opacity_group(0.5));
        assert_eq!(renderer.opacity_group_capacity(), 63);
        renderer.translate(10., 20.);
        renderer.modulate_opacity(0.5);
        assert!(renderer.begin_opacity_group(0.25));
        renderer.scale(2., 3.);
        assert!(renderer.end_opacity_group());
        assert_eq!(renderer.transform, Mat2D([1.,0.,0.,1.,14.,25.]));
        assert_eq!(renderer.opacity, 0.375);
        assert!(renderer.end_opacity_group());
        assert_eq!(renderer.transform, Mat2D([1.,0.,0.,1.,4.,5.]));
        assert_eq!(renderer.opacity, 0.75);
        assert_eq!(renderer.opacity_group_capacity(), 64);
        assert!(!renderer.end_opacity_group());
    }

    #[test]
    fn unsupported_opacity_group_leaves_glyph_state_untouched() {
        let mut inner = NullRenderer;
        let mut cache = GlyphCache::new(RecordingFactory::new());
        let mut renderer = cache.wrap(&mut inner);
        assert_eq!(renderer.opacity_group_capacity(), 0);
        assert!(!renderer.begin_opacity_group(0.5));
        assert!(renderer.groups.is_empty());
        assert_eq!(renderer.transform, Mat2D::IDENTITY);
        assert_eq!(renderer.opacity, 1.0);
    }
}
