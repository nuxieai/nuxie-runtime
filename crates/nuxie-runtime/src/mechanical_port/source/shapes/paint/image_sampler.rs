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
pub const ImageFilterCount: usize = 2;
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
pub const ImageWrapCount: usize = 3;

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
    pub const MAX_SAMPLER_PERMUTATIONS: usize = ImageFilterCount * ImageWrapCount * ImageWrapCount;
    pub const fn linear_clamp() -> Self {
        Self {
            wrap_x: ImageWrap::Clamp,
            wrap_y: ImageWrap::Clamp,
            filter: ImageFilter::Bilinear,
        }
    }
    pub const fn as_key(self) -> u8 {
        Self::make_key(self.filter, self.wrap_x, self.wrap_y)
    }
    pub const fn linear_wrap() -> Self {
        Self {
            wrap_x: ImageWrap::Repeat,
            wrap_y: ImageWrap::Repeat,
            filter: ImageFilter::Bilinear,
        }
    }
    pub const fn make_key(filter: ImageFilter, wrap_x: ImageWrap, wrap_y: ImageWrap) -> u8 {
        (wrap_x.0 as usize
            + wrap_y.0 as usize * ImageWrapCount
            + filter.0 as usize * ImageWrapCount * ImageWrapCount) as u8
    }
    pub const fn make_key_for_wrap(filter: ImageFilter, wrap: ImageWrap) -> u8 {
        Self::make_key(filter, wrap, wrap)
    }
    pub fn sampler_from_key(key: u8) -> Self {
        Self {
            wrap_x: Self::get_wrap_x_option_from_key(key),
            wrap_y: Self::get_wrap_y_option_from_key(key),
            filter: Self::get_filter_option_from_key(key),
        }
    }
    pub fn get_wrap_x_option_from_key(key: u8) -> ImageWrap {
        ImageWrap::from(key % ImageWrapCount as u8)
    }
    pub fn get_wrap_y_option_from_key(key: u8) -> ImageWrap {
        ImageWrap::from((key / ImageWrapCount as u8) % ImageWrapCount as u8)
    }
    pub fn get_filter_option_from_key(key: u8) -> ImageFilter {
        ImageFilter::from(key / (ImageWrapCount * ImageWrapCount) as u8)
    }
}

#[allow(non_upper_case_globals)]
pub const BilinearClampImageSamplerKey: u8 =
    ImageSampler::make_key_for_wrap(ImageFilter::Bilinear, ImageWrap::Clamp);
#[allow(non_upper_case_globals)]
pub const BilinearRepeatImageSamplerKey: u8 =
    ImageSampler::make_key_for_wrap(ImageFilter::Bilinear, ImageWrap::Repeat);

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
