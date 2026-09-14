//! Swapchain ownership for the presentation-fence-capable surface path.
//! The caller retains the instance, device and native surface until this owner
//! is dropped, and confines all calls and queue access to its native lane.
use super::surface_config::SurfaceConfig;
use ash::vk;

pub(super) struct AcquiredImage {
    pub image: vk::Image,
    pub index: u32,
    pub suboptimal: bool,
}

pub(super) struct SurfaceSwapchain {
    device: ash::Device,
    loader: ash::khr::swapchain::Device,
    handle: vk::SwapchainKHR,
    images: Vec<vk::Image>,
    acquire_semaphore: vk::Semaphore,
    acquire_done: vk::Fence,
    acquire_pending: bool,
    render_done: Vec<vk::Semaphore>,
    present_done: Vec<vk::Fence>,
    present_pending: Vec<bool>,
    submit_done: vk::Fence,
    submit_pending: bool,
    acquired: Option<(u32, bool)>,
    unusable: bool,
}

impl SurfaceSwapchain {
    /// Wait before reusing the renderer's command pool or target resources.
    pub(super) fn wait_submission(&mut self) -> Result<(), vk::Result> {
        if self.submit_pending {
            unsafe {
                self.device
                    .wait_for_fences(&[self.submit_done], true, u64::MAX)?
            };
            self.submit_pending = false;
            self.acquire_pending = false;
        }
        Ok(())
    }
    /// # Safety
    /// Device extensions KHR_swapchain and EXT_swapchain_maintenance1 (including
    /// its feature) must be enabled. The surface/configuration must be admitted
    /// for this device and its graphics/present queue. All parent handles must
    /// remain valid through Drop, and no other owner may use these resources.
    pub(super) unsafe fn new(
        instance: &ash::Instance,
        device: &ash::Device,
        surface: vk::SurfaceKHR,
        config: SurfaceConfig,
    ) -> Result<Self, vk::Result> {
        let loader = ash::khr::swapchain::Device::new(instance, device);
        let create = vk::SwapchainCreateInfoKHR::default()
            .surface(surface)
            .min_image_count(config.image_count)
            .image_format(config.format.format)
            .image_color_space(config.format.color_space)
            .image_extent(config.extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::TRANSFER_DST)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(config.transform)
            .composite_alpha(config.alpha)
            .present_mode(vk::PresentModeKHR::FIFO)
            .clipped(true);
        let handle = unsafe { loader.create_swapchain(&create, None)? };
        let mut owned = Self {
            device: device.clone(),
            loader,
            handle,
            images: Vec::new(),
            acquire_semaphore: vk::Semaphore::null(),
            acquire_done: vk::Fence::null(),
            acquire_pending: false,
            render_done: Vec::new(),
            present_done: Vec::new(),
            present_pending: Vec::new(),
            submit_done: vk::Fence::null(),
            submit_pending: false,
            acquired: None,
            unusable: false,
        };
        owned.images = unsafe { owned.loader.get_swapchain_images(handle)? };
        owned.acquire_semaphore =
            unsafe { device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None)? };
        owned.acquire_done = unsafe { device.create_fence(&vk::FenceCreateInfo::default(), None)? };
        owned.submit_done = unsafe { device.create_fence(&vk::FenceCreateInfo::default(), None)? };
        for _ in &owned.images {
            owned.render_done.push(unsafe {
                device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None)?
            });
            owned
                .present_done
                .push(unsafe { device.create_fence(&vk::FenceCreateInfo::default(), None)? });
            owned.present_pending.push(false);
        }
        Ok(owned)
    }

    /// Returns None when no image is ready within the caller's finite budget.
    /// OUT_OF_DATE and SURFACE_LOST stay distinct for the surface owner.
    pub(super) fn acquire(&mut self, timeout_ns: u64) -> Result<Option<AcquiredImage>, vk::Result> {
        if self.unusable || self.acquired.is_some() {
            return Err(vk::Result::ERROR_INITIALIZATION_FAILED);
        }
        if self.submit_pending {
            match unsafe {
                self.device
                    .wait_for_fences(&[self.submit_done], true, timeout_ns)
            } {
                Ok(()) => {
                    self.submit_pending = false;
                    self.acquire_pending = false;
                }
                Err(vk::Result::TIMEOUT) => return Ok(None),
                Err(error) => return Err(error),
            }
        }
        unsafe { self.device.reset_fences(&[self.acquire_done])? };
        let (index, suboptimal) = match unsafe {
            self.loader.acquire_next_image(
                self.handle,
                timeout_ns,
                self.acquire_semaphore,
                self.acquire_done,
            )
        } {
            Ok(image) => image,
            Err(vk::Result::TIMEOUT | vk::Result::NOT_READY) => return Ok(None),
            Err(error) => return Err(error),
        };
        self.acquire_pending = true;
        // Per-image presentation fences guard reuse as well as final teardown.
        if self.present_pending[index as usize] {
            if let Err(error) = unsafe {
                self.device
                    .wait_for_fences(&[self.present_done[index as usize]], true, u64::MAX)
            } {
                self.unusable = true;
                return Err(error);
            }
            self.present_pending[index as usize] = false;
        }
        self.acquired = Some((index, suboptimal));
        Ok(Some(AcquiredImage {
            image: self.images[index as usize],
            index,
            suboptimal,
        }))
    }

    /// # Safety
    /// The executable command buffer must use the acquired image only after
    /// TRANSFER-stage acquisition, initialize its contents, and leave it in
    /// PRESENT_SRC_KHR. It and its pool remain alive until the next successful
    /// acquire or this owner's Drop. Queue access must be externally serialized.
    pub(super) unsafe fn submit_and_present(
        &mut self,
        queue: vk::Queue,
        command: vk::CommandBuffer,
    ) -> Result<bool, vk::Result> {
        let Some((index, acquired_suboptimal)) = self.acquired.take() else {
            return Err(vk::Result::ERROR_INITIALIZATION_FAILED);
        };
        // A failure after consuming acquisition retires this owner. In
        // particular, never reuse a signaled binary semaphore after an error.
        self.unusable = true;
        let index_usize = index as usize;
        unsafe {
            self.device
                .reset_fences(&[self.submit_done, self.present_done[index_usize]])?
        };
        let waits = [self.acquire_semaphore];
        let stages = [vk::PipelineStageFlags::TRANSFER];
        let signals = [self.render_done[index_usize]];
        let commands = [command];
        let submit = vk::SubmitInfo::default()
            .wait_semaphores(&waits)
            .wait_dst_stage_mask(&stages)
            .command_buffers(&commands)
            .signal_semaphores(&signals);
        unsafe {
            self.device
                .queue_submit(queue, &[submit], self.submit_done)?
        };
        self.submit_pending = true;
        let handles = [self.handle];
        let indices = [index];
        let fences = [self.present_done[index_usize]];
        let mut completion = vk::SwapchainPresentFenceInfoEXT::default().fences(&fences);
        let present = vk::PresentInfoKHR::default()
            .wait_semaphores(&signals)
            .swapchains(&handles)
            .image_indices(&indices)
            .push_next(&mut completion);
        let result = unsafe { self.loader.queue_present(queue, &present) };
        self.present_pending[index_usize] = presentation_was_enqueued(&result);
        if result.is_ok() {
            self.unusable = false;
        }
        result.map(|suboptimal| suboptimal || acquired_suboptimal)
    }
}

fn presentation_was_enqueued(result: &Result<bool, vk::Result>) -> bool {
    matches!(
        result,
        Ok(_)
            | Err(vk::Result::ERROR_OUT_OF_DATE_KHR
                | vk::Result::ERROR_SURFACE_LOST_KHR
                | vk::Result::ERROR_FULL_SCREEN_EXCLUSIVE_MODE_LOST_EXT)
    )
}

impl Drop for SurfaceSwapchain {
    fn drop(&mut self) {
        unsafe {
            if self.submit_pending {
                let _ = self
                    .device
                    .wait_for_fences(&[self.submit_done], true, u64::MAX);
            }
            if self.acquire_pending {
                let _ = self
                    .device
                    .wait_for_fences(&[self.acquire_done], true, u64::MAX);
            }
            for (index, pending) in self.present_pending.iter().enumerate() {
                if *pending {
                    let _ =
                        self.device
                            .wait_for_fences(&[self.present_done[index]], true, u64::MAX);
                }
            }
            for fence in &self.present_done {
                self.device.destroy_fence(*fence, None);
            }
            for semaphore in &self.render_done {
                self.device.destroy_semaphore(*semaphore, None);
            }
            self.device.destroy_fence(self.submit_done, None);
            self.device.destroy_fence(self.acquire_done, None);
            self.device.destroy_semaphore(self.acquire_semaphore, None);
            self.loader.destroy_swapchain(self.handle, None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn surface_rejection_still_enqueues_present_waits() {
        for result in [
            Ok(false),
            Ok(true),
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR),
            Err(vk::Result::ERROR_SURFACE_LOST_KHR),
        ] {
            assert!(presentation_was_enqueued(&result));
        }
    }
    #[test]
    fn allocation_failure_does_not_arm_a_presentation_fence() {
        for error in [
            vk::Result::ERROR_OUT_OF_HOST_MEMORY,
            vk::Result::ERROR_OUT_OF_DEVICE_MEMORY,
            vk::Result::ERROR_DEVICE_LOST,
        ] {
            assert!(!presentation_was_enqueued(&Err(error)));
        }
    }
}
