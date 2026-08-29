//! The C++ probe's drawable observations, read from live native owners.
//! Snapshot DTOs never execute, solve, or reconstruct a runtime graph.
#![cfg(feature = "tools")]
use nuxie_render_api::{Mat2D, PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    component::ComponentOccurrenceHandle,
    drawable::{Drawable, RuntimeDrawableOccurrence},
    math::{path_types::PathVerb, raw_path::RawPath},
    shapes::{
        paint::{
            color::color_modulate_opacity,
            effects_container::EffectsContainer,
            feather::Feather,
            gradient_stop::GradientStop,
            linear_gradient::LinearGradient,
            radial_gradient::RadialGradient,
            shape_paint::{ShapePaintPathKind as PaintPathKind, ShapePaintType},
            shape_paint_mutator::ShapePaintMutator,
            solid_color::SolidColor,
        },
        shape::Shape,
    },
};
use nuxie_runtime::{
    Artboard, CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle,
    RuntimeFileHandle,
};
use serde::Deserialize;
mod cpp_probe_support;
use cpp_probe_support::*;

fn push_color_property(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: u32) {
    let key = property_key_for_name(type_name, property_name);
    push_var_uint(bytes, u64::from(key));
    bytes.extend_from_slice(&value.to_le_bytes());
}

#[derive(Debug, Deserialize)]
struct CppProbeFile {
    artboards: Vec<CppArtboard>,
}
#[derive(Debug, Deserialize)]
struct CppArtboard {
    #[serde(rename = "drawCommandStream")]
    draw_command_stream: Vec<CppDrawCommand>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DrawKind {
    Draw,
    ClipStart,
    ClipEnd,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PaintKind {
    Fill,
    Stroke,
    Unknown,
}
#[derive(Debug, Clone, PartialEq)]
enum PaintState {
    SolidColor {
        color: u32,
        render_color: u32,
    },
    LinearGradient {
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
        opacity: f32,
        render_opacity: f32,
        stops: Vec<GradientStopState>,
    },
    RadialGradient {
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
        opacity: f32,
        render_opacity: f32,
        stops: Vec<GradientStopState>,
    },
}
#[derive(Debug, Clone, Copy, PartialEq)]
struct GradientStopState {
    color: u32,
    render_color: u32,
    position: f32,
}
#[derive(Debug, Clone, PartialEq)]
struct FeatherState {
    feather_local: usize,
    space_value: u32,
    strength: f32,
    offset_x: f32,
    offset_y: f32,
    inner: bool,
    inner_path_commands: Vec<PathCommand>,
}
#[derive(Debug, Clone, Copy, PartialEq)]
enum PathCommand {
    Move {
        x: f32,
        y: f32,
    },
    Line {
        x: f32,
        y: f32,
    },
    Cubic {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
    },
    Close,
}
#[derive(Debug)]
struct PaintObservation {
    paint_local: usize,
    mutator_local: Option<usize>,
    paint_type: PaintKind,
    path_kind: PaintPathKind,
    blend_mode_value: u32,
    render_blend_mode_value: u32,
    paint_state: Option<PaintState>,
    feather_state: Option<FeatherState>,
    path_commands: Vec<PathCommand>,
    effect_path_commands: Vec<PathCommand>,
    needs_save_operation: bool,
    paint_space_transform: Option<Mat2D>,
}
#[derive(Debug)]
struct DrawObservation {
    local_id: Option<usize>,
    kind: DrawKind,
    needs_save_operation: bool,
    shape_paints: Vec<PaintObservation>,
}

fn read_native_instance_from_bytes(
    bytes: &[u8],
    label: &str,
) -> (RuntimeFileHandle, RuntimeArtboardInstanceHandle) {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let retained =
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained native factory");
    let file = File::import(bytes, retained, None, None, None)
        .unwrap_or_else(|| panic!("failed to import {label}"));
    let source = file
        .with_file(|file| file.artboard())
        .expect("source artboard");
    let instance = Artboard::instance_from_handle(&source)
        .unwrap_or_else(|| panic!("failed to instantiate {label}"));
    (file, instance)
}

fn local_id(objects: &[Option<CoreHandle>], owner: &CoreHandle) -> Option<usize> {
    objects
        .iter()
        .position(|object| object.as_ref() == Some(owner))
}

fn path_commands(path: &RawPath) -> Vec<PathCommand> {
    path.segments()
        .into_iter()
        .map(|segment| {
            let p = segment.points;
            match segment.verb {
                PathVerb::Move => PathCommand::Move {
                    x: p[0].x,
                    y: p[0].y,
                },
                PathVerb::Line => PathCommand::Line {
                    x: p[1].x,
                    y: p[1].y,
                },
                PathVerb::Cubic => PathCommand::Cubic {
                    x1: p[1].x,
                    y1: p[1].y,
                    x2: p[2].x,
                    y2: p[2].y,
                    x3: p[3].x,
                    y3: p[3].y,
                },
                PathVerb::Close => PathCommand::Close,
                PathVerb::Quad => panic!("these authored path fixtures contain no quadratic verb"),
            }
        })
        .collect()
}

fn gradient_state(gradient: &LinearGradient, radial: bool) -> PaintState {
    let render_opacity = gradient.render_opacity();
    let opacity = gradient.opacity();
    let stops = gradient
        .stops()
        .iter()
        .map(|stop| {
            stop.with_downcast::<GradientStop, _>(|stop| {
                let color = stop.color_value() as u32;
                let position = stop.position();
                GradientStopState {
                    color,
                    render_color: color_modulate_opacity(color, opacity * render_opacity),
                    // Exact std::max(0,std::min(position,1)) wire observation.
                    position: if position.is_nan() {
                        0.0
                    } else {
                        position.clamp(0.0, 1.0)
                    },
                }
            })
            .expect("actual gradient stop")
        })
        .collect();
    if radial {
        PaintState::RadialGradient {
            start_x: gradient.start_x(),
            start_y: gradient.start_y(),
            end_x: gradient.end_x(),
            end_y: gradient.end_y(),
            opacity,
            render_opacity,
            stops,
        }
    } else {
        PaintState::LinearGradient {
            start_x: gradient.start_x(),
            start_y: gradient.start_y(),
            end_x: gradient.end_x(),
            end_y: gradient.end_y(),
            opacity,
            render_opacity,
            stops,
        }
    }
}

fn paint_state(mutator: &CoreHandle) -> Option<PaintState> {
    mutator
        .with(|mutator| {
            if let Some(solid) = mutator.as_any().downcast_ref::<SolidColor>() {
                let color = solid.color_value() as u32;
                Some(PaintState::SolidColor {
                    color,
                    render_color: color_modulate_opacity(color, solid.render_opacity()),
                })
            } else if let Some(gradient) = mutator.as_any().downcast_ref::<LinearGradient>() {
                Some(gradient_state(gradient, false))
            } else {
                mutator
                    .as_any()
                    .downcast_ref::<RadialGradient>()
                    .map(|gradient| gradient_state(&gradient.base.base, true))
            }
        })
        .flatten()
}

fn observe_paints(
    shape: &CoreHandle,
    objects: &[Option<CoreHandle>],
    needs_save: bool,
) -> Vec<PaintObservation> {
    let (paints, shape_blend) = shape
        .with_downcast::<Shape, _>(|shape| {
            (
                shape.paint_container.shape_paints().to_vec(),
                shape.blend_mode_value(),
            )
        })
        .expect("Shape paint container");
    let needs_save = needs_save || paints.len() > 1;
    paints
        .into_iter()
        .filter_map(|identity| {
            let properties = identity
                .with(|paint| {
                    let behavior = paint
                        .as_shape_paint_behavior()
                        .expect("actual shape paint behavior");
                    if !behavior.is_visible() {
                        return None;
                    }
                    let paint = behavior.shape_paint();
                    Some((
                        behavior.paint_type(),
                        behavior.pick_path_kind(),
                        paint.paint(),
                        paint.feather(),
                        paint.blend_mode_value(),
                    ))
                })
                .flatten()?;
            let (paint_type, path_kind, mutator, feather, blend) = properties;
            let paths = shape
                .with_downcast::<Shape, _>(|shape| {
                    shape.with_path_mut(path_kind, |path| path_commands(path.raw_path()))
                })
                .expect("actual selected shape path");
            let effect_path_commands = identity
                .with_mut(|paint| {
                    let paint = paint
                        .as_shape_paint_mut()
                        .expect("actual paint effects owner");
                    let provider = *paint.path_provider();
                    paint
                        .last_effect_path(&provider)
                        .map(|path| path_commands(path.borrow().raw_path()))
                        .unwrap_or_default()
                })
                .expect("actual retained paint");
            let feather_state = feather.map(|feather| {
                feather
                    .with_downcast::<Feather, _>(|value| FeatherState {
                        feather_local: local_id(objects, &feather)
                            .expect("authored Feather identity"),
                        space_value: value.space_value(),
                        strength: value.strength(),
                        offset_x: value.offset_x(),
                        offset_y: value.offset_y(),
                        inner: value.is_inner(),
                        inner_path_commands: if value.is_inner() {
                            path_commands(value.inner_path().borrow().raw_path())
                        } else {
                            Vec::new()
                        },
                    })
                    .expect("actual Feather")
            });
            let paint_state = mutator.as_ref().and_then(paint_state);
            let paint_space_transform = if path_kind == PaintPathKind::World
                && matches!(
                    paint_state,
                    Some(PaintState::LinearGradient { .. } | PaintState::RadialGradient { .. })
                ) {
                // The world transform of the actual gradient's owning paint
                // container is the matrix used by LinearGradient::apply_to.
                Some(
                    shape
                        .with_downcast::<Shape, _>(|shape| {
                            Mat2D(*shape.shape_world_transform().values())
                        })
                        .unwrap(),
                )
            } else {
                None
            };
            Some(PaintObservation {
                paint_local: local_id(objects, &identity).expect("authored paint identity"),
                mutator_local: mutator
                    .as_ref()
                    .and_then(|mutator| local_id(objects, mutator)),
                paint_type: match paint_type {
                    ShapePaintType::Fill => PaintKind::Fill,
                    ShapePaintType::Stroke => PaintKind::Stroke,
                },
                path_kind,
                blend_mode_value: blend,
                render_blend_mode_value: if blend == 127 { shape_blend } else { blend },
                paint_state,
                feather_state,
                path_commands: paths,
                effect_path_commands,
                needs_save_operation: needs_save,
                paint_space_transform,
            })
        })
        .collect()
}

fn observe_draw(
    drawable: &RuntimeDrawableOccurrence,
    objects: &[Option<CoreHandle>],
) -> DrawObservation {
    let needs_save = drawable
        .with(Drawable::needs_save_operation)
        .expect("retained drawable");
    let identity = drawable.authored_handle();
    let shape_paints = identity
        .as_ref()
        .filter(|owner| {
            owner.is_type_of(
                nuxie_runtime::source::generated::shapes::shape_base::ShapeBase::TYPE_KEY,
            )
        })
        .map(|shape| observe_paints(shape, objects, needs_save))
        .unwrap_or_default();
    DrawObservation {
        local_id: identity.as_ref().and_then(|owner| local_id(objects, owner)),
        kind: if drawable.is_clip_start() {
            DrawKind::ClipStart
        } else if drawable.is_clip_end() {
            DrawKind::ClipEnd
        } else {
            DrawKind::Draw
        },
        needs_save_operation: needs_save,
        shape_paints,
    }
}

fn observe_draw_stream(artboard: &RuntimeArtboardInstanceHandle) -> Vec<DrawObservation> {
    let (objects, mut current) =
        artboard.with_artboard(|artboard| (artboard.objects().to_vec(), artboard.first_drawable()));
    let mut result = Vec::new();
    let mut empty_clips = 0;
    let mut pending = Vec::new();
    // Mirrors only cpp-probe::write_draw_command_stream's observation walk.
    // Sorting, geometry, willDraw, clip counts and paint choices are all
    // supplied by actual translated owners, never recomputed from a graph.
    while let Some(drawable) = current {
        current = drawable.with(Drawable::prev_drawable).flatten();
        let previous = empty_clips;
        empty_clips += drawable.empty_clip_count();
        if !drawable.will_draw() || empty_clips != previous || empty_clips > 0 {
            continue;
        }
        if drawable.is_clip_start() {
            pending.push(drawable);
            continue;
        } else if !pending.is_empty() {
            if drawable.is_clip_end() {
                pending.pop();
                continue;
            } else {
                for pending in pending.drain(..) {
                    result.push(observe_draw(&pending, &objects));
                }
            }
        }
        result.push(observe_draw(&drawable, &objects));
    }
    result
}

#[derive(Debug, Deserialize)]
struct CppDrawCommand {
    #[serde(rename = "localId")]
    local_id: Option<usize>,
    #[serde(rename = "isClipStart")]
    is_clip_start: bool,
    #[serde(rename = "isClipEnd")]
    is_clip_end: bool,
    #[serde(rename = "needsSaveOperation")]
    needs_save_operation: bool,
    #[serde(default, rename = "shapePaintCommands")]
    shape_paint_commands: Vec<CppShapePaintCommand>,
}

impl CppDrawCommand {
    fn kind(&self) -> DrawKind {
        if self.is_clip_start {
            DrawKind::ClipStart
        } else if self.is_clip_end {
            DrawKind::ClipEnd
        } else {
            DrawKind::Draw
        }
    }
}

#[derive(Debug, Deserialize)]
struct CppShapePaintCommand {
    #[serde(rename = "paintLocal")]
    paint_local: Option<usize>,
    #[serde(rename = "mutatorLocal")]
    mutator_local: Option<usize>,
    #[serde(rename = "paintType")]
    paint_type: String,
    #[serde(rename = "pathKind")]
    path_kind: String,
    #[serde(rename = "blendModeValue")]
    blend_mode_value: u32,
    #[serde(rename = "renderBlendModeValue")]
    render_blend_mode_value: u32,
    #[serde(rename = "paintState")]
    paint_state: Option<CppShapePaintState>,
    #[serde(default)]
    feather: Option<CppFeatherState>,
    #[serde(default, rename = "pathCommands")]
    path_commands: Vec<CppPathCommand>,
    #[serde(default, rename = "effectPathCommands")]
    effect_path_commands: Vec<CppPathCommand>,
    #[serde(rename = "needsSaveOperation")]
    needs_save_operation: bool,
}

impl CppShapePaintCommand {
    fn paint_type(&self) -> PaintKind {
        match self.paint_type.as_str() {
            "fill" => PaintKind::Fill,
            "stroke" => PaintKind::Stroke,
            _ => PaintKind::Unknown,
        }
    }

    fn path_kind(&self) -> PaintPathKind {
        match self.path_kind.as_str() {
            "local" => PaintPathKind::Local,
            "localClockwise" => PaintPathKind::LocalClockwise,
            "world" => PaintPathKind::World,
            other => panic!("unexpected C++ shape paint path kind {other}"),
        }
    }

    fn paint_state(&self) -> Option<PaintState> {
        let state = self.paint_state.as_ref()?;
        match state.kind.as_str() {
            "solidColor" => Some(PaintState::SolidColor {
                color: state.color,
                render_color: state.render_color,
            }),
            "linearGradient" => Some(PaintState::LinearGradient {
                start_x: state.start_x,
                start_y: state.start_y,
                end_x: state.end_x,
                end_y: state.end_y,
                opacity: state.opacity,
                render_opacity: state.render_opacity,
                stops: state
                    .stops
                    .iter()
                    .map(CppGradientStopState::gradient_stop)
                    .collect(),
            }),
            "radialGradient" => Some(PaintState::RadialGradient {
                start_x: state.start_x,
                start_y: state.start_y,
                end_x: state.end_x,
                end_y: state.end_y,
                opacity: state.opacity,
                render_opacity: state.render_opacity,
                stops: state
                    .stops
                    .iter()
                    .map(CppGradientStopState::gradient_stop)
                    .collect(),
            }),
            other => panic!("unexpected C++ shape paint state kind {other}"),
        }
    }

    fn feather_state(&self) -> Option<FeatherState> {
        self.feather.as_ref().map(CppFeatherState::feather_state)
    }

    fn path_commands(&self) -> Vec<PathCommand> {
        self.path_commands
            .iter()
            .map(CppPathCommand::path_command)
            .collect()
    }

    fn effect_path_commands(&self) -> Vec<PathCommand> {
        self.effect_path_commands
            .iter()
            .map(CppPathCommand::path_command)
            .collect()
    }
}

#[derive(Debug, Deserialize)]
struct CppShapePaintState {
    kind: String,
    #[serde(default, rename = "startX")]
    start_x: f32,
    #[serde(default, rename = "startY")]
    start_y: f32,
    #[serde(default, rename = "endX")]
    end_x: f32,
    #[serde(default, rename = "endY")]
    end_y: f32,
    #[serde(default)]
    opacity: f32,
    #[serde(default, rename = "renderOpacity")]
    render_opacity: f32,
    #[serde(default)]
    stops: Vec<CppGradientStopState>,
    #[serde(default)]
    color: u32,
    #[serde(default, rename = "renderColor")]
    render_color: u32,
}

#[derive(Debug, Deserialize)]
struct CppGradientStopState {
    color: u32,
    #[serde(rename = "renderColor")]
    render_color: u32,
    position: f32,
}

impl CppGradientStopState {
    fn gradient_stop(&self) -> GradientStopState {
        GradientStopState {
            color: self.color,
            render_color: self.render_color,
            position: self.position,
        }
    }
}

#[derive(Debug, Deserialize)]
struct CppFeatherState {
    local: usize,
    #[serde(rename = "spaceValue")]
    space_value: u32,
    strength: f32,
    #[serde(rename = "offsetX")]
    offset_x: f32,
    #[serde(rename = "offsetY")]
    offset_y: f32,
    inner: bool,
    #[serde(default, rename = "innerPathCommands")]
    inner_path_commands: Vec<CppPathCommand>,
}

impl CppFeatherState {
    fn feather_state(&self) -> FeatherState {
        FeatherState {
            feather_local: self.local,
            space_value: self.space_value,
            strength: self.strength,
            offset_x: self.offset_x,
            offset_y: self.offset_y,
            inner: self.inner,
            inner_path_commands: self
                .inner_path_commands
                .iter()
                .map(CppPathCommand::path_command)
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct CppPathCommand {
    verb: String,
    points: Vec<[f32; 2]>,
}

impl CppPathCommand {
    fn path_command(&self) -> PathCommand {
        match self.verb.as_str() {
            "move" => PathCommand::Move {
                x: self.points[0][0],
                y: self.points[0][1],
            },
            "line" => PathCommand::Line {
                x: self.points[0][0],
                y: self.points[0][1],
            },
            "cubic" => PathCommand::Cubic {
                x1: self.points[0][0],
                y1: self.points[0][1],
                x2: self.points[1][0],
                y2: self.points[1][1],
                x3: self.points[2][0],
                y3: self.points[2][1],
            },
            "close" => PathCommand::Close,
            other => panic!("unexpected C++ raw path verb {other}"),
        }
    }
}

fn assert_path_commands_close(actual: &[PathCommand], expected: &[PathCommand], label: &str) {
    assert_eq!(
        actual.len(),
        expected.len(),
        "{label} command count mismatch: expected {expected:?}, got {actual:?}"
    );
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        let point_label = |field: &str| format!("{label}[{index}].{field}");
        match (actual, expected) {
            (
                PathCommand::Move {
                    x: actual_x,
                    y: actual_y,
                },
                PathCommand::Move {
                    x: expected_x,
                    y: expected_y,
                },
            )
            | (
                PathCommand::Line {
                    x: actual_x,
                    y: actual_y,
                },
                PathCommand::Line {
                    x: expected_x,
                    y: expected_y,
                },
            ) => {
                assert_close(*actual_x, *expected_x, &point_label("x"));
                assert_close(*actual_y, *expected_y, &point_label("y"));
            }
            (
                PathCommand::Cubic {
                    x1: actual_x1,
                    y1: actual_y1,
                    x2: actual_x2,
                    y2: actual_y2,
                    x3: actual_x3,
                    y3: actual_y3,
                },
                PathCommand::Cubic {
                    x1: expected_x1,
                    y1: expected_y1,
                    x2: expected_x2,
                    y2: expected_y2,
                    x3: expected_x3,
                    y3: expected_y3,
                },
            ) => {
                assert_close(*actual_x1, *expected_x1, &point_label("x1"));
                assert_close(*actual_y1, *expected_y1, &point_label("y1"));
                assert_close(*actual_x2, *expected_x2, &point_label("x2"));
                assert_close(*actual_y2, *expected_y2, &point_label("y2"));
                assert_close(*actual_x3, *expected_x3, &point_label("x3"));
                assert_close(*actual_y3, *expected_y3, &point_label("y3"));
            }
            (PathCommand::Close, PathCommand::Close) => {}
            _ => panic!("{label}[{index}] command mismatch: expected {expected:?}, got {actual:?}"),
        }
    }
}

#[test]
fn runtime_drawable_dispatch_stream_filters_hidden_and_opacity_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_drawable_dispatch_filtering.riv";
    let bytes = synthetic_runtime_file(8201, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
        });
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
            push_f32_property(bytes, "Node", "opacity", 0.0);
        });
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
            push_uint_property(bytes, "Drawable", "drawableFlags", 1);
        });
    });

    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    Artboard::update_components_handle(&rust.core_handle());

    let rust_commands = observe_draw_stream(&rust)
        .into_iter()
        .map(|command| (command.local_id, command.kind, command.needs_save_operation))
        .collect::<Vec<_>>();
    let cpp_commands = cpp.artboards[0]
        .draw_command_stream
        .iter()
        .map(|command| {
            (
                command.local_id,
                command.kind(),
                command.needs_save_operation,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        cpp_commands,
        vec![(Some(1), DrawKind::Draw, true)],
        "C++ draw command stream should skip hidden and opacity-zero shapes"
    );
    assert_eq!(
        rust_commands, cpp_commands,
        "Rust runtime draw command stream should match C++ willDraw filtering for simple shapes"
    );
}

#[test]
fn runtime_drawable_dispatch_stream_filters_image_and_nested_artboard_will_draw_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_drawable_will_draw_prereqs.riv";
    let bytes = synthetic_runtime_file(8232, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "ImageAsset", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
        });
        push_object_with_properties(bytes, "Image", |bytes| {
            push_uint_property(bytes, "Image", "parentId", 0);
        });
        push_object_with_properties(bytes, "Image", |bytes| {
            push_uint_property(bytes, "Image", "parentId", 0);
            push_uint_property(bytes, "Image", "assetId", 0);
        });
        push_object_with_properties(bytes, "Image", |bytes| {
            push_uint_property(bytes, "Image", "parentId", 0);
            push_uint_property(bytes, "Image", "assetId", 1);
        });
        push_object_with_properties(bytes, "NestedArtboard", |bytes| {
            push_uint_property(bytes, "NestedArtboard", "parentId", 0);
            push_uint_property(bytes, "NestedArtboard", "artboardId", 1);
        });
        push_object_with_properties(bytes, "NestedArtboard", |bytes| {
            push_uint_property(bytes, "NestedArtboard", "parentId", 0);
            push_uint_property(bytes, "NestedArtboard", "artboardId", 99);
        });
        push_object_with_properties(bytes, "Artboard", |_| {});
    });

    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    Artboard::update_components_handle(&rust.core_handle());

    let rust_commands = observe_draw_stream(&rust)
        .into_iter()
        .map(|command| (command.local_id, command.kind, command.needs_save_operation))
        .collect::<Vec<_>>();
    let cpp_commands = cpp.artboards[0]
        .draw_command_stream
        .iter()
        .map(|command| {
            (
                command.local_id,
                command.kind(),
                command.needs_save_operation,
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        cpp_commands,
        vec![
            (Some(5), DrawKind::Draw, true),
            (Some(3), DrawKind::Draw, true),
            (Some(1), DrawKind::Draw, true),
        ],
        "C++ should only draw the plain shape, image with ImageAsset, and nested artboard with a resolved artboard"
    );
    assert_eq!(
        rust_commands, cpp_commands,
        "Rust runtime draw command stream should match C++ type-specific willDraw prerequisites"
    );
}

#[test]
fn runtime_drawable_dispatch_stream_suppresses_empty_clips_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_empty_clip_filtering.riv";
    let bytes = synthetic_runtime_file(8202, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
        });
        push_object_with_properties(bytes, "Node", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
        });
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 2);
        });
        push_object_with_properties(bytes, "ClippingShape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 2);
            push_uint_property(bytes, "ClippingShape", "sourceId", 1);
        });
    });

    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    Artboard::update_components_handle(&rust.core_handle());

    let rust_commands = observe_draw_stream(&rust)
        .into_iter()
        .map(|command| (command.local_id, command.kind, command.needs_save_operation))
        .collect::<Vec<_>>();
    let cpp_commands = cpp.artboards[0]
        .draw_command_stream
        .iter()
        .map(|command| {
            (
                command.local_id,
                command.kind(),
                command.needs_save_operation,
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        cpp_commands,
        vec![(Some(1), DrawKind::Draw, true)],
        "C++ draw command stream should suppress drawables inside an empty visible clipping shape"
    );
    assert_eq!(
        rust_commands, cpp_commands,
        "Rust runtime draw command stream should match C++ empty-clip suppression for source shapes with no paths"
    );
}

#[test]
fn runtime_drawable_dispatch_stream_treats_hidden_clip_paths_as_empty_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_hidden_clip_path_filtering.riv";
    let bytes = synthetic_runtime_file(8203, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
        });
        push_object_with_properties(bytes, "PointsPath", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 1);
            push_uint_property(bytes, "Path", "pathFlags", 1);
        });
        push_object_with_properties(bytes, "Node", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
        });
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 3);
        });
        push_object_with_properties(bytes, "ClippingShape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 3);
            push_uint_property(bytes, "ClippingShape", "sourceId", 1);
        });
    });

    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    Artboard::update_components_handle(&rust.core_handle());

    let rust_commands = observe_draw_stream(&rust)
        .into_iter()
        .map(|command| (command.local_id, command.kind, command.needs_save_operation))
        .collect::<Vec<_>>();
    let cpp_commands = cpp.artboards[0]
        .draw_command_stream
        .iter()
        .map(|command| {
            (
                command.local_id,
                command.kind(),
                command.needs_save_operation,
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        cpp_commands,
        vec![(Some(1), DrawKind::Draw, true)],
        "C++ draw command stream should suppress drawables clipped by a source shape with only hidden paths"
    );
    assert_eq!(
        rust_commands, cpp_commands,
        "Rust runtime draw command stream should match C++ empty-clip suppression for hidden source paths"
    );
}

#[test]
fn runtime_drawable_dispatch_stream_treats_collapsed_clip_paths_as_empty_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_collapsed_clip_path_filtering.riv";
    let bytes = synthetic_runtime_file(8204, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
        });
        push_object_with_properties(bytes, "PointsPath", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 1);
        });
        push_object_with_properties(bytes, "Node", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
        });
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 3);
        });
        push_object_with_properties(bytes, "ClippingShape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 3);
            push_uint_property(bytes, "ClippingShape", "sourceId", 1);
        });
    });

    let cpp = read_cpp_probe_bytes_with_args(
        &probe,
        label,
        &bytes,
        &[
            "--runtime-collapse-component".to_owned(),
            "2".to_owned(),
            "true".to_owned(),
        ],
    );
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    let path = rust
        .with_artboard(|artboard| artboard.resolve_handle(2))
        .expect("actual path occurrence");
    assert!(ComponentOccurrenceHandle::Authored(path).collapse(true));
    Artboard::update_components_handle(&rust.core_handle());

    let rust_commands = observe_draw_stream(&rust)
        .into_iter()
        .map(|command| (command.local_id, command.kind, command.needs_save_operation))
        .collect::<Vec<_>>();
    let cpp_commands = cpp.artboards[0]
        .draw_command_stream
        .iter()
        .map(|command| {
            (
                command.local_id,
                command.kind(),
                command.needs_save_operation,
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        cpp_commands,
        vec![(Some(1), DrawKind::Draw, true)],
        "C++ draw command stream should suppress drawables clipped by a source shape with only collapsed paths"
    );
    assert_eq!(
        rust_commands, cpp_commands,
        "Rust runtime draw command stream should match C++ empty-clip suppression for collapsed source paths"
    );
}

#[test]
fn runtime_drawable_dispatch_stream_exposes_shape_paint_payloads_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_shape_paint_payloads.riv";
    let bytes = synthetic_runtime_file(8205, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
            push_f32_property(bytes, "Node", "x", 100.0);
            push_f32_property(bytes, "Node", "opacity", 0.5);
            push_uint_property(bytes, "Drawable", "blendModeValue", 24);
        });
        push_object_with_properties(bytes, "Fill", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 1);
            push_uint_property(bytes, "Fill", "fillRule", 2);
        });
        push_object_with_properties(bytes, "SolidColor", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 2);
            push_color_property(bytes, "SolidColor", "colorValue", 0x8040_2010);
        });
        push_object_with_properties(bytes, "Fill", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 1);
            push_bool_property(bytes, "ShapePaint", "isVisible", false);
        });
        push_object_with_properties(bytes, "SolidColor", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 4);
        });
        push_object_with_properties(bytes, "Stroke", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 1);
            push_uint_property(bytes, "ShapePaint", "blendModeValue", 14);
            push_bool_property(bytes, "Stroke", "transformAffectsStroke", false);
        });
        push_object_with_properties(bytes, "SolidColor", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 6);
            push_color_property(bytes, "SolidColor", "colorValue", 0xff11_2233);
        });
        push_object_with_properties(bytes, "Stroke", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 1);
            push_f32_property(bytes, "Stroke", "thickness", 0.0);
        });
        push_object_with_properties(bytes, "SolidColor", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 8);
        });
        push_object_with_properties(bytes, "PointsPath", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 1);
            push_f32_property(bytes, "Node", "x", 10.0);
            push_bool_property(bytes, "PointsCommonPath", "isClosed", true);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 10);
            push_f32_property(bytes, "Vertex", "x", 0.0);
            push_f32_property(bytes, "Vertex", "y", 0.0);
        });
        push_object_with_properties(bytes, "CubicAsymmetricVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 10);
            push_f32_property(bytes, "Vertex", "x", 10.0);
            push_f32_property(bytes, "Vertex", "y", 0.0);
            push_f32_property(bytes, "CubicAsymmetricVertex", "inDistance", 5.0);
            push_f32_property(bytes, "CubicAsymmetricVertex", "outDistance", 5.0);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 10);
            push_f32_property(bytes, "Vertex", "x", 10.0);
            push_f32_property(bytes, "Vertex", "y", 20.0);
        });
    });

    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    Artboard::update_components_handle(&rust.core_handle());

    let rust_payloads = observe_draw_stream(&rust)
        .into_iter()
        .flat_map(|command| command.shape_paints)
        .map(|paint| {
            (
                Some(paint.paint_local),
                paint.mutator_local,
                paint.paint_type,
                paint.path_kind,
                paint.blend_mode_value,
                paint.render_blend_mode_value,
                paint.paint_state,
                paint.feather_state,
                paint.path_commands,
                paint.needs_save_operation,
            )
        })
        .collect::<Vec<_>>();
    let cpp_payloads = cpp.artboards[0]
        .draw_command_stream
        .iter()
        .flat_map(|command| command.shape_paint_commands.iter())
        .map(|paint| {
            (
                paint.paint_local,
                paint.mutator_local,
                paint.paint_type(),
                paint.path_kind(),
                paint.blend_mode_value,
                paint.render_blend_mode_value,
                paint.paint_state(),
                paint.feather_state(),
                paint.path_commands(),
                paint.needs_save_operation,
            )
        })
        .collect::<Vec<_>>();
    let expected_local_path_commands = vec![
        PathCommand::Move { x: 10.0, y: 0.0 },
        PathCommand::Cubic {
            x1: 10.0,
            y1: 0.0,
            x2: 15.0,
            y2: 0.0,
            x3: 20.0,
            y3: 0.0,
        },
        PathCommand::Cubic {
            x1: 25.0,
            y1: 0.0,
            x2: 20.0,
            y2: 20.0,
            x3: 20.0,
            y3: 20.0,
        },
        PathCommand::Line { x: 10.0, y: 0.0 },
        PathCommand::Close,
    ];
    let expected_world_path_commands = vec![
        PathCommand::Move { x: 110.0, y: 0.0 },
        PathCommand::Cubic {
            x1: 110.0,
            y1: 0.0,
            x2: 115.0,
            y2: 0.0,
            x3: 120.0,
            y3: 0.0,
        },
        PathCommand::Cubic {
            x1: 125.0,
            y1: 0.0,
            x2: 120.0,
            y2: 20.0,
            x3: 120.0,
            y3: 20.0,
        },
        PathCommand::Line { x: 110.0, y: 0.0 },
        PathCommand::Close,
    ];

    assert_eq!(
        cpp_payloads,
        vec![
            (
                Some(2),
                Some(3),
                PaintKind::Fill,
                PaintPathKind::LocalClockwise,
                127,
                24,
                Some(PaintState::SolidColor {
                    color: 0x8040_2010,
                    render_color: 0x4040_2010,
                }),
                None,
                expected_local_path_commands,
                true
            ),
            (
                Some(6),
                Some(7),
                PaintKind::Stroke,
                PaintPathKind::World,
                14,
                14,
                Some(PaintState::SolidColor {
                    color: 0xff11_2233,
                    render_color: 0x8011_2233,
                }),
                None,
                expected_world_path_commands,
                true
            ),
        ],
        "C++ shape payloads should include visible paints and skip invisible/zero-thickness paints"
    );
    assert_eq!(
        rust_payloads, cpp_payloads,
        "Rust shape paint command payloads should match C++ Shape::draw paint filtering and path selection"
    );
}

#[test]
fn runtime_drawable_dispatch_stream_exposes_rounded_point_path_payloads_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_rounded_point_path_payloads.riv";
    let bytes = synthetic_runtime_file(8214, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
        });
        push_object_with_properties(bytes, "Fill", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 1);
            push_uint_property(bytes, "Fill", "fillRule", 2);
        });
        push_object_with_properties(bytes, "SolidColor", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 2);
            push_color_property(bytes, "SolidColor", "colorValue", 0xffa0_3020);
        });
        push_object_with_properties(bytes, "PointsPath", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 1);
            push_bool_property(bytes, "PointsCommonPath", "isClosed", true);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 4);
            push_f32_property(bytes, "Vertex", "x", 0.0);
            push_f32_property(bytes, "Vertex", "y", 0.0);
            push_f32_property(bytes, "StraightVertex", "radius", 2.0);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 4);
            push_f32_property(bytes, "Vertex", "x", 10.0);
            push_f32_property(bytes, "Vertex", "y", 0.0);
            push_f32_property(bytes, "StraightVertex", "radius", -2.0);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 4);
            push_f32_property(bytes, "Vertex", "x", 10.0);
            push_f32_property(bytes, "Vertex", "y", 10.0);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 4);
            push_f32_property(bytes, "Vertex", "x", 0.0);
            push_f32_property(bytes, "Vertex", "y", 10.0);
            push_f32_property(bytes, "StraightVertex", "radius", 2.0);
        });
    });

    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    Artboard::update_components_handle(&rust.core_handle());

    let rust_paints = observe_draw_stream(&rust)
        .into_iter()
        .flat_map(|command| command.shape_paints)
        .collect::<Vec<_>>();
    let cpp_paints = cpp.artboards[0]
        .draw_command_stream
        .iter()
        .flat_map(|command| command.shape_paint_commands.iter())
        .collect::<Vec<_>>();

    assert_eq!(
        cpp_paints.len(),
        1,
        "C++ should emit one visible shape paint"
    );
    assert_eq!(
        rust_paints.len(),
        1,
        "Rust should emit one visible shape paint"
    );

    let cpp_paint = cpp_paints[0];
    let rust_paint = &rust_paints[0];
    assert_eq!(cpp_paint.paint_local, Some(rust_paint.paint_local));
    assert_eq!(cpp_paint.mutator_local, rust_paint.mutator_local);
    assert_eq!(cpp_paint.paint_type(), rust_paint.paint_type);
    assert_eq!(cpp_paint.path_kind(), rust_paint.path_kind);
    assert_eq!(cpp_paint.blend_mode_value, rust_paint.blend_mode_value);
    assert_eq!(
        cpp_paint.render_blend_mode_value,
        rust_paint.render_blend_mode_value
    );
    assert_eq!(cpp_paint.paint_state(), rust_paint.paint_state);
    assert_eq!(cpp_paint.feather_state(), rust_paint.feather_state);
    assert_eq!(
        cpp_paint.needs_save_operation,
        rust_paint.needs_save_operation
    );

    let expected_path_commands = vec![
        PathCommand::Move { x: 0.0, y: 2.0 },
        PathCommand::Cubic {
            x1: 0.0,
            y1: 0.895_430_3,
            x2: 0.895_430_3,
            y2: 0.0,
            x3: 2.0,
            y3: 0.0,
        },
        PathCommand::Line { x: 8.0, y: 0.0 },
        PathCommand::Cubic {
            x1: 8.0,
            y1: 1.104_569_4,
            x2: 8.895_431,
            y2: 2.0,
            x3: 10.0,
            y3: 2.0,
        },
        PathCommand::Line { x: 10.0, y: 10.0 },
        PathCommand::Line { x: 2.0, y: 10.0 },
        PathCommand::Cubic {
            x1: 0.895_430_3,
            y1: 10.0,
            x2: 0.0,
            y2: 9.104_569,
            x3: 0.0,
            y3: 8.0,
        },
        PathCommand::Line { x: 0.0, y: 2.0 },
        PathCommand::Close,
    ];
    let cpp_path_commands = cpp_paint.path_commands();
    assert_path_commands_close(
        &cpp_path_commands,
        &expected_path_commands,
        "C++ rounded point path commands",
    );
    assert_path_commands_close(
        &rust_paint.path_commands,
        &cpp_path_commands,
        "Rust rounded point path commands",
    );
}

#[test]
fn runtime_drawable_dispatch_stream_deforms_weighted_points_path_payloads_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_weighted_points_path_payloads.riv";
    let bytes = synthetic_runtime_file(8221, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "RootBone", |bytes| {
            push_uint_property(bytes, "RootBone", "parentId", 0);
            push_f32_property(bytes, "RootBone", "x", 10.0);
            push_f32_property(bytes, "RootBone", "y", 20.0);
        });
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
        });
        push_object_with_properties(bytes, "Fill", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 2);
            push_uint_property(bytes, "Fill", "fillRule", 2);
        });
        push_object_with_properties(bytes, "SolidColor", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 3);
            push_color_property(bytes, "SolidColor", "colorValue", 0xff24_68ac);
        });
        push_object_with_properties(bytes, "PointsPath", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 2);
            push_bool_property(bytes, "PointsCommonPath", "isClosed", true);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 5);
            push_f32_property(bytes, "Vertex", "x", 1.0);
            push_f32_property(bytes, "Vertex", "y", 2.0);
        });
        push_object_with_properties(bytes, "Weight", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 6);
            push_uint_property(bytes, "Weight", "values", 255);
            push_uint_property(bytes, "Weight", "indices", 1);
        });
        push_object_with_properties(bytes, "CubicMirroredVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 5);
            push_f32_property(bytes, "Vertex", "x", 5.0);
            push_f32_property(bytes, "Vertex", "y", 2.0);
            push_f32_property(bytes, "CubicMirroredVertex", "distance", 2.0);
        });
        push_object_with_properties(bytes, "CubicWeight", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 8);
            push_uint_property(bytes, "Weight", "values", 255);
            push_uint_property(bytes, "Weight", "indices", 1);
            push_uint_property(bytes, "CubicWeight", "inValues", 255);
            push_uint_property(bytes, "CubicWeight", "inIndices", 1);
            push_uint_property(bytes, "CubicWeight", "outValues", 255);
            push_uint_property(bytes, "CubicWeight", "outIndices", 1);
        });
        push_object_with_properties(bytes, "Skin", |bytes| {
            push_uint_property(bytes, "Skin", "parentId", 5);
            push_f32_property(bytes, "Skin", "tx", 5.0);
            push_f32_property(bytes, "Skin", "ty", -1.0);
        });
        push_object_with_properties(bytes, "Tendon", |bytes| {
            push_uint_property(bytes, "Tendon", "parentId", 10);
            push_uint_property(bytes, "Tendon", "boneId", 1);
            push_f32_property(bytes, "Tendon", "tx", 2.0);
            push_f32_property(bytes, "Tendon", "ty", 3.0);
        });
    });

    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    Artboard::update_components_handle(&rust.core_handle());

    let rust_paints = observe_draw_stream(&rust)
        .into_iter()
        .flat_map(|command| command.shape_paints)
        .collect::<Vec<_>>();
    let cpp_paints = cpp.artboards[0]
        .draw_command_stream
        .iter()
        .flat_map(|command| command.shape_paint_commands.iter())
        .collect::<Vec<_>>();

    assert_eq!(
        cpp_paints.len(),
        1,
        "C++ should emit one visible weighted shape paint"
    );
    assert_eq!(
        rust_paints.len(),
        1,
        "Rust should emit one visible weighted shape paint"
    );
    let cpp_paint = cpp_paints[0];
    let rust_paint = &rust_paints[0];
    assert_eq!(cpp_paint.paint_local, Some(rust_paint.paint_local));
    assert_eq!(cpp_paint.path_kind(), rust_paint.path_kind);

    let expected_path_commands = vec![
        PathCommand::Move { x: 14.0, y: 18.0 },
        PathCommand::Cubic {
            x1: 14.0,
            y1: 18.0,
            x2: 16.0,
            y2: 18.0,
            x3: 18.0,
            y3: 18.0,
        },
        PathCommand::Cubic {
            x1: 20.0,
            y1: 18.0,
            x2: 14.0,
            y2: 18.0,
            x3: 14.0,
            y3: 18.0,
        },
        PathCommand::Close,
    ];
    let cpp_path_commands = cpp_paint.path_commands();
    assert_path_commands_close(
        &cpp_path_commands,
        &expected_path_commands,
        "C++ weighted points path commands",
    );
    assert_path_commands_close(
        &rust_paint.path_commands,
        &cpp_path_commands,
        "Rust weighted points path commands",
    );
}

#[test]
fn runtime_drawable_dispatch_stream_exposes_line_trim_path_effect_payloads_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_line_trim_path_effect_payloads.riv";
    let bytes = synthetic_runtime_file(8233, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
        });
        push_object_with_properties(bytes, "Stroke", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 1);
        });
        push_object_with_properties(bytes, "SolidColor", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 2);
            push_color_property(bytes, "SolidColor", "colorValue", 0xff22_4466);
        });
        push_object_with_properties(bytes, "TrimPath", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 2);
            push_f32_property(bytes, "TrimPath", "start", 0.25);
            push_f32_property(bytes, "TrimPath", "end", 0.75);
            push_uint_property(bytes, "TrimPath", "modeValue", 1);
        });
        push_object_with_properties(bytes, "PointsPath", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 1);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 5);
            push_f32_property(bytes, "Vertex", "x", 0.0);
            push_f32_property(bytes, "Vertex", "y", 0.0);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 5);
            push_f32_property(bytes, "Vertex", "x", 10.0);
            push_f32_property(bytes, "Vertex", "y", 0.0);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 5);
            push_f32_property(bytes, "Vertex", "x", 20.0);
            push_f32_property(bytes, "Vertex", "y", 0.0);
        });
    });

    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    Artboard::update_components_handle(&rust.core_handle());

    let rust_paints = observe_draw_stream(&rust)
        .into_iter()
        .flat_map(|command| command.shape_paints)
        .collect::<Vec<_>>();
    let cpp_paints = cpp.artboards[0]
        .draw_command_stream
        .iter()
        .flat_map(|command| command.shape_paint_commands.iter())
        .collect::<Vec<_>>();

    assert_eq!(cpp_paints.len(), 1, "C++ should emit one trimmed stroke");
    assert_eq!(rust_paints.len(), 1, "Rust should emit one trimmed stroke");

    let expected_source = vec![
        PathCommand::Move { x: 0.0, y: 0.0 },
        PathCommand::Line { x: 10.0, y: 0.0 },
        PathCommand::Line { x: 20.0, y: 0.0 },
    ];
    let expected_effect = vec![
        PathCommand::Move { x: 5.0, y: 0.0 },
        PathCommand::Line { x: 10.0, y: 0.0 },
        PathCommand::Line { x: 15.0, y: 0.0 },
    ];

    let cpp_paint = cpp_paints[0];
    let rust_paint = &rust_paints[0];
    assert_eq!(cpp_paint.path_commands(), expected_source);
    assert_eq!(rust_paint.path_commands, cpp_paint.path_commands());
    assert_eq!(cpp_paint.effect_path_commands(), expected_effect);
    assert_eq!(
        rust_paint.effect_path_commands,
        cpp_paint.effect_path_commands()
    );
}

#[test]
fn runtime_drawable_dispatch_stream_exposes_gradient_paint_payloads_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_gradient_paint_payloads.riv";
    let bytes = synthetic_runtime_file(8215, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
            push_f32_property(bytes, "Node", "opacity", 0.5);
        });
        push_object_with_properties(bytes, "Fill", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 1);
        });
        push_object_with_properties(bytes, "LinearGradient", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 2);
            push_f32_property(bytes, "LinearGradient", "startX", 1.0);
            push_f32_property(bytes, "LinearGradient", "startY", 2.0);
            push_f32_property(bytes, "LinearGradient", "endX", 11.0);
            push_f32_property(bytes, "LinearGradient", "endY", 12.0);
            push_f32_property(bytes, "LinearGradient", "opacity", 0.5);
        });
        push_object_with_properties(bytes, "GradientStop", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 3);
            push_color_property(bytes, "GradientStop", "colorValue", 0xff00_ff00);
            push_f32_property(bytes, "GradientStop", "position", 1.5);
        });
        push_object_with_properties(bytes, "GradientStop", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 3);
            push_color_property(bytes, "GradientStop", "colorValue", 0x8000_00ff);
            push_f32_property(bytes, "GradientStop", "position", -0.25);
        });
        push_object_with_properties(bytes, "GradientStop", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 3);
            push_color_property(bytes, "GradientStop", "colorValue", 0xffff_0000);
            push_f32_property(bytes, "GradientStop", "position", 0.5);
        });
        push_object_with_properties(bytes, "PointsPath", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 1);
            push_bool_property(bytes, "PointsCommonPath", "isClosed", true);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 7);
            push_f32_property(bytes, "Vertex", "x", 0.0);
            push_f32_property(bytes, "Vertex", "y", 0.0);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 7);
            push_f32_property(bytes, "Vertex", "x", 10.0);
            push_f32_property(bytes, "Vertex", "y", 0.0);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 7);
            push_f32_property(bytes, "Vertex", "x", 0.0);
            push_f32_property(bytes, "Vertex", "y", 10.0);
        });
    });

    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    Artboard::update_components_handle(&rust.core_handle());

    let rust_paints = observe_draw_stream(&rust)
        .into_iter()
        .flat_map(|command| command.shape_paints)
        .collect::<Vec<_>>();
    let cpp_paints = cpp.artboards[0]
        .draw_command_stream
        .iter()
        .flat_map(|command| command.shape_paint_commands.iter())
        .collect::<Vec<_>>();

    assert_eq!(cpp_paints.len(), 1, "C++ should emit one gradient paint");
    assert_eq!(rust_paints.len(), 1, "Rust should emit one gradient paint");

    let expected_state = PaintState::LinearGradient {
        start_x: 1.0,
        start_y: 2.0,
        end_x: 11.0,
        end_y: 12.0,
        opacity: 0.5,
        render_opacity: 0.5,
        stops: vec![
            GradientStopState {
                color: 0x8000_00ff,
                render_color: 0x2000_00ff,
                position: 0.0,
            },
            GradientStopState {
                color: 0xffff_0000,
                render_color: 0x40ff_0000,
                position: 0.5,
            },
            GradientStopState {
                color: 0xff00_ff00,
                render_color: 0x4000_ff00,
                position: 1.0,
            },
        ],
    };

    let cpp_paint = cpp_paints[0];
    let rust_paint = &rust_paints[0];
    assert_eq!(cpp_paint.paint_state(), Some(expected_state.clone()));
    assert_eq!(rust_paint.paint_state, Some(expected_state));
    assert_eq!(rust_paint.paint_state, cpp_paint.paint_state());
    assert_eq!(cpp_paint.feather_state(), None);
    assert_eq!(rust_paint.feather_state, None);
}

#[test]
fn runtime_drawable_dispatch_stream_world_stroke_gradient_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_world_stroke_gradient_payloads.riv";
    let bytes = synthetic_runtime_file(8252, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
            push_f32_property(bytes, "Node", "x", 100.0);
            push_f32_property(bytes, "Node", "y", 50.0);
        });
        push_object_with_properties(bytes, "Stroke", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 1);
            push_bool_property(bytes, "Stroke", "transformAffectsStroke", false);
        });
        push_object_with_properties(bytes, "LinearGradient", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 2);
            push_f32_property(bytes, "LinearGradient", "startX", 1.0);
            push_f32_property(bytes, "LinearGradient", "startY", 2.0);
            push_f32_property(bytes, "LinearGradient", "endX", 11.0);
            push_f32_property(bytes, "LinearGradient", "endY", 12.0);
            push_f32_property(bytes, "LinearGradient", "opacity", 0.75);
        });
        push_object_with_properties(bytes, "GradientStop", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 3);
            push_color_property(bytes, "GradientStop", "colorValue", 0xff00_ff00);
            push_f32_property(bytes, "GradientStop", "position", 0.0);
        });
        push_object_with_properties(bytes, "GradientStop", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 3);
            push_color_property(bytes, "GradientStop", "colorValue", 0x8000_00ff);
            push_f32_property(bytes, "GradientStop", "position", 1.0);
        });
        push_object_with_properties(bytes, "PointsPath", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 1);
            push_bool_property(bytes, "PointsCommonPath", "isClosed", true);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 6);
            push_f32_property(bytes, "Vertex", "x", 0.0);
            push_f32_property(bytes, "Vertex", "y", 0.0);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 6);
            push_f32_property(bytes, "Vertex", "x", 10.0);
            push_f32_property(bytes, "Vertex", "y", 0.0);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 6);
            push_f32_property(bytes, "Vertex", "x", 0.0);
            push_f32_property(bytes, "Vertex", "y", 10.0);
        });
    });

    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    Artboard::update_components_handle(&rust.core_handle());

    let rust_paints = observe_draw_stream(&rust)
        .into_iter()
        .flat_map(|command| command.shape_paints)
        .collect::<Vec<_>>();
    let cpp_paints = cpp.artboards[0]
        .draw_command_stream
        .iter()
        .flat_map(|command| command.shape_paint_commands.iter())
        .collect::<Vec<_>>();

    assert_eq!(cpp_paints.len(), 1, "C++ should emit one gradient stroke");
    assert_eq!(rust_paints.len(), 1, "Rust should emit one gradient stroke");

    let expected_state = PaintState::LinearGradient {
        start_x: 1.0,
        start_y: 2.0,
        end_x: 11.0,
        end_y: 12.0,
        opacity: 0.75,
        render_opacity: 1.0,
        stops: vec![
            GradientStopState {
                color: 0xff00_ff00,
                render_color: 0xbf00_ff00,
                position: 0.0,
            },
            GradientStopState {
                color: 0x8000_00ff,
                render_color: 0x6000_00ff,
                position: 1.0,
            },
        ],
    };

    let cpp_paint = cpp_paints[0];
    let rust_paint = &rust_paints[0];
    assert_eq!(cpp_paint.paint_state(), Some(expected_state.clone()));
    assert_eq!(rust_paint.paint_state, Some(expected_state));
    assert_eq!(rust_paint.paint_state, cpp_paint.paint_state());
    assert_eq!(
        rust_paint.paint_space_transform,
        Some(Mat2D([1.0, 0.0, -0.0, 1.0, 100.0, 50.0]))
    );
}

#[test]
fn runtime_drawable_dispatch_stream_exposes_feather_paint_payloads_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_feather_paint_payloads.riv";
    let bytes = synthetic_runtime_file(8216, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
        });
        push_object_with_properties(bytes, "Fill", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 1);
        });
        push_object_with_properties(bytes, "SolidColor", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 2);
            push_color_property(bytes, "SolidColor", "colorValue", 0xffaa_5500);
        });
        push_object_with_properties(bytes, "Feather", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 2);
            push_uint_property(bytes, "Feather", "spaceValue", 1);
            push_f32_property(bytes, "Feather", "strength", 8.0);
            push_f32_property(bytes, "Feather", "offsetX", 3.0);
            push_f32_property(bytes, "Feather", "offsetY", -4.0);
            push_bool_property(bytes, "Feather", "inner", true);
        });
        push_object_with_properties(bytes, "PointsPath", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 1);
            push_bool_property(bytes, "PointsCommonPath", "isClosed", true);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 5);
            push_f32_property(bytes, "Vertex", "x", 0.0);
            push_f32_property(bytes, "Vertex", "y", 0.0);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 5);
            push_f32_property(bytes, "Vertex", "x", 10.0);
            push_f32_property(bytes, "Vertex", "y", 0.0);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 5);
            push_f32_property(bytes, "Vertex", "x", 0.0);
            push_f32_property(bytes, "Vertex", "y", 10.0);
        });
    });

    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    Artboard::update_components_handle(&rust.core_handle());

    let rust_paints = observe_draw_stream(&rust)
        .into_iter()
        .flat_map(|command| command.shape_paints)
        .collect::<Vec<_>>();
    let cpp_paints = cpp.artboards[0]
        .draw_command_stream
        .iter()
        .flat_map(|command| command.shape_paint_commands.iter())
        .collect::<Vec<_>>();

    assert_eq!(cpp_paints.len(), 1, "C++ should emit one feather paint");
    assert_eq!(rust_paints.len(), 1, "Rust should emit one feather paint");

    let expected_feather = FeatherState {
        feather_local: 4,
        space_value: 1,
        strength: 8.0,
        offset_x: 3.0,
        offset_y: -4.0,
        inner: true,
        inner_path_commands: vec![
            PathCommand::Move { x: -12.0, y: -12.0 },
            PathCommand::Line { x: 22.0, y: -12.0 },
            PathCommand::Line { x: 22.0, y: 22.0 },
            PathCommand::Line { x: -12.0, y: 22.0 },
            PathCommand::Close,
            PathCommand::Move { x: 3.0, y: -4.0 },
            PathCommand::Line { x: 3.0, y: 6.0 },
            PathCommand::Line { x: 13.0, y: -4.0 },
            PathCommand::Line { x: 3.0, y: -4.0 },
            PathCommand::Close,
        ],
    };

    let cpp_paint = cpp_paints[0];
    let rust_paint = &rust_paints[0];
    assert_eq!(cpp_paint.feather_state(), Some(expected_feather.clone()));
    assert_eq!(rust_paint.feather_state, Some(expected_feather));
    assert_eq!(rust_paint.feather_state, cpp_paint.feather_state());
}

#[test]
fn runtime_drawable_dispatch_stream_exposes_rectangle_parametric_path_payloads_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_rectangle_parametric_path_payloads.riv";
    let bytes = synthetic_runtime_file(8217, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
            push_f32_property(bytes, "Node", "x", 100.0);
        });
        push_object_with_properties(bytes, "Fill", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 1);
        });
        push_object_with_properties(bytes, "SolidColor", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 2);
            push_color_property(bytes, "SolidColor", "colorValue", 0xff22_8844);
        });
        push_object_with_properties(bytes, "Rectangle", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 1);
            push_f32_property(bytes, "Node", "x", 7.0);
            push_f32_property(bytes, "Node", "y", -2.0);
            push_f32_property(bytes, "ParametricPath", "width", 20.0);
            push_f32_property(bytes, "ParametricPath", "height", 10.0);
            push_f32_property(bytes, "ParametricPath", "originX", 0.25);
            push_f32_property(bytes, "ParametricPath", "originY", 0.5);
            push_bool_property(bytes, "Rectangle", "linkCornerRadius", true);
            push_f32_property(bytes, "Rectangle", "cornerRadiusTL", 2.0);
        });
    });

    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    Artboard::update_components_handle(&rust.core_handle());

    let rust_paints = observe_draw_stream(&rust)
        .into_iter()
        .flat_map(|command| command.shape_paints)
        .collect::<Vec<_>>();
    let cpp_paints = cpp.artboards[0]
        .draw_command_stream
        .iter()
        .flat_map(|command| command.shape_paint_commands.iter())
        .collect::<Vec<_>>();

    assert_eq!(cpp_paints.len(), 1, "C++ should emit one rectangle paint");
    assert_eq!(rust_paints.len(), 1, "Rust should emit one rectangle paint");

    let cpp_path_commands = cpp_paints[0].path_commands();
    assert_eq!(
        cpp_path_commands.len(),
        10,
        "rounded rectangle should produce move + four rounded corners + close"
    );
    assert_path_commands_close(
        &rust_paints[0].path_commands,
        &cpp_path_commands,
        "Rust rectangle parametric path commands",
    );
}

#[test]
fn runtime_drawable_dispatch_stream_exposes_ellipse_parametric_path_payloads_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_ellipse_parametric_path_payloads.riv";
    let bytes = synthetic_runtime_file(8218, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
            push_f32_property(bytes, "Node", "x", 100.0);
        });
        push_object_with_properties(bytes, "Fill", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 1);
        });
        push_object_with_properties(bytes, "SolidColor", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 2);
            push_color_property(bytes, "SolidColor", "colorValue", 0xff33_66aa);
        });
        push_object_with_properties(bytes, "Ellipse", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 1);
            push_f32_property(bytes, "Node", "x", 3.0);
            push_f32_property(bytes, "Node", "y", 4.0);
            push_f32_property(bytes, "ParametricPath", "width", 30.0);
            push_f32_property(bytes, "ParametricPath", "height", 12.0);
            push_f32_property(bytes, "ParametricPath", "originX", 0.25);
            push_f32_property(bytes, "ParametricPath", "originY", 0.75);
        });
    });

    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    Artboard::update_components_handle(&rust.core_handle());

    let rust_paints = observe_draw_stream(&rust)
        .into_iter()
        .flat_map(|command| command.shape_paints)
        .collect::<Vec<_>>();
    let cpp_paints = cpp.artboards[0]
        .draw_command_stream
        .iter()
        .flat_map(|command| command.shape_paint_commands.iter())
        .collect::<Vec<_>>();

    assert_eq!(cpp_paints.len(), 1, "C++ should emit one ellipse paint");
    assert_eq!(rust_paints.len(), 1, "Rust should emit one ellipse paint");

    let cpp_path_commands = cpp_paints[0].path_commands();
    assert_eq!(
        cpp_path_commands.len(),
        6,
        "ellipse should produce move + four cubics + close"
    );
    assert_path_commands_close(
        &rust_paints[0].path_commands,
        &cpp_path_commands,
        "Rust ellipse parametric path commands",
    );
}

#[test]
fn runtime_drawable_dispatch_stream_exposes_polygon_parametric_path_payloads_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_polygon_parametric_path_payloads.riv";
    let bytes = synthetic_runtime_file(8219, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
            push_f32_property(bytes, "Node", "x", 100.0);
        });
        push_object_with_properties(bytes, "Fill", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 1);
        });
        push_object_with_properties(bytes, "SolidColor", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 2);
            push_color_property(bytes, "SolidColor", "colorValue", 0xff66_44aa);
        });
        push_object_with_properties(bytes, "Polygon", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 1);
            push_f32_property(bytes, "Node", "x", 3.0);
            push_f32_property(bytes, "Node", "y", 4.0);
            push_f32_property(bytes, "ParametricPath", "width", 30.0);
            push_f32_property(bytes, "ParametricPath", "height", 12.0);
            push_f32_property(bytes, "ParametricPath", "originX", 0.25);
            push_f32_property(bytes, "ParametricPath", "originY", 0.75);
            push_uint_property(bytes, "Polygon", "points", 5);
            push_f32_property(bytes, "Polygon", "cornerRadius", 1.5);
        });
    });

    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    Artboard::update_components_handle(&rust.core_handle());

    let rust_paints = observe_draw_stream(&rust)
        .into_iter()
        .flat_map(|command| command.shape_paints)
        .collect::<Vec<_>>();
    let cpp_paints = cpp.artboards[0]
        .draw_command_stream
        .iter()
        .flat_map(|command| command.shape_paint_commands.iter())
        .collect::<Vec<_>>();

    assert_eq!(cpp_paints.len(), 1, "C++ should emit one polygon paint");
    assert_eq!(rust_paints.len(), 1, "Rust should emit one polygon paint");

    let cpp_path_commands = cpp_paints[0].path_commands();
    assert_eq!(
        cpp_path_commands.len(),
        12,
        "rounded five-point polygon should produce move + five rounded corners + close"
    );
    assert_path_commands_close(
        &rust_paints[0].path_commands,
        &cpp_path_commands,
        "Rust polygon parametric path commands",
    );
}

#[test]
fn runtime_drawable_dispatch_stream_exposes_star_parametric_path_payloads_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_star_parametric_path_payloads.riv";
    let bytes = synthetic_runtime_file(8220, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
            push_f32_property(bytes, "Node", "x", 100.0);
        });
        push_object_with_properties(bytes, "Fill", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 1);
        });
        push_object_with_properties(bytes, "SolidColor", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 2);
            push_color_property(bytes, "SolidColor", "colorValue", 0xff44_88cc);
        });
        push_object_with_properties(bytes, "Star", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 1);
            push_f32_property(bytes, "Node", "x", 3.0);
            push_f32_property(bytes, "Node", "y", 4.0);
            push_f32_property(bytes, "ParametricPath", "width", 30.0);
            push_f32_property(bytes, "ParametricPath", "height", 12.0);
            push_f32_property(bytes, "ParametricPath", "originX", 0.25);
            push_f32_property(bytes, "ParametricPath", "originY", 0.75);
            push_uint_property(bytes, "Polygon", "points", 5);
            push_f32_property(bytes, "Polygon", "cornerRadius", 1.5);
            push_f32_property(bytes, "Star", "innerRadius", 0.4);
        });
    });

    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    Artboard::update_components_handle(&rust.core_handle());

    let rust_paints = observe_draw_stream(&rust)
        .into_iter()
        .flat_map(|command| command.shape_paints)
        .collect::<Vec<_>>();
    let cpp_paints = cpp.artboards[0]
        .draw_command_stream
        .iter()
        .flat_map(|command| command.shape_paint_commands.iter())
        .collect::<Vec<_>>();

    assert_eq!(cpp_paints.len(), 1, "C++ should emit one star paint");
    assert_eq!(rust_paints.len(), 1, "Rust should emit one star paint");

    let cpp_path_commands = cpp_paints[0].path_commands();
    assert_eq!(
        cpp_path_commands.len(),
        22,
        "rounded five-point star should produce move + ten rounded corners + close"
    );
    assert_path_commands_close(
        &rust_paints[0].path_commands,
        &cpp_path_commands,
        "Rust star parametric path commands",
    );
}

#[test]
fn runtime_drawable_dispatch_stream_exposes_triangle_parametric_path_payloads_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_triangle_parametric_path_payloads.riv";
    let bytes = synthetic_runtime_file(8221, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
            push_f32_property(bytes, "Node", "x", 100.0);
        });
        push_object_with_properties(bytes, "Fill", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 1);
        });
        push_object_with_properties(bytes, "SolidColor", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 2);
            push_color_property(bytes, "SolidColor", "colorValue", 0xff55_aacc);
        });
        push_object_with_properties(bytes, "Triangle", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 1);
            push_f32_property(bytes, "Node", "x", 3.0);
            push_f32_property(bytes, "Node", "y", 4.0);
            push_f32_property(bytes, "ParametricPath", "width", 30.0);
            push_f32_property(bytes, "ParametricPath", "height", 12.0);
            push_f32_property(bytes, "ParametricPath", "originX", 0.25);
            push_f32_property(bytes, "ParametricPath", "originY", 0.75);
        });
    });

    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    Artboard::update_components_handle(&rust.core_handle());

    let rust_paints = observe_draw_stream(&rust)
        .into_iter()
        .flat_map(|command| command.shape_paints)
        .collect::<Vec<_>>();
    let cpp_paints = cpp.artboards[0]
        .draw_command_stream
        .iter()
        .flat_map(|command| command.shape_paint_commands.iter())
        .collect::<Vec<_>>();

    assert_eq!(cpp_paints.len(), 1, "C++ should emit one triangle paint");
    assert_eq!(rust_paints.len(), 1, "Rust should emit one triangle paint");

    let cpp_path_commands = cpp_paints[0].path_commands();
    assert_eq!(
        cpp_path_commands.len(),
        5,
        "triangle should produce move + two lines + closing line + close"
    );
    assert_path_commands_close(
        &rust_paints[0].path_commands,
        &cpp_path_commands,
        "Rust triangle parametric path commands",
    );
}
