//! Surface policy shared by Android swapchain creation and recreation.
//! Vulkan surface capabilities, rather than requested window dimensions, own
//! the final extent. A suspended surface has no swapchain configuration.
use crate::RendererError;
use ash::vk;

#[derive(Debug, Clone, Copy)]
pub(super) struct SurfaceConfig {
    pub format: vk::SurfaceFormatKHR,
    pub extent: vk::Extent2D,
    pub image_count: u32,
    pub alpha: vk::CompositeAlphaFlagsKHR,
    pub transform: vk::SurfaceTransformFlagsKHR,
}

impl SurfaceConfig {
    /// `native_premultiplied_alpha` is affirmative native-window configuration,
    /// not an assumption about the platform's default INHERIT behavior.
    pub(super) fn choose(
        capabilities: &vk::SurfaceCapabilitiesKHR,
        formats: &[vk::SurfaceFormatKHR],
        requested: vk::Extent2D,
        native_premultiplied_alpha: bool,
    ) -> Result<Option<Self>, RendererError> {
        let extent = if capabilities.current_extent.width == u32::MAX {
            if requested.width == 0 || requested.height == 0 {
                return Ok(None);
            }
            let min = capabilities.min_image_extent;
            let max = capabilities.max_image_extent;
            if min.width > max.width || min.height > max.height {
                return Err(RendererError::Device(
                    "invalid Vulkan surface extent bounds".into(),
                ));
            }
            vk::Extent2D {
                width: requested.width.clamp(min.width, max.width),
                height: requested.height.clamp(min.height, max.height),
            }
        } else {
            capabilities.current_extent
        };
        if extent.width == 0 || extent.height == 0 {
            return Ok(None);
        }
        if !capabilities
            .supported_usage_flags
            .contains(vk::ImageUsageFlags::TRANSFER_DST)
        {
            return Err(RendererError::Unsupported(
                "Vulkan surface transfer destination",
            ));
        }
        let alpha = if capabilities
            .supported_composite_alpha
            .contains(vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED)
        {
            vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED
        } else if native_premultiplied_alpha
            && capabilities
                .supported_composite_alpha
                .contains(vk::CompositeAlphaFlagsKHR::INHERIT)
        {
            vk::CompositeAlphaFlagsKHR::INHERIT
        } else {
            return Err(RendererError::Unsupported(
                "premultiplied Vulkan surface alpha",
            ));
        };
        // BGRA matches the retained render target. RGBA requires GPU format
        // conversion during presentation; never byte-copy BGRA into RGBA.
        let format = [vk::Format::B8G8R8A8_UNORM, vk::Format::R8G8B8A8_UNORM]
            .into_iter()
            .find_map(|preferred| {
                formats
                    .iter()
                    .find(|format| {
                        format.format == preferred
                            && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
                    })
                    .copied()
            })
            .or_else(|| {
                (formats.len() == 1
                    && formats[0].format == vk::Format::UNDEFINED
                    && formats[0].color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR)
                    .then_some(vk::SurfaceFormatKHR {
                        format: vk::Format::B8G8R8A8_UNORM,
                        color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
                    })
            })
            .ok_or(RendererError::Unsupported("RGBA8 Vulkan surface format"))?;
        let min = capabilities.min_image_count;
        let max = capabilities.max_image_count;
        if min == 0 || (max != 0 && max < min) {
            return Err(RendererError::Device(
                "invalid Vulkan surface image count bounds".into(),
            ));
        }
        // The renderer produces upright TextureView coordinates. When identity
        // is advertised, let the presentation engine apply currentTransform.
        // Setting currentTransform here would falsely claim our pixels were
        // already pre-rotated. Native pre-rotation remains a separate path.
        let transform = if capabilities
            .supported_transforms
            .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
        {
            vk::SurfaceTransformFlagsKHR::IDENTITY
        } else {
            return Err(RendererError::Unsupported(
                "upright Vulkan surface presentation",
            ));
        };
        let desired = min.saturating_add(1);
        let image_count = if max == 0 { desired } else { desired.min(max) };
        Ok(Some(Self {
            format,
            extent,
            image_count,
            alpha,
            transform,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capabilities() -> vk::SurfaceCapabilitiesKHR {
        vk::SurfaceCapabilitiesKHR {
            min_image_count: 2,
            max_image_count: 0,
            current_extent: vk::Extent2D {
                width: u32::MAX,
                height: u32::MAX,
            },
            min_image_extent: vk::Extent2D {
                width: 32,
                height: 32,
            },
            max_image_extent: vk::Extent2D {
                width: 2048,
                height: 2048,
            },
            supported_usage_flags: vk::ImageUsageFlags::TRANSFER_DST,
            supported_composite_alpha: vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED,
            current_transform: vk::SurfaceTransformFlagsKHR::ROTATE_90,
            supported_transforms: vk::SurfaceTransformFlagsKHR::IDENTITY
                | vk::SurfaceTransformFlagsKHR::ROTATE_90,
            ..Default::default()
        }
    }
    fn formats() -> [vk::SurfaceFormatKHR; 1] {
        [vk::SurfaceFormatKHR {
            format: vk::Format::R8G8B8A8_UNORM,
            color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
        }]
    }
    const REQUESTED: vk::Extent2D = vk::Extent2D {
        width: 1080,
        height: 2400,
    };

    #[test]
    fn variable_extent_clamps_and_unbounded_image_count_is_not_zero() {
        let plan = SurfaceConfig::choose(&capabilities(), &formats(), REQUESTED, false)
            .unwrap()
            .unwrap();
        assert_eq!(
            plan.extent,
            vk::Extent2D {
                width: 1080,
                height: 2048
            }
        );
        assert_eq!(plan.image_count, 3);
        assert_eq!(plan.format.format, vk::Format::R8G8B8A8_UNORM);
        assert_eq!(plan.transform, vk::SurfaceTransformFlagsKHR::IDENTITY);
    }

    #[test]
    fn presentation_engine_handles_supported_rotation_without_claiming_prerotation() {
        for current in [
            vk::SurfaceTransformFlagsKHR::IDENTITY,
            vk::SurfaceTransformFlagsKHR::ROTATE_90,
            vk::SurfaceTransformFlagsKHR::ROTATE_180,
            vk::SurfaceTransformFlagsKHR::ROTATE_270,
        ] {
            let mut caps = capabilities();
            caps.current_transform = current;
            caps.supported_transforms = current | vk::SurfaceTransformFlagsKHR::IDENTITY;
            let config = SurfaceConfig::choose(&caps, &formats(), REQUESTED, false)
                .unwrap()
                .unwrap();
            assert_eq!(config.transform, vk::SurfaceTransformFlagsKHR::IDENTITY);
            assert_eq!(
                config.extent,
                vk::Extent2D {
                    width: 1080,
                    height: 2048
                }
            );
        }
        let mut caps = capabilities();
        caps.supported_transforms = vk::SurfaceTransformFlagsKHR::ROTATE_90;
        assert!(SurfaceConfig::choose(&caps, &formats(), REQUESTED, false).is_err());
    }

    #[test]
    fn fixed_surface_extent_and_image_limit_override_request() {
        let mut caps = capabilities();
        caps.current_extent = vk::Extent2D {
            width: 720,
            height: 1280,
        };
        caps.max_image_count = 2;
        let plan = SurfaceConfig::choose(&caps, &formats(), REQUESTED, false)
            .unwrap()
            .unwrap();
        assert_eq!(plan.extent, caps.current_extent);
        assert_eq!(plan.image_count, 2);
    }

    #[test]
    fn zero_surface_is_suspended() {
        let mut caps = capabilities();
        assert!(
            SurfaceConfig::choose(&caps, &formats(), vk::Extent2D::default(), false)
                .unwrap()
                .is_none()
        );
        caps.current_extent = vk::Extent2D::default();
        assert!(
            SurfaceConfig::choose(&caps, &formats(), REQUESTED, false)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn inherit_requires_native_alpha_contract_and_opaque_is_not_substituted() {
        let mut caps = capabilities();
        caps.supported_composite_alpha =
            vk::CompositeAlphaFlagsKHR::INHERIT | vk::CompositeAlphaFlagsKHR::OPAQUE;
        assert!(SurfaceConfig::choose(&caps, &formats(), REQUESTED, false).is_err());
        assert_eq!(
            SurfaceConfig::choose(&caps, &formats(), REQUESTED, true)
                .unwrap()
                .unwrap()
                .alpha,
            vk::CompositeAlphaFlagsKHR::INHERIT
        );
        caps.supported_composite_alpha = vk::CompositeAlphaFlagsKHR::OPAQUE;
        assert!(SurfaceConfig::choose(&caps, &formats(), REQUESTED, true).is_err());
    }

    #[test]
    fn unsupported_transfer_or_format_is_rejected() {
        let mut caps = capabilities();
        caps.supported_usage_flags = vk::ImageUsageFlags::COLOR_ATTACHMENT;
        assert!(SurfaceConfig::choose(&caps, &formats(), REQUESTED, true).is_err());
        assert!(SurfaceConfig::choose(&capabilities(), &[], REQUESTED, true).is_err());
    }
}
