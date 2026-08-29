//! One-for-one scripted silver ports of pinned
//! `tests/unit_tests/runtime/scripting/scripting_properties_test.cpp#9-#16`.
//!
//! Import and instance construction use the same native File-owned ScriptVm
//! registration and occurrence lifecycle as the silver runner. The test reads
//! live owners; it never mounts or hydrates a second script graph.

use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use nuxie_render_api::{PersistentFactory, SerializingFactory};
use nuxie_runtime::source::{
    math::vec2d::Vec2D, scripted::scripted_drawable::ScriptedDrawable,
    viewmodel::runtime::viewmodel_instance_runtime::ViewModelInstanceRuntime,
};
use nuxie_runtime::{
    Artboard, CoreHandle, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle,
};
use silver_corpus::{Action, ActionTarget, PointerCoordinate, compare_sriv, parse_sriv};

#[path = "../src/scripting.rs"]
mod native_scripting;

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
    let mut factory = PersistentFactory::new(SerializingFactory::new());
    let file = native_scripting::import_file(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).context("retained serializing factory")?,
    )
    .with_context(|| format!("import pinned fixture {fixture}"))?;
    let source = file
        .with_file(|file| file.artboard())
        .context("missing default artboard")?;
    let artboard =
        Artboard::instance_from_handle(&source).context("instantiate default artboard")?;
    let (width, height) = artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
    factory.borrow_mut().frame_size(width as u32, height as u32);
    let machine = artboard
        .state_machine_instance_handle(0)
        .context("missing state machine 0")?;

    let mut actions = actions.into_iter();
    match actions.next() {
        Some(Action::BindDefaultViewModel) => {}
        other => bail!("{id} exact stream must begin with BindDefaultViewModel, got {other:?}"),
    }
    let view_model_owner = exact_view_model(&file, &artboard, ordinal)
        .with_context(|| format!("{id} has no exact pinned ViewModel instance"))?;
    let view_model = ViewModelInstanceRuntime::new(view_model_owner.clone());
    // The reset case retains this property before binding, exactly as C++.
    let reset_trigger = if ordinal == 16 {
        Some(
            view_model
                .property_trigger("tri1")
                .context("missing trigger property tri1")?,
        )
    } else {
        None
    };
    // These two cases construct the renderer before bind; 9/10/11 do so
    // after bind, and 12/14/15 do so after their first advance.
    let mut renderer = matches!(ordinal, 13 | 16).then(|| factory.borrow().make_renderer());
    machine.with_instance_mut(|machine| {
        machine.bind_view_model_instance(view_model_owner.clone());
    });
    let context = machine
        .with_instance(|machine| machine.data_context())
        .context("exact bound ViewModel context was not retained")?;
    let context_owner = context.with_context(|context| context.main_view_model_instance());
    if context_owner.as_ref() != Some(&view_model_owner) {
        bail!("bound ViewModel context is not the exact live owner");
    }
    if matches!(ordinal, 9..=11) {
        renderer = Some(factory.borrow().make_renderer());
    }
    let scripted_drawables = artboard.with_artboard(|artboard| {
        artboard.objects().iter().flatten()
            .filter(|owner| owner.is_type_of(
                nuxie_runtime::source::generated::scripted::scripted_drawable_base::ScriptedDrawableBase::TYPE_KEY,
            ))
            .cloned().collect::<Vec<_>>()
    });
    if scripted_drawables.is_empty() {
        bail!("{fixture} default artboard has no concrete ScriptedDrawable");
    }

    let mut observed_times = Vec::new();
    for action in actions {
        match action {
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds,
            } => {
                machine.advance_and_apply(seconds);
                // The pinned native lifecycle, not a harness mount step,
                // initializes the actual scripted occurrences on advance.
                if observed_times.is_empty() {
                    for owner in &scripted_drawables {
                        if !owner
                            .with_downcast::<ScriptedDrawable, _>(|drawable| {
                                drawable.scripted.self_ref() != 0
                            })
                            .unwrap_or(false)
                        {
                            bail!("{id} ScriptedDrawable {owner:?} has no live ScriptInstance");
                        }
                    }
                }
                observed_times.push(seconds);
            }
            Action::Draw => {
                let renderer = renderer.get_or_insert_with(|| factory.borrow().make_renderer());
                artboard.draw(renderer);
            }
            Action::Frame => factory.borrow_mut().add_frame(),
            Action::PointerDown { x, y, pointer_id } => {
                let (width, height) =
                    artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
                let (x, y) = resolve_pointer_pair(&x, &y, width, height)?;
                machine.with_instance_mut(|machine| {
                    machine.pointer_down(Vec2D::new(x, y), pointer_id);
                });
            }
            Action::PointerUp { x, y, pointer_id } => {
                let (width, height) =
                    artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
                let (x, y) = resolve_pointer_pair(&x, &y, width, height)?;
                machine.with_instance_mut(|machine| {
                    machine.pointer_up(Vec2D::new(x, y), pointer_id);
                });
            }
            Action::FireViewModelTrigger { property } => {
                if ordinal == 16 && property == "tri1" {
                    reset_trigger
                        .as_ref()
                        .expect("retained reset trigger")
                        .trigger();
                } else {
                    view_model
                        .property_trigger(&property)
                        .with_context(|| format!("{id} missing trigger property {property}"))?
                        .trigger();
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
    file: &RuntimeFileHandle,
    artboard: &RuntimeArtboardInstanceHandle,
    ordinal: usize,
) -> Option<CoreHandle> {
    let root = artboard.core_handle();
    match ordinal {
        9 | 12 => {
            let model_id = artboard.with_artboard(|artboard| artboard.view_model_id());
            if model_id == u32::MAX {
                file.with_file_mut(|file| file.create_view_model_instance_for_artboard(root))
            } else {
                file.with_file(|file| file.create_view_model_instance_at(model_id as usize, 0))
            }
        }
        _ => file.with_file_mut(|file| file.create_default_view_model_instance_for_artboard(root)),
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
