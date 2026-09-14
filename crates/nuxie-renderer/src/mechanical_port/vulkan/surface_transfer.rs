//! GPU-only transfer from the resolved target into an acquired surface image.
use crate::RendererError;
use ash::vk;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SurfaceTransfer {
    Copy,
    Blit,
}

impl SurfaceTransfer {
    pub(super) fn choose(
        source: vk::Format,
        destination: vk::Format,
        source_features: vk::FormatFeatureFlags,
        destination_features: vk::FormatFeatureFlags,
    ) -> Result<Self, RendererError> {
        let rgba = |format| {
            matches!(
                format,
                vk::Format::B8G8R8A8_UNORM | vk::Format::R8G8B8A8_UNORM
            )
        };
        if !rgba(source) || !rgba(destination) {
            return Err(RendererError::Unsupported(
                "surface transfer RGBA8 UNORM formats",
            ));
        }
        if !source_features.contains(vk::FormatFeatureFlags::TRANSFER_SRC)
            || !destination_features.contains(vk::FormatFeatureFlags::TRANSFER_DST)
        {
            return Err(RendererError::Unsupported(
                "surface format transfer features",
            ));
        }
        if source == destination {
            return Ok(Self::Copy);
        }
        if source_features.contains(vk::FormatFeatureFlags::BLIT_SRC)
            && destination_features.contains(vk::FormatFeatureFlags::BLIT_DST)
        {
            Ok(Self::Blit)
        } else {
            Err(RendererError::Unsupported(
                "surface format conversion blit features",
            ))
        }
    }

    /// # Safety
    /// Both single-sampled, optimal-tiling images must have the admitted formats,
    /// transfer usages, equal nonzero extents bounded by i32::MAX, and disjoint
    /// memory. Source contents must be synchronized in TRANSFER_SRC_OPTIMAL.
    /// Destination must be acquired and its semaphore waited at TRANSFER before
    /// this command executes. Destination contents are discarded. The recording
    /// command buffer and images must remain alive through submission completion.
    /// Destination is left in TRANSFER_DST_OPTIMAL; presentation release is separate.
    pub(super) unsafe fn record(
        self,
        device: &ash::Device,
        command: vk::CommandBuffer,
        source: vk::Image,
        destination: vk::Image,
        extent: vk::Extent2D,
    ) {
        let range = vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .level_count(1)
            .layer_count(1);
        let layers = vk::ImageSubresourceLayers::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .layer_count(1);
        let transition = vk::ImageMemoryBarrier::default()
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(destination)
            .subresource_range(range);
        unsafe {
            device.cmd_pipeline_barrier(
                command,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[transition
                    .old_layout(vk::ImageLayout::UNDEFINED)
                    .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)],
            );
            match self {
                Self::Copy => device.cmd_copy_image(
                    command,
                    source,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    destination,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &[vk::ImageCopy::default()
                        .src_subresource(layers)
                        .dst_subresource(layers)
                        .extent(vk::Extent3D {
                            width: extent.width,
                            height: extent.height,
                            depth: 1,
                        })],
                ),
                Self::Blit => {
                    let offsets = [
                        vk::Offset3D::default(),
                        vk::Offset3D {
                            x: extent.width as i32,
                            y: extent.height as i32,
                            z: 1,
                        },
                    ];
                    device.cmd_blit_image(
                        command,
                        source,
                        vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                        destination,
                        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        &[vk::ImageBlit::default()
                            .src_subresource(layers)
                            .src_offsets(offsets)
                            .dst_subresource(layers)
                            .dst_offsets(offsets)],
                        vk::Filter::NEAREST,
                    );
                }
            }
        }
    }

    /// # Safety
    /// The acquired destination has completed the transfer recorded above in
    /// this command buffer. It must be a presentable image, not an offscreen image.
    pub(super) unsafe fn release_for_present(
        device: &ash::Device,
        command: vk::CommandBuffer,
        destination: vk::Image,
    ) {
        let transition = vk::ImageMemoryBarrier::default()
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(destination)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .level_count(1)
                    .layer_count(1),
            );
        unsafe {
            device.cmd_pipeline_barrier(
                command,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[transition
                    .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                    .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)],
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identical_format_does_not_require_blit_support() {
        assert_eq!(
            SurfaceTransfer::choose(
                vk::Format::B8G8R8A8_UNORM,
                vk::Format::B8G8R8A8_UNORM,
                vk::FormatFeatureFlags::TRANSFER_SRC,
                vk::FormatFeatureFlags::TRANSFER_DST
            )
            .unwrap(),
            SurfaceTransfer::Copy
        );
    }
    #[test]
    fn channel_conversion_requires_blit_and_never_uses_raw_copy() {
        let src = vk::FormatFeatureFlags::TRANSFER_SRC | vk::FormatFeatureFlags::BLIT_SRC;
        let dst = vk::FormatFeatureFlags::TRANSFER_DST | vk::FormatFeatureFlags::BLIT_DST;
        assert_eq!(
            SurfaceTransfer::choose(
                vk::Format::B8G8R8A8_UNORM,
                vk::Format::R8G8B8A8_UNORM,
                src,
                dst
            )
            .unwrap(),
            SurfaceTransfer::Blit
        );
        assert!(
            SurfaceTransfer::choose(
                vk::Format::B8G8R8A8_UNORM,
                vk::Format::R8G8B8A8_UNORM,
                src,
                vk::FormatFeatureFlags::TRANSFER_DST
            )
            .is_err()
        );
        assert!(
            SurfaceTransfer::choose(
                vk::Format::B8G8R8A8_UNORM,
                vk::Format::R8G8B8A8_UNORM,
                vk::FormatFeatureFlags::BLIT_SRC,
                dst
            )
            .is_err()
        );
        assert!(
            SurfaceTransfer::choose(
                vk::Format::B8G8R8A8_UNORM,
                vk::Format::R8G8B8A8_SRGB,
                src,
                dst
            )
            .is_err()
        );
    }
}
