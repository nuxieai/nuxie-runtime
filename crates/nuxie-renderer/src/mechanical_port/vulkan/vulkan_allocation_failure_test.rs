//! Mechanical translation of `tests/unit_tests/renderer/vulkan_allocation_failure_test.cpp`
//! at upstream `ecfc9746dc5bc72f1311cdd561c979b65bc11d8a`.
//! These tests use the real VulkanContext and VMA with a stand-in driver, not a GPU.

use super::vkutil_decl::Mappability;
use super::vulkan_context_decl::{AllocationFailureScope, VulkanContext, VulkanFeatures};
use ash::vk::{self, Handle};
use std::ffi::{CStr, c_char, c_void};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

static DRIVER_LOCK: Mutex<()> = Mutex::new(());
static CREATE_RENDER_PASS: AtomicI32 = AtomicI32::new(0);
static CREATE_SAMPLER: AtomicI32 = AtomicI32::new(0);
static CREATE_BUFFER: AtomicI32 = AtomicI32::new(0);
static MAP_MEMORY: AtomicI32 = AtomicI32::new(0);
const LIVE: u64 = 0x1000;
const POISON: u64 = 0xDEADBEEF;

fn reset_driver() -> MutexGuard<'static, ()> {
    let lock = DRIVER_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    for failure in [
        &CREATE_RENDER_PASS,
        &CREATE_SAMPLER,
        &CREATE_BUFFER,
        &MAP_MEMORY,
    ] {
        failure.store(vk::Result::SUCCESS.as_raw(), Ordering::Relaxed);
    }
    lock
}

macro_rules! fake_create {
    ($name:ident, $info:ident, $handle:ident, $failure:ident) => {
        unsafe extern "system" fn $name(
            _: vk::Device,
            _: *const vk::$info<'_>,
            _: *const vk::AllocationCallbacks<'_>,
            out: *mut vk::$handle,
        ) -> vk::Result {
            let result = vk::Result::from_raw($failure.load(Ordering::Relaxed));
            unsafe {
                *out = vk::$handle::from_raw(if result == vk::Result::SUCCESS {
                    LIVE
                } else {
                    POISON
                });
            }
            result
        }
    };
}
fake_create!(
    create_render_pass,
    RenderPassCreateInfo,
    RenderPass,
    CREATE_RENDER_PASS
);
fake_create!(create_sampler, SamplerCreateInfo, Sampler, CREATE_SAMPLER);
fake_create!(create_buffer, BufferCreateInfo, Buffer, CREATE_BUFFER);

unsafe extern "system" fn physical_device_properties(
    _: vk::PhysicalDevice,
    props: *mut vk::PhysicalDeviceProperties,
) {
    let mut value = vk::PhysicalDeviceProperties::default();
    value.api_version = vk::API_VERSION_1_1;
    value.vendor_id = 0x1234;
    value.limits.buffer_image_granularity = 1;
    value.limits.max_memory_allocation_count = 4096;
    value.limits.non_coherent_atom_size = 1;
    for (target, source) in value.device_name.iter_mut().zip(b"Rive fake driver\0") {
        *target = *source as c_char;
    }
    unsafe {
        *props = value;
    }
}

unsafe extern "system" fn memory_properties(
    _: vk::PhysicalDevice,
    props: *mut vk::PhysicalDeviceMemoryProperties,
) {
    let mut value = vk::PhysicalDeviceMemoryProperties::default();
    value.memory_heap_count = 1;
    value.memory_heaps[0].size = 256 * 1024 * 1024;
    value.memory_heaps[0].flags = vk::MemoryHeapFlags::DEVICE_LOCAL;
    value.memory_type_count = 2;
    value.memory_types[0].property_flags = vk::MemoryPropertyFlags::DEVICE_LOCAL;
    value.memory_types[1].property_flags =
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
    unsafe {
        *props = value;
    }
}

unsafe extern "system" fn memory_properties2(
    device: vk::PhysicalDevice,
    props: *mut vk::PhysicalDeviceMemoryProperties2<'_>,
) {
    unsafe {
        memory_properties(device, &mut (*props).memory_properties);
    }
}

unsafe extern "system" fn format_properties(
    _: vk::PhysicalDevice,
    _: vk::Format,
    props: *mut vk::FormatProperties,
) {
    unsafe {
        *props = vk::FormatProperties {
            optimal_tiling_features: vk::FormatFeatureFlags::from_raw(!0),
            ..Default::default()
        };
    }
}

unsafe extern "system" fn physical_device_features(
    _: vk::PhysicalDevice,
    features: *mut vk::PhysicalDeviceFeatures,
) {
    unsafe {
        *features = vk::PhysicalDeviceFeatures::default();
    }
}

unsafe extern "system" fn allocate_memory(
    _: vk::Device,
    _: *const vk::MemoryAllocateInfo<'_>,
    _: *const vk::AllocationCallbacks<'_>,
    memory: *mut vk::DeviceMemory,
) -> vk::Result {
    unsafe {
        *memory = vk::DeviceMemory::from_raw(LIVE);
    }
    vk::Result::SUCCESS
}

unsafe extern "system" fn buffer_requirements(
    _: vk::Device,
    _: vk::Buffer,
    requirements: *mut vk::MemoryRequirements,
) {
    unsafe {
        *requirements = vk::MemoryRequirements {
            size: 256,
            alignment: 16,
            memory_type_bits: !0,
        };
    }
}

unsafe extern "system" fn image_requirements(
    _: vk::Device,
    _: vk::Image,
    requirements: *mut vk::MemoryRequirements,
) {
    unsafe {
        *requirements = vk::MemoryRequirements {
            size: 256,
            alignment: 16,
            memory_type_bits: !0,
        };
    }
}

unsafe extern "system" fn buffer_requirements2(
    device: vk::Device,
    info: *const vk::BufferMemoryRequirementsInfo2<'_>,
    requirements: *mut vk::MemoryRequirements2<'_>,
) {
    unsafe {
        buffer_requirements(
            device,
            (*info).buffer,
            &mut (*requirements).memory_requirements,
        );
    }
}

unsafe extern "system" fn image_requirements2(
    device: vk::Device,
    info: *const vk::ImageMemoryRequirementsInfo2<'_>,
    requirements: *mut vk::MemoryRequirements2<'_>,
) {
    unsafe {
        image_requirements(
            device,
            (*info).image,
            &mut (*requirements).memory_requirements,
        );
    }
}

unsafe extern "system" fn create_image(
    _: vk::Device,
    _: *const vk::ImageCreateInfo<'_>,
    _: *const vk::AllocationCallbacks<'_>,
    image: *mut vk::Image,
) -> vk::Result {
    unsafe {
        *image = vk::Image::from_raw(LIVE);
    }
    vk::Result::SUCCESS
}

unsafe extern "system" fn map_memory(
    _: vk::Device,
    _: vk::DeviceMemory,
    _: vk::DeviceSize,
    _: vk::DeviceSize,
    _: vk::MemoryMapFlags,
    data: *mut *mut c_void,
) -> vk::Result {
    static mut SCRATCH: [u8; 4096] = [0; 4096];
    let result = vk::Result::from_raw(MAP_MEMORY.load(Ordering::Relaxed));
    unsafe {
        *data = if result == vk::Result::SUCCESS {
            std::ptr::addr_of_mut!(SCRATCH).cast()
        } else {
            POISON as usize as *mut c_void
        };
    }
    result
}

// Unlike C++'s fakeIgnore/fakeSucceed casts, use correctly typed Rust callbacks:
// calling a function through a mismatching signature is undefined in Rust.
unsafe extern "system" fn free_memory(
    _: vk::Device,
    _: vk::DeviceMemory,
    _: *const vk::AllocationCallbacks<'_>,
) {
}
unsafe extern "system" fn unmap_memory(_: vk::Device, _: vk::DeviceMemory) {}
unsafe extern "system" fn destroy_buffer(
    _: vk::Device,
    _: vk::Buffer,
    _: *const vk::AllocationCallbacks<'_>,
) {
}
unsafe extern "system" fn destroy_image(
    _: vk::Device,
    _: vk::Image,
    _: *const vk::AllocationCallbacks<'_>,
) {
}
unsafe extern "system" fn copy_buffer(
    _: vk::CommandBuffer,
    _: vk::Buffer,
    _: vk::Buffer,
    _: u32,
    _: *const vk::BufferCopy,
) {
}
unsafe extern "system" fn memory_ranges(
    _: vk::Device,
    _: u32,
    _: *const vk::MappedMemoryRange<'_>,
) -> vk::Result {
    vk::Result::SUCCESS
}
unsafe extern "system" fn bind_buffer(
    _: vk::Device,
    _: vk::Buffer,
    _: vk::DeviceMemory,
    _: vk::DeviceSize,
) -> vk::Result {
    vk::Result::SUCCESS
}
unsafe extern "system" fn bind_image(
    _: vk::Device,
    _: vk::Image,
    _: vk::DeviceMemory,
    _: vk::DeviceSize,
) -> vk::Result {
    vk::Result::SUCCESS
}
unsafe extern "system" fn bind_buffers2(
    _: vk::Device,
    _: u32,
    _: *const vk::BindBufferMemoryInfo<'_>,
) -> vk::Result {
    vk::Result::SUCCESS
}
unsafe extern "system" fn bind_images2(
    _: vk::Device,
    _: u32,
    _: *const vk::BindImageMemoryInfo<'_>,
) -> vk::Result {
    vk::Result::SUCCESS
}

unsafe fn resolve(name: *const c_char) -> vk::PFN_vkVoidFunction {
    let pointer = match unsafe { CStr::from_ptr(name) }.to_bytes() {
        b"vkGetDeviceProcAddr" => device_proc_addr as *const (),
        b"vkCreateRenderPass" => create_render_pass as *const (),
        b"vkCreateSampler" => create_sampler as *const (),
        b"vkCreateBuffer" => create_buffer as *const (),
        b"vkCreateImage" => create_image as *const (),
        b"vkGetPhysicalDeviceProperties" => physical_device_properties as *const (),
        b"vkGetPhysicalDeviceFeatures" => physical_device_features as *const (),
        b"vkGetPhysicalDeviceFormatProperties" => format_properties as *const (),
        b"vkGetPhysicalDeviceMemoryProperties" => memory_properties as *const (),
        b"vkGetPhysicalDeviceMemoryProperties2" | b"vkGetPhysicalDeviceMemoryProperties2KHR" => {
            memory_properties2 as *const ()
        }
        b"vkAllocateMemory" => allocate_memory as *const (),
        b"vkMapMemory" => map_memory as *const (),
        b"vkGetBufferMemoryRequirements" => buffer_requirements as *const (),
        b"vkGetImageMemoryRequirements" => image_requirements as *const (),
        b"vkGetBufferMemoryRequirements2" | b"vkGetBufferMemoryRequirements2KHR" => {
            buffer_requirements2 as *const ()
        }
        b"vkGetImageMemoryRequirements2" | b"vkGetImageMemoryRequirements2KHR" => {
            image_requirements2 as *const ()
        }
        b"vkFreeMemory" => free_memory as *const (),
        b"vkUnmapMemory" => unmap_memory as *const (),
        b"vkDestroyBuffer" => destroy_buffer as *const (),
        b"vkDestroyImage" => destroy_image as *const (),
        b"vkCmdCopyBuffer" => copy_buffer as *const (),
        b"vkFlushMappedMemoryRanges" | b"vkInvalidateMappedMemoryRanges" => {
            memory_ranges as *const ()
        }
        b"vkBindBufferMemory" => bind_buffer as *const (),
        b"vkBindImageMemory" => bind_image as *const (),
        b"vkBindBufferMemory2" | b"vkBindBufferMemory2KHR" => bind_buffers2 as *const (),
        b"vkBindImageMemory2" | b"vkBindImageMemory2KHR" => bind_images2 as *const (),
        _ => return None,
    };
    Some(unsafe { std::mem::transmute::<*const (), unsafe extern "system" fn()>(pointer) })
}

unsafe extern "system" fn device_proc_addr(
    _: vk::Device,
    name: *const c_char,
) -> vk::PFN_vkVoidFunction {
    unsafe { resolve(name) }
}
unsafe extern "system" fn instance_proc_addr(
    _: vk::Instance,
    name: *const c_char,
) -> vk::PFN_vkVoidFunction {
    unsafe { resolve(name) }
}

struct FakeContext(Arc<VulkanContext>);
impl FakeContext {
    fn new() -> Self {
        Self(
            unsafe {
                VulkanContext::make(
                    vk::Instance::from_raw(LIVE),
                    vk::PhysicalDevice::from_raw(LIVE),
                    vk::Device::from_raw(LIVE),
                    VulkanFeatures::default(),
                    instance_proc_addr,
                )
            }
            .expect("stand-in driver must support the real VMA allocator"),
        )
    }
}
impl std::ops::Deref for FakeContext {
    type Target = Arc<VulkanContext>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Drop for FakeContext {
    fn drop(&mut self) {
        self.0.shutdown();
    }
}

fn render_pass(vk: &VulkanContext) -> vk::RenderPass {
    let info = vk::RenderPassCreateInfo::default();
    vk.createHandle(
        unsafe { vk.ashDevice().create_render_pass(&info, None) },
        file!(),
        line!(),
    )
}
fn sampler(vk: &VulkanContext) -> vk::Sampler {
    let info = vk::SamplerCreateInfo::default();
    vk.createHandle(
        unsafe { vk.ashDevice().create_sampler(&info, None) },
        file!(),
        line!(),
    )
}
fn init_render_pass(vk: &VulkanContext) -> (bool, vk::RenderPass) {
    let baseline = vk.allocationFailureCount();
    let handle = render_pass(vk);
    (vk.allocationFailureCount() == baseline, handle)
}

#[test]
fn create_handle_publishes_driver_handle_on_success() {
    let _reset = reset_driver();
    let vk = FakeContext::new();
    let _scope = AllocationFailureScope::new(Arc::clone(&vk));
    assert_eq!(render_pass(&vk), vk::RenderPass::from_raw(LIVE));
    assert_eq!(vk.allocationFailureCount(), 0);
}

#[test]
fn create_handle_drops_failed_driver_scribbled_output() {
    let _reset = reset_driver();
    for failure in [
        vk::Result::ERROR_OUT_OF_HOST_MEMORY,
        vk::Result::ERROR_OUT_OF_DEVICE_MEMORY,
        vk::Result::ERROR_INITIALIZATION_FAILED,
        vk::Result::ERROR_DEVICE_LOST,
    ] {
        let vk = FakeContext::new();
        let _scope = AllocationFailureScope::new(Arc::clone(&vk));
        CREATE_RENDER_PASS.store(failure.as_raw(), Ordering::Relaxed);
        let handle = render_pass(&vk);
        assert_ne!(handle, vk::RenderPass::from_raw(POISON));
        assert_eq!(handle, vk::RenderPass::null());
        assert_eq!(vk.allocationFailureCount(), 1);
    }
}

#[test]
fn create_handle_deduces_command_handle_type() {
    let _reset = reset_driver();
    let vk = FakeContext::new();
    let _scope = AllocationFailureScope::new(Arc::clone(&vk));
    assert_eq!(sampler(&vk), vk::Sampler::from_raw(LIVE));
    CREATE_SAMPLER.store(
        vk::Result::ERROR_OUT_OF_DEVICE_MEMORY.as_raw(),
        Ordering::Relaxed,
    );
    assert_eq!(sampler(&vk), vk::Sampler::null());
    assert_eq!(vk.allocationFailureCount(), 1);
}

#[test]
fn allocation_count_baseline_bails_out_init_path() {
    let _reset = reset_driver();
    for section in 0..3 {
        CREATE_RENDER_PASS.store(0, Ordering::Relaxed);
        CREATE_SAMPLER.store(0, Ordering::Relaxed);
        let vk = FakeContext::new();
        let _scope = AllocationFailureScope::new(Arc::clone(&vk));
        if section == 1 {
            CREATE_RENDER_PASS.store(
                vk::Result::ERROR_OUT_OF_DEVICE_MEMORY.as_raw(),
                Ordering::Relaxed,
            );
        } else if section == 2 {
            CREATE_SAMPLER.store(
                vk::Result::ERROR_OUT_OF_DEVICE_MEMORY.as_raw(),
                Ordering::Relaxed,
            );
            sampler(&vk);
            assert_eq!(vk.allocationFailureCount(), 1);
        }
        let (success, handle) = init_render_pass(&vk);
        assert_eq!(success, section != 1);
        if section == 1 {
            assert_eq!(handle, vk::RenderPass::null());
        } else {
            assert_eq!(handle, vk::RenderPass::from_raw(LIVE));
        }
    }
}

fn buffer_info() -> vk::BufferCreateInfo<'static> {
    vk::BufferCreateInfo::default()
        .size(1024)
        .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
}

#[test]
fn failed_buffer_allocation_counts_and_stays_null() {
    let _reset = reset_driver();
    let vk = FakeContext::new();
    let _scope = AllocationFailureScope::new(Arc::clone(&vk));
    CREATE_BUFFER.store(
        vk::Result::ERROR_OUT_OF_DEVICE_MEMORY.as_raw(),
        Ordering::Relaxed,
    );
    let buffer = vk.makeBuffer(buffer_info(), Mappability::writeOnly);
    assert_eq!(vk.allocationFailureCount(), 1);
    assert_eq!(buffer.vkBuffer(), vk::Buffer::null());
    drop(buffer);
}

#[test]
fn buffer_that_allocates_but_cannot_map_stays_null() {
    let _reset = reset_driver();
    let vk = FakeContext::new();
    let _scope = AllocationFailureScope::new(Arc::clone(&vk));
    MAP_MEMORY.store(
        vk::Result::ERROR_MEMORY_MAP_FAILED.as_raw(),
        Ordering::Relaxed,
    );
    let buffer = vk.makeBuffer(buffer_info(), Mappability::writeOnly);
    assert_eq!(vk.allocationFailureCount(), 1);
    assert!(!buffer.hasContents());
    assert_eq!(buffer.vkBuffer(), vk::Buffer::null());
    drop(buffer);
}

#[test]
fn successful_buffer_allocation_does_not_count() {
    let _reset = reset_driver();
    let vk = FakeContext::new();
    let _scope = AllocationFailureScope::new(Arc::clone(&vk));
    let buffer = vk.makeBuffer(buffer_info(), Mappability::writeOnly);
    assert_eq!(vk.allocationFailureCount(), 0);
    assert_ne!(buffer.vkBuffer(), vk::Buffer::null());
    assert!(buffer.hasContents());
    drop(buffer);
}

#[test]
fn each_allocation_failure_scope_starts_where_it_began() {
    let _reset = reset_driver();
    let vk = FakeContext::new();
    {
        let first = AllocationFailureScope::new(Arc::clone(&vk));
        assert!(!first.anyFailed());
        CREATE_RENDER_PASS.store(
            vk::Result::ERROR_OUT_OF_DEVICE_MEMORY.as_raw(),
            Ordering::Relaxed,
        );
        assert_eq!(render_pass(&vk), vk::RenderPass::null());
        assert!(first.anyFailed());
    }
    CREATE_RENDER_PASS.store(0, Ordering::Relaxed);
    {
        let second = AllocationFailureScope::new(Arc::clone(&vk));
        assert_eq!(vk.allocationFailureCount(), 1);
        assert!(!second.anyFailed());
        assert_eq!(render_pass(&vk), vk::RenderPass::from_raw(LIVE));
        assert!(!second.anyFailed());
    }
}
