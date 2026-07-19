use anyhow::{Context, Result, anyhow, bail};
use nuxie_binary::{RuntimeFile, RuntimeObject, read_runtime_file};
use nuxie_graph::{
    ArtboardGraph, GraphFile, ShapePaintContainerNode, ShapePaintKind, ShapePaintPathKind,
    ShapePaintStateNode,
};
use nuxie_render_api::{
    Factory as RenderFactory, NullFactory, RecordingFactory, Renderer as RenderRenderer,
};
use nuxie_runtime::{
    ArtboardInstance, RuntimeLayoutBoundsReport, RuntimeOwnedViewModelContextHandle,
    RuntimeOwnedViewModelHandle, RuntimeOwnedViewModelInstance, RuntimeRenderPathCache,
    StateMachineInstance, preallocate_render_paint_cache_for_artboard_tree,
    static_text_support_error,
};
#[cfg(feature = "scripting")]
use nuxie_runtime::{
    NoopScriptHost, ScriptArtboard, ScriptError, ScriptMethod, ScriptValue, ScriptViewModel,
    preallocate_render_paint_cache_for_artboard_instance,
    preallocate_render_paint_cache_for_scripted_artboard_tree_after_source_paints,
    preallocate_source_render_paints,
};
#[cfg(feature = "scripting")]
use nuxie_scripting::vm::ScriptVm;
#[cfg(feature = "scripting")]
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::env;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
#[cfg(feature = "scripting")]
use std::{cell::RefCell, rc::Rc};

const TIME_EPSILON: f32 = 0.000001;
const DATA_BIND_FLAG_DIRECTION_TO_SOURCE: u64 = 1 << 0;
const DATA_BIND_FLAG_TWO_WAY: u64 = 1 << 1;

trait RunnerBackend {
    fn as_factory(&mut self) -> &mut dyn RenderFactory;
    fn make_renderer(&self) -> Box<dyn RenderRenderer>;
    fn source(&mut self, file: &str, artboard: &str, scene: &str);
    fn frame_size(&mut self, width: u32, height: u32);
    fn add_input_event(&mut self, kind: &str, seconds: f32, x: f32, y: f32, pointer_id: i32);
    fn add_sample(&mut self, seconds: f32);
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

    fn add_sample(&mut self, seconds: f32) {
        RecordingFactory::add_sample(self, seconds);
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

    fn add_sample(&mut self, _seconds: f32) {}

    fn add_frame(&mut self) {}

    fn stream(&self) -> String {
        String::new()
    }
}

fn main() {
    match run() {
        Ok(stream) => print!("{stream}"),
        Err(error) => {
            #[cfg(feature = "scripting")]
            if let Some(feature) = scripting_unsupported_feature(&error) {
                eprintln!(
                    "rust-golden-runner error: unsupported: {feature} in Rust golden runner ({error:#})"
                );
                std::process::exit(1);
            }
            eprintln!("rust-golden-runner error: {error:#}");
            std::process::exit(1);
        }
    }
}

#[cfg(feature = "scripting")]
fn scripting_unsupported_feature(error: &anyhow::Error) -> Option<&'static str> {
    let message = format!("{error:#}");
    if message.contains("attempt to call missing method 'advance' of userdata") {
        Some("script-artboard-advance")
    } else if message.contains("attempt to call missing method 'animation' of userdata") {
        Some("script-artboard-animation")
    } else if message.contains("attempt to call missing method 'node' of userdata") {
        Some("script-artboard-node")
    } else if message.contains("Paint allocation requires an active scripted draw context") {
        Some("script-init-paint")
    } else if message.contains("attempt to index nil with 'viewModel'")
        || message.contains("attempt to index nil with 'lis'")
        || message.contains("attempt to index nil with 'Child'")
        || message.contains("attempt to index nil with 'value'")
    {
        Some("script-view-model")
    } else if message.contains("attempt to index nil with 'addListener'") {
        Some("script-view-model-property-listener")
    } else {
        None
    }
}

fn run() -> Result<String> {
    let options = Options::parse(env::args().skip(1).collect())?;
    let input_events = options
        .input_script
        .as_deref()
        .map(load_input_script)
        .transpose()?
        .unwrap_or_default();
    let bytes = std::fs::read(&options.file)
        .with_context(|| format!("failed to read {}", options.file.display()))?;
    let runtime = read_runtime_file(&bytes).context("failed to import runtime file")?;
    let graph = GraphFile::from_runtime_file(&runtime).context("failed to build graph")?;
    let (artboard_index, artboard) = select_artboard(&graph, options.artboard.as_deref())?;
    if !options.layout_bounds {
        ensure_static_draw_supported(&runtime, &graph, artboard, !input_events.is_empty())?;
    }
    let mut instance =
        ArtboardInstance::from_graph_with_artboards(&runtime, artboard, &graph.artboards)
            .context("failed to instantiate artboard")?;
    let mut factory: Box<dyn RunnerBackend> = if options.benchmark {
        Box::new(NullFactory::new())
    } else {
        Box::new(RecordingFactory::new())
    };
    #[cfg(feature = "scripting")]
    let has_scripted_layout = artboard
        .local_objects
        .iter()
        .any(|object| object.type_name == Some("ScriptedLayout"));
    #[cfg(feature = "scripting")]
    let has_script_artboard_input = artboard
        .local_objects
        .iter()
        .any(|object| object.type_name == Some("ScriptInputArtboard"));
    #[cfg(feature = "scripting")]
    let (script_artboard_render_state, mut paint_cache) = if has_scripted_layout {
        let _source_paints = preallocate_source_render_paints(&runtime, factory.as_factory());
        let state = initialize_scripted_drawables_and_realize(
            &runtime,
            artboard_index,
            artboard,
            &graph.artboards,
            &mut instance,
            factory.as_factory(),
        )?;
        let cache = preallocate_render_paint_cache_for_scripted_artboard_tree_after_source_paints(
            &runtime,
            artboard,
            &graph.artboards,
            factory.as_factory(),
        );
        (state, cache)
    } else if has_script_artboard_input {
        let _source_paints = preallocate_source_render_paints(&runtime, factory.as_factory());
        let state = initialize_scripted_drawables(
            &runtime,
            artboard_index,
            artboard,
            &graph.artboards,
            &mut instance,
            factory.as_factory(),
        )
        .context("failed to initialize scripted drawables")?;
        let cache = preallocate_render_paint_cache_for_scripted_artboard_tree_after_source_paints(
            &runtime,
            artboard,
            &graph.artboards,
            factory.as_factory(),
        );
        if let Some(state) = state.as_ref() {
            state
                .borrow_mut()
                .realize_pending(&runtime, &graph.artboards, factory.as_factory())
                .context("failed to allocate initialized script artboard paints")?;
        }
        (state, cache)
    } else {
        let cache = nuxie_runtime::preallocate_render_paint_cache_for_scripted_artboard_tree(
            &runtime,
            artboard,
            &graph.artboards,
            factory.as_factory(),
        );
        let state = initialize_scripted_drawables_and_realize(
            &runtime,
            artboard_index,
            artboard,
            &graph.artboards,
            &mut instance,
            factory.as_factory(),
        )?;
        (state, cache)
    };
    #[cfg(not(feature = "scripting"))]
    let mut paint_cache = preallocate_render_paint_cache_for_artboard_tree(
        &runtime,
        artboard,
        &graph.artboards,
        factory.as_factory(),
    );
    let scene = select_scene(
        &runtime,
        artboard_index,
        artboard,
        options.state_machine.as_deref(),
    )?;
    let mut state_machine = scene
        .state_machine_index
        .map(|index| {
            instance
                .state_machine_instance(index)
                .with_context(|| format!("failed to instantiate state machine index {index}"))
        })
        .transpose()?;
    let mut owned_view_model_context =
        selected_artboard_owned_view_model_context(&runtime, artboard_index);
    instance.bind_default_view_model_artboard_list_context(&runtime);
    #[cfg(feature = "scripting")]
    if let Some(state) = script_artboard_render_state.as_ref() {
        // Mirrors RIVLoader's Artboard::bindViewModelInstance rehydration.
        rebind_scripted_drawables(
            &runtime,
            artboard,
            &graph.artboards,
            &mut instance,
            state,
            factory.as_factory(),
            owned_view_model_context.as_ref(),
        )?;
    }
    if let Some(state_machine) = state_machine.as_mut() {
        if let Some(context) = owned_view_model_context.as_ref() {
            state_machine.bind_owned_view_model_handle(context);
        }
        state_machine.advance_data_context();
    }
    if let Some(context) = owned_view_model_context.as_ref() {
        bind_selected_artboard_view_model_context(&mut instance, &runtime, context);
    }
    #[cfg(feature = "scripting")]
    if owned_view_model_context.is_some()
        && let Some(state) = script_artboard_render_state.as_ref()
    {
        rebind_scripted_drawables(
            &runtime,
            artboard,
            &graph.artboards,
            &mut instance,
            state,
            factory.as_factory(),
            owned_view_model_context.as_ref(),
        )?;
    }
    #[cfg(feature = "scripting")]
    if let Some(state) = script_artboard_render_state.as_ref() {
        initialize_nested_scripted_drawables(
            &runtime,
            artboard_index,
            &graph.artboards,
            &mut instance,
            factory.as_factory(),
            state,
        )?;
        state
            .borrow_mut()
            .realize_pending(&runtime, &graph.artboards, factory.as_factory())
            .context("failed to allocate nested script artboard paints")?;
    }
    #[cfg(feature = "scripting")]
    if script_artboard_render_state.is_some() {
        // C++ leaves ScriptUpdate dirty for the first update pass after binds.
        mark_scripted_drawables_for_update(artboard, &mut instance);
    }

    let artboard_name = artboard.name.clone().unwrap_or_default();
    if options.layout_bounds {
        return write_layout_bounds_report(
            &options,
            &runtime,
            artboard,
            &artboard_name,
            &scene.name,
            &mut instance,
            &mut state_machine,
            &mut owned_view_model_context,
            &input_events,
        );
    }

    if options.benchmark && options.benchmark_repeat > 1 {
        return write_benchmark_repeat_report(
            &options,
            &runtime,
            &graph,
            artboard_index,
            artboard,
            &scene,
        );
    }

    let mut path_cache = RuntimeRenderPathCache::default();
    let mut renderer = factory.make_renderer();

    let artboard_object = runtime
        .artboard(artboard_index)
        .context("missing selected artboard object")?;
    let width = artboard_object.double_property("width").unwrap_or(0.0);
    let height = artboard_object.double_property("height").unwrap_or(0.0);

    factory.source(&options.file.to_string_lossy(), &artboard_name, &scene.name);
    factory.frame_size(frame_dimension(width), frame_dimension(height));

    let benchmark_start = Instant::now();
    let mut advance_elapsed = Duration::ZERO;
    let mut input_elapsed = Duration::ZERO;
    let mut prepare_elapsed = Duration::ZERO;
    let mut draw_elapsed = Duration::ZERO;
    let mut current_seconds = 0.0;
    let mut next_input = 0;
    #[cfg(feature = "scripting")]
    let mut bound_script_artboards = BTreeMap::new();
    for _ in 0..options.benchmark_repeat {
        for sample in &options.samples {
            while next_input < input_events.len()
                && input_events[next_input].seconds <= *sample + TIME_EPSILON
            {
                let event = &input_events[next_input];
                timed_result(options.benchmark, &mut advance_elapsed, || {
                    advance_scene_to(
                        &mut instance,
                        &runtime,
                        state_machine.as_mut(),
                        owned_view_model_context.as_ref(),
                        event.seconds,
                        &mut current_seconds,
                    )
                })?;
                timed(options.benchmark, &mut input_elapsed, || {
                    apply_input_event(
                        event,
                        &instance,
                        state_machine.as_mut(),
                        owned_view_model_context.as_ref(),
                    );
                    factory.add_input_event(
                        event.kind.name(),
                        event.seconds,
                        event.x,
                        event.y,
                        event.pointer_id,
                    );
                });
                next_input += 1;
            }
            timed_result(options.benchmark, &mut advance_elapsed, || {
                advance_scene_to(
                    &mut instance,
                    &runtime,
                    state_machine.as_mut(),
                    owned_view_model_context.as_ref(),
                    *sample,
                    &mut current_seconds,
                )
            })?;
            #[cfg(feature = "scripting")]
            if let Some(state) = script_artboard_render_state.as_ref() {
                refresh_bound_script_artboard_inputs(
                    &runtime,
                    artboard,
                    &graph.artboards,
                    &mut instance,
                    state,
                    owned_view_model_context.as_ref(),
                    &mut bound_script_artboards,
                )?;
                state.borrow_mut().realize_pending(
                    &runtime,
                    &graph.artboards,
                    factory.as_factory(),
                )?;
            }
            timed_result(options.benchmark, &mut prepare_elapsed, || {
                instance.prepare_static_artboard_tree_paints(
                    &runtime,
                    artboard,
                    &graph.artboards,
                    factory.as_factory(),
                    &mut paint_cache,
                    &mut path_cache,
                )
            })?;
            factory.add_sample(*sample);
            timed_result(options.benchmark, &mut draw_elapsed, || {
                instance
                    .draw_prepared_static_artboard_with_render_cache(
                        &runtime,
                        artboard,
                        &graph.artboards,
                        factory.as_factory(),
                        &mut *renderer,
                        &mut paint_cache,
                        &mut path_cache,
                    )
                    .map_err(unsupported_static_text_draw_error)
            })?;
            factory.add_frame();
        }
    }
    let benchmark_elapsed = benchmark_start.elapsed();

    if options.benchmark {
        let accounted_elapsed = advance_elapsed + input_elapsed + prepare_elapsed + draw_elapsed;
        let bookkeeping_elapsed = benchmark_elapsed.saturating_sub(accounted_elapsed);
        Ok(format!(
            "rive-golden-benchmark-v1\nelapsed_ms={}\nadvance_ms={}\ninput_ms={}\nprepare_ms={}\ndraw_ms={}\nbookkeeping_ms={}\nsegments={}\n",
            benchmark_elapsed.as_secs_f64() * 1000.0,
            advance_elapsed.as_secs_f64() * 1000.0,
            input_elapsed.as_secs_f64() * 1000.0,
            prepare_elapsed.as_secs_f64() * 1000.0,
            draw_elapsed.as_secs_f64() * 1000.0,
            bookkeeping_elapsed.as_secs_f64() * 1000.0,
            options.samples.len() * options.benchmark_repeat
        ))
    } else {
        Ok(factory.stream())
    }
}

#[cfg(feature = "scripting")]
fn refresh_bound_script_artboard_inputs(
    runtime: &RuntimeFile,
    artboard: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    instance: &mut ArtboardInstance,
    render_state: &Rc<RefCell<RunnerScriptArtboardRenderState>>,
    owned_view_model_context: Option<&RuntimeOwnedViewModelHandle>,
    current_artboards: &mut BTreeMap<u32, u64>,
) -> Result<()> {
    let Some(context) = owned_view_model_context else {
        return Ok(());
    };
    for global_id in artboard_object_range(runtime, artboard, artboards) {
        let Some(input) = runtime.object(global_id) else {
            continue;
        };
        if input.type_name != "ScriptInputArtboard" {
            continue;
        }
        let Some(parent_local_id) = input
            .uint_property("parentId")
            .and_then(|id| usize::try_from(id).ok())
        else {
            continue;
        };
        let Some(scripted_global_id) = artboard
            .local_objects
            .iter()
            .find(|object| object.local_id == parent_local_id)
            .map(|object| object.global_id)
        else {
            continue;
        };
        let Some(bound_artboard_id) =
            nuxie_runtime::bound_script_artboard_input(runtime, &context.borrow(), input)
        else {
            continue;
        };
        let authored_artboard_id = input.uint_property("artboardId").unwrap_or(u64::MAX);
        let current_artboard_id = current_artboards
            .entry(input.id)
            .or_insert(authored_artboard_id);
        if *current_artboard_id == bound_artboard_id {
            continue;
        }
        let artboard_index = usize::try_from(bound_artboard_id)
            .context("bound script artboard id does not fit usize")?;
        let script_artboard =
            RunnerScriptArtboard::new(runtime, artboards, artboard_index, Rc::clone(render_state))?;
        let name = input.string_property("name").unwrap_or_default();
        instance
            .set_script_artboard_input_for_global(
                scripted_global_id,
                name,
                Box::new(script_artboard),
            )
            .with_context(|| format!("failed to update bound artboard script input '{name}'"))?;
        *current_artboard_id = bound_artboard_id;
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

fn write_benchmark_repeat_report(
    options: &Options,
    runtime: &RuntimeFile,
    graph: &GraphFile,
    artboard_index: usize,
    artboard: &ArtboardGraph,
    scene: &SelectedScene,
) -> Result<String> {
    let total = run_benchmark_repeat_pass(
        options,
        runtime,
        graph,
        artboard_index,
        artboard,
        scene,
        false,
    )?;
    let phases = run_benchmark_repeat_pass(
        options,
        runtime,
        graph,
        artboard_index,
        artboard,
        scene,
        true,
    )?;
    let accounted_elapsed = phases.advance + phases.input + phases.prepare + phases.draw;
    let bookkeeping_elapsed = phases.elapsed.saturating_sub(accounted_elapsed);
    Ok(format!(
        "rive-golden-benchmark-v1\nelapsed_ms={}\ntotal_ms={}\nadvance_ms={}\ninput_ms={}\nprepare_ms={}\ndraw_ms={}\nbookkeeping_ms={}\nsegments={}\n",
        total.elapsed.as_secs_f64() * 1000.0,
        total.elapsed.as_secs_f64() * 1000.0,
        phases.advance.as_secs_f64() * 1000.0,
        phases.input.as_secs_f64() * 1000.0,
        phases.prepare.as_secs_f64() * 1000.0,
        phases.draw.as_secs_f64() * 1000.0,
        bookkeeping_elapsed.as_secs_f64() * 1000.0,
        options.samples.len() * options.benchmark_repeat
    ))
}

fn run_benchmark_repeat_pass(
    options: &Options,
    runtime: &RuntimeFile,
    graph: &GraphFile,
    artboard_index: usize,
    artboard: &ArtboardGraph,
    scene: &SelectedScene,
    collect_phases: bool,
) -> Result<BenchmarkTimings> {
    let (mut instance, mut state_machine, owned_view_model_context) =
        instantiate_scene_state(runtime, artboard_index, artboard, &graph.artboards, scene)?;
    let mut factory = NullFactory::new();
    let mut paint_cache = preallocate_render_paint_cache_for_artboard_tree(
        runtime,
        artboard,
        &graph.artboards,
        &mut factory,
    );
    let mut path_cache = RuntimeRenderPathCache::default();
    let mut renderer = factory.make_renderer();

    let mut advance_elapsed = Duration::ZERO;
    let input_elapsed = Duration::ZERO;
    let mut prepare_elapsed = Duration::ZERO;
    let mut draw_elapsed = Duration::ZERO;
    let mut current_seconds = 0.0;
    let benchmark_start = Instant::now();
    for _ in 0..options.benchmark_repeat {
        for sample in &options.samples {
            if collect_phases {
                timed_result(true, &mut advance_elapsed, || {
                    advance_scene_to(
                        &mut instance,
                        runtime,
                        state_machine.as_mut(),
                        owned_view_model_context.as_ref(),
                        *sample,
                        &mut current_seconds,
                    )
                })?;
                timed_result(true, &mut prepare_elapsed, || {
                    instance.prepare_static_artboard_tree_paints(
                        runtime,
                        artboard,
                        &graph.artboards,
                        &mut factory,
                        &mut paint_cache,
                        &mut path_cache,
                    )
                })?;
                timed_result(true, &mut draw_elapsed, || {
                    instance
                        .draw_prepared_static_artboard_with_render_cache(
                            runtime,
                            artboard,
                            &graph.artboards,
                            &mut factory,
                            &mut renderer,
                            &mut paint_cache,
                            &mut path_cache,
                        )
                        .map_err(unsupported_static_text_draw_error)
                })?;
            } else {
                advance_scene_to(
                    &mut instance,
                    runtime,
                    state_machine.as_mut(),
                    owned_view_model_context.as_ref(),
                    *sample,
                    &mut current_seconds,
                )?;
                instance.prepare_static_artboard_tree_paints(
                    runtime,
                    artboard,
                    &graph.artboards,
                    &mut factory,
                    &mut paint_cache,
                    &mut path_cache,
                )?;
                instance
                    .draw_prepared_static_artboard_with_render_cache(
                        runtime,
                        artboard,
                        &graph.artboards,
                        &mut factory,
                        &mut renderer,
                        &mut paint_cache,
                        &mut path_cache,
                    )
                    .map_err(unsupported_static_text_draw_error)?;
            }
        }
    }

    Ok(BenchmarkTimings {
        elapsed: benchmark_start.elapsed(),
        advance: advance_elapsed,
        input: input_elapsed,
        prepare: prepare_elapsed,
        draw: draw_elapsed,
    })
}

fn instantiate_scene_state(
    runtime: &RuntimeFile,
    artboard_index: usize,
    artboard: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    scene: &SelectedScene,
) -> Result<(
    ArtboardInstance,
    Option<StateMachineInstance>,
    Option<RuntimeOwnedViewModelHandle>,
)> {
    let mut instance = ArtboardInstance::from_graph_with_artboards(runtime, artboard, artboards)
        .context("failed to instantiate artboard")?;
    let mut state_machine = scene
        .state_machine_index
        .map(|index| {
            instance
                .state_machine_instance(index)
                .with_context(|| format!("failed to instantiate state machine index {index}"))
        })
        .transpose()?;
    let owned_view_model_context =
        selected_artboard_owned_view_model_context(runtime, artboard_index);
    instance.bind_default_view_model_artboard_list_context(runtime);
    if let Some(state_machine) = state_machine.as_mut() {
        if let Some(context) = owned_view_model_context.as_ref() {
            state_machine.bind_owned_view_model_handle(context);
        }
        state_machine.advance_data_context();
    }
    if let Some(context) = owned_view_model_context.as_ref() {
        bind_selected_artboard_view_model_context(&mut instance, runtime, context);
    }
    Ok((instance, state_machine, owned_view_model_context))
}

fn bind_selected_artboard_view_model_context(
    instance: &mut ArtboardInstance,
    runtime: &RuntimeFile,
    context: &RuntimeOwnedViewModelHandle,
) {
    #[cfg(feature = "scripting")]
    {
        instance.bind_owned_view_model_artboard_handle(runtime, context);
        instance.rebind_nested_script_owned_contexts(runtime);
    }
    #[cfg(not(feature = "scripting"))]
    instance.bind_owned_view_model_nested_artboard_contexts(runtime, &context.borrow());
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

fn write_layout_bounds_report(
    options: &Options,
    runtime: &RuntimeFile,
    artboard: &ArtboardGraph,
    artboard_name: &str,
    scene_name: &str,
    instance: &mut ArtboardInstance,
    state_machine: &mut Option<StateMachineInstance>,
    owned_view_model_context: &mut Option<RuntimeOwnedViewModelHandle>,
    input_events: &[InputEvent],
) -> Result<String> {
    let mut out = String::new();
    out.push('{');
    out.push_str("\"source\":");
    push_json_string(&mut out, &options.file.to_string_lossy());
    out.push_str(",\"artboard\":");
    push_json_string(&mut out, artboard_name);
    out.push_str(",\"scene\":");
    push_json_string(&mut out, scene_name);
    out.push_str(",\"samples\":[");

    let mut current_seconds = 0.0;
    let mut next_input = 0;
    for (sample_index, sample) in options.samples.iter().enumerate() {
        while next_input < input_events.len()
            && input_events[next_input].seconds <= *sample + TIME_EPSILON
        {
            let event = &input_events[next_input];
            advance_scene_to(
                instance,
                runtime,
                state_machine.as_mut(),
                owned_view_model_context.as_ref(),
                event.seconds,
                &mut current_seconds,
            )?;
            apply_input_event(
                event,
                instance,
                state_machine.as_mut(),
                owned_view_model_context.as_ref(),
            );
            next_input += 1;
        }

        advance_scene_to(
            instance,
            runtime,
            state_machine.as_mut(),
            owned_view_model_context.as_ref(),
            *sample,
            &mut current_seconds,
        )?;
        let reports = instance
            .debug_taffy_layout_bounds_report(runtime, artboard)
            .context("failed to compute Taffy layout bounds")?;

        if sample_index != 0 {
            out.push(',');
        }
        out.push_str("{\"sample\":");
        write!(&mut out, "{sample}")?;
        out.push_str(",\"layoutBounds\":[");
        for (report_index, report) in reports.iter().enumerate() {
            if report_index != 0 {
                out.push(',');
            }
            push_layout_bounds_report(&mut out, report)?;
        }
        out.push_str("]}");
    }

    out.push_str("]}\n");
    Ok(out)
}

fn push_layout_bounds_report(out: &mut String, report: &RuntimeLayoutBoundsReport) -> Result<()> {
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

fn unsupported_static_text_draw_error(error: anyhow::Error) -> anyhow::Error {
    let message = format!("{error:#}");
    if message.contains("static text subset") {
        let feature = unsupported_static_text_feature(&message);
        anyhow!("unsupported: {feature} in Rust golden runner ({message})")
    } else {
        error
    }
}

fn unsupported_static_text_feature(message: &str) -> &'static str {
    if message.contains("verticalTrim") {
        "text-vertical-trim"
    } else if message.contains("data binding target Joystick.") {
        "text-joystick-data-bind"
    } else if message.contains("NestedArtboardLayout") || message.contains("NestedArtboardLeaf") {
        "nested-artboard-layout"
    } else if message.contains("TextModifierGroup flags") {
        "text-modifier-group-flags"
    } else if message.contains("sibling Polygon") {
        "text-polygon-sibling"
    } else {
        "text"
    }
}

fn selected_artboard_view_model_index(
    runtime: &RuntimeFile,
    artboard_index: usize,
) -> Option<usize> {
    let artboard = runtime.artboard(artboard_index)?;
    usize::try_from(artboard.uint_property("viewModelId")?).ok()
}

fn selected_artboard_owned_view_model_context(
    runtime: &RuntimeFile,
    artboard_index: usize,
) -> Option<RuntimeOwnedViewModelHandle> {
    let view_model_index = selected_artboard_view_model_index(runtime, artboard_index)?;
    #[cfg(feature = "scripting")]
    {
        return RuntimeOwnedViewModelInstance::from_instance(runtime, view_model_index, 0)
            .map(RuntimeOwnedViewModelHandle::new);
    }
    #[cfg(not(feature = "scripting"))]
    RuntimeOwnedViewModelInstance::new(runtime, view_model_index)
        .map(RuntimeOwnedViewModelHandle::new)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputKind {
    PointerDown,
    PointerMove,
    PointerUp,
    PointerExit,
}

impl InputKind {
    fn parse(value: &str, line_number: usize) -> Result<Self> {
        match value {
            "pointerDown" => Ok(Self::PointerDown),
            "pointerMove" => Ok(Self::PointerMove),
            "pointerUp" => Ok(Self::PointerUp),
            "pointerExit" => Ok(Self::PointerExit),
            _ => bail!("unknown input event on line {line_number}: {value}"),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::PointerDown => "pointerDown",
            Self::PointerMove => "pointerMove",
            Self::PointerUp => "pointerUp",
            Self::PointerExit => "pointerExit",
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
    order: usize,
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
        if tokens.len() != 4 && tokens.len() != 5 {
            bail!("input script line {line_number} must be: <seconds> <event> <x> <y> [pointerId]");
        }
        let seconds = parse_script_float(
            tokens[0],
            &format!("input script line {line_number} seconds"),
        )?;
        if seconds < 0.0 {
            bail!("input script line {line_number} has a negative time");
        }
        let kind = InputKind::parse(tokens[1], line_number)?;
        let x = parse_script_float(tokens[2], &format!("input script line {line_number} x"))?;
        let y = parse_script_float(tokens[3], &format!("input script line {line_number} y"))?;
        let pointer_id = if let Some(pointer_id) = tokens.get(4) {
            pointer_id.parse::<i32>().with_context(|| {
                format!(
                    "invalid integer for input script line {line_number} pointerId: {pointer_id}"
                )
            })?
        } else {
            0
        };
        events.push(InputEvent {
            seconds,
            kind,
            x,
            y,
            pointer_id,
            order: events.len(),
        });
    }

    events.sort_by(|left, right| {
        if (left.seconds - right.seconds).abs() <= TIME_EPSILON {
            left.order.cmp(&right.order)
        } else {
            left.seconds.total_cmp(&right.seconds)
        }
    });
    Ok(events)
}

fn parse_script_float(value: &str, context: &str) -> Result<f32> {
    value
        .parse::<f32>()
        .with_context(|| format!("invalid float for {context}: {value}"))
}

fn apply_input_event(
    event: &InputEvent,
    instance: &ArtboardInstance,
    state_machine: Option<&mut StateMachineInstance>,
    owned_view_model_context: Option<&RuntimeOwnedViewModelHandle>,
) {
    let Some(state_machine) = state_machine else {
        return;
    };
    match event.kind {
        InputKind::PointerDown => {
            if let Some(context) = owned_view_model_context {
                let mut context = context.borrow_mut();
                state_machine.pointer_down_with_owned_view_model_context(
                    instance,
                    event.x,
                    event.y,
                    event.pointer_id,
                    &mut context,
                );
            } else {
                state_machine.pointer_down(instance, event.x, event.y, event.pointer_id);
            }
        }
        InputKind::PointerMove => {
            if let Some(context) = owned_view_model_context {
                let mut context = context.borrow_mut();
                state_machine.pointer_move_with_owned_view_model_context(
                    instance,
                    event.x,
                    event.y,
                    event.seconds,
                    event.pointer_id,
                    &mut context,
                );
            } else {
                state_machine.pointer_move(
                    instance,
                    event.x,
                    event.y,
                    event.seconds,
                    event.pointer_id,
                );
            }
        }
        InputKind::PointerUp => {
            if let Some(context) = owned_view_model_context {
                let mut context = context.borrow_mut();
                state_machine.pointer_up_with_owned_view_model_context(
                    instance,
                    event.x,
                    event.y,
                    event.pointer_id,
                    &mut context,
                );
            } else {
                state_machine.pointer_up(instance, event.x, event.y, event.pointer_id);
            }
        }
        InputKind::PointerExit => {
            if let Some(context) = owned_view_model_context {
                let mut context = context.borrow_mut();
                state_machine.pointer_exit_with_owned_view_model_context(
                    instance,
                    event.x,
                    event.y,
                    event.pointer_id,
                    &mut context,
                );
            } else {
                state_machine.pointer_exit(instance, event.x, event.y, event.pointer_id);
            }
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
    fn sample_parser_matches_golden_runner_tolerance() {
        assert_eq!(
            parse_samples("0.1,0.0999995,0.2").expect("within epsilon is sorted"),
            vec![0.1, 0.0999995, 0.2]
        );
        assert!(parse_samples("-0.1").is_err());
        assert!(parse_samples("0.1,0.099").is_err());
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
}

#[derive(Debug)]
struct SelectedScene {
    name: String,
    state_machine_index: Option<usize>,
}

fn select_scene(
    runtime: &RuntimeFile,
    artboard_index: usize,
    artboard: &ArtboardGraph,
    state_machine_name: Option<&str>,
) -> Result<SelectedScene> {
    let artboard_name = artboard.name.clone().unwrap_or_default();
    if let Some(name) = state_machine_name {
        let index = artboard
            .state_machines
            .iter()
            .position(|state_machine| state_machine.name.as_deref() == Some(name))
            .with_context(|| format!("missing state machine {name}"))?;
        return Ok(SelectedScene {
            name: name.to_owned(),
            state_machine_index: Some(index),
        });
    }

    let Some(default_state_machine_index) = runtime
        .artboard(artboard_index)
        .and_then(|artboard| {
            artboard
                .property("defaultStateMachineId")
                .and_then(|_| artboard.uint_property("defaultStateMachineId"))
        })
        .and_then(|index| usize::try_from(index).ok())
        .filter(|index| artboard.state_machines.get(*index).is_some())
    else {
        return Ok(SelectedScene {
            name: artboard_name,
            state_machine_index: None,
        });
    };

    let name = artboard.state_machines[default_state_machine_index]
        .name
        .clone()
        .unwrap_or_else(|| artboard_name);
    Ok(SelectedScene {
        name,
        state_machine_index: Some(default_state_machine_index),
    })
}

fn advance_scene_to(
    instance: &mut ArtboardInstance,
    runtime: &RuntimeFile,
    mut state_machine: Option<&mut StateMachineInstance>,
    owned_view_model_context: Option<&RuntimeOwnedViewModelHandle>,
    target_seconds: f32,
    current_seconds: &mut f32,
) -> Result<()> {
    if target_seconds + TIME_EPSILON < *current_seconds {
        bail!("cannot move timeline backwards");
    }
    let elapsed_seconds = (target_seconds - *current_seconds).max(0.0);
    if let Some(state_machine) = state_machine.as_deref_mut() {
        instance.advance_state_machine_instance(state_machine, elapsed_seconds);
        if instance.advance_nested_artboards_with_state_machine(elapsed_seconds, state_machine) {
            instance.advance_state_machine_instance(state_machine, 0.0);
        }
    } else {
        instance.advance_nested_artboards(elapsed_seconds);
    }
    if let Some(context) = owned_view_model_context {
        bind_selected_artboard_view_model_context(instance, runtime, context);
    }
    instance.advance_artboard_data_binds_with_elapsed(elapsed_seconds);
    instance
        .advance_script_instances(elapsed_seconds)
        .context("scripted drawable advance failed")?;
    instance
        .update_script_instances()
        .context("scripted drawable update failed")?;
    if let Some(state_machine) = state_machine.as_deref_mut() {
        instance.settle_state_machine_update_passes_with_state_machines(std::slice::from_mut(
            state_machine,
        ));
    } else {
        instance.update_pass();
    }
    *current_seconds = target_seconds;
    Ok(())
}

#[derive(Debug)]
struct Options {
    file: PathBuf,
    artboard: Option<String>,
    state_machine: Option<String>,
    input_script: Option<PathBuf>,
    samples: Vec<f32>,
    layout_bounds: bool,
    benchmark: bool,
    benchmark_repeat: usize,
}

impl Options {
    fn parse(args: Vec<String>) -> Result<Self> {
        let mut file = None::<PathBuf>;
        let mut artboard = None;
        let mut state_machine = None;
        let mut input_script = None;
        let mut samples = vec![0.0];
        let mut layout_bounds = false;
        let mut benchmark = false;
        let mut benchmark_repeat = 1usize;

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
                "--artboard" => artboard = Some(value(arg)?),
                "--state-machine" => state_machine = Some(value(arg)?),
                "--input-script" => input_script = Some(PathBuf::from(value(arg)?)),
                "--samples" => samples = parse_samples(&value(arg)?)?,
                "--layout-bounds" => layout_bounds = true,
                "--benchmark" => benchmark = true,
                "--benchmark-repeat" => {
                    benchmark_repeat = parse_positive_usize(&value(arg)?, arg)?;
                }
                "--help" | "-h" => {
                    println!(
                        "usage: rust-golden-runner --file <path> [--artboard <name>] [--samples <t0,t1,...>] [--layout-bounds] [--benchmark] [--benchmark-repeat N]"
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
        if benchmark_repeat > 1 {
            if !benchmark {
                bail!("--benchmark-repeat requires --benchmark");
            }
            if input_script.is_some() {
                bail!("--benchmark-repeat cannot be combined with --input-script");
            }
            if samples.len() != 1 {
                bail!("--benchmark-repeat requires exactly one sample");
            }
        }

        Ok(Self {
            file: file.context("missing --file <path>")?,
            artboard,
            state_machine,
            input_script,
            samples,
            layout_bounds,
            benchmark,
            benchmark_repeat,
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

fn select_artboard<'a>(
    graph: &'a GraphFile,
    name: Option<&str>,
) -> Result<(usize, &'a ArtboardGraph)> {
    if let Some(name) = name {
        graph
            .artboards
            .iter()
            .enumerate()
            .find(|(_, artboard)| artboard.name.as_deref() == Some(name))
            .with_context(|| format!("missing artboard {name}"))
    } else {
        graph
            .artboards
            .first()
            .map(|artboard| (0, artboard))
            .context("missing default artboard")
    }
}

#[cfg(feature = "scripting")]
#[derive(Clone)]
struct ExtractedScriptAsset {
    name: String,
    payload: Vec<u8>,
}

#[cfg(feature = "scripting")]
struct RunnerScriptArtboard {
    runtime: RuntimeFile,
    artboards: Vec<ArtboardGraph>,
    artboard_index: usize,
    instance: ArtboardInstance,
    state_machine: Option<StateMachineInstance>,
    view_model: Option<nuxie_runtime::ScriptViewModel>,
    width: f32,
    height: f32,
    frame_origin: bool,
    render_instance_id: u64,
    render_state: Rc<RefCell<RunnerScriptArtboardRenderState>>,
    path_cache: RuntimeRenderPathCache,
}

#[cfg(feature = "scripting")]
#[derive(Default)]
struct RunnerScriptArtboardRenderState {
    next_instance_id: u64,
    pending: Vec<(u64, usize)>,
    caches: BTreeMap<u64, nuxie_runtime::RuntimeRenderPaintCache>,
}

#[cfg(feature = "scripting")]
impl RunnerScriptArtboardRenderState {
    fn register(&mut self, artboard_index: usize) -> u64 {
        let instance_id = self.next_instance_id;
        self.next_instance_id += 1;
        self.pending.push((instance_id, artboard_index));
        instance_id
    }

    fn realize_pending(
        &mut self,
        runtime: &RuntimeFile,
        artboards: &[ArtboardGraph],
        factory: &mut dyn RenderFactory,
    ) -> Result<()> {
        for (instance_id, artboard_index) in std::mem::take(&mut self.pending) {
            let graph = artboards
                .get(artboard_index)
                .with_context(|| format!("missing scripted artboard index {artboard_index}"))?;
            let cache = preallocate_render_paint_cache_for_artboard_instance(
                runtime, graph, artboards, factory,
            );
            self.caches.insert(instance_id, cache);
        }
        Ok(())
    }
}

#[cfg(feature = "scripting")]
impl RunnerScriptArtboard {
    fn new(
        runtime: &RuntimeFile,
        artboards: &[ArtboardGraph],
        artboard_index: usize,
        render_state: Rc<RefCell<RunnerScriptArtboardRenderState>>,
    ) -> Result<Self> {
        let graph = artboards
            .get(artboard_index)
            .with_context(|| format!("missing scripted artboard index {artboard_index}"))?;
        let instance = ArtboardInstance::from_graph_with_artboards(runtime, graph, artboards)
            .with_context(|| {
                format!("failed to instantiate scripted artboard index {artboard_index}")
            })?;
        let state_machine_index = runtime
            .artboard(artboard_index)
            .and_then(|artboard| artboard.uint_property("defaultStateMachineId"))
            .and_then(|index| usize::try_from(index).ok())
            .filter(|index| graph.state_machines.get(*index).is_some());
        let state_machine =
            state_machine_index.and_then(|index| instance.state_machine_instance(index));
        let object = runtime.object(graph.global_id as usize);
        let render_instance_id = render_state.borrow_mut().register(artboard_index);
        Ok(Self {
            runtime: runtime.clone(),
            artboards: artboards.to_vec(),
            artboard_index,
            instance,
            state_machine,
            view_model: selected_artboard_owned_view_model_context(runtime, artboard_index)
                .as_ref()
                .and_then(|context| nuxie_runtime::script_view_model_from_owned(runtime, context)),
            width: object
                .and_then(|object| object.double_property("width"))
                .unwrap_or(0.0),
            height: object
                .and_then(|object| object.double_property("height"))
                .unwrap_or(0.0),
            frame_origin: false,
            render_instance_id,
            render_state,
            path_cache: RuntimeRenderPathCache::default(),
        })
    }

    fn bind_view_model(&mut self) {
        let Some(view_model) = self.view_model.as_ref() else {
            return;
        };
        let owned = view_model.owned_instance();
        if let Some(state_machine) = self.state_machine.as_mut() {
            state_machine.bind_owned_view_model_context_handle(&owned);
            state_machine.advance_data_context();
        }
        self.instance
            .bind_owned_view_model_artboard_context_handle(&self.runtime, &owned);
    }
}

#[cfg(feature = "scripting")]
impl ScriptArtboard for RunnerScriptArtboard {
    fn width(&self) -> f32 {
        self.width
    }

    fn height(&self) -> f32 {
        self.height
    }

    fn frame_origin(&self) -> bool {
        self.frame_origin
    }

    fn set_width(&mut self, width: f32) {
        self.width = width;
        self.instance
            .set_artboard_dimensions(self.width, self.height);
    }

    fn set_height(&mut self, height: f32) {
        self.height = height;
        self.instance
            .set_artboard_dimensions(self.width, self.height);
    }

    fn set_frame_origin(&mut self, frame_origin: bool) {
        self.frame_origin = frame_origin;
    }

    fn data(&self) -> Option<nuxie_runtime::ScriptViewModel> {
        self.view_model.clone()
    }

    fn instance(
        &self,
        view_model: Option<nuxie_runtime::ScriptViewModel>,
    ) -> std::result::Result<Box<dyn ScriptArtboard>, ScriptError> {
        let mut instance = RunnerScriptArtboard::new(
            &self.runtime,
            &self.artboards,
            self.artboard_index,
            Rc::clone(&self.render_state),
        )
        .map_err(|error| ScriptError::new(error.to_string()))?;
        instance.view_model = view_model;
        instance.bind_view_model();
        Ok(Box::new(instance))
    }

    fn advance(&mut self, seconds: f32) -> std::result::Result<bool, ScriptError> {
        self.bind_view_model();
        let mut changed = if let Some(state_machine) = self.state_machine.as_mut() {
            self.instance
                .advance_state_machine_instance(state_machine, seconds)
        } else {
            self.instance.advance_nested_artboards(seconds)
        };
        changed |= self
            .instance
            .advance_artboard_data_binds_with_elapsed(seconds);
        changed |= self.instance.update_pass();
        Ok(changed)
    }

    fn animation(
        &self,
        name: &str,
    ) -> std::result::Result<Option<nuxie_runtime::ScriptAnimation>, ScriptError> {
        Ok(nuxie_runtime::ScriptAnimation::named(&self.instance, name))
    }

    fn advance_animation(
        &mut self,
        animation: &mut nuxie_runtime::ScriptAnimation,
        seconds: f32,
    ) -> std::result::Result<bool, ScriptError> {
        Ok(animation.advance(&mut self.instance, seconds))
    }

    fn set_animation_time(
        &mut self,
        animation: &mut nuxie_runtime::ScriptAnimation,
        value: f32,
        mode: nuxie_runtime::ScriptAnimationTime,
    ) -> std::result::Result<(), ScriptError> {
        animation.set_time(&mut self.instance, value, mode);
        Ok(())
    }

    fn node(
        &self,
        name: &str,
    ) -> std::result::Result<Option<nuxie_runtime::ScriptNode>, ScriptError> {
        let graph = self.artboards.get(self.artboard_index).ok_or_else(|| {
            ScriptError::new(format!(
                "missing scripted artboard index {}",
                self.artboard_index
            ))
        })?;
        Ok(nuxie_runtime::script_node_for_artboard(
            &self.instance,
            graph,
            name,
        ))
    }

    fn draw(
        &mut self,
        factory: &mut dyn RenderFactory,
        renderer: &mut dyn RenderRenderer,
    ) -> std::result::Result<(), ScriptError> {
        let graph = self.artboards.get(self.artboard_index).ok_or_else(|| {
            ScriptError::new(format!(
                "missing scripted artboard index {}",
                self.artboard_index
            ))
        })?;
        let mut paint_cache = self
            .render_state
            .borrow_mut()
            .caches
            .remove(&self.render_instance_id)
            .unwrap_or_else(|| {
                preallocate_render_paint_cache_for_artboard_instance(
                    &self.runtime,
                    graph,
                    &self.artboards,
                    factory,
                )
            });
        let result = self
            .instance
            .prepare_static_artboard_tree_paints(
                &self.runtime,
                graph,
                &self.artboards,
                factory,
                &mut paint_cache,
                &mut self.path_cache,
            )
            .map_err(|error| ScriptError::new(error.to_string()))
            .and_then(|()| {
                let result = self
                    .instance
                    .draw_prepared_static_artboard_with_render_cache_and_origin(
                        &self.runtime,
                        graph,
                        &self.artboards,
                        factory,
                        renderer,
                        &mut paint_cache,
                        &mut self.path_cache,
                        self.frame_origin,
                    )
                    .map_err(|error| ScriptError::new(error.to_string()));
                result
            });
        self.render_state
            .borrow_mut()
            .caches
            .insert(self.render_instance_id, paint_cache);
        result
    }
}

#[cfg(feature = "scripting")]
fn initialize_scripted_drawables_and_realize(
    runtime: &RuntimeFile,
    artboard_index: usize,
    artboard: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    instance: &mut ArtboardInstance,
    factory: &mut dyn RenderFactory,
) -> Result<Option<Rc<RefCell<RunnerScriptArtboardRenderState>>>> {
    let state = initialize_scripted_drawables(
        runtime,
        artboard_index,
        artboard,
        artboards,
        instance,
        factory,
    )
    .context("failed to initialize scripted drawables")?;
    if let Some(state) = state.as_ref() {
        state
            .borrow_mut()
            .realize_pending(runtime, artboards, factory)
            .context("failed to allocate initialized script artboard paints")?;
    }
    Ok(state)
}

#[cfg(feature = "scripting")]
fn initialize_scripted_drawables(
    runtime: &RuntimeFile,
    artboard_index: usize,
    artboard: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    instance: &mut ArtboardInstance,
    factory: &mut dyn RenderFactory,
) -> Result<Option<Rc<RefCell<RunnerScriptArtboardRenderState>>>> {
    let script_assets = extract_script_assets(runtime);
    if script_assets.is_empty() {
        return Ok(None);
    }

    let render_state = Rc::new(RefCell::new(RunnerScriptArtboardRenderState::default()));
    let root_context_model = selected_script_view_model(runtime, artboard_index);
    initialize_scripted_drawables_for_artboard(
        runtime,
        artboard_index,
        artboard,
        artboards,
        instance,
        factory,
        &script_assets,
        Rc::clone(&render_state),
        root_context_model,
        Vec::new(),
        false,
    )?;
    Ok(Some(render_state))
}

#[cfg(feature = "scripting")]
fn initialize_nested_scripted_drawables(
    runtime: &RuntimeFile,
    root_artboard_index: usize,
    artboards: &[ArtboardGraph],
    instance: &mut ArtboardInstance,
    factory: &mut dyn RenderFactory,
    render_state: &Rc<RefCell<RunnerScriptArtboardRenderState>>,
) -> Result<()> {
    let script_assets = extract_script_assets(runtime);
    let root_context_model = selected_script_view_model(runtime, root_artboard_index);
    let mut context_models_by_depth = vec![root_context_model];
    let mut bound_nested_contexts = Vec::new();
    instance.try_visit_nested_artboard_instances_mut(
        &mut |depth, graph_global_id, child_instance| {
            let child_index = artboards
                .iter()
                .position(|candidate| candidate.global_id == graph_global_id)
                .with_context(|| format!("missing nested artboard graph {graph_global_id}"))?;
            context_models_by_depth.truncate(depth);
            let child_context_model = selected_script_view_model(runtime, child_index);
            let parent_context_models = context_models_by_depth
                .iter()
                .rev()
                .filter_map(Clone::clone)
                .collect();
            if !artboard_script_payloads_contain(
                runtime,
                &artboards[child_index],
                &script_assets,
                b"dataContext",
            ) {
                context_models_by_depth.push(child_context_model);
                return Ok::<(), anyhow::Error>(());
            }
            initialize_scripted_drawables_for_artboard(
                runtime,
                child_index,
                &artboards[child_index],
                artboards,
                child_instance,
                factory,
                &script_assets,
                Rc::clone(render_state),
                child_context_model.clone(),
                parent_context_models,
                true,
            )?;
            if let Some(model) = child_context_model.as_ref() {
                bound_nested_contexts.push((graph_global_id, model.clone()));
            }
            context_models_by_depth.push(child_context_model);
            Ok::<(), anyhow::Error>(())
        },
    )?;
    for (graph_global_id, model) in bound_nested_contexts {
        let owned = model.owned_instance();
        if let Some(snapshot) = owned.detached_snapshot() {
            instance.set_nested_script_owned_context_for_graph(graph_global_id, snapshot);
        }
    }
    instance.rebind_nested_script_owned_contexts(runtime);
    instance.update_pass();
    Ok(())
}

#[cfg(feature = "scripting")]
fn artboard_script_payloads_contain(
    runtime: &RuntimeFile,
    artboard: &ArtboardGraph,
    script_assets: &BTreeMap<u64, ExtractedScriptAsset>,
    marker: &[u8],
) -> bool {
    artboard.local_objects.iter().any(|local_object| {
        if !local_object
            .type_name
            .is_some_and(is_scripted_drawable_type)
        {
            return false;
        }
        let Some(script_asset_id) = runtime
            .object(local_object.global_id as usize)
            .and_then(|object| object.uint_property("scriptAssetId"))
        else {
            return false;
        };
        script_assets.get(&script_asset_id).is_some_and(|script| {
            script
                .payload
                .windows(marker.len())
                .any(|window| window == marker)
        })
    })
}

#[cfg(feature = "scripting")]
#[allow(clippy::too_many_arguments)]
fn initialize_scripted_drawables_for_artboard(
    runtime: &RuntimeFile,
    _artboard_index: usize,
    artboard: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    instance: &mut ArtboardInstance,
    factory: &mut dyn RenderFactory,
    script_assets: &BTreeMap<u64, ExtractedScriptAsset>,
    render_state: Rc<RefCell<RunnerScriptArtboardRenderState>>,
    context_model: Option<ScriptViewModel>,
    parent_context_models: Vec<ScriptViewModel>,
    script_context_is_bound: bool,
) -> Result<()> {
    let mut vm = ScriptVm::new();
    vm.set_view_models(nuxie_runtime::script_view_models(runtime));
    let bound_context_model = context_model.clone();
    vm.set_default_context_view_model_chain(context_model, parent_context_models);
    let mut host = NoopScriptHost;
    initialize_scripted_data_converters(
        runtime,
        instance,
        factory,
        script_assets,
        bound_context_model.as_ref(),
        &mut vm,
        &mut host,
    )?;
    for local_object in &artboard.local_objects {
        if !local_object
            .type_name
            .is_some_and(is_scripted_drawable_type)
        {
            continue;
        }
        let object = runtime
            .object(local_object.global_id as usize)
            .with_context(|| {
                format!("missing ScriptedDrawable global {}", local_object.global_id)
            })?;
        let script_asset_id = object.uint_property("scriptAssetId").unwrap_or(0);
        let script = script_assets.get(&script_asset_id).with_context(|| {
            format!(
                "ScriptedDrawable global {} references missing ScriptAsset id {}",
                local_object.global_id, script_asset_id
            )
        })?;
        let mut script_instance = vm
            .instantiate_script_with_factory(&script.name, &script.payload, &mut host, factory)
            .with_context(|| {
                format!(
                    "failed to instantiate ScriptAsset '{}' for ScriptedDrawable global {}",
                    script.name, local_object.global_id
                )
            })?;
        if !script_context_is_bound {
            script_instance
                .set_context_view_model(None)
                .context("failed to clear cold script context view model")?;
        }
        let defer_cold_hydration = scripted_object_has_view_model_input(
            runtime,
            artboard,
            artboards,
            local_object.local_id,
        );
        if !defer_cold_hydration {
            hydrate_script_inputs(
                runtime,
                artboard,
                artboards,
                local_object.local_id,
                script_instance.as_mut(),
                Rc::clone(&render_state),
            )?;
            if script_instance
                .has_method(ScriptMethod::Init)
                .context("failed to inspect script init method")?
            {
                script_instance
                    .call_method_with_factory(ScriptMethod::Init, &[], &mut host, factory)
                    .with_context(|| {
                        format!(
                            "script init failed for ScriptedDrawable global {}",
                            local_object.global_id
                        )
                    })?;
            }
        }
        if local_object.type_name == Some("ScriptedLayout")
            && script_instance
                .has_method(ScriptMethod::Resize)
                .context("failed to inspect script resize method")?
        {
            let artboard_object = runtime.object(artboard.global_id as usize);
            let width = artboard_object
                .and_then(|object| object.double_property("width"))
                .unwrap_or(0.0);
            let height = artboard_object
                .and_then(|object| object.double_property("height"))
                .unwrap_or(0.0);
            script_instance
                .call_method(
                    ScriptMethod::Resize,
                    &[ScriptValue::Vec2 {
                        x: width,
                        y: height,
                    }],
                    &mut host,
                )
                .with_context(|| {
                    format!(
                        "script resize failed for ScriptedLayout global {}",
                        local_object.global_id
                    )
                })?;
        }
        if local_object.type_name == Some("ScriptedPathEffect") {
            instance.set_script_path_effect_instance_for_global(
                local_object.global_id,
                script_instance,
            );
        } else {
            instance.set_script_instance_for_global(local_object.global_id, script_instance);
        }
    }

    if let Some(model) = bound_context_model {
        let owned = model.owned_instance();
        instance.bind_owned_view_model_artboard_context_handle(runtime, &owned);
        instance.advance_artboard_data_binds();
        instance.update_pass();
    }

    Ok(())
}

#[cfg(feature = "scripting")]
fn initialize_scripted_data_converters(
    runtime: &RuntimeFile,
    instance: &mut ArtboardInstance,
    factory: &mut dyn RenderFactory,
    script_assets: &BTreeMap<u64, ExtractedScriptAsset>,
    context_model: Option<&ScriptViewModel>,
    vm: &mut ScriptVm,
    host: &mut NoopScriptHost,
) -> Result<()> {
    for converter in runtime
        .data_converters()
        .into_iter()
        .filter(|converter| converter.type_name == "ScriptedDataConverter")
    {
        let script_asset_id = converter.uint_property("scriptAssetId").unwrap_or(0);
        let script = script_assets.get(&script_asset_id).with_context(|| {
            format!(
                "ScriptedDataConverter global {} references missing ScriptAsset id {}",
                converter.id, script_asset_id
            )
        })?;
        let mut script_instance = vm
            .instantiate_script_with_factory(&script.name, &script.payload, host, factory)
            .with_context(|| {
                format!(
                    "failed to instantiate ScriptAsset '{}' for ScriptedDataConverter global {}",
                    script.name, converter.id
                )
            })?;
        for input in scripted_data_converter_inputs(runtime, converter.id) {
            let Some(name) = input.string_property("name") else {
                continue;
            };
            let value = context_model
                .and_then(|model| {
                    let owned = model.owned_instance();
                    let root = owned.root_handle();
                    nuxie_runtime::bound_script_input_value(runtime, &root.borrow(), input)
                })
                .or_else(|| default_script_input_value(input));
            if let Some(value) = value {
                script_instance.set_input(name, value).with_context(|| {
                    format!(
                        "failed to hydrate input '{name}' for ScriptedDataConverter global {}",
                        converter.id
                    )
                })?;
            }
        }
        if script_instance
            .has_method(ScriptMethod::Init)
            .context("failed to inspect scripted converter init method")?
        {
            script_instance
                .call_method_with_factory(ScriptMethod::Init, &[], host, factory)
                .with_context(|| {
                    format!(
                        "script init failed for ScriptedDataConverter global {}",
                        converter.id
                    )
                })?;
        }
        instance.set_scripted_data_converter_instance_for_global(converter.id, script_instance);
    }
    Ok(())
}

#[cfg(feature = "scripting")]
fn scripted_data_converter_inputs(
    runtime: &RuntimeFile,
    converter_global_id: u32,
) -> Vec<&nuxie_binary::RuntimeObject> {
    ((converter_global_id as usize + 1)..runtime.object_count())
        .map_while(|global_id| {
            let object = runtime.object(global_id)?;
            (object.type_name.starts_with("ScriptInput")
                || object.type_name.starts_with("DataBind"))
            .then_some(object)
        })
        .filter(|object| object.type_name.starts_with("ScriptInput"))
        .collect()
}

#[cfg(feature = "scripting")]
fn default_script_input_value(input: &nuxie_binary::RuntimeObject) -> Option<ScriptValue> {
    match input.type_name {
        "ScriptInputBoolean" => input.bool_property("propertyValue").map(ScriptValue::Bool),
        "ScriptInputColor" => input
            .color_property("propertyValue")
            .map(|value| ScriptValue::Number(value as f64)),
        "ScriptInputNumber" => input
            .double_property("propertyValue")
            .map(|value| ScriptValue::Number(f64::from(value))),
        "ScriptInputString" => input
            .string_property("propertyValue")
            .map(|value| ScriptValue::String(value.to_owned())),
        _ => None,
    }
}

#[cfg(feature = "scripting")]
fn selected_script_view_model(
    runtime: &RuntimeFile,
    artboard_index: usize,
) -> Option<ScriptViewModel> {
    selected_artboard_owned_view_model_context(runtime, artboard_index)
        .as_ref()
        .and_then(|context| nuxie_runtime::script_view_model_from_owned(runtime, context))
}

#[cfg(feature = "scripting")]
fn scripted_object_has_view_model_input(
    runtime: &RuntimeFile,
    artboard: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    scripted_local_id: usize,
) -> bool {
    artboard_object_range(runtime, artboard, artboards).any(|global_id| {
        runtime.object(global_id).is_some_and(|object| {
            object.type_name == "ScriptInputViewModelProperty"
                && object.uint_property("parentId") == Some(scripted_local_id as u64)
        })
    })
}

#[cfg(feature = "scripting")]
fn extract_script_assets(runtime: &RuntimeFile) -> BTreeMap<u64, ExtractedScriptAsset> {
    let mut scripts = BTreeMap::new();
    let script_asset_indices_by_global = runtime
        .file_assets()
        .into_iter()
        .enumerate()
        .filter_map(|(index, object)| {
            (object.type_name == "ScriptAsset").then_some((object.id, index as u64))
        })
        .collect::<BTreeMap<_, _>>();
    let file_asset_globals = runtime
        .file_assets()
        .into_iter()
        .map(|object| object.id)
        .collect::<BTreeSet<_>>();
    let mut latest_script: Option<(u64, String)> = None;

    for id in 0..runtime.object_count() {
        let Some(object) = runtime.object(id) else {
            continue;
        };
        match object.type_name {
            "ScriptAsset" => {
                latest_script = Some((
                    script_asset_indices_by_global
                        .get(&object.id)
                        .copied()
                        .or_else(|| object.uint_property("assetId"))
                        .unwrap_or(0),
                    object
                        .string_property("name")
                        .unwrap_or("unnamed")
                        .to_owned(),
                ));
            }
            "FileAssetContents" => {
                if let Some((asset_id, name)) = latest_script.take()
                    && let Some(payload) = object.bytes_property("bytes")
                {
                    scripts.insert(
                        asset_id,
                        ExtractedScriptAsset {
                            name,
                            payload: payload.to_vec(),
                        },
                    );
                }
            }
            _ if file_asset_globals.contains(&object.id) => {
                latest_script = None;
            }
            _ => {}
        }
    }

    scripts
}

#[cfg(feature = "scripting")]
fn hydrate_script_inputs(
    runtime: &RuntimeFile,
    artboard: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    scripted_local_id: usize,
    script_instance: &mut dyn nuxie_runtime::ScriptInstance,
    render_state: Rc<RefCell<RunnerScriptArtboardRenderState>>,
) -> Result<()> {
    let object_range = artboard_object_range(runtime, artboard, artboards);
    for global_id in object_range {
        let Some(object) = runtime.object(global_id) else {
            continue;
        };
        let type_name = object.type_name;
        if !type_name.starts_with("ScriptInput") {
            continue;
        }
        if object.uint_property("parentId") != Some(scripted_local_id as u64) {
            continue;
        }
        let Some(name) = object.string_property("name") else {
            continue;
        };
        if type_name == "ScriptInputArtboard" {
            let Some(artboard_index) = object.uint_property("artboardId") else {
                continue;
            };
            let artboard = RunnerScriptArtboard::new(
                runtime,
                artboards,
                artboard_index as usize,
                Rc::clone(&render_state),
            )?;
            script_instance
                .set_artboard_input(name, Box::new(artboard))
                .with_context(|| format!("failed to hydrate artboard script input '{name}'"))?;
            continue;
        }
        let value = match type_name {
            "ScriptInputBoolean" => object.bool_property("propertyValue").map(ScriptValue::Bool),
            "ScriptInputColor" => object
                .color_property("propertyValue")
                .map(|value| ScriptValue::Number(value as f64)),
            "ScriptInputNumber" => object
                .double_property("propertyValue")
                .map(|value| ScriptValue::Number(f64::from(value))),
            "ScriptInputString" => object
                .string_property("propertyValue")
                .map(|value| ScriptValue::String(value.to_owned())),
            _ => None,
        };
        if let Some(value) = value {
            script_instance
                .set_input(name, value)
                .with_context(|| format!("failed to hydrate script input '{name}'"))?;
        }
    }

    Ok(())
}

#[cfg(feature = "scripting")]
fn rebind_scripted_drawables(
    runtime: &RuntimeFile,
    artboard: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    instance: &mut ArtboardInstance,
    render_state: &Rc<RefCell<RunnerScriptArtboardRenderState>>,
    factory: &mut dyn RenderFactory,
    owned_view_model_context: Option<&RuntimeOwnedViewModelHandle>,
) -> Result<()> {
    let context_view_model = owned_view_model_context
        .and_then(|context| nuxie_runtime::script_view_model_from_owned(runtime, context));
    let owned_view_model_context = owned_view_model_context
        .map(|context| RuntimeOwnedViewModelContextHandle::root(runtime, context.clone()));
    instance
        .set_script_context_view_model(context_view_model)
        .context("failed to bind scripted context view model")?;
    for local_object in &artboard.local_objects {
        if !local_object
            .type_name
            .is_some_and(is_scripted_drawable_type)
        {
            continue;
        }
        rehydrate_script_inputs(
            runtime,
            artboard,
            artboards,
            local_object.local_id,
            local_object.global_id,
            instance,
            Rc::clone(render_state),
            owned_view_model_context.as_ref(),
        )?;
        if owned_view_model_context.is_some() {
            instance
                .reinitialize_script_instance_with_factory(local_object.global_id, factory)
                .context("scripted drawable rebind init failed")?;
        }
    }
    if owned_view_model_context.is_none() {
        instance
            .update_script_instances()
            .context("scripted drawable rebind update failed")?;
    }
    render_state
        .borrow_mut()
        .realize_pending(runtime, artboards, factory)
        .context("failed to allocate rebound script artboard paints")
}

#[cfg(feature = "scripting")]
fn rehydrate_script_inputs(
    runtime: &RuntimeFile,
    artboard: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    scripted_local_id: usize,
    scripted_global_id: u32,
    instance: &mut ArtboardInstance,
    render_state: Rc<RefCell<RunnerScriptArtboardRenderState>>,
    owned_view_model_context: Option<&RuntimeOwnedViewModelContextHandle>,
) -> Result<()> {
    for global_id in artboard_object_range(runtime, artboard, artboards) {
        let Some(object) = runtime.object(global_id) else {
            continue;
        };
        let type_name = object.type_name;
        if !type_name.starts_with("ScriptInput")
            || object.uint_property("parentId") != Some(scripted_local_id as u64)
        {
            continue;
        }
        let Some(name) = object.string_property("name") else {
            continue;
        };
        if type_name == "ScriptInputArtboard" {
            let Some(artboard_index) = object.uint_property("artboardId") else {
                continue;
            };
            let artboard = RunnerScriptArtboard::new(
                runtime,
                artboards,
                artboard_index as usize,
                Rc::clone(&render_state),
            )?;
            instance
                .set_script_artboard_input_for_global(scripted_global_id, name, Box::new(artboard))
                .with_context(|| format!("failed to rebind artboard script input '{name}'"))?;
            continue;
        }
        if type_name == "ScriptInputViewModelProperty" {
            let Some(view_model) = owned_view_model_context.and_then(|context| {
                nuxie_runtime::bound_script_view_model_from_owned_context(runtime, context, object)
            }) else {
                continue;
            };
            instance
                .set_script_view_model_input_for_global(scripted_global_id, name, view_model)
                .with_context(|| format!("failed to rebind view-model script input '{name}'"))?;
            continue;
        }
        let value = owned_view_model_context
            .and_then(|context| {
                let root = context.root_handle();
                nuxie_runtime::bound_script_input_value(runtime, &root.borrow(), object)
            })
            .or_else(|| match type_name {
                "ScriptInputBoolean" => {
                    object.bool_property("propertyValue").map(ScriptValue::Bool)
                }
                "ScriptInputColor" => object
                    .color_property("propertyValue")
                    .map(|value| ScriptValue::Number(value as f64)),
                "ScriptInputNumber" => object
                    .double_property("propertyValue")
                    .map(|value| ScriptValue::Number(f64::from(value))),
                "ScriptInputString" => object
                    .string_property("propertyValue")
                    .map(|value| ScriptValue::String(value.to_owned())),
                _ => None,
            });
        if let Some(value) = value {
            instance
                .set_script_input_for_global(scripted_global_id, name, value)
                .with_context(|| format!("failed to rebind script input '{name}'"))?;
        }
    }
    Ok(())
}

#[cfg(feature = "scripting")]
fn mark_scripted_drawables_for_update(artboard: &ArtboardGraph, instance: &mut ArtboardInstance) {
    for local_object in &artboard.local_objects {
        if local_object
            .type_name
            .is_some_and(is_scripted_drawable_type)
        {
            instance.mark_script_update_for_global(local_object.global_id);
        }
    }
}

#[cfg(feature = "scripting")]
fn is_scripted_drawable_type(type_name: &str) -> bool {
    matches!(
        type_name,
        "ScriptedDrawable" | "ScriptedLayout" | "ScriptedPathEffect"
    )
}

#[cfg(feature = "scripting")]
fn artboard_object_range(
    runtime: &RuntimeFile,
    artboard: &ArtboardGraph,
    artboards: &[ArtboardGraph],
) -> std::ops::Range<usize> {
    let start = artboard.global_id as usize;
    let end = artboards
        .iter()
        .filter_map(|candidate| {
            (candidate.global_id > artboard.global_id).then_some(candidate.global_id as usize)
        })
        .min()
        .unwrap_or_else(|| runtime.object_count());
    start..end
}

fn ensure_static_draw_supported(
    runtime: &RuntimeFile,
    graph: &GraphFile,
    artboard: &ArtboardGraph,
    has_input_events: bool,
) -> Result<()> {
    let mut visiting = BTreeSet::new();
    ensure_static_draw_supported_for_artboard(
        runtime,
        graph,
        artboard,
        &mut visiting,
        false,
        has_input_events,
    )
}

fn ensure_static_draw_supported_for_artboard(
    runtime: &RuntimeFile,
    graph: &GraphFile,
    artboard: &ArtboardGraph,
    visiting: &mut BTreeSet<u32>,
    is_nested_child: bool,
    has_input_events: bool,
) -> Result<()> {
    if !visiting.insert(artboard.global_id) {
        return Ok(());
    }

    if let Some(nested) = artboard.nested_artboards.iter().find(|nested| {
        !matches!(
            nested.type_name,
            "NestedArtboard" | "NestedArtboardLayout" | "NestedArtboardLeaf"
        )
    }) {
        bail!(
            "unsupported: nested artboards in Rust golden runner ({})",
            nested.type_name
        );
    }

    if let Some((type_name, global_id)) = nested_stateful_view_model_object(runtime, artboard) {
        bail!(
            "unsupported: data-binding-nested-stateful-view-model in Rust golden runner ({type_name} global {global_id})"
        );
    }

    if is_nested_child {
        if let Some(data_bind) = artboard
            .data_binds
            .iter()
            .find(|data_bind| !nested_child_data_bind_supported(data_bind))
        {
            if nested_child_data_bind_is_text(data_bind) {
                bail!(
                    "unsupported: text in Rust golden runner (nested child data bind global {} target {:?})",
                    data_bind.global_id,
                    data_bind.target_type_name
                );
            }
            let unsupported_feature = nested_child_data_bind_unsupported_feature(data_bind);
            bail!(
                "unsupported: {unsupported_feature} in Rust golden runner (data bind global {} target {:?})",
                data_bind.global_id,
                data_bind.target_type_name
            );
        }
        if let Some(focus_object) = artboard
            .local_objects
            .iter()
            .find(|object| object.type_name == Some("FocusData") && has_input_events)
        {
            bail!(
                "unsupported: focus-data in Rust golden runner (nested child global {})",
                focus_object.global_id
            );
        }
    }

    if let Some(data_bind) = nested_artboard_host_control_data_bind(graph, artboard) {
        bail!(
            "unsupported: data-binding-nested-host in Rust golden runner (data bind global {} property key {} flags {} converter {:?})",
            data_bind.global_id,
            data_bind.property_key,
            data_bind.flags,
            data_bind.converter_type_name
        );
    }

    if let Some(child_global) =
        nested_unsupported_listener_propagation_child_global(runtime, graph, artboard)
    {
        bail!(
            "unsupported: nested artboards in Rust golden runner (listener propagation to nested child global {child_global})"
        );
    }

    if has_input_events
        && runtime_has_type(runtime, "ListenerAlignTarget")
        && artboard_has_recursive_nested_artboard(graph, artboard, &mut BTreeSet::new())
    {
        bail!(
            "unsupported: nested artboards in Rust golden runner (recursive nested listener align target)"
        );
    }

    if let Some(global_id) = unsupported_image_global(runtime, graph, artboard) {
        bail!("unsupported: images in Rust golden runner (global {global_id})");
    }

    if let Some(global_id) = taffy_refused_layout_dependent_draw(runtime, graph, artboard)? {
        bail!(
            "unsupported: layout-component-paint in Rust golden runner (Taffy refused layout tree for global {global_id})"
        );
    }

    if let Some(container) = unsupported_layout_component_paint(runtime, artboard) {
        bail!(
            "unsupported: layout-component-paint in Rust golden runner (global {})",
            container.global_id
        );
    }

    if let Some(scroll_constraint) =
        unsupported_scroll_constraint_global(runtime, graph, artboard, has_input_events)
    {
        bail!(
            "unsupported: scroll-constraints in Rust golden runner (global {})",
            scroll_constraint
        );
    }

    if let Some((text, reason)) = artboard
        .local_objects
        .iter()
        .filter(|object| object.type_name == Some("Text"))
        .find_map(|object| {
            static_text_support_error(runtime, artboard, object.local_id)
                .map(|reason| (object, reason))
        })
    {
        let feature = unsupported_static_text_feature(&reason);
        bail!(
            "unsupported: {feature} in Rust golden runner (global {}, {reason})",
            text.global_id
        );
    }

    if let Some(data_bind) = artboard
        .data_binds
        .iter()
        .find(|data_bind| {
            data_bind.target_type_name == Some("SolidColor")
                && !solid_color_data_bind_supported(data_bind)
        })
        .filter(|_| !is_nested_child)
    {
        bail!(
            "unsupported: data-binding-color in Rust golden runner (data bind global {} target global {:?})",
            data_bind.global_id,
            data_bind.target_global
        );
    }

    if let Some(data_bind) = artboard.data_binds.iter().find(|data_bind| {
        data_bind.target_type_name == Some("CustomPropertyEnum")
            && !custom_property_enum_data_bind_supported(data_bind)
    }) {
        bail!(
            "unsupported: data-binding-custom-property-enum in Rust golden runner (data bind global {} target global {:?})",
            data_bind.global_id,
            data_bind.target_global
        );
    }

    if let Some(scripted_object) = artboard
        .state_machines
        .iter()
        .flat_map(|state_machine| &state_machine.scripted_objects)
        .find(|scripted_object| scripted_object.type_name == "ScriptedTransitionCondition")
    {
        bail!(
            "unsupported: scripted-transition-condition in Rust golden runner (global {})",
            scripted_object.global_id
        );
    }

    if artboard
        .local_objects
        .iter()
        .any(|object| object.type_name == Some("ScriptedPathEffect"))
        && runtime.file_assets().iter().any(|asset| {
            asset.type_name == "ScriptAsset"
                && asset.string_property("name") == Some("FlipPathEffect")
        })
    {
        bail!("unsupported: script-path-commands in Rust golden runner");
    }

    if let Some((constraint_type, global_id)) = artboard.local_objects.iter().find_map(|object| {
        let type_name = object.type_name?;
        (type_name.ends_with("Constraint")
            && type_name != "DistanceConstraint"
            && type_name != "TranslationConstraint"
            && type_name != "RotationConstraint"
            && type_name != "ScaleConstraint"
            && type_name != "TransformConstraint"
            && type_name != "FollowPathConstraint"
            && type_name != "ListFollowPathConstraint"
            && type_name != "ScrollConstraint"
            && type_name != "IKConstraint")
            .then_some((type_name, object.global_id))
    }) {
        bail!(
            "unsupported: constraints in Rust golden runner ({constraint_type} global {global_id})"
        );
    }

    for referenced_artboard_global in artboard
        .sorted_drawable_order
        .iter()
        .filter_map(|drawable| drawable.referenced_artboard_global)
    {
        let child_artboard = graph
            .artboards
            .iter()
            .find(|artboard| artboard.global_id == referenced_artboard_global)
            .with_context(|| {
                format!("missing nested artboard graph for global {referenced_artboard_global}")
            })?;
        ensure_static_draw_supported_for_artboard(
            runtime,
            graph,
            child_artboard,
            visiting,
            true,
            has_input_events,
        )?;
    }

    Ok(())
}

fn unsupported_layout_component_paint<'a>(
    runtime: &RuntimeFile,
    artboard: &'a ArtboardGraph,
) -> Option<&'a ShapePaintContainerNode> {
    artboard.shape_paint_containers.iter().find(|container| {
        container.type_name == "LayoutComponent"
            && !container.paints.is_empty()
            && !layout_component_paint_supported(runtime, artboard, container)
    })
}

fn taffy_refused_layout_dependent_draw(
    runtime: &RuntimeFile,
    graph: &GraphFile,
    artboard: &ArtboardGraph,
) -> Result<Option<u32>> {
    let Some(global_id) = first_layout_dependent_draw_global(artboard) else {
        return Ok(None);
    };

    let instance = ArtboardInstance::from_graph_with_artboards(runtime, artboard, &graph.artboards)
        .context("failed to instantiate artboard for Taffy layout support check")?;
    if instance
        .debug_taffy_layout_bounds_report(runtime, artboard)
        .is_none()
    {
        return Ok(Some(global_id));
    }

    Ok(None)
}

fn first_layout_dependent_draw_global(artboard: &ArtboardGraph) -> Option<u32> {
    artboard
        .shape_paint_containers
        .iter()
        .find(|container| container.type_name == "LayoutComponent" && !container.paints.is_empty())
        .map(|container| container.global_id)
        .or_else(|| {
            artboard.sorted_drawable_order.iter().find_map(|drawable| {
                drawable
                    .layout_global
                    .or(drawable.global_id)
                    .filter(|_| layout_dependent_drawable(artboard, drawable))
            })
        })
}

fn layout_dependent_drawable(
    artboard: &ArtboardGraph,
    drawable: &nuxie_graph::SortedDrawableNode,
) -> bool {
    if drawable.layout_local.is_some() {
        return true;
    }
    let Some(local_id) = drawable.local_id else {
        return false;
    };
    component_parent_chain_has_layout_component(artboard, local_id)
}

fn component_parent_chain_has_layout_component(
    artboard: &ArtboardGraph,
    mut local_id: usize,
) -> bool {
    while let Some(component) = artboard
        .components
        .iter()
        .find(|component| component.local_id == local_id)
    {
        if component.type_name == "LayoutComponent" {
            return true;
        }
        let Some(parent_local) = component.parent_local else {
            return false;
        };
        if parent_local == local_id {
            return false;
        }
        local_id = parent_local;
    }
    false
}

fn unsupported_scroll_constraint_global(
    runtime: &RuntimeFile,
    graph: &GraphFile,
    artboard: &ArtboardGraph,
    has_input_events: bool,
) -> Option<u32> {
    for scroll_constraint in artboard
        .local_objects
        .iter()
        .filter(|object| object.type_name == Some("ScrollConstraint"))
    {
        if has_input_events
            || !passive_initial_scroll_constraint_supported(
                runtime,
                graph,
                artboard,
                scroll_constraint.local_id,
                scroll_constraint.global_id,
            )
        {
            return Some(scroll_constraint.global_id);
        }
    }
    None
}

fn passive_initial_scroll_constraint_supported(
    runtime: &RuntimeFile,
    graph: &GraphFile,
    artboard: &ArtboardGraph,
    scroll_local: usize,
    scroll_global: u32,
) -> bool {
    // C++ scroll is implementation-defined runtime behavior. This runner admits
    // only the passive initial-state slice ported from
    // `src/constraints/scrolling/scroll_constraint.cpp`: no pointer/input
    // driving, no authored offset/index, no listener targeting the constraint,
    // and a coherent Taffy layout snapshot for the scroll tree. Virtualized
    // and infinite metadata is admitted only while the constraint is still at
    // rest, so the sample-0 render path stays layout-only.
    let Some(scroll) = runtime.object(scroll_global as usize) else {
        return false;
    };
    if scroll.double_property("scrollOffsetX").unwrap_or(0.0) != 0.0
        || scroll.double_property("scrollOffsetY").unwrap_or(0.0) != 0.0
        || scroll.double_property("scrollPercentX").unwrap_or(0.0) != 0.0
        || scroll.double_property("scrollPercentY").unwrap_or(0.0) != 0.0
        || scroll.double_property("scrollIndex").unwrap_or(0.0) != 0.0
    {
        return false;
    }

    if artboard.state_machines.iter().any(|state_machine| {
        state_machine.listeners.iter().any(|listener| {
            usize::try_from(listener.target_id)
                .ok()
                .is_some_and(|target| target == scroll_local)
        })
    }) {
        return false;
    }

    if !artboard
        .layout_constraint_registrations
        .iter()
        .any(|registration| registration.constraint_local == scroll_local)
    {
        return false;
    }

    let Ok(instance) =
        ArtboardInstance::from_graph_with_artboards(runtime, artboard, &graph.artboards)
    else {
        return false;
    };
    instance
        .debug_taffy_layout_bounds_report(runtime, artboard)
        .is_some()
}

fn layout_component_paint_supported(
    runtime: &RuntimeFile,
    artboard: &ArtboardGraph,
    container: &ShapePaintContainerNode,
) -> bool {
    simple_flex_layout_component_paint_supported(runtime, artboard, container)
        || simple_root_layout_component_paint_supported(runtime, artboard, container)
        || root_layout_component_paint_supported(runtime, artboard, container)
        || clipped_nested_empty_list_layout_component_paint_supported(runtime, artboard, container)
}

fn root_layout_component_paint_supported(
    runtime: &RuntimeFile,
    artboard: &ArtboardGraph,
    container: &ShapePaintContainerNode,
) -> bool {
    let Some(component) = artboard
        .components
        .iter()
        .find(|component| component.local_id == container.local_id)
    else {
        return false;
    };
    if component.parent_local != Some(0) {
        return false;
    }
    let Some(layout_object) = runtime.object(container.global_id as usize) else {
        return false;
    };
    if layout_object.double_property("width").unwrap_or(0.0) <= 0.0
        || layout_object.double_property("height").unwrap_or(0.0) <= 0.0
    {
        return false;
    }
    if layout_style_object(runtime, artboard, layout_object).is_none() {
        return false;
    }

    container
        .paints
        .iter()
        .all(root_layout_background_paint_supported)
}

fn simple_root_layout_component_paint_supported(
    runtime: &RuntimeFile,
    artboard: &ArtboardGraph,
    container: &ShapePaintContainerNode,
) -> bool {
    let Some(component) = artboard
        .components
        .iter()
        .find(|component| component.local_id == container.local_id)
    else {
        return false;
    };
    if component.parent_local != Some(0) {
        return false;
    }

    let Some(layout_object) = runtime.object(container.global_id as usize) else {
        return false;
    };
    if layout_object.bool_property("clip").unwrap_or(false) {
        return false;
    }

    let Some(artboard_object) = runtime.object(artboard.global_id as usize) else {
        return false;
    };
    let width = layout_object.double_property("width").unwrap_or(0.0);
    let height = layout_object.double_property("height").unwrap_or(0.0);
    let artboard_width = artboard_object.double_property("width").unwrap_or(0.0);
    let artboard_height = artboard_object.double_property("height").unwrap_or(0.0);
    let origin_x = artboard_object.double_property("originX").unwrap_or(0.0);
    let origin_y = artboard_object.double_property("originY").unwrap_or(0.0);
    if !nearly_equal(width, artboard_width)
        || !nearly_equal(height, artboard_height)
        || !nearly_equal(origin_x, 0.0)
        || !nearly_equal(origin_y, 0.0)
    {
        return false;
    }

    let Some(style_local) = layout_object
        .uint_property("styleId")
        .and_then(|style| usize::try_from(style).ok())
    else {
        return false;
    };
    let Some(style_global) = artboard
        .local_objects
        .iter()
        .find(|object| {
            object.local_id == style_local && object.type_name == Some("LayoutComponentStyle")
        })
        .map(|object| object.global_id)
    else {
        return false;
    };
    let Some(style_object) = runtime.object(style_global as usize) else {
        return false;
    };

    const LAYOUT_SCALE_TYPE_FILL: u64 = 1;
    if style_object
        .uint_property("layoutWidthScaleType")
        .unwrap_or(0)
        != LAYOUT_SCALE_TYPE_FILL
        || style_object
            .uint_property("layoutHeightScaleType")
            .unwrap_or(0)
            != LAYOUT_SCALE_TYPE_FILL
    {
        return false;
    }
    if [
        "cornerRadiusTL",
        "cornerRadiusTR",
        "cornerRadiusBL",
        "cornerRadiusBR",
    ]
    .into_iter()
    .any(|property| !nearly_equal(style_object.double_property(property).unwrap_or(0.0), 0.0))
    {
        return false;
    }

    container
        .paints
        .iter()
        .all(simple_layout_background_paint_supported)
}

fn clipped_nested_empty_list_layout_component_paint_supported(
    runtime: &RuntimeFile,
    artboard: &ArtboardGraph,
    container: &ShapePaintContainerNode,
) -> bool {
    let Some(component) = artboard
        .components
        .iter()
        .find(|component| component.local_id == container.local_id)
    else {
        return false;
    };
    let Some(parent_local) = component.parent_local else {
        return false;
    };
    let Some(parent) = artboard
        .components
        .iter()
        .find(|component| component.local_id == parent_local)
    else {
        return false;
    };
    if parent.type_name != "LayoutComponent" || parent.parent_local != Some(0) {
        return false;
    }

    let Some(layout_object) = runtime.object(container.global_id as usize) else {
        return false;
    };
    let Some(parent_object) = runtime.object(parent.global_id as usize) else {
        return false;
    };
    if !layout_object.bool_property("clip").unwrap_or(false)
        || !parent_object.bool_property("clip").unwrap_or(false)
    {
        return false;
    }

    let Some(artboard_object) = runtime.object(artboard.global_id as usize) else {
        return false;
    };
    if !nearly_equal(
        parent_object.double_property("width").unwrap_or(0.0),
        artboard_object.double_property("width").unwrap_or(0.0),
    ) || !nearly_equal(
        parent_object.double_property("height").unwrap_or(0.0),
        artboard_object.double_property("height").unwrap_or(0.0),
    ) {
        return false;
    }

    let Some(style_object) = layout_style_object(runtime, artboard, layout_object) else {
        return false;
    };
    const LAYOUT_SCALE_TYPE_FILL: u64 = 1;
    const LAYOUT_SCALE_TYPE_HUG: u64 = 2;
    let width_scale = style_object
        .uint_property("layoutWidthScaleType")
        .unwrap_or(0);
    let height_scale = style_object
        .uint_property("layoutHeightScaleType")
        .unwrap_or(0);
    if !matches!(
        (width_scale, height_scale),
        (LAYOUT_SCALE_TYPE_FILL, LAYOUT_SCALE_TYPE_HUG)
            | (LAYOUT_SCALE_TYPE_HUG, LAYOUT_SCALE_TYPE_FILL)
    ) {
        return false;
    }
    if !style_object
        .bool_property("intrinsicallySizedValue")
        .unwrap_or(false)
    {
        return false;
    }
    if !layout_style_has_zero_corners(style_object) {
        return false;
    }

    let list_children = component
        .children
        .iter()
        .filter_map(|child_local| {
            artboard
                .components
                .iter()
                .find(|component| component.local_id == *child_local)
        })
        .filter(|child| child.type_name == "ArtboardComponentList")
        .collect::<Vec<_>>();
    let [list_child] = list_children.as_slice() else {
        return false;
    };
    if artboard
        .component_lists
        .iter()
        .find(|list| list.local_id == list_child.local_id)
        .is_none_or(|list| !list.map_rules.is_empty())
    {
        return false;
    }

    let override_matches_fill_axis = list_child.children.iter().any(|override_local| {
        let Some(override_object) = runtime_object_for_local(
            runtime,
            artboard,
            *override_local,
            "ArtboardComponentListOverride",
        ) else {
            return false;
        };
        (width_scale == LAYOUT_SCALE_TYPE_FILL
            && override_object
                .uint_property("instanceWidthScaleType")
                .unwrap_or(0)
                == LAYOUT_SCALE_TYPE_FILL)
            || (height_scale == LAYOUT_SCALE_TYPE_FILL
                && override_object
                    .uint_property("instanceHeightScaleType")
                    .unwrap_or(0)
                    == LAYOUT_SCALE_TYPE_FILL)
    });
    if !override_matches_fill_axis {
        return false;
    }

    container
        .paints
        .iter()
        .all(simple_layout_background_paint_supported)
}

fn simple_flex_layout_component_paint_supported(
    runtime: &RuntimeFile,
    artboard: &ArtboardGraph,
    container: &ShapePaintContainerNode,
) -> bool {
    let Some(component) = artboard
        .components
        .iter()
        .find(|component| component.local_id == container.local_id)
    else {
        return false;
    };
    if component.type_name != "LayoutComponent" {
        return false;
    }
    let Some(parent_local) = component.parent_local else {
        return false;
    };
    let Some(parent) = artboard
        .components
        .iter()
        .find(|component| component.local_id == parent_local)
    else {
        return false;
    };
    if !matches!(parent.type_name, "Artboard" | "LayoutComponent") {
        return false;
    }

    let Some(layout_object) = runtime.object(container.global_id as usize) else {
        return false;
    };
    if layout_style_object(runtime, artboard, layout_object).is_none() {
        return false;
    }

    let Some(parent_object) = runtime.object(parent.global_id as usize) else {
        return false;
    };
    let Some(parent_style) = layout_style_object(runtime, artboard, parent_object) else {
        return false;
    };
    let parent_direction = parent_style
        .uint_property("flexDirectionValue")
        .unwrap_or(2);
    if !matches!(parent_direction, 0 | 2) {
        return false;
    }
    if !simple_flex_layout_spacing_supported(parent_style, parent_direction == 2) {
        return false;
    }

    let layout_children = parent
        .children
        .iter()
        .filter_map(|child_local| {
            artboard
                .components
                .iter()
                .find(|component| component.local_id == *child_local)
        })
        .filter(|child| child.type_name == "LayoutComponent")
        .collect::<Vec<_>>();
    if !layout_children
        .iter()
        .any(|child| child.local_id == component.local_id)
    {
        return false;
    }
    if !layout_children.iter().all(|child| {
        simple_flex_layout_child_supported(runtime, artboard, child, parent_direction == 2)
    }) {
        return false;
    }

    container
        .paints
        .iter()
        .all(simple_layout_background_paint_supported)
}

fn simple_flex_layout_child_supported(
    runtime: &RuntimeFile,
    artboard: &ArtboardGraph,
    child: &nuxie_graph::ComponentNode,
    parent_is_row: bool,
) -> bool {
    let Some(child_object) = runtime.object(child.global_id as usize) else {
        return false;
    };
    let Some(style_object) = layout_style_object(runtime, artboard, child_object) else {
        return false;
    };
    simple_flex_axis_supported(style_object, parent_is_row)
        && simple_flex_axis_supported(style_object, !parent_is_row)
}

fn simple_flex_axis_supported(style_object: &RuntimeObject, width_axis: bool) -> bool {
    let scale_property = if width_axis {
        "layoutWidthScaleType"
    } else {
        "layoutHeightScaleType"
    };
    let scale = style_object.uint_property(scale_property).unwrap_or(0);
    match scale {
        0 => simple_flex_dimension_unit_supported(style_object, width_axis),
        1 | 2 => true,
        _ => false,
    }
}

fn simple_flex_dimension_unit_supported(style_object: &RuntimeObject, width_axis: bool) -> bool {
    let unit_property = if width_axis {
        "widthUnitsValue"
    } else {
        "heightUnitsValue"
    };
    matches!(
        style_object.uint_property(unit_property).unwrap_or(1),
        0 | 1 | 2 | 3
    )
}

fn simple_flex_layout_spacing_supported(style_object: &RuntimeObject, parent_is_row: bool) -> bool {
    let gap_units = if parent_is_row {
        "gapHorizontalUnitsValue"
    } else {
        "gapVerticalUnitsValue"
    };
    [
        "paddingLeftUnitsValue",
        "paddingRightUnitsValue",
        "paddingTopUnitsValue",
        "paddingBottomUnitsValue",
        gap_units,
    ]
    .into_iter()
    .all(|property| {
        matches!(
            style_object.uint_property(property).unwrap_or(0),
            0 | 1 | 2 | 3
        )
    })
}

fn layout_style_object<'a>(
    runtime: &'a RuntimeFile,
    artboard: &ArtboardGraph,
    layout_object: &RuntimeObject,
) -> Option<&'a RuntimeObject> {
    let style_local = layout_object
        .uint_property("styleId")
        .and_then(|style| usize::try_from(style).ok())?;
    runtime_object_for_local(runtime, artboard, style_local, "LayoutComponentStyle")
}

fn runtime_object_for_local<'a>(
    runtime: &'a RuntimeFile,
    artboard: &ArtboardGraph,
    local_id: usize,
    type_name: &'static str,
) -> Option<&'a RuntimeObject> {
    let global_id = artboard
        .local_objects
        .iter()
        .find(|object| object.local_id == local_id && object.type_name == Some(type_name))?
        .global_id;
    runtime.object(global_id as usize)
}

fn layout_style_has_zero_corners(style_object: &RuntimeObject) -> bool {
    [
        "cornerRadiusTL",
        "cornerRadiusTR",
        "cornerRadiusBL",
        "cornerRadiusBR",
    ]
    .into_iter()
    .all(|property| nearly_equal(style_object.double_property(property).unwrap_or(0.0), 0.0))
}

fn root_layout_background_paint_supported(paint: &nuxie_graph::ShapePaintNode) -> bool {
    paint.is_visible
        && paint.paint_type == ShapePaintKind::Fill
        && matches!(
            paint.path_kind,
            Some(ShapePaintPathKind::Local | ShapePaintPathKind::LocalClockwise)
        )
        && matches!(
            paint.paint_state.as_ref(),
            Some(
                ShapePaintStateNode::SolidColor { .. }
                    | ShapePaintStateNode::LinearGradient { .. }
                    | ShapePaintStateNode::RadialGradient { .. }
            )
        )
        && paint.effects.is_empty()
}

fn simple_layout_background_paint_supported(paint: &nuxie_graph::ShapePaintNode) -> bool {
    if !paint.is_visible {
        return paint.feather.is_none() && paint.effects.is_empty();
    }
    matches!(
        paint.paint_type,
        ShapePaintKind::Fill | ShapePaintKind::Stroke
    ) && matches!(
        paint.path_kind,
        Some(ShapePaintPathKind::Local | ShapePaintPathKind::LocalClockwise)
    ) && matches!(
        paint.paint_state.as_ref(),
        Some(
            ShapePaintStateNode::SolidColor { .. }
                | ShapePaintStateNode::LinearGradient { .. }
                | ShapePaintStateNode::RadialGradient { .. }
        )
    ) && paint.effects.is_empty()
}

fn nearly_equal(a: f32, b: f32) -> bool {
    (a - b).abs() <= 0.0001
}

fn nested_stateful_view_model_object(
    runtime: &RuntimeFile,
    artboard: &ArtboardGraph,
) -> Option<(&'static str, u32)> {
    if artboard.nested_artboards.is_empty() {
        return None;
    }
    let nested_host_locals = artboard
        .nested_artboards
        .iter()
        .filter(|host| {
            matches!(
                host.type_name,
                "NestedArtboard" | "NestedArtboardLayout" | "NestedArtboardLeaf"
            )
        })
        .map(|host| host.local_id)
        .collect::<BTreeSet<_>>();
    let mut allowed_stateful_child_locals = BTreeSet::<usize>::new();
    let mut changed = true;
    while changed {
        changed = false;
        for object in &artboard.local_objects {
            let Some(type_name) = object.type_name else {
                continue;
            };
            if !type_name.starts_with("ViewModelInstance") {
                continue;
            }
            if allowed_stateful_child_locals.contains(&object.local_id) {
                continue;
            }
            let parent_local = runtime
                .object(object.global_id as usize)
                .and_then(|object| object.uint_property("parentId"))
                .and_then(|parent| usize::try_from(parent).ok());
            let is_stateful_root =
                parent_local.is_some_and(|parent| nested_host_locals.contains(&parent));
            let is_stateful_descendant =
                parent_local.is_some_and(|parent| allowed_stateful_child_locals.contains(&parent));
            if is_stateful_root || is_stateful_descendant {
                allowed_stateful_child_locals.insert(object.local_id);
                changed = true;
            }
        }
    }
    artboard.local_objects.iter().find_map(|object| {
        let type_name = object.type_name?;
        (type_name.starts_with("ViewModelInstance")
            && !allowed_stateful_child_locals.contains(&object.local_id))
        .then_some((type_name, object.global_id))
    })
}

fn nested_child_data_bind_supported(data_bind: &nuxie_graph::DataBindNode) -> bool {
    if data_bind.target_type_name == Some("SolidColor") {
        return true;
    }
    if data_bind.target_type_name == Some("ArtboardComponentList")
        // ArtboardComponentListBase::listSourcePropertyKey in C++ generated/artboard_component_list_base.hpp.
        && data_bind.property_key == 800
        && (data_bind.converter_global.is_none()
            || data_bind.converter_type_name == Some("DataConverterNumberToList"))
    {
        return true;
    }
    if data_bind.target_type_name == Some("CustomPropertyNumber")
        // CustomPropertyNumberBase::propertyValuePropertyKey in C++ generated/custom_property_number_base.hpp.
        && data_bind.property_key == 243
        && (data_bind.converter_global.is_none()
            || matches!(
                data_bind.converter_type_name,
                Some("DataConverterGroup" | "DataConverterFormula")
            ))
    {
        return true;
    }
    if matches!(
        data_bind.target_type_name,
        Some(
            "ViewModelInstanceBoolean"
                | "ViewModelInstanceColor"
                | "ViewModelInstanceString"
                | "ViewModelInstanceEnum"
                | "ViewModelInstanceNumber"
        )
    )
        // ViewModelInstance*Base::propertyValuePropertyKey in generated/viewmodel.
        && matches!(data_bind.property_key, 555 | 560 | 561 | 575 | 593)
        && data_bind.converter_global.is_none()
    {
        return true;
    }
    (data_bind.target_type_name == Some("Ellipse")
        && matches!(data_bind.property_key, 20 | 21)
        && data_bind.converter_global.is_none())
        || (data_bind.target_type_name == Some("Shape")
            && matches!(data_bind.property_key, 13 | 14)
            && data_bind.converter_global.is_none())
        || (data_bind.target_type_name == Some("Shape")
            // NodeBase::computedRootXPropertyKey/computedRootYPropertyKey in
            // C++ generated/node_base.hpp.
            && matches!(data_bind.property_key, 864 | 865)
            && data_bind.converter_global.is_none())
        || (data_bind.target_type_name == Some("Shape")
            // ShapeBase::lengthPropertyKey is a computed target-to-source value.
            && data_bind.property_key == 781
            && data_bind.converter_global.is_none())
        || (data_bind.target_type_name == Some("Shape")
            // TransformComponentBase::rotationPropertyKey in C++ generated/transform_component_base.hpp.
            && data_bind.property_key == 15
            && data_bind.converter_type_name == Some("DataConverterSystemDegsToRads"))
        || (data_bind.target_type_name == Some("Shape")
            // TransformComponentBase::scaleX/scaleYPropertyKey.
            && matches!(data_bind.property_key, 16 | 17)
            && data_bind.converter_type_name == Some("DataConverterSystemNormalizer"))
        || (data_bind.target_type_name == Some("RootBone")
            && matches!(data_bind.property_key, 90 | 91)
            && data_bind.converter_global.is_none())
        || (data_bind.target_type_name == Some("FollowPathConstraint")
            // FollowPathConstraintBase::distancePropertyKey.
            && data_bind.property_key == 363
            && data_bind.converter_type_name == Some("DataConverterRangeMapper"))
        || (data_bind.target_type_name == Some("Artboard")
            // NodeBase::x/yPropertyKey in C++ generated/node_base.hpp.
            && matches!(data_bind.property_key, 13 | 14)
            && data_bind.converter_global.is_none())
        || (data_bind.target_type_name == Some("Artboard")
            // ArtboardBase::clipPropertyKey in C++ generated/artboard_base.hpp.
            && data_bind.property_key == 196
            && data_bind.converter_global.is_none())
        || (data_bind.target_type_name == Some("Node")
            // NodeBase::x/yPropertyKey in C++ generated/node_base.hpp.
            && matches!(data_bind.property_key, 13 | 14)
            && data_bind.converter_global.is_none())
        || (data_bind.target_type_name == Some("Node")
            // TransformComponentBase::rotationPropertyKey in C++ generated/transform_component_base.hpp.
            && data_bind.property_key == 15
            && data_bind.converter_type_name == Some("DataConverterGroup"))
        || (data_bind.target_type_name == Some("Node")
            // WorldTransformComponentBase::opacityPropertyKey in C++ generated/world_transform_component_base.hpp.
            && data_bind.property_key == 18
            && data_bind.converter_global.is_none())
        || (data_bind.target_type_name == Some("NestedArtboard")
            // NestedArtboard inherits NodeBase::x/yPropertyKey.
            && matches!(data_bind.property_key, 13 | 14)
            && data_bind.converter_global.is_none())
        || (data_bind.target_type_name == Some("NestedArtboard")
            // NestedArtboardBase::artboardIdPropertyKey.
            && data_bind.property_key == 197
            && data_bind.converter_global.is_none())
        || (data_bind.target_type_name == Some("Rectangle")
            // ParametricPathBase::widthPropertyKey/heightPropertyKey in C++ generated/shapes/parametric_path_base.hpp.
            && matches!(data_bind.property_key, 16 | 17 | 20 | 21)
            && data_bind.converter_global.is_none())
        || (data_bind.target_type_name == Some("CubicMirroredVertex")
            // CubicMirroredVertexBase::distancePropertyKey.
            && data_bind.property_key == 83
            && data_bind.converter_global.is_none())
        || (data_bind.target_type_name == Some("LinearGradient")
            // LinearGradientBase start/end coordinate property keys.
            && matches!(data_bind.property_key, 32 | 33 | 34 | 35)
            && (data_bind.converter_global.is_none()
                || matches!(
                    data_bind.converter_type_name,
                    Some("DataConverterOperationValue" | "DataConverterFormula")
                )))
        || (data_bind.target_type_name == Some("CustomPropertyString")
            // CustomPropertyStringBase::propertyValuePropertyKey in C++ generated/custom_property_string_base.hpp.
            && data_bind.property_key == 246
            && (data_bind.converter_global.is_none()
                || data_bind.converter_type_name == Some("DataConverterToString")))
        || (data_bind.target_type_name == Some("TextValueRun")
            // TextValueRunBase::textPropertyKey in C++ generated/text/text_value_run_base.hpp.
            && data_bind.property_key == 268
            && (data_bind.converter_global.is_none()
                || data_bind.converter_type_name == Some("DataConverterToString")
                || data_bind.converter_type_name == Some("DataConverterGroup")
                || data_bind.converter_type_name == Some("DataConverterFormula")))
        || (data_bind.target_type_name == Some("TextFollowPathModifier")
            // TextFollowPathModifierBase start/end/strength/offset/radial/orient.
            && matches!(data_bind.property_key, 779 | 782 | 783 | 784 | 785 | 786)
            && (data_bind.converter_global.is_none()
                || data_bind.converter_type_name == Some("DataConverterFormula")))
        || (data_bind.target_type_name == Some("Text")
            // TextBase verticalTrimTopValue/verticalTrimBottomValue bitmask passthroughs.
            && matches!(data_bind.property_key, 1027 | 1028)
            && data_bind.converter_global.is_none())
        || (data_bind.target_type_name == Some("Image")
            // ImageBase::assetIdPropertyKey in C++ generated/shapes/image_base.hpp.
            && data_bind.property_key == 206
            && data_bind.converter_global.is_none())
        || (data_bind.target_type_name == Some("TrimPath")
            // TrimPathBase::start/end/offsetPropertyKey in C++ generated/shapes/paint/trim_path_base.hpp.
            && matches!(data_bind.property_key, 114 | 115 | 116)
            && (data_bind.converter_global.is_none()
                || matches!(
                    data_bind.converter_type_name,
                    Some("DataConverterGroup" | "DataConverterRangeMapper")
                )))
        || (data_bind.target_type_name == Some("LayoutComponent")
            // LayoutComponentBase::width/heightPropertyKey in C++ generated/layout/layout_component_base.hpp.
            && matches!(data_bind.property_key, 7 | 8)
            && (data_bind.converter_global.is_none()
                || data_bind.converter_type_name == Some("DataConverterInterpolator")))
        || (data_bind.target_type_name == Some("LayoutComponentStyle")
            // LayoutComponentStyleBase::displayValuePropertyKey in C++ generated/layout/layout_component_style_base.hpp.
            && data_bind.property_key == 596
            && data_bind.converter_global.is_none())
}

fn nested_child_data_bind_unsupported_feature(
    data_bind: &nuxie_graph::DataBindNode,
) -> &'static str {
    match (data_bind.target_type_name, data_bind.property_key) {
        (Some("TrimPath"), 114 | 115 | 116) => "nested-trim-path-data-bind",
        (Some("Artboard"), 13 | 14) => "nested-artboard-root-transform",
        (Some("Artboard"), 196) => "nested-layout-clip-data-bind",
        (Some("LayoutComponent"), 7 | 8) => "nested-layout-size-data-bind",
        // Transform properties can surface through nested child Node targets.
        (Some("Node"), 13 | 14 | 15) => "nested-node-transform-data-bind",
        (
            Some(
                "ViewModelInstanceBoolean"
                | "ViewModelInstanceColor"
                | "ViewModelInstanceString"
                | "ViewModelInstanceEnum"
                | "ViewModelInstanceNumber",
            ),
            555 | 560 | 561 | 575 | 593,
        ) => "nested-stateful-view-model-property",
        _ => "data-binding-nested-child",
    }
}

fn solid_color_data_bind_supported(data_bind: &nuxie_graph::DataBindNode) -> bool {
    // SolidColorBase::colorValuePropertyKey in C++ generated/shapes/paint/solid_color_base.hpp.
    const SOLID_COLOR_VALUE_PROPERTY_KEY: u64 = 37;
    let source_to_target = data_bind.flags & DATA_BIND_FLAG_TWO_WAY != 0
        || data_bind.flags & DATA_BIND_FLAG_DIRECTION_TO_SOURCE == 0;
    data_bind.property_key == SOLID_COLOR_VALUE_PROPERTY_KEY
        && source_to_target
        && (data_bind.converter_global.is_none()
            || data_bind.converter_type_name == Some("DataConverterInterpolator"))
}

fn custom_property_enum_data_bind_supported(data_bind: &nuxie_graph::DataBindNode) -> bool {
    // CustomPropertyEnumBase::propertyValuePropertyKey in C++ generated/custom_property_enum_base.hpp.
    const CUSTOM_PROPERTY_ENUM_VALUE_PROPERTY_KEY: u64 = 872;
    let target_to_source = data_bind.flags & DATA_BIND_FLAG_TWO_WAY != 0
        || data_bind.flags & DATA_BIND_FLAG_DIRECTION_TO_SOURCE != 0;
    data_bind.property_key == CUSTOM_PROPERTY_ENUM_VALUE_PROPERTY_KEY
        && data_bind.converter_global.is_none()
        && target_to_source
}

fn nested_child_data_bind_is_text(data_bind: &nuxie_graph::DataBindNode) -> bool {
    matches!(
        data_bind.target_type_name,
        Some("Text" | "TextValueRun" | "TextStylePaint" | "TextFollowPathModifier")
    )
}

fn nested_artboard_host_control_data_bind<'a>(
    graph: &GraphFile,
    artboard: &'a ArtboardGraph,
) -> Option<&'a nuxie_graph::DataBindNode> {
    artboard.data_binds.iter().find(|data_bind| {
        if data_bind.target_type_name != Some("NestedArtboard") {
            return false;
        }
        match data_bind.property_key {
            // NestedArtboardBase::artboardIdPropertyKey in C++ generated/nested_artboard_base.hpp.
            197 => {
                let source_to_target = data_bind.flags & DATA_BIND_FLAG_TWO_WAY != 0
                    || data_bind.flags & DATA_BIND_FLAG_DIRECTION_TO_SOURCE == 0;
                graph.artboards.len() > 1
                    && artboard_has_state_machine_listeners(artboard)
                    && (!source_to_target || data_bind.converter_global.is_some())
            }
            _ => false,
        }
    })
}

fn nested_unsupported_listener_propagation_child_global(
    runtime: &RuntimeFile,
    graph: &GraphFile,
    artboard: &ArtboardGraph,
) -> Option<u32> {
    let target_nested_child = artboard
        .sorted_drawable_order
        .iter()
        .filter_map(|drawable| Some((drawable.local_id?, drawable.referenced_artboard_global?)))
        .collect::<Vec<_>>();

    for state_machine in &artboard.state_machines {
        for listener in &state_machine.listeners {
            if state_machine_listener_has_event_type(runtime, artboard, listener) {
                continue;
            }
            let Ok(target_id) = usize::try_from(listener.target_id) else {
                continue;
            };
            let Some((_, child_global)) = target_nested_child
                .iter()
                .find(|(local_id, _)| *local_id == target_id)
            else {
                continue;
            };
            if graph
                .artboards
                .iter()
                .find(|child| child.global_id == *child_global)
                .is_some_and(artboard_has_state_machine_listeners)
            {
                return Some(*child_global);
            }
        }
    }

    None
}

fn state_machine_listener_has_event_type(
    runtime: &RuntimeFile,
    artboard: &ArtboardGraph,
    listener: &nuxie_graph::StateMachineListenerGraph,
) -> bool {
    let Some(listener_object) = runtime.object(listener.global_id as usize) else {
        return false;
    };
    if listener_object.type_name == "StateMachineListenerSingle" {
        return listener_object.uint_property("listenerTypeValue") == Some(5);
    }

    let Some(listener_local) = artboard
        .local_objects
        .iter()
        .find(|object| object.global_id == listener.global_id)
        .map(|object| object.local_id)
    else {
        return false;
    };

    artboard.local_objects.iter().any(|object| {
        let Some(runtime_object) = runtime.object(object.global_id as usize) else {
            return false;
        };
        runtime_object.uint_property("parentId") == Some(listener_local as u64)
            && (runtime_object.type_name == "ListenerInputTypeEvent"
                || runtime_object.uint_property("listenerTypeValue") == Some(5))
    })
}

fn artboard_has_state_machine_listeners(artboard: &ArtboardGraph) -> bool {
    artboard
        .state_machines
        .iter()
        .any(|state_machine| !state_machine.listeners.is_empty())
}

fn artboard_has_recursive_nested_artboard(
    graph: &GraphFile,
    artboard: &ArtboardGraph,
    visiting: &mut BTreeSet<u32>,
) -> bool {
    if !visiting.insert(artboard.global_id) {
        return false;
    }

    for referenced_artboard_global in artboard
        .sorted_drawable_order
        .iter()
        .filter_map(|drawable| drawable.referenced_artboard_global)
    {
        let Some(child) = graph
            .artboards
            .iter()
            .find(|artboard| artboard.global_id == referenced_artboard_global)
        else {
            continue;
        };
        if child
            .sorted_drawable_order
            .iter()
            .any(|drawable| drawable.referenced_artboard_global.is_some())
            || artboard_has_recursive_nested_artboard(graph, child, visiting)
        {
            return true;
        }
    }

    false
}

fn runtime_has_type(runtime: &RuntimeFile, type_name: &str) -> bool {
    runtime
        .objects
        .iter()
        .flatten()
        .any(|object| object.type_name == type_name)
}

fn first_image_or_asset_global(graph: &GraphFile, artboard: &ArtboardGraph) -> Option<u32> {
    artboard
        .local_objects
        .iter()
        .find(|object| object.type_name == Some("Image"))
        .map(|object| object.global_id)
        .or_else(|| {
            graph
                .file_assets
                .iter()
                .find(|asset| asset.type_name == "ImageAsset")
                .map(|asset| asset.global_id)
        })
}

fn unsupported_image_global(
    runtime: &RuntimeFile,
    graph: &GraphFile,
    artboard: &ArtboardGraph,
) -> Option<u32> {
    let image_global = first_image_or_asset_global(graph, artboard)?;
    (!simple_static_image_artboard_supported(runtime, graph, artboard)).then_some(image_global)
}

fn simple_static_image_artboard_supported(
    runtime: &RuntimeFile,
    graph: &GraphFile,
    artboard: &ArtboardGraph,
) -> bool {
    let mut visiting = BTreeSet::new();
    simple_static_image_artboard_tree_supported(runtime, graph, artboard, &mut visiting)
}

fn simple_static_image_artboard_tree_supported(
    runtime: &RuntimeFile,
    graph: &GraphFile,
    artboard: &ArtboardGraph,
    visiting: &mut BTreeSet<u32>,
) -> bool {
    if !visiting.insert(artboard.global_id) {
        return false;
    }
    let supported =
        simple_static_image_artboard_tree_supported_entered(runtime, graph, artboard, visiting);
    visiting.remove(&artboard.global_id);
    supported
}

fn simple_static_image_artboard_tree_supported_entered(
    runtime: &RuntimeFile,
    graph: &GraphFile,
    artboard: &ArtboardGraph,
    visiting: &mut BTreeSet<u32>,
) -> bool {
    // Ported image drawing covers C++ `src/shapes/image.cpp` for direct image
    // drawables, including the `Mesh::draw` vertex-buffer path. Keep the guard
    // focused on asset resolution and nested image tree admission.
    let image_asset_globals = graph
        .file_assets
        .iter()
        .filter(|asset| asset.type_name == "ImageAsset")
        .map(|asset| asset.global_id)
        .collect::<BTreeSet<_>>();
    if image_asset_globals.is_empty() {
        return false;
    }
    artboard
        .sorted_drawable_order
        .iter()
        .all(|drawable| match drawable.type_name {
            "Image" => drawable
                .resolved_image_asset_global
                .is_some_and(|asset_global| image_asset_globals.contains(&asset_global)),
            "NestedArtboard" => {
                let Some(referenced_artboard_global) = drawable.referenced_artboard_global else {
                    // C++ suppresses unresolved nested hosts; there is no child
                    // image tree to validate.
                    return true;
                };
                let Some(child_artboard) = graph
                    .artboards
                    .iter()
                    .find(|artboard| artboard.global_id == referenced_artboard_global)
                else {
                    return false;
                };
                simple_static_image_artboard_tree_supported(
                    runtime,
                    graph,
                    child_artboard,
                    visiting,
                )
            }
            _ => true,
        })
}

fn frame_dimension(value: f32) -> u32 {
    value.ceil().max(1.0) as u32
}
