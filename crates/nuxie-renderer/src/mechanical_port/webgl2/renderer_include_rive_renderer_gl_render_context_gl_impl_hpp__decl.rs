//! Complete mechanical declaration translation of
//! `renderer/include/rive/renderer/gl/render_context_gl_impl.hpp` for the
//! frozen `RIVE_WEBGL + RIVE_CANVAS + WITH_RIVE_TOOLS` authority.

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use super::gl_state_decl::GLState;
use super::gl_utils_decl::{Buffer, Framebuffer, Program, Shader, Texture as GLTexture, VAO};
use super::gles3_decl::{
    GLCapabilities, GLExecutionProvider, GLExecutionStamp, GLenum, GLint, GLuint,
};
use crate::mechanical_port::source::include::rive::gpu_texture_format_hpp::GPUTextureFormat;
use crate::mechanical_port::source::include::rive::refcnt_hpp::{rcp, RefCntTarget};
use crate::mechanical_port::source::include::rive::renderer_hpp::{
    RenderBuffer, RenderBufferFlags, RenderBufferType,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::buffer_ring_hpp::{
    BufferRing, BufferRingContract,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
    DrawBatch, DrawType, FlushDescriptor, InterlockMode, PipelineState, PlatformFeatures,
    ShaderFeatures, ShaderMiscFlags, StorageBufferStructure, TriState,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::render_canvas_hpp::RenderCanvas;
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_helper_impl_hpp::RenderContextHelperImpl;
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::RenderContext;
use crate::mechanical_port::source::renderer::include::rive::renderer::render_target_hpp::RenderTarget;
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_image_hpp::RiveRenderImage;
use crate::mechanical_port::source::renderer::include::rive::renderer::texture_hpp::Texture;
use core::ffi::c_void;
use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::mem::ManuallyDrop;
use std::rc::{Rc, Weak};
use std::sync::{Condvar, Mutex};
use std::thread::JoinHandle;

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_include_rive_renderer_gl_render_context_gl_impl.hpp");

/// Exact declaration imported from `shader_compilation_mode.hpp` by the
/// source's AsyncPipelineManager include.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ShaderCompilationMode {
    #[default]
    allowAsynchronous = 0,
    alwaysSynchronous = 1,
    onlyUbershaders = 2,
}

impl ShaderCompilationMode {
    pub(crate) const standard: Self = Self::allowAsynchronous;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PipelineCreateType {
    sync,
    r#async,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum PipelineStatus {
    #[default]
    notReady,
    ready,
    errored,
}

/// Exact source `StandardPipelineProps`, including the admitted tools field.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct StandardPipelineProps {
    pub(crate) drawType: DrawType,
    pub(crate) shaderFeatures: ShaderFeatures,
    pub(crate) interlockMode: InterlockMode,
    pub(crate) shaderMiscFlags: ShaderMiscFlags,
    #[cfg(feature = "with-rive-tools")]
    pub(crate) synthesizedFailureType:
        crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::SynthesizedFailureType,
}

impl StandardPipelineProps {
    pub(crate) fn createKey(&self, _platformFeatures: &PlatformFeatures) -> u32 {
        crate::mechanical_port::source::renderer::src::gpu_cpp::ShaderUniqueKey(
            self.drawType,
            self.shaderFeatures,
            self.interlockMode,
            self.shaderMiscFlags,
        )
    }
}

/// The three exact source option fields. The Rust execution authority is
/// intentionally not hidden in this source value.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ContextOptions {
    pub(crate) shaderCompilationMode: ShaderCompilationMode,
    pub(crate) disablePixelLocalStorage: bool,
    pub(crate) disableFragmentShaderInterlock: bool,
}

impl Default for ContextOptions {
    fn default() -> Self {
        Self {
            shaderCompilationMode: ShaderCompilationMode::standard,
            disablePixelLocalStorage: false,
            disableFragmentShaderInterlock: false,
        }
    }
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum FeatherAtlasRenderType {
    #[default]
    r16f = 0,
    r32f = 1,
    r32uiFramebufferFetch = 2,
    r8PixelLocalStorageEXT = 3,
    r32uiPixelLocalStorageANGLE = 4,
    r32iAtomicTexture = 5,
    rgba8 = 6,
}

/// The sole authored data member of the abstract PixelLocalStorageImpl base.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PixelLocalStorageImplState {
    pub(crate) m_rasterOrderingEnabled: TriState,
}

impl Default for PixelLocalStorageImplState {
    fn default() -> Self {
        Self {
            m_rasterOrderingEnabled: TriState::unknown,
        }
    }
}

/// Exact virtual surface of the pinned abstract PLS owner. Component 097 owns
/// the concrete `PLSImplWebGL`; this component never manufactures a fallback.
pub(crate) trait PixelLocalStorageImpl {
    fn state(&self) -> &PixelLocalStorageImplState;
    fn stateMut(&mut self) -> &mut PixelLocalStorageImplState;

    fn init(&mut self, _state: GLStateOwner) {}
    fn getSupportedInterlockModes(
        &self,
        capabilities: &GLCapabilities,
        platformFeatures: &mut PlatformFeatures,
    );
    fn resizeTransientPLSBacking(&mut self, _width: u32, _height: u32, _planeCount: u32) {}
    fn resizeAtomicCoverageBacking(&mut self, _width: u32, _height: u32) {}
    fn shaderMiscFlags(&self, _desc: &FlushDescriptor, _drawType: DrawType) -> ShaderMiscFlags {
        ShaderMiscFlags::none
    }
    fn pushShaderDefines(&self, interlockMode: InterlockMode, defines: &mut Vec<&'static str>);
    fn applyPipelineStateOverrides(
        &self,
        _batch: &DrawBatch,
        _desc: &FlushDescriptor,
        _platformFeatures: &PlatformFeatures,
        _pipelineState: &mut PipelineState,
    ) {
    }
    fn activatePixelLocalStorage(
        &mut self,
        renderContextImpl: &mut RenderContextGLImpl,
        desc: &FlushDescriptor,
    );
    fn deactivatePixelLocalStorage(
        &mut self,
        renderContextImpl: &mut RenderContextGLImpl,
        desc: &FlushDescriptor,
    );
    fn onEnableRasterOrdering(&mut self, _enabled: bool) {}
    fn onBarrier(&mut self, _desc: &FlushDescriptor) {}

    fn ensureRasterOrderingEnabled(
        &mut self,
        renderContextImpl: &RenderContextGLImpl,
        desc: &FlushDescriptor,
        enabled: bool,
    ) {
        assert!(
            !enabled
                || renderContextImpl
                    .platformFeatures()
                    .supportsRasterOrderingMode
                || renderContextImpl.platformFeatures().supportsClockwiseMode
        );
        let next = if enabled { TriState::yes } else { TriState::no };
        if self.state().m_rasterOrderingEnabled != next {
            self.onEnableRasterOrdering(enabled);
            self.stateMut().m_rasterOrderingEnabled = next;
            if next == TriState::no {
                self.onBarrier(desc);
            }
        }
    }

    fn barrier(&mut self, desc: &FlushDescriptor) {
        assert_eq!(self.state().m_rasterOrderingEnabled, TriState::no);
        self.onBarrier(desc);
    }

    fn rasterOrderingKnownDisabled(&self) -> bool {
        self.state().m_rasterOrderingEnabled == TriState::no
    }
}

/// Explicit unresolved final-PLS factory seam. The final component supplies
/// this exact owner; `None` is not accepted when WebGL PLS was selected.
pub(crate) trait PixelLocalStorageFactory {
    fn MakePLSImplWebGL(&self) -> Box<dyn PixelLocalStorageImpl>;
}

pub(crate) type GLStateOwner = Rc<RefCell<GLState>>;

#[repr(C)]
pub(crate) struct FeatherAtlasProgram {
    pub(crate) m_program: ManuallyDrop<Program>,
    pub(crate) m_baseInstanceUniformLocation: GLint,
}

impl Default for FeatherAtlasProgram {
    fn default() -> Self {
        Self {
            m_program: ManuallyDrop::new(Program::Zero()),
            m_baseInstanceUniformLocation: -1,
        }
    }
}

impl FeatherAtlasProgram {
    pub(crate) fn id(&self) -> GLuint {
        self.m_program.id()
    }
    pub(crate) fn baseInstanceUniformLocation(&self) -> GLint {
        self.m_baseInstanceUniformLocation
    }
}

#[repr(C)]
pub(crate) struct DrawShader {
    pub(crate) m_id: GLuint,
}

impl Default for DrawShader {
    fn default() -> Self {
        Self { m_id: 0 }
    }
}

impl DrawShader {
    pub(crate) fn id(&self) -> GLuint {
        self.m_id
    }
}

#[repr(C)]
pub(crate) struct DrawProgram {
    pub(crate) m_fragmentShader: *const DrawShader,
    pub(crate) m_vertexShader: *const DrawShader,
    pub(crate) m_pipelineStatus: PipelineStatus,
    pub(crate) m_id: GLuint,
    pub(crate) m_baseInstanceUniformLocation: GLint,
    pub(crate) m_state: ManuallyDrop<GLStateOwner>,
    #[cfg(feature = "with-rive-tools")]
    pub(crate) m_synthesizedFailureType:
        crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::SynthesizedFailureType,
}

impl DrawProgram {
    pub(crate) fn id(&self) -> GLuint {
        self.m_id
    }
    pub(crate) fn baseInstanceUniformLocation(&self) -> GLint {
        self.m_baseInstanceUniformLocation
    }
    pub(crate) fn status(&self) -> PipelineStatus {
        self.m_pipelineStatus
    }
}

pub(crate) struct GLPipelineJobParams {
    pub(crate) props: StandardPipelineProps,
    pub(crate) key: u32,
    pub(crate) platformFeatures: *const PlatformFeatures,
}

pub(crate) struct GLPipelineCompletedJob {
    pub(crate) key: u32,
    pub(crate) program: Box<DrawProgram>,
}

/// Complete state inherited from AsyncPipelineManager<DrawProgram>. The GL
/// subclass uses driver polling and therefore never populates `m_jobThread`.
pub(crate) struct AsyncPipelineManagerGLState {
    pub(crate) m_vertexShaderMap: BTreeMap<u32, Option<Box<DrawShader>>>,
    pub(crate) m_fragmentShaderMap: BTreeMap<u32, Option<Box<DrawShader>>>,
    pub(crate) m_pipelines: BTreeMap<u32, Option<Box<DrawProgram>>>,
    pub(crate) m_jobQueue: VecDeque<GLPipelineJobParams>,
    pub(crate) m_completedJobs: Vec<GLPipelineCompletedJob>,
    pub(crate) m_isDone: bool,
    pub(crate) m_activePipelineCreationCount: u32,
    pub(crate) m_mode: ShaderCompilationMode,
    pub(crate) m_jobThread: Option<JoinHandle<()>>,
    pub(crate) m_currentThreadPipelineKey: Option<u32>,
    pub(crate) m_mutex: Mutex<()>,
    pub(crate) m_newJobCV: Condvar,
    pub(crate) m_jobCompleteCV: Condvar,
    pub(crate) m_sharedObjectReadyCV: Condvar,
}

impl AsyncPipelineManagerGLState {
    pub(crate) fn new(mode: ShaderCompilationMode) -> Self {
        Self {
            m_vertexShaderMap: BTreeMap::new(),
            m_fragmentShaderMap: BTreeMap::new(),
            m_pipelines: BTreeMap::new(),
            m_jobQueue: VecDeque::new(),
            m_completedJobs: Vec::new(),
            m_isDone: false,
            m_activePipelineCreationCount: 0,
            m_mode: mode,
            m_jobThread: None,
            m_currentThreadPipelineKey: None,
            m_mutex: Mutex::new(()),
            m_newJobCV: Condvar::new(),
            m_jobCompleteCV: Condvar::new(),
            m_sharedObjectReadyCV: Condvar::new(),
        }
    }
}

#[repr(C)]
pub(crate) struct GLPipelineManager {
    pub(crate) base: ManuallyDrop<AsyncPipelineManagerGLState>,
    pub(crate) m_context: *mut RenderContextGLImpl,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GLFlushInjector {
    pub(crate) m_maxSupportedInstancesPerFlush: u32,
    pub(crate) m_currentFlushInstanceCount: u32,
}

impl GLFlushInjector {
    pub(crate) fn new(capabilities: &GLCapabilities) -> Self {
        Self {
            m_maxSupportedInstancesPerFlush: capabilities.maxSupportedInstancesPerFlush,
            m_currentFlushInstanceCount: 0,
        }
    }

    pub(crate) fn flushBeforeInstancedDrawIfNeeded(&mut self, nextInstanceCount: u32) {
        if self
            .m_currentFlushInstanceCount
            .wrapping_add(nextInstanceCount)
            > self.m_maxSupportedInstancesPerFlush
        {
            super::gles3_decl::recordGLCommand(super::gles3_decl::GLCommand::Flush);
            self.m_currentFlushInstanceCount = 0;
        }
        self.m_currentFlushInstanceCount += nextInstanceCount;
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct CanvasMirrorEntry {
    pub(crate) mirrorTex: GLuint,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) readFBO: GLuint,
    pub(crate) drawFBO: GLuint,
    pub(crate) hasMirror: bool,
}

/// Shared logical owner for the source canvas-mirror map. Canvas textures keep
/// only a weak sidecar, so their final-release callbacks never need to
/// dereference the source-authored raw `m_owner` pointer.
pub(crate) type CanvasMirrorRegistry = Rc<RefCell<BTreeMap<GLuint, CanvasMirrorEntry>>>;
pub(crate) type WeakCanvasMirrorRegistry = Weak<RefCell<BTreeMap<GLuint, CanvasMirrorEntry>>>;

/// Offset-zero Texture base plus the source's sole GL texture member. The
/// execution owner follows the complete source prefix as a Rust-only lifetime
/// sidecar.
#[repr(C)]
pub(crate) struct TextureGLImpl {
    pub(crate) base: ManuallyDrop<Texture>,
    pub(crate) m_texture: ManuallyDrop<GLTexture>,
    pub(crate) rust_execution: ManuallyDrop<GLExecutionStamp>,
}

#[repr(C)]
pub(crate) struct CanvasSourceTextureGLImpl {
    pub(crate) base: ManuallyDrop<TextureGLImpl>,
    pub(crate) m_owner: *mut RenderContextGLImpl,
    pub(crate) m_glID: GLuint,
    pub(crate) rust_canvas_registry: WeakCanvasMirrorRegistry,
}

#[repr(C)]
pub(crate) struct CanvasMirrorTextureGLImpl {
    pub(crate) base: ManuallyDrop<TextureGLImpl>,
    pub(crate) m_owner: *mut RenderContextGLImpl,
    pub(crate) m_sourceTexID: GLuint,
    pub(crate) rust_canvas_registry: WeakCanvasMirrorRegistry,
}

#[repr(C)]
pub(crate) struct BufferRingGLImpl {
    pub(crate) base: ManuallyDrop<BufferRing>,
    pub(crate) m_target: GLenum,
    pub(crate) m_bufferID: GLuint,
    pub(crate) m_state: ManuallyDrop<GLStateOwner>,
    pub(crate) rust_execution: ManuallyDrop<GLExecutionStamp>,
}

#[repr(C)]
pub(crate) struct StorageBufferRingGLImpl {
    pub(crate) base: ManuallyDrop<BufferRingGLImpl>,
    pub(crate) m_bufferStructure: StorageBufferStructure,
}

#[repr(C)]
pub(crate) struct TexelBufferRingWebGL {
    pub(crate) base: ManuallyDrop<BufferRing>,
    pub(crate) m_bufferStructure: StorageBufferStructure,
    pub(crate) m_state: ManuallyDrop<GLStateOwner>,
    pub(crate) m_textureID: GLuint,
    pub(crate) rust_execution: ManuallyDrop<GLExecutionStamp>,
}

/// Exact source prefix: offset-zero RenderContextHelperImpl base followed by
/// all forty RenderContextGLImpl fields in declaration order. The final two
/// fields are clearly named Rust sidecars and are outside the frozen prefix.
#[repr(C)]
pub(crate) struct RenderContextGLImpl {
    pub(crate) base: ManuallyDrop<RenderContextHelperImpl>,
    pub(crate) m_capabilities: GLCapabilities,
    pub(crate) m_plsImpl: ManuallyDrop<Option<Box<dyn PixelLocalStorageImpl>>>,
    pub(crate) m_colorRampProgram: ManuallyDrop<Program>,
    pub(crate) m_colorRampVAO: ManuallyDrop<VAO>,
    pub(crate) m_colorRampFBO: ManuallyDrop<Framebuffer>,
    pub(crate) m_gradientTexture: GLuint,
    pub(crate) m_gaussianIntegralTexture: ManuallyDrop<GLTexture>,
    pub(crate) m_tessellateProgram: ManuallyDrop<Program>,
    pub(crate) m_tessellateVAO: ManuallyDrop<VAO>,
    pub(crate) m_tessSpanIndexBuffer: ManuallyDrop<Buffer>,
    pub(crate) m_tessellateFBO: ManuallyDrop<Framebuffer>,
    pub(crate) m_tessVertexTexture: GLuint,
    pub(crate) m_featherAtlasRenderType: FeatherAtlasRenderType,
    pub(crate) m_featherAtlasVertexShader: ManuallyDrop<Shader>,
    pub(crate) m_featherAtlasFillProgram: ManuallyDrop<FeatherAtlasProgram>,
    pub(crate) m_featherAtlasStrokeProgram: ManuallyDrop<FeatherAtlasProgram>,
    pub(crate) m_featherAtlasFillPipelineState: PipelineState,
    pub(crate) m_featherAtlasStrokePipelineState: PipelineState,
    pub(crate) m_featherAtlasResolveVertexShader: ManuallyDrop<Shader>,
    pub(crate) m_featherAtlasClearProgram: ManuallyDrop<Program>,
    pub(crate) m_featherAtlasResolveProgram: ManuallyDrop<Program>,
    pub(crate) m_featherAtlasResolveVAO: ManuallyDrop<VAO>,
    pub(crate) m_featherAtlasRenderTexture: ManuallyDrop<GLTexture>,
    pub(crate) m_featherAtlasTexture: ManuallyDrop<GLTexture>,
    pub(crate) m_featherAtlasRenderFBO: ManuallyDrop<Framebuffer>,
    pub(crate) m_featherAtlasResolveFBO: ManuallyDrop<Framebuffer>,
    pub(crate) m_pipelineManager: ManuallyDrop<GLPipelineManager>,
    pub(crate) m_drawVAO: ManuallyDrop<VAO>,
    pub(crate) m_patchVerticesBuffer: ManuallyDrop<Buffer>,
    pub(crate) m_patchIndicesBuffer: ManuallyDrop<Buffer>,
    pub(crate) m_trianglesVAO: ManuallyDrop<VAO>,
    pub(crate) m_imageRectVAO: ManuallyDrop<VAO>,
    pub(crate) m_imageRectVertexBuffer: ManuallyDrop<Buffer>,
    pub(crate) m_imageRectIndexBuffer: ManuallyDrop<Buffer>,
    pub(crate) m_imageMeshVAO: ManuallyDrop<VAO>,
    pub(crate) m_emptyVAO: ManuallyDrop<VAO>,
    pub(crate) m_blitAsDrawProgram: ManuallyDrop<Program>,
    pub(crate) m_state: ManuallyDrop<GLStateOwner>,
    pub(crate) m_testForAdvancedBlendError: bool,
    pub(crate) m_canvasMirrors: ManuallyDrop<CanvasMirrorRegistry>,

    pub(crate) rust_execution: ManuallyDrop<GLExecutionStamp>,
    pub(crate) rust_source_renderer_string: ManuallyDrop<Vec<u8>>,
}

impl RenderContextGLImpl {
    pub(crate) fn capabilities(&self) -> &GLCapabilities {
        &self.m_capabilities
    }
    pub(crate) fn state(&self) -> &GLStateOwner {
        &self.m_state
    }
    pub(crate) fn platformFeatures(&self) -> &PlatformFeatures {
        self.base.base.platformFeatures()
    }
    pub(crate) fn featherAtlasRenderType(&self) -> FeatherAtlasRenderType {
        self.m_featherAtlasRenderType
    }
    pub(crate) fn invalidateGLState(&mut self) {
        super::render_context_gl_impl::invalidateGLState(self)
    }
    pub(crate) fn unbindGLInternalResources(&mut self) {
        super::render_context_gl_impl::unbindGLInternalResources(self)
    }
    pub(crate) fn blitTextureToFramebufferAsDraw(
        &mut self,
        textureID: GLuint,
        bounds: &crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::IAABB,
        renderTargetHeight: u32,
    ) {
        super::render_context_gl_impl::blitTextureToFramebufferAsDraw(
            self,
            textureID,
            bounds,
            renderTargetHeight,
        )
    }
    pub(crate) fn adoptImageTexture(
        &mut self,
        width: u32,
        height: u32,
        textureID: GLuint,
    ) -> rcp<Texture> {
        super::render_context_gl_impl::adoptImageTexture(self, width, height, textureID)
    }
    pub(crate) fn registerCanvasTarget(&mut self, sourceTex: GLuint) {
        super::render_context_gl_impl::registerCanvasTarget(self, sourceTex)
    }
    pub(crate) unsafe fn getCanvasImportMirror(
        &mut self,
        sourceTex: *mut Texture,
        width: u32,
        height: u32,
    ) -> rcp<RiveRenderImage> {
        unsafe {
            super::render_context_gl_impl::getCanvasImportMirror(self, sourceTex, width, height)
        }
    }
    pub(crate) fn unregisterCanvasTarget(&mut self, sourceTex: GLuint) {
        super::render_context_gl_impl::unregisterCanvasTarget(self, sourceTex)
    }
    pub(crate) fn getOrCreateCanvasMirror(
        &mut self,
        sourceTex: GLuint,
        width: u32,
        height: u32,
    ) -> rcp<RiveRenderImage> {
        super::render_context_gl_impl::getOrCreateCanvasMirror(self, sourceTex, width, height)
    }
    pub(crate) fn blitMirrorIfRegistered(&mut self, targetTex: GLuint) {
        super::render_context_gl_impl::blitMirrorIfRegistered(self, targetTex)
    }

    #[cfg(feature = "with-rive-tools")]
    pub(crate) fn testingOnly_resetFeatherAtlasDesiredRenderType(
        &mut self,
        owningRenderContext: &mut RenderContext,
        desiredRenderType: FeatherAtlasRenderType,
    ) -> FeatherAtlasRenderType {
        super::render_context_gl_impl::testingOnly_resetFeatherAtlasDesiredRenderType(
            self,
            owningRenderContext,
            desiredRenderType,
        )
    }

    #[cfg(feature = "with-rive-tools")]
    pub(crate) fn testingOnly_setBlendAdvancedCoherentKHRSupported(
        &mut self,
        supported: bool,
    ) -> bool {
        super::render_context_gl_impl::testingOnly_setBlendAdvancedCoherentKHRSupported(
            self, supported,
        )
    }

    #[cfg(feature = "with-rive-tools")]
    pub(crate) fn testingOnly_setBlendAdvancedKHRSupported(&mut self, supported: bool) -> bool {
        super::render_context_gl_impl::testingOnly_setBlendAdvancedKHRSupported(self, supported)
    }
}

pub(crate) fn MakeContext(
    options: ContextOptions,
    provider: Box<dyn GLExecutionProvider>,
) -> Option<std::pin::Pin<Box<RenderContext>>> {
    super::render_context_gl_impl::MakeContext(options, provider)
}

pub(crate) fn MakeContextDefault(
    provider: Box<dyn GLExecutionProvider>,
) -> Option<std::pin::Pin<Box<RenderContext>>> {
    super::render_context_gl_impl::MakeContext(ContextOptions::default(), provider)
}

pub(crate) const SOURCE_CONTEXT_OPTION_FIELD_COUNT: usize = 3;
pub(crate) const SOURCE_RENDER_CONTEXT_FIELD_COUNT: usize = 40;
pub(crate) const SOURCE_CANVAS_MIRROR_ENTRY_FIELD_COUNT: usize = 6;
pub(crate) const SOURCE_FEATHER_ATLAS_PROGRAM_FIELD_COUNT: usize = 2;
pub(crate) const SOURCE_DRAW_SHADER_FIELD_COUNT: usize = 1;
pub(crate) const SOURCE_DRAW_PROGRAM_FIELD_COUNT: usize = 7;
pub(crate) const SOURCE_GL_FLUSH_INJECTOR_FIELD_COUNT: usize = 2;
pub(crate) const SOURCE_GL_PIPELINE_MANAGER_FIELD_COUNT: usize = 1;
pub(crate) const SOURCE_PLS_IMPL_FIELD_COUNT: usize = 1;
pub(crate) const SOURCE_FIELD_DENOMINATOR: usize = 63;
pub(crate) const RUST_RENDER_CONTEXT_SIDECAR_COUNT: usize = 2;
const _: [(); 21_790] = [(); PINNED_SOURCE.len()];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_header_and_field_denominators_are_locked() {
        assert_eq!(PINNED_SOURCE.lines().count(), 580);
        assert_eq!(SOURCE_CONTEXT_OPTION_FIELD_COUNT, 3);
        assert_eq!(SOURCE_RENDER_CONTEXT_FIELD_COUNT, 40);
        assert_eq!(SOURCE_CANVAS_MIRROR_ENTRY_FIELD_COUNT, 6);
        assert_eq!(SOURCE_FEATHER_ATLAS_PROGRAM_FIELD_COUNT, 2);
        assert_eq!(SOURCE_DRAW_SHADER_FIELD_COUNT, 1);
        assert_eq!(SOURCE_DRAW_PROGRAM_FIELD_COUNT, 7);
        assert_eq!(SOURCE_GL_FLUSH_INJECTOR_FIELD_COUNT, 2);
        assert_eq!(SOURCE_GL_PIPELINE_MANAGER_FIELD_COUNT, 1);
        assert_eq!(SOURCE_PLS_IMPL_FIELD_COUNT, 1);
        assert_eq!(SOURCE_FIELD_DENOMINATOR, 63);
        assert_eq!(std::mem::offset_of!(RenderContextGLImpl, base), 0);
        assert!(
            std::mem::offset_of!(RenderContextGLImpl, rust_execution)
                > std::mem::offset_of!(RenderContextGLImpl, m_canvasMirrors)
        );
    }
}
