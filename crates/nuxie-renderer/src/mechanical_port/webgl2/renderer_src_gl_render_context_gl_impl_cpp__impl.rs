//! Complete mechanical implementation translation of
//! `renderer/src/gl/render_context_gl_impl.cpp` for the frozen
//! `RIVE_WEBGL + RIVE_CANVAS + WITH_RIVE_TOOLS` profile.

#![allow(non_snake_case, non_upper_case_globals)]

use super::gl_state_decl::{GLState, ScissorAction};
use super::gl_utils_decl::{self as glutils, Buffer, Framebuffer, Program, Shader, Texture as GLTexture, VAO};
use super::gles3_decl::*;
use super::render_buffer_gl_impl_decl::RenderBufferGLImpl;
use super::render_context_gl_decl::*;
use super::render_target_gl_decl::{
    FramebufferRenderTargetGL, MSAAResolveAction, RenderTargetGL, RenderTargetGLApi,
    TextureRenderTargetGL, FRAMEBUFFER_RENDER_TARGET_GL_LITE_RTTI_TYPE_ID,
    TEXTURE_RENDER_TARGET_GL_LITE_RTTI_TYPE_ID,
};
use crate::mechanical_port::source::include::rive::gpu_texture_format_hpp::GPUTextureFormat;
use crate::mechanical_port::source::include::rive::refcnt_hpp::{make_rcp, rcp, static_rcp_cast, RefCntTarget};
use crate::mechanical_port::source::include::rive::renderer_hpp::{
    RenderBuffer, RenderBufferFlags, RenderBufferType, RenderPathContract, RendererContract,
};
use crate::mechanical_port::source::include::utils::lite_rtti_hpp::{
    LiteRttiBase, LiteRttiTypeId,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::buffer_ring_hpp::{
    BufferRing, BufferRingContract,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp as gpu;
use crate::mechanical_port::source::renderer::include::rive::renderer::render_canvas_hpp::RenderCanvas;
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_helper_impl_hpp::{
    RenderContextHelperBackendContract, RenderContextHelperBufferFactoryContract,
    RenderContextHelperImpl, RenderContextHelperImplAccess,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::{
    FlushResources, FrameDescriptor, RenderContext, RenderContextContract,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_renderer_hpp::RiveRenderer;
use crate::mechanical_port::source::renderer::src::rive_render_paint_hpp::RiveRenderPaint;
use crate::mechanical_port::source::renderer::src::rive_render_path_hpp::RiveRenderPath;
use crate::mechanical_port::source::renderer::src::gpu_cpp::{
    StorageTextureBufferSize, StorageTextureSize,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_impl_hpp::RenderContextImpl;
use crate::mechanical_port::source::renderer::include::rive::renderer::render_target_hpp::RenderTarget;
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_image_hpp::RiveRenderImage;
use crate::mechanical_port::source::renderer::include::rive::renderer::texture_hpp::Texture as RiveTexture;
use core::ffi::c_void;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::mem::ManuallyDrop;
use std::rc::Rc;

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_gl_render_context_gl_impl.cpp");
const _: [(); 153_801] = [(); PINNED_SOURCE.len()];

// Exact host-side bindings from shaders/constants.glsl.
const FLUSH_UNIFORM_BUFFER_IDX: GLuint = 0;
const PATH_BUFFER_IDX: GLuint = 2;
const PAINT_BUFFER_IDX: GLuint = 3;
const PAINT_AUX_BUFFER_IDX: GLuint = 4;
const CONTOUR_BUFFER_IDX: GLuint = 5;
const TESS_VERTEX_TEXTURE_IDX: GLuint = 7;
const GRAD_TEXTURE_IDX: GLuint = 8;
const GAUSSIAN_INTEGRAL_TEXTURE_IDX: GLuint = 9;
const FEATHER_ATLAS_TEXTURE_IDX: GLuint = 10;
const IMAGE_TEXTURE_IDX: GLuint = 11;
const DST_COLOR_TEXTURE_IDX: GLuint = 12;
const DEFAULT_BINDINGS_SET_SIZE: GLuint = 13;
const COLOR_PLANE_IDX: usize = 0;
const CLIP_PLANE_IDX: usize = 1;
const COVERAGE_PLANE_IDX: usize = 3;
const IMAGE_FIRST_ATTRIB_IDX: GLuint = 2;
const IMAGE_VIEW_MATRIX_ATTRIB_IDX: GLuint = 2;
const IMAGE_CLIP_RECT_INVERSE_MATRIX_ATTRIB_IDX: GLuint = 3;
const IMAGE_TRANSLATES_ATTRIB_IDX: GLuint = 4;
const IMAGE_PACKED_ATTRIBS_IDX: GLuint = 5;
const IMAGE_LAST_ATTRIB_IDX: GLuint = 5;

// Exact export substitutions emitted by the frozen shader minifier.
const GLSL_ATLAS_FEATHERED_FILL: &str = "NC";
const GLSL_ATLAS_FEATHERED_STROKE: &str = "TC";
const GLSL_ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE: &str = "VD";
const GLSL_ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH: &str = "TD";
const GLSL_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE: &str =
    "EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE";
const GLSL_ATLAS_RENDER_TARGET_R8_PLS_EXT: &str = "UD";
const GLSL_ATLAS_RENDER_TARGET_RGBA8_UNORM: &str = "TE";
const GLSL_BORROWED_COVERAGE_PASS: &str = "EC";
const GLSL_CLEAR_COVERAGE: &str = "AE";
const GLSL_CLOCKWISE_FILL: &str = "BE";
const GLSL_COALESCED_PLS_RESOLVE_AND_TRANSFER: &str = "ZC";
const GLSL_DISABLE_SHADER_STORAGE_BUFFERS: &str = "JF";
const GLSL_DRAW_IMAGE: &str = "HE";
const GLSL_DRAW_IMAGE_MESH: &str = "OB";
const GLSL_DRAW_IMAGE_RECT: &str = "JD";
const GLSL_DRAW_INTERIOR_TRIANGLES: &str = "EB";
const GLSL_DRAW_PATH: &str = "ID";
const GLSL_DRAW_RENDER_TARGET_UPDATE_BOUNDS: &str = "AF";
const GLSL_ENABLE_FEATHER: &str = "HB";
const GLSL_ENABLE_INSTANCE_INDEX: &str = "NE";
const GLSL_ENABLE_KHR_BLEND: &str = "GE";
const GLSL_FEATHER_ATLAS_BLIT: &str = "FB";
const GLSL_FIXED_FUNCTION_COLOR_OUTPUT: &str = "Q";
const GLSL_FRAMEBUFFER_BOTTOM_UP: &str = "ZF";
const GLSL_OPTIONALLY_FLAT: &str = "MB";
const GLSL_RENDER_MODE_MSAA: &str = "CB";
const GLSL_RESOLVE_PLS: &str = "QC";
const GLSL_USING_PLS_STORAGE_TEXTURES: &str = "KF";
const GLSL_FlushUniforms: &str = "CC";
const GLSL_atlasRenderTexture: &str = "WE";
const GLSL_contourBuffer: &str = "ED";
const GLSL_dstColorTexture: &str = "SD";
const GLSL_featherAtlasTexture: &str = "BD";
const GLSL_gaussianIntegralTexture: &str = "XC";
const GLSL_gradTexture: &str = "KD";
const GLSL_imageTexture: &str = "IC";
const GLSL_paintAuxBuffer: &str = "RB";
const GLSL_paintBuffer: &str = "AD";
const GLSL_pathBuffer: &str = "PB";
const GLSL_sourceTexture: &str = "JC";
const GLSL_tessVertexTexture: &str = "LC";

const GLSL_GLSL: &str = include_str!("../webgpu/source/generated_glsl/glsl.minified.glsl");
const GLSL_CONSTANTS: &str =
    include_str!("../webgpu/source/generated_glsl/constants.minified.glsl");
const GLSL_FLUSH_UNIFORMS: &str =
    include_str!("../webgpu/source/generated_glsl/flush_uniforms.minified.glsl");
const GLSL_COMMON: &str = include_str!("../webgpu/source/generated_glsl/common.minified.glsl");
const GLSL_COLOR_RAMP: &str =
    include_str!("../webgpu/source/generated_glsl/color_ramp.minified.glsl");
const GLSL_BEZIER_UTILS: &str =
    include_str!("../webgpu/source/generated_glsl/bezier_utils.minified.glsl");
const GLSL_TESSELLATE: &str =
    include_str!("../webgpu/source/generated_glsl/tessellate.minified.glsl");
const GLSL_RENDER_ATLAS: &str =
    include_str!("../webgpu/source/generated_glsl/render_atlas.minified.glsl");
const GLSL_ADVANCED_BLEND: &str =
    include_str!("../webgpu/source/generated_glsl/advanced_blend.minified.glsl");
const GLSL_DRAW_PATH_COMMON: &str =
    include_str!("../webgpu/source/generated_glsl/draw_path_common.minified.glsl");
const GLSL_DRAW_PATH_VERT: &str =
    include_str!("../webgpu/source/generated_glsl/draw_path.minified.vert");
const GLSL_DRAW_RASTER_ORDER_PATH_FRAG: &str =
    include_str!("../webgpu/source/generated_glsl/draw_raster_order_path.minified.frag");
const GLSL_DRAW_CLOCKWISE_PATH_FRAG: &str =
    include_str!("../webgpu/source/generated_glsl/draw_clockwise_path.minified.frag");
const GLSL_DRAW_CLOCKWISE_CLIP_FRAG: &str =
    include_str!("../webgpu/source/generated_glsl/draw_clockwise_clip.minified.frag");
const GLSL_DRAW_IMAGE_MESH_VERT: &str =
    include_str!("../webgpu/source/generated_glsl/draw_image_mesh.minified.vert");
const GLSL_DRAW_MESH_FRAG: &str =
    include_str!("../webgpu/source/generated_glsl/draw_mesh.minified.frag");
const GLSL_ATOMIC_DRAW: &str = "";

const GLSL_RESOLVE_ATLAS: &str = r#"#ifdef DB
y1(SF,e0,F,B,r){g U;U.x=(B!=2)?-1.:3.;U.y=(B!=1)?-1.:3.;U.zw=d(.0,1.);z1(U);}
#endif
#ifdef GB
e ivec2 Vd(){return ivec2(floor(gl_FragCoord));}
#ifdef TD
layout(location=0)inout G p0;layout(location=1)out i i4;void main(){i4.x=uintBitsToFloat(p0.x);}
#elif defined(UD)
#ifdef AE
__pixel_local_outEXT R1{layout(r32f)float p0;};
#else
__pixel_local_inEXT R1{layout(r32f)float p0;};layout(location=0)out i i4;
#endif
void main(){
#ifdef AE
p0=.0;
#else
i4.x=p0;
#endif
}
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui)uniform highp upixelLocalANGLE p0;layout(location=0)out i i4;void main(){i4.x=uintBitsToFloat(pixelLocalLoadANGLE(p0).x);}
#elif defined(VD)
layout(binding=0,r32i)uniform highp coherent iimage2D S8;layout(location=0)out i i4;void main(){i4.x=float(imageLoad(S8,Vd()).x)*(1./Xc);}
#elif defined(TE)
X2(a3,0,WE);layout(location=0)out i i4;void main(){i P=q1(WE,Vd());i4.x=(P.x-P.y)*pa+(P.z-P.w)*255.;}
#endif
#endif
"#;

const GLSL_BLIT_TEXTURE_AS_DRAW: &str = r#"k2
#ifdef CD
J0 W(0,d,X1);
#endif
f2
#ifdef DB
R3 S3 y4 z4 g1(e0)h1 y1(DF,e0,F,B,r){d m2;m2.x=(B&1)==0?-1.:1.;m2.y=(B&2)==0?-1.:1.;
#ifdef CD
V(X1,d);X1.x=m2.x*.5+.5;X1.y=m2.y*-.5+.5;a0(X1);
#endif
g U=g(m2,0,1);z1(U);}
#endif
#ifdef GB
B3
#ifdef ND
rf(Y4,T3,JC);
#else
X2(Y4,T3,JC);
#endif
C3
#ifdef CD
Z4 U3(vf)a5
#endif
Y2(i,KE){i f8;
#ifdef CD
A(X1,d);f8=N6(JC,vf,X1,.0);
#elif defined(ND)
f8=(g8(JC,0,X(floor(Y.xy)))+g8(JC,1,X(floor(Y.xy)))+g8(JC,2,X(floor(Y.xy)))+g8(JC,3,X(floor(Y.xy))))*0.25;
#else
f8=q1(JC,X(floor(Y.xy)));
#endif
G2(f8);}
#endif
"#;

const GLSL_STENCIL_DRAW: &str = r#"#ifdef DB
g1(e0)L(0,K3,KB);h1 R3 S3 y4 z4 y1(VF,e0,F,B,r){M(B,F,KB,K3);g U=J3(KB.xy);uint ha=floatBitsToUint(KB.z)&0xffffu;U.z=ga(ha);z1(U);}
#endif
#ifdef GB
B3 C3 Y2(i,KE){G2(C0(.0));}
#endif
"#;

const GLSL_DRAW_MSAA_OBJECT_FRAG: &str = r#"#ifdef GB
#ifdef OB
B3 X2(Y4,T3,IC);
#ifdef AB
d7(SD);
#endif
C3 Z4 U3(R5)a5
#endif
Y2(i,JB){
#ifdef OB
A(D5,d);A(H1,c);
#ifdef AB
A(A1,N);
#endif
#else
A(f1,g);
#ifdef FB
A(C2,d);
#endif
#ifdef AB
A(e2,c);
#endif
#endif
#ifdef OB
i j=r7(IC,R5,D5,n.Bd)*H1;
#else
c o=
#ifdef FB
clamp(n2(BD,M9,C2,.0).x,G0(.0),G0(1.));
#else
1.;
#endif
i j=G7(f1,o S2);
#endif
#if defined(AB)&&!defined(Q)
#ifdef OB
j.xyz=B6(j);N Q3=A1;
#else
N Q3=W5(e2);
#endif
i K1=N8(SD);j.xyz=P4(j.xyz,K1,Q3);j.xyz*=j.w;
#endif
#ifdef BC
if(BC){j=k3(j);}
#endif
j.xyz=E2(j.xyz,j.w,Y.xy,n.y3,n.z3);G2(j);}
#endif
"#;

fn isTessellationDraw(drawType: gpu::DrawType) -> bool {
    matches!(
        drawType,
        gpu::DrawType::midpointFanPatches
            | gpu::DrawType::midpointFanCenterAAPatches
            | gpu::DrawType::outerCurvePatches
            | gpu::DrawType::msaaStrokes
            | gpu::DrawType::msaaMidpointFanBorrowedCoverage
            | gpu::DrawType::msaaDynamicMidpointFans
            | gpu::DrawType::msaaMidpointFans
            | gpu::DrawType::msaaMidpointFanStencilReset
            | gpu::DrawType::msaaMidpointFanPathsStencil
            | gpu::DrawType::msaaMidpointFanPathsCover
            | gpu::DrawType::msaaOuterCubics
    )
}

fn selectFeatherAtlasRenderType(
    capabilities: &GLCapabilities,
    desired: FeatherAtlasRenderType,
) -> FeatherAtlasRenderType {
    if desired <= FeatherAtlasRenderType::r16f && capabilities.EXT_color_buffer_half_float() {
        return FeatherAtlasRenderType::r16f;
    }
    if desired <= FeatherAtlasRenderType::r32f
        && capabilities.EXT_color_buffer_float()
        && capabilities.EXT_float_blend()
    {
        return FeatherAtlasRenderType::r32f;
    }
    if desired <= FeatherAtlasRenderType::r32uiFramebufferFetch
        && capabilities.EXT_shader_framebuffer_fetch()
    {
        return FeatherAtlasRenderType::r32uiFramebufferFetch;
    }
    // RIVE_WEBGL admits ANGLE PLS, never native EXT PLS or image atomics.
    if desired <= FeatherAtlasRenderType::r32uiPixelLocalStorageANGLE
        && capabilities.ANGLE_shader_pixel_local_storage_coherent()
    {
        return FeatherAtlasRenderType::r32uiPixelLocalStorageANGLE;
    }
    FeatherAtlasRenderType::rgba8
}

fn needsFeatherAtlasResolveDraw(renderType: FeatherAtlasRenderType) -> bool {
    !matches!(
        renderType,
        FeatherAtlasRenderType::r16f | FeatherAtlasRenderType::r32f
    )
}

unsafe fn textureGLNativeHandle(base: *const RiveTexture) -> *mut c_void {
    let texture = unsafe { &*base.cast::<TextureGLImpl>() };
    let execution = (&*texture.rust_execution).clone();
    execution.withCurrent(|| texture.m_texture.id() as usize as *mut c_void)
}

impl TextureGLImpl {
    pub(crate) fn new(
        width: u32,
        height: u32,
        textureID: GLuint,
        execution: GLExecutionStamp,
    ) -> Self {
        let mut base = RiveTexture::new(width, height);
        base.destroy_complete = |base| unsafe { drop(Box::from_raw(base.cast::<TextureGLImpl>())) };
        base.setNativeHandleDispatch(textureGLNativeHandle);
        base.install_owner_thread_execution(
            execution.domain().ownerThreadFinalReleaseRoute(),
            execution.domain().key(),
            execution.generation(),
        );
        Self {
            base: ManuallyDrop::new(base),
            m_texture: ManuallyDrop::new(GLTexture::Adopt(textureID)),
            rust_execution: ManuallyDrop::new(execution),
        }
    }
}

unsafe impl RefCntTarget for TextureGLImpl {
    fn r#ref(&self) {
        self.base.r#ref();
    }
    unsafe fn unref(&self) {
        unsafe { self.base.unref() }
    }
    unsafe fn onRefCntReachedZero(ptr: *const Self) {
        unsafe { drop(Box::from_raw(ptr.cast_mut())) }
    }
}

impl Drop for TextureGLImpl {
    fn drop(&mut self) {
        let execution = (&*self.rust_execution).clone();
        if execution
            .withDeleteCurrent(|| unsafe { ManuallyDrop::drop(&mut self.m_texture) })
            .is_none()
        {
            self.m_texture.0.m_id = 0;
            unsafe { ManuallyDrop::drop(&mut self.m_texture) };
        }
        unsafe {
            ManuallyDrop::drop(&mut self.base);
            ManuallyDrop::drop(&mut self.rust_execution);
        }
    }
}

impl CanvasSourceTextureGLImpl {
    fn new(
        width: u32,
        height: u32,
        textureID: GLuint,
        execution: GLExecutionStamp,
        owner: *mut RenderContextGLImpl,
        canvasRegistry: WeakCanvasMirrorRegistry,
    ) -> Self {
        let mut base = TextureGLImpl::new(width, height, textureID, execution);
        base.base.destroy_complete =
            |base| unsafe { drop(Box::from_raw(base.cast::<CanvasSourceTextureGLImpl>())) };
        Self {
            base: ManuallyDrop::new(base),
            m_owner: owner,
            m_glID: textureID,
            rust_canvas_registry: canvasRegistry,
        }
    }
}

unsafe impl RefCntTarget for CanvasSourceTextureGLImpl {
    fn r#ref(&self) {
        self.base.base.r#ref();
    }
    unsafe fn unref(&self) {
        unsafe { self.base.base.unref() }
    }
    unsafe fn onRefCntReachedZero(ptr: *const Self) {
        unsafe { drop(Box::from_raw(ptr.cast_mut())) }
    }
}

impl Drop for CanvasSourceTextureGLImpl {
    fn drop(&mut self) {
        let entry = self.rust_canvas_registry.upgrade().and_then(|registry| {
            let entry = registry.borrow_mut().remove(&self.m_glID);
            entry
        });
        if let Some(entry) = entry {
            let execution = (&*self.base.rust_execution).clone();
            let _ = execution.withDeleteCurrent(|| {
                if entry.readFBO != 0 {
                    recordGLCommand(GLCommand::DeleteFramebuffer(entry.readFBO));
                }
                if entry.drawFBO != 0 {
                    recordGLCommand(GLCommand::DeleteFramebuffer(entry.drawFBO));
                }
            });
        }
        unsafe { ManuallyDrop::drop(&mut self.base) };
    }
}

impl CanvasMirrorTextureGLImpl {
    fn new(
        width: u32,
        height: u32,
        textureID: GLuint,
        execution: GLExecutionStamp,
        owner: *mut RenderContextGLImpl,
        sourceTexID: GLuint,
        canvasRegistry: WeakCanvasMirrorRegistry,
    ) -> Self {
        let mut base = TextureGLImpl::new(width, height, textureID, execution);
        base.base.destroy_complete =
            |base| unsafe { drop(Box::from_raw(base.cast::<CanvasMirrorTextureGLImpl>())) };
        Self {
            base: ManuallyDrop::new(base),
            m_owner: owner,
            m_sourceTexID: sourceTexID,
            rust_canvas_registry: canvasRegistry,
        }
    }
}

unsafe impl RefCntTarget for CanvasMirrorTextureGLImpl {
    fn r#ref(&self) {
        self.base.base.r#ref();
    }
    unsafe fn unref(&self) {
        unsafe { self.base.base.unref() }
    }
    unsafe fn onRefCntReachedZero(ptr: *const Self) {
        unsafe { drop(Box::from_raw(ptr.cast_mut())) }
    }
}

impl Drop for CanvasMirrorTextureGLImpl {
    fn drop(&mut self) {
        let framebuffers = self.rust_canvas_registry.upgrade().and_then(|registry| {
            let mut registry = registry.borrow_mut();
            let entry = registry.get_mut(&self.m_sourceTexID)?;
            let framebuffers = (entry.readFBO, entry.drawFBO);
            entry.readFBO = 0;
            entry.drawFBO = 0;
            entry.mirrorTex = 0;
            entry.hasMirror = false;
            Some(framebuffers)
        });
        if let Some((readFBO, drawFBO)) = framebuffers {
            let execution = (&*self.base.rust_execution).clone();
            let _ = execution.withDeleteCurrent(|| {
                if readFBO != 0 {
                    recordGLCommand(GLCommand::DeleteFramebuffer(readFBO));
                }
                if drawFBO != 0 {
                    recordGLCommand(GLCommand::DeleteFramebuffer(drawFBO));
                }
            });
        }
        unsafe { ManuallyDrop::drop(&mut self.base) };
    }
}

impl BufferRingGLImpl {
    fn new(
        target: GLenum,
        capacityInBytes: usize,
        state: GLStateOwner,
        execution: GLExecutionStamp,
    ) -> Self {
        let mut owner = Self {
            base: ManuallyDrop::new(BufferRing::new(capacityInBytes)),
            m_target: target,
            m_bufferID: 0,
            m_state: ManuallyDrop::new(state),
            rust_execution: ManuallyDrop::new(execution.clone()),
        };
        execution.withCurrent(|| {
            owner.m_bufferID = generateGLObject(GLObjectKind::Buffer);
            owner
                .m_state
                .borrow_mut()
                .bindBuffer(target, owner.m_bufferID);
            recordGLCommand(GLCommand::BufferData {
                target,
                size: capacityInBytes,
                data: None,
                usage: GL_DYNAMIC_DRAW,
            });
        });
        owner
    }

    fn bufferID(&self) -> GLuint {
        self.m_bufferID
    }
}

impl BufferRingContract for BufferRingGLImpl {
    fn bufferRing(&self) -> &BufferRing {
        &self.base
    }
    fn bufferRingMut(&mut self) -> &mut BufferRing {
        &mut self.base
    }
    fn onMapBuffer(&mut self, _bufferIdx: i32, _mapSizeInBytes: usize) -> *mut c_void {
        let execution = (&*self.rust_execution).clone();
        execution.withCurrent(|| self.base.shadowBuffer().cast())
    }
    fn onUnmapAndSubmitBuffer(&mut self, _bufferIdx: i32, mapSizeInBytes: usize) {
        let data = unsafe {
            std::slice::from_raw_parts(self.base.shadowBuffer(), mapSizeInBytes).to_vec()
        };
        let execution = (&*self.rust_execution).clone();
        execution.withCurrent(|| {
            self.m_state
                .borrow_mut()
                .bindBuffer(self.m_target, self.m_bufferID);
            recordGLCommand(GLCommand::BufferSubData {
                target: self.m_target,
                offset: 0,
                data,
            });
        });
    }
}

impl Drop for BufferRingGLImpl {
    fn drop(&mut self) {
        let execution = (&*self.rust_execution).clone();
        let _ =
            execution.withDeleteCurrent(|| self.m_state.borrow_mut().deleteBuffer(self.m_bufferID));
        unsafe {
            ManuallyDrop::drop(&mut self.m_state);
            ManuallyDrop::drop(&mut self.base);
            ManuallyDrop::drop(&mut self.rust_execution);
        }
    }
}

impl StorageBufferRingGLImpl {
    fn new(
        capacityInBytes: usize,
        structure: gpu::StorageBufferStructure,
        state: GLStateOwner,
        execution: GLExecutionStamp,
    ) -> Self {
        Self {
            base: ManuallyDrop::new(BufferRingGLImpl::new(
                GL_SHADER_STORAGE_BUFFER,
                capacityInBytes,
                state,
                execution,
            )),
            m_bufferStructure: structure,
        }
    }

    fn bindToRenderContext(&self, bindingIdx: GLuint, bindingSize: usize, offset: usize) {
        let execution = (&*self.base.rust_execution).clone();
        execution.withCurrent(|| {
            recordGLCommand(GLCommand::BindBufferRange {
                target: GL_SHADER_STORAGE_BUFFER,
                index: bindingIdx,
                buffer: self.base.bufferID(),
                offset: u32::try_from(offset).expect("storage offset fits WebGL"),
                size: u32::try_from(bindingSize).expect("storage binding fits WebGL"),
            });
        });
    }
}

impl BufferRingContract for StorageBufferRingGLImpl {
    fn bufferRing(&self) -> &BufferRing {
        &self.base.base
    }
    fn bufferRingMut(&mut self) -> &mut BufferRing {
        &mut self.base.base
    }
    fn onMapBuffer(&mut self, bufferIdx: i32, mapSizeInBytes: usize) -> *mut c_void {
        self.base.onMapBuffer(bufferIdx, mapSizeInBytes)
    }
    fn onUnmapAndSubmitBuffer(&mut self, bufferIdx: i32, mapSizeInBytes: usize) {
        self.base.onUnmapAndSubmitBuffer(bufferIdx, mapSizeInBytes)
    }
}

impl Drop for StorageBufferRingGLImpl {
    fn drop(&mut self) {
        unsafe { ManuallyDrop::drop(&mut self.base) }
    }
}

fn storageTextureInternalFormat(structure: gpu::StorageBufferStructure) -> GLenum {
    match structure {
        gpu::StorageBufferStructure::uint32x4 => GL_RGBA32UI,
        gpu::StorageBufferStructure::uint32x2 => GL_RG32UI,
        gpu::StorageBufferStructure::float32x4 => GL_RGBA32F,
    }
}

fn storageTextureFormat(structure: gpu::StorageBufferStructure) -> GLenum {
    match structure {
        gpu::StorageBufferStructure::uint32x4 => GL_RGBA_INTEGER,
        gpu::StorageBufferStructure::uint32x2 => GL_RG_INTEGER,
        gpu::StorageBufferStructure::float32x4 => GL_RGBA,
    }
}

fn storageTextureType(structure: gpu::StorageBufferStructure) -> GLenum {
    match structure {
        gpu::StorageBufferStructure::uint32x4 | gpu::StorageBufferStructure::uint32x2 => {
            GL_UNSIGNED_INT
        }
        gpu::StorageBufferStructure::float32x4 => GL_FLOAT,
    }
}

impl TexelBufferRingWebGL {
    fn new(
        capacityInBytes: usize,
        structure: gpu::StorageBufferStructure,
        state: GLStateOwner,
        execution: GLExecutionStamp,
    ) -> Self {
        let storageCapacity = StorageTextureBufferSize(capacityInBytes, structure);
        let mut owner = Self {
            base: ManuallyDrop::new(BufferRing::new(storageCapacity)),
            m_bufferStructure: structure,
            m_state: ManuallyDrop::new(state),
            m_textureID: 0,
            rust_execution: ManuallyDrop::new(execution.clone()),
        };
        execution.withCurrent(|| {
            let (width, height) = StorageTextureSize(capacityInBytes, structure);
            owner.m_textureID = generateGLObject(GLObjectKind::Texture);
            recordGLCommand(GLCommand::ActiveTexture(GL_TEXTURE0));
            recordGLCommand(GLCommand::BindTexture(GL_TEXTURE_2D, owner.m_textureID));
            recordGLCommand(GLCommand::TexStorage2D {
                target: GL_TEXTURE_2D,
                levels: 1,
                internal_format: storageTextureInternalFormat(structure),
                width,
                height,
            });
            glutils::SetTexture2DSamplingParams(GL_NEAREST, GL_NEAREST);
            recordGLCommand(GLCommand::BindTexture(GL_TEXTURE_2D, 0));
        });
        owner
    }

    fn bindToRenderContext(&self, bindingIdx: GLuint, bindingSize: usize, offset: usize) {
        let (width, height) = StorageTextureSize(bindingSize, self.m_bufferStructure);
        let bytes = unsafe {
            std::slice::from_raw_parts(self.base.shadowBuffer().add(offset), bindingSize).to_vec()
        };
        let execution = (&*self.rust_execution).clone();
        execution.withCurrent(|| {
            recordGLCommand(GLCommand::ActiveTexture(GL_TEXTURE0 + bindingIdx));
            recordGLCommand(GLCommand::BindTexture(GL_TEXTURE_2D, self.m_textureID));
            recordGLCommand(GLCommand::TexSubImage2D {
                target: GL_TEXTURE_2D,
                level: 0,
                x: 0,
                y: 0,
                width,
                height,
                format: storageTextureFormat(self.m_bufferStructure),
                type_: storageTextureType(self.m_bufferStructure),
                data: bytes,
            });
        });
    }
}

impl BufferRingContract for TexelBufferRingWebGL {
    fn bufferRing(&self) -> &BufferRing {
        &self.base
    }
    fn bufferRingMut(&mut self) -> &mut BufferRing {
        &mut self.base
    }
    fn onMapBuffer(&mut self, _bufferIdx: i32, _mapSizeInBytes: usize) -> *mut c_void {
        let execution = (&*self.rust_execution).clone();
        execution.withCurrent(|| self.base.shadowBuffer().cast())
    }
    fn onUnmapAndSubmitBuffer(&mut self, _bufferIdx: i32, _mapSizeInBytes: usize) {}
}

impl Drop for TexelBufferRingWebGL {
    fn drop(&mut self) {
        let execution = (&*self.rust_execution).clone();
        let _ = execution
            .withDeleteCurrent(|| recordGLCommand(GLCommand::DeleteTexture(self.m_textureID)));
        unsafe {
            ManuallyDrop::drop(&mut self.m_state);
            ManuallyDrop::drop(&mut self.base);
            ManuallyDrop::drop(&mut self.rust_execution);
        }
    }
}

fn bytesOfSlice<T: Copy>(values: &[T]) -> Vec<u8> {
    unsafe {
        std::slice::from_raw_parts(values.as_ptr().cast::<u8>(), std::mem::size_of_val(values))
            .to_vec()
    }
}

fn bindUniformBlock(execution: &GLExecutionStamp, program: GLuint) {
    let block = execution
        .domain()
        .uniformBlockIndex(program, GLSL_FlushUniforms.as_bytes());
    recordGLCommand(GLCommand::UniformBlockBinding {
        program,
        block_index: block,
        binding: FLUSH_UNIFORM_BUFFER_IDX,
    });
}

fn newContextOwner(
    rendererString: Vec<u8>,
    capabilities: GLCapabilities,
    plsImpl: Option<Box<dyn PixelLocalStorageImpl>>,
    mode: ShaderCompilationMode,
    executionDomain: GLExecutionDomain,
) -> Box<RenderContextGLImpl> {
    let execution = executionDomain.stamp();
    let state = Rc::new(RefCell::new(GLState::newInDomain(
        capabilities,
        executionDomain,
    )));
    let mut owner = Box::new(RenderContextGLImpl {
        base: ManuallyDrop::new(RenderContextHelperImpl::new(RenderContextImpl::default())),
        m_capabilities: capabilities,
        m_plsImpl: ManuallyDrop::new(plsImpl),
        m_colorRampProgram: ManuallyDrop::new(Program::new()),
        m_colorRampVAO: ManuallyDrop::new(VAO::new()),
        m_colorRampFBO: ManuallyDrop::new(Framebuffer::new()),
        m_gradientTexture: 0,
        m_gaussianIntegralTexture: ManuallyDrop::new(GLTexture::new()),
        m_tessellateProgram: ManuallyDrop::new(Program::new()),
        m_tessellateVAO: ManuallyDrop::new(VAO::new()),
        m_tessSpanIndexBuffer: ManuallyDrop::new(Buffer::new()),
        m_tessellateFBO: ManuallyDrop::new(Framebuffer::new()),
        m_tessVertexTexture: 0,
        m_featherAtlasRenderType: selectFeatherAtlasRenderType(
            &capabilities,
            FeatherAtlasRenderType::r16f,
        ),
        m_featherAtlasVertexShader: ManuallyDrop::new(Shader::default()),
        m_featherAtlasFillProgram: ManuallyDrop::new(FeatherAtlasProgram::default()),
        m_featherAtlasStrokeProgram: ManuallyDrop::new(FeatherAtlasProgram::default()),
        m_featherAtlasFillPipelineState: gpu::FEATHER_ATLAS_FILL_PIPELINE_STATE,
        m_featherAtlasStrokePipelineState: gpu::FEATHER_ATLAS_STROKE_PIPELINE_STATE,
        m_featherAtlasResolveVertexShader: ManuallyDrop::new(Shader::default()),
        m_featherAtlasClearProgram: ManuallyDrop::new(Program::Zero()),
        m_featherAtlasResolveProgram: ManuallyDrop::new(Program::Zero()),
        m_featherAtlasResolveVAO: ManuallyDrop::new(VAO::new()),
        m_featherAtlasRenderTexture: ManuallyDrop::new(GLTexture::Zero()),
        m_featherAtlasTexture: ManuallyDrop::new(GLTexture::Zero()),
        m_featherAtlasRenderFBO: ManuallyDrop::new(Framebuffer::Zero()),
        m_featherAtlasResolveFBO: ManuallyDrop::new(Framebuffer::Zero()),
        m_pipelineManager: ManuallyDrop::new(GLPipelineManager {
            base: ManuallyDrop::new(AsyncPipelineManagerGLState::new(mode)),
            m_context: std::ptr::null_mut(),
        }),
        m_drawVAO: ManuallyDrop::new(VAO::new()),
        m_patchVerticesBuffer: ManuallyDrop::new(Buffer::new()),
        m_patchIndicesBuffer: ManuallyDrop::new(Buffer::new()),
        m_trianglesVAO: ManuallyDrop::new(VAO::new()),
        m_imageRectVAO: ManuallyDrop::new(VAO::new()),
        m_imageRectVertexBuffer: ManuallyDrop::new(Buffer::new()),
        m_imageRectIndexBuffer: ManuallyDrop::new(Buffer::new()),
        m_imageMeshVAO: ManuallyDrop::new(VAO::new()),
        m_emptyVAO: ManuallyDrop::new(VAO::new()),
        m_blitAsDrawProgram: ManuallyDrop::new(Program::Zero()),
        m_state: ManuallyDrop::new(state),
        m_testForAdvancedBlendError: false,
        m_canvasMirrors: ManuallyDrop::new(Rc::new(RefCell::new(BTreeMap::new()))),
        rust_execution: ManuallyDrop::new(execution.clone()),
        rust_source_renderer_string: ManuallyDrop::new(rendererString),
    });
    owner.m_pipelineManager.m_context = &mut *owner;
    initializeContext(&mut owner);
    owner
}

#[cfg(test)]
pub(crate) fn newComponent097TestContextOwner(
    capabilities: GLCapabilities,
    executionDomain: GLExecutionDomain,
) -> Box<RenderContextGLImpl> {
    newContextOwner(
        b"WebGL component097 test renderer\0".to_vec(),
        capabilities,
        None,
        ShaderCompilationMode::standard,
        executionDomain,
    )
}

#[cfg(test)]
pub(crate) fn newComponent097SelectedContextOwner(
    options: ContextOptions,
    executionDomain: GLExecutionDomain,
) -> Option<Box<RenderContextGLImpl>> {
    executionDomain.withCurrent(|| {
        makeContextOwnerInCurrent(
            options,
            executionDomain.clone(),
            Some(&super::pls_impl_webgl_impl::PLS_IMPL_WEBGL_FACTORY),
        )
    })
}

fn initializeContext(context: &mut RenderContextGLImpl) {
    let execution = (&*context.rust_execution).clone();
    execution.withCurrent(|| {
        let renderer = String::from_utf8_lossy(&context.rust_source_renderer_string);
        if context.m_capabilities.isANGLESystemDriver()
            && context.m_capabilities.KHR_blend_equation_advanced()
        {
            context.m_testForAdvancedBlendError = true;
        }

        if let Some(pls) = context.m_plsImpl.as_mut() {
            pls.getSupportedInterlockModes(
                &context.m_capabilities,
                &mut context.base.base.m_platformFeatures,
            );
        }
        let features = &mut context.base.base.m_platformFeatures;
        if context.m_capabilities.KHR_blend_equation_advanced()
            || context.m_capabilities.KHR_blend_equation_advanced_coherent()
        {
            features.supportsBlendAdvancedKHR = true;
        }
        if context.m_capabilities.KHR_blend_equation_advanced_coherent() {
            features.supportsBlendAdvancedCoherentKHR = true;
        }
        if context.m_capabilities.EXT_clip_cull_distance() {
            features.supportsClipPlanes = true;
        }
        if renderer.contains("Apple") && renderer.contains("Metal") {
            features.avoidFlatVaryings = true;
        }
        if context.m_capabilities.isPowerVR() || renderer.contains("Mali-G52") {
            features.alwaysFeatherToAtlas = true;
        }
        features.clipSpaceBottomUp = true;
        features.framebufferBottomUp = true;
        features.maxTextureSize = u32::try_from(execution.domain().getInteger(GL_MAX_TEXTURE_SIZE))
            .expect("GL_MAX_TEXTURE_SIZE is nonnegative");
        features.supportsClipScissor = !(context.m_capabilities.isAdreno()
            && (600..700).contains(&context.m_capabilities.adrenoSeries)
            || context.m_capabilities.isANGLESystemDriver());
        features.supportsTextureCompressionBC = context.m_capabilities.EXT_texture_compression_s3tc()
            && context.m_capabilities.EXT_texture_compression_bptc();
        features.supportsTextureCompressionASTC =
            context.m_capabilities.KHR_texture_compression_astc_ldr();
        features.supportsTextureCompressionETC2 = context.m_capabilities.supportsETC2();

        let mut generalDefines = Vec::new();
        if !context.m_capabilities.ARB_shader_storage_buffer_object() {
            generalDefines.push(GLSL_DISABLE_SHADER_STORAGE_BUFFERS);
        }
        let colorRampSources = [
            GLSL_CONSTANTS,
            GLSL_FLUSH_UNIFORMS,
            GLSL_COMMON,
            GLSL_COLOR_RAMP,
        ];
        context.m_colorRampProgram.compileAndAttachShaderParts(
            GL_VERTEX_SHADER,
            &generalDefines,
            &colorRampSources,
            &context.m_capabilities,
        );
        context.m_colorRampProgram.compileAndAttachShaderParts(
            GL_FRAGMENT_SHADER,
            &generalDefines,
            &colorRampSources,
            &context.m_capabilities,
        );
        context.m_colorRampProgram.link();
        bindUniformBlock(&execution, context.m_colorRampProgram.id());

        context
            .m_state
            .borrow_mut()
            .bindVAO(context.m_colorRampVAO.id());
        recordGLCommand(GLCommand::EnableVertexAttribArray(0));
        recordGLCommand(GLCommand::VertexAttribDivisor(0, 1));

        recordGLCommand(GLCommand::ActiveTexture(
            GL_TEXTURE0 + GAUSSIAN_INTEGRAL_TEXTURE_IDX,
        ));
        recordGLCommand(GLCommand::BindTexture(
            GL_TEXTURE_2D,
            context.m_gaussianIntegralTexture.id(),
        ));
        recordGLCommand(GLCommand::TexStorage2D {
            target: GL_TEXTURE_2D,
            levels: 1,
            internal_format: GL_R16F,
            width: gpu::GAUSSIAN_TABLE_SIZE,
            height: 2,
        });
        context
            .m_state
            .borrow_mut()
            .bindBuffer(GL_PIXEL_UNPACK_BUFFER, 0);
        let gaussian = unsafe { &gpu::g_gaussianIntegralTableF16 };
        let inverse = unsafe { &gpu::g_inverseGaussianIntegralTableF16 };
        for (y, table) in [(0, gaussian), (1, inverse)] {
            recordGLCommand(GLCommand::TexSubImage2D {
                target: GL_TEXTURE_2D,
                level: 0,
                x: 0,
                y,
                width: gpu::GAUSSIAN_TABLE_SIZE,
                height: 1,
                format: GL_RED,
                type_: GL_HALF_FLOAT,
                data: bytesOfSlice(table),
            });
        }
        let filter = if context.m_capabilities.OES_texture_half_float_linear() {
            GL_LINEAR
        } else {
            GL_NEAREST
        };
        glutils::SetTexture2DSamplingParams(filter, filter);

        let tessellateSources = [
            GLSL_CONSTANTS,
            GLSL_FLUSH_UNIFORMS,
            GLSL_COMMON,
            GLSL_BEZIER_UTILS,
            GLSL_TESSELLATE,
        ];
        context.m_tessellateProgram.compileAndAttachShaderParts(
            GL_VERTEX_SHADER,
            &generalDefines,
            &tessellateSources,
            &context.m_capabilities,
        );
        context.m_tessellateProgram.compileAndAttachShaderParts(
            GL_FRAGMENT_SHADER,
            &generalDefines,
            &tessellateSources,
            &context.m_capabilities,
        );
        context.m_tessellateProgram.link();
        context
            .m_state
            .borrow_mut()
            .bindProgram(context.m_tessellateProgram.id());
        glutils::Uniform1iByName(
            context.m_tessellateProgram.id(),
            GLSL_gaussianIntegralTexture,
            GAUSSIAN_INTEGRAL_TEXTURE_IDX as GLint,
        );
        bindUniformBlock(&execution, context.m_tessellateProgram.id());
        if !context.m_capabilities.ARB_shader_storage_buffer_object() {
            glutils::Uniform1iByName(
                context.m_tessellateProgram.id(),
                GLSL_pathBuffer,
                PATH_BUFFER_IDX as GLint,
            );
            glutils::Uniform1iByName(
                context.m_tessellateProgram.id(),
                GLSL_contourBuffer,
                CONTOUR_BUFFER_IDX as GLint,
            );
        }

        context
            .m_state
            .borrow_mut()
            .bindVAO(context.m_tessellateVAO.id());
        for index in 0..4 {
            recordGLCommand(GLCommand::EnableVertexAttribArray(index));
            recordGLCommand(GLCommand::VertexAttribDivisor(index, 1));
        }
        context
            .m_state
            .borrow_mut()
            .bindBuffer(GL_ELEMENT_ARRAY_BUFFER, context.m_tessSpanIndexBuffer.id());
        recordGLCommand(GLCommand::BufferData {
            target: GL_ELEMENT_ARRAY_BUFFER,
            size: std::mem::size_of_val(&gpu::kTessSpanIndices),
            data: Some(bytesOfSlice(&gpu::kTessSpanIndices)),
            usage: GL_STATIC_DRAW,
        });

        context.m_state.borrow_mut().bindVAO(context.m_drawVAO.id());
        let mut patchVertices =
            vec![gpu::PatchVertex::default(); gpu::kPatchVertexBufferCount as usize];
        let mut patchIndices = vec![0u16; gpu::kPatchIndexBufferCount as usize];
        unsafe {
            gpu::GeneratePatchBufferData(patchVertices.as_mut_ptr(), patchIndices.as_mut_ptr())
        };
        context
            .m_state
            .borrow_mut()
            .bindBuffer(GL_ARRAY_BUFFER, context.m_patchVerticesBuffer.id());
        recordGLCommand(GLCommand::BufferData {
            target: GL_ARRAY_BUFFER,
            size: std::mem::size_of_val(patchVertices.as_slice()),
            data: Some(bytesOfSlice(&patchVertices)),
            usage: GL_STATIC_DRAW,
        });
        context
            .m_state
            .borrow_mut()
            .bindBuffer(GL_ELEMENT_ARRAY_BUFFER, context.m_patchIndicesBuffer.id());
        recordGLCommand(GLCommand::BufferData {
            target: GL_ELEMENT_ARRAY_BUFFER,
            size: std::mem::size_of_val(patchIndices.as_slice()),
            data: Some(bytesOfSlice(&patchIndices)),
            usage: GL_STATIC_DRAW,
        });
        recordGLCommand(GLCommand::EnableVertexAttribArray(0));
        recordGLCommand(GLCommand::VertexAttribPointer {
            index: 0,
            size: 4,
            type_: GL_FLOAT,
            normalized: GL_FALSE,
            stride: std::mem::size_of::<gpu::PatchVertex>() as GLsizei,
            offset: 0,
        });
        recordGLCommand(GLCommand::EnableVertexAttribArray(1));
        recordGLCommand(GLCommand::VertexAttribPointer {
            index: 1,
            size: 4,
            type_: GL_FLOAT,
            normalized: GL_FALSE,
            stride: std::mem::size_of::<gpu::PatchVertex>() as GLsizei,
            offset: (std::mem::size_of::<f32>() * 4) as u32,
        });

        context
            .m_state
            .borrow_mut()
            .bindVAO(context.m_trianglesVAO.id());
        recordGLCommand(GLCommand::EnableVertexAttribArray(0));

        context
            .m_state
            .borrow_mut()
            .bindVAO(context.m_imageRectVAO.id());
        recordGLCommand(GLCommand::EnableVertexAttribArray(0));
        for index in IMAGE_FIRST_ATTRIB_IDX..=IMAGE_LAST_ATTRIB_IDX {
            recordGLCommand(GLCommand::EnableVertexAttribArray(index));
            recordGLCommand(GLCommand::VertexAttribDivisor(index, 1));
        }
        context
            .m_state
            .borrow_mut()
            .bindBuffer(GL_ARRAY_BUFFER, context.m_imageRectVertexBuffer.id());
        recordGLCommand(GLCommand::BufferData {
            target: GL_ARRAY_BUFFER,
            size: std::mem::size_of_val(&gpu::kImageRectVertices),
            data: Some(bytesOfSlice(&gpu::kImageRectVertices)),
            usage: GL_STATIC_DRAW,
        });
        recordGLCommand(GLCommand::VertexAttribPointer {
            index: 0,
            size: 4,
            type_: GL_FLOAT,
            normalized: GL_FALSE,
            stride: std::mem::size_of::<gpu::ImageRectVertex>() as GLsizei,
            offset: 0,
        });
        context
            .m_state
            .borrow_mut()
            .bindBuffer(GL_ELEMENT_ARRAY_BUFFER, context.m_imageRectIndexBuffer.id());
        recordGLCommand(GLCommand::BufferData {
            target: GL_ELEMENT_ARRAY_BUFFER,
            size: std::mem::size_of_val(&gpu::kImageRectIndices),
            data: Some(bytesOfSlice(&gpu::kImageRectIndices)),
            usage: GL_STATIC_DRAW,
        });

        context
            .m_state
            .borrow_mut()
            .bindVAO(context.m_imageMeshVAO.id());
        recordGLCommand(GLCommand::EnableVertexAttribArray(0));
        recordGLCommand(GLCommand::EnableVertexAttribArray(1));
        for index in IMAGE_FIRST_ATTRIB_IDX..=IMAGE_LAST_ATTRIB_IDX {
            recordGLCommand(GLCommand::EnableVertexAttribArray(index));
            recordGLCommand(GLCommand::VertexAttribDivisor(index, 1));
        }
        if let Some(pls) = context.m_plsImpl.as_mut() {
            pls.init((&*context.m_state).clone());
        }
    });
}

pub(crate) fn invalidateGLState(context: &mut RenderContextGLImpl) {
    let execution = (&*context.rust_execution).clone();
    execution.withCurrent(|| {
        for (unit, texture) in [
            (TESS_VERTEX_TEXTURE_IDX, context.m_tessVertexTexture),
            (GRAD_TEXTURE_IDX, context.m_gradientTexture),
            (
                GAUSSIAN_INTEGRAL_TEXTURE_IDX,
                context.m_gaussianIntegralTexture.id(),
            ),
            (
                FEATHER_ATLAS_TEXTURE_IDX,
                context.m_featherAtlasTexture.id(),
            ),
        ] {
            recordGLCommand(GLCommand::ActiveTexture(GL_TEXTURE0 + unit));
            recordGLCommand(GLCommand::BindTexture(GL_TEXTURE_2D, texture));
        }
        context.m_state.borrow_mut().invalidate();
    });
}

pub(crate) fn unbindGLInternalResources(context: &mut RenderContextGLImpl) {
    let execution = (&*context.rust_execution).clone();
    execution.withCurrent(|| {
        let mut state = context.m_state.borrow_mut();
        state.bindVAO(0);
        state.bindBuffer(GL_ELEMENT_ARRAY_BUFFER, 0);
        state.bindBuffer(GL_ARRAY_BUFFER, 0);
        state.bindBuffer(GL_UNIFORM_BUFFER, 0);
        drop(state);
        recordGLCommand(GLCommand::BindFramebuffer(GL_FRAMEBUFFER, 0));
        for unit in 0..=DEFAULT_BINDINGS_SET_SIZE {
            recordGLCommand(GLCommand::ActiveTexture(GL_TEXTURE0 + unit));
            recordGLCommand(GLCommand::BindTexture(GL_TEXTURE_2D, 0));
        }
    });
}

pub(crate) fn makeRenderBuffer(
    context: &mut RenderContextGLImpl,
    bufferType: RenderBufferType,
    flags: RenderBufferFlags,
    sizeInBytes: usize,
) -> rcp<RenderBuffer> {
    let execution = (&*context.rust_execution).clone();
    execution.withCurrent(|| {
        let owner = Box::new(RenderBufferGLImpl::new(
            bufferType,
            flags,
            sizeInBytes,
            (&*context.m_state).clone(),
        ));
        unsafe { rcp::from_ptr(Box::into_raw(owner).cast::<RenderBuffer>()) }
    })
}

pub(crate) fn adoptImageTexture(
    context: &RenderContextGLImpl,
    width: u32,
    height: u32,
    textureID: GLuint,
) -> rcp<RiveTexture> {
    let derived = make_rcp(|| {
        TextureGLImpl::new(width, height, textureID, (&*context.rust_execution).clone())
    });
    unsafe { static_rcp_cast(derived) }
}

pub(crate) fn makeImageTexture(
    context: &mut RenderContextGLImpl,
    width: u32,
    height: u32,
    mipLevelCount: u32,
    format: GPUTextureFormat,
    imageData: &[u8],
    blockWidth: u8,
    blockHeight: u8,
    _srgb: bool,
    generateRemainingMips: bool,
) -> rcp<RiveTexture> {
    let (sizedInternal, compressed, bytesPerBlock) = match format {
        GPUTextureFormat::rgba32 => {
            assert_eq!((blockWidth, blockHeight), (1, 1));
            (GL_RGBA8, false, 4usize)
        }
        GPUTextureFormat::bc7 => (GL_COMPRESSED_RGBA_BPTC_UNORM, true, 16),
        GPUTextureFormat::etc2 => (GL_COMPRESSED_RGBA8_ETC2_EAC, true, 16),
        GPUTextureFormat::astc => {
            let index = crate::mechanical_port::source::decoders::include::rive::decoders::astc_footprints_hpp::astcFootprintIndex(
                blockWidth,
                blockHeight,
            );
            if index < 0 {
                debug_assert!(false, "unsupported ASTC block footprint");
                return rcp::new();
            }
            (GL_COMPRESSED_RGBA_ASTC_4x4_KHR + index as GLenum, true, 16)
        }
        _ => {
            debug_assert!(false, "unsupported format");
            return rcp::new();
        }
    };
    assert!(!(generateRemainingMips && compressed));
    let execution = (&*context.rust_execution).clone();
    execution.withCurrent(|| {
        let textureID = generateGLObject(GLObjectKind::Texture);
        recordGLCommand(GLCommand::ActiveTexture(GL_TEXTURE0 + IMAGE_TEXTURE_IDX));
        recordGLCommand(GLCommand::BindTexture(GL_TEXTURE_2D, textureID));
        recordGLCommand(GLCommand::TexStorage2D {
            target: GL_TEXTURE_2D,
            levels: mipLevelCount,
            internal_format: sizedInternal,
            width,
            height,
        });
        if !imageData.is_empty() {
            let levelsToUpload = if generateRemainingMips {
                1
            } else {
                mipLevelCount
            };
            let mut offset = 0usize;
            for level in 0..levelsToUpload {
                let levelWidth = 1u32.max(width >> level);
                let levelHeight = 1u32.max(height >> level);
                let blocksX = levelWidth.div_ceil(blockWidth as u32);
                let blocksY = levelHeight.div_ceil(blockHeight as u32);
                let levelBytes = blocksX as usize * blocksY as usize * bytesPerBlock;
                let bytes = imageData[offset..offset + levelBytes].to_vec();
                if compressed {
                    recordGLCommand(GLCommand::CompressedTexSubImage2D {
                        target: GL_TEXTURE_2D,
                        level,
                        x: 0,
                        y: 0,
                        width: levelWidth,
                        height: levelHeight,
                        format: sizedInternal,
                        data: bytes,
                    });
                } else {
                    recordGLCommand(GLCommand::TexSubImage2D {
                        target: GL_TEXTURE_2D,
                        level,
                        x: 0,
                        y: 0,
                        width: levelWidth,
                        height: levelHeight,
                        format: GL_RGBA,
                        type_: GL_UNSIGNED_BYTE,
                        data: bytes,
                    });
                }
                offset += levelBytes;
            }
            if generateRemainingMips && mipLevelCount > 1 {
                recordGLCommand(GLCommand::GenerateMipmap(GL_TEXTURE_2D));
            }
        }
        adoptImageTexture(context, width, height, textureID)
    })
}

pub(crate) fn makeRenderCanvas(
    context: &mut RenderContextGLImpl,
    width: u32,
    height: u32,
) -> rcp<RenderCanvas> {
    let execution = (&*context.rust_execution).clone();
    execution.withCurrent(|| {
        let textureID = generateGLObject(GLObjectKind::Texture);
        recordGLCommand(GLCommand::ActiveTexture(GL_TEXTURE0));
        recordGLCommand(GLCommand::BindTexture(GL_TEXTURE_2D, textureID));
        recordGLCommand(GLCommand::TexStorage2D {
            target: GL_TEXTURE_2D,
            levels: 1,
            internal_format: GL_RGBA8,
            width,
            height,
        });
        let source = make_rcp(|| {
            CanvasSourceTextureGLImpl::new(
                width,
                height,
                textureID,
                execution.clone(),
                context,
                Rc::downgrade(&context.m_canvasMirrors),
            )
        });
        let source: rcp<RiveTexture> = unsafe { static_rcp_cast(source) };
        let image = make_rcp(|| unsafe { RiveRenderImage::new(source) });
        let mut target = make_rcp(|| TextureRenderTargetGL::new(width, height, execution.clone()));
        unsafe { (&mut *target.get()).setTargetTexture(textureID) };
        let target: rcp<RenderTarget> = unsafe { static_rcp_cast(target) };
        registerCanvasTarget(context, textureID);
        make_rcp(|| unsafe { RenderCanvas::new(image, target) })
    })
}

pub(crate) fn makeOreContext(
    context: &mut RenderContextGLImpl,
) -> Option<Box<crate::mechanical_port::source::include::rive::factory_hpp::OreContext>> {
    super::ore_context_gl_decl::ContextGL::Make((&*context.rust_execution).clone()).map(|context| {
        Box::new(
            crate::mechanical_port::source::include::rive::factory_hpp::OreContext::GL(context),
        )
    })
}

pub(crate) fn registerCanvasTarget(context: &mut RenderContextGLImpl, sourceTex: GLuint) {
    context
        .m_canvasMirrors
        .borrow_mut()
        .insert(sourceTex, CanvasMirrorEntry::default());
}

pub(crate) unsafe fn getCanvasImportMirror(
    context: &mut RenderContextGLImpl,
    sourceTex: *mut RiveTexture,
    width: u32,
    height: u32,
) -> rcp<RiveRenderImage> {
    if sourceTex.is_null() {
        return rcp::new();
    }
    let execution = (&*context.rust_execution).clone();
    if !unsafe { &*sourceTex }.belongs_to_owner_thread_execution(
        execution.domain().key(),
        execution.generation(),
    ) {
        return rcp::new();
    }
    let glID = unsafe { (&*sourceTex).nativeHandle() } as usize as GLuint;
    if glID == 0 {
        return rcp::new();
    }
    getOrCreateCanvasMirror(context, glID, width, height)
}

pub(crate) fn unregisterCanvasTarget(context: &mut RenderContextGLImpl, sourceTex: GLuint) {
    let entry = context.m_canvasMirrors.borrow_mut().remove(&sourceTex);
    let Some(entry) = entry else {
        return;
    };
    let execution = (&*context.rust_execution).clone();
    execution.withCurrent(|| {
        if entry.readFBO != 0 {
            recordGLCommand(GLCommand::DeleteFramebuffer(entry.readFBO));
        }
        if entry.drawFBO != 0 {
            recordGLCommand(GLCommand::DeleteFramebuffer(entry.drawFBO));
        }
    });
}

pub(crate) fn getOrCreateCanvasMirror(
    context: &mut RenderContextGLImpl,
    sourceTex: GLuint,
    width: u32,
    height: u32,
) -> rcp<RiveRenderImage> {
    let execution = (&*context.rust_execution).clone();
    let canvasRegistry = (&*context.m_canvasMirrors).clone();
    execution.withCurrent(|| {
        let canCreateMirror = canvasRegistry
            .borrow()
            .get(&sourceTex)
            .is_some_and(|entry| !entry.hasMirror);
        // Preserve the authored defect: a second request while the mirror is
        // live returns null and lets the caller sample the bottom-up source
        // directly. This lookup intentionally happens after the ingress drain.
        if !canCreateMirror {
            return rcp::new();
        }

        let mirrorTex = generateGLObject(GLObjectKind::Texture);
        recordGLCommand(GLCommand::ActiveTexture(GL_TEXTURE0));
        recordGLCommand(GLCommand::BindTexture(GL_TEXTURE_2D, mirrorTex));
        recordGLCommand(GLCommand::TexStorage2D {
            target: GL_TEXTURE_2D,
            levels: 1,
            internal_format: GL_RGBA8,
            width,
            height,
        });
        let readFBO = generateGLObject(GLObjectKind::Framebuffer);
        let drawFBO = generateGLObject(GLObjectKind::Framebuffer);
        recordGLCommand(GLCommand::BindFramebuffer(GL_READ_FRAMEBUFFER, readFBO));
        recordGLCommand(GLCommand::FramebufferTexture2D {
            target: GL_READ_FRAMEBUFFER,
            attachment: GL_COLOR_ATTACHMENT0,
            texture_target: GL_TEXTURE_2D,
            texture: sourceTex,
            level: 0,
        });
        recordGLCommand(GLCommand::BindFramebuffer(GL_DRAW_FRAMEBUFFER, drawFBO));
        recordGLCommand(GLCommand::FramebufferTexture2D {
            target: GL_DRAW_FRAMEBUFFER,
            attachment: GL_COLOR_ATTACHMENT0,
            texture_target: GL_TEXTURE_2D,
            texture: mirrorTex,
            level: 0,
        });
        recordGLCommand(GLCommand::BindFramebuffer(GL_READ_FRAMEBUFFER, 0));
        recordGLCommand(GLCommand::BindFramebuffer(GL_DRAW_FRAMEBUFFER, 0));
        let registered = {
            let mut registry = canvasRegistry.borrow_mut();
            match registry.get_mut(&sourceTex) {
                Some(entry) if !entry.hasMirror => {
                    *entry = CanvasMirrorEntry {
                        mirrorTex,
                        width,
                        height,
                        readFBO,
                        drawFBO,
                        hasMirror: true,
                    };
                    true
                }
                _ => false,
            }
        };
        if !registered {
            // A source finalizer can arrive from a worker while provider calls
            // are in flight. It removes the logical entry before these owned
            // names are retired, so no stale mirror is published.
            recordGLCommand(GLCommand::DeleteFramebuffer(readFBO));
            recordGLCommand(GLCommand::DeleteFramebuffer(drawFBO));
            recordGLCommand(GLCommand::DeleteTexture(mirrorTex));
            context.m_state.borrow_mut().invalidate();
            return rcp::new();
        }
        let texture = make_rcp(|| {
            CanvasMirrorTextureGLImpl::new(
                width,
                height,
                mirrorTex,
                execution.clone(),
                context,
                sourceTex,
                Rc::downgrade(&canvasRegistry),
            )
        });
        let texture: rcp<RiveTexture> = unsafe { static_rcp_cast(texture) };
        let image = make_rcp(|| unsafe { RiveRenderImage::new(texture) });
        context.m_state.borrow_mut().invalidate();
        image
    })
}

pub(crate) fn blitMirrorIfRegistered(context: &mut RenderContextGLImpl, targetTex: GLuint) {
    let execution = (&*context.rust_execution).clone();
    let canvasRegistry = (&*context.m_canvasMirrors).clone();
    execution.withCurrent(|| {
        let entry = canvasRegistry.borrow().get(&targetTex).copied();
        let Some(entry) = entry.filter(|entry| entry.hasMirror) else {
            return;
        };
        recordGLCommand(GLCommand::BindFramebuffer(
            GL_READ_FRAMEBUFFER,
            entry.readFBO,
        ));
        recordGLCommand(GLCommand::BindFramebuffer(
            GL_DRAW_FRAMEBUFFER,
            entry.drawFBO,
        ));
        recordGLCommand(GLCommand::BlitFramebuffer(
            [
                0,
                0,
                entry.width as i32,
                entry.height as i32,
                0,
                entry.height as i32,
                entry.width as i32,
                0,
            ],
            GL_COLOR_BUFFER_BIT,
            GL_NEAREST,
        ));
        recordGLCommand(GLCommand::BindFramebuffer(GL_READ_FRAMEBUFFER, 0));
        recordGLCommand(GLCommand::BindFramebuffer(GL_DRAW_FRAMEBUFFER, 0));
        context.m_state.borrow_mut().invalidate();
    });
}

fn makeUniformBufferRing(
    context: &RenderContextGLImpl,
    capacityInBytes: usize,
) -> Option<Box<dyn BufferRingContract>> {
    (capacityInBytes != 0).then(|| {
        Box::new(BufferRingGLImpl::new(
            GL_UNIFORM_BUFFER,
            capacityInBytes,
            (&*context.m_state).clone(),
            (&*context.rust_execution).clone(),
        )) as Box<dyn BufferRingContract>
    })
}

fn makeStorageBufferRing(
    context: &RenderContextGLImpl,
    capacityInBytes: usize,
    structure: gpu::StorageBufferStructure,
) -> Option<Box<dyn BufferRingContract>> {
    if capacityInBytes == 0 {
        None
    } else if context.m_capabilities.ARB_shader_storage_buffer_object() {
        Some(Box::new(StorageBufferRingGLImpl::new(
            capacityInBytes,
            structure,
            (&*context.m_state).clone(),
            (&*context.rust_execution).clone(),
        )))
    } else {
        Some(Box::new(TexelBufferRingWebGL::new(
            capacityInBytes,
            structure,
            (&*context.m_state).clone(),
            (&*context.rust_execution).clone(),
        )))
    }
}

fn makeVertexBufferRing(
    context: &RenderContextGLImpl,
    capacityInBytes: usize,
) -> Option<Box<dyn BufferRingContract>> {
    (capacityInBytes != 0).then(|| {
        Box::new(BufferRingGLImpl::new(
            GL_ARRAY_BUFFER,
            capacityInBytes,
            (&*context.m_state).clone(),
            (&*context.rust_execution).clone(),
        )) as Box<dyn BufferRingContract>
    })
}

pub(crate) fn resizeGradientTexture(context: &mut RenderContextGLImpl, width: u32, height: u32) {
    let execution = (&*context.rust_execution).clone();
    execution.withCurrent(|| {
        recordGLCommand(GLCommand::DeleteTexture(context.m_gradientTexture));
        context.m_gradientTexture = if width == 0 || height == 0 {
            0
        } else {
            let texture = generateGLObject(GLObjectKind::Texture);
            recordGLCommand(GLCommand::ActiveTexture(GL_TEXTURE0 + GRAD_TEXTURE_IDX));
            recordGLCommand(GLCommand::BindTexture(GL_TEXTURE_2D, texture));
            recordGLCommand(GLCommand::TexStorage2D {
                target: GL_TEXTURE_2D,
                levels: 1,
                internal_format: GL_RGBA8,
                width,
                height,
            });
            glutils::SetTexture2DSamplingParams(GL_LINEAR, GL_LINEAR);
            texture
        };
        recordGLCommand(GLCommand::BindFramebuffer(
            GL_FRAMEBUFFER,
            context.m_colorRampFBO.id(),
        ));
        recordGLCommand(GLCommand::FramebufferTexture2D {
            target: GL_FRAMEBUFFER,
            attachment: GL_COLOR_ATTACHMENT0,
            texture_target: GL_TEXTURE_2D,
            texture: context.m_gradientTexture,
            level: 0,
        });
    });
}

pub(crate) fn resizeTessellationTexture(
    context: &mut RenderContextGLImpl,
    width: u32,
    height: u32,
) {
    let execution = (&*context.rust_execution).clone();
    execution.withCurrent(|| {
        recordGLCommand(GLCommand::DeleteTexture(context.m_tessVertexTexture));
        context.m_tessVertexTexture = if width == 0 || height == 0 {
            0
        } else {
            let texture = generateGLObject(GLObjectKind::Texture);
            recordGLCommand(GLCommand::ActiveTexture(
                GL_TEXTURE0 + TESS_VERTEX_TEXTURE_IDX,
            ));
            recordGLCommand(GLCommand::BindTexture(GL_TEXTURE_2D, texture));
            recordGLCommand(GLCommand::TexStorage2D {
                target: GL_TEXTURE_2D,
                levels: 1,
                internal_format: if context.m_capabilities.needsFloatingPointTessellationTexture {
                    GL_RGBA32F
                } else {
                    GL_RGBA32UI
                },
                width,
                height,
            });
            glutils::SetTexture2DSamplingParams(GL_NEAREST, GL_NEAREST);
            texture
        };
        recordGLCommand(GLCommand::BindFramebuffer(
            GL_FRAMEBUFFER,
            context.m_tessellateFBO.id(),
        ));
        recordGLCommand(GLCommand::FramebufferTexture2D {
            target: GL_FRAMEBUFFER,
            attachment: GL_COLOR_ATTACHMENT0,
            texture_target: GL_TEXTURE_2D,
            texture: context.m_tessVertexTexture,
            level: 0,
        });
    });
}

fn compileFeatherAtlasProgram(
    program: &mut FeatherAtlasProgram,
    vertexShaderID: GLuint,
    defines: &[&str],
    sources: &[&str],
    capabilities: &GLCapabilities,
    state: &GLStateOwner,
    execution: &GLExecutionStamp,
) {
    program.m_program.moveAssign(Program::new());
    recordGLCommand(GLCommand::AttachShader(
        program.m_program.id(),
        vertexShaderID,
    ));
    program.m_program.compileAndAttachShaderParts(
        GL_FRAGMENT_SHADER,
        defines,
        sources,
        capabilities,
    );
    program.m_program.link();
    state.borrow_mut().bindProgram(program.m_program.id());
    bindUniformBlock(execution, program.m_program.id());
    glutils::Uniform1iByName(
        program.m_program.id(),
        GLSL_tessVertexTexture,
        TESS_VERTEX_TEXTURE_IDX as GLint,
    );
    glutils::Uniform1iByName(
        program.m_program.id(),
        GLSL_gaussianIntegralTexture,
        GAUSSIAN_INTEGRAL_TEXTURE_IDX as GLint,
    );
    if !capabilities.ARB_shader_storage_buffer_object() {
        glutils::Uniform1iByName(
            program.m_program.id(),
            GLSL_pathBuffer,
            PATH_BUFFER_IDX as GLint,
        );
        glutils::Uniform1iByName(
            program.m_program.id(),
            GLSL_contourBuffer,
            CONTOUR_BUFFER_IDX as GLint,
        );
    }
    if !capabilities.ANGLE_base_vertex_base_instance_shader_builtin() {
        program.m_baseInstanceUniformLocation = execution.domain().uniformLocation(
            program.m_program.id(),
            glutils::BASE_INSTANCE_UNIFORM_NAME.as_bytes(),
        );
    }
}

fn buildFeatherAtlasRenderPipelines(context: &mut RenderContextGLImpl) {
    let execution = (&*context.rust_execution).clone();
    execution.withCurrent(|| {
        let mut defines = vec![
            GLSL_DRAW_PATH,
            GLSL_ENABLE_FEATHER,
            GLSL_ENABLE_INSTANCE_INDEX,
        ];
        if !context.m_capabilities.ARB_shader_storage_buffer_object() {
            defines.push(GLSL_DISABLE_SHADER_STORAGE_BUFFERS);
        }
        context.m_featherAtlasFillPipelineState = gpu::FEATHER_ATLAS_FILL_PIPELINE_STATE;
        context.m_featherAtlasStrokePipelineState = gpu::FEATHER_ATLAS_STROKE_PIPELINE_STATE;
        match context.m_featherAtlasRenderType {
            FeatherAtlasRenderType::r16f | FeatherAtlasRenderType::r32f => {}
            FeatherAtlasRenderType::r32uiFramebufferFetch => {
                defines.push(GLSL_ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH);
                context.m_featherAtlasFillPipelineState.blendEquation = gpu::BlendEquation::none;
                context.m_featherAtlasStrokePipelineState.blendEquation = gpu::BlendEquation::none;
            }
            FeatherAtlasRenderType::r32uiPixelLocalStorageANGLE => {
                defines.push(GLSL_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE);
                context.m_featherAtlasFillPipelineState.blendEquation = gpu::BlendEquation::none;
                context.m_featherAtlasFillPipelineState.colorWriteEnabled = false;
                context.m_featherAtlasStrokePipelineState.blendEquation = gpu::BlendEquation::none;
                context.m_featherAtlasStrokePipelineState.colorWriteEnabled = false;
            }
            FeatherAtlasRenderType::rgba8 => {
                defines.push(GLSL_ATLAS_RENDER_TARGET_RGBA8_UNORM);
            }
            FeatherAtlasRenderType::r8PixelLocalStorageEXT
            | FeatherAtlasRenderType::r32iAtomicTexture => {
                panic!("atlas type is excluded by the frozen RIVE_WEBGL profile")
            }
        }
        let sources = [
            GLSL_CONSTANTS,
            GLSL_FLUSH_UNIFORMS,
            GLSL_COMMON,
            GLSL_DRAW_PATH_COMMON,
            GLSL_RENDER_ATLAS,
        ];
        context.m_featherAtlasVertexShader.compileParts(
            GL_VERTEX_SHADER,
            &defines,
            &sources,
            &context.m_capabilities,
        );
        defines.push(GLSL_ATLAS_FEATHERED_FILL);
        compileFeatherAtlasProgram(
            &mut context.m_featherAtlasFillProgram,
            context.m_featherAtlasVertexShader.id(),
            &defines,
            &sources,
            &context.m_capabilities,
            &context.m_state,
            &execution,
        );
        defines.pop();
        defines.push(GLSL_ATLAS_FEATHERED_STROKE);
        compileFeatherAtlasProgram(
            &mut context.m_featherAtlasStrokeProgram,
            context.m_featherAtlasVertexShader.id(),
            &defines,
            &sources,
            &context.m_capabilities,
            &context.m_state,
            &execution,
        );
        defines.pop();

        if needsFeatherAtlasResolveDraw(context.m_featherAtlasRenderType) {
            context.m_featherAtlasResolveVertexShader.compile(
                GL_VERTEX_SHADER,
                GLSL_RESOLVE_ATLAS,
                &context.m_capabilities,
            );
            context
                .m_featherAtlasResolveProgram
                .moveAssign(Program::new());
            recordGLCommand(GLCommand::AttachShader(
                context.m_featherAtlasResolveProgram.id(),
                context.m_featherAtlasResolveVertexShader.id(),
            ));
            let resolveSources = [
                GLSL_CONSTANTS,
                GLSL_FLUSH_UNIFORMS,
                GLSL_COMMON,
                GLSL_RESOLVE_ATLAS,
            ];
            context
                .m_featherAtlasResolveProgram
                .compileAndAttachShaderParts(
                    GL_FRAGMENT_SHADER,
                    &defines,
                    &resolveSources,
                    &context.m_capabilities,
                );
            context.m_featherAtlasResolveProgram.link();
            if context.m_featherAtlasRenderType == FeatherAtlasRenderType::rgba8 {
                context
                    .m_state
                    .borrow_mut()
                    .bindProgram(context.m_featherAtlasResolveProgram.id());
                glutils::Uniform1iByName(
                    context.m_featherAtlasResolveProgram.id(),
                    GLSL_atlasRenderTexture,
                    0,
                );
            }
        }
    });
}

fn featherAtlasRenderFormat(renderType: FeatherAtlasRenderType) -> GLenum {
    match renderType {
        FeatherAtlasRenderType::r16f => GL_R16F,
        FeatherAtlasRenderType::r32f => GL_R32F,
        FeatherAtlasRenderType::r32uiFramebufferFetch
        | FeatherAtlasRenderType::r32uiPixelLocalStorageANGLE => GL_R32UI,
        FeatherAtlasRenderType::r8PixelLocalStorageEXT => GL_R8,
        FeatherAtlasRenderType::r32iAtomicTexture => GL_R32I,
        FeatherAtlasRenderType::rgba8 => GL_RGBA8,
    }
}

pub(crate) fn resizeFeatherAtlasTexture(
    context: &mut RenderContextGLImpl,
    width: u32,
    height: u32,
) {
    let execution = (&*context.rust_execution).clone();
    execution.withCurrent(|| {
        context
            .m_featherAtlasRenderTexture
            .moveAssign(GLTexture::Zero());
        context.m_featherAtlasTexture.moveAssign(GLTexture::Zero());
        context
            .m_featherAtlasRenderFBO
            .moveAssign(Framebuffer::Zero());
        context
            .m_featherAtlasResolveFBO
            .moveAssign(Framebuffer::Zero());
        if width == 0 || height == 0 {
            return;
        }
        let renderFormat = featherAtlasRenderFormat(context.m_featherAtlasRenderType);
        let canSample = renderFormat == GL_R8
            || renderFormat == GL_R16F && context.m_capabilities.OES_texture_half_float_linear();
        if !canSample {
            context
                .m_featherAtlasRenderTexture
                .moveAssign(GLTexture::new());
            recordGLCommand(GLCommand::ActiveTexture(GL_TEXTURE0));
            recordGLCommand(GLCommand::BindTexture(
                GL_TEXTURE_2D,
                context.m_featherAtlasRenderTexture.id(),
            ));
            recordGLCommand(GLCommand::TexStorage2D {
                target: GL_TEXTURE_2D,
                levels: 1,
                internal_format: renderFormat,
                width,
                height,
            });
            glutils::SetTexture2DSamplingParams(GL_NEAREST, GL_NEAREST);
        }
        context.m_featherAtlasTexture.moveAssign(GLTexture::new());
        recordGLCommand(GLCommand::ActiveTexture(
            GL_TEXTURE0 + FEATHER_ATLAS_TEXTURE_IDX,
        ));
        recordGLCommand(GLCommand::BindTexture(
            GL_TEXTURE_2D,
            context.m_featherAtlasTexture.id(),
        ));
        recordGLCommand(GLCommand::TexStorage2D {
            target: GL_TEXTURE_2D,
            levels: 1,
            internal_format: if canSample { renderFormat } else { GL_R8 },
            width,
            height,
        });
        glutils::SetTexture2DSamplingParams(GL_LINEAR, GL_LINEAR);
        if context.m_featherAtlasVertexShader.id() == 0 {
            buildFeatherAtlasRenderPipelines(context);
        }

        context
            .m_featherAtlasRenderFBO
            .moveAssign(Framebuffer::new());
        recordGLCommand(GLCommand::BindFramebuffer(
            GL_FRAMEBUFFER,
            context.m_featherAtlasRenderFBO.id(),
        ));
        match context.m_featherAtlasRenderType {
            FeatherAtlasRenderType::r16f
            | FeatherAtlasRenderType::r32f
            | FeatherAtlasRenderType::rgba8 => {
                let texture = if context.m_featherAtlasRenderTexture.id() != 0 {
                    context.m_featherAtlasRenderTexture.id()
                } else {
                    context.m_featherAtlasTexture.id()
                };
                recordGLCommand(GLCommand::FramebufferTexture2D {
                    target: GL_FRAMEBUFFER,
                    attachment: GL_COLOR_ATTACHMENT0,
                    texture_target: GL_TEXTURE_2D,
                    texture,
                    level: 0,
                });
                if context.m_featherAtlasRenderTexture.id() != 0 {
                    context
                        .m_featherAtlasResolveFBO
                        .moveAssign(Framebuffer::new());
                    recordGLCommand(GLCommand::BindFramebuffer(
                        GL_FRAMEBUFFER,
                        context.m_featherAtlasResolveFBO.id(),
                    ));
                    recordGLCommand(GLCommand::FramebufferTexture2D {
                        target: GL_FRAMEBUFFER,
                        attachment: GL_COLOR_ATTACHMENT0,
                        texture_target: GL_TEXTURE_2D,
                        texture: context.m_featherAtlasTexture.id(),
                        level: 0,
                    });
                }
            }
            FeatherAtlasRenderType::r32uiFramebufferFetch => {
                recordGLCommand(GLCommand::FramebufferTexture2D {
                    target: GL_FRAMEBUFFER,
                    attachment: GL_COLOR_ATTACHMENT0,
                    texture_target: GL_TEXTURE_2D,
                    texture: context.m_featherAtlasRenderTexture.id(),
                    level: 0,
                });
                recordGLCommand(GLCommand::FramebufferTexture2D {
                    target: GL_FRAMEBUFFER,
                    attachment: GL_COLOR_ATTACHMENT1,
                    texture_target: GL_TEXTURE_2D,
                    texture: context.m_featherAtlasTexture.id(),
                    level: 0,
                });
                recordGLCommand(GLCommand::DrawBuffers(vec![
                    GL_COLOR_ATTACHMENT0,
                    GL_COLOR_ATTACHMENT1,
                ]));
            }
            FeatherAtlasRenderType::r32uiPixelLocalStorageANGLE => {
                assert_ne!(context.m_featherAtlasRenderTexture.id(), 0);
                recordGLCommand(GLCommand::FramebufferTexturePixelLocalStorageANGLE {
                    plane: 0,
                    backing_texture: context.m_featherAtlasRenderTexture.id(),
                    level: 0,
                    layer: 0,
                    usage: GL_NONE,
                });
                recordGLCommand(GLCommand::FramebufferTexture2D {
                    target: GL_FRAMEBUFFER,
                    attachment: GL_COLOR_ATTACHMENT0,
                    texture_target: GL_TEXTURE_2D,
                    texture: context.m_featherAtlasTexture.id(),
                    level: 0,
                });
            }
            FeatherAtlasRenderType::r8PixelLocalStorageEXT
            | FeatherAtlasRenderType::r32iAtomicTexture => {
                panic!("atlas type is excluded by the frozen RIVE_WEBGL profile")
            }
        }
    });
}

pub(crate) fn resizeTransientPLSBacking(
    context: &mut RenderContextGLImpl,
    width: u32,
    height: u32,
    planeCount: u32,
) {
    if let Some(pls) = context.m_plsImpl.as_mut() {
        pls.resizeTransientPLSBacking(width, height, planeCount);
    } else {
        assert_eq!(width | height | planeCount, 0);
    }
}

pub(crate) fn resizeAtomicCoverageBacking(
    context: &mut RenderContextGLImpl,
    width: u32,
    height: u32,
) {
    if let Some(pls) = context.m_plsImpl.as_mut() {
        pls.resizeAtomicCoverageBacking(width, height);
    } else {
        assert_eq!(width | height, 0);
    }
}

fn shaderFeatureDefine(feature: gpu::ShaderFeatures) -> &'static str {
    match feature {
        gpu::ShaderFeatures::ENABLE_CLIPPING => "I",
        gpu::ShaderFeatures::ENABLE_CLIP_RECT => "BB",
        gpu::ShaderFeatures::ENABLE_ADVANCED_BLEND => "AB",
        gpu::ShaderFeatures::ENABLE_FEATHER => "HB",
        gpu::ShaderFeatures::ENABLE_EVEN_ODD => "WC",
        gpu::ShaderFeatures::ENABLE_NESTED_CLIPPING => "YC",
        gpu::ShaderFeatures::ENABLE_HSL_BLEND_MODES => "FC",
        gpu::ShaderFeatures::ENABLE_DITHER => "LB",
        _ => panic!("combined or empty shader feature"),
    }
}

fn hasShaderFeature(features: gpu::ShaderFeatures, flag: gpu::ShaderFeatures) -> bool {
    features.0 & flag.0 != 0
}

fn hasMiscFlag(flags: gpu::ShaderMiscFlags, flag: gpu::ShaderMiscFlags) -> bool {
    flags.0 & flag.0 != 0
}

fn newDrawShader(
    context: &mut RenderContextGLImpl,
    shaderType: GLenum,
    drawType: gpu::DrawType,
    shaderFeatures: gpu::ShaderFeatures,
    interlockMode: gpu::InterlockMode,
    shaderMiscFlags: gpu::ShaderMiscFlags,
) -> DrawShader {
    // DISABLE_PLS_ATOMICS is authored by the admitted RIVE_WEBGL branch.
    if interlockMode == gpu::InterlockMode::atomics {
        return DrawShader { m_id: 0 };
    }
    let mut defines = Vec::new();
    if let Some(pls) = context.m_plsImpl.as_ref() {
        pls.pushShaderDefines(interlockMode, &mut defines);
    }
    if hasMiscFlag(
        shaderMiscFlags,
        gpu::ShaderMiscFlags::fixedFunctionColorOutput,
    ) {
        defines.push(GLSL_FIXED_FUNCTION_COLOR_OUTPUT);
    }
    if hasMiscFlag(shaderMiscFlags, gpu::ShaderMiscFlags::clockwiseFill) {
        defines.push(GLSL_CLOCKWISE_FILL);
    }
    if hasMiscFlag(shaderMiscFlags, gpu::ShaderMiscFlags::borrowedCoveragePass) {
        defines.push(GLSL_BORROWED_COVERAGE_PASS);
    }
    for bit in 0..gpu::kShaderFeatureCount {
        let feature = gpu::ShaderFeatures(1 << bit);
        if hasShaderFeature(shaderFeatures, feature) {
            assert!(
                hasShaderFeature(gpu::kVertexShaderFeaturesMask, feature)
                    || shaderType == GL_FRAGMENT_SHADER
            );
            if interlockMode == gpu::InterlockMode::msaa
                && feature == gpu::ShaderFeatures::ENABLE_ADVANCED_BLEND
                && context.m_capabilities.KHR_blend_equation_advanced()
            {
                defines.push(GLSL_ENABLE_KHR_BLEND);
            } else {
                defines.push(shaderFeatureDefine(feature));
            }
        }
    }
    if interlockMode == gpu::InterlockMode::msaa {
        defines.push(GLSL_RENDER_MODE_MSAA);
    }
    assert!(context.platformFeatures().framebufferBottomUp);
    defines.push(GLSL_FRAMEBUFFER_BOTTOM_UP);
    if !context.m_capabilities.ARB_shader_storage_buffer_object() {
        defines.push(GLSL_DISABLE_SHADER_STORAGE_BUFFERS);
    }
    match drawType {
        gpu::DrawType::midpointFanPatches
        | gpu::DrawType::midpointFanCenterAAPatches
        | gpu::DrawType::outerCurvePatches
        | gpu::DrawType::msaaStrokes
        | gpu::DrawType::msaaMidpointFanBorrowedCoverage
        | gpu::DrawType::msaaDynamicMidpointFans
        | gpu::DrawType::msaaMidpointFans
        | gpu::DrawType::msaaMidpointFanStencilReset
        | gpu::DrawType::msaaMidpointFanPathsStencil
        | gpu::DrawType::msaaMidpointFanPathsCover
        | gpu::DrawType::msaaOuterCubics => {
            if shaderType == GL_VERTEX_SHADER {
                defines.push(GLSL_ENABLE_INSTANCE_INDEX);
            }
            defines.push(GLSL_DRAW_PATH);
        }
        gpu::DrawType::clipReset => {}
        gpu::DrawType::interiorTriangulation => defines.push(GLSL_DRAW_INTERIOR_TRIANGLES),
        gpu::DrawType::featherAtlasBlit => defines.push(GLSL_FEATHER_ATLAS_BLIT),
        gpu::DrawType::imageRect => {
            assert_eq!(interlockMode, gpu::InterlockMode::atomics);
            defines.extend([GLSL_DRAW_IMAGE, GLSL_DRAW_IMAGE_RECT]);
        }
        gpu::DrawType::imageMesh => {
            defines.extend([GLSL_DRAW_IMAGE, GLSL_DRAW_IMAGE_MESH]);
        }
        gpu::DrawType::renderPassResolve => {
            assert_eq!(interlockMode, gpu::InterlockMode::atomics);
            defines.extend([GLSL_DRAW_RENDER_TARGET_UPDATE_BOUNDS, GLSL_RESOLVE_PLS]);
            if hasMiscFlag(
                shaderMiscFlags,
                gpu::ShaderMiscFlags::coalescedResolveAndTransfer,
            ) {
                assert_eq!(shaderType, GL_FRAGMENT_SHADER);
                defines.push(GLSL_COALESCED_PLS_RESOLVE_AND_TRANSFER);
            }
        }
        gpu::DrawType::renderPassInitialize => panic!("unreachable draw shader type"),
    }

    let mut sources = vec![
        if context.platformFeatures().avoidFlatVaryings {
            "#define MB\n"
        } else {
            "#define MB flat\n"
        },
        GLSL_CONSTANTS,
        GLSL_FLUSH_UNIFORMS,
        GLSL_COMMON,
    ];
    if shaderType == GL_FRAGMENT_SHADER
        && hasShaderFeature(shaderFeatures, gpu::ShaderFeatures::ENABLE_ADVANCED_BLEND)
    {
        sources.push(GLSL_ADVANCED_BLEND);
    }
    match interlockMode {
        gpu::InterlockMode::rasterOrdering | gpu::InterlockMode::clockwise => match drawType {
            gpu::DrawType::midpointFanPatches
            | gpu::DrawType::midpointFanCenterAAPatches
            | gpu::DrawType::outerCurvePatches
            | gpu::DrawType::interiorTriangulation => {
                sources.extend([GLSL_DRAW_PATH_COMMON, GLSL_DRAW_PATH_VERT]);
                sources.push(if interlockMode == gpu::InterlockMode::clockwise {
                    if hasMiscFlag(shaderMiscFlags, gpu::ShaderMiscFlags::clipUpdateOnly) {
                        GLSL_DRAW_CLOCKWISE_CLIP_FRAG
                    } else {
                        GLSL_DRAW_CLOCKWISE_PATH_FRAG
                    }
                } else {
                    GLSL_DRAW_RASTER_ORDER_PATH_FRAG
                });
            }
            gpu::DrawType::featherAtlasBlit => {
                sources.extend([
                    GLSL_DRAW_PATH_COMMON,
                    GLSL_DRAW_PATH_VERT,
                    GLSL_DRAW_MESH_FRAG,
                ]);
            }
            gpu::DrawType::imageMesh => {
                sources.extend([GLSL_DRAW_IMAGE_MESH_VERT, GLSL_DRAW_MESH_FRAG]);
            }
            _ => panic!("unreachable raster-ordering draw shader"),
        },
        gpu::InterlockMode::atomics => {
            sources.extend([GLSL_DRAW_PATH_COMMON, GLSL_ATOMIC_DRAW]);
        }
        gpu::InterlockMode::msaa => match drawType {
            gpu::DrawType::msaaStrokes
            | gpu::DrawType::msaaMidpointFanBorrowedCoverage
            | gpu::DrawType::msaaDynamicMidpointFans
            | gpu::DrawType::msaaMidpointFans
            | gpu::DrawType::msaaMidpointFanStencilReset
            | gpu::DrawType::msaaMidpointFanPathsStencil
            | gpu::DrawType::msaaMidpointFanPathsCover
            | gpu::DrawType::msaaOuterCubics
            | gpu::DrawType::interiorTriangulation
            | gpu::DrawType::featherAtlasBlit => {
                sources.extend([
                    GLSL_DRAW_PATH_COMMON,
                    GLSL_DRAW_PATH_VERT,
                    GLSL_DRAW_MSAA_OBJECT_FRAG,
                ]);
            }
            gpu::DrawType::clipReset => sources.push(GLSL_STENCIL_DRAW),
            gpu::DrawType::imageMesh => {
                sources.extend([GLSL_DRAW_IMAGE_MESH_VERT, GLSL_DRAW_MSAA_OBJECT_FRAG]);
            }
            _ => panic!("unreachable MSAA draw shader"),
        },
        gpu::InterlockMode::clockwiseAtomic => panic!("unreachable clockwiseAtomic GL mode"),
    }
    DrawShader {
        m_id: glutils::CompileShaderParts(
            shaderType,
            &defines,
            &sources,
            &context.m_capabilities,
            glutils::DebugPrintErrorAndAbort::no,
        ),
    }
}

impl Drop for DrawShader {
    fn drop(&mut self) {
        if self.m_id != 0 {
            recordGLCommand(GLCommand::DeleteShader(self.m_id));
        }
    }
}

fn getVertexShaderSynchronous(
    context: &mut RenderContextGLImpl,
    drawType: gpu::DrawType,
    mut shaderFeatures: gpu::ShaderFeatures,
    interlockMode: gpu::InterlockMode,
) -> *const DrawShader {
    shaderFeatures.0 &= gpu::kVertexShaderFeaturesMask.0;
    let key = crate::mechanical_port::source::renderer::src::gpu_cpp::ShaderUniqueKey(
        drawType,
        shaderFeatures,
        interlockMode,
        gpu::ShaderMiscFlags::none,
    );
    if !context
        .m_pipelineManager
        .base
        .m_vertexShaderMap
        .contains_key(&key)
    {
        context
            .m_pipelineManager
            .base
            .m_vertexShaderMap
            .insert(key, None);
        let shader = Box::new(newDrawShader(
            context,
            GL_VERTEX_SHADER,
            drawType,
            shaderFeatures,
            interlockMode,
            gpu::ShaderMiscFlags::none,
        ));
        context
            .m_pipelineManager
            .base
            .m_vertexShaderMap
            .insert(key, Some(shader));
        context
            .m_pipelineManager
            .base
            .m_sharedObjectReadyCV
            .notify_all();
    }
    context.m_pipelineManager.base.m_vertexShaderMap[&key]
        .as_deref()
        .map(std::ptr::from_ref)
        .expect("vertex shader cache entry is published")
}

fn getFragmentShaderSynchronous(
    context: &mut RenderContextGLImpl,
    drawType: gpu::DrawType,
    shaderFeatures: gpu::ShaderFeatures,
    interlockMode: gpu::InterlockMode,
    miscFlags: gpu::ShaderMiscFlags,
) -> *const DrawShader {
    let key = crate::mechanical_port::source::renderer::src::gpu_cpp::ShaderUniqueKey(
        drawType,
        shaderFeatures,
        interlockMode,
        miscFlags,
    );
    if !context
        .m_pipelineManager
        .base
        .m_fragmentShaderMap
        .contains_key(&key)
    {
        context
            .m_pipelineManager
            .base
            .m_fragmentShaderMap
            .insert(key, None);
        let shader = Box::new(newDrawShader(
            context,
            GL_FRAGMENT_SHADER,
            drawType,
            shaderFeatures,
            interlockMode,
            miscFlags,
        ));
        context
            .m_pipelineManager
            .base
            .m_fragmentShaderMap
            .insert(key, Some(shader));
        context
            .m_pipelineManager
            .base
            .m_sharedObjectReadyCV
            .notify_all();
    }
    context.m_pipelineManager.base.m_fragmentShaderMap[&key]
        .as_deref()
        .map(std::ptr::from_ref)
        .expect("fragment shader cache entry is published")
}

fn newDrawProgram(
    context: &mut RenderContextGLImpl,
    createType: PipelineCreateType,
    drawType: gpu::DrawType,
    shaderFeatures: gpu::ShaderFeatures,
    interlockMode: gpu::InterlockMode,
    shaderMiscFlags: gpu::ShaderMiscFlags,
    #[cfg(feature = "with-rive-tools")] synthesizedFailureType: gpu::SynthesizedFailureType,
) -> DrawProgram {
    let mut program = DrawProgram {
        m_fragmentShader: std::ptr::null(),
        m_vertexShader: std::ptr::null(),
        m_pipelineStatus: PipelineStatus::notReady,
        m_id: 0,
        m_baseInstanceUniformLocation: -1,
        m_state: ManuallyDrop::new((&*context.m_state).clone()),
        #[cfg(feature = "with-rive-tools")]
        m_synthesizedFailureType: synthesizedFailureType,
    };
    #[cfg(feature = "with-rive-tools")]
    if synthesizedFailureType == gpu::SynthesizedFailureType::shaderCompilation {
        program.m_pipelineStatus = PipelineStatus::errored;
        return program;
    }
    program.m_vertexShader =
        getVertexShaderSynchronous(context, drawType, shaderFeatures, interlockMode);
    program.m_fragmentShader = getFragmentShaderSynchronous(
        context,
        drawType,
        shaderFeatures,
        interlockMode,
        shaderMiscFlags,
    );
    program.m_id = createGLProgram();
    unsafe {
        recordGLCommand(GLCommand::AttachShader(
            program.m_id,
            (*program.m_vertexShader).id(),
        ));
        recordGLCommand(GLCommand::AttachShader(
            program.m_id,
            (*program.m_fragmentShader).id(),
        ));
    }
    glutils::LinkProgram(program.m_id, glutils::DebugPrintErrorAndAbort::no);
    let _ = advanceDrawProgram(
        &mut program,
        context,
        createType,
        drawType,
        shaderFeatures,
        interlockMode,
        shaderMiscFlags,
    );
    program
}

fn advanceDrawProgram(
    program: &mut DrawProgram,
    context: &mut RenderContextGLImpl,
    createType: PipelineCreateType,
    drawType: gpu::DrawType,
    shaderFeatures: gpu::ShaderFeatures,
    interlockMode: gpu::InterlockMode,
    shaderMiscFlags: gpu::ShaderMiscFlags,
) -> bool {
    assert_eq!(program.m_pipelineStatus, PipelineStatus::notReady);
    let execution = (&*context.rust_execution).clone();
    if createType == PipelineCreateType::r#async
        && context.m_capabilities.KHR_parallel_shader_compile()
        && execution
            .domain()
            .programParameter(program.m_id, GL_COMPLETION_STATUS_KHR)
            == 0
    {
        return false;
    }
    #[cfg(feature = "with-rive-tools")]
    if program.m_synthesizedFailureType == gpu::SynthesizedFailureType::pipelineCreation {
        program.m_pipelineStatus = PipelineStatus::errored;
        return false;
    }
    if execution
        .domain()
        .programParameter(program.m_id, GL_LINK_STATUS)
        == GL_FALSE as GLint
    {
        if cfg!(debug_assertions) {
            unsafe {
                if execution
                    .domain()
                    .shaderParameter((*program.m_vertexShader).id(), GL_COMPILE_STATUS)
                    == GL_FALSE as GLint
                {
                    glutils::PrintShaderCompilationErrors((*program.m_vertexShader).id());
                }
                if execution
                    .domain()
                    .shaderParameter((*program.m_fragmentShader).id(), GL_COMPILE_STATUS)
                    == GL_FALSE as GLint
                {
                    glutils::PrintShaderCompilationErrors((*program.m_fragmentShader).id());
                }
            }
            glutils::PrintLinkProgramErrors(program.m_id);
        }
        program.m_pipelineStatus = PipelineStatus::errored;
        return false;
    }
    program.m_state.borrow_mut().bindProgram(program.m_id);
    bindUniformBlock(&execution, program.m_id);

    let isImageDraw = gpu::DrawTypeIsImageDraw(drawType);
    let tessellation = isTessellationDraw(drawType);
    let paintDraw = (tessellation
        || drawType == gpu::DrawType::interiorTriangulation
        || drawType == gpu::DrawType::featherAtlasBlit)
        && shaderMiscFlags.0
            & (gpu::ShaderMiscFlags::clipUpdateOnly.0
                | gpu::ShaderMiscFlags::borrowedCoveragePass.0)
            == 0;
    if tessellation {
        glutils::Uniform1iByName(
            program.m_id,
            GLSL_tessVertexTexture,
            TESS_VERTEX_TEXTURE_IDX as GLint,
        );
    }
    if paintDraw || interlockMode == gpu::InterlockMode::atomics {
        glutils::Uniform1iByName(program.m_id, GLSL_gradTexture, GRAD_TEXTURE_IDX as GLint);
    }
    if tessellation && hasShaderFeature(shaderFeatures, gpu::ShaderFeatures::ENABLE_FEATHER) {
        assert!(paintDraw || interlockMode == gpu::InterlockMode::atomics);
        glutils::Uniform1iByName(
            program.m_id,
            GLSL_gaussianIntegralTexture,
            GAUSSIAN_INTEGRAL_TEXTURE_IDX as GLint,
        );
    }
    if drawType == gpu::DrawType::featherAtlasBlit {
        glutils::Uniform1iByName(
            program.m_id,
            GLSL_featherAtlasTexture,
            FEATHER_ATLAS_TEXTURE_IDX as GLint,
        );
    }
    if isImageDraw || paintDraw && interlockMode != gpu::InterlockMode::atomics {
        glutils::Uniform1iByName(program.m_id, GLSL_imageTexture, IMAGE_TEXTURE_IDX as GLint);
    }
    if !context.m_capabilities.ARB_shader_storage_buffer_object() {
        if paintDraw {
            glutils::Uniform1iByName(program.m_id, GLSL_pathBuffer, PATH_BUFFER_IDX as GLint);
        }
        if paintDraw || interlockMode == gpu::InterlockMode::atomics {
            glutils::Uniform1iByName(program.m_id, GLSL_paintBuffer, PAINT_BUFFER_IDX as GLint);
            glutils::Uniform1iByName(
                program.m_id,
                GLSL_paintAuxBuffer,
                PAINT_AUX_BUFFER_IDX as GLint,
            );
        }
        if tessellation {
            glutils::Uniform1iByName(
                program.m_id,
                GLSL_contourBuffer,
                CONTOUR_BUFFER_IDX as GLint,
            );
        }
    }
    if interlockMode == gpu::InterlockMode::msaa
        && hasShaderFeature(shaderFeatures, gpu::ShaderFeatures::ENABLE_ADVANCED_BLEND)
        && !context.m_capabilities.KHR_blend_equation_advanced()
        && !hasMiscFlag(
            shaderMiscFlags,
            gpu::ShaderMiscFlags::fixedFunctionColorOutput,
        )
    {
        glutils::Uniform1iByName(
            program.m_id,
            GLSL_dstColorTexture,
            DST_COLOR_TEXTURE_IDX as GLint,
        );
    }
    if !context
        .m_capabilities
        .ANGLE_base_vertex_base_instance_shader_builtin()
    {
        program.m_baseInstanceUniformLocation = execution
            .domain()
            .uniformLocation(program.m_id, glutils::BASE_INSTANCE_UNIFORM_NAME.as_bytes());
    }
    program.m_pipelineStatus = PipelineStatus::ready;
    true
}

impl Drop for DrawProgram {
    fn drop(&mut self) {
        if self.m_id != 0 {
            self.m_state.borrow_mut().deleteProgram(self.m_id);
        }
        unsafe { ManuallyDrop::drop(&mut self.m_state) };
    }
}

fn createPipeline(
    context: &mut RenderContextGLImpl,
    createType: PipelineCreateType,
    props: &StandardPipelineProps,
) -> Box<DrawProgram> {
    Box::new(newDrawProgram(
        context,
        createType,
        props.drawType,
        props.shaderFeatures,
        props.interlockMode,
        props.shaderMiscFlags,
        #[cfg(feature = "with-rive-tools")]
        props.synthesizedFailureType,
    ))
}

fn pipelinePointer(context: &RenderContextGLImpl, key: u32) -> Option<*const DrawProgram> {
    context
        .m_pipelineManager
        .base
        .m_pipelines
        .get(&key)
        .and_then(Option::as_deref)
        .map(std::ptr::from_ref)
}

fn tryGetPipeline(
    context: &mut RenderContextGLImpl,
    input: &StandardPipelineProps,
) -> Option<*const DrawProgram> {
    let mut props = *input;
    let ubershaderFeatures = gpu::UbershaderFeaturesMaskFor(
        props.shaderFeatures,
        props.drawType,
        props.interlockMode,
        props.shaderMiscFlags,
        context.platformFeatures(),
    );
    let createType = match context.m_pipelineManager.base.m_mode {
        ShaderCompilationMode::allowAsynchronous => {
            if props.shaderFeatures == ubershaderFeatures {
                PipelineCreateType::sync
            } else {
                PipelineCreateType::r#async
            }
        }
        ShaderCompilationMode::onlyUbershaders => {
            props.shaderFeatures = ubershaderFeatures;
            PipelineCreateType::sync
        }
        ShaderCompilationMode::alwaysSynchronous => PipelineCreateType::sync,
    };
    let key = props.createKey(context.platformFeatures());
    #[cfg(feature = "with-rive-tools")]
    {
        if props.synthesizedFailureType == gpu::SynthesizedFailureType::ubershaderLoad
            && props.drawType != gpu::DrawType::renderPassResolve
        {
            return None;
        }
        if props.shaderFeatures == ubershaderFeatures {
            props.synthesizedFailureType = gpu::SynthesizedFailureType::none;
        }
    }
    if !context
        .m_pipelineManager
        .base
        .m_pipelines
        .contains_key(&key)
    {
        let pipeline = createPipeline(context, createType, &props);
        context
            .m_pipelineManager
            .base
            .m_pipelines
            .insert(key, Some(pipeline));
    }
    if let Some(pointer) = pipelinePointer(context, key) {
        let status = unsafe { (*pointer).m_pipelineStatus };
        if status == PipelineStatus::notReady {
            let mut pipeline = context
                .m_pipelineManager
                .base
                .m_pipelines
                .remove(&key)
                .and_then(|entry| entry)
                .expect("pending GL pipeline");
            let _ = advanceDrawProgram(
                &mut pipeline,
                context,
                createType,
                props.drawType,
                props.shaderFeatures,
                props.interlockMode,
                props.shaderMiscFlags,
            );
            context
                .m_pipelineManager
                .base
                .m_pipelines
                .insert(key, Some(pipeline));
        }
    }
    if let Some(pointer) = pipelinePointer(context, key) {
        match unsafe { (*pointer).m_pipelineStatus } {
            PipelineStatus::ready => return Some(pointer),
            PipelineStatus::notReady | PipelineStatus::errored => {}
        }
    }
    if props.shaderFeatures == ubershaderFeatures {
        return None;
    }
    props.shaderFeatures = ubershaderFeatures;
    tryGetPipeline(context, &props)
}

fn queuePipelineIfNotFound(
    context: &mut RenderContextGLImpl,
    props: &StandardPipelineProps,
) -> bool {
    let key = props.createKey(context.platformFeatures());
    if context
        .m_pipelineManager
        .base
        .m_pipelines
        .contains_key(&key)
    {
        return false;
    }
    let pipeline = createPipeline(context, PipelineCreateType::r#async, props);
    context
        .m_pipelineManager
        .base
        .m_pipelines
        .insert(key, Some(pipeline));
    true
}

fn clearPipelineCache(context: &mut RenderContextGLImpl) {
    let manager = &mut context.m_pipelineManager.base;
    manager.m_jobQueue.clear();
    assert_eq!(manager.m_activePipelineCreationCount, 0);
    manager.m_completedJobs.clear();
    manager.m_pipelines.clear();
    manager.m_fragmentShaderMap.clear();
    manager.m_vertexShaderMap.clear();
}

fn instanceChunks(
    instanceCount: u32,
    baseInstance: u32,
    maxInstancesPerChunk: u32,
) -> impl Iterator<Item = (u32, u32)> {
    assert_ne!(maxInstancesPerChunk, 0);
    let mut remaining = instanceCount;
    let mut base = baseInstance;
    std::iter::from_fn(move || {
        if remaining == 0 {
            return None;
        }
        let count = remaining.min(maxInstancesPerChunk);
        let result = (count, base);
        remaining -= count;
        base = base.wrapping_add(count);
        Some(result)
    })
}

unsafe fn glBufferId(bufferRing: *mut BufferRing) -> GLuint {
    assert!(!bufferRing.is_null());
    unsafe { (*bufferRing.cast::<BufferRingGLImpl>()).bufferID() }
}

fn setImageDrawInstanceAttribs(byteOffset: usize) {
    let stride = std::mem::size_of::<gpu::ImageDrawInstance>() as GLsizei;
    let word = std::mem::size_of::<u32>();
    assert_eq!(std::mem::size_of::<gpu::ImageDrawInstance>(), word * 16);
    for (index, offset) in [
        (IMAGE_VIEW_MATRIX_ATTRIB_IDX, byteOffset),
        (
            IMAGE_CLIP_RECT_INVERSE_MATRIX_ATTRIB_IDX,
            byteOffset + word * 4,
        ),
        (IMAGE_TRANSLATES_ATTRIB_IDX, byteOffset + word * 8),
    ] {
        recordGLCommand(GLCommand::VertexAttribPointer {
            index,
            size: 4,
            type_: GL_FLOAT,
            normalized: GL_FALSE,
            stride,
            offset: u32::try_from(offset).expect("WebGL vertex attribute offset fits u32"),
        });
    }
    recordGLCommand(GLCommand::VertexAttribIPointer {
        index: IMAGE_PACKED_ATTRIBS_IDX,
        size: 4,
        type_: GL_UNSIGNED_INT,
        stride,
        offset: u32::try_from(byteOffset + word * 12)
            .expect("WebGL integer attribute offset fits u32"),
    });
}

unsafe fn bindStorageBuffer(
    capabilities: &GLCapabilities,
    bufferRing: *mut BufferRing,
    bindingIdx: GLuint,
    bindingSizeInBytes: usize,
    offsetInBytes: usize,
) {
    assert!(!bufferRing.is_null());
    assert_ne!(bindingSizeInBytes, 0);
    if capabilities.ARB_shader_storage_buffer_object() {
        unsafe {
            (&*bufferRing.cast::<StorageBufferRingGLImpl>()).bindToRenderContext(
                bindingIdx,
                bindingSizeInBytes,
                offsetInBytes,
            )
        };
    } else {
        unsafe {
            (&*bufferRing.cast::<TexelBufferRingWebGL>()).bindToRenderContext(
                bindingIdx,
                bindingSizeInBytes,
                offsetInBytes,
            )
        };
    }
}

unsafe fn renderTargetGL<'a>(
    target: core::ptr::NonNull<RenderTarget>,
    execution: &GLExecutionStamp,
) -> &'a mut dyn RenderTargetGLApi {
    let targetBase = unsafe { target.as_ref() };
    assert!(
        targetBase.belongs_to_owner_thread_execution(
            execution.domain().key(),
            execution.generation(),
        ),
        "RenderContextGLImpl received a non-GL, stale, or foreign render target"
    );
    let base = target.as_ptr().cast::<RenderTargetGL>();
    match unsafe { (&*base).liteTypeID() } {
        TEXTURE_RENDER_TARGET_GL_LITE_RTTI_TYPE_ID => unsafe {
            &mut *base.cast::<TextureRenderTargetGL>()
        },
        FRAMEBUFFER_RENDER_TARGET_GL_LITE_RTTI_TYPE_ID => unsafe {
            &mut *base.cast::<FramebufferRenderTargetGL>()
        },
        other => panic!("RenderContextGLImpl received non-GL render target type {other:#x}"),
    }
}

pub(crate) unsafe fn withRenderTargetGL<R>(
    target: core::ptr::NonNull<RenderTarget>,
    execution: &GLExecutionStamp,
    callback: impl FnOnce(&mut dyn RenderTargetGLApi) -> R,
) -> R {
    callback(unsafe { renderTargetGL(target, execution) })
}

unsafe fn textureGL<'a>(
    texture: core::ptr::NonNull<gpu::Texture>,
    execution: &GLExecutionStamp,
) -> &'a TextureGLImpl {
    let textureBase = unsafe { texture.as_ref() };
    assert!(
        textureBase.belongs_to_owner_thread_execution(
            execution.domain().key(),
            execution.generation(),
        ),
        "RenderContextGLImpl received a non-GL, stale, or foreign texture"
    );
    unsafe { &*texture.as_ptr().cast::<TextureGLImpl>() }
}

/// Temporarily moves the final PLS owner out of `RenderContextGLImpl` while
/// source virtual callbacks receive the complete mutable context. The C++
/// object model permits that callback shape; keeping a Rust borrow into
/// `m_plsImpl` at the same time would alias the whole-context borrow.
struct DetachedPixelLocalStorage<'a> {
    context: &'a mut RenderContextGLImpl,
    value: Option<Box<dyn PixelLocalStorageImpl>>,
}

impl<'a> DetachedPixelLocalStorage<'a> {
    fn take(context: &'a mut RenderContextGLImpl) -> Self {
        let value = context.m_plsImpl.take();
        Self { context, value }
    }

    fn call<R>(
        &mut self,
        message: &str,
        callback: impl FnOnce(&mut dyn PixelLocalStorageImpl, &mut RenderContextGLImpl) -> R,
    ) -> R {
        let Self { context, value } = self;
        callback(value.as_deref_mut().expect(message), context)
    }
}

impl Drop for DetachedPixelLocalStorage<'_> {
    fn drop(&mut self) {
        assert!(
            self.context.m_plsImpl.is_none(),
            "detached PLS slot must stay empty during source callbacks"
        );
        *self.context.m_plsImpl = self.value.take();
    }
}

fn withDetachedPixelLocalStorage<R>(
    context: &mut RenderContextGLImpl,
    message: &str,
    callback: impl FnOnce(&mut dyn PixelLocalStorageImpl, &mut RenderContextGLImpl) -> R,
) -> R {
    let mut detached = DetachedPixelLocalStorage::take(context);
    detached.call(message, callback)
}

fn intersectScissor(a: gpu::AABBu16, b: gpu::AABBu16) -> gpu::AABBu16 {
    let left = a.left.max(b.left);
    let top = a.top.max(b.top);
    let right = a.right.min(b.right);
    let bottom = a.bottom.min(b.bottom);
    if left >= right || top >= bottom {
        gpu::AABBu16 {
            left,
            top,
            right: left,
            bottom: top,
        }
    } else {
        gpu::AABBu16 {
            left,
            top,
            right,
            bottom,
        }
    }
}

fn boundsToU16(bounds: gpu::IAABB) -> gpu::AABBu16 {
    gpu::AABBu16 {
        left: u16::try_from(bounds.left).expect("update bound left is losslessly u16"),
        top: u16::try_from(bounds.top).expect("update bound top is losslessly u16"),
        right: u16::try_from(bounds.right).expect("update bound right is losslessly u16"),
        bottom: u16::try_from(bounds.bottom).expect("update bound bottom is losslessly u16"),
    }
}

fn makeWH(width: u32, height: u32) -> gpu::IAABB {
    gpu::IAABB {
        left: 0,
        top: 0,
        right: i32::try_from(width).expect("render target width fits i32"),
        bottom: i32::try_from(height).expect("render target height fits i32"),
    }
}

fn intersectBounds(a: gpu::IAABB, b: gpu::IAABB) -> gpu::IAABB {
    let left = a.left.max(b.left);
    let top = a.top.max(b.top);
    let right = a.right.min(b.right);
    let bottom = a.bottom.min(b.bottom);
    gpu::IAABB {
        left,
        top,
        right: right.max(left),
        bottom: bottom.max(top),
    }
}

pub(crate) fn unpackColorToRGBA32FPremul(color: u32) -> [f32; 4] {
    let alpha = ((color >> 24) & 0xff) as f32 * (1.0 / 255.0);
    [
        ((color >> 16) & 0xff) as f32 * (1.0 / 255.0) * alpha,
        ((color >> 8) & 0xff) as f32 * (1.0 / 255.0) * alpha,
        (color & 0xff) as f32 * (1.0 / 255.0) * alpha,
        alpha,
    ]
}

fn drawIndexedInstancedNoInstancedAttribs(
    context: &mut RenderContextGLImpl,
    primitiveTopology: GLenum,
    indexCount: u32,
    baseIndex: u32,
    instanceCount: u32,
    baseInstance: u32,
    baseInstanceUniformLocation: GLint,
    flushInjector: &mut GLFlushInjector,
) {
    assert_eq!(
        context
            .m_capabilities
            .ANGLE_base_vertex_base_instance_shader_builtin(),
        baseInstanceUniformLocation < 0
    );
    let indexOffset = baseIndex
        .checked_mul(std::mem::size_of::<u16>() as u32)
        .expect("index byte offset fits u32");
    for (chunkInstanceCount, chunkBaseInstance) in instanceChunks(
        instanceCount,
        baseInstance,
        context.m_capabilities.maxSupportedInstancesPerFlush,
    ) {
        flushInjector.flushBeforeInstancedDrawIfNeeded(chunkInstanceCount);
        // The admitted RIVE_WEBGL branch deliberately excludes the native
        // EXT base-instance draw and supplies gl_BaseInstance by uniform.
        recordGLCommand(GLCommand::Uniform1iLocation {
            location: baseInstanceUniformLocation,
            value: chunkBaseInstance as GLint,
        });
        recordGLCommand(GLCommand::DrawElementsInstanced {
            mode: primitiveTopology,
            count: indexCount,
            type_: GL_UNSIGNED_SHORT,
            offset: indexOffset,
            instanceCount: chunkInstanceCount,
        });
    }
}

pub(crate) fn blitTextureToFramebufferAsDraw(
    context: &mut RenderContextGLImpl,
    textureID: GLuint,
    bounds: &gpu::IAABB,
    renderTargetHeight: u32,
) {
    let execution = (&*context.rust_execution).clone();
    execution.withCurrent(|| {
        if context.m_blitAsDrawProgram.id() == 0 {
            let sources = [
                GLSL_CONSTANTS,
                GLSL_FLUSH_UNIFORMS,
                GLSL_COMMON,
                GLSL_BLIT_TEXTURE_AS_DRAW,
            ];
            context.m_blitAsDrawProgram.moveAssign(Program::new());
            context.m_blitAsDrawProgram.compileAndAttachShaderParts(
                GL_VERTEX_SHADER,
                &[],
                &sources,
                &context.m_capabilities,
            );
            context.m_blitAsDrawProgram.compileAndAttachShaderParts(
                GL_FRAGMENT_SHADER,
                &[],
                &sources,
                &context.m_capabilities,
            );
            context.m_blitAsDrawProgram.link();
            context
                .m_state
                .borrow_mut()
                .bindProgram(context.m_blitAsDrawProgram.id());
            glutils::Uniform1iByName(context.m_blitAsDrawProgram.id(), GLSL_sourceTexture, 0);
        }
        let mut state = context.m_state.borrow_mut();
        state.setPipelineState(&gpu::COLOR_ONLY_PIPELINE_STATE, ScissorAction::ignore);
        state.setScissor(*bounds, renderTargetHeight);
        state.bindProgram(context.m_blitAsDrawProgram.id());
        state.bindVAO(context.m_emptyVAO.id());
        drop(state);
        recordGLCommand(GLCommand::ActiveTexture(GL_TEXTURE0));
        recordGLCommand(GLCommand::BindTexture(GL_TEXTURE_2D, textureID));
        recordGLCommand(GLCommand::DrawArrays {
            mode: GL_TRIANGLE_STRIP,
            first: 0,
            count: 4,
        });
    });
}

#[cfg(feature = "with-rive-tools")]
pub(crate) fn testingOnly_resetFeatherAtlasDesiredRenderType(
    context: &mut RenderContextGLImpl,
    owningRenderContext: &mut RenderContext,
    desiredRenderType: FeatherAtlasRenderType,
) -> FeatherAtlasRenderType {
    let execution = (&*context.rust_execution).clone();
    execution.withCurrent(|| {
        owningRenderContext.releaseResources();
        assert_eq!(context.m_featherAtlasRenderTexture.id(), 0);
        assert_eq!(context.m_featherAtlasTexture.id(), 0);
        assert_eq!(context.m_featherAtlasRenderFBO.id(), 0);
        assert_eq!(context.m_featherAtlasResolveFBO.id(), 0);

        context.m_featherAtlasVertexShader.reset(0);
        context
            .m_featherAtlasFillProgram
            .m_program
            .moveAssign(Program::Zero());
        context
            .m_featherAtlasFillProgram
            .m_baseInstanceUniformLocation = -1;
        context
            .m_featherAtlasStrokeProgram
            .m_program
            .moveAssign(Program::Zero());
        context
            .m_featherAtlasStrokeProgram
            .m_baseInstanceUniformLocation = -1;
        context.m_featherAtlasResolveVertexShader.reset(0);
        context
            .m_featherAtlasClearProgram
            .moveAssign(Program::Zero());
        context
            .m_featherAtlasResolveProgram
            .moveAssign(Program::Zero());
        clearPipelineCache(context);

        let previous = context.m_featherAtlasRenderType;
        context.m_featherAtlasRenderType =
            selectFeatherAtlasRenderType(&context.m_capabilities, desiredRenderType);
        previous
    })
}

#[cfg(feature = "with-rive-tools")]
pub(crate) fn testingOnly_setBlendAdvancedCoherentKHRSupported(
    context: &mut RenderContextGLImpl,
    supported: bool,
) -> bool {
    let execution = (&*context.rust_execution).clone();
    execution.withCurrent(|| {
        let previous = context.m_capabilities.KHR_blend_equation_advanced_coherent();
        assert_eq!(
            previous,
            context
                .base
                .base
                .m_platformFeatures
                .supportsBlendAdvancedCoherentKHR
        );
        context
            .m_capabilities
            .setKHR_blend_equation_advanced_coherent(supported);
        context
            .base
            .base
            .m_platformFeatures
            .supportsBlendAdvancedCoherentKHR = supported;
        clearPipelineCache(context);
        previous
    })
}

#[cfg(feature = "with-rive-tools")]
pub(crate) fn testingOnly_setBlendAdvancedKHRSupported(
    context: &mut RenderContextGLImpl,
    supported: bool,
) -> bool {
    let execution = (&*context.rust_execution).clone();
    execution.withCurrent(|| {
        let previous = context.m_capabilities.KHR_blend_equation_advanced();
        assert_eq!(
            previous,
            context
                .base
                .base
                .m_platformFeatures
                .supportsBlendAdvancedKHR
        );
        context
            .m_capabilities
            .setKHR_blend_equation_advanced(supported);
        context
            .base
            .base
            .m_platformFeatures
            .supportsBlendAdvancedKHR = supported;
        clearPipelineCache(context);
        previous
    })
}

pub(crate) unsafe fn preBeginFrame(
    context: &mut RenderContextGLImpl,
    renderContext: *mut RenderContext,
) {
    if !context.m_testForAdvancedBlendError {
        return;
    }
    assert!(!renderContext.is_null());
    let execution = (&*context.rust_execution).clone();
    execution.withCurrent(|| unsafe {
        // Set before the nested beginFrame call so this exact one-shot test
        // cannot recursively invoke itself.
        context.m_testForAdvancedBlendError = false;
        const RT_WIDTH: u32 = 4;
        const RT_HEIGHT: u32 = 4;
        const RT_CLEAR_COLOR: u32 = 0x8000ffff;
        let frame = FrameDescriptor {
            renderTargetWidth: RT_WIDTH,
            renderTargetHeight: RT_HEIGHT,
            clearColor: RT_CLEAR_COLOR,
            ..FrameDescriptor::default()
        };

        let texture = GLTexture::new();
        recordGLCommand(GLCommand::ActiveTexture(GL_TEXTURE0));
        recordGLCommand(GLCommand::BindTexture(GL_TEXTURE_2D, texture.id()));
        recordGLCommand(GLCommand::TexStorage2D {
            target: GL_TEXTURE_2D,
            levels: 1,
            internal_format: GL_RGBA8,
            width: RT_WIDTH,
            height: RT_HEIGHT,
        });
        let mut target = TextureRenderTargetGL::new(RT_WIDTH, RT_HEIGHT, execution.clone());
        target.assertSameExecution(&execution);
        target.setTargetTexture(texture.id());

        let renderContext = &mut *renderContext;
        renderContext.beginFrameExecutable(&frame);
        let mut renderer = RiveRenderer::new(renderContext);
        const RT_QUAD_FILL_COLOR: u32 = 0x80404000;
        let paint = renderContext.makeRenderPaint();
        let paintImpl = &mut *paint.get().cast::<RiveRenderPaint>();
        paintImpl.style(nuxie_render_api::RenderPaintStyle::Fill);
        paintImpl.color(RT_QUAD_FILL_COLOR);
        paintImpl.blendMode(nuxie_render_api::BlendMode::ColorBurn);

        let path = renderContext.makeEmptyRenderPath();
        let pathImpl = &mut *path.get().cast::<RiveRenderPath>();
        RenderPathContract::fillRule(pathImpl, nuxie_render_api::FillRule::Clockwise);
        RenderPathContract::moveTo(pathImpl, -1.0, -1.0);
        RenderPathContract::lineTo(pathImpl, (RT_WIDTH + 1) as f32, -1.0);
        RenderPathContract::lineTo(pathImpl, (RT_WIDTH + 1) as f32, (RT_HEIGHT + 1) as f32);
        RenderPathContract::lineTo(pathImpl, -1.0, (RT_HEIGHT + 1) as f32);
        renderer.drawPath(path.get(), paint.get());

        renderContext.flushExecutable(&FlushResources {
            renderTarget: (&mut *target.base.base) as *mut RenderTarget,
            ..FlushResources::default()
        });
        target.bindDestinationFramebuffer(GL_READ_FRAMEBUFFER);
        let pixel = execution.domain().readPixelsRGBA8(1, 1, 1, 1);
        assert_eq!(pixel.len(), 4);
        const EXPECTED_COLOR: [u8; 4] = [0x10, 0x90, 0x80, 0xc0];
        let maxDiff = (0..3)
            .map(|index| (pixel[index] as i32 - EXPECTED_COLOR[index] as i32).abs())
            .max()
            .unwrap();
        if maxDiff > 40 {
            context
                .m_capabilities
                .setKHR_blend_equation_advanced_coherent(false);
            context
                .m_capabilities
                .setKHR_blend_equation_advanced(false);
            context
                .base
                .base
                .m_platformFeatures
                .supportsBlendAdvancedCoherentKHR = false;
            context
                .base
                .base
                .m_platformFeatures
                .supportsBlendAdvancedKHR = false;
            clearPipelineCache(context);
        }
    });
}

pub(crate) unsafe fn flush(context: &mut RenderContextGLImpl, desc: &gpu::FlushDescriptor) {
    let execution = (&*context.rust_execution).clone();
    execution.withCurrent(|| unsafe {
        assert_ne!(desc.interlockMode, gpu::InterlockMode::clockwiseAtomic);
        let renderTargetHandle = desc
            .renderTarget
            .expect("RenderContextGLImpl flush requires a render target");
        renderTargetGL(renderTargetHandle, &execution)
            .base()
            .assertSameExecution(&execution);

        recordGLCommand(GLCommand::BindBufferRange {
            target: GL_UNIFORM_BUFFER,
            index: FLUSH_UNIFORM_BUFFER_IDX,
            buffer: glBufferId(context.base.flushUniformBufferRing()),
            offset: u32::try_from(desc.flushUniformDataOffsetInBytes)
                .expect("flush uniform offset fits WebGL"),
            size: u32::try_from(std::mem::size_of::<gpu::FlushUniforms>())
                .expect("FlushUniforms size fits WebGL"),
        });

        if desc.pathCount > 0 {
            bindStorageBuffer(
                &context.m_capabilities,
                context.base.pathBufferRing(),
                PATH_BUFFER_IDX,
                desc.pathCount as usize * std::mem::size_of::<gpu::PathData>(),
                desc.firstPath * std::mem::size_of::<gpu::PathData>(),
            );
            bindStorageBuffer(
                &context.m_capabilities,
                context.base.paintBufferRing(),
                PAINT_BUFFER_IDX,
                desc.pathCount as usize * std::mem::size_of::<gpu::PaintData>(),
                desc.firstPaint * std::mem::size_of::<gpu::PaintData>(),
            );
            bindStorageBuffer(
                &context.m_capabilities,
                context.base.paintAuxBufferRing(),
                PAINT_AUX_BUFFER_IDX,
                desc.pathCount as usize * std::mem::size_of::<gpu::PaintAuxData>(),
                desc.firstPaintAux * std::mem::size_of::<gpu::PaintAuxData>(),
            );
        }
        if desc.contourCount > 0 {
            bindStorageBuffer(
                &context.m_capabilities,
                context.base.contourBufferRing(),
                CONTOUR_BUFFER_IDX,
                desc.contourCount as usize * std::mem::size_of::<gpu::ContourData>(),
                desc.firstContour * std::mem::size_of::<gpu::ContourData>(),
            );
        }

        let mut flushInjector = GLFlushInjector::new(&context.m_capabilities);

        if desc.gradSpanCount > 0 {
            if context.m_capabilities.isPowerVR() {
                recordGLCommand(GLCommand::ActiveTexture(GL_TEXTURE0 + GRAD_TEXTURE_IDX));
                recordGLCommand(GLCommand::TexSubImage2D {
                    target: GL_TEXTURE_2D,
                    level: 0,
                    x: 0,
                    y: 0,
                    width: 1,
                    height: 1,
                    format: GL_RGBA,
                    type_: GL_UNSIGNED_BYTE,
                    data: 0u32.to_ne_bytes().to_vec(),
                });
            }
            let mut state = context.m_state.borrow_mut();
            state.bindProgram(context.m_colorRampProgram.id());
            recordGLCommand(GLCommand::BindFramebuffer(
                GL_FRAMEBUFFER,
                context.m_colorRampFBO.id(),
            ));
            recordGLCommand(GLCommand::Viewport(
                0,
                0,
                gpu::kGradTextureWidth as i32,
                desc.gradDataHeight as i32,
            ));
            state.setPipelineState(&gpu::COLOR_ONLY_PIPELINE_STATE, ScissorAction::disable);
            state.bindBuffer(GL_ARRAY_BUFFER, glBufferId(context.base.gradSpanBufferRing()));
            state.bindVAO(context.m_colorRampVAO.id());
            drop(state);
            recordGLCommand(GLCommand::InvalidateFramebuffer {
                target: GL_FRAMEBUFFER,
                attachments: vec![GL_COLOR_ATTACHMENT0],
            });
            for (chunkCount, chunkBase) in instanceChunks(
                desc.gradSpanCount,
                u32::try_from(desc.firstGradSpan).expect("gradient span base fits u32"),
                context.m_capabilities.maxSupportedInstancesPerFlush,
            ) {
                recordGLCommand(GLCommand::VertexAttribIPointer {
                    index: 0,
                    size: 4,
                    type_: GL_UNSIGNED_INT,
                    stride: 0,
                    offset: chunkBase
                        .checked_mul(std::mem::size_of::<gpu::GradientSpan>() as u32)
                        .expect("gradient span offset fits u32"),
                });
                flushInjector.flushBeforeInstancedDrawIfNeeded(chunkCount);
                recordGLCommand(GLCommand::DrawArraysInstanced {
                    mode: GL_TRIANGLE_STRIP,
                    first: 0,
                    count: gpu::GRAD_SPAN_TRI_STRIP_VERTEX_COUNT,
                    instanceCount: chunkCount,
                });
            }
        }

        if desc.tessVertexSpanCount > 0 {
            let mut state = context.m_state.borrow_mut();
            state.bindProgram(context.m_tessellateProgram.id());
            recordGLCommand(GLCommand::BindFramebuffer(
                GL_FRAMEBUFFER,
                context.m_tessellateFBO.id(),
            ));
            recordGLCommand(GLCommand::Viewport(
                0,
                0,
                gpu::kTessTextureWidth as i32,
                desc.tessDataHeight as i32,
            ));
            state.setPipelineState(&gpu::COLOR_ONLY_PIPELINE_STATE, ScissorAction::disable);
            state.bindBuffer(GL_ARRAY_BUFFER, glBufferId(context.base.tessSpanBufferRing()));
            state.bindVAO(context.m_tessellateVAO.id());
            drop(state);
            recordGLCommand(GLCommand::InvalidateFramebuffer {
                target: GL_FRAMEBUFFER,
                attachments: vec![GL_COLOR_ATTACHMENT0],
            });
            for (chunkCount, chunkBase) in instanceChunks(
                desc.tessVertexSpanCount,
                u32::try_from(desc.firstTessVertexSpan).expect("tess span base fits u32"),
                context.m_capabilities.maxSupportedInstancesPerFlush,
            ) {
                let spanOffset = chunkBase as usize * std::mem::size_of::<gpu::TessVertexSpan>();
                for index in 0..3 {
                    recordGLCommand(GLCommand::VertexAttribPointer {
                        index,
                        size: 4,
                        type_: GL_FLOAT,
                        normalized: GL_FALSE,
                        stride: std::mem::size_of::<gpu::TessVertexSpan>() as GLsizei,
                        offset: u32::try_from(spanOffset + index as usize * 4 * 4)
                            .expect("tess span attribute offset fits u32"),
                    });
                }
                recordGLCommand(GLCommand::VertexAttribIPointer {
                    index: 3,
                    size: 4,
                    type_: GL_UNSIGNED_INT,
                    stride: std::mem::size_of::<gpu::TessVertexSpan>() as GLsizei,
                    offset: u32::try_from(
                        spanOffset + std::mem::offset_of!(gpu::TessVertexSpan, x0x1),
                    )
                    .expect("tess span integer attribute offset fits u32"),
                });
                flushInjector.flushBeforeInstancedDrawIfNeeded(chunkCount);
                recordGLCommand(GLCommand::DrawElementsInstanced {
                    mode: GL_TRIANGLES,
                    count: gpu::kTessSpanIndices.len() as u32,
                    type_: GL_UNSIGNED_SHORT,
                    offset: 0,
                    instanceCount: chunkCount,
                });
            }
        }

        if desc.featherAtlasFillBatchCount | desc.featherAtlasStrokeBatchCount != 0 {
            context.m_state.borrow_mut().setPipelineState(
                &gpu::COLOR_ONLY_PIPELINE_STATE,
                ScissorAction::ignore,
            );
            recordGLCommand(GLCommand::BindFramebuffer(
                GL_FRAMEBUFFER,
                context.m_featherAtlasRenderFBO.id(),
            ));
            recordGLCommand(GLCommand::Viewport(
                0,
                0,
                desc.featherAtlasContentWidth as i32,
                desc.featherAtlasContentHeight as i32,
            ));
            context.m_state.borrow_mut().setScissorRaw(
                0,
                0,
                desc.featherAtlasContentWidth as u32,
                desc.featherAtlasContentHeight as u32,
            );
            recordGLCommand(GLCommand::FrontFace(GL_CCW));

            match context.m_featherAtlasRenderType {
                FeatherAtlasRenderType::r16f
                | FeatherAtlasRenderType::r32f
                | FeatherAtlasRenderType::rgba8 => {
                    recordGLCommand(GLCommand::ClearBufferFloat {
                        buffer: GL_COLOR,
                        drawbuffer: 0,
                        values: [0.0; 4],
                        value_count: 4,
                    });
                }
                FeatherAtlasRenderType::r32uiFramebufferFetch => {
                    recordGLCommand(GLCommand::ClearBufferUInt {
                        buffer: GL_COLOR,
                        drawbuffer: 1,
                        values: [0; 4],
                        value_count: 4,
                    });
                }
                FeatherAtlasRenderType::r32uiPixelLocalStorageANGLE => {
                    recordGLCommand(GLCommand::BeginPixelLocalStorageANGLE {
                        load_ops: vec![GL_LOAD_OP_ZERO_ANGLE],
                    });
                }
                FeatherAtlasRenderType::r8PixelLocalStorageEXT
                | FeatherAtlasRenderType::r32iAtomicTexture => {
                    panic!("feather atlas mode is excluded by RIVE_WEBGL")
                }
            }

            context.m_state.borrow_mut().bindVAO(context.m_drawVAO.id());
            if desc.featherAtlasFillBatchCount != 0 {
                context.m_state.borrow_mut().setPipelineState(
                    &context.m_featherAtlasFillPipelineState,
                    ScissorAction::ignore,
                );
                context
                    .m_state
                    .borrow_mut()
                    .bindProgram(context.m_featherAtlasFillProgram.id());
                let batches = std::slice::from_raw_parts(
                    desc.featherAtlasFillBatches
                        .expect("fill atlas batches accompany nonzero count")
                        .as_ptr(),
                    desc.featherAtlasFillBatchCount,
                );
                for batch in batches {
                    context.m_state.borrow_mut().setScissorRaw(
                        batch.scissor.left as u32,
                        batch.scissor.top as u32,
                        (batch.scissor.right - batch.scissor.left) as u32,
                        (batch.scissor.bottom - batch.scissor.top) as u32,
                    );
                    drawIndexedInstancedNoInstancedAttribs(
                        context,
                        GL_TRIANGLES,
                        gpu::kMidpointFanCenterAAPatchIndexCount,
                        gpu::kMidpointFanCenterAAPatchBaseIndex,
                        batch.patchCount,
                        batch.basePatch,
                        context
                            .m_featherAtlasFillProgram
                            .baseInstanceUniformLocation(),
                        &mut flushInjector,
                    );
                }
            }

            if desc.featherAtlasStrokeBatchCount != 0 {
                context.m_state.borrow_mut().setPipelineState(
                    &context.m_featherAtlasStrokePipelineState,
                    ScissorAction::ignore,
                );
                context
                    .m_state
                    .borrow_mut()
                    .bindProgram(context.m_featherAtlasStrokeProgram.id());
                let batches = std::slice::from_raw_parts(
                    desc.featherAtlasStrokeBatches
                        .expect("stroke atlas batches accompany nonzero count")
                        .as_ptr(),
                    desc.featherAtlasStrokeBatchCount,
                );
                for batch in batches {
                    context.m_state.borrow_mut().setScissorRaw(
                        batch.scissor.left as u32,
                        batch.scissor.top as u32,
                        (batch.scissor.right - batch.scissor.left) as u32,
                        (batch.scissor.bottom - batch.scissor.top) as u32,
                    );
                    drawIndexedInstancedNoInstancedAttribs(
                        context,
                        GL_TRIANGLES,
                        gpu::kMidpointFanPatchBorderIndexCount,
                        gpu::kMidpointFanPatchBaseIndex,
                        batch.patchCount,
                        batch.basePatch,
                        context
                            .m_featherAtlasStrokeProgram
                            .baseInstanceUniformLocation(),
                        &mut flushInjector,
                    );
                }
            }

            if context.m_featherAtlasResolveProgram.id() != 0 {
                if context.m_featherAtlasResolveFBO.id() != 0 {
                    recordGLCommand(GLCommand::BindFramebuffer(
                        GL_FRAMEBUFFER,
                        context.m_featherAtlasResolveFBO.id(),
                    ));
                }
                if context.m_featherAtlasRenderType == FeatherAtlasRenderType::rgba8 {
                    recordGLCommand(GLCommand::ActiveTexture(GL_TEXTURE0));
                    recordGLCommand(GLCommand::BindTexture(
                        GL_TEXTURE_2D,
                        context.m_featherAtlasRenderTexture.id(),
                    ));
                }
                let mut state = context.m_state.borrow_mut();
                state.bindProgram(context.m_featherAtlasResolveProgram.id());
                state.bindVAO(context.m_featherAtlasResolveVAO.id());
                state.setCullFace(GL_NONE);
                state.setScissorRaw(
                    0,
                    0,
                    desc.featherAtlasContentWidth as u32,
                    desc.featherAtlasContentHeight as u32,
                );
                state.disableBlending();
                state.setWriteMasks(true, false, 0);
                drop(state);
                recordGLCommand(GLCommand::DrawArrays {
                    mode: GL_TRIANGLES,
                    first: 0,
                    count: 3,
                });
            }

            match context.m_featherAtlasRenderType {
                FeatherAtlasRenderType::r16f | FeatherAtlasRenderType::r32f => {
                    if context.m_featherAtlasResolveFBO.id() != 0 {
                        recordGLCommand(GLCommand::BindFramebuffer(
                            GL_DRAW_FRAMEBUFFER,
                            context.m_featherAtlasResolveFBO.id(),
                        ));
                        context.m_state.borrow_mut().disableScissor();
                        let width = desc.featherAtlasContentWidth as i32;
                        let height = desc.featherAtlasContentHeight as i32;
                        recordGLCommand(GLCommand::BlitFramebuffer(
                            [0, 0, width, height, 0, 0, width, height],
                            GL_COLOR_BUFFER_BIT,
                            GL_NEAREST,
                        ));
                    }
                }
                FeatherAtlasRenderType::r32uiFramebufferFetch => {
                    recordGLCommand(GLCommand::InvalidateFramebuffer {
                        target: GL_FRAMEBUFFER,
                        attachments: vec![GL_COLOR_ATTACHMENT0],
                    });
                }
                FeatherAtlasRenderType::r32uiPixelLocalStorageANGLE => {
                    recordGLCommand(GLCommand::EndPixelLocalStorageANGLE {
                        store_ops: vec![GL_DONT_CARE],
                    });
                }
                FeatherAtlasRenderType::rgba8 => {}
                FeatherAtlasRenderType::r8PixelLocalStorageEXT
                | FeatherAtlasRenderType::r32iAtomicTexture => {
                    panic!("feather atlas mode is excluded by RIVE_WEBGL")
                }
            }
            recordGLCommand(GLCommand::FrontFace(GL_CW));
        }

        if desc.hasTriangleVertices {
            let mut state = context.m_state.borrow_mut();
            state.bindVAO(context.m_trianglesVAO.id());
            state.bindBuffer(GL_ARRAY_BUFFER, glBufferId(context.base.triangleBufferRing()));
            drop(state);
            recordGLCommand(GLCommand::VertexAttribPointer {
                index: 0,
                size: 3,
                type_: GL_FLOAT,
                normalized: GL_FALSE,
                stride: 0,
                offset: 0,
            });
        }

        let targetWidth = renderTargetGL(renderTargetHandle, &execution)
            .base()
            .width();
        let targetHeight = renderTargetGL(renderTargetHandle, &execution)
            .base()
            .height();
        recordGLCommand(GLCommand::Viewport(
            0,
            0,
            targetWidth as i32,
            targetHeight as i32,
        ));
        let mut msaaResolveAction = MSAAResolveAction::automatic;
        let mut msaaDepthStencilColor = [GL_NONE; 3];
        let mut clipPlanesEnabled = false;
        if desc.interlockMode != gpu::InterlockMode::msaa {
            assert_eq!(desc.msaaSampleCount, 0);
            withDetachedPixelLocalStorage(
                context,
                "non-MSAA GL flush requires final PLS implementation",
                |pls, context| pls.activatePixelLocalStorage(context, desc),
            );
            if desc.interlockMode == gpu::InterlockMode::atomics {
                withDetachedPixelLocalStorage(
                    context,
                    "atomic mode requires final PLS implementation",
                    |pls, context| pls.ensureRasterOrderingEnabled(context, desc, false),
                );
            }
        } else {
            assert!(desc.msaaSampleCount > 0);
            let preserve = desc.colorLoadAction == gpu::LoadAction::preserveRenderTarget;
            let mut isFBO0 = false;
            msaaResolveAction = renderTargetGL(renderTargetHandle, &execution)
                .bindMSAAFramebuffer(
                    context,
                    desc.msaaSampleCount,
                    preserve.then_some(&desc.renderTargetUpdateBounds),
                    Some(&mut isFBO0),
                );
            msaaDepthStencilColor = if isFBO0 {
                [GL_DEPTH, GL_STENCIL, GL_COLOR]
            } else {
                [GL_DEPTH_ATTACHMENT, GL_STENCIL_ATTACHMENT, GL_COLOR_ATTACHMENT0]
            };
            recordGLCommand(GLCommand::InvalidateFramebuffer {
                target: GL_FRAMEBUFFER,
                attachments: msaaDepthStencilColor[..if preserve { 2 } else { 3 }].to_vec(),
            });
            let mut buffers = GL_STENCIL_BUFFER_BIT | GL_DEPTH_BUFFER_BIT;
            if desc.colorLoadAction == gpu::LoadAction::clear {
                let clear = unpackColorToRGBA32FPremul(desc.colorClearValue);
                recordGLCommand(GLCommand::ClearColor(clear[0], clear[1], clear[2], clear[3]));
                buffers |= GL_COLOR_BUFFER_BIT;
            }
            context
                .m_state
                .borrow_mut()
                .setPipelineState(&gpu::GL_DEFAULT_PIPELINE_STATE, ScissorAction::disable);
            recordGLCommand(GLCommand::Clear(buffers));

            if hasShaderFeature(
                desc.combinedShaderFeatures,
                gpu::ShaderFeatures::ENABLE_ADVANCED_BLEND,
            ) {
                if context.m_capabilities.KHR_blend_equation_advanced_coherent() {
                    recordGLCommand(GLCommand::Enable(GL_BLEND_ADVANCED_COHERENT_KHR));
                } else {
                    let texture = renderTargetGL(renderTargetHandle, &execution)
                        .baseMut()
                        .dstColorTexture();
                    recordGLCommand(GLCommand::ActiveTexture(
                        GL_TEXTURE0 + DST_COLOR_TEXTURE_IDX,
                    ));
                    recordGLCommand(GLCommand::BindTexture(GL_TEXTURE_2D, texture));
                }
            }
        }

        let fullUpdateScissorRect = boundsToU16(desc.renderTargetUpdateBounds);
        let drawList = desc
            .drawList
            .expect("flush descriptor carries a draw list")
            .as_ref();

        for batch in drawList.iter() {
            let drawType = batch.drawType;
            let shaderFeatures = if desc.interlockMode == gpu::InterlockMode::atomics {
                desc.combinedShaderFeatures
            } else {
                batch.shaderFeatures
            };
            let mut shaderMiscFlags = batch.shaderMiscFlags;
            if desc.interlockMode != gpu::InterlockMode::msaa {
                shaderMiscFlags |= context
                    .m_plsImpl
                    .as_deref()
                    .expect("non-MSAA GL flush requires final PLS implementation")
                    .shaderMiscFlags(desc, drawType);
            }
            let props = StandardPipelineProps {
                drawType,
                shaderFeatures,
                interlockMode: desc.interlockMode,
                shaderMiscFlags,
                #[cfg(feature = "with-rive-tools")]
                synthesizedFailureType: desc.synthesizedFailureType,
            };
            let Some(drawProgram) = tryGetPipeline(context, &props) else {
                continue;
            };
            let drawProgram = &*drawProgram;
            context.m_state.borrow_mut().bindProgram(drawProgram.id());

            if let Some(imageTexture) = batch.imageTexture {
                let texture = textureGL(imageTexture, &execution);
                let textureExecution = (&*texture.rust_execution).clone();
                assert!(textureExecution.sameDomain(&execution));
                assert_eq!(textureExecution.generation(), execution.generation());
                recordGLCommand(GLCommand::ActiveTexture(GL_TEXTURE0 + IMAGE_TEXTURE_IDX));
                recordGLCommand(GLCommand::BindTexture(
                    GL_TEXTURE_2D,
                    texture.m_texture.id(),
                ));
                glutils::SetTexture2DSamplingParamsFromSampler(batch.imageSampler);
            }

            let mut pipelineState =
                crate::mechanical_port::source::renderer::src::gpu_cpp::get_pipeline_state_for_batch(
                    batch,
                    desc.interlockMode,
                    desc.fixedFunctionColorOutput,
                    context.platformFeatures(),
            );
            if desc.interlockMode != gpu::InterlockMode::msaa {
                context
                    .m_plsImpl
                    .as_deref()
                    .expect("non-MSAA GL flush requires final PLS implementation")
                    .applyPipelineStateOverrides(
                        batch,
                        desc,
                        context.platformFeatures(),
                        &mut pipelineState,
                    );
            } else {
                let needsClipPlanes = hasShaderFeature(
                    shaderFeatures,
                    gpu::ShaderFeatures::ENABLE_CLIP_RECT,
                );
                if needsClipPlanes != clipPlanesEnabled {
                    for plane in [
                        GL_CLIP_DISTANCE0_EXT,
                        GL_CLIP_DISTANCE1_EXT,
                        GL_CLIP_DISTANCE2_EXT,
                        GL_CLIP_DISTANCE3_EXT,
                    ] {
                        recordGLCommand(if needsClipPlanes {
                            GLCommand::Enable(plane)
                        } else {
                            GLCommand::Disable(plane)
                        });
                    }
                    clipPlanesEnabled = needsClipPlanes;
                }
            }

            if batch.barriers.0
                & (gpu::BarrierFlags::plsAtomic.0 | gpu::BarrierFlags::plsAtomicPreResolve.0)
                != 0
            {
                assert_eq!(desc.interlockMode, gpu::InterlockMode::atomics);
                context
                    .m_plsImpl
                    .as_deref_mut()
                    .expect("atomic barrier requires final PLS implementation")
                    .barrier(desc);
            } else if batch.barriers.0 & gpu::BarrierFlags::dstBlend.0 != 0 {
                assert!(!context.m_capabilities.KHR_blend_equation_advanced_coherent());
                if context.m_capabilities.KHR_blend_equation_advanced() {
                    recordGLCommand(GLCommand::BlendBarrierKHR);
                } else {
                    assert_eq!(desc.interlockMode, gpu::InterlockMode::msaa);
                    assert!(batch.dstReadList.is_some());
                    renderTargetGL(renderTargetHandle, &execution)
                        .baseMut()
                        .bindDstColorFramebuffer(GL_DRAW_FRAMEBUFFER);
                    context.m_state.borrow_mut().disableScissor();
                    if context.m_capabilities.avoidPartialFramebufferBlits {
                        glutils::BlitFramebuffer(
                            makeWH(targetWidth, targetHeight),
                            targetHeight,
                            GL_COLOR_BUFFER_BIT,
                        );
                    } else {
                        let mut draw = batch
                            .dstReadList
                            .map_or(std::ptr::null(), |draw| draw.as_ptr());
                        while !draw.is_null() {
                            assert_ne!((*draw).blendMode(), nuxie_render_api::BlendMode::SrcOver);
                            glutils::BlitFramebuffer(
                                intersectBounds(
                                    desc.renderTargetUpdateBounds,
                                    *(*draw).pixelBounds(),
                                ),
                                targetHeight,
                                GL_COLOR_BUFFER_BIT,
                            );
                            draw = (*draw).nextDstRead();
                        }
                    }
                    let _ = renderTargetGL(renderTargetHandle, &execution)
                        .bindMSAAFramebuffer(context, desc.msaaSampleCount, None, None);
                }
            }

            if let Some(scissor) = batch.scissorRect {
                let scissor = intersectScissor(fullUpdateScissorRect, scissor);
                let mut state = context.m_state.borrow_mut();
                state.setPipelineState(&pipelineState, ScissorAction::ignore);
                state.setScissorU16(scissor, targetHeight);
            } else {
                context
                    .m_state
                    .borrow_mut()
                    .setPipelineState(&pipelineState, ScissorAction::disable);
            }

            match drawType {
                gpu::DrawType::midpointFanPatches
                | gpu::DrawType::midpointFanCenterAAPatches
                | gpu::DrawType::outerCurvePatches
                | gpu::DrawType::msaaStrokes
                | gpu::DrawType::msaaMidpointFanBorrowedCoverage
                | gpu::DrawType::msaaDynamicMidpointFans
                | gpu::DrawType::msaaMidpointFans
                | gpu::DrawType::msaaMidpointFanStencilReset
                | gpu::DrawType::msaaMidpointFanPathsStencil
                | gpu::DrawType::msaaMidpointFanPathsCover
                | gpu::DrawType::msaaOuterCubics => {
                    context.m_state.borrow_mut().bindVAO(context.m_drawVAO.id());
                    if desc.interlockMode == gpu::InterlockMode::rasterOrdering {
                        withDetachedPixelLocalStorage(
                            context,
                            "raster ordering requires final PLS implementation",
                            |pls, context| {
                                pls.ensureRasterOrderingEnabled(context, desc, true)
                            },
                        );
                    }
                    drawIndexedInstancedNoInstancedAttribs(
                        context,
                        GL_TRIANGLES,
                        batch.indexCountPerInstance,
                        batch.baseIndex,
                        batch.elementCount,
                        batch.baseElement,
                        drawProgram.baseInstanceUniformLocation(),
                        &mut flushInjector,
                    );
                }
                gpu::DrawType::clipReset => {
                    context
                        .m_state
                        .borrow_mut()
                        .bindVAO(context.m_trianglesVAO.id());
                    recordGLCommand(GLCommand::DrawArrays {
                        mode: GL_TRIANGLES,
                        first: batch.baseElement,
                        count: batch.elementCount,
                    });
                }
                gpu::DrawType::interiorTriangulation | gpu::DrawType::featherAtlasBlit => {
                    context
                        .m_state
                        .borrow_mut()
                        .bindVAO(context.m_trianglesVAO.id());
                    if desc.interlockMode == gpu::InterlockMode::rasterOrdering {
                        withDetachedPixelLocalStorage(
                            context,
                            "raster ordering requires final PLS implementation",
                            |pls, context| pls.ensureRasterOrderingEnabled(
                                context,
                                desc,
                                drawType != gpu::DrawType::interiorTriangulation,
                            ),
                        );
                    }
                    recordGLCommand(GLCommand::DrawArrays {
                        mode: GL_TRIANGLES,
                        first: batch.baseElement,
                        count: batch.elementCount,
                    });
                    if desc.interlockMode == gpu::InterlockMode::rasterOrdering
                        && drawType != gpu::DrawType::featherAtlasBlit
                    {
                        context
                            .m_plsImpl
                            .as_deref_mut()
                            .expect("raster ordering requires final PLS implementation")
                            .barrier(desc);
                    }
                }
                gpu::DrawType::imageRect => {
                    assert_eq!(desc.interlockMode, gpu::InterlockMode::atomics);
                    assert!(context
                        .m_plsImpl
                        .as_deref()
                        .expect("atomic image draw requires final PLS implementation")
                        .rasterOrderingKnownDisabled());
                    assert_ne!(context.m_imageRectVAO.id(), 0);
                    let mut state = context.m_state.borrow_mut();
                    state.bindVAO(context.m_imageRectVAO.id());
                    state.bindBuffer(
                        GL_ARRAY_BUFFER,
                        glBufferId(context.base.imageDrawInstanceBufferRing()),
                    );
                    drop(state);
                    setImageDrawInstanceAttribs(
                        batch.baseElement as usize
                            * std::mem::size_of::<gpu::ImageDrawInstance>(),
                    );
                    recordGLCommand(GLCommand::DrawElementsInstanced {
                        mode: GL_TRIANGLES,
                        count: batch.indexCountPerInstance,
                        type_: GL_UNSIGNED_SHORT,
                        offset: batch
                            .baseIndex
                            .checked_mul(std::mem::size_of::<u16>() as u32)
                            .expect("image rect index offset fits u32"),
                        instanceCount: batch.elementCount,
                    });
                }
                gpu::DrawType::imageMesh => {
                    let (Some(vertex), Some(uv), Some(index)) =
                        (batch.vertexBuffer, batch.uvBuffer, batch.indexBuffer)
                    else {
                        continue;
                    };
                    if vertex.as_ref().liteTypeID() != RenderBufferGLImpl::LITE_RTTI_TYPE_ID
                        || uv.as_ref().liteTypeID() != RenderBufferGLImpl::LITE_RTTI_TYPE_ID
                        || index.as_ref().liteTypeID() != RenderBufferGLImpl::LITE_RTTI_TYPE_ID
                    {
                        continue;
                    }
                    let vertex = &*vertex.as_ptr().cast::<RenderBufferGLImpl>();
                    let uv = &*uv.as_ptr().cast::<RenderBufferGLImpl>();
                    let index = &*index.as_ptr().cast::<RenderBufferGLImpl>();
                    for buffer in [vertex, uv, index] {
                        let state = buffer.state().borrow();
                        let domain = state
                            .executionDomain()
                            .expect("GL render buffer retains execution domain");
                        assert_eq!(domain.key(), execution.domain().key());
                        assert_eq!(state.m_executionGeneration, execution.generation());
                    }
                    let mut state = context.m_state.borrow_mut();
                    state.bindVAO(context.m_imageMeshVAO.id());
                    state.bindBuffer(GL_ARRAY_BUFFER, vertex.bufferID());
                    drop(state);
                    recordGLCommand(GLCommand::VertexAttribPointer {
                        index: 0,
                        size: 2,
                        type_: GL_FLOAT,
                        normalized: GL_FALSE,
                        stride: 0,
                        offset: 0,
                    });
                    let mut state = context.m_state.borrow_mut();
                    state.bindBuffer(GL_ARRAY_BUFFER, uv.bufferID());
                    drop(state);
                    recordGLCommand(GLCommand::VertexAttribPointer {
                        index: 1,
                        size: 2,
                        type_: GL_FLOAT,
                        normalized: GL_FALSE,
                        stride: 0,
                        offset: 0,
                    });
                    let mut state = context.m_state.borrow_mut();
                    state.bindBuffer(
                        GL_ARRAY_BUFFER,
                        glBufferId(context.base.imageDrawInstanceBufferRing()),
                    );
                    drop(state);
                    setImageDrawInstanceAttribs(
                        batch.baseElement as usize
                            * std::mem::size_of::<gpu::ImageDrawInstance>(),
                    );
                    context
                        .m_state
                        .borrow_mut()
                        .bindBuffer(GL_ELEMENT_ARRAY_BUFFER, index.bufferID());
                    if desc.interlockMode == gpu::InterlockMode::rasterOrdering {
                        withDetachedPixelLocalStorage(
                            context,
                            "raster ordering requires final PLS implementation",
                            |pls, context| {
                                pls.ensureRasterOrderingEnabled(context, desc, true)
                            },
                        );
                    }
                    recordGLCommand(GLCommand::DrawElementsInstanced {
                        mode: GL_TRIANGLES,
                        count: batch.indexCountPerInstance,
                        type_: GL_UNSIGNED_SHORT,
                        offset: batch
                            .baseIndex
                            .checked_mul(std::mem::size_of::<u16>() as u32)
                            .expect("image mesh index offset fits u32"),
                        instanceCount: batch.elementCount,
                    });
                }
                gpu::DrawType::renderPassResolve => {
                    assert_eq!(desc.interlockMode, gpu::InterlockMode::atomics);
                    assert!(context
                        .m_plsImpl
                        .as_deref()
                        .expect("atomic resolve requires final PLS implementation")
                        .rasterOrderingKnownDisabled());
                    context.m_state.borrow_mut().bindVAO(context.m_emptyVAO.id());
                    recordGLCommand(GLCommand::DrawArrays {
                        mode: GL_TRIANGLE_STRIP,
                        first: 0,
                        count: 4,
                    });
                }
                gpu::DrawType::renderPassInitialize => {
                    panic!("renderPassInitialize is never executed as a GL draw batch")
                }
            }
        }

        if desc.interlockMode != gpu::InterlockMode::msaa {
            withDetachedPixelLocalStorage(
                context,
                "non-MSAA GL flush requires final PLS implementation",
                |pls, context| pls.deactivatePixelLocalStorage(context, desc),
            );
        } else {
            recordGLCommand(GLCommand::InvalidateFramebuffer {
                target: GL_FRAMEBUFFER,
                attachments: msaaDepthStencilColor[..2].to_vec(),
            });
            if msaaResolveAction == MSAAResolveAction::framebufferBlit {
                renderTargetGL(renderTargetHandle, &execution)
                    .bindDestinationFramebuffer(GL_DRAW_FRAMEBUFFER);
                context
                    .m_state
                    .borrow_mut()
                    .setPipelineState(&gpu::COLOR_ONLY_PIPELINE_STATE, ScissorAction::disable);
                glutils::BlitFramebuffer(
                    desc.renderTargetUpdateBounds,
                    targetHeight,
                    GL_COLOR_BUFFER_BIT,
                );
                recordGLCommand(GLCommand::InvalidateFramebuffer {
                    target: GL_READ_FRAMEBUFFER,
                    attachments: vec![msaaDepthStencilColor[2]],
                });
            }
            if hasShaderFeature(
                desc.combinedShaderFeatures,
                gpu::ShaderFeatures::ENABLE_ADVANCED_BLEND,
            ) && context.m_capabilities.KHR_blend_equation_advanced_coherent()
            {
                recordGLCommand(GLCommand::Disable(GL_BLEND_ADVANCED_COHERENT_KHR));
            }
            if clipPlanesEnabled {
                for plane in [
                    GL_CLIP_DISTANCE0_EXT,
                    GL_CLIP_DISTANCE1_EXT,
                    GL_CLIP_DISTANCE2_EXT,
                    GL_CLIP_DISTANCE3_EXT,
                ] {
                    recordGLCommand(GLCommand::Disable(plane));
                }
            }
        }

        recordGLCommand(GLCommand::Flush);
        let targetTexture = renderTargetGL(renderTargetHandle, &execution).renderTexture();
        blitMirrorIfRegistered(context, targetTexture);
    });
}

fn glString(bytes: &[u8]) -> &str {
    let bytes = bytes.split(|byte| *byte == 0).next().unwrap_or(bytes);
    std::str::from_utf8(bytes).unwrap_or("")
}

fn parseVersionPair(text: &str, prefix: &str) -> (u32, u32) {
    let Some(version) = text.strip_prefix(prefix) else {
        return (0, 0);
    };
    let mut pieces = version.split(|character: char| !character.is_ascii_digit());
    let major = pieces
        .next()
        .and_then(|piece| piece.parse().ok())
        .unwrap_or(0);
    let minor = pieces
        .next()
        .and_then(|piece| piece.parse().ok())
        .unwrap_or(0);
    (major, minor)
}

fn parsePowerVRVersion(text: &str) -> (u32, u32, u32, u32) {
    let (major, minor) = parseVersionPair(text, "OpenGL ES ");
    let Some(build) = text.split(" build ").nth(1) else {
        return (major, minor, 0, 0);
    };
    let build = build.split('@').next().unwrap_or(build);
    let mut pieces = build.split('.');
    (
        major,
        minor,
        pieces
            .next()
            .and_then(|piece| piece.parse().ok())
            .unwrap_or(0),
        pieces
            .next()
            .and_then(|piece| piece.parse().ok())
            .unwrap_or(0),
    )
}

fn parseAdrenoSeries(renderer: &str) -> u32 {
    renderer
        .strip_prefix("Adreno (TM) ")
        .and_then(|suffix| {
            suffix
                .split(|character: char| !character.is_ascii_digit())
                .next()
        })
        .and_then(|digits| digits.parse().ok())
        .unwrap_or(0)
}

fn makeContextOwnerInCurrent(
    options: ContextOptions,
    executionDomain: GLExecutionDomain,
    finalPLSFactory: Option<&dyn PixelLocalStorageFactory>,
) -> Option<Box<RenderContextGLImpl>> {
    let glVersionBytes = executionDomain
        .getString(GL_VERSION)
        .expect("source dereferences non-null glGetString(GL_VERSION)");
    let rendererToken = if executionDomain.enableWebGLExtension("WEBGL_debug_renderer_info") {
        GL_UNMASKED_RENDERER_WEBGL
    } else {
        GL_RENDERER
    };
    let rendererBytes = executionDomain
        .getString(rendererToken)
        .expect("source dereferences non-null GL renderer string");
    let glVersion = glString(&glVersionBytes);
    let renderer = glString(&rendererBytes);

    let mut capabilities = GLCapabilities::default();
    capabilities.setIsGLES(true);
    capabilities.setIsANGLESystemDriver(renderer.contains("ANGLE"));
    capabilities.setIsAdreno(renderer.contains("Adreno"));
    capabilities.setIsMali(renderer.contains("Mali"));
    capabilities.setIsPowerVR(renderer.contains("PowerVR"));
    if capabilities.isPowerVR() {
        let (major, minor, vendorMajor, vendorMinor) = parsePowerVRVersion(glVersion);
        capabilities.contextVersionMajor = major;
        capabilities.contextVersionMinor = minor;
        capabilities.vendorDriverVersionMajor = vendorMajor;
        capabilities.vendorDriverVersionMinor = vendorMinor;
    } else {
        let (major, minor) = parseVersionPair(glVersion, "OpenGL ES ");
        capabilities.contextVersionMajor = major;
        capabilities.contextVersionMinor = minor;
    }
    if capabilities.isAdreno() {
        capabilities.adrenoSeries = parseAdrenoSeries(renderer);
    }
    if !capabilities.isContextVersionAtLeast(3, 0) {
        eprintln!(
            "OpenGL ES {}.{} not supported. Minimum supported version is 3.0.",
            capabilities.contextVersionMajor, capabilities.contextVersionMinor
        );
        return None;
    }

    capabilities.maxSupportedInstancesPerFlush = if capabilities.isMali()
        || capabilities.isPowerVR()
        || capabilities.isAdreno() && capabilities.adrenoSeries < 600
    {
        (1 << 13) - 1
    } else {
        u32::MAX
    };
    capabilities.setSupportsETC2(true);
    if capabilities.isContextVersionAtLeast(3, 1) {
        capabilities.setARB_shader_storage_buffer_object(true);
    }
    if capabilities.isContextVersionAtLeast(3, 2) {
        capabilities.setOES_shader_image_atomic(true);
    }

    if super::pls_impl_webgl_impl::webglEnableShaderPixelLocalStorageCoherent(&executionDomain) {
        capabilities.setANGLE_shader_pixel_local_storage(true);
        capabilities.setANGLE_shader_pixel_local_storage_coherent(true);
    }
    if super::pls_impl_webgl_impl::webglEnableProvokingVertex(&executionDomain) {
        capabilities.setANGLE_provoking_vertex(true);
    }
    capabilities.setEXT_clip_cull_distance(
        executionDomain.enableWebGLExtension("WEBGL_clip_cull_distance"),
    );
    capabilities.setEXT_color_buffer_half_float(
        executionDomain.enableWebGLExtension("EXT_color_buffer_half_float"),
    );
    capabilities.setOES_texture_half_float_linear(
        executionDomain.enableWebGLExtension("OES_texture_half_float_linear"),
    );
    capabilities.setEXT_color_buffer_float(
        executionDomain.enableWebGLExtension("EXT_color_buffer_float"),
    );
    capabilities.setEXT_float_blend(executionDomain.enableWebGLExtension("EXT_float_blend"));
    capabilities.setKHR_parallel_shader_compile(
        executionDomain.enableWebGLExtension("KHR_parallel_shader_compile"),
    );
    capabilities.setEXT_texture_compression_s3tc(
        executionDomain.enableWebGLExtension("WEBGL_compressed_texture_s3tc"),
    );
    capabilities.setEXT_texture_compression_bptc(
        executionDomain.enableWebGLExtension("EXT_texture_compression_bptc"),
    );
    capabilities.setKHR_texture_compression_astc_ldr(
        executionDomain.enableWebGLExtension("WEBGL_compressed_texture_astc"),
    );

    if capabilities.ARB_shader_storage_buffer_object()
        && executionDomain.getInteger(GL_MAX_VERTEX_SHADER_STORAGE_BLOCKS)
            < gpu::kMaxStorageBuffers as GLint
    {
        capabilities.setARB_shader_storage_buffer_object(false);
    }
    if capabilities.OES_shader_image_atomic()
        && (capabilities.isMali()
            || capabilities.isPowerVR()
            || capabilities.isAdreno() && !renderer.contains("Adreno (TM) 640"))
    {
        capabilities.setOES_shader_image_atomic(false);
    }
    if capabilities.ANGLE_base_vertex_base_instance_shader_builtin()
        && capabilities.isANGLESystemDriver()
    {
        capabilities.setANGLE_base_vertex_base_instance_shader_builtin(false);
    }
    if capabilities.EXT_clip_cull_distance() && capabilities.isANGLESystemDriver() {
        capabilities.setEXT_clip_cull_distance(false);
    }
    if capabilities.EXT_multisampled_render_to_texture() {
        if renderer.contains("Direct3D")
            || capabilities.isPowerVR() && !capabilities.isVendorDriverVersionAtLeast(1, 13)
        {
            capabilities.setEXT_multisampled_render_to_texture(false);
        }
    }
    if options.disableFragmentShaderInterlock {
        capabilities.setARB_fragment_shader_interlock(false);
        capabilities.setINTEL_fragment_shader_ordering(false);
    }
    capabilities.needsFloatingPointTessellationTexture =
        renderer.contains("ANGLE Metal Renderer") && capabilities.EXT_color_buffer_float();
    if capabilities.EXT_shader_pixel_local_storage2()
        && capabilities.isPowerVR()
        && !capabilities.isVendorDriverVersionAtLeast(1, 11)
    {
        capabilities.usePixelLocalStorage2AsWorkaround = true;
    }
    if capabilities.ANGLE_shader_pixel_local_storage() && renderer.contains("Direct3D11") {
        capabilities.avoidTexture2DArrayWithWebGLPLS = true;
    }

    let plsImpl = if !options.disablePixelLocalStorage
        && capabilities.ANGLE_shader_pixel_local_storage_coherent()
        && !capabilities.isAdreno()
    {
        let Some(factory) = finalPLSFactory else {
            eprintln!(
                "WEBGL_shader_pixel_local_storage_coherent requires component097 PLSImplWebGL"
            );
            return None;
        };
        Some(factory.MakePLSImplWebGL())
    } else {
        None
    };

    Some(newContextOwner(
        rendererBytes,
        capabilities,
        plsImpl,
        options.shaderCompilationMode,
        executionDomain,
    ))
}

fn makeContextInCurrent(
    options: ContextOptions,
    executionDomain: GLExecutionDomain,
    finalPLSFactory: Option<&dyn PixelLocalStorageFactory>,
) -> Option<std::pin::Pin<Box<RenderContext>>> {
    let implementation =
        makeContextOwnerInCurrent(options, executionDomain, finalPLSFactory)?;
    Some(<RenderContext as RenderContextContract>::new(implementation))
}

pub(crate) fn MakeContext(
    options: ContextOptions,
    provider: Box<dyn GLExecutionProvider>,
) -> Option<std::pin::Pin<Box<RenderContext>>> {
    let executionDomain = GLExecutionDomain::new(provider);
    let result = executionDomain.withCurrent(|| {
        makeContextInCurrent(
            options,
            executionDomain.clone(),
            Some(&super::pls_impl_webgl_impl::PLS_IMPL_WEBGL_FACTORY),
        )
    });
    if result.is_none() {
        executionDomain.shutdown();
    }
    result
}

impl RenderContextHelperImplAccess for RenderContextGLImpl {
    fn renderContextHelperImpl(&self) -> &RenderContextHelperImpl {
        &self.base
    }

    fn renderContextHelperImplMut(&mut self) -> &mut RenderContextHelperImpl {
        &mut self.base
    }
}

impl RenderContextHelperBufferFactoryContract for RenderContextGLImpl {
    fn makeUniformBufferRing(
        &mut self,
        capacityInBytes: usize,
    ) -> Option<Box<dyn BufferRingContract>> {
        makeUniformBufferRing(self, capacityInBytes)
    }

    fn makeStorageBufferRing(
        &mut self,
        capacityInBytes: usize,
        bufferStructure: gpu::StorageBufferStructure,
    ) -> Option<Box<dyn BufferRingContract>> {
        makeStorageBufferRing(self, capacityInBytes, bufferStructure)
    }

    fn makeVertexBufferRing(
        &mut self,
        capacityInBytes: usize,
    ) -> Option<Box<dyn BufferRingContract>> {
        makeVertexBufferRing(self, capacityInBytes)
    }
}

impl RenderContextHelperBackendContract for RenderContextGLImpl {
    fn makeRenderBuffer(
        &mut self,
        ty: RenderBufferType,
        flags: RenderBufferFlags,
        bytes: usize,
    ) -> rcp<RenderBuffer> {
        makeRenderBuffer(self, ty, flags, bytes)
    }

    fn makeImageTexture(
        &mut self,
        width: u32,
        height: u32,
        levels: u32,
        format: GPUTextureFormat,
        data: &[u8],
        blockWidth: u8,
        blockHeight: u8,
        srgb: bool,
        generateRemainingMips: bool,
    ) -> rcp<RiveTexture> {
        makeImageTexture(
            self,
            width,
            height,
            levels,
            format,
            data,
            blockWidth,
            blockHeight,
            srgb,
            generateRemainingMips,
        )
    }

    #[cfg(any(
        feature = "native-ore-metal-experimental",
        feature = "native-ore-vulkan-experimental",
        feature = "ore-gl"
    ))]
    fn makeRenderCanvas(&mut self, width: u32, height: u32) -> rcp<RenderCanvas> {
        makeRenderCanvas(self, width, height)
    }

    #[cfg(any(
        feature = "native-ore-metal-experimental",
        feature = "native-ore-vulkan-experimental",
        feature = "ore-gl"
    ))]
    fn makeOreContext(
        &mut self,
    ) -> Option<Box<crate::mechanical_port::source::include::rive::factory_hpp::OreContext>> {
        makeOreContext(self)
    }

    unsafe fn preBeginFrame(&mut self, context: *mut RenderContext) {
        unsafe { preBeginFrame(self, context) }
    }

    fn resizeGradientTexture(&mut self, width: u32, height: u32) {
        resizeGradientTexture(self, width, height)
    }

    fn resizeTessellationTexture(&mut self, width: u32, height: u32) {
        resizeTessellationTexture(self, width, height)
    }

    fn resizeFeatherAtlasTexture(&mut self, width: u32, height: u32) {
        resizeFeatherAtlasTexture(self, width, height)
    }

    fn resizeTransientPLSBacking(&mut self, width: u32, height: u32, planes: u32) {
        resizeTransientPLSBacking(self, width, height, planes)
    }

    fn resizeAtomicCoverageBacking(&mut self, width: u32, height: u32) {
        resizeAtomicCoverageBacking(self, width, height)
    }

    unsafe fn flush(&mut self, descriptor: &gpu::FlushDescriptor) {
        unsafe { flush(self, descriptor) }
    }
}

fn neutralizeProgram(program: &mut Program) {
    program.m_fragmentShader.0.m_id = 0;
    program.m_vertexShader.0.m_id = 0;
    program.m_object.m_id = 0;
}

unsafe fn dropProgram(program: &mut ManuallyDrop<Program>, currentGeneration: bool) {
    if !currentGeneration {
        neutralizeProgram(program);
    }
    unsafe { ManuallyDrop::drop(program) };
}

unsafe fn dropShader(shader: &mut ManuallyDrop<Shader>, currentGeneration: bool) {
    if !currentGeneration {
        shader.0.m_id = 0;
    }
    unsafe { ManuallyDrop::drop(shader) };
}

unsafe fn dropTexture(texture: &mut ManuallyDrop<GLTexture>, currentGeneration: bool) {
    if !currentGeneration {
        texture.0.m_id = 0;
    }
    unsafe { ManuallyDrop::drop(texture) };
}

unsafe fn dropFramebuffer(framebuffer: &mut ManuallyDrop<Framebuffer>, currentGeneration: bool) {
    if !currentGeneration {
        framebuffer.0.m_id = 0;
    }
    unsafe { ManuallyDrop::drop(framebuffer) };
}

unsafe fn dropBuffer(buffer: &mut ManuallyDrop<Buffer>, currentGeneration: bool) {
    if currentGeneration {
        unsafe { ManuallyDrop::drop(buffer) };
    }
    // Buffer is one scalar GLObject. On a stale generation there is no Rust
    // allocation to release, and invoking its source destructor would submit
    // even GLuint 0 to the replacement context.
}

unsafe fn dropVAO(vao: &mut ManuallyDrop<VAO>, currentGeneration: bool) {
    if currentGeneration {
        unsafe { ManuallyDrop::drop(vao) };
    }
    // VAO has the same unconditional-delete source destructor as Buffer.
}

fn preparePipelineManagerForDrop(manager: &mut GLPipelineManager, currentGeneration: bool) {
    let state = &mut *manager.base;
    state.m_isDone = true;
    state.m_newJobCV.notify_all();
    state.m_jobCompleteCV.notify_all();
    state.m_sharedObjectReadyCV.notify_all();
    assert!(
        state.m_jobThread.is_none(),
        "the GL subclass never starts the AsyncPipelineManager CPU worker"
    );
    state.m_jobQueue.clear();
    state.m_activePipelineCreationCount = 0;
    state.m_currentThreadPipelineKey = None;
    if !currentGeneration {
        for pipeline in state
            .m_pipelines
            .values_mut()
            .filter_map(Option::as_deref_mut)
        {
            pipeline.m_id = 0;
        }
        for completed in &mut state.m_completedJobs {
            completed.program.m_id = 0;
        }
        for shader in state
            .m_fragmentShaderMap
            .values_mut()
            .chain(state.m_vertexShaderMap.values_mut())
            .filter_map(Option::as_deref_mut)
        {
            shader.m_id = 0;
        }
    }
    // Programs own raw shader pointers, so release all programs before their
    // cached shader owners, matching the source AsyncPipelineManager teardown.
    state.m_completedJobs.clear();
    state.m_pipelines.clear();
    state.m_fragmentShaderMap.clear();
    state.m_vertexShaderMap.clear();
}

unsafe fn destroyContextSourceFields(context: &mut RenderContextGLImpl, currentGeneration: bool) {
    // Detach the logical registry before the first provider boundary. Any
    // queued or outliving canvas callback now observes an empty map and cannot
    // race this teardown's ownership of the remaining FBO names.
    let canvasMirrors = {
        let mut registry = context.m_canvasMirrors.borrow_mut();
        std::mem::take(&mut *registry)
    };
    if currentGeneration {
        recordGLCommand(GLCommand::DeleteTexture(context.m_gradientTexture));
        recordGLCommand(GLCommand::DeleteTexture(context.m_tessVertexTexture));
        // Canvas owners may legally outlive the context. Release the detached
        // registry-owned FBOs while their creating generation is current.
        for entry in canvasMirrors.values() {
            if entry.readFBO != 0 {
                recordGLCommand(GLCommand::DeleteFramebuffer(entry.readFBO));
            }
            if entry.drawFBO != 0 {
                recordGLCommand(GLCommand::DeleteFramebuffer(entry.drawFBO));
            }
        }
        context.m_state.borrow_mut().invalidate();
    }
    context.m_gradientTexture = 0;
    context.m_tessVertexTexture = 0;

    // Exact reverse declaration order for every nontrivial source field.
    unsafe { ManuallyDrop::drop(&mut context.m_canvasMirrors) };
    unsafe { ManuallyDrop::drop(&mut context.m_state) };
    unsafe { dropProgram(&mut context.m_blitAsDrawProgram, currentGeneration) };
    unsafe { dropVAO(&mut context.m_emptyVAO, currentGeneration) };
    unsafe { dropVAO(&mut context.m_imageMeshVAO, currentGeneration) };
    unsafe { dropBuffer(&mut context.m_imageRectIndexBuffer, currentGeneration) };
    unsafe { dropBuffer(&mut context.m_imageRectVertexBuffer, currentGeneration) };
    unsafe { dropVAO(&mut context.m_imageRectVAO, currentGeneration) };
    unsafe { dropVAO(&mut context.m_trianglesVAO, currentGeneration) };
    unsafe { dropBuffer(&mut context.m_patchIndicesBuffer, currentGeneration) };
    unsafe { dropBuffer(&mut context.m_patchVerticesBuffer, currentGeneration) };
    unsafe { dropVAO(&mut context.m_drawVAO, currentGeneration) };

    preparePipelineManagerForDrop(&mut context.m_pipelineManager, currentGeneration);
    unsafe { ManuallyDrop::drop(&mut context.m_pipelineManager.base) };
    unsafe { ManuallyDrop::drop(&mut context.m_pipelineManager) };

    unsafe { dropFramebuffer(&mut context.m_featherAtlasResolveFBO, currentGeneration) };
    unsafe { dropFramebuffer(&mut context.m_featherAtlasRenderFBO, currentGeneration) };
    unsafe { dropTexture(&mut context.m_featherAtlasTexture, currentGeneration) };
    unsafe { dropTexture(&mut context.m_featherAtlasRenderTexture, currentGeneration) };
    unsafe { dropVAO(&mut context.m_featherAtlasResolveVAO, currentGeneration) };
    unsafe { dropProgram(&mut context.m_featherAtlasResolveProgram, currentGeneration) };
    unsafe { dropProgram(&mut context.m_featherAtlasClearProgram, currentGeneration) };
    unsafe {
        dropShader(
            &mut context.m_featherAtlasResolveVertexShader,
            currentGeneration,
        )
    };
    if !currentGeneration {
        neutralizeProgram(&mut context.m_featherAtlasStrokeProgram.m_program);
    }
    unsafe { ManuallyDrop::drop(&mut context.m_featherAtlasStrokeProgram.m_program) };
    if !currentGeneration {
        neutralizeProgram(&mut context.m_featherAtlasFillProgram.m_program);
    }
    unsafe { ManuallyDrop::drop(&mut context.m_featherAtlasFillProgram.m_program) };
    unsafe { dropShader(&mut context.m_featherAtlasVertexShader, currentGeneration) };

    unsafe { dropFramebuffer(&mut context.m_tessellateFBO, currentGeneration) };
    unsafe { dropBuffer(&mut context.m_tessSpanIndexBuffer, currentGeneration) };
    unsafe { dropVAO(&mut context.m_tessellateVAO, currentGeneration) };
    unsafe { dropProgram(&mut context.m_tessellateProgram, currentGeneration) };
    unsafe { dropTexture(&mut context.m_gaussianIntegralTexture, currentGeneration) };
    unsafe { dropFramebuffer(&mut context.m_colorRampFBO, currentGeneration) };
    unsafe { dropVAO(&mut context.m_colorRampVAO, currentGeneration) };
    unsafe { dropProgram(&mut context.m_colorRampProgram, currentGeneration) };
    unsafe { ManuallyDrop::drop(&mut context.m_plsImpl) };
    unsafe { ManuallyDrop::drop(&mut context.base) };
}

impl Drop for RenderContextGLImpl {
    fn drop(&mut self) {
        let execution = (&*self.rust_execution).clone();
        let executionDomain = execution.domain().clone();
        if execution
            .withDeleteCurrent(|| unsafe { destroyContextSourceFields(self, true) })
            .is_none()
        {
            unsafe { destroyContextSourceFields(self, false) };
        }

        // Rust sidecars are after the complete source prefix. Retire the
        // renderer root only after every internal source owner is gone, but
        // keep the final-release domain alive for source-external Canvas,
        // image, and target owners. Their asynchronous late release closes the
        // domain after the last strong execution stamp disappears.
        executionDomain.retireRenderer();
        drop(execution);
        unsafe {
            ManuallyDrop::drop(&mut self.rust_source_renderer_string);
            ManuallyDrop::drop(&mut self.rust_execution);
        }
        drop(executionDomain);
    }
}

/// Exact `ORE_BACKEND_GL && RIVE_CANVAS` namespace-level source callable.
pub(crate) unsafe fn getCanvasImportMirrorGL(
    renderContext: *mut RenderContext,
    sourceTex: *mut RiveTexture,
    width: u32,
    height: u32,
) -> rcp<RiveRenderImage> {
    if renderContext.is_null()
        || !unsafe { (&*renderContext).platformFeatures().framebufferBottomUp }
    {
        return rcp::new();
    }
    let implementation = unsafe { (&*renderContext).static_impl_cast::<RenderContextGLImpl>() };
    unsafe { (&mut *implementation).getCanvasImportMirror(sourceTex, width, height) }
}

#[cfg(feature = "ore-gl")]
impl crate::mechanical_port::source::renderer::include::rive::renderer::render_context_impl_hpp::CanvasImportMirrorGL
    for RenderContextGLImpl
{
    unsafe fn getCanvasImportMirrorGL(
        renderContext: *mut RenderContext,
        sourceTex: *mut RiveTexture,
        width: u32,
        height: u32,
    ) -> rcp<RiveRenderImage> {
        unsafe { getCanvasImportMirrorGL(renderContext, sourceTex, width, height) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct CanvasTestProvider {
        commands: Rc<RefCell<Vec<GLCommand>>>,
        lifecycleIngress: Option<GLContextLifecycleIngress>,
        finalReleaseIngress: Rc<RefCell<Option<GLFinalReleaseIngress>>>,
        finalReleaseWake: std::sync::Arc<TestFinalReleaseWake>,
    }

    impl GLExecutionProvider for CanvasTestProvider {
        fn installContextLifecycleIngress(&mut self, ingress: GLContextLifecycleIngress) {
            assert!(self.lifecycleIngress.replace(ingress).is_none());
        }

        fn installFinalReleaseIngress(
            &mut self,
            ingress: GLFinalReleaseIngress,
        ) -> std::sync::Arc<dyn nuxie_ore_metal::gpu_resource::ResourceFinalReleaseWake> {
            assert!(self.finalReleaseIngress.borrow_mut().replace(ingress).is_none());
            self.finalReleaseWake.clone()
        }

        fn submit(&mut self, command: GLCommand) {
            self.commands.borrow_mut().push(command);
        }

        fn generateObject(&mut self, _kind: GLObjectKind) -> GLuint {
            1
        }

        fn createProgram(&mut self) -> GLuint {
            1
        }

        fn createShader(&mut self, _shaderType: GLenum) -> GLuint {
            1
        }

        fn getInteger(&mut self, _parameter: GLenum) -> GLint {
            0
        }

        fn getString(&mut self, _parameter: GLenum) -> Option<Vec<u8>> {
            None
        }

        fn getExtension(&mut self, _index: GLuint) -> Option<Vec<u8>> {
            None
        }

        fn enableWebGLExtension(&mut self, _name: &str) -> bool {
            false
        }

        fn isObject(&mut self, _kind: GLObjectKind, _name: GLuint) -> bool {
            false
        }

        fn checkFramebufferStatus(&mut self, _target: GLenum) -> GLenum {
            GL_FRAMEBUFFER_COMPLETE
        }

        fn shaderParameter(&mut self, _shader: GLuint, _parameter: GLenum) -> GLint {
            0
        }

        fn shaderInfoLog(&mut self, _shader: GLuint, _maxLength: usize) -> Vec<u8> {
            Vec::new()
        }

        fn programParameter(&mut self, _program: GLuint, _parameter: GLenum) -> GLint {
            0
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

    #[test]
    fn frozen_implementation_receipt_is_locked() {
        assert_eq!(PINNED_SOURCE.lines().count(), 3_985);
        assert_eq!(PINNED_SOURCE.len(), 153_801);
    }

    #[test]
    fn worker_queued_canvas_finalizers_update_registry_before_method_body() {
        let commands = Rc::new(RefCell::new(Vec::new()));
        let finalReleaseIngress = Rc::new(RefCell::new(None));
        let finalReleaseWake = std::sync::Arc::new(TestFinalReleaseWake::default());
        let domain = GLExecutionDomain::new(Box::new(CanvasTestProvider {
            commands: Rc::clone(&commands),
            lifecycleIngress: None,
            finalReleaseIngress: Rc::clone(&finalReleaseIngress),
            finalReleaseWake: std::sync::Arc::clone(&finalReleaseWake),
        }));
        let execution = domain.stamp();
        let canvasRegistry: CanvasMirrorRegistry = Rc::new(RefCell::new(BTreeMap::from([(
            11,
            CanvasMirrorEntry {
                mirrorTex: 17,
                width: 32,
                height: 24,
                readFBO: 41,
                drawFBO: 43,
                hasMirror: true,
            },
        )])));

        let mirror = make_rcp(|| {
            CanvasMirrorTextureGLImpl::new(
                32,
                24,
                17,
                execution.clone(),
                std::ptr::null_mut(),
                11,
                Rc::downgrade(&canvasRegistry),
            )
        });
        let mirror: rcp<RiveTexture> = unsafe { static_rcp_cast(mirror) };
        std::thread::spawn(move || drop(mirror)).join().unwrap();

        assert_eq!(finalReleaseWake.takePosts(), 1);
        assert!(canvasRegistry.borrow().get(&11).unwrap().hasMirror);
        let mirrorEntry = execution.withCurrent(|| {
            // The worker finalizer is drained at this method-style ingress,
            // before the logical registry is borrowed by the body.
            *canvasRegistry.borrow().get(&11).unwrap()
        });
        assert_eq!(mirrorEntry.mirrorTex, 0);
        assert_eq!(mirrorEntry.readFBO, 0);
        assert_eq!(mirrorEntry.drawFBO, 0);
        assert!(!mirrorEntry.hasMirror);

        let source = make_rcp(|| {
            CanvasSourceTextureGLImpl::new(
                32,
                24,
                11,
                execution.clone(),
                std::ptr::null_mut(),
                Rc::downgrade(&canvasRegistry),
            )
        });
        let source: rcp<RiveTexture> = unsafe { static_rcp_cast(source) };
        std::thread::spawn(move || drop(source)).join().unwrap();

        assert_eq!(finalReleaseWake.takePosts(), 1);
        assert!(canvasRegistry.borrow().contains_key(&11));
        execution.withCurrent(|| {
            assert!(!canvasRegistry.borrow().contains_key(&11));
        });

        let commands = commands.borrow();
        assert!(commands.contains(&GLCommand::DeleteFramebuffer(41)));
        assert!(commands.contains(&GLCommand::DeleteFramebuffer(43)));
        assert!(commands.contains(&GLCommand::DeleteTexture(17)));
        assert!(commands.contains(&GLCommand::DeleteTexture(11)));
        drop(commands);

        drop(execution);
        domain.shutdown();
        assert!(finalReleaseIngress.borrow().is_some());
    }
}
