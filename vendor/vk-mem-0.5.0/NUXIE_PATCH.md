# Nuxie Vulkan campaign patch

This is `vk-mem` 0.5.0, used as the Rust FFI/API surface for the pinned Rive
Vulkan renderer. Its compiled native inputs are deliberately replaced with the
exact dependency authority used by upstream ref
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`:

- `vk_mem_alloc.h`: Rive dependency `VulkanMemoryAllocator_v3.3.0`, SHA-256
  `90ce12fc4a2466235a09ae02905dd0c13aee80c1bbf11b331ab61230c2ceb112`.
- Vulkan headers: Rive dependency `vulkan-sdk-1.4.321`; `vulkan_core.h`
  SHA-256 `3172a758081a172f4c46771a88046a53d5791ef2bd24329ff4bd7ed8c6f12abd`;
  deterministic 49-file include-tree SHA-256
  `cf45a9c19d3db4f56da563c2ca5a70ab350be6b7858a37b262b32f97b6fbe98c`.
- `VMA_STATIC_VULKAN_FUNCTIONS=0` and `VMA_DYNAMIC_VULKAN_FUNCTIONS=1`, matching
  `renderer/premake5_pls_renderer.lua` exactly.

`wrapper.cpp` remains the source crate's two-line implementation owner and is
semantically identical to Rive's `vulkan_memory_allocator.cpp`.
