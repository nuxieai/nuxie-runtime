use crate::mechanical_port::source::{
    audio::audio_source::AudioSource,
    math::{aabb::Aabb, raw_path::RawPath},
    refcnt::Rcp,
    renderer::{
        ColorInt, FillRule, RenderBuffer, RenderBufferFlags, RenderBufferType, RenderImage,
        RenderPaint, RenderPath, RenderShader,
    },
    simple_array::SimpleArray,
    text_engine::Font,
};

pub trait Factory {
    type OreContext;

    fn make_render_buffer(
        &mut self,
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        size_in_bytes: usize,
    ) -> Rcp<RenderBuffer>;

    fn make_linear_gradient(
        &mut self,
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
        colors: &[ColorInt],
        stops: &[f32],
        count: usize,
    ) -> Rcp<RenderShader>;

    fn make_radial_gradient(
        &mut self,
        center_x: f32,
        center_y: f32,
        radius: f32,
        colors: &[ColorInt],
        stops: &[f32],
        count: usize,
    ) -> Rcp<RenderShader>;

    fn make_render_path(&mut self, path: &mut RawPath, fill_rule: FillRule) -> Rcp<RenderPath>;
    fn make_empty_render_path(&mut self) -> Rcp<RenderPath>;
    fn make_render_paint(&mut self) -> Rcp<RenderPaint>;
    fn decode_image(&mut self, bytes: &[u8]) -> Rcp<RenderImage>;

    fn ore(&mut self) -> Option<&mut Self::OreContext> {
        None
    }

    fn decode_font(&mut self, bytes: &[u8]) -> Option<Rcp<Font>> {
        #[cfg(feature = "rive_text")]
        {
            crate::mechanical_port::source::text::font_hb::HbFont::decode(bytes)
        }
        #[cfg(not(feature = "rive_text"))]
        {
            let _ = bytes;
            None
        }
    }

    fn decode_audio(&mut self, bytes: &[u8]) -> Option<Rcp<AudioSource>> {
        #[cfg(feature = "rive_audio")]
        {
            AudioSource::make_audio_source(SimpleArray::from_slice(bytes))
        }
        #[cfg(not(feature = "rive_audio"))]
        {
            let _ = bytes;
            None
        }
    }

    fn make_render_path_from_aabb(&mut self, bounds: &Aabb) -> Rcp<RenderPath> {
        let mut raw_path = RawPath::default();
        raw_path.add_rect(bounds);
        self.make_render_path(&mut raw_path, FillRule::NonZero)
    }
}
