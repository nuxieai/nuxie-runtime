//! Bitmap's retained pixel storage, format declarations, and conversion owner.
//! Platform selection and image recognition live in bitmap_decoder_thirdparty.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelFormat {
    Rgb,
    Rgba,
    RgbaPremul,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageType {
    Png,
    Jpeg,
    Webp,
}

pub type BitmapDecoder = fn(&[u8]) -> Option<Bitmap>;

pub struct ImageFormat {
    pub name: &'static str,
    pub image_type: ImageType,
    pub fingerprint: &'static [u8],
    pub decode_image: Option<BitmapDecoder>,
}

pub struct Bitmap {
    width: u32,
    height: u32,
    num_bytes: usize,
    pixel_format: PixelFormat,
    bytes: Vec<u8>,
}

impl Bitmap {
    pub(crate) fn from_decoded(decoded: nuxie_image_codec::DecodedBitmap) -> Self {
        let format = match decoded.pixel_format {
            nuxie_image_codec::BitmapPixelFormat::Rgb => PixelFormat::Rgb,
            nuxie_image_codec::BitmapPixelFormat::Rgba => PixelFormat::Rgba,
            nuxie_image_codec::BitmapPixelFormat::RgbaPremul => PixelFormat::RgbaPremul,
        };
        Self::new(
            decoded.width,
            decoded.height,
            decoded.pixels.len(),
            format,
            decoded.pixels,
        )
    }

    pub fn new(
        width: u32,
        height: u32,
        num_bytes: usize,
        pixel_format: PixelFormat,
        bytes: Vec<u8>,
    ) -> Self {
        Self {
            width,
            height,
            num_bytes,
            pixel_format,
            bytes,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn num_bytes(&self) -> usize {
        self.num_bytes
    }
    pub fn pixel_format(&self) -> PixelFormat {
        self.pixel_format
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes[..self.num_bytes]
    }
    pub fn detach_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.bytes)
    }

    pub fn set_pixel_format(&mut self, format: PixelFormat) {
        if format == self.pixel_format {
            return;
        }
        let image_num_pixels = self.height.wrapping_mul(self.width) as usize;
        let from_bytes_per_pixel = bytes_per_pixel(self.pixel_format);
        let to_bytes_per_pixel = bytes_per_pixel(format);
        let to_size_in_bytes = image_num_pixels * to_bytes_per_pixel;
        let alloc_size = (to_size_in_bytes + 7) & !7;
        assert!(alloc_size >= to_size_in_bytes && alloc_size <= to_size_in_bytes + 7);
        let mut to_bytes = vec![0; alloc_size];
        let mut write_index = 0;
        let mut read_index = 0;
        for _ in 0..image_num_pixels {
            for channel in 0..to_bytes_per_pixel {
                to_bytes[write_index] = if channel < from_bytes_per_pixel {
                    let value = self.bytes[read_index];
                    read_index += 1;
                    value
                } else {
                    255
                };
                write_index += 1;
            }
        }
        if format == PixelFormat::RgbaPremul {
            for offset in (0..to_size_in_bytes).step_by(8) {
                assert!(offset + 8 <= alloc_size);
                let alpha0 = to_bytes[offset + 3];
                let alpha1 = to_bytes[offset + 7];
                if alpha0 != 255 || alpha1 != 255 {
                    let alpha = [alpha0, alpha0, alpha0, 255, alpha1, alpha1, alpha1, 255];
                    for channel in 0..8 {
                        let wide =
                            u16::from(to_bytes[offset + channel]) * u16::from(alpha[channel]) + 128;
                        to_bytes[offset + channel] = ((wide + (wide >> 8)) >> 8) as u8;
                    }
                }
            }
        } else {
            assert!(self.pixel_format != PixelFormat::RgbaPremul);
        }
        self.bytes = to_bytes;
        self.pixel_format = format;
        self.num_bytes = to_size_in_bytes;
    }
}

fn bytes_per_pixel(format: PixelFormat) -> usize {
    match format {
        PixelFormat::Rgb => 3,
        PixelFormat::Rgba | PixelFormat::RgbaPremul => 4,
    }
}
