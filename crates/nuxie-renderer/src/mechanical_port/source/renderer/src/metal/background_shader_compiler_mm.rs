/*
 * Mechanical source-shaped translation of the complete pinned
 * renderer/src/metal/background_shader_compiler.mm.
 *
 * The complete Objective-C++ source is retained verbatim below for line-level
 * audit. The Rust owner follows its queue, worker, source assembly, Metal
 * compile, diagnostic, failure, and destruction order.
 *
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use std::collections::VecDeque;
use std::marker::PhantomPinned;
use std::mem::ManuallyDrop;
use std::pin::Pin;
use std::sync::{Condvar, Mutex, MutexGuard};
use std::thread::{self, JoinHandle};

#[inline(never)]
fn rive_unreachable() -> ! {
    #[cfg(debug_assertions)]
    panic!("RIVE_UNREACHABLE");
    #[cfg(not(debug_assertions))]
    unsafe {
        core::hint::unreachable_unchecked()
    }
}

/// C `assert(false)` with exact NDEBUG behavior at a worker-thread boundary.
/// A normal Rust `debug_assert!` would only unwind this thread, while the
/// authored C assertion terminates the process.
macro_rules! debug_assert_abort {
    () => {{
        #[cfg(debug_assertions)]
        std::process::abort();
    }};
}

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/metal/background_shader_compiler.mm";
pub const PINNED_SOURCE_SHA256: &str =
    "7618c621c233aa090935acc98cd484a497dcb82a96d28036f18713499b01af4a";
pub const PINNED_SOURCE_LINE_COUNT: usize = 346;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 12612;
pub const TRANSLATION_UNIT: &str = "metal-background-shader-compiler";
pub const TRANSLATION_TARGET: &str = "crates/nuxie-renderer/src/mechanical_port/source/renderer/src/metal/background_shader_compiler_mm.rs";
pub const TRANSLATION_DISPOSITION: &str = "required";
pub const TRANSLATION_BEHAVIOR: &str =
    "preserve-background-compiler-queue-and-native-compile-semantics";

/// Production worker-boundary evidence.  These events are emitted by the
/// native iteration owner itself; the renderer ownership gate only consumes
/// this stream and never fabricates background rows from a selector table.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackgroundOwnerPhase {
    Create,
    Borrow,
    Transfer,
    Release,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BackgroundOwnerEvent {
    pub ledger_id: &'static str,
    pub phase: BackgroundOwnerPhase,
    pub identity: usize,
}

#[cfg(test)]
static BACKGROUND_OWNER_EVENTS: std::sync::OnceLock<std::sync::Mutex<Vec<BackgroundOwnerEvent>>> =
    std::sync::OnceLock::new();

/// Test-only, source-boundary detail stream for the ten BG rows in the owner
/// expectations table.  `BackgroundOwnerEvent` is intentionally kept as the
/// small compatibility stream consumed by the renderer-wide gate; this
/// stream carries the finer lexical phases without adding test state to the
/// production owner.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BackgroundOwnerDetailEvent {
    pub ledger_id: &'static str,
    pub phase: &'static str,
    pub identity: usize,
    /// Native identity related to this expression owner. For appendFormat
    /// rows this is the mutable source receiver while `identity` is the
    /// immortal NSString format literal.
    pub related_identity: Option<usize>,
}

#[cfg(test)]
static BACKGROUND_OWNER_DETAIL_EVENTS: std::sync::OnceLock<
    std::sync::Mutex<Vec<BackgroundOwnerDetailEvent>>,
> = std::sync::OnceLock::new();

#[cfg(test)]
fn owner_event(ledger_id: &'static str, phase: BackgroundOwnerPhase, identity: usize) {
    if let Ok(mut events) = BACKGROUND_OWNER_EVENTS
        .get_or_init(|| std::sync::Mutex::new(Vec::new()))
        .lock()
    {
        events.push(BackgroundOwnerEvent {
            ledger_id,
            phase,
            identity,
        });
    }
}

#[cfg(test)]
fn owner_detail_event(ledger_id: &'static str, phase: &'static str, identity: usize) {
    if let Ok(mut events) = BACKGROUND_OWNER_DETAIL_EVENTS
        .get_or_init(|| std::sync::Mutex::new(Vec::new()))
        .lock()
    {
        events.push(BackgroundOwnerDetailEvent {
            ledger_id,
            phase,
            identity,
            related_identity: None,
        });
    }
}

#[cfg(test)]
pub(crate) fn record_compiled_library_context_phase(phase: &'static str, identity: usize) {
    owner_detail_event("BG-LIB-COMPILED", phase, identity);
}

#[cfg(test)]
fn owner_detail_event_related(
    ledger_id: &'static str,
    phase: &'static str,
    identity: usize,
    related_identity: usize,
) {
    if let Ok(mut events) = BACKGROUND_OWNER_DETAIL_EVENTS
        .get_or_init(|| std::sync::Mutex::new(Vec::new()))
        .lock()
    {
        events.push(BackgroundOwnerDetailEvent {
            ledger_id,
            phase,
            identity,
            related_identity: Some(related_identity),
        });
    }
}

#[cfg(not(test))]
#[inline]
fn owner_detail_event(_ledger_id: &'static str, _phase: &'static str, _identity: usize) {}

#[cfg(not(test))]
#[inline]
fn owner_detail_event_related(
    _ledger_id: &'static str,
    _phase: &'static str,
    _identity: usize,
    _related_identity: usize,
) {
}

#[cfg(not(test))]
#[inline]
fn owner_event(_ledger_id: &'static str, _phase: BackgroundOwnerPhase, _identity: usize) {}

#[cfg(test)]
pub fn take_owner_events() -> Vec<BackgroundOwnerEvent> {
    BACKGROUND_OWNER_EVENTS
        .get_or_init(|| std::sync::Mutex::new(Vec::new()))
        .lock()
        .map(|mut events| std::mem::take(&mut *events))
        .unwrap_or_default()
}

#[cfg(test)]
pub fn take_owner_detail_events() -> Vec<BackgroundOwnerDetailEvent> {
    BACKGROUND_OWNER_DETAIL_EVENTS
        .get_or_init(|| std::sync::Mutex::new(Vec::new()))
        .lock()
        .map(|mut events| std::mem::take(&mut *events))
        .unwrap_or_default()
}

/// Exact pinned Objective-C++ source, retained for provenance and line audit.
pub const PINNED_BACKGROUND_SHADER_COMPILER_MM_SOURCE: &str = r####"/*
 * Copyright 2023 Rive
 */

#include "background_shader_compiler.h"

#include "generated/shaders/metal.glsl.hpp"
#include "generated/shaders/constants.glsl.hpp"
#include "generated/shaders/flush_uniforms.glsl.hpp"
#include "generated/shaders/common.glsl.hpp"
#include "generated/shaders/advanced_blend.glsl.hpp"
#include "generated/shaders/draw_path_common.glsl.hpp"
#include "generated/shaders/draw_path.vert.hpp"
#include "generated/shaders/draw_raster_order_path.frag.hpp"
#include "generated/shaders/draw_image_mesh.vert.hpp"
#include "generated/shaders/draw_mesh.frag.hpp"

#ifndef RIVE_IOS
// iOS doesn't need the atomic shaders; every non-simulated iOS device supports
// framebuffer reads.
#include "generated/shaders/atomic_draw.glsl.hpp"
#endif

#include <sstream>

namespace rive::gpu
{
BackgroundShaderCompiler::~BackgroundShaderCompiler()
{
    if (m_compilerThread.joinable())
    {
        {
            std::lock_guard lock(m_mutex);
            m_shouldQuit = true;
        }

        m_workAddedCondition.notify_all();
        m_compilerThread.join();
    }
}

void BackgroundShaderCompiler::pushJob(const BackgroundCompileJob& job)
{
    {
        std::lock_guard lock(m_mutex);
        if (!m_compilerThread.joinable())
        {
            m_compilerThread =
                std::thread(&BackgroundShaderCompiler::threadMain, this);
        }
        m_pendingJobs.push(std::move(job));
    }
    m_workAddedCondition.notify_all();
}

bool BackgroundShaderCompiler::popFinishedJob(BackgroundCompileJob* job,
                                              bool wait)
{
    std::unique_lock lock(m_mutex);
    while (m_finishedJobs.empty())
    {
        if (!wait)
        {
            return false;
        }
        m_workFinishedCondition.wait(lock);
    }
    *job = std::move(m_finishedJobs.back());
    m_finishedJobs.pop_back();
    return true;
}

void BackgroundShaderCompiler::threadMain()
{
    BackgroundCompileJob job;
    std::unique_lock lock(m_mutex);
    for (;;)
    {
        while (m_pendingJobs.empty() && !m_shouldQuit)
        {
            m_workAddedCondition.wait(lock);
        }

        if (m_shouldQuit)
        {
            return;
        }

        job = std::move(m_pendingJobs.front());
        m_pendingJobs.pop();

        lock.unlock();

        gpu::DrawType drawType = job.drawType;
        gpu::ShaderFeatures shaderFeatures = job.shaderFeatures;
        gpu::InterlockMode interlockMode = job.interlockMode;
        gpu::ShaderMiscFlags shaderMiscFlags = job.shaderMiscFlags;

        auto defines = [[NSMutableDictionary alloc] init];
        defines[@GLSL_VERTEX] = @"";
        defines[@GLSL_FRAGMENT] = @"";
        for (size_t i = 0; i < gpu::kShaderFeatureCount; ++i)
        {
            const auto feature = ShaderFeatures(1 << i);
            if (enums::is_flag_set(shaderFeatures, feature))
            {
                const char* macro = gpu::GetShaderFeatureGLSLName(feature);
                defines[[NSString stringWithUTF8String:macro]] = @"1";
            }
        }
        if (interlockMode == gpu::InterlockMode::atomics)
        {
            // Atomic mode uses device buffers instead of framebuffer fetches.
            defines[@GLSL_PLS_IMPL_DEVICE_BUFFER] = @"";
            if (m_metalFeatures.atomicBarrierType ==
                AtomicBarrierType::rasterOrderGroup)
            {
                defines[@GLSL_PLS_IMPL_DEVICE_BUFFER_RASTER_ORDERED] = @"";
            }
            if (enums::is_flag_set(
                    shaderMiscFlags,
                    gpu::ShaderMiscFlags::fixedFunctionColorOutput))
            {
                defines[@GLSL_FIXED_FUNCTION_COLOR_OUTPUT] = @"";
            }
        }
        if (enums::is_flag_set(shaderMiscFlags,
                               gpu::ShaderMiscFlags::clockwiseFill))
        {
            defines[@GLSL_CLOCKWISE_FILL] = @"1";
        }

        auto source =
            [[NSMutableString alloc] initWithCString:gpu::glsl::metal
                                            encoding:NSUTF8StringEncoding];
        [source appendFormat:@"%s\n%s\n%s\n",
                             gpu::glsl::constants,
                             gpu::glsl::flush_uniforms,
                             gpu::glsl::common];
        if (enums::is_flag_set(shaderFeatures,
                               ShaderFeatures::ENABLE_ADVANCED_BLEND))
        {
            [source appendFormat:@"%s\n", gpu::glsl::advanced_blend];
        }

        switch (drawType)
        {
            case DrawType::midpointFanPatches:
            case DrawType::midpointFanCenterAAPatches:
            case DrawType::outerCurvePatches:
                // Add baseInstance to the instanceID for path draws.
                defines[@GLSL_ENABLE_INSTANCE_INDEX] = @"";
                defines[@GLSL_DRAW_PATH] = @"";
                break;
            case DrawType::interiorTriangulation:
                defines[@GLSL_DRAW_INTERIOR_TRIANGLES] = @"";
                break;
            case DrawType::featherAtlasBlit:
                defines[@GLSL_FEATHER_ATLAS_BLIT] = @"1";
                break;
            case DrawType::imageRect:
#ifdef RIVE_IOS
                RIVE_UNREACHABLE();
#else
                assert(interlockMode == InterlockMode::atomics);
                defines[@GLSL_DRAW_IMAGE] = @"";
                defines[@GLSL_DRAW_IMAGE_RECT] = @"";
#endif
                break;
            case DrawType::imageMesh:
                defines[@GLSL_DRAW_IMAGE] = @"";
                defines[@GLSL_DRAW_IMAGE_MESH] = @"";
                break;
            case DrawType::renderPassInitialize:
#ifdef RIVE_IOS
                RIVE_UNREACHABLE();
#else
                assert(interlockMode == InterlockMode::atomics);
                defines[@GLSL_DRAW_RENDER_TARGET_UPDATE_BOUNDS] = @"";
                defines[@GLSL_INITIALIZE_PLS] = @"";
                if (enums::is_flag_set(shaderMiscFlags,
                                       gpu::ShaderMiscFlags::storeColorClear))
                {
                    // Define this as "true" instead of an empty string because
                    // it's a specialization constant in some backends and gets
                    // branched on at runtime.
                    defines[@GLSL_STORE_COLOR_CLEAR] = @"true";
                }
                // Metal copies the render target directly to the storage buffer
                // instead of seeding it with the shader.
                assert(!enums::is_flag_set(
                    shaderMiscFlags,
                    gpu::ShaderMiscFlags::loadColorFromDstTexture));
                if (enums::is_flag_set(
                        shaderMiscFlags,
                        gpu::ShaderMiscFlags::swizzleColorBGRAToRGBA))
                {
                    defines[@GLSL_SWIZZLE_COLOR_BGRA_TO_RGBA] = @"";
                }
#endif
                break;
            case DrawType::renderPassResolve:
#ifdef RIVE_IOS
                RIVE_UNREACHABLE();
#else
                assert(interlockMode == InterlockMode::atomics);
                defines[@GLSL_DRAW_RENDER_TARGET_UPDATE_BOUNDS] = @"";
                defines[@GLSL_RESOLVE_PLS] = @"";
                if (enums::is_flag_set(
                        shaderMiscFlags,
                        gpu::ShaderMiscFlags::coalescedResolveAndTransfer))
                {
                    defines[@GLSL_COALESCED_PLS_RESOLVE_AND_TRANSFER] = @"";
                }
#endif
                break;
            case DrawType::msaaStrokes:
            case DrawType::msaaMidpointFanBorrowedCoverage:
            case DrawType::msaaDynamicMidpointFans:
            case DrawType::msaaMidpointFans:
            case DrawType::msaaMidpointFanStencilReset:
            case DrawType::msaaMidpointFanPathsStencil:
            case DrawType::msaaMidpointFanPathsCover:
            case DrawType::msaaOuterCubics:
            case DrawType::clipReset:
                RIVE_UNREACHABLE();
        }

#ifndef RIVE_IOS
        if (interlockMode == gpu::InterlockMode::atomics)
        {
            [source appendFormat:@"%s\n", gpu::glsl::draw_path_common];
            [source appendFormat:@"%s\n", gpu::glsl::atomic_draw];
        }
        else
#endif
        {
            assert(interlockMode == gpu::InterlockMode::rasterOrdering);
            switch (drawType)
            {
                case DrawType::midpointFanPatches:
                case DrawType::midpointFanCenterAAPatches:
                case DrawType::outerCurvePatches:
                case DrawType::interiorTriangulation:
                    [source appendFormat:@"%s\n", gpu::glsl::draw_path_common];
                    [source appendFormat:@"%s\n", gpu::glsl::draw_path_vert];
                    [source
                        appendFormat:@"%s\n",
                                     gpu::glsl::draw_raster_order_path_frag];
                    break;
                case DrawType::featherAtlasBlit:
                    [source appendFormat:@"%s\n", gpu::glsl::draw_path_common];
                    [source appendFormat:@"%s\n", gpu::glsl::draw_path_vert];
                    [source appendFormat:@"%s\n", gpu::glsl::draw_mesh_frag];
                    break;
                case DrawType::imageMesh:
                    [source
                        appendFormat:@"%s\n", gpu::glsl::draw_image_mesh_vert];
                    [source appendFormat:@"%s\n", gpu::glsl::draw_mesh_frag];
                    break;
                case DrawType::imageRect:
                case DrawType::msaaStrokes:
                case DrawType::msaaMidpointFanBorrowedCoverage:
                case DrawType::msaaDynamicMidpointFans:
                case DrawType::msaaMidpointFans:
                case DrawType::msaaMidpointFanStencilReset:
                case DrawType::msaaMidpointFanPathsStencil:
                case DrawType::msaaMidpointFanPathsCover:
                case DrawType::msaaOuterCubics:
                case DrawType::clipReset:
                case DrawType::renderPassInitialize:
                case DrawType::renderPassResolve:
                    RIVE_UNREACHABLE();
            }
        }

        NSError* err = nil;
        MTLCompileOptions* compileOptions = [MTLCompileOptions new];
#if defined(RIVE_IOS) || defined(RIVE_IOS_SIMULATOR)
        compileOptions.languageVersion =
            MTLLanguageVersion2_2; // On ios, we need version 2.2+
#else
        compileOptions.languageVersion =
            MTLLanguageVersion2_3; // On mac, we need version 2.3+
#endif
        compileOptions.fastMathEnabled = YES;
        if (@available(iOS 14, *))
        {
            compileOptions.preserveInvariance = YES;
        }
        compileOptions.preprocessorMacros = defines;
#ifdef WITH_RIVE_TOOLS
        if (job.synthesizedFailureType ==
            SynthesizedFailureType::shaderCompilation)
        {
            assert(job.compiledLibrary == nil);
        }
        else
#endif
        {
            job.compiledLibrary = [m_gpu newLibraryWithSource:source
                                                      options:compileOptions
                                                        error:&err];
        }

        lock.lock();

        if (err != nil || job.compiledLibrary == nil)
        {
#ifdef WITH_RIVE_TOOLS
            if (job.synthesizedFailureType ==
                SynthesizedFailureType::shaderCompilation)
            {
                NSLog(@"RIVE: Synthesizing shader compilation failure...");
            }
            else
#endif
            {
                // The compile job failed, most likely to external environmental
                // factors. Give up on this shader and let the render context
                // fall back on an uber shader instead.
                int lineNumber = 1;
                std::stringstream stream(source.UTF8String);
                std::string lineStr;
                while (std::getline(stream, lineStr, '\n'))
                {
                    NSLog(@"RIVE: %4i| %s", lineNumber++, lineStr.c_str());
                }
                NSLog(@"RIVE: Shader compilation error: %@",
                      err != nil ? err.localizedDescription : @"<nil>");
            }

            NSLog(@"RIVE: Failed to compile shader.");
            assert(false
#ifdef WITH_RIVE_TOOLS
                   || job.synthesizedFailureType ==
                          SynthesizedFailureType::shaderCompilation
#endif
            );
        }

        m_finishedJobs.push_back(std::move(job));
        m_workFinishedCondition.notify_all();
    }
}
} // namespace rive::gpu
"####;

/// Stable aliases used by source-audit queues.
pub const PINNED_SOURCE: &str = PINNED_BACKGROUND_SHADER_COMPILER_MM_SOURCE;
pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceMetadata {
    pub upstream_commit: &'static str,
    pub upstream_path: &'static str,
    pub source_sha256: &'static str,
    pub source_line_count: usize,
    pub source_byte_count: usize,
    pub target_path: &'static str,
    pub translation_unit: &'static str,
    pub translation_disposition: &'static str,
    pub translation_behavior: &'static str,
}

pub const SOURCE_METADATA: SourceMetadata = SourceMetadata {
    upstream_commit: PINNED_UPSTREAM_COMMIT,
    upstream_path: PINNED_SOURCE_PATH,
    source_sha256: PINNED_SOURCE_SHA256,
    source_line_count: PINNED_SOURCE_LINE_COUNT,
    source_byte_count: PINNED_SOURCE_BYTE_COUNT,
    target_path: TRANSLATION_TARGET,
    translation_unit: TRANSLATION_UNIT,
    translation_disposition: TRANSLATION_DISPOSITION,
    translation_behavior: TRANSLATION_BEHAVIOR,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConditionalBlock {
    pub block_id: &'static str,
    pub start_line: u16,
    pub end_line: u16,
    pub directive: &'static str,
    pub rust_condition: &'static str,
}

pub const CONDITIONAL_BLOCKS: &[ConditionalBlock] = &[
    ConditionalBlock {
        block_id: "atomic-shader-include",
        start_line: 18,
        end_line: 22,
        directive: "#ifndef RIVE_IOS",
        rust_condition: "!cfg!(all(target_os = \"ios\", not(target_abi = \"sim\")))",
    },
    ConditionalBlock {
        block_id: "image-rect-platform",
        start_line: 162,
        end_line: 168,
        directive: "#ifdef RIVE_IOS",
        rust_condition: "cfg!(all(target_os = \"ios\", not(target_abi = \"sim\")))",
    },
    ConditionalBlock {
        block_id: "initialize-platform",
        start_line: 175,
        end_line: 200,
        directive: "#ifdef RIVE_IOS",
        rust_condition: "cfg!(all(target_os = \"ios\", not(target_abi = \"sim\")))",
    },
    ConditionalBlock {
        block_id: "resolve-platform",
        start_line: 203,
        end_line: 215,
        directive: "#ifdef RIVE_IOS",
        rust_condition: "cfg!(all(target_os = \"ios\", not(target_abi = \"sim\")))",
    },
    ConditionalBlock {
        block_id: "atomic-source-append",
        start_line: 229,
        end_line: 236,
        directive: "#ifndef RIVE_IOS",
        rust_condition: "!cfg!(all(target_os = \"ios\", not(target_abi = \"sim\")))",
    },
    ConditionalBlock {
        block_id: "language-version",
        start_line: 279,
        end_line: 285,
        directive: "#if defined(RIVE_IOS) || defined(RIVE_IOS_SIMULATOR)",
        rust_condition: "cfg!(target_os = \"ios\")",
    },
    ConditionalBlock {
        block_id: "shader-compilation-injection",
        start_line: 292,
        end_line: 299,
        directive: "#ifdef WITH_RIVE_TOOLS",
        rust_condition: "cfg!(feature = \"with-rive-tools\")",
    },
    ConditionalBlock {
        block_id: "shader-failure-log-routing",
        start_line: 310,
        end_line: 317,
        directive: "#ifdef WITH_RIVE_TOOLS",
        rust_condition: "cfg!(feature = \"with-rive-tools\")",
    },
    ConditionalBlock {
        block_id: "shader-failure-assertion",
        start_line: 335,
        end_line: 338,
        directive: "#ifdef WITH_RIVE_TOOLS",
        rust_condition: "cfg!(feature = \"with-rive-tools\")",
    },
];

/// Generated fragments are explicit so the pinned append order remains visible.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GeneratedShaderSources {
    pub metal: &'static str,
    pub constants: &'static str,
    pub flush_uniforms: &'static str,
    pub common: &'static str,
    pub advanced_blend: &'static str,
    pub draw_path_common: &'static str,
    pub draw_path_vert: &'static str,
    pub draw_raster_order_path_frag: &'static str,
    pub draw_image_mesh_vert: &'static str,
    pub draw_mesh_frag: &'static str,
    pub atomic_draw: &'static str,
}

/// Runtime shader fragments emitted by the translated minifier in `build.rs`.
/// The raw `PINNED_*_SOURCE` constants remain the source/provenance authority;
/// the pinned Objective-C++ worker includes these generated fragments.
mod runtime_generated_shader_sources {
    pub const METAL: &str = include_str!(concat!(
        env!("OUT_DIR"),
        "/mechanical_shader_generated/metal.minified.glsl.runtime"
    ));
    pub const CONSTANTS: &str = include_str!(concat!(
        env!("OUT_DIR"),
        "/mechanical_shader_generated/constants.minified.glsl.runtime"
    ));
    pub const FLUSH_UNIFORMS: &str = include_str!(concat!(
        env!("OUT_DIR"),
        "/mechanical_shader_generated/flush_uniforms.minified.glsl.runtime"
    ));
    pub const COMMON: &str = include_str!(concat!(
        env!("OUT_DIR"),
        "/mechanical_shader_generated/common.minified.glsl.runtime"
    ));
    pub const ADVANCED_BLEND: &str = include_str!(concat!(
        env!("OUT_DIR"),
        "/mechanical_shader_generated/advanced_blend.minified.glsl.runtime"
    ));
    pub const DRAW_PATH_COMMON: &str = include_str!(concat!(
        env!("OUT_DIR"),
        "/mechanical_shader_generated/draw_path_common.minified.glsl.runtime"
    ));
    pub const DRAW_PATH_VERT: &str = include_str!(concat!(
        env!("OUT_DIR"),
        "/mechanical_shader_generated/draw_path.minified.vert.runtime"
    ));
    pub const DRAW_RASTER_ORDER_PATH_FRAG: &str = include_str!(concat!(
        env!("OUT_DIR"),
        "/mechanical_shader_generated/draw_raster_order_path.minified.frag.runtime"
    ));
    pub const DRAW_IMAGE_MESH_VERT: &str = include_str!(concat!(
        env!("OUT_DIR"),
        "/mechanical_shader_generated/draw_image_mesh.minified.vert.runtime"
    ));
    pub const DRAW_MESH_FRAG: &str = include_str!(concat!(
        env!("OUT_DIR"),
        "/mechanical_shader_generated/draw_mesh.minified.frag.runtime"
    ));
    pub const ATOMIC_DRAW: &str = include_str!(concat!(
        env!("OUT_DIR"),
        "/mechanical_shader_generated/atomic_draw.minified.glsl.runtime"
    ));
}

/// Exact identifiers emitted by the same translated minifier invocation as
/// the runtime fragments above. In the pinned Objective-C++ these enter this
/// unit through the generated `*.exports.h` headers.
pub(crate) mod runtime_generated_shader_exports {
    include!(concat!(
        env!("OUT_DIR"),
        "/mechanical_shader_generated/runtime_shader_exports.rs"
    ));
}

/// Exact generated-fragment assembly used by the pinned background worker.
/// Keeping this constructor beside the worker prevents the context owner from
/// silently selecting the legacy shader source universe.
pub fn generated_shader_sources() -> GeneratedShaderSources {
    GeneratedShaderSources {
        metal: runtime_generated_shader_sources::METAL,
        constants: runtime_generated_shader_sources::CONSTANTS,
        flush_uniforms: runtime_generated_shader_sources::FLUSH_UNIFORMS,
        common: runtime_generated_shader_sources::COMMON,
        advanced_blend: runtime_generated_shader_sources::ADVANCED_BLEND,
        draw_path_common: runtime_generated_shader_sources::DRAW_PATH_COMMON,
        draw_path_vert: runtime_generated_shader_sources::DRAW_PATH_VERT,
        draw_raster_order_path_frag:
            runtime_generated_shader_sources::DRAW_RASTER_ORDER_PATH_FRAG,
        draw_image_mesh_vert: runtime_generated_shader_sources::DRAW_IMAGE_MESH_VERT,
        draw_mesh_frag: runtime_generated_shader_sources::DRAW_MESH_FRAG,
        atomic_draw: runtime_generated_shader_sources::ATOMIC_DRAW,
    }
}

/// Build the source-owned worker around a retained Metal device.  The
/// Objective-C bridge is deliberately kept with this mechanical owner; the
/// host compiler/cache is not consulted by the renderer context.
#[cfg(target_vendor = "apple")]
pub fn new_for_device(
    device: Retained<crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::MTLDevice>,
    features: MetalFeatures,
) -> BackgroundShaderCompilerOwner {
    BackgroundShaderCompiler::new(
        MetalDeviceOwner::new(device),
        features,
        generated_shader_sources(),
    )
}

/// Test-only native worker entry point. It preserves the production worker,
/// queue, Objective-C owner, and adoption boundaries while making the source
/// override local to this compiler instance. Production `new_for_device`
/// always stores the translated generated fragments and cannot be masked by a
/// process-global test override.
#[cfg(all(target_vendor = "apple", test))]
pub fn new_for_device_with_sources(
    device: Retained<crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::MTLDevice>,
    features: MetalFeatures,
    sources: GeneratedShaderSources,
) -> BackgroundShaderCompilerOwner {
    BackgroundShaderCompiler::new(MetalDeviceOwner::new(device), features, sources)
}

#[cfg(target_vendor = "apple")]
unsafe fn native_new_library_with_source(
    device: *mut crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::MTLDevice,
    mut iteration: NativeCompileIteration,
) -> MetalLibraryCreation {
    use objc2::msg_send;
    use objc2::runtime::ProtocolObject;
    use objc2_foundation::NSError;
    use objc2_metal::MTLDevice as ObjcDevice;

    let device = &*(device.cast::<ProtocolObject<dyn ObjcDevice>>());
    let compile_options = iteration
        .options
        .as_ref()
        .expect("source compile options were created after source assembly");
    let source_identity = objc_retained_identity(&*iteration.source);
    let options_identity = objc_retained_identity(compile_options);
    owner_detail_event_related(
        "BG-NS-SOURCE",
        "LastUse(compile)",
        source_identity,
        options_identity,
    );
    owner_detail_event_related(
        "BG-COMPILE-OPTIONS",
        "LastUse(newLibrary)",
        options_identity,
        source_identity,
    );
    let mut error: Option<objc2::rc::Retained<NSError>> = None;
    let library: Option<objc2::rc::Retained<ProtocolObject<dyn objc2_metal::MTLLibrary>>> = msg_send![device, newLibraryWithSource: &**iteration.source, options: Some(&**compile_options), error: &mut error];
    if let Some(library) = library.as_ref() {
        owner_event(
            "BG-LIB-COMPILED",
            BackgroundOwnerPhase::Create,
            objc_retained_identity(library),
        );
        owner_detail_event(
            "BG-LIB-COMPILED",
            "Create",
            objc_retained_identity(library),
        );
    }
    if let Some(error) = error.as_ref() {
        owner_event(
            "BG-ERR-COMPILE",
            BackgroundOwnerPhase::Create,
            objc_retained_identity(error),
        );
        owner_detail_event("BG-ERR-COMPILE", "Create", objc_retained_identity(error));
    }
    let library = library.map(|library| {
        let pointer = objc2::rc::Retained::into_raw(library).cast::<crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::MTLLibrary>();
        owner_event(
            "BG-LIB-COMPILED",
            BackgroundOwnerPhase::Transfer,
            pointer as usize,
        );
        owner_detail_event("BG-LIB-COMPILED", "TransferJob", pointer as usize);
        pointer
    });
    iteration.error = std::mem::ManuallyDrop::new(error);
    if let Some(error) = unsafe { (&*iteration.error).as_ref() } {
        owner_detail_event(
            "BG-ERR-COMPILE",
            "TransferToIteration",
            objc_retained_identity(error),
        );
    }
    MetalLibraryCreation {
        library: library.unwrap_or(core::ptr::null_mut()),
        error: iteration
            .error
            .as_ref()
            .map(|_| MetalCompileError { struct_marker: () }),
        native_owners: iteration,
    }
}

#[cfg(target_vendor = "apple")]
unsafe fn native_preserve_invariance(
    _device: *mut crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::MTLDevice,
) -> bool {
    // The source property was introduced on iOS 14.  Keep the selector out
    // of older SDK/runtime paths while preserving the desktop behavior.
    #[cfg(target_os = "ios")]
    {
        objc2::available!(ios = 14.0)
    }
    #[cfg(not(target_os = "ios"))]
    {
        true
    }
}

pub use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
    DrawType, InterlockMode, ShaderFeatures, ShaderMiscFlags,
};

pub use crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::{
    AtomicBarrierType, MetalFeatures,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::{
    MTLDevice, MTLLibrary, Retained,
};
#[cfg(target_vendor = "apple")]
fn objc_retained_identity<T: objc2::Message + ?Sized>(value: &objc2::rc::Retained<T>) -> usize {
    objc2::rc::Retained::<T>::as_ptr(value).cast::<()>() as usize
}

#[cfg(target_vendor = "apple")]
fn metal_retained_identity<T>(value: &Retained<T>) -> usize {
    Retained::<T>::as_ptr(value) as usize
}
#[cfg(feature = "with-rive-tools")]
pub use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::SynthesizedFailureType;

const K_SHADER_FEATURE_COUNT: usize = 8;
const GLSL_VERTEX: &str = runtime_generated_shader_exports::GLSL_VERTEX;
const GLSL_FRAGMENT: &str = runtime_generated_shader_exports::GLSL_FRAGMENT;
const GLSL_PLS_IMPL_DEVICE_BUFFER: &str =
    runtime_generated_shader_exports::GLSL_PLS_IMPL_DEVICE_BUFFER;
const GLSL_PLS_IMPL_DEVICE_BUFFER_RASTER_ORDERED: &str =
    runtime_generated_shader_exports::GLSL_PLS_IMPL_DEVICE_BUFFER_RASTER_ORDERED;
const GLSL_FIXED_FUNCTION_COLOR_OUTPUT: &str =
    runtime_generated_shader_exports::GLSL_FIXED_FUNCTION_COLOR_OUTPUT;
const GLSL_CLOCKWISE_FILL: &str = runtime_generated_shader_exports::GLSL_CLOCKWISE_FILL;
const GLSL_ENABLE_INSTANCE_INDEX: &str =
    runtime_generated_shader_exports::GLSL_ENABLE_INSTANCE_INDEX;
const GLSL_DRAW_PATH: &str = runtime_generated_shader_exports::GLSL_DRAW_PATH;
const GLSL_DRAW_INTERIOR_TRIANGLES: &str =
    runtime_generated_shader_exports::GLSL_DRAW_INTERIOR_TRIANGLES;
const GLSL_FEATHER_ATLAS_BLIT: &str =
    runtime_generated_shader_exports::GLSL_FEATHER_ATLAS_BLIT;
const GLSL_DRAW_IMAGE: &str = runtime_generated_shader_exports::GLSL_DRAW_IMAGE;
const GLSL_DRAW_IMAGE_RECT: &str = runtime_generated_shader_exports::GLSL_DRAW_IMAGE_RECT;
const GLSL_DRAW_IMAGE_MESH: &str = runtime_generated_shader_exports::GLSL_DRAW_IMAGE_MESH;
const GLSL_DRAW_RENDER_TARGET_UPDATE_BOUNDS: &str =
    runtime_generated_shader_exports::GLSL_DRAW_RENDER_TARGET_UPDATE_BOUNDS;
const GLSL_INITIALIZE_PLS: &str = runtime_generated_shader_exports::GLSL_INITIALIZE_PLS;
const GLSL_STORE_COLOR_CLEAR: &str = runtime_generated_shader_exports::GLSL_STORE_COLOR_CLEAR;
const GLSL_SWIZZLE_COLOR_BGRA_TO_RGBA: &str =
    runtime_generated_shader_exports::GLSL_SWIZZLE_COLOR_BGRA_TO_RGBA;
const GLSL_RESOLVE_PLS: &str = runtime_generated_shader_exports::GLSL_RESOLVE_PLS;
const GLSL_COALESCED_PLS_RESOLVE_AND_TRANSFER: &str =
    runtime_generated_shader_exports::GLSL_COALESCED_PLS_RESOLVE_AND_TRANSFER;

fn get_shader_feature_glsl_name(feature: ShaderFeatures) -> &'static str {
    const NAMES: [&str; K_SHADER_FEATURE_COUNT] = [
        runtime_generated_shader_exports::GLSL_ENABLE_CLIPPING,
        runtime_generated_shader_exports::GLSL_ENABLE_CLIP_RECT,
        runtime_generated_shader_exports::GLSL_ENABLE_ADVANCED_BLEND,
        runtime_generated_shader_exports::GLSL_ENABLE_FEATHER,
        runtime_generated_shader_exports::GLSL_ENABLE_EVEN_ODD,
        runtime_generated_shader_exports::GLSL_ENABLE_NESTED_CLIPPING,
        runtime_generated_shader_exports::GLSL_ENABLE_HSL_BLEND_MODES,
        runtime_generated_shader_exports::GLSL_ENABLE_DITHER,
    ];
    NAMES[feature.0.trailing_zeros() as usize]
}

pub struct BackgroundCompileJob {
    pub drawType: DrawType,
    pub shaderFeatures: ShaderFeatures,
    pub interlockMode: InterlockMode,
    pub shaderMiscFlags: ShaderMiscFlags,
    pub compiledLibrary: Option<Retained<MTLLibrary>>,
    #[cfg(feature = "with-rive-tools")]
    pub synthesizedFailureType: SynthesizedFailureType,
}

#[cfg(target_vendor = "apple")]
impl Drop for BackgroundCompileJob {
    fn drop(&mut self) {
        if let Some(library) = self.compiledLibrary.take() {
            owner_event(
                "BG-LIB-COMPILED",
                BackgroundOwnerPhase::Release,
                metal_retained_identity(&library),
            );
            owner_detail_event(
                "BG-LIB-COMPILED",
                "ReleaseContext",
                metal_retained_identity(&library),
            );
            drop(library);
        }
    }
}

impl BackgroundCompileJob {
    pub fn new(
        drawType: DrawType,
        shaderFeatures: ShaderFeatures,
        interlockMode: InterlockMode,
        shaderMiscFlags: ShaderMiscFlags,
    ) -> Self {
        Self {
            drawType,
            shaderFeatures,
            interlockMode,
            shaderMiscFlags,
            compiledLibrary: None,
            #[cfg(feature = "with-rive-tools")]
            synthesizedFailureType: SynthesizedFailureType::none,
        }
    }

    /// Transfer the worker's compiled library +1 to the execution adapter.
    /// The adapter must publish that exact native object before it is used by
    /// selector-based pipeline construction; no registry alias is fabricated
    /// here.
    #[cfg(target_vendor = "apple")]
    pub fn take_compiled_library_raw(&mut self) -> Option<*mut MTLLibrary> {
        self.compiledLibrary.take().map(|library| {
            let identity = metal_retained_identity(&library);
            owner_detail_event("BG-LIB-COMPILED", "TransferCaller", identity);
            Retained::into_raw(library)
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetalCompileOptions {
    pub languageVersion: MetalLanguageVersion,
    pub fastMathEnabled: bool,
    pub preserveInvariance: bool,
    pub preprocessorMacros: MacroDictionary,
}

#[repr(u64)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetalLanguageVersion {
    Version2_2,
    Version2_3,
}

pub struct MetalCompileError {
    /// Recording/non-Apple builds have no native NSError owner; preserve the
    /// textual diagnostic only for that adapter path.
    #[cfg(not(target_vendor = "apple"))]
    pub localizedDescription: Option<String>,
    #[cfg(target_vendor = "apple")]
    /// Native NSError ownership is held by the surrounding
    /// `NativeCompileIteration`; this marker keeps the public creation result
    /// non-owning so there is exactly one native retain.
    pub struct_marker: (),
}

impl MetalCompileError {
    #[cfg(target_vendor = "apple")]
    fn localized_description(&self, iteration: &NativeCompileIteration) -> Option<String> {
        use objc2::runtime::{AnyObject, Sel};
        use objc2::{Message, sel};
        use objc2_foundation::NSString;

        // `err.localizedDescription` is a borrowed Objective-C expression in
        // the pinned worker.  Calling the safe generated method would retain
        // the autoreleased/property result and invent a strong local that the
        // source does not have, so preserve the raw +0 boundary here.
        type LocalizedDescription = unsafe extern "C" fn(*const AnyObject, Sel) -> *const NSString;
        let localized_description: LocalizedDescription =
            unsafe { core::mem::transmute(objc2::ffi::objc_msgSend as *const ()) };
        let error = unsafe { (&*iteration.error).as_ref()? };
        owner_detail_event(
            "BG-ERR-COMPILE",
            "LastUse(log)",
            objc_retained_identity(error),
        );
        let description = unsafe {
            localized_description(
                error.as_ref() as *const _ as *const AnyObject,
                sel!(localizedDescription),
            )
            .as_ref()?
        };
        owner_event(
            "BG-NS-ERR-DESC",
            BackgroundOwnerPhase::Borrow,
            description as *const NSString as usize,
        );
        let identity = description as *const NSString as usize;
        owner_detail_event("BG-NS-ERR-DESC", "Borrow", identity);
        let text = description.to_string();
        owner_detail_event("BG-NS-ERR-DESC", "LastUse(log)", identity);
        owner_detail_event("BG-NS-ERR-DESC", "ExpressionEnd", identity);
        Some(text)
    }

    #[cfg(not(target_vendor = "apple"))]
    fn localized_description(&self) -> Option<String> {
        self.localizedDescription.clone()
    }
}

pub struct MetalLibraryCreation {
    pub library: *mut MTLLibrary,
    pub error: Option<MetalCompileError>,
    #[cfg(target_vendor = "apple")]
    pub native_owners: NativeCompileIterationOwners,
}

#[cfg(target_vendor = "apple")]
pub struct NativeCompileIteration {
    /// The source creates this dictionary before assembling any source text.
    pub macros: std::mem::ManuallyDrop<
        objc2::rc::Retained<
            objc2_foundation::NSMutableDictionary<
                objc2_foundation::NSString,
                objc2_foundation::NSObject,
            >,
        >,
    >,
    /// Keep the mutable source object for the whole worker iteration.  The
    /// bridge must not rebuild an immutable NSString from a Rust DTO after
    /// source assembly.
    pub source: std::mem::ManuallyDrop<objc2::rc::Retained<objc2_foundation::NSMutableString>>,
    pub options:
        std::mem::ManuallyDrop<Option<objc2::rc::Retained<objc2_metal::MTLCompileOptions>>>,
    /// NSError is declared after compile options in the source iteration and
    /// is released before the mutable source and macro dictionary.
    pub error: std::mem::ManuallyDrop<Option<objc2::rc::Retained<objc2_foundation::NSError>>>,
}

#[cfg(target_vendor = "apple")]
impl NativeCompileIteration {
    pub fn new(seed: &str) -> Self {
        use objc2::AnyThread;
        use std::ffi::{CString, c_char};
        use std::ptr::NonNull;
        let seed = CString::new(seed).expect("generated shader seed contains no NUL");
        let source = unsafe {
            objc2_foundation::NSMutableString::initWithCString_encoding(
                objc2_foundation::NSMutableString::alloc(),
                NonNull::new(seed.as_ptr().cast_mut().cast::<c_char>())
                    .expect("CString pointer is non-null"),
                objc2_foundation::NSUTF8StringEncoding,
            )
            .expect("generated shader seed is valid UTF-8")
        };
        let macros = objc2_foundation::NSMutableDictionary::new();
        owner_event(
            "BG-DICT-DEFINES",
            BackgroundOwnerPhase::Create,
            objc_retained_identity(&macros),
        );
        owner_detail_event("BG-DICT-DEFINES", "Create", objc_retained_identity(&macros));
        owner_event(
            "BG-NS-SOURCE",
            BackgroundOwnerPhase::Create,
            objc_retained_identity(&source),
        );
        owner_detail_event("BG-NS-SOURCE", "Create", objc_retained_identity(&source));
        Self {
            macros: std::mem::ManuallyDrop::new(macros),
            // The pinned worker initializes the mutable source with the
            // complete metal seed before any appendFormat fragment calls.
            // Keeping this as the one native mutable source also makes the
            // compile/log pointer identity explicit.
            source: std::mem::ManuallyDrop::new(source),
            options: std::mem::ManuallyDrop::new(None),
            error: std::mem::ManuallyDrop::new(None),
        }
    }

    fn define_static(
        &self,
        key: &'static objc2_foundation::NSString,
        value: &'static objc2_foundation::NSString,
    ) {
        use objc2::msg_send;
        owner_event(
            "BG-NS-MACRO-LITERALS",
            BackgroundOwnerPhase::Borrow,
            key as *const _ as usize,
        );
        let key_identity = key as *const _ as usize;
        let value_identity = value as *const _ as usize;
        owner_detail_event_related(
            "BG-NS-MACRO-LITERALS",
            "Borrow",
            key_identity,
            value_identity,
        );
        // SAFETY: both NSString arguments are retained through this message;
        // NSMutableDictionary copies/retains the key and value as upstream.
        unsafe {
            let _: () = msg_send![
                &**self.macros,
                setObject: &*value,
                forKeyedSubscript: &*key
            ];
        }
        owner_detail_event_related(
            "BG-NS-MACRO-LITERALS",
            "LastUse(dictionary set)",
            key_identity,
            value_identity,
        );
    }

    fn define_dynamic(&self, key: &str, value: &'static objc2_foundation::NSString) {
        use objc2::runtime::{AnyObject, Sel};
        use objc2::{ClassType, msg_send, sel};
        use objc2_foundation::NSString;
        use std::ffi::{CString, c_char};
        use std::ptr::NonNull;
        let Ok(key_bytes) = CString::new(key) else {
            return;
        };
        let Some(key) = NonNull::new(key_bytes.as_ptr().cast_mut().cast::<c_char>()) else {
            return;
        };
        // The pinned dynamic macro path uses the autoreleased +0 result from
        // `stringWithUTF8String:` directly in the dictionary expression.
        // objc2's safe wrapper retains that result, which would add a local
        // owner absent from the source, so preserve the raw message boundary.
        type StringWithUtf8String =
            unsafe extern "C" fn(*const AnyObject, Sel, NonNull<c_char>) -> *const NSString;
        let string_with_utf8_string: StringWithUtf8String =
            unsafe { core::mem::transmute(objc2::ffi::objc_msgSend as *const ()) };
        let Some(key) = (unsafe {
            string_with_utf8_string(
                NSString::class() as *const _ as *const AnyObject,
                sel!(stringWithUTF8String:),
                key,
            )
            .as_ref()
        }) else {
            return;
        };
        owner_event(
            "BG-NS-MACRO-KEY-DYNAMIC",
            BackgroundOwnerPhase::Borrow,
            key as *const NSString as usize,
        );
        let key_identity = key as *const NSString as usize;
        let value_identity = value as *const NSString as usize;
        owner_detail_event_related(
            "BG-NS-MACRO-KEY-DYNAMIC",
            "Borrow",
            key_identity,
            value_identity,
        );
        unsafe {
            let _: () = msg_send![
                &**self.macros,
                setObject: value,
                forKeyedSubscript: key
            ];
        }
        owner_detail_event_related(
            "BG-NS-MACRO-KEY-DYNAMIC",
            "LastUse(dictionary set)",
            key_identity,
            value_identity,
        );
        owner_detail_event_related(
            "BG-NS-MACRO-KEY-DYNAMIC",
            "AutoreleaseEnd",
            key_identity,
            value_identity,
        );
    }

    fn append_raw_source(&self, fragment: &str) {
        use objc2::runtime::{AnyObject, Sel};
        use objc2_foundation::NSString;
        use std::ffi::{CString, c_char};
        let fragment = CString::new(fragment).expect("generated shader fragment contains no NUL");
        // The pinned source uses appendFormat:@"%s\n" directly. Calling the
        // known Objective-C variadic ABI avoids materializing a temporary
        // NSString for every shader fragment and preserves the exact source
        // bytes and lexical owner boundary.
        type AppendFormat = unsafe extern "C" fn(*const AnyObject, Sel, *const NSString, ...);
        let append_format: AppendFormat =
            unsafe { core::mem::transmute(objc2::ffi::objc_msgSend as *const ()) };
        let receiver = core::ptr::from_ref(&**self.source).cast::<AnyObject>();
        let format = objc2_foundation::ns_string!("%s\n");
        let format_identity = core::ptr::from_ref(format) as usize;
        owner_event(
            "BG-NS-APPEND-TEMP",
            BackgroundOwnerPhase::Borrow,
            format_identity,
        );
        owner_detail_event_related(
            "BG-NS-APPEND-TEMP",
            "BorrowFormat",
            format_identity,
            receiver as usize,
        );
        unsafe {
            append_format(
                receiver,
                Sel::register(c"appendFormat:"),
                core::ptr::from_ref(format),
                fragment.as_ptr(),
            );
        }
        owner_detail_event_related(
            "BG-NS-APPEND-TEMP",
            "LastUse(appendFormat)",
            format_identity,
            receiver as usize,
        );
    }

    fn append_raw_source_prefix(&self, constants: &str, flush_uniforms: &str, common: &str) {
        use objc2::runtime::{AnyObject, Sel};
        use objc2_foundation::NSString;
        use std::ffi::{CString, c_char};
        let constants = CString::new(constants).expect("generated shader fragment contains no NUL");
        let flush_uniforms =
            CString::new(flush_uniforms).expect("generated shader fragment contains no NUL");
        let common = CString::new(common).expect("generated shader fragment contains no NUL");
        type AppendFormat = unsafe extern "C" fn(*const AnyObject, Sel, *const NSString, ...);
        let append_format: AppendFormat =
            unsafe { core::mem::transmute(objc2::ffi::objc_msgSend as *const ()) };
        let receiver = core::ptr::from_ref(&**self.source).cast::<AnyObject>();
        let format = objc2_foundation::ns_string!("%s\n%s\n%s\n");
        let format_identity = core::ptr::from_ref(format) as usize;
        owner_event(
            "BG-NS-APPEND-TEMP",
            BackgroundOwnerPhase::Borrow,
            format_identity,
        );
        owner_detail_event_related(
            "BG-NS-APPEND-TEMP",
            "BorrowFormat",
            format_identity,
            receiver as usize,
        );
        unsafe {
            append_format(
                receiver,
                Sel::register(c"appendFormat:"),
                core::ptr::from_ref(format),
                constants.as_ptr(),
                flush_uniforms.as_ptr(),
                common.as_ptr(),
            );
        }
        owner_detail_event_related(
            "BG-NS-APPEND-TEMP",
            "LastUse(appendFormat)",
            format_identity,
            receiver as usize,
        );
    }

    fn append_source_prefix(&self, constants: &str, flush_uniforms: &str, common: &str) {
        self.append_raw_source_prefix(constants, flush_uniforms, common);
    }

    pub fn append_source(&self, fragment: &str) {
        self.append_raw_source(fragment);
    }

    fn source_for_log(&self) -> String {
        let identity = objc_retained_identity(&*self.source);
        owner_detail_event("BG-NS-SOURCE", "LastUse(log)", identity);
        (&**self.source).to_string()
    }

    fn finish_options_values(
        &mut self,
        language_version: MetalLanguageVersion,
        fast_math_enabled: bool,
        preserve_invariance: bool,
    ) {
        use objc2_metal::{MTLCompileOptions, MTLLanguageVersion};
        let compile_options = MTLCompileOptions::new();
        compile_options.setLanguageVersion(match language_version {
            MetalLanguageVersion::Version2_2 => MTLLanguageVersion::Version2_2,
            MetalLanguageVersion::Version2_3 => MTLLanguageVersion::Version2_3,
        });
        #[allow(deprecated)]
        compile_options.setFastMathEnabled(fast_math_enabled);
        if preserve_invariance {
            compile_options.setPreserveInvariance(true);
        }
        // SAFETY: the dictionary is the source-owned local and remains alive
        // through the synchronous library call.
        unsafe {
            compile_options.setPreprocessorMacros(Some(&**self.macros));
        }
        owner_detail_event_related(
            "BG-DICT-DEFINES",
            "LastUse(options setter)",
            objc_retained_identity(&*self.macros),
            objc_retained_identity(&compile_options),
        );
        owner_event(
            "BG-COMPILE-OPTIONS",
            BackgroundOwnerPhase::Create,
            objc_retained_identity(&compile_options),
        );
        owner_detail_event(
            "BG-COMPILE-OPTIONS",
            "Create",
            objc_retained_identity(&compile_options),
        );
        *self.options = Some(compile_options);
    }

    #[cfg(test)]
    pub fn finish_options(&mut self, options: &MetalCompileOptions) {
        self.finish_options_values(
            options.languageVersion,
            options.fastMathEnabled,
            options.preserveInvariance,
        );
    }

    fn drop_options(&mut self) {
        unsafe {
            // Clear the Option before releasing the owner.  Leaving the
            // ManuallyDrop slot populated would make the fallback Drop path
            // release the same MTLCompileOptions a second time.
            if let Some(options) = (&mut *self.options).take() {
                owner_event(
                    "BG-COMPILE-OPTIONS",
                    BackgroundOwnerPhase::Release,
                    objc_retained_identity(&options),
                );
                owner_detail_event(
                    "BG-COMPILE-OPTIONS",
                    "Release",
                    objc_retained_identity(&options),
                );
                drop(options);
            }
        }
    }
}

#[cfg(target_vendor = "apple")]
impl Drop for NativeCompileIteration {
    fn drop(&mut self) {
        // The normal caller removes options first, logs NSError, then this
        // Drop releases source before the macro dictionary, matching ARC
        // locals.  Keep the fallback here as well: a panic after
        // finish_options must not leak the compile-options owner.
        unsafe {
            if let Some(options) = (&mut *self.options).take() {
                owner_event(
                    "BG-COMPILE-OPTIONS",
                    BackgroundOwnerPhase::Release,
                    objc_retained_identity(&options),
                );
                owner_detail_event(
                    "BG-COMPILE-OPTIONS",
                    "Release",
                    objc_retained_identity(&options),
                );
                drop(options);
            }
            if let Some(error) = std::mem::ManuallyDrop::take(&mut self.error) {
                owner_event(
                    "BG-ERR-COMPILE",
                    BackgroundOwnerPhase::Release,
                    objc_retained_identity(&error),
                );
                owner_detail_event("BG-ERR-COMPILE", "Release", objc_retained_identity(&error));
                drop(error);
            }
            let source = std::mem::ManuallyDrop::take(&mut self.source);
            owner_event(
                "BG-NS-SOURCE",
                BackgroundOwnerPhase::Release,
                objc_retained_identity(&source),
            );
            owner_detail_event("BG-NS-SOURCE", "Release", objc_retained_identity(&source));
            drop(source);
            let macros = std::mem::ManuallyDrop::take(&mut self.macros);
            owner_event(
                "BG-DICT-DEFINES",
                BackgroundOwnerPhase::Release,
                objc_retained_identity(&macros),
            );
            owner_detail_event(
                "BG-DICT-DEFINES",
                "Release",
                objc_retained_identity(&macros),
            );
            drop(macros);
        }
    }
}

#[cfg(target_vendor = "apple")]
pub type NativeCompileIterationOwners = NativeCompileIteration;

#[cfg(target_vendor = "apple")]
macro_rules! append_source_fragment {
    ($source:expr, $native:expr, $fragment:expr $(,)?) => {{
        $native.append_source($fragment);
    }};
}

#[cfg(not(target_vendor = "apple"))]
macro_rules! append_source_fragment {
    ($source:expr, $native:expr, $fragment:expr $(,)?) => {{
        $source.push_str($fragment);
        $source.push('\n');
    }};
}

#[cfg(target_vendor = "apple")]
macro_rules! append_source_prefix {
    ($source:expr, $native:expr, $constants:expr, $flush_uniforms:expr, $common:expr $(,)?) => {{
        $native.append_source_prefix($constants, $flush_uniforms, $common);
    }};
}

#[cfg(not(target_vendor = "apple"))]
macro_rules! append_source_prefix {
    ($source:expr, $native:expr, $constants:expr, $flush_uniforms:expr, $common:expr $(,)?) => {{
        $source.push_str($constants);
        $source.push('\n');
        $source.push_str($flush_uniforms);
        $source.push('\n');
        $source.push_str($common);
        $source.push('\n');
    }};
}

#[cfg(target_vendor = "apple")]
macro_rules! insert_dynamic_define {
    ($defines:expr, $native:expr, $key:expr) => {{
        $native.define_dynamic($key, source_macro_literal("1"));
    }};
}

#[cfg(not(target_vendor = "apple"))]
macro_rules! insert_dynamic_define {
    ($defines:expr, $native:expr, $key:expr) => {{
        $defines.insert($key, "1");
    }};
}

#[cfg(target_vendor = "apple")]
const SOURCE_MACRO_LITERAL_TEXTS: [&str; 22] =
    runtime_generated_shader_exports::SOURCE_MACRO_LITERAL_TEXTS;

#[cfg(target_vendor = "apple")]
fn source_macro_literal(text: &str) -> &'static objc2_foundation::NSString {
    runtime_generated_shader_exports::source_macro_literal(text)
}

#[cfg(target_vendor = "apple")]
macro_rules! insert_static_define {
    ($defines:expr, $native:expr, $key:expr, $value:expr) => {{
        $native.define_static(source_macro_literal($key), source_macro_literal($value));
    }};
}

#[cfg(not(target_vendor = "apple"))]
macro_rules! insert_static_define {
    ($defines:expr, $native:expr, $key:expr, $value:expr) => {{
        $defines.insert($key, $value);
    }};
}

struct RetainedMetalLibraryCreation {
    library: Option<Retained<MTLLibrary>>,
    error: Option<MetalCompileError>,
    #[cfg(target_vendor = "apple")]
    native_owners: NativeCompileIterationOwners,
}

#[repr(transparent)]
pub struct MetalDeviceOwner {
    retained: Retained<MTLDevice>,
}

impl MetalDeviceOwner {
    pub fn new(retained: Retained<MTLDevice>) -> Self {
        owner_event(
            "BG-GPU-MEMBER",
            BackgroundOwnerPhase::Create,
            metal_retained_identity(&retained),
        );
        owner_detail_event(
            "BG-GPU-MEMBER",
            "Create",
            metal_retained_identity(&retained),
        );
        Self { retained }
    }

    #[cfg(target_vendor = "apple")]
    fn newLibraryWithSource(
        &self,
        iteration: NativeCompileIteration,
    ) -> RetainedMetalLibraryCreation {
        let creation = unsafe { native_new_library_with_source(self.retained.as_ptr(), iteration) };
        RetainedMetalLibraryCreation {
            library: unsafe { Retained::from_raw_retained(creation.library) },
            error: creation.error,
            native_owners: creation.native_owners,
        }
    }

    #[cfg(not(target_vendor = "apple"))]
    fn newLibraryWithSource(
        &self,
        source: &str,
        options: &MetalCompileOptions,
    ) -> RetainedMetalLibraryCreation {
        let creation = unsafe { native_new_library_with_source(self.retained.as_ptr(), source, options) };
        RetainedMetalLibraryCreation {
            library: unsafe { Retained::from_raw_retained(creation.library) },
            error: creation.error,
        }
    }

    fn preserveInvarianceAvailable(&self) -> bool {
        unsafe { native_preserve_invariance(self.retained.as_ptr()) }
    }
}

#[cfg(target_vendor = "apple")]
impl Drop for MetalDeviceOwner {
    fn drop(&mut self) {
        owner_event(
            "BG-GPU-MEMBER",
            BackgroundOwnerPhase::Release,
            metal_retained_identity(&self.retained),
        );
        owner_detail_event(
            "BG-GPU-MEMBER",
            "Release",
            metal_retained_identity(&self.retained),
        );
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MacroDictionary {
    entries: Vec<(String, String)>,
}

impl MacroDictionary {
    fn insert(&mut self, key: &str, value: &str) {
        self.entries.push((key.to_owned(), value.to_owned()));
    }
}

/// The source declaration order is preserved exactly: device/features,
// pending FIFO/finished LIFO, mutex/conditions, quit flag, then worker.
// The worker stores only a non-owning pinned pointer; the boxed owner joins it
// before any member is torn down.
struct CompilerSelfPtr(*const BackgroundShaderCompiler);
unsafe impl Send for CompilerSelfPtr {}

/// Compiles draw shaders serially on a lazily started, joinable worker.
#[repr(C)]
pub struct BackgroundShaderCompiler {
    pub(crate) m_gpu: ManuallyDrop<MetalDeviceOwner>,
    pub(crate) m_metalFeatures: ManuallyDrop<MetalFeatures>,
    m_pendingJobs: ManuallyDrop<std::cell::UnsafeCell<VecDeque<BackgroundCompileJob>>>,
    m_finishedJobs: ManuallyDrop<std::cell::UnsafeCell<Vec<BackgroundCompileJob>>>,
    m_mutex: ManuallyDrop<Mutex<()>>,
    m_workAddedCondition: ManuallyDrop<Condvar>,
    m_workFinishedCondition: ManuallyDrop<Condvar>,
    m_shouldQuit: std::cell::UnsafeCell<bool>,
    m_compilerThread: ManuallyDrop<std::cell::UnsafeCell<Option<JoinHandle<()>>>>,
    #[cfg(test)]
    m_testShaderSources: GeneratedShaderSources,
    /// Zero-sized pin seal: the pointee is never moved after the worker is
    /// started, matching the C++ unique_ptr owner and raw `this` capture.
    _pin: PhantomPinned,
}

/// Pinned unique-owner translation of the C++ `unique_ptr` field. Public
/// operations borrow the stable pointee; shutdown joins through shared access
/// before the owner obtains exclusive access for reverse member destruction.
pub struct BackgroundShaderCompilerOwner {
    inner: Pin<Box<BackgroundShaderCompiler>>,
}

impl BackgroundShaderCompilerOwner {
    pub fn pushJob(&self, job: BackgroundCompileJob) {
        self.inner.pushJob(job);
    }

    pub fn popFinishedJob(&self, job: &mut BackgroundCompileJob, wait: bool) -> bool {
        self.inner.popFinishedJob(job, wait)
    }
}

impl BackgroundShaderCompiler {
    fn new(
        gpu: MetalDeviceOwner,
        metalFeatures: MetalFeatures,
        sources: GeneratedShaderSources,
    ) -> BackgroundShaderCompilerOwner {
        #[cfg(not(test))]
        let _ = sources;
        BackgroundShaderCompilerOwner {
            inner: Box::pin(Self {
                m_gpu: ManuallyDrop::new(gpu),
                m_metalFeatures: ManuallyDrop::new(metalFeatures),
                m_pendingJobs: ManuallyDrop::new(std::cell::UnsafeCell::new(VecDeque::new())),
                m_finishedJobs: ManuallyDrop::new(std::cell::UnsafeCell::new(Vec::new())),
                m_mutex: ManuallyDrop::new(Mutex::new(())),
                m_workAddedCondition: ManuallyDrop::new(Condvar::new()),
                m_workFinishedCondition: ManuallyDrop::new(Condvar::new()),
                m_shouldQuit: std::cell::UnsafeCell::new(false),
                m_compilerThread: ManuallyDrop::new(std::cell::UnsafeCell::new(None)),
                #[cfg(test)]
                m_testShaderSources: sources,
                _pin: PhantomPinned,
            }),
        }
    }

    pub fn pushJob(&self, job: BackgroundCompileJob) {
        let lock = lock_recovering_poison(&self.m_mutex);
        let compilerThread = unsafe { &mut *self.m_compilerThread.get() };
        if compilerThread.is_none() {
            let thread_self = CompilerSelfPtr(self as *const Self);
            *compilerThread = Some(thread::spawn(move || Self::threadMain(thread_self)));
        }
        unsafe {
            (&mut *self.m_pendingJobs.get()).push_back(job);
        }
        drop(lock);
        self.m_workAddedCondition.notify_all();
    }

    pub fn popFinishedJob(&self, job: &mut BackgroundCompileJob, wait: bool) -> bool {
        let mut lock = lock_recovering_poison(&self.m_mutex);
        while unsafe { (&*self.m_finishedJobs.get()).is_empty() } {
            if !wait {
                return false;
            }
            lock = wait_recovering_poison(&self.m_workFinishedCondition, lock);
        }
        *job = unsafe {
            (&mut *self.m_finishedJobs.get())
                .pop()
                .expect("finished queue was non-empty")
        };
        #[cfg(target_vendor = "apple")]
        if let Some(library) = job.compiledLibrary.as_ref() {
            owner_detail_event(
                "BG-LIB-COMPILED",
                "TransferFinished",
                metal_retained_identity(library),
            );
        }
        true
    }

    fn threadMain(thread_self: CompilerSelfPtr) {
        // The owner is always boxed by RenderContextMetal.  Drop joins this
        // worker before releasing any source member, so this pinned, non-owning
        // pointer remains valid for the complete worker lifetime.
        let this = thread_self.0;
        // Keep the worker entirely raw: `Drop` owns the unique shutdown path,
        // joins this thread, and only then permits the boxed fields to drop.
        let gpu = unsafe { core::ptr::addr_of!((*this).m_gpu) };
        let metalFeatures = unsafe { core::ptr::addr_of!((*this).m_metalFeatures) };
        let pendingJobs = unsafe { core::ptr::addr_of!((*this).m_pendingJobs) };
        let finishedJobs = unsafe { core::ptr::addr_of!((*this).m_finishedJobs) };
        let mutex = unsafe { core::ptr::addr_of!((*this).m_mutex) };
        let workAddedCondition = unsafe { core::ptr::addr_of!((*this).m_workAddedCondition) };
        let workFinishedCondition = unsafe { core::ptr::addr_of!((*this).m_workFinishedCondition) };
        let shouldQuit = unsafe { (*core::ptr::addr_of!((*this).m_shouldQuit)).get() };
        #[cfg(target_vendor = "apple")]
        owner_detail_event("BG-GPU-MEMBER", "Borrow(worker)", unsafe {
            let gpu_owner = gpu.cast::<MetalDeviceOwner>();
            let retained = core::ptr::addr_of!((*gpu_owner).retained);
            metal_retained_identity(&*retained)
        });
        #[cfg(test)]
        let sources = unsafe { *core::ptr::addr_of!((*this).m_testShaderSources) };
        #[cfg(not(test))]
        let sources = generated_shader_sources();
        let mut job: Option<BackgroundCompileJob> = None;
        let mut lock = lock_recovering_poison(unsafe { &*mutex });
        loop {
            while unsafe { (&*(*pendingJobs).get()).is_empty() }
                && !unsafe { core::ptr::read_volatile(shouldQuit) }
            {
                lock = wait_recovering_poison(unsafe { &*workAddedCondition }, lock);
            }

            if unsafe { core::ptr::read_volatile(shouldQuit) } {
                return;
            }

            job = Some(unsafe {
                (&mut *(*pendingJobs).get())
                    .pop_front()
                    .expect("pending queue was non-empty")
            });

            drop(lock);

            let mut job_value = job.take().expect("worker job was assigned");
            let drawType = job_value.drawType;
            let shaderFeatures = job_value.shaderFeatures;
            let interlockMode = job_value.interlockMode;
            let shaderMiscFlags = job_value.shaderMiscFlags;

            // Pinned order: the mutable macro dictionary and source object
            // are iteration locals, created before source assembly; compile
            // options are created only after the complete source exists.
            #[cfg(target_vendor = "apple")]
            let mut native_iteration = NativeCompileIteration::new(sources.metal);
            #[cfg(not(target_vendor = "apple"))]
            let mut native_iteration = ();
            #[cfg(not(target_vendor = "apple"))]
            let mut defines = MacroDictionary::default();
            insert_static_define!(&mut defines, &native_iteration, GLSL_VERTEX, "");
            insert_static_define!(&mut defines, &native_iteration, GLSL_FRAGMENT, "");
            for i in 0..K_SHADER_FEATURE_COUNT {
                let feature = ShaderFeatures(1 << i);
                if shaderFeatures.0 & feature.0 != 0 {
                    let macro_name = get_shader_feature_glsl_name(feature);
                    insert_dynamic_define!(&mut defines, &native_iteration, macro_name);
                }
            }
            if interlockMode == InterlockMode::atomics {
                // Atomic mode uses device buffers instead of framebuffer fetches.
                insert_static_define!(
                    &mut defines,
                    &native_iteration,
                    GLSL_PLS_IMPL_DEVICE_BUFFER,
                    ""
                );
                if unsafe {
                    core::ptr::read(metalFeatures.cast::<MetalFeatures>()).atomicBarrierType
                } == AtomicBarrierType::rasterOrderGroup
                {
                    insert_static_define!(
                        &mut defines,
                        &native_iteration,
                        GLSL_PLS_IMPL_DEVICE_BUFFER_RASTER_ORDERED,
                        ""
                    );
                }
                if shaderMiscFlags.0 & ShaderMiscFlags::fixedFunctionColorOutput.0 != 0 {
                    insert_static_define!(
                        &mut defines,
                        &native_iteration,
                        GLSL_FIXED_FUNCTION_COLOR_OUTPUT,
                        ""
                    );
                }
            }
            if shaderMiscFlags.0 & ShaderMiscFlags::clockwiseFill.0 != 0 {
                insert_static_define!(&mut defines, &native_iteration, GLSL_CLOCKWISE_FILL, "1");
            }

            #[cfg(not(target_vendor = "apple"))]
            let mut source = String::from(sources.metal);
            append_source_prefix!(
                &mut source,
                &native_iteration,
                sources.constants,
                sources.flush_uniforms,
                sources.common,
            );
            if shaderFeatures.0 & ShaderFeatures::ENABLE_ADVANCED_BLEND.0 != 0 {
                append_source_fragment!(&mut source, &native_iteration, sources.advanced_blend);
            }

            match drawType {
                DrawType::midpointFanPatches
                | DrawType::midpointFanCenterAAPatches
                | DrawType::outerCurvePatches => {
                    // Add baseInstance to instanceID for path draws.
                    insert_static_define!(
                        &mut defines,
                        &native_iteration,
                        GLSL_ENABLE_INSTANCE_INDEX,
                        ""
                    );
                    insert_static_define!(&mut defines, &native_iteration, GLSL_DRAW_PATH, "");
                }
                DrawType::interiorTriangulation => {
                    insert_static_define!(
                        &mut defines,
                        &native_iteration,
                        GLSL_DRAW_INTERIOR_TRIANGLES,
                        ""
                    );
                }
                DrawType::featherAtlasBlit => {
                    insert_static_define!(
                        &mut defines,
                        &native_iteration,
                        GLSL_FEATHER_ATLAS_BLIT,
                        "1"
                    );
                }
                DrawType::imageRect => {
                    if cfg!(all(target_os = "ios", not(target_abi = "sim"))) {
                        rive_unreachable();
                    } else {
                        debug_assert_eq!(interlockMode, InterlockMode::atomics);
                        insert_static_define!(&mut defines, &native_iteration, GLSL_DRAW_IMAGE, "");
                        insert_static_define!(
                            &mut defines,
                            &native_iteration,
                            GLSL_DRAW_IMAGE_RECT,
                            ""
                        );
                    }
                }
                DrawType::imageMesh => {
                    insert_static_define!(&mut defines, &native_iteration, GLSL_DRAW_IMAGE, "");
                    insert_static_define!(
                        &mut defines,
                        &native_iteration,
                        GLSL_DRAW_IMAGE_MESH,
                        ""
                    );
                }
                DrawType::renderPassInitialize => {
                    if cfg!(all(target_os = "ios", not(target_abi = "sim"))) {
                        rive_unreachable();
                    } else {
                        debug_assert_eq!(interlockMode, InterlockMode::atomics);
                        insert_static_define!(
                            &mut defines,
                            &native_iteration,
                            GLSL_DRAW_RENDER_TARGET_UPDATE_BOUNDS,
                            ""
                        );
                        insert_static_define!(
                            &mut defines,
                            &native_iteration,
                            GLSL_INITIALIZE_PLS,
                            ""
                        );
                        if shaderMiscFlags.0 & ShaderMiscFlags::storeColorClear.0 != 0 {
                            // Preserve "true" instead of an empty string.
                            insert_static_define!(
                                &mut defines,
                                &native_iteration,
                                GLSL_STORE_COLOR_CLEAR,
                                "true"
                            );
                        }
                        debug_assert!(
                            shaderMiscFlags.0 & ShaderMiscFlags::loadColorFromDstTexture.0 == 0
                        );
                        if shaderMiscFlags.0 & ShaderMiscFlags::swizzleColorBGRAToRGBA.0 != 0 {
                            insert_static_define!(
                                &mut defines,
                                &native_iteration,
                                GLSL_SWIZZLE_COLOR_BGRA_TO_RGBA,
                                ""
                            );
                        }
                    }
                }
                DrawType::renderPassResolve => {
                    if cfg!(all(target_os = "ios", not(target_abi = "sim"))) {
                        rive_unreachable();
                    } else {
                        debug_assert_eq!(interlockMode, InterlockMode::atomics);
                        insert_static_define!(
                            &mut defines,
                            &native_iteration,
                            GLSL_DRAW_RENDER_TARGET_UPDATE_BOUNDS,
                            ""
                        );
                        insert_static_define!(
                            &mut defines,
                            &native_iteration,
                            GLSL_RESOLVE_PLS,
                            ""
                        );
                        if shaderMiscFlags.0 & ShaderMiscFlags::coalescedResolveAndTransfer.0 != 0 {
                            insert_static_define!(
                                &mut defines,
                                &native_iteration,
                                GLSL_COALESCED_PLS_RESOLVE_AND_TRANSFER,
                                ""
                            );
                        }
                    }
                }
                DrawType::msaaStrokes
                | DrawType::msaaMidpointFanBorrowedCoverage
                | DrawType::msaaDynamicMidpointFans
                | DrawType::msaaMidpointFans
                | DrawType::msaaMidpointFanStencilReset
                | DrawType::msaaMidpointFanPathsStencil
                | DrawType::msaaMidpointFanPathsCover
                | DrawType::msaaOuterCubics
                | DrawType::clipReset => rive_unreachable(),
            }

            if !cfg!(all(target_os = "ios", not(target_abi = "sim")))
                && interlockMode == InterlockMode::atomics
            {
                append_source_fragment!(&mut source, &native_iteration, sources.draw_path_common);
                append_source_fragment!(&mut source, &native_iteration, sources.atomic_draw);
            } else {
                debug_assert_eq!(interlockMode, InterlockMode::rasterOrdering);
                match drawType {
                    DrawType::midpointFanPatches
                    | DrawType::midpointFanCenterAAPatches
                    | DrawType::outerCurvePatches
                    | DrawType::interiorTriangulation => {
                        append_source_fragment!(
                            &mut source,
                            &native_iteration,
                            sources.draw_path_common,
                        );
                        append_source_fragment!(
                            &mut source,
                            &native_iteration,
                            sources.draw_path_vert,
                        );
                        append_source_fragment!(
                            &mut source,
                            &native_iteration,
                            sources.draw_raster_order_path_frag,
                        );
                    }
                    DrawType::featherAtlasBlit => {
                        append_source_fragment!(
                            &mut source,
                            &native_iteration,
                            sources.draw_path_common,
                        );
                        append_source_fragment!(
                            &mut source,
                            &native_iteration,
                            sources.draw_path_vert,
                        );
                        append_source_fragment!(
                            &mut source,
                            &native_iteration,
                            sources.draw_mesh_frag,
                        );
                    }
                    DrawType::imageMesh => {
                        append_source_fragment!(
                            &mut source,
                            &native_iteration,
                            sources.draw_image_mesh_vert,
                        );
                        append_source_fragment!(
                            &mut source,
                            &native_iteration,
                            sources.draw_mesh_frag,
                        );
                    }
                    DrawType::imageRect
                    | DrawType::msaaStrokes
                    | DrawType::msaaMidpointFanBorrowedCoverage
                    | DrawType::msaaDynamicMidpointFans
                    | DrawType::msaaMidpointFans
                    | DrawType::msaaMidpointFanStencilReset
                    | DrawType::msaaMidpointFanPathsStencil
                    | DrawType::msaaMidpointFanPathsCover
                    | DrawType::msaaOuterCubics
                    | DrawType::clipReset
                    | DrawType::renderPassInitialize
                    | DrawType::renderPassResolve => rive_unreachable(),
                }
            }

            let mut err: Option<MetalCompileError> = None;
            let languageVersion = if cfg!(target_os = "ios") {
                // On ios, we need version 2.2+.
                MetalLanguageVersion::Version2_2
            } else {
                // On mac, we need version 2.3+.
                MetalLanguageVersion::Version2_3
            };
            let preserveInvariance = unsafe { (&*gpu).preserveInvarianceAvailable() };
            #[cfg(target_vendor = "apple")]
            native_iteration.finish_options_values(languageVersion, true, preserveInvariance);
            #[cfg(not(target_vendor = "apple"))]
            let compileOptions = MetalCompileOptions {
                languageVersion,
                fastMathEnabled: true,
                preserveInvariance,
                preprocessorMacros: defines,
            };

            #[cfg(target_vendor = "apple")]
            let mut native_iteration_owner = Some(native_iteration);

            #[cfg(feature = "with-rive-tools")]
            let synthesize_failure =
                job_value.synthesizedFailureType == SynthesizedFailureType::shaderCompilation;
            #[cfg(not(feature = "with-rive-tools"))]
            let synthesize_failure = false;

            if synthesize_failure {
                debug_assert!(job_value.compiledLibrary.is_none());
            } else {
                #[cfg(target_vendor = "apple")]
                let creation = unsafe {
                    (&*gpu).newLibraryWithSource(
                        native_iteration_owner
                            .take()
                            .expect("native iteration owner exists before compile"),
                    )
                };
                #[cfg(target_vendor = "apple")]
                {
                    native_iteration_owner = Some(creation.native_owners);
                }
                #[cfg(not(target_vendor = "apple"))]
                let creation = unsafe { (&*gpu).newLibraryWithSource(&source, &compileOptions) };
                job_value.compiledLibrary = creation.library;
                err = creation.error;
            }

            lock = lock_recovering_poison(unsafe { &*mutex });

            if err.is_some() || job_value.compiledLibrary.is_none() {
                if synthesize_failure {
                    eprintln!("RIVE: Synthesizing shader compilation failure...");
                } else {
                    // Preserve the source fallback-to-uber-shader route.
                    #[cfg(target_vendor = "apple")]
                    let source_for_log = native_iteration_owner
                        .as_ref()
                        .map(NativeCompileIteration::source_for_log)
                        .expect("native iteration owner survives through logging");
                    #[cfg(not(target_vendor = "apple"))]
                    let source_for_log = source.clone();
                    let mut lineNumber = 1;
                    for line in source_for_log.lines() {
                        eprintln!("RIVE: {:4}| {}", lineNumber, line);
                        lineNumber += 1;
                    }
                    #[cfg(target_vendor = "apple")]
                    let description = err
                        .as_ref()
                        .and_then(|error| {
                            native_iteration_owner
                                .as_ref()
                                .and_then(|iteration| error.localized_description(iteration))
                        })
                        .unwrap_or_else(|| "<nil>".to_owned());
                    #[cfg(not(target_vendor = "apple"))]
                    let description = err
                        .as_ref()
                        .and_then(MetalCompileError::localized_description)
                        .unwrap_or_else(|| "<nil>".to_owned());
                    eprintln!("RIVE: Shader compilation error: {}", description);
                }

                eprintln!("RIVE: Failed to compile shader.");
                if !synthesize_failure {
                    // A Rust assertion unwind would strand the source worker
                    // protocol. Preserve C assert's process boundary instead.
                    debug_assert_abort!();
                }
            }

            unsafe {
                (&mut *(*finishedJobs).get()).push(job_value);
            }
            unsafe { (&*workFinishedCondition).notify_all() };
            #[cfg(target_vendor = "apple")]
            if let Some(mut native_iteration_owner) = native_iteration_owner {
                owner_detail_event(
                    "BG-DICT-DEFINES",
                    "AliveAt(finished push)",
                    objc_retained_identity(&*native_iteration_owner.macros),
                );
                owner_detail_event(
                    "BG-NS-SOURCE",
                    "AliveAt(finished push)",
                    objc_retained_identity(&*native_iteration_owner.source),
                );
                if let Some(options) = unsafe { (&*native_iteration_owner.options).as_ref() } {
                    owner_detail_event(
                        "BG-COMPILE-OPTIONS",
                        "AliveAt(finished push)",
                        objc_retained_identity(options),
                    );
                }
                if let Some(error) = unsafe { (&*native_iteration_owner.error).as_ref() } {
                    owner_detail_event(
                        "BG-ERR-COMPILE",
                        "AliveAt(finished push)",
                        objc_retained_identity(error),
                    );
                }
                // Pinned local destruction: compile options, then NSError
                // (already logged), then mutable source, then macro defines.
                native_iteration_owner.drop_options();
                drop(err);
                drop(native_iteration_owner);
            }
        }
    }
}

impl BackgroundShaderCompiler {
    /// Request shutdown and detach the worker handle while borrowing the
    /// pinned pointee through shared access.  The owner joins the returned
    /// handle before obtaining an exclusive reference to the pointee, so the
    /// worker's raw `this` access cannot overlap member destruction.
    fn request_shutdown(&self) -> Option<JoinHandle<()>> {
        let lock = lock_recovering_poison(&self.m_mutex);
        let compilerThread = unsafe { (&mut *self.m_compilerThread.get()).take() };
        let should_notify = compilerThread.is_some();
        if should_notify {
            unsafe { *self.m_shouldQuit.get() = true };
        }
        drop(lock);
        // Match the source's lock -> quit -> unlock -> notify -> join order.
        // The worker must be able to reacquire the mutex before waking it.
        if should_notify {
            self.m_workAddedCondition.notify_all();
        }
        compilerThread
    }

    /// Destroy the source members in the exact reverse declaration order.
    /// This is called only after `request_shutdown`'s worker has been joined.
    unsafe fn drop_members(&mut self) {
        // The source destructor destroys members in reverse declaration order;
        // ManuallyDrop keeps Rust from silently using declaration order.
        unsafe { ManuallyDrop::drop(&mut self.m_compilerThread) };
        unsafe { ManuallyDrop::drop(&mut self.m_workFinishedCondition) };
        unsafe { ManuallyDrop::drop(&mut self.m_workAddedCondition) };
        unsafe { ManuallyDrop::drop(&mut self.m_mutex) };
        unsafe { ManuallyDrop::drop(&mut self.m_finishedJobs) };
        unsafe { ManuallyDrop::drop(&mut self.m_pendingJobs) };
        unsafe { ManuallyDrop::drop(&mut self.m_metalFeatures) };
        unsafe { ManuallyDrop::drop(&mut self.m_gpu) };
    }
}

impl Drop for BackgroundShaderCompilerOwner {
    fn drop(&mut self) {
        let thread = self.inner.as_ref().get_ref().request_shutdown();
        if let Some(thread) = thread {
            #[cfg(target_vendor = "apple")]
            owner_detail_event("BG-GPU-MEMBER", "ShutdownJoin", unsafe {
                metal_retained_identity(&self.inner.as_ref().get_ref().m_gpu.retained)
            });
            thread.join().expect("compiler thread joined");
        }
        // SAFETY: the worker was detached and joined above, so no thread can
        // access the pinned pointee while its source members are destroyed.
        unsafe { self.inner.as_mut().get_unchecked_mut().drop_members() };
    }
}
fn lock_recovering_poison<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

fn wait_recovering_poison<'a, T>(
    condition: &Condvar,
    guard: MutexGuard<'a, T>,
) -> MutexGuard<'a, T> {
    condition
        .wait(guard)
        .unwrap_or_else(|poison| poison.into_inner())
}

#[cfg(test)]
mod tests {
    use super::{BackgroundOwnerDetailEvent, BackgroundShaderCompiler};

    const EXPECTATIONS: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/metal-port-reports/metal-native-owner-expectations.tsv"
    ));

    const BG_ROWS: [&str; 10] = [
        "BG-DICT-DEFINES",
        "BG-NS-MACRO-KEY-DYNAMIC",
        "BG-NS-MACRO-LITERALS",
        "BG-NS-SOURCE",
        "BG-NS-APPEND-TEMP",
        "BG-COMPILE-OPTIONS",
        "BG-ERR-COMPILE",
        "BG-NS-ERR-DESC",
        "BG-LIB-COMPILED",
        "BG-GPU-MEMBER",
    ];

    #[test]
    fn production_shader_sources_are_exact_translated_minifier_outputs() {
        use crate::mechanical_port::source::renderer::src::shaders::{
            advanced_blend_glsl, atomic_draw_glsl, common_glsl, constants_glsl,
            draw_image_mesh_vert, draw_mesh_frag, draw_path_common_glsl, draw_path_vert,
            draw_raster_order_path_frag, flush_uniforms_glsl, metal_glsl,
        };

        let actual = super::generated_shader_sources();
        let expected = super::GeneratedShaderSources {
            metal: super::runtime_generated_shader_sources::METAL,
            constants: super::runtime_generated_shader_sources::CONSTANTS,
            flush_uniforms: super::runtime_generated_shader_sources::FLUSH_UNIFORMS,
            common: super::runtime_generated_shader_sources::COMMON,
            advanced_blend: super::runtime_generated_shader_sources::ADVANCED_BLEND,
            draw_path_common: super::runtime_generated_shader_sources::DRAW_PATH_COMMON,
            draw_path_vert: super::runtime_generated_shader_sources::DRAW_PATH_VERT,
            draw_raster_order_path_frag:
                super::runtime_generated_shader_sources::DRAW_RASTER_ORDER_PATH_FRAG,
            draw_image_mesh_vert: super::runtime_generated_shader_sources::DRAW_IMAGE_MESH_VERT,
            draw_mesh_frag: super::runtime_generated_shader_sources::DRAW_MESH_FRAG,
            atomic_draw: super::runtime_generated_shader_sources::ATOMIC_DRAW,
        };
        assert_eq!(actual, expected, "runtime must use build-generated bytes");

        let generated = [
            actual.metal,
            actual.constants,
            actual.flush_uniforms,
            actual.common,
            actual.advanced_blend,
            actual.draw_path_common,
            actual.draw_path_vert,
            actual.draw_raster_order_path_frag,
            actual.draw_image_mesh_vert,
            actual.draw_mesh_frag,
            actual.atomic_draw,
        ];

        macro_rules! minifier_output {
            ($name:literal) => {
                include_str!(concat!(
                    env!("OUT_DIR"),
                    "/mechanical_shader_generated/",
                    $name
                ))
            };
        }
        let embedded_headers = [
            minifier_output!("metal.glsl.hpp"),
            minifier_output!("constants.glsl.hpp"),
            minifier_output!("flush_uniforms.glsl.hpp"),
            minifier_output!("common.glsl.hpp"),
            minifier_output!("advanced_blend.glsl.hpp"),
            minifier_output!("draw_path_common.glsl.hpp"),
            minifier_output!("draw_path.vert.hpp"),
            minifier_output!("draw_raster_order_path.frag.hpp"),
            minifier_output!("draw_image_mesh.vert.hpp"),
            minifier_output!("draw_mesh.frag.hpp"),
            minifier_output!("atomic_draw.glsl.hpp"),
        ];
        for (runtime, embedded) in generated.iter().zip(embedded_headers) {
            let payload_start = embedded.find("R\"===(").unwrap() + "R\"===(".len();
            let payload_end = embedded[payload_start..].find(")===" ).unwrap() + payload_start;
            assert_eq!(
                *runtime,
                &embedded[payload_start..payload_end],
                "runtime fragment must be the exact pinned embedded-string payload"
            );
            assert!(runtime.ends_with('\n'));
        }
        for source in generated {
            assert!(!source.is_empty());
            assert!(
                !source.as_bytes().windows(2).any(|window| {
                    matches!(window[0], b'$' | b'@')
                        && (window[1].is_ascii_alphabetic() || window[1] == b'_')
                }),
                "translated minifier output retained a source substitution token"
            );
        }

        let raw = [
            metal_glsl::PINNED_METAL_GLSL_SOURCE,
            constants_glsl::PINNED_CONSTANTS_GLSL_SOURCE,
            flush_uniforms_glsl::PINNED_FLUSH_UNIFORMS_GLSL_SOURCE,
            common_glsl::PINNED_COMMON_GLSL_SOURCE,
            advanced_blend_glsl::PINNED_ADVANCED_BLEND_GLSL_SOURCE,
            draw_path_common_glsl::PINNED_DRAW_PATH_COMMON_GLSL_SOURCE,
            draw_path_vert::PINNED_DRAW_PATH_VERT_SOURCE,
            draw_raster_order_path_frag::PINNED_DRAW_RASTER_ORDER_PATH_FRAG_SOURCE,
            draw_image_mesh_vert::PINNED_DRAW_IMAGE_MESH_VERT_SOURCE,
            draw_mesh_frag::PINNED_DRAW_MESH_FRAG_SOURCE,
            atomic_draw_glsl::PINNED_ATOMIC_DRAW_SOURCE,
        ];
        assert!(generated
            .iter()
            .zip(raw)
            .all(|(generated, raw)| *generated != raw));
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn production_generated_sources_compile_a_real_dynamic_pipeline_job() {
        use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp as gpu;
        use crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::{MetalFeatures, Retained};
        use objc2::rc::Retained as ObjcRetained;

        objc2::rc::autoreleasepool(|_| {
            let device =
                objc2_metal::MTLCreateSystemDefaultDevice().expect("native Metal device required");
            let device =
                unsafe { Retained::from_raw_retained(ObjcRetained::into_raw(device).cast()) }
                    .expect("device transfer");
            let compiler = super::new_for_device(device, MetalFeatures::default());
            compiler.pushJob(super::BackgroundCompileJob::new(
                gpu::DrawType::midpointFanPatches,
                gpu::ShaderFeatures::ENABLE_DITHER,
                gpu::InterlockMode::rasterOrdering,
                gpu::ShaderMiscFlags::none,
            ));
            let mut finished = super::BackgroundCompileJob::new(
                gpu::DrawType::midpointFanPatches,
                gpu::ShaderFeatures::ENABLE_DITHER,
                gpu::InterlockMode::rasterOrdering,
                gpu::ShaderMiscFlags::none,
            );
            assert!(compiler.popFinishedJob(&mut finished, true));
            assert!(
                finished.compiledLibrary.is_some(),
                "the real production-generated source must compile"
            );
            let library = finished.compiledLibrary.as_ref().unwrap();
            let library = unsafe {
                &*library
                    .as_ptr()
                    .cast::<objc2::runtime::ProtocolObject<dyn objc2_metal::MTLLibrary>>()
            };
            for export in [
                super::runtime_generated_shader_exports::GLSL_drawVertexMain,
                super::runtime_generated_shader_exports::GLSL_drawFragmentMain,
            ] {
                let name = objc2_foundation::NSString::from_str(export);
                let function: Option<objc2::rc::Retained<objc2::runtime::AnyObject>> =
                    unsafe { objc2::msg_send![library, newFunctionWithName: &*name] };
                assert!(
                    function.is_some(),
                    "production dynamic library must export generated function {export}"
                );
            }
        });
    }

    #[test]
    fn background_expectations_are_checked_in_and_have_all_rows() {
        let rows = EXPECTATIONS
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with("id\t"))
            .filter(|line| {
                BG_ROWS
                    .iter()
                    .any(|id| line.starts_with(&format!("{id}\t")))
            })
            .collect::<Vec<_>>();
        assert_eq!(rows.len(), BG_ROWS.len());
        for (row, expected_id) in rows.iter().zip(BG_ROWS) {
            let columns = row.split('\t').collect::<Vec<_>>();
            assert_eq!(columns.len(), 11, "{expected_id} expectation schema");
            assert_eq!(columns[0], expected_id);
            assert!(!columns[4].is_empty(), "{expected_id} phase contract");
            assert!(!columns[6].is_empty(), "{expected_id} identity contract");
            assert!(!columns[7].is_empty(), "{expected_id} release contract");
            assert!(!columns[9].is_empty(), "{expected_id} probe contract");
        }
    }

    fn events<'a>(
        all: &'a [BackgroundOwnerDetailEvent],
        id: &str,
    ) -> Vec<&'a BackgroundOwnerDetailEvent> {
        all.iter().filter(|event| event.ledger_id == id).collect()
    }

    fn assert_identity_pairs(all: &[BackgroundOwnerDetailEvent], id: &str) {
        let row = events(all, id);
        let creates = row.iter().filter(|event| event.phase == "Create");
        let releases = row.iter().filter(|event| event.phase == "Release");
        let creates = creates.map(|event| event.identity).collect::<Vec<_>>();
        let releases = releases.map(|event| event.identity).collect::<Vec<_>>();
        assert!(!creates.is_empty(), "{id} has no source Create event");
        assert_eq!(creates, releases, "{id} create/release identity or order");
        assert_eq!(row.first().unwrap().phase, "Create", "{id} source order");
        assert_eq!(row.last().unwrap().phase, "Release", "{id} source order");
    }

    fn assert_exact_identity_phases(
        all: &[BackgroundOwnerDetailEvent],
        id: &str,
        phases: &[&str],
    ) -> usize {
        let row = events(all, id);
        assert_eq!(
            row.iter().map(|event| event.phase).collect::<Vec<_>>(),
            phases,
            "{id} exact phase sequence"
        );
        assert!(!row.is_empty(), "{id} source path");
        assert!(row.iter().all(|event| event.identity == row[0].identity));
        row[0].identity
    }

    #[test]
    fn background_detail_validator_rejects_missing_or_swapped_lifecycle_events() {
        let events = vec![
            BackgroundOwnerDetailEvent {
                ledger_id: "BG-DICT-DEFINES",
                phase: "Create",
                identity: 1,
                related_identity: None,
            },
            BackgroundOwnerDetailEvent {
                ledger_id: "BG-DICT-DEFINES",
                phase: "Release",
                identity: 1,
                related_identity: None,
            },
        ];
        assert_identity_pairs(&events, "BG-DICT-DEFINES");

        let missing = vec![events[0]];
        assert!(
            std::panic::catch_unwind(|| assert_identity_pairs(&missing, "BG-DICT-DEFINES"))
                .is_err()
        );
        let swapped = vec![
            BackgroundOwnerDetailEvent {
                ledger_id: "BG-DICT-DEFINES",
                phase: "Release",
                identity: 1,
                related_identity: None,
            },
            BackgroundOwnerDetailEvent {
                ledger_id: "BG-DICT-DEFINES",
                phase: "Create",
                identity: 1,
                related_identity: None,
            },
        ];
        assert!(
            std::panic::catch_unwind(|| assert_identity_pairs(&swapped, "BG-DICT-DEFINES"))
                .is_err()
        );
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn native_append_format_sequence_matches_the_pinned_source_bytes() {
        use super::{NativeCompileIteration, take_owner_detail_events};

        objc2::rc::autoreleasepool(|_| {
            let _ = take_owner_detail_events();
            let iteration = NativeCompileIteration::new("metal-seed\n");
            iteration.append_source_prefix("constants", "flush-uniforms", "common");
            iteration.append_source("draw-path");
            iteration.append_source("draw-fragment");
            assert_eq!(
                iteration.source_for_log(),
                concat!(
                    "metal-seed\n",
                    "constants\n",
                    "flush-uniforms\n",
                    "common\n",
                    "draw-path\n",
                    "draw-fragment\n",
                )
            );

            let detail = take_owner_detail_events();
            let source_identity = events(&detail, "BG-NS-SOURCE")
                .into_iter()
                .find(|event| event.phase == "Create")
                .unwrap()
                .identity;
            let formats = events(&detail, "BG-NS-APPEND-TEMP");
            assert_eq!(
                formats
                    .iter()
                    .filter(|event| event.phase == "BorrowFormat")
                    .count(),
                3,
                "one three-argument prefix append and two single-fragment appends"
            );
            assert!(formats
                .iter()
                .all(|event| event.related_identity == Some(source_identity)));
        });
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn static_macro_literals_have_one_complete_stable_native_identity_census() {
        let identities = super::SOURCE_MACRO_LITERAL_TEXTS
            .iter()
            .map(|text| {
                let first = super::source_macro_literal(text);
                let second = super::source_macro_literal(text);
                assert!(core::ptr::eq(first, second));
                assert_eq!(first.to_string(), *text);
                first as *const _ as usize
            })
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(identities.len(), super::SOURCE_MACRO_LITERAL_TEXTS.len());
        assert!(super::SOURCE_MACRO_LITERAL_TEXTS.contains(&""));
        assert!(super::SOURCE_MACRO_LITERAL_TEXTS.contains(&"1"));
        assert!(super::SOURCE_MACRO_LITERAL_TEXTS.contains(&"true"));
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn native_iteration_uses_its_exact_dictionary_source_and_options_owners() {
        use super::{MetalCompileOptions, MetalLanguageVersion, NativeCompileIteration};

        objc2::rc::autoreleasepool(|_| {
            let mut iteration = NativeCompileIteration::new("using namespace metal;\n");
            iteration.define_static(
                super::source_macro_literal(super::GLSL_VERTEX),
                super::source_macro_literal(""),
            );
            iteration.finish_options(&MetalCompileOptions {
                languageVersion: MetalLanguageVersion::Version2_3,
                fastMathEnabled: true,
                preserveInvariance: false,
                preprocessorMacros: super::MacroDictionary::default(),
            });

            let macros_identity = super::objc_retained_identity(&*iteration.macros);
            let source_identity = super::objc_retained_identity(&*iteration.source);
            let options = unsafe { (&*iteration.options).as_ref() }
                .expect("source compile options owner");
            let options_identity = super::objc_retained_identity(options);
            let configured_macros = options
                .preprocessorMacros()
                .expect("compile options retain the source dictionary");
            assert_ne!(
                objc2::rc::Retained::as_ptr(&configured_macros).cast::<()>() as usize,
                macros_identity,
                "the source property copies the mutable dictionary; it is not a second local"
            );
            assert_eq!(configured_macros.count(), 1);
            assert!(configured_macros
                .objectForKey(super::source_macro_literal(super::GLSL_VERTEX))
                .is_some());
            assert_ne!(macros_identity, source_identity);
            assert_ne!(source_identity, options_identity);
            assert_ne!(options_identity, macros_identity);

            iteration.drop_options();
            drop(iteration);
        });
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn native_iteration_unwind_releases_options_source_and_defines_once_in_reverse_order() {
        use super::{
            MetalCompileOptions, MetalLanguageVersion, NativeCompileIteration,
            take_owner_detail_events,
        };

        let _ = take_owner_detail_events();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut iteration = NativeCompileIteration::new("metal-seed\n");
            iteration.define_static(
                objc2_foundation::ns_string!("GLSL_VERTEX"),
                objc2_foundation::ns_string!(""),
            );
            iteration.append_source_prefix("constants", "flush", "common");
            iteration.finish_options(&MetalCompileOptions {
                languageVersion: MetalLanguageVersion::Version2_3,
                fastMathEnabled: true,
                preserveInvariance: false,
                preprocessorMacros: super::MacroDictionary::default(),
            });
            panic!("injected after finish_options");
        }));
        assert!(result.is_err());

        let detail = take_owner_detail_events();
        for id in ["BG-DICT-DEFINES", "BG-NS-SOURCE", "BG-COMPILE-OPTIONS"] {
            assert_identity_pairs(&detail, id);
        }
        let release_order = ["BG-COMPILE-OPTIONS", "BG-NS-SOURCE", "BG-DICT-DEFINES"]
            .map(|id| {
                detail
                    .iter()
                    .position(|event| event.ledger_id == id && event.phase == "Release")
                    .unwrap()
            });
        assert!(release_order.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(events(&detail, "BG-ERR-COMPILE").is_empty());
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn native_compile_error_keeps_nserror_and_description_through_log_then_releases_reverse() {
        use super::{
            MetalCompileOptions, MetalLanguageVersion, NativeCompileIteration,
            native_new_library_with_source, take_owner_detail_events,
        };

        objc2::rc::autoreleasepool(|_| {
            let _ = take_owner_detail_events();
            let device = objc2_metal::MTLCreateSystemDefaultDevice()
                .expect("native Metal device required");
            let mut iteration = NativeCompileIteration::new(concat!(
                "#include <metal_stdlib>\n",
                "using namespace metal;\n",
                "#error exact-owner-failure\n",
            ));
            iteration.define_static(
                objc2_foundation::ns_string!("GLSL_VERTEX"),
                objc2_foundation::ns_string!(""),
            );
            iteration.define_dynamic(
                super::runtime_generated_shader_exports::GLSL_ENABLE_CLIPPING,
                objc2_foundation::ns_string!("1"),
            );
            iteration.append_source_prefix("", "", "");
            iteration.finish_options(&MetalCompileOptions {
                languageVersion: MetalLanguageVersion::Version2_3,
                fastMathEnabled: true,
                preserveInvariance: false,
                preprocessorMacros: super::MacroDictionary::default(),
            });
            let mut creation = unsafe {
                native_new_library_with_source(
                    objc2::rc::Retained::as_ptr(&device).cast_mut().cast(),
                    iteration,
                )
            };
            assert!(creation.library.is_null());
            let error = creation.error.take().expect("native NSError result");
            let _source_listing = creation.native_owners.source_for_log();
            let description = error
                .localized_description(&creation.native_owners)
                .expect("localizedDescription expression");
            assert!(description.contains("exact-owner-failure"));
            creation.native_owners.drop_options();
            drop(error);
            drop(creation.native_owners);

            let detail = take_owner_detail_events();
            assert_exact_identity_phases(
                &detail,
                "BG-DICT-DEFINES",
                &["Create", "LastUse(options setter)", "Release"],
            );
            assert_exact_identity_phases(
                &detail,
                "BG-NS-SOURCE",
                &["Create", "LastUse(compile)", "LastUse(log)", "Release"],
            );
            assert_exact_identity_phases(
                &detail,
                "BG-COMPILE-OPTIONS",
                &["Create", "LastUse(newLibrary)", "Release"],
            );
            let error_identity = assert_exact_identity_phases(
                &detail,
                "BG-ERR-COMPILE",
                &["Create", "TransferToIteration", "LastUse(log)", "Release"],
            );
            let description = events(&detail, "BG-NS-ERR-DESC");
            assert_eq!(
                description.iter().map(|event| event.phase).collect::<Vec<_>>(),
                vec!["Borrow", "LastUse(log)", "ExpressionEnd"]
            );
            assert!(description
                .iter()
                .all(|event| event.identity == description[0].identity));
            assert_ne!(description[0].identity, error_identity);
            let release_order = [
                "BG-COMPILE-OPTIONS",
                "BG-ERR-COMPILE",
                "BG-NS-SOURCE",
                "BG-DICT-DEFINES",
            ]
            .map(|id| {
                detail
                    .iter()
                    .position(|event| event.ledger_id == id && event.phase == "Release")
                    .unwrap()
            });
            assert!(release_order.windows(2).all(|pair| pair[0] < pair[1]));
            let description_end = detail
                .iter()
                .position(|event| {
                    event.ledger_id == "BG-NS-ERR-DESC"
                        && event.phase == "ExpressionEnd"
                })
                .unwrap();
            assert!(description_end < release_order[1]);

            let dynamic = events(&detail, "BG-NS-MACRO-KEY-DYNAMIC");
            assert_eq!(
                dynamic.iter().map(|event| event.phase).collect::<Vec<_>>(),
                vec!["Borrow", "LastUse(dictionary set)", "AutoreleaseEnd"]
            );
            assert!(dynamic
                .iter()
                .all(|event| event.identity == dynamic[0].identity));
            let dynamic_key = unsafe {
                &*(dynamic[0].identity as *const objc2_foundation::NSString)
            };
            assert_eq!(
                dynamic_key.to_string(),
                super::runtime_generated_shader_exports::GLSL_ENABLE_CLIPPING
            );
            assert!(dynamic.iter().all(|event| {
                event.related_identity
                    == Some(super::source_macro_literal("1") as *const _ as usize)
            }));
        });
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn background_detail_success_path_binds_native_identity_and_reverse_release() {
        use super::{new_for_device_with_sources, take_owner_detail_events, take_owner_events};
        use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp as gpu;
        use crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::{MetalFeatures, Retained};
        use objc2::rc::Retained as ObjcRetained;
        use objc2_foundation::NSObjectProtocol;

        objc2::rc::autoreleasepool(|_| {
            let _ = take_owner_events();
            let _ = take_owner_detail_events();
            let device =
                objc2_metal::MTLCreateSystemDefaultDevice().expect("native Metal device required");
            let device_observer = device.clone();
            let retained_before_compiler = device_observer.retainCount();
            let device =
                unsafe { Retained::from_raw_retained(ObjcRetained::into_raw(device).cast()) }
                    .expect("device transfer");
            let mut compiler = new_for_device_with_sources(
                device,
                MetalFeatures::default(),
                super::GeneratedShaderSources {
                    metal: "#include <metal_stdlib>\nusing namespace metal;\n",
                    constants: "",
                    flush_uniforms: "",
                    common: "",
                    advanced_blend: "",
                    draw_path_common: "",
                    draw_path_vert: "",
                    draw_raster_order_path_frag: "",
                    draw_image_mesh_vert: "",
                    draw_mesh_frag: "",
                    atomic_draw: "",
                },
            );
            compiler.pushJob(super::BackgroundCompileJob::new(
                gpu::DrawType::imageMesh,
                gpu::ShaderFeatures::ENABLE_CLIPPING,
                gpu::InterlockMode::rasterOrdering,
                gpu::ShaderMiscFlags::none,
            ));
            let mut finished = super::BackgroundCompileJob::new(
                gpu::DrawType::imageMesh,
                gpu::ShaderFeatures::ENABLE_CLIPPING,
                gpu::InterlockMode::rasterOrdering,
                gpu::ShaderMiscFlags::none,
            );
            assert!(compiler.popFinishedJob(&mut finished, true));
            drop(finished);
            drop(compiler);
            assert_eq!(
                device_observer.retainCount() + 1,
                retained_before_compiler,
                "compiler device +1 releases only after shutdown/join"
            );

            let detail = take_owner_detail_events();
            for id in [
                "BG-DICT-DEFINES",
                "BG-NS-SOURCE",
                "BG-COMPILE-OPTIONS",
                "BG-GPU-MEMBER",
            ] {
                assert_identity_pairs(&detail, id);
            }
            assert_exact_identity_phases(
                &detail,
                "BG-DICT-DEFINES",
                &[
                    "Create",
                    "LastUse(options setter)",
                    "AliveAt(finished push)",
                    "Release",
                ],
            );
            assert_exact_identity_phases(
                &detail,
                "BG-NS-SOURCE",
                &[
                    "Create",
                    "LastUse(compile)",
                    "AliveAt(finished push)",
                    "Release",
                ],
            );
            assert_exact_identity_phases(
                &detail,
                "BG-COMPILE-OPTIONS",
                &[
                    "Create",
                    "LastUse(newLibrary)",
                    "AliveAt(finished push)",
                    "Release",
                ],
            );
            assert_exact_identity_phases(
                &detail,
                "BG-GPU-MEMBER",
                &["Create", "Borrow(worker)", "ShutdownJoin", "Release"],
            );
            let dictionary_identity = events(&detail, "BG-DICT-DEFINES")[0].identity;
            let source_identity = events(&detail, "BG-NS-SOURCE")[0].identity;
            let options_identity = events(&detail, "BG-COMPILE-OPTIONS")[0].identity;
            assert_eq!(
                events(&detail, "BG-DICT-DEFINES")
                    .iter()
                    .find(|event| event.phase == "LastUse(options setter)")
                    .unwrap()
                    .related_identity,
                Some(options_identity)
            );
            assert_eq!(
                events(&detail, "BG-NS-SOURCE")
                    .iter()
                    .find(|event| event.phase == "LastUse(compile)")
                    .unwrap()
                    .related_identity,
                Some(options_identity)
            );
            assert_eq!(
                events(&detail, "BG-COMPILE-OPTIONS")
                    .iter()
                    .find(|event| event.phase == "LastUse(newLibrary)")
                    .unwrap()
                    .related_identity,
                Some(source_identity)
            );
            assert_ne!(dictionary_identity, source_identity);
            assert_ne!(source_identity, options_identity);
            assert_exact_identity_phases(
                &detail,
                "BG-LIB-COMPILED",
                &["Create", "TransferJob", "TransferFinished", "ReleaseContext"],
            );
            for id in [
                "BG-NS-MACRO-KEY-DYNAMIC",
                "BG-NS-MACRO-LITERALS",
                "BG-NS-APPEND-TEMP",
            ] {
                assert!(!events(&detail, id).is_empty(), "{id} detail events");
            }
            assert_eq!(
                events(&detail, "BG-NS-MACRO-KEY-DYNAMIC")
                    .iter()
                    .filter(|event| event.phase == "Borrow")
                    .count(),
                1,
                "one dynamic feature key"
            );
            assert_eq!(
                events(&detail, "BG-NS-MACRO-LITERALS")
                    .iter()
                    .filter(|event| event.phase == "Borrow")
                    .count(),
                4,
                "four fixed macro keys"
            );
            let empty_identity = super::source_macro_literal("") as *const _ as usize;
            let expected_static_keys = [
                super::source_macro_literal(super::GLSL_VERTEX) as *const _ as usize,
                super::source_macro_literal(super::GLSL_FRAGMENT) as *const _ as usize,
                super::source_macro_literal(super::GLSL_DRAW_IMAGE) as *const _ as usize,
                super::source_macro_literal(super::GLSL_DRAW_IMAGE_MESH) as *const _ as usize,
            ];
            let literals = events(&detail, "BG-NS-MACRO-LITERALS");
            assert_eq!(literals.len(), 8);
            for (index, pair) in literals.chunks_exact(2).enumerate() {
                assert_eq!(pair[0].phase, "Borrow");
                assert_eq!(pair[1].phase, "LastUse(dictionary set)");
                assert_eq!(pair[0].identity, expected_static_keys[index]);
                assert_eq!(pair[1].identity, expected_static_keys[index]);
                assert_eq!(pair[0].related_identity, Some(empty_identity));
                assert_eq!(pair[1].related_identity, Some(empty_identity));
            }
            let dynamic = events(&detail, "BG-NS-MACRO-KEY-DYNAMIC");
            assert_eq!(
                dynamic.iter().map(|event| event.phase).collect::<Vec<_>>(),
                vec!["Borrow", "LastUse(dictionary set)", "AutoreleaseEnd"]
            );
            assert!(dynamic
                .iter()
                .all(|event| event.identity == dynamic[0].identity));
            assert!(dynamic.iter().all(|event| {
                event.related_identity
                    == Some(super::source_macro_literal("1") as *const _ as usize)
            }));
            assert_eq!(
                events(&detail, "BG-NS-APPEND-TEMP")
                    .iter()
                    .filter(|event| event.phase == "BorrowFormat")
                    .count(),
                3,
                "prefix plus two raster image fragments"
            );
            let source = events(&detail, "BG-NS-SOURCE");
            let source_id = source
                .iter()
                .find(|event| event.phase == "Create")
                .unwrap()
                .identity;
            let append = events(&detail, "BG-NS-APPEND-TEMP");
            assert!(append.iter().all(|event| {
                event.identity != source_id && event.related_identity == Some(source_id)
            }));
            let borrow_formats = append
                .iter()
                .filter(|event| event.phase == "BorrowFormat")
                .map(|event| event.identity)
                .collect::<Vec<_>>();
            assert_eq!(borrow_formats.len(), 3);
            assert_ne!(borrow_formats[0], borrow_formats[1]);
            assert_eq!(borrow_formats[1], borrow_formats[2]);
            let options_release = detail
                .iter()
                .position(|event| {
                    event.ledger_id == "BG-COMPILE-OPTIONS" && event.phase == "Release"
                })
                .unwrap();
            let source_release = detail
                .iter()
                .position(|event| event.ledger_id == "BG-NS-SOURCE" && event.phase == "Release")
                .unwrap();
            let dict_release = detail
                .iter()
                .position(|event| event.ledger_id == "BG-DICT-DEFINES" && event.phase == "Release")
                .unwrap();
            assert!(options_release < source_release && source_release < dict_release);
            let libraries = events(&detail, "BG-LIB-COMPILED");
            assert!(libraries.iter().any(|event| event.phase == "TransferJob"));
            assert!(
                libraries
                    .iter()
                    .any(|event| event.phase == "TransferFinished")
            );
            assert!(
                libraries
                    .iter()
                    .any(|event| event.phase == "ReleaseContext")
            );
            assert!(
                events(&detail, "BG-GPU-MEMBER")
                    .iter()
                    .any(|event| event.phase == "Borrow(worker)")
            );
            assert!(
                events(&detail, "BG-GPU-MEMBER")
                    .iter()
                    .any(|event| event.phase == "ShutdownJoin")
            );
        });
    }

    #[cfg(all(target_vendor = "apple", feature = "with-rive-tools"))]
    #[test]
    fn synthesized_nil_library_without_error_releases_all_iteration_owners() {
        use super::{new_for_device_with_sources, take_owner_detail_events};
        use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp as gpu;
        use crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::{MetalFeatures, Retained};
        use objc2::rc::Retained as ObjcRetained;

        objc2::rc::autoreleasepool(|_| {
            let _ = take_owner_detail_events();
            let device =
                objc2_metal::MTLCreateSystemDefaultDevice().expect("native Metal device required");
            let device =
                unsafe { Retained::from_raw_retained(ObjcRetained::into_raw(device).cast()) }
                    .expect("device transfer");
            let mut compiler = new_for_device_with_sources(
                device,
                MetalFeatures::default(),
                super::GeneratedShaderSources {
                    metal: "#include <metal_stdlib>\nusing namespace metal;\n",
                    constants: "",
                    flush_uniforms: "",
                    common: "",
                    advanced_blend: "",
                    draw_path_common: "",
                    draw_path_vert: "",
                    draw_raster_order_path_frag: "",
                    draw_image_mesh_vert: "",
                    draw_mesh_frag: "",
                    atomic_draw: "",
                },
            );
            let mut pending = super::BackgroundCompileJob::new(
                gpu::DrawType::imageMesh,
                gpu::ShaderFeatures::NONE,
                gpu::InterlockMode::rasterOrdering,
                gpu::ShaderMiscFlags::none,
            );
            pending.synthesizedFailureType = gpu::SynthesizedFailureType::shaderCompilation;
            compiler.pushJob(pending);
            let mut finished = super::BackgroundCompileJob::new(
                gpu::DrawType::imageMesh,
                gpu::ShaderFeatures::NONE,
                gpu::InterlockMode::rasterOrdering,
                gpu::ShaderMiscFlags::none,
            );
            assert!(compiler.popFinishedJob(&mut finished, true));
            assert!(finished.compiledLibrary.is_none());
            drop(finished);
            drop(compiler);

            let detail = take_owner_detail_events();
            assert_exact_identity_phases(
                &detail,
                "BG-DICT-DEFINES",
                &[
                    "Create",
                    "LastUse(options setter)",
                    "AliveAt(finished push)",
                    "Release",
                ],
            );
            for id in ["BG-NS-SOURCE", "BG-COMPILE-OPTIONS"] {
                assert_exact_identity_phases(
                    &detail,
                    id,
                    &["Create", "AliveAt(finished push)", "Release"],
                );
            }
            assert!(events(&detail, "BG-ERR-COMPILE").is_empty());
            assert!(events(&detail, "BG-NS-ERR-DESC").is_empty());
            assert!(events(&detail, "BG-LIB-COMPILED").is_empty());
            let release_order = ["BG-COMPILE-OPTIONS", "BG-NS-SOURCE", "BG-DICT-DEFINES"]
                .map(|id| {
                    detail
                        .iter()
                        .position(|event| event.ledger_id == id && event.phase == "Release")
                        .unwrap()
                });
            assert!(release_order.windows(2).all(|pair| pair[0] < pair[1]));
        });
    }

    #[cfg(all(target_vendor = "apple", not(debug_assertions)))]
    #[test]
    fn release_worker_error_keeps_all_locals_through_finished_push() {
        use super::{new_for_device_with_sources, take_owner_detail_events};
        use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp as gpu;
        use crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::{MetalFeatures, Retained};
        use objc2::rc::Retained as ObjcRetained;

        objc2::rc::autoreleasepool(|_| {
            let _ = take_owner_detail_events();
            let device =
                objc2_metal::MTLCreateSystemDefaultDevice().expect("native Metal device required");
            let device =
                unsafe { Retained::from_raw_retained(ObjcRetained::into_raw(device).cast()) }
                    .expect("device transfer");
            let compiler = new_for_device_with_sources(
                device,
                MetalFeatures::default(),
                super::GeneratedShaderSources {
                    metal: concat!(
                        "#include <metal_stdlib>\n",
                        "using namespace metal;\n",
                        "#error worker-owner-failure\n",
                    ),
                    constants: "",
                    flush_uniforms: "",
                    common: "",
                    advanced_blend: "",
                    draw_path_common: "",
                    draw_path_vert: "",
                    draw_raster_order_path_frag: "",
                    draw_image_mesh_vert: "",
                    draw_mesh_frag: "",
                    atomic_draw: "",
                },
            );
            compiler.pushJob(super::BackgroundCompileJob::new(
                gpu::DrawType::imageMesh,
                gpu::ShaderFeatures::ENABLE_CLIPPING,
                gpu::InterlockMode::rasterOrdering,
                gpu::ShaderMiscFlags::none,
            ));
            let mut finished = super::BackgroundCompileJob::new(
                gpu::DrawType::imageMesh,
                gpu::ShaderFeatures::NONE,
                gpu::InterlockMode::rasterOrdering,
                gpu::ShaderMiscFlags::none,
            );
            assert!(compiler.popFinishedJob(&mut finished, true));
            assert!(finished.compiledLibrary.is_none());
            drop(finished);
            drop(compiler);

            let detail = take_owner_detail_events();
            assert_exact_identity_phases(
                &detail,
                "BG-DICT-DEFINES",
                &[
                    "Create",
                    "LastUse(options setter)",
                    "AliveAt(finished push)",
                    "Release",
                ],
            );
            assert_exact_identity_phases(
                &detail,
                "BG-NS-SOURCE",
                &[
                    "Create",
                    "LastUse(compile)",
                    "LastUse(log)",
                    "AliveAt(finished push)",
                    "Release",
                ],
            );
            assert_exact_identity_phases(
                &detail,
                "BG-COMPILE-OPTIONS",
                &[
                    "Create",
                    "LastUse(newLibrary)",
                    "AliveAt(finished push)",
                    "Release",
                ],
            );
            let error_identity = assert_exact_identity_phases(
                &detail,
                "BG-ERR-COMPILE",
                &[
                    "Create",
                    "TransferToIteration",
                    "LastUse(log)",
                    "AliveAt(finished push)",
                    "Release",
                ],
            );
            let description = events(&detail, "BG-NS-ERR-DESC");
            assert_eq!(
                description.iter().map(|event| event.phase).collect::<Vec<_>>(),
                vec!["Borrow", "LastUse(log)", "ExpressionEnd"]
            );
            assert_ne!(description[0].identity, error_identity);
            let release_order = [
                "BG-COMPILE-OPTIONS",
                "BG-ERR-COMPILE",
                "BG-NS-SOURCE",
                "BG-DICT-DEFINES",
            ]
            .map(|id| {
                detail
                    .iter()
                    .position(|event| event.ledger_id == id && event.phase == "Release")
                    .unwrap()
            });
            assert!(release_order.windows(2).all(|pair| pair[0] < pair[1]));
        });
    }

    #[test]
    fn background_compiler_preserves_pinned_member_order() {
        assert!(
            core::mem::offset_of!(BackgroundShaderCompiler, m_gpu)
                < core::mem::offset_of!(BackgroundShaderCompiler, m_metalFeatures)
        );
        assert!(
            core::mem::offset_of!(BackgroundShaderCompiler, m_metalFeatures)
                < core::mem::offset_of!(BackgroundShaderCompiler, m_pendingJobs)
        );
        assert!(
            core::mem::offset_of!(BackgroundShaderCompiler, m_pendingJobs)
                < core::mem::offset_of!(BackgroundShaderCompiler, m_finishedJobs)
        );
        assert!(
            core::mem::offset_of!(BackgroundShaderCompiler, m_finishedJobs)
                < core::mem::offset_of!(BackgroundShaderCompiler, m_mutex)
        );
        assert!(
            core::mem::offset_of!(BackgroundShaderCompiler, m_mutex)
                < core::mem::offset_of!(BackgroundShaderCompiler, m_workAddedCondition)
        );
        assert!(
            core::mem::offset_of!(BackgroundShaderCompiler, m_workAddedCondition)
                < core::mem::offset_of!(BackgroundShaderCompiler, m_workFinishedCondition)
        );
        assert!(
            core::mem::offset_of!(BackgroundShaderCompiler, m_workFinishedCondition)
                < core::mem::offset_of!(BackgroundShaderCompiler, m_shouldQuit)
        );
        assert!(
            core::mem::offset_of!(BackgroundShaderCompiler, m_shouldQuit)
                < core::mem::offset_of!(BackgroundShaderCompiler, m_compilerThread)
        );
    }
}
