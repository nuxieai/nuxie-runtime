/*
 * Copyright 2023 Rive
 */

// Mechanical translation of the complete pinned source header
// renderer/src/metal/background_shader_compiler.h.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

// /*
//  * Copyright 2023 Rive
//  */
//
// #pragma once
//
// #include "rive/renderer/gpu.hpp"
// #include "rive/renderer/metal/render_context_metal_impl.h"
//
// #include <queue>
// #include <thread>
//
// #import <Metal/Metal.h>
//
// namespace rive::gpu
// {
// // Defines a job to compile a "draw" shader, with a specific set of features
// // enabled.
// struct BackgroundCompileJob
// {
//     gpu::DrawType drawType;
//     gpu::ShaderFeatures shaderFeatures;
//     gpu::InterlockMode interlockMode;
//     gpu::ShaderMiscFlags shaderMiscFlags;
//     id<MTLLibrary> compiledLibrary = nil;
// #ifdef WITH_RIVE_TOOLS
//     gpu::SynthesizedFailureType synthesizedFailureType =
//         gpu::SynthesizedFailureType::none;
// #endif
// };
//
// // Compiles "draw" shaders in a background thread, with a specific set of
// // features enabled.
// class BackgroundShaderCompiler
// {
// public:
//     using AtomicBarrierType = RenderContextMetalImpl::AtomicBarrierType;
//     using MetalFeatures = RenderContextMetalImpl::MetalFeatures;
//
//     BackgroundShaderCompiler(id<MTLDevice> gpu, MetalFeatures metalFeatures) :
//         m_gpu(gpu), m_metalFeatures(metalFeatures)
//     {}
//
//     ~BackgroundShaderCompiler();
//
//     void pushJob(const BackgroundCompileJob&);
//     bool popFinishedJob(BackgroundCompileJob* job, bool wait);
//
// private:
//     void threadMain();
//
//     const id<MTLDevice> m_gpu;
//     const MetalFeatures m_metalFeatures;
//     std::queue<BackgroundCompileJob> m_pendingJobs;
//     std::vector<BackgroundCompileJob> m_finishedJobs;
//     std::mutex m_mutex;
//     std::condition_variable m_workAddedCondition;
//     std::condition_variable m_workFinishedCondition;
//     bool m_shouldQuit = false;
//     std::thread m_compilerThread;
// };
// } // namespace rive::gpu

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

// The declaration and the out-of-line Objective-C++ implementation name one
// Rust owner.  Re-exporting it here is the Rust equivalent of including this
// header and linking `background_shader_compiler.mm`; no second queue, thread,
// or native device owner exists in the declaration module.
pub use crate::mechanical_port::source::renderer::src::metal::background_shader_compiler_mm::{
    BackgroundCompileJob, BackgroundShaderCompiler, BackgroundShaderCompilerOwner,
    GeneratedShaderSources, MetalCompileError,
    MetalCompileOptions, MetalDeviceOwner, MetalLanguageVersion,
};

pub type AtomicBarrierType = crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::AtomicBarrierType;
pub type MetalFeatures = crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::MetalFeatures;
