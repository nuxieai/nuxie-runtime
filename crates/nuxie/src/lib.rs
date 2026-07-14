//! Public Rust API facade for loading and drawing Rive files.
//!
//! This crate keeps the user-facing surface narrow while the lower-level
//! crates continue to carry the import, graph, runtime, and renderer details.

use std::sync::Arc;

use anyhow::{Context, Result};
use nuxie_binary::{RuntimeFile, read_runtime_file};
use nuxie_graph::{ArtboardGraph, GraphFile};
use nuxie_runtime::{
    ArtboardInstance as RuntimeArtboardInstance, RuntimeOwnedViewModelInstance,
    RuntimeRenderPaintCache, RuntimeRenderPathCache,
    preallocate_render_paint_cache_for_artboard_tree,
};

mod scene;

pub use scene::*;

pub use nuxie_render_api::{
    BlendMode, ColorInt, Factory, FillRule, ImageFilter, ImageSampler, ImageWrap, Mat2D, PathVerb,
    RawPath, RecordingFactory, RenderBuffer, RenderBufferFlags, RenderBufferType, RenderImage,
    RenderPaint, RenderPaintStyle, RenderPath, RenderShader, Renderer, StrokeCap, StrokeJoin,
    Vec2D,
};
pub use nuxie_runtime::{
    LinearAnimationInstance, NoopScriptHost, RuntimeLayerState, RuntimeStateMachineInput,
    ScriptError, ScriptHost, ScriptInstance, ScriptMethod, ScriptModule, ScriptModuleFailure,
    ScriptValue, ScriptingVm, StateMachineInputInstance, StateMachineInputKind,
    StateMachineInstance, StateMachineReportedEvent,
};

#[cfg(feature = "scripting")]
pub use nuxie_scripting::vm::{LuaScriptInstance, ScriptVm};

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
        Self::from_runtime(runtime)
    }

    pub(crate) fn from_runtime(runtime: RuntimeFile) -> Result<Self> {
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
        // Safe by construction: every Artboard is created with an index bounds-
        // checked against this same vec (artboards()/artboard()/artboard_named()).
        #[allow(clippy::indexing_slicing)]
        &self.file.graph.artboards[self.index]
    }

    pub fn animation_count(self) -> usize {
        self.graph().animations.len()
    }

    pub fn state_machine_count(self) -> usize {
        self.graph().state_machines.len()
    }

    /// Name of the state machine at `index`, when it has one.
    pub fn state_machine_name(self, index: usize) -> Option<&'a str> {
        self.graph().state_machines.get(index)?.name.as_deref()
    }

    /// Index of the state machine flagged as the artboard default in the
    /// source file, validated against the artboard's state machine list.
    pub fn default_state_machine_index(self) -> Option<usize> {
        let artboard = self.file.runtime.artboard(self.index)?;
        artboard.property("defaultStateMachineId")?;
        let index = usize::try_from(artboard.uint_property("defaultStateMachineId")?).ok()?;
        (index < self.state_machine_count()).then_some(index)
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

/// Render resources retained across draws of one [`ArtboardInstance`].
///
/// Create this from [`ArtboardInstance::new_render_cache`] with the same
/// factory that will back subsequent draws. The factory's render resources
/// remain owned by this cache until it is dropped.
pub struct ArtboardRenderCache {
    paint: RuntimeRenderPaintCache,
    path: RuntimeRenderPathCache,
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

    /// Instantiate the artboard's default state machine: the one flagged in
    /// the source file when present, otherwise the first state machine.
    pub fn default_state_machine_instance(&self) -> Option<StateMachineInstance> {
        let index = self.artboard().default_state_machine_index().unwrap_or(0);
        self.state_machine_instance(index)
    }

    /// Index of the view model backing this artboard's data binds (the source
    /// `viewModelId`), when it declares one. Artboards with no view model carry
    /// the `0xFFFFFFFF` (-1) sentinel, reported here as `None`.
    pub fn view_model_index(&self) -> Option<usize> {
        let artboard = self.file.runtime.artboard(self.artboard_index)?;
        let view_model_id = artboard.uint_property("viewModelId")?;
        if view_model_id == u32::MAX as u64 {
            return None;
        }
        usize::try_from(view_model_id).ok()
    }

    /// Instantiate this artboard's view model with generated defaults, mirroring
    /// `file->createDefaultViewModelInstance(artboard)` in the C++ runtime.
    /// Returns `None` when the artboard has no view model. Bind the returned
    /// context with [`ArtboardInstance::bind_view_model`] before advancing.
    pub fn instantiate_view_model(&self) -> Option<ViewModelInstance> {
        let view_model_index = self.view_model_index()?;
        let raw = RuntimeOwnedViewModelInstance::new(&self.file.runtime, view_model_index)?;
        Some(ViewModelInstance { raw })
    }

    /// Instantiate this artboard's view model from the source instance at
    /// `instance_index` (the order the instances appear in the file). Returns
    /// `None` when the artboard has no view model or the index is out of range.
    pub fn instantiate_view_model_instance(
        &self,
        instance_index: usize,
    ) -> Option<ViewModelInstance> {
        let view_model_index = self.view_model_index()?;
        let raw = RuntimeOwnedViewModelInstance::from_instance(
            &self.file.runtime,
            view_model_index,
            instance_index,
        )?;
        Some(ViewModelInstance { raw })
    }

    /// Bind `view_model` to this artboard's own data binds and its nested
    /// artboard contexts, mirroring `artboard->bindViewModelInstance(...)` in
    /// the C++ runtime.
    ///
    /// The context is copied into the data-bind graph, so this must be called
    /// again after mutating `view_model` for the change to reach the next
    /// [`ArtboardInstance::advance`]. Returns whether the binding changed
    /// anything. State-machine-driven binds must additionally be bound through
    /// [`StateMachineInstance::bind_owned_view_model_context`] using
    /// [`ViewModelInstance::raw`].
    ///
    pub fn bind_view_model(&mut self, view_model: &ViewModelInstance) -> bool {
        let mut changed = self
            .raw
            .bind_default_view_model_artboard_list_context(&self.file.runtime);
        changed |= self
            .raw
            .bind_owned_view_model_artboard_context(&self.file.runtime, view_model.raw());
        changed
    }

    /// Advance the scene while driving `state_machine`, mirroring the golden
    /// runner's advance order (state machine, nested artboards, data binds,
    /// update pass). Returns whether anything changed.
    pub fn advance_with_state_machine(
        &mut self,
        state_machine: &mut StateMachineInstance,
        elapsed_seconds: f32,
    ) -> bool {
        let mut changed = self
            .raw
            .advance_state_machine_instance(state_machine, elapsed_seconds);
        if self
            .raw
            .advance_nested_artboards_with_state_machine(elapsed_seconds, state_machine)
        {
            self.raw.advance_state_machine_instance(state_machine, 0.0);
            changed = true;
        }
        changed |= self
            .raw
            .advance_artboard_data_binds_with_elapsed(elapsed_seconds);
        changed |= self.raw.update_pass();
        changed
    }

    pub fn draw(&mut self, factory: &mut dyn Factory, renderer: &mut dyn Renderer) -> Result<()> {
        let mut cache = self.new_render_cache(factory);
        self.draw_with_render_cache(factory, renderer, &mut cache)
    }

    /// Allocate the render resources retained by [`Self::draw_with_render_cache`].
    pub fn new_render_cache(&self, factory: &mut dyn Factory) -> ArtboardRenderCache {
        ArtboardRenderCache {
            paint: preallocate_render_paint_cache_for_artboard_tree(
                &self.file.runtime,
                self.artboard().graph(),
                &self.file.graph.artboards,
                factory,
            ),
            path: RuntimeRenderPathCache::default(),
        }
    }

    /// Draw while retaining paint and path handles across frames.
    ///
    /// `cache` must have been created by this instance with a factory backed
    /// by the same renderer integration.
    pub fn draw_with_render_cache(
        &mut self,
        factory: &mut dyn Factory,
        renderer: &mut dyn Renderer,
        cache: &mut ArtboardRenderCache,
    ) -> Result<()> {
        self.raw.update_pass();
        let artboard = self.artboard().graph();
        self.raw
            .prepare_static_artboard_tree_paints(
                &self.file.runtime,
                artboard,
                &self.file.graph.artboards,
                factory,
                &mut cache.paint,
                &mut cache.path,
            )
            .context("failed to prepare Rive paints")?;
        self.raw
            .draw_prepared_static_artboard_with_render_cache(
                &self.file.runtime,
                artboard,
                &self.file.graph.artboards,
                factory,
                renderer,
                &mut cache.paint,
                &mut cache.path,
            )
            .context("failed to draw Rive artboard")
    }
}

/// Owning variant of [`ArtboardInstance`] for hosts that cannot hold a
/// borrow of the [`File`] — editors, long-lived embeddings, FFI surfaces.
/// Shares the file via [`Arc`] and owns the runtime instance; a
/// method-for-method mirror of [`ArtboardInstance`] (which stays the
/// zero-overhead choice when a borrow works).
pub struct OwnedArtboardInstance {
    file: Arc<File>,
    artboard_index: usize,
    raw: RuntimeArtboardInstance,
}

impl OwnedArtboardInstance {
    /// Instantiate `artboard_index` of `file` as an owning instance.
    pub fn instantiate(file: Arc<File>, artboard_index: usize) -> Result<Self> {
        let raw = {
            let artboard = file
                .artboard(artboard_index)
                .with_context(|| format!("artboard index {artboard_index} out of range"))?;
            RuntimeArtboardInstance::from_graph_with_artboards(
                &file.runtime,
                artboard.graph(),
                &file.graph.artboards,
            )
            .with_context(|| {
                format!(
                    "failed to instantiate artboard {}",
                    artboard.name().unwrap_or("<unnamed>")
                )
            })?
        };
        Ok(Self {
            file,
            artboard_index,
            raw,
        })
    }

    /// Instantiate the file's default artboard as an owning instance.
    pub fn instantiate_default(file: Arc<File>) -> Result<Self> {
        let artboard_index = file
            .default_artboard()
            .context("file has no artboards")?
            .index();
        Self::instantiate(file, artboard_index)
    }

    pub fn file(&self) -> &Arc<File> {
        &self.file
    }

    pub fn artboard(&self) -> Artboard<'_> {
        Artboard {
            file: &self.file,
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

    /// See [`ArtboardInstance::default_state_machine_instance`].
    pub fn default_state_machine_instance(&self) -> Option<StateMachineInstance> {
        let index = self.artboard().default_state_machine_index().unwrap_or(0);
        self.state_machine_instance(index)
    }

    /// See [`ArtboardInstance::view_model_index`].
    pub fn view_model_index(&self) -> Option<usize> {
        let artboard = self.file.runtime.artboard(self.artboard_index)?;
        let view_model_id = artboard.uint_property("viewModelId")?;
        if view_model_id == u32::MAX as u64 {
            return None;
        }
        usize::try_from(view_model_id).ok()
    }

    /// See [`ArtboardInstance::instantiate_view_model`].
    pub fn instantiate_view_model(&self) -> Option<ViewModelInstance> {
        let view_model_index = self.view_model_index()?;
        let raw = RuntimeOwnedViewModelInstance::new(&self.file.runtime, view_model_index)?;
        Some(ViewModelInstance { raw })
    }

    /// See [`ArtboardInstance::instantiate_view_model_instance`].
    pub fn instantiate_view_model_instance(
        &self,
        instance_index: usize,
    ) -> Option<ViewModelInstance> {
        let view_model_index = self.view_model_index()?;
        let raw = RuntimeOwnedViewModelInstance::from_instance(
            &self.file.runtime,
            view_model_index,
            instance_index,
        )?;
        Some(ViewModelInstance { raw })
    }

    /// See [`ArtboardInstance::bind_view_model`].
    pub fn bind_view_model(&mut self, view_model: &ViewModelInstance) -> bool {
        let mut changed = self
            .raw
            .bind_default_view_model_artboard_list_context(&self.file.runtime);
        changed |= self
            .raw
            .bind_owned_view_model_artboard_context(&self.file.runtime, view_model.raw());
        changed
    }

    /// See [`ArtboardInstance::advance_with_state_machine`].
    pub fn advance_with_state_machine(
        &mut self,
        state_machine: &mut StateMachineInstance,
        elapsed_seconds: f32,
    ) -> bool {
        let mut changed = self
            .raw
            .advance_state_machine_instance(state_machine, elapsed_seconds);
        if self
            .raw
            .advance_nested_artboards_with_state_machine(elapsed_seconds, state_machine)
        {
            self.raw.advance_state_machine_instance(state_machine, 0.0);
            changed = true;
        }
        changed |= self
            .raw
            .advance_artboard_data_binds_with_elapsed(elapsed_seconds);
        changed |= self.raw.update_pass();
        changed
    }

    pub fn draw(&mut self, factory: &mut dyn Factory, renderer: &mut dyn Renderer) -> Result<()> {
        let mut cache = self.new_render_cache(factory);
        self.draw_with_render_cache(factory, renderer, &mut cache)
    }

    /// See [`ArtboardInstance::new_render_cache`].
    pub fn new_render_cache(&self, factory: &mut dyn Factory) -> ArtboardRenderCache {
        ArtboardRenderCache {
            paint: preallocate_render_paint_cache_for_artboard_tree(
                &self.file.runtime,
                self.artboard().graph(),
                &self.file.graph.artboards,
                factory,
            ),
            path: RuntimeRenderPathCache::default(),
        }
    }

    /// See [`ArtboardInstance::draw_with_render_cache`].
    pub fn draw_with_render_cache(
        &mut self,
        factory: &mut dyn Factory,
        renderer: &mut dyn Renderer,
        cache: &mut ArtboardRenderCache,
    ) -> Result<()> {
        self.raw.update_pass();
        let artboard = self.artboard().graph();
        self.raw
            .prepare_static_artboard_tree_paints(
                &self.file.runtime,
                artboard,
                &self.file.graph.artboards,
                factory,
                &mut cache.paint,
                &mut cache.path,
            )
            .context("failed to prepare Rive paints")?;
        self.raw
            .draw_prepared_static_artboard_with_render_cache(
                &self.file.runtime,
                artboard,
                &self.file.graph.artboards,
                factory,
                renderer,
                &mut cache.paint,
                &mut cache.path,
            )
            .context("failed to draw Rive artboard")
    }
}

/// Owned view-model context for driving an artboard's data binds.
///
/// Instantiate one from an [`ArtboardInstance`], set properties by name path,
/// bind it with [`ArtboardInstance::bind_view_model`], then advance and draw.
/// The context owns its own copy of the view model's values, so it does not
/// borrow the originating [`File`] and can outlive nothing in particular; it is
/// only meaningful when bound back to the artboard it came from.
///
/// Property paths address nested view models with `/` separators (for example
/// `"child/width"`); a single segment addresses a property on the root view
/// model. Every setter returns whether a matching, settable property existed
/// and its value changed.
#[derive(Debug, Clone)]
pub struct ViewModelInstance {
    raw: RuntimeOwnedViewModelInstance,
}

impl ViewModelInstance {
    /// Low-level owned context for advanced integrations (for example binding
    /// through [`StateMachineInstance::bind_owned_view_model_context`]).
    pub fn raw(&self) -> &RuntimeOwnedViewModelInstance {
        &self.raw
    }

    /// Low-level mutable owned context for advanced integrations.
    pub fn raw_mut(&mut self) -> &mut RuntimeOwnedViewModelInstance {
        &mut self.raw
    }

    /// Set a number property by name path. Returns whether the property existed
    /// and changed.
    pub fn set_number(&mut self, name_path: &str, value: f32) -> bool {
        self.raw.set_number_by_property_name_path(name_path, value)
    }

    /// Set a boolean property by name path. Returns whether the property existed
    /// and changed.
    pub fn set_bool(&mut self, name_path: &str, value: bool) -> bool {
        self.raw.set_boolean_by_property_name_path(name_path, value)
    }

    /// Set a string property by name path. The value is stored as its UTF-8
    /// bytes. Returns whether the property existed and changed.
    pub fn set_string(&mut self, name_path: &str, value: &str) -> bool {
        self.raw
            .set_string_by_property_name_path(name_path, value.as_bytes())
    }

    /// Set an enum property by its numeric value at the given name path. Returns
    /// whether the property existed and changed. (Enum-by-value-name is not
    /// exposed here because the owned context resolves enums by index.)
    pub fn set_enum(&mut self, name_path: &str, value: u64) -> bool {
        self.raw.set_enum_by_property_name_path(name_path, value)
    }
}

#[cfg(test)]
mod owned_instance_tests {
    use super::*;
    use nuxie_render_api::RecordingFactory;

    const FIXTURE: &[u8] = include_bytes!("../../../fixtures/graph/dependency_test.riv");

    fn stream_of(draw: impl FnOnce(&mut RecordingFactory) -> Result<()>) -> String {
        let mut factory = RecordingFactory::new();
        draw(&mut factory).expect("draw succeeds");
        factory.stream()
    }

    #[test]
    fn owned_instance_draws_identically_to_borrowed() {
        let borrowed_stream = stream_of(|factory| {
            let file = File::import(FIXTURE)?;
            let artboard = file.default_artboard().context("default artboard")?;
            let mut instance = artboard.instantiate()?;
            instance.advance(0.0);
            let mut renderer = factory.make_renderer();
            instance.draw(factory, &mut renderer)
        });

        let owned_stream = stream_of(|factory| {
            let file = Arc::new(File::import(FIXTURE)?);
            let mut instance = OwnedArtboardInstance::instantiate_default(file)?;
            instance.advance(0.0);
            let mut renderer = factory.make_renderer();
            instance.draw(factory, &mut renderer)
        });

        assert_eq!(
            owned_stream, borrowed_stream,
            "owned and borrowed instances must draw the identical stream"
        );
    }

    #[test]
    fn owned_instance_outlives_the_importing_scope() {
        let mut instance = {
            let file = Arc::new(File::import(FIXTURE).expect("import"));
            OwnedArtboardInstance::instantiate_default(file).expect("instantiate")
        };
        assert!(!instance.advance(0.016) || instance.raw().components().len() > 0);
    }

    #[test]
    fn promoted_property_writes_report_missing_targets() {
        let file = Arc::new(File::import(FIXTURE).expect("import"));
        let mut instance = OwnedArtboardInstance::instantiate_default(file).expect("instantiate");
        // Nonexistent property key: the typed write path must report false
        // (no match), never panic.
        assert!(!instance.raw_mut().set_double_property(0, u16::MAX, 1.0));
    }
}
