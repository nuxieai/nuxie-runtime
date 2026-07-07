//! Public Rust API facade for loading and drawing Rive files.
//!
//! This crate keeps the user-facing surface narrow while the lower-level
//! crates continue to carry the import, graph, runtime, and renderer details.

use anyhow::{Context, Result};
use rive_binary::{RuntimeFile, read_runtime_file};
use rive_graph::{ArtboardGraph, GraphFile};
use rive_render_api::{Factory, Renderer};
use rive_runtime::{
    ArtboardInstance as RuntimeArtboardInstance, RuntimeRenderPathCache,
    preallocate_render_paint_cache_for_artboard_tree,
};

pub use rive_render_api::{
    BlendMode, ColorInt, FillRule, ImageFilter, ImageSampler, ImageWrap, Mat2D, PathVerb, RawPath,
    RecordingFactory, RenderBuffer, RenderBufferFlags, RenderBufferType, RenderImage, RenderPaint,
    RenderPaintStyle, RenderPath, RenderShader, StrokeCap, StrokeJoin, Vec2D,
};
pub use rive_runtime::{
    LinearAnimationInstance, RuntimeLayerState, RuntimeStateMachineInput,
    StateMachineInputInstance, StateMachineInputKind, StateMachineInstance,
    StateMachineReportedEvent,
};

/// Imported Rive file plus its runtime graph projection.
#[derive(Debug, Clone)]
pub struct File {
    runtime: RuntimeFile,
    graph: GraphFile,
}

impl File {
    /// Import `.riv` bytes and build the runtime graph needed for instancing.
    pub fn import(bytes: &[u8]) -> Result<Self> {
        let runtime = read_runtime_file(bytes).context("failed to import Rive file")?;
        let graph = GraphFile::from_runtime_file(&runtime).context("failed to build Rive graph")?;
        Ok(Self { runtime, graph })
    }

    /// Low-level imported file data for advanced integrations.
    pub fn runtime(&self) -> &RuntimeFile {
        &self.runtime
    }

    /// Low-level graph projection for advanced integrations.
    pub fn graph(&self) -> &GraphFile {
        &self.graph
    }

    pub fn artboard_count(&self) -> usize {
        self.graph.artboards.len()
    }

    pub fn artboards(&self) -> impl ExactSizeIterator<Item = Artboard<'_>> + '_ {
        (0..self.artboard_count()).map(|index| Artboard { file: self, index })
    }

    pub fn default_artboard(&self) -> Option<Artboard<'_>> {
        self.artboard(0)
    }

    pub fn artboard(&self, index: usize) -> Option<Artboard<'_>> {
        (index < self.artboard_count()).then_some(Artboard { file: self, index })
    }

    pub fn artboard_named(&self, name: &str) -> Option<Artboard<'_>> {
        let index = self
            .graph
            .artboards
            .iter()
            .position(|artboard| artboard.name.as_deref() == Some(name))?;
        Some(Artboard { file: self, index })
    }
}

/// Borrowed handle to an artboard inside an imported [`File`].
#[derive(Debug, Clone, Copy)]
pub struct Artboard<'a> {
    file: &'a File,
    index: usize,
}

impl<'a> Artboard<'a> {
    pub fn index(self) -> usize {
        self.index
    }

    pub fn name(self) -> Option<&'a str> {
        self.graph().name.as_deref()
    }

    pub fn graph(self) -> &'a ArtboardGraph {
        &self.file.graph.artboards[self.index]
    }

    pub fn animation_count(self) -> usize {
        self.graph().animations.len()
    }

    pub fn state_machine_count(self) -> usize {
        self.graph().state_machines.len()
    }

    pub fn instantiate(self) -> Result<ArtboardInstance<'a>> {
        let raw = RuntimeArtboardInstance::from_graph_with_artboards(
            &self.file.runtime,
            self.graph(),
            &self.file.graph.artboards,
        )
        .with_context(|| {
            format!(
                "failed to instantiate artboard {}",
                self.name().unwrap_or("<unnamed>")
            )
        })?;
        Ok(ArtboardInstance {
            file: self.file,
            artboard_index: self.index,
            raw,
        })
    }
}

/// User-facing artboard instance that keeps file and graph context available.
#[derive(Debug)]
pub struct ArtboardInstance<'a> {
    file: &'a File,
    artboard_index: usize,
    raw: RuntimeArtboardInstance,
}

impl<'a> ArtboardInstance<'a> {
    pub fn artboard(&self) -> Artboard<'a> {
        Artboard {
            file: self.file,
            index: self.artboard_index,
        }
    }

    pub fn raw(&self) -> &RuntimeArtboardInstance {
        &self.raw
    }

    pub fn raw_mut(&mut self) -> &mut RuntimeArtboardInstance {
        &mut self.raw
    }

    pub fn advance_nested_artboards(&mut self, elapsed_seconds: f32) -> bool {
        self.raw.advance_nested_artboards(elapsed_seconds)
    }

    pub fn advance(&mut self, elapsed_seconds: f32) -> bool {
        let mut changed = self.raw.advance_nested_artboards(elapsed_seconds);
        changed |= self
            .raw
            .advance_artboard_data_binds_with_elapsed(elapsed_seconds);
        changed |= self.raw.update_pass();
        changed
    }

    pub fn state_machine_instance(&self, index: usize) -> Option<StateMachineInstance> {
        self.raw.state_machine_instance(index)
    }

    pub fn draw(&mut self, factory: &mut dyn Factory, renderer: &mut dyn Renderer) -> Result<()> {
        self.raw.update_pass();
        let artboard = self.artboard().graph();
        let mut paint_cache = preallocate_render_paint_cache_for_artboard_tree(
            &self.file.runtime,
            artboard,
            &self.file.graph.artboards,
            factory,
        );
        let mut path_cache = RuntimeRenderPathCache::default();
        self.raw
            .prepare_static_artboard_tree_paints(
                &self.file.runtime,
                artboard,
                &self.file.graph.artboards,
                factory,
                &mut paint_cache,
                &mut path_cache,
            )
            .context("failed to prepare Rive paints")?;
        self.raw
            .draw_prepared_static_artboard_with_render_cache(
                &self.file.runtime,
                artboard,
                &self.file.graph.artboards,
                factory,
                renderer,
                &mut paint_cache,
                &mut path_cache,
            )
            .context("failed to draw Rive artboard")
    }
}
