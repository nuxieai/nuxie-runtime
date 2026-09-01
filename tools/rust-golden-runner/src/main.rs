use anyhow::{Context, Result, anyhow, bail};
use nuxie_render_api::{
    Factory as RenderFactory, NullFactory, PersistentFactory, RecordingFactory,
    Renderer as RenderRenderer, SideChannelEvent, SideChannelEventProperty,
    SideChannelEventPropertyValue, SideChannelSemanticsBoundsUpdate,
    SideChannelSemanticsChildrenUpdate, SideChannelSemanticsDiff, SideChannelSemanticsNode,
};
use nuxie_runtime::source::{
    animation::state_machine_instance::{EventReport, RuntimeStateMachineInstanceHandle},
    generated::core_registry::CoreRegistry,
    hit_result::HitResult,
    lua::scripting_vm::RuntimeScriptingVmHandle,
    math::{random::set_runtime_deterministic_mode, vec2d::Vec2D},
    semantic::semantic_snapshot::{SemanticsDiff, SemanticsDiffNode},
    static_scene::StaticScene,
    viewmodel::runtime::viewmodel_instance_runtime::ViewModelInstanceRuntime,
};
use nuxie_runtime::{
    Artboard, CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle,
    RuntimeFileHandle,
};
use sha2::{Digest, Sha256};
#[cfg(feature = "coverage-trace")]
use std::alloc::{GlobalAlloc, Layout, System};
use std::env;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
#[cfg(feature = "coverage-trace")]
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

const TIME_EPSILON: f32 = 0.000001;

#[cfg(feature = "coverage-trace")]
struct FrameLoopCountingAllocator;

#[cfg(feature = "coverage-trace")]
static COUNT_FRAME_LOOP_ALLOCATIONS: AtomicBool = AtomicBool::new(false);
#[cfg(feature = "coverage-trace")]
static FRAME_LOOP_ALLOCATIONS: AtomicU64 = AtomicU64::new(0);

#[cfg(feature = "coverage-trace")]
unsafe impl GlobalAlloc for FrameLoopCountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if COUNT_FRAME_LOOP_ALLOCATIONS.load(Ordering::Relaxed) {
            FRAME_LOOP_ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        // SAFETY: this allocator delegates the unchanged layout to System.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: `pointer` was allocated by the delegated System allocator.
        unsafe { System.dealloc(pointer, layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if COUNT_FRAME_LOOP_ALLOCATIONS.load(Ordering::Relaxed) {
            FRAME_LOOP_ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        // SAFETY: this allocator delegates the unchanged layout to System.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        if COUNT_FRAME_LOOP_ALLOCATIONS.load(Ordering::Relaxed) {
            FRAME_LOOP_ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        // SAFETY: `pointer` and `layout` came from the delegated allocator.
        unsafe { System.realloc(pointer, layout, size) }
    }
}

#[cfg(feature = "coverage-trace")]
#[global_allocator]
static FRAME_LOOP_COUNTING_ALLOCATOR: FrameLoopCountingAllocator = FrameLoopCountingAllocator;

#[cfg(feature = "coverage-trace")]
unsafe extern "C" {
    fn __llvm_profile_reset_counters();
}

fn reset_coverage_profile_for_frame_loop_if_requested() {
    #[cfg(feature = "coverage-trace")]
    if env::var_os("RIVE_GOLDEN_COVERAGE_FRAME_ONLY").is_some() {
        // SAFETY: this feature is only linked with `-Cinstrument-coverage`;
        // the symbol is supplied by that compiler runtime.
        unsafe {
            __llvm_profile_reset_counters();
        }
    }
}

fn reset_coverage_profile_for_occurrence_if_requested() {
    #[cfg(feature = "coverage-trace")]
    if env::var_os("RIVE_GOLDEN_COVERAGE_OCCURRENCE_ONLY").is_some() {
        // SAFETY: this feature is only linked with `-Cinstrument-coverage`;
        // the symbol is supplied by that compiler runtime.
        unsafe {
            __llvm_profile_reset_counters();
        }
    }
}

fn reset_frame_loop_allocation_counter_if_requested() {
    #[cfg(feature = "coverage-trace")]
    if env::var_os("RIVE_GOLDEN_ALLOCATION_COUNTER").is_some() {
        FRAME_LOOP_ALLOCATIONS.store(0, Ordering::Relaxed);
        COUNT_FRAME_LOOP_ALLOCATIONS.store(true, Ordering::Relaxed);
    }
}

fn stop_frame_loop_allocation_counter() -> u64 {
    #[cfg(feature = "coverage-trace")]
    {
        COUNT_FRAME_LOOP_ALLOCATIONS.store(false, Ordering::Relaxed);
        return FRAME_LOOP_ALLOCATIONS.load(Ordering::Relaxed);
    }
    #[cfg(not(feature = "coverage-trace"))]
    0
}

fn validate_trace_options(options: &Options) -> Result<()> {
    let frame_only = env::var_os("RIVE_GOLDEN_COVERAGE_FRAME_ONLY").is_some();
    let allocations = env::var_os("RIVE_GOLDEN_ALLOCATION_COUNTER").is_some();
    let steady_only = env::var_os("RIVE_GOLDEN_COVERAGE_STEADY_ONLY").is_some();
    let occurrence_only = env::var_os("RIVE_GOLDEN_COVERAGE_OCCURRENCE_ONLY").is_some();
    let mechanism_input = env::var_os("RIVE_GOLDEN_COVERAGE_MECHANISM_INPUT").is_some();

    #[cfg(not(feature = "coverage-trace"))]
    if frame_only || allocations || occurrence_only || mechanism_input {
        bail!(
            "frame-loop coverage/allocation tracing requires \
             --features coverage-trace and RUSTFLAGS=-Cinstrument-coverage"
        );
    }

    if options.benchmark_repeat > 1 && (frame_only || allocations) {
        bail!(
            "frame-only coverage and allocation tracing require \
             --benchmark-repeat 1"
        );
    }
    if options.layout_bounds && (frame_only || allocations) {
        bail!("frame-loop tracing cannot be combined with --layout-bounds");
    }
    if mechanism_input
        && (!frame_only
            || occurrence_only
            || steady_only
            || options.input_script.is_none()
            || options.benchmark_repeat != 1)
    {
        bail!(
            "mechanism input coverage requires frame-only coverage, an input \
             script, --benchmark-repeat 1, and non-occurrence/non-steady mode"
        );
    }
    if steady_only
        && (!frame_only
            || options.samples.len() != 1
            || options.benchmark_repeat != 1
            || options.input_script.is_some()
            || options.view_model_script.is_some())
    {
        bail!(
            "steady-only coverage requires frame-only coverage, one sample, \
             --benchmark-repeat 1, and no input script"
        );
    }
    Ok(())
}

trait RunnerBackend {
    fn as_factory(&mut self) -> &mut dyn RenderFactory;
    fn make_renderer(&self) -> Box<dyn RenderRenderer>;
    fn source(&mut self, file: &str, artboard: &str, scene: &str);
    fn frame_size(&mut self, width: u32, height: u32);
    fn add_input_event(&mut self, kind: &str, seconds: f32, x: f32, y: f32, pointer_id: i32);
    fn add_set_input_boolean(&mut self, seconds: f32, name: &str, value: bool);
    fn add_set_input_number(&mut self, seconds: f32, name: &str, value: f32);
    fn add_set_input_trigger(&mut self, seconds: f32, name: &str);
    fn add_view_model_boolean(&mut self, seconds: f32, property: &str, value: bool);
    fn add_view_model_number(&mut self, seconds: f32, property: &str, value: f32);
    fn add_view_model_string(&mut self, seconds: f32, property: &str, value: &str);
    fn add_view_model_enum(&mut self, seconds: f32, property: &str, value: u32);
    fn add_view_model_color(&mut self, seconds: f32, property: &str, value: u32);
    fn add_view_model_trigger(&mut self, seconds: f32, property: &str);
    fn add_resize(
        &mut self,
        seconds: f32,
        width: f32,
        height: f32,
        dpr: f32,
        pixel_width: u32,
        pixel_height: u32,
    );
    fn add_sample(&mut self, seconds: f32);
    fn add_advance(&mut self, seconds: f32, settled: bool);
    fn add_advance_with_states(&mut self, seconds: f32, settled: bool, states_changed: usize);
    fn add_side_channel_event(&mut self, event: &SideChannelEvent);
    fn add_semantics_diff(&mut self, diff: &SideChannelSemanticsDiff);
    fn add_semantic_action(&mut self, seconds: f32, node_id: u32, action: &str, dispatched: bool);
    fn add_semantic_focus(&mut self, seconds: f32, node_id: u32, focused: bool);
    fn add_hit_result(&mut self, result: &str);
    fn add_frame(&mut self);
    fn stream(&self) -> String;
}

impl RunnerBackend for RecordingFactory {
    fn as_factory(&mut self) -> &mut dyn RenderFactory {
        self
    }

    fn make_renderer(&self) -> Box<dyn RenderRenderer> {
        Box::new(RecordingFactory::make_renderer(self))
    }

    fn source(&mut self, file: &str, artboard: &str, scene: &str) {
        RecordingFactory::source(self, file, artboard, scene);
    }

    fn frame_size(&mut self, width: u32, height: u32) {
        RecordingFactory::frame_size(self, width, height);
    }

    fn add_input_event(&mut self, kind: &str, seconds: f32, x: f32, y: f32, pointer_id: i32) {
        RecordingFactory::add_input_event(self, kind, seconds, x, y, pointer_id);
    }

    fn add_set_input_boolean(&mut self, seconds: f32, name: &str, value: bool) {
        RecordingFactory::add_set_input_boolean(self, seconds, name, value);
    }

    fn add_set_input_number(&mut self, seconds: f32, name: &str, value: f32) {
        RecordingFactory::add_set_input_number(self, seconds, name, value);
    }

    fn add_set_input_trigger(&mut self, seconds: f32, name: &str) {
        RecordingFactory::add_set_input_trigger(self, seconds, name);
    }

    fn add_view_model_boolean(&mut self, seconds: f32, property: &str, value: bool) {
        RecordingFactory::add_view_model_boolean(self, seconds, property, value);
    }

    fn add_view_model_number(&mut self, seconds: f32, property: &str, value: f32) {
        RecordingFactory::add_view_model_number(self, seconds, property, value);
    }

    fn add_view_model_string(&mut self, seconds: f32, property: &str, value: &str) {
        RecordingFactory::add_view_model_string(self, seconds, property, value);
    }

    fn add_view_model_enum(&mut self, seconds: f32, property: &str, value: u32) {
        RecordingFactory::add_view_model_enum(self, seconds, property, value);
    }

    fn add_view_model_color(&mut self, seconds: f32, property: &str, value: u32) {
        RecordingFactory::add_view_model_color(self, seconds, property, value);
    }

    fn add_view_model_trigger(&mut self, seconds: f32, property: &str) {
        RecordingFactory::add_view_model_trigger(self, seconds, property);
    }

    fn add_resize(
        &mut self,
        seconds: f32,
        width: f32,
        height: f32,
        dpr: f32,
        pixel_width: u32,
        pixel_height: u32,
    ) {
        RecordingFactory::add_resize(self, seconds, width, height, dpr, pixel_width, pixel_height);
    }

    fn add_sample(&mut self, seconds: f32) {
        RecordingFactory::add_sample(self, seconds);
    }

    fn add_advance(&mut self, seconds: f32, settled: bool) {
        RecordingFactory::add_advance(self, seconds, settled);
    }

    fn add_advance_with_states(&mut self, seconds: f32, settled: bool, states_changed: usize) {
        RecordingFactory::add_advance_with_states(self, seconds, settled, states_changed);
    }

    fn add_side_channel_event(&mut self, event: &SideChannelEvent) {
        RecordingFactory::add_side_channel_event(self, event);
    }

    fn add_semantics_diff(&mut self, diff: &SideChannelSemanticsDiff) {
        RecordingFactory::add_semantics_diff(self, diff);
    }

    fn add_semantic_action(&mut self, seconds: f32, node_id: u32, action: &str, dispatched: bool) {
        RecordingFactory::add_semantic_action(self, seconds, node_id, action, dispatched);
    }

    fn add_semantic_focus(&mut self, seconds: f32, node_id: u32, focused: bool) {
        RecordingFactory::add_semantic_focus(self, seconds, node_id, focused);
    }

    fn add_hit_result(&mut self, result: &str) {
        RecordingFactory::add_hit_result(self, result);
    }

    fn add_frame(&mut self) {
        RecordingFactory::add_frame(self);
    }

    fn stream(&self) -> String {
        RecordingFactory::stream(self)
    }
}

impl RunnerBackend for NullFactory {
    fn as_factory(&mut self) -> &mut dyn RenderFactory {
        self
    }

    fn make_renderer(&self) -> Box<dyn RenderRenderer> {
        Box::new(NullFactory::make_renderer(self))
    }

    fn source(&mut self, _file: &str, _artboard: &str, _scene: &str) {}

    fn frame_size(&mut self, _width: u32, _height: u32) {}

    fn add_input_event(&mut self, _kind: &str, _seconds: f32, _x: f32, _y: f32, _pointer_id: i32) {}

    fn add_set_input_boolean(&mut self, _seconds: f32, _name: &str, _value: bool) {}

    fn add_set_input_number(&mut self, _seconds: f32, _name: &str, _value: f32) {}

    fn add_set_input_trigger(&mut self, _seconds: f32, _name: &str) {}

    fn add_view_model_boolean(&mut self, _seconds: f32, _property: &str, _value: bool) {}

    fn add_view_model_number(&mut self, _seconds: f32, _property: &str, _value: f32) {}

    fn add_view_model_string(&mut self, _seconds: f32, _property: &str, _value: &str) {}

    fn add_view_model_enum(&mut self, _seconds: f32, _property: &str, _value: u32) {}

    fn add_view_model_color(&mut self, _seconds: f32, _property: &str, _value: u32) {}

    fn add_view_model_trigger(&mut self, _seconds: f32, _property: &str) {}

    fn add_resize(
        &mut self,
        _seconds: f32,
        _width: f32,
        _height: f32,
        _dpr: f32,
        _pixel_width: u32,
        _pixel_height: u32,
    ) {
    }

    fn add_sample(&mut self, _seconds: f32) {}

    fn add_advance(&mut self, _seconds: f32, _settled: bool) {}

    fn add_advance_with_states(&mut self, _seconds: f32, _settled: bool, _states_changed: usize) {}

    fn add_side_channel_event(&mut self, _event: &SideChannelEvent) {}

    fn add_semantics_diff(&mut self, _diff: &SideChannelSemanticsDiff) {}

    fn add_semantic_action(
        &mut self,
        _seconds: f32,
        _node_id: u32,
        _action: &str,
        _dispatched: bool,
    ) {
    }

    fn add_semantic_focus(&mut self, _seconds: f32, _node_id: u32, _focused: bool) {}

    fn add_hit_result(&mut self, _result: &str) {}

    fn add_frame(&mut self) {}

    fn stream(&self) -> String {
        String::new()
    }
}

impl<F> RunnerBackend for PersistentFactory<F>
where
    F: RunnerBackend + RenderFactory + 'static,
{
    fn as_factory(&mut self) -> &mut dyn RenderFactory {
        self
    }

    fn make_renderer(&self) -> Box<dyn RenderRenderer> {
        self.borrow().make_renderer()
    }

    fn source(&mut self, file: &str, artboard: &str, scene: &str) {
        self.borrow_mut().source(file, artboard, scene);
    }

    fn frame_size(&mut self, width: u32, height: u32) {
        self.borrow_mut().frame_size(width, height);
    }

    fn add_input_event(&mut self, kind: &str, seconds: f32, x: f32, y: f32, pointer_id: i32) {
        self.borrow_mut()
            .add_input_event(kind, seconds, x, y, pointer_id);
    }

    fn add_set_input_boolean(&mut self, seconds: f32, name: &str, value: bool) {
        self.borrow_mut()
            .add_set_input_boolean(seconds, name, value);
    }

    fn add_set_input_number(&mut self, seconds: f32, name: &str, value: f32) {
        self.borrow_mut().add_set_input_number(seconds, name, value);
    }

    fn add_set_input_trigger(&mut self, seconds: f32, name: &str) {
        self.borrow_mut().add_set_input_trigger(seconds, name);
    }

    fn add_view_model_boolean(&mut self, seconds: f32, property: &str, value: bool) {
        self.borrow_mut()
            .add_view_model_boolean(seconds, property, value);
    }

    fn add_view_model_number(&mut self, seconds: f32, property: &str, value: f32) {
        self.borrow_mut()
            .add_view_model_number(seconds, property, value);
    }

    fn add_view_model_string(&mut self, seconds: f32, property: &str, value: &str) {
        self.borrow_mut()
            .add_view_model_string(seconds, property, value);
    }

    fn add_view_model_enum(&mut self, seconds: f32, property: &str, value: u32) {
        self.borrow_mut()
            .add_view_model_enum(seconds, property, value);
    }

    fn add_view_model_color(&mut self, seconds: f32, property: &str, value: u32) {
        self.borrow_mut()
            .add_view_model_color(seconds, property, value);
    }

    fn add_view_model_trigger(&mut self, seconds: f32, property: &str) {
        self.borrow_mut().add_view_model_trigger(seconds, property);
    }

    fn add_resize(
        &mut self,
        seconds: f32,
        width: f32,
        height: f32,
        dpr: f32,
        pixel_width: u32,
        pixel_height: u32,
    ) {
        self.borrow_mut()
            .add_resize(seconds, width, height, dpr, pixel_width, pixel_height);
    }

    fn add_sample(&mut self, seconds: f32) {
        self.borrow_mut().add_sample(seconds);
    }

    fn add_advance(&mut self, seconds: f32, settled: bool) {
        self.borrow_mut().add_advance(seconds, settled);
    }

    fn add_advance_with_states(&mut self, seconds: f32, settled: bool, states_changed: usize) {
        self.borrow_mut()
            .add_advance_with_states(seconds, settled, states_changed);
    }

    fn add_side_channel_event(&mut self, event: &SideChannelEvent) {
        self.borrow_mut().add_side_channel_event(event);
    }

    fn add_semantics_diff(&mut self, diff: &SideChannelSemanticsDiff) {
        self.borrow_mut().add_semantics_diff(diff);
    }

    fn add_semantic_action(&mut self, seconds: f32, node_id: u32, action: &str, dispatched: bool) {
        self.borrow_mut()
            .add_semantic_action(seconds, node_id, action, dispatched);
    }

    fn add_semantic_focus(&mut self, seconds: f32, node_id: u32, focused: bool) {
        self.borrow_mut()
            .add_semantic_focus(seconds, node_id, focused);
    }

    fn add_hit_result(&mut self, result: &str) {
        self.borrow_mut().add_hit_result(result);
    }

    fn add_frame(&mut self) {
        self.borrow_mut().add_frame();
    }

    fn stream(&self) -> String {
        self.borrow().stream()
    }
}

fn main() {
    match run() {
        Ok(stream) => print!("{stream}"),
        Err(error) => {
            eprintln!("rust-golden-runner error: {error:#}");
            std::process::exit(1);
        }
    }
}

/// Harness-owned handles only. Import, scripting, dependency updates, resource
/// creation, and drawing all run through the translated native owners.
struct LoadedScene {
    _file: RuntimeFileHandle,
    artboard: RuntimeArtboardInstanceHandle,
    machine: Option<RuntimeStateMachineInstanceHandle>,
    static_scene: Option<StaticScene>,
    main: Option<ViewModelInstanceRuntime>,
    artboard_index: usize,
    machine_index: Option<usize>,
    artboard_name: String,
    scene_name: String,
}

fn import_file(
    bytes: &[u8],
    factory: &mut dyn RenderFactory,
    execute_scripts: bool,
) -> Result<RuntimeFileHandle> {
    let retained = RuntimeFactoryHandle::from_factory(factory)
        .context("golden runner requires a retained renderer factory")?;
    let vm: Option<RuntimeScriptingVmHandle> = if execute_scripts {
        #[cfg(feature = "scripting")]
        {
            Some(RuntimeScriptingVmHandle::new(Box::new(
                nuxie_scripting::vm::ScriptVm::new(),
            )))
        }
        #[cfg(not(feature = "scripting"))]
        {
            bail!("--execute-scripts requires the scripting feature");
        }
    } else {
        None
    };
    File::import(bytes, retained, None, None, vm).context("native File import failed")
}

impl LoadedScene {
    fn load(bytes: &[u8], factory: &mut dyn RenderFactory, options: &Options) -> Result<Self> {
        let file = import_file(bytes, factory, options.execute_scripts)?;
        Self::from_file(file, options)
    }

    fn from_file(file: RuntimeFileHandle, options: &Options) -> Result<Self> {
        let definitions = file.with_file(|file| file.artboards().to_vec());
        let artboard_index = if let Some(name) = options.artboard.as_deref() {
            definitions
                .iter()
                .position(|owner| {
                    owner
                        .with(|owner| {
                            owner
                                .as_artboard()
                                .is_some_and(|owner| owner.name() == name)
                        })
                        .unwrap_or(false)
                })
                .with_context(|| format!("artboard '{name}' was not found"))?
        } else {
            if definitions.is_empty() {
                bail!("file has no artboards");
            }
            0
        };
        // Match the C++ occurrence boundary: definitions have already imported,
        // but the selected ArtboardInstance has not yet been cloned.
        reset_coverage_profile_for_occurrence_if_requested();
        let artboard = Artboard::instance_from_handle(&definitions[artboard_index])
            .context("failed to instantiate selected native artboard")?;
        let root = artboard.core_handle();
        let artboard_name = artboard.with_artboard(|artboard| artboard.name().to_owned());
        // RIVLoader binds the Artboard first, constructs its default/named SMI
        // second, then binds the scene to that same view-model occurrence.
        #[cfg(feature = "scripting")]
        let main = {
            let model_id = artboard.with_artboard(|artboard| artboard.view_model_id());
            file.with_file_mut(|file| {
                if model_id == u32::MAX {
                    file.create_view_model_instance_for_artboard(root.clone())
                } else {
                    file.create_view_model_instance_at(model_id as usize, 0)
                }
            })
        };
        #[cfg(not(feature = "scripting"))]
        let main = file.with_file_mut(|file| {
            if options.semantic_default_view_model {
                file.create_default_view_model_instance_for_artboard(root.clone())
            } else {
                file.create_view_model_instance_for_artboard(root.clone())
            }
        });
        artboard.bind_view_model_instance(main.clone());
        let machine_index = if let Some(name) = options.state_machine.as_deref() {
            Some(
                artboard
                    .with_artboard(|artboard| {
                        artboard.state_machine_handles().iter().position(|machine| {
                            machine
                                .with_downcast::<nuxie_runtime::source::animation::state_machine::StateMachine, _>(|machine| {
                                    machine.name() == name
                                })
                                .unwrap_or(false)
                        })
                    })
                    .with_context(|| format!("state machine '{name}' was not found"))?,
            )
        } else {
            usize::try_from(
                artboard.with_artboard(|artboard| artboard.default_state_machine_index()),
            )
            .ok()
        };
        let machine = machine_index
            .map(|index| {
                artboard
                    .state_machine_at(index)
                    .with_context(|| format!("failed to instantiate state machine index {index}"))
            })
            .transpose()?;
        if let Some(machine) = &machine {
            if let Some(main) = &main {
                machine.with_instance_mut(|machine| machine.bind_view_model_instance(main.clone()));
            }
            if options.side_channel {
                machine.with_instance_mut(|machine| machine.enable_semantics());
            }
        }
        let scene_name = machine.as_ref().map_or_else(
            || artboard_name.clone(),
            |machine| machine.with_instance(|machine| machine.name()),
        );
        let static_scene = machine
            .is_none()
            .then(|| StaticScene::new(artboard.downgrade()));
        Ok(Self {
            _file: file,
            artboard,
            machine,
            static_scene,
            main: main.map(ViewModelInstanceRuntime::new),
            artboard_index,
            machine_index,
            artboard_name,
            scene_name,
        })
    }

    fn advance_to(&mut self, target: f32, current: &mut f32) -> Result<bool> {
        if target + TIME_EPSILON < *current {
            bail!(
                "cannot advance scene backwards from {} to {target}",
                *current
            );
        }
        let elapsed = (target - *current).max(0.0);
        let keep_going = if let Some(machine) = &self.machine {
            machine.advance_and_apply(elapsed)
        } else {
            self.static_scene
                .as_mut()
                .expect("a scene has a native SMI or StaticScene")
                .advance_and_apply(elapsed)
        };
        *current = target;
        Ok(keep_going)
    }

    fn apply(
        &mut self,
        event: &ScriptEvent,
        factory: &mut dyn RunnerBackend,
        side_channel: bool,
    ) -> Result<()> {
        match &event.kind {
            ScriptEventKind::Input(event)
                if matches!(
                    event.kind,
                    InputKind::SemanticAction | InputKind::SemanticFocus
                ) =>
            {
                apply_semantic_input(factory, self.machine.as_ref(), event, side_channel);
            }
            ScriptEventKind::Input(event) if event.kind == InputKind::SetInput => {
                apply_set_input(self.machine.as_ref(), event)?;
                emit_input_mutation(factory, event)?;
            }
            ScriptEventKind::Input(event) if event.kind == InputKind::Resize => {
                let root = self.artboard.core_handle();
                CoreRegistry::set_double_handle(&root, 7, event.width);
                CoreRegistry::set_double_handle(&root, 8, event.height);
                emit_input_mutation(factory, event)?;
            }
            ScriptEventKind::Input(event) => {
                let hit = apply_input_event(event, self.machine.as_ref());
                factory.add_input_event(
                    event.kind.name(),
                    event.seconds,
                    event.x,
                    event.y,
                    event.pointer_id,
                );
                if side_channel {
                    factory.add_hit_result(hit_result_name(hit));
                }
            }
            ScriptEventKind::ViewModel(event) => {
                apply_view_model_event(self.main.as_ref(), event)?;
                emit_view_model_mutation(factory, event);
            }
        }
        Ok(())
    }
}

fn run() -> Result<String> {
    set_runtime_deterministic_mode(true);
    let options = Options::parse(env::args().skip(1).collect())?;
    validate_trace_options(&options)?;
    let input_events = options
        .input_script
        .as_deref()
        .map(load_input_script)
        .transpose()?
        .unwrap_or_default();
    let view_model_events = options
        .view_model_script
        .as_deref()
        .map(load_view_model_script)
        .transpose()?
        .unwrap_or_default();
    let events = merge_script_events(input_events, view_model_events);
    let bytes = std::fs::read(&options.file)
        .with_context(|| format!("failed to read {}", options.file.display()))?;
    verify_expected_file_sha256(&bytes, options.expected_file_sha256.as_deref())?;
    if options.benchmark && options.benchmark_repeat > 1 {
        return write_benchmark_repeat_report(&options, &bytes);
    }
    let mut factory: Box<dyn RunnerBackend> = if options.benchmark {
        Box::new(PersistentFactory::new(NullFactory::new()))
    } else {
        Box::new(PersistentFactory::new(RecordingFactory::new()))
    };
    let mut scene = LoadedScene::load(&bytes, factory.as_factory(), &options)?;
    if options.layout_bounds {
        return write_layout_bounds_report(&options, &bytes, &mut scene, &events, &mut *factory);
    }
    let mut renderer = factory.make_renderer();
    let (width, height) = scene
        .artboard
        .with_artboard(|artboard| (artboard.width(), artboard.height()));
    factory.source(
        &options.file.to_string_lossy(),
        &scene.artboard_name,
        &scene.scene_name,
    );
    factory.frame_size(frame_dimension(width), frame_dimension(height));
    let mut current = 0.0;
    if env::var_os("RIVE_GOLDEN_COVERAGE_STEADY_ONLY").is_some() {
        let keep = scene.advance_to(options.samples[0], &mut current)?;
        if options.side_channel {
            record_advance_side_channel(
                &mut *factory,
                scene.machine.as_ref(),
                options.samples[0],
                keep,
            )?;
        }
        scene.artboard.draw(&mut *renderer);
    }
    reset_coverage_profile_for_frame_loop_if_requested();
    reset_frame_loop_allocation_counter_if_requested();
    let started = Instant::now();
    let mut advance = Duration::ZERO;
    let mut input = Duration::ZERO;
    let mut draw = Duration::ZERO;
    let mut next_event = 0;
    for _ in 0..options.benchmark_repeat {
        for &sample in &options.samples {
            while next_event < events.len() && events[next_event].seconds <= sample + TIME_EPSILON {
                let event = &events[next_event];
                let keep = timed_result(options.benchmark, &mut advance, || {
                    scene.advance_to(event.seconds, &mut current)
                })?;
                if options.side_channel {
                    record_advance_side_channel(
                        &mut *factory,
                        scene.machine.as_ref(),
                        event.seconds,
                        keep,
                    )?;
                }
                timed_result(options.benchmark, &mut input, || {
                    scene.apply(event, &mut *factory, options.side_channel)
                })?;
                next_event += 1;
            }
            let keep = timed_result(options.benchmark, &mut advance, || {
                scene.advance_to(sample, &mut current)
            })?;
            if options.side_channel {
                record_advance_side_channel(&mut *factory, scene.machine.as_ref(), sample, keep)?;
            }
            factory.add_sample(sample);
            timed(options.benchmark, &mut draw, || {
                scene.artboard.draw(&mut *renderer)
            });
            factory.add_frame();
        }
    }
    let elapsed = started.elapsed();
    let allocations = stop_frame_loop_allocation_counter();
    if env::var_os("RIVE_GOLDEN_ALLOCATION_COUNTER").is_some() {
        eprintln!("frame_loop_allocations={allocations}");
    }
    if options.benchmark {
        // Native advanceAndApply performs resource updates itself; there is no
        // separate late renderer preparation phase to run or account twice.
        let bookkeeping = elapsed.saturating_sub(advance + input + draw);
        Ok(format!(
            "rive-golden-benchmark-v1\nelapsed_ms={}\nadvance_ms={}\ninput_ms={}\nprepare_ms=0\ndraw_ms={}\nbookkeeping_ms={}\nsegments={}\n",
            elapsed.as_secs_f64() * 1000.0,
            advance.as_secs_f64() * 1000.0,
            input.as_secs_f64() * 1000.0,
            draw.as_secs_f64() * 1000.0,
            bookkeeping.as_secs_f64() * 1000.0,
            options.samples.len() * options.benchmark_repeat,
        ))
    } else {
        Ok(factory.stream())
    }
}

fn run_benchmark_repeat_pass(
    options: &Options,
    bytes: &[u8],
    phases: bool,
) -> Result<(BenchmarkTimings, Option<usize>, bool)> {
    let mut factory = PersistentFactory::new(NullFactory::new());
    let mut scene = LoadedScene::load(bytes, &mut factory, options)?;
    let mut renderer = factory.make_renderer();
    let mut current = 0.0;
    let mut advance = Duration::ZERO;
    let mut draw = Duration::ZERO;
    reset_coverage_profile_for_frame_loop_if_requested();
    reset_frame_loop_allocation_counter_if_requested();
    let started = Instant::now();
    for _ in 0..options.benchmark_repeat {
        for &sample in &options.samples {
            timed_result(phases, &mut advance, || {
                scene.advance_to(sample, &mut current)
            })?;
            timed(phases, &mut draw, || scene.artboard.draw(&mut *renderer));
        }
    }
    let elapsed = started.elapsed();
    let allocations = stop_frame_loop_allocation_counter();
    if env::var_os("RIVE_GOLDEN_ALLOCATION_COUNTER").is_some() {
        eprintln!("frame_loop_allocations={allocations}");
    }
    Ok((
        BenchmarkTimings {
            elapsed,
            advance,
            input: Duration::ZERO,
            prepare: Duration::ZERO,
            draw,
        },
        scene.machine_index,
        scene.main.is_some(),
    ))
}

fn write_benchmark_repeat_report(options: &Options, bytes: &[u8]) -> Result<String> {
    let (total, machine_index, has_main) = run_benchmark_repeat_pass(options, bytes, false)?;
    let (phases, _, _) = run_benchmark_repeat_pass(options, bytes, true)?;
    let bookkeeping = phases
        .elapsed
        .saturating_sub(phases.advance + phases.input + phases.prepare + phases.draw);
    Ok(format!(
        "rive-golden-benchmark-v1\nelapsed_ms={}\ntotal_ms={}\nadvance_ms={}\ninput_ms={}\nprepare_ms={}\ndraw_ms={}\nbookkeeping_ms={}\nsegments={}\nscene_kind={}\ndefault_state_machine_id={}\nview_model_initialization={}\n",
        total.elapsed.as_secs_f64() * 1000.0,
        total.elapsed.as_secs_f64() * 1000.0,
        phases.advance.as_secs_f64() * 1000.0,
        phases.input.as_secs_f64() * 1000.0,
        phases.prepare.as_secs_f64() * 1000.0,
        phases.draw.as_secs_f64() * 1000.0,
        bookkeeping.as_secs_f64() * 1000.0,
        options.samples.len() * options.benchmark_repeat,
        if machine_index.is_some() {
            "state_machine"
        } else {
            "static"
        },
        machine_index.map_or_else(|| "none".to_owned(), |index| index.to_string()),
        if has_main { "schema-default" } else { "none" },
    ))
}

fn verify_expected_file_sha256(bytes: &[u8], expected: Option<&str>) -> Result<()> {
    let Some(expected) = expected else {
        return Ok(());
    };
    let actual = format!("{:x}", Sha256::digest(bytes));
    if !actual.eq_ignore_ascii_case(expected) {
        bail!("fixture sha256 mismatch: expected={expected} actual={actual}");
    }
    Ok(())
}

struct BenchmarkTimings {
    elapsed: Duration,
    advance: Duration,
    input: Duration,
    prepare: Duration,
    draw: Duration,
}

fn timed_result<T>(
    enabled: bool,
    elapsed: &mut Duration,
    action: impl FnOnce() -> Result<T>,
) -> Result<T> {
    if !enabled {
        return action();
    }
    let start = Instant::now();
    let result = action();
    *elapsed += start.elapsed();
    result
}

fn timed(enabled: bool, elapsed: &mut Duration, action: impl FnOnce()) {
    if !enabled {
        action();
        return;
    }
    let start = Instant::now();
    action();
    *elapsed += start.elapsed();
}

/// Output DTO only: all geometry is read from the already-solved native tree.
struct LayoutBoundsReport {
    local_id: usize,
    global_id: u32,
    type_name: &'static str,
    name: Option<String>,
    parent_local: Option<usize>,
    collapsed: bool,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    world_transform: [f32; 6],
}

/// Wire ordinals are labels in the established JSON contract. Structural
/// metadata is used only for those labels, never to build/solve a second graph.
fn layout_wire_ids(bytes: &[u8], artboard_index: usize) -> Result<Vec<u32>> {
    let metadata = nuxie_binary::read_runtime_metadata(bytes, None, None)?;
    let starts: Vec<_> = metadata
        .objects
        .iter()
        .enumerate()
        .filter_map(|(index, object)| {
            object
                .as_ref()
                .filter(|object| object.type_name == "Artboard")
                .map(|_| index)
        })
        .collect();
    let start = *starts
        .get(artboard_index)
        .context("selected artboard wire ordinal")?;
    let end = starts
        .get(artboard_index + 1)
        .copied()
        .unwrap_or(metadata.objects.len());
    Ok((start..end)
        .filter(|&index| {
            metadata.objects[index].as_ref().is_none_or(|object| {
                nuxie_schema::definition_by_type_key(object.type_key).is_some_and(|definition| {
                    (definition.is_a("Component") && !definition.is_a("ScrollPhysics"))
                        || definition.is_a("KeyFrameInterpolator")
                        || definition.is_a("UserInput")
                })
            })
        })
        .map(|index| index as u32)
        .collect())
}

fn native_layout_bounds(scene: &LoadedScene, ids: &[u32]) -> Result<Vec<LayoutBoundsReport>> {
    let objects = scene
        .artboard
        .with_artboard(|artboard| artboard.objects().to_vec());
    if ids.len() != objects.len() {
        bail!(
            "layout JSON wire/native slot correspondence differs: {} vs {}",
            ids.len(),
            objects.len()
        );
    }
    let mut reports = Vec::new();
    for (local_id, owner) in objects.iter().enumerate() {
        let Some(owner) = owner else {
            continue;
        };
        let snapshot = owner
            .with(|object| {
                let world = object.as_world_transform_component()?;
                let component = object.as_component()?;
                Some((
                    component.name().to_owned(),
                    component.parent_handle(),
                    *world.world_transform().values(),
                    object
                        .as_layout_component()
                        .map(|layout| layout.layout_bounds()),
                    object.layout_provider_handle(),
                ))
            })
            .flatten();
        let Some((name, parent, world_transform, layout_bounds, provider)) = snapshot else {
            continue;
        };
        let bounds = layout_bounds.or_else(|| {
            provider.and_then(|provider| {
                provider
                    .with(|provider| provider.layout_provider_bounds(0))
                    .flatten()
            })
        });
        let Some(bounds) = bounds else {
            continue;
        };
        let mut collapsed = false;
        let mut ancestor = Some(owner.clone());
        while let Some(current) = ancestor {
            let state = current
                .with(|current| {
                    current
                        .as_component()
                        .map(|component| (component.is_collapsed(), component.parent_handle()))
                })
                .flatten();
            let Some((is_collapsed, parent)) = state else {
                break;
            };
            collapsed |= is_collapsed;
            ancestor = parent;
        }
        let definition = nuxie_schema::definition_by_type_key(
            owner.core_type().expect("live layout component type"),
        )
        .context("layout component schema name")?;
        reports.push(LayoutBoundsReport {
            local_id,
            global_id: ids[local_id],
            type_name: definition.name,
            name: Some(name),
            parent_local: parent.and_then(|parent| {
                objects
                    .iter()
                    .position(|object| object.as_ref() == Some(&parent))
            }),
            collapsed,
            x: bounds.min_x,
            y: bounds.min_y,
            width: bounds.width(),
            height: bounds.height(),
            world_transform,
        });
    }
    Ok(reports)
}

fn write_layout_bounds_report(
    options: &Options,
    bytes: &[u8],
    scene: &mut LoadedScene,
    events: &[ScriptEvent],
    factory: &mut dyn RunnerBackend,
) -> Result<String> {
    let ids = layout_wire_ids(bytes, scene.artboard_index)?;
    let mut out = String::from("{\"source\":");
    push_json_string(&mut out, &options.file.to_string_lossy());
    out.push_str(",\"artboard\":");
    push_json_string(&mut out, &scene.artboard_name);
    out.push_str(",\"scene\":");
    push_json_string(&mut out, &scene.scene_name);
    out.push_str(",\"samples\":[");
    let mut current = 0.0;
    let mut next_event = 0;
    for (sample_index, &sample) in options.samples.iter().enumerate() {
        while next_event < events.len() && events[next_event].seconds <= sample + TIME_EPSILON {
            let event = &events[next_event];
            scene.advance_to(event.seconds, &mut current)?;
            scene.apply(event, factory, false)?;
            next_event += 1;
        }
        scene.advance_to(sample, &mut current)?;
        let reports = native_layout_bounds(scene, &ids)?;
        if sample_index != 0 {
            out.push(',');
        }
        write!(&mut out, "{{\"sample\":{sample},\"layoutBounds\":[")?;
        for (index, report) in reports.iter().enumerate() {
            if index != 0 {
                out.push(',');
            }
            push_layout_bounds_report(&mut out, report)?;
        }
        out.push_str("]}");
    }
    out.push_str("]}\n");
    Ok(out)
}

fn push_layout_bounds_report(out: &mut String, report: &LayoutBoundsReport) -> Result<()> {
    out.push('{');
    out.push_str("\"localId\":");
    write!(out, "{}", report.local_id)?;
    out.push_str(",\"globalId\":");
    write!(out, "{}", report.global_id)?;
    out.push_str(",\"typeName\":");
    push_json_string(out, report.type_name);
    out.push_str(",\"name\":");
    if let Some(name) = report.name.as_deref() {
        push_json_string(out, name);
    } else {
        out.push_str("null");
    }
    out.push_str(",\"parentLocal\":");
    if let Some(parent_local) = report.parent_local {
        write!(out, "{parent_local}")?;
    } else {
        out.push_str("null");
    }
    out.push_str(",\"collapsed\":");
    out.push_str(if report.collapsed { "true" } else { "false" });
    out.push_str(",\"x\":");
    write!(out, "{}", report.x)?;
    out.push_str(",\"y\":");
    write!(out, "{}", report.y)?;
    out.push_str(",\"width\":");
    write!(out, "{}", report.width)?;
    out.push_str(",\"height\":");
    write!(out, "{}", report.height)?;
    out.push_str(",\"worldTransform\":[");
    for (index, value) in report.world_transform.iter().enumerate() {
        if index != 0 {
            out.push(',');
        }
        write!(out, "{value}")?;
    }
    out.push_str("],\"worldBounds\":{\"x\":");
    write!(out, "{}", report.world_transform[4])?;
    out.push_str(",\"y\":");
    write!(out, "{}", report.world_transform[5])?;
    out.push_str(",\"width\":");
    write!(out, "{}", report.width)?;
    out.push_str(",\"height\":");
    write!(out, "{}", report.height)?;
    out.push_str("}}");
    Ok(())
}

fn push_json_string(out: &mut String, value: &str) {
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            c if c.is_control() => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputKind {
    PointerDown,
    PointerMove,
    PointerUp,
    PointerExit,
    SemanticAction,
    SemanticFocus,
    SetInput,
    Resize,
}

impl InputKind {
    fn parse(value: &str, line_number: usize) -> Result<Self> {
        match value {
            "pointerDown" => Ok(Self::PointerDown),
            "pointerMove" => Ok(Self::PointerMove),
            "pointerUp" => Ok(Self::PointerUp),
            "pointerExit" => Ok(Self::PointerExit),
            "semanticAction" => Ok(Self::SemanticAction),
            "semanticFocus" => Ok(Self::SemanticFocus),
            "setInput" => Ok(Self::SetInput),
            "resize" => Ok(Self::Resize),
            _ => bail!("unknown input event on line {line_number}: {value}"),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::PointerDown => "pointerDown",
            Self::PointerMove => "pointerMove",
            Self::PointerUp => "pointerUp",
            Self::PointerExit => "pointerExit",
            Self::SemanticAction => "semanticAction",
            Self::SemanticFocus => "semanticFocus",
            Self::SetInput => "setInput",
            Self::Resize => "resize",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScriptValueKind {
    Boolean,
    Number,
    Trigger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewModelKind {
    SetBoolean,
    SetNumber,
    SetString,
    SetEnum,
    SetColor,
    FireTrigger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SemanticInputAction {
    Tap,
    Increase,
    Decrease,
}

impl SemanticInputAction {
    fn parse(value: &str, line_number: usize) -> Result<Self> {
        match value {
            "tap" => Ok(Self::Tap),
            "increase" => Ok(Self::Increase),
            "decrease" => Ok(Self::Decrease),
            _ => bail!("unknown semantic action on line {line_number}: {value}"),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Tap => "tap",
            Self::Increase => "increase",
            Self::Decrease => "decrease",
        }
    }

    fn raw(self) -> u32 {
        match self {
            Self::Tap => 0,
            Self::Increase => 1,
            Self::Decrease => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct InputEvent {
    seconds: f32,
    kind: InputKind,
    x: f32,
    y: f32,
    pointer_id: i32,
    semantic_node_id: u32,
    semantic_action: SemanticInputAction,
    name: String,
    value_kind: ScriptValueKind,
    bool_value: bool,
    number_value: f32,
    width: f32,
    height: f32,
    dpr: f32,
    order: usize,
}

#[derive(Debug, Clone, PartialEq)]
struct ViewModelEvent {
    seconds: f32,
    kind: ViewModelKind,
    property: String,
    bool_value: bool,
    number_value: f32,
    string_value: String,
    uint_value: u32,
    order: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum ScriptEventKind {
    Input(InputEvent),
    ViewModel(ViewModelEvent),
}

#[derive(Debug, Clone, PartialEq)]
struct ScriptEvent {
    seconds: f32,
    kind: ScriptEventKind,
}

impl InputEvent {
    fn is_pointer(&self) -> bool {
        matches!(
            self.kind,
            InputKind::PointerDown
                | InputKind::PointerMove
                | InputKind::PointerUp
                | InputKind::PointerExit
        )
    }
}

fn load_input_script(path: &Path) -> Result<Vec<InputEvent>> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("unable to read input script: {}", path.display()))?;
    parse_input_script(&contents)
}

fn parse_input_script(contents: &str) -> Result<Vec<InputEvent>> {
    let mut events = Vec::new();
    for (line_index, line) in contents.lines().enumerate() {
        let line_number = line_index + 1;
        let line = line.split_once('#').map_or(line, |(value, _)| value).trim();
        if line.is_empty() {
            continue;
        }
        let tokens = line.split_whitespace().collect::<Vec<_>>();
        if tokens.len() < 2 {
            bail!("input script line {line_number} must start with: <seconds> <event>");
        }
        let seconds = parse_script_float(
            tokens[0],
            &format!("input script line {line_number} seconds"),
        )?;
        if seconds < 0.0 {
            bail!("input script line {line_number} has a negative time");
        }
        let kind = InputKind::parse(tokens[1], line_number)?;
        let context = format!("input script line {line_number}");
        if matches!(kind, InputKind::SetInput | InputKind::Resize) && !seconds.is_finite() {
            bail!("{context} seconds must be finite");
        }
        let mut event = InputEvent {
            seconds,
            kind,
            x: 0.0,
            y: 0.0,
            pointer_id: 0,
            semantic_node_id: 0,
            semantic_action: SemanticInputAction::Tap,
            name: String::new(),
            value_kind: ScriptValueKind::Boolean,
            bool_value: false,
            number_value: 0.0,
            width: 0.0,
            height: 0.0,
            dpr: 1.0,
            order: events.len(),
        };
        match kind {
            InputKind::SemanticAction => {
                if tokens.len() != 4 {
                    bail!(
                        "{context} must be: <seconds> semanticAction <nodeId> <tap|increase|decrease>"
                    );
                }
                event.semantic_node_id = tokens[2]
                    .parse::<u32>()
                    .with_context(|| format!("invalid unsigned integer for {context} nodeId"))?;
                event.semantic_action = SemanticInputAction::parse(tokens[3], line_number)?;
            }
            InputKind::SemanticFocus => {
                if tokens.len() != 3 {
                    bail!("{context} must be: <seconds> semanticFocus <nodeId>");
                }
                event.semantic_node_id = tokens[2]
                    .parse::<u32>()
                    .with_context(|| format!("invalid unsigned integer for {context} nodeId"))?;
            }
            InputKind::SetInput => {
                if tokens.len() < 4 {
                    bail!(
                        "{context} must be: <seconds> setInput <name> <bool|number|trigger> [value]"
                    );
                }
                event.name = tokens[2].to_owned();
                match tokens[3] {
                    "bool" => {
                        if tokens.len() != 5 {
                            bail!("{context} bool input requires one value");
                        }
                        event.value_kind = ScriptValueKind::Boolean;
                        event.bool_value =
                            parse_script_bool(tokens[4], &format!("{context} value"))?;
                    }
                    "number" => {
                        if tokens.len() != 5 {
                            bail!("{context} number input requires one value");
                        }
                        event.value_kind = ScriptValueKind::Number;
                        event.number_value =
                            parse_finite_script_float(tokens[4], &format!("{context} value"))?;
                    }
                    "trigger" => {
                        if tokens.len() != 4 {
                            bail!("{context} trigger input takes no value");
                        }
                        event.value_kind = ScriptValueKind::Trigger;
                    }
                    other => bail!("unknown setInput type on line {line_number}: {other}"),
                }
            }
            InputKind::Resize => {
                if tokens.len() != 5 {
                    bail!("{context} must be: <seconds> resize <width> <height> <dpr>");
                }
                event.width = parse_finite_script_float(tokens[2], &format!("{context} width"))?;
                event.height = parse_finite_script_float(tokens[3], &format!("{context} height"))?;
                event.dpr = parse_finite_script_float(tokens[4], &format!("{context} dpr"))?;
                if event.width <= 0.0 || event.height <= 0.0 || event.dpr <= 0.0 {
                    bail!("{context} resize width, height, and dpr must be greater than 0");
                }
            }
            InputKind::PointerDown
            | InputKind::PointerMove
            | InputKind::PointerUp
            | InputKind::PointerExit => {
                if tokens.len() != 4 && tokens.len() != 5 {
                    bail!("{context} must be: <seconds> <pointer-event> <x> <y> [pointerId]");
                }
                event.pointer_id = if let Some(pointer_id) = tokens.get(4) {
                    pointer_id.parse::<i32>().with_context(|| {
                        format!("invalid integer for {context} pointerId: {pointer_id}")
                    })?
                } else {
                    0
                };
                event.x = parse_script_float(tokens[2], &format!("{context} x"))?;
                event.y = parse_script_float(tokens[3], &format!("{context} y"))?;
            }
        }
        events.push(event);
    }

    events.sort_by(|left, right| left.seconds.total_cmp(&right.seconds));
    Ok(events)
}

fn load_view_model_script(path: &Path) -> Result<Vec<ViewModelEvent>> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("unable to read view-model script: {}", path.display()))?;
    parse_view_model_script(&contents)
}

fn parse_view_model_script(contents: &str) -> Result<Vec<ViewModelEvent>> {
    let mut events = Vec::new();
    for (line_index, line) in contents.lines().enumerate() {
        let line_number = line_index + 1;
        let line = line.split_once('#').map_or(line, |(value, _)| value).trim();
        if line.is_empty() {
            continue;
        }
        let tokens = line.split_whitespace().collect::<Vec<_>>();
        if tokens.len() < 2 {
            bail!("view-model script line {line_number} must start with: <seconds> <event>");
        }
        let context = format!("view-model script line {line_number}");
        let seconds = parse_finite_script_float(tokens[0], &format!("{context} seconds"))?;
        if seconds < 0.0 {
            bail!("{context} has a negative time");
        }
        let kind = match tokens[1] {
            "setVmBool" => ViewModelKind::SetBoolean,
            "setVmNumber" => ViewModelKind::SetNumber,
            "setVmString" => ViewModelKind::SetString,
            "setVmEnum" => ViewModelKind::SetEnum,
            "setVmColor" => ViewModelKind::SetColor,
            "fireVmTrigger" => ViewModelKind::FireTrigger,
            other => bail!("unknown view-model event on line {line_number}: {other}"),
        };
        if kind == ViewModelKind::FireTrigger {
            if tokens.len() != 3 {
                bail!("{context} must be: <seconds> fireVmTrigger <path>");
            }
        } else if tokens.len() != 4 {
            bail!("{context} must be: <seconds> <view-model-setter> <path> <value>");
        }
        if tokens[2].starts_with('/') || tokens[2].ends_with('/') || tokens[2].contains("//") {
            bail!("{context} has an invalid property path");
        }
        let mut event = ViewModelEvent {
            seconds,
            kind,
            property: tokens[2].to_owned(),
            bool_value: false,
            number_value: 0.0,
            string_value: String::new(),
            uint_value: 0,
            order: events.len(),
        };
        match kind {
            ViewModelKind::SetBoolean => {
                event.bool_value = parse_script_bool(tokens[3], &format!("{context} value"))?;
            }
            ViewModelKind::SetNumber => {
                event.number_value =
                    parse_finite_script_float(tokens[3], &format!("{context} value"))?;
            }
            ViewModelKind::SetString => event.string_value = tokens[3].to_owned(),
            ViewModelKind::SetEnum => {
                event.uint_value = tokens[3].parse::<u32>().with_context(|| {
                    format!(
                        "invalid unsigned integer for {context} value: {}",
                        tokens[3]
                    )
                })?;
            }
            ViewModelKind::SetColor => {
                event.uint_value = parse_script_color(tokens[3], &format!("{context} value"))?;
            }
            ViewModelKind::FireTrigger => {}
        }
        events.push(event);
    }
    events.sort_by(|left, right| left.seconds.total_cmp(&right.seconds));
    Ok(events)
}

fn merge_script_events(
    input_events: Vec<InputEvent>,
    view_model_events: Vec<ViewModelEvent>,
) -> Vec<ScriptEvent> {
    let mut input_events = input_events.into_iter().peekable();
    let mut view_model_events = view_model_events.into_iter().peekable();
    let mut events = Vec::new();
    while input_events.peek().is_some() || view_model_events.peek().is_some() {
        let use_input = match (input_events.peek(), view_model_events.peek()) {
            (Some(input), Some(view_model)) => input.seconds <= view_model.seconds,
            (Some(_), None) => true,
            (None, Some(_)) => false,
            (None, None) => break,
        };
        if use_input {
            let event = input_events.next().expect("peeked input event");
            events.push(ScriptEvent {
                seconds: event.seconds,
                kind: ScriptEventKind::Input(event),
            });
        } else {
            let event = view_model_events.next().expect("peeked view-model event");
            events.push(ScriptEvent {
                seconds: event.seconds,
                kind: ScriptEventKind::ViewModel(event),
            });
        }
    }
    events
}

fn parse_script_float(value: &str, context: &str) -> Result<f32> {
    value
        .parse::<f32>()
        .with_context(|| format!("invalid float for {context}: {value}"))
}

fn parse_finite_script_float(value: &str, context: &str) -> Result<f32> {
    let value = parse_script_float(value, context)?;
    if !value.is_finite() {
        bail!("{context} must be finite");
    }
    Ok(value)
}

fn parse_script_color(value: &str, context: &str) -> Result<u32> {
    let digits = value
        .strip_prefix("0x")
        .filter(|digits| digits.len() == 8)
        .with_context(|| format!("{context} must be 0x followed by eight hex digits"))?;
    u32::from_str_radix(digits, 16).with_context(|| format!("invalid color for {context}: {value}"))
}

fn parse_script_bool(value: &str, context: &str) -> Result<bool> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => bail!("invalid boolean for {context}: {value}"),
    }
}

fn hit_result_name(result: HitResult) -> &'static str {
    match result {
        HitResult::None => "none",
        HitResult::Hit => "hit",
        HitResult::HitOpaque => "hitOpaque",
    }
}

fn side_channel_event(report: &EventReport) -> SideChannelEvent {
    use nuxie_runtime::source::{
        custom_property_boolean::CustomPropertyBoolean, custom_property_color::CustomPropertyColor,
        custom_property_enum::CustomPropertyEnum, custom_property_number::CustomPropertyNumber,
        custom_property_string::CustomPropertyString,
        custom_property_trigger::CustomPropertyTrigger, open_url_event::OpenUrlEvent,
    };
    let Some(event) = &report.event else {
        return SideChannelEvent {
            core_type: 0,
            name: String::new(),
            delay: 0.0,
            url_target: None,
            properties: Vec::new(),
        };
    };
    let (name, url_target, children) = event
        .with(|event| {
            let component = event.as_component().expect("reported Event is a Component");
            let url = event.as_any().downcast_ref::<OpenUrlEvent>().map(|url| {
                let target = match url.target_value() {
                    0 => "_blank",
                    1 => "_parent",
                    2 => "_self",
                    3 => "_top",
                    _ => "",
                };
                (url.url().to_owned(), target.to_owned())
            });
            (
                component.name().to_owned(),
                url,
                event
                    .as_container_component()
                    .expect("reported Event is a container")
                    .children()
                    .to_vec(),
            )
        })
        .expect("reported Event remains live");
    let properties = children
        .iter()
        .filter_map(|child| {
            child
                .with(|child| {
                    let name = child.as_component()?.name().to_owned();
                    let any = child.as_any();
                    let value = if let Some(value) = any.downcast_ref::<CustomPropertyNumber>() {
                        SideChannelEventPropertyValue::Number(value.property_value())
                    } else if let Some(value) = any.downcast_ref::<CustomPropertyBoolean>() {
                        SideChannelEventPropertyValue::Bool(value.property_value())
                    } else if let Some(value) = any.downcast_ref::<CustomPropertyString>() {
                        SideChannelEventPropertyValue::String(value.property_value().to_owned())
                    } else if let Some(value) = any.downcast_ref::<CustomPropertyColor>() {
                        SideChannelEventPropertyValue::Color(value.property_value() as u32)
                    } else if let Some(value) = any.downcast_ref::<CustomPropertyEnum>() {
                        SideChannelEventPropertyValue::Uint(value.property_value() as u64)
                    } else if let Some(value) = any.downcast_ref::<CustomPropertyTrigger>() {
                        SideChannelEventPropertyValue::Uint(value.property_value() as u64)
                    } else {
                        return None;
                    };
                    Some(SideChannelEventProperty { name, value })
                })
                .flatten()
        })
        .collect();
    SideChannelEvent {
        core_type: event.core_type().expect("reported Event type") as u32,
        name,
        delay: report.seconds_delay,
        url_target,
        properties,
    }
}

fn record_advance_side_channel(
    factory: &mut dyn RunnerBackend,
    machine: Option<&RuntimeStateMachineInstanceHandle>,
    target_seconds: f32,
    keep_going: bool,
) -> Result<()> {
    let Some(machine) = machine else {
        factory.add_advance(target_seconds, !keep_going);
        return Ok(());
    };
    let changed = machine.with_instance(|machine| machine.state_changed_count());
    factory.add_advance_with_states(target_seconds, !keep_going, changed);
    let count = machine.with_instance(|machine| machine.reported_event_count());
    for index in 0..count {
        let report = machine.with_instance(|machine| machine.reported_event_at(index));
        factory.add_side_channel_event(&side_channel_event(&report));
    }
    if let Some(manager) = machine.with_instance(|machine| machine.semantic_manager()) {
        let diff = manager.with_semantic_manager_mut(|manager| manager.drain_diff());
        factory.add_semantics_diff(&side_channel_semantics_diff(&diff));
    }
    Ok(())
}

fn apply_semantic_input(
    factory: &mut dyn RunnerBackend,
    machine: Option<&RuntimeStateMachineInstanceHandle>,
    event: &InputEvent,
    record: bool,
) {
    let manager =
        machine.and_then(|machine| machine.with_instance(|machine| machine.semantic_manager()));
    match event.kind {
        InputKind::SemanticAction => {
            // This is the C++ runner's dispatch observation, not a synthetic
            // success flag: the actual node must own actual SemanticData.
            let dispatched = manager
                .as_ref()
                .and_then(|manager| {
                    manager
                        .with_semantic_manager(|manager| manager.node_by_id(event.semantic_node_id))
                })
                .is_some_and(|node| node.borrow().semantic_data.is_some());
            if dispatched {
                machine
                    .expect("a semantic manager belongs to the selected machine")
                    .fire_semantic_action(
                        event.semantic_node_id,
                        event.semantic_action.raw() as u8,
                    );
            }
            if record {
                factory.add_semantic_action(
                    event.seconds,
                    event.semantic_node_id,
                    event.semantic_action.name(),
                    dispatched,
                );
            }
        }
        InputKind::SemanticFocus => {
            let focused =
                manager.is_some_and(|manager| manager.request_focus(event.semantic_node_id));
            if record {
                factory.add_semantic_focus(event.seconds, event.semantic_node_id, focused);
            }
        }
        _ => {}
    }
}

fn apply_set_input(
    machine: Option<&RuntimeStateMachineInstanceHandle>,
    event: &InputEvent,
) -> Result<()> {
    let machine = machine.context("setInput requires a state-machine scene")?;
    match event.value_kind {
        ScriptValueKind::Boolean => {
            if !machine.with_instance(|machine| machine.get_bool(&event.name).is_some()) {
                bail!("state-machine input '{}' was not found as bool", event.name);
            }
            machine.set_bool(&event.name, event.bool_value);
        }
        ScriptValueKind::Number => {
            if !machine.with_instance(|machine| machine.get_number(&event.name).is_some()) {
                bail!(
                    "state-machine input '{}' was not found as number",
                    event.name
                );
            }
            machine.set_number(&event.name, event.number_value);
        }
        ScriptValueKind::Trigger => {
            machine.with_instance_mut(|machine| {
                machine
                    .get_trigger_mut(&event.name)
                    .with_context(|| {
                        format!(
                            "state-machine input '{}' was not found as trigger",
                            event.name
                        )
                    })?
                    .fire();
                Ok::<_, anyhow::Error>(())
            })?;
        }
    }
    Ok(())
}

fn apply_view_model_event(
    main: Option<&ViewModelInstanceRuntime>,
    event: &ViewModelEvent,
) -> Result<()> {
    let main = main.context("view-model script requires a bound main view model")?;
    let missing = |kind| {
        format!(
            "view-model property '{}' was not found as {kind}",
            event.property
        )
    };
    match event.kind {
        ViewModelKind::SetBoolean => main
            .property_boolean(&event.property)
            .with_context(|| missing("bool"))?
            .set_value(event.bool_value),
        ViewModelKind::SetNumber => main
            .property_number(&event.property)
            .with_context(|| missing("number"))?
            .set_value(event.number_value),
        ViewModelKind::SetString => main
            .property_string(&event.property)
            .with_context(|| missing("string"))?
            .set_value(event.string_value.clone()),
        ViewModelKind::SetEnum => {
            if !main
                .property_enum(&event.property)
                .with_context(|| missing("enum"))?
                .set_value_index(event.uint_value)
            {
                bail!(
                    "view-model property '{}' rejected enum index {}",
                    event.property,
                    event.uint_value
                );
            }
        }
        ViewModelKind::SetColor => main
            .property_color(&event.property)
            .with_context(|| missing("color"))?
            .set_value(event.uint_value as i32),
        ViewModelKind::FireTrigger => {
            let value = main
                .property_trigger(&event.property)
                .with_context(|| missing("trigger"))?;
            value.trigger();
        }
    }
    Ok(())
}

fn apply_input_event(
    event: &InputEvent,
    machine: Option<&RuntimeStateMachineInstanceHandle>,
) -> HitResult {
    let Some(machine) = machine else {
        return HitResult::None;
    };
    machine.with_instance_mut(|machine| {
        let position = Vec2D::new(event.x, event.y);
        match event.kind {
            InputKind::PointerDown => machine.pointer_down(position, event.pointer_id),
            InputKind::PointerMove => {
                machine.pointer_move(position, event.seconds, event.pointer_id)
            }
            InputKind::PointerUp => machine.pointer_up(position, event.pointer_id),
            InputKind::PointerExit => machine.pointer_exit(position, event.pointer_id),
            _ => HitResult::None,
        }
    })
}

fn side_channel_semantics_node(node: &SemanticsDiffNode) -> SideChannelSemanticsNode {
    SideChannelSemanticsNode {
        id: node.id,
        role: node.role,
        label: node.label.clone(),
        value: node.value.clone(),
        hint: node.hint.clone(),
        state_flags: node.state_flags,
        trait_flags: node.trait_flags,
        heading_level: node.heading_level,
        min_x: node.min_x,
        min_y: node.min_y,
        max_x: node.max_x,
        max_y: node.max_y,
        parent_id: node.parent_id,
        sibling_index: node.sibling_index,
    }
}

fn side_channel_semantics_diff(diff: &SemanticsDiff) -> SideChannelSemanticsDiff {
    SideChannelSemanticsDiff {
        frame_number: diff.frame_number,
        tree_version: diff.tree_version,
        root_id: diff.root_id,
        removed: diff.removed.clone(),
        added: diff.added.iter().map(side_channel_semantics_node).collect(),
        moved: diff.moved.iter().map(side_channel_semantics_node).collect(),
        children_updated: diff
            .children_updated
            .iter()
            .map(|update| SideChannelSemanticsChildrenUpdate {
                parent_id: update.parent_id,
                child_ids: update.child_ids.clone(),
            })
            .collect(),
        updated_semantic: diff
            .updated_semantic
            .iter()
            .map(side_channel_semantics_node)
            .collect(),
        updated_geometry: diff
            .updated_geometry
            .iter()
            .map(|update| SideChannelSemanticsBoundsUpdate {
                id: update.id,
                min_x: update.min_x,
                min_y: update.min_y,
                max_x: update.max_x,
                max_y: update.max_y,
            })
            .collect(),
    }
}

fn resize_pixel_dimension(logical: f32, dpr: f32) -> Result<u32> {
    let pixels = (f64::from(logical) * f64::from(dpr)).ceil();
    if !pixels.is_finite() || pixels < 1.0 || pixels > f64::from(u32::MAX) {
        bail!("resize physical extent is outside the u32 range");
    }
    Ok(pixels as u32)
}

fn emit_input_mutation(factory: &mut dyn RunnerBackend, event: &InputEvent) -> Result<()> {
    match event.kind {
        InputKind::SetInput => match event.value_kind {
            ScriptValueKind::Boolean => {
                factory.add_set_input_boolean(event.seconds, &event.name, event.bool_value);
            }
            ScriptValueKind::Number => {
                factory.add_set_input_number(event.seconds, &event.name, event.number_value);
            }
            ScriptValueKind::Trigger => {
                factory.add_set_input_trigger(event.seconds, &event.name);
            }
        },
        InputKind::Resize => factory.add_resize(
            event.seconds,
            event.width,
            event.height,
            event.dpr,
            resize_pixel_dimension(event.width, event.dpr)?,
            resize_pixel_dimension(event.height, event.dpr)?,
        ),
        _ => {}
    }
    Ok(())
}

fn emit_view_model_mutation(factory: &mut dyn RunnerBackend, event: &ViewModelEvent) {
    match event.kind {
        ViewModelKind::SetBoolean => {
            factory.add_view_model_boolean(event.seconds, &event.property, event.bool_value);
        }
        ViewModelKind::SetNumber => {
            factory.add_view_model_number(event.seconds, &event.property, event.number_value);
        }
        ViewModelKind::SetString => {
            factory.add_view_model_string(event.seconds, &event.property, &event.string_value);
        }
        ViewModelKind::SetEnum => {
            factory.add_view_model_enum(event.seconds, &event.property, event.uint_value);
        }
        ViewModelKind::SetColor => {
            factory.add_view_model_color(event.seconds, &event.property, event.uint_value);
        }
        ViewModelKind::FireTrigger => {
            factory.add_view_model_trigger(event.seconds, &event.property);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_script_parser_matches_golden_runner_shape() {
        let events = parse_input_script(
            r#"
            # comments and blank lines are ignored
            0.2 pointerUp 5 6 7
            0.1 pointerDown 1 2
            0.1 pointerMove 3 4 # same-time events keep file order
            "#,
        )
        .expect("script parses");

        assert_eq!(
            events
                .iter()
                .map(|event| (
                    event.seconds,
                    event.kind,
                    event.x,
                    event.y,
                    event.pointer_id
                ))
                .collect::<Vec<_>>(),
            vec![
                (0.1, InputKind::PointerDown, 1.0, 2.0, 0),
                (0.1, InputKind::PointerMove, 3.0, 4.0, 0),
                (0.2, InputKind::PointerUp, 5.0, 6.0, 7),
            ]
        );
    }

    #[test]
    fn input_script_parser_rejects_bad_shape() {
        assert!(parse_input_script("0.1 pointerDown 1\n").is_err());
        assert!(parse_input_script("-0.1 pointerDown 1 2\n").is_err());
        assert!(parse_input_script("0.1 pointerCancel 1 2\n").is_err());
    }

    #[test]
    fn mutation_input_script_verbs_preserve_types_values_and_dimensions() {
        let events = parse_input_script(
            "0 setInput enabled bool true\n0 setInput amount number 12.5\n\
             0 setInput launch trigger\n0 resize 320 180 2\n",
        )
        .expect("mutation input verbs parse");

        assert_eq!(events[0].kind, InputKind::SetInput);
        assert_eq!(events[0].name, "enabled");
        assert_eq!(events[0].value_kind, ScriptValueKind::Boolean);
        assert!(events[0].bool_value);
        assert_eq!(events[1].value_kind, ScriptValueKind::Number);
        assert_eq!(events[1].number_value, 12.5);
        assert_eq!(events[2].value_kind, ScriptValueKind::Trigger);
        assert_eq!(events[3].kind, InputKind::Resize);
        assert_eq!(
            (events[3].width, events[3].height, events[3].dpr),
            (320.0, 180.0, 2.0)
        );

        assert!(parse_input_script("0 setInput enabled bool 1\n").is_err());
        assert!(parse_input_script("0 setInput amount number nan\n").is_err());
        assert!(parse_input_script("0 setInput launch trigger now\n").is_err());
        assert!(parse_input_script("0 resize 320 0 2\n").is_err());
    }

    #[test]
    fn view_model_script_parser_and_merge_preserve_cross_stream_order() {
        let input = parse_input_script("0 setInput enabled bool true\n1 resize 10 20 2\n")
            .expect("input script parses");
        let view_model = parse_view_model_script(
            "0 setVmBool visible false\n0 setVmNumber progress 0.5\n\
             0 setVmString child/label ready\n0 setVmEnum status 2\n\
             0 setVmColor tint 0xff123456\n0 fireVmTrigger go\n",
        )
        .expect("view-model script parses");

        assert_eq!(view_model[0].kind, ViewModelKind::SetBoolean);
        assert!(!view_model[0].bool_value);
        assert_eq!(view_model[1].kind, ViewModelKind::SetNumber);
        assert_eq!(view_model[1].number_value, 0.5);
        assert_eq!(view_model[2].kind, ViewModelKind::SetString);
        assert_eq!(view_model[2].property, "child/label");
        assert_eq!(view_model[2].string_value, "ready");
        assert_eq!(view_model[3].kind, ViewModelKind::SetEnum);
        assert_eq!(view_model[3].uint_value, 2);
        assert_eq!(view_model[4].kind, ViewModelKind::SetColor);
        assert_eq!(view_model[4].uint_value, 0xff12_3456);
        assert_eq!(view_model[5].kind, ViewModelKind::FireTrigger);

        let merged = merge_script_events(input, view_model);
        assert!(matches!(merged[0].kind, ScriptEventKind::Input(_)));
        assert!(matches!(merged[1].kind, ScriptEventKind::ViewModel(_)));
        assert!(matches!(merged[2].kind, ScriptEventKind::ViewModel(_)));
        assert!(matches!(merged[3].kind, ScriptEventKind::ViewModel(_)));
        assert!(matches!(merged[6].kind, ScriptEventKind::ViewModel(_)));
        assert!(matches!(merged[7].kind, ScriptEventKind::Input(_)));

        assert!(parse_view_model_script("0 setVmBool visible yes\n").is_err());
        assert!(parse_view_model_script("0 setVmNumber progress inf\n").is_err());
        assert!(parse_view_model_script("0 setVmColor tint ff123456\n").is_err());
        assert!(parse_view_model_script("0 setVmString child//label x\n").is_err());
        assert!(parse_view_model_script("0 fireVmTrigger go now\n").is_err());

        let ordered =
            parse_view_model_script("0.0000005 setVmBool later true\n0 setVmBool earlier true\n")
                .expect("nearby distinct timestamps parse");
        assert_eq!(ordered[0].property, "earlier");
        assert_eq!(ordered[1].property, "later");
    }

    #[test]
    fn semantic_input_script_verbs_preserve_ids_actions_and_order() {
        let events = parse_input_script(
            "0.2 semanticFocus 16\n0.1 semanticAction 4 tap\n0.1 semanticAction 7 decrease\n",
        )
        .expect("semantic input verbs parse");

        assert_eq!(
            events
                .iter()
                .map(|event| (
                    event.seconds,
                    event.kind,
                    event.semantic_node_id,
                    event.semantic_action
                ))
                .collect::<Vec<_>>(),
            vec![
                (0.1, InputKind::SemanticAction, 4, SemanticInputAction::Tap),
                (
                    0.1,
                    InputKind::SemanticAction,
                    7,
                    SemanticInputAction::Decrease,
                ),
                (0.2, InputKind::SemanticFocus, 16, SemanticInputAction::Tap),
            ]
        );
        assert!(parse_input_script("0 semanticAction 1 unknown\n").is_err());
        assert!(parse_input_script("0 semanticFocus\n").is_err());
        assert!(events.iter().all(|event| !event.is_pointer()));
    }

    #[test]
    fn sample_parser_matches_golden_runner_tolerance() {
        assert_eq!(
            parse_samples("0.1,0.0999995,0.2").expect("within epsilon is sorted"),
            vec![0.1, 0.0999995, 0.2]
        );
        assert!(parse_samples("-0.1").is_err());
        assert!(parse_samples("0.1,0.099").is_err());
    }

    #[test]
    fn side_channel_is_stream_mode_only() {
        let base = |extra: &[&str]| {
            let mut args = vec!["--file".to_owned(), "file.riv".to_owned()];
            args.extend(extra.iter().map(|value| (*value).to_owned()));
            Options::parse(args)
        };

        let options = base(&["--side-channel"]).expect("side-channel parses");
        assert!(options.side_channel);
        assert!(!base(&[]).expect("plain parses").side_channel);
        assert!(base(&["--side-channel", "--benchmark"]).is_err());
        assert!(base(&["--side-channel", "--layout-bounds"]).is_err());
        assert!(base(&["--semantic-default-view-model"]).is_err());
        assert!(base(&["--semantic-side-channel-only"]).is_err());
        let semantic_defaults = base(&["--side-channel", "--semantic-default-view-model"])
            .expect("semantic fixture defaults parse with the side channel");
        assert!(semantic_defaults.semantic_default_view_model);
        let semantic_projection = base(&["--side-channel", "--semantic-side-channel-only"])
            .expect("semantic projection parses with the side channel");
        assert!(semantic_projection.semantic_side_channel_only);
    }

    #[test]
    fn benchmark_repeat_is_benchmark_only_single_sample() {
        let options = Options::parse(vec![
            "--file".to_owned(),
            "fixture.riv".to_owned(),
            "--benchmark".to_owned(),
            "--benchmark-repeat".to_owned(),
            "10".to_owned(),
        ])
        .expect("parse benchmark repeat");
        assert_eq!(options.benchmark_repeat, 10);

        let error = Options::parse(vec![
            "--file".to_owned(),
            "fixture.riv".to_owned(),
            "--benchmark-repeat".to_owned(),
            "10".to_owned(),
        ])
        .unwrap_err();
        assert!(error.to_string().contains("requires --benchmark"));

        let error = Options::parse(vec![
            "--file".to_owned(),
            "fixture.riv".to_owned(),
            "--benchmark".to_owned(),
            "--samples".to_owned(),
            "0,1".to_owned(),
            "--benchmark-repeat".to_owned(),
            "10".to_owned(),
        ])
        .unwrap_err();
        assert!(error.to_string().contains("exactly one sample"));
    }

    #[test]
    fn expected_file_sha256_parses_for_sealed_fixture_runs() {
        let expected = "a".repeat(64);
        let options = Options::parse(vec![
            "--file".to_owned(),
            "fixture.riv".to_owned(),
            "--expected-file-sha256".to_owned(),
            expected.clone(),
        ])
        .expect("parse sealed fixture identity");

        assert_eq!(
            options.expected_file_sha256.as_deref(),
            Some(expected.as_str())
        );
        assert!(
            Options::parse(vec![
                "--file".to_owned(),
                "fixture.riv".to_owned(),
                "--expected-file-sha256".to_owned(),
                "not-a-sha".to_owned(),
            ])
            .is_err()
        );

        verify_expected_file_sha256(b"sealed", None).expect("unsealed runs remain valid");
        let sealed_sha = format!("{:x}", Sha256::digest(b"sealed"));
        verify_expected_file_sha256(b"sealed", Some(&sealed_sha))
            .expect("matching sealed bytes are accepted");
        let error = verify_expected_file_sha256(b"swapped", Some(&sealed_sha)).unwrap_err();
        assert!(error.to_string().contains("fixture sha256 mismatch"));
    }

    #[test]
    fn active_script_import_is_explicit() {
        let plain = Options::parse(vec!["--file".to_owned(), "fixture.riv".to_owned()])
            .expect("parse plain runner options");
        assert!(!plain.execute_scripts);

        let active = Options::parse(vec![
            "--file".to_owned(),
            "fixture.riv".to_owned(),
            "--execute-scripts".to_owned(),
        ])
        .expect("parse active script import");
        assert!(active.execute_scripts);
    }

    #[cfg(feature = "scripting")]
    struct CountingVm {
        vm: nuxie_scripting::vm::ScriptVm,
        tails: std::rc::Rc<std::cell::Cell<usize>>,
    }

    #[cfg(feature = "scripting")]
    impl nuxie_runtime::ScriptingVm for CountingVm {
        fn install_native_file_assets(
            &self,
            file: nuxie_runtime::source::file::RuntimeFileWeakHandle,
        ) -> std::result::Result<(), nuxie_runtime::ScriptError> {
            nuxie_runtime::ScriptingVm::install_native_file_assets(&self.vm, file)
        }
        fn initialize_data_global(
            &self,
            models: std::collections::BTreeMap<String, nuxie_runtime::ScriptViewModel>,
        ) -> std::result::Result<(), nuxie_runtime::ScriptError> {
            nuxie_runtime::ScriptingVm::initialize_data_global(&self.vm, models)
        }
        fn install_render_factory(
            &self,
            factory: &mut dyn RenderFactory,
        ) -> std::result::Result<(), nuxie_runtime::ScriptError> {
            nuxie_runtime::ScriptingVm::install_render_factory(&self.vm, factory)
        }
        fn install_rive_globals(&self) -> std::result::Result<(), nuxie_runtime::ScriptError> {
            nuxie_runtime::ScriptingVm::install_rive_globals(&self.vm)
        }
        fn register_module(
            &self,
            name: &str,
            payload: &[u8],
        ) -> std::result::Result<(), nuxie_runtime::ScriptError> {
            nuxie_runtime::ScriptingVm::register_module(&self.vm, name, payload)
        }
        fn register_script_assets(
            &self,
            assets: &[nuxie_runtime::ScriptAssetRegistration<'_>],
        ) -> Vec<nuxie_runtime::ScriptAssetRegistrationResult> {
            nuxie_runtime::ScriptingVm::register_script_assets(&self.vm, assets)
        }
        fn instantiate_program(
            &self,
            program: &nuxie_runtime::RuntimeScriptProgram,
            present: bool,
            source: Option<nuxie_runtime::ScriptedContextSource>,
            view_model: Option<nuxie_runtime::ScriptViewModel>,
            parents: Vec<Option<nuxie_runtime::ScriptViewModel>>,
            host: &mut dyn nuxie_runtime::ScriptHost,
        ) -> std::result::Result<Box<dyn nuxie_runtime::ScriptInstance>, nuxie_runtime::ScriptError>
        {
            nuxie_runtime::ScriptingVm::instantiate_program(
                &self.vm, program, present, source, view_model, parents, host,
            )
        }
        fn instantiate_script(
            &self,
            name: &str,
            payload: &[u8],
            host: &mut dyn nuxie_runtime::ScriptHost,
        ) -> std::result::Result<Box<dyn nuxie_runtime::ScriptInstance>, nuxie_runtime::ScriptError>
        {
            nuxie_runtime::ScriptingVm::instantiate_script(&self.vm, name, payload, host)
        }
        fn advance_detached_view_models(&self) -> bool {
            self.tails.set(self.tails.get() + 1);
            nuxie_runtime::ScriptingVm::advance_detached_view_models(&self.vm)
        }
    }

    #[cfg(feature = "scripting")]
    #[test]
    fn file_vm_tail_requires_a_root_state_machine_and_runs_once_per_host_frame() {
        use std::{cell::Cell, rc::Rc};
        let tails = Rc::new(Cell::new(0));
        let vm = RuntimeScriptingVmHandle::new(Box::new(CountingVm {
            vm: nuxie_scripting::vm::ScriptVm::new(),
            tails: tails.clone(),
        }));
        let mut factory = PersistentFactory::new(NullFactory::new());
        let retained = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
        let file = File::import(
            include_bytes!("../../../fixtures/graph/dependency_test.riv"),
            retained.clone(),
            None,
            None,
            Some(vm.clone()),
        )
        .unwrap();
        let options = Options::parse(vec!["--file".into(), "fixture.riv".into()]).unwrap();
        let mut scene = LoadedScene::from_file(file, &options).unwrap();
        assert!(scene.machine.is_none());
        tails.set(0);
        let mut current = 0.0;
        scene.advance_to(0.0, &mut current).unwrap();
        assert_eq!(tails.get(), 0);
        scene.advance_to(0.0, &mut current).unwrap();
        assert_eq!(
            tails.get(),
            0,
            "StaticScene has no root StateMachineInstance frame tail"
        );

        let file = File::import(
            include_bytes!("../../../fixtures/animation/smi_test.riv"),
            retained,
            None,
            None,
            Some(vm),
        )
        .unwrap();
        let mut scene = LoadedScene::from_file(file, &options).unwrap();
        // This fixture may not author a default index; the test deliberately
        // selects its first real machine, as the prior test did.
        scene.machine = scene.artboard.state_machine_at(0);
        scene.static_scene = None;
        assert!(scene.machine.is_some());
        tails.set(0);
        let mut current = 0.0;
        scene.advance_to(0.0, &mut current).unwrap();
        assert_eq!(tails.get(), 1);
        scene.advance_to(0.0, &mut current).unwrap();
        assert_eq!(
            tails.get(),
            2,
            "each root SMI host call owns one File VM tail"
        );
    }

    #[cfg(feature = "scripting")]
    fn fixture_record(
        type_name: &str,
        properties: Vec<(&str, nuxie_binary::FixtureValue)>,
    ) -> nuxie_binary::FixtureRecord {
        let definition = nuxie_schema::definition_by_name(type_name).unwrap();
        nuxie_binary::FixtureRecord {
            type_key: definition.type_key.int,
            properties: properties
                .into_iter()
                .map(|(name, value)| {
                    let key = std::iter::once(definition)
                        .chain(
                            definition
                                .ancestors
                                .iter()
                                .filter_map(|ancestor| nuxie_schema::definition_by_name(ancestor)),
                        )
                        .flat_map(|owner| owner.properties)
                        .find(|property| property.name == name)
                        .unwrap()
                        .key
                        .int;
                    nuxie_binary::FixtureProperty { key, value }
                })
                .collect(),
        }
    }

    #[cfg(feature = "scripting")]
    fn import_authored_fixture(records: Vec<nuxie_binary::FixtureRecord>) -> RuntimeFileHandle {
        // Test authoring only: materialize fixture bytes once, then use the
        // same native importer as production. No descriptor executes.
        let descriptor = nuxie_binary::RuntimeFile::from_fixture_records(records).unwrap();
        let bytes = nuxie_binary::encode_runtime_file(&descriptor).unwrap();
        let mut factory = PersistentFactory::new(NullFactory::new());
        import_file(&bytes, &mut factory, true).unwrap()
    }

    #[cfg(feature = "scripting")]
    #[test]
    fn native_scene_binds_the_same_main_view_model_after_default_machine_construction() {
        use nuxie_binary::FixtureValue as V;
        let file = import_authored_fixture(vec![
            fixture_record("Backboard", vec![]),
            fixture_record("ViewModel", vec![("name", V::String("Child".into()))]),
            fixture_record(
                "ViewModelInstance",
                vec![
                    ("name", V::String("Child defaults".into())),
                    ("viewModelId", V::Uint(0)),
                ],
            ),
            fixture_record(
                "Artboard",
                vec![
                    ("viewModelId", V::Uint(0)),
                    ("defaultStateMachineId", V::Uint(0)),
                ],
            ),
            fixture_record("StateMachine", vec![]),
        ]);
        let options = Options::parse(vec!["--file".into(), "fixture.riv".into()]).unwrap();
        let scene = LoadedScene::from_file(file, &options).unwrap();
        assert!(
            scene.machine.is_some(),
            "authored default machine exists before use"
        );
        let context = scene.artboard.data_context().expect("Artboard is bound");
        let machine_context = scene
            .machine
            .as_ref()
            .unwrap()
            .with_instance(|machine| machine.data_context())
            .unwrap();
        assert!(
            context.ptr_eq(&machine_context),
            "Artboard and default scene share the actual DataContext"
        );
        assert!(
            context
                .with_context(|context| context.main_view_model_instance())
                .is_some()
        );
    }

    #[cfg(feature = "scripting")]
    #[test]
    fn native_data_context_preserves_parent_only_value_resolution() {
        use nuxie_binary::FixtureValue as V;
        use nuxie_runtime::source::data_bind::data_context::{
            DataContext, RuntimeDataContextHandle,
        };
        use nuxie_runtime::source::viewmodel::viewmodel::ViewModel;
        let file = import_authored_fixture(vec![
            fixture_record("Backboard", vec![]),
            fixture_record("ViewModel", vec![("name", V::String("Shared".into()))]),
            fixture_record(
                "ViewModelPropertyNumber",
                vec![("name", V::String("local".into()))],
            ),
            fixture_record(
                "ViewModelPropertyNumber",
                vec![("name", V::String("parentOnly".into()))],
            ),
            fixture_record("ViewModelInstance", vec![("viewModelId", V::Uint(0))]),
            fixture_record(
                "ViewModelInstanceNumber",
                vec![
                    ("parentId", V::Uint(0)),
                    ("viewModelPropertyId", V::Uint(0)),
                    ("propertyValue", V::Double(1.0)),
                ],
            ),
            fixture_record("ViewModelInstance", vec![("viewModelId", V::Uint(0))]),
            fixture_record(
                "ViewModelInstanceNumber",
                vec![
                    ("parentId", V::Uint(1)),
                    ("viewModelPropertyId", V::Uint(1)),
                    ("propertyValue", V::Double(35.0)),
                ],
            ),
            fixture_record(
                "Artboard",
                vec![
                    ("viewModelId", V::Uint(0)),
                    ("defaultStateMachineId", V::Uint(0)),
                ],
            ),
            fixture_record("StateMachine", vec![]),
        ]);
        let model = file
            .with_file(|file| file.view_model_named("Shared"))
            .unwrap();
        // Use the actual authored partial instances. Completing a model would
        // intentionally create missing default values and is a different case.
        let local = model
            .with_downcast::<ViewModel, _>(|model| model.instance_at(0))
            .flatten()
            .unwrap();
        let parent = model
            .with_downcast::<ViewModel, _>(|model| model.instance_at(1))
            .flatten()
            .unwrap();
        let parent = RuntimeDataContextHandle::new(DataContext::new(Some(parent)));
        let mut context = DataContext::new(Some(local));
        context.set_parent(Some(parent));
        assert!(context.parent().is_some());
        let value = context
            .get_view_model_property(&[0, 1])
            .expect("partial local falls through to parent");
        assert_eq!(
            value.with(|value| value
                .as_view_model_instance_number()
                .unwrap()
                .base
                .property_value()),
            Some(35.0)
        );
    }
}
#[derive(Debug)]
struct Options {
    file: PathBuf,
    expected_file_sha256: Option<String>,
    artboard: Option<String>,
    state_machine: Option<String>,
    input_script: Option<PathBuf>,
    view_model_script: Option<PathBuf>,
    samples: Vec<f32>,
    layout_bounds: bool,
    benchmark: bool,
    benchmark_repeat: usize,
    execute_scripts: bool,
    side_channel: bool,
    semantic_default_view_model: bool,
    semantic_side_channel_only: bool,
}

impl Options {
    fn parse(args: Vec<String>) -> Result<Self> {
        let mut file = None::<PathBuf>;
        let mut expected_file_sha256 = None;
        let mut artboard = None;
        let mut state_machine = None;
        let mut input_script = None;
        let mut view_model_script = None;
        let mut samples = vec![0.0];
        let mut layout_bounds = false;
        let mut benchmark = false;
        let mut benchmark_repeat = 1usize;
        let mut execute_scripts = false;
        let mut side_channel = false;
        let mut semantic_default_view_model = false;
        let mut semantic_side_channel_only = false;

        let mut index = 0;
        while index < args.len() {
            let arg = &args[index];
            let mut value = |option: &str| -> Result<String> {
                index += 1;
                args.get(index)
                    .cloned()
                    .with_context(|| format!("{option} requires a value"))
            };

            match arg.as_str() {
                "--file" => file = Some(PathBuf::from(value(arg)?)),
                "--expected-file-sha256" => {
                    let expected = value(arg)?;
                    if expected.len() != 64
                        || !expected.bytes().all(|byte| byte.is_ascii_hexdigit())
                    {
                        bail!("--expected-file-sha256 requires 64 hexadecimal characters");
                    }
                    expected_file_sha256 = Some(expected);
                }
                "--artboard" => artboard = Some(value(arg)?),
                "--state-machine" => state_machine = Some(value(arg)?),
                "--input-script" => input_script = Some(PathBuf::from(value(arg)?)),
                "--view-model-script" => view_model_script = Some(PathBuf::from(value(arg)?)),
                "--samples" => samples = parse_samples(&value(arg)?)?,
                "--layout-bounds" => layout_bounds = true,
                "--benchmark" => benchmark = true,
                "--execute-scripts" => execute_scripts = true,
                "--side-channel" => side_channel = true,
                "--semantic-default-view-model" => semantic_default_view_model = true,
                "--semantic-side-channel-only" => semantic_side_channel_only = true,
                "--benchmark-repeat" => {
                    benchmark_repeat = parse_positive_usize(&value(arg)?, arg)?;
                }
                "--help" | "-h" => {
                    println!(
                        "usage: rust-golden-runner --file <path> [--expected-file-sha256 SHA256] [--artboard <name>] [--samples <t0,t1,...>] [--input-script <path>] [--view-model-script <path>] [--layout-bounds] [--execute-scripts] [--side-channel] [--semantic-default-view-model] [--semantic-side-channel-only] [--benchmark] [--benchmark-repeat N]"
                    );
                    std::process::exit(0);
                }
                other if !other.starts_with('-') && file.is_none() => {
                    file = Some(PathBuf::from(other));
                }
                other => bail!("unknown option: {other}"),
            }
            index += 1;
        }

        if layout_bounds && benchmark {
            bail!("--benchmark cannot be combined with --layout-bounds");
        }
        if side_channel && benchmark {
            bail!("--side-channel cannot be combined with --benchmark");
        }
        if side_channel && layout_bounds {
            bail!("--side-channel cannot be combined with --layout-bounds");
        }
        if semantic_default_view_model && !side_channel {
            bail!("--semantic-default-view-model requires --side-channel");
        }
        if semantic_side_channel_only && !side_channel {
            bail!("--semantic-side-channel-only requires --side-channel");
        }
        if benchmark_repeat > 1 {
            if !benchmark {
                bail!("--benchmark-repeat requires --benchmark");
            }
            if input_script.is_some() || view_model_script.is_some() {
                bail!("--benchmark-repeat cannot be combined with scripts");
            }
            if samples.len() != 1 {
                bail!("--benchmark-repeat requires exactly one sample");
            }
        }

        Ok(Self {
            file: file.context("missing --file <path>")?,
            expected_file_sha256,
            artboard,
            state_machine,
            input_script,
            view_model_script,
            samples,
            layout_bounds,
            benchmark,
            benchmark_repeat,
            execute_scripts,
            side_channel,
            semantic_default_view_model,
            semantic_side_channel_only,
        })
    }
}

fn parse_positive_usize(value: &str, option: &str) -> Result<usize> {
    let parsed = value
        .parse::<usize>()
        .with_context(|| format!("{option} must be a positive integer"))?;
    if parsed == 0 {
        bail!("{option} must be greater than 0");
    }
    Ok(parsed)
}

fn parse_samples(value: &str) -> Result<Vec<f32>> {
    let samples = value
        .split(',')
        .map(|part| {
            part.trim()
                .parse::<f32>()
                .with_context(|| format!("invalid sample {}", part.trim()))
        })
        .collect::<Result<Vec<_>>>()?;
    if samples.is_empty() {
        bail!("--samples must include at least one sample");
    }
    if samples.iter().any(|sample| *sample < 0.0) {
        bail!("samples must be non-negative");
    }
    for pair in samples.windows(2) {
        if pair[1] + TIME_EPSILON < pair[0] {
            bail!("samples must be sorted");
        }
    }
    Ok(samples)
}

fn frame_dimension(value: f32) -> u32 {
    value.ceil().max(1.0) as u32
}
