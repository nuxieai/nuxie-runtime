#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ImageFilter {
    Bilinear = 0,
    Nearest = 1,
}
pub const NUM_IMAGE_FILTERS: usize = 2;
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ImageWrap {
    Clamp = 0,
    Repeat = 1,
    Mirror = 2,
}
pub const NUM_IMAGE_WRAP: usize = 3;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ImageSampler {
    pub wrap_x: ImageWrap,
    pub wrap_y: ImageWrap,
    pub filter: ImageFilter,
}
impl Default for ImageSampler {
    fn default() -> Self {
        Self {
            wrap_x: ImageWrap::Clamp,
            wrap_y: ImageWrap::Clamp,
            filter: ImageFilter::Bilinear,
        }
    }
}
impl ImageSampler {
    pub const LINEAR_CLAMP_SAMPLER_KEY: u8 = 0;
    pub const MAX_SAMPLER_PERMUTATIONS: usize = NUM_IMAGE_FILTERS * NUM_IMAGE_WRAP * NUM_IMAGE_WRAP;
    pub const fn linear_clamp() -> Self {
        Self {
            wrap_x: ImageWrap::Clamp,
            wrap_y: ImageWrap::Clamp,
            filter: ImageFilter::Bilinear,
        }
    }
    pub const fn as_key(self) -> u8 {
        self.wrap_x as u8
            + self.wrap_y as u8 * NUM_IMAGE_WRAP as u8
            + self.filter as u8 * NUM_IMAGE_WRAP as u8 * NUM_IMAGE_WRAP as u8
    }
    pub fn sampler_from_key(key: u8) -> Self {
        Self {
            wrap_x: Self::get_wrap_x_option_from_key(key),
            wrap_y: Self::get_wrap_y_option_from_key(key),
            filter: Self::get_filter_option_from_key(key),
        }
    }
    pub fn get_wrap_x_option_from_key(key: u8) -> ImageWrap {
        ImageWrap::from(key % NUM_IMAGE_WRAP as u8)
    }
    pub fn get_wrap_y_option_from_key(key: u8) -> ImageWrap {
        ImageWrap::from((key / NUM_IMAGE_WRAP as u8) % NUM_IMAGE_WRAP as u8)
    }
    pub fn get_filter_option_from_key(key: u8) -> ImageFilter {
        ImageFilter::from(key / (NUM_IMAGE_WRAP * NUM_IMAGE_WRAP) as u8)
    }
}
