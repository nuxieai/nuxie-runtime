use super::bitmap_decoder::Bitmap;

/// The native PNG owner delegates the libpng substrate to the approved Rust
/// PNG backend; expansion, strip-16, RGB/RGBA and unpremultiplied pixels are
/// retained by that explicit format-preserving boundary.
pub fn decode_png(bytes: &[u8]) -> Option<Bitmap> {
    nuxie_image_codec::decode_png_bitmap(bytes).map(Bitmap::from_decoded)
}
