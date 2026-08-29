use super::bitmap_decoder::Bitmap;

/// DecodeJpeg's RGB output uses the existing Rust JPEG backend, without the
/// renderer's ICC conversion or alpha-channel expansion.
pub fn decode_jpeg(bytes: &[u8]) -> Option<Bitmap> {
    nuxie_image_codec::decode_jpeg_bitmap(bytes).map(Bitmap::from_decoded)
}
