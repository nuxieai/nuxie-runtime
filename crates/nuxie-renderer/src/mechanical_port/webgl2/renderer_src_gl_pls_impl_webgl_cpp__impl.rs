//! Complete mechanical implementation translation of
//! `renderer/src/gl/pls_impl_webgl.cpp` for `RIVE_WEBGL`.

#![allow(non_snake_case, non_upper_case_globals)]

use super::gl_state_decl::ScissorAction;
use super::gl_utils_decl as glutils;
use super::gles3_decl::*;
use super::render_context_gl_decl::{
    PixelLocalStorageFactory, PixelLocalStorageImpl, PixelLocalStorageImplState,
    RenderContextGLImpl,
};
use super::render_context_gl_impl::{unpackColorToRGBA32FPremul, withRenderTargetGL};
use super::render_target_gl_decl::{
    FramebufferRenderTargetGL, RenderTargetGL, RenderTargetGLApi,
    FRAMEBUFFER_RENDER_TARGET_GL_LITE_RTTI_TYPE_ID,
};
use crate::mechanical_port::source::include::utils::lite_rtti_hpp::LiteRttiBase;
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
    self as gpu, FlushDescriptor, InterlockMode, LoadAction, PlatformFeatures,
};

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_gl_pls_impl_webgl.cpp");
const _: [(); 11_347] = [(); PINNED_SOURCE.len()];

const COLOR_PLANE_IDX: GLint = 0;
const CLIP_PLANE_IDX: GLint = 1;
const SCRATCH_COLOR_PLANE_IDX: GLint = 2;
const COVERAGE_PLANE_IDX: GLint = 3;
const GLSL_PLS_IMPL_ANGLE: &str = "EXPORTED_PLS_IMPL_ANGLE";
pub(crate) const DEPRECATED_WEBGL_PLS_WARNING: &str = "WEBGL_shader_pixel_local_storage is advertised, but a deprecated version has been detected. Disabling.";

/// Exact provider bridge for the source's coherent WebGL PLS enable helper.
/// The provider contract owns the JS-object lookup, validation, warning, and
/// per-context `gl.pls` retention because those values cannot cross into Rust.
pub(crate) fn webglEnableShaderPixelLocalStorageCoherent(
    executionDomain: &GLExecutionDomain,
) -> bool {
    executionDomain.enableWebGLShaderPixelLocalStorageCoherent(DEPRECATED_WEBGL_PLS_WARNING)
        == WebGLShaderPixelLocalStorageEnableResult::Enabled
}

pub(crate) fn framebufferTexturePixelLocalStorageANGLE(
    plane: GLint,
    backingTexture: GLuint,
    level: GLint,
    layer: GLint,
    usage: GLenum,
) {
    recordGLCommand(GLCommand::FramebufferTexturePixelLocalStorageANGLE {
        plane,
        backing_texture: backingTexture,
        level,
        layer,
        usage,
    });
}

pub(crate) fn framebufferPixelLocalClearValuefvANGLE(
    plane: GLint,
    value: [GLfloat; 4],
) {
    recordGLCommand(GLCommand::FramebufferPixelLocalClearValuefvANGLE { plane, value });
}

pub(crate) fn beginPixelLocalStorageANGLE(loadOps: &[GLenum]) {
    recordGLCommand(GLCommand::BeginPixelLocalStorageANGLE {
        load_ops: loadOps.to_vec(),
    });
}

pub(crate) fn endPixelLocalStorageANGLE(storeOps: &[GLenum]) {
    recordGLCommand(GLCommand::EndPixelLocalStorageANGLE {
        store_ops: storeOps.to_vec(),
    });
}

pub(crate) fn getFramebufferPixelLocalStorageParameterivANGLE(
    executionDomain: &GLExecutionDomain,
    plane: GLint,
    parameter: GLenum,
) -> GLint {
    executionDomain.getFramebufferPixelLocalStorageParameter(plane, parameter)
}

pub(crate) fn webglEnableProvokingVertex(executionDomain: &GLExecutionDomain) -> bool {
    executionDomain.enableWebGLProvokingVertex()
}

pub(crate) fn provokingVertexANGLE(provokeMode: GLenum) {
    recordGLCommand(GLCommand::ProvokingVertex(provokeMode));
}

fn webgl_load_op(loadAction: LoadAction) -> GLenum {
    match loadAction {
        LoadAction::clear => GL_LOAD_OP_CLEAR_ANGLE,
        LoadAction::preserveRenderTarget => GL_LOAD_OP_LOAD_ANGLE,
        LoadAction::dontCare => GL_LOAD_OP_ZERO_ANGLE,
    }
}

unsafe fn framebufferRenderTargetGL(
    renderTarget: &mut dyn RenderTargetGLApi,
) -> Option<&mut FramebufferRenderTargetGL> {
    if renderTarget.base().liteTypeID() != FRAMEBUFFER_RENDER_TARGET_GL_LITE_RTTI_TYPE_ID {
        return None;
    }
    let base = renderTarget.baseMut() as *mut RenderTargetGL;
    Some(unsafe { &mut *base.cast::<FramebufferRenderTargetGL>() })
}

#[repr(C)]
pub(crate) struct PLSImplWebGL {
    state: PixelLocalStorageImplState,
}

impl Default for PLSImplWebGL {
    fn default() -> Self {
        Self {
            state: PixelLocalStorageImplState::default(),
        }
    }
}

impl PixelLocalStorageImpl for PLSImplWebGL {
    fn state(&self) -> &PixelLocalStorageImplState {
        &self.state
    }

    fn stateMut(&mut self) -> &mut PixelLocalStorageImplState {
        &mut self.state
    }

    fn getSupportedInterlockModes(
        &self,
        capabilities: &GLCapabilities,
        platformFeatures: &mut PlatformFeatures,
    ) {
        assert!(capabilities.ANGLE_shader_pixel_local_storage);
        if capabilities.ANGLE_shader_pixel_local_storage_coherent {
            platformFeatures.supportsRasterOrderingMode = true;
        }
    }

    fn activatePixelLocalStorage(
        &mut self,
        renderContextImpl: &mut RenderContextGLImpl,
        desc: &FlushDescriptor,
    ) {
        let execution = (&*renderContextImpl.rust_execution).clone();
        let capabilities = *renderContextImpl.capabilities();
        unsafe {
            withRenderTargetGL(
                desc.renderTarget
                    .expect("PLS activation requires FlushDescriptor::renderTarget"),
                &execution,
                |renderTarget| {
                    renderTarget.allocateWebGLPLSBacking(&capabilities);

                    if let Some(framebufferRenderTarget) =
                        unsafe { framebufferRenderTargetGL(renderTarget) }
                    {
                        framebufferRenderTarget.allocateOffscreenTargetTexture();
                        if desc.colorLoadAction == LoadAction::preserveRenderTarget {
                            framebufferRenderTarget
                                .bindDestinationFramebuffer(GL_READ_FRAMEBUFFER);
                            framebufferRenderTarget.bindTextureFramebuffer(GL_DRAW_FRAMEBUFFER);
                            renderContextImpl.state().borrow_mut().setPipelineState(
                                &gpu::COLOR_ONLY_PIPELINE_STATE,
                                ScissorAction::disable,
                            );
                            glutils::BlitFramebuffer(
                                desc.renderTargetUpdateBounds,
                                framebufferRenderTarget.height(),
                                GL_COLOR_BUFFER_BIT,
                            );
                        }
                    }

                    renderTarget.bindHeadlessFramebuffer(&capabilities);
                    if desc.colorLoadAction == LoadAction::clear {
                        framebufferPixelLocalClearValuefvANGLE(
                            COLOR_PLANE_IDX,
                            unpackColorToRGBA32FPremul(desc.colorClearValue),
                        );
                    }
                    let clipLoadAction = if desc.combinedShaderFeatures.0
                        & gpu::ShaderFeatures::ENABLE_CLIPPING.0
                        != 0
                    {
                        GL_LOAD_OP_ZERO_ANGLE
                    } else {
                        GL_DONT_CARE
                    };
                    let loadOps = [
                        webgl_load_op(desc.colorLoadAction),
                        clipLoadAction,
                        GL_DONT_CARE,
                        GL_LOAD_OP_ZERO_ANGLE,
                    ];
                    const _: () = assert!(COLOR_PLANE_IDX == 0);
                    const _: () = assert!(CLIP_PLANE_IDX == 1);
                    const _: () = assert!(SCRATCH_COLOR_PLANE_IDX == 2);
                    const _: () = assert!(COVERAGE_PLANE_IDX == 3);
                    beginPixelLocalStorageANGLE(&loadOps);
                },
            );
        }
    }

    fn deactivatePixelLocalStorage(
        &mut self,
        renderContextImpl: &mut RenderContextGLImpl,
        desc: &FlushDescriptor,
    ) {
        const STORE_OPS: [GLenum; 4] = [
            GL_STORE_OP_STORE_ANGLE,
            GL_DONT_CARE,
            GL_DONT_CARE,
            GL_DONT_CARE,
        ];
        const _: () = assert!(COLOR_PLANE_IDX == 0);
        const _: () = assert!(CLIP_PLANE_IDX == 1);
        const _: () = assert!(SCRATCH_COLOR_PLANE_IDX == 2);
        const _: () = assert!(COVERAGE_PLANE_IDX == 3);
        endPixelLocalStorageANGLE(&STORE_OPS);

        let execution = (&*renderContextImpl.rust_execution).clone();
        unsafe {
            withRenderTargetGL(
                desc.renderTarget
                    .expect("PLS deactivation requires FlushDescriptor::renderTarget"),
                &execution,
                |renderTarget| {
                    if let Some(framebufferRenderTarget) =
                        unsafe { framebufferRenderTargetGL(renderTarget) }
                    {
                        framebufferRenderTarget.bindTextureFramebuffer(GL_READ_FRAMEBUFFER);
                        framebufferRenderTarget
                            .bindDestinationFramebuffer(GL_DRAW_FRAMEBUFFER);
                        renderContextImpl.state().borrow_mut().setPipelineState(
                            &gpu::COLOR_ONLY_PIPELINE_STATE,
                            ScissorAction::disable,
                        );
                        glutils::BlitFramebuffer(
                            desc.renderTargetUpdateBounds,
                            framebufferRenderTarget.height(),
                            GL_COLOR_BUFFER_BIT,
                        );
                    }
                },
            );
        }
    }

    fn pushShaderDefines(
        &self,
        _interlockMode: InterlockMode,
        defines: &mut Vec<&'static str>,
    ) {
        defines.push(GLSL_PLS_IMPL_ANGLE);
    }
}

pub(crate) struct PLSImplWebGLFactory;

impl PixelLocalStorageFactory for PLSImplWebGLFactory {
    fn MakePLSImplWebGL(&self) -> Box<dyn PixelLocalStorageImpl> {
        Box::new(PLSImplWebGL::default())
    }
}

pub(crate) static PLS_IMPL_WEBGL_FACTORY: PLSImplWebGLFactory = PLSImplWebGLFactory;

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::render_context_gl_decl::ContextOptions;
    use super::super::render_context_gl_impl::{
        flush as flushRenderContextGL, newComponent097SelectedContextOwner,
        newComponent097TestContextOwner,
    };
    use super::super::render_target_gl_decl::TextureRenderTargetGL;
    use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_helper_impl_hpp::RenderContextHelperImplContract;
    use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::BlockAllocatedLinkedList;
    use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::DitherMode;
    use std::cell::{Cell, RefCell};
    use std::ptr::NonNull;
    use std::rc::Rc;
    use std::sync::Arc;

    #[derive(Default)]
    struct ProviderTrace {
        commands: Vec<GLCommand>,
        coherentEnableCalls: usize,
        deprecatedPLSWarnings: Vec<String>,
        provokingVertexEnableCalls: usize,
        enabledExtensions: Vec<String>,
        plsQueries: Vec<(GLint, GLenum)>,
    }

    struct Component097Provider {
        trace: Rc<RefCell<ProviderTrace>>,
        coherentEnableResult: Rc<Cell<WebGLShaderPixelLocalStorageEnableResult>>,
        provokingVertexEnabled: Rc<Cell<bool>>,
        queryResult: Rc<Cell<GLint>>,
        nextName: GLuint,
        lifecycleIngress: Option<GLContextLifecycleIngress>,
        finalReleaseIngress: Option<GLFinalReleaseIngress>,
        finalReleaseWake: Arc<TestFinalReleaseWake>,
        rendererString: Vec<u8>,
    }

    impl Component097Provider {
        fn nextName(&mut self) -> GLuint {
            let name = self.nextName;
            self.nextName += 1;
            name
        }
    }

    impl GLExecutionProvider for Component097Provider {
        fn installContextLifecycleIngress(&mut self, ingress: GLContextLifecycleIngress) {
            assert!(self.lifecycleIngress.replace(ingress).is_none());
        }

        fn installFinalReleaseIngress(
            &mut self,
            ingress: GLFinalReleaseIngress,
        ) -> Arc<dyn nuxie_ore_metal::gpu_resource::ResourceFinalReleaseWake> {
            assert!(self.finalReleaseIngress.replace(ingress).is_none());
            self.finalReleaseWake.clone()
        }

        fn submit(&mut self, command: GLCommand) {
            self.trace.borrow_mut().commands.push(command);
        }

        fn generateObject(&mut self, _kind: GLObjectKind) -> GLuint {
            self.nextName()
        }

        fn createProgram(&mut self) -> GLuint {
            self.nextName()
        }

        fn createShader(&mut self, _shaderType: GLenum) -> GLuint {
            self.nextName()
        }

        fn getInteger(&mut self, parameter: GLenum) -> GLint {
            if parameter == GL_MAX_TEXTURE_SIZE { 4096 } else { 0 }
        }

        fn getString(&mut self, parameter: GLenum) -> Option<Vec<u8>> {
            match parameter {
                GL_VERSION => Some(b"OpenGL ES 3.0 component097\0".to_vec()),
                GL_RENDERER | GL_UNMASKED_RENDERER_WEBGL => Some(self.rendererString.clone()),
                _ => None,
            }
        }

        fn getExtension(&mut self, _index: GLuint) -> Option<Vec<u8>> {
            None
        }

        fn enableWebGLExtension(&mut self, name: &str) -> bool {
            self.trace
                .borrow_mut()
                .enabledExtensions
                .push(name.to_owned());
            name == "WEBGL_provoking_vertex"
        }

        fn enableWebGLShaderPixelLocalStorageCoherent(
            &mut self,
            deprecatedVersionWarning: &'static str,
        ) -> WebGLShaderPixelLocalStorageEnableResult {
            self.trace.borrow_mut().coherentEnableCalls += 1;
            let result = self.coherentEnableResult.get();
            if result == WebGLShaderPixelLocalStorageEnableResult::DeprecatedVersion {
                self.trace
                    .borrow_mut()
                    .deprecatedPLSWarnings
                    .push(deprecatedVersionWarning.to_owned());
            }
            result
        }

        fn enableWebGLProvokingVertex(&mut self) -> bool {
            self.trace.borrow_mut().provokingVertexEnableCalls += 1;
            self.provokingVertexEnabled.get()
        }

        fn getFramebufferPixelLocalStorageParameter(
            &mut self,
            plane: GLint,
            parameter: GLenum,
        ) -> GLint {
            self.trace.borrow_mut().plsQueries.push((plane, parameter));
            self.queryResult.get()
        }

        fn isObject(&mut self, _kind: GLObjectKind, name: GLuint) -> bool {
            name != 0
        }

        fn checkFramebufferStatus(&mut self, _target: GLenum) -> GLenum {
            GL_FRAMEBUFFER_COMPLETE
        }

        fn shaderParameter(&mut self, _shader: GLuint, _parameter: GLenum) -> GLint {
            GL_TRUE.into()
        }

        fn shaderInfoLog(&mut self, _shader: GLuint, _maxLength: usize) -> Vec<u8> {
            Vec::new()
        }

        fn programParameter(&mut self, _program: GLuint, _parameter: GLenum) -> GLint {
            GL_TRUE.into()
        }

        fn programInfoLog(&mut self, _program: GLuint, _maxLength: usize) -> Vec<u8> {
            Vec::new()
        }

        fn uniformBlockIndex(&mut self, _program: GLuint, _name: &[u8]) -> GLuint {
            0
        }

        fn uniformLocation(&mut self, _program: GLuint, _name: &[u8]) -> GLint {
            0
        }

        fn readPixelsRGBA8(
            &mut self,
            _x: i32,
            _y: i32,
            width: u32,
            height: u32,
        ) -> Vec<u8> {
            vec![0; width as usize * height as usize * 4]
        }

        fn contextLost(&mut self, _nextGeneration: u64) {}
    }

    fn boxedProviderForRenderer(
        renderer: &str,
    ) -> (
        Box<dyn GLExecutionProvider>,
        Rc<RefCell<ProviderTrace>>,
        Rc<Cell<WebGLShaderPixelLocalStorageEnableResult>>,
        Rc<Cell<bool>>,
        Rc<Cell<GLint>>,
    ) {
        let trace = Rc::new(RefCell::new(ProviderTrace::default()));
        let coherentEnableResult = Rc::new(Cell::new(
            WebGLShaderPixelLocalStorageEnableResult::ExtensionUnavailable,
        ));
        let provokingVertexEnabled = Rc::new(Cell::new(false));
        let queryResult = Rc::new(Cell::new(0));
        let provider = Box::new(Component097Provider {
            trace: trace.clone(),
            coherentEnableResult: coherentEnableResult.clone(),
            provokingVertexEnabled: provokingVertexEnabled.clone(),
            queryResult: queryResult.clone(),
            nextName: 100,
            lifecycleIngress: None,
            finalReleaseIngress: None,
            finalReleaseWake: Arc::new(TestFinalReleaseWake::default()),
            rendererString: format!("{renderer}\0").into_bytes(),
        });
        (
            provider,
            trace,
            coherentEnableResult,
            provokingVertexEnabled,
            queryResult,
        )
    }

    fn providerFixture() -> (
        GLExecutionDomain,
        Rc<RefCell<ProviderTrace>>,
        Rc<Cell<WebGLShaderPixelLocalStorageEnableResult>>,
        Rc<Cell<bool>>,
        Rc<Cell<GLint>>,
    ) {
        let (provider, trace, coherentEnableResult, provokingVertexEnabled, queryResult) =
            boxedProviderForRenderer("WebGL component097 test renderer");
        let domain = GLExecutionDomain::new(provider);
        (
            domain,
            trace,
            coherentEnableResult,
            provokingVertexEnabled,
            queryResult,
        )
    }

    fn capabilities() -> GLCapabilities {
        GLCapabilities {
            isGLES: true,
            contextVersionMajor: 3,
            ANGLE_shader_pixel_local_storage: true,
            ANGLE_shader_pixel_local_storage_coherent: true,
            supportsETC2: true,
            ..GLCapabilities::default()
        }
    }

    fn flushDescriptor(renderTarget: NonNull<gpu::RenderTarget>) -> FlushDescriptor {
        FlushDescriptor {
            renderTarget: Some(renderTarget),
            combinedShaderFeatures: gpu::ShaderFeatures::NONE,
            interlockMode: InterlockMode::rasterOrdering,
            msaaSampleCount: 0,
            colorLoadAction: LoadAction::dontCare,
            colorClearValue: 0,
            coverageClearValue: 0,
            depthClearValue: 0.0,
            stencilClearValue: 0,
            renderTargetUpdateBounds: gpu::IAABB {
                left: 1,
                top: 1,
                right: 7,
                bottom: 5,
            },
            virtualTileWidth: 0,
            virtualTileHeight: 0,
            manuallyResolved: false,
            fixedFunctionColorOutput: false,
            featherAtlasTextureWidth: 0,
            featherAtlasTextureHeight: 0,
            featherAtlasContentWidth: 0,
            featherAtlasContentHeight: 0,
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
            ditherMode: DitherMode::none,
            #[cfg(feature = "with-rive-tools")]
            synthesizedFailureType: gpu::SynthesizedFailureType::none,
            externalCommandBuffer: None,
            featherAtlasFillBatches: None,
            featherAtlasFillBatchCount: 0,
            featherAtlasStrokeBatches: None,
            featherAtlasStrokeBatchCount: 0,
            drawList: None,
            firstDstBlendBarrier: None,
            unresolvedBarriers: gpu::BarrierFlags::none,
        }
    }

    #[test]
    fn source_denominator_and_load_op_mapping_are_exact() {
        assert_eq!(PINNED_SOURCE.lines().count(), 337);
        assert_eq!(PINNED_SOURCE.len(), 11_347);
        assert_eq!(webgl_load_op(LoadAction::clear), GL_LOAD_OP_CLEAR_ANGLE);
        assert_eq!(
            webgl_load_op(LoadAction::preserveRenderTarget),
            GL_LOAD_OP_LOAD_ANGLE
        );
        assert_eq!(webgl_load_op(LoadAction::dontCare), GL_LOAD_OP_ZERO_ANGLE);
    }

    #[test]
    fn coherent_pls_enables_raster_ordering_and_exports_exact_define() {
        let mut capabilities = GLCapabilities {
            ANGLE_shader_pixel_local_storage: true,
            ANGLE_shader_pixel_local_storage_coherent: true,
            ..GLCapabilities::default()
        };
        let mut features = PlatformFeatures::default();
        let pls = PLSImplWebGL::default();
        pls.getSupportedInterlockModes(&capabilities, &mut features);
        assert!(features.supportsRasterOrderingMode);

        capabilities.ANGLE_shader_pixel_local_storage_coherent = false;
        features.supportsRasterOrderingMode = false;
        pls.getSupportedInterlockModes(&capabilities, &mut features);
        assert!(!features.supportsRasterOrderingMode);

        let mut defines = Vec::new();
        pls.pushShaderDefines(InterlockMode::rasterOrdering, &mut defines);
        assert_eq!(defines, ["EXPORTED_PLS_IMPL_ANGLE"]);
    }

    #[test]
    #[should_panic]
    fn supported_modes_rejects_missing_pls_capability() {
        PLSImplWebGL::default().getSupportedInterlockModes(
            &GLCapabilities::default(),
            &mut PlatformFeatures::default(),
        );
    }

    #[test]
    fn factory_constructs_the_exact_webgl_pls_owner() {
        let pls = PLS_IMPL_WEBGL_FACTORY.MakePLSImplWebGL();
        assert_eq!(
            pls.state().m_rasterOrderingEnabled,
            crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::TriState::unknown
        );
    }

    #[test]
    fn make_context_selects_webgl_pls_only_for_the_exact_source_conditions() {
        for (
            renderer,
            disablePixelLocalStorage,
            coherentEnableResultValue,
            expectCapability,
            expectPLS,
        ) in [
            (
                "WebGL component097 renderer",
                false,
                WebGLShaderPixelLocalStorageEnableResult::Enabled,
                true,
                true,
            ),
            (
                "WebGL component097 renderer",
                true,
                WebGLShaderPixelLocalStorageEnableResult::Enabled,
                true,
                false,
            ),
            (
                "Adreno (TM) 640",
                false,
                WebGLShaderPixelLocalStorageEnableResult::Enabled,
                true,
                false,
            ),
            (
                "WebGL component097 renderer",
                false,
                WebGLShaderPixelLocalStorageEnableResult::ExtensionUnavailable,
                false,
                false,
            ),
            (
                "WebGL component097 renderer",
                false,
                WebGLShaderPixelLocalStorageEnableResult::NonCoherent,
                false,
                false,
            ),
            (
                "WebGL component097 renderer",
                false,
                WebGLShaderPixelLocalStorageEnableResult::DeprecatedVersion,
                false,
                false,
            ),
        ] {
            let (provider, trace, coherentEnableResult, _, _) =
                boxedProviderForRenderer(renderer);
            coherentEnableResult.set(coherentEnableResultValue);
            let domain = GLExecutionDomain::new(provider);
            let context = newComponent097SelectedContextOwner(
                ContextOptions {
                    disablePixelLocalStorage,
                    ..ContextOptions::default()
                },
                domain.clone(),
            )
            .expect("OpenGL ES 3.0 component097 provider creates a context");

            assert_eq!(
                context.capabilities().ANGLE_shader_pixel_local_storage,
                expectCapability
            );
            assert_eq!(
                context
                    .capabilities()
                    .ANGLE_shader_pixel_local_storage_coherent,
                expectCapability
            );
            assert_eq!(context.m_plsImpl.is_some(), expectPLS);
            assert_eq!(
                context.platformFeatures().supportsRasterOrderingMode,
                expectPLS
            );
            assert_eq!(trace.borrow().coherentEnableCalls, 1);
            drop(context);
            domain.shutdown();
        }
    }

    #[test]
    fn make_context_factory_owner_wraps_an_empty_non_msaa_flush_in_pls() {
        let (provider, trace, coherentEnableResult, _, _) =
            boxedProviderForRenderer("WebGL component097 renderer");
        coherentEnableResult.set(WebGLShaderPixelLocalStorageEnableResult::Enabled);
        let domain = GLExecutionDomain::new(provider);
        let mut context = newComponent097SelectedContextOwner(
            ContextOptions::default(),
            domain.clone(),
        )
            .expect("coherent WebGL PLS creates a context");
        RenderContextHelperImplContract::resizeFlushUniformBuffer(
            &mut *context,
            std::mem::size_of::<gpu::FlushUniforms>(),
        );

        let execution = (&*context.rust_execution).clone();
        let mut target = TextureRenderTargetGL::new(8, 6, execution.clone());
        target.setTargetTexture(77);
        target.m_headlessFramebuffer.0.m_id = 901;
        target.m_webglPLSBackingR32UI.0.m_id = 801;
        target.m_webglPLSBackingR32UIFallback.0.m_id = 802;
        target.m_webglPLSBackingRGBA8.0.m_id = 803;
        target.m_webglPLSBindingsDirty = true;
        let renderTarget = NonNull::from(&mut *target.base.base);
        let mut drawList = BlockAllocatedLinkedList::<gpu::DrawBatch>::default();
        let mut desc = flushDescriptor(renderTarget);
        desc.drawList = Some(NonNull::from(&mut drawList));
        trace.borrow_mut().commands.clear();

        unsafe { flushRenderContextGL(&mut context, &desc) };

        let sourcePLSCommands: Vec<_> = trace
            .borrow()
            .commands
            .iter()
            .filter(|command| {
                matches!(
                    command,
                    GLCommand::FramebufferTexturePixelLocalStorageANGLE { .. }
                        | GLCommand::FramebufferPixelLocalClearValuefvANGLE { .. }
                        | GLCommand::BeginPixelLocalStorageANGLE { .. }
                        | GLCommand::EndPixelLocalStorageANGLE { .. }
                )
            })
            .cloned()
            .collect();
        assert_eq!(
            sourcePLSCommands,
            [
                GLCommand::FramebufferTexturePixelLocalStorageANGLE {
                    plane: COLOR_PLANE_IDX,
                    backing_texture: 77,
                    level: 0,
                    layer: 0,
                    usage: GL_NONE,
                },
                GLCommand::FramebufferTexturePixelLocalStorageANGLE {
                    plane: COVERAGE_PLANE_IDX,
                    backing_texture: 801,
                    level: 0,
                    layer: 0,
                    usage: GL_NONE,
                },
                GLCommand::FramebufferTexturePixelLocalStorageANGLE {
                    plane: CLIP_PLANE_IDX,
                    backing_texture: 801,
                    level: 0,
                    layer: 1,
                    usage: GL_NONE,
                },
                GLCommand::FramebufferTexturePixelLocalStorageANGLE {
                    plane: SCRATCH_COLOR_PLANE_IDX,
                    backing_texture: 803,
                    level: 0,
                    layer: 0,
                    usage: GL_NONE,
                },
                GLCommand::BeginPixelLocalStorageANGLE {
                    load_ops: vec![
                        GL_LOAD_OP_ZERO_ANGLE,
                        GL_DONT_CARE,
                        GL_DONT_CARE,
                        GL_LOAD_OP_ZERO_ANGLE,
                    ],
                },
                GLCommand::EndPixelLocalStorageANGLE {
                    store_ops: vec![
                        GL_STORE_OP_STORE_ANGLE,
                        GL_DONT_CARE,
                        GL_DONT_CARE,
                        GL_DONT_CARE,
                    ],
                },
            ]
        );

        drop(target);
        drop(context);
        domain.shutdown();
    }

    #[test]
    fn every_webgl_bridge_forwards_synchronously_with_owned_payloads() {
        let (domain, trace, coherentEnableResult, provokingVertexEnabled, queryResult) =
            providerFixture();
        coherentEnableResult.set(WebGLShaderPixelLocalStorageEnableResult::Enabled);
        provokingVertexEnabled.set(true);
        queryResult.set(73);

        domain.withCurrent(|| {
            assert!(webglEnableShaderPixelLocalStorageCoherent(&domain));
            assert!(webglEnableProvokingVertex(&domain));
            assert_eq!(
                getFramebufferPixelLocalStorageParameterivANGLE(
                    &domain,
                    2,
                    GL_PIXEL_LOCAL_TEXTURE_LEVEL_ANGLE,
                ),
                73
            );
            framebufferTexturePixelLocalStorageANGLE(1, 91, 2, 3, GL_NONE);
            framebufferPixelLocalClearValuefvANGLE(0, [0.1, 0.2, 0.3, 0.4]);
            beginPixelLocalStorageANGLE(&[1, 2, 3, 4]);
            endPixelLocalStorageANGLE(&[5, 6, 7, 8]);
            provokingVertexANGLE(GL_FIRST_VERTEX_CONVENTION_ANGLE);
        });

        let traceRef = trace.borrow();
        assert_eq!(traceRef.coherentEnableCalls, 1);
        assert_eq!(traceRef.provokingVertexEnableCalls, 1);
        assert!(traceRef.enabledExtensions.is_empty());
        assert_eq!(
            traceRef.plsQueries,
            [(2, GL_PIXEL_LOCAL_TEXTURE_LEVEL_ANGLE)]
        );
        assert_eq!(
            traceRef.commands,
            [
                GLCommand::FramebufferTexturePixelLocalStorageANGLE {
                    plane: 1,
                    backing_texture: 91,
                    level: 2,
                    layer: 3,
                    usage: GL_NONE,
                },
                GLCommand::FramebufferPixelLocalClearValuefvANGLE {
                    plane: 0,
                    value: [0.1, 0.2, 0.3, 0.4],
                },
                GLCommand::BeginPixelLocalStorageANGLE {
                    load_ops: vec![1, 2, 3, 4],
                },
                GLCommand::EndPixelLocalStorageANGLE {
                    store_ops: vec![5, 6, 7, 8],
                },
                GLCommand::ProvokingVertex(GL_FIRST_VERTEX_CONVENTION_ANGLE),
            ]
        );
        drop(traceRef);

        for result in [
            WebGLShaderPixelLocalStorageEnableResult::ExtensionUnavailable,
            WebGLShaderPixelLocalStorageEnableResult::NonCoherent,
            WebGLShaderPixelLocalStorageEnableResult::DeprecatedVersion,
        ] {
            coherentEnableResult.set(result);
            domain.withCurrent(|| {
                assert!(!webglEnableShaderPixelLocalStorageCoherent(&domain));
            });
        }
        provokingVertexEnabled.set(false);
        queryResult.set(0);
        domain.withCurrent(|| {
            assert!(!webglEnableProvokingVertex(&domain));
            assert_eq!(
                getFramebufferPixelLocalStorageParameterivANGLE(
                    &domain,
                    0,
                    GL_PIXEL_LOCAL_TEXTURE_NAME_ANGLE,
                ),
                0
            );
        });
        let traceRef = trace.borrow();
        assert_eq!(traceRef.coherentEnableCalls, 4);
        assert_eq!(traceRef.provokingVertexEnableCalls, 2);
        assert_eq!(
            traceRef.deprecatedPLSWarnings,
            [DEPRECATED_WEBGL_PLS_WARNING]
        );
        assert_eq!(
            traceRef.plsQueries,
            [
                (2, GL_PIXEL_LOCAL_TEXTURE_LEVEL_ANGLE),
                (0, GL_PIXEL_LOCAL_TEXTURE_NAME_ANGLE),
            ]
        );
        drop(traceRef);
        domain.shutdown();
    }

    #[test]
    fn texture_target_clear_and_clip_begin_then_end_with_exact_plane_ops() {
        let (domain, trace, _, _, _) = providerFixture();
        let mut context = newComponent097TestContextOwner(capabilities(), domain.clone());
        trace.borrow_mut().commands.clear();

        let execution = domain.stamp();
        let mut target = TextureRenderTargetGL::new(8, 6, execution.clone());
        target.setTargetTexture(77);
        target.m_headlessFramebuffer.0.m_id = 901;
        target.m_webglPLSBackingR32UI.0.m_id = 801;
        target.m_webglPLSBackingR32UIFallback.0.m_id = 802;
        target.m_webglPLSBackingRGBA8.0.m_id = 803;
        target.m_webglPLSBindingsDirty = true;
        let renderTarget = NonNull::from(&mut *target.base.base);
        let mut desc = flushDescriptor(renderTarget);
        desc.colorLoadAction = LoadAction::clear;
        desc.colorClearValue = 0x8040_2010;
        desc.combinedShaderFeatures = gpu::ShaderFeatures::ENABLE_CLIPPING;
        let mut pls = PLSImplWebGL::default();

        execution.withCurrent(|| {
            pls.activatePixelLocalStorage(&mut context, &desc);
        });

        let alpha = 128.0f32 * (1.0 / 255.0);
        assert_eq!(
            trace.borrow().commands,
            [
                GLCommand::BindFramebuffer(GL_DRAW_FRAMEBUFFER, 901),
                GLCommand::FramebufferTexturePixelLocalStorageANGLE {
                    plane: COLOR_PLANE_IDX,
                    backing_texture: 77,
                    level: 0,
                    layer: 0,
                    usage: GL_NONE,
                },
                GLCommand::FramebufferTexturePixelLocalStorageANGLE {
                    plane: COVERAGE_PLANE_IDX,
                    backing_texture: 801,
                    level: 0,
                    layer: 0,
                    usage: GL_NONE,
                },
                GLCommand::FramebufferTexturePixelLocalStorageANGLE {
                    plane: CLIP_PLANE_IDX,
                    backing_texture: 801,
                    level: 0,
                    layer: 1,
                    usage: GL_NONE,
                },
                GLCommand::FramebufferTexturePixelLocalStorageANGLE {
                    plane: SCRATCH_COLOR_PLANE_IDX,
                    backing_texture: 803,
                    level: 0,
                    layer: 0,
                    usage: GL_NONE,
                },
                GLCommand::FramebufferPixelLocalClearValuefvANGLE {
                    plane: COLOR_PLANE_IDX,
                    value: [
                        64.0 * (1.0 / 255.0) * alpha,
                        32.0 * (1.0 / 255.0) * alpha,
                        16.0 * (1.0 / 255.0) * alpha,
                        alpha,
                    ],
                },
                GLCommand::BeginPixelLocalStorageANGLE {
                    load_ops: vec![
                        GL_LOAD_OP_CLEAR_ANGLE,
                        GL_LOAD_OP_ZERO_ANGLE,
                        GL_DONT_CARE,
                        GL_LOAD_OP_ZERO_ANGLE,
                    ],
                },
            ]
        );

        trace.borrow_mut().commands.clear();
        execution.withCurrent(|| {
            pls.deactivatePixelLocalStorage(&mut context, &desc);
        });
        assert_eq!(
            trace.borrow().commands,
            [GLCommand::EndPixelLocalStorageANGLE {
                store_ops: vec![
                    GL_STORE_OP_STORE_ANGLE,
                    GL_DONT_CARE,
                    GL_DONT_CARE,
                    GL_DONT_CARE,
                ],
            }]
        );

        drop(target);
        drop(context);
        domain.shutdown();
    }

    #[test]
    fn framebuffer_preserve_blits_before_begin_and_end_before_copy_back() {
        let (domain, trace, _, _, _) = providerFixture();
        let mut context = newComponent097TestContextOwner(capabilities(), domain.clone());
        trace.borrow_mut().commands.clear();

        let execution = domain.stamp();
        let mut target = FramebufferRenderTargetGL::new(8, 6, 77, 1, execution.clone());
        target.m_offscreenTargetTexture.0.m_id = 701;
        target.m_textureRenderTarget.m_externalTextureID = 701;
        target.m_textureRenderTarget.m_framebufferID.0.m_id = 902;
        target.m_textureRenderTarget.m_headlessFramebuffer.0.m_id = 901;
        target.m_textureRenderTarget.m_framebufferTargetAttachmentDirty = false;
        target.m_textureRenderTarget.m_webglPLSBackingR32UI.0.m_id = 801;
        target
            .m_textureRenderTarget
            .m_webglPLSBackingR32UIFallback
            .0
            .m_id = 802;
        target.m_textureRenderTarget.m_webglPLSBackingRGBA8.0.m_id = 803;
        target.m_textureRenderTarget.m_webglPLSBindingsDirty = true;
        let renderTarget = NonNull::from(&mut *target.base.base);
        let mut desc = flushDescriptor(renderTarget);
        desc.colorLoadAction = LoadAction::preserveRenderTarget;
        let mut pls = PLSImplWebGL::default();

        execution.withCurrent(|| {
            context.state().borrow_mut().setPipelineState(
                &gpu::COLOR_ONLY_PIPELINE_STATE,
                ScissorAction::disable,
            );
        });
        trace.borrow_mut().commands.clear();

        execution.withCurrent(|| {
            pls.activatePixelLocalStorage(&mut context, &desc);
        });
        assert_eq!(
            trace.borrow().commands,
            [
                GLCommand::BindFramebuffer(GL_READ_FRAMEBUFFER, 77),
                GLCommand::BindFramebuffer(GL_DRAW_FRAMEBUFFER, 902),
                GLCommand::BlitFramebuffer(
                    [1, 1, 7, 5, 1, 1, 7, 5],
                    GL_COLOR_BUFFER_BIT,
                    GL_NEAREST,
                ),
                GLCommand::BindFramebuffer(GL_DRAW_FRAMEBUFFER, 901),
                GLCommand::FramebufferTexturePixelLocalStorageANGLE {
                    plane: COLOR_PLANE_IDX,
                    backing_texture: 701,
                    level: 0,
                    layer: 0,
                    usage: GL_NONE,
                },
                GLCommand::FramebufferTexturePixelLocalStorageANGLE {
                    plane: COVERAGE_PLANE_IDX,
                    backing_texture: 801,
                    level: 0,
                    layer: 0,
                    usage: GL_NONE,
                },
                GLCommand::FramebufferTexturePixelLocalStorageANGLE {
                    plane: CLIP_PLANE_IDX,
                    backing_texture: 801,
                    level: 0,
                    layer: 1,
                    usage: GL_NONE,
                },
                GLCommand::FramebufferTexturePixelLocalStorageANGLE {
                    plane: SCRATCH_COLOR_PLANE_IDX,
                    backing_texture: 803,
                    level: 0,
                    layer: 0,
                    usage: GL_NONE,
                },
                GLCommand::BeginPixelLocalStorageANGLE {
                    load_ops: vec![
                        GL_LOAD_OP_LOAD_ANGLE,
                        GL_DONT_CARE,
                        GL_DONT_CARE,
                        GL_LOAD_OP_ZERO_ANGLE,
                    ],
                },
            ]
        );

        trace.borrow_mut().commands.clear();
        execution.withCurrent(|| {
            pls.deactivatePixelLocalStorage(&mut context, &desc);
        });
        assert_eq!(
            trace.borrow().commands,
            [
                GLCommand::EndPixelLocalStorageANGLE {
                    store_ops: vec![
                        GL_STORE_OP_STORE_ANGLE,
                        GL_DONT_CARE,
                        GL_DONT_CARE,
                        GL_DONT_CARE,
                    ],
                },
                GLCommand::BindFramebuffer(GL_READ_FRAMEBUFFER, 902),
                GLCommand::BindFramebuffer(GL_DRAW_FRAMEBUFFER, 77),
                GLCommand::BlitFramebuffer(
                    [1, 1, 7, 5, 1, 1, 7, 5],
                    GL_COLOR_BUFFER_BIT,
                    GL_NEAREST,
                ),
            ]
        );

        drop(target);
        drop(context);
        domain.shutdown();
    }
}
