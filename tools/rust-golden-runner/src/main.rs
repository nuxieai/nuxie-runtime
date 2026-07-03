use anyhow::{Context, Result, bail};
use rive_binary::read_runtime_file;
use rive_graph::{ArtboardGraph, GraphFile};
use rive_render_api::RecordingFactory;
use rive_runtime::{ArtboardInstance, preallocate_render_paints};
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
    if options.state_machine.is_some() {
        bail!("unsupported: Rust golden runner does not advance state machines yet");
    }
    if options.input_script.is_some() {
        bail!("unsupported: Rust golden runner does not replay input scripts yet");
    }
    if options.samples.iter().any(|sample| *sample != 0.0) {
        bail!("unsupported: Rust golden runner only supports sample 0");
    }

    let bytes = std::fs::read(&options.file)
        .with_context(|| format!("failed to read {}", options.file.display()))?;
    let runtime = read_runtime_file(&bytes).context("failed to import runtime file")?;
    let graph = GraphFile::from_runtime_file(&runtime).context("failed to build graph")?;
    let (artboard_index, artboard) = select_artboard(&graph, options.artboard.as_deref())?;
    let mut instance = ArtboardInstance::from_graph(&runtime, artboard)
        .context("failed to instantiate artboard")?;
    instance.update_components();

    let mut factory = RecordingFactory::new();
    let mut paint_by_global = preallocate_render_paints(&runtime, artboard, &mut factory);
    let mut renderer = factory.make_renderer();

    let artboard_object = runtime
        .artboard(artboard_index)
        .context("missing selected artboard object")?;
    let width = artboard_object.double_property("width").unwrap_or(0.0);
    let height = artboard_object.double_property("height").unwrap_or(0.0);
    let artboard_name = artboard.name.clone().unwrap_or_default();

    factory.source(
        &options.file.to_string_lossy(),
        &artboard_name,
        &artboard_name,
    );
    factory.frame_size(frame_dimension(width), frame_dimension(height));

    for sample in &options.samples {
        factory.add_sample(*sample);
        instance.draw_static_artboard(
            &runtime,
            artboard,
            &mut factory,
            &mut renderer,
            &mut paint_by_global,
        )?;
        factory.add_frame();
    }

    Ok(factory.stream())
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
    value
        .split(',')
        .map(|part| {
            part.trim()
                .parse::<f32>()
                .with_context(|| format!("invalid sample {}", part.trim()))
        })
        .collect()
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

fn frame_dimension(value: f32) -> u32 {
    value.ceil().max(1.0) as u32
}
