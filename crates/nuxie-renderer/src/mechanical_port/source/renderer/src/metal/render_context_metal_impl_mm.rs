/*
 * Mechanical translation of the complete pinned Objective-C++ source
 * renderer/src/metal/render_context_metal_impl.mm.
 *
 * Every declaration and branch remains visible below in pinned source order
 * as audit provenance. Executable owners and the native renderer connection
 * precede that source text.
 *
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/metal/render_context_metal_impl.mm";
pub const PINNED_SOURCE_SHA256: &str =
    "facf8946dd5084734e21669d29676ba5ac8ed979851ea23d611a8ed1afc1b810";
pub const PINNED_SOURCE_LINE_COUNT: usize = 2030;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 84602;
pub const TRANSLATION_UNIT: &str = "metal-render-context-implementation";
pub const TRANSLATION_TARGET: &str = "crates/nuxie-renderer/src/mechanical_port/source/renderer/src/metal/render_context_metal_impl_mm.rs";

use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp as gpu;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MetallibBytes {
    pub bytes: &'static [u8],
    pub count: usize,
}

macro_rules! metallib_owner {
    ($name:ident, $file:literal) => {
        pub static $name: MetallibBytes = {
            const BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/", $file));
            MetallibBytes {
                bytes: BYTES,
                count: BYTES.len(),
            }
        };
    };
}

#[cfg(all(target_os = "ios", not(target_abi = "sim")))]
metallib_owner!(RIVE_PLS_IOS_METALLIB, "rive_pls_ios.metallib");
#[cfg(all(target_os = "ios", not(target_abi = "sim")))]
pub static rive_pls_ios_metallib: &[u8] = RIVE_PLS_IOS_METALLIB.bytes;
#[cfg(all(target_os = "ios", not(target_abi = "sim")))]
pub static rive_pls_ios_metallib_len: usize = RIVE_PLS_IOS_METALLIB.count;
#[cfg(all(target_os = "ios", target_abi = "sim"))]
metallib_owner!(
    RIVE_PLS_IOS_SIMULATOR_METALLIB,
    "rive_pls_ios_simulator.metallib"
);
#[cfg(all(target_os = "ios", target_abi = "sim"))]
pub static rive_pls_ios_simulator_metallib: &[u8] = RIVE_PLS_IOS_SIMULATOR_METALLIB.bytes;
#[cfg(all(target_os = "ios", target_abi = "sim"))]
pub static rive_pls_ios_simulator_metallib_len: usize = RIVE_PLS_IOS_SIMULATOR_METALLIB.count;
#[cfg(all(target_os = "visionos", not(target_abi = "sim")))]
metallib_owner!(RIVE_RENDERER_XROS_METALLIB, "rive_renderer_xros.metallib");
#[cfg(all(target_os = "visionos", not(target_abi = "sim")))]
pub static rive_renderer_xros_metallib: &[u8] = RIVE_RENDERER_XROS_METALLIB.bytes;
#[cfg(all(target_os = "visionos", not(target_abi = "sim")))]
pub static rive_renderer_xros_metallib_len: usize = RIVE_RENDERER_XROS_METALLIB.count;
#[cfg(all(target_os = "visionos", target_abi = "sim"))]
metallib_owner!(
    RIVE_RENDERER_XROS_SIMULATOR_METALLIB,
    "rive_renderer_xros_simulator.metallib"
);
#[cfg(all(target_os = "visionos", target_abi = "sim"))]
pub static rive_renderer_xros_simulator_metallib: &[u8] =
    RIVE_RENDERER_XROS_SIMULATOR_METALLIB.bytes;
#[cfg(all(target_os = "visionos", target_abi = "sim"))]
pub static rive_renderer_xros_simulator_metallib_len: usize =
    RIVE_RENDERER_XROS_SIMULATOR_METALLIB.count;
#[cfg(all(target_os = "tvos", not(target_abi = "sim")))]
metallib_owner!(
    RIVE_RENDERER_APPLETVOS_METALLIB,
    "rive_renderer_appletvos.metallib"
);
#[cfg(all(target_os = "tvos", not(target_abi = "sim")))]
pub static rive_renderer_appletvos_metallib: &[u8] = RIVE_RENDERER_APPLETVOS_METALLIB.bytes;
#[cfg(all(target_os = "tvos", not(target_abi = "sim")))]
pub static rive_renderer_appletvos_metallib_len: usize = RIVE_RENDERER_APPLETVOS_METALLIB.count;
#[cfg(all(target_os = "tvos", target_abi = "sim"))]
metallib_owner!(
    RIVE_RENDERER_APPLETVSIMULATOR_METALLIB,
    "rive_renderer_appletvsimulator.metallib"
);
#[cfg(all(target_os = "tvos", target_abi = "sim"))]
pub static rive_renderer_appletvsimulator_metallib: &[u8] =
    RIVE_RENDERER_APPLETVSIMULATOR_METALLIB.bytes;
#[cfg(all(target_os = "tvos", target_abi = "sim"))]
pub static rive_renderer_appletvsimulator_metallib_len: usize =
    RIVE_RENDERER_APPLETVSIMULATOR_METALLIB.count;
#[cfg(target_os = "macos")]
metallib_owner!(RIVE_PLS_MACOSX_METALLIB, "rive_pls_macosx.metallib");
#[cfg(target_os = "macos")]
pub static rive_pls_macosx_metallib: &[u8] = RIVE_PLS_MACOSX_METALLIB.bytes;
#[cfg(target_os = "macos")]
pub static rive_pls_macosx_metallib_len: usize = RIVE_PLS_MACOSX_METALLIB.count;

pub use crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::{
    ImageFilter, ImageWrap,
};

#[repr(u64)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SamplerAddressMode {
    clampToEdge = 0,
    repeat = 2,
    mirrorRepeat = 3,
}

#[repr(u64)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SamplerMinMagFilter {
    nearest = 0,
    linear = 1,
}

#[repr(u64)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SamplerMipFilter {
    notMipmapped = 0,
    nearest = 1,
    linear = 2,
}

pub const fn address_mode_for_image_wrap(wrap: ImageWrap) -> SamplerAddressMode {
    match wrap {
        ImageWrap::clamp => SamplerAddressMode::clampToEdge,
        ImageWrap::repeat => SamplerAddressMode::repeat,
        ImageWrap::mirror => SamplerAddressMode::mirrorRepeat,
        _ => panic!("invalid ImageWrap"),
    }
}

pub const fn min_mag_filter_for_image_filter(filter: ImageFilter) -> SamplerMinMagFilter {
    match filter {
        ImageFilter::nearest => SamplerMinMagFilter::nearest,
        ImageFilter::bilinear => SamplerMinMagFilter::linear,
        _ => panic!("invalid ImageFilter"),
    }
}

pub const fn mip_filter_for_image_filter(filter: ImageFilter) -> SamplerMipFilter {
    match filter {
        ImageFilter::nearest => SamplerMipFilter::nearest,
        ImageFilter::bilinear => SamplerMipFilter::nearest,
        _ => panic!("invalid ImageFilter"),
    }
}

pub use crate::mechanical_port::source::include::rive::gpu_texture_format_hpp::GPUTextureFormat as TextureFormat;
pub use crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::AtomicBarrierType;

#[inline(never)]
fn rive_unreachable() -> ! {
    if cfg!(debug_assertions) {
        panic!("RIVE_UNREACHABLE")
    } else {
        unsafe { core::hint::unreachable_unchecked() }
    }
}

pub fn precompiled_function_name(
    draw_type: crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::DrawType,
    shader_features: u32,
    clockwise_fill: bool,
    function_base_name: &str,
) -> Option<String> {
    const SHADER_FEATURE_COUNT: usize = 8;
    let mut namespace_id = [b'0'; SHADER_FEATURE_COUNT + 2];
    for (index, value) in namespace_id[..SHADER_FEATURE_COUNT].iter_mut().enumerate() {
        if shader_features & (1 << index) != 0 {
            *value = b'1';
        }
    }
    match draw_type {
        gpu::DrawType::interiorTriangulation => {
            namespace_id[SHADER_FEATURE_COUNT] = b'1';
        }
        gpu::DrawType::featherAtlasBlit => {
            namespace_id[SHADER_FEATURE_COUNT] = b'1';
            namespace_id[SHADER_FEATURE_COUNT + 1] = b'1';
        }
        _ => {}
    }
    let namespace_prefix = match draw_type {
        gpu::DrawType::midpointFanPatches
        | gpu::DrawType::midpointFanCenterAAPatches
        | gpu::DrawType::outerCurvePatches
        | gpu::DrawType::interiorTriangulation
        | gpu::DrawType::featherAtlasBlit => {
            if clockwise_fill {
                'c'
            } else {
                'p'
            }
        }
        gpu::DrawType::imageRect => return None,
        gpu::DrawType::imageMesh => 'm',
        _ => return None,
    };
    // The source constructs this identifier exclusively from ASCII digits;
    // keep the release continuation and only diagnose violations.
    debug_assert!(namespace_id.iter().all(u8::is_ascii_digit));
    let namespace_id = unsafe { core::str::from_utf8_unchecked(&namespace_id) };
    Some(format!(
        "{namespace_prefix}{namespace_id}::{function_base_name}"
    ))
}

/// Source-order execution engine for the general `FlushDescriptor` draw list.
/// Production adapters implement `MetalExecution`; `RecordingMetal` remains
/// available as the audit adapter over the same branch-complete engine.
pub mod source_execution {
    use super::{precompiled_function_name, AtomicBarrierType, ImageFilter, SamplerAddressMode, SamplerMinMagFilter, SamplerMipFilter};
    use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp as gpu;
    use crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::ImageSampler;
    use crate::mechanical_port::source::include::rive::refcnt_hpp::RefCntTarget;
    use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_helper_impl_hpp::RenderContextHelperImpl;
    use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_impl_hpp::RenderContextImpl;
    use crate::mechanical_port::source::renderer::include::rive::renderer::buffer_ring_hpp::{BufferRing, BufferRingContract};
    use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_buffer_hpp::RiveRenderBuffer;
    use crate::mechanical_port::source::include::rive::renderer_hpp::{RenderBufferContract, RenderBufferFlags, RenderBufferType};
    use crate::mechanical_port::source::include::utils::lite_rtti_hpp::{
        lite_rtti_cast, CONST_ID, LiteRttiCastFrom, LiteRttiTypeId,
    };
    use crate::mechanical_port::source::renderer::include::rive::renderer::texture_hpp::Texture;
    use crate::mechanical_port::source::renderer::include::rive::renderer::render_target_hpp::RenderTarget;
    use crate::mechanical_port::source::renderer::src::metal::background_shader_compiler_h::BackgroundShaderCompilerOwner;
    use crate::mechanical_port::source::renderer::src::metal::background_shader_compiler_mm::{
        BackgroundCompileJob, runtime_generated_shader_exports as shader_exports,
    };
    #[cfg(target_vendor = "apple")]
    use crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::MTLLibrary;
    use core::cell::Cell;
    use core::mem::ManuallyDrop;
    use core::ops::{Deref, DerefMut};
    use std::collections::{HashMap, VecDeque};
    use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
    use std::sync::{Arc, Condvar, Mutex};
    #[cfg(test)]
    use core::sync::atomic::{AtomicUsize, Ordering};

    #[cfg(test)]
    pub(crate) static RENDER_TARGET_METAL_DROP_COUNT: AtomicUsize = AtomicUsize::new(0);
    #[cfg(test)]
    pub(crate) static RENDER_TARGET_METAL_DROP_STAGE: AtomicUsize = AtomicUsize::new(0);
    #[cfg(test)]
    pub(crate) static TEXTURE_METAL_DROP_COUNT: AtomicUsize = AtomicUsize::new(0);
    #[cfg(test)]
    pub(crate) static RENDER_TARGET_METAL_DROP_TRACE: Mutex<Vec<&'static str>> =
        Mutex::new(Vec::new());
    #[cfg(test)]
    pub(crate) static RENDER_CONTEXT_METAL_DROP_TRACE: Mutex<Vec<&'static str>> =
        Mutex::new(Vec::new());
    #[cfg(test)]
    pub(crate) static RENDER_CONTEXT_OWNER_DROP_EVENTS: Mutex<Vec<OwnerEvent>> =
        Mutex::new(Vec::new());
    #[cfg(test)]
    pub(crate) static RENDER_CONTEXT_OWNER_DROP_RETIREMENTS: Mutex<Vec<Handle>> =
        Mutex::new(Vec::new());

    // The header translation owns the one complete RenderTargetMetal identity;
    // this implementation module only supplies its source-order call sites.
    pub use crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::RenderTargetMetal;

    #[repr(u8)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum MetalObjectKind {
        Nil,
        Unknown,
        Device,
        DispatchData,
        Buffer,
        Texture,
        TextureDescriptor,
        SamplerDescriptor,
        SamplerState,
        NSString,
        Library,
        Function,
        RenderPipelineDescriptor,
        RenderPipelineColorAttachmentDescriptorArray,
        RenderPipelineColorAttachmentDescriptor,
        RenderPipelineState,
        RenderPassDescriptor,
        RenderPassColorAttachmentDescriptorArray,
        RenderPassColorAttachmentDescriptor,
        CommandQueue,
        CommandBuffer,
        RenderCommandEncoder,
        BlitCommandEncoder,
        OreContext,
    }

    /// Copyable reference into a production executor's retained Objective-C
    /// object table. `kind` prevents a selector from reinterpreting an object
    /// from a different Metal protocol, and `generation` prevents a retired
    /// slot from being addressed through a stale handle after slot reuse. The
    /// table initially owns a selector-created +1. `take_owned` transfers that
    /// +1 into the canonical source field and leaves this token as a borrowed
    /// alias until that owner invalidates it.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Handle {
        pub slot: u32,
        pub kind: MetalObjectKind,
        pub generation: u64,
        pub registry: u64,
    }

    /// Shared liveness for one nonowning registry alias. The canonical source
    /// owner invalidates this before releasing its native +1. Registry lookup
    /// therefore fails closed without retaining the native object itself.
    #[derive(Clone)]
    pub struct MetalAliasValidity(Arc<AtomicBool>);

    impl MetalAliasValidity {
        pub fn live() -> Self {
            Self(Arc::new(AtomicBool::new(true)))
        }

        pub fn is_live(&self) -> bool {
            self.0.load(AtomicOrdering::Acquire)
        }

        pub fn invalidate(&self) {
            self.0.store(false, AtomicOrdering::Release);
        }

        pub fn ptr_eq(&self, other: &Self) -> bool {
            Arc::ptr_eq(&self.0, &other.0)
        }
    }

    /// A source-owned +1 retain paired with its generation-checked, nonowning
    /// execution alias. The executor transfers its creation retain into this
    /// value; it does not keep a second retain. Drop invalidates the alias
    /// synchronously before the native +1 is released.
    pub struct OwnedMetalHandle {
        handle: Handle,
        kind: MetalObjectKind,
        alias: Option<MetalAliasValidity>,
        #[cfg(test)]
        recording_drop_probe: Option<Arc<Mutex<Vec<Handle>>>>,
        #[cfg(test)]
        recording_clone_events: Option<Arc<Mutex<Vec<RecordingCloneOwnerEvent>>>>,
        _recording_thread: core::marker::PhantomData<std::rc::Rc<()>>,
        #[cfg(target_vendor = "apple")]
        object: Option<objc2::rc::Retained<objc2::runtime::AnyObject>>,
    }

    impl core::fmt::Debug for OwnedMetalHandle {
        fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            formatter
                .debug_struct("OwnedMetalHandle")
                .field("handle", &self.handle)
                .finish()
        }
    }

    impl OwnedMetalHandle {
        pub fn token(handle: Handle) -> Self {
            Self {
                handle,
                kind: handle.kind,
                alias: None,
                #[cfg(test)]
                recording_drop_probe: None,
                #[cfg(test)]
                recording_clone_events: None,
                _recording_thread: core::marker::PhantomData,
                #[cfg(target_vendor = "apple")]
                object: None,
            }
        }

        #[cfg(test)]
        fn recording_clone(
            handle: Handle,
            source: Handle,
            drop_probe: Arc<Mutex<Vec<Handle>>>,
            events: Arc<Mutex<Vec<RecordingCloneOwnerEvent>>>,
        ) -> Self {
            events
                .lock()
                .unwrap()
                .push(RecordingCloneOwnerEvent::Clone {
                    alias: handle,
                    source,
                });
            Self {
                handle,
                kind: handle.kind,
                alias: None,
                recording_drop_probe: Some(drop_probe),
                recording_clone_events: Some(events),
                _recording_thread: core::marker::PhantomData,
                #[cfg(target_vendor = "apple")]
                object: None,
            }
        }

        #[cfg(target_vendor = "apple")]
        /// # Safety
        /// `object` must be the native protocol named by `kind`, and
        /// `alias` must be the generation-valid alias for that exact object.
        pub unsafe fn native(
            handle: Handle,
            object: objc2::rc::Retained<objc2::runtime::AnyObject>,
            alias: MetalAliasValidity,
        ) -> Self {
            Self {
                handle,
                kind: handle.kind,
                alias: Some(alias),
                #[cfg(test)]
                recording_drop_probe: None,
                #[cfg(test)]
                recording_clone_events: None,
                _recording_thread: core::marker::PhantomData,
                object: Some(object),
            }
        }

        #[cfg(target_vendor = "apple")]
        /// # Safety
        /// `object` must implement the protocol represented by `kind` and
        /// carry the selector-created +1 transferred to this owner.
        pub(crate) unsafe fn detached_native(
            kind: MetalObjectKind,
            object: objc2::rc::Retained<objc2::runtime::AnyObject>,
        ) -> Self {
            Self {
                handle: Handle::NIL,
                kind,
                alias: None,
                #[cfg(test)]
                recording_drop_probe: None,
                #[cfg(test)]
                recording_clone_events: None,
                _recording_thread: core::marker::PhantomData,
                object: Some(object),
            }
        }

        pub fn handle(&self) -> Handle {
            match &self.alias {
                Some(alias) if !alias.is_live() => Handle::NIL,
                Some(_) | None => self.handle,
            }
        }

        pub fn kind(&self) -> MetalObjectKind {
            self.kind
        }

        #[cfg(target_vendor = "apple")]
        pub fn native_object(&self) -> Option<&objc2::runtime::AnyObject> {
            self.object.as_deref()
        }

        #[cfg(target_vendor = "apple")]
        pub fn new_buffer_with_length(
            &self,
            length: usize,
            options: objc2_metal::MTLResourceOptions,
        ) -> Option<OwnedMetalHandle> {
            use objc2_metal::MTLDevice;

            if self.kind != MetalObjectKind::Device {
                return None;
            }
            let device = self.native_object()?;
            let device = unsafe {
                &*(device as *const objc2::runtime::AnyObject
                    as *const objc2::runtime::ProtocolObject<dyn objc2_metal::MTLDevice>)
            };
            let buffer = device.newBufferWithLength_options(length, options)?;
            Some(unsafe {
                Self::detached_native(
                    MetalObjectKind::Buffer,
                    objc2::rc::Retained::cast_unchecked::<objc2::runtime::AnyObject>(buffer),
                )
            })
        }

        #[cfg(target_vendor = "apple")]
        pub fn buffer_contents(&self) -> Option<*mut u8> {
            use objc2_metal::MTLBuffer;

            if self.kind != MetalObjectKind::Buffer {
                return None;
            }
            let buffer = self.native_object()?;
            let buffer = unsafe {
                &*(buffer as *const objc2::runtime::AnyObject
                    as *const objc2::runtime::ProtocolObject<dyn objc2_metal::MTLBuffer>)
            };
            Some(buffer.contents().as_ptr().cast())
        }

        #[cfg(not(target_vendor = "apple"))]
        pub fn buffer_contents(&self) -> Option<*mut u8> {
            None
        }

        #[cfg(target_vendor = "apple")]
        pub(crate) fn alias_validity(&self) -> Option<&MetalAliasValidity> {
            self.alias.as_ref().filter(|alias| alias.is_live())
        }

        #[cfg(target_vendor = "apple")]
        pub(crate) fn install_alias(&mut self, handle: Handle, alias: MetalAliasValidity) -> bool {
            if handle == Handle::NIL || handle.kind != self.kind || self.object.is_none() {
                return false;
            }
            if self.alias_validity().is_some() {
                return false;
            }
            self.handle = handle;
            self.alias = Some(alias);
            true
        }

        #[cfg(target_vendor = "apple")]
        pub(crate) fn clear_alias_for_republication(&mut self) {
            if let Some(alias) = self.alias.take() {
                alias.invalidate();
            }
            self.handle = Handle::NIL;
        }
    }

    impl Drop for OwnedMetalHandle {
        fn drop(&mut self) {
            if let Some(alias) = self.alias.take() {
                alias.invalidate();
            }
            #[cfg(test)]
            if let Some(probe) = self.recording_drop_probe.take() {
                probe.lock().unwrap().push(self.handle);
            }
            #[cfg(test)]
            if let Some(events) = self.recording_clone_events.take() {
                events
                    .lock()
                    .unwrap()
                    .push(RecordingCloneOwnerEvent::Drop { alias: self.handle });
            }
            // Rust drops `object` after this method returns, so alias
            // invalidation always precedes native release.
        }
    }

    impl Handle {
        pub const NIL: Self = Self::with_generation(0, MetalObjectKind::Nil, 0);

        pub const fn new(slot: u32, kind: MetalObjectKind) -> Self {
            Self::with_generation(slot, kind, 1)
        }

        pub const fn with_generation(slot: u32, kind: MetalObjectKind, generation: u64) -> Self {
            Self::with_registry(slot, kind, generation, 0)
        }

        pub const fn with_registry(
            slot: u32,
            kind: MetalObjectKind,
            generation: u64,
            registry: u64,
        ) -> Self {
            Self {
                slot,
                kind,
                generation,
                registry,
            }
        }
    }

    fn result_kind(receiver: &'static str, selector: &'static str) -> MetalObjectKind {
        match (receiver, selector) {
            ("dispatch", "dispatch_data_create") => MetalObjectKind::DispatchData,
            ("gpu", "newBufferWithLength:options:")
            | ("gpu", "newBufferWithBytes:length:options:") => MetalObjectKind::Buffer,
            ("gpu", "newTextureWithDescriptor:") => MetalObjectKind::Texture,
            ("MTLTextureDescriptor", "alloc/init") => MetalObjectKind::TextureDescriptor,
            ("MTLSamplerDescriptor", "new") => MetalObjectKind::SamplerDescriptor,
            ("gpu", "newSamplerStateWithDescriptor:") => MetalObjectKind::SamplerState,
            ("gpu", "newLibraryWithData:error:") => MetalObjectKind::Library,
            ("library", "newFunctionWithName:") => MetalObjectKind::Function,
            ("MTLRenderPipelineDescriptor", "alloc/init") => {
                MetalObjectKind::RenderPipelineDescriptor
            }
            ("descriptor", "colorAttachments") => {
                MetalObjectKind::RenderPipelineColorAttachmentDescriptorArray
            }
            ("descriptor", "colorAttachmentAtIndex:") => {
                MetalObjectKind::RenderPipelineColorAttachmentDescriptor
            }
            ("gpu", "newRenderPipelineStateWithDescriptor:error:") => {
                MetalObjectKind::RenderPipelineState
            }
            ("MTLRenderPassDescriptor", "renderPassDescriptor") => {
                MetalObjectKind::RenderPassDescriptor
            }
            ("pass", "colorAttachments") => {
                MetalObjectKind::RenderPassColorAttachmentDescriptorArray
            }
            ("pass", "colorAttachmentAtIndex:") => {
                MetalObjectKind::RenderPassColorAttachmentDescriptor
            }
            ("commandQueue", "commandBuffer (__bridge_retained)") => MetalObjectKind::CommandBuffer,
            ("commandBuffer", "renderCommandEncoderWithDescriptor:") => {
                MetalObjectKind::RenderCommandEncoder
            }
            ("commandBuffer", "blitCommandEncoder") => MetalObjectKind::BlitCommandEncoder,
            _ => MetalObjectKind::Unknown,
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    pub enum Value {
        U64(u64),
        I64(i64),
        Bool(bool),
        F64(f64),
        Handle(Handle),
        Text(String),
        StaticText(&'static str),
        Bytes(Arc<[u8]>),
        Origin(Origin),
        Size(Size),
        Viewport(Viewport),
        Scissor(Scissor),
        ClearColor(ClearColor),
        Nil,
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct Origin {
        pub x: usize,
        pub y: usize,
        pub z: usize,
    }
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct Size {
        pub width: usize,
        pub height: usize,
        pub depth: usize,
    }
    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct Viewport {
        pub origin_x: f64,
        pub origin_y: f64,
        pub width: f64,
        pub height: f64,
        pub znear: f64,
        pub zfar: f64,
    }
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct Scissor {
        pub x: usize,
        pub y: usize,
        pub width: usize,
        pub height: usize,
    }
    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct ClearColor {
        pub red: f64,
        pub green: f64,
        pub blue: f64,
        pub alpha: f64,
    }

    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct Rect {
        pub left: u32,
        pub top: u32,
        pub right: u32,
        pub bottom: u32,
    }
    impl Rect {
        pub fn width(self) -> u32 {
            self.right - self.left
        }
        pub fn height(self) -> u32 {
            self.bottom - self.top
        }
        pub fn intersect_or_empty(self, other: Self) -> Self {
            let left = self.left.max(other.left);
            let top = self.top.max(other.top);
            let right = self.right.min(other.right);
            let bottom = self.bottom.min(other.bottom);
            if right < left || bottom < top {
                Self {
                    left,
                    top,
                    right: left,
                    bottom: top,
                }
            } else {
                Self {
                    left,
                    top,
                    right,
                    bottom,
                }
            }
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct SelectorCall {
        pub receiver: &'static str,
        pub selector: &'static str,
        pub args: Vec<Value>,
    }

    #[derive(Default)]
    pub struct ObjectCreation {
        pub object: Option<Handle>,
        pub error: Option<String>,
        pub error_present: bool,
        /// Recording's scripted NSError is a real source-result owner: the
        /// constructor emits its create/log/release events at the same
        /// call-with-error boundary as the native opaque NSError.
        pub error_owner_handle: Option<Handle>,
        /// Native NSError ownership remains with the source lexical result
        /// until the caller has logged/inspected it.  The recording adapter
        /// has no native error object and leaves this absent.
        #[cfg(target_vendor = "apple")]
        pub error_owner: Option<objc2::rc::Retained<objc2_foundation::NSError>>,
    }

    impl ObjectCreation {
        pub fn has_error(&self) -> bool {
            self.error_present || self.error.is_some()
        }

        pub fn error_description(&self) -> Option<String> {
            if self.error.is_some() {
                return self.error.clone();
            }
            #[cfg(target_vendor = "apple")]
            return self
                .error_owner
                .as_ref()
                .map(|error| error.localizedDescription().to_string());
            #[cfg(not(target_vendor = "apple"))]
            {
                None
            }
        }
    }

    #[inline(never)]
    fn rive_unreachable() -> ! {
        #[cfg(debug_assertions)]
        panic!("RIVE_UNREACHABLE");
        #[cfg(not(debug_assertions))]
        unsafe {
            core::hint::unreachable_unchecked()
        }
    }

    pub trait HostExecution {
        fn log(&mut self, message: String);
        fn generate_patch_buffer_data(&mut self, vertex_buffer: Handle, index_buffer: Handle);
        fn make_ore_context(&mut self, device: Handle, queue: Option<Handle>) -> Option<Handle>;
    }

    pub trait MetalExecution: HostExecution {
        /// Source-owner evidence hook. Production adapters may route this to
        /// the native lifetime probe; the recording adapter retains the full
        /// ordered event stream for the exhaustive ownership gate.
        fn owner_event(
            &mut self,
            _ledger_id: &'static str,
            _phase: OwnerEventPhase,
            _handle: Handle,
        ) {
        }
        fn device_handle(&self) -> Handle;
        /// Canonical source capability queries. The device handle is passed
        /// explicitly so construction cannot accidentally consult a mutable
        /// adapter device after the source owner has retained its constructor
        /// device.
        fn device_supports_family(&mut self, _device: Handle, _family: u64) -> bool {
            false
        }
        fn device_raster_order_groups_supported(&mut self, _device: Handle) -> bool {
            false
        }
        fn device_is_apple_silicon(&mut self, _device: Handle) -> bool {
            false
        }
        /// The simulator branch follows NXGetLocalArchInfo(), i.e. the host
        /// architecture, rather than Rust's target architecture.
        fn host_architecture_is_arm64(&mut self) -> bool {
            false
        }
        /// Runtime availability for the source's macOS 10.14 memory-barrier
        /// branch.  This is deliberately a selector/availability seam rather
        /// than a target-os constant: the pinned code tests @available at the
        /// call site and reaches its unreachable fallback otherwise.
        fn memory_barrier_available(&mut self) -> bool {
            cfg!(target_os = "macos")
        }
        /// Transfers the registry's creation +1 into the canonical source
        /// owner and leaves a generation-checked nonowning alias behind.
        fn take_owned(&mut self, handle: Handle, kind: MetalObjectKind)
        -> Option<OwnedMetalHandle>;
        /// Creates the separate +1 required by a source strong assignment,
        /// publishes it under its own generation, and transfers that owner.
        fn clone_owned(
            &mut self,
            handle: Handle,
            kind: MetalObjectKind,
        ) -> Option<OwnedMetalHandle>;
        /// Creates the source-scoped native NSString used by
        /// `newFunctionWithName:`. Recording adapters may fall back to the
        /// textual Value path; native adapters retain this object only for
        /// the authored DrawPipeline constructor scope.
        fn make_function_name(&mut self, _name: &str) -> Option<OwnedMetalHandle> {
            None
        }
        /// Produces the source `GetPrecompiledFunctionName` NSString before
        /// DrawPipeline construction. Native execution overrides this with
        /// the exact `stringWithFormat:@"%c%s::%s"` boundary; recording
        /// execution may use the textual fallback.
        fn make_precompiled_function_name(
            &mut self,
            prefix: u8,
            namespace_id: &str,
            function_base_name: &str,
        ) -> Option<Handle> {
            let _ = (prefix, namespace_id, function_base_name);
            None
        }
        /// Publishes a detached direct native owner as a nonowning selector
        /// alias. Re-publication is allowed only after the prior executor has
        /// invalidated its alias.
        fn publish_owned(&mut self, owner: &mut OwnedMetalHandle) -> Option<Handle>;
        /// Adopt a +1 returned by the canonical background compiler and
        /// publish its nonowning selector alias in this execution domain.
        /// Implementations that cannot bridge a native library fail closed;
        /// they must not route this source-owned result through the legacy
        /// host compiler.
        #[cfg(target_vendor = "apple")]
        unsafe fn adopt_compiled_library(&mut self, library: *mut MTLLibrary) -> Option<Handle> {
            // The callback consumes the compiler's +1 even when this
            // execution cannot install a registry alias. This fail-closed
            // default therefore cannot leak a completed library.
            unsafe {
                let _ = objc2::rc::Retained::<objc2::runtime::AnyObject>::from_raw(library.cast());
            }
            None
        }
        fn buffer_contents(&mut self, buffer: Handle) -> *mut u8;
        /// Releases the executor's retained owner for this exact typed handle.
        /// NIL, stale-generation, wrong-kind, and already-retired handles are
        /// required to be no-ops.
        fn retire_handle(&mut self, handle: Handle);
        /// Source `compatibleWith` metadata query for a candidate target
        /// texture. Product adapters may apply additional validation, but the
        /// source setter derives admission from the actual native object.
        fn texture_compatible(
            &mut self,
            texture: Handle,
            width: u32,
            height: u32,
            format: PixelFormat,
        ) -> bool;
        /// Attaches source pipeline identity to one successfully-created native
        /// pipeline state. Implementations must generation-check `pipeline` and
        /// ignore NIL, stale, or wrong-kind handles.
        fn tag_pipeline(&mut self, pipeline: Handle, semantic: PipelineSemantic);
        /// Replaces a bound ubershader's compile-time feature superset with the
        /// exact source-requested semantics for the next draw submission.
        fn record_draw_semantic(&mut self, encoder: Handle, semantic: PipelineSemantic);
        /// Records a source-requested render-pass split only after the
        /// replacement native encoder was successfully created.
        fn record_render_pass_break(&mut self);
        /// Records the source's executed raster-order-group barrier branch.
        /// Metal emits no selector for this policy, so the executor must first
        /// validate that `encoder` is a live render-command encoder.
        fn record_raster_order_group_barrier(&mut self, encoder: Handle);
        fn call(
            &mut self,
            receiver: &'static str,
            selector: &'static str,
            args: Vec<Value>,
        ) -> Option<Handle>;
        fn call_with_error(
            &mut self,
            receiver: &'static str,
            selector: &'static str,
            args: Vec<Value>,
        ) -> ObjectCreation;
        fn add_completed_handler(
            &mut self,
            command_buffer: Handle,
            handler: Box<dyn FnOnce(Result<(), String>) + Send + 'static>,
        ) -> bool;
        fn completion_block_identity(&mut self, command_buffer: Handle) -> Handle {
            command_buffer
        }
        fn end_completion_block_identity(&mut self, _block: Handle) {}
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum OwnerEventPhase {
        Create,
        CreateBridge,
        CreateClone,
        CreateStrong,
        CloneToTarget,
        CloneToImage,
        Borrow,
        BorrowAlias,
        BorrowStack,
        Transfer,
        CopyTransfer,
        LastUse,
        Invoke,
        Release,
        ReleaseStrong,
        ReleaseLocal,
        ReleaseCopy,
        AliasEnd,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct OwnerEvent {
        pub ledger_id: &'static str,
        pub phase: OwnerEventPhase,
        /// Registry alias used at this exact source boundary.
        pub handle: Handle,
        /// Alias from which a strong clone was made, or `handle` for a
        /// selector-created/transferred owner.
        pub source_handle: Handle,
        /// Recording identity for the underlying native object. Distinct
        /// registry aliases created by `clone_owned` share this value.
        pub native_identity: Handle,
        /// Parent descriptor for a +0 attachment/collection child alias.
        pub parent_handle: Option<Handle>,
        /// The immediately preceding selector occurrence at this exact
        /// source boundary.  This binds failure ordinals and last-use order
        /// to the real scenario trace instead of a selector-name inventory.
        pub selector_ordinal: Option<(&'static str, usize)>,
    }
    #[cfg(test)]
    fn record_owner_drop(ledger_id: &'static str, handle: Handle) {
        RENDER_CONTEXT_OWNER_DROP_EVENTS
            .lock()
            .unwrap()
            .push(OwnerEvent {
                ledger_id,
                phase: OwnerEventPhase::Release,
                handle,
                source_handle: handle,
                native_identity: handle,
                parent_handle: None,
                selector_ordinal: None,
            });
        RENDER_CONTEXT_OWNER_DROP_RETIREMENTS
            .lock()
            .unwrap()
            .push(handle);
    }

    #[cfg(test)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum RecordingCloneOwnerEvent {
        Clone { alias: Handle, source: Handle },
        Drop { alias: Handle },
    }

    #[derive(Default)]
    pub struct RecordingMetal {
        pub calls: Vec<SelectorCall>,
        pub owner_events: Vec<OwnerEvent>,
        /// Source bridge events which do not travel through the generic
        /// selector recorder (notably the precompiled NSString producer).
        /// Keeping these alongside selector calls lets the ownership gate
        /// distinguish a real bridge boundary from a MetalObjectKind table.
        pub bridge_events: Vec<&'static str>,
        /// Source lexical-owner evidence: each explicit release is recorded
        /// at the caller scope rather than hidden in a selector adapter.
        pub retirements: Vec<Handle>,
        /// Selector-call count observed at each explicit retirement. This
        /// puts native release and selector use on one monotonic recording
        /// axis, so a release moved before its final selector cannot pass an
        /// ownership-order regression by appearing in a separate vector.
        pub retirement_call_counts: Vec<(Handle, usize)>,
        /// Owner-event count observed at each actual retirement. This binds
        /// selector-free handoffs (notably helper return into a caller strong
        /// local) to the native release boundary as well as the call axis.
        pub retirement_event_counts: Vec<(Handle, usize)>,
        /// Actual Drop boundaries for source strong locals created through
        /// `clone_owned`. These are deliberately distinct from
        /// `retire_handle`: retiring the original executor alias instead of
        /// dropping the clone is the ownership bug this probe guards against.
        cloned_owner_drops: Arc<Mutex<Vec<Handle>>>,
        #[cfg(test)]
        recording_clone_events: Arc<Mutex<Vec<RecordingCloneOwnerEvent>>>,
        /// `(clone alias, source alias)` edges. Recording handles have no
        /// Objective-C pointer, so their root source alias is the stable
        /// native-identity witness used by the ownership gate.
        clone_sources: Vec<(Handle, Handle)>,
        borrowed_child_sources: Vec<(Handle, Handle)>,
        next: u32,
        pub fail: VecDeque<&'static str>,
        pub fail_exact: Option<(&'static str, usize)>,
        selector_occurrences: HashMap<&'static str, usize>,
        pub fail_clone_exact: Option<(MetalObjectKind, usize)>,
        clone_occurrences: Vec<(MetalObjectKind, usize)>,
        pub errors: VecDeque<(&'static str, String)>,
        pub completed_handler_install_fail: bool,
        pending_completion_block: Option<Handle>,
        completed_handlers: VecDeque<(
            Handle,
            Handle,
            Box<dyn FnOnce(Result<(), String>) + Send + 'static>,
        )>,
    }
    impl MetalExecution for RecordingMetal {
        fn owner_event(&mut self, ledger_id: &'static str, phase: OwnerEventPhase, handle: Handle) {
            if handle == Handle::NIL {
                return;
            }
            let source_handle = self
                .clone_sources
                .iter()
                .rev()
                .find_map(|(clone, source)| (*clone == handle).then_some(*source))
                .unwrap_or(handle);
            let mut native_identity = source_handle;
            while let Some(parent) = self
                .clone_sources
                .iter()
                .rev()
                .find_map(|(clone, source)| (*clone == native_identity).then_some(*source))
            {
                native_identity = parent;
            }
            self.owner_events.push(OwnerEvent {
                ledger_id,
                phase,
                handle,
                source_handle,
                native_identity,
                parent_handle: self
                    .borrowed_child_sources
                    .iter()
                    .rev()
                    .find_map(|(child, parent)| {
                        (*child == handle || *child == source_handle).then_some(*parent)
                    }),
                selector_ordinal: self.calls.last().map(|call| {
                    (
                        call.selector,
                        self.selector_occurrences
                            .get(call.selector)
                            .copied()
                            .unwrap_or(0),
                    )
                }),
            });
        }
        fn device_handle(&self) -> Handle {
            Handle::new(1, MetalObjectKind::Device)
        }

        fn take_owned(
            &mut self,
            handle: Handle,
            _kind: MetalObjectKind,
        ) -> Option<OwnedMetalHandle> {
            (handle != Handle::NIL).then(|| OwnedMetalHandle::token(handle))
        }

        fn clone_owned(
            &mut self,
            handle: Handle,
            kind: MetalObjectKind,
        ) -> Option<OwnedMetalHandle> {
            if handle == Handle::NIL {
                return None;
            }
            let occurrence = if let Some((_, occurrence)) = self
                .clone_occurrences
                .iter_mut()
                .find(|(candidate, _)| *candidate == kind)
            {
                *occurrence += 1;
                *occurrence
            } else {
                self.clone_occurrences.push((kind, 1));
                1
            };
            if self.fail_clone_exact == Some((kind, occurrence)) {
                self.fail_clone_exact = None;
                return None;
            }
            self.next = self.next.max(handle.slot);
            self.next += 1;
            let clone = Handle::new(self.next, kind);
            self.clone_sources.push((clone, handle));
            #[cfg(test)]
            {
                Some(OwnedMetalHandle::recording_clone(
                    clone,
                    handle,
                    Arc::clone(&self.cloned_owner_drops),
                    Arc::clone(&self.recording_clone_events),
                ))
            }
            #[cfg(not(test))]
            {
                Some(OwnedMetalHandle::token(clone))
            }
        }

        fn make_function_name(&mut self, _name: &str) -> Option<OwnedMetalHandle> {
            None
        }

        fn make_precompiled_function_name(
            &mut self,
            _prefix: u8,
            _namespace_id: &str,
            _function_base_name: &str,
        ) -> Option<Handle> {
            self.bridge_events.push("stringWithFormat:");
            self.next += 1;
            Some(Handle::new(self.next, MetalObjectKind::NSString))
        }

        fn publish_owned(&mut self, owner: &mut OwnedMetalHandle) -> Option<Handle> {
            (owner.handle() != Handle::NIL).then(|| owner.handle())
        }

        fn buffer_contents(&mut self, _buffer: Handle) -> *mut u8 {
            core::ptr::null_mut()
        }

        fn retire_handle(&mut self, handle: Handle) {
            self.retirements.push(handle);
            self.retirement_call_counts.push((handle, self.calls.len()));
            self.retirement_event_counts
                .push((handle, self.owner_events.len()));
        }

        fn texture_compatible(
            &mut self,
            _texture: Handle,
            _width: u32,
            _height: u32,
            _format: PixelFormat,
        ) -> bool {
            true
        }

        fn tag_pipeline(&mut self, _pipeline: Handle, _semantic: PipelineSemantic) {}

        fn record_draw_semantic(&mut self, _encoder: Handle, _semantic: PipelineSemantic) {}

        fn record_render_pass_break(&mut self) {}

        fn record_raster_order_group_barrier(&mut self, _encoder: Handle) {}

        fn call(
            &mut self,
            receiver: &'static str,
            selector: &'static str,
            args: Vec<Value>,
        ) -> Option<Handle> {
            let borrowed_parent = matches!(selector, "colorAttachments" | "colorAttachmentAtIndex:")
            .then(|| {
                args.iter().find_map(|value| match value {
                    Value::Handle(handle) => Some(*handle),
                    _ => None,
                })
            })
            .flatten();
            self.calls.push(SelectorCall {
                receiver,
                selector,
                args,
            });
            let occurrence = self.selector_occurrences.entry(selector).or_default();
            *occurrence += 1;
            if self.fail_exact == Some((selector, *occurrence)) {
                self.fail_exact = None;
                return None;
            }
            if self.fail.front().copied() == Some(selector) {
                self.fail.pop_front();
                return None;
            }
            self.next += 1;
            let result = Handle::new(self.next, result_kind(receiver, selector));
            if let Some(parent) = borrowed_parent {
                self.borrowed_child_sources.push((result, parent));
            }
            Some(result)
        }

        fn call_with_error(
            &mut self,
            receiver: &'static str,
            selector: &'static str,
            args: Vec<Value>,
        ) -> ObjectCreation {
            let object = self.call(receiver, selector, args);
            let error = if self
                .errors
                .front()
                .is_some_and(|(target, _)| *target == selector)
            {
                self.errors.pop_front().map(|(_, error)| error)
            } else {
                None
            };
            let error_present = error.is_some();
            ObjectCreation {
                object,
                error,
                error_present,
                error_owner_handle: error_present.then(|| {
                    self.next += 1;
                    Handle::new(self.next, MetalObjectKind::Unknown)
                }),
                #[cfg(target_vendor = "apple")]
                error_owner: None,
            }
        }

        fn add_completed_handler(
            &mut self,
            command_buffer: Handle,
            handler: Box<dyn FnOnce(Result<(), String>) + Send + 'static>,
        ) -> bool {
            self.calls.push(SelectorCall {
                receiver: "commandBuffer",
                selector: "addCompletedHandler:",
                args: vec![Value::Handle(command_buffer)],
            });
            if self.completed_handler_install_fail {
                self.pending_completion_block.take();
                return false;
            }
            let block = self.pending_completion_block.take().unwrap_or(command_buffer);
            self.completed_handlers
                .push_back((command_buffer, block, handler));
            true
        }

        fn completion_block_identity(&mut self, command_buffer: Handle) -> Handle {
            self.next += 1;
            let block = Handle::new(self.next, MetalObjectKind::Unknown);
            self.borrowed_child_sources.push((block, command_buffer));
            self.pending_completion_block = Some(block);
            block
        }

        fn end_completion_block_identity(&mut self, block: Handle) {
            self.retire_handle(block);
        }
    }

    #[cfg(test)]
    impl RecordingMetal {
        pub(crate) fn recording_clone_events(&self) -> Vec<RecordingCloneOwnerEvent> {
            self.recording_clone_events.lock().unwrap().clone()
        }

        pub(crate) fn drain_recorded_clone_drops(&mut self) {
            self.retirements.extend(
                self.cloned_owner_drops
                    .lock()
                    .unwrap()
                    .drain(..),
            );
        }

        pub(crate) fn selector_occurrence_count(&self, selector: &'static str) -> usize {
            self.selector_occurrences.get(selector).copied().unwrap_or(0)
        }
    }

    impl HostExecution for RecordingMetal {
        fn log(&mut self, message: String) {
            self.calls.push(SelectorCall {
                receiver: "host",
                selector: "log",
                args: vec![Value::Text(message)],
            });
        }

        fn generate_patch_buffer_data(&mut self, vertex_buffer: Handle, index_buffer: Handle) {
            self.calls.push(SelectorCall {
                receiver: "host",
                selector: "generatePatchBufferData",
                args: vec![Value::Handle(vertex_buffer), Value::Handle(index_buffer)],
            });
        }

        fn make_ore_context(&mut self, device: Handle, queue: Option<Handle>) -> Option<Handle> {
            let mut args = vec![Value::Handle(device)];
            args.push(queue.map(Value::Handle).unwrap_or(Value::Nil));
            self.call("host", "makeOreContext", args).map(|handle| {
                Handle::with_registry(
                    handle.slot,
                    MetalObjectKind::OreContext,
                    handle.generation,
                    handle.registry,
                )
            })
        }
    }

    impl RecordingMetal {
        /// RecordingMetal deliberately has no Objective-C backing object;
        /// native compiler ownership is therefore an Apple-product-only
        /// path, never a fabricated recording event.
        pub fn native_object_for_test(&self, _handle: Handle) -> Option<usize> {
            None
        }

        pub fn run_next_completed_handler(&mut self) {
            self.run_next_completed_handler_with(Ok(()));
        }

        pub fn run_next_completed_handler_with(&mut self, result: Result<(), String>) {
            let (_command, block, handler) = self
                .completed_handlers
                .pop_front()
                .expect("completion callback was installed");
            // This is the actual RecordingMetal completion callback boundary,
            // whose body unlocks the ring before publishing the product token.
            self.owner_event("RC-BLOCK-COMPLETE", OwnerEventPhase::Invoke, block);
            handler(result);
            self.retire_handle(block);
            self.owner_event("RC-BLOCK-COMPLETE", OwnerEventPhase::ReleaseCopy, block);
        }
    }

    fn h(value: Handle) -> Value {
        Value::Handle(value)
    }
    fn u(value: impl TryInto<u64>) -> Value {
        let Ok(value) = value.try_into() else {
            panic!("Metal selector integer argument must fit in uint64_t")
        };
        Value::U64(value)
    }
    fn b(value: bool) -> Value {
        Value::Bool(value)
    }
    fn text(value: impl Into<String>) -> Value {
        Value::Text(value.into())
    }
    fn static_text(value: &'static str) -> Value {
        Value::StaticText(value)
    }

    /// Complete census of the immortal Objective-C NSString literals passed
    /// to `newFunctionWithName:` by the pinned Metal implementation.  Dynamic
    /// precompiled names are deliberately absent: those are produced by
    /// `GetPrecompiledFunctionName` and have their own scoped +0 owner rows.
    pub(crate) const SOURCE_STATIC_FUNCTION_NAMES: [&str; 9] = [
        shader_exports::GLSL_colorRampVertexMain,
        shader_exports::GLSL_colorRampFragmentMain,
        shader_exports::GLSL_tessellateVertexMain,
        shader_exports::GLSL_tessellateFragmentMain,
        shader_exports::GLSL_atlasVertexMain,
        shader_exports::GLSL_atlasFillFragmentMain,
        shader_exports::GLSL_atlasStrokeFragmentMain,
        shader_exports::GLSL_drawVertexMain,
        shader_exports::GLSL_drawFragmentMain,
    ];

    const COLOR_RAMP_VERTEX_NAME: &str = SOURCE_STATIC_FUNCTION_NAMES[0];
    const COLOR_RAMP_FRAGMENT_NAME: &str = SOURCE_STATIC_FUNCTION_NAMES[1];
    const TESSELLATE_VERTEX_NAME: &str = SOURCE_STATIC_FUNCTION_NAMES[2];
    const TESSELLATE_FRAGMENT_NAME: &str = SOURCE_STATIC_FUNCTION_NAMES[3];
    const FEATHER_VERTEX_NAME: &str = SOURCE_STATIC_FUNCTION_NAMES[4];
    pub(crate) const ATLAS_FILL_FRAGMENT_NAME: &str = SOURCE_STATIC_FUNCTION_NAMES[5];
    pub(crate) const ATLAS_STROKE_FRAGMENT_NAME: &str = SOURCE_STATIC_FUNCTION_NAMES[6];
    pub(crate) const DRAW_VERTEX_NAME: &str = SOURCE_STATIC_FUNCTION_NAMES[7];
    pub(crate) const DRAW_FRAGMENT_NAME: &str = SOURCE_STATIC_FUNCTION_NAMES[8];
    pub(crate) enum SourceFunctionName {
        Static(&'static str),
        /// The native producer has already created this exact source
        /// NSString. DrawPipeline only borrows its selector alias.
        Dynamic(Handle),
        /// Recording/non-native fallback when no native NSString bridge is
        /// available. Production Apple execution never takes this branch.
        DynamicText(String),
    }

    fn source_function_name(name: SourceFunctionName) -> (Option<Handle>, Value) {
        match name {
            SourceFunctionName::Static(name) => (None, static_text(name)),
            SourceFunctionName::Dynamic(handle) => {
                let value = h(handle);
                (Some(handle), value)
            }
            SourceFunctionName::DynamicText(name) => (None, text(name)),
        }
    }
    fn bytes<T>(values: &[T]) -> Value {
        let pointer = values.as_ptr().cast::<u8>();
        let length = core::mem::size_of_val(values);
        let copied = unsafe { core::slice::from_raw_parts(pointer, length) };
        Value::Bytes(Arc::from(copied))
    }
    fn set<E: MetalExecution>(
        metal: &mut E,
        receiver: &'static str,
        selector: &'static str,
        args: Vec<Value>,
    ) {
        let _ = metal.call(receiver, selector, args);
    }

    const MTL_LOAD_ACTION_DONT_CARE: u64 = 0;
    const MTL_LOAD_ACTION_LOAD: u64 = 1;
    const MTL_LOAD_ACTION_CLEAR: u64 = 2;
    const MTL_STORE_ACTION_DONT_CARE: u64 = 0;
    const MTL_STORE_ACTION_STORE: u64 = 1;
    const MTL_PRIMITIVE_TYPE_TRIANGLE: u64 = 3;
    const MTL_PRIMITIVE_TYPE_TRIANGLE_STRIP: u64 = 4;
    const MTL_INDEX_TYPE_UINT16: u64 = 0;
    const MTL_CULL_MODE_NONE: u64 = 0;
    const MTL_CULL_MODE_BACK: u64 = 2;
    const MTL_TRIANGLE_FILL_MODE_LINES: u64 = 1;
    const MTL_BARRIER_SCOPE_BUFFERS_AND_RENDER_TARGETS: u64 = 1 | 4;
    const MTL_RENDER_STAGE_FRAGMENT: u64 = 2;
    const METAL_BUFFER_INDEX_OFFSET: u64 = 3;
    const FLUSH_UNIFORM_BUFFER_IDX: u64 = METAL_BUFFER_INDEX_OFFSET;
    const PATH_BASE_INSTANCE_UNIFORM_BUFFER_IDX: u64 = METAL_BUFFER_INDEX_OFFSET + 1;
    const PATH_BUFFER_IDX: u64 = METAL_BUFFER_INDEX_OFFSET + 2;
    const PAINT_BUFFER_IDX: u64 = METAL_BUFFER_INDEX_OFFSET + 3;
    const PAINT_AUX_BUFFER_IDX: u64 = METAL_BUFFER_INDEX_OFFSET + 4;
    const CONTOUR_BUFFER_IDX: u64 = METAL_BUFFER_INDEX_OFFSET + 5;
    const TESS_VERTEX_TEXTURE_IDX: u64 = 7;
    const GRAD_TEXTURE_IDX: u64 = 8;
    const GAUSSIAN_INTEGRAL_TEXTURE_IDX: u64 = 9;
    const FEATHER_ATLAS_TEXTURE_IDX: u64 = 10;
    const IMAGE_TEXTURE_IDX: u64 = 11;
    const COLOR_ATOMIC_BUFFER_IDX: u64 = METAL_BUFFER_INDEX_OFFSET + 13;
    const CLIP_ATOMIC_BUFFER_IDX: u64 = METAL_BUFFER_INDEX_OFFSET + 14;
    const COVERAGE_ATOMIC_BUFFER_IDX: u64 = METAL_BUFFER_INDEX_OFFSET + 16;

    fn scissor(rect: Rect) -> Value {
        Value::Scissor(Scissor {
            x: rect.left as usize,
            y: rect.top as usize,
            width: rect.width() as usize,
            height: rect.height() as usize,
        })
    }

    fn gpu_call<E: MetalExecution>(
        metal: &mut E,
        device: Handle,
        selector: &'static str,
        mut args: Vec<Value>,
    ) -> Option<Handle> {
        args.insert(0, h(device));
        metal.call("gpu", selector, args)
    }

    fn gpu_call_with_error<E: MetalExecution>(
        metal: &mut E,
        device: Handle,
        selector: &'static str,
        mut args: Vec<Value>,
    ) -> ObjectCreation {
        args.insert(0, h(device));
        metal.call_with_error("gpu", selector, args)
    }

    fn make_texture<E: MetalExecution>(
        metal: &mut E,
        device: Handle,
        format: PixelFormat,
        width: u32,
        height: u32,
        mip_count: u32,
        usage: u64,
        texture_type: u64,
        storage_mode: Option<u64>,
        array_length: Option<u32>,
    ) -> Option<Handle> {
        let descriptor = metal
            .call("MTLTextureDescriptor", "alloc/init", vec![])
            .unwrap_or(Handle::NIL);
        metal.owner_event("RC-TD-MEMORYLESS-X3", OwnerEventPhase::Create, descriptor);
        set(
            metal,
            "textureDescriptor",
            "setPixelFormat:",
            vec![h(descriptor), u(format as u64)],
        );
        set(
            metal,
            "textureDescriptor",
            "setTextureType:",
            vec![h(descriptor), u(texture_type)],
        );
        set(
            metal,
            "textureDescriptor",
            "setWidth:",
            vec![h(descriptor), u(width)],
        );
        if texture_type != 1 {
            set(
                metal,
                "textureDescriptor",
                "setHeight:",
                vec![h(descriptor), u(height)],
            );
        }
        set(
            metal,
            "textureDescriptor",
            "setMipmapLevelCount:",
            vec![h(descriptor), u(mip_count)],
        );
        if let Some(array_length) = array_length {
            set(
                metal,
                "textureDescriptor",
                "setArrayLength:",
                vec![h(descriptor), u(array_length)],
            );
        }
        set(
            metal,
            "textureDescriptor",
            "setUsage:",
            vec![h(descriptor), u(usage)],
        );
        if let Some(storage_mode) = storage_mode {
            set(
                metal,
                "textureDescriptor",
                "setStorageMode:",
                vec![h(descriptor), u(storage_mode)],
            );
        }
        let texture = gpu_call(
            metal,
            device,
            "newTextureWithDescriptor:",
            vec![h(descriptor)],
        );
        metal.owner_event(
            "RC-TD-MEMORYLESS-X3",
            OwnerEventPhase::LastUse,
            descriptor,
        );
        // The descriptor is a source lexical local.  The selector borrows
        // it; release the local only after the creation expression returns,
        // including the nil/error path.
        metal.retire_handle(descriptor);
        metal.owner_event("RC-TD-MEMORYLESS-X3", OwnerEventPhase::Release, descriptor);
        texture
    }

    /// Constructor-local Gaussian path.  The pinned initializer keeps the
    /// descriptor alive through both array-slice uploads; generic texture
    /// creation may retire its descriptor immediately, so this source-specific
    /// helper returns the descriptor for the caller's lexical scope.
    fn make_gaussian_texture<E: MetalExecution>(metal: &mut E, device: Handle) -> (Handle, Handle) {
        let descriptor = metal
            .call("MTLTextureDescriptor", "alloc/init", vec![])
            .unwrap_or(Handle::NIL);
        metal.owner_event("RC-TD-GAUSSIAN", OwnerEventPhase::Create, descriptor);
        set(
            metal,
            "textureDescriptor",
            "setPixelFormat:",
            vec![h(descriptor), u(PixelFormat::R16Float as u64)],
        );
        set(
            metal,
            "textureDescriptor",
            "setTextureType:",
            vec![h(descriptor), u(1)],
        );
        set(
            metal,
            "textureDescriptor",
            "setWidth:",
            vec![h(descriptor), u(gpu::GAUSSIAN_TABLE_SIZE as u64)],
        );
        set(
            metal,
            "textureDescriptor",
            "setMipmapLevelCount:",
            vec![h(descriptor), u(1)],
        );
        set(
            metal,
            "textureDescriptor",
            "setArrayLength:",
            vec![h(descriptor), u(2)],
        );
        set(
            metal,
            "textureDescriptor",
            "setUsage:",
            vec![h(descriptor), u(1)],
        );
        let texture = gpu_call(
            metal,
            device,
            "newTextureWithDescriptor:",
            vec![h(descriptor)],
        )
        .unwrap_or(Handle::NIL);
        (texture, descriptor)
    }

    fn make_upload_texture<E: MetalExecution>(
        metal: &mut E,
        device: Handle,
        format: PixelFormat,
        width: u32,
        height: u32,
        mip_count: u32,
    ) -> (Handle, Handle) {
        let descriptor = metal
            .call("MTLTextureDescriptor", "alloc/init", vec![])
            .unwrap_or(Handle::NIL);
        metal.owner_event("RC-TD-IMAGE-UPLOAD", OwnerEventPhase::Create, descriptor);
        set(
            metal,
            "textureDescriptor",
            "setPixelFormat:",
            vec![h(descriptor), u(format as u64)],
        );
        set(
            metal,
            "textureDescriptor",
            "setTextureType:",
            vec![h(descriptor), u(2)],
        );
        set(
            metal,
            "textureDescriptor",
            "setWidth:",
            vec![h(descriptor), u(width as u64)],
        );
        set(
            metal,
            "textureDescriptor",
            "setHeight:",
            vec![h(descriptor), u(height as u64)],
        );
        set(
            metal,
            "textureDescriptor",
            "setMipmapLevelCount:",
            vec![h(descriptor), u(mip_count as u64)],
        );
        set(
            metal,
            "textureDescriptor",
            "setUsage:",
            vec![h(descriptor), u(1)],
        );
        let texture = gpu_call(
            metal,
            device,
            "newTextureWithDescriptor:",
            vec![h(descriptor)],
        )
        .unwrap_or(Handle::NIL);
        (texture, descriptor)
    }

    fn make_canvas_texture<E: MetalExecution>(
        metal: &mut E,
        device: Handle,
        width: u32,
        height: u32,
    ) -> (Handle, Handle) {
        let descriptor = metal
            .call("MTLTextureDescriptor", "alloc/init", vec![])
            .unwrap_or(Handle::NIL);
        metal.owner_event("RC-TD-CANVAS", OwnerEventPhase::Create, descriptor);
        set(
            metal,
            "textureDescriptor",
            "setPixelFormat:",
            vec![h(descriptor), u(PixelFormat::RGBA8Unorm as u64)],
        );
        set(
            metal,
            "textureDescriptor",
            "setWidth:",
            vec![h(descriptor), u(width as u64)],
        );
        set(
            metal,
            "textureDescriptor",
            "setHeight:",
            vec![h(descriptor), u(height as u64)],
        );
        set(
            metal,
            "textureDescriptor",
            "setUsage:",
            vec![h(descriptor), u(5)],
        );
        set(
            metal,
            "textureDescriptor",
            "setTextureType:",
            vec![h(descriptor), u(2)],
        );
        set(
            metal,
            "textureDescriptor",
            "setMipmapLevelCount:",
            vec![h(descriptor), u(1)],
        );
        set(
            metal,
            "textureDescriptor",
            "setStorageMode:",
            vec![h(descriptor), u(2)],
        );
        let texture = gpu_call(
            metal,
            device,
            "newTextureWithDescriptor:",
            vec![h(descriptor)],
        )
        .unwrap_or(Handle::NIL);
        (texture, descriptor)
    }

    fn make_resize_texture<E: MetalExecution>(
        metal: &mut E,
        device: Handle,
        ledger_id: &'static str,
        format: PixelFormat,
        width: u32,
        height: u32,
    ) -> (Handle, Handle) {
        let descriptor = metal
            .call("MTLTextureDescriptor", "alloc/init", vec![])
            .unwrap_or(Handle::NIL);
        metal.owner_event(ledger_id, OwnerEventPhase::Create, descriptor);
        set(
            metal,
            "textureDescriptor",
            "setPixelFormat:",
            vec![h(descriptor), u(format as u64)],
        );
        set(
            metal,
            "textureDescriptor",
            "setTextureType:",
            vec![h(descriptor), u(2)],
        );
        set(
            metal,
            "textureDescriptor",
            "setWidth:",
            vec![h(descriptor), u(width as u64)],
        );
        set(
            metal,
            "textureDescriptor",
            "setHeight:",
            vec![h(descriptor), u(height as u64)],
        );
        set(
            metal,
            "textureDescriptor",
            "setMipmapLevelCount:",
            vec![h(descriptor), u(1)],
        );
        set(
            metal,
            "textureDescriptor",
            "setUsage:",
            vec![h(descriptor), u(5)],
        );
        set(
            metal,
            "textureDescriptor",
            "setStorageMode:",
            vec![h(descriptor), u(2)],
        );
        let texture = gpu_call(
            metal,
            device,
            "newTextureWithDescriptor:",
            vec![h(descriptor)],
        )
        .unwrap_or(Handle::NIL);
        (texture, descriptor)
    }

    #[repr(u64)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum PixelFormat {
        RGBA8Unorm = 70,
        RGBA8UnormSrgb = 71,
        BGRA8Unorm = 80,
        BGRA8UnormSrgb = 81,
        R16Float = 25,
        R32Uint = 53,
        RGBA16Float = 115,
        RGBA32Uint = 123,
        BC7RGBAUnorm = 152,
        EacRGBA8 = 178,
        ASTC4x4Ldr = 186,
        ASTC5x4Ldr = 187,
        ASTC5x5Ldr = 188,
        ASTC6x5Ldr = 189,
        ASTC6x6Ldr = 190,
        ASTC8x5Ldr = 191,
        ASTC8x6Ldr = 192,
        ASTC8x8Ldr = 193,
        ASTC10x5Ldr = 194,
        ASTC10x6Ldr = 195,
        ASTC10x8Ldr = 196,
        ASTC10x10Ldr = 197,
        ASTC12x10Ldr = 198,
        ASTC12x12Ldr = 199,
    }
    pub use gpu::{DrawType, InterlockMode, LoadAction, ShaderFeatures, ShaderMiscFlags};
    pub use crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::ShaderCompilationMode;
    #[cfg(feature = "with-rive-tools")]
    pub use gpu::SynthesizedFailureType;
    #[cfg(not(feature = "with-rive-tools"))]
    #[repr(i32)]
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub enum SynthesizedFailureType {
        #[default]
        none = 0,
        ubershaderLoad = 1,
        shaderCompilation = 2,
        pipelineCreation = 3,
    }

    pub use gpu::BarrierFlags;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum PipelineSemanticKind {
        ColorRamp,
        Tessellate,
        FeatherFill,
        FeatherStroke,
        Draw,
    }

    /// Source identity retained alongside a native pipeline-state handle.
    /// Tags describe the state that Metal actually binds; they are not a
    /// prediction derived from a logical flush descriptor.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct PipelineSemantic {
        pub kind: PipelineSemanticKind,
        pub draw_type: Option<DrawType>,
        pub interlock: Option<InterlockMode>,
        pub features: ShaderFeatures,
        pub misc: ShaderMiscFlags,
    }

    impl PipelineSemantic {
        pub const fn simple(kind: PipelineSemanticKind) -> Self {
            Self {
                kind,
                draw_type: None,
                interlock: None,
                features: ShaderFeatures(0),
                misc: ShaderMiscFlags(0),
            }
        }

        pub const fn draw(
            draw_type: DrawType,
            interlock: InterlockMode,
            features: ShaderFeatures,
            misc: ShaderMiscFlags,
        ) -> Self {
            Self {
                kind: PipelineSemanticKind::Draw,
                draw_type: Some(draw_type),
                interlock: Some(interlock),
                features,
                misc,
            }
        }
    }

    fn make_pipeline_state<E: MetalExecution>(
        metal: &mut E,
        device: Handle,
        descriptor: Handle,
    ) -> Option<Handle> {
        let creation = gpu_call_with_error(
            metal,
            device,
            "newRenderPipelineStateWithDescriptor:error:",
            vec![h(descriptor)],
        );
        if let Some(error) = creation.error_owner_handle {
            metal.owner_event("RC-ERR-PIPE", OwnerEventPhase::Create, error);
        }
        if creation.has_error() || creation.object.is_none() {
            metal.log(format!(
                "RIVE: make_pipeline_state error {}",
                creation.error_description().as_deref().unwrap_or("<nil>")
            ));
            if let Some(error) = creation.error_owner_handle {
                metal.owner_event("RC-ERR-PIPE", OwnerEventPhase::LastUse, error);
                metal.retire_handle(error);
                metal.owner_event("RC-ERR-PIPE", OwnerEventPhase::Release, error);
            }
        }
        creation.object
    }

    fn pipeline_attachment<E: MetalExecution>(
        metal: &mut E,
        descriptor: Handle,
        index: u64,
    ) -> Handle {
        // The C++ expression first materializes the descriptor's attachment
        // collection, then obtains one child alias from it. The collection is
        // parent-tied (+0), so its event closes without retiring the parent.
        // Objective-C still sends both messages when either receiver is nil;
        // preserve that selector sequence and merely suppress owner events
        // for the resulting nil expression.
        let collection_result = metal
            .call("descriptor", "colorAttachments", vec![h(descriptor)])
            .unwrap_or(Handle::NIL);
        let collection = if descriptor == Handle::NIL {
            Handle::NIL
        } else {
            collection_result
        };
        if collection != Handle::NIL {
            metal.owner_event(
                "RC-ATT-COLLECTION-PIPE",
                OwnerEventPhase::Borrow,
                collection,
            );
        }
        let attachment_result = metal
            .call(
                "descriptor",
                "colorAttachmentAtIndex:",
                vec![h(descriptor), h(collection), u(index)],
            )
            .unwrap_or(Handle::NIL);
        if collection != Handle::NIL {
            metal.owner_event(
                "RC-ATT-COLLECTION-PIPE",
                OwnerEventPhase::LastUse,
                collection,
            );
            metal.retire_handle(collection);
            metal.owner_event(
                "RC-ATT-COLLECTION-PIPE",
                OwnerEventPhase::AliasEnd,
                collection,
            );
        }
        if descriptor == Handle::NIL || collection == Handle::NIL {
            Handle::NIL
        } else {
            attachment_result
        }
    }

    fn set_pipeline_attachment<E: MetalExecution>(
        metal: &mut E,
        descriptor: Handle,
        ledger_id: &'static str,
        selector: &'static str,
        value: Value,
    ) {
        let attachment = pipeline_attachment(metal, descriptor, 0);
        if attachment != Handle::NIL {
            metal.owner_event(ledger_id, OwnerEventPhase::Borrow, attachment);
        }
        set(metal, "framebuffer", selector, vec![h(attachment), value]);
        if attachment != Handle::NIL {
            metal.owner_event(ledger_id, OwnerEventPhase::LastUse, attachment);
        }
        // Each source property expression owns a separate temporary
        // attachment result; do not extend one registry retain across the
        // following property calls.
        if attachment != Handle::NIL {
            metal.retire_handle(attachment);
            metal.owner_event(ledger_id, OwnerEventPhase::AliasEnd, attachment);
        }
    }

    fn pass_attachment<E: MetalExecution>(
        metal: &mut E,
        descriptor: Handle,
        index: u64,
        ledger_id: &'static str,
    ) -> Handle {
        if descriptor == Handle::NIL {
            return Handle::NIL;
        }
        let collection = metal
            .call("pass", "colorAttachments", vec![h(descriptor)])
            .unwrap_or(Handle::NIL);
        if collection == Handle::NIL {
            return Handle::NIL;
        }
        metal.owner_event(
            "RC-ATT-COLLECTION-PASS",
            OwnerEventPhase::Borrow,
            collection,
        );
        let attachment = metal
            .call(
                "pass",
                "colorAttachmentAtIndex:",
                vec![h(descriptor), h(collection), u(index)],
            )
            .unwrap_or(Handle::NIL);
        metal.owner_event(
            "RC-ATT-COLLECTION-PASS",
            OwnerEventPhase::LastUse,
            collection,
        );
        metal.retire_handle(collection);
        metal.owner_event(
            "RC-ATT-COLLECTION-PASS",
            OwnerEventPhase::AliasEnd,
            collection,
        );
        metal.owner_event(ledger_id, OwnerEventPhase::Borrow, attachment);
        attachment
    }

    fn set_pass_attachment<E: MetalExecution>(
        metal: &mut E,
        descriptor: Handle,
        index: u64,
        ledger_id: &'static str,
        receiver: &'static str,
        selector: &'static str,
        value: Value,
    ) {
        let attachment = pass_attachment(metal, descriptor, index, ledger_id);
        set(metal, receiver, selector, vec![h(attachment), value]);
        if attachment != Handle::NIL {
            metal.owner_event(ledger_id, OwnerEventPhase::LastUse, attachment);
            metal.retire_handle(attachment);
            metal.owner_event(ledger_id, OwnerEventPhase::AliasEnd, attachment);
        }
    }

    #[derive(Debug, Default)]
    pub struct ColorRampPipeline {
        pub state: Option<OwnedMetalHandle>,
    }

    /// The three nested pipeline classes are separate source owners. Their
    /// payloads happen to have the same shape, but their complete-object
    /// identities and destruction boundaries are not interchangeable.
    #[derive(Debug, Default)]
    pub struct TessellatePipeline {
        pub state: Option<OwnedMetalHandle>,
    }

    #[derive(Debug, Default)]
    pub struct FeatherAtlasPipeline {
        pub state: Option<OwnedMetalHandle>,
    }
    impl ColorRampPipeline {
        pub fn color_ramp<E: MetalExecution>(
            metal: &mut E,
            device: Handle,
            library: Handle,
        ) -> Self {
            let desc = metal
                .call("MTLRenderPipelineDescriptor", "alloc/init", vec![])
                .unwrap_or(Handle::NIL);
            if desc != Handle::NIL {
                metal.owner_event("RC-PD-COLOR", OwnerEventPhase::Create, desc);
            }
            let vertex_name = static_text(COLOR_RAMP_VERTEX_NAME);
            let vertex = metal.call(
                "library",
                "newFunctionWithName:",
                vec![h(library), vertex_name],
            );
            if let Some(vertex) = vertex {
                metal.owner_event("RC-FN-COLOR-V", OwnerEventPhase::Create, vertex);
            }
            set(
                metal,
                "descriptor",
                "setVertexFunction:",
                vec![h(desc), vertex.map(h).unwrap_or(Value::Nil)],
            );
            if let Some(vertex) = vertex {
                metal.owner_event("RC-FN-COLOR-V", OwnerEventPhase::LastUse, vertex);
                metal.retire_handle(vertex);
                metal.owner_event("RC-FN-COLOR-V", OwnerEventPhase::Release, vertex);
            }
            let fragment_name = static_text(COLOR_RAMP_FRAGMENT_NAME);
            let fragment = metal.call(
                "library",
                "newFunctionWithName:",
                vec![h(library), fragment_name],
            );
            if let Some(fragment) = fragment {
                metal.owner_event("RC-FN-COLOR-F", OwnerEventPhase::Create, fragment);
            }
            set(
                metal,
                "descriptor",
                "setFragmentFunction:",
                vec![h(desc), fragment.map(h).unwrap_or(Value::Nil)],
            );
            if let Some(fragment) = fragment {
                metal.owner_event("RC-FN-COLOR-F", OwnerEventPhase::LastUse, fragment);
                metal.retire_handle(fragment);
                metal.owner_event("RC-FN-COLOR-F", OwnerEventPhase::Release, fragment);
            }
            let framebuffer = pipeline_attachment(metal, desc, 0);
            if framebuffer != Handle::NIL {
                metal.owner_event("RC-ATT-COLOR-0", OwnerEventPhase::Borrow, framebuffer);
            }
            set(
                metal,
                "framebuffer",
                "setPixelFormat:",
                vec![h(framebuffer), u(PixelFormat::RGBA8Unorm as u64)],
            );
            if framebuffer != Handle::NIL {
                metal.owner_event("RC-ATT-COLOR-0", OwnerEventPhase::LastUse, framebuffer);
                metal.retire_handle(framebuffer);
                metal.owner_event("RC-ATT-COLOR-0", OwnerEventPhase::AliasEnd, framebuffer);
            }
            if desc != Handle::NIL {
                metal.owner_event("RC-PD-COLOR", OwnerEventPhase::LastUse, desc);
            }
            let state = make_pipeline_state(metal, device, desc);
            if let Some(state) = state {
                metal.owner_event("RC-STATE-PIPE", OwnerEventPhase::Create, state);
                metal.tag_pipeline(
                    state,
                    PipelineSemantic::simple(PipelineSemanticKind::ColorRamp),
                );
            }
            let state = state.and_then(|handle| {
                let owner = metal.take_owned(handle, MetalObjectKind::RenderPipelineState);
                if owner.is_some() {
                    metal.owner_event("RC-STATE-PIPE", OwnerEventPhase::Transfer, handle);
                }
                owner
            });
            if desc != Handle::NIL {
                metal.retire_handle(desc);
                metal.owner_event("RC-PD-COLOR", OwnerEventPhase::Release, desc);
            }
            Self { state }
        }
    }
    impl Drop for ColorRampPipeline {
        fn drop(&mut self) {
            #[cfg(test)]
            if let Some(owner) = self.state.as_ref() {
                record_owner_drop("RC-STATE-PIPE", owner.handle());
            }
        }
    }

    impl TessellatePipeline {
        pub fn new<E: MetalExecution>(metal: &mut E, device: Handle, library: Handle) -> Self {
            let desc = metal
                .call("MTLRenderPipelineDescriptor", "alloc/init", vec![])
                .unwrap_or(Handle::NIL);
            if desc != Handle::NIL {
                metal.owner_event("RC-PD-TESS", OwnerEventPhase::Create, desc);
            }
            let vertex_name = static_text(TESSELLATE_VERTEX_NAME);
            let vertex = metal.call(
                "library",
                "newFunctionWithName:",
                vec![h(library), vertex_name],
            );
            if let Some(vertex) = vertex {
                metal.owner_event("RC-FN-TESS-V", OwnerEventPhase::Create, vertex);
            }
            set(
                metal,
                "descriptor",
                "setVertexFunction:",
                vec![h(desc), vertex.map(h).unwrap_or(Value::Nil)],
            );
            if let Some(vertex) = vertex {
                metal.owner_event("RC-FN-TESS-V", OwnerEventPhase::LastUse, vertex);
                metal.retire_handle(vertex);
                metal.owner_event("RC-FN-TESS-V", OwnerEventPhase::Release, vertex);
            }
            let fragment_name = static_text(TESSELLATE_FRAGMENT_NAME);
            let fragment = metal.call(
                "library",
                "newFunctionWithName:",
                vec![h(library), fragment_name],
            );
            if let Some(fragment) = fragment {
                metal.owner_event("RC-FN-TESS-F", OwnerEventPhase::Create, fragment);
            }
            set(
                metal,
                "descriptor",
                "setFragmentFunction:",
                vec![h(desc), fragment.map(h).unwrap_or(Value::Nil)],
            );
            if let Some(fragment) = fragment {
                metal.owner_event("RC-FN-TESS-F", OwnerEventPhase::LastUse, fragment);
                metal.retire_handle(fragment);
                metal.owner_event("RC-FN-TESS-F", OwnerEventPhase::Release, fragment);
            }
            let framebuffer = pipeline_attachment(metal, desc, 0);
            if framebuffer != Handle::NIL {
                metal.owner_event("RC-ATT-TESS-0", OwnerEventPhase::Borrow, framebuffer);
            }
            set(
                metal,
                "framebuffer",
                "setPixelFormat:",
                vec![h(framebuffer), u(PixelFormat::RGBA32Uint as u64)],
            );
            if framebuffer != Handle::NIL {
                metal.owner_event("RC-ATT-TESS-0", OwnerEventPhase::LastUse, framebuffer);
                metal.retire_handle(framebuffer);
                metal.owner_event("RC-ATT-TESS-0", OwnerEventPhase::AliasEnd, framebuffer);
            }
            if desc != Handle::NIL {
                metal.owner_event("RC-PD-TESS", OwnerEventPhase::LastUse, desc);
            }
            let state = make_pipeline_state(metal, device, desc);
            if let Some(state) = state {
                metal.owner_event("RC-STATE-PIPE", OwnerEventPhase::Create, state);
                metal.tag_pipeline(
                    state,
                    PipelineSemantic::simple(PipelineSemanticKind::Tessellate),
                );
            }
            let state = state.and_then(|handle| {
                let owner = metal.take_owned(handle, MetalObjectKind::RenderPipelineState);
                if owner.is_some() {
                    metal.owner_event("RC-STATE-PIPE", OwnerEventPhase::Transfer, handle);
                }
                owner
            });
            if desc != Handle::NIL {
                metal.retire_handle(desc);
                metal.owner_event("RC-PD-TESS", OwnerEventPhase::Release, desc);
            }
            Self { state }
        }
    }
    impl Drop for TessellatePipeline {
        fn drop(&mut self) {
            #[cfg(test)]
            if let Some(owner) = self.state.as_ref() {
                record_owner_drop("RC-STATE-PIPE", owner.handle());
            }
        }
    }

    impl FeatherAtlasPipeline {
        pub fn new<E: MetalExecution>(
            metal: &mut E,
            device: Handle,
            library: Handle,
            fragment_name: &'static str,
            blend_max: bool,
        ) -> Self {
            let desc = metal
                .call("MTLRenderPipelineDescriptor", "alloc/init", vec![])
                .unwrap_or(Handle::NIL);
            metal.owner_event("RC-PD-FEATHER", OwnerEventPhase::Create, desc);
            let vertex_name = static_text(FEATHER_VERTEX_NAME);
            let vertex = metal.call(
                "library",
                "newFunctionWithName:",
                vec![h(library), vertex_name],
            );
            if let Some(vertex) = vertex {
                metal.owner_event("RC-FN-FEATHER-V", OwnerEventPhase::Create, vertex);
            }
            set(
                metal,
                "descriptor",
                "setVertexFunction:",
                vec![h(desc), vertex.map(h).unwrap_or(Value::Nil)],
            );
            if let Some(vertex) = vertex {
                metal.owner_event("RC-FN-FEATHER-V", OwnerEventPhase::LastUse, vertex);
                metal.retire_handle(vertex);
                metal.owner_event("RC-FN-FEATHER-V", OwnerEventPhase::Release, vertex);
            }
            let fragment_name = static_text(fragment_name);
            let fragment = metal.call(
                "library",
                "newFunctionWithName:",
                vec![h(library), fragment_name],
            );
            if let Some(fragment) = fragment {
                metal.owner_event("RC-FN-FEATHER-F", OwnerEventPhase::Create, fragment);
            }
            set(
                metal,
                "descriptor",
                "setFragmentFunction:",
                vec![h(desc), fragment.map(h).unwrap_or(Value::Nil)],
            );
            if let Some(fragment) = fragment {
                metal.owner_event("RC-FN-FEATHER-F", OwnerEventPhase::LastUse, fragment);
                metal.retire_handle(fragment);
                metal.owner_event("RC-FN-FEATHER-F", OwnerEventPhase::Release, fragment);
            }
            set_pipeline_attachment(
                metal,
                desc,
                "RC-ATT-FEATHER-0-X9",
                "setPixelFormat:",
                u(PixelFormat::R16Float as u64),
            );
            set_pipeline_attachment(
                metal,
                desc,
                "RC-ATT-FEATHER-0-X9",
                "setBlendingEnabled:",
                b(true),
            );
            set_pipeline_attachment(
                metal,
                desc,
                "RC-ATT-FEATHER-0-X9",
                "setSourceRGBBlendFactor:",
                u(1),
            );
            set_pipeline_attachment(
                metal,
                desc,
                "RC-ATT-FEATHER-0-X9",
                "setDestinationRGBBlendFactor:",
                u(1),
            );
            set_pipeline_attachment(
                metal,
                desc,
                "RC-ATT-FEATHER-0-X9",
                "setRgbBlendOperation:",
                u(if blend_max { 4 } else { 0 }),
            );
            set_pipeline_attachment(
                metal,
                desc,
                "RC-ATT-FEATHER-0-X9",
                "setSourceAlphaBlendFactor:",
                u(1),
            );
            set_pipeline_attachment(
                metal,
                desc,
                "RC-ATT-FEATHER-0-X9",
                "setDestinationAlphaBlendFactor:",
                u(1),
            );
            set_pipeline_attachment(
                metal,
                desc,
                "RC-ATT-FEATHER-0-X9",
                "setAlphaBlendOperation:",
                u(if blend_max { 4 } else { 0 }),
            );
            set_pipeline_attachment(metal, desc, "RC-ATT-FEATHER-0-X9", "setWriteMask:", u(15));
            metal.owner_event("RC-PD-FEATHER", OwnerEventPhase::LastUse, desc);
            let state = make_pipeline_state(metal, device, desc);
            if let Some(state) = state {
                metal.owner_event("RC-STATE-PIPE", OwnerEventPhase::Create, state);
                let kind = if blend_max {
                    PipelineSemanticKind::FeatherStroke
                } else {
                    PipelineSemanticKind::FeatherFill
                };
                metal.tag_pipeline(state, PipelineSemantic::simple(kind));
            }
            let state = state.and_then(|handle| {
                let owner = metal.take_owned(handle, MetalObjectKind::RenderPipelineState);
                if owner.is_some() {
                    metal.owner_event("RC-STATE-PIPE", OwnerEventPhase::Transfer, handle);
                }
                owner
            });
            metal.retire_handle(desc);
            metal.owner_event("RC-PD-FEATHER", OwnerEventPhase::Release, desc);
            Self { state }
        }
    }
    impl Drop for FeatherAtlasPipeline {
        fn drop(&mut self) {
            #[cfg(test)]
            if let Some(owner) = self.state.as_ref() {
                record_owner_drop("RC-STATE-PIPE", owner.handle());
            }
        }
    }

    #[derive(Debug, Default)]
    pub struct DrawPipeline {
        pub rgba8: Option<OwnedMetalHandle>,
        pub bgra8: Option<OwnedMetalHandle>,
    }
    impl DrawPipeline {
        pub fn new<E: MetalExecution>(
            metal: &mut E,
            device: Handle,
            library: Option<Handle>,
            vertex_name: SourceFunctionName,
            fragment_name: SourceFunctionName,
            draw_type: DrawType,
            interlock: InterlockMode,
            features: ShaderFeatures,
            misc: ShaderMiscFlags,
            synthesized_failure: SynthesizedFailureType,
        ) -> Self {
            let Some(library) = library else {
                return Self::default();
            };
            if synthesized_failure == SynthesizedFailureType::pipelineCreation {
                metal.log("RIVE: Synthesizing pipeline creation failure...".into());
                return Self::default();
            }
            // These are source lexical NSString locals. Keep both native
            // owners alive for the complete DrawPipeline constructor; the
            // selector bridge only borrows them.
            let (_vertex_name_owner, vertex_name_value) = source_function_name(vertex_name);
            let (_fragment_name_owner, fragment_name_value) = source_function_name(fragment_name);
            // The Objective-C++ lambda captures `gpu` by value.  Keep one
            // independent strong device owner alive across both format
            // builds and release it only after the captured function locals
            // have been retired below.
            let Some(gpu_capture_owner) = metal.clone_owned(device, MetalObjectKind::Device) else {
                // The source lambda captures a valid device by value.  Do not
                // silently fall back to the ambient executor device when that
                // strong capture cannot be established.
                for (ledger_id, name) in [
                    ("RC-NS-FUNCTION-NAME-V", _vertex_name_owner),
                    ("RC-NS-FUNCTION-NAME-F", _fragment_name_owner),
                ] {
                    if let Some(name) = name {
                        metal.retire_handle(name);
                        metal.owner_event(ledger_id, OwnerEventPhase::AliasEnd, name);
                    }
                }
                return Self::default();
            };
            metal.owner_event(
                "RC-DRAW-LAMBDA-GPU",
                OwnerEventPhase::CreateClone,
                gpu_capture_owner.handle(),
            );
            let captured_device = gpu_capture_owner.handle();
            if let Some(name) = _vertex_name_owner {
                metal.owner_event("RC-NS-FUNCTION-NAME-V", OwnerEventPhase::Borrow, name);
            }
            let vertex = metal.call(
                "library",
                "newFunctionWithName:",
                vec![h(library), vertex_name_value],
            );
            if let Some(vertex) = vertex {
                metal.owner_event("RC-FN-DRAW-V", OwnerEventPhase::Create, vertex);
            }
            if let Some(name) = _fragment_name_owner {
                metal.owner_event("RC-NS-FUNCTION-NAME-F", OwnerEventPhase::Borrow, name);
            }
            let fragment = metal.call(
                "library",
                "newFunctionWithName:",
                vec![h(library), fragment_name_value],
            );
            if let Some(fragment) = fragment {
                metal.owner_event("RC-FN-DRAW-F", OwnerEventPhase::Create, fragment);
            }
            // The precompiled names are autoreleased +0 source locals. Their
            // registry entries are nonowning aliases and end at the authored
            // selector scope; no synthetic retain is introduced here.
            if let Some(name) = _vertex_name_owner {
                metal.retire_handle(name);
                metal.owner_event("RC-NS-FUNCTION-NAME-V", OwnerEventPhase::AliasEnd, name);
            }
            if let Some(name) = _fragment_name_owner {
                metal.retire_handle(name);
                metal.owner_event("RC-NS-FUNCTION-NAME-F", OwnerEventPhase::AliasEnd, name);
            }
            let mut build_failed = false;
            let mut build = |format: PixelFormat| {
                if build_failed {
                    return None;
                }
                // Odr-use the captured device owner from the source lambda;
                // this is intentionally one owner for both RGBA/BGRA builds,
                // not one clone per invocation.
                let _captured_gpu = captured_device;
                let desc = metal
                    .call("MTLRenderPipelineDescriptor", "alloc/init", vec![])
                    .unwrap_or(Handle::NIL);
                if desc != Handle::NIL {
                    metal.owner_event("RC-PD-DRAW-X2", OwnerEventPhase::Create, desc);
                }
                set(
                    metal,
                    "descriptor",
                    "setVertexFunction:",
                    vec![h(desc), vertex.map(h).unwrap_or(Value::Nil)],
                );
                set(
                    metal,
                    "descriptor",
                    "setFragmentFunction:",
                    vec![h(desc), fragment.map(h).unwrap_or(Value::Nil)],
                );
                let framebuffer_alias = pipeline_attachment(metal, desc, 0);
                if framebuffer_alias != Handle::NIL {
                    metal.owner_event(
                        "RC-ATT-DRAW-FB-X2",
                        OwnerEventPhase::BorrowAlias,
                        framebuffer_alias,
                    );
                }
                // Unlike the direct-expression MRT children, the source
                // `auto* framebuffer` is a named strong local and remains
                // alive through make_pipeline_state. Keep its independent
                // owner while the descriptor consumes the attachment.
                let framebuffer_owner = if framebuffer_alias == Handle::NIL {
                    None
                } else if let Some(owner) = metal.clone_owned(
                    framebuffer_alias,
                    MetalObjectKind::RenderPipelineColorAttachmentDescriptor,
                ) {
                    Some(owner)
                } else {
                    // A valid Objective-C strong local cannot fail to retain.
                    // If the erased adapter cannot establish that exact +1,
                    // close the borrowed expression and descriptor scope and
                    // fail this translated constructor without using a +0
                    // alias as a substitute owner.
                    if framebuffer_alias != Handle::NIL {
                        metal.retire_handle(framebuffer_alias);
                        metal.owner_event(
                            "RC-ATT-DRAW-FB-X2",
                            OwnerEventPhase::AliasEnd,
                            framebuffer_alias,
                        );
                    }
                    if desc != Handle::NIL {
                        metal.retire_handle(desc);
                        metal.owner_event("RC-PD-DRAW-X2", OwnerEventPhase::Release, desc);
                    }
                    build_failed = true;
                    return None;
                };
                if let Some(owner) = framebuffer_owner.as_ref() {
                    metal.owner_event(
                        "RC-ATT-DRAW-FB-X2",
                        OwnerEventPhase::CreateStrong,
                        owner.handle(),
                    );
                }
                let framebuffer = framebuffer_owner
                    .as_ref()
                    .map(OwnedMetalHandle::handle)
                    .unwrap_or(Handle::NIL);
                set(
                    metal,
                    "framebuffer",
                    "setPixelFormat:",
                    vec![h(framebuffer), u(format as u64)],
                );
                match interlock {
                    InterlockMode::RasterOrdering => {
                        let clip = pipeline_attachment(metal, desc, 1);
                        metal.owner_event("RC-ATT-DRAW-CLIP", OwnerEventPhase::Borrow, clip);
                        set(
                            metal,
                            "clipAttachment",
                            "setPixelFormat:",
                            vec![h(clip), u(PixelFormat::R32Uint as u64)],
                        );
                        metal.owner_event("RC-ATT-DRAW-CLIP", OwnerEventPhase::LastUse, clip);
                        if clip != Handle::NIL {
                            metal.retire_handle(clip);
                        }
                        metal.owner_event("RC-ATT-DRAW-CLIP", OwnerEventPhase::AliasEnd, clip);
                        let scratch = pipeline_attachment(metal, desc, 2);
                        metal.owner_event("RC-ATT-DRAW-SCRATCH", OwnerEventPhase::Borrow, scratch);
                        set(
                            metal,
                            "scratchAttachment",
                            "setPixelFormat:",
                            vec![h(scratch), u(format as u64)],
                        );
                        metal.owner_event(
                            "RC-ATT-DRAW-SCRATCH",
                            OwnerEventPhase::LastUse,
                            scratch,
                        );
                        if scratch != Handle::NIL {
                            metal.retire_handle(scratch);
                        }
                        metal.owner_event(
                            "RC-ATT-DRAW-SCRATCH",
                            OwnerEventPhase::AliasEnd,
                            scratch,
                        );
                        let coverage = pipeline_attachment(metal, desc, 3);
                        metal.owner_event(
                            "RC-ATT-DRAW-COVERAGE",
                            OwnerEventPhase::Borrow,
                            coverage,
                        );
                        set(
                            metal,
                            "coverageAttachment",
                            "setPixelFormat:",
                            vec![h(coverage), u(PixelFormat::R32Uint as u64)],
                        );
                        metal.owner_event(
                            "RC-ATT-DRAW-COVERAGE",
                            OwnerEventPhase::LastUse,
                            coverage,
                        );
                        if coverage != Handle::NIL {
                            metal.retire_handle(coverage);
                        }
                        metal.owner_event(
                            "RC-ATT-DRAW-COVERAGE",
                            OwnerEventPhase::AliasEnd,
                            coverage,
                        );
                    }
                    InterlockMode::Atomics
                        if misc.has(ShaderMiscFlags::FIXED_FUNCTION_COLOR_OUTPUT) =>
                    {
                        set(
                            metal,
                            "framebuffer",
                            "setBlendingEnabled:",
                            vec![h(framebuffer), b(true)],
                        );
                        set(
                            metal,
                            "framebuffer",
                            "setSourceRGBBlendFactor:",
                            vec![h(framebuffer), u(1)],
                        );
                        set(
                            metal,
                            "framebuffer",
                            "setDestinationRGBBlendFactor:",
                            vec![h(framebuffer), u(5)],
                        );
                        set(
                            metal,
                            "framebuffer",
                            "setRgbBlendOperation:",
                            vec![h(framebuffer), u(0)],
                        );
                        set(
                            metal,
                            "framebuffer",
                            "setSourceAlphaBlendFactor:",
                            vec![h(framebuffer), u(1)],
                        );
                        set(
                            metal,
                            "framebuffer",
                            "setDestinationAlphaBlendFactor:",
                            vec![h(framebuffer), u(5)],
                        );
                        set(
                            metal,
                            "framebuffer",
                            "setAlphaBlendOperation:",
                            vec![h(framebuffer), u(0)],
                        );
                        set(
                            metal,
                            "framebuffer",
                            "setWriteMask:",
                            vec![h(framebuffer), u(15)],
                        );
                    }
                    InterlockMode::Atomics if draw_type == DrawType::RenderPassResolve => {
                        set(
                            metal,
                            "framebuffer",
                            "setBlendingEnabled:",
                            vec![h(framebuffer), b(false)],
                        );
                        set(
                            metal,
                            "framebuffer",
                            "setWriteMask:",
                            vec![h(framebuffer), u(15)],
                        );
                    }
                    InterlockMode::Atomics => {
                        set(
                            metal,
                            "framebuffer",
                            "setBlendingEnabled:",
                            vec![h(framebuffer), b(false)],
                        );
                        set(
                            metal,
                            "framebuffer",
                            "setWriteMask:",
                            vec![h(framebuffer), u(0)],
                        );
                    }
                    _ => rive_unreachable(),
                }
                let state = make_pipeline_state(metal, captured_device, desc).and_then(|handle| {
                    metal.take_owned(handle, MetalObjectKind::RenderPipelineState)
                });
                if desc != Handle::NIL {
                    metal.owner_event("RC-PD-DRAW-X2", OwnerEventPhase::LastUse, desc);
                }
                if let Some(owner) = framebuffer_owner.as_ref() {
                    metal.owner_event(
                        "RC-ATT-DRAW-FB-X2",
                        OwnerEventPhase::LastUse,
                        owner.handle(),
                    );
                }
                // The source lambda owns only the framebuffer attachment and
                // descriptor at this boundary; the MRT property expressions
                // above were released immediately after each setter.
                if let Some(framebuffer_owner) = framebuffer_owner {
                    let framebuffer = framebuffer_owner.handle();
                    drop(framebuffer_owner);
                    metal.owner_event(
                        "RC-ATT-DRAW-FB-X2",
                        OwnerEventPhase::ReleaseStrong,
                        framebuffer,
                    );
                }
                if framebuffer_alias != Handle::NIL {
                    metal.retire_handle(framebuffer_alias);
                    metal.owner_event(
                        "RC-ATT-DRAW-FB-X2",
                        OwnerEventPhase::AliasEnd,
                        framebuffer_alias,
                    );
                }
                // The named framebuffer ARC local is declared after the
                // descriptor and therefore releases before that descriptor at
                // lambda exit.  Keep the explicit source order here rather
                // than allowing the registry alias bookkeeping to invert it.
                if desc != Handle::NIL {
                    metal.retire_handle(desc);
                    metal.owner_event("RC-PD-DRAW-X2", OwnerEventPhase::Release, desc);
                }
                state
            };
            let rgba8 = build(PixelFormat::RGBA8Unorm);
            let bgra8 = build(PixelFormat::BGRA8Unorm);
            drop(build);
            metal.owner_event(
                "RC-DRAW-LAMBDA-GPU",
                OwnerEventPhase::LastUse,
                captured_device,
            );
            let semantic = PipelineSemantic::draw(draw_type, interlock, features, misc);
            for state in [rgba8.as_ref(), bgra8.as_ref()].into_iter().flatten() {
                metal.tag_pipeline(state.handle(), semantic);
            }
            if let Some(handle) = fragment {
                metal.owner_event("RC-FN-DRAW-F", OwnerEventPhase::LastUse, handle);
                metal.retire_handle(handle);
                metal.owner_event("RC-FN-DRAW-F", OwnerEventPhase::Release, handle);
            }
            if let Some(handle) = vertex {
                metal.owner_event("RC-FN-DRAW-V", OwnerEventPhase::LastUse, handle);
                metal.retire_handle(handle);
                metal.owner_event("RC-FN-DRAW-V", OwnerEventPhase::Release, handle);
            }
            // The source lambda is destroyed after its fragment/vertex
            // locals, so its captured device retain is released last.
            let gpu_capture_handle = gpu_capture_owner.handle();
            drop(gpu_capture_owner);
            metal.owner_event(
                "RC-DRAW-LAMBDA-GPU",
                OwnerEventPhase::Release,
                gpu_capture_handle,
            );
            if build_failed {
                return Self::default();
            }
            Self { rgba8, bgra8 }
        }
        pub fn valid(&self) -> bool {
            debug_assert_eq!(self.rgba8.is_some(), self.bgra8.is_some());
            self.rgba8.is_some()
        }
        pub fn pipeline_state(&self, format: PixelFormat) -> Handle {
            debug_assert!(self.valid());
            debug_assert!(matches!(
                format,
                PixelFormat::RGBA8Unorm
                    | PixelFormat::RGBA8UnormSrgb
                    | PixelFormat::RGBA16Float
                    | PixelFormat::BGRA8Unorm
                    | PixelFormat::BGRA8UnormSrgb
            ));
            match format {
                PixelFormat::RGBA8Unorm
                | PixelFormat::RGBA8UnormSrgb
                | PixelFormat::RGBA16Float => self
                    .rgba8
                    .as_ref()
                    .map(OwnedMetalHandle::handle)
                    .unwrap_or(Handle::NIL),
                PixelFormat::BGRA8Unorm | PixelFormat::BGRA8UnormSrgb => self
                    .bgra8
                    .as_ref()
                    .map(OwnedMetalHandle::handle)
                    .unwrap_or(Handle::NIL),
                // The pinned switch's format assertion is NDEBUG-elided and
                // its default branch uses the BGRA state.
                _ => self
                    .bgra8
                    .as_ref()
                    .map(OwnedMetalHandle::handle)
                    .unwrap_or(Handle::NIL),
            }
        }
    }

    impl Drop for DrawPipeline {
        fn drop(&mut self) {
            // The source declares RGBA before BGRA; ARC scope destruction is
            // therefore BGRA then RGBA.
            drop(self.bgra8.take());
            drop(self.rgba8.take());
        }
    }

    pub(crate) fn precompiled_name<E: MetalExecution>(
        metal: &mut E,
        ledger_id: &'static str,
        draw: DrawType,
        features: ShaderFeatures,
        misc: ShaderMiscFlags,
        base: &str,
    ) -> SourceFunctionName {
        let translated = match draw {
            DrawType::MidpointFanPatches => gpu::DrawType::midpointFanPatches,
            DrawType::MidpointFanCenterAAPatches => gpu::DrawType::midpointFanCenterAAPatches,
            DrawType::OuterCurvePatches => gpu::DrawType::outerCurvePatches,
            DrawType::InteriorTriangulation => gpu::DrawType::interiorTriangulation,
            DrawType::FeatherAtlasBlit => gpu::DrawType::featherAtlasBlit,
            DrawType::ImageMesh => gpu::DrawType::imageMesh,
            DrawType::ImageRect => gpu::DrawType::imageRect,
            _ => rive_unreachable(),
        };
        let name = precompiled_function_name(
            translated,
            features.0,
            misc.has(ShaderMiscFlags::CLOCKWISE_FILL),
            base,
        )
        .unwrap_or_else(|| super::rive_unreachable());
        let (prefix, rest) = name.split_at(1);
        let (namespace_id, function_base_name) = rest
            .split_once("::")
            .unwrap_or_else(|| super::rive_unreachable());
        let prefix = prefix.as_bytes()[0];
        let result = metal
            .make_precompiled_function_name(prefix, namespace_id, function_base_name)
            .map(SourceFunctionName::Dynamic)
            .unwrap_or_else(|| SourceFunctionName::DynamicText(name));
        if let SourceFunctionName::Dynamic(handle) = result {
            metal.owner_event(ledger_id, OwnerEventPhase::CreateBridge, handle);
        }
        result
    }

    #[repr(C)]
    pub struct BufferRingMetal {
        base: ManuallyDrop<BufferRing>,
        buffers: [ManuallyDrop<OwnedMetalHandle>; 3],
    }
    impl BufferRingMetal {
        #[cfg(test)]
        pub fn base_offset_for_test() -> usize {
            core::mem::offset_of!(Self, base)
        }
        pub fn make<E: MetalExecution>(
            metal: &mut E,
            device: Handle,
            capacity: usize,
        ) -> Option<Self> {
            if capacity == 0 {
                return None;
            }
            let buffers = [
                gpu_call(
                    metal,
                    device,
                    "newBufferWithLength:options:",
                    vec![u(capacity as u64), u(0)],
                )
                .unwrap_or(Handle::NIL),
                gpu_call(
                    metal,
                    device,
                    "newBufferWithLength:options:",
                    vec![u(capacity as u64), u(0)],
                )
                .unwrap_or(Handle::NIL),
                gpu_call(
                    metal,
                    device,
                    "newBufferWithLength:options:",
                    vec![u(capacity as u64), u(0)],
                )
                .unwrap_or(Handle::NIL),
            ];
            let buffers = buffers.map(|handle| {
                ManuallyDrop::new(
                    metal
                        .take_owned(handle, MetalObjectKind::Buffer)
                        .unwrap_or_else(|| OwnedMetalHandle::token(handle)),
                )
            });
            Some(Self {
                base: ManuallyDrop::new(BufferRing::new(capacity)),
                buffers,
            })
        }
        pub fn capacity(&self) -> usize {
            unsafe { (&*self.base).capacityInBytes() }
        }
        pub fn submitted_buffer(&self) -> Handle {
            unsafe { (&*self.buffers[(&*self.base).submittedBufferIdx() as usize]).handle() }
        }
        pub fn map(&self, buffer: usize, size: usize) -> *mut u8 {
            debug_assert!(size <= unsafe { (&*self.base).capacityInBytes() });
            unsafe {
                (&*self.buffers[buffer])
                    .buffer_contents()
                    .unwrap_or(core::ptr::null_mut())
            }
        }
        pub fn unmap_and_submit(&mut self, buffer: usize, _size: usize) {
            let _ = buffer;
        }
        pub fn handles(&self) -> [Handle; 3] {
            std::array::from_fn(|index| unsafe { (&*self.buffers[index]).handle() })
        }
    }

    /// Complete source owner for `RenderBufferMetalImpl : RiveRenderBuffer`.
    /// The inherited RiveRenderBuffer is at offset zero; the derived members
    /// follow the pinned declaration order and are released in reverse.
    #[repr(C)]
    pub struct RenderBufferMetal {
        pub base: ManuallyDrop<RiveRenderBuffer>,
        pub m_gpu: ManuallyDrop<OwnedMetalHandle>,
        pub m_buffers: [ManuallyDrop<OwnedMetalHandle>; 3],
        pub m_submittedBufferIdx: i32,
    }
    impl core::fmt::Debug for RenderBufferMetal {
        fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            formatter
                .debug_struct("RenderBufferMetal")
                .finish_non_exhaustive()
        }
    }
    impl RenderBufferMetal {
        pub fn new<E: MetalExecution>(
            metal: &mut E,
            device: OwnedMetalHandle,
            buffer_type: RenderBufferType,
            flags: RenderBufferFlags,
            size: usize,
            mapped_once: bool,
        ) -> Self {
            let mut buffers: [ManuallyDrop<OwnedMetalHandle>; 3] =
                core::array::from_fn(|_| ManuallyDrop::new(OwnedMetalHandle::token(Handle::NIL)));
            for (index, slot) in buffers.iter_mut().enumerate() {
                if mapped_once && index != 0 {
                    continue;
                }
                let handle = gpu_call(
                    metal,
                    device.handle(),
                    "newBufferWithLength:options:",
                    vec![u(size as u64), u(0)],
                )
                .and_then(|handle| metal.take_owned(handle, MetalObjectKind::Buffer))
                .unwrap_or_else(|| OwnedMetalHandle::token(Handle::NIL));
                *slot = ManuallyDrop::new(handle);
            }
            Self {
                base: ManuallyDrop::new(unsafe {
                    RiveRenderBuffer::new_for_owner::<Self>(buffer_type, flags, size)
                }),
                m_gpu: ManuallyDrop::new(device),
                m_buffers: buffers,
                m_submittedBufferIdx: -1,
            }
        }
        pub fn submitted_buffer(&mut self) -> Option<Handle> {
            let index = unsafe { (&mut *self.base).frontBufferIdx() };
            usize::try_from(index)
                .ok()
                .and_then(|index| self.m_buffers.get(index))
                .map(|owner| unsafe { (&*owner).handle() })
                .filter(|handle| *handle != Handle::NIL)
        }
        pub fn front_buffer_index(&mut self) -> Option<usize> {
            usize::try_from(unsafe { (&mut *self.base).frontBufferIdx() }).ok()
        }
        pub fn back_buffer_contents(&self) -> *mut u8 {
            let index = usize::try_from(unsafe { (&*self.base).backBufferIdx() }).ok();
            index
                .and_then(|index| self.m_buffers.get(index))
                .and_then(|owner| unsafe { (&*owner).buffer_contents() })
                .unwrap_or(core::ptr::null_mut())
        }
        pub fn map(&self) -> Handle {
            let index = unsafe { (&*self.base).backBufferIdx() };
            usize::try_from(index)
                .ok()
                .and_then(|index| self.m_buffers.get(index))
                .map(|owner| unsafe { (&*owner).handle() })
                .filter(|handle| *handle != Handle::NIL)
                .unwrap_or_else(|| {
                    debug_assert!(false, "back buffer must exist");
                    Handle::NIL
                })
        }
        pub fn buffer_handles(&self) -> [Option<Handle>; 3] {
            core::array::from_fn(|index| {
                let handle = unsafe { (&*self.m_buffers[index]).handle() };
                (handle != Handle::NIL).then_some(handle)
            })
        }
        pub fn unmap(&mut self) {
            unsafe { (&mut *self.base).unmap() };
        }
    }

    impl Drop for RenderBufferMetal {
        fn drop(&mut self) {
            unsafe {
                ManuallyDrop::drop(&mut self.m_buffers[2]);
                ManuallyDrop::drop(&mut self.m_buffers[1]);
                ManuallyDrop::drop(&mut self.m_buffers[0]);
                ManuallyDrop::drop(&mut self.m_gpu);
                ManuallyDrop::drop(&mut self.base);
            }
        }
    }

    impl LiteRttiTypeId for RenderBufferMetal {
        const LITE_RTTI_TYPE_ID: u32 = CONST_ID("RenderBufferMetalImpl");
    }

    impl LiteRttiCastFrom<crate::mechanical_port::source::include::rive::renderer_hpp::RenderBuffer>
        for RenderBufferMetal
    {
        unsafe fn from_base(
            base: *mut crate::mechanical_port::source::include::rive::renderer_hpp::RenderBuffer,
        ) -> *mut Self {
            base.cast()
        }
    }

    impl RenderBufferContract for RenderBufferMetal {
        fn onMap(&mut self) -> *mut core::ffi::c_void {
            self.back_buffer_contents().cast()
        }

        fn onUnmap(&mut self) {}
    }

    impl BufferRingContract for BufferRingMetal {
        fn bufferRing(&self) -> &BufferRing {
            unsafe { &*self.base }
        }

        fn bufferRingMut(&mut self) -> &mut BufferRing {
            unsafe { &mut *self.base }
        }

        fn onMapBuffer(&mut self, bufferIdx: i32, mapSizeInBytes: usize) -> *mut core::ffi::c_void {
            self.map(bufferIdx as usize, mapSizeInBytes).cast()
        }

        fn onUnmapAndSubmitBuffer(&mut self, bufferIdx: i32, mapSizeInBytes: usize) {
            self.unmap_and_submit(bufferIdx as usize, mapSizeInBytes);
        }

        fn submittedHandle(&self) -> Option<Handle> {
            self.submitted_buffer().into()
        }
    }

    impl Drop for BufferRingMetal {
        fn drop(&mut self) {
            unsafe {
                // Source declaration is m_buffers then the BufferRing base.
                ManuallyDrop::drop(&mut self.buffers[2]);
                ManuallyDrop::drop(&mut self.buffers[1]);
                ManuallyDrop::drop(&mut self.buffers[0]);
                ManuallyDrop::drop(&mut self.base);
            }
        }
    }

    #[repr(C)]
    pub struct TextureMetal {
        base: ManuallyDrop<Texture>,
        pub texture: ManuallyDrop<OwnedMetalHandle>,
        mips_dirty: Cell<bool>,
    }

    unsafe fn destroy_texture_metal(base: *mut Texture) {
        #[cfg(test)]
        TEXTURE_METAL_DROP_COUNT.fetch_add(1, Ordering::SeqCst);
        unsafe { drop(Box::from_raw(base.cast::<TextureMetal>())) };
    }

    unsafe fn texture_metal_native_handle(base: *const Texture) -> *mut core::ffi::c_void {
        unsafe { (&*base.cast::<TextureMetal>()).native_pointer() }
    }

    impl TextureMetal {
        #[cfg(test)]
        pub fn base_offset_for_test() -> usize {
            core::mem::offset_of!(Self, base)
        }

        #[cfg(test)]
        pub unsafe fn release_for_test(owner: *mut Self) {
            unsafe { (&*owner).base.unref() };
        }
        pub unsafe fn install_dispatch(
            &mut self,
            destroy_complete: unsafe fn(*mut Texture),
            native_handle: unsafe fn(*const Texture) -> *mut core::ffi::c_void,
        ) {
            unsafe {
                (&mut *self.base).destroy_complete = destroy_complete;
                (&mut *self.base).setNativeHandleDispatch(native_handle);
            }
        }
        pub fn new<E: MetalExecution>(
            metal: &mut E,
            device: Handle,
            width: u32,
            height: u32,
            mip_count: u32,
            image_data: Arc<[u8]>,
            format: PixelFormat,
            block_width: u8,
            block_height: u8,
            bytes_per_block: u32,
            generate_mips: bool,
        ) -> Option<Self> {
            let levels = if generate_mips { 1 } else { mip_count };
            let mut required = 0usize;
            for level in 0..levels {
                let w = 1.max(width >> level);
                let ht = 1.max(height >> level);
                let blocks_x = (w + block_width as u32 - 1) / block_width as u32;
                let blocks_y = (ht + block_height as u32 - 1) / block_height as u32;
                let row = blocks_x.checked_mul(bytes_per_block)?;
                let level_bytes = row.checked_mul(blocks_y)? as usize;
                required = required.checked_add(level_bytes)?;
            }
            if image_data.len() < required {
                return None;
            }
            // SAFETY: the adapter above validated every mip-level byte span;
            // the private source constructor preserves the authored selector
            // order and nil-texture continuation without mid-loop admission.
            unsafe {
                return Self::from_upload_unchecked(
                    metal,
                    device,
                    width,
                    height,
                    mip_count,
                    image_data,
                    format,
                    block_width,
                    block_height,
                    bytes_per_block,
                    generate_mips,
                );
            }
        }

        unsafe fn from_upload_unchecked<E: MetalExecution>(
            metal: &mut E,
            device: Handle,
            width: u32,
            height: u32,
            mip_count: u32,
            image_data: Arc<[u8]>,
            format: PixelFormat,
            block_width: u8,
            block_height: u8,
            bytes_per_block: u32,
            generate_mips: bool,
        ) -> Option<Self> {
            // Pinned Objective-C++ keeps constructing after a nil
            // `newTextureWithDescriptor:` result; later messages to nil are
            // intentional no-ops.
            let (texture, texture_descriptor) =
                make_upload_texture(metal, device, format, width, height, mip_count);
            let levels = if generate_mips { 1 } else { mip_count };
            let mut source_offset = 0usize;
            for level in 0..levels {
                let w = 1.max(width >> level);
                let ht = 1.max(height >> level);
                let blocks_x = (w + block_width as u32 - 1) / block_width as u32;
                let blocks_y = (ht + block_height as u32 - 1) / block_height as u32;
                let row = blocks_x * bytes_per_block;
                let level_bytes = (row * blocks_y) as usize;
                let source_end = source_offset + level_bytes as usize;
                let source = &image_data[source_offset..source_end];
                set(
                    metal,
                    "texture",
                    "replaceRegion:mipmapLevel:withBytes:bytesPerRow:",
                    vec![
                        h(texture),
                        Value::Origin(Origin::default()),
                        Value::Size(Size {
                            width: w as usize,
                            height: ht as usize,
                            depth: 1,
                        }),
                        u(level),
                        Value::Bytes(Arc::from(source)),
                        u(row),
                    ],
                );
                source_offset = source_end;
            }
            let texture = metal
                .take_owned(texture, MetalObjectKind::Texture)
                .unwrap_or_else(|| OwnedMetalHandle::token(texture));
            let mut texture = Self {
                base: ManuallyDrop::new(Texture::new(width, height)),
                texture: ManuallyDrop::new(texture),
                mips_dirty: Cell::new(generate_mips && mip_count > 1),
            };
            unsafe { texture.install_dispatch(destroy_texture_metal, texture_metal_native_handle) };
            // Keep the descriptor through every authored mip upload and the
            // complete source TextureMetal installation.
            metal.owner_event(
                "RC-TD-IMAGE-UPLOAD",
                OwnerEventPhase::LastUse,
                texture_descriptor,
            );
            metal.retire_handle(texture_descriptor);
            metal.owner_event(
                "RC-TD-IMAGE-UPLOAD",
                OwnerEventPhase::Release,
                texture_descriptor,
            );
            Some(texture)
        }
        pub fn adopt(texture: Option<OwnedMetalHandle>, width: u32, height: u32) -> Option<Self> {
            if width == 0 || height == 0 {
                None
            } else {
                texture
                    .filter(|texture| texture.handle() != Handle::NIL)
                    .map(|texture| Self::from_native(texture, width, height))
            }
        }
        /// Private source constructor used by makeRenderCanvas. The pinned
        /// TextureMetalImpl constructor accepts a nil native texture; public
        /// adoptImageTexture performs the separate nonnil admission check.
        pub fn from_native(texture: OwnedMetalHandle, width: u32, height: u32) -> Self {
            let mut texture = Self {
                base: ManuallyDrop::new(Texture::new(width, height)),
                texture: ManuallyDrop::new(texture),
                mips_dirty: Cell::new(false),
            };
            unsafe { texture.install_dispatch(destroy_texture_metal, texture_metal_native_handle) };
            texture
        }
        pub fn ensure_mipmaps<E: MetalExecution>(&self, metal: &mut E, command: Handle) {
            if self.mips_dirty.get() {
                let encoder = metal.call("commandBuffer", "blitCommandEncoder", vec![h(command)]);
                if let Some(encoder) = encoder {
                    metal.owner_event("RC-ENC-MIP", OwnerEventPhase::Create, encoder);
                    set(
                        metal,
                        "blitEncoder",
                        "generateMipmapsForTexture:",
                        vec![h(encoder), h(unsafe { (&*self.texture).handle() })],
                    );
                    set(metal, "blitEncoder", "endEncoding", vec![h(encoder)]);
                }
                self.mips_dirty.set(false);
                // Source local scope releases the encoder after the dirty
                // state transition, not inside the selector adapter.
                if let Some(encoder) = encoder {
                    metal.owner_event("RC-ENC-MIP", OwnerEventPhase::LastUse, encoder);
                    metal.retire_handle(encoder);
                    metal.owner_event("RC-ENC-MIP", OwnerEventPhase::Release, encoder);
                }
            }
        }
        #[cfg(test)]
        pub fn mark_mipmaps_dirty_for_test(&self) {
            self.mips_dirty.set(true);
        }
        #[cfg(test)]
        pub fn mipmaps_dirty_for_test(&self) -> bool {
            self.mips_dirty.get()
        }
        pub fn native_handle(&self) -> Handle {
            unsafe { (&*self.texture).handle() }
        }
        pub fn native_pointer(&self) -> *mut core::ffi::c_void {
            #[cfg(target_vendor = "apple")]
            {
                unsafe {
                    self.texture
                        .native_object()
                        .map_or(core::ptr::null_mut(), |object| {
                            core::ptr::from_ref(object).cast_mut().cast()
                        })
                }
            }
            #[cfg(not(target_vendor = "apple"))]
            {
                core::ptr::null_mut()
            }
        }
    }

    impl Drop for TextureMetal {
        fn drop(&mut self) {
            unsafe {
                ManuallyDrop::drop(&mut self.texture);
                ManuallyDrop::drop(&mut self.base);
            }
        }
    }

    impl LiteRttiTypeId for TextureMetal {
        const LITE_RTTI_TYPE_ID: u32 = CONST_ID("TextureMetalImpl");
    }

    impl LiteRttiCastFrom<Texture> for TextureMetal {
        unsafe fn from_base(base: *mut Texture) -> *mut Self {
            base.cast()
        }
    }

    impl crate::mechanical_port::source::renderer::include::rive::renderer::texture_hpp::TextureContract
        for TextureMetal
    {
        fn nativeHandle(&self) -> *mut core::ffi::c_void {
            self.native_pointer()
        }
    }

    fn make_image_texture_with_device<E: MetalExecution>(
        metal: &mut E,
        device: Handle,
        width: u32,
        height: u32,
        mip_count: u32,
        image_data: Arc<[u8]>,
        format: super::TextureFormat,
        block_width: u8,
        block_height: u8,
        _srgb: bool,
        generate_mips: bool,
    ) -> Option<TextureMetal> {
        let (pixel, bytes, compressed) = match format {
            super::TextureFormat::rgba32 => {
                debug_assert_eq!((block_width, block_height), (1, 1));
                (PixelFormat::RGBA8Unorm, 4, false)
            }
            super::TextureFormat::bc7
                if cfg!(not(any(
                    target_os = "ios",
                    target_os = "tvos",
                    target_os = "visionos"
                ))) =>
            {
                (PixelFormat::BC7RGBAUnorm, 16, true)
            }
            super::TextureFormat::astc => {
                const FORMATS: [PixelFormat; 14] = [
                    PixelFormat::ASTC4x4Ldr,
                    PixelFormat::ASTC5x4Ldr,
                    PixelFormat::ASTC5x5Ldr,
                    PixelFormat::ASTC6x5Ldr,
                    PixelFormat::ASTC6x6Ldr,
                    PixelFormat::ASTC8x5Ldr,
                    PixelFormat::ASTC8x6Ldr,
                    PixelFormat::ASTC8x8Ldr,
                    PixelFormat::ASTC10x5Ldr,
                    PixelFormat::ASTC10x6Ldr,
                    PixelFormat::ASTC10x8Ldr,
                    PixelFormat::ASTC10x10Ldr,
                    PixelFormat::ASTC12x10Ldr,
                    PixelFormat::ASTC12x12Ldr,
                ];
                let index = crate::mechanical_port::source::decoders::include::rive::decoders::astc_footprints_hpp::astcFootprintIndex(block_width, block_height);
                let index: usize = index.try_into().unwrap_or(usize::MAX);
                debug_assert!(index < FORMATS.len());
                let pixel = FORMATS.get(index).copied()?;
                (pixel, 16, true)
            }
            super::TextureFormat::etc2 => (PixelFormat::EacRGBA8, 16, true),
            _ => {
                debug_assert!(false, "unsupported GPU texture format");
                return None;
            }
        };
        debug_assert!(
            !(generate_mips && compressed),
            "mipmap generation is undefined for compressed textures"
        );
        TextureMetal::new(
            metal,
            device,
            width,
            height,
            mip_count,
            image_data,
            pixel,
            block_width,
            block_height,
            bytes,
            generate_mips,
        )
    }

    pub use gpu::PlatformFeatures;
    fn shader_key(
        draw: DrawType,
        features: ShaderFeatures,
        interlock: InterlockMode,
        misc: ShaderMiscFlags,
    ) -> u32 {
        let draw = match draw {
            DrawType::MidpointFanPatches => gpu::DrawType::midpointFanPatches,
            DrawType::MidpointFanCenterAAPatches => gpu::DrawType::midpointFanCenterAAPatches,
            DrawType::OuterCurvePatches => gpu::DrawType::outerCurvePatches,
            DrawType::InteriorTriangulation => gpu::DrawType::interiorTriangulation,
            DrawType::FeatherAtlasBlit => gpu::DrawType::featherAtlasBlit,
            DrawType::ImageRect => gpu::DrawType::imageRect,
            DrawType::ImageMesh => gpu::DrawType::imageMesh,
            DrawType::MsaaStrokes => gpu::DrawType::msaaStrokes,
            DrawType::MsaaMidpointFanBorrowedCoverage => {
                gpu::DrawType::msaaMidpointFanBorrowedCoverage
            }
            DrawType::MsaaMidpointFans => gpu::DrawType::msaaMidpointFans,
            DrawType::MsaaMidpointFanStencilReset => gpu::DrawType::msaaMidpointFanStencilReset,
            DrawType::MsaaDynamicMidpointFans => gpu::DrawType::msaaDynamicMidpointFans,
            DrawType::MsaaMidpointFanPathsStencil => gpu::DrawType::msaaMidpointFanPathsStencil,
            DrawType::MsaaMidpointFanPathsCover => gpu::DrawType::msaaMidpointFanPathsCover,
            DrawType::MsaaOuterCubics => gpu::DrawType::msaaOuterCubics,
            DrawType::ClipReset => gpu::DrawType::clipReset,
            DrawType::RenderPassInitialize => gpu::DrawType::renderPassInitialize,
            DrawType::RenderPassResolve => gpu::DrawType::renderPassResolve,
        };
        let interlock = match interlock {
            InterlockMode::RasterOrdering => gpu::InterlockMode::rasterOrdering,
            InterlockMode::Atomics => gpu::InterlockMode::atomics,
            InterlockMode::Clockwise => gpu::InterlockMode::clockwise,
            InterlockMode::ClockwiseAtomic => gpu::InterlockMode::clockwiseAtomic,
            InterlockMode::Msaa => gpu::InterlockMode::msaa,
        };
        crate::mechanical_port::source::renderer::src::gpu_cpp::ShaderUniqueKey(
            draw,
            gpu::ShaderFeatures(features.0),
            interlock,
            gpu::ShaderMiscFlags(misc.0),
        )
    }
    #[cfg(test)]
    pub(crate) fn shader_key_for_test(
        draw: DrawType,
        features: ShaderFeatures,
        interlock: InterlockMode,
        misc: ShaderMiscFlags,
    ) -> u32 {
        shader_key(draw, features, interlock, misc)
    }
    #[derive(Default)]
    pub struct PipelineCache {
        pub pipelines: HashMap<u32, Option<DrawPipeline>>,
    }

    #[inline]
    fn draw_batches(desc: &gpu::FlushDescriptor) -> impl Iterator<Item = &gpu::DrawBatch> {
        desc.drawList
            .map(|list| unsafe { list.as_ref() })
            .into_iter()
            .flat_map(|list| list.iter())
    }

    #[inline]
    unsafe fn atlas_batches<'a>(
        pointer: Option<core::ptr::NonNull<gpu::AtlasDrawBatch>>,
        count: usize,
    ) -> &'a [gpu::AtlasDrawBatch] {
        if count == 0 {
            return &[];
        }
        let pointer = pointer.unwrap_or_else(|| super::rive_unreachable());
        unsafe { core::slice::from_raw_parts(pointer.as_ptr(), count) }
    }

    #[inline]
    fn source_scissor(value: gpu::AABBu16) -> Rect {
        Rect {
            left: value.left.into(),
            top: value.top.into(),
            right: value.right.into(),
            bottom: value.bottom.into(),
        }
    }

    #[inline]
    fn source_bounds(value: gpu::IAABB) -> Rect {
        Rect {
            left: value.left as u32,
            top: value.top as u32,
            right: value.right as u32,
            bottom: value.bottom as u32,
        }
    }

    #[inline]
    fn source_clear_color(value: u32) -> [f64; 4] {
        let [alpha, red, green, blue] = value.to_be_bytes();
        let alpha = f64::from(alpha) / 255.0;
        [
            f64::from(red) / 255.0 * alpha,
            f64::from(green) / 255.0 * alpha,
            f64::from(blue) / 255.0 * alpha,
            alpha,
        ]
    }

    #[inline]
    pub(crate) unsafe fn metal_buffer_handle(
        buffer: core::ptr::NonNull<
            crate::mechanical_port::source::include::rive::renderer_hpp::RenderBuffer,
        >,
    ) -> Option<Handle> {
        let cast = unsafe { lite_rtti_cast::<RenderBufferMetal, _>(buffer.as_ptr()) };
        if cast.is_null() {
            None
        } else {
            // A successful source cast is distinct from a NIL submitted
            // handle. The pinned image-mesh branch binds NIL and continues;
            // only a failed RTTI cast breaks the batch.
            Some(unsafe { (&mut *cast).submitted_buffer().unwrap_or(Handle::NIL) })
        }
    }

    /// Source image-mesh preflight: all three Lite RTTI casts happen before
    /// any selector is emitted. A valid Metal buffer may still carry NIL for
    /// its submitted native buffer; that is different from a failed cast.
    pub(crate) unsafe fn image_mesh_buffer_handles(
        vertex: Option<
            core::ptr::NonNull<
                crate::mechanical_port::source::include::rive::renderer_hpp::RenderBuffer,
            >,
        >,
        uv: Option<
            core::ptr::NonNull<
                crate::mechanical_port::source::include::rive::renderer_hpp::RenderBuffer,
            >,
        >,
        index: Option<
            core::ptr::NonNull<
                crate::mechanical_port::source::include::rive::renderer_hpp::RenderBuffer,
            >,
        >,
    ) -> Option<(Handle, Handle, Handle)> {
        let (Some(vertex), Some(uv), Some(index)) = (
            vertex.and_then(|buffer| unsafe { metal_buffer_handle(buffer) }),
            uv.and_then(|buffer| unsafe { metal_buffer_handle(buffer) }),
            index.and_then(|buffer| unsafe { metal_buffer_handle(buffer) }),
        ) else {
            return None;
        };
        Some((vertex, uv, index))
    }

    #[inline]
    unsafe fn metal_texture_handle(texture: core::ptr::NonNull<Texture>) -> Option<Handle> {
        // Texture has no lite-RTTI root in the pinned hierarchy; the Metal
        // source path performs the authored static cast from the retained
        // Texture base to TextureMetalImpl (base is offset zero).
        Some(unsafe { (&*texture.as_ptr().cast::<TextureMetal>()).native_handle() })
    }

    #[inline]
    #[cfg(feature = "with-rive-tools")]
    fn synthesized_failure(desc: &gpu::FlushDescriptor) -> SynthesizedFailureType {
        desc.synthesizedFailureType
    }

    #[inline]
    #[cfg(not(feature = "with-rive-tools"))]
    fn synthesized_failure(_: &gpu::FlushDescriptor) -> SynthesizedFailureType {
        SynthesizedFailureType::none
    }

    /// A safe source-shaped adaptation of the three authored std::mutex
    /// members. The C++ callback unlocks a mutex on a different thread; Rust
    /// represents that handoff with a guarded held bit and condition variable,
    /// never by moving a MutexGuard across threads. `post_flush` remains
    /// unsafe because the callback borrows the pinned member address.
    pub(crate) struct SourceMutex {
        mutex: Mutex<bool>,
        released: Condvar,
    }
    impl SourceMutex {
        pub(crate) fn new() -> Self {
            Self {
                mutex: Mutex::new(false),
                released: Condvar::new(),
            }
        }

        pub(crate) fn lock(&self) {
            let mut held = self
                .mutex
                .lock()
                .unwrap_or_else(|poison| poison.into_inner());
            while *held {
                held = self
                    .released
                    .wait(held)
                    .unwrap_or_else(|poison| poison.into_inner());
            }
            *held = true;
        }

        pub(crate) fn try_lock(&self) -> bool {
            let Ok(mut held) = self.mutex.try_lock() else {
                return false;
            };
            if *held {
                return false;
            }
            *held = true;
            true
        }

        pub(crate) unsafe fn unlock(&self) {
            let mut held = self
                .mutex
                .lock()
                .unwrap_or_else(|poison| poison.into_inner());
            debug_assert!(*held);
            *held = false;
            self.released.notify_one();
        }
    }

    #[repr(C)]
    pub struct BufferRingLock {
        pub(crate) mutex: SourceMutex,
    }

    impl BufferRingLock {
        pub(crate) fn new() -> Self {
            Self {
                mutex: SourceMutex::new(),
            }
        }
    }

    #[derive(Clone, Copy)]
    struct BufferRingLockPtr(*const BufferRingLock);
    unsafe impl Send for BufferRingLockPtr {}
    impl BufferRingLockPtr {
        unsafe fn as_ref(&self) -> &BufferRingLock {
            unsafe { &*self.0 }
        }
    }

    fn source_platform_features(base: &RenderContextHelperImpl) -> PlatformFeatures {
        let features = unsafe { &*base.base }.m_platformFeatures;
        PlatformFeatures {
            supportsRasterOrderingMode: features.supportsRasterOrderingMode,
            supportsAtomicMode: features.supportsAtomicMode,
            ..PlatformFeatures::default()
        }
    }

    fn derive_source_capabilities<E: MetalExecution>(
        metal: &mut E,
        device: Handle,
        options: &crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::ContextOptions,
    ) -> (PlatformFeatures, AtomicBarrierType) {
        let max_texture_size = if metal.device_supports_family(device, 1002)
            || metal.device_supports_family(device, 2002)
        {
            16384
        } else {
            8192
        };
        let mut features = PlatformFeatures {
            avoidFlatVaryings: true,
            clipSpaceBottomUp: true,
            framebufferBottomUp: false,
            maxTextureSize: max_texture_size,
            atomicPLSInitNeedsDraw: true,
            supportsClipScissor: true,
            ..PlatformFeatures::default()
        };
        #[cfg(all(
            any(target_os = "ios", target_os = "tvos", target_os = "visionos"),
            not(target_abi = "sim")
        ))]
        {
            features.supportsRasterOrderingMode = true;
            features.supportsAtomicMode = false;
            if !metal.device_is_apple_silicon(device) {
                features.pathIDGranularity = 8;
            }
        }
        #[cfg(any(target_os = "ios", target_os = "tvos", target_os = "visionos"))]
        #[cfg(target_abi = "sim")]
        {
            features.supportsRasterOrderingMode = false;
            features.supportsAtomicMode = true;
        }
        #[cfg(any(target_os = "ios", target_os = "tvos", target_os = "visionos"))]
        {
            // The pinned compression capability block is shared by embedded
            // devices and simulators; it is not part of the device-only
            // raster-ordering branch above.
            features.supportsTextureCompressionETC2 = true;
            features.supportsTextureCompressionASTC = true;
        }
        #[cfg(not(any(target_os = "ios", target_os = "tvos", target_os = "visionos")))]
        {
            let apple1 = metal.device_supports_family(device, 1001);
            features.supportsRasterOrderingMode = apple1 && !options.disableFramebufferReads;
            features.supportsAtomicMode = true;
            features.supportsTextureCompressionBC = true;
            // The upstream constructor deliberately asks for Apple1 again
            // at the later ASTC assignment.  Keep this as a separate selector
            // call: a device seam may observe (or trace) the two authored
            // queries independently.
            features.supportsTextureCompressionASTC = metal.device_supports_family(device, 1001);
        }
        #[cfg(all(
            any(target_os = "ios", target_os = "tvos", target_os = "visionos"),
            not(target_abi = "sim")
        ))]
        let barrier = AtomicBarrierType::rasterOrderGroup;
        #[cfg(all(target_os = "ios", target_abi = "sim"))]
        let barrier = if metal.host_architecture_is_arm64() {
            AtomicBarrierType::rasterOrderGroup
        } else {
            AtomicBarrierType::renderPassBreak
        };
        #[cfg(any(
            all(target_os = "tvos", target_abi = "sim"),
            all(target_os = "visionos", target_abi = "sim")
        ))]
        let barrier = AtomicBarrierType::rasterOrderGroup;
        #[cfg(not(any(target_os = "ios", target_os = "tvos", target_os = "visionos")))]
        let barrier = if (metal.device_supports_family(device, 3002)
            || metal.device_supports_family(device, 2002))
            && !metal.device_supports_family(device, 1003)
        {
            AtomicBarrierType::memoryBarrier
        } else if metal.device_raster_order_groups_supported(device) {
            AtomicBarrierType::rasterOrderGroup
        } else {
            AtomicBarrierType::renderPassBreak
        };
        (features, barrier)
    }

    #[cfg(target_vendor = "apple")]
    pub(super) fn canonical_metallib() -> Arc<[u8]> {
        #[cfg(all(target_os = "ios", not(target_abi = "sim")))]
        let bytes = super::rive_pls_ios_metallib;
        #[cfg(all(target_os = "ios", target_abi = "sim"))]
        let bytes = super::rive_pls_ios_simulator_metallib;
        #[cfg(all(target_os = "visionos", not(target_abi = "sim")))]
        let bytes = super::rive_renderer_xros_metallib;
        #[cfg(all(target_os = "visionos", target_abi = "sim"))]
        let bytes = super::rive_renderer_xros_simulator_metallib;
        #[cfg(all(target_os = "tvos", not(target_abi = "sim")))]
        let bytes = super::rive_renderer_appletvos_metallib;
        #[cfg(all(target_os = "tvos", target_abi = "sim"))]
        let bytes = super::rive_renderer_appletvsimulator_metallib;
        #[cfg(target_os = "macos")]
        let bytes = super::rive_pls_macosx_metallib;
        Arc::from(bytes)
    }

    #[cfg(not(target_vendor = "apple"))]
    pub(super) fn canonical_metallib() -> Arc<[u8]> {
        Arc::from(&[][..])
    }

    #[cfg(target_vendor = "apple")]
    fn make_background_shader_compiler(
        gpu: &OwnedMetalHandle,
        features: crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::MetalFeatures,
    ) -> Option<BackgroundShaderCompilerOwner> {
        let object = gpu.native_object()?;
        let retained = unsafe {
            crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::Retained::<crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::MTLDevice>::retain(
                object as *const objc2::runtime::AnyObject as *mut _
            )?
        };
        Some(
            crate::mechanical_port::source::renderer::src::metal::background_shader_compiler_mm::new_for_device(
                retained,
                features,
            ),
        )
    }

    /// Canonical complete translation of
    /// `RenderContextMetalImpl : RenderContextHelperImpl`. Header and `.mm`
    /// declarations share this owner; native integration may append host-only
    /// lifecycle state outside it but never another RenderContextImpl base.
    #[repr(C)]
    pub struct RenderContextMetal {
        pub(crate) base: ManuallyDrop<RenderContextHelperImpl>,
        m_contextOptions: ManuallyDrop<crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::ContextOptions>,
        m_gpu: ManuallyDrop<OwnedMetalHandle>,
        pub(crate) m_commandQueue: ManuallyDrop<Option<OwnedMetalHandle>>,
        m_metalFeatures: ManuallyDrop<crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::MetalFeatures>,
        m_backgroundShaderCompiler: ManuallyDrop<Option<BackgroundShaderCompilerOwner>>,
        m_plsPrecompiledLibrary: ManuallyDrop<Option<OwnedMetalHandle>>,
        m_colorRampPipeline: ManuallyDrop<Option<ColorRampPipeline>>,
        m_gradientTexture: ManuallyDrop<Option<OwnedMetalHandle>>,
        m_gaussianIntegralTexture: ManuallyDrop<Option<OwnedMetalHandle>>,
        m_tessPipeline: ManuallyDrop<Option<TessellatePipeline>>,
        m_tessSpanIndexBuffer: ManuallyDrop<Option<OwnedMetalHandle>>,
        m_tessVertexTexture: ManuallyDrop<Option<OwnedMetalHandle>>,
        m_featherAtlasFillPipeline: ManuallyDrop<Option<FeatherAtlasPipeline>>,
        m_featherAtlasStrokePipeline: ManuallyDrop<Option<FeatherAtlasPipeline>>,
        m_featherAtlasTexture: ManuallyDrop<Option<OwnedMetalHandle>>,
        m_imageSamplers: ManuallyDrop<[Option<OwnedMetalHandle>; ImageSampler::MAX_SAMPLER_PERMUTATIONS]>,
        m_drawPipelines: ManuallyDrop<PipelineCache>,
        m_pathPatchVertexBuffer: ManuallyDrop<Option<OwnedMetalHandle>>,
        m_pathPatchIndexBuffer: ManuallyDrop<Option<OwnedMetalHandle>>,
        m_imageRectVertexBuffer: ManuallyDrop<Option<OwnedMetalHandle>>,
        m_imageRectIndexBuffer: ManuallyDrop<Option<OwnedMetalHandle>>,
        m_bufferRingLocks: ManuallyDrop<[BufferRingLock; 3]>,
        m_bufferRingIdx: ManuallyDrop<i32>,
    }

    impl Drop for RenderContextMetal {
        fn drop(&mut self) {
            unsafe {
                let m_contextOptions = ManuallyDrop::take(&mut self.m_contextOptions);
                let m_gpu = ManuallyDrop::take(&mut self.m_gpu);
                let m_commandQueue = ManuallyDrop::take(&mut self.m_commandQueue);
                let m_metalFeatures = ManuallyDrop::take(&mut self.m_metalFeatures);
                let m_backgroundShaderCompiler =
                    ManuallyDrop::take(&mut self.m_backgroundShaderCompiler);
                let m_plsPrecompiledLibrary = ManuallyDrop::take(&mut self.m_plsPrecompiledLibrary);
                let m_colorRampPipeline = ManuallyDrop::take(&mut self.m_colorRampPipeline);
                let m_gradientTexture = ManuallyDrop::take(&mut self.m_gradientTexture);
                let m_gaussianIntegralTexture =
                    ManuallyDrop::take(&mut self.m_gaussianIntegralTexture);
                let m_tessPipeline = ManuallyDrop::take(&mut self.m_tessPipeline);
                let m_tessSpanIndexBuffer = ManuallyDrop::take(&mut self.m_tessSpanIndexBuffer);
                let m_tessVertexTexture = ManuallyDrop::take(&mut self.m_tessVertexTexture);
                let m_featherAtlasFillPipeline =
                    ManuallyDrop::take(&mut self.m_featherAtlasFillPipeline);
                let m_featherAtlasStrokePipeline =
                    ManuallyDrop::take(&mut self.m_featherAtlasStrokePipeline);
                let m_featherAtlasTexture = ManuallyDrop::take(&mut self.m_featherAtlasTexture);
                let m_imageSamplers = ManuallyDrop::take(&mut self.m_imageSamplers);
                let m_drawPipelines = ManuallyDrop::take(&mut self.m_drawPipelines);
                let m_pathPatchVertexBuffer = ManuallyDrop::take(&mut self.m_pathPatchVertexBuffer);
                let m_pathPatchIndexBuffer = ManuallyDrop::take(&mut self.m_pathPatchIndexBuffer);
                let m_imageRectVertexBuffer = ManuallyDrop::take(&mut self.m_imageRectVertexBuffer);
                let m_imageRectIndexBuffer = ManuallyDrop::take(&mut self.m_imageRectIndexBuffer);
                let m_bufferRingLocks = ManuallyDrop::take(&mut self.m_bufferRingLocks);
                let m_bufferRingIdx = ManuallyDrop::take(&mut self.m_bufferRingIdx);
                macro_rules! trace_stage {
                    ($stage:literal) => {
                        #[cfg(test)]
                        RENDER_CONTEXT_METAL_DROP_TRACE.lock().unwrap().push($stage);
                    };
                }
                macro_rules! release {
                    ($name:ident, $stage:literal) => {{
                        trace_stage!($stage);
                        drop($name);
                    }};
                }
                trace_stage!("bufferRingIdx");
                let _ = m_bufferRingIdx;
                trace_stage!("bufferRingLocks");
                for lock in m_bufferRingLocks.into_iter().rev() {
                    drop(lock);
                }
                release!(m_imageRectIndexBuffer, "imageRectIndexBuffer");
                release!(m_imageRectVertexBuffer, "imageRectVertexBuffer");
                release!(m_pathPatchIndexBuffer, "pathPatchIndexBuffer");
                release!(m_pathPatchVertexBuffer, "pathPatchVertexBuffer");
                release!(m_drawPipelines, "drawPipelines");
                trace_stage!("imageSamplers");
                for sampler in m_imageSamplers.into_iter().rev() {
                    drop(sampler);
                }
                release!(m_featherAtlasTexture, "featherAtlasTexture");
                release!(m_featherAtlasStrokePipeline, "featherAtlasStrokePipeline");
                release!(m_featherAtlasFillPipeline, "featherAtlasFillPipeline");
                release!(m_tessVertexTexture, "tessVertexTexture");
                release!(m_tessSpanIndexBuffer, "tessSpanIndexBuffer");
                release!(m_tessPipeline, "tessPipeline");
                release!(m_gaussianIntegralTexture, "gaussianIntegralTexture");
                release!(m_gradientTexture, "gradientTexture");
                release!(m_colorRampPipeline, "colorRampPipeline");
                release!(m_plsPrecompiledLibrary, "plsPrecompiledLibrary");
                release!(m_backgroundShaderCompiler, "backgroundShaderCompiler");
                trace_stage!("metalFeatures");
                let _ = m_metalFeatures;
                release!(m_commandQueue, "commandQueue");
                release!(m_gpu, "gpu");
                trace_stage!("contextOptions");
                let _ = m_contextOptions;
                ManuallyDrop::drop(&mut self.base);
            }
        }
    }

    impl RenderContextMetal {
        /// Immutable capability snapshot published by the one canonical
        /// source owner. Product admission must consume this after source
        /// construction rather than probing the device in a parallel path.
        pub fn source_capability_snapshot(
            &self,
        ) -> (
            crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::PlatformFeatures,
            crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::MetalFeatures,
        ){
            let platform = unsafe { (&*self.base.base).m_platformFeatures };
            let metal = *self.m_metalFeatures;
            (platform, metal)
        }

        /// Source-owned image construction always receives the canonical
        /// context device. The executor only supplies the current selector
        /// alias for that retained owner; its ambient device is never used.
        pub fn make_image_texture<E: MetalExecution>(
            &mut self,
            metal: &mut E,
            width: u32,
            height: u32,
            mip_count: u32,
            image_data: Arc<[u8]>,
            format: super::TextureFormat,
            block_width: u8,
            block_height: u8,
            srgb: bool,
            generate_mips: bool,
        ) -> Option<TextureMetal> {
            let device = metal.publish_owned(&mut *self.m_gpu)?;
            make_image_texture_with_device(
                metal,
                device,
                width,
                height,
                mip_count,
                image_data,
                format,
                block_width,
                block_height,
                srgb,
                generate_mips,
            )
        }

        pub fn new<E: MetalExecution>(
            metal: &mut E,
            device: Handle,
            context_options: crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::ContextOptions,
        ) -> Self {
            // The pinned initializer retains/copies these source members
            // before entering the resource-building body. Later failures may
            // therefore leave the compiler and options alive with resources
            // still nil.
            let canonical_gpu = metal
                .take_owned(device, MetalObjectKind::Device)
                .unwrap_or_else(|| OwnedMetalHandle::token(device));
            let (features, atomic_barrier) =
                derive_source_capabilities(metal, canonical_gpu.handle(), &context_options);
            let mut base = RenderContextHelperImpl::new(RenderContextImpl::default());
            unsafe {
                (&mut *base.base).m_platformFeatures = features;
            }
            // The source derives all sampler descriptors from the complete
            // ImageSampler key space; the adapter tuple is retained only for
            // ABI compatibility and is not an authority.
            let sampler_permutations: Vec<(u32, u32, u32)> = (0
                ..ImageSampler::MAX_SAMPLER_PERMUTATIONS)
                .map(|key| {
                    let sampler = ImageSampler::SamplerFromKey(key as u8);
                    (
                        sampler.wrapX.0.into(),
                        sampler.wrapY.0.into(),
                        sampler.filter.0.into(),
                    )
                })
                .collect();
            let metal_features = crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::MetalFeatures { atomicBarrierType: atomic_barrier };
            let ring_state = core::array::from_fn(|_| BufferRingLock::new());
            // Construct the complete source owner before any selector work;
            // subsequent statements publish members in pinned source order.
            let mut this = Self {
                base: ManuallyDrop::new(base),
                m_contextOptions: ManuallyDrop::new(context_options),
                m_gpu: ManuallyDrop::new(canonical_gpu),
                m_commandQueue: ManuallyDrop::new(None),
                m_metalFeatures: ManuallyDrop::new(metal_features),
                m_backgroundShaderCompiler: ManuallyDrop::new(None),
                m_plsPrecompiledLibrary: ManuallyDrop::new(None),
                m_colorRampPipeline: ManuallyDrop::new(None),
                m_gradientTexture: ManuallyDrop::new(None),
                m_gaussianIntegralTexture: ManuallyDrop::new(None),
                m_tessPipeline: ManuallyDrop::new(None),
                m_tessSpanIndexBuffer: ManuallyDrop::new(None),
                m_tessVertexTexture: ManuallyDrop::new(None),
                m_featherAtlasFillPipeline: ManuallyDrop::new(None),
                m_featherAtlasStrokePipeline: ManuallyDrop::new(None),
                m_featherAtlasTexture: ManuallyDrop::new(None),
                m_imageSamplers: ManuallyDrop::new(core::array::from_fn(|_| None)),
                m_drawPipelines: ManuallyDrop::new(PipelineCache::default()),
                m_pathPatchVertexBuffer: ManuallyDrop::new(None),
                m_pathPatchIndexBuffer: ManuallyDrop::new(None),
                m_imageRectVertexBuffer: ManuallyDrop::new(None),
                m_imageRectIndexBuffer: ManuallyDrop::new(None),
                m_bufferRingLocks: ManuallyDrop::new(ring_state),
                m_bufferRingIdx: ManuallyDrop::new(0),
            };
            let source_device = (&*this.m_gpu).handle();
            for (sampler_index, &(wrap_x, wrap_y, filter)) in
                sampler_permutations.iter().enumerate()
            {
                let descriptor = metal
                    .call("MTLSamplerDescriptor", "new", vec![])
                    .unwrap_or(Handle::NIL);
                metal.owner_event("RC-SD-IMAGE-X18", OwnerEventPhase::Create, descriptor);
                let min_mag = if filter == u32::from(ImageFilter::bilinear.0) {
                    SamplerMinMagFilter::linear as u64
                } else {
                    SamplerMinMagFilter::nearest as u64
                };
                let address = |wrap| match wrap {
                    0 => SamplerAddressMode::clampToEdge as u64,
                    1 => SamplerAddressMode::repeat as u64,
                    2 => SamplerAddressMode::mirrorRepeat as u64,
                    _ => panic!("invalid ImageWrap key"),
                };
                set(
                    metal,
                    "samplerDescriptor",
                    "setMinFilter:",
                    vec![h(descriptor), u(min_mag)],
                );
                set(
                    metal,
                    "samplerDescriptor",
                    "setMagFilter:",
                    vec![h(descriptor), u(min_mag)],
                );
                set(
                    metal,
                    "samplerDescriptor",
                    "setMipFilter:",
                    vec![h(descriptor), u(SamplerMipFilter::nearest as u64)],
                );
                set(
                    metal,
                    "samplerDescriptor",
                    "setSAddressMode:",
                    vec![h(descriptor), u(address(wrap_x))],
                );
                set(
                    metal,
                    "samplerDescriptor",
                    "setTAddressMode:",
                    vec![h(descriptor), u(address(wrap_y))],
                );
                (&mut *this.m_imageSamplers)[sampler_index] = gpu_call(
                    metal,
                    source_device,
                    "newSamplerStateWithDescriptor:",
                    vec![h(descriptor)],
                )
                .and_then(|handle| metal.take_owned(handle, MetalObjectKind::SamplerState));
                // Keep the descriptor alive for the selector only; ARC
                // releases this source local at the end of the permutation
                // block, after the sampler state has been created.
                metal.owner_event(
                    "RC-SD-IMAGE-X18",
                    OwnerEventPhase::LastUse,
                    descriptor,
                );
                metal.retire_handle(descriptor);
                metal.owner_event("RC-SD-IMAGE-X18", OwnerEventPhase::Release, descriptor);
            }
            // The source initializer constructs every sampler before it
            // starts the background compiler worker.
            let background_shader_compiler = {
                #[cfg(target_vendor = "apple")]
                {
                    make_background_shader_compiler(&*this.m_gpu, metal_features)
                }
                #[cfg(not(target_vendor = "apple"))]
                {
                    None
                }
            };
            this.m_backgroundShaderCompiler = ManuallyDrop::new(background_shader_compiler);
            let data = metal
                .call(
                    "dispatch",
                    "dispatch_data_create",
                    vec![Value::Bytes(canonical_metallib())],
                )
                .unwrap_or(Handle::NIL);
            metal.owner_event("RC-DD-METALLIB", OwnerEventPhase::Create, data);
            let library_creation = gpu_call_with_error(
                metal,
                source_device,
                "newLibraryWithData:error:",
                vec![h(data)],
            );
            let library = library_creation.object;
            if let Some(error) = library_creation.error_owner_handle {
                metal.owner_event("RC-ERR-METALLIB", OwnerEventPhase::Create, error);
            }
            // The source member takes ownership before inspecting NSError;
            // an object+error pair therefore remains retained on early exit.
            this.m_plsPrecompiledLibrary = ManuallyDrop::new(
                metal.take_owned(library.unwrap_or(Handle::NIL), MetalObjectKind::Library),
            );
            if library_creation.has_error() || library.is_none() {
                metal.log(format!(
                    "RIVE: Failed to load pls metallib error: {}",
                    library_creation
                        .error_description()
                        .as_deref()
                        .unwrap_or("<nil>")
                ));
                if let Some(error) = library_creation.error_owner_handle {
                    metal.owner_event("RC-ERR-METALLIB", OwnerEventPhase::LastUse, error);
                    metal.retire_handle(error);
                    metal.owner_event("RC-ERR-METALLIB", OwnerEventPhase::Release, error);
                }
                // The source dispatchData local remains alive through the
                // library/member assignment and is released at this early
                // return boundary.
                // NSError is declared after dispatchData in the source and
                // therefore releases first on this early return.
                drop(library_creation);
                metal.owner_event("RC-DD-METALLIB", OwnerEventPhase::LastUse, data);
                metal.retire_handle(data);
                metal.owner_event("RC-DD-METALLIB", OwnerEventPhase::Release, data);
                return this;
            }
            let library = library.unwrap_or(Handle::NIL);
            this.m_colorRampPipeline = ManuallyDrop::new(Some(ColorRampPipeline::color_ramp(
                metal,
                source_device,
                library,
            )));
            let (gaussian_handle, gaussian_descriptor) =
                make_gaussian_texture(metal, source_device);
            this.m_gaussianIntegralTexture =
                ManuallyDrop::new(metal.take_owned(gaussian_handle, MetalObjectKind::Texture));
            let gaussian_handle = (&*this.m_gaussianIntegralTexture)
                .as_ref()
                .map(OwnedMetalHandle::handle)
                .unwrap_or(Handle::NIL);
            unsafe {
                let tables = [&crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::g_gaussianIntegralTableF16, &crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::g_inverseGaussianIntegralTableF16];
                for (slice, table) in tables.into_iter().enumerate() {
                    set(
                        metal,
                        "gaussianTexture",
                        "replaceRegion:mipmapLevel:slice:withBytes:bytesPerRow:bytesPerImage:",
                        vec![
                            h(gaussian_handle),
                            Value::Origin(Origin { x: 0, y: 0, z: 0 }),
                            Value::Size(Size {
                                width: table.len(),
                                height: 1,
                                depth: 1,
                            }),
                            u(0),
                            u(slice as u64),
                            bytes(table),
                            u(core::mem::size_of_val(table) as u64),
                            u(core::mem::size_of_val(table) as u64),
                        ],
                    );
                }
            }
            this.m_tessPipeline =
                ManuallyDrop::new(Some(TessellatePipeline::new(metal, source_device, library)));
            let tess_indices = &crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::kTessSpanIndices;
            let tess_index = gpu_call(
                metal,
                source_device,
                "newBufferWithBytes:length:options:",
                vec![
                    bytes(tess_indices),
                    u(core::mem::size_of_val(tess_indices) as u64),
                    u(0),
                ],
            );
            this.m_tessSpanIndexBuffer = ManuallyDrop::new(
                metal.take_owned(tess_index.unwrap_or(Handle::NIL), MetalObjectKind::Buffer),
            );
            if library != Handle::NIL && features.supportsRasterOrderingMode {
                for draw in [
                    DrawType::MidpointFanPatches,
                    DrawType::InteriorTriangulation,
                    DrawType::FeatherAtlasBlit,
                    DrawType::ImageMesh,
                ] {
                    for misc in [ShaderMiscFlags(0), ShaderMiscFlags::CLOCKWISE_FILL] {
                        if draw == DrawType::FeatherAtlasBlit && misc.0 != 0 {
                            continue;
                        }
                        let all =
                            ShaderFeatures(features_mask_for(draw, InterlockMode::RasterOrdering));
                        let key = shader_key(draw, all, InterlockMode::RasterOrdering, misc);
                        let vertex_name = precompiled_name(
                            metal,
                            "RC-NS-FUNCTION-NAME-V",
                            draw,
                            ShaderFeatures(all.0 & VERTEX_SHADER_FEATURES_MASK),
                            ShaderMiscFlags(0),
                            DRAW_VERTEX_NAME,
                        );
                        let fragment_name = precompiled_name(
                            metal,
                            "RC-NS-FUNCTION-NAME-F",
                            draw,
                            all,
                            misc,
                            DRAW_FRAGMENT_NAME,
                        );
                        this.m_drawPipelines.pipelines.insert(
                            key,
                            Some(DrawPipeline::new(
                                metal,
                                source_device,
                                Some(library),
                                vertex_name,
                                fragment_name,
                                draw,
                                InterlockMode::RasterOrdering,
                                all,
                                misc,
                                SynthesizedFailureType::none,
                            )),
                        );
                    }
                }
            }
            this.m_pathPatchVertexBuffer = ManuallyDrop::new(gpu_call(metal, source_device, "newBufferWithLength:options:", vec![u(crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::kPatchVertexBufferCount as u64 * core::mem::size_of::<crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::PatchVertex>() as u64), u(0)]).and_then(|handle| metal.take_owned(handle, MetalObjectKind::Buffer)));
            this.m_pathPatchIndexBuffer = ManuallyDrop::new(gpu_call(metal, source_device, "newBufferWithLength:options:", vec![u(crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::kPatchIndexBufferCount as u64 * core::mem::size_of::<u16>() as u64), u(0)]).and_then(|handle| metal.take_owned(handle, MetalObjectKind::Buffer)));
            // The source calls this even when either Objective-C allocation
            // returned nil; the nil contents path is an intentional quirk.
            metal.generate_patch_buffer_data(
                this.m_pathPatchVertexBuffer
                    .as_ref()
                    .map(OwnedMetalHandle::handle)
                    .unwrap_or(Handle::NIL),
                this.m_pathPatchIndexBuffer
                    .as_ref()
                    .map(OwnedMetalHandle::handle)
                    .unwrap_or(Handle::NIL),
            );
            let image_rect_vertices = &crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::kImageRectVertices;
            let image_rect_indices = &crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::kImageRectIndices;
            this.m_imageRectVertexBuffer = ManuallyDrop::new(
                (library != Handle::NIL)
                    .then(|| {
                        gpu_call(
                            metal,
                            source_device,
                            "newBufferWithBytes:length:options:",
                            vec![
                                bytes(image_rect_vertices),
                                u(core::mem::size_of_val(image_rect_vertices) as u64),
                                u(0),
                            ],
                        )
                        .and_then(|handle| metal.take_owned(handle, MetalObjectKind::Buffer))
                    })
                    .flatten(),
            );
            this.m_imageRectIndexBuffer = ManuallyDrop::new(
                (library != Handle::NIL)
                    .then(|| {
                        gpu_call(
                            metal,
                            source_device,
                            "newBufferWithBytes:length:options:",
                            vec![
                                bytes(image_rect_indices),
                                u(core::mem::size_of_val(image_rect_indices) as u64),
                                u(0),
                            ],
                        )
                        .and_then(|handle| metal.take_owned(handle, MetalObjectKind::Buffer))
                    })
                    .flatten(),
            );
            // The source Gaussian descriptor is a constructor local whose
            // lifetime spans the complete initializer, not merely the two
            // replaceRegion calls. Release it only after every later
            // pipeline and buffer assignment has completed.
            metal.owner_event(
                "RC-TD-GAUSSIAN",
                OwnerEventPhase::LastUse,
                gaussian_descriptor,
            );
            metal.retire_handle(gaussian_descriptor);
            metal.owner_event(
                "RC-TD-GAUSSIAN",
                OwnerEventPhase::Release,
                gaussian_descriptor,
            );
            // NSError follows the Gaussian descriptor in source declaration
            // order and must be released before the dispatch-data local.
            drop(library_creation);
            // dispatchData is a constructor local in the pinned source and
            // survives all successful resource construction.
            metal.owner_event("RC-DD-METALLIB", OwnerEventPhase::LastUse, data);
            metal.retire_handle(data);
            metal.owner_event("RC-DD-METALLIB", OwnerEventPhase::Release, data);
            this
        }

        pub fn make_command_buffer<E: MetalExecution>(&mut self, metal: &mut E) -> Option<Handle> {
            self.command_queue().and_then(|queue| {
                let command = metal.call(
                    "commandQueue",
                    "commandBuffer (__bridge_retained)",
                    vec![h(queue)],
                );
                if let Some(command) = command {
                    metal.owner_event("RC-CB-RETAINED", OwnerEventPhase::Create, command);
                }
                command
            })
        }
        pub fn commit_command_buffer<E: MetalExecution>(
            &mut self,
            metal: &mut E,
            token: Option<Handle>,
        ) {
            if let Some(command) = token {
                metal.owner_event("RC-CB-RETAINED", OwnerEventPhase::Transfer, command);
                metal.owner_event("RC-CB-TRANSFER", OwnerEventPhase::Transfer, command);
                metal.owner_event("RC-CB-TRANSFER", OwnerEventPhase::LastUse, command);
                set(
                    metal,
                    "commandBuffer",
                    "commit (__bridge_transfer)",
                    vec![h(command)],
                );
                metal.retire_handle(command);
                metal.owner_event("RC-CB-RETAINED", OwnerEventPhase::Release, command);
                metal.owner_event("RC-CB-TRANSFER", OwnerEventPhase::Release, command);
            }
        }
        /// Source `setCommandQueue` assigns the member immediately.  The
        /// outer mechanical adapter separately retains/retires the native
        /// queue handle, then calls this exact source seam.
        pub fn set_command_queue<E: MetalExecution>(
            &mut self,
            metal: &mut E,
            queue: Option<Handle>,
        ) {
            // `setCommandQueue` receives a borrowed source pointer.  The
            // canonical member assignment is a distinct strong retain, so a
            // self-assignment remains valid and the previous member is only
            // released after the replacement has been retained.
            let replacement = ManuallyDrop::new(
                queue.and_then(|handle| metal.clone_owned(handle, MetalObjectKind::CommandQueue)),
            );
            let mut old = core::mem::replace(&mut self.m_commandQueue, replacement);
            unsafe { ManuallyDrop::drop(&mut old) };
        }
        pub fn command_queue(&self) -> Option<Handle> {
            self.m_commandQueue.as_ref().map(OwnedMetalHandle::handle)
        }
        #[cfg(test)]
        pub(crate) fn seed_pipeline_for_test(&mut self, key: u32, pipeline: DrawPipeline) {
            self.m_drawPipelines.pipelines.insert(key, Some(pipeline));
        }
        #[cfg(test)]
        pub(crate) fn has_precompiled_library_for_test(&self) -> bool {
            self.m_plsPrecompiledLibrary.is_some()
        }
        #[cfg(test)]
        pub(crate) fn has_color_ramp_pipeline_for_test(&self) -> bool {
            self.m_colorRampPipeline.is_some()
        }
        #[cfg(test)]
        pub(crate) fn resized_texture_handle_for_test(
            &self,
            ledger_id: &str,
        ) -> Option<Handle> {
            match ledger_id {
                "RC-TD-GRAD-RESIZE" => self
                    .m_gradientTexture
                    .as_ref()
                    .map(OwnedMetalHandle::handle),
                "RC-TD-TESS-RESIZE" => self
                    .m_tessVertexTexture
                    .as_ref()
                    .map(OwnedMetalHandle::handle),
                "RC-TD-FEATHER-RESIZE" => self
                    .m_featherAtlasTexture
                    .as_ref()
                    .map(OwnedMetalHandle::handle),
                _ => None,
            }
        }
        #[cfg(test)]
        pub(crate) fn feather_pipelines_initialized_for_test(&self) -> bool {
            self.m_featherAtlasFillPipeline.is_some()
                && self.m_featherAtlasStrokePipeline.is_some()
        }
        pub fn make_render_target<E: MetalExecution>(
            &self,
            metal: &mut E,
            format: PixelFormat,
            width: u32,
            height: u32,
        ) -> RenderTargetMetal {
            let platform_features = source_platform_features(&self.base);
            let device = metal
                .clone_owned(self.m_gpu.handle(), MetalObjectKind::Device)
                .unwrap_or_else(|| OwnedMetalHandle::token(self.m_gpu.handle()));
            RenderTargetMetal::new_with_device(
                metal,
                device,
                format,
                width,
                height,
                platform_features,
            )
        }
        pub fn make_render_buffer<E: MetalExecution>(
            &self,
            metal: &mut E,
            buffer_type: RenderBufferType,
            flags: RenderBufferFlags,
            size: usize,
            mapped_once_at_initialization: bool,
        ) -> RenderBufferMetal {
            let device = metal
                .clone_owned(self.m_gpu.handle(), MetalObjectKind::Device)
                .unwrap_or_else(|| OwnedMetalHandle::token(self.m_gpu.handle()));
            RenderBufferMetal::new::<E>(
                metal,
                device,
                buffer_type,
                flags,
                size,
                mapped_once_at_initialization,
            )
        }
        pub fn adopt_image_texture<E: MetalExecution>(
            &mut self,
            metal: &mut E,
            texture: Option<Handle>,
            width: u32,
            height: u32,
        ) -> Option<TextureMetal> {
            let owner = texture
                .filter(|handle| *handle != Handle::NIL)
                .and_then(|handle| metal.take_owned(handle, MetalObjectKind::Texture));
            TextureMetal::adopt(owner, width, height)
        }
        pub fn prepare_to_flush(&mut self) {
            let next = (*self.m_bufferRingIdx + 1) % 3;
            self.m_bufferRingIdx = ManuallyDrop::new(next);
            let state = &(&*self.m_bufferRingLocks)[*self.m_bufferRingIdx as usize];
            // Source std::mutex::lock blocks until the previous command's
            // completion callback unlocks this independent ring slot.
            state.mutex.lock();
        }

        /// Rust-unwind safety adapter for the raw source flush seam.
        ///
        /// The pinned code cannot unwind between `prepareToFlush()` and
        /// `postFlush()`, but a Rust host/executor can. In that case no Metal
        /// completion owns this slot yet, so the adapter must restore the
        /// exact selected ring before the pinned context may be destroyed.
        /// `try_lock` also covers an unwind before `prepareToFlush`: it
        /// acquires the available slot and the following unlock is neutral.
        pub(crate) fn abort_unarmed_flush_after_unwind(&self) {
            let ring = *self.m_bufferRingIdx as usize;
            let state = &(&*self.m_bufferRingLocks)[ring];
            let _was_available = state.mutex.try_lock();
            unsafe { state.mutex.unlock() };
        }
        fn replace_texture(
            destination: &mut Option<OwnedMetalHandle>,
            replacement: Option<OwnedMetalHandle>,
        ) {
            *destination = replacement;
        }
        pub fn resize_gradient<E: MetalExecution>(
            &mut self,
            metal: &mut E,
            width: u32,
            height: u32,
        ) {
            let (texture, descriptor) = if width == 0 || height == 0 {
                (Handle::NIL, Handle::NIL)
            } else {
                make_resize_texture(
                    metal,
                    (&*self.m_gpu).handle(),
                    "RC-TD-GRAD-RESIZE",
                    PixelFormat::RGBA8Unorm,
                    width,
                    height,
                )
            };
            let replacement = metal.take_owned(texture, MetalObjectKind::Texture);
            Self::replace_texture(&mut self.m_gradientTexture, replacement);
            metal.owner_event(
                "RC-TD-GRAD-RESIZE",
                OwnerEventPhase::LastUse,
                descriptor,
            );
            metal.retire_handle(descriptor);
            metal.owner_event("RC-TD-GRAD-RESIZE", OwnerEventPhase::Release, descriptor);
        }
        pub fn resize_tessellation<E: MetalExecution>(
            &mut self,
            metal: &mut E,
            width: u32,
            height: u32,
        ) {
            let (texture, descriptor) = if width == 0 || height == 0 {
                (Handle::NIL, Handle::NIL)
            } else {
                make_resize_texture(
                    metal,
                    (&*self.m_gpu).handle(),
                    "RC-TD-TESS-RESIZE",
                    PixelFormat::RGBA32Uint,
                    width,
                    height,
                )
            };
            let replacement = metal.take_owned(texture, MetalObjectKind::Texture);
            Self::replace_texture(&mut self.m_tessVertexTexture, replacement);
            metal.owner_event(
                "RC-TD-TESS-RESIZE",
                OwnerEventPhase::LastUse,
                descriptor,
            );
            metal.retire_handle(descriptor);
            metal.owner_event("RC-TD-TESS-RESIZE", OwnerEventPhase::Release, descriptor);
        }
        pub fn resize_feather<E: MetalExecution>(
            &mut self,
            metal: &mut E,
            width: u32,
            height: u32,
        ) {
            let (texture, descriptor) = if width == 0 || height == 0 {
                (Handle::NIL, Handle::NIL)
            } else {
                make_resize_texture(
                    metal,
                    (&*self.m_gpu).handle(),
                    "RC-TD-FEATHER-RESIZE",
                    PixelFormat::R16Float,
                    width,
                    height,
                )
            };
            let replacement = metal.take_owned(texture, MetalObjectKind::Texture);
            Self::replace_texture(&mut self.m_featherAtlasTexture, replacement);
            if width == 0 || height == 0 {
                metal.retire_handle(descriptor);
                metal.owner_event("RC-TD-FEATHER-RESIZE", OwnerEventPhase::Release, descriptor);
                return;
            }
            debug_assert_eq!(
                self.m_featherAtlasFillPipeline.is_none(),
                self.m_featherAtlasStrokePipeline.is_none()
            );
            if self.m_featherAtlasFillPipeline.is_none() {
                let library = self
                    .m_plsPrecompiledLibrary
                    .as_ref()
                    .map(OwnedMetalHandle::handle)
                    .unwrap_or(Handle::NIL);
                self.m_featherAtlasFillPipeline = ManuallyDrop::new(Some(
                    FeatherAtlasPipeline::new(
                        metal,
                        (&*self.m_gpu).handle(),
                        library,
                        ATLAS_FILL_FRAGMENT_NAME,
                        false,
                    ),
                ));
                self.m_featherAtlasStrokePipeline = ManuallyDrop::new(Some(
                    FeatherAtlasPipeline::new(
                        metal,
                        (&*self.m_gpu).handle(),
                        library,
                        ATLAS_STROKE_FRAGMENT_NAME,
                        true,
                    ),
                ));
            }
            // Feather resource descriptors remain live through lazy pipeline
            // construction, as in the source function scope.
            metal.owner_event(
                "RC-TD-FEATHER-RESIZE",
                OwnerEventPhase::LastUse,
                descriptor,
            );
            metal.retire_handle(descriptor);
            metal.owner_event("RC-TD-FEATHER-RESIZE", OwnerEventPhase::Release, descriptor);
        }

        fn ring_slot_mut(
            &mut self,
            name: &'static str,
        ) -> &mut ManuallyDrop<Option<Box<dyn BufferRingContract>>> {
            let base: &mut RenderContextHelperImpl = unsafe {
                &mut *(&mut self.base as *mut ManuallyDrop<RenderContextHelperImpl>
                    as *mut RenderContextHelperImpl)
            };
            match name {
                "flushUniform" => &mut base.m_flushUniformBuffer,
                "path" => &mut base.m_pathBuffer,
                "paint" => &mut base.m_paintBuffer,
                "paintAux" => &mut base.m_paintAuxBuffer,
                "contour" => &mut base.m_contourBuffer,
                "gradSpan" => &mut base.m_gradSpanBuffer,
                "tessSpan" => &mut base.m_tessSpanBuffer,
                "triangle" => &mut base.m_triangleBuffer,
                "imageDrawInstance" => &mut base.m_imageDrawInstanceBuffer,
                _ => panic!("unknown source buffer ring"),
            }
        }

        fn ring_slot(
            &self,
            name: &'static str,
        ) -> &ManuallyDrop<Option<Box<dyn BufferRingContract>>> {
            let base: &RenderContextHelperImpl = unsafe {
                &*(&self.base as *const ManuallyDrop<RenderContextHelperImpl>
                    as *const RenderContextHelperImpl)
            };
            match name {
                "flushUniform" => &base.m_flushUniformBuffer,
                "path" => &base.m_pathBuffer,
                "paint" => &base.m_paintBuffer,
                "paintAux" => &base.m_paintAuxBuffer,
                "contour" => &base.m_contourBuffer,
                "gradSpan" => &base.m_gradSpanBuffer,
                "tessSpan" => &base.m_tessSpanBuffer,
                "triangle" => &base.m_triangleBuffer,
                "imageDrawInstance" => &base.m_imageDrawInstanceBuffer,
                _ => panic!("unknown source buffer ring"),
            }
        }

        fn replace_buffer_ring<E: MetalExecution>(
            &mut self,
            metal: &mut E,
            name: &'static str,
            capacity: usize,
        ) {
            let replacement = BufferRingMetal::make(metal, (&*self.m_gpu).handle(), capacity);
            let slot = self.ring_slot_mut(name);
            unsafe { ManuallyDrop::drop(slot) };
            *slot = ManuallyDrop::new(
                replacement.map(|ring| Box::new(ring) as Box<dyn BufferRingContract>),
            );
        }
        pub fn make_uniform_buffer_ring<E: MetalExecution>(
            &mut self,
            metal: &mut E,
            name: &'static str,
            capacity: usize,
        ) {
            self.replace_buffer_ring(metal, name, capacity);
        }
        pub fn make_storage_buffer_ring<E: MetalExecution>(
            &mut self,
            metal: &mut E,
            name: &'static str,
            capacity: usize,
        ) {
            self.replace_buffer_ring(metal, name, capacity);
        }
        pub fn make_vertex_buffer_ring<E: MetalExecution>(
            &mut self,
            metal: &mut E,
            name: &'static str,
            capacity: usize,
        ) {
            self.replace_buffer_ring(metal, name, capacity);
        }

        #[cfg(feature = "native-ore-metal-experimental")]
        pub fn make_render_canvas<E: MetalExecution>(
            &mut self,
            metal: &mut E,
            width: u32,
            height: u32,
        ) -> Option<(TextureMetal, RenderTargetMetal, Handle)> {
            let (texture, texture_descriptor) =
                make_canvas_texture(metal, (&*self.m_gpu).handle(), width, height);
            metal.owner_event("RC-TEX-CANVAS-LOCAL", OwnerEventPhase::Create, texture);
            let device = metal
                .clone_owned(self.m_gpu.handle(), MetalObjectKind::Device)
                .unwrap_or_else(|| OwnedMetalHandle::token(self.m_gpu.handle()));
            let mut target = RenderTargetMetal::new_with_device(
                metal,
                device,
                PixelFormat::RGBA8Unorm,
                width,
                height,
                source_platform_features(&self.base),
            );
            target.set_target_texture(
                metal,
                (texture.kind != MetalObjectKind::Nil).then_some(texture),
            );
            if let Some(target_texture) = target.m_targetTexture.as_ref() {
                metal.owner_event(
                    "RC-TEX-CANVAS-LOCAL",
                    OwnerEventPhase::CloneToTarget,
                    target_texture.handle(),
                );
            }
            // The native texture local is distinct from both source owners:
            // allocation (1) -> target assignment (2) -> image assignment
            // (3) -> local registry release (2).  Do not transfer the
            // original handle directly into the image, which collapses the
            // source ARC scope and leaves no separate local to release.
            let image_texture = metal
                .clone_owned(texture, MetalObjectKind::Texture)
                .unwrap_or_else(|| OwnedMetalHandle::token(texture));
            metal.owner_event(
                "RC-TEX-CANVAS-LOCAL",
                OwnerEventPhase::CloneToImage,
                image_texture.handle(),
            );
            metal.retire_handle(texture);
            metal.owner_event(
                "RC-TEX-CANVAS-LOCAL",
                OwnerEventPhase::ReleaseLocal,
                texture,
            );
            let image = TextureMetal::from_native(image_texture, width, height);
            // Carry the source descriptor through the outer RenderCanvas
            // construction. The mechanical adapter releases it only after
            // RenderCanvas::new has consumed both complete source owners.
            Some((image, target, texture_descriptor))
        }

        #[cfg(feature = "native-ore-metal-experimental")]
        pub fn make_ore_context<E: MetalExecution>(&mut self, metal: &mut E) -> Option<Handle> {
            let queue = self
                .m_commandQueue
                .as_mut()
                .and_then(|queue| metal.publish_owned(queue))
                .unwrap_or(Handle::NIL);
            debug_assert_ne!(queue, Handle::NIL, "m_commandQueue");
            let device = metal.publish_owned(&mut *self.m_gpu)?;
            metal.make_ore_context(device, Some(queue))
        }

        fn compiler_job(
            draw_type: DrawType,
            features: ShaderFeatures,
            interlock: InterlockMode,
            misc: ShaderMiscFlags,
            synthesized_failure: SynthesizedFailureType,
        ) -> BackgroundCompileJob {
            let mut compiled = BackgroundCompileJob::new(draw_type, features, interlock, misc);
            #[cfg(feature = "with-rive-tools")]
            {
                compiled.synthesizedFailureType = synthesized_failure;
            }
            compiled
        }

        fn push_source_compile_job(
            &self,
            draw_type: DrawType,
            features: ShaderFeatures,
            interlock: InterlockMode,
            misc: ShaderMiscFlags,
            synthesized_failure: SynthesizedFailureType,
        ) {
            #[cfg(target_vendor = "apple")]
            if let Some(compiler) = self.m_backgroundShaderCompiler.as_ref() {
                compiler.pushJob(Self::compiler_job(
                    draw_type,
                    features,
                    interlock,
                    misc,
                    synthesized_failure,
                ));
            }
        }

        fn pop_source_compile_job<E: MetalExecution>(
            &mut self,
            metal: &mut E,
            wait: bool,
        ) -> Option<(BackgroundCompileJob, Option<Handle>)> {
            #[cfg(target_vendor = "apple")]
            if let Some(compiler) = self.m_backgroundShaderCompiler.as_ref() {
                let mut compiled = BackgroundCompileJob::new(
                    DrawType::MidpointFanPatches,
                    ShaderFeatures::NONE,
                    InterlockMode::rasterOrdering,
                    ShaderMiscFlags::none,
                );
                if compiler.popFinishedJob(&mut compiled, wait) {
                    let library = compiled
                        .take_compiled_library_raw()
                        .and_then(|raw| unsafe { metal.adopt_compiled_library(raw) });
                    return Some((compiled, library));
                }
            }
            let _ = (metal, wait);
            None
        }

        pub(crate) fn find_compatible_pipeline<E: MetalExecution>(
            &mut self,
            metal: &mut E,
            draw: DrawType,
            mut features: ShaderFeatures,
            interlock: InterlockMode,
            misc: ShaderMiscFlags,
            fully_featured: ShaderFeatures,
            synthesized_failure: SynthesizedFailureType,
        ) -> Option<&DrawPipeline> {
            if self.m_contextOptions.shaderCompilationMode as i32 == crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::ShaderCompilationMode::onlyUbershaders as i32 { features = fully_featured; }
            if synthesized_failure == SynthesizedFailureType::ubershaderLoad {
                return None;
            }
            let key = shader_key(draw, features, interlock, misc);
            if !self.m_drawPipelines.pipelines.contains_key(&key) {
                self.m_drawPipelines.pipelines.insert(key, None);
                self.push_source_compile_job(draw, features, interlock, misc, synthesized_failure);
            }
            if self
                .m_drawPipelines
                .pipelines
                .get(&key)
                .is_some_and(Option::is_none)
            {
                // Pinned source preloads fully-featured raster-ordering
                // pipelines from the static library; reaching compilation
                // here for that exact key is a debug-only invariant.
                debug_assert!(
                    features != fully_featured || interlock != InterlockMode::RasterOrdering
                );
                let should_wait = features == fully_featured || self.m_contextOptions.shaderCompilationMode as i32 != crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::ShaderCompilationMode::allowAsynchronous as i32;
                while let Some((job, library)) = self.pop_source_compile_job(metal, should_wait) {
                    let job_key = shader_key(
                        job.drawType,
                        job.shaderFeatures,
                        job.interlockMode,
                        job.shaderMiscFlags,
                    );
                    #[cfg(feature = "with-rive-tools")]
                    let job_failure = job.synthesizedFailureType;
                    #[cfg(not(feature = "with-rive-tools"))]
                    let job_failure = SynthesizedFailureType::none;
                    let pipeline = DrawPipeline::new(
                        metal,
                        (&*self.m_gpu).handle(),
                        library,
                        SourceFunctionName::Static(DRAW_VERTEX_NAME),
                        SourceFunctionName::Static(DRAW_FRAGMENT_NAME),
                        job.drawType,
                        job.interlockMode,
                        job.shaderFeatures,
                        job.shaderMiscFlags,
                        job_failure,
                    );
                    self.m_drawPipelines
                        .pipelines
                        .insert(job_key, Some(pipeline));
                    if let Some(library) = library {
                        metal.retire_handle(library);
                    }
                    if job_key == key {
                        break;
                    }
                }
            }
            let pipeline_valid = self
                .m_drawPipelines
                .pipelines
                .get(&key)
                .and_then(Option::as_ref)
                .is_some_and(DrawPipeline::valid);
            if !pipeline_valid && features != fully_featured {
                return self.find_compatible_pipeline(
                    metal,
                    draw,
                    fully_featured,
                    interlock,
                    misc,
                    fully_featured,
                    synthesized_failure,
                );
            }
            self.m_drawPipelines
                .pipelines
                .get(&key)
                .and_then(Option::as_ref)
        }

        fn ring_buffer(&self, name: &'static str) -> Handle {
            let slot = self.ring_slot(name);
            unsafe { (&*slot).as_ref() }
                .and_then(|ring| ring.submittedHandle())
                .unwrap_or_else(|| super::rive_unreachable())
        }

        pub fn map_buffer_ring(
            &mut self,
            name: &'static str,
            size: usize,
        ) -> *mut core::ffi::c_void {
            let slot = self.ring_slot_mut(name);
            unsafe { (&mut *slot).as_mut() }
                .map(|ring| ring.mapBuffer(size))
                .unwrap_or(core::ptr::null_mut())
        }

        pub fn unmap_buffer_ring(&mut self, name: &'static str) {
            let slot = self.ring_slot_mut(name);
            if let Some(ring) = unsafe { (&mut *slot).as_mut() } {
                ring.unmapAndSubmitBuffer();
            }
        }

        fn begin_draw_pass<E: MetalExecution>(
            &self,
            metal: &mut E,
            desc: &gpu::FlushDescriptor,
            target: &mut RenderTargetMetal,
            command: Handle,
            pass: Handle,
            baseline: ShaderMiscFlags,
        ) -> Handle {
            let encoder = metal
                .call(
                    "commandBuffer",
                    "renderCommandEncoderWithDescriptor:",
                    vec![h(command), h(pass)],
                )
                .unwrap_or(Handle::NIL);
            metal.owner_event("RC-RPD-MAIN", OwnerEventPhase::LastUse, pass);
            metal.owner_event("RC-ENC-HELPER", OwnerEventPhase::Create, encoder);
            set(
                metal,
                "encoder",
                "setViewport:",
                vec![
                    h(encoder),
                    Value::Viewport(Viewport {
                        origin_x: 0.0,
                        origin_y: 0.0,
                        width: target.base.width() as f64,
                        height: target.base.height() as f64,
                        znear: 0.0,
                        zfar: 1.0,
                    }),
                ],
            );
            let flush = self.ring_buffer("flushUniform");
            set(
                metal,
                "encoder",
                "setVertexBuffer:offset:atIndex:",
                vec![
                    h(encoder),
                    h(flush),
                    u(desc.flushUniformDataOffsetInBytes as u64),
                    u(FLUSH_UNIFORM_BUFFER_IDX),
                ],
            );
            set(
                metal,
                "encoder",
                "setFragmentBuffer:offset:atIndex:",
                vec![
                    h(encoder),
                    h(flush),
                    u(desc.flushUniformDataOffsetInBytes as u64),
                    u(FLUSH_UNIFORM_BUFFER_IDX),
                ],
            );
            set(
                metal,
                "encoder",
                "setVertexTexture:atIndex:",
                vec![
                    h(encoder),
                    self.m_tessVertexTexture
                        .as_ref()
                        .map(|owner| h(owner.handle()))
                        .unwrap_or(Value::Nil),
                    u(TESS_VERTEX_TEXTURE_IDX),
                ],
            );
            set(
                metal,
                "encoder",
                "setVertexTexture:atIndex:",
                vec![
                    h(encoder),
                    self.m_gaussianIntegralTexture
                        .as_ref()
                        .map(|owner| h(owner.handle()))
                        .unwrap_or(Value::Nil),
                    u(GAUSSIAN_INTEGRAL_TEXTURE_IDX),
                ],
            );
            set(
                metal,
                "encoder",
                "setFragmentTexture:atIndex:",
                vec![
                    h(encoder),
                    self.m_gradientTexture
                        .as_ref()
                        .map(|owner| h(owner.handle()))
                        .unwrap_or(Value::Nil),
                    u(GRAD_TEXTURE_IDX),
                ],
            );
            set(
                metal,
                "encoder",
                "setFragmentTexture:atIndex:",
                vec![
                    h(encoder),
                    self.m_gaussianIntegralTexture
                        .as_ref()
                        .map(|owner| h(owner.handle()))
                        .unwrap_or(Value::Nil),
                    u(GAUSSIAN_INTEGRAL_TEXTURE_IDX),
                ],
            );
            set(
                metal,
                "encoder",
                "setFragmentTexture:atIndex:",
                vec![
                    h(encoder),
                    self.m_featherAtlasTexture
                        .as_ref()
                        .map(|owner| h(owner.handle()))
                        .unwrap_or(Value::Nil),
                    u(FEATHER_ATLAS_TEXTURE_IDX),
                ],
            );
            if desc.pathCount > 0 {
                set(
                    metal,
                    "encoder",
                    "setVertexBuffer:offset:atIndex:",
                    vec![
                        h(encoder),
                        h(self.ring_buffer("path")),
                        u(desc.firstPath as u64 * core::mem::size_of::<gpu::PathData>() as u64),
                        u(PATH_BUFFER_IDX),
                    ],
                );
                let paint_stage = if desc.interlockMode == InterlockMode::Atomics {
                    "setFragmentBuffer:offset:atIndex:"
                } else {
                    "setVertexBuffer:offset:atIndex:"
                };
                set(
                    metal,
                    "encoder",
                    paint_stage,
                    vec![
                        h(encoder),
                        h(self.ring_buffer("paint")),
                        u(desc.firstPaint as u64 * core::mem::size_of::<gpu::PaintData>() as u64),
                        u(PAINT_BUFFER_IDX),
                    ],
                );
                set(
                    metal,
                    "encoder",
                    paint_stage,
                    vec![
                        h(encoder),
                        h(self.ring_buffer("paintAux")),
                        u(desc.firstPaintAux as u64
                            * core::mem::size_of::<gpu::PaintAuxData>() as u64),
                        u(PAINT_AUX_BUFFER_IDX),
                    ],
                );
            }
            if desc.contourCount > 0 {
                set(
                    metal,
                    "encoder",
                    "setVertexBuffer:offset:atIndex:",
                    vec![
                        h(encoder),
                        h(self.ring_buffer("contour")),
                        u(desc.firstContour as u64
                            * core::mem::size_of::<gpu::ContourData>() as u64),
                        u(CONTOUR_BUFFER_IDX),
                    ],
                );
            }
            if desc.interlockMode == InterlockMode::Atomics {
                if !baseline.has(ShaderMiscFlags::FIXED_FUNCTION_COLOR_OUTPUT) {
                    let buffer = target.color_atomic_buffer_handle(metal);
                    set(
                        metal,
                        "encoder",
                        "setFragmentBuffer:offset:atIndex:",
                        vec![
                            h(encoder),
                            buffer.map(h).unwrap_or(Value::Nil),
                            u(0),
                            u(COLOR_ATOMIC_BUFFER_IDX),
                        ],
                    );
                }
                let clip = target.clip_atomic_buffer_handle(metal);
                set(
                    metal,
                    "encoder",
                    "setFragmentBuffer:offset:atIndex:",
                    vec![
                        h(encoder),
                        clip.map(h).unwrap_or(Value::Nil),
                        u(0),
                        u(CLIP_ATOMIC_BUFFER_IDX),
                    ],
                );
                let coverage = target.coverage_atomic_buffer_handle(metal);
                set(
                    metal,
                    "encoder",
                    "setFragmentBuffer:offset:atIndex:",
                    vec![
                        h(encoder),
                        coverage.map(h).unwrap_or(Value::Nil),
                        u(0),
                        u(COVERAGE_ATOMIC_BUFFER_IDX),
                    ],
                );
            }
            if desc.wireframe {
                set(
                    metal,
                    "encoder",
                    "setTriangleFillMode:",
                    vec![h(encoder), u(MTL_TRIANGLE_FILL_MODE_LINES)],
                );
            }
            metal.owner_event("RC-ENC-HELPER", OwnerEventPhase::Transfer, encoder);
            encoder
        }

        /// # Safety
        /// `desc` and every linked owner it references must remain live and
        /// synchronously accessible for the duration of this call. This is
        /// the source `const FlushDescriptor&` contract; the generic caller
        /// owns the flush arena and draw-list lifetime.
        pub unsafe fn flush<E: MetalExecution>(
            &mut self,
            metal: &mut E,
            desc: &gpu::FlushDescriptor,
            target: &mut RenderTargetMetal,
            command: Handle,
        ) {
            debug_assert!(!matches!(
                desc.interlockMode,
                InterlockMode::Clockwise | InterlockMode::ClockwiseAtomic | InterlockMode::Msaa
            ));

            // The source `__bridge` command-buffer local is a strong local
            // for the complete flush scope. Keep an explicit transferred
            // lease alive here; the selector adapter only borrows it.
            let command_owner = metal.clone_owned(command, MetalObjectKind::CommandBuffer);
            let command = command_owner
                .as_ref()
                .map(OwnedMetalHandle::handle)
                .unwrap_or(command);
            if command_owner.is_some() {
                metal.owner_event(
                    "RC-CB-FLUSH-STRONG",
                    OwnerEventPhase::CreateClone,
                    command,
                );
            }

            if desc.gradSpanCount > 0 {
                let Some(pipeline_handle) = self
                    .m_colorRampPipeline
                    .as_ref()
                    .and_then(|pipeline| pipeline.state.as_ref().map(OwnedMetalHandle::handle))
                else {
                    return;
                };
                let pipeline_owner =
                    metal.clone_owned(pipeline_handle, MetalObjectKind::RenderPipelineState);
                if let Some(owner) = pipeline_owner.as_ref() {
                    metal.owner_event(
                        "RC-PS-GRAD",
                        OwnerEventPhase::CreateClone,
                        owner.handle(),
                    );
                }
                let pipeline = pipeline_owner
                    .as_ref()
                    .map(OwnedMetalHandle::handle)
                    .unwrap_or(pipeline_handle);
                let pass = metal
                    .call("MTLRenderPassDescriptor", "renderPassDescriptor", vec![])
                    .unwrap_or(Handle::NIL);
                metal.owner_event("RC-RPD-GRAD", OwnerEventPhase::Create, pass);
                set(
                    metal,
                    "gradPass",
                    "setRenderTargetWidth:",
                    vec![h(pass), u(gpu::kGradTextureWidth)],
                );
                set(
                    metal,
                    "gradPass",
                    "setRenderTargetHeight:",
                    vec![h(pass), u(desc.gradDataHeight)],
                );
                let attachment = pass_attachment(metal, pass, 0, "RC-RPA-GRAD-0");
                set(
                    metal,
                    "gradAttachment",
                    "setLoadAction:",
                    vec![h(attachment), u(MTL_LOAD_ACTION_DONT_CARE)],
                );
                metal.owner_event("RC-RPA-GRAD-0", OwnerEventPhase::LastUse, attachment);
                metal.retire_handle(attachment);
                metal.owner_event("RC-RPA-GRAD-0", OwnerEventPhase::AliasEnd, attachment);
                let attachment = pass_attachment(metal, pass, 0, "RC-RPA-GRAD-0");
                set(
                    metal,
                    "gradAttachment",
                    "setStoreAction:",
                    vec![h(attachment), u(MTL_STORE_ACTION_STORE)],
                );
                metal.owner_event("RC-RPA-GRAD-0", OwnerEventPhase::LastUse, attachment);
                metal.retire_handle(attachment);
                metal.owner_event("RC-RPA-GRAD-0", OwnerEventPhase::AliasEnd, attachment);
                let attachment = pass_attachment(metal, pass, 0, "RC-RPA-GRAD-0");
                set(
                    metal,
                    "gradAttachment",
                    "setTexture:",
                    vec![
                        h(attachment),
                        self.m_gradientTexture
                            .as_ref()
                            .map(|owner| h(owner.handle()))
                        .unwrap_or(Value::Nil),
                    ],
                );
                metal.owner_event("RC-RPA-GRAD-0", OwnerEventPhase::LastUse, attachment);
                metal.retire_handle(attachment);
                metal.owner_event("RC-RPA-GRAD-0", OwnerEventPhase::AliasEnd, attachment);
                let encoder = metal
                    .call(
                        "commandBuffer",
                        "renderCommandEncoderWithDescriptor:",
                        vec![h(command), h(pass)],
                    )
                    .unwrap_or(Handle::NIL);
                metal.owner_event("RC-RPD-GRAD", OwnerEventPhase::LastUse, pass);
                metal.owner_event("RC-ENC-GRAD", OwnerEventPhase::Create, encoder);
                set(
                    metal,
                    "gradEncoder",
                    "setViewport:",
                    vec![
                        h(encoder),
                        Value::Viewport(Viewport {
                            origin_x: 0.0,
                            origin_y: 0.0,
                            width: gpu::kGradTextureWidth as f64,
                            height: desc.gradDataHeight as f64,
                            znear: 0.0,
                            zfar: 1.0,
                        }),
                    ],
                );
                set(
                    metal,
                    "gradEncoder",
                    "setRenderPipelineState:",
                    vec![h(encoder), h(pipeline)],
                );
                set(
                    metal,
                    "gradEncoder",
                    "setVertexBuffer:offset:atIndex:",
                    vec![
                        h(encoder),
                        h(self.ring_buffer("flushUniform")),
                        u(desc.flushUniformDataOffsetInBytes as u64),
                        u(FLUSH_UNIFORM_BUFFER_IDX),
                    ],
                );
                set(
                    metal,
                    "gradEncoder",
                    "setVertexBuffer:offset:atIndex:",
                    vec![
                        h(encoder),
                        h(self.ring_buffer("gradSpan")),
                        u(desc.firstGradSpan as u64
                            * core::mem::size_of::<gpu::GradientSpan>() as u64),
                        u(0),
                    ],
                );
                set(
                    metal,
                    "gradEncoder",
                    "setCullMode:",
                    vec![h(encoder), u(MTL_CULL_MODE_BACK)],
                );
                set(
                    metal,
                    "gradEncoder",
                    "drawPrimitives:vertexStart:vertexCount:instanceCount:",
                    vec![
                        h(encoder),
                        u(MTL_PRIMITIVE_TYPE_TRIANGLE_STRIP),
                        u(0),
                        u(gpu::GRAD_SPAN_TRI_STRIP_VERTEX_COUNT),
                        u(desc.gradSpanCount),
                    ],
                );
                metal.owner_event("RC-ENC-GRAD", OwnerEventPhase::LastUse, encoder);
                set(metal, "gradEncoder", "endEncoding", vec![h(encoder)]);
                metal.retire_handle(encoder);
                metal.owner_event("RC-ENC-GRAD", OwnerEventPhase::Release, encoder);
                metal.retire_handle(pass);
                metal.owner_event("RC-RPD-GRAD", OwnerEventPhase::Release, pass);
                if let Some(pipeline_owner) = pipeline_owner {
                    let pipeline_handle = pipeline_owner.handle();
                    metal.owner_event(
                        "RC-PS-GRAD",
                        OwnerEventPhase::LastUse,
                        pipeline_handle,
                    );
                    drop(pipeline_owner);
                    metal.owner_event("RC-PS-GRAD", OwnerEventPhase::Release, pipeline_handle);
                }
            }

            if desc.tessVertexSpanCount > 0 {
                let Some(pipeline_handle) = self
                    .m_tessPipeline
                    .as_ref()
                    .and_then(|pipeline| pipeline.state.as_ref().map(OwnedMetalHandle::handle))
                else {
                    return;
                };
                let pipeline_owner =
                    metal.clone_owned(pipeline_handle, MetalObjectKind::RenderPipelineState);
                if let Some(owner) = pipeline_owner.as_ref() {
                    metal.owner_event(
                        "RC-PS-TESS",
                        OwnerEventPhase::CreateClone,
                        owner.handle(),
                    );
                }
                let pipeline = pipeline_owner
                    .as_ref()
                    .map(OwnedMetalHandle::handle)
                    .unwrap_or(pipeline_handle);
                let pass = metal
                    .call("MTLRenderPassDescriptor", "renderPassDescriptor", vec![])
                    .unwrap_or(Handle::NIL);
                metal.owner_event("RC-RPD-TESS", OwnerEventPhase::Create, pass);
                set(
                    metal,
                    "tessPass",
                    "setRenderTargetWidth:",
                    vec![h(pass), u(gpu::kTessTextureWidth)],
                );
                set(
                    metal,
                    "tessPass",
                    "setRenderTargetHeight:",
                    vec![h(pass), u(desc.tessDataHeight)],
                );
                let attachment = pass_attachment(metal, pass, 0, "RC-RPA-TESS-0");
                set(
                    metal,
                    "tessAttachment",
                    "setLoadAction:",
                    vec![h(attachment), u(MTL_LOAD_ACTION_DONT_CARE)],
                );
                metal.owner_event("RC-RPA-TESS-0", OwnerEventPhase::LastUse, attachment);
                metal.retire_handle(attachment);
                metal.owner_event("RC-RPA-TESS-0", OwnerEventPhase::AliasEnd, attachment);
                let attachment = pass_attachment(metal, pass, 0, "RC-RPA-TESS-0");
                set(
                    metal,
                    "tessAttachment",
                    "setStoreAction:",
                    vec![h(attachment), u(MTL_STORE_ACTION_STORE)],
                );
                metal.owner_event("RC-RPA-TESS-0", OwnerEventPhase::LastUse, attachment);
                metal.retire_handle(attachment);
                metal.owner_event("RC-RPA-TESS-0", OwnerEventPhase::AliasEnd, attachment);
                let attachment = pass_attachment(metal, pass, 0, "RC-RPA-TESS-0");
                set(
                    metal,
                    "tessAttachment",
                    "setTexture:",
                    vec![
                        h(attachment),
                        self.m_tessVertexTexture
                            .as_ref()
                            .map(|owner| h(owner.handle()))
                        .unwrap_or(Value::Nil),
                    ],
                );
                metal.owner_event("RC-RPA-TESS-0", OwnerEventPhase::LastUse, attachment);
                metal.retire_handle(attachment);
                metal.owner_event("RC-RPA-TESS-0", OwnerEventPhase::AliasEnd, attachment);
                let encoder = metal
                    .call(
                        "commandBuffer",
                        "renderCommandEncoderWithDescriptor:",
                        vec![h(command), h(pass)],
                    )
                    .unwrap_or(Handle::NIL);
                metal.owner_event("RC-RPD-TESS", OwnerEventPhase::LastUse, pass);
                metal.owner_event("RC-ENC-TESS", OwnerEventPhase::Create, encoder);
                set(
                    metal,
                    "tessEncoder",
                    "setViewport:",
                    vec![
                        h(encoder),
                        Value::Viewport(Viewport {
                            origin_x: 0.0,
                            origin_y: 0.0,
                            width: gpu::kTessTextureWidth as f64,
                            height: desc.tessDataHeight as f64,
                            znear: 0.0,
                            zfar: 1.0,
                        }),
                    ],
                );
                set(
                    metal,
                    "tessEncoder",
                    "setRenderPipelineState:",
                    vec![h(encoder), h(pipeline)],
                );
                set(
                    metal,
                    "tessEncoder",
                    "setVertexTexture:atIndex:",
                    vec![
                        h(encoder),
                        self.m_gaussianIntegralTexture
                            .as_ref()
                            .map(|owner| h(owner.handle()))
                            .unwrap_or(Value::Nil),
                        u(GAUSSIAN_INTEGRAL_TEXTURE_IDX),
                    ],
                );
                set(
                    metal,
                    "tessEncoder",
                    "setVertexBuffer:offset:atIndex:",
                    vec![
                        h(encoder),
                        h(self.ring_buffer("flushUniform")),
                        u(desc.flushUniformDataOffsetInBytes as u64),
                        u(FLUSH_UNIFORM_BUFFER_IDX),
                    ],
                );
                set(
                    metal,
                    "tessEncoder",
                    "setVertexBuffer:offset:atIndex:",
                    vec![
                        h(encoder),
                        h(self.ring_buffer("tessSpan")),
                        u(desc.firstTessVertexSpan as u64
                            * core::mem::size_of::<gpu::TessVertexSpan>() as u64),
                        u(0),
                    ],
                );
                debug_assert!(desc.pathCount > 0);
                debug_assert!(desc.contourCount > 0);
                set(
                    metal,
                    "tessEncoder",
                    "setVertexBuffer:offset:atIndex:",
                    vec![
                        h(encoder),
                        h(self.ring_buffer("path")),
                        u(desc.firstPath as u64 * core::mem::size_of::<gpu::PathData>() as u64),
                        u(PATH_BUFFER_IDX),
                    ],
                );
                set(
                    metal,
                    "tessEncoder",
                    "setVertexBuffer:offset:atIndex:",
                    vec![
                        h(encoder),
                        h(self.ring_buffer("contour")),
                        u(desc.firstContour as u64
                            * core::mem::size_of::<gpu::ContourData>() as u64),
                        u(CONTOUR_BUFFER_IDX),
                    ],
                );
                set(
                    metal,
                    "tessEncoder",
                    "setCullMode:",
                    vec![h(encoder), u(MTL_CULL_MODE_BACK)],
                );
                set(
                    metal,
                    "tessEncoder",
                    "drawIndexedPrimitives:indexCount:indexType:indexBuffer:indexBufferOffset:instanceCount:",
                    vec![
                        h(encoder),
                        u(MTL_PRIMITIVE_TYPE_TRIANGLE),
                        u(gpu::kTessSpanIndices.len() as u64),
                        u(MTL_INDEX_TYPE_UINT16),
                        self.m_tessSpanIndexBuffer
                            .as_ref()
                            .map(|owner| h(owner.handle()))
                            .unwrap_or(Value::Nil),
                        u(0),
                        u(desc.tessVertexSpanCount),
                    ],
                );
                metal.owner_event("RC-ENC-TESS", OwnerEventPhase::LastUse, encoder);
                set(metal, "tessEncoder", "endEncoding", vec![h(encoder)]);
                metal.retire_handle(encoder);
                metal.owner_event("RC-ENC-TESS", OwnerEventPhase::Release, encoder);
                metal.retire_handle(pass);
                metal.owner_event("RC-RPD-TESS", OwnerEventPhase::Release, pass);
                if let Some(pipeline_owner) = pipeline_owner {
                    let pipeline_handle = pipeline_owner.handle();
                    metal.owner_event(
                        "RC-PS-TESS",
                        OwnerEventPhase::LastUse,
                        pipeline_handle,
                    );
                    drop(pipeline_owner);
                    metal.owner_event("RC-PS-TESS", OwnerEventPhase::Release, pipeline_handle);
                }
            }

            if desc.featherAtlasFillBatchCount != 0 || desc.featherAtlasStrokeBatchCount != 0 {
                let Some(fill_state_handle) = self
                    .m_featherAtlasFillPipeline
                    .as_ref()
                    .and_then(|pipeline| pipeline.state.as_ref().map(OwnedMetalHandle::handle))
                else {
                    return;
                };
                let Some(stroke_state_handle) = self
                    .m_featherAtlasStrokePipeline
                    .as_ref()
                    .and_then(|pipeline| pipeline.state.as_ref().map(OwnedMetalHandle::handle))
                else {
                    return;
                };
                let fill_state_owner =
                    metal.clone_owned(fill_state_handle, MetalObjectKind::RenderPipelineState);
                let stroke_state_owner =
                    metal.clone_owned(stroke_state_handle, MetalObjectKind::RenderPipelineState);
                if let Some(owner) = fill_state_owner.as_ref() {
                    metal.owner_event(
                        "RC-PS-ATLAS-FILL",
                        OwnerEventPhase::CreateClone,
                        owner.handle(),
                    );
                }
                if let Some(owner) = stroke_state_owner.as_ref() {
                    metal.owner_event(
                        "RC-PS-ATLAS-STROKE",
                        OwnerEventPhase::CreateClone,
                        owner.handle(),
                    );
                }
                let fill_state = fill_state_owner
                    .as_ref()
                    .map(OwnedMetalHandle::handle)
                    .unwrap_or(fill_state_handle);
                let stroke_state = stroke_state_owner
                    .as_ref()
                    .map(OwnedMetalHandle::handle)
                    .unwrap_or(stroke_state_handle);
                let pass = metal
                    .call("MTLRenderPassDescriptor", "renderPassDescriptor", vec![])
                    .unwrap_or(Handle::NIL);
                metal.owner_event("RC-RPD-ATLAS", OwnerEventPhase::Create, pass);
                set(
                    metal,
                    "atlasPass",
                    "setRenderTargetWidth:",
                    vec![h(pass), u(desc.featherAtlasContentWidth)],
                );
                set(
                    metal,
                    "atlasPass",
                    "setRenderTargetHeight:",
                    vec![h(pass), u(desc.featherAtlasContentHeight)],
                );
                let attachment = pass_attachment(metal, pass, 0, "RC-RPA-ATLAS-0");
                set(
                    metal,
                    "atlasAttachment",
                    "setLoadAction:",
                    vec![h(attachment), u(MTL_LOAD_ACTION_CLEAR)],
                );
                metal.owner_event(
                    "RC-RPA-ATLAS-0",
                    OwnerEventPhase::LastUse,
                    attachment,
                );
                metal.retire_handle(attachment);
                metal.owner_event(
                    "RC-RPA-ATLAS-0",
                    OwnerEventPhase::AliasEnd,
                    attachment,
                );
                let attachment = pass_attachment(metal, pass, 0, "RC-RPA-ATLAS-0");
                set(
                    metal,
                    "atlasAttachment",
                    "setStoreAction:",
                    vec![h(attachment), u(MTL_STORE_ACTION_STORE)],
                );
                metal.owner_event(
                    "RC-RPA-ATLAS-0",
                    OwnerEventPhase::LastUse,
                    attachment,
                );
                metal.retire_handle(attachment);
                metal.owner_event(
                    "RC-RPA-ATLAS-0",
                    OwnerEventPhase::AliasEnd,
                    attachment,
                );
                let attachment = pass_attachment(metal, pass, 0, "RC-RPA-ATLAS-0");
                set(
                    metal,
                    "atlasAttachment",
                    "setTexture:",
                    vec![
                        h(attachment),
                        self.m_featherAtlasTexture
                            .as_ref()
                            .map(|owner| h(owner.handle()))
                        .unwrap_or(Value::Nil),
                    ],
                );
                metal.owner_event(
                    "RC-RPA-ATLAS-0",
                    OwnerEventPhase::LastUse,
                    attachment,
                );
                metal.retire_handle(attachment);
                metal.owner_event(
                    "RC-RPA-ATLAS-0",
                    OwnerEventPhase::AliasEnd,
                    attachment,
                );
                let attachment = pass_attachment(metal, pass, 0, "RC-RPA-ATLAS-0");
                set(
                    metal,
                    "atlasAttachment",
                    "setClearColor:",
                    vec![h(attachment), Value::ClearColor(ClearColor::default())],
                );
                metal.owner_event(
                    "RC-RPA-ATLAS-0",
                    OwnerEventPhase::LastUse,
                    attachment,
                );
                metal.retire_handle(attachment);
                metal.owner_event(
                    "RC-RPA-ATLAS-0",
                    OwnerEventPhase::AliasEnd,
                    attachment,
                );
                let encoder = metal
                    .call(
                        "commandBuffer",
                        "renderCommandEncoderWithDescriptor:",
                        vec![h(command), h(pass)],
                    )
                    .unwrap_or(Handle::NIL);
                metal.owner_event("RC-RPD-ATLAS", OwnerEventPhase::LastUse, pass);
                metal.owner_event("RC-ENC-ATLAS", OwnerEventPhase::Create, encoder);
                set(
                    metal,
                    "atlasEncoder",
                    "setViewport:",
                    vec![
                        h(encoder),
                        Value::Viewport(Viewport {
                            origin_x: 0.0,
                            origin_y: 0.0,
                            width: desc.featherAtlasContentWidth as f64,
                            height: desc.featherAtlasContentHeight as f64,
                            znear: 0.0,
                            zfar: 1.0,
                        }),
                    ],
                );
                set(
                    metal,
                    "atlasEncoder",
                    "setVertexBuffer:offset:atIndex:",
                    vec![
                        h(encoder),
                        h(self.ring_buffer("flushUniform")),
                        u(desc.flushUniformDataOffsetInBytes as u64),
                        u(FLUSH_UNIFORM_BUFFER_IDX),
                    ],
                );
                set(
                    metal,
                    "atlasEncoder",
                    "setFragmentBuffer:offset:atIndex:",
                    vec![
                        h(encoder),
                        h(self.ring_buffer("flushUniform")),
                        u(desc.flushUniformDataOffsetInBytes as u64),
                        u(FLUSH_UNIFORM_BUFFER_IDX),
                    ],
                );
                set(
                    metal,
                    "atlasEncoder",
                    "setVertexTexture:atIndex:",
                    vec![
                        h(encoder),
                        self.m_tessVertexTexture
                            .as_ref()
                            .map(|owner| h(owner.handle()))
                            .unwrap_or(Value::Nil),
                        u(TESS_VERTEX_TEXTURE_IDX),
                    ],
                );
                set(
                    metal,
                    "atlasEncoder",
                    "setVertexTexture:atIndex:",
                    vec![
                        h(encoder),
                        self.m_gaussianIntegralTexture
                            .as_ref()
                            .map(|owner| h(owner.handle()))
                            .unwrap_or(Value::Nil),
                        u(GAUSSIAN_INTEGRAL_TEXTURE_IDX),
                    ],
                );
                set(
                    metal,
                    "atlasEncoder",
                    "setFragmentTexture:atIndex:",
                    vec![
                        h(encoder),
                        self.m_gradientTexture
                            .as_ref()
                            .map(|owner| h(owner.handle()))
                            .unwrap_or(Value::Nil),
                        u(GRAD_TEXTURE_IDX),
                    ],
                );
                set(
                    metal,
                    "atlasEncoder",
                    "setFragmentTexture:atIndex:",
                    vec![
                        h(encoder),
                        self.m_gaussianIntegralTexture
                            .as_ref()
                            .map(|owner| h(owner.handle()))
                            .unwrap_or(Value::Nil),
                        u(GAUSSIAN_INTEGRAL_TEXTURE_IDX),
                    ],
                );
                if desc.pathCount > 0 {
                    set(
                        metal,
                        "atlasEncoder",
                        "setVertexBuffer:offset:atIndex:",
                        vec![
                            h(encoder),
                            h(self.ring_buffer("path")),
                            u(
                                desc.firstPath as u64
                                    * core::mem::size_of::<gpu::PathData>() as u64,
                            ),
                            u(PATH_BUFFER_IDX),
                        ],
                    );
                    set(
                        metal,
                        "atlasEncoder",
                        "setVertexBuffer:offset:atIndex:",
                        vec![
                            h(encoder),
                            h(self.ring_buffer("paint")),
                            u(desc.firstPaint as u64
                                * core::mem::size_of::<gpu::PaintData>() as u64),
                            u(PAINT_BUFFER_IDX),
                        ],
                    );
                    set(
                        metal,
                        "atlasEncoder",
                        "setVertexBuffer:offset:atIndex:",
                        vec![
                            h(encoder),
                            h(self.ring_buffer("paintAux")),
                            u(desc.firstPaintAux as u64
                                * core::mem::size_of::<gpu::PaintAuxData>() as u64),
                            u(PAINT_AUX_BUFFER_IDX),
                        ],
                    );
                }
                if desc.contourCount > 0 {
                    set(
                        metal,
                        "atlasEncoder",
                        "setVertexBuffer:offset:atIndex:",
                        vec![
                            h(encoder),
                            h(self.ring_buffer("contour")),
                            u(desc.firstContour as u64
                                * core::mem::size_of::<gpu::ContourData>() as u64),
                            u(CONTOUR_BUFFER_IDX),
                        ],
                    );
                }
                set(
                    metal,
                    "atlasEncoder",
                    "setVertexBuffer:offset:atIndex:",
                    vec![
                        h(encoder),
                        self.m_pathPatchVertexBuffer
                            .as_ref()
                            .map(|owner| h(owner.handle()))
                            .unwrap_or(Value::Nil),
                        u(0),
                        u(0),
                    ],
                );
                if desc.featherAtlasFillBatchCount != 0 {
                    set(
                        metal,
                        "atlasEncoder",
                        "setCullMode:",
                        vec![h(encoder), u(MTL_CULL_MODE_NONE)],
                    );
                    set(
                        metal,
                        "atlasEncoder",
                        "setRenderPipelineState:",
                        vec![h(encoder), h(fill_state)],
                    );
                    for batch in unsafe {
                        atlas_batches(
                            desc.featherAtlasFillBatches,
                            desc.featherAtlasFillBatchCount,
                        )
                    } {
                        set(
                            metal,
                            "atlasEncoder",
                            "setScissorRect:",
                            vec![h(encoder), scissor(source_scissor(batch.scissor))],
                        );
                        set(
                            metal,
                            "atlasEncoder",
                            "setVertexBytes:length:atIndex:",
                            vec![
                                h(encoder),
                                bytes(core::slice::from_ref(&batch.basePatch)),
                                u(core::mem::size_of::<u32>() as u64),
                                u(PATH_BASE_INSTANCE_UNIFORM_BUFFER_IDX),
                            ],
                        );
                        set(
                            metal,
                            "atlasEncoder",
                            "drawIndexedPrimitives:indexCount:indexType:indexBuffer:indexBufferOffset:instanceCount:",
                            vec![
                                h(encoder),
                                u(MTL_PRIMITIVE_TYPE_TRIANGLE),
                                u(gpu::kMidpointFanCenterAAPatchIndexCount),
                                u(MTL_INDEX_TYPE_UINT16),
                                self.m_pathPatchIndexBuffer
                                    .as_ref()
                                    .map(|owner| h(owner.handle()))
                                    .unwrap_or(Value::Nil),
                                u(gpu::kMidpointFanCenterAAPatchBaseIndex as u64
                                    * core::mem::size_of::<u16>() as u64),
                                u(batch.patchCount),
                            ],
                        );
                    }
                }
                if desc.featherAtlasStrokeBatchCount != 0 {
                    set(
                        metal,
                        "atlasEncoder",
                        "setCullMode:",
                        vec![h(encoder), u(MTL_CULL_MODE_BACK)],
                    );
                    set(
                        metal,
                        "atlasEncoder",
                        "setRenderPipelineState:",
                        vec![h(encoder), h(stroke_state)],
                    );
                    for batch in unsafe {
                        atlas_batches(
                            desc.featherAtlasStrokeBatches,
                            desc.featherAtlasStrokeBatchCount,
                        )
                    } {
                        set(
                            metal,
                            "atlasEncoder",
                            "setScissorRect:",
                            vec![h(encoder), scissor(source_scissor(batch.scissor))],
                        );
                        set(
                            metal,
                            "atlasEncoder",
                            "setVertexBytes:length:atIndex:",
                            vec![
                                h(encoder),
                                bytes(core::slice::from_ref(&batch.basePatch)),
                                u(core::mem::size_of::<u32>() as u64),
                                u(PATH_BASE_INSTANCE_UNIFORM_BUFFER_IDX),
                            ],
                        );
                        set(
                            metal,
                            "atlasEncoder",
                            "drawIndexedPrimitives:indexCount:indexType:indexBuffer:indexBufferOffset:instanceCount:",
                            vec![
                                h(encoder),
                                u(MTL_PRIMITIVE_TYPE_TRIANGLE),
                                u(gpu::kMidpointFanPatchBorderIndexCount),
                                u(MTL_INDEX_TYPE_UINT16),
                                self.m_pathPatchIndexBuffer
                                    .as_ref()
                                    .map(|owner| h(owner.handle()))
                                    .unwrap_or(Value::Nil),
                                u(gpu::kMidpointFanPatchBaseIndex as u64
                                    * core::mem::size_of::<u16>() as u64),
                                u(batch.patchCount),
                            ],
                        );
                    }
                }
                metal.owner_event("RC-ENC-ATLAS", OwnerEventPhase::LastUse, encoder);
                set(metal, "atlasEncoder", "endEncoding", vec![h(encoder)]);
                metal.retire_handle(encoder);
                metal.owner_event("RC-ENC-ATLAS", OwnerEventPhase::Release, encoder);
                metal.retire_handle(pass);
                metal.owner_event("RC-RPD-ATLAS", OwnerEventPhase::Release, pass);
                if let Some(stroke_state_owner) = stroke_state_owner {
                    let stroke_state_handle = stroke_state_owner.handle();
                    metal.owner_event(
                        "RC-PS-ATLAS-STROKE",
                        OwnerEventPhase::LastUse,
                        stroke_state_handle,
                    );
                    drop(stroke_state_owner);
                    metal.owner_event(
                        "RC-PS-ATLAS-STROKE",
                        OwnerEventPhase::Release,
                        stroke_state_handle,
                    );
                }
                if let Some(fill_state_owner) = fill_state_owner {
                    let fill_state_handle = fill_state_owner.handle();
                    metal.owner_event(
                        "RC-PS-ATLAS-FILL",
                        OwnerEventPhase::LastUse,
                        fill_state_handle,
                    );
                    drop(fill_state_owner);
                    metal.owner_event(
                        "RC-PS-ATLAS-FILL",
                        OwnerEventPhase::Release,
                        fill_state_handle,
                    );
                }
            }

            for batch in draw_batches(desc) {
                if let Some(texture) = batch.imageTexture {
                    // The raw DrawBatch pointer is nonowning. The unsafe
                    // flush contract supplies the complete source Texture
                    // lifetime for this synchronous traversal; no context-
                    // owned texture table or implicit retain is introduced.
                    unsafe {
                        texture
                            .as_ptr()
                            .cast::<TextureMetal>()
                            .as_ref()
                            .expect("retained Texture base must point to TextureMetal")
                            .ensure_mipmaps(metal, command);
                    }
                }
            }

            let pass = metal
                .call("MTLRenderPassDescriptor", "renderPassDescriptor", vec![])
                .unwrap_or(Handle::NIL);
            metal.owner_event("RC-RPD-MAIN", OwnerEventPhase::Create, pass);
            set(
                metal,
                "pass",
                "setRenderTargetWidth:",
                vec![h(pass), u(desc.renderTargetUpdateBounds.right)],
            );
            set(
                metal,
                "pass",
                "setRenderTargetHeight:",
                vec![h(pass), u(desc.renderTargetUpdateBounds.bottom)],
            );
            set_pass_attachment(
                metal,
                pass,
                0,
                "RC-RPA-MAIN-COLOR",
                "colorAttachment",
                "setTexture:",
                target.target_handle().map(h).unwrap_or(Value::Nil),
            );
            match desc.colorLoadAction {
                LoadAction::Clear => {
                    set_pass_attachment(
                        metal,
                        pass,
                        0,
                        "RC-RPA-MAIN-COLOR",
                        "colorAttachment",
                        "setLoadAction:",
                        u(MTL_LOAD_ACTION_CLEAR),
                    );
                    set_pass_attachment(
                        metal,
                        pass,
                        0,
                        "RC-RPA-MAIN-COLOR",
                        "colorAttachment",
                        "setClearColor:",
                        Value::ClearColor(ClearColor {
                            red: source_clear_color(desc.colorClearValue)[0],
                            green: source_clear_color(desc.colorClearValue)[1],
                            blue: source_clear_color(desc.colorClearValue)[2],
                            alpha: source_clear_color(desc.colorClearValue)[3],
                        }),
                    );
                }
                LoadAction::PreserveRenderTarget => set_pass_attachment(
                    metal,
                    pass,
                    0,
                    "RC-RPA-MAIN-COLOR",
                    "colorAttachment",
                    "setLoadAction:",
                    u(MTL_LOAD_ACTION_LOAD),
                ),
                LoadAction::DontCare => set_pass_attachment(
                    metal,
                    pass,
                    0,
                    "RC-RPA-MAIN-COLOR",
                    "colorAttachment",
                    "setLoadAction:",
                    u(MTL_LOAD_ACTION_DONT_CARE),
                ),
            }
            set_pass_attachment(
                metal,
                pass,
                0,
                "RC-RPA-MAIN-COLOR",
                "colorAttachment",
                "setStoreAction:",
                u(MTL_STORE_ACTION_STORE),
            );
            let baseline = ShaderMiscFlags(0);
            if desc.interlockMode == InterlockMode::RasterOrdering {
                set_pass_attachment(
                    metal,
                    pass,
                    1,
                    "RC-RPA-MAIN-CLIP",
                    "clipAttachment",
                    "setTexture:",
                    target.clip_handle().map(h).unwrap_or(Value::Nil),
                );
                set_pass_attachment(
                    metal,
                    pass,
                    1,
                    "RC-RPA-MAIN-CLIP",
                    "clipAttachment",
                    "setLoadAction:",
                    u(MTL_LOAD_ACTION_CLEAR),
                );
                set_pass_attachment(
                    metal,
                    pass,
                    1,
                    "RC-RPA-MAIN-CLIP",
                    "clipAttachment",
                    "setClearColor:",
                    Value::ClearColor(ClearColor::default()),
                );
                set_pass_attachment(
                    metal,
                    pass,
                    1,
                    "RC-RPA-MAIN-CLIP",
                    "clipAttachment",
                    "setStoreAction:",
                    u(MTL_STORE_ACTION_DONT_CARE),
                );
                set_pass_attachment(
                    metal,
                    pass,
                    2,
                    "RC-RPA-MAIN-SCRATCH",
                    "scratchAttachment",
                    "setTexture:",
                    target.scratch_handle().map(h).unwrap_or(Value::Nil),
                );
                set_pass_attachment(
                    metal,
                    pass,
                    2,
                    "RC-RPA-MAIN-SCRATCH",
                    "scratchAttachment",
                    "setLoadAction:",
                    u(MTL_LOAD_ACTION_DONT_CARE),
                );
                set_pass_attachment(
                    metal,
                    pass,
                    2,
                    "RC-RPA-MAIN-SCRATCH",
                    "scratchAttachment",
                    "setStoreAction:",
                    u(MTL_STORE_ACTION_DONT_CARE),
                );
                set_pass_attachment(
                    metal,
                    pass,
                    3,
                    "RC-RPA-MAIN-COVERAGE",
                    "coverageAttachment",
                    "setTexture:",
                    target.coverage_handle().map(h).unwrap_or(Value::Nil),
                );
                set_pass_attachment(
                    metal,
                    pass,
                    3,
                    "RC-RPA-MAIN-COVERAGE",
                    "coverageAttachment",
                    "setLoadAction:",
                    u(MTL_LOAD_ACTION_CLEAR),
                );
                set_pass_attachment(
                    metal,
                    pass,
                    3,
                    "RC-RPA-MAIN-COVERAGE",
                    "coverageAttachment",
                    "setClearColor:",
                    Value::ClearColor(ClearColor {
                        red: desc.coverageClearValue as f64,
                        green: 0.0,
                        blue: 0.0,
                        alpha: 0.0,
                    }),
                );
                set_pass_attachment(
                    metal,
                    pass,
                    3,
                    "RC-RPA-MAIN-COVERAGE",
                    "coverageAttachment",
                    "setStoreAction:",
                    u(MTL_STORE_ACTION_DONT_CARE),
                );
            } else if desc.colorLoadAction == LoadAction::PreserveRenderTarget
                && !desc.fixedFunctionColorOutput
            {
                debug_assert_eq!(desc.interlockMode, InterlockMode::Atomics);
                let copy = metal
                    .call("commandBuffer", "blitCommandEncoder", vec![h(command)])
                    .unwrap_or(Handle::NIL);
                metal.owner_event("RC-ENC-COPY", OwnerEventPhase::Create, copy);
                let color_atomic = target.color_atomic_buffer_handle(metal);
                let bounds = desc.renderTargetUpdateBounds;
                set(
                    metal,
                    "copyEncoder",
                    "copyFromTexture:sourceSlice:sourceLevel:sourceOrigin:sourceSize:toBuffer:destinationOffset:destinationBytesPerRow:destinationBytesPerImage:",
                    vec![
                        h(copy),
                        target.target_handle().map(h).unwrap_or(Value::Nil),
                        u(0),
                        u(0),
                        Value::Origin(Origin {
                            x: bounds.left as usize,
                            y: bounds.top as usize,
                            z: 0,
                        }),
                        Value::Size(Size {
                            width: (bounds.right - bounds.left) as usize,
                            height: (bounds.bottom - bounds.top) as usize,
                            depth: 1,
                        }),
                        color_atomic.map(h).unwrap_or(Value::Nil),
                        u(
                            (bounds.top as u64 * target.base.width() as u64 + bounds.left as u64)
                                * core::mem::size_of::<u32>() as u64,
                        ),
                        u(target.base.width() as u64 * core::mem::size_of::<u32>() as u64),
                        u(target.base.height() as u64
                            * target.base.width() as u64
                            * core::mem::size_of::<u32>() as u64),
                    ],
                );
                set(metal, "copyEncoder", "endEncoding", vec![h(copy)]);
                metal.owner_event("RC-ENC-COPY", OwnerEventPhase::LastUse, copy);
                metal.retire_handle(copy);
                metal.owner_event("RC-ENC-COPY", OwnerEventPhase::Release, copy);
            }

            let full_scissor = source_bounds(desc.renderTargetUpdateBounds);
            let mut current_scissor = Rect {
                left: 0xffff,
                top: 0xffff,
                right: 0,
                bottom: 0,
            };
            let mut encoder = self.begin_draw_pass(metal, desc, target, command, pass, baseline);
            metal.owner_event("RC-ENC-MAIN", OwnerEventPhase::Transfer, encoder);
            for batch in draw_batches(desc) {
                let shader_features = if desc.interlockMode == InterlockMode::Atomics {
                    ShaderFeatures(
                        desc.combinedShaderFeatures.0
                            & features_mask_for(batch.drawType, desc.interlockMode),
                    )
                } else {
                    batch.shaderFeatures
                };
                let mut misc = ShaderMiscFlags(baseline.0 | batch.shaderMiscFlags.0);
                if !misc.has(ShaderMiscFlags::FIXED_FUNCTION_COLOR_OUTPUT) {
                    if batch.drawType == DrawType::RenderPassResolve {
                        debug_assert_eq!(desc.interlockMode, InterlockMode::Atomics);
                        misc = misc.with(ShaderMiscFlags::COALESCED_RESOLVE_AND_TRANSFER);
                    } else if batch.drawType == DrawType::RenderPassInitialize {
                        debug_assert_eq!(desc.interlockMode, InterlockMode::Atomics);
                        if desc.colorLoadAction == LoadAction::Clear {
                            misc = misc.with(ShaderMiscFlags::STORE_COLOR_CLEAR);
                        } else if desc.colorLoadAction == LoadAction::PreserveRenderTarget
                            && target.format() == PixelFormat::BGRA8Unorm
                        {
                            misc = misc.with(ShaderMiscFlags::SWIZZLE_COLOR_BGRA_TO_RGBA);
                        }
                    }
                }
                let fully_featured = ShaderFeatures(ubershader_features_mask(
                    shader_features,
                    batch.drawType,
                    desc.interlockMode,
                    misc,
                ));
                let Some(pipeline) = self.find_compatible_pipeline(
                    metal,
                    batch.drawType,
                    shader_features,
                    desc.interlockMode,
                    misc,
                    fully_featured,
                    synthesized_failure(desc),
                ) else {
                    continue;
                };
                if !pipeline.valid() {
                    continue;
                }
                let desired_scissor = batch
                    .scissorRect
                    .map(|rect| full_scissor.intersect_or_empty(source_scissor(rect)))
                    .unwrap_or(full_scissor);
                if desired_scissor != current_scissor {
                    current_scissor = desired_scissor;
                    set(
                        metal,
                        "encoder",
                        "setScissorRect:",
                        vec![h(encoder), scissor(current_scissor)],
                    );
                }
                let state_handle = pipeline.pipeline_state(target.format());
                // PipelineState is returned as a borrowed source member, but
                // the draw lambda receives an ARC strong local. Hold that
                // local through the complete batch before releasing it.
                let state_owner =
                    metal.clone_owned(state_handle, MetalObjectKind::RenderPipelineState);
                if let Some(owner) = state_owner.as_ref() {
                    metal.owner_event(
                        "RC-PS-DRAW",
                        OwnerEventPhase::CreateClone,
                        owner.handle(),
                    );
                }
                let state = state_owner
                    .as_ref()
                    .map(OwnedMetalHandle::handle)
                    .unwrap_or(state_handle);
                if let Some(texture_owner) = batch.imageTexture {
                    let texture = unsafe { metal_texture_handle(texture_owner) };
                    set(
                        metal,
                        "encoder",
                        "setFragmentTexture:atIndex:",
                        vec![
                            h(encoder),
                            texture.map(h).unwrap_or(Value::Nil),
                            u(IMAGE_TEXTURE_IDX),
                        ],
                    );
                    set(
                        metal,
                        "encoder",
                        "setFragmentSamplerState:atIndex:",
                        vec![
                            h(encoder),
                            self.m_imageSamplers[batch.imageSampler.asKey() as usize]
                                .as_ref()
                                .map(|owner| h(owner.handle()))
                                .unwrap_or(Value::Nil),
                            u(IMAGE_TEXTURE_IDX),
                        ],
                    );
                } else {
                    set(
                        metal,
                        "encoder",
                        "setFragmentSamplerState:atIndex:",
                        vec![
                            h(encoder),
                            self.m_imageSamplers[0]
                                .as_ref()
                                .map(|owner| h(owner.handle()))
                                .unwrap_or(Value::Nil),
                            u(IMAGE_TEXTURE_IDX),
                        ],
                    );
                }

                if batch.barriers.needs_atomic() {
                    debug_assert_eq!(desc.interlockMode, InterlockMode::Atomics);
                    match self.m_metalFeatures.atomicBarrierType {
                        crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::AtomicBarrierType::memoryBarrier => {
                            if metal.memory_barrier_available() {
                                set(
                                    metal,
                                    "encoder",
                                    "memoryBarrierWithScope:afterStages:beforeStages:",
                                    vec![
                                        h(encoder),
                                        u(MTL_BARRIER_SCOPE_BUFFERS_AND_RENDER_TARGETS),
                                        u(MTL_RENDER_STAGE_FRAGMENT),
                                        u(MTL_RENDER_STAGE_FRAGMENT),
                                    ],
                                );
                            } else {
                                rive_unreachable();
                            }
                        }
                        crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::AtomicBarrierType::rasterOrderGroup => {
                            metal.record_raster_order_group_barrier(encoder)
                        }
                        crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::AtomicBarrierType::renderPassBreak => {
                            set(metal, "encoder", "endEncoding", vec![h(encoder)]);
                            metal.owner_event("RC-ENC-MAIN", OwnerEventPhase::LastUse, encoder);
                            set_pass_attachment(
                                metal,
                                pass,
                                0,
                                "RC-RPA-MAIN-COLOR",
                                "colorAttachment",
                                "setLoadAction:",
                                u(MTL_LOAD_ACTION_LOAD),
                            );
                            let replacement =
                                self.begin_draw_pass(metal, desc, target, command, pass, baseline);
                            metal.owner_event(
                                "RC-ENC-MAIN",
                                OwnerEventPhase::Transfer,
                                replacement,
                            );
                            if replacement != Handle::NIL {
                                metal.record_render_pass_break();
                            }
                            // The source assignment evaluates the new
                            // encoder and mutates the attachment before the
                            // old strong encoder local is released.
                            metal.retire_handle(encoder);
                            metal.owner_event("RC-ENC-MAIN", OwnerEventPhase::Release, encoder);
                            encoder = replacement;
                        }
                    }
                }

                match batch.drawType {
                    DrawType::MidpointFanPatches
                    | DrawType::MidpointFanCenterAAPatches
                    | DrawType::OuterCurvePatches => {
                        set(
                            metal,
                            "encoder",
                            "setRenderPipelineState:",
                            vec![h(encoder), h(state)],
                        );
                        metal.record_draw_semantic(
                            encoder,
                            PipelineSemantic::draw(
                                batch.drawType,
                                desc.interlockMode,
                                shader_features,
                                misc,
                            ),
                        );
                        set(
                            metal,
                            "encoder",
                            "setVertexBuffer:offset:atIndex:",
                            vec![
                                h(encoder),
                                self.m_pathPatchVertexBuffer
                                    .as_ref()
                                    .map(|owner| h(owner.handle()))
                                    .unwrap_or(Value::Nil),
                                u(0),
                                u(0),
                            ],
                        );
                        set(
                            metal,
                            "encoder",
                            "setCullMode:",
                            vec![h(encoder), u(MTL_CULL_MODE_BACK)],
                        );
                        set(
                            metal,
                            "encoder",
                            "setVertexBytes:length:atIndex:",
                            vec![
                                h(encoder),
                                bytes(core::slice::from_ref(&batch.baseElement)),
                                u(core::mem::size_of::<u32>() as u64),
                                u(PATH_BASE_INSTANCE_UNIFORM_BUFFER_IDX),
                            ],
                        );
                        set(
                            metal,
                            "encoder",
                            "drawIndexedPrimitives:indexCount:indexType:indexBuffer:indexBufferOffset:instanceCount:",
                            vec![
                                h(encoder),
                                u(MTL_PRIMITIVE_TYPE_TRIANGLE),
                                u(batch.indexCountPerInstance),
                                u(MTL_INDEX_TYPE_UINT16),
                                self.m_pathPatchIndexBuffer
                                    .as_ref()
                                    .map(|owner| h(owner.handle()))
                                    .unwrap_or(Value::Nil),
                                u(batch.baseIndex as u64 * core::mem::size_of::<u16>() as u64),
                                u(batch.elementCount),
                            ],
                        );
                    }
                    DrawType::InteriorTriangulation | DrawType::FeatherAtlasBlit => {
                        set(
                            metal,
                            "encoder",
                            "setRenderPipelineState:",
                            vec![h(encoder), h(state)],
                        );
                        metal.record_draw_semantic(
                            encoder,
                            PipelineSemantic::draw(
                                batch.drawType,
                                desc.interlockMode,
                                shader_features,
                                misc,
                            ),
                        );
                        set(
                            metal,
                            "encoder",
                            "setVertexBuffer:offset:atIndex:",
                            vec![h(encoder), h(self.ring_buffer("triangle")), u(0), u(0)],
                        );
                        set(
                            metal,
                            "encoder",
                            "setCullMode:",
                            vec![h(encoder), u(MTL_CULL_MODE_BACK)],
                        );
                        set(
                            metal,
                            "encoder",
                            "drawPrimitives:vertexStart:vertexCount:",
                            vec![
                                h(encoder),
                                u(MTL_PRIMITIVE_TYPE_TRIANGLE),
                                u(batch.baseElement),
                                u(batch.elementCount),
                            ],
                        );
                    }
                    DrawType::ImageRect | DrawType::ImageMesh => {
                        set(
                            metal,
                            "encoder",
                            "setRenderPipelineState:",
                            vec![h(encoder), h(state)],
                        );
                        metal.record_draw_semantic(
                            encoder,
                            PipelineSemantic::draw(
                                batch.drawType,
                                desc.interlockMode,
                                shader_features,
                                misc,
                            ),
                        );
                        set(
                            metal,
                            "encoder",
                            "setVertexBuffer:offset:atIndex:",
                            vec![
                                h(encoder),
                                h(self.ring_buffer("imageDrawInstance")),
                                u(batch.baseElement as u64
                                    * core::mem::size_of::<gpu::ImageDrawInstance>() as u64),
                                u(2),
                            ],
                        );
                        set(
                            metal,
                            "encoder",
                            "setCullMode:",
                            vec![h(encoder), u(MTL_CULL_MODE_NONE)],
                        );
                        if batch.drawType == DrawType::ImageRect {
                            debug_assert_eq!(desc.interlockMode, InterlockMode::Atomics);
                            set(
                                metal,
                                "encoder",
                                "setVertexBuffer:offset:atIndex:",
                                vec![
                                    h(encoder),
                                    self.m_imageRectVertexBuffer
                                        .as_ref()
                                        .map(|owner| h(owner.handle()))
                                        .unwrap_or(Value::Nil),
                                    u(0),
                                    u(0),
                                ],
                            );
                            set(
                                metal,
                                "encoder",
                                "drawIndexedPrimitives:indexCount:indexType:indexBuffer:indexBufferOffset:instanceCount:",
                                vec![
                                    h(encoder),
                                    u(MTL_PRIMITIVE_TYPE_TRIANGLE),
                                    u(batch.indexCountPerInstance),
                                    u(MTL_INDEX_TYPE_UINT16),
                                    self.m_imageRectIndexBuffer
                                        .as_ref()
                                        .map(|owner| h(owner.handle()))
                                        .unwrap_or(Value::Nil),
                                    u(batch.baseIndex as u64 * core::mem::size_of::<u16>() as u64),
                                    u(batch.elementCount),
                                ],
                            );
                        } else {
                            // The three LITE_RTTI_CAST_OR_BREAK checks occur
                            // after the common image pipeline/instance/cull
                            // bindings and break only this switch case. The
                            // per-batch strong pipeline local still reaches
                            // its authored iteration-scope release.
                            if let Some((vertex, uv, index)) = unsafe {
                                image_mesh_buffer_handles(
                                    batch.vertexBuffer,
                                    batch.uvBuffer,
                                    batch.indexBuffer,
                                )
                            } {
                                set(
                                    metal,
                                    "encoder",
                                    "setVertexBuffer:offset:atIndex:",
                                    vec![h(encoder), h(vertex), u(0), u(0)],
                                );
                                set(
                                    metal,
                                    "encoder",
                                    "setVertexBuffer:offset:atIndex:",
                                    vec![h(encoder), h(uv), u(0), u(1)],
                                );
                                set(
                                    metal,
                                    "encoder",
                                    "drawIndexedPrimitives:indexCount:indexType:indexBuffer:indexBufferOffset:",
                                    vec![
                                        h(encoder),
                                        u(MTL_PRIMITIVE_TYPE_TRIANGLE),
                                        u(batch.indexCountPerInstance),
                                        u(MTL_INDEX_TYPE_UINT16),
                                        h(index),
                                        u(
                                            batch.baseIndex as u64
                                                * core::mem::size_of::<u16>() as u64,
                                        ),
                                    ],
                                );
                            }
                        }
                    }
                    DrawType::RenderPassInitialize | DrawType::RenderPassResolve => {
                        debug_assert_eq!(desc.interlockMode, InterlockMode::Atomics);
                        set(
                            metal,
                            "encoder",
                            "setRenderPipelineState:",
                            vec![h(encoder), h(state)],
                        );
                        metal.record_draw_semantic(
                            encoder,
                            PipelineSemantic::draw(
                                batch.drawType,
                                desc.interlockMode,
                                shader_features,
                                misc,
                            ),
                        );
                        set(
                            metal,
                            "encoder",
                            "drawPrimitives:vertexStart:vertexCount:",
                            vec![h(encoder), u(MTL_PRIMITIVE_TYPE_TRIANGLE_STRIP), u(0), u(4)],
                        );
                    }
                    DrawType::MsaaStrokes
                    | DrawType::MsaaMidpointFanBorrowedCoverage
                    | DrawType::MsaaDynamicMidpointFans
                    | DrawType::MsaaMidpointFans
                    | DrawType::MsaaMidpointFanStencilReset
                    | DrawType::MsaaMidpointFanPathsStencil
                    | DrawType::MsaaMidpointFanPathsCover
                    | DrawType::MsaaOuterCubics
                    | DrawType::ClipReset => rive_unreachable(),
                }
                if let Some(state_owner) = state_owner {
                    let state = state_owner.handle();
                    metal.owner_event("RC-PS-DRAW", OwnerEventPhase::LastUse, state);
                    drop(state_owner);
                    metal.owner_event("RC-PS-DRAW", OwnerEventPhase::Release, state);
                }
            }
            set(metal, "encoder", "endEncoding", vec![h(encoder)]);
            metal.owner_event("RC-ENC-MAIN", OwnerEventPhase::LastUse, encoder);
            metal.retire_handle(encoder);
            metal.owner_event("RC-ENC-MAIN", OwnerEventPhase::Release, encoder);
            metal.retire_handle(pass);
            metal.owner_event("RC-RPD-MAIN", OwnerEventPhase::Release, pass);
            if let Some(command_owner) = command_owner {
                let command = command_owner.handle();
                metal.owner_event(
                    "RC-CB-FLUSH-STRONG",
                    OwnerEventPhase::LastUse,
                    command,
                );
                drop(command_owner);
                metal.owner_event("RC-CB-FLUSH-STRONG", OwnerEventPhase::Release, command);
            }
        }

        /// # Safety
        /// The caller must keep this pinned context alive and unmoved until
        /// the completion callback runs; the callback stores a raw pointer to
        /// the source ring lock, matching the pinned C++ member lifetime.
        pub unsafe fn post_flush<E: MetalExecution>(
            &mut self,
            metal: &mut E,
            command: Handle,
            completion: Option<Arc<dyn Fn(Result<(), String>) + Send + Sync + 'static>>,
        ) {
            // `__bridge` creates a strong command-buffer local for this
            // function. Keep the transferred lease alive until the callback
            // has been installed; the adapter itself only borrows it.
            let command_owner = metal.clone_owned(command, MetalObjectKind::CommandBuffer);
            let command = command_owner
                .as_ref()
                .map(OwnedMetalHandle::handle)
                .unwrap_or(command);
            if command_owner.is_some() {
                metal.owner_event(
                    "RC-CB-POST-STRONG",
                    OwnerEventPhase::CreateClone,
                    command,
                );
            }
            let ring = *self.m_bufferRingIdx as usize;
            let state = BufferRingLockPtr(&(&*self.m_bufferRingLocks)[ring] as *const _);
            let completion_for_handler = completion.clone();
            let completion_block = metal.completion_block_identity(command);
            metal.owner_event(
                "RC-BLOCK-COMPLETE",
                OwnerEventPhase::BorrowStack,
                completion_block,
            );
            let installed = metal.add_completed_handler(
                command,
                Box::new(move |result| {
                    let state = unsafe { state.as_ref() };
                    debug_assert!(!state.mutex.try_lock());
                    unsafe { state.mutex.unlock() };
                    // The product completion token is deliberately published
                    // from this same source callback, after the raw ring-lock
                    // pointer has been released.  A second command-buffer
                    // handler cannot establish this ordering: Metal only
                    // guarantees that all handlers have completed by the
                    // terminal wait, not FIFO registration order.
                    if let Some(completion) = completion_for_handler {
                        completion(result);
                    }
                }),
            );
            if installed {
                metal.owner_event(
                    "RC-BLOCK-COMPLETE",
                    OwnerEventPhase::CopyTransfer,
                    completion_block,
                );
            }
            if !installed {
                metal.end_completion_block_identity(completion_block);
                metal.owner_event(
                    "RC-BLOCK-COMPLETE",
                    OwnerEventPhase::AliasEnd,
                    completion_block,
                );
                // A stale/NIL command or a native installation failure must
                // not leave either the source ring lock or product token
                // permanently pending. The handler was never installed, so
                // release both synchronously on this thread.
                let state = unsafe { state.as_ref() };
                debug_assert!(!state.mutex.try_lock());
                unsafe { state.mutex.unlock() };
                if let Some(completion) = completion {
                    completion(Err(
                        "failed to install command-buffer completion handler".into()
                    ));
                }
            }
            if let Some(command_owner) = command_owner {
                metal.owner_event("RC-CB-POST-STRONG", OwnerEventPhase::LastUse, command);
                let command = command_owner.handle();
                drop(command_owner);
                metal.owner_event("RC-CB-POST-STRONG", OwnerEventPhase::Release, command);
            }
        }

        #[cfg(test)]
        pub(crate) fn lock_current_ring_for_test(&self) {
            let ring = *self.m_bufferRingIdx as usize;
            (&self.m_bufferRingLocks)[ring].mutex.lock();
        }

        #[cfg(test)]
        pub(crate) fn current_ring_is_available_for_test(&self) -> bool {
            let ring = *self.m_bufferRingIdx as usize;
            let available = (&self.m_bufferRingLocks)[ring].mutex.try_lock();
            if available {
                unsafe {
                    (&self.m_bufferRingLocks)[ring].mutex.unlock();
                }
            }
            available
        }

        #[cfg(test)]
        pub(crate) fn current_ring_mutex_address_for_test(&self) -> usize {
            let ring = *self.m_bufferRingIdx as usize;
            &(&self.m_bufferRingLocks)[ring].mutex as *const SourceMutex as usize
        }

        #[cfg(test)]
        pub(crate) fn color_ramp_pipeline_state_for_test(&self) -> Option<Handle> {
            self.m_colorRampPipeline
                .as_ref()
                .and_then(|pipeline| pipeline.state.as_ref().map(OwnedMetalHandle::handle))
        }

        #[cfg(test)]
        pub(crate) fn tess_pipeline_state_for_test(&self) -> Option<Handle> {
            self.m_tessPipeline
                .as_ref()
                .and_then(|pipeline| pipeline.state.as_ref().map(OwnedMetalHandle::handle))
        }

        #[cfg(test)]
        pub(crate) fn feather_pipeline_states_for_test(
            &self,
        ) -> (Option<Handle>, Option<Handle>) {
            let fill = self
                .m_featherAtlasFillPipeline
                .as_ref()
                .and_then(|pipeline| pipeline.state.as_ref().map(OwnedMetalHandle::handle));
            let stroke = self
                .m_featherAtlasStrokePipeline
                .as_ref()
                .and_then(|pipeline| pipeline.state.as_ref().map(OwnedMetalHandle::handle));
            (fill, stroke)
        }

        #[cfg(test)]
        pub(crate) fn set_atomic_barrier_for_test(
            &mut self,
            barrier: crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::AtomicBarrierType,
        ) {
            self.m_metalFeatures.atomicBarrierType = barrier;
        }
    }

    const VERTEX_SHADER_FEATURES_MASK: u32 = 0x0f;
    fn interlock_features(interlock: InterlockMode) -> u32 {
        match interlock {
            InterlockMode::RasterOrdering => 0xff,
            InterlockMode::Atomics => 0xdf,
            InterlockMode::Clockwise => 0xef,
            InterlockMode::ClockwiseAtomic => 0xcf,
            InterlockMode::Msaa => 0xc6,
        }
    }
    fn features_mask_for(draw: DrawType, interlock: InterlockMode) -> u32 {
        let mask = match draw {
            DrawType::ImageRect | DrawType::ImageMesh | DrawType::FeatherAtlasBlit
                if interlock != InterlockMode::Atomics =>
            {
                0xc7
            }
            DrawType::MidpointFanPatches
            | DrawType::MidpointFanCenterAAPatches
            | DrawType::OuterCurvePatches
            | DrawType::InteriorTriangulation
            | DrawType::MsaaStrokes
            | DrawType::MsaaMidpointFanBorrowedCoverage
            | DrawType::MsaaDynamicMidpointFans
            | DrawType::MsaaMidpointFans
            | DrawType::MsaaMidpointFanStencilReset
            | DrawType::MsaaMidpointFanPathsStencil
            | DrawType::MsaaMidpointFanPathsCover
            | DrawType::MsaaOuterCubics
            | DrawType::ImageRect
            | DrawType::ImageMesh
            | DrawType::FeatherAtlasBlit => 0xff,
            DrawType::ClipReset => 0x80,
            DrawType::RenderPassInitialize => match interlock {
                InterlockMode::Atomics => 0x85,
                InterlockMode::Msaa => 0x80,
                _ => 0,
            },
            DrawType::RenderPassResolve if interlock == InterlockMode::Atomics => 0xff,
            DrawType::RenderPassResolve => 0x80,
        };
        mask & interlock_features(interlock)
    }
    fn ubershader_features_mask(
        features: ShaderFeatures,
        draw: DrawType,
        interlock: InterlockMode,
        misc: ShaderMiscFlags,
    ) -> u32 {
        let mut out = features_mask_for(draw, interlock);
        if interlock == InterlockMode::Atomics {
            out &= features.0 | !0x04;
        }
        if misc.has(ShaderMiscFlags::FIXED_FUNCTION_COLOR_OUTPUT) {
            out &= !0x04;
        }
        if interlock == InterlockMode::Atomics
            && misc.has(ShaderMiscFlags::COALESCED_RESOLVE_AND_TRANSFER)
        {
            out |= 0x04;
        }
        debug_assert_eq!(features.0 & out, features.0);
        out
    }
}

#[cfg(test)]
mod source_owner_regressions {
    use super::source_execution::{
        canonical_metallib, image_mesh_buffer_handles, precompiled_name, shader_key_for_test, BufferRingLock, ColorRampPipeline,
        DrawPipeline, DrawType, Handle,
        InterlockMode, MetalExecution, MetalObjectKind, OwnerEvent, OwnerEventPhase,
        OwnedMetalHandle, PixelFormat, RecordingMetal, RenderBufferMetal, RenderContextMetal,
        RenderTargetMetal, ShaderFeatures, ShaderMiscFlags, SourceFunctionName, SourceMutex,
        Size, SynthesizedFailureType, TessellatePipeline, TextureMetal, Value,
        ATLAS_FILL_FRAGMENT_NAME, ATLAS_STROKE_FRAGMENT_NAME, DRAW_FRAGMENT_NAME,
        DRAW_VERTEX_NAME, SOURCE_STATIC_FUNCTION_NAMES,
        RENDER_CONTEXT_METAL_DROP_TRACE,
        RENDER_CONTEXT_OWNER_DROP_EVENTS, RENDER_CONTEXT_OWNER_DROP_RETIREMENTS,
    };
    use crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::ContextOptions;
    use super::gpu;
    use crate::mechanical_port::source::include::rive::refcnt_hpp::RefCnt;
    use crate::mechanical_port::source::include::rive::refcnt_hpp::rcp;
    use crate::mechanical_port::source::include::rive::renderer_hpp::{
        RenderBuffer, RenderBufferFlags, RenderBufferType,
    };
    use crate::mechanical_port::source::renderer::include::rive::renderer::render_target_hpp::RenderTarget;
    #[cfg(feature = "native-ore-metal-experimental")]
    use crate::mechanical_port::source::renderer::include::rive::renderer::{
        render_canvas_hpp::RenderCanvas, rive_render_image_hpp::RiveRenderImage,
        texture_hpp::Texture,
    };
    use crate::mechanical_port::source::include::utils::lite_rtti_hpp::CONST_ID;
    use std::collections::{BTreeMap, BTreeSet};
    use std::sync::Arc;
    use sha2::Digest;

    const OWNER_EXPECTATIONS: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/metal-port-reports/metal-native-owner-expectations.tsv"
    ));
    const OWNER_EXPECTATIONS_SHA256: &str =
        "08824cc2e288d9f29711c1e5d85666b25c0eb9d0bc46d33e14b6de51623e6765";

    unsafe fn foreign_destroy(_: *mut RenderBuffer) {}
    unsafe fn foreign_map(_: *mut RenderBuffer) -> *mut core::ffi::c_void {
        core::ptr::null_mut()
    }
    unsafe fn foreign_unmap(_: *mut RenderBuffer) {}

    fn foreign_buffer() -> Box<RenderBuffer> {
        Box::new(RenderBuffer {
            base: RefCnt::new(),
            destroy_complete: foreign_destroy,
            on_map: foreign_map,
            on_unmap: foreign_unmap,
            m_liteTypeId: CONST_ID("ForeignRenderBuffer"),
            m_type: RenderBufferType::vertex,
            m_flags: RenderBufferFlags::none,
            m_sizeInBytes: 16,
            m_dirty: false,
            #[cfg(debug_assertions)]
            m_mapCount: 0,
            #[cfg(debug_assertions)]
            m_unmapCount: 0,
            #[cfg(any(
                feature = "native-webgpu-experimental",
                feature = "ore-gl"
            ))]
            rust_final_release_route: None,
        })
    }

    fn empty_source_flush_descriptor() -> gpu::FlushDescriptor {
        gpu::FlushDescriptor {
            renderTarget: None,
            combinedShaderFeatures: gpu::ShaderFeatures::NONE,
            interlockMode: gpu::InterlockMode::RasterOrdering,
            msaaSampleCount: 1,
            colorLoadAction: gpu::LoadAction::DontCare,
            colorClearValue: 0,
            coverageClearValue: 0,
            depthClearValue: 0.0,
            stencilClearValue: 0,
            renderTargetUpdateBounds: gpu::IAABB {
                left: 0,
                top: 0,
                right: 4,
                bottom: 4,
            },
            virtualTileWidth: 0,
            virtualTileHeight: 0,
            manuallyResolved: false,
            fixedFunctionColorOutput: false,
            featherAtlasTextureWidth: 0,
            featherAtlasTextureHeight: 0,
            featherAtlasContentWidth: 4,
            featherAtlasContentHeight: 4,
            coverageBufferPrefix: 0,
            needsCoverageBufferClear: false,
            flushUniformDataOffsetInBytes: 0,
            pathCount: 0,
            firstPath: 0,
            firstPaint: 0,
            firstPaintAux: 0,
            contourCount: 0,
            firstContour: 0,
            gradSpanCount: 0,
            firstGradSpan: 0,
            tessVertexSpanCount: 0,
            firstTessVertexSpan: 0,
            gradDataHeight: 0,
            tessDataHeight: 0,
            clockwiseFillOverride: false,
            hasTriangleVertices: false,
            wireframe: false,
            ditherMode: gpu::DitherMode::none,
            #[cfg(feature = "with-rive-tools")]
            synthesizedFailureType: gpu::SynthesizedFailureType::none,
            externalCommandBuffer: None,
            featherAtlasFillBatches: None,
            featherAtlasFillBatchCount: 0,
            featherAtlasStrokeBatches: None,
            featherAtlasStrokeBatchCount: 0,
            drawList: None,
            firstDstBlendBarrier: None,
            unresolvedBarriers: gpu::BarrierFlags::default(),
        }
    }

    fn recording_flush_fixture(
        with_feather_pipelines: bool,
    ) -> (RecordingMetal, RenderContextMetal, RenderTargetMetal) {
        let mut metal = RecordingMetal::default();
        let device = metal.device_handle();
        let mut context = RenderContextMetal::new(&mut metal, device, ContextOptions::default());
        if with_feather_pipelines {
            context.resize_feather(&mut metal, 4, 4);
        }
        let target_device = metal
            .clone_owned(device, MetalObjectKind::Device)
            .unwrap_or_else(|| OwnedMetalHandle::token(device));
        let target = RenderTargetMetal::new_with_device(
            &mut metal,
            target_device,
            PixelFormat::RGBA8Unorm,
            4,
            4,
            gpu::PlatformFeatures::default(),
        );
        for name in [
            "flushUniform",
            "path",
            "paint",
            "paintAux",
            "contour",
            "gradSpan",
            "tessSpan",
            "triangle",
            "imageDrawInstance",
        ] {
            context.make_uniform_buffer_ring(&mut metal, name, 16);
        }
        metal.owner_events.clear();
        metal.retirements.clear();
        metal.retirement_call_counts.clear();
        metal.retirement_event_counts.clear();
        metal.calls.clear();
        (metal, context, target)
    }

    fn assert_clone_triplets(
        metal: &RecordingMetal,
        ledger_id: &str,
        source: Handle,
        count: usize,
    ) -> Vec<OwnerEvent> {
        let events = row_events(&metal.owner_events, ledger_id);
        assert_eq!(events.len(), count * 3, "{ledger_id} multiplicity");
        for triple in events.chunks_exact(3) {
            assert_eq!(
                triple.iter().map(|event| event.phase).collect::<Vec<_>>(),
                vec![
                    OwnerEventPhase::CreateClone,
                    OwnerEventPhase::LastUse,
                    OwnerEventPhase::Release,
                ]
            );
            assert!(triple.iter().all(|event| {
                event.handle == triple[0].handle
                    && event.source_handle == source
                    && event.native_identity == source
            }));
            assert_ne!(triple[0].handle, source);
            assert!(metal.retirements.contains(&triple[0].handle));
            assert!(!metal.retirements.contains(&source));
        }
        events.into_iter().copied().collect()
    }

    #[test]
    fn flush_descriptor_and_draw_batch_are_exact_gpu_owners() {
        fn first_linked_batch<'a>(desc: &'a gpu::FlushDescriptor) -> Option<&'a gpu::DrawBatch> {
            desc.drawList
                .map(|list| unsafe { list.as_ref() })
                .and_then(|list| list.iter().next())
        }
        let _: for<'a> fn(&'a gpu::FlushDescriptor) -> Option<&'a gpu::DrawBatch> =
            first_linked_batch;
        let _: fn(&gpu::DrawBatch) -> usize = |batch| batch.baseElement as usize;
        // These exact source fields are size_t in the pinned ABI; the witness
        // deliberately refuses any u32 conversion/narrowing.
        let _: fn(&gpu::FlushDescriptor) -> usize = |desc| desc.firstPath;
        if usize::BITS > 32 {
            let mut raw = core::mem::MaybeUninit::<gpu::FlushDescriptor>::zeroed();
            let large = (u32::MAX as usize) + 1;
            unsafe {
                (*raw.as_mut_ptr()).firstPath = large;
            }
            let raw = unsafe { raw.assume_init() };
            assert_eq!(raw.firstPath, large);
        }
    }

    #[test]
    fn flush_uses_three_node_source_list_in_authored_order() {
        let mut list = gpu::BlockAllocatedLinkedList::<gpu::DrawBatch>::default();
        for (base, draw_type) in [
            (11_u32, gpu::DrawType::midpointFanPatches),
            (22, gpu::DrawType::interiorTriangulation),
            (33, gpu::DrawType::imageMesh),
        ] {
            let mut batch = gpu::DrawBatch::new(
                draw_type,
                gpu::ShaderMiscFlags(0),
                gpu::DrawContents::none,
                base + 1,
                base,
                nuxie_render_api::BlendMode::SrcOver,
                gpu::ImageSampler::default(),
                gpu::BarrierFlags(0),
            );
            batch.indexCountPerInstance = base + 2;
            list.push_back(batch);
        }
        let ordered: Vec<_> = list
            .iter()
            .map(|batch| {
                (
                    batch.drawType,
                    batch.baseElement,
                    batch.indexCountPerInstance,
                )
            })
            .collect();
        assert_eq!(ordered[0].0, gpu::DrawType::midpointFanPatches);
        assert_eq!(ordered[1].0, gpu::DrawType::interiorTriangulation);
        assert_eq!(ordered[2].0, gpu::DrawType::imageMesh);
        assert_eq!(
            ordered.iter().map(|(_, base, _)| *base).collect::<Vec<_>>(),
            [11, 22, 33]
        );
        assert_eq!(
            ordered
                .iter()
                .map(|(_, _, count)| *count)
                .collect::<Vec<_>>(),
            [13, 24, 35]
        );
    }

    #[test]
    fn ring_slot_is_an_independent_lock_handoff() {
        let first = SourceMutex::new();
        let second = SourceMutex::new();
        first.lock();
        assert!(!first.try_lock());
        assert!(second.try_lock());
        unsafe {
            first.unlock();
            second.unlock();
        }
        let slot = BufferRingLock::new();
        slot.mutex.lock();
        unsafe {
            slot.mutex.unlock();
        }
    }

    #[test]
    fn fourth_ring_reuse_waits_for_completion_unlock() {
        use std::sync::{Arc, mpsc};
        use std::time::Duration;

        let slots: Arc<[SourceMutex; 3]> =
            Arc::new([SourceMutex::new(), SourceMutex::new(), SourceMutex::new()]);
        for slot in slots.iter() {
            slot.lock();
        }
        let (ready_tx, ready_rx) = mpsc::channel();
        let (acquired_tx, acquired_rx) = mpsc::channel();
        let first = Arc::clone(&slots);
        let waiter = std::thread::spawn(move || {
            ready_tx.send(()).unwrap();
            first[0].lock();
            acquired_tx.send(()).unwrap();
            unsafe {
                first[0].unlock();
            }
        });
        ready_rx.recv().unwrap();
        assert!(acquired_rx.recv_timeout(Duration::from_millis(20)).is_err());
        unsafe {
            slots[0].unlock();
        }
        acquired_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        waiter.join().unwrap();
        unsafe {
            slots[1].unlock();
            slots[2].unlock();
        }
    }

    #[test]
    fn image_mesh_cast_helper_fails_before_native_mesh_buffer_binds() {
        let mut metal = RecordingMetal::default();
        let device = metal.device_handle();
        let owner = Box::new(RenderBufferMetal::new(
            &mut metal,
            super::source_execution::OwnedMetalHandle::token(device),
            RenderBufferType::vertex,
            RenderBufferFlags::none,
            16,
            false,
        ));
        let mut owner = owner;
        let valid = core::ptr::NonNull::from(&mut owner.base.base);
        let foreign = foreign_buffer();
        let foreign = core::ptr::NonNull::from(foreign.as_ref());
        metal.calls.clear();

        for (vertex, uv, index) in [
            (Some(foreign), Some(valid), Some(valid)),
            (Some(valid), Some(foreign), Some(valid)),
            (Some(valid), Some(valid), Some(foreign)),
        ] {
            assert!(unsafe { image_mesh_buffer_handles(vertex, uv, index) }.is_none());
        }
        // This helper is called at the three authored
        // LITE_RTTI_CAST_OR_BREAK expressions. The surrounding switch case
        // has already issued its common pipeline/instance/cull selectors;
        // None suppresses only the native mesh-buffer binds and draw.
        assert!(metal.calls.is_empty());
    }

    #[test]
    fn image_mesh_preflight_keeps_valid_metal_nil_submitted_buffers() {
        let mut metal = RecordingMetal::default();
        let device = metal.device_handle();
        let mut owner = Box::new(RenderBufferMetal::new(
            &mut metal,
            super::source_execution::OwnedMetalHandle::token(device),
            RenderBufferType::vertex,
            RenderBufferFlags::none,
            16,
            false,
        ));
        let valid = core::ptr::NonNull::from(&mut owner.base.base);
        assert_eq!(
            unsafe { image_mesh_buffer_handles(Some(valid), Some(valid), Some(valid)) },
            Some((
                super::source_execution::Handle::NIL,
                super::source_execution::Handle::NIL,
                super::source_execution::Handle::NIL,
            ))
        );
    }

    #[test]
    fn post_flush_install_failure_unlocks_ring_and_publishes_error() {
        use std::sync::{Arc, Mutex};

        let mut metal = RecordingMetal::default();
        let device = metal.device_handle();
        let mut context = RenderContextMetal::new(&mut metal, device, ContextOptions::default());
        context.lock_current_ring_for_test();
        metal.completed_handler_install_fail = true;
        let result = Arc::new(Mutex::new(None));
        let observed = Arc::clone(&result);
        let command = super::source_execution::Handle::new(
            99,
            super::source_execution::MetalObjectKind::CommandBuffer,
        );
        unsafe {
            context.post_flush(
                &mut metal,
                command,
                Some(Arc::new(move |value| {
                    *observed.lock().unwrap() = Some(value);
                })),
            );
        }
        assert!(context.current_ring_is_available_for_test());
        assert!(matches!(
            result.lock().unwrap().as_ref(),
            Some(Err(message)) if message.contains("install")
        ));
        let post = metal
            .owner_events
            .iter()
            .filter(|event| event.ledger_id == "RC-CB-POST-STRONG")
            .copied()
            .collect::<Vec<_>>();
        let block = metal
            .owner_events
            .iter()
            .filter(|event| event.ledger_id == "RC-BLOCK-COMPLETE")
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(
            block.iter().map(|event| event.phase).collect::<Vec<_>>(),
            vec![OwnerEventPhase::BorrowStack, OwnerEventPhase::AliasEnd]
        );
        assert_eq!(block[0].handle, block[1].handle);
        assert_ne!(block[0].handle, command);
        assert!(block
            .iter()
            .all(|event| event.parent_handle == Some(post[0].handle)));
        assert!(metal.retirements.contains(&block[0].handle));
    }

    #[test]
    fn post_flush_copies_invokes_and_releases_distinct_completion_block() {
        use std::sync::{Arc, Mutex};

        let mut metal = RecordingMetal::default();
        let device = metal.device_handle();
        let mut context = RenderContextMetal::new(&mut metal, device, ContextOptions::default());
        context.lock_current_ring_for_test();
        let result = Arc::new(Mutex::new(None));
        let observed = Arc::clone(&result);
        let available_during_publish = Arc::new(Mutex::new(None));
        let observed_available = Arc::clone(&available_during_publish);
        let ring_address = context.current_ring_mutex_address_for_test();
        let command = super::source_execution::Handle::new(
            99,
            super::source_execution::MetalObjectKind::CommandBuffer,
        );
        unsafe {
            context.post_flush(
                &mut metal,
                command,
                Some(Arc::new(move |value| {
                    let ring = unsafe { &*(ring_address as *const SourceMutex) };
                    let available = ring.try_lock();
                    if available {
                        unsafe { ring.unlock() };
                    }
                    *observed_available.lock().unwrap() = Some(available);
                    *observed.lock().unwrap() = Some(value);
                })),
            );
        }
        assert!(!context.current_ring_is_available_for_test());
        assert!(result.lock().unwrap().is_none());
        metal.run_next_completed_handler();
        assert!(context.current_ring_is_available_for_test());
        assert!(matches!(result.lock().unwrap().as_ref(), Some(Ok(()))));
        assert_eq!(*available_during_publish.lock().unwrap(), Some(true));

        let post = metal
            .owner_events
            .iter()
            .filter(|event| event.ledger_id == "RC-CB-POST-STRONG")
            .copied()
            .collect::<Vec<_>>();
        let block = metal
            .owner_events
            .iter()
            .filter(|event| event.ledger_id == "RC-BLOCK-COMPLETE")
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(
            post.iter().map(|event| event.phase).collect::<Vec<_>>(),
            vec![
                OwnerEventPhase::CreateClone,
                OwnerEventPhase::LastUse,
                OwnerEventPhase::Release,
            ]
        );
        assert!(post.iter().all(|event| {
            event.handle == post[0].handle
                && event.source_handle == command
                && event.native_identity == command
        }));
        assert_ne!(post[0].handle, command);
        metal.drain_recorded_clone_drops();
        assert!(metal.retirements.contains(&post[0].handle));
        assert_eq!(
            block.iter().map(|event| event.phase).collect::<Vec<_>>(),
            vec![
                OwnerEventPhase::BorrowStack,
                OwnerEventPhase::CopyTransfer,
                OwnerEventPhase::Invoke,
                OwnerEventPhase::ReleaseCopy,
            ]
        );
        assert!(block
            .iter()
            .all(|event| event.handle == block[0].handle
                && event.native_identity == block[0].native_identity
                && event.parent_handle == Some(post[0].handle)));
        assert_ne!(block[0].native_identity, post[0].native_identity);
        assert!(metal.retirements.contains(&block[0].handle));
    }

    #[test]
    fn post_flush_installed_callback_error_unlocks_before_publishing_and_releases_copy() {
        use std::sync::{Arc, Mutex};

        let mut metal = RecordingMetal::default();
        let device = metal.device_handle();
        let mut context = RenderContextMetal::new(&mut metal, device, ContextOptions::default());
        context.lock_current_ring_for_test();
        let result = Arc::new(Mutex::new(None));
        let observed = Arc::clone(&result);
        let available_during_publish = Arc::new(Mutex::new(None));
        let observed_available = Arc::clone(&available_during_publish);
        let ring_address = context.current_ring_mutex_address_for_test();
        let command = Handle::new(99, MetalObjectKind::CommandBuffer);
        unsafe {
            context.post_flush(
                &mut metal,
                command,
                Some(Arc::new(move |value| {
                    let ring = unsafe { &*(ring_address as *const SourceMutex) };
                    let available = ring.try_lock();
                    if available {
                        unsafe { ring.unlock() };
                    }
                    *observed_available.lock().unwrap() = Some(available);
                    *observed.lock().unwrap() = Some(value);
                })),
            );
        }

        assert!(!context.current_ring_is_available_for_test());
        metal.run_next_completed_handler_with(Err("forced command failure".into()));
        assert!(context.current_ring_is_available_for_test());
        assert!(matches!(
            result.lock().unwrap().as_ref(),
            Some(Err(message)) if message == "forced command failure"
        ));
        assert_eq!(*available_during_publish.lock().unwrap(), Some(true));

        let post = row_events(&metal.owner_events, "RC-CB-POST-STRONG");
        let block = row_events(&metal.owner_events, "RC-BLOCK-COMPLETE");
        assert_eq!(
            block.iter().map(|event| event.phase).collect::<Vec<_>>(),
            vec![
                OwnerEventPhase::BorrowStack,
                OwnerEventPhase::CopyTransfer,
                OwnerEventPhase::Invoke,
                OwnerEventPhase::ReleaseCopy,
            ]
        );
        assert!(block.iter().all(|event| {
            event.handle == block[0].handle
                && event.native_identity == block[0].native_identity
                && event.parent_handle == Some(post[0].handle)
        }));
        assert_ne!(block[0].native_identity, post[0].native_identity);
        assert!(metal.retirements.contains(&block[0].handle));
    }

    #[test]
    fn post_flush_command_clone_failure_is_exact_and_does_not_strand_the_ring() {
        use std::sync::{Arc, Mutex};

        let mut metal = RecordingMetal::default();
        let device = metal.device_handle();
        let mut context = RenderContextMetal::new(&mut metal, device, ContextOptions::default());
        context.lock_current_ring_for_test();
        metal.fail_clone_exact = Some((MetalObjectKind::CommandBuffer, 1));
        let result = Arc::new(Mutex::new(None));
        let observed = Arc::clone(&result);
        let command = Handle::new(99, MetalObjectKind::CommandBuffer);
        unsafe {
            context.post_flush(
                &mut metal,
                command,
                Some(Arc::new(move |value| {
                    *observed.lock().unwrap() = Some(value);
                })),
            );
        }
        assert_eq!(metal.fail_clone_exact, None);
        assert!(row_events(&metal.owner_events, "RC-CB-POST-STRONG").is_empty());
        assert!(!context.current_ring_is_available_for_test());
        metal.run_next_completed_handler();
        assert!(context.current_ring_is_available_for_test());
        assert!(matches!(result.lock().unwrap().as_ref(), Some(Ok(()))));
        let block = row_events(&metal.owner_events, "RC-BLOCK-COMPLETE");
        assert!(block
            .iter()
            .all(|event| event.parent_handle == Some(command)));
    }

    #[test]
    fn command_buffer_bridge_lifecycle_preserves_one_opaque_owner_through_commit() {
        let run = |fail_commit: bool| {
            let mut metal = RecordingMetal::default();
            let device = metal.device_handle();
            let mut context =
                RenderContextMetal::new(&mut metal, device, ContextOptions::default());
            context.set_command_queue(
                &mut metal,
                Some(Handle::new(90, MetalObjectKind::CommandQueue)),
            );
            metal.owner_events.clear();
            metal.retirements.clear();
            metal.retirement_call_counts.clear();
            metal.retirement_event_counts.clear();
            metal.calls.clear();
            let command = context
                .make_command_buffer(&mut metal)
                .expect("source __bridge_retained command buffer");
            context.lock_current_ring_for_test();
            unsafe { context.post_flush(&mut metal, command, None) };
            metal.run_next_completed_handler();
            if fail_commit {
                metal.fail_exact = Some(("commit (__bridge_transfer)", 1));
            }
            context.commit_command_buffer(&mut metal, Some(command));
            metal.drain_recorded_clone_drops();
            (metal, command)
        };

        for fail_commit in [false, true] {
            let (metal, command) = run(fail_commit);
            assert_eq!(metal.fail_exact, None);
            let retained = row_events(&metal.owner_events, "RC-CB-RETAINED");
            let transfer = row_events(&metal.owner_events, "RC-CB-TRANSFER");
            let post = row_events(&metal.owner_events, "RC-CB-POST-STRONG");
            let block = row_events(&metal.owner_events, "RC-BLOCK-COMPLETE");
            assert_eq!(
                retained.iter().map(|event| event.phase).collect::<Vec<_>>(),
                vec![
                    OwnerEventPhase::Create,
                    OwnerEventPhase::Transfer,
                    OwnerEventPhase::Release,
                ]
            );
            assert_eq!(
                transfer.iter().map(|event| event.phase).collect::<Vec<_>>(),
                vec![
                    OwnerEventPhase::Transfer,
                    OwnerEventPhase::LastUse,
                    OwnerEventPhase::Release,
                ]
            );
            assert!(retained
                .iter()
                .chain(transfer.iter())
                .all(|event| event.handle == command
                    && event.native_identity == command));
            assert_eq!(post.len(), 3);
            assert_eq!(post[0].source_handle, command);
            assert!(block.iter().all(|event| {
                event.parent_handle == Some(post[0].handle)
                    && event.native_identity != command
            }));
            assert!(
                event_position(
                    &metal.owner_events,
                    "RC-BLOCK-COMPLETE",
                    OwnerEventPhase::CopyTransfer,
                ) < event_position(
                    &metal.owner_events,
                    "RC-CB-POST-STRONG",
                    OwnerEventPhase::Release,
                )
            );
            assert!(
                event_position(
                    &metal.owner_events,
                    "RC-CB-POST-STRONG",
                    OwnerEventPhase::Release,
                ) < event_position(
                    &metal.owner_events,
                    "RC-CB-RETAINED",
                    OwnerEventPhase::Transfer,
                )
            );
            let commit = metal
                .calls
                .iter()
                .position(|call| call.selector == "commit (__bridge_transfer)")
                .unwrap();
            let transfer_open = metal
                .owner_events
                .iter()
                .position(|event| {
                    event.ledger_id == "RC-CB-TRANSFER"
                        && event.phase == OwnerEventPhase::Transfer
                })
                .unwrap();
            let transfer_close = metal
                .owner_events
                .iter()
                .position(|event| {
                    event.ledger_id == "RC-CB-TRANSFER"
                        && event.phase == OwnerEventPhase::Release
                })
                .unwrap();
            assert!(transfer_open < transfer_close);
            assert_eq!(metal.calls[commit].args, vec![Value::Handle(command)]);
            let retirement_after_calls = metal
                .retirement_call_counts
                .iter()
                .find_map(|(retired, call_count)| (*retired == command).then_some(*call_count))
                .expect("opaque command native retirement boundary");
            assert!(
                retirement_after_calls > commit,
                "__bridge_transfer released before its commit last-use"
            );
            assert!(metal.retirements.contains(&command));
        }

        let mut allocation_failure = RecordingMetal::default();
        let device = allocation_failure.device_handle();
        let mut context = RenderContextMetal::new(
            &mut allocation_failure,
            device,
            ContextOptions::default(),
        );
        context.set_command_queue(
            &mut allocation_failure,
            Some(Handle::new(90, MetalObjectKind::CommandQueue)),
        );
        let selector = "commandBuffer (__bridge_retained)";
        let next = allocation_failure.selector_occurrence_count(selector) + 1;
        allocation_failure.fail_exact = Some((selector, next));
        assert!(context.make_command_buffer(&mut allocation_failure).is_none());
        assert_eq!(allocation_failure.fail_exact, None);
        assert!(row_events(&allocation_failure.owner_events, "RC-CB-RETAINED").is_empty());
        assert!(row_events(&allocation_failure.owner_events, "RC-CB-TRANSFER").is_empty());
    }

    #[test]
    fn flush_command_strong_local_clones_the_opaque_owner_for_the_complete_flush_scope() {
        let run = |fail_clone: bool| {
            let mut metal = RecordingMetal::default();
            let device = metal.device_handle();
            let mut context =
                RenderContextMetal::new(&mut metal, device, ContextOptions::default());
            let target_device = metal
                .clone_owned(device, MetalObjectKind::Device)
                .unwrap_or_else(|| OwnedMetalHandle::token(device));
            let mut target = RenderTargetMetal::new_with_device(
                &mut metal,
                target_device,
                PixelFormat::RGBA8Unorm,
                4,
                4,
                gpu::PlatformFeatures::default(),
            );
            for name in [
                "flushUniform",
                "path",
                "paint",
                "paintAux",
                "contour",
                "gradSpan",
                "tessSpan",
                "triangle",
                "imageDrawInstance",
            ] {
                context.make_uniform_buffer_ring(&mut metal, name, 16);
            }
            metal.owner_events.clear();
            metal.retirements.clear();
            metal.calls.clear();
            if fail_clone {
                metal.fail_clone_exact = Some((MetalObjectKind::CommandBuffer, 1));
            }
            let command = Handle::new(88, MetalObjectKind::CommandBuffer);
            let desc = empty_source_flush_descriptor();
            unsafe { context.flush(&mut metal, &desc, &mut target, command) };
            metal.drain_recorded_clone_drops();
            (metal, command)
        };

        let (metal, command) = run(false);
        let command_local = row_events(&metal.owner_events, "RC-CB-FLUSH-STRONG");
        assert_eq!(
            command_local
                .iter()
                .map(|event| event.phase)
                .collect::<Vec<_>>(),
            vec![
                OwnerEventPhase::CreateClone,
                OwnerEventPhase::LastUse,
                OwnerEventPhase::Release,
            ]
        );
        assert!(command_local.iter().all(|event| {
            event.handle == command_local[0].handle
                && event.source_handle == command
                && event.native_identity == command
        }));
        assert_ne!(command_local[0].handle, command);
        assert!(metal.retirements.contains(&command_local[0].handle));
        assert!(!metal.retirements.contains(&command));
        assert!(
            event_position(
                &metal.owner_events,
                "RC-RPD-MAIN",
                OwnerEventPhase::Release,
            ) < event_position(
                &metal.owner_events,
                "RC-CB-FLUSH-STRONG",
                OwnerEventPhase::LastUse,
            )
        );
        assert_eq!(
            command_local[1].selector_ordinal.map(|ordinal| ordinal.0),
            Some("endEncoding")
        );

        let (failed, command) = run(true);
        assert_eq!(failed.fail_clone_exact, None);
        assert!(row_events(&failed.owner_events, "RC-CB-FLUSH-STRONG").is_empty());
        assert!(!failed.retirements.contains(&command));
        let encoder_creation = failed
            .calls
            .iter()
            .find(|call| call.selector == "renderCommandEncoderWithDescriptor:")
            .expect("source flush still reaches its encoder on clone failpoint");
        assert_eq!(encoder_creation.args.first(), Some(&Value::Handle(command)));
    }

    #[test]
    fn gradient_and_tessellation_pipeline_locals_span_encoder_and_pass_teardown() {
        let run = |gradient: bool, fail_clone: bool| {
            let (mut metal, mut context, mut target) = recording_flush_fixture(false);
            let source = if gradient {
                context
                    .color_ramp_pipeline_state_for_test()
                    .expect("constructor color-ramp pipeline state")
            } else {
                context
                    .tess_pipeline_state_for_test()
                    .expect("constructor tessellation pipeline state")
            };
            if fail_clone {
                metal.fail_clone_exact = Some((MetalObjectKind::RenderPipelineState, 1));
            }
            let mut desc = empty_source_flush_descriptor();
            if gradient {
                desc.gradSpanCount = 1;
                desc.gradDataHeight = 1;
            } else {
                desc.tessVertexSpanCount = 1;
                desc.tessDataHeight = 1;
                desc.pathCount = 1;
                desc.contourCount = 1;
            }
            unsafe {
                context.flush(
                    &mut metal,
                    &desc,
                    &mut target,
                    Handle::new(88, MetalObjectKind::CommandBuffer),
                );
            }
            metal.drain_recorded_clone_drops();
            (metal, source)
        };

        for (gradient, ledger_id, encoder_id, pass_id) in [
            (true, "RC-PS-GRAD", "RC-ENC-GRAD", "RC-RPD-GRAD"),
            (false, "RC-PS-TESS", "RC-ENC-TESS", "RC-RPD-TESS"),
        ] {
            let (metal, source) = run(gradient, false);
            let pipeline = assert_clone_triplets(&metal, ledger_id, source, 1);
            assert!(
                event_position(&metal.owner_events, encoder_id, OwnerEventPhase::Release)
                    < event_position(&metal.owner_events, pass_id, OwnerEventPhase::Release)
            );
            assert!(
                event_position(&metal.owner_events, pass_id, OwnerEventPhase::Release)
                    < event_position(&metal.owner_events, ledger_id, OwnerEventPhase::LastUse)
            );
            assert_eq!(
                pipeline[1].selector_ordinal.map(|ordinal| ordinal.0),
                Some("endEncoding")
            );

            let (failed, source) = run(gradient, true);
            assert_eq!(failed.fail_clone_exact, None);
            assert!(row_events(&failed.owner_events, ledger_id).is_empty());
            assert!(!failed.retirements.contains(&source));
            assert!(failed
                .calls
                .iter()
                .any(|call| call.selector == "renderCommandEncoderWithDescriptor:"));
        }
    }

    #[test]
    fn gradient_and_tessellation_pass_expressions_are_distinct_and_failure_exact() {
        let run = |gradient: bool, fail_selector: Option<(&'static str, usize)>| {
            let (mut metal, mut context, mut target) = recording_flush_fixture(false);
            if let Some((selector, offset)) = fail_selector {
                let occurrence = metal.selector_occurrence_count(selector) + offset;
                metal.fail_exact = Some((selector, occurrence));
            }
            let mut desc = empty_source_flush_descriptor();
            if gradient {
                desc.gradSpanCount = 1;
                desc.gradDataHeight = 1;
            } else {
                desc.tessVertexSpanCount = 1;
                desc.tessDataHeight = 1;
                desc.pathCount = 1;
                desc.contourCount = 1;
            }
            unsafe {
                context.flush(
                    &mut metal,
                    &desc,
                    &mut target,
                    Handle::new(88, MetalObjectKind::CommandBuffer),
                );
            }
            metal.drain_recorded_clone_drops();
            metal
        };

        for (gradient, pass_id, attachment_id, encoder_id) in [
            (true, "RC-RPD-GRAD", "RC-RPA-GRAD-0", "RC-ENC-GRAD"),
            (false, "RC-RPD-TESS", "RC-RPA-TESS-0", "RC-ENC-TESS"),
        ] {
            let metal = run(gradient, None);
            let pass = row_events(&metal.owner_events, pass_id);
            let attachment = row_events(&metal.owner_events, attachment_id);
            let encoder = row_events(&metal.owner_events, encoder_id);
            for (row, phases) in [
                (
                    &pass,
                    [
                        OwnerEventPhase::Create,
                        OwnerEventPhase::LastUse,
                        OwnerEventPhase::Release,
                    ],
                ),
                (
                    &encoder,
                    [
                        OwnerEventPhase::Create,
                        OwnerEventPhase::LastUse,
                        OwnerEventPhase::Release,
                    ],
                ),
            ] {
                assert_eq!(
                    row.iter().map(|event| event.phase).collect::<Vec<_>>(),
                    phases
                );
                assert!(row.iter().all(|event| event.handle == row[0].handle));
            }
            assert_eq!(attachment.len(), 9);
            let mut aliases = Vec::new();
            for (index, triple) in attachment.chunks_exact(3).enumerate() {
                assert_eq!(
                    triple.iter().map(|event| event.phase).collect::<Vec<_>>(),
                    vec![
                        OwnerEventPhase::Borrow,
                        OwnerEventPhase::LastUse,
                        OwnerEventPhase::AliasEnd,
                    ]
                );
                assert!(triple.iter().all(|event| {
                    event.handle == triple[0].handle
                        && event.parent_handle == Some(pass[0].handle)
                }));
                assert_eq!(
                    triple[1].selector_ordinal.map(|ordinal| ordinal.0),
                    Some(["setLoadAction:", "setStoreAction:", "setTexture:"][index])
                );
                assert!(!aliases.contains(&triple[0].handle));
                aliases.push(triple[0].handle);
                assert!(metal.retirements.contains(&triple[0].handle));
            }
            let last_alias_end = metal
                .owner_events
                .iter()
                .rposition(|event| {
                    event.ledger_id == attachment_id
                        && event.phase == OwnerEventPhase::AliasEnd
                })
                .unwrap();
            assert!(
                last_alias_end
                    < event_position(
                        &metal.owner_events,
                        encoder_id,
                        OwnerEventPhase::Create,
                    )
            );
            assert!(
                event_position(&metal.owner_events, encoder_id, OwnerEventPhase::Release)
                    < event_position(&metal.owner_events, pass_id, OwnerEventPhase::Release)
            );

            let pass_failed = run(gradient, Some(("renderPassDescriptor", 1)));
            assert_eq!(pass_failed.fail_exact, None);
            assert!(row_events(&pass_failed.owner_events, pass_id).is_empty());

            for occurrence in 1..=3 {
                let attachment_failed =
                    run(gradient, Some(("colorAttachmentAtIndex:", occurrence)));
                assert_eq!(attachment_failed.fail_exact, None);
                assert_eq!(
                    row_events(&attachment_failed.owner_events, attachment_id).len(),
                    6
                );
                assert_eq!(row_events(&attachment_failed.owner_events, pass_id).len(), 3);
            }

            let encoder_failed =
                run(gradient, Some(("renderCommandEncoderWithDescriptor:", 1)));
            assert_eq!(encoder_failed.fail_exact, None);
            assert!(row_events(&encoder_failed.owner_events, encoder_id).is_empty());
            assert_eq!(
                row_events(&encoder_failed.owner_events, attachment_id).len(),
                9
            );
            assert_eq!(row_events(&encoder_failed.owner_events, pass_id).len(), 3);
        }
    }

    #[test]
    fn attachment_collection_getters_are_scoped_parent_tied_and_failure_exact() {
        let run_pipeline = |fail: Option<(&'static str, usize)>| {
            let mut metal = RecordingMetal::default();
            metal.fail_exact = fail;
            let device = metal.device_handle();
            drop(ColorRampPipeline::color_ramp(
                &mut metal,
                device,
                Handle::new(2, MetalObjectKind::Library),
            ));
            metal.drain_recorded_clone_drops();
            metal
        };
        let pipeline = run_pipeline(None);
        let descriptor = row_events(&pipeline.owner_events, "RC-PD-COLOR");
        let collection = row_events(
            &pipeline.owner_events,
            "RC-ATT-COLLECTION-PIPE",
        );
        let child = row_events(&pipeline.owner_events, "RC-ATT-COLOR-0");
        assert_eq!(collection.len(), 3);
        assert_eq!(
            collection.iter().map(|event| event.phase).collect::<Vec<_>>(),
            vec![
                OwnerEventPhase::Borrow,
                OwnerEventPhase::LastUse,
                OwnerEventPhase::AliasEnd,
            ]
        );
        assert!(collection.iter().all(|event| {
            event.handle == collection[0].handle
                && event.handle.kind
                    == MetalObjectKind::RenderPipelineColorAttachmentDescriptorArray
                && event.parent_handle == Some(descriptor[0].handle)
        }));
        assert_eq!(
            collection[0].selector_ordinal,
            Some(("colorAttachments", 1))
        );
        assert_eq!(
            collection[1].selector_ordinal.map(|ordinal| ordinal.0),
            Some("colorAttachmentAtIndex:")
        );
        assert_ne!(collection[0].handle, child[0].handle);
        assert_eq!(child[0].parent_handle, Some(descriptor[0].handle));
        let indexed_call = pipeline
            .calls
            .iter()
            .find(|call| call.selector == "colorAttachmentAtIndex:")
            .unwrap();
        assert_eq!(
            indexed_call.args.get(1),
            Some(&Value::Handle(collection[0].handle))
        );
        assert!(pipeline.retirements.contains(&collection[0].handle));
        assert!(
            event_position(
                &pipeline.owner_events,
                "RC-ATT-COLLECTION-PIPE",
                OwnerEventPhase::AliasEnd,
            ) < event_position(
                &pipeline.owner_events,
                "RC-ATT-COLOR-0",
                OwnerEventPhase::Borrow,
            )
        );

        let getter_failed = run_pipeline(Some(("colorAttachments", 1)));
        assert_eq!(getter_failed.fail_exact, None);
        assert!(row_events(
            &getter_failed.owner_events,
            "RC-ATT-COLLECTION-PIPE"
        )
        .is_empty());
        assert!(row_events(&getter_failed.owner_events, "RC-ATT-COLOR-0").is_empty());
        let child_failed = run_pipeline(Some(("colorAttachmentAtIndex:", 1)));
        assert_eq!(child_failed.fail_exact, None);
        assert_eq!(
            row_events(
                &child_failed.owner_events,
                "RC-ATT-COLLECTION-PIPE"
            )
            .len(),
            3
        );
        assert!(row_events(&child_failed.owner_events, "RC-ATT-COLOR-0").is_empty());

        let run_pass = |fail: Option<(&'static str, usize)>| {
            let (mut metal, mut context, mut target) = recording_flush_fixture(false);
            if let Some((selector, offset)) = fail {
                let occurrence = metal.selector_occurrence_count(selector) + offset;
                metal.fail_exact = Some((selector, occurrence));
            }
            let mut desc = empty_source_flush_descriptor();
            desc.gradSpanCount = 1;
            desc.gradDataHeight = 1;
            unsafe {
                context.flush(
                    &mut metal,
                    &desc,
                    &mut target,
                    Handle::new(88, MetalObjectKind::CommandBuffer),
                );
            }
            metal.drain_recorded_clone_drops();
            metal
        };
        let pass = run_pass(None);
        let descriptor = row_events(&pass.owner_events, "RC-RPD-GRAD");
        let collections = row_events(&pass.owner_events, "RC-ATT-COLLECTION-PASS")
            .into_iter()
            .filter(|event| event.parent_handle == Some(descriptor[0].handle))
            .collect::<Vec<_>>();
        let children = row_events(&pass.owner_events, "RC-RPA-GRAD-0");
        assert_eq!(collections.len(), 9);
        for (index, triple) in collections.chunks_exact(3).enumerate() {
            assert_eq!(
                triple.iter().map(|event| event.phase).collect::<Vec<_>>(),
                vec![
                    OwnerEventPhase::Borrow,
                    OwnerEventPhase::LastUse,
                    OwnerEventPhase::AliasEnd,
                ]
            );
            assert!(triple.iter().all(|event| {
                event.handle == triple[0].handle
                    && event.handle.kind
                        == MetalObjectKind::RenderPassColorAttachmentDescriptorArray
                    && event.parent_handle == Some(descriptor[0].handle)
            }));
            assert_eq!(
                triple[0].selector_ordinal.map(|ordinal| ordinal.0),
                Some("colorAttachments")
            );
            assert_eq!(
                triple[1].selector_ordinal.map(|ordinal| ordinal.0),
                Some("colorAttachmentAtIndex:")
            );
            assert_ne!(triple[0].handle, children[index * 3].handle);
            let indexed_call = pass
                .calls
                .iter()
                .filter(|call| call.selector == "colorAttachmentAtIndex:")
                .filter(|call| {
                    call.args.first() == Some(&Value::Handle(descriptor[0].handle))
                })
                .nth(index)
                .unwrap();
            assert_eq!(
                indexed_call.args.get(1),
                Some(&Value::Handle(triple[0].handle))
            );
            assert!(pass.retirements.contains(&triple[0].handle));
            let collection_end = pass
                .owner_events
                .iter()
                .position(|event| {
                    event.ledger_id == "RC-ATT-COLLECTION-PASS"
                        && event.phase == OwnerEventPhase::AliasEnd
                        && event.handle == triple[0].handle
                })
                .unwrap();
            let child_borrow = pass
                .owner_events
                .iter()
                .position(|event| {
                    event.ledger_id == "RC-RPA-GRAD-0"
                        && event.phase == OwnerEventPhase::Borrow
                        && event.handle == children[index * 3].handle
                })
                .unwrap();
            assert!(collection_end < child_borrow);
        }
        assert_eq!(
            collections
                .chunks_exact(3)
                .map(|triple| {
                    let handle = triple[0].handle;
                    (handle.registry, handle.slot, handle.generation)
                })
                .collect::<BTreeSet<_>>()
                .len(),
            3
        );

        for selector in ["colorAttachments", "colorAttachmentAtIndex:"] {
            for occurrence in 1..=3 {
                let failed = run_pass(Some((selector, occurrence)));
                assert_eq!(failed.fail_exact, None, "{selector}#{occurrence}");
                let failed_descriptor = row_events(&failed.owner_events, "RC-RPD-GRAD");
                let collection_count = row_events(
                    &failed.owner_events,
                    "RC-ATT-COLLECTION-PASS",
                )
                .into_iter()
                .filter(|event| {
                    event.parent_handle == Some(failed_descriptor[0].handle)
                })
                .count();
                let child_count = row_events(&failed.owner_events, "RC-RPA-GRAD-0").len();
                assert_eq!(
                    collection_count,
                    if selector == "colorAttachments" { 6 } else { 9 }
                );
                assert_eq!(child_count, 6);
            }
        }
    }

    #[test]
    fn main_pass_attachment_expressions_and_copy_encoder_are_source_exact() {
        let run = |load_action: gpu::LoadAction,
                   fail: Option<(&'static str, usize)>| {
            let (mut metal, mut context, mut target) = recording_flush_fixture(false);
            if let Some((selector, offset)) = fail {
                let occurrence = metal.selector_occurrence_count(selector) + offset;
                metal.fail_exact = Some((selector, occurrence));
            }
            let mut desc = empty_source_flush_descriptor();
            desc.colorLoadAction = load_action;
            unsafe {
                context.flush(
                    &mut metal,
                    &desc,
                    &mut target,
                    Handle::new(88, MetalObjectKind::CommandBuffer),
                );
            }
            metal.drain_recorded_clone_drops();
            metal
        };

        let metal = run(gpu::LoadAction::Clear, None);
        let pass = row_events(&metal.owner_events, "RC-RPD-MAIN");
        assert_eq!(
            pass.iter().map(|event| event.phase).collect::<Vec<_>>(),
            vec![
                OwnerEventPhase::Create,
                OwnerEventPhase::LastUse,
                OwnerEventPhase::Release,
            ]
        );
        let expected = [
            (
                "RC-RPA-MAIN-COLOR",
                ["setTexture:", "setLoadAction:", "setClearColor:", "setStoreAction:"].as_slice(),
            ),
            (
                "RC-RPA-MAIN-CLIP",
                ["setTexture:", "setLoadAction:", "setClearColor:", "setStoreAction:"].as_slice(),
            ),
            (
                "RC-RPA-MAIN-SCRATCH",
                ["setTexture:", "setLoadAction:", "setStoreAction:"].as_slice(),
            ),
            (
                "RC-RPA-MAIN-COVERAGE",
                ["setTexture:", "setLoadAction:", "setClearColor:", "setStoreAction:"].as_slice(),
            ),
        ];
        let mut all_aliases = BTreeSet::new();
        for (id, setters) in expected {
            let aliases = row_events(&metal.owner_events, id);
            assert_eq!(aliases.len(), setters.len() * 3, "{id} multiplicity");
            for (index, triple) in aliases.chunks_exact(3).enumerate() {
                assert_eq!(
                    triple.iter().map(|event| event.phase).collect::<Vec<_>>(),
                    vec![
                        OwnerEventPhase::Borrow,
                        OwnerEventPhase::LastUse,
                        OwnerEventPhase::AliasEnd,
                    ]
                );
                assert!(triple.iter().all(|event| {
                    event.handle == triple[0].handle
                        && event.parent_handle == Some(pass[0].handle)
                }));
                assert_eq!(
                    triple[1].selector_ordinal.map(|ordinal| ordinal.0),
                    Some(setters[index])
                );
                let handle = triple[0].handle;
                assert!(all_aliases.insert((handle.registry, handle.slot, handle.generation)));
                let setter_index = metal
                    .calls
                    .iter()
                    .position(|call| {
                        call.selector == setters[index]
                            && call.args.first() == Some(&Value::Handle(handle))
                    })
                    .unwrap();
                assert_eq!(
                    metal
                        .retirement_call_counts
                        .iter()
                        .filter(|(retired, _)| *retired == handle)
                        .copied()
                        .collect::<Vec<_>>(),
                    vec![(handle, setter_index + 1)],
                    "{id} alias must retire immediately after its setter"
                );
                let alias_end_position = metal
                    .owner_events
                    .iter()
                    .position(|event| {
                        event.ledger_id == id
                            && event.phase == OwnerEventPhase::AliasEnd
                            && event.handle == handle
                    })
                    .unwrap();
                assert_eq!(
                    metal
                        .retirement_event_counts
                        .iter()
                        .find_map(|(retired, count)| {
                            (*retired == handle).then_some(*count)
                        }),
                    Some(alias_end_position),
                    "{id} native alias ends between LastUse and AliasEnd telemetry"
                );
            }
        }
        let last_attachment_end = metal
            .owner_events
            .iter()
            .rposition(|event| {
                event.ledger_id.starts_with("RC-RPA-MAIN-")
                    && event.phase == OwnerEventPhase::AliasEnd
            })
            .unwrap();
        assert!(
            last_attachment_end
                < event_position(
                    &metal.owner_events,
                    "RC-RPD-MAIN",
                    OwnerEventPhase::LastUse,
                )
        );
        let main_encoder = row_events(&metal.owner_events, "RC-ENC-MAIN");
        let final_encoder = main_encoder
            .iter()
            .find(|event| event.phase == OwnerEventPhase::Release)
            .unwrap()
            .handle;
        let final_end = metal
            .calls
            .iter()
            .rposition(|call| {
                call.selector == "endEncoding"
                    && call.args.first() == Some(&Value::Handle(final_encoder))
            })
            .unwrap();
        assert_eq!(
            metal
                .retirement_call_counts
                .iter()
                .filter(|(handle, _)| *handle == pass[0].handle)
                .copied()
                .collect::<Vec<_>>(),
            vec![(pass[0].handle, final_end + 1)]
        );
        let final_encoder_retirement = metal
            .retirements
            .iter()
            .position(|handle| *handle == final_encoder)
            .unwrap();
        let pass_retirement = metal
            .retirements
            .iter()
            .position(|handle| *handle == pass[0].handle)
            .unwrap();
        assert!(final_encoder_retirement < pass_retirement);
        assert_eq!(
            metal
                .retirement_event_counts
                .iter()
                .find_map(|(handle, count)| {
                    (*handle == pass[0].handle).then_some(*count)
                }),
            Some(event_position(
                &metal.owner_events,
                "RC-RPD-MAIN",
                OwnerEventPhase::Release,
            ))
        );

        for load_action in [
            gpu::LoadAction::PreserveRenderTarget,
            gpu::LoadAction::DontCare,
        ] {
            let variant = run(load_action, None);
            let color = row_events(&variant.owner_events, "RC-RPA-MAIN-COLOR");
            assert_eq!(color.len(), 9);
            assert_eq!(
                color
                    .chunks_exact(3)
                    .map(|triple| triple[1].selector_ordinal.unwrap().0)
                    .collect::<Vec<_>>(),
                vec!["setTexture:", "setLoadAction:", "setStoreAction:"]
            );
        }

        let pass_failed = run(gpu::LoadAction::Clear, Some(("renderPassDescriptor", 1)));
        assert_eq!(pass_failed.fail_exact, None);
        assert!(row_events(&pass_failed.owner_events, "RC-RPD-MAIN").is_empty());
        for selector in ["colorAttachments", "colorAttachmentAtIndex:"] {
            for occurrence in 1..=15 {
                let failed = run(gpu::LoadAction::Clear, Some((selector, occurrence)));
                assert_eq!(failed.fail_exact, None, "{selector}#{occurrence}");
                let child_events = [
                    "RC-RPA-MAIN-COLOR",
                    "RC-RPA-MAIN-CLIP",
                    "RC-RPA-MAIN-SCRATCH",
                    "RC-RPA-MAIN-COVERAGE",
                ]
                .into_iter()
                .map(|id| row_events(&failed.owner_events, id).len())
                .sum::<usize>();
                assert_eq!(child_events, 14 * 3);
                let collection_events = row_events(
                    &failed.owner_events,
                    "RC-ATT-COLLECTION-PASS",
                )
                .len();
                assert_eq!(
                    collection_events,
                    if selector == "colorAttachments" { 14 * 3 } else { 15 * 3 }
                );
            }
        }

        let run_copy = |fail: bool| {
            let (mut metal, mut context, mut target) = recording_flush_fixture(false);
            if fail {
                metal.fail_exact = Some(("blitCommandEncoder", 1));
            }
            let mut desc = empty_source_flush_descriptor();
            desc.interlockMode = gpu::InterlockMode::Atomics;
            desc.colorLoadAction = gpu::LoadAction::PreserveRenderTarget;
            unsafe {
                context.flush(
                    &mut metal,
                    &desc,
                    &mut target,
                    Handle::new(88, MetalObjectKind::CommandBuffer),
                );
            }
            metal.drain_recorded_clone_drops();
            metal
        };
        let copied = run_copy(false);
        let copy = row_events(&copied.owner_events, "RC-ENC-COPY");
        assert_eq!(
            copy.iter().map(|event| event.phase).collect::<Vec<_>>(),
            vec![
                OwnerEventPhase::Create,
                OwnerEventPhase::LastUse,
                OwnerEventPhase::Release,
            ]
        );
        assert!(copy.iter().all(|event| event.handle == copy[0].handle));
        assert_eq!(
            copy[1].selector_ordinal.map(|ordinal| ordinal.0),
            Some("endEncoding")
        );
        assert!(
            event_position(&copied.owner_events, "RC-ENC-COPY", OwnerEventPhase::Release)
                < event_position(&copied.owner_events, "RC-ENC-HELPER", OwnerEventPhase::Create)
        );
        let copy_end = copied
            .calls
            .iter()
            .position(|call| {
                call.selector == "endEncoding"
                    && call.args.first() == Some(&Value::Handle(copy[0].handle))
            })
            .unwrap();
        assert_eq!(
            copied
                .retirement_call_counts
                .iter()
                .filter(|(handle, _)| *handle == copy[0].handle)
                .copied()
                .collect::<Vec<_>>(),
            vec![(copy[0].handle, copy_end + 1)]
        );
        assert_eq!(
            copied
                .retirement_event_counts
                .iter()
                .find_map(|(handle, count)| {
                    (*handle == copy[0].handle).then_some(*count)
                }),
            Some(event_position(
                &copied.owner_events,
                "RC-ENC-COPY",
                OwnerEventPhase::Release,
            ))
        );
        let helper_create_call = copied
            .calls
            .iter()
            .position(|call| call.selector == "renderCommandEncoderWithDescriptor:")
            .unwrap();
        assert!(copy_end < helper_create_call);
        let copy_failed = run_copy(true);
        assert_eq!(copy_failed.fail_exact, None);
        assert!(row_events(&copy_failed.owner_events, "RC-ENC-COPY").is_empty());
    }

    #[test]
    fn main_encoder_helper_handoff_and_pass_break_replacement_are_source_exact() {
        let run = |break_count: usize, fail: Option<(&'static str, usize)>| {
            let (mut metal, mut context, mut target) = recording_flush_fixture(false);
            context.set_atomic_barrier_for_test(
                crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::AtomicBarrierType::renderPassBreak,
            );
            let device = metal.device_handle();
            let pipeline = DrawPipeline::new(
                &mut metal,
                device,
                Some(Handle::new(2, MetalObjectKind::Library)),
                SourceFunctionName::Static(DRAW_VERTEX_NAME),
                SourceFunctionName::Static(DRAW_FRAGMENT_NAME),
                DrawType::MidpointFanPatches,
                InterlockMode::Atomics,
                ShaderFeatures(1),
                ShaderMiscFlags::none,
                SynthesizedFailureType::none,
            );
            context.seed_pipeline_for_test(
                shader_key_for_test(
                    DrawType::MidpointFanPatches,
                    ShaderFeatures(1),
                    InterlockMode::Atomics,
                    ShaderMiscFlags::none,
                ),
                pipeline,
            );
            metal.owner_events.clear();
            metal.retirements.clear();
            metal.retirement_call_counts.clear();
            metal.retirement_event_counts.clear();
            metal.calls.clear();
            if let Some((selector, offset)) = fail {
                let occurrence = metal.selector_occurrence_count(selector) + offset;
                metal.fail_exact = Some((selector, occurrence));
            }
            let mut draws = gpu::BlockAllocatedLinkedList::default();
            for base in 0..break_count.max(1) {
                let mut batch = gpu::DrawBatch::new(
                    gpu::DrawType::midpointFanPatches,
                    gpu::ShaderMiscFlags::none,
                    gpu::DrawContents::none,
                    1,
                    base as u32,
                    nuxie_render_api::BlendMode::SrcOver,
                    gpu::ImageSampler::LinearClamp(),
                    if base < break_count {
                        gpu::BarrierFlags::plsAtomic
                    } else {
                        gpu::BarrierFlags::none
                    },
                );
                batch.indexCountPerInstance = 3;
                draws.push_back(batch);
            }
            let mut desc = empty_source_flush_descriptor();
            desc.combinedShaderFeatures = gpu::ShaderFeatures(1);
            desc.interlockMode = gpu::InterlockMode::Atomics;
            desc.pathCount = 1;
            desc.drawList = Some(core::ptr::NonNull::from(&draws));
            unsafe {
                context.flush(
                    &mut metal,
                    &desc,
                    &mut target,
                    Handle::new(88, MetalObjectKind::CommandBuffer),
                );
            }
            metal.drain_recorded_clone_drops();
            metal
        };

        for break_count in 0..=2 {
            let metal = run(break_count, None);
            let generation_count = break_count + 1;
            let helper = row_events(&metal.owner_events, "RC-ENC-HELPER");
            let main = row_events(&metal.owner_events, "RC-ENC-MAIN");
            let pass = row_events(&metal.owner_events, "RC-RPD-MAIN");
            assert_eq!(helper.len(), generation_count * 2);
            assert_eq!(main.len(), generation_count * 3);
            assert_eq!(pass.len(), generation_count + 2);
            assert_eq!(pass[0].phase, OwnerEventPhase::Create);
            assert_eq!(pass.last().unwrap().phase, OwnerEventPhase::Release);
            assert!(pass[1..pass.len() - 1]
                .iter()
                .all(|event| event.phase == OwnerEventPhase::LastUse
                    && event.handle == pass[0].handle
                    && event.selector_ordinal.map(|ordinal| ordinal.0)
                        == Some("renderCommandEncoderWithDescriptor:")));

            let mut generations = Vec::new();
            for pair in helper.chunks_exact(2) {
                assert_eq!(pair[0].phase, OwnerEventPhase::Create);
                assert_eq!(pair[1].phase, OwnerEventPhase::Transfer);
                assert_eq!(pair[0].handle, pair[1].handle);
                assert_eq!(pair[0].native_identity, pair[1].native_identity);
                assert_eq!(
                    metal
                        .retirements
                        .iter()
                        .filter(|handle| **handle == pair[0].handle)
                        .count(),
                    1,
                    "the helper transfers its sole owner for caller release"
                );
                generations.push(pair[0].handle);
            }
            assert_eq!(
                generations
                    .iter()
                    .map(|handle| (handle.registry, handle.slot, handle.generation))
                    .collect::<BTreeSet<_>>()
                    .len(),
                generation_count
            );
            for handle in &generations {
                let phases = main
                    .iter()
                    .filter(|event| event.handle == *handle)
                    .map(|event| event.phase)
                    .collect::<Vec<_>>();
                assert_eq!(
                    phases,
                    vec![
                        OwnerEventPhase::Transfer,
                        OwnerEventPhase::LastUse,
                        OwnerEventPhase::Release,
                    ]
                );
                assert_eq!(
                    metal
                        .retirement_call_counts
                        .iter()
                        .filter(|(retired, _)| retired == handle)
                        .count(),
                    1,
                    "each caller encoder generation retires exactly once"
                );
                let release_position = metal
                    .owner_events
                    .iter()
                    .position(|event| {
                        event.ledger_id == "RC-ENC-MAIN"
                            && event.phase == OwnerEventPhase::Release
                            && event.handle == *handle
                    })
                    .unwrap();
                assert_eq!(
                    metal
                        .retirement_event_counts
                        .iter()
                        .find_map(|(retired, count)| {
                            (*retired == *handle).then_some(*count)
                        }),
                    Some(release_position),
                    "caller retirement must occur after helper/main handoff events and before Release telemetry"
                );
            }
            for index in 0..break_count {
                let old = generations[index];
                let new = generations[index + 1];
                let old_last_use = metal
                    .owner_events
                    .iter()
                    .position(|event| {
                        event.ledger_id == "RC-ENC-MAIN"
                            && event.phase == OwnerEventPhase::LastUse
                            && event.handle == old
                    })
                    .unwrap();
                let break_alias = row_events(&metal.owner_events, "RC-RPA-MAIN-COLOR")
                    .chunks_exact(3)
                    .nth(3 + index)
                    .unwrap()
                    .to_vec();
                let alias_start = metal
                    .owner_events
                    .iter()
                    .position(|event| core::ptr::eq(event, break_alias[0]))
                    .unwrap();
                let alias_end = metal
                    .owner_events
                    .iter()
                    .position(|event| core::ptr::eq(event, break_alias[2]))
                    .unwrap();
                let helper_create = metal
                    .owner_events
                    .iter()
                    .position(|event| {
                        event.ledger_id == "RC-ENC-HELPER"
                            && event.phase == OwnerEventPhase::Create
                            && event.handle == new
                    })
                    .unwrap();
                let main_transfer = metal
                    .owner_events
                    .iter()
                    .position(|event| {
                        event.ledger_id == "RC-ENC-MAIN"
                            && event.phase == OwnerEventPhase::Transfer
                            && event.handle == new
                    })
                    .unwrap();
                let old_release = metal
                    .owner_events
                    .iter()
                    .position(|event| {
                        event.ledger_id == "RC-ENC-MAIN"
                            && event.phase == OwnerEventPhase::Release
                            && event.handle == old
                    })
                    .unwrap();
                assert!(old_last_use < alias_start);
                assert!(alias_start < alias_end);
                assert!(alias_end < helper_create);
                assert!(helper_create < main_transfer);
                assert!(main_transfer < old_release);
                assert_eq!(
                    break_alias[1].selector_ordinal.map(|ordinal| ordinal.0),
                    Some("setLoadAction:")
                );
                let replacement_occurrence = helper[index * 2 + 2]
                    .selector_ordinal
                    .expect("replacement helper selector ordinal")
                    .1;
                let replacement_creation = metal
                    .calls
                    .iter()
                    .enumerate()
                    .filter(|(_, call)| {
                        call.selector == "renderCommandEncoderWithDescriptor:"
                    })
                    .nth(replacement_occurrence - 1)
                    .map(|(call_index, _)| call_index)
                    .unwrap();
                let old_retirement_count = metal
                    .retirement_call_counts
                    .iter()
                    .find_map(|(handle, count)| (*handle == old).then_some(*count))
                    .unwrap();
                let old_retirement_event_count = metal
                    .retirement_event_counts
                    .iter()
                    .find_map(|(handle, count)| (*handle == old).then_some(*count))
                    .unwrap();
                assert!(
                    old_retirement_count > replacement_creation + 1,
                    "old encoder must survive the complete replacement helper RHS"
                );
                assert_eq!(
                    metal.calls[old_retirement_count].selector,
                    "setRenderPipelineState:",
                    "old encoder retires at assignment before the new generation's first draw"
                );
                assert!(
                    old_retirement_event_count > main_transfer,
                    "old encoder must remain owned through the selector-free caller transfer"
                );
            }
            let final_encoder = *generations.last().unwrap();
            let final_end = metal
                .calls
                .iter()
                .rposition(|call| {
                    call.selector == "endEncoding"
                        && call.args.first() == Some(&Value::Handle(final_encoder))
                })
                .unwrap();
            assert_eq!(
                metal
                    .retirement_call_counts
                    .iter()
                    .find_map(|(handle, count)| {
                        (*handle == final_encoder).then_some(*count)
                    }),
                Some(final_end + 1)
            );
            assert_eq!(
                metal
                    .retirement_call_counts
                    .iter()
                    .find_map(|(handle, count)| {
                        (*handle == pass[0].handle).then_some(*count)
                    }),
                Some(final_end + 1)
            );
            assert!(
                event_position(&metal.owner_events, "RC-ENC-MAIN", OwnerEventPhase::Release)
                    < event_position(&metal.owner_events, "RC-RPD-MAIN", OwnerEventPhase::Release)
            );
        }

        for break_count in 0..=2 {
            for occurrence in 1..=(break_count + 1) {
                let failed = run(
                    break_count,
                    Some(("renderCommandEncoderWithDescriptor:", occurrence)),
                );
                assert_eq!(failed.fail_exact, None);
                assert_eq!(
                    row_events(&failed.owner_events, "RC-ENC-HELPER").len(),
                    break_count * 2
                );
                assert_eq!(
                    row_events(&failed.owner_events, "RC-ENC-MAIN").len(),
                    break_count * 3
                );
            }
        }
        for selector in ["colorAttachments", "colorAttachmentAtIndex:"] {
            for break_count in 1..=2 {
                for break_index in 0..break_count {
                    let failed = run(
                        break_count,
                        Some((selector, 4 + break_index)),
                    );
                    assert_eq!(failed.fail_exact, None);
                    assert_eq!(
                        row_events(&failed.owner_events, "RC-RPA-MAIN-COLOR").len(),
                        (3 + break_count - 1) * 3
                    );
                }
            }
        }
    }

    #[test]
    fn atlas_pipeline_locals_cover_fill_only_combined_reverse_drop_and_clone_failures() {
        let run = |fill_count: usize, stroke_count: usize, fail_clone: Option<usize>| {
            let (mut metal, mut context, mut target) = recording_flush_fixture(true);
            let (fill_source, stroke_source) = context.feather_pipeline_states_for_test();
            let fill_source = fill_source.expect("lazy feather fill pipeline state");
            let stroke_source = stroke_source.expect("lazy feather stroke pipeline state");
            if let Some(occurrence) = fail_clone {
                metal.fail_clone_exact =
                    Some((MetalObjectKind::RenderPipelineState, occurrence));
            }
            let fill = gpu::AtlasDrawBatch {
                scissor: gpu::AABBu16 {
                    left: 0,
                    top: 0,
                    right: 4,
                    bottom: 4,
                },
                patchCount: 1,
                basePatch: 0,
            };
            let stroke = fill;
            let mut desc = empty_source_flush_descriptor();
            desc.featherAtlasFillBatches =
                (fill_count != 0).then(|| core::ptr::NonNull::from(&fill));
            desc.featherAtlasFillBatchCount = fill_count;
            desc.featherAtlasStrokeBatches =
                (stroke_count != 0).then(|| core::ptr::NonNull::from(&stroke));
            desc.featherAtlasStrokeBatchCount = stroke_count;
            unsafe {
                context.flush(
                    &mut metal,
                    &desc,
                    &mut target,
                    Handle::new(88, MetalObjectKind::CommandBuffer),
                );
            }
            metal.drain_recorded_clone_drops();
            (metal, fill_source, stroke_source)
        };

        let (fill_only, fill_source, stroke_source) = run(1, 0, None);
        let fill = assert_clone_triplets(&fill_only, "RC-PS-ATLAS-FILL", fill_source, 1);
        let stroke =
            assert_clone_triplets(&fill_only, "RC-PS-ATLAS-STROKE", stroke_source, 1);
        let pipeline_binds = |handle| {
            fill_only
                .calls
                .iter()
                .filter(|call| {
                    call.selector == "setRenderPipelineState:"
                        && call.args.contains(&Value::Handle(handle))
                })
                .count()
        };
        assert_eq!(pipeline_binds(fill[0].handle), 1);
        assert_eq!(pipeline_binds(stroke[0].handle), 0);

        let (stroke_only, fill_source, stroke_source) = run(0, 1, None);
        let fill =
            assert_clone_triplets(&stroke_only, "RC-PS-ATLAS-FILL", fill_source, 1);
        let stroke = assert_clone_triplets(
            &stroke_only,
            "RC-PS-ATLAS-STROKE",
            stroke_source,
            1,
        );
        let pipeline_binds = |handle| {
            stroke_only
                .calls
                .iter()
                .filter(|call| {
                    call.selector == "setRenderPipelineState:"
                        && call.args.contains(&Value::Handle(handle))
                })
                .count()
        };
        assert_eq!(pipeline_binds(fill[0].handle), 0);
        assert_eq!(pipeline_binds(stroke[0].handle), 1);
        assert!(
            event_position(
                &stroke_only.owner_events,
                "RC-RPD-ATLAS",
                OwnerEventPhase::Release,
            ) < event_position(
                &stroke_only.owner_events,
                "RC-PS-ATLAS-STROKE",
                OwnerEventPhase::LastUse,
            )
        );
        assert!(
            event_position(
                &stroke_only.owner_events,
                "RC-PS-ATLAS-STROKE",
                OwnerEventPhase::Release,
            ) < event_position(
                &stroke_only.owner_events,
                "RC-PS-ATLAS-FILL",
                OwnerEventPhase::LastUse,
            )
        );

        let (combined, fill_source, stroke_source) = run(1, 1, None);
        let fill = assert_clone_triplets(&combined, "RC-PS-ATLAS-FILL", fill_source, 1);
        let stroke =
            assert_clone_triplets(&combined, "RC-PS-ATLAS-STROKE", stroke_source, 1);
        for owner in [fill[0].handle, stroke[0].handle] {
            assert_eq!(
                combined
                    .calls
                    .iter()
                    .filter(|call| call.selector == "setRenderPipelineState:"
                        && call.args.contains(&Value::Handle(owner)))
                    .count(),
                1
            );
        }
        assert!(
            event_position(
                &combined.owner_events,
                "RC-RPD-ATLAS",
                OwnerEventPhase::Release,
            ) < event_position(
                &combined.owner_events,
                "RC-PS-ATLAS-STROKE",
                OwnerEventPhase::LastUse,
            )
        );
        assert!(
            event_position(
                &combined.owner_events,
                "RC-PS-ATLAS-STROKE",
                OwnerEventPhase::Release,
            ) < event_position(
                &combined.owner_events,
                "RC-PS-ATLAS-FILL",
                OwnerEventPhase::LastUse,
            )
        );

        let (fill_failed, fill_source, stroke_source) = run(1, 1, Some(1));
        assert_eq!(fill_failed.fail_clone_exact, None);
        assert!(row_events(&fill_failed.owner_events, "RC-PS-ATLAS-FILL").is_empty());
        assert_clone_triplets(
            &fill_failed,
            "RC-PS-ATLAS-STROKE",
            stroke_source,
            1,
        );
        assert!(!fill_failed.retirements.contains(&fill_source));

        let (stroke_failed, fill_source, stroke_source) = run(1, 1, Some(2));
        assert_eq!(stroke_failed.fail_clone_exact, None);
        assert_clone_triplets(
            &stroke_failed,
            "RC-PS-ATLAS-FILL",
            fill_source,
            1,
        );
        assert!(row_events(&stroke_failed.owner_events, "RC-PS-ATLAS-STROKE").is_empty());
        assert!(!stroke_failed.retirements.contains(&stroke_source));
    }

    #[test]
    fn atlas_pass_attachment_and_encoder_follow_parent_scope_and_exact_failures() {
        let run = |fill_count: usize,
                   stroke_count: usize,
                   fail_selector: Option<(&'static str, usize)>| {
            let (mut metal, mut context, mut target) = recording_flush_fixture(true);
            if let Some((selector, offset)) = fail_selector {
                let occurrence = metal.selector_occurrence_count(selector) + offset;
                metal.fail_exact = Some((selector, occurrence));
            }
            let fill = gpu::AtlasDrawBatch {
                scissor: gpu::AABBu16 {
                    left: 0,
                    top: 0,
                    right: 4,
                    bottom: 4,
                },
                patchCount: 1,
                basePatch: 0,
            };
            let stroke = fill;
            let mut desc = empty_source_flush_descriptor();
            desc.featherAtlasFillBatches =
                (fill_count != 0).then(|| core::ptr::NonNull::from(&fill));
            desc.featherAtlasFillBatchCount = fill_count;
            desc.featherAtlasStrokeBatches =
                (stroke_count != 0).then(|| core::ptr::NonNull::from(&stroke));
            desc.featherAtlasStrokeBatchCount = stroke_count;
            unsafe {
                context.flush(
                    &mut metal,
                    &desc,
                    &mut target,
                    Handle::new(88, MetalObjectKind::CommandBuffer),
                );
            }
            metal.drain_recorded_clone_drops();
            metal
        };

        for (fill_count, stroke_count) in [(1, 0), (0, 1), (1, 1)] {
            let metal = run(fill_count, stroke_count, None);
            let pass = row_events(&metal.owner_events, "RC-RPD-ATLAS");
            let attachment = row_events(&metal.owner_events, "RC-RPA-ATLAS-0");
            let encoder = row_events(&metal.owner_events, "RC-ENC-ATLAS");
            assert_eq!(
                pass.iter().map(|event| event.phase).collect::<Vec<_>>(),
                vec![
                    OwnerEventPhase::Create,
                    OwnerEventPhase::LastUse,
                    OwnerEventPhase::Release,
                ]
            );
            assert_eq!(attachment.len(), 12);
            for (index, triple) in attachment.chunks_exact(3).enumerate() {
                assert_eq!(
                    triple.iter().map(|event| event.phase).collect::<Vec<_>>(),
                    vec![
                        OwnerEventPhase::Borrow,
                        OwnerEventPhase::LastUse,
                        OwnerEventPhase::AliasEnd,
                    ]
                );
                assert!(triple.iter().all(|event| {
                    event.handle == triple[0].handle
                        && event.parent_handle == Some(pass[0].handle)
                }));
                assert_eq!(
                    triple[1].selector_ordinal.map(|ordinal| ordinal.0),
                    Some([
                        "setLoadAction:",
                        "setStoreAction:",
                        "setTexture:",
                        "setClearColor:",
                    ][index])
                );
            }
            let attachment_handles = attachment
                .chunks_exact(3)
                .map(|triple| triple[0].handle)
                .collect::<Vec<_>>();
            assert!(attachment_handles
                .iter()
                .enumerate()
                .all(|(index, handle)| !attachment_handles[..index].contains(handle)));
            let last_attachment_end = metal
                .owner_events
                .iter()
                .rposition(|event| {
                    event.ledger_id == "RC-RPA-ATLAS-0"
                        && event.phase == OwnerEventPhase::AliasEnd
                })
                .unwrap();
            assert!(
                last_attachment_end
                    < event_position(
                        &metal.owner_events,
                        "RC-ENC-ATLAS",
                        OwnerEventPhase::Create,
                    )
            );
            assert_eq!(
                encoder
                    .iter()
                    .map(|event| event.phase)
                    .collect::<Vec<_>>(),
                vec![
                    OwnerEventPhase::Create,
                    OwnerEventPhase::LastUse,
                    OwnerEventPhase::Release,
                ]
            );
            assert!(pass.iter().all(|event| event.handle == pass[0].handle));
            assert!(encoder
                .iter()
                .all(|event| event.handle == encoder[0].handle));
            assert!(
                event_position(&metal.owner_events, "RC-ENC-ATLAS", OwnerEventPhase::Release)
                    < event_position(
                    &metal.owner_events,
                    "RC-RPD-ATLAS",
                    OwnerEventPhase::Release,
                )
            );
            assert!(metal.retirements.contains(&encoder[0].handle));
            assert!(attachment
                .chunks_exact(3)
                .all(|triple| metal.retirements.contains(&triple[0].handle)));
            assert!(metal.retirements.contains(&pass[0].handle));
        }

        let pass_failed = run(1, 1, Some(("renderPassDescriptor", 1)));
        assert_eq!(pass_failed.fail_exact, None);
        assert!(row_events(&pass_failed.owner_events, "RC-RPD-ATLAS").is_empty());

        for occurrence in 1..=4 {
            let attachment_failed =
                run(1, 1, Some(("colorAttachmentAtIndex:", occurrence)));
            assert_eq!(attachment_failed.fail_exact, None);
            let attachment = row_events(
                &attachment_failed.owner_events,
                "RC-RPA-ATLAS-0",
            );
            assert_eq!(attachment.len(), 9);
            for triple in attachment.chunks_exact(3) {
                assert_eq!(
                    triple.iter().map(|event| event.phase).collect::<Vec<_>>(),
                    vec![
                        OwnerEventPhase::Borrow,
                        OwnerEventPhase::LastUse,
                        OwnerEventPhase::AliasEnd,
                    ]
                );
            }
            assert_eq!(
                row_events(&attachment_failed.owner_events, "RC-RPD-ATLAS").len(),
                3
            );
        }

        let encoder_failed = run(1, 1, Some(("renderCommandEncoderWithDescriptor:", 1)));
        assert_eq!(encoder_failed.fail_exact, None);
        assert!(row_events(&encoder_failed.owner_events, "RC-ENC-ATLAS").is_empty());
        assert_eq!(
            row_events(&encoder_failed.owner_events, "RC-RPA-ATLAS-0").len(),
            12
        );
        assert_eq!(
            row_events(&encoder_failed.owner_events, "RC-RPD-ATLAS").len(),
            3
        );
    }

    #[test]
    fn per_batch_pipeline_local_releases_before_the_next_batch_and_on_mesh_cast_break() {
        let run_path_batches = |fail_clone: Option<usize>| {
            let (mut metal, mut context, mut target) = recording_flush_fixture(false);
            let device = metal.device_handle();
            let pipeline = DrawPipeline::new(
                &mut metal,
                device,
                Some(Handle::new(2, MetalObjectKind::Library)),
                SourceFunctionName::Static(DRAW_VERTEX_NAME),
                SourceFunctionName::Static(DRAW_FRAGMENT_NAME),
                DrawType::MidpointFanPatches,
                InterlockMode::Atomics,
                ShaderFeatures(1),
                ShaderMiscFlags::none,
                SynthesizedFailureType::none,
            );
            let source = pipeline
                .rgba8
                .as_ref()
                .map(OwnedMetalHandle::handle)
                .expect("RGBA draw pipeline state");
            context.seed_pipeline_for_test(
                shader_key_for_test(
                    DrawType::MidpointFanPatches,
                    ShaderFeatures(1),
                    InterlockMode::Atomics,
                    ShaderMiscFlags::none,
                ),
                pipeline,
            );
            metal.owner_events.clear();
            metal.retirements.clear();
            metal.retirement_call_counts.clear();
            metal.retirement_event_counts.clear();
            metal.calls.clear();
            if let Some(occurrence) = fail_clone {
                metal.fail_clone_exact =
                    Some((MetalObjectKind::RenderPipelineState, occurrence));
            }
            let mut draws = gpu::BlockAllocatedLinkedList::default();
            for base in [0, 1] {
                let mut batch = gpu::DrawBatch::new(
                    gpu::DrawType::midpointFanPatches,
                    gpu::ShaderMiscFlags::none,
                    gpu::DrawContents::none,
                    1,
                    base,
                    nuxie_render_api::BlendMode::SrcOver,
                    gpu::ImageSampler::LinearClamp(),
                    gpu::BarrierFlags::none,
                );
                batch.indexCountPerInstance = 3;
                draws.push_back(batch);
            }
            let mut desc = empty_source_flush_descriptor();
            desc.combinedShaderFeatures = gpu::ShaderFeatures(1);
            desc.interlockMode = gpu::InterlockMode::Atomics;
            desc.pathCount = 1;
            desc.drawList = Some(core::ptr::NonNull::from(&draws));
            unsafe {
                context.flush(
                    &mut metal,
                    &desc,
                    &mut target,
                    Handle::new(88, MetalObjectKind::CommandBuffer),
                );
            }
            metal.drain_recorded_clone_drops();
            (metal, source)
        };

        let (metal, source) = run_path_batches(None);
        let batches = assert_clone_triplets(&metal, "RC-PS-DRAW", source, 2);
        assert!(batches[2].handle != batches[3].handle);
        let draw_positions = metal
            .owner_events
            .iter()
            .enumerate()
            .filter_map(|(index, event)| (event.ledger_id == "RC-PS-DRAW").then_some(index))
            .collect::<Vec<_>>();
        assert!(draw_positions[2] < draw_positions[3]);
        assert!(batches.chunks_exact(3).all(|triple| {
            triple[1].selector_ordinal.map(|ordinal| ordinal.0)
                == Some(
                    "drawIndexedPrimitives:indexCount:indexType:indexBuffer:indexBufferOffset:instanceCount:",
                )
        }));

        for occurrence in [1, 2] {
            let (failed, source) = run_path_batches(Some(occurrence));
            assert_eq!(failed.fail_clone_exact, None);
            assert_clone_triplets(&failed, "RC-PS-DRAW", source, 1);
            assert_eq!(
                failed
                    .calls
                    .iter()
                    .filter(|call| call.selector.starts_with("drawIndexedPrimitives:"))
                    .count(),
                2
            );
        }

        let (mut metal, mut context, mut target) = recording_flush_fixture(false);
        let device = metal.device_handle();
        let pipeline = DrawPipeline::new(
            &mut metal,
            device,
            Some(Handle::new(2, MetalObjectKind::Library)),
            SourceFunctionName::Static(DRAW_VERTEX_NAME),
            SourceFunctionName::Static(DRAW_FRAGMENT_NAME),
            DrawType::ImageMesh,
            InterlockMode::Atomics,
            ShaderFeatures(1),
            ShaderMiscFlags::none,
            SynthesizedFailureType::none,
        );
        let source = pipeline
            .rgba8
            .as_ref()
            .map(OwnedMetalHandle::handle)
            .expect("RGBA image-mesh pipeline state");
        context.seed_pipeline_for_test(
            shader_key_for_test(
                DrawType::ImageMesh,
                ShaderFeatures(1),
                InterlockMode::Atomics,
                ShaderMiscFlags::none,
            ),
            pipeline,
        );
        let foreign = foreign_buffer();
        let foreign = core::ptr::NonNull::from(foreign.as_ref());
        let mut mesh = gpu::DrawBatch::new(
            gpu::DrawType::imageMesh,
            gpu::ShaderMiscFlags::none,
            gpu::DrawContents::none,
            1,
            0,
            nuxie_render_api::BlendMode::SrcOver,
            gpu::ImageSampler::LinearClamp(),
            gpu::BarrierFlags::none,
        );
        mesh.vertexBuffer = Some(foreign);
        mesh.uvBuffer = Some(foreign);
        mesh.indexBuffer = Some(foreign);
        let mut draws = gpu::BlockAllocatedLinkedList::default();
        draws.push_back(mesh);
        metal.owner_events.clear();
        metal.retirements.clear();
        metal.retirement_call_counts.clear();
        metal.retirement_event_counts.clear();
        metal.calls.clear();
        let mut desc = empty_source_flush_descriptor();
        desc.combinedShaderFeatures = gpu::ShaderFeatures(1);
        desc.interlockMode = gpu::InterlockMode::Atomics;
        desc.drawList = Some(core::ptr::NonNull::from(&draws));
        unsafe {
            context.flush(
                &mut metal,
                &desc,
                &mut target,
                Handle::new(88, MetalObjectKind::CommandBuffer),
            );
        }
        metal.drain_recorded_clone_drops();
        let mesh_owner = assert_clone_triplets(&metal, "RC-PS-DRAW", source, 1);
        assert!(metal.calls.iter().any(|call| {
            call.selector == "setRenderPipelineState:"
                && call.args.contains(&Value::Handle(mesh_owner[0].handle))
        }));
        assert!(metal.calls.iter().any(|call| {
            call.selector == "setCullMode:" && call.args.contains(&Value::U64(0))
        }));
        assert!(!metal
            .calls
            .iter()
            .any(|call| call.selector == "drawIndexedPrimitives:indexCount:indexType:indexBuffer:indexBufferOffset:"));
    }

    #[test]
    fn canonical_constructor_sweeps_each_actual_selector_failpoint() {
        use std::collections::BTreeSet;

        // First collect selectors from the real source constructor. The
        // second pass fails each selector at its actual RecordingMetal call
        // boundary and drops the partially initialized canonical owner. This
        // deliberately exercises source creation scopes rather than a
        // synthetic list of MetalObjectKind values.
        let mut baseline = RecordingMetal::default();
        let device = baseline.device_handle();
        let _context = RenderContextMetal::new(&mut baseline, device, ContextOptions::default());
        let selectors: BTreeSet<&'static str> =
            baseline.calls.iter().map(|call| call.selector).collect();
        assert!(
            !selectors.is_empty(),
            "canonical constructor made no selectors"
        );

        for selector in selectors {
            let mut metal = RecordingMetal::default();
            metal.fail.push_back(selector);
            let device = metal.device_handle();
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _context =
                    RenderContextMetal::new(&mut metal, device, ContextOptions::default());
            }));
            assert!(
                result.is_ok(),
                "source constructor panicked at actual selector failpoint {selector}"
            );
            metal.drain_recorded_clone_drops();
            for event in &metal.owner_events {
                if matches!(
                    event.phase,
                    OwnerEventPhase::Release
                        | OwnerEventPhase::ReleaseStrong
                        | OwnerEventPhase::ReleaseLocal
                        | OwnerEventPhase::AliasEnd
                ) {
                    assert!(
                        metal.retirements.contains(&event.handle),
                        "selector failpoint {selector} left {} outside its retire boundary",
                        event.ledger_id
                    );
                }
            }
        }
    }

    fn expectation_row(id: &str) -> Vec<&str> {
        OWNER_EXPECTATIONS
            .lines()
            .skip(1)
            .find_map(|line| {
                let row = line.split('\t').collect::<Vec<_>>();
                (row.first().copied() == Some(id)).then_some(row)
            })
            .unwrap_or_else(|| panic!("missing owner expectation {id}"))
    }

    fn row_events<'a>(events: &'a [OwnerEvent], id: &str) -> Vec<&'a OwnerEvent> {
        events
            .iter()
            .filter(|event| event.ledger_id == id)
            .collect()
    }

    fn assert_one_exact_triplet(
        events: &[OwnerEvent],
        id: &str,
        create: OwnerEventPhase,
        final_phase_name: &str,
    ) {
        let row = expectation_row(id);
        // Consume the complete row contract in the scenario which exercises
        // it. These assertions intentionally bind human-readable source
        // anchors/probes as well as the machine-checked lifecycle fields.
        assert_eq!(row.len(), 11);
        assert!(!row[1].is_empty(), "{id} callsite contract");
        assert!(!row[2].is_empty(), "{id} execution-path contract");
        assert!(!row[3].is_empty(), "{id} ownership-class contract");
        assert_eq!(row[5], "1", "{id} scenario multiplicity");
        assert!(!row[6].is_empty(), "{id} identity relation");
        assert!(!row[7].is_empty(), "{id} predecessor relation");
        assert_ne!(row[8], "none", "{id} exact failure ordinal");
        assert!(
            row[9].contains("actual") || row[9].contains("real"),
            "{id} non-synthetic probe"
        );
        assert!(!row[10].is_empty(), "{id} recorded audit gap");
        let expected_tail = if final_phase_name == "AliasEnd" {
            ["Borrow", "LastUse", "AliasEnd"]
        } else {
            ["Create", "LastUse", "Release"]
        };
        let table_phases = row[4]
            .split('>')
            .map(|phase| phase.split('(').next().unwrap_or(phase))
            .collect::<Vec<_>>();
        assert_eq!(table_phases, expected_tail, "{id} TSV phase sequence");
        let actual = row_events(events, id);
        assert_eq!(actual.len(), 3, "{id} exact phase multiplicity");
        assert_eq!(actual[0].phase, create, "{id} opening phase");
        assert_eq!(actual[1].phase, OwnerEventPhase::LastUse, "{id} last use");
        assert_eq!(
            actual[2].phase,
            if final_phase_name == "AliasEnd" {
                OwnerEventPhase::AliasEnd
            } else {
                OwnerEventPhase::Release
            },
            "{id} close phase"
        );
        assert!(actual.iter().all(|event| {
            event.native_identity == actual[0].native_identity
                && event.source_handle == actual[0].source_handle
        }), "{id} stable identity");
    }

    fn event_position(events: &[OwnerEvent], id: &str, phase: OwnerEventPhase) -> usize {
        events
            .iter()
            .position(|event| event.ledger_id == id && event.phase == phase)
            .unwrap_or_else(|| panic!("missing {id} {phase:?}"))
    }

    #[test]
    fn color_and_tess_scenarios_bind_exact_owner_rows_and_failure_ordinals() {
        fn run_color() -> RecordingMetal {
            let mut metal = RecordingMetal::default();
            let device = metal.device_handle();
            let pipeline = ColorRampPipeline::color_ramp(
                &mut metal,
                device,
                Handle::new(2, MetalObjectKind::Library),
            );
            // The state member is intentionally kept out of this lexical-row
            // scenario. Its cross-boundary member Drop is certified in the
            // constructor/destructor scenario rather than conflated here.
            drop(pipeline);
            RENDER_CONTEXT_OWNER_DROP_EVENTS.lock().unwrap().clear();
            RENDER_CONTEXT_OWNER_DROP_RETIREMENTS.lock().unwrap().clear();
            metal
        }

        fn run_tess() -> RecordingMetal {
            let mut metal = RecordingMetal::default();
            let device = metal.device_handle();
            let pipeline = TessellatePipeline::new(
                &mut metal,
                device,
                Handle::new(2, MetalObjectKind::Library),
            );
            drop(pipeline);
            RENDER_CONTEXT_OWNER_DROP_EVENTS.lock().unwrap().clear();
            RENDER_CONTEXT_OWNER_DROP_RETIREMENTS.lock().unwrap().clear();
            metal
        }

        for (events, family) in [
            (run_color().owner_events, "COLOR"),
            (run_tess().owner_events, "TESS"),
        ] {
            for suffix in ["PD", "FN-V", "FN-F", "ATT-0"] {
                let id = if suffix == "PD" {
                    format!("RC-PD-{family}")
                } else if suffix == "ATT-0" {
                    format!("RC-ATT-{family}-0")
                } else if suffix == "FN-V" {
                    format!("RC-FN-{family}-V")
                } else if suffix == "FN-F" {
                    format!("RC-FN-{family}-F")
                } else {
                    format!("RC-{suffix}-{family}")
                };
                let opening = if suffix == "ATT-0" {
                    OwnerEventPhase::Borrow
                } else {
                    OwnerEventPhase::Create
                };
                assert_one_exact_triplet(
                    &events,
                    &id,
                    opening,
                    if suffix == "ATT-0" { "AliasEnd" } else { "Release" },
                );
            }
            let vertex = format!("RC-FN-{family}-V");
            let fragment = format!("RC-FN-{family}-F");
            let attachment = format!("RC-ATT-{family}-0");
            let descriptor = format!("RC-PD-{family}");
            assert!(
                event_position(&events, &vertex, OwnerEventPhase::Release)
                    < event_position(&events, &fragment, OwnerEventPhase::Create)
            );
            assert!(
                event_position(&events, &fragment, OwnerEventPhase::Release)
                    < event_position(&events, &attachment, OwnerEventPhase::Borrow)
            );
            assert!(
                event_position(&events, &attachment, OwnerEventPhase::AliasEnd)
                    < event_position(&events, &descriptor, OwnerEventPhase::LastUse)
            );
            assert!(
                event_position(&events, &descriptor, OwnerEventPhase::LastUse)
                    < event_position(&events, &descriptor, OwnerEventPhase::Release)
            );
        }

        // Exercise each exact selector occurrence named by the four lexical
        // rows plus the state/error boundary. In particular, the second
        // newFunction occurrence proves the fragment ordinal rather than
        // merely failing the first selector with the same name.
        for (family, selector, occurrence) in [
            ("color", "alloc/init", 1),
            ("color", "newFunctionWithName:", 1),
            ("color", "newFunctionWithName:", 2),
            ("color", "colorAttachmentAtIndex:", 1),
            ("color", "newRenderPipelineStateWithDescriptor:error:", 1),
            ("tess", "alloc/init", 1),
            ("tess", "newFunctionWithName:", 1),
            ("tess", "newFunctionWithName:", 2),
            ("tess", "colorAttachmentAtIndex:", 1),
            ("tess", "newRenderPipelineStateWithDescriptor:error:", 1),
        ] {
            let mut metal = RecordingMetal::default();
            metal.fail_exact = Some((selector, occurrence));
            let device = metal.device_handle();
            if family == "color" {
                let pipeline = ColorRampPipeline::color_ramp(
                    &mut metal,
                    device,
                    Handle::new(2, MetalObjectKind::Library),
                );
                drop(pipeline);
            } else {
                let pipeline = TessellatePipeline::new(
                    &mut metal,
                    device,
                    Handle::new(2, MetalObjectKind::Library),
                );
                drop(pipeline);
            }
            RENDER_CONTEXT_OWNER_DROP_EVENTS.lock().unwrap().clear();
            RENDER_CONTEXT_OWNER_DROP_RETIREMENTS.lock().unwrap().clear();
            assert_eq!(metal.fail_exact, None, "{family} {selector}#{occurrence}");
            assert!(
                metal.selector_occurrence_count(selector) >= occurrence,
                "{family} did not execute {selector} occurrence {occurrence}"
            );
            for id in if family == "color" {
                ["RC-PD-COLOR", "RC-FN-COLOR-V", "RC-FN-COLOR-F", "RC-ATT-COLOR-0"]
            } else {
                ["RC-PD-TESS", "RC-FN-TESS-V", "RC-FN-TESS-F", "RC-ATT-TESS-0"]
            } {
                let row = row_events(&metal.owner_events, id);
                if !row.is_empty() {
                    assert_eq!(
                        row.last().unwrap().phase,
                        if id.contains("ATT-") {
                            OwnerEventPhase::AliasEnd
                        } else {
                            OwnerEventPhase::Release
                        },
                        "{id} failpoint cleanup"
                    );
                }
            }
        }

        // The attachment accessor is a parent-tied +0 alias, not another
        // strong Objective-C owner. Bind the concrete source expression to
        // its descriptor and prove its registry alias ends before the parent
        // descriptor scope for both simple pipelines.
        for (metal, attachment_id, descriptor_id) in [
            (run_color(), "RC-ATT-COLOR-0", "RC-PD-COLOR"),
            (run_tess(), "RC-ATT-TESS-0", "RC-PD-TESS"),
        ] {
            let attachment = row_events(&metal.owner_events, attachment_id);
            let descriptor = row_events(&metal.owner_events, descriptor_id);
            assert_eq!(attachment.len(), 3);
            assert_eq!(descriptor.len(), 3);
            assert_eq!(
                attachment[0].parent_handle,
                Some(descriptor[0].native_identity)
            );
            assert_ne!(attachment[0].handle, descriptor[0].handle);
            assert!(metal.retirements.contains(&attachment[0].handle));
            assert!(
                metal
                    .retirements
                    .iter()
                    .position(|handle| *handle == attachment[0].handle)
                    .unwrap()
                    < metal
                        .retirements
                        .iter()
                        .position(|handle| *handle == descriptor[0].handle)
                        .unwrap()
            );
        }
    }

    fn run_color_pipeline_result_scenario(object: bool, error: bool) -> RecordingMetal {
        RENDER_CONTEXT_OWNER_DROP_EVENTS.lock().unwrap().clear();
        RENDER_CONTEXT_OWNER_DROP_RETIREMENTS.lock().unwrap().clear();
        let mut metal = RecordingMetal::default();
        if !object {
            metal.fail_exact = Some(("newRenderPipelineStateWithDescriptor:error:", 1));
        }
        if error {
            metal.errors.push_back((
                "newRenderPipelineStateWithDescriptor:error:",
                "scripted pipeline error".into(),
            ));
        }
        let device = metal.device_handle();
        let pipeline = ColorRampPipeline::color_ramp(
            &mut metal,
            device,
            Handle::new(2, MetalObjectKind::Library),
        );
        drop(pipeline);
        metal.owner_events.extend(
            RENDER_CONTEXT_OWNER_DROP_EVENTS
                .lock()
                .unwrap()
                .drain(..),
        );
        metal.retirements.extend(
            RENDER_CONTEXT_OWNER_DROP_RETIREMENTS
                .lock()
                .unwrap()
                .drain(..),
        );
        metal
    }

    #[test]
    fn pipeline_result_scenario_covers_every_object_error_outcome_and_member_drop() {
        for (object, error) in [(true, false), (true, true), (false, true), (false, false)] {
            let metal = run_color_pipeline_result_scenario(object, error);
            let states = row_events(&metal.owner_events, "RC-STATE-PIPE");
            let errors = row_events(&metal.owner_events, "RC-ERR-PIPE");
            let descriptor = row_events(&metal.owner_events, "RC-PD-COLOR");

            if object {
                assert_eq!(
                    states.iter().map(|event| event.phase).collect::<Vec<_>>(),
                    vec![
                        OwnerEventPhase::Create,
                        OwnerEventPhase::Transfer,
                        OwnerEventPhase::Release,
                    ],
                    "state lifecycle for object={object} error={error}"
                );
                assert!(states.iter().all(|event| {
                    event.handle == states[0].handle
                        && event.native_identity == states[0].native_identity
                }));
                let transfer = metal
                    .owner_events
                    .iter()
                    .position(|event| {
                        event.ledger_id == "RC-STATE-PIPE"
                            && event.phase == OwnerEventPhase::Transfer
                    })
                    .unwrap();
                let descriptor_release = metal
                    .owner_events
                    .iter()
                    .position(|event| {
                        event.ledger_id == "RC-PD-COLOR"
                            && event.phase == OwnerEventPhase::Release
                    })
                    .unwrap();
                let member_release = metal
                    .owner_events
                    .iter()
                    .position(|event| {
                        event.ledger_id == "RC-STATE-PIPE"
                            && event.phase == OwnerEventPhase::Release
                    })
                    .unwrap();
                assert!(transfer < descriptor_release && descriptor_release < member_release);
                assert!(metal.retirements.contains(&states[0].handle));
            } else {
                assert!(states.is_empty());
                assert_eq!(metal.fail_exact, None);
            }

            if error {
                assert_eq!(
                    errors.iter().map(|event| event.phase).collect::<Vec<_>>(),
                    vec![
                        OwnerEventPhase::Create,
                        OwnerEventPhase::LastUse,
                        OwnerEventPhase::Release,
                    ]
                );
                assert!(errors.iter().all(|event| {
                    event.handle == errors[0].handle
                        && event.native_identity == errors[0].native_identity
                }));
                assert!(metal.retirements.contains(&errors[0].handle));
                assert!(states
                    .first()
                    .is_none_or(|state| state.native_identity != errors[0].native_identity));
                let error_release = metal
                    .owner_events
                    .iter()
                    .position(|event| {
                        event.ledger_id == "RC-ERR-PIPE"
                            && event.phase == OwnerEventPhase::Release
                    })
                    .unwrap();
                let descriptor_release = metal
                    .owner_events
                    .iter()
                    .position(|event| {
                        event.ledger_id == "RC-PD-COLOR"
                            && event.phase == OwnerEventPhase::Release
                    })
                    .unwrap();
                assert!(error_release < descriptor_release);
            } else {
                assert!(errors.is_empty());
            }
            assert_eq!(descriptor.len(), 3);
        }
    }

    fn run_feather_pair(fail: Option<(&'static str, usize)>) -> RecordingMetal {
        let mut metal = RecordingMetal::default();
        metal.fail_exact = fail;
        let device = metal.device_handle();
        let library = Handle::new(2, MetalObjectKind::Library);
        let fill = super::source_execution::FeatherAtlasPipeline::new(
            &mut metal,
            device,
            library,
            ATLAS_FILL_FRAGMENT_NAME,
            false,
        );
        let stroke = super::source_execution::FeatherAtlasPipeline::new(
            &mut metal,
            device,
            library,
            ATLAS_STROKE_FRAGMENT_NAME,
            true,
        );
        drop(stroke);
        drop(fill);
        RENDER_CONTEXT_OWNER_DROP_EVENTS.lock().unwrap().clear();
        RENDER_CONTEXT_OWNER_DROP_RETIREMENTS.lock().unwrap().clear();
        metal
    }

    fn assert_repeated_exact_triplets(
        events: &[OwnerEvent],
        id: &str,
        count: usize,
        opening: OwnerEventPhase,
        closing: OwnerEventPhase,
    ) {
        let actual = row_events(events, id);
        assert_eq!(actual.len(), count * 3, "{id} exact event multiplicity");
        for (occurrence, triplet) in actual.chunks_exact(3).enumerate() {
            assert_eq!(triplet[0].phase, opening, "{id}[{occurrence}] open");
            assert_eq!(
                triplet[1].phase,
                OwnerEventPhase::LastUse,
                "{id}[{occurrence}] last use"
            );
            assert_eq!(
                triplet[2].phase,
                closing,
                "{id}[{occurrence}] close"
            );
            assert!(triplet.iter().all(|event| {
                event.native_identity == triplet[0].native_identity
                    && event.source_handle == triplet[0].source_handle
                    && event.parent_handle == triplet[0].parent_handle
            }));
        }
    }

    #[test]
    fn feather_pair_scenario_binds_multiplicity_parent_order_and_each_failure_ordinal() {
        let metal = run_feather_pair(None);
        let events = &metal.owner_events;
        for (id, count, opening) in [
            ("RC-PD-FEATHER", 2, OwnerEventPhase::Create),
            ("RC-FN-FEATHER-V", 2, OwnerEventPhase::Create),
            ("RC-FN-FEATHER-F", 2, OwnerEventPhase::Create),
            ("RC-ATT-FEATHER-0-X9", 18, OwnerEventPhase::Borrow),
        ] {
            let row = expectation_row(id);
            assert_eq!(row.len(), 11);
            assert!(!row[1].is_empty());
            assert!(row[2].contains("feather") || row[2].contains("resize_feather"));
            assert!(!row[3].is_empty());
            assert!(!row[4].is_empty());
            assert!(!row[5].is_empty());
            assert!(!row[6].is_empty());
            assert!(!row[7].is_empty());
            assert_ne!(row[8], "none");
            assert!(row[9].contains("actual"));
            assert!(!row[10].is_empty());
            assert_repeated_exact_triplets(
                events,
                id,
                count,
                opening,
                if id == "RC-ATT-FEATHER-0-X9" {
                    OwnerEventPhase::AliasEnd
                } else {
                    OwnerEventPhase::Release
                },
            );
        }
        assert_eq!(expectation_row("RC-PD-FEATHER")[5], "2");
        assert_eq!(expectation_row("RC-FN-FEATHER-V")[5], "2");
        assert_eq!(expectation_row("RC-FN-FEATHER-F")[5], "2");
        assert_eq!(
            expectation_row("RC-ATT-FEATHER-0-X9")[5],
            "18 total; 9 per pipeline"
        );

        let descriptors = row_events(events, "RC-PD-FEATHER");
        let fill_parent = descriptors[0].native_identity;
        let stroke_parent = descriptors[3].native_identity;
        let attachments = row_events(events, "RC-ATT-FEATHER-0-X9");
        let attachment_aliases = attachments
            .chunks_exact(3)
            .map(|triplet| {
                assert!(metal.retirements.contains(&triplet[0].handle));
                let identity = triplet[0].native_identity;
                (identity.registry, identity.slot, identity.generation)
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(attachment_aliases.len(), 18);
        for (index, triplet) in attachments.chunks_exact(3).enumerate() {
            let expected_parent = if index < 9 { fill_parent } else { stroke_parent };
            assert_eq!(
                triplet[0].parent_handle,
                Some(expected_parent),
                "attachment expression {index} parent"
            );
        }
        // Each expression ends before the next accessor, and the complete
        // fill descriptor scope ends before stroke construction begins.
        for pair in attachments.chunks_exact(3).collect::<Vec<_>>().windows(2) {
            let first_release = events
                .iter()
                .position(|event| {
                    event.ledger_id == "RC-ATT-FEATHER-0-X9"
                        && event.phase == OwnerEventPhase::AliasEnd
                        && event.handle == pair[0][2].handle
                })
                .unwrap();
            let next_borrow = events
                .iter()
                .position(|event| {
                    event.ledger_id == "RC-ATT-FEATHER-0-X9"
                        && event.phase == OwnerEventPhase::Borrow
                        && event.handle == pair[1][0].handle
                })
                .unwrap();
            assert!(first_release < next_borrow);
        }
        assert!(
            event_position(events, "RC-PD-FEATHER", OwnerEventPhase::Release)
                < events
                    .iter()
                    .enumerate()
                    .filter(|(_, event)| {
                        event.ledger_id == "RC-PD-FEATHER"
                            && event.phase == OwnerEventPhase::Create
                    })
                    .nth(1)
                    .map(|(index, _)| index)
                    .unwrap()
        );

        for (selector, occurrences) in [
            ("alloc/init", 2usize),
            ("newFunctionWithName:", 4),
            ("colorAttachmentAtIndex:", 18),
            ("newRenderPipelineStateWithDescriptor:error:", 2),
        ] {
            for occurrence in 1..=occurrences {
                let metal = run_feather_pair(Some((selector, occurrence)));
                assert_eq!(
                    metal.fail_exact, None,
                    "feather {selector} exact occurrence {occurrence} was not injected"
                );
                assert!(metal.selector_occurrence_count(selector) >= occurrence);
                for id in [
                    "RC-PD-FEATHER",
                    "RC-FN-FEATHER-V",
                    "RC-FN-FEATHER-F",
                    "RC-ATT-FEATHER-0-X9",
                ] {
                    let row = row_events(&metal.owner_events, id);
                    if !row.is_empty() {
                        assert_eq!(
                            row.last().unwrap().phase,
                            if id == "RC-ATT-FEATHER-0-X9" {
                                OwnerEventPhase::AliasEnd
                            } else {
                                OwnerEventPhase::Release
                            },
                            "{id} cleanup at {selector}#{occurrence}"
                        );
                    }
                }
            }
        }
    }

    fn run_draw_pipeline_scenario(
        fail_selector: Option<(&'static str, usize)>,
        fail_clone: Option<(MetalObjectKind, usize)>,
    ) -> RecordingMetal {
        let mut metal = RecordingMetal::default();
        metal.fail_exact = fail_selector;
        metal.fail_clone_exact = fail_clone;
        let vertex = precompiled_name(
            &mut metal,
            "RC-NS-FUNCTION-NAME-V",
            DrawType::ImageMesh,
            ShaderFeatures(0),
            ShaderMiscFlags(0),
            DRAW_VERTEX_NAME,
        );
        let fragment = precompiled_name(
            &mut metal,
            "RC-NS-FUNCTION-NAME-F",
            DrawType::ImageMesh,
            ShaderFeatures(0),
            ShaderMiscFlags(0),
            DRAW_FRAGMENT_NAME,
        );
        let device = metal.device_handle();
        let pipeline = DrawPipeline::new(
            &mut metal,
            device,
            Some(Handle::new(2, MetalObjectKind::Library)),
            vertex,
            fragment,
            DrawType::ImageMesh,
            InterlockMode::RasterOrdering,
            ShaderFeatures(0),
            ShaderMiscFlags(0),
            SynthesizedFailureType::none,
        );
        drop(pipeline);
        metal.drain_recorded_clone_drops();
        metal
    }

    fn run_static_literal_scenario(
        fail_selector: Option<(&'static str, usize)>,
    ) -> RecordingMetal {
        let mut metal = RecordingMetal::default();
        metal.fail_exact = fail_selector;
        let device = metal.device_handle();
        let library = Handle::new(2, MetalObjectKind::Library);
        drop(ColorRampPipeline::color_ramp(&mut metal, device, library));
        drop(TessellatePipeline::new(&mut metal, device, library));
        drop(super::source_execution::FeatherAtlasPipeline::new(
            &mut metal,
            device,
            library,
            ATLAS_FILL_FRAGMENT_NAME,
            false,
        ));
        drop(super::source_execution::FeatherAtlasPipeline::new(
            &mut metal,
            device,
            library,
            ATLAS_STROKE_FRAGMENT_NAME,
            true,
        ));
        drop(DrawPipeline::new(
            &mut metal,
            device,
            Some(library),
            SourceFunctionName::Static(DRAW_VERTEX_NAME),
            SourceFunctionName::Static(DRAW_FRAGMENT_NAME),
            DrawType::ImageMesh,
            InterlockMode::RasterOrdering,
            ShaderFeatures(0),
            ShaderMiscFlags(0),
            SynthesizedFailureType::none,
        ));
        metal.drain_recorded_clone_drops();
        metal
    }

    #[test]
    fn static_nsstring_literals_have_an_exact_owner_free_source_census() {
        let assert_scenario = |metal: &RecordingMetal| {
            let names = metal
                .calls
                .iter()
                .filter(|call| call.selector == "newFunctionWithName:")
                .map(|call| match call.args.get(1) {
                    Some(Value::StaticText(name)) => *name,
                    other => panic!("source static function name was rematerialized: {other:?}"),
                })
                .collect::<Vec<_>>();
            assert_eq!(
                names,
                [
                    SOURCE_STATIC_FUNCTION_NAMES[0],
                    SOURCE_STATIC_FUNCTION_NAMES[1],
                    SOURCE_STATIC_FUNCTION_NAMES[2],
                    SOURCE_STATIC_FUNCTION_NAMES[3],
                    SOURCE_STATIC_FUNCTION_NAMES[4],
                    ATLAS_FILL_FRAGMENT_NAME,
                    SOURCE_STATIC_FUNCTION_NAMES[4],
                    ATLAS_STROKE_FRAGMENT_NAME,
                    DRAW_VERTEX_NAME,
                    DRAW_FRAGMENT_NAME,
                ]
            );
            assert_eq!(
                names.iter().copied().collect::<BTreeSet<_>>(),
                SOURCE_STATIC_FUNCTION_NAMES.into_iter().collect::<BTreeSet<_>>()
            );
            assert!(metal
                .owner_events
                .iter()
                .all(|event| event.ledger_id != "RC-STATIC-NS-LITERALS"));
            assert!(metal
                .retirements
                .iter()
                .all(|handle| handle.kind != MetalObjectKind::NSString));
        };

        let baseline = run_static_literal_scenario(None);
        assert_scenario(&baseline);
        for occurrence in 1..=10 {
            let failed = run_static_literal_scenario(Some(("newFunctionWithName:", occurrence)));
            assert_eq!(failed.fail_exact, None, "literal failure ordinal {occurrence}");
            assert_scenario(&failed);
        }
    }

    #[test]
    fn objective_c_parameters_remain_caller_owned_across_representative_source_calls() {
        let mut metal = RecordingMetal::default();
        let device = metal.device_handle();
        let library = Handle::new(20, MetalObjectKind::Library);
        let descriptor = Handle::new(21, MetalObjectKind::RenderPipelineDescriptor);
        let function = Handle::new(22, MetalObjectKind::Function);
        let encoder = Handle::new(23, MetalObjectKind::RenderCommandEncoder);
        let pipeline = Handle::new(24, MetalObjectKind::RenderPipelineState);
        let caller_owned = [device, library, descriptor, function, encoder, pipeline];

        let _ = metal.call(
            "library",
            "newFunctionWithName:",
            vec![Value::Handle(library), Value::StaticText(DRAW_VERTEX_NAME)],
        );
        let _ = metal.call(
            "descriptor",
            "setVertexFunction:",
            vec![Value::Handle(descriptor), Value::Handle(function)],
        );
        let _ = metal.call_with_error(
            "gpu",
            "newRenderPipelineStateWithDescriptor:error:",
            vec![Value::Handle(device), Value::Handle(descriptor)],
        );
        let _ = metal.call(
            "encoder",
            "setRenderPipelineState:",
            vec![Value::Handle(encoder), Value::Handle(pipeline)],
        );

        assert!(caller_owned
            .iter()
            .all(|handle| !metal.retirements.contains(handle)));
        assert!(metal.owner_events.iter().all(|event| {
            event.ledger_id != "EXCL-OBJCPARAMS"
                && !matches!(
                    event.phase,
                    OwnerEventPhase::CreateClone | OwnerEventPhase::Release
                )
        }));
        let passed_parameters = metal
            .calls
            .iter()
            .flat_map(|call| call.args.iter())
            .filter_map(|value| match value {
                Value::Handle(handle) => Some(*handle),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(caller_owned
            .into_iter()
            .all(|handle| passed_parameters.contains(&handle)));
    }

    #[test]
    fn cpp_owners_are_bound_to_the_intrusive_box_and_raw_pointer_ledger() {
        use core::any::type_name;
        use core::pin::Pin;

        let mut inventory = vec![
            type_name::<rcp<RenderTargetMetal>>(),
            type_name::<rcp<RenderBuffer>>(),
            type_name::<rcp<RenderTarget>>(),
            type_name::<Box<RenderBufferMetal>>(),
            type_name::<Box<TextureMetal>>(),
            type_name::<Pin<Box<RenderContextMetal>>>(),
            type_name::<*mut RenderBuffer>(),
            type_name::<*const gpu::DrawBatch>(),
        ];
        #[cfg(feature = "native-ore-metal-experimental")]
        inventory.extend([
            type_name::<rcp<RenderCanvas>>(),
            type_name::<rcp<RiveRenderImage>>(),
            type_name::<rcp<Texture>>(),
        ]);
        assert!(inventory[0].contains("refcnt_hpp::rcp"));
        assert!(inventory[0].ends_with("RenderTargetMetal>"));
        assert!(inventory[1].ends_with("RenderBuffer>"));
        assert!(inventory[2].ends_with("RenderTarget>"));
        assert!(inventory[3].ends_with("RenderBufferMetal>"));
        assert!(inventory[4].ends_with("TextureMetal>"));
        assert!(inventory[5].contains("Pin<alloc::boxed::Box"));
        assert!(inventory[6].starts_with("*mut "));
        assert!(inventory[7].starts_with("*const "));
        #[cfg(feature = "native-ore-metal-experimental")]
        {
            assert_eq!(inventory.len(), 11);
            assert!(inventory[8].ends_with("RenderCanvas>"));
            assert!(inventory[9].ends_with("RiveRenderImage>"));
            assert!(inventory[10].ends_with("Texture>"));
        }
        assert!(core::mem::needs_drop::<rcp<RenderTargetMetal>>());
        assert!(core::mem::needs_drop::<Box<RenderBufferMetal>>());
        assert!(core::mem::needs_drop::<Box<TextureMetal>>());
        assert!(core::mem::needs_drop::<Pin<Box<RenderContextMetal>>>());
        assert!(!core::mem::needs_drop::<*mut RenderBuffer>());
        assert!(!core::mem::needs_drop::<*const gpu::DrawBatch>());

        let mut metal = RecordingMetal::default();
        let target = RenderTargetMetal::new(
            &mut metal,
            PixelFormat::RGBA8Unorm,
            4,
            4,
            Default::default(),
        );
        let target = unsafe { rcp::from_ptr(Box::into_raw(Box::new(target))) };
        assert_eq!(unsafe { (&*target.get()).base.base.debugging_refcnt() }, 1);
        let target_clone = target.clone();
        assert_eq!(unsafe { (&*target.get()).base.base.debugging_refcnt() }, 2);
        drop(target_clone);
        assert_eq!(unsafe { (&*target.get()).base.base.debugging_refcnt() }, 1);
        drop(target);

        let mut foreign = foreign_buffer();
        let raw = core::ptr::NonNull::from(&mut *foreign);
        let before = foreign.base.debugging_refcnt();
        assert!(unsafe { image_mesh_buffer_handles(Some(raw), Some(raw), Some(raw)) }.is_none());
        assert_eq!(foreign.base.debugging_refcnt(), before);

        let device = metal.device_handle();
        let context = Box::pin(RenderContextMetal::new(
            &mut metal,
            device,
            ContextOptions::default(),
        ));
        let stable_address = core::ptr::from_ref(&*context);
        let moved_pin = context;
        assert_eq!(core::ptr::from_ref(&*moved_pin), stable_address);
        drop(moved_pin);
        assert!(metal.owner_events.iter().all(|event| {
            event.ledger_id != "EXCL-CPP-OWNERS"
                && event.ledger_id != "EXCL-OBJCPARAMS"
        }));
    }

    #[test]
    fn draw_pipeline_scenario_binds_names_clone_aliases_order_and_failure_ordinals() {
        let metal = run_draw_pipeline_scenario(None, None);
        let events = &metal.owner_events;
        for id in ["RC-NS-FUNCTION-NAME-V", "RC-NS-FUNCTION-NAME-F"] {
            let row = expectation_row(id);
            assert_eq!(row.len(), 11);
            assert_eq!(row[4], "CreateBridge>Borrow(newFunctionWithName)>AliasEnd");
            assert_eq!(row[5], "1 per DrawPipeline");
            assert!(row[6].contains("pointer"));
            assert!(row[7].contains("survive") || row[7].contains("outlive"));
            assert_ne!(row[8], "none");
            assert!(row[9].contains("actual"));
            let actual = row_events(events, id);
            assert_eq!(actual.len(), 3);
            assert_eq!(actual[0].phase, OwnerEventPhase::CreateBridge);
            assert_eq!(actual[1].phase, OwnerEventPhase::Borrow);
            assert_eq!(actual[2].phase, OwnerEventPhase::AliasEnd);
            assert!(actual.iter().all(|event| {
                event.handle == actual[0].handle
                    && event.native_identity == actual[0].native_identity
            }));
        }

        assert_repeated_exact_triplets(
            events,
            "RC-PD-DRAW-X2",
            2,
            OwnerEventPhase::Create,
            OwnerEventPhase::Release,
        );
        let descriptors = row_events(events, "RC-PD-DRAW-X2");
        for descriptor in descriptors.chunks_exact(3) {
            let handle = descriptor[0].handle;
            let state_ordinal = descriptor[1]
                .selector_ordinal
                .expect("descriptor LastUse is the pipeline-state selector");
            assert_eq!(
                state_ordinal.0,
                "newRenderPipelineStateWithDescriptor:error:"
            );
            let state_call = metal
                .calls
                .iter()
                .enumerate()
                .filter(|(_, call)| {
                    call.selector == "newRenderPipelineStateWithDescriptor:error:"
                })
                .nth(state_ordinal.1 - 1)
                .map(|(index, _)| index)
                .unwrap();
            assert_eq!(
                metal
                    .retirement_call_counts
                    .iter()
                    .filter(|(retired, _)| *retired == handle)
                    .copied()
                    .collect::<Vec<_>>(),
                vec![(handle, state_call + 1)],
                "each descriptor retires exactly after its state selector"
            );
            let release = metal
                .owner_events
                .iter()
                .position(|event| {
                    event.ledger_id == "RC-PD-DRAW-X2"
                        && event.phase == OwnerEventPhase::Release
                        && event.handle == handle
                })
                .unwrap();
            assert_eq!(
                metal
                    .retirement_event_counts
                    .iter()
                    .find_map(|(retired, count)| (*retired == handle).then_some(*count)),
                Some(release),
                "descriptor retirement is the exact boundary before Release telemetry"
            );
        }
        for id in ["RC-ATT-DRAW-CLIP", "RC-ATT-DRAW-SCRATCH", "RC-ATT-DRAW-COVERAGE"] {
            assert_repeated_exact_triplets(
                events,
                id,
                2,
                OwnerEventPhase::Borrow,
                OwnerEventPhase::AliasEnd,
            );
            assert_eq!(expectation_row(id)[5], "2");
            let actual = row_events(events, id);
            for triple in actual.chunks_exact(3) {
                let handle = triple[0].handle;
                assert_eq!(triple[1].selector_ordinal.map(|ordinal| ordinal.0), Some("setPixelFormat:"));
                let setter = metal
                    .calls
                    .iter()
                    .position(|call| {
                        call.selector == "setPixelFormat:"
                            && call.args.first() == Some(&Value::Handle(handle))
                    })
                    .unwrap();
                assert_eq!(
                    metal
                        .retirement_call_counts
                        .iter()
                        .filter(|(retired, _)| *retired == handle)
                        .copied()
                        .collect::<Vec<_>>(),
                    vec![(handle, setter + 1)],
                    "{id} ends immediately after its one setter"
                );
                let alias_end = metal
                    .owner_events
                    .iter()
                    .position(|event| {
                        event.ledger_id == id
                            && event.phase == OwnerEventPhase::AliasEnd
                            && event.handle == handle
                    })
                    .unwrap();
                assert_eq!(
                    metal
                        .retirement_event_counts
                        .iter()
                        .find_map(|(retired, count)| (*retired == handle).then_some(*count)),
                    Some(alias_end)
                );
            }
        }
        for id in ["RC-FN-DRAW-V", "RC-FN-DRAW-F"] {
            assert_repeated_exact_triplets(
                events,
                id,
                1,
                OwnerEventPhase::Create,
                OwnerEventPhase::Release,
            );
            assert_eq!(expectation_row(id)[5], "1");
            let function = row_events(events, id);
            let handle = function[0].handle;
            assert_eq!(
                function[1].selector_ordinal.map(|ordinal| ordinal.0),
                Some("newRenderPipelineStateWithDescriptor:error:")
            );
            assert_eq!(
                metal
                    .calls
                    .iter()
                    .filter(|call| {
                        matches!(
                            call.selector,
                            "setVertexFunction:" | "setFragmentFunction:"
                        ) && call.args.contains(&Value::Handle(handle))
                    })
                    .count(),
                2,
                "the same {id} native function feeds both format descriptors"
            );
            assert_eq!(
                metal.retirements.iter().filter(|retired| **retired == handle).count(),
                1
            );
            let release = metal
                .owner_events
                .iter()
                .position(|event| {
                    event.ledger_id == id
                        && event.phase == OwnerEventPhase::Release
                        && event.handle == handle
                })
                .unwrap();
            assert_eq!(
                metal
                    .retirement_event_counts
                    .iter()
                    .find_map(|(retired, count)| (*retired == handle).then_some(*count)),
                Some(release)
            );
        }
        assert_repeated_exact_triplets(
            events,
            "RC-DRAW-LAMBDA-GPU",
            1,
            OwnerEventPhase::CreateClone,
            OwnerEventPhase::Release,
        );
        assert_eq!(expectation_row("RC-DRAW-LAMBDA-GPU")[5], "1");
        let gpu_capture = row_events(events, "RC-DRAW-LAMBDA-GPU");
        let captured_gpu = gpu_capture[0].handle;
        assert_ne!(captured_gpu, metal.device_handle());
        assert_eq!(gpu_capture[0].source_handle, metal.device_handle());
        assert_eq!(gpu_capture[0].native_identity, metal.device_handle());
        assert_eq!(
            gpu_capture[1].selector_ordinal,
            Some(("newRenderPipelineStateWithDescriptor:error:", 2))
        );
        assert_eq!(
            metal
                .retirements
                .iter()
                .filter(|retired| **retired == captured_gpu)
                .count(),
            1,
            "the block capture clone drops exactly once"
        );
        assert!(!metal.retirements.contains(&metal.device_handle()));
        assert!(metal
            .retirement_call_counts
            .iter()
            .all(|(retired, _)| *retired != captured_gpu));
        assert_eq!(
            metal
                .calls
                .iter()
                .filter(|call| {
                    call.selector == "newRenderPipelineStateWithDescriptor:error:"
                        && call.args.first() == Some(&Value::Handle(captured_gpu))
                })
                .count(),
            2,
            "both format states use the captured device alias"
        );

        let framebuffer = row_events(events, "RC-ATT-DRAW-FB-X2");
        assert_eq!(framebuffer.len(), 10);
        for (index, owner) in framebuffer.chunks_exact(5).enumerate() {
            assert_eq!(
                owner.iter().map(|event| event.phase).collect::<Vec<_>>(),
                vec![
                    OwnerEventPhase::BorrowAlias,
                    OwnerEventPhase::CreateStrong,
                    OwnerEventPhase::LastUse,
                    OwnerEventPhase::ReleaseStrong,
                    OwnerEventPhase::AliasEnd,
                ]
            );
            assert!(owner.iter().all(|event| {
                event.native_identity == owner[0].native_identity
                    && event.parent_handle == Some(descriptors[index * 3].native_identity)
            }));
            assert_ne!(owner[0].handle, owner[1].handle, "named strong clone alias");
            assert_eq!(owner[1].source_handle, owner[0].handle);
            assert_eq!(
                owner[2].selector_ordinal.map(|ordinal| ordinal.0),
                Some("newRenderPipelineStateWithDescriptor:error:")
            );
            assert_eq!(
                metal
                    .retirements
                    .iter()
                    .filter(|retired| **retired == owner[0].handle)
                    .count(),
                1,
                "the +0 framebuffer alias closes exactly once"
            );
            assert_eq!(
                metal
                    .retirements
                    .iter()
                    .filter(|retired| **retired == owner[1].handle)
                    .count(),
                1,
                "the named framebuffer clone drops exactly once"
            );
            assert!(metal
                .retirement_call_counts
                .iter()
                .all(|(retired, _)| *retired != owner[1].handle));
            let alias_end = metal
                .owner_events
                .iter()
                .position(|event| {
                    event.ledger_id == "RC-ATT-DRAW-FB-X2"
                        && event.phase == OwnerEventPhase::AliasEnd
                        && event.handle == owner[0].handle
                })
                .unwrap();
            assert_eq!(
                metal
                    .retirement_event_counts
                    .iter()
                    .find_map(|(retired, count)| {
                        (*retired == owner[0].handle).then_some(*count)
                    }),
                Some(alias_end)
            );
            let descriptor_release = metal
                .owner_events
                .iter()
                .position(|event| {
                    event.ledger_id == "RC-PD-DRAW-X2"
                        && event.phase == OwnerEventPhase::Release
                        && event.handle == descriptors[index * 3].handle
                })
                .unwrap();
            let strong_release = metal
                .owner_events
                .iter()
                .position(|event| {
                    event.ledger_id == "RC-ATT-DRAW-FB-X2"
                        && event.phase == OwnerEventPhase::ReleaseStrong
                        && event.handle == owner[1].handle
                })
                .unwrap();
            assert!(strong_release < alias_end && alias_end < descriptor_release);
        }

        // RGBA scope closes before BGRA starts. Within each raster build the
        // direct-expression children close clip→scratch→coverage before the
        // named framebuffer strong local and descriptor.
        let first_desc_release = events
            .iter()
            .position(|event| {
                event.ledger_id == "RC-PD-DRAW-X2"
                    && event.phase == OwnerEventPhase::Release
            })
            .unwrap();
        let second_desc_create = events
            .iter()
            .enumerate()
            .filter(|(_, event)| {
                event.ledger_id == "RC-PD-DRAW-X2"
                    && event.phase == OwnerEventPhase::Create
            })
            .nth(1)
            .map(|(index, _)| index)
            .unwrap();
        assert!(first_desc_release < second_desc_create);
        let mut mrt_aliases = BTreeSet::new();
        for format in 0..2 {
            let parent = descriptors[format * 3].native_identity;
            let child_close = |id: &str| {
                events
                    .iter()
                    .position(|event| {
                        event.ledger_id == id
                            && event.phase == OwnerEventPhase::AliasEnd
                            && event.parent_handle == Some(parent)
                    })
                    .unwrap()
            };
            assert!(child_close("RC-ATT-DRAW-CLIP") < child_close("RC-ATT-DRAW-SCRATCH"));
            assert!(
                child_close("RC-ATT-DRAW-SCRATCH")
                    < child_close("RC-ATT-DRAW-COVERAGE")
            );
            let framebuffer_last_use = events
                .iter()
                .position(|event| {
                    event.ledger_id == "RC-ATT-DRAW-FB-X2"
                        && event.phase == OwnerEventPhase::LastUse
                        && event.parent_handle == Some(parent)
                })
                .unwrap();
            assert!(child_close("RC-ATT-DRAW-COVERAGE") < framebuffer_last_use);
            for id in ["RC-ATT-DRAW-CLIP", "RC-ATT-DRAW-SCRATCH", "RC-ATT-DRAW-COVERAGE"] {
                let borrowed = events
                    .iter()
                    .find(|event| {
                        event.ledger_id == id
                            && event.phase == OwnerEventPhase::Borrow
                            && event.parent_handle == Some(parent)
                    })
                    .unwrap();
                assert!(mrt_aliases.insert((
                    borrowed.handle.registry,
                    borrowed.handle.slot,
                    borrowed.handle.generation,
                )));
            }
        }
        assert_eq!(mrt_aliases.len(), 6);
        assert!(
            event_position(events, "RC-FN-DRAW-F", OwnerEventPhase::Release)
                < event_position(events, "RC-FN-DRAW-V", OwnerEventPhase::Release)
        );
        assert!(
            event_position(events, "RC-FN-DRAW-V", OwnerEventPhase::Release)
                < event_position(events, "RC-DRAW-LAMBDA-GPU", OwnerEventPhase::Release)
        );

        let assert_all_opened_draw_owners_close = |metal: &RecordingMetal| {
            for (id, closing) in [
                ("RC-NS-FUNCTION-NAME-V", OwnerEventPhase::AliasEnd),
                ("RC-NS-FUNCTION-NAME-F", OwnerEventPhase::AliasEnd),
                ("RC-PD-DRAW-X2", OwnerEventPhase::Release),
                ("RC-ATT-DRAW-FB-X2", OwnerEventPhase::AliasEnd),
                ("RC-ATT-DRAW-FB-X2", OwnerEventPhase::ReleaseStrong),
                ("RC-DRAW-LAMBDA-GPU", OwnerEventPhase::Release),
                ("RC-ATT-DRAW-CLIP", OwnerEventPhase::AliasEnd),
                ("RC-ATT-DRAW-SCRATCH", OwnerEventPhase::AliasEnd),
                ("RC-ATT-DRAW-COVERAGE", OwnerEventPhase::AliasEnd),
                ("RC-FN-DRAW-V", OwnerEventPhase::Release),
                ("RC-FN-DRAW-F", OwnerEventPhase::Release),
            ] {
                for close in metal
                    .owner_events
                    .iter()
                    .filter(|event| event.ledger_id == id && event.phase == closing)
                {
                    assert_eq!(
                        metal
                            .retirements
                            .iter()
                            .filter(|retired| **retired == close.handle)
                            .count(),
                        1,
                        "{id} {:?} must correspond to exactly one actual owner/alias close",
                        close.phase
                    );
                    assert_eq!(
                        metal
                            .owner_events
                            .iter()
                            .filter(|event| {
                                event.ledger_id == id && event.handle == close.handle
                            })
                            .last()
                            .map(|event| event.phase),
                        Some(closing),
                        "{id} handle remains open after its claimed close"
                    );
                }
            }
        };
        assert_all_opened_draw_owners_close(&metal);

        for (selector, occurrences) in [
            ("newFunctionWithName:", 2usize),
            ("alloc/init", 2),
            ("colorAttachments", 8),
            ("colorAttachmentAtIndex:", 8),
            ("newRenderPipelineStateWithDescriptor:error:", 2),
        ] {
            for occurrence in 1..=occurrences {
                let metal = run_draw_pipeline_scenario(Some((selector, occurrence)), None);
                assert_eq!(metal.fail_exact, None, "draw {selector}#{occurrence}");
                assert!(metal.selector_occurrence_count(selector) >= occurrence);
                if matches!(
                    selector,
                    "alloc/init" | "colorAttachments" | "colorAttachmentAtIndex:"
                ) {
                    assert_eq!(
                        metal.selector_occurrence_count("alloc/init"),
                        2,
                        "both RGBA/BGRA descriptor expressions execute"
                    );
                    assert_eq!(
                        metal.selector_occurrence_count("colorAttachments"),
                        8,
                        "all four attachment expressions execute in both formats"
                    );
                    assert_eq!(
                        metal.selector_occurrence_count("colorAttachmentAtIndex:"),
                        8,
                        "the indexed nil message is not short-circuited"
                    );
                    assert_eq!(
                        metal.selector_occurrence_count(
                            "newRenderPipelineStateWithDescriptor:error:"
                        ),
                        2,
                        "both pipeline-state calls execute even with nil descriptor children"
                    );
                    if occurrence == 1 {
                        match selector {
                            "alloc/init" => {
                                assert!(metal
                                    .calls
                                    .iter()
                                    .filter(|call| call.selector == "colorAttachments")
                                    .take(4)
                                    .all(|call| {
                                        call.args.first()
                                            == Some(&Value::Handle(Handle::NIL))
                                    }));
                                assert!(metal
                                    .calls
                                    .iter()
                                    .find(|call| {
                                        call.selector
                                            == "newRenderPipelineStateWithDescriptor:error:"
                                    })
                                    .unwrap()
                                    .args
                                    .contains(&Value::Handle(Handle::NIL)));
                            }
                            "colorAttachments" => {
                                let child = metal
                                    .calls
                                    .iter()
                                    .find(|call| call.selector == "colorAttachmentAtIndex:")
                                    .unwrap();
                                assert!(child.args.contains(&Value::Handle(Handle::NIL)));
                                assert_eq!(
                                    metal
                                        .calls
                                        .iter()
                                        .find(|call| call.selector == "setPixelFormat:")
                                        .unwrap()
                                        .args
                                        .first(),
                                    Some(&Value::Handle(Handle::NIL))
                                );
                            }
                            "colorAttachmentAtIndex:" => {
                                assert_eq!(
                                    metal
                                        .calls
                                        .iter()
                                        .find(|call| call.selector == "setPixelFormat:")
                                        .unwrap()
                                        .args
                                        .first(),
                                    Some(&Value::Handle(Handle::NIL))
                                );
                            }
                            _ => unreachable!(),
                        }
                    }
                }
                assert_all_opened_draw_owners_close(&metal);
            }
        }
        for (kind, occurrences) in [
            (MetalObjectKind::Device, 1usize),
            (
                MetalObjectKind::RenderPipelineColorAttachmentDescriptor,
                2,
            ),
        ] {
            for occurrence in 1..=occurrences {
                let metal = run_draw_pipeline_scenario(None, Some((kind, occurrence)));
                assert_eq!(
                    metal.fail_clone_exact, None,
                    "draw {kind:?} clone #{occurrence}"
                );
                assert_all_opened_draw_owners_close(&metal);
                match kind {
                    MetalObjectKind::Device => {
                        assert!(row_events(
                            &metal.owner_events,
                            "RC-DRAW-LAMBDA-GPU"
                        )
                        .is_empty());
                        assert!(row_events(&metal.owner_events, "RC-PD-DRAW-X2").is_empty());
                        for id in ["RC-NS-FUNCTION-NAME-V", "RC-NS-FUNCTION-NAME-F"] {
                            assert_eq!(
                                row_events(&metal.owner_events, id)
                                    .iter()
                                    .map(|event| event.phase)
                                    .collect::<Vec<_>>(),
                                vec![OwnerEventPhase::CreateBridge, OwnerEventPhase::AliasEnd]
                            );
                        }
                    }
                    MetalObjectKind::RenderPipelineColorAttachmentDescriptor => {
                        assert_eq!(
                            metal.selector_occurrence_count(
                                "newRenderPipelineStateWithDescriptor:error:"
                            ),
                            occurrence - 1,
                            "a failed named-local retain must not use the borrowed child or continue to the next format"
                        );
                        assert_eq!(
                            row_events(&metal.owner_events, "RC-ATT-DRAW-FB-X2").len(),
                            (occurrence - 1) * 5 + 2
                        );
                        assert_eq!(
                            row_events(&metal.owner_events, "RC-PD-DRAW-X2").len(),
                            (occurrence - 1) * 3 + 2
                        );
                    }
                    _ => unreachable!(),
                }
            }
        }

        for id in [
            "RC-PD-DRAW-X2",
            "RC-ATT-DRAW-FB-X2",
            "RC-DRAW-LAMBDA-GPU",
            "RC-ATT-DRAW-CLIP",
            "RC-ATT-DRAW-SCRATCH",
            "RC-ATT-DRAW-COVERAGE",
            "RC-FN-DRAW-V",
            "RC-FN-DRAW-F",
        ] {
            let row = expectation_row(id);
            assert_eq!(row.len(), 11);
            assert!(!row[1].is_empty());
            assert!(row[2].contains("DrawPipeline"));
            assert!(!row[3].is_empty());
            assert!(!row[4].is_empty());
            assert!(!row[5].is_empty());
            assert!(!row[6].is_empty());
            assert!(!row[7].is_empty());
            assert_ne!(row[8], "none");
            assert!(row[9].contains("actual") || row[9].contains("force"));
            assert!(!row[10].is_empty());
        }
    }

    fn run_resize_owner_scenario(
        id: &'static str,
        width: u32,
        height: u32,
        fail_selector: Option<&'static str>,
    ) -> RecordingMetal {
        let mut metal = RecordingMetal::default();
        let device = metal.device_handle();
        let mut context =
            RenderContextMetal::new(&mut metal, device, ContextOptions::default());
        metal.owner_events.clear();
        metal.retirements.clear();
        metal.calls.clear();
        if let Some(selector) = fail_selector {
            let occurrence = metal.selector_occurrence_count(selector) + 1;
            metal.fail_exact = Some((selector, occurrence));
        }
        match id {
            "RC-TD-GRAD-RESIZE" => context.resize_gradient(&mut metal, width, height),
            "RC-TD-TESS-RESIZE" => context.resize_tessellation(&mut metal, width, height),
            "RC-TD-FEATHER-RESIZE" => context.resize_feather(&mut metal, width, height),
            _ => panic!("unknown resize owner row {id}"),
        }
        drop(context);
        metal.drain_recorded_clone_drops();
        metal
    }

    #[test]
    fn resource_scenarios_bind_descriptor_spans_nil_paths_and_failure_ordinals() {
        for id in [
            "RC-TD-GRAD-RESIZE",
            "RC-TD-TESS-RESIZE",
            "RC-TD-FEATHER-RESIZE",
        ] {
            let metal = run_resize_owner_scenario(id, 4, 4, None);
            let actual = row_events(&metal.owner_events, id);
            assert_eq!(actual.len(), 3, "{id} success phases");
            assert_eq!(
                actual.iter().map(|event| event.phase).collect::<Vec<_>>(),
                vec![
                    OwnerEventPhase::Create,
                    OwnerEventPhase::LastUse,
                    OwnerEventPhase::Release,
                ]
            );
            assert!(actual.iter().all(|event| {
                event.native_identity == actual[0].native_identity
                    && event.handle == actual[0].handle
            }));
            assert_eq!(actual[0].selector_ordinal.unwrap().0, "alloc/init");
            assert!(metal.calls.iter().any(|call| {
                call.selector == "newTextureWithDescriptor:"
                    && call.args.iter().any(|value| {
                        matches!(value, Value::Handle(handle) if *handle == actual[0].handle)
                    })
            }));
            if id == "RC-TD-FEATHER-RESIZE" {
                assert_eq!(
                    actual[1].selector_ordinal.unwrap().0,
                    "newRenderPipelineStateWithDescriptor:error:"
                );
            } else {
                assert_eq!(
                    actual[1].selector_ordinal.unwrap().0,
                    "newTextureWithDescriptor:"
                );
            }
            let row = expectation_row(id);
            assert_eq!(row.len(), 11);
            let expected_phases = if id == "RC-TD-FEATHER-RESIZE" {
                "Create>LastUse(texture replacement and lazy fill/stroke pipeline construction)>Release"
            } else {
                "Create>LastUse(texture replacement)>Release"
            };
            assert_eq!(row[4], expected_phases);
            assert_eq!(
                row[5],
                if id == "RC-TD-FEATHER-RESIZE" {
                    "0_or_1 per call"
                } else {
                    "0_or_1"
                }
            );
            assert_ne!(row[8], "none");
            assert!(row[9].contains("actual"));

            let zero = run_resize_owner_scenario(id, 0, 0, None);
            assert!(row_events(&zero.owner_events, id).is_empty(), "{id} zero path");

            let alloc_failure = run_resize_owner_scenario(id, 4, 4, Some("alloc/init"));
            assert_eq!(alloc_failure.fail_exact, None, "{id} alloc failpoint");
            assert!(
                row_events(&alloc_failure.owner_events, id).is_empty(),
                "{id} nil descriptor has no owner"
            );

            let texture_failure =
                run_resize_owner_scenario(id, 4, 4, Some("newTextureWithDescriptor:"));
            assert_eq!(texture_failure.fail_exact, None, "{id} texture failpoint");
            assert_eq!(
                row_events(&texture_failure.owner_events, id)
                    .iter()
                    .map(|event| event.phase)
                    .collect::<Vec<_>>(),
                vec![
                    OwnerEventPhase::Create,
                    OwnerEventPhase::LastUse,
                    OwnerEventPhase::Release,
                ],
                "{id} descriptor drains after nil texture"
            );
        }

        // Reuse one canonical context so replacement and the feather lazy
        // branch cannot be accidentally certified by separate fixtures.
        for id in [
            "RC-TD-GRAD-RESIZE",
            "RC-TD-TESS-RESIZE",
            "RC-TD-FEATHER-RESIZE",
        ] {
            let mut metal = RecordingMetal::default();
            let device = metal.device_handle();
            let mut context =
                RenderContextMetal::new(&mut metal, device, ContextOptions::default());
            metal.owner_events.clear();
            metal.retirements.clear();
            metal.calls.clear();
            match id {
                "RC-TD-GRAD-RESIZE" => context.resize_gradient(&mut metal, 4, 4),
                "RC-TD-TESS-RESIZE" => context.resize_tessellation(&mut metal, 4, 4),
                "RC-TD-FEATHER-RESIZE" => context.resize_feather(&mut metal, 4, 4),
                _ => unreachable!(),
            }
            let first = context
                .resized_texture_handle_for_test(id)
                .expect("first nonzero resize installs its texture member");
            if id == "RC-TD-FEATHER-RESIZE" {
                assert!(context.feather_pipelines_initialized_for_test());
                assert_eq!(
                    row_events(&metal.owner_events, id)[1]
                        .selector_ordinal
                        .unwrap()
                        .0,
                    "newRenderPipelineStateWithDescriptor:error:"
                );
            }

            metal.owner_events.clear();
            metal.retirements.clear();
            metal.calls.clear();
            match id {
                "RC-TD-GRAD-RESIZE" => context.resize_gradient(&mut metal, 8, 8),
                "RC-TD-TESS-RESIZE" => context.resize_tessellation(&mut metal, 8, 8),
                "RC-TD-FEATHER-RESIZE" => context.resize_feather(&mut metal, 8, 8),
                _ => unreachable!(),
            }
            let second = context
                .resized_texture_handle_for_test(id)
                .expect("second nonzero resize replaces its texture member");
            assert_ne!(first, second, "{id} member replacement identity");
            let second_descriptor = row_events(&metal.owner_events, id);
            assert_eq!(
                second_descriptor
                    .iter()
                    .map(|event| event.phase)
                    .collect::<Vec<_>>(),
                vec![
                    OwnerEventPhase::Create,
                    OwnerEventPhase::LastUse,
                    OwnerEventPhase::Release,
                ]
            );
            assert_eq!(
                second_descriptor[1].selector_ordinal.unwrap().0,
                "newTextureWithDescriptor:"
            );
            if id == "RC-TD-FEATHER-RESIZE" {
                assert!(context.feather_pipelines_initialized_for_test());
                assert!(!metal.calls.iter().any(|call| {
                    call.selector == "newRenderPipelineStateWithDescriptor:error:"
                }));
            }
        }

        for pipeline_occurrence in 1..=2 {
            let mut metal = RecordingMetal::default();
            let device = metal.device_handle();
            let mut context =
                RenderContextMetal::new(&mut metal, device, ContextOptions::default());
            metal.owner_events.clear();
            metal.retirements.clear();
            metal.calls.clear();
            let selector = "newRenderPipelineStateWithDescriptor:error:";
            let baseline = metal.selector_occurrence_count(selector);
            metal.fail_exact = Some((selector, baseline + pipeline_occurrence));
            context.resize_feather(&mut metal, 4, 4);
            assert_eq!(metal.fail_exact, None, "feather pipeline failpoint");
            assert_eq!(
                row_events(&metal.owner_events, "RC-TD-FEATHER-RESIZE")
                    .iter()
                    .map(|event| event.phase)
                    .collect::<Vec<_>>(),
                vec![
                    OwnerEventPhase::Create,
                    OwnerEventPhase::LastUse,
                    OwnerEventPhase::Release,
                ]
            );
            assert!(context.feather_pipelines_initialized_for_test());
        }

        let run_upload = |fail_selector: Option<&'static str>| {
            let mut metal = RecordingMetal::default();
            if let Some(selector) = fail_selector {
                metal.fail_exact = Some((selector, 1));
            }
            let device = metal.device_handle();
            let texture = TextureMetal::new(
                &mut metal,
                device,
                4,
                4,
                2,
                Arc::from(vec![0u8; 80]),
                PixelFormat::RGBA8Unorm,
                1,
                1,
                4,
                false,
            );
            drop(texture);
            metal
        };
        let upload = run_upload(None);
        let upload_row = row_events(&upload.owner_events, "RC-TD-IMAGE-UPLOAD");
        assert_eq!(upload_row.len(), 3);
        assert_eq!(upload_row[0].phase, OwnerEventPhase::Create);
        assert_eq!(upload_row[1].phase, OwnerEventPhase::LastUse);
        assert_eq!(upload_row[2].phase, OwnerEventPhase::Release);
        assert_eq!(upload_row[0].selector_ordinal.unwrap(), ("alloc/init", 1));
        assert_eq!(
            upload_row[1].selector_ordinal.unwrap(),
            ("replaceRegion:mipmapLevel:withBytes:bytesPerRow:", 2)
        );
        assert_eq!(expectation_row("RC-TD-IMAGE-UPLOAD")[5], "1");
        let upload_alloc_failure = run_upload(Some("alloc/init"));
        assert_eq!(upload_alloc_failure.fail_exact, None);
        assert!(row_events(
            &upload_alloc_failure.owner_events,
            "RC-TD-IMAGE-UPLOAD"
        )
        .is_empty());
        let upload_texture_failure = run_upload(Some("newTextureWithDescriptor:"));
        assert_eq!(upload_texture_failure.fail_exact, None);
        assert_eq!(
            row_events(
                &upload_texture_failure.owner_events,
                "RC-TD-IMAGE-UPLOAD"
            )
            .len(),
            3
        );
        let nil_uploads = upload_texture_failure
            .calls
            .iter()
            .filter(|call| {
                call.selector == "replaceRegion:mipmapLevel:withBytes:bytesPerRow:"
            })
            .collect::<Vec<_>>();
        assert_eq!(nil_uploads.len(), 2);
        assert!(nil_uploads.iter().all(|call| {
            matches!(call.args.first(), Some(Value::Handle(handle)) if *handle == Handle::NIL)
        }));

        // Exercise the source block-compressed pointer walk separately from
        // the ordinary RGBA rows above. Each selector receives exactly the
        // authored mip span and row pitch, including the rounded block grid.
        let mut compressed = RecordingMetal::default();
        let compressed_bytes = (0u8..80).collect::<Vec<_>>();
        let device = compressed.device_handle();
        let compressed_texture = TextureMetal::new(
            &mut compressed,
            device,
            7,
            5,
            2,
            Arc::from(compressed_bytes.clone()),
            PixelFormat::ASTC4x4Ldr,
            4,
            4,
            16,
            false,
        )
        .expect("valid two-level ASTC upload");
        drop(compressed_texture);
        let uploads = compressed
            .calls
            .iter()
            .filter(|call| {
                call.selector == "replaceRegion:mipmapLevel:withBytes:bytesPerRow:"
            })
            .collect::<Vec<_>>();
        assert_eq!(uploads.len(), 2);
        for (call, level, width, height, range, row_pitch) in [
            (uploads[0], 0u64, 7usize, 5usize, 0usize..64usize, 32u64),
            (uploads[1], 1u64, 3usize, 2usize, 64usize..80usize, 16u64),
        ] {
            assert!(matches!(
                call.args.as_slice(),
                [
                    Value::Handle(_),
                    Value::Origin(_),
                    Value::Size(Size { width: actual_width, height: actual_height, depth: 1 }),
                    Value::U64(actual_level),
                    Value::Bytes(bytes),
                    Value::U64(actual_row_pitch),
                ] if *actual_width == width
                    && *actual_height == height
                    && *actual_level == level
                    && bytes.as_ref() == &compressed_bytes[range]
                    && *actual_row_pitch == row_pitch
            ));
        }
        let compressed_descriptor = row_events(
            &compressed.owner_events,
            "RC-TD-IMAGE-UPLOAD",
        );
        assert_eq!(compressed_descriptor.len(), 3);
        assert_eq!(
            compressed_descriptor[1].selector_ordinal,
            Some(("replaceRegion:mipmapLevel:withBytes:bytesPerRow:", 2))
        );

        let mut short = RecordingMetal::default();
        let device = short.device_handle();
        assert!(TextureMetal::new(
            &mut short,
            device,
            7,
            5,
            2,
            Arc::from(vec![0u8; 79]),
            PixelFormat::ASTC4x4Ldr,
            4,
            4,
            16,
            false,
        )
        .is_none());
        assert!(short.calls.is_empty(), "safe admission precedes source selectors");
        assert!(row_events(&short.owner_events, "RC-TD-IMAGE-UPLOAD").is_empty());

        let mut mip_metal = RecordingMetal::default();
        let device = mip_metal.device_handle();
        let texture = TextureMetal::new(
            &mut mip_metal,
            device,
            4,
            4,
            2,
            Arc::from(vec![0u8; 80]),
            PixelFormat::RGBA8Unorm,
            1,
            1,
            4,
            true,
        )
        .unwrap();
        mip_metal.owner_events.clear();
        mip_metal.retirements.clear();
        texture.mark_mipmaps_dirty_for_test();
        let command = Handle::new(900, MetalObjectKind::CommandBuffer);
        let texture_handle = texture.native_handle();
        texture.ensure_mipmaps(&mut mip_metal, command);
        assert!(!texture.mipmaps_dirty_for_test());
        let generate = mip_metal
            .calls
            .iter()
            .find(|call| call.selector == "generateMipmapsForTexture:")
            .expect("dirty source texture generates its remaining mip levels");
        assert_eq!(generate.args.len(), 2);
        let Value::Handle(encoder) = generate.args[0] else {
            panic!("generateMipmaps receiver must be the blit encoder");
        };
        assert_eq!(encoder.kind, MetalObjectKind::BlitCommandEncoder);
        assert_eq!(generate.args[1], Value::Handle(texture_handle));
        let mip = row_events(&mip_metal.owner_events, "RC-ENC-MIP");
        assert_eq!(
            mip.iter().map(|event| event.phase).collect::<Vec<_>>(),
            vec![
                OwnerEventPhase::Create,
                OwnerEventPhase::LastUse,
                OwnerEventPhase::Release,
            ]
        );
        assert_eq!(mip[0].selector_ordinal.unwrap().0, "blitCommandEncoder");
        assert_eq!(mip[1].selector_ordinal.unwrap().0, "endEncoding");
        let mut nil_mip = RecordingMetal::default();
        let device = nil_mip.device_handle();
        let texture = TextureMetal::new(
            &mut nil_mip,
            device,
            4,
            4,
            2,
            Arc::from(vec![0u8; 80]),
            PixelFormat::RGBA8Unorm,
            1,
            1,
            4,
            true,
        )
        .unwrap();
        nil_mip.owner_events.clear();
        let occurrence = nil_mip.selector_occurrence_count("blitCommandEncoder") + 1;
        nil_mip.fail_exact = Some(("blitCommandEncoder", occurrence));
        texture.mark_mipmaps_dirty_for_test();
        texture.ensure_mipmaps(
            &mut nil_mip,
            Handle::new(901, MetalObjectKind::CommandBuffer),
        );
        assert_eq!(nil_mip.fail_exact, None);
        assert!(!texture.mipmaps_dirty_for_test());
        assert!(row_events(&nil_mip.owner_events, "RC-ENC-MIP").is_empty());

        let run_target = |fail: Option<(&'static str, usize)>| {
            let mut metal = RecordingMetal::default();
            metal.fail_exact = fail;
            let device = metal.device_handle();
            let owner = metal
                .clone_owned(device, MetalObjectKind::Device)
                .unwrap_or_else(|| OwnedMetalHandle::token(device));
            let target = RenderTargetMetal::new_with_device(
                &mut metal,
                owner,
                PixelFormat::RGBA8Unorm,
                4,
                4,
                gpu::PlatformFeatures {
                    supportsRasterOrderingMode: true,
                    ..gpu::PlatformFeatures::default()
                },
            );
            drop(target);
            metal.drain_recorded_clone_drops();
            metal
        };
        let target = run_target(None);
        assert_repeated_exact_triplets(
            &target.owner_events,
            "RC-TD-MEMORYLESS-X3",
            3,
            OwnerEventPhase::Create,
            OwnerEventPhase::Release,
        );
        for occurrence in 1..=3 {
            let failed = run_target(Some(("alloc/init", occurrence)));
            assert_eq!(failed.fail_exact, None, "memoryless alloc #{occurrence}");
            assert_eq!(
                row_events(&failed.owner_events, "RC-TD-MEMORYLESS-X3").len(),
                6,
                "two surviving descriptor scopes"
            );
            let failed = run_target(Some(("newTextureWithDescriptor:", occurrence)));
            assert_eq!(failed.fail_exact, None, "memoryless texture #{occurrence}");
            assert_eq!(
                row_events(&failed.owner_events, "RC-TD-MEMORYLESS-X3").len(),
                9,
                "nil texture still drains all descriptors"
            );
        }
    }

    #[test]
    fn gaussian_constructor_descriptor_spans_all_later_members_and_exact_failures() {
        let run = |fail: Option<(&'static str, usize)>| {
            let mut metal = RecordingMetal::default();
            metal.fail_exact = fail;
            let device = metal.device_handle();
            let context =
                RenderContextMetal::new(&mut metal, device, ContextOptions::default());
            drop(context);
            metal.drain_recorded_clone_drops();
            metal
        };
        let baseline = run(None);
        let gaussian = row_events(&baseline.owner_events, "RC-TD-GAUSSIAN");
        assert_eq!(
            gaussian.iter().map(|event| event.phase).collect::<Vec<_>>(),
            vec![
                OwnerEventPhase::Create,
                OwnerEventPhase::LastUse,
                OwnerEventPhase::Release,
            ]
        );
        assert!(gaussian.iter().all(|event| {
            event.handle == gaussian[0].handle
                && event.native_identity == gaussian[0].native_identity
        }));
        let descriptor_ordinal = gaussian[0].selector_ordinal.unwrap();
        assert_eq!(descriptor_ordinal.0, "alloc/init");
        assert_eq!(
            gaussian[1].selector_ordinal,
            baseline.calls.last().map(|call| {
                (
                    call.selector,
                    baseline.selector_occurrence_count(call.selector),
                )
            }),
            "Gaussian descriptor must survive the final constructor selector"
        );
        let gaussian_release = event_position(
            &baseline.owner_events,
            "RC-TD-GAUSSIAN",
            OwnerEventPhase::Release,
        );
        let data_release = event_position(
            &baseline.owner_events,
            "RC-DD-METALLIB",
            OwnerEventPhase::Release,
        );
        assert!(gaussian_release < data_release);

        let row = expectation_row("RC-TD-GAUSSIAN");
        assert_eq!(row.len(), 11);
        assert_eq!(row[5], "1");
        assert!(row[4].contains("all later ctor members"));
        assert!(row[7].contains("all pipelines/buffers"));
        assert_eq!(row[8], "context_ctor#gaussian_descriptor_alloc");

        let descriptor_failure = run(Some(descriptor_ordinal));
        assert_eq!(descriptor_failure.fail_exact, None);
        assert!(row_events(
            &descriptor_failure.owner_events,
            "RC-TD-GAUSSIAN"
        )
        .is_empty());

        let mut texture_occurrence = 0usize;
        let gaussian_texture_ordinal = baseline
            .calls
            .iter()
            .find_map(|call| {
                if call.selector != "newTextureWithDescriptor:" {
                    return None;
                }
                texture_occurrence += 1;
                call.args
                    .iter()
                    .any(|value| {
                        matches!(value, Value::Handle(handle) if *handle == gaussian[0].handle)
                    })
                    .then_some((call.selector, texture_occurrence))
            })
            .expect("Gaussian texture selector ordinal");
        let texture_failure = run(Some(gaussian_texture_ordinal));
        assert_eq!(texture_failure.fail_exact, None);
        assert_eq!(
            row_events(&texture_failure.owner_events, "RC-TD-GAUSSIAN")
                .iter()
                .map(|event| event.phase)
                .collect::<Vec<_>>(),
            vec![
                OwnerEventPhase::Create,
                OwnerEventPhase::LastUse,
                OwnerEventPhase::Release,
            ]
        );

        let later_selector = "newBufferWithBytes:length:options:";
        let later_occurrence = baseline.selector_occurrence_count(later_selector);
        assert!(later_occurrence > 0);
        let later_failure = run(Some((later_selector, later_occurrence)));
        assert_eq!(later_failure.fail_exact, None);
        assert_eq!(
            row_events(&later_failure.owner_events, "RC-TD-GAUSSIAN").len(),
            3,
            "Gaussian descriptor drains after a later constructor failure"
        );
    }

    fn run_constructor_creation_outcome(
        library_object: bool,
        library_error: bool,
    ) -> (RecordingMetal, RenderContextMetal) {
        let mut metal = RecordingMetal::default();
        if !library_object {
            metal.fail_exact = Some(("newLibraryWithData:error:", 1));
        }
        if library_error {
            metal.errors.push_back((
                "newLibraryWithData:error:",
                "scripted metallib error".into(),
            ));
        }
        let device = metal.device_handle();
        let context = RenderContextMetal::new(&mut metal, device, ContextOptions::default());
        (metal, context)
    }

    #[test]
    fn constructor_sampler_dispatch_and_metallib_outcomes_bind_exact_source_scopes() {
        let (metal, context) = run_constructor_creation_outcome(true, false);
        let samplers = row_events(&metal.owner_events, "RC-SD-IMAGE-X18");
        assert_eq!(samplers.len(), 18 * 3);
        let mut sampler_descriptors = BTreeSet::new();
        for (index, events) in samplers.chunks_exact(3).enumerate() {
            assert_eq!(
                events.iter().map(|event| event.phase).collect::<Vec<_>>(),
                vec![
                    OwnerEventPhase::Create,
                    OwnerEventPhase::LastUse,
                    OwnerEventPhase::Release,
                ]
            );
            assert!(events.iter().all(|event| {
                event.handle == events[0].handle
                    && event.native_identity == events[0].native_identity
            }));
            let identity = events[0].native_identity;
            assert!(sampler_descriptors.insert((
                identity.registry,
                identity.slot,
                identity.generation,
            )));
            assert_eq!(events[0].selector_ordinal, Some(("new", index + 1)));
            assert_eq!(
                events[1].selector_ordinal,
                Some(("newSamplerStateWithDescriptor:", index + 1))
            );
            assert!(metal.retirements.contains(&events[0].handle));
        }
        assert_eq!(
            metal.selector_occurrence_count("newSamplerStateWithDescriptor:"),
            18
        );
        let dispatch = row_events(&metal.owner_events, "RC-DD-METALLIB");
        assert_eq!(
            dispatch.iter().map(|event| event.phase).collect::<Vec<_>>(),
            vec![
                OwnerEventPhase::Create,
                OwnerEventPhase::LastUse,
                OwnerEventPhase::Release,
            ]
        );
        assert!(dispatch.iter().all(|event| {
            event.handle == dispatch[0].handle
                && event.native_identity == dispatch[0].native_identity
        }));
        assert!(metal.retirements.contains(&dispatch[0].handle));
        let dispatch_call = metal
            .calls
            .iter()
            .find(|call| call.selector == "dispatch_data_create")
            .unwrap();
        assert!(matches!(
            dispatch_call.args.as_slice(),
            [Value::Bytes(bytes)] if bytes.as_ref() == canonical_metallib().as_ref()
        ));
        assert!(context.has_precompiled_library_for_test());
        drop(context);

        for (selector, occurrences) in [
            ("new", 18usize),
            ("newSamplerStateWithDescriptor:", 18usize),
        ] {
            for occurrence in 1..=occurrences {
                let mut metal = RecordingMetal::default();
                metal.fail_exact = Some((selector, occurrence));
                let device = metal.device_handle();
                let context =
                    RenderContextMetal::new(&mut metal, device, ContextOptions::default());
                drop(context);
                assert_eq!(metal.fail_exact, None, "{selector}#{occurrence}");
                let row = row_events(&metal.owner_events, "RC-SD-IMAGE-X18");
                assert_eq!(row.len() % 3, 0);
                for events in row.chunks_exact(3) {
                    assert_eq!(events[0].phase, OwnerEventPhase::Create);
                    assert_eq!(events[1].phase, OwnerEventPhase::LastUse);
                    assert_eq!(events[2].phase, OwnerEventPhase::Release);
                }
            }
        }

        for (library_object, library_error) in
            [(true, false), (true, true), (false, true), (false, false)]
        {
            let (metal, context) =
                run_constructor_creation_outcome(library_object, library_error);
            assert_eq!(context.has_precompiled_library_for_test(), library_object);
            assert_eq!(row_events(&metal.owner_events, "RC-DD-METALLIB").len(), 3);
            let errors = row_events(&metal.owner_events, "RC-ERR-METALLIB");
            if library_error {
                assert_eq!(
                    errors.iter().map(|event| event.phase).collect::<Vec<_>>(),
                    vec![
                        OwnerEventPhase::Create,
                        OwnerEventPhase::LastUse,
                        OwnerEventPhase::Release,
                    ]
                );
                assert!(
                    event_position(
                        &metal.owner_events,
                        "RC-ERR-METALLIB",
                        OwnerEventPhase::Release,
                    ) < event_position(
                        &metal.owner_events,
                        "RC-DD-METALLIB",
                        OwnerEventPhase::Release,
                    )
                );
            } else {
                assert!(errors.is_empty());
            }
            if library_error || !library_object {
                assert!(!context.has_color_ramp_pipeline_for_test());
                let library_call = metal
                    .calls
                    .iter()
                    .position(|call| call.selector == "newLibraryWithData:error:")
                    .unwrap();
                assert!(metal.calls[library_call + 1..]
                    .iter()
                    .all(|call| call.receiver == "host" && call.selector == "log"));
            } else {
                assert!(context.has_color_ramp_pipeline_for_test());
            }
            drop(context);
        }
    }

    #[cfg(feature = "native-ore-metal-experimental")]
    #[test]
    fn canvas_source_scenario_binds_three_retain_ladder_and_nil_allocation() {
        let run = |fail_selector: Option<&'static str>| {
            let mut metal = RecordingMetal::default();
            let device = metal.device_handle();
            let mut context =
                RenderContextMetal::new(&mut metal, device, ContextOptions::default());
            metal.owner_events.clear();
            metal.retirements.clear();
            metal.calls.clear();
            if let Some(selector) = fail_selector {
                let occurrence = metal.selector_occurrence_count(selector) + 1;
                metal.fail_exact = Some((selector, occurrence));
            }
            let result = context.make_render_canvas(&mut metal, 4, 4);
            if let Some((image, target, descriptor)) = result {
                // This is the exact outer adapter boundary after image,
                // target, and RenderCanvas construction. The production
                // MechanicalRenderContext emits the same LastUse immediately
                // before retiring the descriptor.
                metal.owner_event("RC-TD-CANVAS", OwnerEventPhase::LastUse, descriptor);
                metal.retire_handle(descriptor);
                metal.owner_event("RC-TD-CANVAS", OwnerEventPhase::Release, descriptor);
                drop(target);
                drop(image);
            }
            drop(context);
            metal.drain_recorded_clone_drops();
            metal
        };

        let metal = run(None);
        let descriptor = row_events(&metal.owner_events, "RC-TD-CANVAS");
        assert_eq!(
            descriptor.iter().map(|event| event.phase).collect::<Vec<_>>(),
            vec![
                OwnerEventPhase::Create,
                OwnerEventPhase::LastUse,
                OwnerEventPhase::Release,
            ]
        );
        assert!(descriptor.iter().all(|event| {
            event.handle == descriptor[0].handle
                && event.native_identity == descriptor[0].native_identity
        }));
        let texture = row_events(&metal.owner_events, "RC-TEX-CANVAS-LOCAL");
        assert_eq!(
            texture.iter().map(|event| event.phase).collect::<Vec<_>>(),
            vec![
                OwnerEventPhase::Create,
                OwnerEventPhase::CloneToTarget,
                OwnerEventPhase::CloneToImage,
                OwnerEventPhase::ReleaseLocal,
            ]
        );
        assert!(texture.iter().all(|event| {
            event.native_identity == texture[0].native_identity
        }));
        assert_ne!(texture[0].handle, texture[1].handle);
        assert_ne!(texture[0].handle, texture[2].handle);
        assert_ne!(texture[1].handle, texture[2].handle);
        assert_eq!(texture[1].source_handle, texture[0].handle);
        assert_eq!(texture[2].source_handle, texture[0].handle);
        assert!(metal.retirements.contains(&texture[0].handle));
        assert!(metal.retirements.contains(&texture[1].handle));
        assert!(metal.retirements.contains(&texture[2].handle));

        for id in ["RC-TD-CANVAS", "RC-TEX-CANVAS-LOCAL"] {
            let row = expectation_row(id);
            assert_eq!(row.len(), 11);
            assert_eq!(row[5], "1");
            assert_ne!(row[8], "none");
            assert!(row[9].contains("actual"));
        }

        let texture_failure = run(Some("newTextureWithDescriptor:"));
        assert_eq!(texture_failure.fail_exact, None);
        assert_eq!(
            row_events(&texture_failure.owner_events, "RC-TD-CANVAS")
                .iter()
                .map(|event| event.phase)
                .collect::<Vec<_>>(),
            vec![
                OwnerEventPhase::Create,
                OwnerEventPhase::LastUse,
                OwnerEventPhase::Release,
            ]
        );
        assert!(row_events(
            &texture_failure.owner_events,
            "RC-TEX-CANVAS-LOCAL"
        )
        .is_empty());

        let descriptor_failure = run(Some("alloc/init"));
        assert_eq!(descriptor_failure.fail_exact, None);
        assert!(row_events(&descriptor_failure.owner_events, "RC-TD-CANVAS").is_empty());
    }

    /// The IDs are an audit inventory only.  The gate below is driven by the
    /// owner events emitted by the executable source paths; it must never
    /// derive events from this list or from a selector/kind table.
    const LOCAL_OWNER_LEDGER_IDS: &[&str] = &[
        "RC-ERR-PIPE",
        "RC-STATE-PIPE",
        "RC-PD-COLOR",
        "RC-FN-COLOR-V",
        "RC-FN-COLOR-F",
        "RC-ATT-COLOR-0",
        "RC-PD-TESS",
        "RC-FN-TESS-V",
        "RC-FN-TESS-F",
        "RC-ATT-TESS-0",
        "RC-PD-FEATHER",
        "RC-FN-FEATHER-V",
        "RC-FN-FEATHER-F",
        "RC-ATT-FEATHER-0-X9",
        "RC-NS-FUNCTION-NAME-V",
        "RC-NS-FUNCTION-NAME-F",
        "RC-PD-DRAW-X2",
        "RC-ATT-DRAW-FB-X2",
        "RC-DRAW-LAMBDA-GPU",
        "RC-ATT-DRAW-CLIP",
        "RC-ATT-DRAW-SCRATCH",
        "RC-ATT-DRAW-COVERAGE",
        "RC-FN-DRAW-V",
        "RC-FN-DRAW-F",
        "RC-SD-IMAGE-X18",
        "RC-DD-METALLIB",
        "RC-ERR-METALLIB",
        "RC-TD-GAUSSIAN",
        "RC-TD-MEMORYLESS-X3",
        "RC-TD-IMAGE-UPLOAD",
        "RC-ENC-MIP",
        "RC-TD-CANVAS",
        "RC-TEX-CANVAS-LOCAL",
        "RC-TD-GRAD-RESIZE",
        "RC-TD-TESS-RESIZE",
        "RC-TD-FEATHER-RESIZE",
        "RC-CB-RETAINED",
        "RC-CB-TRANSFER",
        "RC-CB-FLUSH-STRONG",
        "RC-CB-POST-STRONG",
        "RC-BLOCK-COMPLETE",
        "RC-PS-GRAD",
        "RC-RPD-GRAD",
        "RC-RPA-GRAD-0",
        "RC-ENC-GRAD",
        "RC-PS-TESS",
        "RC-RPD-TESS",
        "RC-RPA-TESS-0",
        "RC-ENC-TESS",
        "RC-PS-ATLAS-FILL",
        "RC-PS-ATLAS-STROKE",
        "RC-RPD-ATLAS",
        "RC-RPA-ATLAS-0",
        "RC-ENC-ATLAS",
        "RC-RPD-MAIN",
        "RC-RPA-MAIN-COLOR",
        "RC-RPA-MAIN-CLIP",
        "RC-RPA-MAIN-SCRATCH",
        "RC-RPA-MAIN-COVERAGE",
        "RC-ENC-COPY",
        "RC-ENC-HELPER",
        "RC-ENC-MAIN",
        "RC-PS-DRAW",
        "RC-ATT-COLLECTION-PIPE",
        "RC-ATT-COLLECTION-PASS",
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
        "RC-STATIC-NS-LITERALS",
        "EXCL-OBJCPARAMS",
        "EXCL-CPP-OWNERS",
        "LEDGER-GATE",
    ];

    fn validate_owner_event_stream(
        events: &[OwnerEvent],
        retirements: &[Handle],
        expected_ids: &[&str],
    ) -> Result<(), String> {
        use std::collections::BTreeSet;

        let expected: BTreeSet<&str> = expected_ids.iter().copied().collect();
        let actual: BTreeSet<&str> = events.iter().map(|event| event.ledger_id).collect();
        for id in &expected {
            if !actual.contains(id) {
                return Err(format!("actual source path emitted no {id} event"));
            }
        }
        let mut open: Vec<(&str, Handle)> = Vec::new();
        let mut borrowed: Vec<(&str, Handle)> = Vec::new();
        let mut released: Vec<(&str, Handle)> = Vec::new();
        for event in events
            .iter()
            .filter(|event| expected.contains(event.ledger_id))
        {
            if event.handle == Handle::NIL {
                return Err(format!("{} emitted a NIL owner", event.ledger_id));
            }
            match event.phase {
                OwnerEventPhase::Create
                | OwnerEventPhase::CreateBridge
                | OwnerEventPhase::CreateClone
                | OwnerEventPhase::CreateStrong => {
                    open.push((event.ledger_id, event.native_identity));
                }
                OwnerEventPhase::Transfer | OwnerEventPhase::CopyTransfer => {
                    // The completion block is a stack local copied into the
                    // command buffer.  Its transfer is a phase transition of
                    // the borrowed block identity, not a second +1 owner.
                    if event.ledger_id == "RC-ENC-HELPER" {
                        let key = (event.ledger_id, event.native_identity);
                        if !open.contains(&key) {
                            return Err(format!(
                                "{} transferred {:?} before helper creation",
                                event.ledger_id, event.handle
                            ));
                        }
                        released.push(key);
                    } else if !matches!(
                        event.ledger_id,
                        "RC-BLOCK-COMPLETE" | "RC-CB-RETAINED" | "RC-STATE-PIPE"
                    ) {
                        open.push((event.ledger_id, event.native_identity));
                    }
                }
                OwnerEventPhase::Borrow
                | OwnerEventPhase::BorrowAlias
                | OwnerEventPhase::BorrowStack => {
                    // The dynamic NSString rows use Borrow to record the
                    // selector use of the already-open +0 bridge alias.  It
                    // is a phase of that alias, not a second borrowed owner.
                    let bridge_alias_is_already_open = matches!(
                        event.ledger_id,
                        "RC-NS-FUNCTION-NAME-V" | "RC-NS-FUNCTION-NAME-F"
                    ) && open.contains(&(event.ledger_id, event.native_identity));
                    if !bridge_alias_is_already_open {
                        borrowed.push((event.ledger_id, event.native_identity));
                    }
                }
                OwnerEventPhase::LastUse
                | OwnerEventPhase::Invoke
                | OwnerEventPhase::CloneToTarget
                | OwnerEventPhase::CloneToImage => {}
                OwnerEventPhase::Release
                | OwnerEventPhase::ReleaseStrong
                | OwnerEventPhase::ReleaseLocal
                | OwnerEventPhase::ReleaseCopy
                | OwnerEventPhase::AliasEnd => {
                    let key = (event.ledger_id, event.native_identity);
                    if !open.contains(&key) && !borrowed.contains(&key) {
                        return Err(format!(
                            "{} released {:?} before its actual create/borrow",
                            event.ledger_id, event.handle
                        ));
                    }
                    let parent_tied_collection = event.ledger_id.starts_with("RC-ATT-COLLECTION-");
                    if !parent_tied_collection && !retirements.contains(&event.handle) {
                        return Err(format!(
                            "{} release {:?} has no executor retirement",
                            event.ledger_id, event.handle
                        ));
                    }
                    let outstanding = open.iter().filter(|candidate| **candidate == key).count()
                        + borrowed
                            .iter()
                            .filter(|candidate| **candidate == key)
                            .count()
                        - released
                            .iter()
                            .filter(|candidate| **candidate == key)
                            .count();
                    if outstanding == 0 {
                        return Err(format!(
                            "{} released {:?} more than once",
                            event.ledger_id, event.handle
                        ));
                    }
                    released.push(key);
                }
            }
        }
        for id in expected {
            let created = open.iter().filter(|(event_id, _)| *event_id == id).count();
            let aliases = borrowed
                .iter()
                .filter(|(event_id, _)| *event_id == id)
                .count();
            let releases = released
                .iter()
                .filter(|(event_id, _)| *event_id == id)
                .count();
            if created + aliases != releases {
                return Err(format!(
                    "{id} has {created} creates + {aliases} borrows but {releases} releases"
                ));
            }
        }
        Ok(())
    }

    fn assert_repeated_owner_pairs(
        rows: &BTreeMap<&str, Vec<OwnerEvent>>,
        id: &str,
        count: usize,
    ) {
        let row = rows.get(id).unwrap_or_else(|| panic!("missing {id}"));
        assert_eq!(row.len(), count * 2, "{id} multiplicity");
        for pair in row.chunks_exact(2) {
            assert_eq!(pair[0].phase, OwnerEventPhase::Create, "{id} create phase");
            assert_eq!(pair[1].phase, OwnerEventPhase::Release, "{id} release phase");
            assert_eq!(
                pair[0].native_identity,
                pair[1].native_identity,
                "{id} native identity"
            );
        }
    }

    fn assert_repeated_clone_triplets(
        rows: &BTreeMap<&str, Vec<OwnerEvent>>,
        id: &str,
        count: usize,
    ) {
        let row = rows.get(id).unwrap_or_else(|| panic!("missing {id}"));
        assert_eq!(row.len(), count * 3, "{id} multiplicity");
        for events in row.chunks_exact(3) {
            assert_eq!(events[0].phase, OwnerEventPhase::CreateClone);
            assert_eq!(events[1].phase, OwnerEventPhase::LastUse);
            assert_eq!(events[2].phase, OwnerEventPhase::Release);
            assert!(events
                .iter()
                .all(|event| event.handle == events[0].handle
                    && event.source_handle == events[0].source_handle
                    && event.native_identity == events[0].native_identity));
            assert_ne!(events[0].handle, events[0].source_handle);
        }
    }

    fn assert_owner_pairs_for_each_actual_instance(
        rows: &BTreeMap<&str, Vec<OwnerEvent>>,
        id: &str,
    ) {
        let row = rows.get(id).unwrap_or_else(|| panic!("missing {id}"));
        if row.len() % 3 == 0
            && row
                .chunks_exact(3)
                .all(|events| events[1].phase == OwnerEventPhase::LastUse)
        {
            for events in row.chunks_exact(3) {
                assert_eq!(events[0].phase, OwnerEventPhase::Create, "{id} create phase");
                assert_eq!(events[2].phase, OwnerEventPhase::Release, "{id} release phase");
                assert!(events.iter().all(|event| {
                    event.native_identity == events[0].native_identity
                }));
            }
            return;
        }
        assert!(!row.is_empty() && row.len() % 2 == 0, "{id} pair shape");
        for pair in row.chunks_exact(2) {
            assert_eq!(pair[0].phase, OwnerEventPhase::Create, "{id} create phase");
            assert_eq!(pair[1].phase, OwnerEventPhase::Release, "{id} release phase");
            assert_eq!(
                pair[0].native_identity,
                pair[1].native_identity,
                "{id} native identity"
            );
        }
    }

    fn assert_repeated_alias_pairs(
        rows: &BTreeMap<&str, Vec<OwnerEvent>>,
        id: &str,
        count: usize,
    ) {
        let row = rows.get(id).unwrap_or_else(|| panic!("missing {id}"));
        assert_eq!(row.len(), count * 3, "{id} multiplicity");
        for triplet in row.chunks_exact(3) {
            assert_eq!(triplet[0].phase, OwnerEventPhase::Borrow, "{id} borrow phase");
            assert_eq!(triplet[1].phase, OwnerEventPhase::LastUse, "{id} last use");
            assert_eq!(triplet[2].phase, OwnerEventPhase::AliasEnd, "{id} alias end");
            assert_ne!(triplet[0].handle, Handle::NIL, "{id} nil alias");
            assert_eq!(
                triplet[0].native_identity,
                triplet[2].native_identity,
                "{id} native identity"
            );
        }
    }

    fn assert_repeated_owner_triplets(
        rows: &BTreeMap<&str, Vec<OwnerEvent>>,
        id: &str,
        count: usize,
    ) {
        let row = rows.get(id).unwrap_or_else(|| panic!("missing {id}"));
        assert_eq!(row.len(), count * 3, "{id} multiplicity");
        for triple in row.chunks_exact(3) {
            assert_eq!(triple[0].phase, OwnerEventPhase::Create, "{id} create phase");
            assert_eq!(triple[1].phase, OwnerEventPhase::LastUse, "{id} last use");
            assert_eq!(triple[2].phase, OwnerEventPhase::Release, "{id} release phase");
            assert!(
                triple
                    .iter()
                    .all(|event| event.native_identity == triple[0].native_identity)
            );
        }
    }

    #[cfg(target_vendor = "apple")]
    fn collect_native_background_owner_events() -> Vec<
        crate::mechanical_port::source::renderer::src::metal::background_shader_compiler_mm::BackgroundOwnerEvent,
    > {
        objc2::rc::autoreleasepool(|_| {
            use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp as gpu;
            use crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::MetalFeatures;
            use crate::mechanical_port::source::renderer::src::metal::background_shader_compiler_mm::{
                self as background, BackgroundCompileJob,
            };
            let device = objc2_metal::MTLCreateSystemDefaultDevice()
                .expect("required live Metal device for native owner evidence");
            let _ = background::take_owner_events();
            let source_device = unsafe {
                crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::Retained::from_raw_retained(
                    objc2::rc::Retained::into_raw(device).cast(),
                )
            }
            .expect("device ownership transfer");
            let mut compiler = background::new_for_device_with_sources(
                source_device,
                MetalFeatures::default(),
                background::GeneratedShaderSources {
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
            compiler.pushJob(BackgroundCompileJob::new(
                gpu::DrawType::ImageMesh,
                gpu::ShaderFeatures::NONE,
                gpu::InterlockMode::rasterOrdering,
                gpu::ShaderMiscFlags::none,
            ));
            let mut finished = BackgroundCompileJob::new(
                gpu::DrawType::ImageMesh,
                gpu::ShaderFeatures::NONE,
                gpu::InterlockMode::rasterOrdering,
                gpu::ShaderMiscFlags::none,
            );
            assert!(compiler.popFinishedJob(&mut finished, true));
            compiler.pushJob(BackgroundCompileJob::new(
                gpu::DrawType::ImageMesh,
                gpu::ShaderFeatures::ENABLE_CLIPPING,
                gpu::InterlockMode::rasterOrdering,
                gpu::ShaderMiscFlags::none,
            ));
            assert!(compiler.popFinishedJob(&mut finished, true));
            drop(finished);
            drop(compiler);
            background::take_owner_events()
        })
    }

    #[cfg(target_vendor = "apple")]
    fn collect_native_background_failure_events() -> Vec<
        crate::mechanical_port::source::renderer::src::metal::background_shader_compiler_mm::BackgroundOwnerEvent,
    > {
        objc2::rc::autoreleasepool(|_| {
            use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp as gpu;
            use crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::MetalFeatures;
            use crate::mechanical_port::source::renderer::src::metal::background_shader_compiler_mm::{
                self as background, BackgroundCompileJob,
            };
            let device = objc2_metal::MTLCreateSystemDefaultDevice()
                .expect("required live Metal device for native owner evidence");
            let _ = background::take_owner_events();
            let source_device = unsafe {
                crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::Retained::from_raw_retained(
                    objc2::rc::Retained::into_raw(device).cast(),
                )
            }
            .expect("device ownership transfer");
            let mut compiler = background::new_for_device_with_sources(
                source_device,
                MetalFeatures::default(),
                background::GeneratedShaderSources {
                    metal: "#include <metal_stdlib>\nusing namespace metal;\n#error forced-owner-error\n",
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
            compiler.pushJob(BackgroundCompileJob::new(
                gpu::DrawType::ImageMesh,
                gpu::ShaderFeatures::NONE,
                gpu::InterlockMode::rasterOrdering,
                gpu::ShaderMiscFlags::none,
            ));
            let mut finished = BackgroundCompileJob::new(
                gpu::DrawType::ImageMesh,
                gpu::ShaderFeatures::NONE,
                gpu::InterlockMode::rasterOrdering,
                gpu::ShaderMiscFlags::none,
            );
            assert!(compiler.popFinishedJob(&mut finished, true));
            drop(finished);
            drop(compiler);
            background::take_owner_events()
        })
    }

    #[cfg(target_vendor = "apple")]
    fn validate_native_background_owner_events(
        events: &[crate::mechanical_port::source::renderer::src::metal::background_shader_compiler_mm::BackgroundOwnerEvent],
    ) {
        use crate::mechanical_port::source::renderer::src::metal::background_shader_compiler_mm::BackgroundOwnerPhase;
        let ids: std::collections::BTreeSet<&str> =
            events.iter().map(|event| event.ledger_id).collect();
        for id in [
            "BG-DICT-DEFINES",
            "BG-NS-SOURCE",
            "BG-COMPILE-OPTIONS",
            "BG-GPU-MEMBER",
            "BG-LIB-COMPILED",
            "BG-NS-APPEND-TEMP",
            "BG-NS-MACRO-LITERALS",
            "BG-NS-MACRO-KEY-DYNAMIC",
        ] {
            assert!(ids.contains(id), "native compiler omitted {id}");
        }
        assert!(events.iter().all(|event| event.identity != 0));

        let row = |id: &str| {
            events
                .iter()
                .filter(|event| event.ledger_id == id)
                .collect::<Vec<_>>()
        };
        for id in ["BG-DICT-DEFINES", "BG-NS-SOURCE", "BG-COMPILE-OPTIONS"] {
            let values = row(id);
            assert_eq!(values.len(), 4, "{id} must have two real jobs");
            for pair in values.chunks_exact(2) {
                assert_eq!(pair[0].phase, BackgroundOwnerPhase::Create, "{id} create");
                assert_eq!(pair[1].phase, BackgroundOwnerPhase::Release, "{id} release");
                assert_eq!(pair[0].identity, pair[1].identity, "{id} identity");
            }
        }
        let gpu = row("BG-GPU-MEMBER");
        assert_eq!(gpu.len(), 2);
        assert_eq!(gpu[0].phase, BackgroundOwnerPhase::Create);
        assert_eq!(gpu[1].phase, BackgroundOwnerPhase::Release);
        assert_eq!(gpu[0].identity, gpu[1].identity);
        let libraries = row("BG-LIB-COMPILED");
        assert_eq!(libraries.len(), 6, "two compile transfers and releases");
        assert_eq!(
            libraries.iter().map(|event| event.phase).collect::<Vec<_>>(),
            vec![
                BackgroundOwnerPhase::Create,
                BackgroundOwnerPhase::Transfer,
                BackgroundOwnerPhase::Create,
                BackgroundOwnerPhase::Transfer,
                BackgroundOwnerPhase::Release,
                BackgroundOwnerPhase::Release,
            ]
        );
        assert_eq!(libraries[0].identity, libraries[1].identity);
        assert_eq!(libraries[2].identity, libraries[3].identity);
        assert_eq!(libraries[4].identity, libraries[0].identity);
        assert_eq!(libraries[5].identity, libraries[2].identity);
        let append = row("BG-NS-APPEND-TEMP");
        assert_eq!(append.len(), 6, "prefix plus two fragments per job");
        assert!(append
            .iter()
            .all(|event| event.phase == BackgroundOwnerPhase::Borrow));
        let literals = row("BG-NS-MACRO-LITERALS");
        assert_eq!(literals.len(), 8, "four fixed macro setters per job");
        assert!(literals
            .iter()
            .all(|event| event.phase == BackgroundOwnerPhase::Borrow));
        let dynamic = row("BG-NS-MACRO-KEY-DYNAMIC");
        assert_eq!(dynamic.len(), 1, "one dynamic feature macro job");
        assert_eq!(dynamic[0].phase, BackgroundOwnerPhase::Borrow);
    }

    #[cfg(target_vendor = "apple")]
    fn validate_native_background_failure_events(
        events: &[crate::mechanical_port::source::renderer::src::metal::background_shader_compiler_mm::BackgroundOwnerEvent],
    ) {
        use crate::mechanical_port::source::renderer::src::metal::background_shader_compiler_mm::BackgroundOwnerPhase;
        assert!(events.iter().all(|event| event.identity != 0));
        for id in [
            "BG-DICT-DEFINES",
            "BG-NS-SOURCE",
            "BG-COMPILE-OPTIONS",
            "BG-GPU-MEMBER",
            "BG-ERR-COMPILE",
            "BG-NS-ERR-DESC",
        ] {
            assert!(events.iter().any(|event| event.ledger_id == id), "missing {id}");
        }
        let error = events
            .iter()
            .filter(|event| event.ledger_id == "BG-ERR-COMPILE")
            .collect::<Vec<_>>();
        assert_eq!(
            error.iter().map(|event| event.phase).collect::<Vec<_>>(),
            vec![BackgroundOwnerPhase::Create, BackgroundOwnerPhase::Release]
        );
        assert_eq!(error[0].identity, error[1].identity);
        let description = events
            .iter()
            .position(|event| event.ledger_id == "BG-NS-ERR-DESC")
            .expect("error description borrow");
        let error_release = events
            .iter()
            .position(|event| event.ledger_id == "BG-ERR-COMPILE" && event.phase == BackgroundOwnerPhase::Release)
            .unwrap();
        assert!(description < error_release, "description outlives NSError");
        let release_order = [
            "BG-COMPILE-OPTIONS",
            "BG-ERR-COMPILE",
            "BG-NS-SOURCE",
            "BG-DICT-DEFINES",
        ]
        .into_iter()
        .map(|id| {
            events
                .iter()
                .position(|event| event.ledger_id == id && event.phase == BackgroundOwnerPhase::Release)
                .unwrap()
        })
        .collect::<Vec<_>>();
        assert!(release_order.windows(2).all(|window| window[0] < window[1]));
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn native_background_owner_stream_uses_real_compiler_path() {
        let events = collect_native_background_owner_events();
        assert!(!events.is_empty(), "native compiler emitted no owner events");
        validate_native_background_owner_events(&events);
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn native_background_compile_failure_matches_source_assert_boundary() {
        const CHILD: &str = "NATIVE_BG_FAILURE_CHILD";
        if std::env::var_os(CHILD).is_none() {
            let status = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "mechanical_port::source::renderer::src::metal::render_context_metal_impl_mm::source_owner_regressions::native_background_compile_failure_matches_source_assert_boundary",
                    "--nocapture",
                ])
                .env(CHILD, "1")
                .status()
                .expect("spawn native BG failure child");
            if cfg!(debug_assertions) {
                assert!(!status.success(), "debug source assert must abort child");
            } else {
                assert!(status.success(), "release source fallback must finish child");
            }
            return;
        }
        let events = collect_native_background_failure_events();
        if !cfg!(debug_assertions) {
            validate_native_background_failure_events(&events);
        }
    }

    #[test]
    fn exhaustive_local_owner_ledger_is_bound_to_actual_boundaries_and_failpoints() {
        use std::collections::{BTreeMap, BTreeSet};

        let inventory: BTreeSet<&str> = LOCAL_OWNER_LEDGER_IDS.iter().copied().collect();
        assert_eq!(inventory.len(), 79, "ledger inventory must remain unique");
        let table_rows: Vec<Vec<&str>> = OWNER_EXPECTATIONS
            .lines()
            .skip(1)
            .map(|line| line.split('\t').collect())
            .collect();
        assert_eq!(table_rows.len(), 79, "owner expectation table row count");
        assert!(
            table_rows.iter().all(|row| row.len() == 11),
            "every owner expectation must retain all 11 contract columns"
        );
        for row in &table_rows {
            for (column, value) in row.iter().enumerate().skip(1) {
                assert!(
                    !value.trim().is_empty(),
                    "owner expectation {} column {} is empty",
                    row[0],
                    column + 1
                );
            }
            for phase in row[4].split('>') {
                let phase = phase.split('(').next().unwrap_or(phase);
                assert!(!phase.trim().is_empty(), "owner expectation {} has empty phase", row[0]);
            }
        }
        assert_eq!(
            table_rows.iter().map(|row| row[0]).collect::<Vec<_>>(),
            LOCAL_OWNER_LEDGER_IDS,
            "expectation rows must remain in pinned source order"
        );
        assert!(
            table_rows
                .iter()
                .all(|row| row[10].starts_with("CLOSED:")),
            "every concrete row and the meta gate require an independent CLOSED disposition"
        );
        let digest = sha2::Sha256::digest(OWNER_EXPECTATIONS.as_bytes());
        assert_eq!(
            format!("{digest:x}"),
            OWNER_EXPECTATIONS_SHA256,
            "expectation table changed without updating its pinned contract"
        );
        RENDER_CONTEXT_OWNER_DROP_EVENTS.lock().unwrap().clear();
        RENDER_CONTEXT_OWNER_DROP_RETIREMENTS.lock().unwrap().clear();

        // This is deliberately a production-path trace.  No selector or
        // MetalObjectKind is synthesized here: the events below are emitted
        // at the authored owner boundaries in the canonical constructor.
        let mut metal = RecordingMetal::default();
        let device = metal.device_handle();
        {
            let mut context =
                RenderContextMetal::new(&mut metal, device, ContextOptions::default());
            context.resize_gradient(&mut metal, 4, 4);
            context.resize_tessellation(&mut metal, 4, 4);
            context.resize_feather(&mut metal, 4, 4);
            // Exercise the source upload constructor with two mip levels,
            // then the authored dirty->blit->clear mipmap path.
            let upload_bytes: Arc<[u8]> = Arc::from(vec![0u8; 80]);
            let uploaded = TextureMetal::new(
                &mut metal,
                device,
                4,
                4,
                2,
                upload_bytes,
                PixelFormat::RGBA8Unorm,
                1,
                1,
                4,
                false,
            )
            .expect("valid two-level source upload");
            uploaded.mark_mipmaps_dirty_for_test();
            uploaded.ensure_mipmaps(&mut metal, Handle::new(87, MetalObjectKind::CommandBuffer));
            // A raster-order target allocates all three source memoryless
            // planes in its constructor. The retained-device lazy atomic
            // getters are called separately below when the Recording backend
            // can provide a native owner.
            let lazy_device = metal
                .clone_owned(device, MetalObjectKind::Device)
                .unwrap_or_else(|| OwnedMetalHandle::token(device));
            let _lazy_target = RenderTargetMetal::new_with_device(
                &mut metal,
                lazy_device,
                PixelFormat::RGBA8Unorm,
                4,
                4,
                gpu::PlatformFeatures {
                    supportsRasterOrderingMode: true,
                    ..gpu::PlatformFeatures::default()
                },
            );
            #[cfg(feature = "native-ore-metal-experimental")]
            if let Some((_image, _target, descriptor)) =
                context.make_render_canvas(&mut metal, 4, 4)
            {
                metal.owner_event("RC-TD-CANVAS", OwnerEventPhase::LastUse, descriptor);
                metal.retire_handle(descriptor);
                metal.owner_event("RC-TD-CANVAS", OwnerEventPhase::Release, descriptor);
            }
            // Keep one caller-produced dynamic precompiled-name pair in the
            // same source fixture.  Its native name producer and DrawPipeline
            // borrow/release events are part of the actual selector ledger;
            // the seeded static pipeline below is a separate flush cache
            // owner.
            let _dynamic_draw_pipeline = DrawPipeline::new(
                &mut metal,
                device,
                Some(Handle::new(2, MetalObjectKind::Library)),
                SourceFunctionName::Dynamic(Handle::new(3, MetalObjectKind::Function)),
                SourceFunctionName::Dynamic(Handle::new(4, MetalObjectKind::Function)),
                DrawType::ImageMesh,
                InterlockMode::RasterOrdering,
                ShaderFeatures(0),
                ShaderMiscFlags(0),
                SynthesizedFailureType::none,
            );
            // Exercise the actual source compiler queue as part of the
            // ownership gate.  On Apple this starts the pinned worker and
            // drives NativeCompileIteration; on RecordingMetal the canonical
            // source path remains a no-op because no native compiler exists.
            let _ = context.find_compatible_pipeline(
                &mut metal,
                super::source_execution::DrawType::ImageMesh,
                super::source_execution::ShaderFeatures(1),
                super::source_execution::InterlockMode::Atomics,
                super::source_execution::ShaderMiscFlags(0),
                super::source_execution::ShaderFeatures(1),
                super::source_execution::SynthesizedFailureType::none,
            );
            // Exercise the complete source flush owner path with a valid,
            // empty linked draw list.  The descriptor is still the exact
            // gpu_hpp type; no test DTO or zeroed enum-bearing memory is used.
            let target_device = metal
                .clone_owned(device, MetalObjectKind::Device)
                .unwrap_or_else(|| OwnedMetalHandle::token(device));
            let mut flush_target = RenderTargetMetal::new_with_device(
                &mut metal,
                target_device,
                PixelFormat::RGBA8Unorm,
                4,
                4,
                gpu::PlatformFeatures::default(),
            );
            // Flush consumes the nine source BufferRing owners through the
            // helper base.  Construct those exact rings before entering the
            // source flush path; an empty descriptor still binds the
            // submitted uniform/coverage rings.
            for name in [
                "flushUniform",
                "path",
                "paint",
                "paintAux",
                "contour",
                "gradSpan",
                "tessSpan",
                "triangle",
                "imageDrawInstance",
            ] {
                context.make_uniform_buffer_ring(&mut metal, name, 16);
            }
            let mut draw_list = gpu::BlockAllocatedLinkedList::default();
            draw_list.push_back(gpu::DrawBatch::new(
                gpu::DrawType::midpointFanPatches,
                gpu::ShaderMiscFlags::none,
                gpu::DrawContents::none,
                3,
                0,
                nuxie_render_api::BlendMode::SrcOver,
                gpu::ImageSampler::LinearClamp(),
                gpu::BarrierFlags::none,
            ));
            // RecordingMetal has no asynchronous native compiler, so seed
            // this exact source cache key with the same canonical DrawPipeline
            // owner that a completed source compile would publish.  The
            // subsequent flush still exercises the real clone/draw/release
            // boundary; no owner event is fabricated by the fixture.
            let draw_key = shader_key_for_test(
                DrawType::MidpointFanPatches,
                ShaderFeatures(1),
                InterlockMode::Atomics,
                ShaderMiscFlags::none,
            );
            context.seed_pipeline_for_test(
                draw_key,
                DrawPipeline::new(
                    &mut metal,
                    device,
                    Some(Handle::new(2, MetalObjectKind::Library)),
                    SourceFunctionName::Static(DRAW_VERTEX_NAME),
                    SourceFunctionName::Static(DRAW_FRAGMENT_NAME),
                    DrawType::MidpointFanPatches,
                    InterlockMode::Atomics,
                    ShaderFeatures(1),
                    ShaderMiscFlags::none,
                    SynthesizedFailureType::none,
                ),
            );
            let atlas_fill = gpu::AtlasDrawBatch {
                scissor: gpu::AABBu16 {
                    left: 0,
                    top: 0,
                    right: 4,
                    bottom: 4,
                },
                patchCount: 1,
                basePatch: 0,
            };
            let atlas_stroke = atlas_fill;
            let mut flush_desc = gpu::FlushDescriptor {
                renderTarget: None,
                combinedShaderFeatures: gpu::ShaderFeatures::NONE,
                interlockMode: gpu::InterlockMode::RasterOrdering,
                msaaSampleCount: 1,
                colorLoadAction: gpu::LoadAction::DontCare,
                colorClearValue: 0,
                coverageClearValue: 0,
                depthClearValue: 0.0,
                stencilClearValue: 0,
                renderTargetUpdateBounds: gpu::IAABB {
                    left: 0,
                    top: 0,
                    right: 4,
                    bottom: 4,
                },
                virtualTileWidth: 0,
                virtualTileHeight: 0,
                manuallyResolved: false,
                fixedFunctionColorOutput: false,
                featherAtlasTextureWidth: 0,
                featherAtlasTextureHeight: 0,
                featherAtlasContentWidth: 4,
                featherAtlasContentHeight: 4,
                coverageBufferPrefix: 0,
                needsCoverageBufferClear: false,
                flushUniformDataOffsetInBytes: 0,
                pathCount: 1,
                firstPath: 0,
                firstPaint: 0,
                firstPaintAux: 0,
                contourCount: 1,
                firstContour: 0,
                gradSpanCount: 1,
                firstGradSpan: 0,
                tessVertexSpanCount: 1,
                firstTessVertexSpan: 0,
                gradDataHeight: 0,
                tessDataHeight: 0,
                clockwiseFillOverride: false,
                hasTriangleVertices: false,
                wireframe: false,
                ditherMode: gpu::DitherMode::none,
                #[cfg(feature = "with-rive-tools")]
                synthesizedFailureType: gpu::SynthesizedFailureType::none,
                externalCommandBuffer: None,
                featherAtlasFillBatches: Some(core::ptr::NonNull::from(&atlas_fill)),
                featherAtlasFillBatchCount: 1,
                featherAtlasStrokeBatches: Some(core::ptr::NonNull::from(&atlas_stroke)),
                featherAtlasStrokeBatchCount: 1,
                drawList: None,
                firstDstBlendBarrier: None,
                unresolvedBarriers: gpu::BarrierFlags::default(),
            };
            unsafe {
                context.flush(
                    &mut metal,
                    &flush_desc,
                    &mut flush_target,
                    Handle::new(88, MetalObjectKind::CommandBuffer),
                );
            }
            // Reuse the same exact descriptor for a non-empty atomic pass;
            // the first pass above covers the main raster-order attachment
            // lifecycle, while this second source call drives per-batch,
            // pipeline, and copy branches.
            flush_desc.combinedShaderFeatures = gpu::ShaderFeatures(1);
            flush_desc.interlockMode = gpu::InterlockMode::Atomics;
            flush_desc.colorLoadAction = gpu::LoadAction::PreserveRenderTarget;
            flush_desc.drawList = Some(core::ptr::NonNull::from(&draw_list));
            flush_target
                .set_target_texture(&mut metal, Some(Handle::new(86, MetalObjectKind::Texture)));
            unsafe {
                context.flush(
                    &mut metal,
                    &flush_desc,
                    &mut flush_target,
                    Handle::new(89, MetalObjectKind::CommandBuffer),
                );
            }
            metal.errors.push_back((
                "newRenderPipelineStateWithDescriptor:error:",
                "ledger-failpoint".into(),
            ));
            let _failed_draw_pipeline = DrawPipeline::new(
                &mut metal,
                device,
                Some(Handle::new(2, MetalObjectKind::Library)),
                SourceFunctionName::Static(DRAW_VERTEX_NAME),
                SourceFunctionName::Static(DRAW_FRAGMENT_NAME),
                super::source_execution::DrawType::ImageMesh,
                super::source_execution::InterlockMode::RasterOrdering,
                super::source_execution::ShaderFeatures(0),
                super::source_execution::ShaderMiscFlags(0),
                super::source_execution::SynthesizedFailureType::none,
            );

            // Drive the real frame lifecycle: the queue is installed through
            // the canonical strong setter, the retained opaque command is
            // created, postFlush installs/copies the completion block, the
            // callback runs, and only then does commit consume the opaque
            // owner.  This is intentionally separate from the install-failure
            // probe below so the success phases have one source frame.
            let queue = Handle::new(90, MetalObjectKind::CommandQueue);
            context.set_command_queue(&mut metal, Some(queue));
            let command = context
                .make_command_buffer(&mut metal)
                .expect("recording command-buffer allocation");
            context.lock_current_ring_for_test();
            unsafe { context.post_flush(&mut metal, command, None) };
            metal.run_next_completed_handler();
            context.commit_command_buffer(&mut metal, Some(command));
        }

        // Exercise both source NSError outcomes at the canonical metallib
        // call-with-error boundary.  The first retains an object+error pair;
        // the second scripts nil+error.  These are real scripted
        // ObjectCreation results, not free-standing owner markers.
        for object_result in [true, false] {
            let mut error_metal = RecordingMetal::default();
            let error_device = error_metal.device_handle();
            if !object_result {
                error_metal
                    .fail
                    .push_back("newLibraryWithData:error:");
            }
            error_metal.errors.push_back((
                "newLibraryWithData:error:",
                if object_result {
                    "object+error".into()
                } else {
                    "nil+error".into()
                },
            ));
            let _partial = RenderContextMetal::new(
                &mut error_metal,
                error_device,
                ContextOptions::default(),
            );
            drop(_partial);
            error_metal.drain_recorded_clone_drops();
            metal.owner_events.extend(error_metal.owner_events);
            metal.retirements.extend(error_metal.retirements);
        }
        metal.drain_recorded_clone_drops();
        metal.owner_events.extend(
            RENDER_CONTEXT_OWNER_DROP_EVENTS
                .lock()
                .unwrap()
                .drain(..),
        );
        metal.retirements.extend(
            RENDER_CONTEXT_OWNER_DROP_RETIREMENTS
                .lock()
                .unwrap()
                .drain(..),
        );
        let events = &metal.owner_events;
        assert!(
            !events.is_empty(),
            "canonical source emitted no owner events"
        );

        let mut proven_native_rows = BTreeSet::new();
        #[cfg(target_vendor = "apple")]
        {
            // RecordingMetal cannot instantiate the native compiler.  Run the
            // real Apple worker here and consume its actual owner stream; the
            // native helper validates identity, job multiplicity, and reverse
            // release order before these rows are marked proven.
            let background_events = collect_native_background_owner_events();
            validate_native_background_owner_events(&background_events);
            proven_native_rows.extend(background_events.iter().map(|event| event.ledger_id));
        }

        let mut by_id: BTreeMap<&str, Vec<OwnerEvent>> = BTreeMap::new();
        for event in events {
            assert!(
                inventory.contains(event.ledger_id),
                "unlisted source owner event {}",
                event.ledger_id
            );
            by_id.entry(event.ledger_id).or_default().push(*event);
        }
        // These are source-defined exclusions, not omitted paths: the native
        // BG rows are proven by the real Apple worker stream above, while the
        // two EXCL rows and static literal have no strong-owner boundary.
        let mut excluded = BTreeSet::from([
            "EXCL-OBJCPARAMS",
            "EXCL-CPP-OWNERS",
            "LEDGER-GATE",
            "RC-STATIC-NS-LITERALS",
        ]);
        #[cfg(not(feature = "native-ore-metal-experimental"))]
        excluded.extend(["RC-TD-CANVAS", "RC-TEX-CANVAS-LOCAL"]);
        excluded.extend(proven_native_rows);
        // C++ assert(false) intentionally aborts debug workers on a real
        // shader compile error.  Those two error-local rows are exercised by
        // the release/subprocess failure campaign, not by this in-process
        // success-and-order gate.
        excluded.extend(["BG-ERR-COMPILE", "BG-NS-ERR-DESC"]);
        let expectation = |id: &str| {
            table_rows
                .iter()
                .find(|row| row[0] == id)
                .unwrap_or_else(|| panic!("missing exclusion contract {id}"))
        };
        let objc_parameters = expectation("EXCL-OBJCPARAMS");
        assert!(objc_parameters[3].contains("borrowed parameter"));
        assert_eq!(objc_parameters[4], "BorrowOnly");
        assert_eq!(objc_parameters[5], "6 source parameters; 4 native object families");
        assert!(objc_parameters[9].contains("actual representative source calls"));
        assert!(objc_parameters[10].starts_with("CLOSED:"));
        let cpp_owners = expectation("EXCL-CPP-OWNERS");
        assert!(cpp_owners[3].contains("out-of-ledger"));
        assert_eq!(cpp_owners[4], "SeparateLedger");
        assert_eq!(cpp_owners[5], "8 core type categories; 11 with RenderCanvas feature");
        assert!(cpp_owners[9].contains("actual RenderCanvas/image/texture"));
        assert!(cpp_owners[10].starts_with("CLOSED:"));
        let static_literals = expectation("RC-STATIC-NS-LITERALS");
        assert!(static_literals[3].contains("immortal"));
        assert_eq!(static_literals[4], "Borrow>LastUse");
        assert_eq!(static_literals[5], "10 scenario occurrences; 9 exact identities");
        assert_eq!(static_literals[8], "newFunctionWithName:[1..10]");
        assert!(static_literals[9].contains("actual source pipeline paths"));
        assert!(static_literals[10].starts_with("CLOSED:"));
        assert!(expectation("LEDGER-GATE")[3].contains("meta"));
        assert!(by_id.get("RC-STATIC-NS-LITERALS").is_none());
        assert!(by_id.get("EXCL-OBJCPARAMS").is_none());
        assert!(by_id.get("EXCL-CPP-OWNERS").is_none());
        assert!(by_id.get("LEDGER-GATE").is_none());
        for row in &table_rows {
            let id = row[0];
            if excluded.contains(id) || id.starts_with("BG-") {
                continue;
            }
            let actual = by_id
                .get(id)
                .unwrap_or_else(|| panic!("owner expectation {id} has no actual source events"));
            let actual_phases: BTreeSet<&str> = actual
                .iter()
                .map(|event| match event.phase {
                    OwnerEventPhase::Create => "Create",
                    OwnerEventPhase::CreateBridge => "CreateBridge",
                    OwnerEventPhase::CreateClone => "CreateClone",
                    OwnerEventPhase::CreateStrong => "CreateStrong",
                    OwnerEventPhase::CloneToTarget => "CloneToTarget",
                    OwnerEventPhase::CloneToImage => "CloneToImage",
                    OwnerEventPhase::Borrow => "Borrow",
                    OwnerEventPhase::BorrowAlias => "BorrowAlias",
                    OwnerEventPhase::BorrowStack => "BorrowStack",
                    OwnerEventPhase::Transfer => "Transfer",
                    OwnerEventPhase::CopyTransfer => "CopyTransfer",
                    OwnerEventPhase::LastUse => "LastUse",
                    OwnerEventPhase::Invoke => "Invoke",
                    OwnerEventPhase::Release => "Release",
                    OwnerEventPhase::ReleaseStrong => "ReleaseStrong",
                    OwnerEventPhase::ReleaseLocal => "ReleaseLocal",
                    OwnerEventPhase::ReleaseCopy => "ReleaseCopy",
                    OwnerEventPhase::AliasEnd => "AliasEnd",
                })
                .collect();
            for phase in row[4].split('>') {
                let phase = phase.split('(').next().unwrap_or(phase);
                if matches!(phase, "Create" | "Borrow" | "Transfer" | "Release") {
                    let present = match phase {
                        "Create" => actual_phases.iter().any(|actual| actual.starts_with("Create")),
                        "Borrow" => actual_phases.iter().any(|actual| actual.starts_with("Borrow")),
                        "Release" => {
                            actual_phases.contains("Release")
                                || actual_phases.contains("ReleaseStrong")
                                || actual_phases.contains("ReleaseLocal")
                                || actual_phases.contains("ReleaseCopy")
                                || actual_phases.contains("AliasEnd")
                        }
                        "Transfer" => {
                            actual_phases.contains("Transfer")
                                || actual_phases.contains("CopyTransfer")
                        }
                        _ => unreachable!(),
                    };
                    assert!(
                        present,
                        "owner expectation {id} requires actual {phase} boundary"
                    );
                }
            }
        }
        let missing_concrete: Vec<&str> = inventory
            .iter()
            .copied()
            .filter(|id| !by_id.contains_key(id) && !excluded.contains(id))
            .collect();
        assert!(by_id.contains_key("RC-PD-COLOR"));
        assert!(by_id.contains_key("RC-PD-TESS"));
        assert!(by_id.contains_key("RC-ERR-PIPE"));
        for id in [
            "RC-PD-FEATHER",
            "RC-FN-FEATHER-V",
            "RC-FN-FEATHER-F",
            "RC-ATT-FEATHER-0-X9",
            "RC-TD-GRAD-RESIZE",
            "RC-TD-TESS-RESIZE",
            "RC-TD-FEATHER-RESIZE",
            "RC-SD-IMAGE-X18",
            "RC-DD-METALLIB",
            "RC-NS-FUNCTION-NAME-V",
            "RC-NS-FUNCTION-NAME-F",
            "RC-PD-DRAW-X2",
            "RC-ATT-DRAW-FB-X2",
            "RC-ATT-DRAW-CLIP",
            "RC-ATT-DRAW-SCRATCH",
            "RC-ATT-DRAW-COVERAGE",
            "RC-FN-DRAW-V",
            "RC-FN-DRAW-F",
            "RC-PS-DRAW",
            "RC-DRAW-LAMBDA-GPU",
        ] {
            assert!(by_id.contains_key(id), "source path did not emit {id}");
        }

        // These are the concrete transient rows that this RecordingMetal
        // fixture actually drives to their lexical release boundary.  The
        // verifier counts Create/Transfer as ownership and Borrow as an
        // alias; a missing, duplicated, swapped, or early Release is a hard
        // failure.  Rows retained by the canonical context or unavailable on
        // a non-native RecordingMetal path are checked separately below.
        const STRICT_ROWS: &[&str] = &[
            "RC-ATT-COLOR-0",
            "RC-ERR-PIPE",
            "RC-ATT-COLLECTION-PASS",
            "RC-ATT-COLLECTION-PIPE",
            "RC-ATT-DRAW-CLIP",
            "RC-ATT-DRAW-COVERAGE",
            "RC-ATT-DRAW-FB-X2",
            "RC-ATT-DRAW-SCRATCH",
            "RC-ATT-FEATHER-0-X9",
            "RC-ATT-TESS-0",
            "RC-DD-METALLIB",
            "RC-DRAW-LAMBDA-GPU",
            "RC-BLOCK-COMPLETE",
            "RC-CB-POST-STRONG",
            "RC-CB-RETAINED",
            "RC-CB-TRANSFER",
            "RC-CB-FLUSH-STRONG",
            "RC-ENC-HELPER",
            "RC-ENC-MAIN",
            "RC-FN-COLOR-F",
            "RC-FN-DRAW-F",
            "RC-FN-DRAW-V",
            "RC-FN-FEATHER-F",
            "RC-FN-FEATHER-V",
            "RC-FN-TESS-F",
            "RC-NS-FUNCTION-NAME-F",
            "RC-NS-FUNCTION-NAME-V",
            "RC-PD-COLOR",
            "RC-PD-DRAW-X2",
            "RC-PD-FEATHER",
            "RC-PD-TESS",
            "RC-PS-DRAW",
            "RC-RPA-MAIN-CLIP",
            "RC-RPA-MAIN-COLOR",
            "RC-RPA-MAIN-COVERAGE",
            "RC-RPA-MAIN-SCRATCH",
            "RC-RPD-MAIN",
            "RC-SD-IMAGE-X18",
            "RC-TD-FEATHER-RESIZE",
            "RC-TD-GAUSSIAN",
            "RC-TD-GRAD-RESIZE",
            "RC-TD-TESS-RESIZE",
        ];
        if let Err(error) = validate_owner_event_stream(events, &metal.retirements, STRICT_ROWS) {
            panic!("actual source owner stream failed exact lifecycle validation: {error}");
        }
        let all_concrete_rows: Vec<&str> = inventory
            .iter()
            .copied()
            .filter(|id| !excluded.contains(id) && !id.starts_with("BG-"))
            .collect();
        if let Err(error) =
            validate_owner_event_stream(events, &metal.retirements, &all_concrete_rows)
        {
            panic!("concrete source owner rows failed exhaustive lifecycle validation: {error}");
        }
        assert_owner_pairs_for_each_actual_instance(&by_id, "RC-SD-IMAGE-X18");
        assert_repeated_owner_triplets(&by_id, "RC-TD-MEMORYLESS-X3", 3);
        assert_repeated_alias_pairs(&by_id, "RC-ATT-FEATHER-0-X9", 18);
        assert_owner_pairs_for_each_actual_instance(&by_id, "RC-PD-DRAW-X2");
        assert_repeated_clone_triplets(&by_id, "RC-CB-FLUSH-STRONG", 2);
        assert_repeated_clone_triplets(&by_id, "RC-PS-DRAW", 1);
        assert_repeated_owner_triplets(&by_id, "RC-ERR-PIPE", 1);
        let retained = by_id.get("RC-CB-RETAINED").unwrap();
        assert_eq!(
            retained.iter().map(|event| event.phase).collect::<Vec<_>>(),
            vec![
                OwnerEventPhase::Create,
                OwnerEventPhase::Transfer,
                OwnerEventPhase::Release
            ]
        );
        assert_eq!(retained[0].handle, retained[1].handle);
        assert_eq!(retained[1].handle, retained[2].handle);
        let transfer = by_id.get("RC-CB-TRANSFER").unwrap();
        assert_eq!(
            transfer.iter().map(|event| event.phase).collect::<Vec<_>>(),
            vec![
                OwnerEventPhase::Transfer,
                OwnerEventPhase::LastUse,
                OwnerEventPhase::Release
            ]
        );
        assert!(transfer.iter().all(|event| event.handle == retained[0].handle));
        let post = by_id.get("RC-CB-POST-STRONG").unwrap();
        assert_eq!(
            post.iter().map(|event| event.phase).collect::<Vec<_>>(),
            vec![
                OwnerEventPhase::CreateClone,
                OwnerEventPhase::LastUse,
                OwnerEventPhase::Release
            ]
        );
        assert!(
            post.iter()
                .all(|event| event.native_identity == retained[0].native_identity)
        );
        assert_ne!(
            post[0].handle, retained[0].handle,
            "postFlush strong local must use its independent clone alias"
        );
        let block = by_id.get("RC-BLOCK-COMPLETE").unwrap();
        assert_eq!(
            block.iter().map(|event| event.phase).collect::<Vec<_>>(),
            vec![
                OwnerEventPhase::BorrowStack,
                OwnerEventPhase::CopyTransfer,
                OwnerEventPhase::Invoke,
                OwnerEventPhase::ReleaseCopy
            ]
        );
        assert!(block
            .iter()
            .all(|event| event.native_identity == block[0].native_identity));
        assert_ne!(block[0].native_identity, retained[0].native_identity);
        assert!(block
            .iter()
            .all(|event| event.parent_handle == Some(post[0].handle)));
        let state = by_id.get("RC-STATE-PIPE").unwrap();
        assert_eq!(state.len(), 12);
        for pair in state[..8].chunks_exact(2) {
            assert_eq!(pair[0].phase, OwnerEventPhase::Create);
            assert_eq!(pair[1].phase, OwnerEventPhase::Transfer);
            assert_eq!(pair[0].handle, pair[1].handle);
        }
        assert!(state[8..].iter().all(|event| event.phase == OwnerEventPhase::Release));
        assert_eq!(
            state[8..]
                .iter()
                .map(|event| event.handle)
                .collect::<Vec<_>>(),
            state[..8]
                .chunks_exact(2)
                .rev()
                .map(|pair| pair[0].handle)
                .collect::<Vec<_>>()
        );

        // Mutation discrimination uses the actual event stream just emitted
        // above.  Each mutation must be rejected by the same verifier: no
        // synthetic marker or freely callable event source can certify the
        // gate.
        let mut missing_release = events.clone();
        let release_index = missing_release
            .iter()
            .position(|event| {
                event.ledger_id == "RC-PD-COLOR" && event.phase == OwnerEventPhase::Release
            })
            .expect("actual color descriptor release");
        missing_release.remove(release_index);
        assert!(
            validate_owner_event_stream(&missing_release, &metal.retirements, STRICT_ROWS).is_err(),
            "removing an actual release must fail the owner gate"
        );

        let mut swapped_release = events.clone();
        let color_release = swapped_release
            .iter()
            .position(|event| {
                event.ledger_id == "RC-PD-COLOR" && event.phase == OwnerEventPhase::Release
            })
            .expect("actual color descriptor release");
        let tess_release = swapped_release
            .iter()
            .position(|event| {
                event.ledger_id == "RC-PD-TESS" && event.phase == OwnerEventPhase::Release
            })
            .expect("actual tess descriptor release");
        swapped_release.swap(color_release, tess_release);
        assert!(
            validate_owner_event_stream(&swapped_release, &metal.retirements, STRICT_ROWS).is_err(),
            "swapping actual releases must fail the owner gate"
        );

        let mut early_release = events.clone();
        let color_create = early_release
            .iter()
            .position(|event| {
                event.ledger_id == "RC-PD-COLOR" && event.phase == OwnerEventPhase::Create
            })
            .expect("actual color descriptor create");
        let color_release = early_release
            .iter()
            .position(|event| {
                event.ledger_id == "RC-PD-COLOR" && event.phase == OwnerEventPhase::Release
            })
            .expect("actual color descriptor release");
        early_release.swap(color_create, color_release);
        assert!(
            validate_owner_event_stream(&early_release, &metal.retirements, STRICT_ROWS).is_err(),
            "moving an actual release before create must fail the owner gate"
        );
        assert!(
            missing_concrete.is_empty(),
            "actual owner paths omitted concrete ledger rows: {missing_concrete:?}"
        );

        // Every transient actually emitted by the source must have a
        // create/borrow/transfer before release, with no duplicate release;
        // handles are checked as well so a NIL placeholder cannot certify a
        // live owner row.
        for (id, row) in &by_id {
            let mut first_create = None;
            let mut created: usize = 0;
            let mut releases: usize = 0;
            for (index, event) in row.iter().enumerate() {
                if matches!(
                    event.phase,
                    OwnerEventPhase::Create
                        | OwnerEventPhase::CreateBridge
                        | OwnerEventPhase::CreateClone
                        | OwnerEventPhase::CreateStrong
                        | OwnerEventPhase::Borrow
                        | OwnerEventPhase::BorrowAlias
                ) {
                    if event.phase == OwnerEventPhase::Borrow
                        && matches!(
                            *id,
                            "RC-NS-FUNCTION-NAME-V" | "RC-NS-FUNCTION-NAME-F"
                        )
                        && row
                            .iter()
                            .any(|candidate| candidate.phase == OwnerEventPhase::CreateBridge)
                    {
                        continue;
                    }
                    assert_ne!(event.handle, Handle::NIL, "{id} created NIL owner");
                    first_create.get_or_insert(index);
                    created += 1;
                }
                if event.phase == OwnerEventPhase::Transfer {
                    assert_ne!(event.handle, Handle::NIL, "{id} transferred NIL owner");
                    first_create.get_or_insert(index);
                    if *id == "RC-ENC-HELPER" {
                        releases += 1;
                    }
                }
                if matches!(
                    event.phase,
                    OwnerEventPhase::Release
                        | OwnerEventPhase::ReleaseStrong
                        | OwnerEventPhase::ReleaseLocal
                        | OwnerEventPhase::AliasEnd
                ) {
                    releases += 1;
                    assert_ne!(event.handle, Handle::NIL, "{id} released NIL owner");
                    assert!(first_create.is_some(), "{id} released before creation");
                    assert!(
                        metal.retirements.contains(&event.handle) || *id == "RC-DRAW-LAMBDA-GPU",
                        "{id} release was not emitted at the actual retire boundary"
                    );
                }
            }
            // Transfer is a handoff of an existing +1, not a second owner.
            // Rows that begin at an upstream transfer boundary (for example
            // command-buffer adoption) are checked by their dedicated phase
            // assertions above; every row with a local Create/Borrow must
            // balance exactly at its actual release boundary.
            if created > 0 {
                assert_eq!(
                    releases,
                    created,
                    "{id} owner release count"
                );
            }
        }

        // The source destructor is also a real boundary, not a presence bit:
        // dropping the owner must record its complete reverse teardown trace.
        let trace = RENDER_CONTEXT_METAL_DROP_TRACE.lock().unwrap().clone();
        assert!(
            trace
                .windows(2)
                .any(|pair| pair == ["gpu", "contextOptions"])
        );

        // Drive the actual completion-handler failure path.  This is kept in
        // the same production trace gate because it is the only source path
        // that creates the completion block owner.
        let mut metal = RecordingMetal::default();
        let device = metal.device_handle();
        let mut context = RenderContextMetal::new(&mut metal, device, ContextOptions::default());
        context.lock_current_ring_for_test();
        metal.completed_handler_install_fail = true;
        unsafe {
            context.post_flush(
                &mut metal,
                Handle::new(99, MetalObjectKind::CommandBuffer),
                None,
            );
        }
        assert!(context.current_ring_is_available_for_test());
    }
}

pub(crate) use source_execution::RenderContextMetal as ExecutableRenderContextMetalImpl;

// The complete pinned source follows line-for-line as audit provenance for the
// executable owners and native-context connection above.
// /*
//  * Copyright 2023 Rive
//  */
//
// #include "rive/renderer/metal/render_context_metal_impl.h"
//
// #include "rive/decoders/astc_footprints.hpp"
//
// #include "background_shader_compiler.h"
// #include "rive/renderer/buffer_ring.hpp"
// #ifdef RIVE_CANVAS
// #include "rive/renderer/render_canvas.hpp"
// #include "rive/renderer/ore/ore_context_metal.hpp"
// #endif
// #include "rive/renderer/texture.hpp"
// #include "rive/renderer/rive_render_buffer.hpp"
// #include "shaders/constants.glsl"
// #include <sstream>
//
// #include "generated/shaders/color_ramp.glsl.exports.h"
// #include "generated/shaders/tessellate.glsl.exports.h"
//
// #if defined(RIVE_IOS_SIMULATOR)
// #import <mach-o/arch.h>
// #endif
//
// namespace rive::gpu
// {
// #if defined(RIVE_IOS)
// #include "generated/shaders/rive_pls_ios.metallib.c"
// #elif defined(RIVE_IOS_SIMULATOR)
// #include "generated/shaders/rive_pls_ios_simulator.metallib.c"
// #elif defined(RIVE_XROS)
// #include "generated/shaders/rive_renderer_xros.metallib.c"
// #elif defined(RIVE_XROS_SIMULATOR)
// #include "generated/shaders/rive_renderer_xros_simulator.metallib.c"
// #elif defined(RIVE_APPLETVOS)
// #include "generated/shaders/rive_renderer_appletvos.metallib.c"
// #elif defined(RIVE_APPLETVOS_SIMULATOR)
// #include "generated/shaders/rive_renderer_appletvsimulator.metallib.c"
// #else
// #include "generated/shaders/rive_pls_macosx.metallib.c"
// #endif
//
// static id<MTLRenderPipelineState> make_pipeline_state(
//     id<MTLDevice> gpu, MTLRenderPipelineDescriptor* desc)
// {
//     NSError* err = nil;
//     id<MTLRenderPipelineState> state =
//         [gpu newRenderPipelineStateWithDescriptor:desc error:&err];
//     if (err != nil || state == nil)
//     {
//         NSLog(@"RIVE: make_pipeline_state error %@",
//               err != nil ? err.localizedDescription : @"<nil>");
//     }
//     return state;
// }
//
// static MTLSamplerAddressMode address_mode_for_image_wrap(ImageWrap wrap)
// {
//     switch (wrap)
//     {
//         case ImageWrap::clamp:
//             return MTLSamplerAddressModeClampToEdge;
//         case ImageWrap::repeat:
//             return MTLSamplerAddressModeRepeat;
//         case ImageWrap::mirror:
//             return MTLSamplerAddressModeMirrorRepeat;
//     }
//
//     RIVE_UNREACHABLE();
// }
//
// static MTLSamplerMinMagFilter min_mag_filter_for_image_filter(
//     ImageFilter option)
// {
//     switch (option)
//     {
//         case ImageFilter::bilinear:
//             return MTLSamplerMinMagFilterLinear;
//         case ImageFilter::nearest:
//             return MTLSamplerMinMagFilterNearest;
//     }
//
//     RIVE_UNREACHABLE();
// }
//
// static MTLSamplerMipFilter mip_filter_for_image_filter(ImageFilter option)
// {
//     switch (option)
//     {
//         case ImageFilter::nearest:
//         case ImageFilter::bilinear:
//             return MTLSamplerMipFilterNearest;
//     }
//
//     RIVE_UNREACHABLE();
// }
//
// // Renders color ramps to the gradient texture.
// class RenderContextMetalImpl::ColorRampPipeline
// {
// public:
//     ColorRampPipeline(id<MTLDevice> gpu, id<MTLLibrary> plsLibrary)
//     {
//         MTLRenderPipelineDescriptor* desc =
//             [[MTLRenderPipelineDescriptor alloc] init];
//         desc.vertexFunction =
//             [plsLibrary newFunctionWithName:@GLSL_colorRampVertexMain];
//         desc.fragmentFunction =
//             [plsLibrary newFunctionWithName:@GLSL_colorRampFragmentMain];
//         desc.colorAttachments[0].pixelFormat = MTLPixelFormatRGBA8Unorm;
//         m_pipelineState = make_pipeline_state(gpu, desc);
//     }
//
//     id<MTLRenderPipelineState> pipelineState() const { return m_pipelineState; }
//
// private:
//     id<MTLRenderPipelineState> m_pipelineState;
// };
//
// // Renders tessellated vertices to the tessellation texture.
// class RenderContextMetalImpl::TessellatePipeline
// {
// public:
//     TessellatePipeline(id<MTLDevice> gpu, id<MTLLibrary> plsLibrary)
//     {
//         MTLRenderPipelineDescriptor* desc =
//             [[MTLRenderPipelineDescriptor alloc] init];
//         desc.vertexFunction =
//             [plsLibrary newFunctionWithName:@GLSL_tessellateVertexMain];
//         desc.fragmentFunction =
//             [plsLibrary newFunctionWithName:@GLSL_tessellateFragmentMain];
//         desc.colorAttachments[0].pixelFormat = MTLPixelFormatRGBA32Uint;
//         m_pipelineState = make_pipeline_state(gpu, desc);
//     }
//
//     id<MTLRenderPipelineState> pipelineState() const { return m_pipelineState; }
//
// private:
//     id<MTLRenderPipelineState> m_pipelineState;
// };
//
// // Renders feathered fills and strokes to the feather atlas.
// class RenderContextMetalImpl::FeatherAtlasPipeline
// {
// public:
//     FeatherAtlasPipeline(id<MTLDevice> gpu,
//                          id<MTLLibrary> plsLibrary,
//                          NSString* fragmentMain,
//                          MTLBlendOperation blendOperation)
//     {
//         MTLRenderPipelineDescriptor* desc =
//             [[MTLRenderPipelineDescriptor alloc] init];
//         desc.vertexFunction =
//             [plsLibrary newFunctionWithName:@GLSL_atlasVertexMain];
//         desc.fragmentFunction = [plsLibrary newFunctionWithName:fragmentMain];
//         desc.colorAttachments[0].pixelFormat = MTLPixelFormatR16Float;
//         desc.colorAttachments[0].blendingEnabled = TRUE;
//         desc.colorAttachments[0].sourceRGBBlendFactor = MTLBlendFactorOne;
//         desc.colorAttachments[0].destinationRGBBlendFactor = MTLBlendFactorOne;
//         desc.colorAttachments[0].rgbBlendOperation = blendOperation;
//         desc.colorAttachments[0].sourceAlphaBlendFactor = MTLBlendFactorOne;
//         desc.colorAttachments[0].destinationAlphaBlendFactor =
//             MTLBlendFactorOne;
//         desc.colorAttachments[0].alphaBlendOperation = blendOperation;
//         desc.colorAttachments[0].writeMask = MTLColorWriteMaskAll;
//         m_pipelineState = make_pipeline_state(gpu, desc);
//     }
//
//     id<MTLRenderPipelineState> pipelineState() const { return m_pipelineState; }
//
// private:
//     id<MTLRenderPipelineState> m_pipelineState;
// };
//
// // Renders paths to the main render target.
// class RenderContextMetalImpl::DrawPipeline
// {
// public:
//     // Precompiled functions are embedded in namespaces. Return the fully
//     // qualified name of the desired function, including its namespace.
//     static NSString* GetPrecompiledFunctionName(
//         DrawType drawType,
//         gpu::ShaderFeatures shaderFeatures,
//         gpu::ShaderMiscFlags shaderMiscFlags,
//         id<MTLLibrary> precompiledLibrary,
//         const char* functionBaseName)
//     {
//         // Each feature corresponds to a specific index in the namespaceID.
//         // These must stay in sync with generate_draw_combinations.py.
//         char namespaceID[] = "0000000000";
//         static_assert(sizeof(namespaceID) ==
//                       gpu::kShaderFeatureCount + 1 /*DRAW_INTERIOR_TRIANGLES*/ +
//                           1 /*FEATHER_ATLAS_BLIT*/ + 1 /*null terminator*/);
//         for (size_t i = 0; i < gpu::kShaderFeatureCount; ++i)
//         {
//             const auto feature = ShaderFeatures(1 << i);
//             if (enums::is_flag_set(shaderFeatures, feature))
//             {
//                 namespaceID[i] = '1';
//             }
//             static_assert((int)ShaderFeatures::ENABLE_CLIPPING == 1 << 0);
//             static_assert((int)ShaderFeatures::ENABLE_CLIP_RECT == 1 << 1);
//             static_assert((int)ShaderFeatures::ENABLE_ADVANCED_BLEND == 1 << 2);
//             static_assert((int)ShaderFeatures::ENABLE_FEATHER == 1 << 3);
//             static_assert((int)ShaderFeatures::ENABLE_EVEN_ODD == 1 << 4);
//             static_assert((int)ShaderFeatures::ENABLE_NESTED_CLIPPING ==
//                           1 << 5);
//             static_assert((int)ShaderFeatures::ENABLE_HSL_BLEND_MODES ==
//                           1 << 6);
//             static_assert((int)ShaderFeatures::ENABLE_DITHER == 1 << 7);
//         }
//         if (drawType == DrawType::interiorTriangulation)
//         {
//             namespaceID[gpu::kShaderFeatureCount] = '1';
//         }
//         else if (drawType == DrawType::featherAtlasBlit)
//         {
//             namespaceID[gpu::kShaderFeatureCount] = '1';
//             namespaceID[gpu::kShaderFeatureCount + 1] = '1';
//         }
//
//         char namespacePrefix;
//         switch (drawType)
//         {
//             case DrawType::midpointFanPatches:
//             case DrawType::midpointFanCenterAAPatches:
//             case DrawType::outerCurvePatches:
//             case DrawType::interiorTriangulation:
//             case DrawType::featherAtlasBlit:
//                 namespacePrefix =
//                     enums::is_flag_set(shaderMiscFlags,
//                                        gpu::ShaderMiscFlags::clockwiseFill)
//                         ? 'c'
//                         : 'p';
//                 break;
//             case DrawType::imageRect:
//                 RIVE_UNREACHABLE();
//             case DrawType::imageMesh:
//                 namespacePrefix = 'm';
//                 break;
//             case DrawType::msaaStrokes:
//             case DrawType::msaaMidpointFanBorrowedCoverage:
//             case DrawType::msaaDynamicMidpointFans:
//             case DrawType::msaaMidpointFans:
//             case DrawType::msaaMidpointFanStencilReset:
//             case DrawType::msaaMidpointFanPathsStencil:
//             case DrawType::msaaMidpointFanPathsCover:
//             case DrawType::msaaOuterCubics:
//             case DrawType::clipReset:
//             case DrawType::renderPassInitialize:
//             case DrawType::renderPassResolve:
//                 RIVE_UNREACHABLE();
//         }
//
//         return [NSString stringWithFormat:@"%c%s::%s",
//                                           namespacePrefix,
//                                           namespaceID,
//                                           functionBaseName];
//     }
//
//     DrawPipeline(id<MTLDevice> gpu,
//                  id<MTLLibrary> library,
//                  NSString* vertexFunctionName,
//                  NSString* fragmentFunctionName,
//                  gpu::DrawType drawType,
//                  gpu::InterlockMode interlockMode,
//                  gpu::ShaderFeatures shaderFeatures,
//                  gpu::ShaderMiscFlags shaderMiscFlags
// #ifdef WITH_RIVE_TOOLS
//                  ,
//                  gpu::SynthesizedFailureType synthesizedFailureType
// #endif
//     )
//     {
//         if (library == nil)
//         {
//             // This pipeline is being built from a shader that failed to
//             // compile. Leave everything nil and let draws fail.
//             return;
//         }
//
// #ifdef WITH_RIVE_TOOLS
//         if (synthesizedFailureType == SynthesizedFailureType::pipelineCreation)
//         {
//             NSLog(@"RIVE: Synthesizing pipeline creation failure...");
//             return;
//         }
// #endif
//
//         auto makePipelineState = [=](id<MTLFunction> vertexMain,
//                                      id<MTLFunction> fragmentMain,
//                                      MTLPixelFormat pixelFormat) {
//             MTLRenderPipelineDescriptor* desc =
//                 [[MTLRenderPipelineDescriptor alloc] init];
//             desc.vertexFunction = vertexMain;
//             desc.fragmentFunction = fragmentMain;
//
//             auto* framebuffer = desc.colorAttachments[COLOR_PLANE_IDX];
//             framebuffer.pixelFormat = pixelFormat;
//
//             switch (interlockMode)
//             {
//                 case gpu::InterlockMode::rasterOrdering:
//                     // In rasterOrdering mode, the PLS planes are accessed as
//                     // color attachments.
//                     desc.colorAttachments[CLIP_PLANE_IDX].pixelFormat =
//                         MTLPixelFormatR32Uint;
//                     desc.colorAttachments[SCRATCH_COLOR_PLANE_IDX].pixelFormat =
//                         pixelFormat;
//                     desc.colorAttachments[COVERAGE_PLANE_IDX].pixelFormat =
//                         MTLPixelFormatR32Uint;
//                     break;
//
//                 case gpu::InterlockMode::atomics:
//                     // In atomic mode, the PLS planes are accessed as device
//                     // buffers. We only use the "framebuffer" attachment
//                     // configured above.
//                     if (enums::is_flag_set(
//                             shaderMiscFlags,
//                             gpu::ShaderMiscFlags::fixedFunctionColorOutput))
//                     {
//                         // The shader expectes a "src-over" blend function in
//                         // order to to implement antialiasing and opacity.
//                         framebuffer.blendingEnabled = TRUE;
//                         framebuffer.sourceRGBBlendFactor = MTLBlendFactorOne;
//                         framebuffer.destinationRGBBlendFactor =
//                             MTLBlendFactorOneMinusSourceAlpha;
//                         framebuffer.rgbBlendOperation = MTLBlendOperationAdd;
//                         framebuffer.sourceAlphaBlendFactor = MTLBlendFactorOne;
//                         framebuffer.destinationAlphaBlendFactor =
//                             MTLBlendFactorOneMinusSourceAlpha;
//                         framebuffer.alphaBlendOperation = MTLBlendOperationAdd;
//                         framebuffer.writeMask = MTLColorWriteMaskAll;
//                     }
//                     else if (drawType == gpu::DrawType::renderPassResolve)
//                     {
//                         // We're resolving from the offscreen color buffer to
//                         // the framebuffer attachment. Write out the final color
//                         // directly without any blend modes.
//                         framebuffer.blendingEnabled = FALSE;
//                         framebuffer.writeMask = MTLColorWriteMaskAll;
//                     }
//                     else
//                     {
//                         // This pipeline renders by storing to the offscreen
//                         // color buffer; disable writes to the framebuffer
//                         // attachment.
//                         framebuffer.blendingEnabled = FALSE;
//                         framebuffer.writeMask = MTLColorWriteMaskNone;
//                     }
//                     break;
//
//                 case gpu::InterlockMode::clockwise:
//                 case gpu::InterlockMode::clockwiseAtomic:
//                 case gpu::InterlockMode::msaa:
//                     RIVE_UNREACHABLE();
//             }
//             return make_pipeline_state(gpu, desc);
//         };
//         id<MTLFunction> vertexMain =
//             [library newFunctionWithName:vertexFunctionName];
//         id<MTLFunction> fragmentMain =
//             [library newFunctionWithName:fragmentFunctionName];
//         m_pipelineStateRGBA8 = makePipelineState(
//             vertexMain, fragmentMain, MTLPixelFormatRGBA8Unorm);
//         m_pipelineStateBGRA8 = makePipelineState(
//             vertexMain, fragmentMain, MTLPixelFormatBGRA8Unorm);
//     }
//
//     bool valid() const
//     {
//         assert((m_pipelineStateRGBA8 != nil) == (m_pipelineStateBGRA8 != nil));
//         return m_pipelineStateRGBA8 != nil;
//     }
//
//     id<MTLRenderPipelineState> pipelineState(MTLPixelFormat pixelFormat) const
//     {
//         assert(valid());
//         assert(pixelFormat == MTLPixelFormatRGBA8Unorm ||
//                pixelFormat == MTLPixelFormatRGBA16Float ||
//                pixelFormat == MTLPixelFormatRGBA8Unorm_sRGB ||
//                pixelFormat == MTLPixelFormatBGRA8Unorm ||
//                pixelFormat == MTLPixelFormatBGRA8Unorm_sRGB);
//
//         switch (pixelFormat)
//         {
//             case MTLPixelFormatRGBA8Unorm_sRGB:
//             case MTLPixelFormatRGBA8Unorm:
//             case MTLPixelFormatRGBA16Float:
//                 return m_pipelineStateRGBA8;
//             default:
//                 return m_pipelineStateBGRA8;
//         }
//     }
//
// private:
//     id<MTLRenderPipelineState> m_pipelineStateRGBA8 = nil;
//     id<MTLRenderPipelineState> m_pipelineStateBGRA8 = nil;
// };
//
// #if defined(RIVE_IOS) || defined(RIVE_XROS) || defined(RIVE_APPLETVOS)
// static bool is_apple_silicon(id<MTLDevice> gpu)
// {
//     if (@available(iOS 13, tvOS 13, visionOS 1, *))
//     {
//         return [gpu supportsFamily:MTLGPUFamilyApple4];
//     }
//     return false;
// }
// #endif
//
// class BufferRingMetalImpl : public BufferRing
// {
// public:
//     static std::unique_ptr<BufferRingMetalImpl> Make(id<MTLDevice> gpu,
//                                                      size_t capacityInBytes)
//     {
//         return capacityInBytes != 0
//                    ? std::make_unique<BufferRingMetalImpl>(gpu, capacityInBytes)
//                    : nullptr;
//     }
//
//     BufferRingMetalImpl(id<MTLDevice> gpu, size_t capacityInBytes) :
//         BufferRing(capacityInBytes)
//     {
//         for (int i = 0; i < kBufferRingSize; ++i)
//         {
//             m_buffers[i] =
//                 [gpu newBufferWithLength:capacityInBytes
//                                  options:MTLResourceStorageModeShared];
//         }
//     }
//
//     id<MTLBuffer> submittedBuffer() const
//     {
//         return m_buffers[submittedBufferIdx()];
//     }
//
// protected:
//     void* onMapBuffer(int bufferIdx, size_t mapSizeInBytes) override
//     {
//         return m_buffers[bufferIdx].contents;
//     }
//
//     void onUnmapAndSubmitBuffer(int bufferIdx, size_t mapSizeInBytes) override
//     {}
//
// private:
//     id<MTLBuffer> m_buffers[kBufferRingSize];
// };
//
// std::unique_ptr<RenderContext> RenderContextMetalImpl::MakeContext(
//     id<MTLDevice> gpu, const ContextOptions& contextOptions)
// {
//     auto renderContextImpl = std::unique_ptr<RenderContextMetalImpl>(
//         new RenderContextMetalImpl(gpu, contextOptions));
//     return std::make_unique<RenderContext>(std::move(renderContextImpl));
// }
//
// RenderContextMetalImpl::RenderContextMetalImpl(
//     id<MTLDevice> gpu, const ContextOptions& contextOptions) :
//     m_contextOptions(contextOptions), m_gpu(gpu)
// {
//     // It appears, so far, that we don't need to use flat interpolation for path
//     // IDs on any Apple device, and it's faster not to.
//     m_platformFeatures.avoidFlatVaryings = true;
//     m_platformFeatures.clipSpaceBottomUp = true;
//     m_platformFeatures.framebufferBottomUp = false;
//     if ([m_gpu supportsFamily:MTLGPUFamilyApple2] ||
//         [m_gpu supportsFamily:MTLGPUFamilyMac2])
//     {
//         m_platformFeatures.maxTextureSize = 16384;
//     }
//     else
//     {
//         m_platformFeatures.maxTextureSize = 8192;
//     }
// #if defined(RIVE_IOS) || defined(RIVE_XROS) || defined(RIVE_APPLETVOS)
//     m_platformFeatures.supportsRasterOrderingMode = true;
//     m_platformFeatures.supportsAtomicMode = false;
//     if (!is_apple_silicon(m_gpu))
//     {
//         // The PowerVR GPU, at least on A10, has fp16 precision issues. We can't
//         // use the the bottom 3 bits of the path and clip IDs in order for our
//         // equality testing to work.
//         m_platformFeatures.pathIDGranularity = 8;
//     }
// #elif defined(RIVE_IOS_SIMULATOR) || defined(RIVE_XROS_SIMULATOR) ||           \
//     defined(RIVE_APPLETVOS_SIMULATOR)
//     // The simulator does not support framebuffer reads. Fall back on atomic
//     // mode.
//     m_platformFeatures.supportsRasterOrderingMode = false;
//     m_platformFeatures.supportsAtomicMode = true;
// #else
//     m_platformFeatures.supportsRasterOrderingMode =
//         [m_gpu supportsFamily:MTLGPUFamilyApple1] &&
//         !contextOptions.disableFramebufferReads;
//     m_platformFeatures.supportsAtomicMode = true;
// #endif
//     m_platformFeatures.atomicPLSInitNeedsDraw = true;
//
//     m_platformFeatures.supportsClipScissor = true;
//
//     // Texture compression support varies by Apple platform family.
// #if defined(RIVE_IOS) || defined(RIVE_XROS) || defined(RIVE_APPLETVOS) ||      \
//     defined(RIVE_IOS_SIMULATOR) || defined(RIVE_XROS_SIMULATOR) ||             \
//     defined(RIVE_APPLETVOS_SIMULATOR)
//     // iOS/tvOS/visionOS: ETC2 and ASTC are always supported.
//     m_platformFeatures.supportsTextureCompressionETC2 = true;
//     m_platformFeatures.supportsTextureCompressionASTC = true;
// #else
//     // macOS: BC is always supported; ASTC only on Apple Silicon.
//     m_platformFeatures.supportsTextureCompressionBC = true;
//     m_platformFeatures.supportsTextureCompressionASTC =
//         [m_gpu supportsFamily:MTLGPUFamilyApple1];
// #endif
//
// #if defined(RIVE_IOS) || defined(RIVE_XROS) || defined(RIVE_XROS_SIMULATOR) || \
//     defined(RIVE_APPLETVOS) || defined(RIVE_APPLETVOS_SIMULATOR)
//     // Atomic barriers are never used on iOS, but if we ever did need them, we
//     // would use rasterOrderGroups.
//     m_metalFeatures.atomicBarrierType = AtomicBarrierType::rasterOrderGroup;
// #elif defined(RIVE_IOS_SIMULATOR)
//     const NXArchInfo* hostArchitecture = NXGetLocalArchInfo();
//     if (strncmp(hostArchitecture->name, "arm64", 5) == 0)
//     {
//         // The simulator doesn't advertise support for raster order groups, but
//         // they appear to work anyway on an Apple-Silicon-hosted simulator. Use
//         // rasterOrderGroup in this case because it's much faster than
//         // renderPassBreak. (On Intel/AMD this doesn't matter anyway because
//         // renderPassBreaks are cheap and actually faster than
//         // rasterOrderGroups.)
//         m_metalFeatures.atomicBarrierType = AtomicBarrierType::rasterOrderGroup;
//     }
//     else
//     {
//         m_metalFeatures.atomicBarrierType = AtomicBarrierType::renderPassBreak;
//     }
// #else
//     // Use real memory barriers for atomic mode if they're availabile.
//     // "GPU devices in Apple3 through Apple9 families don’t support memory
//     // barriers that include the MTLRenderStages.fragment or .tile stages in the
//     // after argument..."
//     if (([m_gpu supportsFamily:MTLGPUFamilyCommon2] ||
//          [m_gpu supportsFamily:MTLGPUFamilyMac2]) &&
//         ![m_gpu supportsFamily:MTLGPUFamilyApple3])
//     {
//         m_metalFeatures.atomicBarrierType = AtomicBarrierType::memoryBarrier;
//     }
//     else if (m_gpu.rasterOrderGroupsSupported)
//     {
//         m_metalFeatures.atomicBarrierType = AtomicBarrierType::rasterOrderGroup;
//     }
//     else
//     {
//         m_metalFeatures.atomicBarrierType = AtomicBarrierType::renderPassBreak;
//     }
// #endif
//
//     for (int i = 0; i < rive::ImageSampler::MAX_SAMPLER_PERMUTATIONS; ++i)
//     {
//         auto wrapX = ImageSampler::GetWrapXOptionFromKey(i);
//         auto wrapY = ImageSampler::GetWrapYOptionFromKey(i);
//         auto filter = ImageSampler::GetFilterOptionFromKey(i);
//
//         MTLSamplerDescriptor* samplerDescriptor = [MTLSamplerDescriptor new];
//         samplerDescriptor.minFilter = min_mag_filter_for_image_filter(filter);
//         samplerDescriptor.magFilter = min_mag_filter_for_image_filter(filter);
//         samplerDescriptor.mipFilter = mip_filter_for_image_filter(filter);
//         samplerDescriptor.sAddressMode = address_mode_for_image_wrap(wrapX);
//         samplerDescriptor.tAddressMode = address_mode_for_image_wrap(wrapY);
//
//         m_imageSamplers[i] =
//             [gpu newSamplerStateWithDescriptor:samplerDescriptor];
//     }
//
//     m_backgroundShaderCompiler =
//         std::make_unique<BackgroundShaderCompiler>(m_gpu, m_metalFeatures);
//
//     // Load the precompiled shaders.
//     dispatch_data_t metallibData = dispatch_data_create(
// #if defined(RIVE_IOS)
//         rive_pls_ios_metallib,
//         rive_pls_ios_metallib_len,
// #elif defined(RIVE_IOS_SIMULATOR)
//         rive_pls_ios_simulator_metallib,
//         rive_pls_ios_simulator_metallib_len,
// #elif defined(RIVE_XROS)
//         rive_renderer_xros_metallib,
//         rive_renderer_xros_metallib_len,
// #elif defined(RIVE_XROS_SIMULATOR)
//         rive_renderer_xros_simulator_metallib,
//         rive_renderer_xros_simulator_metallib_len,
// #elif defined(RIVE_APPLETVOS)
//         rive_renderer_appletvos_metallib,
//         rive_renderer_appletvos_metallib_len,
// #elif defined(RIVE_APPLETVOS_SIMULATOR)
//         rive_renderer_appletvsimulator_metallib,
//         rive_renderer_appletvsimulator_metallib_len,
// #else
//         rive_pls_macosx_metallib,
//         rive_pls_macosx_metallib_len,
// #endif
//         nil,
//         nil);
//     NSError* err = nil;
//     m_plsPrecompiledLibrary = [m_gpu newLibraryWithData:metallibData
//                                                   error:&err];
//     if (err != nil || m_plsPrecompiledLibrary == nil)
//     {
//         NSLog(@"RIVE: Failed to load pls metallib error: %@",
//               err != nil ? err.localizedDescription : @"<nil>");
//         return;
//     }
//
//     m_colorRampPipeline =
//         std::make_unique<ColorRampPipeline>(m_gpu, m_plsPrecompiledLibrary);
//
//     MTLTextureDescriptor* desc = [[MTLTextureDescriptor alloc] init];
//     desc.pixelFormat = MTLPixelFormatR16Float;
//     desc.textureType = MTLTextureType1DArray;
//     desc.width = gpu::GAUSSIAN_TABLE_SIZE;
//     desc.mipmapLevelCount = 1;
//     desc.arrayLength = GAUSSIAN_INTEGRAL_TEXTURE_1D_ARRAY_LENGTH;
//     desc.usage = MTLTextureUsageShaderRead;
//     m_gaussianIntegralTexture = [m_gpu newTextureWithDescriptor:desc];
//     [m_gaussianIntegralTexture
//         replaceRegion:MTLRegionMake2D(0, 0, gpu::GAUSSIAN_TABLE_SIZE, 1)
//           mipmapLevel:0
//                 slice:FEATHER_FUNCTION_ARRAY_INDEX
//             withBytes:gpu::g_gaussianIntegralTableF16
//           bytesPerRow:sizeof(gpu::g_gaussianIntegralTableF16)
//         bytesPerImage:sizeof(gpu::g_gaussianIntegralTableF16)];
//     [m_gaussianIntegralTexture
//         replaceRegion:MTLRegionMake2D(0, 0, gpu::GAUSSIAN_TABLE_SIZE, 1)
//           mipmapLevel:0
//                 slice:FEATHER_INVERSE_FUNCTION_ARRAY_INDEX
//             withBytes:gpu::g_inverseGaussianIntegralTableF16
//           bytesPerRow:sizeof(gpu::g_gaussianIntegralTableF16)
//         bytesPerImage:sizeof(gpu::g_gaussianIntegralTableF16)];
//
//     m_tessPipeline =
//         std::make_unique<TessellatePipeline>(m_gpu, m_plsPrecompiledLibrary);
//     m_tessSpanIndexBuffer =
//         [m_gpu newBufferWithBytes:gpu::kTessSpanIndices
//                            length:sizeof(gpu::kTessSpanIndices)
//                           options:MTLResourceStorageModeShared];
//
//     // The precompiled static library has a fully-featured shader for each
//     // drawType in "rasterOrdering" mode. We load these at initialization and
//     // use them while waiting for the background compiler to generate more
//     // specialized, higher performance shaders.
//     if (m_platformFeatures.supportsRasterOrderingMode)
//     {
//         for (auto drawType : {DrawType::midpointFanPatches,
//                               DrawType::interiorTriangulation,
//                               DrawType::featherAtlasBlit,
//                               DrawType::imageMesh})
//         {
//             for (auto shaderMiscFlags : {gpu::ShaderMiscFlags::none,
//                                          gpu::ShaderMiscFlags::clockwiseFill})
//             {
//                 if (drawType == gpu::DrawType::featherAtlasBlit &&
//                     shaderMiscFlags != gpu::ShaderMiscFlags::none)
//                 {
//                     continue;
//                 }
//                 gpu::ShaderFeatures allShaderFeatures =
//                     gpu::ShaderFeaturesMaskFor(
//                         drawType, gpu::InterlockMode::rasterOrdering);
//                 uint32_t pipelineKey =
//                     ShaderUniqueKey(drawType,
//                                     allShaderFeatures,
//                                     gpu::InterlockMode::rasterOrdering,
//                                     shaderMiscFlags);
//                 m_drawPipelines[pipelineKey] = std::make_unique<DrawPipeline>(
//                     m_gpu,
//                     m_plsPrecompiledLibrary,
//                     DrawPipeline::GetPrecompiledFunctionName(
//                         drawType,
//                         allShaderFeatures & gpu::kVertexShaderFeaturesMask,
//                         gpu::ShaderMiscFlags::none,
//                         m_plsPrecompiledLibrary,
//                         GLSL_drawVertexMain),
//                     DrawPipeline::GetPrecompiledFunctionName(
//                         drawType,
//                         allShaderFeatures,
//                         shaderMiscFlags,
//                         m_plsPrecompiledLibrary,
//                         GLSL_drawFragmentMain),
//                     drawType,
//                     gpu::InterlockMode::rasterOrdering,
//                     allShaderFeatures,
//                     shaderMiscFlags
// #ifdef WITH_RIVE_TOOLS
//                     ,
//                     SynthesizedFailureType::none
// #endif
//                 );
//             }
//         }
//     }
//
//     // Create vertex and index buffers for the different PLS patches.
//     m_pathPatchVertexBuffer =
//         [m_gpu newBufferWithLength:kPatchVertexBufferCount * sizeof(PatchVertex)
//                            options:MTLResourceStorageModeShared];
//     m_pathPatchIndexBuffer =
//         [m_gpu newBufferWithLength:kPatchIndexBufferCount * sizeof(uint16_t)
//                            options:MTLResourceStorageModeShared];
//     GeneratePatchBufferData(
//         reinterpret_cast<PatchVertex*>(m_pathPatchVertexBuffer.contents),
//         reinterpret_cast<uint16_t*>(m_pathPatchIndexBuffer.contents));
//
//     // Set up the imageRect rendering buffers. (gpu::InterlockMode::atomics
//     // only.)
//     m_imageRectVertexBuffer =
//         [m_gpu newBufferWithBytes:gpu::kImageRectVertices
//                            length:sizeof(gpu::kImageRectVertices)
//                           options:MTLResourceStorageModeShared];
//     m_imageRectIndexBuffer =
//         [m_gpu newBufferWithBytes:gpu::kImageRectIndices
//                            length:sizeof(gpu::kImageRectIndices)
//                           options:MTLResourceStorageModeShared];
// }
//
// RenderContextMetalImpl::~RenderContextMetalImpl() {}
//
// // If the GPU supports framebuffer reads (called "programmable blending" in the
// // feature tables), PLS planes besides the main framebuffer can exist in
// // ephemeral "memoryless" storage. This means their contents are never actually
// // written to main memory, and they only exist in fast tiled memory.
// static id<MTLTexture> make_pls_memoryless_texture(id<MTLDevice> gpu,
//                                                   MTLPixelFormat pixelFormat,
//                                                   uint32_t width,
//                                                   uint32_t height)
// {
//     MTLTextureDescriptor* desc = [[MTLTextureDescriptor alloc] init];
//     desc.pixelFormat = pixelFormat;
//     desc.width = width;
//     desc.height = height;
//     desc.usage = MTLTextureUsageRenderTarget;
//     desc.textureType = MTLTextureType2D;
//     desc.mipmapLevelCount = 1;
//     desc.storageMode = MTLStorageModeMemoryless;
//     return [gpu newTextureWithDescriptor:desc];
// }
//
// RenderTargetMetal::RenderTargetMetal(id<MTLDevice> gpu,
//                                      MTLPixelFormat pixelFormat,
//                                      uint32_t width,
//                                      uint32_t height,
//                                      const PlatformFeatures& platformFeatures) :
//     RenderTarget(width, height), m_gpu(gpu), m_pixelFormat(pixelFormat)
// {
//     m_targetTexture = nil; // Will be configured later by setTargetTexture().
//     if (platformFeatures.supportsRasterOrderingMode)
//     {
//         m_coverageMemorylessTexture = make_pls_memoryless_texture(
//             gpu, MTLPixelFormatR32Uint, width, height);
//         m_clipMemorylessTexture = make_pls_memoryless_texture(
//             gpu, MTLPixelFormatR32Uint, width, height);
//         m_scratchColorMemorylessTexture =
//             make_pls_memoryless_texture(gpu, m_pixelFormat, width, height);
//     }
// }
//
// void RenderTargetMetal::setTargetTexture(id<MTLTexture> texture)
// {
//     assert(!texture || compatibleWith(texture));
//     m_targetTexture = texture;
// }
//
// rcp<RenderTargetMetal> RenderContextMetalImpl::makeRenderTarget(
//     MTLPixelFormat pixelFormat, uint32_t width, uint32_t height)
// {
//     return rcp(new RenderTargetMetal(
//         m_gpu, pixelFormat, width, height, m_platformFeatures));
// }
//
// class RenderBufferMetalImpl
//     : public LITE_RTTI_OVERRIDE(RiveRenderBuffer, RenderBufferMetalImpl)
// {
// public:
//     RenderBufferMetalImpl(RenderBufferType renderBufferType,
//                           RenderBufferFlags renderBufferFlags,
//                           size_t sizeInBytes,
//                           id<MTLDevice> gpu) :
//         lite_rtti_override(renderBufferType, renderBufferFlags, sizeInBytes),
//         m_gpu(gpu)
//     {
//         int bufferCount =
//             enums::is_flag_set(flags(),
//                                RenderBufferFlags::mappedOnceAtInitialization)
//                 ? 1
//                 : gpu::kBufferRingSize;
//         for (int i = 0; i < bufferCount; ++i)
//         {
//             m_buffers[i] =
//                 [gpu newBufferWithLength:sizeInBytes
//                                  options:MTLResourceStorageModeShared];
//         }
//     }
//
//     id<MTLBuffer> submittedBuffer() { return m_buffers[frontBufferIdx()]; }
//
// protected:
//     void* onMap() override
//     {
//         assert(m_buffers[backBufferIdx()] != nil);
//         return m_buffers[backBufferIdx()].contents;
//     }
//
//     void onUnmap() override {}
//
// private:
//     id<MTLDevice> m_gpu;
//     id<MTLBuffer> m_buffers[gpu::kBufferRingSize];
//     int m_submittedBufferIdx = -1;
// };
//
// rcp<RenderBuffer> RenderContextMetalImpl::makeRenderBuffer(
//     RenderBufferType type, RenderBufferFlags flags, size_t sizeInBytes)
// {
//     return make_rcp<RenderBufferMetalImpl>(type, flags, sizeInBytes, m_gpu);
// }
//
// class TextureMetalImpl : public Texture
// {
// public:
//     TextureMetalImpl(id<MTLDevice> gpu,
//                      uint32_t width,
//                      uint32_t height,
//                      uint32_t mipLevelCount,
//                      const uint8_t imageData[],
//                      MTLPixelFormat pixelFormat = MTLPixelFormatRGBA8Unorm,
//                      uint8_t blockWidth = 1,
//                      uint8_t blockHeight = 1,
//                      uint32_t bytesPerBlock = 4,
//                      bool generateRemainingMips = false) :
//         Texture(width, height),
//         m_mipsDirty(generateRemainingMips && mipLevelCount > 1)
//     {
//         // Create the texture.
//         MTLTextureDescriptor* desc = [[MTLTextureDescriptor alloc] init];
//         desc.pixelFormat = pixelFormat;
//         desc.width = width;
//         desc.height = height;
//         desc.mipmapLevelCount = mipLevelCount;
//         desc.usage = MTLTextureUsageShaderRead;
//         desc.textureType = MTLTextureType2D;
//         m_texture = [gpu newTextureWithDescriptor:desc];
//
//         // Upload mip 0 only when the caller asks for auto-mipgen
//         // (generateRemainingMips=true). Otherwise upload every level the
//         // texture was created with from the caller-supplied tight blob.
//         const uint32_t levelsToUpload =
//             generateRemainingMips ? 1u : mipLevelCount;
//         const uint8_t* src = imageData;
//         for (uint32_t i = 0; i < levelsToUpload; ++i)
//         {
//             const uint32_t logW = std::max<uint32_t>(1u, width >> i);
//             const uint32_t logH = std::max<uint32_t>(1u, height >> i);
//             const uint32_t blocksX = (logW + blockWidth - 1) / blockWidth;
//             const uint32_t blocksY = (logH + blockHeight - 1) / blockHeight;
//             const NSUInteger bytesPerRow =
//                 static_cast<NSUInteger>(blocksX) * bytesPerBlock;
//             const size_t levelBytes =
//                 static_cast<size_t>(bytesPerRow) * blocksY;
//             MTLRegion region = MTLRegionMake2D(0, 0, logW, logH);
//             [m_texture replaceRegion:region
//                          mipmapLevel:i
//                            withBytes:src
//                          bytesPerRow:bytesPerRow];
//             src += levelBytes;
//         }
//     }
//
//     void ensureMipmaps(id<MTLCommandBuffer> commandBuffer) const
//     {
//         if (m_mipsDirty)
//         {
//             // Generate mipmaps.
//             id<MTLBlitCommandEncoder> mipEncoder =
//                 [commandBuffer blitCommandEncoder];
//             [mipEncoder generateMipmapsForTexture:m_texture];
//             [mipEncoder endEncoding];
//             m_mipsDirty = false;
//         }
//     }
//
//     // Adopt a pre-created MTLTexture (for RenderCanvas).
//     TextureMetalImpl(id<MTLTexture> texture, uint32_t width, uint32_t height) :
//         Texture(width, height), m_texture(texture), m_mipsDirty(false)
//     {}
//
//     id<MTLTexture> texture() const { return m_texture; }
//     void* nativeHandle() const override { return (__bridge void*)m_texture; }
//
// private:
//     id<MTLTexture> m_texture;
//     mutable bool m_mipsDirty = true;
// };
//
// rcp<Texture> RenderContextMetalImpl::makeImageTexture(
//     uint32_t width,
//     uint32_t height,
//     uint32_t mipLevelCount,
//     GPUTextureFormat format,
//     const uint8_t imageData[],
//     uint8_t blockWidth,
//     uint8_t blockHeight,
//     [[maybe_unused]] bool srgb,
//     bool generateRemainingMips)
// {
//     MTLPixelFormat pixelFormat = MTLPixelFormatRGBA8Unorm;
//     uint32_t bytesPerBlock = 4;
//     bool isCompressed = false;
//
//     switch (format)
//     {
//         case GPUTextureFormat::rgba32:
//             assert(blockWidth == 1 && blockHeight == 1);
//             break;
// #if !TARGET_OS_IPHONE
//         case GPUTextureFormat::bc7:
//             pixelFormat = MTLPixelFormatBC7_RGBAUnorm;
//             bytesPerBlock = 16;
//             isCompressed = true;
//             break;
// #endif
//         case GPUTextureFormat::astc:
//         {
//             // MTLPixelFormat ASTC LDR enums are sequential in Vulkan/GL
//             // footprint order, starting at MTLPixelFormatASTC_4x4_LDR.
//             const int idx = rive::astcFootprintIndex(blockWidth, blockHeight);
//             if (idx < 0)
//             {
//                 assert(!"unsupported ASTC block footprint");
//                 return nullptr;
//             }
//             pixelFormat =
//                 static_cast<MTLPixelFormat>(MTLPixelFormatASTC_4x4_LDR + idx);
//             bytesPerBlock = 16;
//             isCompressed = true;
//             break;
//         }
//         case GPUTextureFormat::etc2:
//             // ETC2 RGBA8: 8 bytes EAC alpha + 8 bytes ETC2 RGB = 16/block.
//             pixelFormat = MTLPixelFormatEAC_RGBA8;
//             bytesPerBlock = 16;
//             isCompressed = true;
//             break;
//         default:
//             assert(!"unsupported format");
//             return nullptr;
//     }
//     assert(!(generateRemainingMips && isCompressed) &&
//            "generateMipmapsForTexture is undefined on compressed formats");
//
//     return make_rcp<TextureMetalImpl>(m_gpu,
//                                       width,
//                                       height,
//                                       mipLevelCount,
//                                       imageData,
//                                       pixelFormat,
//                                       blockWidth,
//                                       blockHeight,
//                                       bytesPerBlock,
//                                       generateRemainingMips);
// }
//
// rcp<Texture> RenderContextMetalImpl::adoptImageTexture(id<MTLTexture> texture,
//                                                        uint32_t width,
//                                                        uint32_t height)
// {
//     if (texture == nil || width == 0 || height == 0)
//     {
//         return nullptr;
//     }
//     return make_rcp<TextureMetalImpl>(texture, width, height);
// }
//
// #ifdef RIVE_CANVAS
// rcp<RenderCanvas> RenderContextMetalImpl::makeRenderCanvas(uint32_t width,
//                                                            uint32_t height)
// {
//     // Create an MTLTexture usable as both a render target and a shader-read
//     // image for compositing into Rive draws.
//     MTLTextureDescriptor* desc = [[MTLTextureDescriptor alloc] init];
//     desc.pixelFormat = MTLPixelFormatRGBA8Unorm;
//     desc.width = width;
//     desc.height = height;
//     desc.usage = MTLTextureUsageRenderTarget | MTLTextureUsageShaderRead;
//     desc.textureType = MTLTextureType2D;
//     desc.mipmapLevelCount = 1;
//     desc.storageMode = MTLStorageModePrivate;
//     id<MTLTexture> mtlTexture = [m_gpu newTextureWithDescriptor:desc];
//
//     // Wrap as a RenderTarget for rendering into.
//     auto renderTarget =
//         makeRenderTarget(MTLPixelFormatRGBA8Unorm, width, height);
//     renderTarget->setTargetTexture(mtlTexture);
//
//     // Wrap as a RiveRenderImage for compositing. The TextureMetalImpl adopt
//     // constructor takes a pre-created MTLTexture without uploading data.
//     auto texture = make_rcp<TextureMetalImpl>(mtlTexture, width, height);
//     auto renderImage = make_rcp<RiveRenderImage>(std::move(texture));
//
//     return make_rcp<RenderCanvas>(std::move(renderImage),
//                                   std::move(renderTarget));
// }
//
// std::unique_ptr<rive::ore::Context> RenderContextMetalImpl::makeOreContext()
// {
//     assert(m_commandQueue);
//     return rive::ore::ContextMetal::Make(m_gpu, m_commandQueue);
// }
// #endif
//
// std::unique_ptr<BufferRing> RenderContextMetalImpl::makeUniformBufferRing(
//     size_t capacityInBytes)
// {
//     return BufferRingMetalImpl::Make(m_gpu, capacityInBytes);
// }
//
// std::unique_ptr<BufferRing> RenderContextMetalImpl::makeStorageBufferRing(
//     size_t capacityInBytes, gpu::StorageBufferStructure)
// {
//     return BufferRingMetalImpl::Make(m_gpu, capacityInBytes);
// }
//
// std::unique_ptr<BufferRing> RenderContextMetalImpl::makeVertexBufferRing(
//     size_t capacityInBytes)
// {
//     return BufferRingMetalImpl::Make(m_gpu, capacityInBytes);
// }
//
// void RenderContextMetalImpl::resizeGradientTexture(uint32_t width,
//                                                    uint32_t height)
// {
//     if (width == 0 || height == 0)
//     {
//         m_gradientTexture = nil;
//         return;
//     }
//     MTLTextureDescriptor* desc = [[MTLTextureDescriptor alloc] init];
//     desc.pixelFormat = MTLPixelFormatRGBA8Unorm;
//     desc.width = width;
//     desc.height = height;
//     desc.usage = MTLTextureUsageRenderTarget | MTLTextureUsageShaderRead;
//     desc.textureType = MTLTextureType2D;
//     desc.mipmapLevelCount = 1;
//     desc.storageMode = MTLStorageModePrivate;
//     m_gradientTexture = [m_gpu newTextureWithDescriptor:desc];
// }
//
// void RenderContextMetalImpl::resizeTessellationTexture(uint32_t width,
//                                                        uint32_t height)
// {
//     if (width == 0 || height == 0)
//     {
//         m_tessVertexTexture = nil;
//         return;
//     }
//     MTLTextureDescriptor* desc = [[MTLTextureDescriptor alloc] init];
//     desc.pixelFormat = MTLPixelFormatRGBA32Uint;
//     desc.width = width;
//     desc.height = height;
//     desc.usage = MTLTextureUsageRenderTarget | MTLTextureUsageShaderRead;
//     desc.textureType = MTLTextureType2D;
//     desc.mipmapLevelCount = 1;
//     desc.storageMode = MTLStorageModePrivate;
//     m_tessVertexTexture = [m_gpu newTextureWithDescriptor:desc];
// }
//
// void RenderContextMetalImpl::resizeFeatherAtlasTexture(uint32_t width,
//                                                        uint32_t height)
// {
//     if (width == 0 || height == 0)
//     {
//         m_featherAtlasTexture = nil;
//         return;
//     }
//
//     MTLTextureDescriptor* desc = [[MTLTextureDescriptor alloc] init];
//     desc.pixelFormat = MTLPixelFormatR16Float;
//     desc.width = width;
//     desc.height = height;
//     desc.usage = MTLTextureUsageRenderTarget | MTLTextureUsageShaderRead;
//     desc.textureType = MTLTextureType2D;
//     desc.mipmapLevelCount = 1;
//     desc.storageMode = MTLStorageModePrivate;
//     m_featherAtlasTexture = [m_gpu newTextureWithDescriptor:desc];
//
//     // Don't build atlas pipelines until we get an indication that they will be
//     // used.
//     assert((m_featherAtlasFillPipeline == nil) ==
//            (m_featherAtlasStrokePipeline == nil));
//     if (m_featherAtlasFillPipeline == nil)
//     {
//         m_featherAtlasFillPipeline =
//             std::make_unique<FeatherAtlasPipeline>(m_gpu,
//                                                    m_plsPrecompiledLibrary,
//                                                    @GLSL_atlasFillFragmentMain,
//                                                    MTLBlendOperationAdd);
//         m_featherAtlasStrokePipeline = std::make_unique<FeatherAtlasPipeline>(
//             m_gpu,
//             m_plsPrecompiledLibrary,
//             @GLSL_atlasStrokeFragmentMain,
//             MTLBlendOperationMax);
//     }
// }
//
// const RenderContextMetalImpl::DrawPipeline* RenderContextMetalImpl::
//     findCompatibleDrawPipeline(gpu::DrawType drawType,
//                                gpu::ShaderFeatures shaderFeatures,
//                                const gpu::FlushDescriptor& desc,
//                                gpu::ShaderMiscFlags shaderMiscFlags)
// {
//     // Find a fully-featured superset of features whose pipeline we can fall
//     // back on while waiting for it to compile.
//     ShaderFeatures fullyFeaturedPipelineFeatures =
//         gpu::UbershaderFeaturesMaskFor(shaderFeatures,
//                                        drawType,
//                                        desc.interlockMode,
//                                        shaderMiscFlags,
//                                        m_platformFeatures);
//
//     if (m_contextOptions.shaderCompilationMode ==
//         ShaderCompilationMode::onlyUbershaders)
//     {
//         // Force the shader features to be the full set if that's what was
//         // requested.
//         shaderFeatures = fullyFeaturedPipelineFeatures;
//     }
//
// #ifdef WITH_RIVE_TOOLS
//     if (desc.synthesizedFailureType == SynthesizedFailureType::ubershaderLoad)
//     {
//         // Pretend that the requested shader is not ready yet and the ubershader
//         // compilation failed
//         return nil;
//     }
// #endif
//
//     uint32_t pipelineKey = gpu::ShaderUniqueKey(
//         drawType, shaderFeatures, desc.interlockMode, shaderMiscFlags);
//     auto pipelineIter = m_drawPipelines.find(pipelineKey);
//     if (pipelineIter == m_drawPipelines.end())
//     {
//         // The shader for this pipeline hasn't been scheduled for compiling yet.
//         // Schedule it to compile in the background.
//         m_backgroundShaderCompiler->pushJob({
//             .drawType = drawType,
//             .shaderFeatures = shaderFeatures,
//             .interlockMode = desc.interlockMode,
//             .shaderMiscFlags = shaderMiscFlags,
// #ifdef WITH_RIVE_TOOLS
//             .synthesizedFailureType = desc.synthesizedFailureType,
// #endif
//         });
//         pipelineIter = m_drawPipelines.insert({pipelineKey, nullptr}).first;
//     }
//
//     if (pipelineIter->second == nullptr)
//     {
//         // The shader for this pipeline hasn't finished compiling yet.
//         // Fully-featured "rasterOrdering" pipelines should have already been
//         // pre-loaded from the static library.
//         assert(shaderFeatures != fullyFeaturedPipelineFeatures ||
//                desc.interlockMode != gpu::InterlockMode::rasterOrdering);
//
//         // Poll to see if the shader is actually done compiling, but only wait
//         // if it's a fully-feature pipeline. Otherwise, we can fall back on the
//         // fully-featured pipeline while we wait for compilation.
//         BackgroundCompileJob job;
//         bool shouldWaitForBackgroundCompilation =
//             shaderFeatures == fullyFeaturedPipelineFeatures ||
//             m_contextOptions.shaderCompilationMode !=
//                 ShaderCompilationMode::allowAsynchronous;
//         while (m_backgroundShaderCompiler->popFinishedJob(
//             &job, shouldWaitForBackgroundCompilation))
//         {
//             uint32_t jobKey = gpu::ShaderUniqueKey(job.drawType,
//                                                    job.shaderFeatures,
//                                                    job.interlockMode,
//                                                    job.shaderMiscFlags);
//             m_drawPipelines[jobKey] =
//                 std::make_unique<DrawPipeline>(m_gpu,
//                                                job.compiledLibrary,
//                                                @GLSL_drawVertexMain,
//                                                @GLSL_drawFragmentMain,
//                                                job.drawType,
//                                                job.interlockMode,
//                                                job.shaderFeatures,
//                                                job.shaderMiscFlags
// #ifdef WITH_RIVE_TOOLS
//                                                ,
//                                                desc.synthesizedFailureType
// #endif
//                 );
//             if (jobKey == pipelineKey)
//             {
//                 // The shader we wanted was actually done compiling and pending
//                 // being built into a pipeline.
//                 break;
//             }
//         }
//     }
//
//     if ((pipelineIter->second == nullptr || !pipelineIter->second->valid()) &&
//         shaderFeatures != fullyFeaturedPipelineFeatures)
//     {
//         // The shader for this feature set hasn't finished compiling (or it
//         // failed to compile). Use the uber-shader pipeline that has all
//         // features enabled while we wait for it to finish.
//         return findCompatibleDrawPipeline(
//             drawType, fullyFeaturedPipelineFeatures, desc, shaderMiscFlags);
//     }
//
//     return pipelineIter->second.get();
// }
//
// void* RenderContextMetalImpl::makeCommandBuffer()
// {
//     if (m_commandQueue == nil)
//     {
//         return nullptr;
//     }
//     // __bridge_retained: transfers ARC ownership to the void* so it stays alive
//     // until commitCommandBuffer() releases it.
//     return (__bridge_retained void*)[m_commandQueue commandBuffer];
// }
//
// void RenderContextMetalImpl::commitCommandBuffer(void* commandBuffer)
// {
//     if (commandBuffer == nullptr)
//     {
//         return;
//     }
//     // __bridge_transfer: reclaims ARC ownership, balancing the
//     // __bridge_retained in makeCommandBuffer().
//     id<MTLCommandBuffer> mtlCmdBuffer =
//         (__bridge_transfer id<MTLCommandBuffer>)commandBuffer;
//     [mtlCmdBuffer commit];
// }
//
// // The buffer-ring lock acquired here is released asymmetrically: postFlush()
// // schedules a GPU completion handler that unlocks it once rendering finishes.
// // Clang's -Wthread-safety analysis cannot follow a lock handed off into that
// // block, so this method is exempt from the analysis.
// void RenderContextMetalImpl::prepareToFlush(uint64_t nextFrameNumber,
//                                             uint64_t safeFrameNumber)
//     RIVE_NO_THREAD_SAFETY_ANALYSIS
// {
//     // Wait until the GPU finishes rendering flush "N + 1 - kBufferRingSize".
//     // This ensures it is safe for the CPU to begin modifying the next buffers
//     // in our rings.
//     m_bufferRingIdx = (m_bufferRingIdx + 1) % kBufferRingSize;
//     m_bufferRingLocks[m_bufferRingIdx].lock();
// }
//
// static id<MTLBuffer> mtl_buffer(const BufferRing* bufferRing)
// {
//     assert(bufferRing != nullptr);
//     return static_cast<const BufferRingMetalImpl*>(bufferRing)
//         ->submittedBuffer();
// }
//
// static MTLViewport make_viewport(uint32_t x,
//                                  uint32_t y,
//                                  uint32_t width,
//                                  uint32_t height)
// {
//     return {
//         static_cast<double>(x),
//         static_cast<double>(y),
//         static_cast<double>(width),
//         static_cast<double>(height),
//         0,
//         1,
//     };
// }
//
// static MTLScissorRect make_scissor(const AABBu16& scissor)
// {
//     return {
//         static_cast<NSUInteger>(scissor.left),
//         static_cast<NSUInteger>(scissor.top),
//         static_cast<NSUInteger>(scissor.width()),
//         static_cast<NSUInteger>(scissor.height()),
//     };
// }
//
// id<MTLRenderCommandEncoder> RenderContextMetalImpl::makeRenderPassForDraws(
//     const gpu::FlushDescriptor& flushDesc,
//     MTLRenderPassDescriptor* passDesc,
//     id<MTLCommandBuffer> commandBuffer,
//     gpu::ShaderMiscFlags baselineShaderMiscFlags)
// {
//     auto* renderTarget =
//         static_cast<RenderTargetMetal*>(flushDesc.renderTarget);
//
//     id<MTLRenderCommandEncoder> encoder =
//         [commandBuffer renderCommandEncoderWithDescriptor:passDesc];
//
//     [encoder
//         setViewport:make_viewport(
//                         0, 0, renderTarget->width(), renderTarget->height())];
//     [encoder setVertexBuffer:mtl_buffer(flushUniformBufferRing())
//                       offset:flushDesc.flushUniformDataOffsetInBytes
//                      atIndex:METAL_BUFFER_IDX(FLUSH_UNIFORM_BUFFER_IDX)];
//     [encoder setFragmentBuffer:mtl_buffer(flushUniformBufferRing())
//                         offset:flushDesc.flushUniformDataOffsetInBytes
//                        atIndex:METAL_BUFFER_IDX(FLUSH_UNIFORM_BUFFER_IDX)];
//     [encoder setVertexTexture:m_tessVertexTexture
//                       atIndex:TESS_VERTEX_TEXTURE_IDX];
//     [encoder setVertexTexture:m_gaussianIntegralTexture
//                       atIndex:GAUSSIAN_INTEGRAL_TEXTURE_IDX];
//     [encoder setFragmentTexture:m_gradientTexture atIndex:GRAD_TEXTURE_IDX];
//     [encoder setFragmentTexture:m_gaussianIntegralTexture
//                         atIndex:GAUSSIAN_INTEGRAL_TEXTURE_IDX];
//     [encoder setFragmentTexture:m_featherAtlasTexture
//                         atIndex:FEATHER_ATLAS_TEXTURE_IDX];
//     if (flushDesc.pathCount > 0)
//     {
//         [encoder setVertexBuffer:mtl_buffer(pathBufferRing())
//                           offset:flushDesc.firstPath * sizeof(gpu::PathData)
//                          atIndex:METAL_BUFFER_IDX(PATH_BUFFER_IDX)];
//         if (flushDesc.interlockMode == gpu::InterlockMode::atomics)
//         {
//             [encoder
//                 setFragmentBuffer:mtl_buffer(paintBufferRing())
//                            offset:flushDesc.firstPaint * sizeof(gpu::PaintData)
//                           atIndex:METAL_BUFFER_IDX(PAINT_BUFFER_IDX)];
//             [encoder setFragmentBuffer:mtl_buffer(paintAuxBufferRing())
//                                 offset:flushDesc.firstPaintAux *
//                                        sizeof(gpu::PaintAuxData)
//                                atIndex:METAL_BUFFER_IDX(PAINT_AUX_BUFFER_IDX)];
//         }
//         else
//         {
//             [encoder
//                 setVertexBuffer:mtl_buffer(paintBufferRing())
//                          offset:flushDesc.firstPaint * sizeof(gpu::PaintData)
//                         atIndex:METAL_BUFFER_IDX(PAINT_BUFFER_IDX)];
//             [encoder setVertexBuffer:mtl_buffer(paintAuxBufferRing())
//                               offset:flushDesc.firstPaintAux *
//                                      sizeof(gpu::PaintAuxData)
//                              atIndex:METAL_BUFFER_IDX(PAINT_AUX_BUFFER_IDX)];
//         }
//     }
//     if (flushDesc.contourCount > 0)
//     {
//         [encoder
//             setVertexBuffer:mtl_buffer(contourBufferRing())
//                      offset:flushDesc.firstContour * sizeof(gpu::ContourData)
//                     atIndex:METAL_BUFFER_IDX(CONTOUR_BUFFER_IDX)];
//     }
//     if (flushDesc.interlockMode == gpu::InterlockMode::atomics)
//     {
//         // In atomic mode, the PLS planes are buffers that we need to bind
//         // separately. Since the PLS plane indices collide with other buffer
//         // bindings, offset the binding indices of these buffers by
//         // DEFAULT_BINDINGS_SET_SIZE.
//         if (!enums::is_flag_set(baselineShaderMiscFlags,
//                                 gpu::ShaderMiscFlags::fixedFunctionColorOutput))
//         {
//             [encoder
//                 setFragmentBuffer:renderTarget->colorAtomicBuffer()
//                            offset:0
//                           atIndex:METAL_BUFFER_IDX(COLOR_PLANE_IDX +
//                                                    DEFAULT_BINDINGS_SET_SIZE)];
//         }
//         [encoder setFragmentBuffer:renderTarget->clipAtomicBuffer()
//                             offset:0
//                            atIndex:METAL_BUFFER_IDX(CLIP_PLANE_IDX +
//                                                     DEFAULT_BINDINGS_SET_SIZE)];
//         [encoder setFragmentBuffer:renderTarget->coverageAtomicBuffer()
//                             offset:0
//                            atIndex:METAL_BUFFER_IDX(COVERAGE_PLANE_IDX +
//                                                     DEFAULT_BINDINGS_SET_SIZE)];
//     }
//     if (flushDesc.wireframe)
//     {
//         [encoder setTriangleFillMode:MTLTriangleFillModeLines];
//     }
//     return encoder;
// }
//
// void RenderContextMetalImpl::flush(const FlushDescriptor& desc)
// {
//     assert(desc.interlockMode != gpu::InterlockMode::clockwise);
//     assert(desc.interlockMode != gpu::InterlockMode::clockwiseAtomic);
//     assert(desc.interlockMode != gpu::InterlockMode::msaa); // TODO: msaa.
//
//     auto* renderTarget = static_cast<RenderTargetMetal*>(desc.renderTarget);
//     id<MTLCommandBuffer> commandBuffer =
//         (__bridge id<MTLCommandBuffer>)desc.externalCommandBuffer;
//
//     // Render the color ramps to the gradient texture.
//     if (desc.gradSpanCount > 0)
//     {
//         // We failed to load the precompiled library and therefore do not have
//         // the abililty to draw anything.
//         if (!m_colorRampPipeline)
//         {
//             return;
//         }
//         // We are removing the abort in the case this doesn't build. So give up
//         // drawing if we still don't have a pipeline here.
//         auto pipelineState = m_colorRampPipeline->pipelineState();
//         if (!pipelineState)
//         {
//             return;
//         }
//         MTLRenderPassDescriptor* gradPass =
//             [MTLRenderPassDescriptor renderPassDescriptor];
//         gradPass.renderTargetWidth = kGradTextureWidth;
//         gradPass.renderTargetHeight = desc.gradDataHeight;
//         gradPass.colorAttachments[0].loadAction = MTLLoadActionDontCare;
//         gradPass.colorAttachments[0].storeAction = MTLStoreActionStore;
//         gradPass.colorAttachments[0].texture = m_gradientTexture;
//
//         id<MTLRenderCommandEncoder> gradEncoder =
//             [commandBuffer renderCommandEncoderWithDescriptor:gradPass];
//         [gradEncoder
//             setViewport:make_viewport(0,
//                                       0,
//                                       kGradTextureWidth,
//                                       static_cast<float>(desc.gradDataHeight))];
//         [gradEncoder setRenderPipelineState:pipelineState];
//         [gradEncoder
//             setVertexBuffer:mtl_buffer(flushUniformBufferRing())
//                      offset:desc.flushUniformDataOffsetInBytes
//                     atIndex:METAL_BUFFER_IDX(FLUSH_UNIFORM_BUFFER_IDX)];
//         [gradEncoder
//             setVertexBuffer:mtl_buffer(gradSpanBufferRing())
//                      offset:desc.firstGradSpan * sizeof(gpu::GradientSpan)
//                     atIndex:0];
//         [gradEncoder setCullMode:MTLCullModeBack];
//         [gradEncoder drawPrimitives:MTLPrimitiveTypeTriangleStrip
//                         vertexStart:0
//                         vertexCount:gpu::GRAD_SPAN_TRI_STRIP_VERTEX_COUNT
//                       instanceCount:desc.gradSpanCount];
//         [gradEncoder endEncoding];
//     }
//
//     // Tessellate all curves into vertices in the tessellation texture.
//     if (desc.tessVertexSpanCount > 0)
//     {
//         // We failed to load the precompiled library and therefore do not have
//         // the abililty to draw anything.
//         if (!m_tessPipeline)
//         {
//             return;
//         }
//         // We are removing the abort in the case this doesn't build. So give up
//         // drawing if we still don't have a pipeline here.
//         auto pipelineState = m_tessPipeline->pipelineState();
//         if (!pipelineState)
//         {
//             return;
//         }
//
//         MTLRenderPassDescriptor* tessPass =
//             [MTLRenderPassDescriptor renderPassDescriptor];
//         tessPass.renderTargetWidth = kTessTextureWidth;
//         tessPass.renderTargetHeight = desc.tessDataHeight;
//         tessPass.colorAttachments[0].loadAction = MTLLoadActionDontCare;
//         tessPass.colorAttachments[0].storeAction = MTLStoreActionStore;
//         tessPass.colorAttachments[0].texture = m_tessVertexTexture;
//
//         id<MTLRenderCommandEncoder> tessEncoder =
//             [commandBuffer renderCommandEncoderWithDescriptor:tessPass];
//         [tessEncoder
//             setViewport:make_viewport(
//                             0, 0, kTessTextureWidth, desc.tessDataHeight)];
//         [tessEncoder setRenderPipelineState:pipelineState];
//         [tessEncoder setVertexTexture:m_gaussianIntegralTexture
//                               atIndex:GAUSSIAN_INTEGRAL_TEXTURE_IDX];
//         [tessEncoder
//             setVertexBuffer:mtl_buffer(flushUniformBufferRing())
//                      offset:desc.flushUniformDataOffsetInBytes
//                     atIndex:METAL_BUFFER_IDX(FLUSH_UNIFORM_BUFFER_IDX)];
//         [tessEncoder setVertexBuffer:mtl_buffer(tessSpanBufferRing())
//                               offset:desc.firstTessVertexSpan *
//                                      sizeof(gpu::TessVertexSpan)
//                              atIndex:0];
//         assert(desc.pathCount > 0);
//         [tessEncoder setVertexBuffer:mtl_buffer(pathBufferRing())
//                               offset:desc.firstPath * sizeof(gpu::PathData)
//                              atIndex:METAL_BUFFER_IDX(PATH_BUFFER_IDX)];
//         assert(desc.contourCount > 0);
//         [tessEncoder
//             setVertexBuffer:mtl_buffer(contourBufferRing())
//                      offset:desc.firstContour * sizeof(gpu::ContourData)
//                     atIndex:METAL_BUFFER_IDX(CONTOUR_BUFFER_IDX)];
//         [tessEncoder setCullMode:MTLCullModeBack];
//         [tessEncoder drawIndexedPrimitives:MTLPrimitiveTypeTriangle
//                                 indexCount:std::size(gpu::kTessSpanIndices)
//                                  indexType:MTLIndexTypeUInt16
//                                indexBuffer:m_tessSpanIndexBuffer
//                          indexBufferOffset:0
//                              instanceCount:desc.tessVertexSpanCount];
//         [tessEncoder endEncoding];
//     }
//
//     // Render the feather atlas if we have any offscreen feathers.
//     if ((desc.featherAtlasFillBatchCount | desc.featherAtlasStrokeBatchCount) !=
//         0)
//     {
//         // We failed to load the precompiled library and therefore do not have
//         // the abililty to draw anything.
//         if (!m_featherAtlasStrokePipeline || !m_featherAtlasFillPipeline)
//         {
//             return;
//         }
//         // We are removing the abort in the case this doesn't build. So give up
//         // drawing if we still don't have a pipeline here.
//         auto atlasFillPipelineState =
//             m_featherAtlasFillPipeline->pipelineState();
//         if (!atlasFillPipelineState)
//         {
//             return;
//         }
//
//         auto atlasStrokePipelineState =
//             m_featherAtlasStrokePipeline->pipelineState();
//         if (!atlasStrokePipelineState)
//         {
//             return;
//         }
//
//         MTLRenderPassDescriptor* atlasPass =
//             [MTLRenderPassDescriptor renderPassDescriptor];
//         atlasPass.renderTargetWidth = desc.featherAtlasContentWidth;
//         atlasPass.renderTargetHeight = desc.featherAtlasContentHeight;
//         atlasPass.colorAttachments[0].loadAction = MTLLoadActionClear;
//         atlasPass.colorAttachments[0].storeAction = MTLStoreActionStore;
//         atlasPass.colorAttachments[0].texture = m_featherAtlasTexture;
//         atlasPass.colorAttachments[0].clearColor =
//             MTLClearColorMake(0, 0, 0, 0);
//
//         id<MTLRenderCommandEncoder> atlasEncoder =
//             [commandBuffer renderCommandEncoderWithDescriptor:atlasPass];
//         [atlasEncoder
//             setViewport:make_viewport(0,
//                                       0,
//                                       desc.featherAtlasContentWidth,
//                                       desc.featherAtlasContentHeight)];
//         [atlasEncoder
//             setVertexBuffer:mtl_buffer(flushUniformBufferRing())
//                      offset:desc.flushUniformDataOffsetInBytes
//                     atIndex:METAL_BUFFER_IDX(FLUSH_UNIFORM_BUFFER_IDX)];
//         [atlasEncoder
//             setFragmentBuffer:mtl_buffer(flushUniformBufferRing())
//                        offset:desc.flushUniformDataOffsetInBytes
//                       atIndex:METAL_BUFFER_IDX(FLUSH_UNIFORM_BUFFER_IDX)];
//         [atlasEncoder setVertexTexture:m_tessVertexTexture
//                                atIndex:TESS_VERTEX_TEXTURE_IDX];
//         [atlasEncoder setVertexTexture:m_gaussianIntegralTexture
//                                atIndex:GAUSSIAN_INTEGRAL_TEXTURE_IDX];
//         [atlasEncoder setFragmentTexture:m_gradientTexture
//                                  atIndex:GRAD_TEXTURE_IDX];
//         [atlasEncoder setFragmentTexture:m_gaussianIntegralTexture
//                                  atIndex:GAUSSIAN_INTEGRAL_TEXTURE_IDX];
//         if (desc.pathCount > 0)
//         {
//             [atlasEncoder setVertexBuffer:mtl_buffer(pathBufferRing())
//                                    offset:desc.firstPath * sizeof(gpu::PathData)
//                                   atIndex:METAL_BUFFER_IDX(PATH_BUFFER_IDX)];
//             [atlasEncoder
//                 setVertexBuffer:mtl_buffer(paintBufferRing())
//                          offset:desc.firstPaint * sizeof(gpu::PaintData)
//                         atIndex:METAL_BUFFER_IDX(PAINT_BUFFER_IDX)];
//             [atlasEncoder
//                 setVertexBuffer:mtl_buffer(paintAuxBufferRing())
//                          offset:desc.firstPaintAux * sizeof(gpu::PaintAuxData)
//                         atIndex:METAL_BUFFER_IDX(PAINT_AUX_BUFFER_IDX)];
//         }
//         if (desc.contourCount > 0)
//         {
//             [atlasEncoder
//                 setVertexBuffer:mtl_buffer(contourBufferRing())
//                          offset:desc.firstContour * sizeof(gpu::ContourData)
//                         atIndex:METAL_BUFFER_IDX(CONTOUR_BUFFER_IDX)];
//         }
//         [atlasEncoder setVertexBuffer:m_pathPatchVertexBuffer
//                                offset:0
//                               atIndex:0];
//
//         if (desc.featherAtlasFillBatchCount != 0)
//         {
//             [atlasEncoder setCullMode:MTLCullModeNone];
//             [atlasEncoder setRenderPipelineState:atlasFillPipelineState];
//             for (size_t i = 0; i < desc.featherAtlasFillBatchCount; ++i)
//             {
//                 const gpu::AtlasDrawBatch& fillBatch =
//                     desc.featherAtlasFillBatches[i];
//                 [atlasEncoder setScissorRect:make_scissor(fillBatch.scissor)];
//                 [atlasEncoder
//                     setVertexBytes:&fillBatch.basePatch
//                             length:sizeof(uint32_t)
//                            atIndex:METAL_BUFFER_IDX(
//                                        PATH_BASE_INSTANCE_UNIFORM_BUFFER_IDX)];
//                 [atlasEncoder
//                     drawIndexedPrimitives:MTLPrimitiveTypeTriangle
//                                indexCount:
//                                    gpu::kMidpointFanCenterAAPatchIndexCount
//                                 indexType:MTLIndexTypeUInt16
//                               indexBuffer:m_pathPatchIndexBuffer
//                         indexBufferOffset:
//                             gpu::kMidpointFanCenterAAPatchBaseIndex *
//                             sizeof(uint16_t)
//                             instanceCount:fillBatch.patchCount];
//             }
//         }
//
//         if (desc.featherAtlasStrokeBatchCount != 0)
//         {
//             [atlasEncoder setCullMode:MTLCullModeBack];
//             [atlasEncoder setRenderPipelineState:atlasStrokePipelineState];
//             for (size_t i = 0; i < desc.featherAtlasStrokeBatchCount; ++i)
//             {
//                 const gpu::AtlasDrawBatch& strokeBatch =
//                     desc.featherAtlasStrokeBatches[i];
//                 [atlasEncoder setScissorRect:make_scissor(strokeBatch.scissor)];
//                 [atlasEncoder
//                     setVertexBytes:&strokeBatch.basePatch
//                             length:sizeof(uint32_t)
//                            atIndex:METAL_BUFFER_IDX(
//                                        PATH_BASE_INSTANCE_UNIFORM_BUFFER_IDX)];
//                 [atlasEncoder
//                     drawIndexedPrimitives:MTLPrimitiveTypeTriangle
//                                indexCount:gpu::kMidpointFanPatchBorderIndexCount
//                                 indexType:MTLIndexTypeUInt16
//                               indexBuffer:m_pathPatchIndexBuffer
//                         indexBufferOffset:gpu::kMidpointFanPatchBaseIndex *
//                                           sizeof(uint16_t)
//                             instanceCount:strokeBatch.patchCount];
//             }
//         }
//
//         [atlasEncoder endEncoding];
//     }
//
//     // Generate mipmaps if needed.
//     for (const DrawBatch& batch : *desc.drawList)
//     {
//         if (auto imageTextureMetal =
//                 static_cast<const TextureMetalImpl*>(batch.imageTexture))
//         {
//             imageTextureMetal->ensureMipmaps(commandBuffer);
//         }
//     }
//
//     // Set up a render pass to do the final rendering using (some form of) pixel
//     // local storage.
//     MTLRenderPassDescriptor* pass =
//         [MTLRenderPassDescriptor renderPassDescriptor];
//     pass.renderTargetWidth = desc.renderTargetUpdateBounds.right;
//     pass.renderTargetHeight = desc.renderTargetUpdateBounds.bottom;
//     pass.colorAttachments[COLOR_PLANE_IDX].texture =
//         renderTarget->targetTexture();
//     switch (desc.colorLoadAction)
//     {
//         case gpu::LoadAction::clear:
//         {
//             float cc[4];
//             UnpackColorToRGBA32FPremul(desc.colorClearValue, cc);
//             pass.colorAttachments[COLOR_PLANE_IDX].loadAction =
//                 MTLLoadActionClear;
//             pass.colorAttachments[COLOR_PLANE_IDX].clearColor =
//                 MTLClearColorMake(cc[0], cc[1], cc[2], cc[3]);
//             break;
//         }
//         case gpu::LoadAction::preserveRenderTarget:
//             pass.colorAttachments[COLOR_PLANE_IDX].loadAction =
//                 MTLLoadActionLoad;
//             break;
//         case gpu::LoadAction::dontCare:
//             pass.colorAttachments[COLOR_PLANE_IDX].loadAction =
//                 MTLLoadActionDontCare;
//             break;
//     }
//     pass.colorAttachments[COLOR_PLANE_IDX].storeAction = MTLStoreActionStore;
//
//     auto baselineShaderMiscFlags = gpu::ShaderMiscFlags::none;
//     if (desc.interlockMode == gpu::InterlockMode::rasterOrdering)
//     {
//         // In rasterOrdering mode, the PLS planes are accessed as color
//         // attachments.
//         pass.colorAttachments[CLIP_PLANE_IDX].texture =
//             renderTarget->m_clipMemorylessTexture;
//         pass.colorAttachments[CLIP_PLANE_IDX].loadAction = MTLLoadActionClear;
//         pass.colorAttachments[CLIP_PLANE_IDX].clearColor =
//             MTLClearColorMake(0, 0, 0, 0);
//         pass.colorAttachments[CLIP_PLANE_IDX].storeAction =
//             MTLStoreActionDontCare;
//
//         pass.colorAttachments[SCRATCH_COLOR_PLANE_IDX].texture =
//             renderTarget->m_scratchColorMemorylessTexture;
//         pass.colorAttachments[SCRATCH_COLOR_PLANE_IDX].loadAction =
//             MTLLoadActionDontCare;
//         pass.colorAttachments[SCRATCH_COLOR_PLANE_IDX].storeAction =
//             MTLStoreActionDontCare;
//
//         pass.colorAttachments[COVERAGE_PLANE_IDX].texture =
//             renderTarget->m_coverageMemorylessTexture;
//         pass.colorAttachments[COVERAGE_PLANE_IDX].loadAction =
//             MTLLoadActionClear;
//         pass.colorAttachments[COVERAGE_PLANE_IDX].clearColor =
//             MTLClearColorMake(desc.coverageClearValue, 0, 0, 0);
//         pass.colorAttachments[COVERAGE_PLANE_IDX].storeAction =
//             MTLStoreActionDontCare;
//     }
//     else if (desc.colorLoadAction == gpu::LoadAction::preserveRenderTarget &&
//              !desc.fixedFunctionColorOutput)
//     {
//         // Since we need to preserve the renderTarget during load, and since
//         // we're rendering to an offscreen color buffer, we have to literally
//         // copy the renderTarget into the color buffer.
//         assert(desc.interlockMode == gpu::InterlockMode::atomics);
//         id<MTLBlitCommandEncoder> copyEncoder =
//             [commandBuffer blitCommandEncoder];
//         auto updateOrigin = MTLOriginMake(desc.renderTargetUpdateBounds.left,
//                                           desc.renderTargetUpdateBounds.top,
//                                           0);
//         auto updateSize = MTLSizeMake(desc.renderTargetUpdateBounds.width(),
//                                       desc.renderTargetUpdateBounds.height(),
//                                       1);
//         [copyEncoder copyFromTexture:renderTarget->targetTexture()
//                          sourceSlice:0
//                          sourceLevel:0
//                         sourceOrigin:updateOrigin
//                           sourceSize:updateSize
//                             toBuffer:renderTarget->colorAtomicBuffer()
//                    destinationOffset:(updateOrigin.y * renderTarget->width() +
//                                       updateOrigin.x) *
//                                      sizeof(uint32_t)
//               destinationBytesPerRow:renderTarget->width() * sizeof(uint32_t)
//             destinationBytesPerImage:renderTarget->height() *
//                                      renderTarget->width() * sizeof(uint32_t)];
//         [copyEncoder endEncoding];
//     }
//
//     // Execute the DrawList.
//
//     // Start the current scissor rect inside out to guarantee that the first
//     // rectangle we get doesn't match it.
//     const auto fullRenderTargetScissorRect =
//         desc.renderTargetUpdateBounds.lossless_numeric_cast<uint16_t>();
//     auto currentScissorRect = AABBu16{0xffff, 0xffff, 0, 0};
//
//     id<MTLRenderCommandEncoder> encoder = makeRenderPassForDraws(
//         desc, pass, commandBuffer, baselineShaderMiscFlags);
//     for (const DrawBatch& batch : *desc.drawList)
//     {
//         // Setup the pipeline for this specific drawType and shaderFeatures.
//         gpu::ShaderFeatures shaderFeatures;
//         if (desc.interlockMode == gpu::InterlockMode::atomics)
//         {
//             // The combined shader features might have more flags set than are
//             // actually relevant for this draw type, so filter them out.
//             shaderFeatures =
//                 desc.combinedShaderFeatures &
//                 gpu::ShaderFeaturesMaskFor(batch.drawType, desc.interlockMode);
//         }
//         else
//         {
//             shaderFeatures = batch.shaderFeatures;
//         }
//
//         gpu::ShaderMiscFlags batchMiscFlags =
//             baselineShaderMiscFlags | batch.shaderMiscFlags;
//         if (!enums::is_flag_set(batchMiscFlags,
//                                 gpu::ShaderMiscFlags::fixedFunctionColorOutput))
//         {
//             if (batch.drawType == gpu::DrawType::renderPassResolve)
//             {
//                 // Atomic mode can always do a coalesced resolve when rendering
//                 // to an offscreen color buffer.
//                 assert(desc.interlockMode == gpu::InterlockMode::atomics);
//                 batchMiscFlags |=
//                     gpu::ShaderMiscFlags::coalescedResolveAndTransfer;
//             }
//             else if (batch.drawType == gpu::DrawType::renderPassInitialize)
//             {
//                 assert(desc.interlockMode == gpu::InterlockMode::atomics);
//                 if (desc.colorLoadAction == gpu::LoadAction::clear)
//                 {
//                     batchMiscFlags |= gpu::ShaderMiscFlags::storeColorClear;
//                 }
//                 else if (desc.colorLoadAction ==
//                              gpu::LoadAction::preserveRenderTarget &&
//                          renderTarget->pixelFormat() ==
//                              MTLPixelFormatBGRA8Unorm)
//                 {
//                     // We already copied the renderTarget to our color buffer,
//                     // but since the target is BGRA, we also need to swizzle it
//                     // to RGBA before it's ready for PLS.
//                     batchMiscFlags |=
//                         gpu::ShaderMiscFlags::swizzleColorBGRAToRGBA;
//                 }
//             }
//         }
//         const DrawPipeline* drawPipeline = findCompatibleDrawPipeline(
//             batch.drawType, shaderFeatures, desc, batchMiscFlags);
//         if (drawPipeline == nullptr || !drawPipeline->valid())
//         {
//             // The shader for this draw AND the uber-shader both failed to
//             // compile. This should virtually never happen, and can only happen
//             // on non-Apple Silicon, where we don't use precompiled shaders.
//             // Skip the draw.
//             continue;
//         }
//
//         {
//             auto desiredScissorRect =
//                 batch.scissorRectRect.has_value()
//                     ? fullRenderTargetScissorRect.intersectOrEmpty(
//                           batch.scissorRectRect.value())
//                     : fullRenderTargetScissorRect;
//
//             if (desiredScissorRect != currentScissorRect)
//             {
//                 currentScissorRect = desiredScissorRect;
//                 [encoder setScissorRect:make_scissor(currentScissorRect)];
//             }
//         }
//
//         id<MTLRenderPipelineState> drawPipelineState =
//             drawPipeline->pipelineState(renderTarget->pixelFormat());
//
//         // Bind the appropriate image texture, if any.
//         if (auto imageTextureMetal =
//                 static_cast<const TextureMetalImpl*>(batch.imageTexture))
//         {
//             [encoder setFragmentTexture:imageTextureMetal->texture()
//                                 atIndex:IMAGE_TEXTURE_IDX];
//
//             [encoder setFragmentSamplerState:m_imageSamplers[batch.imageSampler
//                                                                  .asKey()]
//                                      atIndex:IMAGE_TEXTURE_IDX];
//         }
//         else
//         {
//             [encoder setFragmentSamplerState:
//                          m_imageSamplers[ImageSampler::LINEAR_CLAMP_SAMPLER_KEY]
//                                      atIndex:IMAGE_TEXTURE_IDX];
//         }
//
//         // Issue any barriers if needed.
//         if (enums::any_flag_set(batch.barriers,
//                                 BarrierFlags::plsAtomic |
//                                     BarrierFlags::plsAtomicPreResolve))
//         {
//             assert(desc.interlockMode == gpu::InterlockMode::atomics);
//             switch (m_metalFeatures.atomicBarrierType)
//             {
//                 case AtomicBarrierType::memoryBarrier:
// #if defined(RIVE_MACOSX)
//                     if (@available(macOS 10.14, *))
//                     {
//                         [encoder
//                             memoryBarrierWithScope:MTLBarrierScopeBuffers |
//                                                    MTLBarrierScopeRenderTargets
//                                        afterStages:MTLRenderStageFragment
//                                       beforeStages:MTLRenderStageFragment];
//                         break;
//                     }
// #endif
//                     // atomicBarrierType shouldn't be "memoryBarrier" in this
//                     // case.
//                     RIVE_UNREACHABLE();
//
//                 case AtomicBarrierType::rasterOrderGroup:
//                     break;
//
//                 case AtomicBarrierType::renderPassBreak:
//                     // On very old hardware that can't support barriers, we just
//                     // take a sledge hammer and break the entire render pass
//                     // between overlapping draws.
//                     // TODO: Is there a lighter way to achieve this?
//                     [encoder endEncoding];
//                     pass.colorAttachments[COLOR_PLANE_IDX].loadAction =
//                         MTLLoadActionLoad;
//                     encoder = makeRenderPassForDraws(
//                         desc, pass, commandBuffer, baselineShaderMiscFlags);
//                     break;
//             }
//         }
//
//         DrawType drawType = batch.drawType;
//         switch (drawType)
//         {
//             case DrawType::midpointFanPatches:
//             case DrawType::midpointFanCenterAAPatches:
//             case DrawType::outerCurvePatches:
//             {
//                 // Draw PLS patches that connect the tessellation vertices.
//                 [encoder setRenderPipelineState:drawPipelineState];
//                 [encoder setVertexBuffer:m_pathPatchVertexBuffer
//                                   offset:0
//                                  atIndex:0];
//                 [encoder setCullMode:MTLCullModeBack];
//                 // Don't use baseInstance in order to run on Apple GPU Family 2.
//                 // TODO: Use baseInstance instead once we deprecate Apple2.
//                 [encoder
//                     setVertexBytes:&batch.baseElement
//                             length:sizeof(uint32_t)
//                            atIndex:METAL_BUFFER_IDX(
//                                        PATH_BASE_INSTANCE_UNIFORM_BUFFER_IDX)];
//                 [encoder
//                     drawIndexedPrimitives:MTLPrimitiveTypeTriangle
//                                indexCount:batch.indexCountPerInstance
//                                 indexType:MTLIndexTypeUInt16
//                               indexBuffer:m_pathPatchIndexBuffer
//                         indexBufferOffset:batch.baseIndex * sizeof(uint16_t)
//                             instanceCount:batch.elementCount];
//                 break;
//             }
//             case DrawType::interiorTriangulation:
//             case DrawType::featherAtlasBlit:
//             {
//                 [encoder setRenderPipelineState:drawPipelineState];
//                 [encoder setVertexBuffer:mtl_buffer(triangleBufferRing())
//                                   offset:0
//                                  atIndex:0];
//                 [encoder setCullMode:MTLCullModeBack];
//                 [encoder drawPrimitives:MTLPrimitiveTypeTriangle
//                             vertexStart:batch.baseElement
//                             vertexCount:batch.elementCount];
//                 break;
//             }
//             case DrawType::imageRect:
//             case DrawType::imageMesh:
//             {
//                 [encoder setRenderPipelineState:drawPipelineState];
//                 [encoder
//                     setVertexBuffer:mtl_buffer(imageDrawInstanceBufferRing())
//                              offset:batch.baseElement *
//                                     sizeof(gpu::ImageDrawInstance)
//                             atIndex:2];
//                 [encoder setCullMode:MTLCullModeNone];
//                 if (drawType == DrawType::imageRect)
//                 {
//                     assert(desc.interlockMode == gpu::InterlockMode::atomics);
//                     [encoder setVertexBuffer:m_imageRectVertexBuffer
//                                       offset:0
//                                      atIndex:0];
//                     [encoder
//                         drawIndexedPrimitives:MTLPrimitiveTypeTriangle
//                                    indexCount:batch.indexCountPerInstance
//                                     indexType:MTLIndexTypeUInt16
//                                   indexBuffer:m_imageRectIndexBuffer
//                             indexBufferOffset:batch.baseIndex * sizeof(uint16_t)
//                                 instanceCount:batch.elementCount];
//                 }
//                 else
//                 {
//                     LITE_RTTI_CAST_OR_BREAK(vertexBuffer,
//                                             RenderBufferMetalImpl*,
//                                             batch.vertexBuffer);
//                     LITE_RTTI_CAST_OR_BREAK(
//                         uvBuffer, RenderBufferMetalImpl*, batch.uvBuffer);
//                     LITE_RTTI_CAST_OR_BREAK(
//                         indexBuffer, RenderBufferMetalImpl*, batch.indexBuffer);
//                     [encoder setVertexBuffer:vertexBuffer->submittedBuffer()
//                                       offset:0
//                                      atIndex:0];
//                     [encoder setVertexBuffer:uvBuffer->submittedBuffer()
//                                       offset:0
//                                      atIndex:1];
//                     [encoder
//                         drawIndexedPrimitives:MTLPrimitiveTypeTriangle
//                                    indexCount:batch.indexCountPerInstance
//                                     indexType:MTLIndexTypeUInt16
//                                   indexBuffer:indexBuffer->submittedBuffer()
//                             indexBufferOffset:batch.baseIndex *
//                                               sizeof(uint16_t)];
//                 }
//                 break;
//             }
//             case DrawType::renderPassInitialize:
//             case DrawType::renderPassResolve:
//             {
//                 assert(desc.interlockMode == gpu::InterlockMode::atomics);
//                 [encoder setRenderPipelineState:drawPipelineState];
//                 [encoder drawPrimitives:MTLPrimitiveTypeTriangleStrip
//                             vertexStart:0
//                             vertexCount:4];
//                 break;
//             }
//             case DrawType::msaaStrokes:
//             case DrawType::msaaMidpointFanBorrowedCoverage:
//             case DrawType::msaaDynamicMidpointFans:
//             case DrawType::msaaMidpointFans:
//             case DrawType::msaaMidpointFanStencilReset:
//             case DrawType::msaaMidpointFanPathsStencil:
//             case DrawType::msaaMidpointFanPathsCover:
//             case DrawType::msaaOuterCubics:
//             case DrawType::clipReset:
//             {
//                 RIVE_UNREACHABLE();
//             }
//         }
//     }
//     [encoder endEncoding];
// }
//
// void RenderContextMetalImpl::postFlush(
//     const RenderContext::FlushResources& flushResources)
// {
//     // Schedule a callback that will unlock the buffers used by this flush,
//     // after the GPU has finished rendering with them. This unblocks the CPU
//     // from reusing them in a future flush.
//     id<MTLCommandBuffer> commandBuffer =
//         (__bridge id<MTLCommandBuffer>)flushResources.externalCommandBuffer;
//     std::mutex& thisFlushLock = m_bufferRingLocks[m_bufferRingIdx];
//     [commandBuffer addCompletedHandler:^(id<MTLCommandBuffer>) {
//       assert(!thisFlushLock.try_lock()); // The mutex should already be locked.
//       thisFlushLock.unlock();
//     }];
// }
// } // namespace rive::gpu
