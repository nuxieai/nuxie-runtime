//! Mechanical capability-selection translation from pinned upstream
//! `renderer/src/metal/render_context_metal_impl.mm` at
//! `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Legacy selector fixture; product uses canonical source capabilities.
pub(crate) enum ApplePlatform {
    IosDevice { is_apple_silicon: bool },
    IosSimulator { host_is_arm64: bool },
    XrOsDevice { is_apple_silicon: bool },
    XrOsSimulator,
    AppleTvOsDevice { is_apple_silicon: bool },
    AppleTvOsSimulator,
    MacOs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(test)]
pub(crate) struct MetalDeviceCapabilities {
    pub(crate) supports_apple1: bool,
    pub(crate) supports_apple2: bool,
    pub(crate) supports_apple3: bool,
    pub(crate) supports_common2: bool,
    pub(crate) supports_mac2: bool,
    pub(crate) raster_order_groups: bool,
}

#[cfg(test)]
pub(crate) use super::source_capabilities::{AtomicBarrierType, MetalCapabilitySelection};

#[cfg(test)]
pub(crate) fn select_capabilities(
    platform: ApplePlatform,
    device: MetalDeviceCapabilities,
    disable_framebuffer_reads: bool,
) -> MetalCapabilitySelection {
    let max_texture_size = if device.supports_apple2 || device.supports_mac2 {
        16_384
    } else {
        8_192
    };

    let (supports_raster_ordering, supports_atomic_mode, path_id_granularity) = match platform {
        ApplePlatform::IosDevice { is_apple_silicon }
        | ApplePlatform::XrOsDevice { is_apple_silicon }
        | ApplePlatform::AppleTvOsDevice { is_apple_silicon } => {
            (true, false, if is_apple_silicon { 1 } else { 8 })
        }
        ApplePlatform::IosSimulator { .. }
        | ApplePlatform::XrOsSimulator
        | ApplePlatform::AppleTvOsSimulator => (false, true, 1),
        ApplePlatform::MacOs => (
            device.supports_apple1 && !disable_framebuffer_reads,
            true,
            1,
        ),
    };

    let (
        supports_texture_compression_etc2,
        supports_texture_compression_astc,
        supports_texture_compression_bc,
    ) = match platform {
        ApplePlatform::IosDevice { .. }
        | ApplePlatform::IosSimulator { .. }
        | ApplePlatform::XrOsDevice { .. }
        | ApplePlatform::XrOsSimulator
        | ApplePlatform::AppleTvOsDevice { .. }
        | ApplePlatform::AppleTvOsSimulator => (true, true, false),
        ApplePlatform::MacOs => (false, device.supports_apple1, true),
    };

    // The native Metal implementation uses a platform-specific barrier policy:
    // iOS uses raster-order groups, while macOS can use memory barriers on
    // Common2/Mac2 devices except Apple3 and newer. If neither path applies,
    // it breaks the render pass between overlapping atomic draws.
    let atomic_barrier_type = match platform {
        ApplePlatform::IosDevice { .. }
        | ApplePlatform::XrOsDevice { .. }
        | ApplePlatform::XrOsSimulator
        | ApplePlatform::AppleTvOsDevice { .. }
        | ApplePlatform::AppleTvOsSimulator => AtomicBarrierType::rasterOrderGroup,
        ApplePlatform::IosSimulator { host_is_arm64 } => {
            if host_is_arm64 {
                AtomicBarrierType::rasterOrderGroup
            } else {
                AtomicBarrierType::renderPassBreak
            }
        }
        ApplePlatform::MacOs => {
            if (device.supports_common2 || device.supports_mac2) && !device.supports_apple3 {
                AtomicBarrierType::memoryBarrier
            } else if device.raster_order_groups {
                AtomicBarrierType::rasterOrderGroup
            } else {
                AtomicBarrierType::renderPassBreak
            }
        }
    };

    MetalCapabilitySelection {
        max_texture_size,
        supports_raster_ordering,
        supports_atomic_mode,
        path_id_granularity,
        supports_texture_compression_etc2,
        supports_texture_compression_astc,
        supports_texture_compression_bc,
        atomic_barrier_type,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NO_FAMILIES: MetalDeviceCapabilities = MetalDeviceCapabilities {
        supports_apple1: false,
        supports_apple2: false,
        supports_apple3: false,
        supports_common2: false,
        supports_mac2: false,
        raster_order_groups: false,
    };

    #[test]
    fn selects_upstream_metal_capabilities_by_platform_and_device_family() {
        let cases = [
            (
                "modern iOS Apple silicon",
                ApplePlatform::IosDevice {
                    is_apple_silicon: true,
                },
                MetalDeviceCapabilities {
                    supports_apple1: true,
                    supports_apple2: true,
                    ..NO_FAMILIES
                },
                false,
                MetalCapabilitySelection {
                    max_texture_size: 16_384,
                    supports_raster_ordering: true,
                    supports_atomic_mode: false,
                    path_id_granularity: 1,
                    supports_texture_compression_etc2: true,
                    supports_texture_compression_astc: true,
                    supports_texture_compression_bc: false,
                    atomic_barrier_type: AtomicBarrierType::rasterOrderGroup,
                },
            ),
            (
                "legacy PowerVR iOS path granularity",
                ApplePlatform::IosDevice {
                    is_apple_silicon: false,
                },
                NO_FAMILIES,
                false,
                MetalCapabilitySelection {
                    max_texture_size: 8_192,
                    supports_raster_ordering: true,
                    supports_atomic_mode: false,
                    path_id_granularity: 8,
                    supports_texture_compression_etc2: true,
                    supports_texture_compression_astc: true,
                    supports_texture_compression_bc: false,
                    atomic_barrier_type: AtomicBarrierType::rasterOrderGroup,
                },
            ),
            (
                "arm64 simulator",
                ApplePlatform::IosSimulator {
                    host_is_arm64: true,
                },
                MetalDeviceCapabilities {
                    supports_apple2: true,
                    ..NO_FAMILIES
                },
                false,
                MetalCapabilitySelection {
                    max_texture_size: 16_384,
                    supports_raster_ordering: false,
                    supports_atomic_mode: true,
                    path_id_granularity: 1,
                    supports_texture_compression_etc2: true,
                    supports_texture_compression_astc: true,
                    supports_texture_compression_bc: false,
                    atomic_barrier_type: AtomicBarrierType::rasterOrderGroup,
                },
            ),
            (
                "x86_64 simulator",
                ApplePlatform::IosSimulator {
                    host_is_arm64: false,
                },
                NO_FAMILIES,
                false,
                MetalCapabilitySelection {
                    max_texture_size: 8_192,
                    supports_raster_ordering: false,
                    supports_atomic_mode: true,
                    path_id_granularity: 1,
                    supports_texture_compression_etc2: true,
                    supports_texture_compression_astc: true,
                    supports_texture_compression_bc: false,
                    atomic_barrier_type: AtomicBarrierType::renderPassBreak,
                },
            ),
            (
                "Apple Silicon macOS with framebuffer reads",
                ApplePlatform::MacOs,
                MetalDeviceCapabilities {
                    supports_apple1: true,
                    supports_apple3: true,
                    raster_order_groups: true,
                    ..NO_FAMILIES
                },
                false,
                MetalCapabilitySelection {
                    max_texture_size: 8_192,
                    supports_raster_ordering: true,
                    supports_atomic_mode: true,
                    path_id_granularity: 1,
                    supports_texture_compression_etc2: false,
                    supports_texture_compression_astc: true,
                    supports_texture_compression_bc: true,
                    atomic_barrier_type: AtomicBarrierType::rasterOrderGroup,
                },
            ),
            (
                "forced atomic macOS",
                ApplePlatform::MacOs,
                MetalDeviceCapabilities {
                    supports_apple1: true,
                    supports_apple3: true,
                    raster_order_groups: true,
                    ..NO_FAMILIES
                },
                true,
                MetalCapabilitySelection {
                    max_texture_size: 8_192,
                    supports_raster_ordering: false,
                    supports_atomic_mode: true,
                    path_id_granularity: 1,
                    supports_texture_compression_etc2: false,
                    supports_texture_compression_astc: true,
                    supports_texture_compression_bc: true,
                    atomic_barrier_type: AtomicBarrierType::rasterOrderGroup,
                },
            ),
            (
                "Intel Common2 memory barrier",
                ApplePlatform::MacOs,
                MetalDeviceCapabilities {
                    supports_common2: true,
                    ..NO_FAMILIES
                },
                false,
                MetalCapabilitySelection {
                    max_texture_size: 8_192,
                    supports_raster_ordering: false,
                    supports_atomic_mode: true,
                    path_id_granularity: 1,
                    supports_texture_compression_etc2: false,
                    supports_texture_compression_astc: false,
                    supports_texture_compression_bc: true,
                    atomic_barrier_type: AtomicBarrierType::memoryBarrier,
                },
            ),
            (
                "AMD Mac2 memory barrier",
                ApplePlatform::MacOs,
                MetalDeviceCapabilities {
                    supports_mac2: true,
                    ..NO_FAMILIES
                },
                false,
                MetalCapabilitySelection {
                    max_texture_size: 16_384,
                    supports_raster_ordering: false,
                    supports_atomic_mode: true,
                    path_id_granularity: 1,
                    supports_texture_compression_etc2: false,
                    supports_texture_compression_astc: false,
                    supports_texture_compression_bc: true,
                    atomic_barrier_type: AtomicBarrierType::memoryBarrier,
                },
            ),
            (
                "old macOS render-pass break",
                ApplePlatform::MacOs,
                NO_FAMILIES,
                false,
                MetalCapabilitySelection {
                    max_texture_size: 8_192,
                    supports_raster_ordering: false,
                    supports_atomic_mode: true,
                    path_id_granularity: 1,
                    supports_texture_compression_etc2: false,
                    supports_texture_compression_astc: false,
                    supports_texture_compression_bc: true,
                    atomic_barrier_type: AtomicBarrierType::renderPassBreak,
                },
            ),
        ];

        for (name, platform, device, disable_framebuffer_reads, expected) in cases {
            assert_eq!(
                select_capabilities(platform, device, disable_framebuffer_reads),
                expected,
                "capability selection mismatch for {name}"
            );
        }
    }
}
