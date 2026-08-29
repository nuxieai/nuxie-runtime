use super::{
    bitmap_decoder::{Bitmap, ImageFormat, ImageType},
    decode_jpeg::decode_jpeg,
    decode_png::decode_png,
    decode_webp::decode_webp,
};

// All three portable codecs are present in the approved Rust host backend.
// Preserve the source table's order and intentionally short fingerprints.
static FORMATS: [ImageFormat; 3] = [
    ImageFormat {
        name: "png",
        image_type: ImageType::Png,
        fingerprint: &[0x89, 0x50, 0x4e, 0x47],
        decode_image: Some(decode_png),
    },
    ImageFormat {
        name: "jpeg",
        image_type: ImageType::Jpeg,
        fingerprint: &[0xff, 0xd8, 0xff],
        decode_image: Some(decode_jpeg),
    },
    ImageFormat {
        name: "webp",
        image_type: ImageType::Webp,
        fingerprint: &[0x52, 0x49, 0x46],
        decode_image: Some(decode_webp),
    },
];

impl Bitmap {
    pub fn recognize_image_format(bytes: &[u8]) -> Option<&'static ImageFormat> {
        FORMATS
            .iter()
            .find(|format| bytes.starts_with(format.fingerprint))
    }

    #[cfg(target_vendor = "apple")]
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if let Some(image) = nuxie_image_codec::decode_apple_bitmap(bytes) {
            return Some(Self::from_decoded(image));
        }
        // The pin only falls back to portable codecs after ImageIO fails on
        // Apple TV; other Apple platforms return the actual ImageIO failure.
        #[cfg(target_os = "tvos")]
        if let Some(format) = Self::recognize_image_format(bytes) {
            return format.decode_image.and_then(|decode| decode(bytes));
        }
        None
    }

    #[cfg(not(target_vendor = "apple"))]
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let format = Self::recognize_image_format(bytes)?;
        let bitmap = format.decode_image.and_then(|decode| decode(bytes));
        if bitmap.is_none() {
            eprintln!("Bitmap::decode - failed to decode a {}.", format.name);
        }
        bitmap
    }
}
