//! Mechanical owner for pinned `src/factory.cpp`.

use std::sync::Arc;

use crate::{
    Aabb, AudioDecodeError, AudioSource, DecodedFont, Factory, FillRule, FontDecodeError, RawPath,
    RenderPath,
};

pub(super) fn make_render_path_from_aabb<F: Factory + ?Sized>(
    factory: &mut F,
    bounds: Aabb,
) -> Box<dyn RenderPath> {
    let mut raw_path = RawPath::new();
    raw_path.add_rect(bounds);
    factory.make_render_path(raw_path, FillRule::NonZero)
}

pub(super) fn decode_font(data: &[u8]) -> Result<DecodedFont, FontDecodeError> {
    harfrust::FontRef::new(data).map_err(|_| FontDecodeError)?;
    Ok(DecodedFont {
        bytes: Arc::from(data),
    })
}

pub(super) fn decode_audio(data: &[u8]) -> Result<Arc<AudioSource>, AudioDecodeError> {
    AudioSource::from_encoded(data.to_vec()).map(Arc::new)
}
