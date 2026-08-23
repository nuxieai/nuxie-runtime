//! Native Metal image-sampler table.
//!
//! Mechanical translation of the pinned upstream implementation in
//! `renderer/src/metal/render_context_metal_impl.mm:59-98,562-577` and
//! `renderer/include/rive/renderer/metal/render_context_metal_impl.h:264` at
//! `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

use nuxie_render_api::{ImageFilter, ImageSampler, ImageWrap};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{
    MTLDevice, MTLSamplerAddressMode, MTLSamplerDescriptor, MTLSamplerMinMagFilter,
    MTLSamplerMipFilter, MTLSamplerState,
};

use crate::RendererError;

/// `3 wrap-X * 3 wrap-Y * 2 filter` permutations, matching
/// `ImageSampler::MAX_SAMPLER_PERMUTATIONS` in the pinned upstream API.
pub(crate) const MAX_SAMPLER_PERMUTATIONS: usize = 18;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SamplerPolicy {
    pub(crate) s_address_mode: MTLSamplerAddressMode,
    pub(crate) t_address_mode: MTLSamplerAddressMode,
    pub(crate) min_filter: MTLSamplerMinMagFilter,
    pub(crate) mag_filter: MTLSamplerMinMagFilter,
    pub(crate) mip_filter: MTLSamplerMipFilter,
}

/// Translate the upstream image-wrap switch exactly.
pub(crate) fn address_mode_for_image_wrap(wrap: ImageWrap) -> MTLSamplerAddressMode {
    match wrap {
        ImageWrap::Clamp => MTLSamplerAddressMode::ClampToEdge,
        ImageWrap::Repeat => MTLSamplerAddressMode::Repeat,
        ImageWrap::Mirror => MTLSamplerAddressMode::MirrorRepeat,
    }
}

/// Translate the upstream min/mag image-filter switch exactly.
pub(crate) fn min_mag_filter_for_image_filter(filter: ImageFilter) -> MTLSamplerMinMagFilter {
    match filter {
        ImageFilter::Bilinear => MTLSamplerMinMagFilter::Linear,
        ImageFilter::Nearest => MTLSamplerMinMagFilter::Nearest,
    }
}

/// The pinned backend always selects nearest mip filtering for both image
/// filters; this is not the same as min/mag filtering.
pub(crate) fn mip_filter_for_image_filter(_filter: ImageFilter) -> MTLSamplerMipFilter {
    MTLSamplerMipFilter::Nearest
}

pub(crate) fn sampler_policy(sampler: ImageSampler) -> SamplerPolicy {
    SamplerPolicy {
        s_address_mode: address_mode_for_image_wrap(sampler.wrap_x),
        t_address_mode: address_mode_for_image_wrap(sampler.wrap_y),
        min_filter: min_mag_filter_for_image_filter(sampler.filter),
        mag_filter: min_mag_filter_for_image_filter(sampler.filter),
        mip_filter: mip_filter_for_image_filter(sampler.filter),
    }
}

fn sampler_for_key(key: usize) -> ImageSampler {
    let wrap_x = match key % 3 {
        0 => ImageWrap::Clamp,
        1 => ImageWrap::Repeat,
        _ => ImageWrap::Mirror,
    };
    let wrap_y = match (key / 3) % 3 {
        0 => ImageWrap::Clamp,
        1 => ImageWrap::Repeat,
        _ => ImageWrap::Mirror,
    };
    let filter = if key / 9 == 0 {
        ImageFilter::Bilinear
    } else {
        ImageFilter::Nearest
    };
    ImageSampler {
        wrap_x,
        wrap_y,
        filter,
    }
}

/// The exact 18-key sampler table owned by a Metal render context.
pub(crate) struct NativeMetalSamplers {
    samplers: [Retained<ProtocolObject<dyn MTLSamplerState>>; MAX_SAMPLER_PERMUTATIONS],
}

impl NativeMetalSamplers {
    pub(crate) fn new(device: &ProtocolObject<dyn MTLDevice>) -> Result<Self, RendererError> {
        let mut samplers = Vec::with_capacity(MAX_SAMPLER_PERMUTATIONS);
        for key in 0..MAX_SAMPLER_PERMUTATIONS {
            let policy = sampler_policy(sampler_for_key(key));
            let descriptor = MTLSamplerDescriptor::new();
            descriptor.setMinFilter(policy.min_filter);
            descriptor.setMagFilter(policy.mag_filter);
            descriptor.setMipFilter(policy.mip_filter);
            descriptor.setSAddressMode(policy.s_address_mode);
            descriptor.setTAddressMode(policy.t_address_mode);
            let sampler = device
                .newSamplerStateWithDescriptor(&descriptor)
                .ok_or_else(|| {
                    RendererError::NativeMetal(format!(
                        "failed to allocate image sampler permutation {key}"
                    ))
                })?;
            samplers.push(sampler);
        }
        let samplers = samplers.try_into().map_err(
            |states: Vec<Retained<ProtocolObject<dyn MTLSamplerState>>>| {
                RendererError::NativeMetal(format!(
                    "constructed {} image sampler permutations, expected {MAX_SAMPLER_PERMUTATIONS}",
                    states.len()
                ))
            },
        )?;
        Ok(Self { samplers })
    }

    pub(crate) fn sampler(&self, sampler: ImageSampler) -> &ProtocolObject<dyn MTLSamplerState> {
        &self.samplers[sampler.as_key() as usize]
    }

    pub(crate) fn as_slice(&self) -> &[Retained<ProtocolObject<dyn MTLSamplerState>>] {
        &self.samplers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sampler_keys_cover_exactly_18_permutations() {
        let mut keys = [false; MAX_SAMPLER_PERMUTATIONS];
        for wrap_x in [ImageWrap::Clamp, ImageWrap::Repeat, ImageWrap::Mirror] {
            for wrap_y in [ImageWrap::Clamp, ImageWrap::Repeat, ImageWrap::Mirror] {
                for filter in [ImageFilter::Bilinear, ImageFilter::Nearest] {
                    let sampler = ImageSampler {
                        wrap_x,
                        wrap_y,
                        filter,
                    };
                    let key = sampler.as_key() as usize;
                    assert!(key < MAX_SAMPLER_PERMUTATIONS);
                    keys[key] = true;
                }
            }
        }
        assert!(keys.into_iter().all(|present| present));
    }

    #[test]
    fn sampler_key_decode_round_trips_upstream_order() {
        for key in 0..MAX_SAMPLER_PERMUTATIONS {
            assert_eq!(sampler_for_key(key).as_key() as usize, key);
        }
    }

    #[test]
    fn wrap_policy_preserves_clamp_repeat_mirror_semantics() {
        assert_eq!(
            address_mode_for_image_wrap(ImageWrap::Clamp),
            MTLSamplerAddressMode::ClampToEdge
        );
        assert_eq!(
            address_mode_for_image_wrap(ImageWrap::Repeat),
            MTLSamplerAddressMode::Repeat
        );
        assert_eq!(
            address_mode_for_image_wrap(ImageWrap::Mirror),
            MTLSamplerAddressMode::MirrorRepeat
        );
    }

    #[test]
    fn filter_policy_keeps_nearest_mips_for_bilinear_and_nearest() {
        for filter in [ImageFilter::Bilinear, ImageFilter::Nearest] {
            let policy = sampler_policy(ImageSampler {
                wrap_x: ImageWrap::Clamp,
                wrap_y: ImageWrap::Clamp,
                filter,
            });
            assert_eq!(policy.mip_filter, MTLSamplerMipFilter::Nearest);
            assert_eq!(policy.min_filter, policy.mag_filter);
        }
        assert_eq!(
            min_mag_filter_for_image_filter(ImageFilter::Bilinear),
            MTLSamplerMinMagFilter::Linear
        );
        assert_eq!(
            min_mag_filter_for_image_filter(ImageFilter::Nearest),
            MTLSamplerMinMagFilter::Nearest
        );
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[test]
    fn live_sampler_table_has_18_retained_states_and_stable_lookup() {
        let Some(device) = objc2_metal::MTLCreateSystemDefaultDevice() else {
            crate::live_metal_test_unavailable("system Metal device");
            return;
        };
        let samplers = NativeMetalSamplers::new(&device).unwrap();
        assert_eq!(samplers.as_slice().len(), MAX_SAMPLER_PERMUTATIONS);
        let linear_clamp = samplers.sampler(ImageSampler::LINEAR_CLAMP) as *const _;
        assert!(std::ptr::eq(
            linear_clamp,
            samplers.sampler(ImageSampler::LINEAR_CLAMP) as *const _
        ));
        assert_ne!(
            linear_clamp,
            samplers.sampler(ImageSampler {
                wrap_x: ImageWrap::Repeat,
                ..ImageSampler::LINEAR_CLAMP
            }) as *const _
        );
    }
}
