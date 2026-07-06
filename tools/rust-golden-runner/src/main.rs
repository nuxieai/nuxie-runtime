use anyhow::{Context, Result, anyhow, bail};
use rive_binary::{RuntimeFile, RuntimeObject, read_runtime_file};
use rive_graph::{
    ArtboardGraph, GraphFile, ShapePaintContainerNode, ShapePaintKind, ShapePaintPathKind,
    ShapePaintStateNode,
};
use rive_render_api::RecordingFactory;
use rive_runtime::{
    ArtboardInstance, RuntimeLayoutBoundsReport, RuntimeOwnedViewModelInstance,
    RuntimeRenderPathCache, StateMachineInstance, preallocate_render_paint_cache_for_artboard_tree,
    static_text_support_error,
};
use std::collections::BTreeSet;
use std::env;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

const TIME_EPSILON: f32 = 0.000001;
const DATA_BIND_FLAG_DIRECTION_TO_SOURCE: u64 = 1 << 0;
const DATA_BIND_FLAG_TWO_WAY: u64 = 1 << 1;

fn main() {
    match run() {
        Ok(stream) => print!("{stream}"),
        Err(error) => {
            eprintln!("rust-golden-runner error: {error:#}");
            std::process::exit(1);
        }
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
    let mut owned_view_model_context = selected_artboard_view_model_index(&runtime, artboard_index)
        .and_then(|view_model_index| {
            RuntimeOwnedViewModelInstance::new(&runtime, view_model_index)
        });
    instance.bind_default_view_model_artboard_list_context(&runtime);
    if let Some(state_machine) = state_machine.as_mut() {
        if let Some(context) = owned_view_model_context.as_ref() {
            state_machine.bind_owned_view_model_context(context);
        }
        state_machine.advance_data_context();
    }
    if let Some(context) = owned_view_model_context.as_ref() {
        instance.bind_owned_view_model_nested_artboard_contexts(&runtime, context);
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

    let mut factory = RecordingFactory::new();
    let mut paint_cache = preallocate_render_paint_cache_for_artboard_tree(
        &runtime,
        artboard,
        &graph.artboards,
        &mut factory,
    );
    let mut path_cache = RuntimeRenderPathCache::default();
    let mut renderer = factory.make_renderer();

    let artboard_object = runtime
        .artboard(artboard_index)
        .context("missing selected artboard object")?;
    let width = artboard_object.double_property("width").unwrap_or(0.0);
    let height = artboard_object.double_property("height").unwrap_or(0.0);

    factory.source(&options.file.to_string_lossy(), &artboard_name, &scene.name);
    factory.frame_size(frame_dimension(width), frame_dimension(height));

    let mut current_seconds = 0.0;
    let mut next_input = 0;
    for sample in &options.samples {
        while next_input < input_events.len()
            && input_events[next_input].seconds <= *sample + TIME_EPSILON
        {
            let event = &input_events[next_input];
            advance_scene_to(
                &mut instance,
                &runtime,
                state_machine.as_mut(),
                owned_view_model_context.as_ref(),
                event.seconds,
                &mut current_seconds,
            )?;
            apply_input_event(
                event,
                &instance,
                state_machine.as_mut(),
                owned_view_model_context.as_mut(),
            );
            factory.add_input_event(
                event.kind.name(),
                event.seconds,
                event.x,
                event.y,
                event.pointer_id,
            );
            next_input += 1;
        }
        advance_scene_to(
            &mut instance,
            &runtime,
            state_machine.as_mut(),
            owned_view_model_context.as_ref(),
            *sample,
            &mut current_seconds,
        )?;
        instance.prepare_static_artboard_tree_paints(
            &runtime,
            artboard,
            &graph.artboards,
            &mut factory,
            &mut paint_cache,
            &mut path_cache,
        )?;
        factory.add_sample(*sample);
        instance
            .draw_prepared_static_artboard_with_render_cache(
                &runtime,
                artboard,
                &graph.artboards,
                &mut factory,
                &mut renderer,
                &mut paint_cache,
                &mut path_cache,
            )
            .map_err(unsupported_static_text_draw_error)?;
        factory.add_frame();
    }

    Ok(factory.stream())
}

fn write_layout_bounds_report(
    options: &Options,
    runtime: &RuntimeFile,
    artboard: &ArtboardGraph,
    artboard_name: &str,
    scene_name: &str,
    instance: &mut ArtboardInstance,
    state_machine: &mut Option<StateMachineInstance>,
    owned_view_model_context: &mut Option<RuntimeOwnedViewModelInstance>,
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
                owned_view_model_context.as_mut(),
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
    owned_view_model_context: Option<&mut RuntimeOwnedViewModelInstance>,
) {
    let Some(state_machine) = state_machine else {
        return;
    };
    match event.kind {
        InputKind::PointerDown => {
            if let Some(context) = owned_view_model_context {
                state_machine.pointer_down_with_owned_view_model_context(
                    instance,
                    event.x,
                    event.y,
                    event.pointer_id,
                    context,
                );
            } else {
                state_machine.pointer_down(instance, event.x, event.y, event.pointer_id);
            }
        }
        InputKind::PointerMove => {
            if let Some(context) = owned_view_model_context {
                state_machine.pointer_move_with_owned_view_model_context(
                    instance,
                    event.x,
                    event.y,
                    event.seconds,
                    event.pointer_id,
                    context,
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
                state_machine.pointer_up_with_owned_view_model_context(
                    instance,
                    event.x,
                    event.y,
                    event.pointer_id,
                    context,
                );
            } else {
                state_machine.pointer_up(instance, event.x, event.y, event.pointer_id);
            }
        }
        InputKind::PointerExit => {
            if let Some(context) = owned_view_model_context {
                state_machine.pointer_exit_with_owned_view_model_context(
                    instance,
                    event.x,
                    event.y,
                    event.pointer_id,
                    context,
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
    state_machine: Option<&mut StateMachineInstance>,
    owned_view_model_context: Option<&RuntimeOwnedViewModelInstance>,
    target_seconds: f32,
    current_seconds: &mut f32,
) -> Result<()> {
    if target_seconds + TIME_EPSILON < *current_seconds {
        bail!("cannot move timeline backwards");
    }
    let elapsed_seconds = (target_seconds - *current_seconds).max(0.0);
    if let Some(state_machine) = state_machine {
        instance.advance_state_machine_instance(state_machine, elapsed_seconds);
        if instance.advance_nested_artboards_with_state_machine(elapsed_seconds, state_machine) {
            instance.advance_state_machine_instance(state_machine, 0.0);
        }
    } else {
        instance.advance_nested_artboards(elapsed_seconds);
    }
    if let Some(context) = owned_view_model_context {
        instance.bind_owned_view_model_nested_artboard_contexts(runtime, context);
    }
    instance.advance_artboard_data_binds_with_elapsed(elapsed_seconds);
    instance.update_pass();
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
}

impl Options {
    fn parse(args: Vec<String>) -> Result<Self> {
        let mut file = None::<PathBuf>;
        let mut artboard = None;
        let mut state_machine = None;
        let mut input_script = None;
        let mut samples = vec![0.0];
        let mut layout_bounds = false;

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
                "--help" | "-h" => {
                    println!(
                        "usage: rust-golden-runner --file <path> [--artboard <name>] [--samples <t0,t1,...>] [--layout-bounds]"
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

        Ok(Self {
            file: file.context("missing --file <path>")?,
            artboard,
            state_machine,
            input_script,
            samples,
            layout_bounds,
        })
    }
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

    if let Some(nested) = artboard
        .nested_artboards
        .iter()
        .find(|nested| nested.type_name != "NestedArtboard")
    {
        if matches!(
            nested.type_name,
            "NestedArtboardLayout" | "NestedArtboardLeaf"
        ) {
            bail!(
                "unsupported: nested-artboard-layout in Rust golden runner ({} global {})",
                nested.type_name,
                nested.global_id
            );
        }
        bail!(
            "unsupported: nested artboards in Rust golden runner ({})",
            nested.type_name
        );
    }

    if let Some((type_name, global_id)) = artboard.local_objects.iter().find_map(|object| {
        let type_name = object.type_name?;
        matches!(type_name, "NestedArtboardLayout" | "NestedArtboardLeaf")
            .then_some((type_name, object.global_id))
    }) {
        bail!(
            "unsupported: nested-artboard-layout in Rust golden runner ({type_name} global {global_id})"
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
            bail!(
                "unsupported: data-binding-nested-child in Rust golden runner (data bind global {} target {:?})",
                data_bind.global_id,
                data_bind.target_type_name
            );
        }
        if let Some(focus_data) = artboard
            .local_objects
            .iter()
            .find(|object| object.type_name == Some("FocusData"))
        {
            bail!(
                "unsupported: focus-data in Rust golden runner (nested child global {})",
                focus_data.global_id
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

    if let Some(container) = unsupported_layout_component_paint(runtime, artboard) {
        bail!(
            "unsupported: layout-component-paint in Rust golden runner (global {})",
            container.global_id
        );
    }

    if let Some(feather) = artboard
        .local_objects
        .iter()
        .find(|object| object.type_name == Some("Feather"))
    {
        bail!(
            "unsupported: feather in Rust golden runner (global {})",
            feather.global_id
        );
    }

    if let Some(scroll_constraint) = artboard
        .local_objects
        .iter()
        .find(|object| object.type_name == Some("ScrollConstraint"))
    {
        bail!(
            "unsupported: scroll-constraints in Rust golden runner (global {})",
            scroll_constraint.global_id
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

    if let Some(n_sliced_node) = artboard
        .local_objects
        .iter()
        .find(|object| object.type_name == Some("NSlicedNode"))
    {
        bail!(
            "unsupported: n-slice in Rust golden runner (global {})",
            n_sliced_node.global_id
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

    if let Some(scripted_drawable) = unsupported_scripted_data_context(runtime, artboard) {
        bail!(
            "unsupported: scripted-data-context in Rust golden runner (ScriptedDrawable global {})",
            scripted_drawable.global_id
        );
    }

    if let Some((path_effect_type, global_id)) =
        artboard
            .local_objects
            .iter()
            .find_map(|object| match object.type_name {
                Some(
                    path_effect_type @ ("ScriptedPathEffect" | "TargetEffect" | "GroupEffect"),
                ) => Some((path_effect_type, object.global_id)),
                _ => None,
            })
    {
        bail!(
            "unsupported: scripted-path-effects in Rust golden runner ({path_effect_type} global {global_id})"
        );
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
    if layout_object.bool_property("clip").unwrap_or(false) {
        return false;
    }
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
    child: &rive_graph::ComponentNode,
    parent_is_row: bool,
) -> bool {
    let Some(child_object) = runtime.object(child.global_id as usize) else {
        return false;
    };
    if child_object.bool_property("clip").unwrap_or(false) {
        return false;
    }
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

fn root_layout_background_paint_supported(paint: &rive_graph::ShapePaintNode) -> bool {
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

fn simple_layout_background_paint_supported(paint: &rive_graph::ShapePaintNode) -> bool {
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
    ) && paint.feather.is_none()
        && paint.effects.is_empty()
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
        .filter(|host| host.type_name == "NestedArtboard")
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

fn nested_child_data_bind_supported(data_bind: &rive_graph::DataBindNode) -> bool {
    if data_bind.target_type_name == Some("SolidColor") {
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
            // TransformComponentBase::rotationPropertyKey in C++ generated/transform_component_base.hpp.
            && data_bind.property_key == 15
            && data_bind.converter_type_name == Some("DataConverterSystemDegsToRads"))
        || (data_bind.target_type_name == Some("RootBone")
            && matches!(data_bind.property_key, 90 | 91)
            && data_bind.converter_global.is_none())
        || (data_bind.target_type_name == Some("Node")
            // WorldTransformComponentBase::opacityPropertyKey in C++ generated/world_transform_component_base.hpp.
            && data_bind.property_key == 18
            && data_bind.converter_global.is_none())
        || (data_bind.target_type_name == Some("Rectangle")
            // ParametricPathBase::widthPropertyKey/heightPropertyKey in C++ generated/shapes/parametric_path_base.hpp.
            && matches!(data_bind.property_key, 16 | 17 | 20 | 21)
            && data_bind.converter_global.is_none())
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
}

fn solid_color_data_bind_supported(data_bind: &rive_graph::DataBindNode) -> bool {
    // SolidColorBase::colorValuePropertyKey in C++ generated/shapes/paint/solid_color_base.hpp.
    const SOLID_COLOR_VALUE_PROPERTY_KEY: u64 = 37;
    let source_to_target = data_bind.flags & DATA_BIND_FLAG_TWO_WAY != 0
        || data_bind.flags & DATA_BIND_FLAG_DIRECTION_TO_SOURCE == 0;
    data_bind.property_key == SOLID_COLOR_VALUE_PROPERTY_KEY
        && source_to_target
        && (data_bind.converter_global.is_none()
            || data_bind.converter_type_name == Some("DataConverterInterpolator"))
}

fn custom_property_enum_data_bind_supported(data_bind: &rive_graph::DataBindNode) -> bool {
    // CustomPropertyEnumBase::propertyValuePropertyKey in C++ generated/custom_property_enum_base.hpp.
    const CUSTOM_PROPERTY_ENUM_VALUE_PROPERTY_KEY: u64 = 872;
    let target_to_source = data_bind.flags & DATA_BIND_FLAG_TWO_WAY != 0
        || data_bind.flags & DATA_BIND_FLAG_DIRECTION_TO_SOURCE != 0;
    data_bind.property_key == CUSTOM_PROPERTY_ENUM_VALUE_PROPERTY_KEY
        && data_bind.converter_global.is_none()
        && target_to_source
}

fn unsupported_scripted_data_context<'a>(
    runtime: &RuntimeFile,
    artboard: &'a ArtboardGraph,
) -> Option<&'a rive_graph::AdvancingComponentNode> {
    let scripted_drawable = artboard
        .advancing_components
        .iter()
        .find(|component| component.type_name == "ScriptedDrawable")?;
    let has_context_text_bind = artboard.data_binds.iter().any(|data_bind| {
        data_bind.type_name == "DataBindContext"
            && data_bind.target_type_name == Some("TextValueRun")
    });
    let has_nested_view_model_context = runtime_has_type(runtime, "ViewModelPropertyViewModel")
        && runtime_has_type(runtime, "ViewModelInstanceViewModel");
    (has_context_text_bind && has_nested_view_model_context).then_some(scripted_drawable)
}

fn nested_child_data_bind_is_text(data_bind: &rive_graph::DataBindNode) -> bool {
    matches!(
        data_bind.target_type_name,
        Some("Text" | "TextValueRun" | "TextStylePaint" | "TextFollowPathModifier")
    )
}

fn nested_artboard_host_control_data_bind<'a>(
    graph: &GraphFile,
    artboard: &'a ArtboardGraph,
) -> Option<&'a rive_graph::DataBindNode> {
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
    listener: &rive_graph::StateMachineListenerGraph,
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

fn unsupported_image_global(
    runtime: &RuntimeFile,
    graph: &GraphFile,
    artboard: &ArtboardGraph,
) -> Option<u32> {
    let first_image_global = artboard
        .local_objects
        .iter()
        .find(|object| object.type_name == Some("Image"))
        .map(|object| object.global_id);
    let first_image_asset_global = graph
        .file_assets
        .iter()
        .find(|asset| asset.type_name == "ImageAsset")
        .map(|asset| asset.global_id);
    if first_image_global.is_none() && first_image_asset_global.is_none() {
        return None;
    }

    (!simple_static_image_artboard_supported(runtime, graph, artboard))
        .then_some(first_image_global.or(first_image_asset_global).unwrap_or(0))
}

fn simple_static_image_artboard_supported(
    runtime: &RuntimeFile,
    graph: &GraphFile,
    artboard: &ArtboardGraph,
) -> bool {
    if !artboard
        .local_objects
        .iter()
        .all(|object| matches!(object.type_name, Some("Artboard" | "Image")))
    {
        return false;
    }
    if !artboard.meshes.is_empty() || !artboard.n_slicer_details.is_empty() {
        return false;
    }

    let image_asset_globals = graph
        .file_assets
        .iter()
        .filter(|asset| asset.type_name == "ImageAsset")
        .map(|asset| asset.global_id)
        .collect::<BTreeSet<_>>();
    if image_asset_globals.is_empty() {
        return false;
    }
    if image_asset_globals
        .iter()
        .any(|asset_global| embedded_file_asset_bytes(runtime, *asset_global).is_none())
    {
        return false;
    }

    artboard
        .sorted_drawable_order
        .iter()
        .filter(|drawable| drawable.type_name == "Image")
        .all(|drawable| {
            drawable
                .resolved_image_asset_global
                .is_some_and(|asset_global| image_asset_globals.contains(&asset_global))
        })
}

fn embedded_file_asset_bytes(runtime: &RuntimeFile, asset_global: u32) -> Option<&[u8]> {
    let file_asset_globals = runtime
        .file_assets()
        .into_iter()
        .map(|asset| asset.id)
        .collect::<BTreeSet<_>>();
    let mut after_asset = false;
    for object in runtime.objects.iter().flatten() {
        if object.id == asset_global {
            after_asset = true;
            continue;
        }
        if !after_asset {
            continue;
        }
        if file_asset_globals.contains(&object.id) {
            return None;
        }
        if object.type_name == "FileAssetContents" {
            return object.bytes_property("bytes");
        }
    }
    None
}

fn frame_dimension(value: f32) -> u32 {
    value.ceil().max(1.0) as u32
}
