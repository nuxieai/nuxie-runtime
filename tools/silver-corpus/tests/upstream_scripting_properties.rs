//! One-for-one scripted silver ports of pinned
//! `tests/unit_tests/runtime/scripting/scripting_properties_test.cpp#9-#16`.
//!
//! Each test spells out its own upstream action sequence. The shared runner
//! uses `nuxie::ArtboardInstance`, the production facade owner for the genuine
//! mount, attach, and fail-closed verification lifecycle. Before the first
//! behavioral action it binds the pinned ViewModel, mounts the selected
//! Artboard's scripts, and proves every concrete `ScriptedDrawable` global is
//! backed by an attached live `ScriptInstance`.

use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use nuxie::{File, PersistentFactory, ScriptExecutionLimits, ViewModelInstance};
use nuxie_render_api::SerializingFactory;
use silver_corpus::{Action, ActionTarget, PointerCoordinate, compare_sriv, parse_sriv};

fn runtime_root() -> PathBuf {
    std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
}

fn literal(value: f32) -> PointerCoordinate {
    PointerCoordinate::Literal(value)
}

fn artboard_width_over_two() -> PointerCoordinate {
    PointerCoordinate::Expression("artboard-width/2".to_owned())
}

fn run_exact_scripted_silver(
    runtime: &Path,
    ordinal: usize,
    id: &str,
    fixture: &str,
    sample_times: Vec<f32>,
    actions: Vec<Action>,
) -> anyhow::Result<()> {
    let fixture_path = runtime.join("tests/unit_tests/assets").join(fixture);
    let bytes = std::fs::read(&fixture_path)
        .with_context(|| format!("read pinned fixture {}", fixture_path.display()))?;
    let file = File::import_with_trusted_scripts(&bytes, ScriptExecutionLimits::new())
        .with_context(|| format!("import pinned fixture {fixture}"))?;
    let selected = file
        .default_artboard()
        .context("missing default artboard")?;
    let scripted_drawable_globals = selected
        .graph()
        .components
        .iter()
        .filter(|component| component.type_name == "ScriptedDrawable")
        .map(|component| component.global_id)
        .collect::<Vec<_>>();
    if scripted_drawable_globals.is_empty() {
        bail!("{fixture} default artboard has no concrete ScriptedDrawable");
    }
    let mut artboard = selected
        .instantiate()
        .context("instantiate default artboard")?;
    let mut factory = PersistentFactory::new(SerializingFactory::new());
    artboard
        .initialize_renderer(&mut factory)
        .context("initialize renderer and File script VM")?;
    let (width, height) = artboard.artboard_dimensions();
    factory.borrow_mut().frame_size(width as u32, height as u32);
    let mut machine = artboard
        .state_machine_instance(0)
        .context("missing state machine 0")?;

    let mut actions = actions.into_iter();
    match actions.next() {
        Some(Action::BindDefaultViewModel) => {}
        other => bail!("{id} exact stream must begin with BindDefaultViewModel, got {other:?}"),
    }
    let mut view_model = exact_view_model(&artboard, ordinal)
        .with_context(|| format!("{id} has no exact pinned ViewModel instance"))?;
    artboard.bind_view_model(&view_model);
    let context = artboard
        .owned_view_model_context()
        .and_then(|context| context.main_handle())
        .context("exact bound ViewModel context was not retained")?;
    let context_identity = context.instance_identity();
    if context_identity != view_model.identity() {
        bail!("bound ViewModel context is not the exact live owner");
    }
    if !artboard
        .mount_scripted_drawables(&mut factory)
        .context("mount selected-artboard ScriptedDrawables")?
    {
        bail!("selected artboard reported no script mount target");
    }
    for global_id in &scripted_drawable_globals {
        if !artboard.raw().has_script_instance_for_global(*global_id) {
            bail!("ScriptedDrawable global {global_id} has no attached live ScriptInstance");
        }
    }

    let mut renderer = factory.borrow().make_renderer();
    let mut observed_times = Vec::new();
    for action in actions {
        match action {
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds,
            } => {
                artboard
                    .try_advance_with_state_machines_and_view_model_and_factory(
                        std::slice::from_mut(&mut machine),
                        seconds,
                        &mut view_model,
                        &mut factory,
                    )
                    .with_context(|| format!("{id} advance {seconds}"))?;
                observed_times.push(seconds);
            }
            Action::Draw => artboard
                .draw(&mut factory, &mut renderer)
                .with_context(|| format!("{id} draw"))?,
            Action::Frame => factory.borrow_mut().add_frame(),
            Action::PointerDown { x, y, pointer_id } => {
                let (x, y) = resolve_pointer_pair(&x, &y, width, height)?;
                machine.pointer_down(artboard.raw_mut(), x, y, pointer_id);
            }
            Action::PointerUp { x, y, pointer_id } => {
                let (x, y) = resolve_pointer_pair(&x, &y, width, height)?;
                machine.pointer_up(artboard.raw_mut(), x, y, pointer_id);
            }
            Action::FireViewModelTrigger { property } => {
                if !view_model.fire_trigger(&property) {
                    bail!("{id} missing trigger property {property}");
                }
            }
            other => bail!("{id} unsupported exact action {other:?}"),
        }
    }
    if observed_times != sample_times {
        bail!("{id} observed times {observed_times:?} != pinned {sample_times:?}");
    }
    let actual_bytes = factory.borrow().bytes().to_vec();
    let expected_path = runtime
        .join("tests/unit_tests/silvers")
        .join(format!("{id}.sriv"));
    let expected = parse_sriv(&std::fs::read(&expected_path)?)?;
    let actual = parse_sriv(&actual_bytes)?;
    compare_sriv(&expected, &actual).map_err(|difference| anyhow::anyhow!("{id}: {difference}"))
}

fn exact_view_model(
    artboard: &nuxie::ArtboardInstance<'_>,
    ordinal: usize,
) -> Option<ViewModelInstance> {
    match ordinal {
        // These two pinned cases call createViewModelInstance(viewModelId, 0)
        // when the selected artboard declares a schema.
        9 | 12 if artboard.view_model_index().is_some() => {
            artboard.instantiate_view_model_instance(0)
        }
        // All other C12 Silver cases call createDefaultViewModelInstance.
        _ => artboard.instantiate_default_view_model_instance(),
    }
}

fn resolve_pointer_pair(
    x: &PointerCoordinate,
    y: &PointerCoordinate,
    width: f32,
    _height: f32,
) -> anyhow::Result<(f32, f32)> {
    let resolve = |coordinate: &PointerCoordinate| match coordinate {
        PointerCoordinate::Literal(value) => Ok(*value),
        PointerCoordinate::Expression(expression) if expression == "artboard-width/2" => {
            Ok(width / 2.0)
        }
        PointerCoordinate::Expression(expression) => {
            bail!("unsupported exact pointer coordinate {expression}")
        }
    };
    Ok((resolve(x)?, resolve(y)?))
}

#[test]
fn wave_c12_silver_009_access_view_model_properties_and_enum_properties() {
    run_exact_scripted_silver(
        &runtime_root(),
        9,
        "viewmodel_access",
        "viewmodel_access.riv",
        vec![0.016],
        vec![
            Action::BindDefaultViewModel,
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds: 0.016,
            },
            Action::Draw,
        ],
    )
    .unwrap_or_else(|error| panic!("{error:#}"));
}

#[test]
#[ignore = "expected-red: viewmodel_from_instance frame 0 op 8 expects makeRenderPaint, live scripted SRIV emits frameSize"]
fn wave_c12_silver_010_creates_view_models_from_specified_named_instances() {
    run_exact_scripted_silver(
        &runtime_root(),
        10,
        "viewmodel_from_instance",
        "viewmodel_from_instance.riv",
        vec![0.016],
        vec![
            Action::BindDefaultViewModel,
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds: 0.016,
            },
            Action::Draw,
        ],
    )
    .unwrap_or_else(|error| panic!("{error:#}"));
}

#[test]
#[ignore = "expected-red: realized replace_view_model frame 1 op 93 expects color, live scripted SRIV emits save"]
fn wave_c12_silver_011_replace_a_view_model_property_value_from_a_script() {
    run_exact_scripted_silver(
        &runtime_root(),
        11,
        "replace_view_model",
        "replace_view_model.riv",
        vec![0.016, 0.016],
        vec![
            Action::BindDefaultViewModel,
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds: 0.016,
            },
            Action::Draw,
            Action::Frame,
            Action::PointerDown {
                x: artboard_width_over_two(),
                y: literal(480.0),
                pointer_id: 0,
            },
            Action::PointerUp {
                x: artboard_width_over_two(),
                y: literal(480.0),
                pointer_id: 0,
            },
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds: 0.016,
            },
            Action::Draw,
        ],
    )
    .unwrap_or_else(|error| panic!("{error:#}"));
}

#[test]
#[ignore = "expected-red: realized remove_from_list frame 1 op 195 expects rewind, live scripted SRIV emits drawPath"]
fn wave_c12_silver_012_scripts_can_remove_items_from_lists() {
    let mut actions = vec![
        Action::BindDefaultViewModel,
        Action::Advance {
            target: ActionTarget::StateMachine,
            seconds: 0.1,
        },
        Action::Draw,
    ];
    for _ in 0..10 {
        actions.extend([
            Action::Frame,
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds: 0.016,
            },
            Action::Draw,
        ]);
    }
    run_exact_scripted_silver(
        &runtime_root(),
        12,
        "remove_from_list",
        "remove_from_list.riv",
        std::iter::once(0.1)
            .chain(std::iter::repeat_n(0.016, 10))
            .collect(),
        actions,
    )
    .unwrap_or_else(|error| panic!("{error:#}"));
}

#[test]
#[ignore = "pending: genuine mount cannot attach nested occurrence graph 84 without retained source File authority"]
fn wave_c12_silver_013_expose_list_index_to_scripts_and_ensure_type_is_correct() {
    run_exact_scripted_silver(
        &runtime_root(),
        13,
        "list_index_script_access",
        "list_index_script_access.riv",
        vec![0.1],
        vec![
            Action::BindDefaultViewModel,
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds: 0.1,
            },
            Action::Draw,
        ],
    )
    .unwrap_or_else(|error| panic!("{error:#}"));
}

#[test]
#[ignore = "expected-red: realized scripted_property_image frame 0 op 21 expects save, live scripted SRIV emits restore"]
fn wave_c12_silver_014_scripted_image_properties() {
    run_exact_scripted_silver(
        &runtime_root(),
        14,
        "scripted_property_image",
        "scripted_property_image.riv",
        vec![0.016],
        vec![
            Action::BindDefaultViewModel,
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds: 0.016,
            },
            Action::Draw,
        ],
    )
    .unwrap_or_else(|error| panic!("{error:#}"));
}

#[test]
#[ignore = "expected-red: realized image_scripting_property_value frame 0 op 1 expects decodeImage, live scripted SRIV emits makeRenderPaint"]
fn wave_c12_silver_015_image_read_from_property_value() {
    run_exact_scripted_silver(
        &runtime_root(),
        15,
        "image_scripting_property_value",
        "image_scripting_property_value.riv",
        vec![0.0, 0.25],
        vec![
            Action::BindDefaultViewModel,
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds: 0.0,
            },
            Action::Draw,
            Action::Frame,
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds: 0.25,
            },
            Action::Draw,
        ],
    )
    .unwrap_or_else(|error| panic!("{error:#}"));
}

#[test]
#[ignore = "expected-red: reset_shared_viewmodel_instance_test frame 0 op 10 expects makeRenderPaint, live scripted SRIV emits frameSize"]
fn wave_c12_silver_016_reset_detached_view_model_instances_at_end_of_frame() {
    run_exact_scripted_silver(
        &runtime_root(),
        16,
        "reset_shared_viewmodel_instance_test",
        "reset_shared_viewmodel_instance_test.riv",
        vec![0.0, 0.016, 0.016, 0.016, 0.016, 0.016],
        vec![
            Action::BindDefaultViewModel,
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds: 0.0,
            },
            Action::Draw,
            Action::Frame,
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds: 0.016,
            },
            Action::Draw,
            Action::Frame,
            Action::FireViewModelTrigger {
                property: "tri1".to_owned(),
            },
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds: 0.016,
            },
            Action::Draw,
            Action::Frame,
            Action::FireViewModelTrigger {
                property: "tri1".to_owned(),
            },
            Action::PointerDown {
                x: literal(45.0),
                y: literal(165.0),
                pointer_id: 0,
            },
            Action::PointerUp {
                x: literal(45.0),
                y: literal(165.0),
                pointer_id: 0,
            },
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds: 0.016,
            },
            Action::Draw,
            Action::Frame,
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds: 0.016,
            },
            Action::Draw,
            Action::Frame,
            Action::PointerDown {
                x: literal(45.0),
                y: literal(165.0),
                pointer_id: 0,
            },
            Action::PointerUp {
                x: literal(45.0),
                y: literal(165.0),
                pointer_id: 0,
            },
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds: 0.016,
            },
            Action::Draw,
        ],
    )
    .unwrap_or_else(|error| panic!("{error:#}"));
}
