// Mechanical translation of the complete pinned source header
// include/rive/shapes/paint/image_sampler.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

// #ifndef IMAGE_SAMPLER
// #define IMAGE_SAMPLER

// #include <stdint.h>

// `uint8_t` and `size_t` retain the source fixed-width and native-size
// representations in the Rust declarations below.

// namespace rive
// {

// enum class ImageFilter : uint8_t
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageFilter(pub u8);

impl ImageFilter {
    // High fidelity linear filter in all 2 directions: x, y
    pub const bilinear: Self = Self(0);
    // Sample with low fidelity, good for things like pixel art.
    pub const nearest: Self = Self(1);
}

// constexpr size_t ImageFilterCount = 2;
pub const ImageFilterCount: usize = 2;

// enum class ImageWrap : uint8_t
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageWrap(pub u8);

impl ImageWrap {
    // Clamp to the color of the nearest edge when a texture sample falls
    // outside 0..1.
    pub const clamp: Self = Self(0);
    // Repeat when a texture sample falls outside 0..1 (e.g., fmod(coord, 1)).
    pub const repeat: Self = Self(1);
    // Similar to repeat, but also mirror the coordinate with each repeat.
    pub const mirror: Self = Self(2);
}

// constexpr size_t ImageWrapCount = 3;
pub const ImageWrapCount: usize = 3;

// struct ImageSampler
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ImageSampler {
    // ImageWrap wrapX = ImageWrap::clamp;
    pub wrapX: ImageWrap,
    // ImageWrap wrapY = ImageWrap::clamp;
    pub wrapY: ImageWrap,
    // How to sample the texture, this will be for both MIN and MAG filtering.
    // ImageFilter filter = ImageFilter::bilinear;
    pub filter: ImageFilter,
}

impl ImageSampler {
    // static constexpr ImageSampler LinearClamp() { return {}; }
    pub const fn LinearClamp() -> Self {
        Self {
            wrapX: ImageWrap::clamp,
            wrapY: ImageWrap::clamp,
            filter: ImageFilter::bilinear,
        }
    }

    pub const fn LinearWrap() -> Self {
        Self {
            wrapX: ImageWrap::repeat,
            wrapY: ImageWrap::repeat,
            filter: ImageFilter::bilinear,
        }
    }

    pub const fn makeKey(filter: ImageFilter, wrapX: ImageWrap, wrapY: ImageWrap) -> u8 {
        (wrapX.0 as usize
            + wrapY.0 as usize * ImageWrapCount
            + filter.0 as usize * ImageWrapCount * ImageWrapCount) as u8
    }

    pub const fn makeKeyForWrap(filter: ImageFilter, wrap: ImageWrap) -> u8 {
        Self::makeKey(filter, wrap, wrap)
    }

    // bool operator==(const ImageSampler other) const
    // {
    //     return other.wrapX == wrapX && other.wrapY == wrapY &&
    //            other.filter == filter;
    // }
    // Rust's PartialEq implementation below supplies this source operator.

    // bool operator!=(const ImageSampler other) const
    // {
    //     return !(*this == other);
    // }
    // Rust derives `!=` from the source-shaped PartialEq implementation.

    // The maximum number of possible combinations of sampler options. Used for
    // array length in implementations.
    // static constexpr size_t MAX_SAMPLER_PERMUTATIONS =
    //     ImageFilterCount * ImageWrapCount * ImageWrapCount;
    pub const MAX_SAMPLER_PERMUTATIONS: usize = ImageFilterCount * ImageWrapCount * ImageWrapCount;

    // Convert struct to a key that can be used to index an array to get a
    // unique sampler that represents these options.
    // const uint8_t asKey() const
    // {
    //     return static_cast<int>(wrapX) +
    //            (static_cast<int>(wrapY) * ImageWrapCount) +
    //            (static_cast<int>(filter) * ImageWrapCount * ImageWrapCount);
    // }
    pub const fn asKey(&self) -> u8 {
        Self::makeKey(self.filter, self.wrapX, self.wrapY)
    }

    // static ImageSampler SamplerFromKey(uint8_t key)
    // {
    //     // Android wouldn't compile with {} style initialization so do it this
    //     // way instead.
    //     ImageSampler sampler;
    //
    //     sampler.wrapX = GetWrapXOptionFromKey(key);
    //     sampler.wrapY = GetWrapYOptionFromKey(key);
    //     sampler.filter = GetFilterOptionFromKey(key);
    //
    //     return sampler;
    // }
    pub fn SamplerFromKey(key: u8) -> Self {
        // C++ aggregate default initialization applies the member defaults;
        // `Self::default()` is the corresponding Rust initialization.
        let mut sampler = Self::default();

        sampler.wrapX = Self::GetWrapXOptionFromKey(key);
        sampler.wrapY = Self::GetWrapYOptionFromKey(key);
        sampler.filter = Self::GetFilterOptionFromKey(key);

        sampler
    }

    // static ImageWrap GetWrapXOptionFromKey(uint8_t key)
    // {
    //     return static_cast<ImageWrap>(key % ImageWrapCount);
    // }
    pub fn GetWrapXOptionFromKey(key: u8) -> ImageWrap {
        ImageWrap((key as usize % ImageWrapCount) as u8)
    }

    // static ImageWrap GetWrapYOptionFromKey(uint8_t key)
    // {
    //     return static_cast<ImageWrap>((key / ImageWrapCount) % ImageWrapCount);
    // }
    pub fn GetWrapYOptionFromKey(key: u8) -> ImageWrap {
        ImageWrap(((key as usize / ImageWrapCount) % ImageWrapCount) as u8)
    }

    // static ImageFilter GetFilterOptionFromKey(uint8_t key)
    // {
    //     return static_cast<ImageFilter>(key /
    //                                     (ImageWrapCount * ImageWrapCount));
    // }
    pub fn GetFilterOptionFromKey(key: u8) -> ImageFilter {
        // The source static_cast preserves values above `nearest` for keys
        // outside MAX_SAMPLER_PERMUTATIONS. A transparent integer newtype is
        // the Rust representation that preserves that byte without creating
        // an invalid enum discriminant.
        ImageFilter((key as usize / (ImageWrapCount * ImageWrapCount)) as u8)
    }
}

impl Default for ImageSampler {
    fn default() -> Self {
        Self::LinearClamp()
    }
}

impl PartialEq for ImageSampler {
    fn eq(&self, other: &Self) -> bool {
        other.wrapX == self.wrapX && other.wrapY == self.wrapY && other.filter == self.filter
    }
}

impl Eq for ImageSampler {}

pub const BilinearClampImageSamplerKey: u8 =
    ImageSampler::makeKeyForWrap(ImageFilter::bilinear, ImageWrap::clamp);
pub const BilinearRepeatImageSamplerKey: u8 =
    ImageSampler::makeKeyForWrap(ImageFilter::bilinear, ImageWrap::repeat);

// } // namespace rive
// #endif
