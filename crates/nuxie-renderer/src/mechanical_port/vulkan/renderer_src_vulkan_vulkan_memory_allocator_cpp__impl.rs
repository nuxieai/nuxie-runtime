//! Complete translation of the single VMA implementation owner
//! `renderer/src/vulkan/vulkan_memory_allocator.cpp`.
//!
//! Rust cannot expand a C++ single-header implementation inside this module.
//! The `vk-mem` dependency rooted here compiles an equivalent two-line
//! `wrapper.cpp` against Rive's byte-exact VMA 3.3.0 and Vulkan 1.4.321 headers.

pub(crate) type VulkanMemoryAllocator = vk_mem::Allocator;
pub(crate) type VulkanAllocation = vk_mem::Allocation;

pub(crate) const VMA_IMPLEMENTATION: bool = true;
pub(crate) const VMA_STATIC_VULKAN_FUNCTIONS: u8 = 0;
pub(crate) const VMA_DYNAMIC_VULKAN_FUNCTIONS: u8 = 1;
pub(crate) const VMA_HEADER_SHA256: &str =
    "90ce12fc4a2466235a09ae02905dd0c13aee80c1bbf11b331ab61230c2ceb112";
pub(crate) const VULKAN_CORE_HEADER_SHA256: &str =
    "3172a758081a172f4c46771a88046a53d5791ef2bd24329ff4bd7ed8c6f12abd";

const _: fn(VulkanMemoryAllocator) = |allocator| drop(allocator);
const _: fn(VulkanAllocation) = |allocation| drop(allocation);
