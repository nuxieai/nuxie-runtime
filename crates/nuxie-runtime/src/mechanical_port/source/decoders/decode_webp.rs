use super::bitmap_decoder::Bitmap;

/// Decode frame one into the canvas-stride RGBA buffer. The approved Rust host
/// adapter preserves the source's non-composited first-frame placement.
pub fn decode_webp(bytes: &[u8]) -> Option<Bitmap> {
    nuxie_image_codec::decode_webp_bitmap(bytes).map(Bitmap::from_decoded)
}
