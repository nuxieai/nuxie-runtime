//! Mechanical translation of rive_vk_bootstrap/vulkan_frame_synchronizer.hpp
//! and renderer/rive_vk_bootstrap/src/vulkan_frame_synchronizer.cpp at d619bc2a.
//! The already-loaded Ash device and queue family replace VulkanDevice/Instance
//! bootstrap. Derived image owners compose this base and implement FrameImage.

#![allow(non_snake_case)]

use super::frame_sync_coordinator_decl::FrameSynchronizer;
use super::vkutil_decl::{Buffer, ImageAccess, ImageAccessAction, Mappability};
use super::vulkan_context_decl::VulkanContext;
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::IAABB;
use ash::vk;
use nuxie_ore_metal::gpu_resource::ResourceHandle;
use std::cell::{Cell, RefCell};
use std::sync::Arc;

fn reported<T>(result: Result<T, vk::Result>, message: &str) -> Result<T, vk::Result> {
    result.map_err(|error| {
        eprintln!("{message}: {error:?}");
        error
    })
}

/// The abstract image-facing portion implemented by the derived surface owner.
pub(crate) trait FrameImage {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn imageFormat(&self) -> vk::Format;
    fn imageUsageFlags(&self) -> vk::ImageUsageFlags;
    fn vkImage(&self) -> vk::Image;
    fn vkImageView(&self) -> vk::ImageView;
    fn lastAccess(&self) -> ImageAccess;
}

pub(crate) struct Options {
    pub(crate) initialFrameNumber: u64,
    pub(crate) inFlightFrameCount: u32,
    pub(crate) externalGPUSynchronization: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            initialFrameNumber: 0,
            inFlightFrameCount: 2,
            externalGPUSynchronization: false,
        }
    }
}

#[derive(Default)]
pub(crate) struct InFlightFrame {
    pub(crate) fence: vk::Fence,
    pub(crate) commandBuffer: vk::CommandBuffer,
    pub(crate) semaphore: vk::Semaphore,
    pub(crate) safeFrameNumber: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PixelReadState {
    None,
    Queued,
    Ready,
}

/// The slice borrows the synchronizer, preventing slot reuse or buffer teardown
/// while the zero-copy read is live. Call finishPixelRead after releasing it.
pub(crate) struct MappedPixelRead<'a> {
    pub(crate) data: &'a [u8],
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) strideBytes: u32,
    pub(crate) format: vk::Format,
}

pub(crate) struct VulkanFrameSynchronizer {
    m_pixelReadBuffer: Option<ResourceHandle<Buffer>>,
    m_vk: Arc<VulkanContext>,
    m_device: ash::Device,
    m_pixelReadWidth: u32,
    m_pixelReadHeight: u32,
    m_pixelReadFormat: vk::Format,
    m_pixelReadState: PixelReadState,
    m_graphicsQueue: vk::Queue,
    m_commandPool: vk::CommandPool,
    m_monotonicFrameNumber: u64,
    m_renderFrameIndex: usize,
    m_inFlightFrames: Vec<InFlightFrame>,
    m_isFrameStarted: bool,
    m_isMostRecentFrameDone: Cell<bool>,
}

impl VulkanFrameSynchronizer {
    /// # Safety
    /// The loaded device must match the context and remain alive through drop.
    /// The enclosing image owner must wait for GPU work to become idle before
    /// dropping this base, as the upstream derived destructor does. Callers
    /// must externally synchronize queue access and use valid Vulkan objects.
    pub(crate) unsafe fn new(
        device: ash::Device,
        graphicsQueueFamilyIndex: u32,
        context: Arc<VulkanContext>,
        opts: &Options,
    ) -> Result<Self, vk::Result> {
        assert!(opts.inFlightFrameCount > 1);
        assert_eq!(device.handle(), context.device);
        let mut this = Self {
            m_pixelReadBuffer: None,
            m_vk: context,
            m_graphicsQueue: unsafe { device.get_device_queue(graphicsQueueFamilyIndex, 0) },
            m_device: device,
            m_pixelReadWidth: 0,
            m_pixelReadHeight: 0,
            m_pixelReadFormat: vk::Format::UNDEFINED,
            m_pixelReadState: PixelReadState::None,
            m_commandPool: vk::CommandPool::null(),
            m_monotonicFrameNumber: opts.initialFrameNumber,
            m_renderFrameIndex: 0,
            m_inFlightFrames: Vec::new(),
            m_isFrameStarted: false,
            m_isMostRecentFrameDone: Cell::new(false),
        };
        // Returning an error drops every handle that has been created so far.
        this.m_commandPool = reported(
            unsafe {
                this.m_device.create_command_pool(
                    &vk::CommandPoolCreateInfo::default()
                        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                        .queue_family_index(graphicsQueueFamilyIndex),
                    None,
                )
            },
            "Failed to create Vulkan command pool",
        )?;
        this.m_inFlightFrames
            .resize_with(opts.inFlightFrameCount as usize, InFlightFrame::default);
        for index in 0..this.m_inFlightFrames.len() {
            this.m_inFlightFrames[index].fence = reported(
                unsafe {
                    this.m_device.create_fence(
                        &vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED),
                        None,
                    )
                },
                "Failed to create Vulkan fence",
            )?;
            this.m_inFlightFrames[index].commandBuffer = reported(
                unsafe {
                    this.m_device.allocate_command_buffers(
                        &vk::CommandBufferAllocateInfo::default()
                            .command_pool(this.m_commandPool)
                            .level(vk::CommandBufferLevel::PRIMARY)
                            .command_buffer_count(1),
                    )
                },
                "Failed to allocate Vulkan command buffers",
            )?[0];
            this.m_inFlightFrames[index].semaphore = this.createSemaphore()?;
            this.m_inFlightFrames[index].safeFrameNumber = this.m_monotonicFrameNumber;
        }
        if !opts.externalGPUSynchronization {
            let signal = [this.prev().semaphore];
            reported(
                unsafe {
                    this.m_device.queue_submit(
                        this.m_graphicsQueue,
                        &[vk::SubmitInfo::default().signal_semaphores(&signal)],
                        vk::Fence::null(),
                    )
                },
                "Failed to submit Vulkan queues",
            )?;
        }
        Ok(this)
    }

    pub(crate) fn currentCommandBuffer(&self) -> vk::CommandBuffer {
        self.current().commandBuffer
    }
    pub(crate) fn safeFrameNumber(&self) -> u64 {
        self.current().safeFrameNumber
    }
    pub(crate) fn currentFrameNumber(&self) -> u64 {
        self.m_monotonicFrameNumber
    }
    pub(crate) fn graphicsQueue(&self) -> vk::Queue {
        self.m_graphicsQueue
    }
    pub(crate) fn vkDevice(&self) -> vk::Device {
        self.m_device.handle()
    }
    pub(crate) fn context(&self) -> &VulkanContext {
        &self.m_vk
    }
    pub(crate) fn current(&self) -> &InFlightFrame {
        &self.m_inFlightFrames[self.m_renderFrameIndex]
    }
    pub(crate) fn prev(&self) -> &InFlightFrame {
        &self.m_inFlightFrames[(self.m_renderFrameIndex + self.m_inFlightFrames.len() - 1)
            % self.m_inFlightFrames.len()]
    }

    pub(crate) fn waitForFenceAndBeginFrame(
        &mut self,
        optionalOutSemaphore: Option<&mut vk::Semaphore>,
    ) -> Result<(), vk::Result> {
        assert!(!self.m_isFrameStarted);
        self.m_isMostRecentFrameDone.set(false);
        unsafe {
            reported(
                self.m_device
                    .wait_for_fences(&[self.current().fence], true, u64::MAX),
                "Failed to wait for Vulkan fence for next frame",
            )?;
            reported(
                self.m_device.reset_command_buffer(
                    self.current().commandBuffer,
                    vk::CommandBufferResetFlags::empty(),
                ),
                "Failed to reset Vulkan command buffer",
            )?;
            reported(
                self.m_device.begin_command_buffer(
                    self.current().commandBuffer,
                    &vk::CommandBufferBeginInfo::default(),
                ),
                "Failed to begin Vulkan command buffer",
            )?;
        }
        self.m_monotonicFrameNumber = self.m_monotonicFrameNumber.wrapping_add(1);
        if let Some(out) = optionalOutSemaphore {
            *out = self.current().semaphore;
        }
        self.m_isFrameStarted = true;
        Ok(())
    }

    pub(crate) fn endFrame(
        &mut self,
        externalSignalSemaphore: Option<vk::Semaphore>,
    ) -> Result<(), vk::Result> {
        assert!(self.m_isFrameStarted);
        let frame = self.current();
        unsafe {
            reported(
                self.m_device.reset_fences(&[frame.fence]),
                "Failed to reset Vulkan fences",
            )?;
            reported(
                self.m_device.end_command_buffer(frame.commandBuffer),
                "Failed to end Vulkan command buffer",
            )?;
            let waits = [if externalSignalSemaphore.is_some() {
                frame.semaphore
            } else {
                self.prev().semaphore
            }];
            let signals = [externalSignalSemaphore.unwrap_or(frame.semaphore)];
            let commands = [frame.commandBuffer];
            let stages = [vk::PipelineStageFlags::ALL_COMMANDS];
            let submit = vk::SubmitInfo::default()
                .wait_semaphores(&waits)
                .wait_dst_stage_mask(&stages)
                .command_buffers(&commands)
                .signal_semaphores(&signals);
            reported(
                self.m_device
                    .queue_submit(self.m_graphicsQueue, &[submit], frame.fence),
                "Failed to submit Vulkan queue",
            )?;
        }
        self.m_inFlightFrames[self.m_renderFrameIndex].safeFrameNumber =
            self.m_monotonicFrameNumber;
        self.m_renderFrameIndex = (self.m_renderFrameIndex + 1) % self.m_inFlightFrames.len();
        if self.m_pixelReadState == PixelReadState::Queued {
            self.m_pixelReadState = PixelReadState::Ready;
        }
        self.m_isFrameStarted = false;
        Ok(())
    }

    pub(crate) fn queueImageCopy(
        &mut self,
        image: vk::Image,
        format: vk::Format,
        inOutLastAccess: &mut ImageAccess,
        pixelReadBounds: IAABB,
    ) {
        assert!(
            self.m_pixelReadState == PixelReadState::None,
            "Pixel read was while another is active."
        );
        let width = pixelReadBounds.width() as u32;
        let height = pixelReadBounds.height() as u32;
        let requiredBufferSize = (width as u64) * (height as u64) * 4;
        if self
            .m_pixelReadBuffer
            .as_ref()
            .is_none_or(|buffer| buffer.info().size < requiredBufferSize)
        {
            self.m_pixelReadBuffer = Some(
                self.m_vk.makeBuffer(
                    vk::BufferCreateInfo::default()
                        .size(requiredBufferSize)
                        .usage(vk::BufferUsageFlags::TRANSFER_DST),
                    Mappability::readWrite,
                ),
            );
        }
        let buffer = self.m_pixelReadBuffer.as_ref().unwrap();
        let command = self.current().commandBuffer;
        *inOutLastAccess = self.m_vk.simpleImageMemoryBarrier(
            command,
            *inOutLastAccess,
            ImageAccess {
                pipelineStages: vk::PipelineStageFlags::TRANSFER,
                accessMask: vk::AccessFlags::TRANSFER_READ,
                layout: vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            },
            image,
            ImageAccessAction::preserveContents,
            vk::DependencyFlags::empty(),
        );
        let copy = vk::BufferImageCopy::default()
            .image_subresource(
                vk::ImageSubresourceLayers::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .layer_count(1),
            )
            .image_offset(vk::Offset3D {
                x: pixelReadBounds.left,
                y: pixelReadBounds.top,
                z: 0,
            })
            .image_extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            });
        unsafe {
            self.m_device.cmd_copy_image_to_buffer(
                command,
                image,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                buffer.vkBuffer(),
                &[copy],
            )
        };
        self.m_vk.bufferMemoryBarrier(
            command,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::HOST,
            vk::DependencyFlags::empty(),
            vk::BufferMemoryBarrier::default()
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(vk::AccessFlags::HOST_READ)
                .buffer(buffer.vkBuffer()),
        );
        self.m_pixelReadWidth = width;
        self.m_pixelReadHeight = height;
        self.m_pixelReadFormat = format;
        self.m_pixelReadState = PixelReadState::Queued;
    }

    pub(crate) fn getPixelsFromLastImageCopy(
        &mut self,
        outPixels: &mut Vec<u8>,
    ) -> Result<(), vk::Result> {
        let read = self.waitForPixelRead()?;
        outPixels.resize(read.width as usize * read.height as usize * 4, 0);
        let stride = read.strideBytes as usize;
        for y in 0..read.height as usize {
            let src = stride * (read.height as usize - 1 - y);
            let dst = &mut outPixels[y * stride..(y + 1) * stride];
            dst.copy_from_slice(&read.data[src..src + stride]);
            if read.format == vk::Format::B8G8R8A8_UNORM {
                for x in (0..stride).step_by(4) {
                    dst.swap(x, x + 2);
                }
            }
        }
        self.finishPixelRead();
        Ok(())
    }

    pub(crate) fn waitForPixelRead(&mut self) -> Result<MappedPixelRead<'_>, vk::Result> {
        assert!(
            self.m_pixelReadState != PixelReadState::None,
            "Pixels from image copy requested without one submitted"
        );
        assert!(
            self.m_pixelReadState != PixelReadState::Queued,
            "Pixels from image copy requested before endFrame was called"
        );
        unsafe {
            self.m_device
                .wait_for_fences(&[self.prev().fence], true, u64::MAX)?
        };
        let buffer = self.m_pixelReadBuffer.as_ref().unwrap();
        buffer.invalidateAllContents();
        let length = self.m_pixelReadWidth as usize * self.m_pixelReadHeight as usize * 4;
        assert!(buffer.info().size >= length as u64);
        let data = unsafe { std::slice::from_raw_parts(buffer.contents(), length) };
        Ok(MappedPixelRead {
            data,
            width: self.m_pixelReadWidth,
            height: self.m_pixelReadHeight,
            strideBytes: self.m_pixelReadWidth * 4,
            format: self.m_pixelReadFormat,
        })
    }

    pub(crate) fn finishPixelRead(&mut self) {
        assert!(
            self.m_pixelReadState != PixelReadState::None,
            "Pixels from image copy requested without one submitted"
        );
        assert!(
            self.m_pixelReadState != PixelReadState::Queued,
            "Pixels from image copy requested before endFrame was called"
        );
        self.m_pixelReadState = PixelReadState::None;
    }

    pub(crate) fn createSemaphore(&self) -> Result<vk::Semaphore, vk::Result> {
        unsafe {
            self.m_device
                .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
        }
    }
    pub(crate) fn destroySemaphore(&self, semaphore: vk::Semaphore) {
        unsafe { self.m_device.destroy_semaphore(semaphore, None) };
    }
    pub(crate) fn checkMostRecentFrameCompletion(&self) -> bool {
        if self.m_isFrameStarted {
            assert!(!self.m_isMostRecentFrameDone.get());
            return false;
        }
        if !self.m_isMostRecentFrameDone.get()
            && matches!(
                unsafe { self.m_device.get_fence_status(self.prev().fence) },
                Ok(true)
            )
        {
            self.m_isMostRecentFrameDone.set(true);
        }
        self.m_isMostRecentFrameDone.get()
    }
}

impl Drop for VulkanFrameSynchronizer {
    fn drop(&mut self) {
        // All functions are supplied by the loaded Ash boundary. Handle guards
        // also cover every intermediate resource-creation failure.
        unsafe {
            for frame in &self.m_inFlightFrames {
                self.destroySemaphore(frame.semaphore);
                if self.m_commandPool != vk::CommandPool::null()
                    && frame.commandBuffer != vk::CommandBuffer::null()
                {
                    self.m_device
                        .free_command_buffers(self.m_commandPool, &[frame.commandBuffer]);
                }
                self.m_device.destroy_fence(frame.fence, None);
            }
            if self.m_commandPool != vk::CommandPool::null() {
                self.m_device.destroy_command_pool(self.m_commandPool, None);
            }
        }
    }
}

impl FrameSynchronizer for VulkanFrameSynchronizer {
    fn current_frame_number(&self) -> u64 {
        self.currentFrameNumber()
    }
    fn safe_frame_number(&self) -> u64 {
        self.safeFrameNumber()
    }
    fn check_most_recent_frame_completion(&self) -> bool {
        self.checkMostRecentFrameCompletion()
    }
}

impl FrameSynchronizer for RefCell<VulkanFrameSynchronizer> {
    fn current_frame_number(&self) -> u64 {
        self.borrow().currentFrameNumber()
    }
    fn safe_frame_number(&self) -> u64 {
        self.borrow().safeFrameNumber()
    }
    fn check_most_recent_frame_completion(&self) -> bool {
        self.borrow().checkMostRecentFrameCompletion()
    }
}

#[cfg(test)]
mod tests {
    use super::super::vulkan_allocation_failure_test::{reset_driver, FakeContext};
    use super::*;
    use ash::vk::Handle;

    #[derive(Default)]
    struct Driver {
        calls: Vec<&'static str>,
        fail: Option<&'static str>,
        ready: bool,
        submission: Option<(vk::Semaphore, vk::Semaphore, vk::Fence)>,
    }
    thread_local! { static DRIVER: RefCell<Driver> = RefCell::new(Driver::default()); }

    fn call(name: &'static str) -> vk::Result {
        DRIVER.with(|driver| {
            let mut driver = driver.borrow_mut();
            driver.calls.push(name);
            if driver.fail == Some(name) {
                vk::Result::ERROR_DEVICE_LOST
            } else {
                vk::Result::SUCCESS
            }
        })
    }
    fn configure(fail: Option<&'static str>, ready: bool) {
        DRIVER.with(|driver| {
            *driver.borrow_mut() = Driver {
                fail,
                ready,
                ..Default::default()
            }
        });
    }
    fn calls() -> Vec<&'static str> {
        DRIVER.with(|driver| driver.borrow().calls.clone())
    }

    unsafe extern "system" fn wait(
        _: vk::Device,
        _: u32,
        _: *const vk::Fence,
        _: vk::Bool32,
        _: u64,
    ) -> vk::Result {
        call("wait")
    }
    unsafe extern "system" fn reset_command(
        _: vk::CommandBuffer,
        _: vk::CommandBufferResetFlags,
    ) -> vk::Result {
        call("reset_command")
    }
    unsafe extern "system" fn begin(
        _: vk::CommandBuffer,
        _: *const vk::CommandBufferBeginInfo<'_>,
    ) -> vk::Result {
        call("begin")
    }
    unsafe extern "system" fn reset_fence(
        _: vk::Device,
        _: u32,
        _: *const vk::Fence,
    ) -> vk::Result {
        call("reset_fence")
    }
    unsafe extern "system" fn end(_: vk::CommandBuffer) -> vk::Result {
        call("end")
    }
    unsafe extern "system" fn submit(
        _: vk::Queue,
        _: u32,
        info: *const vk::SubmitInfo<'_>,
        fence: vk::Fence,
    ) -> vk::Result {
        let info = unsafe { &*info };
        DRIVER.with(|driver| {
            driver.borrow_mut().submission =
                Some(unsafe { (*info.p_wait_semaphores, *info.p_signal_semaphores, fence) });
        });
        call("submit")
    }
    unsafe extern "system" fn status(_: vk::Device, _: vk::Fence) -> vk::Result {
        let result = call("status");
        if result != vk::Result::SUCCESS {
            return result;
        }
        DRIVER.with(|driver| {
            if driver.borrow().ready {
                vk::Result::SUCCESS
            } else {
                vk::Result::NOT_READY
            }
        })
    }
    unsafe extern "system" fn destroy_semaphore(
        _: vk::Device,
        _: vk::Semaphore,
        _: *const vk::AllocationCallbacks<'_>,
    ) {
    }
    unsafe extern "system" fn destroy_fence(
        _: vk::Device,
        _: vk::Fence,
        _: *const vk::AllocationCallbacks<'_>,
    ) {
    }
    unsafe extern "system" fn free_commands(
        _: vk::Device,
        _: vk::CommandPool,
        _: u32,
        _: *const vk::CommandBuffer,
    ) {
    }
    unsafe extern "system" fn destroy_pool(
        _: vk::Device,
        _: vk::CommandPool,
        _: *const vk::AllocationCallbacks<'_>,
    ) {
    }

    // Reuse the real context/VMA fixture; only frame-command dispatch is fake.
    // Every resource here is a stand-in handle and no GPU work is enqueued.
    fn synchronizer(context: &Arc<VulkanContext>) -> VulkanFrameSynchronizer {
        let device = unsafe {
            ash::Device::load_with(
                |name| match name.to_bytes() {
                    b"vkWaitForFences" => wait as *const () as *const _,
                    b"vkResetCommandBuffer" => reset_command as *const () as *const _,
                    b"vkBeginCommandBuffer" => begin as *const () as *const _,
                    b"vkResetFences" => reset_fence as *const () as *const _,
                    b"vkEndCommandBuffer" => end as *const () as *const _,
                    b"vkQueueSubmit" => submit as *const () as *const _,
                    b"vkGetFenceStatus" => status as *const () as *const _,
                    b"vkDestroySemaphore" => destroy_semaphore as *const () as *const _,
                    b"vkDestroyFence" => destroy_fence as *const () as *const _,
                    b"vkFreeCommandBuffers" => free_commands as *const () as *const _,
                    b"vkDestroyCommandPool" => destroy_pool as *const () as *const _,
                    _ => std::ptr::null(),
                },
                context.device,
            )
        };
        VulkanFrameSynchronizer {
            m_pixelReadBuffer: None,
            m_vk: context.clone(),
            m_device: device,
            m_pixelReadWidth: 0,
            m_pixelReadHeight: 0,
            m_pixelReadFormat: vk::Format::UNDEFINED,
            m_pixelReadState: PixelReadState::None,
            m_graphicsQueue: vk::Queue::from_raw(1),
            m_commandPool: vk::CommandPool::from_raw(2),
            m_monotonicFrameNumber: 10,
            m_renderFrameIndex: 0,
            m_inFlightFrames: (0..2)
                .map(|i| InFlightFrame {
                    fence: vk::Fence::from_raw(3 + i),
                    commandBuffer: vk::CommandBuffer::from_raw(5 + i),
                    semaphore: vk::Semaphore::from_raw(7 + i),
                    safeFrameNumber: 10,
                })
                .collect(),
            m_isFrameStarted: false,
            m_isMostRecentFrameDone: Cell::new(false),
        }
    }

    #[test]
    fn completion_is_polled_until_success_then_cached_until_begin() {
        let _lock = reset_driver();
        let context = FakeContext::new();
        let mut sync = synchronizer(&context);
        configure(None, false);
        assert!(!sync.checkMostRecentFrameCompletion());
        assert!(!sync.checkMostRecentFrameCompletion());
        assert_eq!(calls(), ["status", "status"]);
        configure(Some("status"), true);
        assert!(!sync.checkMostRecentFrameCompletion());
        configure(None, true);
        assert!(sync.checkMostRecentFrameCompletion());
        assert!(sync.checkMostRecentFrameCompletion());
        assert_eq!(calls(), ["status"]);
        sync.waitForFenceAndBeginFrame(None).unwrap();
        assert!(!sync.checkMostRecentFrameCompletion());
        assert_eq!(calls(), ["status", "wait", "reset_command", "begin"]);
        sync.endFrame(None).unwrap();
        assert!(sync.checkMostRecentFrameCompletion());
        assert_eq!(calls().last(), Some(&"status"));
    }

    #[test]
    fn begin_failures_stop_in_order_without_advancing_the_frame() {
        let _lock = reset_driver();
        let context = FakeContext::new();
        for (failure, expected) in [
            ("wait", &["wait"][..]),
            ("reset_command", &["wait", "reset_command"][..]),
            ("begin", &["wait", "reset_command", "begin"][..]),
        ] {
            let mut sync = synchronizer(&context);
            sync.m_isMostRecentFrameDone.set(true);
            let mut semaphore = vk::Semaphore::null();
            configure(Some(failure), true);
            assert_eq!(
                sync.waitForFenceAndBeginFrame(Some(&mut semaphore)),
                Err(vk::Result::ERROR_DEVICE_LOST)
            );
            assert_eq!(calls(), expected);
            assert_eq!(sync.currentFrameNumber(), 10);
            assert_eq!(sync.m_renderFrameIndex, 0);
            assert!(!sync.m_isFrameStarted);
            assert!(!sync.m_isMostRecentFrameDone.get());
            assert_eq!(semaphore, vk::Semaphore::null());
        }
    }

    #[test]
    fn end_failures_keep_started_slot_and_queued_read_until_submission_succeeds() {
        let _lock = reset_driver();
        let context = FakeContext::new();
        for (failure, expected) in [
            ("reset_fence", &["reset_fence"][..]),
            ("end", &["reset_fence", "end"][..]),
            ("submit", &["reset_fence", "end", "submit"][..]),
        ] {
            let mut sync = synchronizer(&context);
            configure(None, false);
            sync.waitForFenceAndBeginFrame(None).unwrap();
            sync.m_pixelReadState = PixelReadState::Queued;
            configure(Some(failure), false);
            assert_eq!(sync.endFrame(None), Err(vk::Result::ERROR_DEVICE_LOST));
            assert_eq!(calls(), expected);
            assert_eq!(sync.currentFrameNumber(), 11);
            assert_eq!(sync.current().safeFrameNumber, 10);
            assert_eq!(sync.m_renderFrameIndex, 0);
            assert!(sync.m_isFrameStarted);
            assert!(sync.m_pixelReadState == PixelReadState::Queued);
        }
    }

    #[test]
    fn successful_end_selects_semaphore_chain_and_advances_only_after_submit() {
        let _lock = reset_driver();
        let context = FakeContext::new();
        for external in [None, Some(vk::Semaphore::from_raw(20))] {
            let mut sync = synchronizer(&context);
            configure(None, false);
            let mut acquired = vk::Semaphore::null();
            sync.waitForFenceAndBeginFrame(Some(&mut acquired)).unwrap();
            assert_eq!(acquired, vk::Semaphore::from_raw(7));
            sync.m_pixelReadState = PixelReadState::Queued;
            configure(None, false);
            sync.endFrame(external).unwrap();
            assert_eq!(calls(), ["reset_fence", "end", "submit"]);
            let expected = if let Some(signal) = external {
                (vk::Semaphore::from_raw(7), signal, vk::Fence::from_raw(3))
            } else {
                (
                    vk::Semaphore::from_raw(8),
                    vk::Semaphore::from_raw(7),
                    vk::Fence::from_raw(3),
                )
            };
            assert_eq!(
                DRIVER.with(|driver| driver.borrow().submission),
                Some(expected)
            );
            assert_eq!(sync.m_renderFrameIndex, 1);
            assert_eq!(sync.prev().safeFrameNumber, 11);
            assert_eq!(sync.safeFrameNumber(), 10);
            assert!(!sync.m_isFrameStarted);
            assert!(sync.m_pixelReadState == PixelReadState::Ready);
        }
    }
}
