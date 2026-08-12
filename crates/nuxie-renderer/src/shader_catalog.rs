//! Single creation seam for renderer-owned built-in shader modules.
//!
//! Authored GPU-canvas shaders are intentionally not part of this catalog:
//! their source and entry points come from runtime content. Every variant here
//! names repository-controlled source whose Apple MSL artifact can be generated
//! and checked ahead of time without changing pipeline call sites.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum BuiltinShaderKey {
    AdvancedComposite,
    AtlasFillFragment,
    AtlasStrokeFragment,
    AtlasVertex,
    AtlasVertexStorageTexture,
    AtomicAdvancedAtlasBlitFragment,
    AtomicAdvancedImageMeshFragment,
    AtomicAdvancedImageRectFragment,
    AtomicAdvancedInitFragment,
    AtomicAdvancedInteriorFragment,
    AtomicAdvancedPathFragment,
    AtomicAdvancedResolveFragment,
    AtomicAdvancedResolveVertex,
    AtomicAtlasBlitFragment,
    AtomicAtlasBlitVertex,
    AtomicImageMeshFragment,
    AtomicImageMeshVertex,
    AtomicImageRectFragment,
    AtomicImageRectVertex,
    AtomicInitFragment,
    AtomicInitVertex,
    AtomicInteriorFragment,
    AtomicInteriorVertex,
    AtomicPathFragment,
    AtomicPathVertex,
    AtomicResolveFragment,
    AtomicResolveVertex,
    ClockwiseAtomicBorrowedInteriorFragment,
    ClockwiseAtomicBorrowedPathFragment,
    ClockwiseAtomicClipInteriorFragment,
    ClockwiseAtomicClipPathFragment,
    ClockwiseAtomicInteriorFragment,
    ClockwiseAtomicInteriorVertex,
    ClockwiseAtomicPathFragment,
    ClockwiseAtomicPathVertex,
    ClockwiseAtomicSampledClipInteriorFragment,
    ClockwiseAtomicSampledClipPathFragment,
    ColorRampFragment,
    ColorRampVertex,
    Composite,
    MipmapFragment,
    MipmapVertex,
    MsaaAtlasAdvancedFragment,
    MsaaAtlasFixedFragment,
    MsaaAtlasVertex,
    MsaaAtlasVertexClipDistance,
    MsaaAtlasVertexStorageTexture,
    MsaaAtlasVertexStorageTextureClipDistance,
    MsaaImageMeshAdvancedFragment,
    MsaaImageMeshFixedFragment,
    MsaaImageMeshVertex,
    MsaaImageMeshVertexClipDistance,
    MsaaPathAdvancedFragment,
    MsaaPathFixedFragment,
    MsaaPathVertex,
    MsaaPathVertexClipDistance,
    MsaaPathVertexStorageTexture,
    MsaaPathVertexStorageTextureClipDistance,
    MsaaStencilFragment,
    MsaaStencilVertex,
    Solid,
    SurfacePresent,
    TessellateFragment,
    TessellateVertex,
    TessellateVertexStorageTexture,
}

impl BuiltinShaderKey {
    fn label(self) -> &'static str {
        match self {
            Self::AdvancedComposite => "nuxie-advanced-composite-shader",
            Self::AtlasFillFragment => "nuxie-atlas-fill-fragment",
            Self::AtlasStrokeFragment => "nuxie-atlas-stroke-fragment",
            Self::AtlasVertex | Self::AtlasVertexStorageTexture => "nuxie-atlas-vertex",
            Self::AtomicAdvancedAtlasBlitFragment => "nuxie-atomic-advanced-atlas-blit-fragment",
            Self::AtomicAdvancedImageMeshFragment => "nuxie-atomic-advanced-image-mesh-fragment",
            Self::AtomicAdvancedImageRectFragment => "nuxie-atomic-advanced-image-rect-fragment",
            Self::AtomicAdvancedInitFragment => "nuxie-atomic-advanced-init-fragment",
            Self::AtomicAdvancedInteriorFragment => "nuxie-atomic-advanced-interior-fragment",
            Self::AtomicAdvancedPathFragment => "nuxie-atomic-advanced-path-fragment",
            Self::AtomicAdvancedResolveFragment => "nuxie-atomic-advanced-resolve-fragment",
            Self::AtomicAdvancedResolveVertex => "nuxie-atomic-advanced-resolve-vertex",
            Self::AtomicAtlasBlitFragment => "nuxie-atomic-atlas-blit-fragment",
            Self::AtomicAtlasBlitVertex => "nuxie-atomic-atlas-blit-vertex",
            Self::AtomicImageMeshFragment => "nuxie-atomic-image-mesh-fragment",
            Self::AtomicImageMeshVertex => "nuxie-atomic-image-mesh-vertex",
            Self::AtomicImageRectFragment => "nuxie-atomic-image-rect-fragment",
            Self::AtomicImageRectVertex => "nuxie-atomic-image-rect-vertex",
            Self::AtomicInitFragment => "nuxie-atomic-init-fragment",
            Self::AtomicInitVertex => "nuxie-atomic-init-vertex",
            Self::AtomicInteriorFragment => "nuxie-atomic-interior-fragment",
            Self::AtomicInteriorVertex => "nuxie-atomic-interior-vertex",
            Self::AtomicPathFragment => "nuxie-atomic-path-fragment",
            Self::AtomicPathVertex => "nuxie-atomic-path-vertex",
            Self::AtomicResolveFragment => "nuxie-atomic-resolve-fragment",
            Self::AtomicResolveVertex => "nuxie-atomic-resolve-vertex",
            Self::ClockwiseAtomicBorrowedInteriorFragment => "nuxie-cwa-borrowed-interior-fragment",
            Self::ClockwiseAtomicBorrowedPathFragment => "nuxie-cwa-borrowed-path-fragment",
            Self::ClockwiseAtomicClipInteriorFragment => "nuxie-cwa-clip-interior-fragment",
            Self::ClockwiseAtomicClipPathFragment => "nuxie-cwa-clip-path-fragment",
            Self::ClockwiseAtomicInteriorFragment => "nuxie-cwa-interior-fragment",
            Self::ClockwiseAtomicInteriorVertex => "nuxie-cwa-interior-vertex",
            Self::ClockwiseAtomicPathFragment => "nuxie-cwa-path-fragment",
            Self::ClockwiseAtomicPathVertex => "nuxie-cwa-path-vertex",
            Self::ClockwiseAtomicSampledClipInteriorFragment => {
                "nuxie-cwa-sampled-clip-interior-fragment"
            }
            Self::ClockwiseAtomicSampledClipPathFragment => "nuxie-cwa-sampled-clip-path-fragment",
            Self::ColorRampFragment => "nuxie-gradient-fragment",
            Self::ColorRampVertex => "nuxie-gradient-vertex",
            Self::Composite => "nuxie-composite-shader",
            Self::MipmapFragment => "nuxie-mipmap-fragment",
            Self::MipmapVertex => "nuxie-mipmap-vertex",
            Self::MsaaAtlasAdvancedFragment => "nuxie-msaa-atlas-advanced-blit-fragment",
            Self::MsaaAtlasFixedFragment => "nuxie-msaa-atlas-blit-fragment",
            Self::MsaaAtlasVertex | Self::MsaaAtlasVertexStorageTexture => {
                "nuxie-msaa-atlas-blit-vertex"
            }
            Self::MsaaAtlasVertexClipDistance | Self::MsaaAtlasVertexStorageTextureClipDistance => {
                "nuxie-msaa-atlas-blit-clip-rect-vertex"
            }
            Self::MsaaImageMeshAdvancedFragment => "nuxie-msaa-image-mesh-advanced-fragment",
            Self::MsaaImageMeshFixedFragment => "nuxie-msaa-image-mesh-fragment",
            Self::MsaaImageMeshVertex => "nuxie-msaa-image-mesh-vertex",
            Self::MsaaImageMeshVertexClipDistance => "nuxie-msaa-image-mesh-clip-rect-vertex",
            Self::MsaaPathAdvancedFragment => "nuxie-msaa-path-advanced-fragment",
            Self::MsaaPathFixedFragment => "nuxie-msaa-path-fragment",
            Self::MsaaPathVertex | Self::MsaaPathVertexStorageTexture => "nuxie-msaa-path-vertex",
            Self::MsaaPathVertexClipDistance | Self::MsaaPathVertexStorageTextureClipDistance => {
                "nuxie-msaa-path-clip-rect-vertex"
            }
            Self::MsaaStencilFragment => "nuxie-msaa-stencil-fragment",
            Self::MsaaStencilVertex => "nuxie-msaa-stencil-vertex",
            Self::Solid => "nuxie-solid-shader",
            Self::SurfacePresent => "nuxie-surface-present-shader",
            Self::TessellateFragment => "nuxie-tessellate-fragment",
            Self::TessellateVertex | Self::TessellateVertexStorageTexture => {
                "nuxie-tessellate-vertex"
            }
        }
    }

    fn wgsl(self) -> &'static str {
        match self {
            Self::AdvancedComposite => include_str!("advanced_composite.wgsl"),
            Self::AtlasFillFragment => include_str!("generated/render_atlas_fill.webgpu_frag.wgsl"),
            Self::AtlasStrokeFragment => {
                include_str!("generated/render_atlas_stroke.webgpu_frag.wgsl")
            }
            Self::AtlasVertex => include_str!("generated/render_atlas.webgpu_vert.wgsl"),
            Self::AtlasVertexStorageTexture => {
                include_str!("generated/render_atlas.webgpu_nossbo_vert.wgsl")
            }
            Self::AtomicAdvancedAtlasBlitFragment => {
                include_str!("generated/atomic_draw_atlas_blit.webgpu_frag.wgsl")
            }
            Self::AtomicAdvancedImageMeshFragment => {
                include_str!("generated/atomic_draw_image_mesh.webgpu_frag.wgsl")
            }
            Self::AtomicAdvancedImageRectFragment => {
                include_str!("generated/atomic_draw_image_rect.webgpu_frag.wgsl")
            }
            Self::AtomicAdvancedInitFragment => {
                include_str!("generated/atomic_init.webgpu_frag.wgsl")
            }
            Self::AtomicAdvancedInteriorFragment => {
                include_str!("generated/atomic_draw_interior_triangles.webgpu_frag.wgsl")
            }
            Self::AtomicAdvancedPathFragment => {
                include_str!("generated/atomic_draw_path.webgpu_frag.wgsl")
            }
            Self::AtomicAdvancedResolveFragment => {
                include_str!("generated/atomic_resolve_coalesced.webgpu_frag.wgsl")
            }
            Self::AtomicAdvancedResolveVertex => {
                include_str!("generated/atomic_resolve_coalesced.webgpu_vert.wgsl")
            }
            Self::AtomicAtlasBlitFragment => {
                include_str!("generated/atomic_draw_atlas_blit.webgpu_fixedcolor_frag.wgsl")
            }
            Self::AtomicAtlasBlitVertex => {
                include_str!("generated/atomic_draw_atlas_blit.webgpu_vert.wgsl")
            }
            Self::AtomicImageMeshFragment => {
                include_str!("generated/atomic_draw_image_mesh.webgpu_fixedcolor_frag.wgsl")
            }
            Self::AtomicImageMeshVertex => {
                include_str!("generated/atomic_draw_image_mesh.webgpu_vert.wgsl")
            }
            Self::AtomicImageRectFragment => {
                include_str!("generated/atomic_draw_image_rect.webgpu_fixedcolor_frag.wgsl")
            }
            Self::AtomicImageRectVertex => {
                include_str!("generated/atomic_draw_image_rect.webgpu_vert.wgsl")
            }
            Self::AtomicInitFragment => {
                include_str!("generated/atomic_init.webgpu_fixedcolor_frag.wgsl")
            }
            Self::AtomicInitVertex => include_str!("generated/atomic_init.webgpu_vert.wgsl"),
            Self::AtomicInteriorFragment => {
                include_str!("generated/atomic_draw_interior_triangles.webgpu_fixedcolor_frag.wgsl")
            }
            Self::AtomicInteriorVertex => {
                include_str!("generated/atomic_draw_interior_triangles.webgpu_vert.wgsl")
            }
            Self::AtomicPathFragment => {
                include_str!("generated/atomic_draw_path.webgpu_fixedcolor_frag.wgsl")
            }
            Self::AtomicPathVertex => include_str!("generated/atomic_draw_path.webgpu_vert.wgsl"),
            Self::AtomicResolveFragment => {
                include_str!("generated/atomic_resolve.webgpu_fixedcolor_frag.wgsl")
            }
            Self::AtomicResolveVertex => include_str!("generated/atomic_resolve.webgpu_vert.wgsl"),
            Self::ClockwiseAtomicBorrowedInteriorFragment => include_str!(
                "generated/clockwise_atomic_draw_interior_triangles_borrowed.webgpu_frag.wgsl"
            ),
            Self::ClockwiseAtomicBorrowedPathFragment => {
                include_str!("generated/clockwise_atomic_draw_path_borrowed.webgpu_frag.wgsl")
            }
            Self::ClockwiseAtomicClipInteriorFragment => include_str!(
                "generated/clockwise_atomic_draw_clip_interior_triangles.webgpu_fixedcolor_frag.wgsl"
            ),
            Self::ClockwiseAtomicClipPathFragment => {
                include_str!("generated/clockwise_atomic_draw_clip.webgpu_fixedcolor_frag.wgsl")
            }
            Self::ClockwiseAtomicInteriorFragment => include_str!(
                "generated/clockwise_atomic_draw_interior_triangles.webgpu_fixedcolor_frag.wgsl"
            ),
            Self::ClockwiseAtomicInteriorVertex => {
                include_str!("generated/clockwise_atomic_draw_interior_triangles.webgpu_vert.wgsl")
            }
            Self::ClockwiseAtomicPathFragment => {
                include_str!("generated/clockwise_atomic_draw_path.webgpu_fixedcolor_frag.wgsl")
            }
            Self::ClockwiseAtomicPathVertex => {
                include_str!("generated/clockwise_atomic_draw_path.webgpu_vert.wgsl")
            }
            Self::ClockwiseAtomicSampledClipInteriorFragment => include_str!(
                "generated/clockwise_atomic_draw_interior_triangles_sampled_clip.webgpu_fixedcolor_frag.wgsl"
            ),
            Self::ClockwiseAtomicSampledClipPathFragment => include_str!(
                "generated/clockwise_atomic_draw_path_sampled_clip.webgpu_fixedcolor_frag.wgsl"
            ),
            Self::ColorRampFragment => include_str!("generated/color_ramp.frag.wgsl"),
            Self::ColorRampVertex => include_str!("generated/color_ramp.vert.wgsl"),
            Self::Composite => include_str!("composite.wgsl"),
            Self::MipmapFragment => {
                include_str!("generated/blit_texture_as_draw_filtered.webgpu_frag.wgsl")
            }
            Self::MipmapVertex => {
                include_str!("generated/blit_texture_as_draw_filtered.webgpu_vert.wgsl")
            }
            Self::MsaaAtlasAdvancedFragment => {
                include_str!("generated/draw_msaa_atlas_blit.webgpu_frag.wgsl")
            }
            Self::MsaaAtlasFixedFragment => {
                include_str!("generated/draw_msaa_atlas_blit.webgpu_fixedcolor_frag.wgsl")
            }
            Self::MsaaAtlasVertex => {
                include_str!("generated/draw_msaa_atlas_blit.webgpu_noclipdistance_vert.wgsl")
            }
            Self::MsaaAtlasVertexClipDistance => {
                include_str!("generated/draw_msaa_atlas_blit.webgpu_vert.wgsl")
            }
            Self::MsaaAtlasVertexStorageTexture => include_str!(
                "generated/draw_msaa_atlas_blit.webgpu_nossbo_noclipdistance_vert.wgsl"
            ),
            Self::MsaaAtlasVertexStorageTextureClipDistance => {
                include_str!("generated/draw_msaa_atlas_blit.webgpu_nossbo_vert.wgsl")
            }
            Self::MsaaImageMeshAdvancedFragment => {
                include_str!("generated/draw_msaa_image_mesh.webgpu_frag.wgsl")
            }
            Self::MsaaImageMeshFixedFragment => {
                include_str!("generated/draw_msaa_image_mesh.webgpu_fixedcolor_frag.wgsl")
            }
            Self::MsaaImageMeshVertex => {
                include_str!("generated/draw_msaa_image_mesh.webgpu_noclipdistance_vert.wgsl")
            }
            Self::MsaaImageMeshVertexClipDistance => {
                include_str!("generated/draw_msaa_image_mesh.webgpu_vert.wgsl")
            }
            Self::MsaaPathAdvancedFragment => {
                include_str!("generated/draw_msaa_path.webgpu_frag.wgsl")
            }
            Self::MsaaPathFixedFragment => {
                include_str!("generated/draw_msaa_path.webgpu_fixedcolor_frag.wgsl")
            }
            Self::MsaaPathVertex => {
                include_str!("generated/draw_msaa_path.webgpu_noclipdistance_vert.wgsl")
            }
            Self::MsaaPathVertexClipDistance => {
                include_str!("generated/draw_msaa_path.webgpu_vert.wgsl")
            }
            Self::MsaaPathVertexStorageTexture => {
                include_str!("generated/draw_msaa_path.webgpu_nossbo_noclipdistance_vert.wgsl")
            }
            Self::MsaaPathVertexStorageTextureClipDistance => {
                include_str!("generated/draw_msaa_path.webgpu_nossbo_vert.wgsl")
            }
            Self::MsaaStencilFragment => {
                include_str!("generated/draw_msaa_stencil.webgpu_fixedcolor_frag.wgsl")
            }
            Self::MsaaStencilVertex => {
                include_str!("generated/draw_msaa_stencil.webgpu_noclipdistance_vert.wgsl")
            }
            Self::Solid => include_str!("solid.wgsl"),
            Self::SurfacePresent => include_str!("surface_present.wgsl"),
            Self::TessellateFragment => include_str!("generated/tessellate.webgpu_frag.wgsl"),
            Self::TessellateVertex => include_str!("generated/tessellate.webgpu_vert.wgsl"),
            Self::TessellateVertexStorageTexture => {
                include_str!("generated/tessellate.webgpu_nossbo_vert.wgsl")
            }
        }
    }

    #[cfg(feature = "apple-msl-capture")]
    fn source_path(self) -> &'static str {
        match self {
            Self::AdvancedComposite => "crates/nuxie-renderer/src/advanced_composite.wgsl",
            Self::AtlasFillFragment => {
                "crates/nuxie-renderer/src/generated/render_atlas_fill.webgpu_frag.wgsl"
            }
            Self::AtlasStrokeFragment => {
                "crates/nuxie-renderer/src/generated/render_atlas_stroke.webgpu_frag.wgsl"
            }
            Self::AtlasVertex => {
                "crates/nuxie-renderer/src/generated/render_atlas.webgpu_vert.wgsl"
            }
            Self::AtlasVertexStorageTexture => {
                "crates/nuxie-renderer/src/generated/render_atlas.webgpu_nossbo_vert.wgsl"
            }
            Self::AtomicAdvancedAtlasBlitFragment => {
                "crates/nuxie-renderer/src/generated/atomic_draw_atlas_blit.webgpu_frag.wgsl"
            }
            Self::AtomicAdvancedImageMeshFragment => {
                "crates/nuxie-renderer/src/generated/atomic_draw_image_mesh.webgpu_frag.wgsl"
            }
            Self::AtomicAdvancedImageRectFragment => {
                "crates/nuxie-renderer/src/generated/atomic_draw_image_rect.webgpu_frag.wgsl"
            }
            Self::AtomicAdvancedInitFragment => {
                "crates/nuxie-renderer/src/generated/atomic_init.webgpu_frag.wgsl"
            }
            Self::AtomicAdvancedInteriorFragment => {
                "crates/nuxie-renderer/src/generated/atomic_draw_interior_triangles.webgpu_frag.wgsl"
            }
            Self::AtomicAdvancedPathFragment => {
                "crates/nuxie-renderer/src/generated/atomic_draw_path.webgpu_frag.wgsl"
            }
            Self::AtomicAdvancedResolveFragment => {
                "crates/nuxie-renderer/src/generated/atomic_resolve_coalesced.webgpu_frag.wgsl"
            }
            Self::AtomicAdvancedResolveVertex => {
                "crates/nuxie-renderer/src/generated/atomic_resolve_coalesced.webgpu_vert.wgsl"
            }
            Self::AtomicAtlasBlitFragment => {
                "crates/nuxie-renderer/src/generated/atomic_draw_atlas_blit.webgpu_fixedcolor_frag.wgsl"
            }
            Self::AtomicAtlasBlitVertex => {
                "crates/nuxie-renderer/src/generated/atomic_draw_atlas_blit.webgpu_vert.wgsl"
            }
            Self::AtomicImageMeshFragment => {
                "crates/nuxie-renderer/src/generated/atomic_draw_image_mesh.webgpu_fixedcolor_frag.wgsl"
            }
            Self::AtomicImageMeshVertex => {
                "crates/nuxie-renderer/src/generated/atomic_draw_image_mesh.webgpu_vert.wgsl"
            }
            Self::AtomicImageRectFragment => {
                "crates/nuxie-renderer/src/generated/atomic_draw_image_rect.webgpu_fixedcolor_frag.wgsl"
            }
            Self::AtomicImageRectVertex => {
                "crates/nuxie-renderer/src/generated/atomic_draw_image_rect.webgpu_vert.wgsl"
            }
            Self::AtomicInitFragment => {
                "crates/nuxie-renderer/src/generated/atomic_init.webgpu_fixedcolor_frag.wgsl"
            }
            Self::AtomicInitVertex => {
                "crates/nuxie-renderer/src/generated/atomic_init.webgpu_vert.wgsl"
            }
            Self::AtomicInteriorFragment => {
                "crates/nuxie-renderer/src/generated/atomic_draw_interior_triangles.webgpu_fixedcolor_frag.wgsl"
            }
            Self::AtomicInteriorVertex => {
                "crates/nuxie-renderer/src/generated/atomic_draw_interior_triangles.webgpu_vert.wgsl"
            }
            Self::AtomicPathFragment => {
                "crates/nuxie-renderer/src/generated/atomic_draw_path.webgpu_fixedcolor_frag.wgsl"
            }
            Self::AtomicPathVertex => {
                "crates/nuxie-renderer/src/generated/atomic_draw_path.webgpu_vert.wgsl"
            }
            Self::AtomicResolveFragment => {
                "crates/nuxie-renderer/src/generated/atomic_resolve.webgpu_fixedcolor_frag.wgsl"
            }
            Self::AtomicResolveVertex => {
                "crates/nuxie-renderer/src/generated/atomic_resolve.webgpu_vert.wgsl"
            }
            Self::ClockwiseAtomicBorrowedInteriorFragment => {
                "crates/nuxie-renderer/src/generated/clockwise_atomic_draw_interior_triangles_borrowed.webgpu_frag.wgsl"
            }
            Self::ClockwiseAtomicBorrowedPathFragment => {
                "crates/nuxie-renderer/src/generated/clockwise_atomic_draw_path_borrowed.webgpu_frag.wgsl"
            }
            Self::ClockwiseAtomicClipInteriorFragment => {
                "crates/nuxie-renderer/src/generated/clockwise_atomic_draw_clip_interior_triangles.webgpu_fixedcolor_frag.wgsl"
            }
            Self::ClockwiseAtomicClipPathFragment => {
                "crates/nuxie-renderer/src/generated/clockwise_atomic_draw_clip.webgpu_fixedcolor_frag.wgsl"
            }
            Self::ClockwiseAtomicInteriorFragment => {
                "crates/nuxie-renderer/src/generated/clockwise_atomic_draw_interior_triangles.webgpu_fixedcolor_frag.wgsl"
            }
            Self::ClockwiseAtomicInteriorVertex => {
                "crates/nuxie-renderer/src/generated/clockwise_atomic_draw_interior_triangles.webgpu_vert.wgsl"
            }
            Self::ClockwiseAtomicPathFragment => {
                "crates/nuxie-renderer/src/generated/clockwise_atomic_draw_path.webgpu_fixedcolor_frag.wgsl"
            }
            Self::ClockwiseAtomicPathVertex => {
                "crates/nuxie-renderer/src/generated/clockwise_atomic_draw_path.webgpu_vert.wgsl"
            }
            Self::ClockwiseAtomicSampledClipInteriorFragment => {
                "crates/nuxie-renderer/src/generated/clockwise_atomic_draw_interior_triangles_sampled_clip.webgpu_fixedcolor_frag.wgsl"
            }
            Self::ClockwiseAtomicSampledClipPathFragment => {
                "crates/nuxie-renderer/src/generated/clockwise_atomic_draw_path_sampled_clip.webgpu_fixedcolor_frag.wgsl"
            }
            Self::ColorRampFragment => "crates/nuxie-renderer/src/generated/color_ramp.frag.wgsl",
            Self::ColorRampVertex => "crates/nuxie-renderer/src/generated/color_ramp.vert.wgsl",
            Self::Composite => "crates/nuxie-renderer/src/composite.wgsl",
            Self::MipmapFragment => {
                "crates/nuxie-renderer/src/generated/blit_texture_as_draw_filtered.webgpu_frag.wgsl"
            }
            Self::MipmapVertex => {
                "crates/nuxie-renderer/src/generated/blit_texture_as_draw_filtered.webgpu_vert.wgsl"
            }
            Self::MsaaAtlasAdvancedFragment => {
                "crates/nuxie-renderer/src/generated/draw_msaa_atlas_blit.webgpu_frag.wgsl"
            }
            Self::MsaaAtlasFixedFragment => {
                "crates/nuxie-renderer/src/generated/draw_msaa_atlas_blit.webgpu_fixedcolor_frag.wgsl"
            }
            Self::MsaaAtlasVertex => {
                "crates/nuxie-renderer/src/generated/draw_msaa_atlas_blit.webgpu_noclipdistance_vert.wgsl"
            }
            Self::MsaaAtlasVertexClipDistance => {
                "crates/nuxie-renderer/src/generated/draw_msaa_atlas_blit.webgpu_vert.wgsl"
            }
            Self::MsaaAtlasVertexStorageTexture => {
                "crates/nuxie-renderer/src/generated/draw_msaa_atlas_blit.webgpu_nossbo_noclipdistance_vert.wgsl"
            }
            Self::MsaaAtlasVertexStorageTextureClipDistance => {
                "crates/nuxie-renderer/src/generated/draw_msaa_atlas_blit.webgpu_nossbo_vert.wgsl"
            }
            Self::MsaaImageMeshAdvancedFragment => {
                "crates/nuxie-renderer/src/generated/draw_msaa_image_mesh.webgpu_frag.wgsl"
            }
            Self::MsaaImageMeshFixedFragment => {
                "crates/nuxie-renderer/src/generated/draw_msaa_image_mesh.webgpu_fixedcolor_frag.wgsl"
            }
            Self::MsaaImageMeshVertex => {
                "crates/nuxie-renderer/src/generated/draw_msaa_image_mesh.webgpu_noclipdistance_vert.wgsl"
            }
            Self::MsaaImageMeshVertexClipDistance => {
                "crates/nuxie-renderer/src/generated/draw_msaa_image_mesh.webgpu_vert.wgsl"
            }
            Self::MsaaPathAdvancedFragment => {
                "crates/nuxie-renderer/src/generated/draw_msaa_path.webgpu_frag.wgsl"
            }
            Self::MsaaPathFixedFragment => {
                "crates/nuxie-renderer/src/generated/draw_msaa_path.webgpu_fixedcolor_frag.wgsl"
            }
            Self::MsaaPathVertex => {
                "crates/nuxie-renderer/src/generated/draw_msaa_path.webgpu_noclipdistance_vert.wgsl"
            }
            Self::MsaaPathVertexClipDistance => {
                "crates/nuxie-renderer/src/generated/draw_msaa_path.webgpu_vert.wgsl"
            }
            Self::MsaaPathVertexStorageTexture => {
                "crates/nuxie-renderer/src/generated/draw_msaa_path.webgpu_nossbo_noclipdistance_vert.wgsl"
            }
            Self::MsaaPathVertexStorageTextureClipDistance => {
                "crates/nuxie-renderer/src/generated/draw_msaa_path.webgpu_nossbo_vert.wgsl"
            }
            Self::MsaaStencilFragment => {
                "crates/nuxie-renderer/src/generated/draw_msaa_stencil.webgpu_fixedcolor_frag.wgsl"
            }
            Self::MsaaStencilVertex => {
                "crates/nuxie-renderer/src/generated/draw_msaa_stencil.webgpu_noclipdistance_vert.wgsl"
            }
            Self::Solid => "crates/nuxie-renderer/src/solid.wgsl",
            Self::SurfacePresent => "crates/nuxie-renderer/src/surface_present.wgsl",
            Self::TessellateFragment => {
                "crates/nuxie-renderer/src/generated/tessellate.webgpu_frag.wgsl"
            }
            Self::TessellateVertex => {
                "crates/nuxie-renderer/src/generated/tessellate.webgpu_vert.wgsl"
            }
            Self::TessellateVertexStorageTexture => {
                "crates/nuxie-renderer/src/generated/tessellate.webgpu_nossbo_vert.wgsl"
            }
        }
    }
}

pub(crate) fn create(device: &wgpu::Device, key: BuiltinShaderKey) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(key.label()),
        source: wgpu::ShaderSource::Wgsl(key.wgsl().into()),
    })
}

#[cfg(feature = "apple-msl-capture")]
#[doc(hidden)]
pub struct BuiltinShaderCaptureIdentity {
    pub label: &'static str,
    pub source_path: &'static str,
    pub wgsl: &'static str,
}

/// Returns the identities owned by this catalog for the opt-in Metal capture
/// tool. Keeping this list next to the creation seam lets tooling reject
/// authored or internal wgpu shaders without teaching it renderer filenames.
#[cfg(feature = "apple-msl-capture")]
#[doc(hidden)]
pub fn capture_inventory() -> Vec<BuiltinShaderCaptureIdentity> {
    use BuiltinShaderKey::*;

    [
        AdvancedComposite,
        AtlasFillFragment,
        AtlasStrokeFragment,
        AtlasVertex,
        AtlasVertexStorageTexture,
        AtomicAdvancedAtlasBlitFragment,
        AtomicAdvancedImageMeshFragment,
        AtomicAdvancedImageRectFragment,
        AtomicAdvancedInitFragment,
        AtomicAdvancedInteriorFragment,
        AtomicAdvancedPathFragment,
        AtomicAdvancedResolveFragment,
        AtomicAdvancedResolveVertex,
        AtomicAtlasBlitFragment,
        AtomicAtlasBlitVertex,
        AtomicImageMeshFragment,
        AtomicImageMeshVertex,
        AtomicImageRectFragment,
        AtomicImageRectVertex,
        AtomicInitFragment,
        AtomicInitVertex,
        AtomicInteriorFragment,
        AtomicInteriorVertex,
        AtomicPathFragment,
        AtomicPathVertex,
        AtomicResolveFragment,
        AtomicResolveVertex,
        ClockwiseAtomicBorrowedInteriorFragment,
        ClockwiseAtomicBorrowedPathFragment,
        ClockwiseAtomicClipInteriorFragment,
        ClockwiseAtomicClipPathFragment,
        ClockwiseAtomicInteriorFragment,
        ClockwiseAtomicInteriorVertex,
        ClockwiseAtomicPathFragment,
        ClockwiseAtomicPathVertex,
        ClockwiseAtomicSampledClipInteriorFragment,
        ClockwiseAtomicSampledClipPathFragment,
        ColorRampFragment,
        ColorRampVertex,
        Composite,
        MipmapFragment,
        MipmapVertex,
        MsaaAtlasAdvancedFragment,
        MsaaAtlasFixedFragment,
        MsaaAtlasVertex,
        MsaaAtlasVertexClipDistance,
        MsaaAtlasVertexStorageTexture,
        MsaaAtlasVertexStorageTextureClipDistance,
        MsaaImageMeshAdvancedFragment,
        MsaaImageMeshFixedFragment,
        MsaaImageMeshVertex,
        MsaaImageMeshVertexClipDistance,
        MsaaPathAdvancedFragment,
        MsaaPathFixedFragment,
        MsaaPathVertex,
        MsaaPathVertexClipDistance,
        MsaaPathVertexStorageTexture,
        MsaaPathVertexStorageTextureClipDistance,
        MsaaStencilFragment,
        MsaaStencilVertex,
        Solid,
        SurfacePresent,
        TessellateFragment,
        TessellateVertex,
        TessellateVertexStorageTexture,
    ]
    .into_iter()
    .map(|key| BuiltinShaderCaptureIdentity {
        label: key.label(),
        source_path: key.source_path(),
        wgsl: key.wgsl(),
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_have_nonempty_stable_inputs() {
        let keys = [
            BuiltinShaderKey::AdvancedComposite,
            BuiltinShaderKey::AtlasVertex,
            BuiltinShaderKey::AtomicPathVertex,
            BuiltinShaderKey::ClockwiseAtomicPathVertex,
            BuiltinShaderKey::ColorRampVertex,
            BuiltinShaderKey::Composite,
            BuiltinShaderKey::MipmapVertex,
            BuiltinShaderKey::MsaaAtlasVertex,
            BuiltinShaderKey::MsaaImageMeshVertex,
            BuiltinShaderKey::MsaaPathVertex,
            BuiltinShaderKey::MsaaStencilVertex,
            BuiltinShaderKey::Solid,
            BuiltinShaderKey::SurfacePresent,
            BuiltinShaderKey::TessellateVertex,
        ];
        for key in keys {
            assert!(!key.label().is_empty());
            assert!(!key.wgsl().trim().is_empty(), "{key:?}");
        }
    }
}
