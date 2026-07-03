use anyhow::{Context, Result, bail};
use rive_binary::{RuntimeFile, RuntimeObject, read_runtime_file};
use rive_graph::{ArtboardGraph, GraphFile, ShapePaintContainerNode};
use rive_render_api::{
    BlendMode, Factory, FillRule, Mat2D as RenderMat2D, RecordingFactory, RenderPaint,
    RenderPaintStyle, RenderPath, Renderer, StrokeCap, StrokeJoin,
};
use rive_runtime::{
    ArtboardInstance, Mat2D, RuntimeDrawCommand, RuntimeDrawCommandKind, RuntimePathCommand,
    RuntimeShapePaintCommand, RuntimeShapePaintKind, RuntimeShapePaintPathKind,
    RuntimeShapePaintState,
};
use std::collections::BTreeMap;
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
    let mut paint_by_global = preallocate_paints(&runtime, &mut factory);
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

    let local_to_global = artboard
        .local_objects
        .iter()
        .map(|object| (object.local_id, object.global_id))
        .collect::<BTreeMap<_, _>>();

    for sample in &options.samples {
        factory.add_sample(*sample);
        draw_static_artboard(
            &runtime,
            artboard,
            &instance,
            &local_to_global,
            width,
            height,
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

fn preallocate_paints(
    runtime: &RuntimeFile,
    factory: &mut RecordingFactory,
) -> BTreeMap<u32, Box<dyn RenderPaint>> {
    let _source_artboard_paints = preallocate_paint_batch(runtime, factory);
    preallocate_paint_batch(runtime, factory)
}

fn preallocate_paint_batch(
    runtime: &RuntimeFile,
    factory: &mut RecordingFactory,
) -> BTreeMap<u32, Box<dyn RenderPaint>> {
    let mut paints = BTreeMap::new();
    for object in runtime.objects.iter().flatten() {
        if matches!(object.type_name, "Fill" | "Stroke") {
            paints.insert(object.id, factory.make_render_paint());
        }
    }
    paints
}

fn draw_static_artboard(
    runtime: &RuntimeFile,
    artboard: &ArtboardGraph,
    instance: &ArtboardInstance,
    local_to_global: &BTreeMap<usize, u32>,
    width: f32,
    height: f32,
    factory: &mut RecordingFactory,
    renderer: &mut dyn Renderer,
    paint_by_global: &mut BTreeMap<u32, Box<dyn RenderPaint>>,
) -> Result<()> {
    renderer.save();
    let mut clip = make_path(
        factory,
        &rect_commands(0.0, 0.0, width, height),
        FillRule::Clockwise,
    );
    renderer.clip_path(clip.as_ref());
    renderer.transform(RenderMat2D::IDENTITY);

    if let Some(background) = artboard
        .shape_paint_containers
        .iter()
        .find(|container| container.local_id == 0)
    {
        draw_background(
            runtime,
            background,
            width,
            height,
            factory,
            renderer,
            paint_by_global,
        )?;
    }

    for command in instance.draw_commands(artboard) {
        draw_command(
            runtime,
            instance,
            local_to_global,
            command,
            factory,
            renderer,
            paint_by_global,
        )?;
    }

    renderer.restore();
    // Keep the clip path alive until after the renderer has consumed it.
    clip.rewind();
    Ok(())
}

fn draw_background(
    runtime: &RuntimeFile,
    container: &ShapePaintContainerNode,
    width: f32,
    height: f32,
    factory: &mut RecordingFactory,
    renderer: &mut dyn Renderer,
    paint_by_global: &mut BTreeMap<u32, Box<dyn RenderPaint>>,
) -> Result<()> {
    let commands = vec![
        RuntimePathCommand::Move { x: -0.0, y: -0.0 },
        RuntimePathCommand::Line { x: width, y: -0.0 },
        RuntimePathCommand::Line {
            x: width,
            y: height,
        },
        RuntimePathCommand::Line { x: -0.0, y: height },
        RuntimePathCommand::Close,
    ];
    for paint in &container.paints {
        let object = runtime
            .object(paint.global_id as usize)
            .with_context(|| format!("missing paint global {}", paint.global_id))?;
        let runtime_paint = RuntimeShapePaintCommand {
            paint_local: paint.local_id,
            mutator_local: paint.mutator_local,
            paint_type: match paint.paint_type {
                rive_graph::ShapePaintKind::Fill => RuntimeShapePaintKind::Fill,
                rive_graph::ShapePaintKind::Stroke => RuntimeShapePaintKind::Stroke,
                rive_graph::ShapePaintKind::Unknown => RuntimeShapePaintKind::Unknown,
            },
            path_kind: RuntimeShapePaintPathKind::Local,
            blend_mode_value: paint.blend_mode_value,
            render_blend_mode_value: 3,
            paint_state: paint.paint_state.clone().map(|state| match state {
                rive_graph::ShapePaintStateNode::SolidColor { color } => {
                    RuntimeShapePaintState::SolidColor {
                        color,
                        render_color: color,
                    }
                }
                rive_graph::ShapePaintStateNode::LinearGradient { .. }
                | rive_graph::ShapePaintStateNode::RadialGradient { .. } => {
                    RuntimeShapePaintState::LinearGradient {
                        start_x: 0.0,
                        start_y: 0.0,
                        end_x: 0.0,
                        end_y: 0.0,
                        opacity: 1.0,
                        render_opacity: 1.0,
                        stops: Vec::new(),
                    }
                }
            }),
            feather_state: None,
            path_commands: commands.clone(),
            effect_path_commands: Vec::new(),
            needs_save_operation: true,
        };
        configure_paint(
            paint_by_global
                .get_mut(&paint.global_id)
                .with_context(|| format!("missing render paint for global {}", paint.global_id))?
                .as_mut(),
            object,
            &runtime_paint,
        )?;
        renderer.save();
        renderer.transform(RenderMat2D::IDENTITY);
        let path = make_path(factory, &commands, fill_rule_for_object(object));
        renderer.draw_path(
            path.as_ref(),
            paint_by_global
                .get(&paint.global_id)
                .with_context(|| format!("missing render paint for global {}", paint.global_id))?
                .as_ref(),
        );
        renderer.restore();
    }
    Ok(())
}

fn draw_command(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    local_to_global: &BTreeMap<usize, u32>,
    command: RuntimeDrawCommand,
    factory: &mut RecordingFactory,
    renderer: &mut dyn Renderer,
    paint_by_global: &mut BTreeMap<u32, Box<dyn RenderPaint>>,
) -> Result<()> {
    match command.kind {
        RuntimeDrawCommandKind::ClipStart | RuntimeDrawCommandKind::ClipEnd => {
            bail!("unsupported: Rust golden runner only supports artboard clipping")
        }
        RuntimeDrawCommandKind::Draw => {}
    }

    let shape_world = command
        .local_id
        .and_then(|local_id| instance.component(local_id))
        .map(|component| component.transform.world_transform)
        .unwrap_or(Mat2D::IDENTITY);

    let mut path_cache = Vec::<(Vec<RuntimePathCommand>, Option<Box<dyn RenderPath>>)>::new();
    for paint in &command.shape_paints {
        let global_id = *local_to_global
            .get(&paint.paint_local)
            .with_context(|| format!("missing paint local {}", paint.paint_local))?;
        let object = runtime
            .object(global_id as usize)
            .with_context(|| format!("missing paint global {global_id}"))?;
        let path_commands = if paint.effect_path_commands.is_empty() {
            &paint.path_commands
        } else {
            &paint.effect_path_commands
        };
        configure_paint(
            paint_by_global
                .get_mut(&global_id)
                .with_context(|| format!("missing render paint for global {global_id}"))?
                .as_mut(),
            object,
            paint,
        )?;
        let path_index = cached_path_slot_index(&mut path_cache, path_commands);
        let mut saved = !paint.needs_save_operation;
        if matches!(
            paint.path_kind,
            RuntimeShapePaintPathKind::Local | RuntimeShapePaintPathKind::LocalClockwise
        ) {
            if !saved {
                saved = true;
                renderer.save();
            }
            renderer.transform(render_mat(shape_world));
        }
        if path_cache[path_index].1.is_none() {
            path_cache[path_index].1 = Some(make_path(factory, path_commands, FillRule::NonZero));
        }
        let path = path_cache[path_index]
            .1
            .as_mut()
            .expect("path cache populated");
        configure_fill_rule(path.as_mut(), object);
        renderer.draw_path(
            path.as_ref(),
            paint_by_global
                .get(&global_id)
                .with_context(|| format!("missing render paint for global {global_id}"))?
                .as_ref(),
        );
        if saved && paint.needs_save_operation {
            renderer.restore();
        }
    }

    Ok(())
}

fn cached_path_slot_index(
    cache: &mut Vec<(Vec<RuntimePathCommand>, Option<Box<dyn RenderPath>>)>,
    commands: &[RuntimePathCommand],
) -> usize {
    if let Some(index) = cache
        .iter()
        .position(|(candidate, _)| candidate.as_slice() == commands)
    {
        return index;
    }
    cache.push((commands.to_vec(), None));
    cache.len() - 1
}

fn configure_paint(
    render_paint: &mut dyn RenderPaint,
    object: &RuntimeObject,
    paint: &RuntimeShapePaintCommand,
) -> Result<()> {
    match paint.paint_type {
        RuntimeShapePaintKind::Fill => {
            render_paint.style(RenderPaintStyle::Fill);
        }
        RuntimeShapePaintKind::Stroke => {
            render_paint.style(RenderPaintStyle::Stroke);
            render_paint.thickness(object.double_property("thickness").unwrap_or(1.0));
            render_paint.cap(stroke_cap(object.uint_property("cap").unwrap_or(0))?);
            render_paint.join(stroke_join(object.uint_property("join").unwrap_or(0))?);
        }
        RuntimeShapePaintKind::Unknown => bail!("unsupported: unknown shape paint"),
    }
    render_paint.shader(None);
    render_paint.blend_mode(blend_mode(paint.render_blend_mode_value)?);
    match paint.paint_state.as_ref() {
        Some(RuntimeShapePaintState::SolidColor { render_color, .. }) => {
            render_paint.color(*render_color);
        }
        Some(RuntimeShapePaintState::LinearGradient { .. })
        | Some(RuntimeShapePaintState::RadialGradient { .. }) => {
            bail!("unsupported: gradients in Rust golden runner")
        }
        None => {}
    }
    Ok(())
}

fn configure_fill_rule(path: &mut dyn RenderPath, object: &RuntimeObject) {
    if object.type_name == "Fill" {
        path.fill_rule(fill_rule_for_object(object));
    }
}

fn fill_rule_for_object(object: &RuntimeObject) -> FillRule {
    match object.uint_property("fillRule").unwrap_or(0) {
        1 => FillRule::EvenOdd,
        2 => FillRule::Clockwise,
        _ => FillRule::NonZero,
    }
}

fn make_path(
    factory: &mut RecordingFactory,
    commands: &[RuntimePathCommand],
    fill_rule: FillRule,
) -> Box<dyn RenderPath> {
    let mut path = factory.make_empty_render_path();
    path.fill_rule(fill_rule);
    for command in commands {
        match *command {
            RuntimePathCommand::Move { x, y } => path.move_to(x, y),
            RuntimePathCommand::Line { x, y } => path.line_to(x, y),
            RuntimePathCommand::Cubic {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
            } => path.cubic_to(x1, y1, x2, y2, x3, y3),
            RuntimePathCommand::Close => path.close(),
        }
    }
    path
}

fn rect_commands(left: f32, top: f32, right: f32, bottom: f32) -> Vec<RuntimePathCommand> {
    vec![
        RuntimePathCommand::Move { x: left, y: top },
        RuntimePathCommand::Line { x: right, y: top },
        RuntimePathCommand::Line {
            x: right,
            y: bottom,
        },
        RuntimePathCommand::Line { x: left, y: bottom },
        RuntimePathCommand::Close,
    ]
}

fn render_mat(mat: Mat2D) -> RenderMat2D {
    RenderMat2D(mat.0)
}

fn frame_dimension(value: f32) -> u32 {
    value.ceil().max(1.0) as u32
}

fn blend_mode(value: u32) -> Result<BlendMode> {
    Ok(match value {
        3 => BlendMode::SrcOver,
        14 => BlendMode::Screen,
        15 => BlendMode::Overlay,
        16 => BlendMode::Darken,
        17 => BlendMode::Lighten,
        18 => BlendMode::ColorDodge,
        19 => BlendMode::ColorBurn,
        20 => BlendMode::HardLight,
        21 => BlendMode::SoftLight,
        22 => BlendMode::Difference,
        23 => BlendMode::Exclusion,
        24 => BlendMode::Multiply,
        25 => BlendMode::Hue,
        26 => BlendMode::Saturation,
        27 => BlendMode::Color,
        28 => BlendMode::Luminosity,
        _ => bail!("unsupported blend mode {value}"),
    })
}

fn stroke_cap(value: u64) -> Result<StrokeCap> {
    Ok(match value {
        0 => StrokeCap::Butt,
        1 => StrokeCap::Round,
        2 => StrokeCap::Square,
        _ => bail!("unsupported stroke cap {value}"),
    })
}

fn stroke_join(value: u64) -> Result<StrokeJoin> {
    Ok(match value {
        0 => StrokeJoin::Miter,
        1 => StrokeJoin::Round,
        2 => StrokeJoin::Bevel,
        _ => bail!("unsupported stroke join {value}"),
    })
}
