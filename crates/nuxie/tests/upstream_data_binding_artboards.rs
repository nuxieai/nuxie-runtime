//! Executable expected-red action-stream ports from pinned
//! `tests/unit_tests/runtime/data_binding_artboards_test.cpp`.

use std::collections::BTreeMap;
use std::path::PathBuf;

use nuxie::{
    File, FileImportLimits, PersistentFactory, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle,
    RuntimeFileHandle, RuntimeStateMachineInstanceHandle, RuntimeViewModelInstanceHandle,
    ScriptExecutionLimits, Vec2D, ViewModelInstanceRuntime, import_unsigned_scripted,
};
use nuxie_render_api::SerializingFactory;
use nuxie_runtime::source::{
    nested_artboard_layout::NestedArtboardLayout,
    viewmodel::viewmodel_instance_artboard::ViewModelInstanceArtboard,
};
use silver_corpus::{compare_sriv, parse_sriv};

#[derive(Clone, Copy, Debug)]
enum Action {
    Import(&'static str),
    ImportScripted(&'static str),
    SelectArtboard(&'static str),
    FrameSize,
    SelectStateMachine,
    CreateAuthoredViewModel,
    CreateFreshViewModel,
    CreateDefaultViewModel,
    BindViewModel,
    BindArtboardViewModel,
    Advance(f32),
    AdvanceArtboard(f32),
    Draw,
    Frame,
    SetArtboard(&'static str, Option<(&'static str, &'static str)>),
    SetArtboardIndex(&'static str, u64),
    SetBoundViewModel(&'static str, &'static str, &'static str),
    ReplaceViewModel(&'static str, &'static str, &'static str),
    SetString(&'static str, &'static str),
    SetBool(&'static str, bool),
    Fire(&'static str, usize),
    PointerDown(f32, f32),
    PointerUp(f32, f32),
    ExpectNested(&'static str, bool),
    Compare(&'static str),
}

fn pinned_fixture(name: &str) -> PathBuf {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));
    root.join("tests/unit_tests/assets").join(name)
}

fn pinned_silver(name: &str) -> PathBuf {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));
    root.join("tests/unit_tests/silvers")
        .join(format!("{name}.sriv"))
}

struct LiveStream {
    files: BTreeMap<&'static str, RuntimeFileHandle>,
    primary_file: Option<&'static str>,
    artboard: Option<RuntimeArtboardInstanceHandle>,
    machine: Option<RuntimeStateMachineInstanceHandle>,
    view_model: Option<RuntimeViewModelInstanceHandle>,
    silver: PersistentFactory<SerializingFactory>,
    factory: RuntimeFactoryHandle,
}

impl LiveStream {
    fn new() -> Self {
        let mut silver = PersistentFactory::new(SerializingFactory::new());
        let factory = RuntimeFactoryHandle::from_factory(&mut silver).expect("retained factory");
        Self {
            files: BTreeMap::new(),
            primary_file: None,
            artboard: None,
            machine: None,
            view_model: None,
            silver,
            factory,
        }
    }

    fn current_file(&self) -> &RuntimeFileHandle {
        &self.files[self.primary_file.expect("primary imported File")]
    }

    fn artboard(&self) -> &RuntimeArtboardInstanceHandle {
        self.artboard.as_ref().expect("selected ArtboardInstance")
    }

    fn view_model(&self) -> &RuntimeViewModelInstanceHandle {
        self.view_model.as_ref().expect("created ViewModelInstance")
    }

    fn advance(&mut self, seconds: f32) {
        self.machine
            .as_ref()
            .expect("StateMachineInstance")
            .advance_and_apply(seconds);
    }

    fn advance_artboard(&self, seconds: f32) {
        self.artboard().advance_default(seconds);
    }

    fn draw(&mut self) {
        let mut renderer = self.silver.borrow().make_renderer();
        self.artboard().draw(&mut renderer);
    }

    fn set_artboard(&mut self, property: &str, source: Option<(&str, &str)>) {
        let bindable = source.map(|(file_name, artboard_name)| {
            self.files[file_name]
                .with_file(|file| {
                    if artboard_name == "default" {
                        file.bindable_artboard_default()
                    } else {
                        file.bindable_artboard_named(artboard_name)
                    }
                })
                .unwrap_or_else(|| panic!("{file_name} has artboard {artboard_name}"))
        });
        self.view_model()
            .property_artboard(property)
            .unwrap_or_else(|| panic!("missing Artboard property {property}"))
            .set_value(bindable);
    }

    fn create_view_model(&mut self, kind: ViewModelKind) {
        let artboard = self.artboard().clone();
        let instance = self
            .current_file()
            .with_file_mut(|file| match kind {
                ViewModelKind::Authored => {
                    let id = artboard.with_artboard(|artboard| artboard.base.view_model_id());
                    if id == u32::MAX {
                        file.create_view_model_instance_for_artboard(artboard.core_handle())
                    } else {
                        file.create_view_model_instance_at(id as usize, 0)
                    }
                }
                ViewModelKind::Fresh => {
                    file.create_view_model_instance_for_artboard(artboard.core_handle())
                }
                ViewModelKind::Default => {
                    file.create_default_view_model_instance_for_artboard(artboard.core_handle())
                }
            })
            .expect("fixture has a live ViewModelInstance");
        self.view_model = Some(ViewModelInstanceRuntime::new(instance).into_handle());
    }
}

#[derive(Clone, Copy)]
enum ViewModelKind {
    Authored,
    Fresh,
    Default,
}

fn execute_until_concrete_parity_boundary(actions: &[Action]) {
    let mut live = LiveStream::new();
    for action in actions {
        match *action {
            Action::Import(name) => {
                let fixture = pinned_fixture(name);
                let bytes = std::fs::read(&fixture).unwrap_or_else(|error| {
                    panic!("missing fixture {}: {error}", fixture.display())
                });
                let file = File::import(&bytes, live.factory.clone(), None, None, None)
                    .unwrap_or_else(|| panic!("fixture {name} imports"));
                live.primary_file.get_or_insert(name);
                live.files.insert(name, file);
            }
            Action::ImportScripted(name) => {
                let fixture = pinned_fixture(name);
                let bytes = std::fs::read(&fixture).unwrap_or_else(|error| {
                    panic!("missing fixture {}: {error}", fixture.display())
                });
                let file = import_unsigned_scripted(
                    &bytes,
                    &mut live.silver,
                    None,
                    FileImportLimits::new(),
                    ScriptExecutionLimits::new(),
                )
                .unwrap_or_else(|error| panic!("scripted fixture {name} imports: {error:#}"));
                live.primary_file.get_or_insert(name);
                live.files.insert(name, file.native_file().clone());
            }
            Action::SelectArtboard(name) => {
                let artboard = live
                    .current_file()
                    .with_file(|file| {
                        if name == "default" {
                            file.artboard_default()
                        } else {
                            file.artboard_named(name)
                        }
                    })
                    .unwrap_or_else(|| panic!("primary fixture has artboard {name}"));
                live.artboard = Some(artboard);
            }
            Action::FrameSize => {
                let (width, height) = live
                    .artboard()
                    .with_artboard(|artboard| (artboard.width(), artboard.height()));
                assert!(width.is_finite() && height.is_finite());
                live.silver
                    .borrow_mut()
                    .frame_size(width as u32, height as u32);
            }
            Action::SelectStateMachine => {
                live.machine = Some(
                    live.artboard()
                        .state_machine_at(0)
                        .expect("StateMachineInstance"),
                );
            }
            Action::CreateAuthoredViewModel => live.create_view_model(ViewModelKind::Authored),
            Action::CreateFreshViewModel => live.create_view_model(ViewModelKind::Fresh),
            Action::CreateDefaultViewModel => live.create_view_model(ViewModelKind::Default),
            Action::BindViewModel => {
                let instance = live.view_model().instance();
                live.machine
                    .as_ref()
                    .expect("StateMachineInstance")
                    .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
            }
            Action::BindArtboardViewModel => {
                live.artboard()
                    .bind_view_model_instance(Some(live.view_model().instance()));
            }
            Action::Draw => live.draw(),
            Action::Frame => live.silver.borrow_mut().add_frame(),
            Action::Advance(seconds) => live.advance(seconds),
            Action::AdvanceArtboard(seconds) => live.advance_artboard(seconds),
            Action::SetArtboard(property, source) => live.set_artboard(property, source),
            Action::SetArtboardIndex(property, value) => {
                let property = live
                    .view_model()
                    .property_artboard(property)
                    .unwrap_or_else(|| panic!("missing Artboard property {property}"));
                property
                    .value_runtime()
                    .handle()
                    .with_downcast_mut::<ViewModelInstanceArtboard, _>(|property| {
                        property.set_property_value(value as u32)
                    })
                    .expect("ViewModelInstanceArtboard owner");
            }
            Action::SetBoundViewModel(property, model, instance) => {
                let bound = live
                    .files
                    .values()
                    .find_map(|file| {
                        file.with_file(|file| file.view_model_by_name(model))
                            .and_then(|view_model| view_model.create_instance_from_name(instance))
                    })
                    .unwrap_or_else(|| panic!("translated bound ViewModel {model}/{instance}"));
                live.view_model()
                    .property_artboard(property)
                    .unwrap_or_else(|| panic!("missing Artboard property {property}"))
                    .set_view_model_instance(Some(bound.instance()));
            }
            Action::ReplaceViewModel(property, model, instance) => {
                let replacement = live
                    .files
                    .values()
                    .find_map(|file| {
                        file.with_file(|file| file.view_model_by_name(model))
                            .and_then(|view_model| view_model.create_instance_from_name(instance))
                    })
                    .unwrap_or_else(|| {
                        panic!("translated replacement ViewModel {model}/{instance}")
                    });
                assert!(live.view_model().replace_view_model(property, replacement));
            }
            Action::SetString(property, value) => {
                live.view_model()
                    .property_string(property)
                    .unwrap_or_else(|| panic!("missing String property {property}"))
                    .set_value(value.to_owned());
            }
            Action::SetBool(property, value) => {
                live.view_model()
                    .property_boolean(property)
                    .unwrap_or_else(|| panic!("missing Boolean property {property}"))
                    .set_value(value);
            }
            Action::Fire(property, count) => {
                let trigger = live
                    .view_model()
                    .property_trigger(property)
                    .unwrap_or_else(|| panic!("missing Trigger property {property}"));
                for _ in 0..count {
                    trigger.trigger();
                }
            }
            Action::PointerDown(x, y) => {
                live.machine
                    .as_ref()
                    .expect("StateMachineInstance")
                    .with_instance_mut(|machine| machine.pointer_down(Vec2D::new(x, y), 0));
            }
            Action::PointerUp(x, y) => {
                live.machine
                    .as_ref()
                    .expect("StateMachineInstance")
                    .with_instance_mut(|machine| machine.pointer_up(Vec2D::new(x, y), 0));
            }
            Action::ExpectNested(name, expected) => {
                let host = live
                    .artboard()
                    .with_artboard(|artboard| artboard.find_handle::<NestedArtboardLayout>(name))
                    .unwrap_or_else(|| panic!("missing NestedArtboardLayout {name}"));
                let present = host
                    .with_downcast::<NestedArtboardLayout, _>(|host| {
                        host.base.base.artboard_instance_handle(0).is_some()
                    })
                    .expect("NestedArtboardLayout owner");
                assert_eq!(present, expected, "nested Artboard occurrence presence");
            }
            Action::Compare(name) => {
                let expected = std::fs::read(pinned_silver(name)).expect("pinned silver");
                let expected = parse_sriv(&expected).expect("pinned SRIV");
                let actual = parse_sriv(&live.silver.borrow().bytes()).expect("Rust SRIV");
                compare_sriv(&expected, &actual)
                    .unwrap_or_else(|difference| panic!("{name} differs: {difference}"));
            }
        }
    }
}

#[test]
#[ignore = "expected-red: frame 7 op 582 expected frame, got makeRenderPaint"]
fn data_binding_artboards_from_same_and_different_sources() {
    execute_until_concrete_parity_boundary(&[
        Action::Import("data_binding_artboards_test.riv"),
        Action::Import("data_binding_artboards_source_test.riv"),
        Action::SelectArtboard("default"),
        Action::FrameSize,
        Action::SelectStateMachine,
        Action::CreateAuthoredViewModel,
        Action::BindViewModel,
        Action::Advance(0.1),
        Action::Draw,
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard("ab", Some(("data_binding_artboards_test.riv", "ch1"))),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard("ab", Some(("data_binding_artboards_test.riv", "ch2"))),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "ab",
            Some(("data_binding_artboards_source_test.riv", "source_1")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Fire("ch/tr", 1),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Fire("ch/tr", 1),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "ab",
            Some(("data_binding_artboards_source_test.riv", "source_2")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "ab",
            Some(("data_binding_artboards_source_test.riv", "source_1")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard("ab", Some(("data_binding_artboards_test.riv", "ch2"))),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard("ab", None),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard("ab", Some(("data_binding_artboards_test.riv", "ch2"))),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Compare("data_binding_artboards_test"),
    ]);
}

#[test]
fn recursive_data_binding_artboards_are_skipped() {
    execute_until_concrete_parity_boundary(&[
        Action::Import("data_binding_artboards_test.riv"),
        Action::SelectArtboard("recursive-grand-parent"),
        Action::FrameSize,
        Action::SelectStateMachine,
        Action::CreateAuthoredViewModel,
        Action::BindViewModel,
        Action::Advance(0.1),
        Action::Draw,
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "ab",
            Some(("data_binding_artboards_test.riv", "recursive-grand-child-1")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "ab",
            Some(("data_binding_artboards_test.riv", "recursive-parent")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "ab",
            Some(("data_binding_artboards_test.riv", "recursive-grand-parent")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "ab",
            Some(("data_binding_artboards_test.riv", "recursive-grand-child-2")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Compare("data_binding_artboards_test_recursive"),
    ]);
}

#[test]
fn default_data_binding_artboard_from_different_source() {
    execute_until_concrete_parity_boundary(&[
        Action::Import("data_binding_artboards_test.riv"),
        Action::Import("data_binding_artboards_source_test.riv"),
        Action::SelectArtboard("default"),
        Action::FrameSize,
        Action::SelectStateMachine,
        Action::CreateAuthoredViewModel,
        Action::BindViewModel,
        Action::Advance(0.1),
        Action::Draw,
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard("ab", Some(("data_binding_artboards_test.riv", "ch1"))),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard("ab", Some(("data_binding_artboards_test.riv", "ch2"))),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "ab",
            Some(("data_binding_artboards_source_test.riv", "default")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Compare("data_binding_artboards_default_test"),
    ]);
}

#[test]
fn scripted_artboard_input_bound_to_internal_and_external_artboards() {
    execute_until_concrete_parity_boundary(&[
        Action::ImportScripted("data_bind_artboard_input.riv"),
        Action::Import("data_binding_artboards_source_test.riv"),
        Action::SelectArtboard("default"),
        Action::FrameSize,
        Action::SelectStateMachine,
        Action::CreateFreshViewModel,
        Action::BindViewModel,
        Action::Draw,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "artboardProperty",
            Some(("data_bind_artboard_input.riv", "child2")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "artboardProperty",
            Some(("data_bind_artboard_input.riv", "child1")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "artboardProperty",
            Some(("data_binding_artboards_source_test.riv", "default")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboardIndex("artboardProperty", 1),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboardIndex("artboardProperty", 10),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Compare("data_bind_artboard_input"),
    ]);
}

#[test]
fn external_artboard_with_no_initial_source() {
    execute_until_concrete_parity_boundary(&[
        Action::Import("databind_external_artboard_main.riv"),
        Action::Import("databind_external_artboard_child.riv"),
        Action::SelectArtboard("default"),
        Action::FrameSize,
        Action::SelectStateMachine,
        Action::Advance(0.0),
        Action::Draw,
        Action::Frame,
        Action::CreateFreshViewModel,
        Action::BindViewModel,
        Action::SetArtboard(
            "ab",
            Some(("databind_external_artboard_child.riv", "ExternalChild")),
        ),
        Action::ReplaceViewModel("child", "Child", "Instance"),
        Action::SetString("child/label", "updated label"),
        Action::Advance(0.016),
        Action::Draw,
        Action::Compare("databind_external_artboard_main"),
    ]);
}

#[test]
#[ignore = "expected-red: imported bindable_artboard_child lacks ViewModelRenamed lookup"]
fn bound_artboard_with_view_model_instance_resets_properties() {
    execute_until_concrete_parity_boundary(&[
        Action::Import("bindable_artboard_nesty.riv"),
        Action::Import("bindable_artboard_child.riv"),
        Action::SelectArtboard("default"),
        Action::FrameSize,
        Action::SelectStateMachine,
        Action::CreateFreshViewModel,
        Action::SetArtboard(
            "someArtboard",
            Some(("bindable_artboard_child.riv", "Artboard")),
        ),
        Action::SetBoundViewModel("someArtboard", "ViewModelRenamed", "new"),
        Action::BindViewModel,
        Action::Advance(0.016),
        Action::Draw,
        Action::Frame,
        Action::PointerDown(250.0, 250.0),
        Action::PointerUp(250.0, 250.0),
        Action::Advance(0.016),
        Action::Draw,
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Frame,
        Action::PointerDown(250.0, 250.0),
        Action::PointerUp(250.0, 250.0),
        Action::Advance(0.016),
        Action::Draw,
        Action::Compare("bindable_artboard_nesty"),
    ]);
}

#[test]
#[ignore = "expected-red: frame 0 op 31 expected makeRenderPaint, got save"]
fn multiple_targets_bound_to_same_artboard_property_bidirectionally() {
    let mut actions = vec![
        Action::Import("bidirectional_binding_source.riv"),
        Action::Import("bidirectional_binding_target_1.riv"),
        Action::Import("bidirectional_binding_target_2.riv"),
        Action::SelectArtboard("default"),
        Action::FrameSize,
        Action::SelectStateMachine,
        Action::CreateDefaultViewModel,
        Action::BindViewModel,
        Action::Advance(0.0),
        Action::SetBool("costume_db_bool", true),
        Action::Draw,
        Action::Frame,
        Action::Advance(0.016),
        Action::Draw,
    ];
    for source in [
        "bidirectional_binding_target_1.riv",
        "bidirectional_binding_target_2.riv",
        "bidirectional_binding_target_1.riv",
        "bidirectional_binding_target_2.riv",
    ] {
        actions.extend([
            Action::Frame,
            Action::SetArtboard("costume_db_artboard", Some((source, "costume_artboard"))),
            Action::Advance(0.016),
            Action::Draw,
            Action::Frame,
            Action::Advance(0.016),
            Action::Draw,
        ]);
    }
    actions.push(Action::Compare("bidirectional_binding_source"));
    execute_until_concrete_parity_boundary(&actions);
}

#[test]
fn null_bound_artboard_swap_survives_pending_layout_sync() {
    execute_until_concrete_parity_boundary(&[
        Action::Import("databind_null_artboard_swap.riv"),
        Action::SelectArtboard("default"),
        Action::CreateFreshViewModel,
        Action::BindArtboardViewModel,
        Action::ExpectNested("swap host", true),
        Action::AdvanceArtboard(0.0),
        Action::AdvanceArtboard(0.0),
        Action::ExpectNested("swap host", false),
    ]);
}
