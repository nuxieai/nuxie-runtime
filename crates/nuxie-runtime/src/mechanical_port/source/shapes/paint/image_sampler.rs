#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ImageFilter(u8);
#[allow(non_upper_case_globals)]
impl ImageFilter {
    pub const Bilinear: Self = Self(0);
    pub const Nearest: Self = Self(1);
}
impl From<u8> for ImageFilter {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
pub const NUM_IMAGE_FILTERS: usize = 2;
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ImageWrap(u8);
#[allow(non_upper_case_globals)]
impl ImageWrap {
    pub const Clamp: Self = Self(0);
    pub const Repeat: Self = Self(1);
    pub const Mirror: Self = Self(2);
}
impl From<u8> for ImageWrap {
    fn from(value: u8) -> Self {
        Self(value)
    }
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
        (self.wrap_x.0 as u16
            + self.wrap_y.0 as u16 * NUM_IMAGE_WRAP as u16
            + self.filter.0 as u16 * NUM_IMAGE_WRAP as u16 * NUM_IMAGE_WRAP as u16) as u8
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

impl From<ImageSampler> for nuxie_render_api::ImageSampler {
    fn from(value: ImageSampler) -> Self {
        Self {
            wrap_x: match value.wrap_x {
                ImageWrap::Clamp => nuxie_render_api::ImageWrap::Clamp,
                ImageWrap::Repeat => nuxie_render_api::ImageWrap::Repeat,
                ImageWrap::Mirror => nuxie_render_api::ImageWrap::Mirror,
                ImageWrap(value) => panic!("unsupported renderer image wrap {value}"),
            },
            wrap_y: match value.wrap_y {
                ImageWrap::Clamp => nuxie_render_api::ImageWrap::Clamp,
                ImageWrap::Repeat => nuxie_render_api::ImageWrap::Repeat,
                ImageWrap::Mirror => nuxie_render_api::ImageWrap::Mirror,
                ImageWrap(value) => panic!("unsupported renderer image wrap {value}"),
            },
            filter: match value.filter {
                ImageFilter::Bilinear => nuxie_render_api::ImageFilter::Bilinear,
                ImageFilter::Nearest => nuxie_render_api::ImageFilter::Nearest,
                ImageFilter(value) => panic!("unsupported renderer image filter {value}"),
            },
        }
    }
}
