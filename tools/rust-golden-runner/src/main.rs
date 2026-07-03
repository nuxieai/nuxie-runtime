use anyhow::{Context, Result, bail};
use rive_binary::{RuntimeFile, read_runtime_file};
use rive_graph::{ArtboardGraph, GraphFile};
use rive_render_api::RecordingFactory;
use rive_runtime::{
    ArtboardInstance, RuntimeRenderPathCache, StateMachineInstance, preallocate_render_paints,
};
use std::env;
use std::path::PathBuf;

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
    if options.input_script.is_some() {
        bail!("unsupported: Rust golden runner does not replay input scripts yet");
    }
    let bytes = std::fs::read(&options.file)
        .with_context(|| format!("failed to read {}", options.file.display()))?;
    let runtime = read_runtime_file(&bytes).context("failed to import runtime file")?;
    let graph = GraphFile::from_runtime_file(&runtime).context("failed to build graph")?;
    let (artboard_index, artboard) = select_artboard(&graph, options.artboard.as_deref())?;
    ensure_static_draw_supported(&graph, artboard)?;
    let mut instance = ArtboardInstance::from_graph(&runtime, artboard)
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

    let mut factory = RecordingFactory::new();
    let mut paint_by_global = preallocate_render_paints(&runtime, artboard, &mut factory);
    let mut path_cache = RuntimeRenderPathCache::default();
    let mut renderer = factory.make_renderer();

    let artboard_object = runtime
        .artboard(artboard_index)
        .context("missing selected artboard object")?;
    let width = artboard_object.double_property("width").unwrap_or(0.0);
    let height = artboard_object.double_property("height").unwrap_or(0.0);
    let artboard_name = artboard.name.clone().unwrap_or_default();

    factory.source(&options.file.to_string_lossy(), &artboard_name, &scene.name);
    factory.frame_size(frame_dimension(width), frame_dimension(height));

    let mut current_seconds = 0.0;
    for sample in &options.samples {
        advance_scene_to(
            &mut instance,
            state_machine.as_mut(),
            *sample,
            &mut current_seconds,
        )?;
        instance.prepare_static_artboard_paints(
            &runtime,
            artboard,
            &mut factory,
            &mut paint_by_global,
            &mut path_cache,
        )?;
        factory.add_sample(*sample);
        instance.draw_prepared_static_artboard_with_path_cache(
            &runtime,
            artboard,
            &mut factory,
            &mut renderer,
            &mut paint_by_global,
            &mut path_cache,
        )?;
        factory.add_frame();
    }

    Ok(factory.stream())
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
    state_machine: Option<&mut StateMachineInstance>,
    target_seconds: f32,
    current_seconds: &mut f32,
) -> Result<()> {
    if target_seconds < *current_seconds {
        bail!("cannot move timeline backwards");
    }
    let elapsed_seconds = (target_seconds - *current_seconds).max(0.0);
    if let Some(state_machine) = state_machine {
        instance.advance_state_machine_instance(state_machine, elapsed_seconds);
    }
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
}

impl Options {
    fn parse(args: Vec<String>) -> Result<Self> {
        let mut file = None::<PathBuf>;
        let mut artboard = None;
        let mut state_machine = None;
        let mut input_script = None;
        let mut samples = vec![0.0];

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
                "--help" | "-h" => {
                    println!(
                        "usage: rust-golden-runner --file <path> [--artboard <name>] [--samples <t0,t1,...>]"
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
    for pair in samples.windows(2) {
        if pair[1] < pair[0] {
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

fn ensure_static_draw_supported(graph: &GraphFile, artboard: &ArtboardGraph) -> Result<()> {
    if let Some(nested) = artboard.nested_artboards.first() {
        bail!(
            "unsupported: nested artboards in Rust golden runner ({})",
            nested.type_name
        );
    }

    if let Some(image) = artboard
        .local_objects
        .iter()
        .find(|object| object.type_name == Some("Image"))
    {
        bail!(
            "unsupported: images in Rust golden runner (global {})",
            image.global_id
        );
    }

    if let Some(asset) = graph
        .file_assets
        .iter()
        .find(|asset| asset.type_name == "ImageAsset")
    {
        bail!(
            "unsupported: images in Rust golden runner (asset global {})",
            asset.global_id
        );
    }

    if let Some(text) = artboard
        .local_objects
        .iter()
        .find(|object| object.type_name == Some("Text"))
    {
        bail!(
            "unsupported: text in Rust golden runner (global {})",
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

    if let Some(container) = artboard
        .shape_paint_containers
        .iter()
        .find(|container| container.type_name == "LayoutComponent" && !container.paints.is_empty())
    {
        bail!(
            "unsupported: layout-component-paint in Rust golden runner (global {})",
            container.global_id
        );
    }

    if let Some(data_bind) = artboard
        .data_binds
        .iter()
        .find(|data_bind| data_bind.target_type_name == Some("SolidColor"))
    {
        bail!(
            "unsupported: data-binding-color in Rust golden runner (data bind global {} target global {:?})",
            data_bind.global_id,
            data_bind.target_global
        );
    }

    if let Some(data_bind) = artboard
        .data_binds
        .iter()
        .find(|data_bind| data_bind.target_type_name == Some("CustomPropertyEnum"))
    {
        bail!(
            "unsupported: data-binding-custom-property-enum in Rust golden runner (data bind global {} target global {:?})",
            data_bind.global_id,
            data_bind.target_global
        );
    }

    if let Some(data_bind) = artboard.data_binds.iter().find(|data_bind| {
        data_bind.target_type_name == Some("Shape")
            && matches!(data_bind.property_key, 13 | 14)
            && (data_bind.converter_global.is_none()
                || matches!(data_bind.converter_type_name, Some("DataConverterGroup"))
                // `interpolation_zero_duration.riv` authors 0.0001s here;
                // park that M5 transform-bind path without catching default
                // duration interpolators that are already sample-0 exact.
                || (data_bind.converter_type_name == Some("DataConverterInterpolator")
                    && data_bind
                        .converter_duration
                        .is_some_and(|duration| duration <= 1.0e-4)))
    }) {
        bail!(
            "unsupported: data-binding-transform in Rust golden runner (data bind global {} target global {:?})",
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
        type_name
            .ends_with("Constraint")
            .then_some((type_name, object.global_id))
    }) {
        bail!(
            "unsupported: constraints in Rust golden runner ({constraint_type} global {global_id})"
        );
    }

    Ok(())
}

fn frame_dimension(value: f32) -> u32 {
    value.ceil().max(1.0) as u32
}
