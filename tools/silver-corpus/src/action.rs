use crate::scripting::import_file;
use crate::{ActionTarget::*, Case};
use anyhow::{Context, bail};
use nuxie_render_api::{Factory as RenderFactory, PersistentFactory, SerializingFactory};
use nuxie_runtime::source::{
    animation::{
        linear_animation_instance::LinearAnimationInstance,
        state_machine_instance::RuntimeStateMachineInstanceHandle,
    },
    constraints::scrolling::scroll_constraint::ScrollConstraint,
    data_bind::data_context::{DataContext, RuntimeDataContextHandle},
    generated::{
        core_registry::CoreRegistry,
        layout_component_base::LayoutComponentBase,
        viewmodel::{
            viewmodel_instance_artboard_base::ViewModelInstanceArtboardBase,
            viewmodel_instance_asset_base::ViewModelInstanceAssetBase,
        },
    },
    input::{
        focusable::{Key, KeyModifiers},
        gamepad_batch::GAMEPAD_BATCH_WIRE_VERSION,
    },
    math::{
        random::{RandomProvider, set_runtime_deterministic_mode},
        vec2d::Vec2D,
    },
    text::font_hb::HbFont,
    viewmodel::runtime::viewmodel_instance_runtime::ViewModelInstanceRuntime,
};
use nuxie_runtime::{
    Artboard as NativeArtboard, CoreHandle, File, RuntimeArtboardInstanceHandle,
    RuntimeFactoryHandle, RuntimeFileHandle,
};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ActionTarget {
    Artboard,
    StateMachine,
    Animation,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum GamepadMapping {
    #[default]
    Standard,
    Unknown,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum GamepadInputKind {
    Button,
    Axis,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum GamepadRecord {
    Connected {
        device_id: i32,
        #[serde(default = "default_gamepad_button_count")]
        button_count: u8,
        #[serde(default = "default_gamepad_axis_count")]
        axis_count: u8,
        #[serde(default)]
        mapping: GamepadMapping,
    },
    Update {
        device_id: i32,
        input: GamepadInputKind,
        index: u8,
        value: f32,
    },
    Disconnected {
        device_id: i32,
    },
}

const fn default_gamepad_button_count() -> u8 {
    17
}

const fn default_gamepad_axis_count() -> u8 {
    4
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum PointerCoordinate {
    Literal(f32),
    Expression(String),
}

impl PointerCoordinate {
    fn resolve(&self, width: f32, height: f32) -> anyhow::Result<f32> {
        match self {
            Self::Literal(value) => Ok(*value),
            Self::Expression(expression) => {
                if let Some(distance) = expression.strip_prefix("artboard-width/2+") {
                    let distance = distance.parse::<f32>().with_context(|| {
                        format!("invalid pointer coordinate expression {expression}")
                    })?;
                    return Ok(width / 2.0 + distance);
                }
                if let Some(distance) = expression.strip_prefix("artboard-height/2-") {
                    let distance = distance.parse::<f32>().with_context(|| {
                        format!("invalid pointer coordinate expression {expression}")
                    })?;
                    return Ok(height / 2.0 - distance);
                }
                Ok(match expression.as_str() {
                    "artboard-width/2" => width / 2.0,
                    "artboard-height/2" => height / 2.0,
                    "artboard-width*0.8" => width * 0.8,
                    "artboard-height-20" => height - 20.0,
                    _ => bail!("unsupported pointer coordinate expression {expression}"),
                })
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Action {
    BindDefaultViewModel,
    BindFreshViewModel,
    BindAuthoredViewModel,
    BindAuthoredViewModelInstance {
        instance_index: usize,
    },
    BindNamedDefaultViewModel {
        view_model: String,
    },
    CreateDefaultViewModel,
    BindPreparedViewModel,
    CreateNamedViewModel {
        view_model: String,
    },
    SetNamedViewModelNumber {
        view_model: String,
        property: String,
        value: f32,
    },
    ReplaceViewModel {
        property: String,
        view_model: String,
    },
    CreateDefaultMainViewModel,
    CreateDefaultGlobalViewModel {
        global: String,
    },
    SetStagedMainString {
        property: String,
        value: String,
    },
    SetStagedGlobalColor {
        global: String,
        property: String,
        value: u32,
    },
    SetStateMachineMainViewModel,
    SetStateMachineGlobalViewModel {
        global: String,
    },
    SetStateMachineDefaultGlobalViewModels,
    BindStateMachineViewModels,
    SetViewModelNumber {
        property: String,
        value: f32,
    },
    SetViewModelBoolean {
        property: String,
        value: bool,
    },
    SetViewModelString {
        property: String,
        value: String,
    },
    SetViewModelEnum {
        property: String,
        value: u64,
    },
    SetViewModelColor {
        property: String,
        value: u32,
    },
    FireViewModelTrigger {
        property: String,
    },
    SetViewModelArtboard {
        property: String,
        value: u64,
    },
    SetViewModelArtboardByName {
        property: String,
        artboard: String,
    },
    SetViewModelBindableArtboardByName {
        property: String,
        artboard: String,
    },
    ClearViewModelArtboard {
        property: String,
    },
    SetViewModelArtboardFromFile {
        property: String,
        file: String,
        artboard: String,
    },
    ReplaceViewModelFromFile {
        property: String,
        file: String,
        view_model: String,
        instance: String,
    },
    SetViewModelAsset {
        property: String,
        value: i64,
    },
    SetViewModelAssetByName {
        property: String,
        asset: String,
    },
    SetViewModelFontBytes {
        property: String,
        source: String,
    },
    SetGlobalViewModelColor {
        global: String,
        property: String,
        value: u32,
    },
    FireViewModelListItemTrigger {
        list: String,
        index: usize,
        trigger: String,
    },
    AppendViewModelListItem {
        list: String,
        view_model: String,
        #[serde(default)]
        index: Option<usize>,
        #[serde(default)]
        number_properties: BTreeMap<String, f32>,
        #[serde(default)]
        string_property: Option<String>,
        #[serde(default)]
        string_value: Option<String>,
    },
    RemoveViewModelListItem {
        list: String,
        index: usize,
    },
    SetViewModelListItemNumber {
        list: String,
        index: usize,
        property: String,
        value: f32,
    },
    ClearRandoms,
    AddRandomValue {
        value: f32,
    },
    AssertRandomCalls {
        count: i32,
    },
    Advance {
        target: ActionTarget,
        seconds: f32,
    },
    Draw,
    Frame,
    FrameSize,
    LayoutFrameSize,
    PointerDown {
        x: PointerCoordinate,
        y: PointerCoordinate,
        #[serde(default)]
        pointer_id: i32,
    },
    PointerMove {
        x: PointerCoordinate,
        y: PointerCoordinate,
        seconds: f32,
        #[serde(default)]
        pointer_id: i32,
    },
    PointerUp {
        x: PointerCoordinate,
        y: PointerCoordinate,
        #[serde(default)]
        pointer_id: i32,
    },
    PointerExit {
        x: PointerCoordinate,
        y: PointerCoordinate,
        #[serde(default)]
        pointer_id: i32,
    },
    VerticalPointerDrag {
        x: PointerCoordinate,
        start_y: PointerCoordinate,
        end_y_exclusive: f32,
        step: f32,
        advance_seconds: f32,
        #[serde(default)]
        pointer_id: i32,
    },
    SetBool {
        input: String,
        value: bool,
    },
    SetNumber {
        input: String,
        value: f32,
    },
    FireTrigger {
        input: String,
    },
    TextInput {
        text: String,
    },
    FocusNext,
    FocusPrevious,
    KeyInput {
        key: u32,
        modifiers: u32,
        pressed: bool,
        repeat: bool,
    },
    GamepadBatch {
        records: Vec<GamepadRecord>,
    },
    SetArtboardSize {
        width: f32,
        height: f32,
    },
    AdvanceDrawUntilScrollPhysicsStops {
        max_frames: usize,
        seconds: f32,
    },
    AdvanceDrawFrames {
        frames: usize,
        seconds: f32,
    },
}

pub struct Execution {
    bytes: Vec<u8>,
}

struct RandomTestingModeGuard;

impl Drop for RandomTestingModeGuard {
    fn drop(&mut self) {
        RandomProvider::clear_testing_mode();
    }
}

impl Execution {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn run(case: &Case, runtime_dir: &Path) -> anyhow::Result<Self> {
        let actions = case
            .actions
            .executable()
            .with_context(|| format!("{} has no executable action stream", case.id))?;
        // The pinned Silver producers are unit tests compiled with TESTING.
        // In that build RandomProvider starts in FIFO mode and returns 0 when
        // the queue is empty; it never falls through to the platform RNG.
        let _random_testing_mode = RandomTestingModeGuard;
        RandomProvider::clear_randoms();
        set_runtime_deterministic_mode(case.deterministic == "enabled");
        let mut first_runtime_action = 0;
        while let Some(action) = actions.get(first_runtime_action) {
            match action {
                Action::ClearRandoms => RandomProvider::clear_randoms(),
                Action::AssertRandomCalls { count } => {
                    let actual = RandomProvider::total_calls();
                    if actual != *count {
                        bail!("random call count before import: expected {count}, got {actual}");
                    }
                }
                _ => break,
            }
            first_runtime_action += 1;
        }
        let fixture = runtime_dir
            .join("tests/unit_tests/assets")
            .join(&case.source);
        let bytes = std::fs::read(&fixture)
            .with_context(|| format!("read fixture {}", fixture.display()))?;
        let mut factory = PersistentFactory::new(SerializingFactory::new());
        let retained_factory = RuntimeFactoryHandle::from_factory(&mut factory)
            .context("retained serializing factory")?;
        let file = import_file(&bytes, retained_factory.clone())?;
        let mut files = BTreeMap::from([(case.source.clone(), file.clone())]);
        for dependency in &case.dependencies {
            let path = runtime_dir.join("tests/unit_tests/assets").join(dependency);
            let bytes =
                std::fs::read(&path).with_context(|| format!("read fixture {}", path.display()))?;
            files.insert(
                dependency.clone(),
                import_file(&bytes, retained_factory.clone())?,
            );
        }
        let source = select_artboard(&file, &case.artboard)?;
        let first_instance =
            NativeArtboard::instance_from_handle(&source).context("instantiate native artboard")?;
        // File::artboardNamed already instances the source. Some pinned tests
        // then call instance() again and retain both instances for the replay.
        let instance = if case.clone_artboard_instance {
            NativeArtboard::instance_from_handle(&first_instance.core_handle())
                .context("clone selected artboard instance")?
        } else {
            first_instance.clone()
        };
        let (width, height) =
            instance.with_artboard(|artboard| (artboard.width(), artboard.height()));
        // Some pinned tests initialize a bound scene before recording its
        // frame size. An explicit action preserves that setup/resource order.
        if !actions
            .iter()
            .any(|action| matches!(action, Action::FrameSize | Action::LayoutFrameSize))
        {
            factory
                .borrow_mut()
                .frame_size(frame_dimension(width), frame_dimension(height));
        }
        let mut renderer = factory.borrow().make_renderer();
        let state_machine = select_state_machine(&instance, &case.state_machine)?;
        let mut animation = select_animation(&instance, &case.animation)?;
        let mut owned_context: Option<RuntimeDataContextHandle> = None;
        let mut staged_main: Option<CoreHandle> = None;
        let mut staged_globals = BTreeMap::<String, CoreHandle>::new();
        let mut staged_named = BTreeMap::<String, CoreHandle>::new();
        for action in &actions[first_runtime_action..] {
            match action {
                Action::BindDefaultViewModel => {
                    let main = file.with_file_mut(|file| {
                        file.create_default_view_model_instance_for_artboard(source.clone())
                    });
                    let context = complete_context(&file, main);
                    bind_context(&instance, state_machine.as_ref(), &context);
                    owned_context = Some(context);
                }
                Action::BindFreshViewModel => {
                    let main = file
                        .with_file_mut(|file| {
                            file.create_view_model_instance_for_artboard(source.clone())
                        })
                        .context("selected artboard has no view-model schema")?;
                    let context = complete_context(&file, Some(main));
                    bind_context(&instance, state_machine.as_ref(), &context);
                    owned_context = Some(context);
                }
                Action::BindAuthoredViewModel => {
                    let id = instance.with_artboard(|artboard| artboard.base.view_model_id());
                    let main = file
                        .with_file_mut(|file| {
                            if id == u32::MAX {
                                file.create_view_model_instance_for_artboard(instance.core_handle())
                            } else {
                                file.create_view_model_instance_at(id as usize, 0)
                            }
                        })
                        .context("selected artboard has no authored view-model instance")?;
                    let context = complete_context(&file, Some(main));
                    bind_context(&instance, state_machine.as_ref(), &context);
                    owned_context = Some(context);
                }
                Action::BindAuthoredViewModelInstance { instance_index } => {
                    let id = instance.with_artboard(|artboard| artboard.base.view_model_id());
                    let main = file.with_file(|file| {
                        file.create_view_model_instance_at(id as usize, *instance_index)
                    });
                    let machine = state_machine
                        .as_ref()
                        .context("authored instance binding requires a state machine")?;
                    // C++ passes the nullable lookup directly to bind: None
                    // clears/unbinds context, without a default-instance fallback.
                    owned_context = machine.with_instance_mut(|machine| {
                        machine.bind_view_model_instance(main);
                        machine.data_context()
                    });
                }
                Action::BindNamedDefaultViewModel { view_model } => {
                    let main = named_model_instance(&file, view_model, true)?;
                    let context = complete_context(&file, Some(main));
                    bind_context(&instance, state_machine.as_ref(), &context);
                    owned_context = Some(context);
                }
                Action::CreateDefaultViewModel => {
                    let main = file
                        .with_file_mut(|file| {
                            file.create_default_view_model_instance_for_artboard(source.clone())
                        })
                        .context("selected artboard has no default view model")?;
                    owned_context = Some(complete_context(&file, Some(main)));
                }
                Action::BindPreparedViewModel => {
                    bind_context(
                        &instance,
                        state_machine.as_ref(),
                        owned_context
                            .as_ref()
                            .context("no prepared view-model instance")?,
                    );
                }
                Action::CreateNamedViewModel { view_model } => {
                    staged_named.insert(
                        view_model.clone(),
                        named_model_instance(&file, view_model, false)?,
                    );
                }
                Action::SetNamedViewModelNumber {
                    view_model,
                    property,
                    value,
                } => {
                    ViewModelInstanceRuntime::new(
                        staged_named
                            .get(view_model)
                            .with_context(|| format!("no staged view model {view_model}"))?
                            .clone(),
                    )
                    .property_number(property)
                    .with_context(|| {
                        format!("missing staged number property {view_model}.{property}")
                    })?
                    .set_value(*value);
                }
                Action::ReplaceViewModel {
                    property,
                    view_model,
                } => {
                    let replacement = ViewModelInstanceRuntime::new(
                        staged_named
                            .get(view_model)
                            .with_context(|| format!("no staged view model {view_model}"))?
                            .clone(),
                    )
                    .into_handle();
                    if !main_runtime(&owned_context)?.replace_view_model(property, replacement) {
                        bail!("failed to replace {property} with staged view model {view_model}");
                    }
                }
                Action::CreateDefaultMainViewModel => {
                    staged_main = Some(
                        file.with_file_mut(|file| {
                            file.create_default_view_model_instance_for_artboard(source.clone())
                        })
                        .context("selected artboard has no default main view-model instance")?,
                    );
                }
                Action::CreateDefaultGlobalViewModel { global } => {
                    staged_globals
                        .insert(global.clone(), named_model_instance(&file, global, true)?);
                }
                Action::SetStagedMainString { property, value } => {
                    ViewModelInstanceRuntime::new(
                        staged_main
                            .as_ref()
                            .context("no staged main view-model instance")?
                            .clone(),
                    )
                    .property_string(property)
                    .with_context(|| format!("missing staged main string property {property}"))?
                    .set_value(value.clone());
                }
                Action::SetStagedGlobalColor {
                    global,
                    property,
                    value,
                } => {
                    ViewModelInstanceRuntime::new(
                        staged_globals
                            .get(global)
                            .with_context(|| format!("no staged global view model {global}"))?
                            .clone(),
                    )
                    .property_color(property)
                    .with_context(|| {
                        format!("missing staged global color property {global}.{property}")
                    })?
                    .set_value(*value as i32);
                }
                Action::SetStateMachineMainViewModel => {
                    let value = staged_main
                        .as_ref()
                        .context("no staged main view-model instance")?
                        .clone();
                    machine(&state_machine)?
                        .with_instance_mut(|machine| machine.set_view_model_instance(value));
                }
                Action::SetStateMachineGlobalViewModel { global } => {
                    let value = staged_globals
                        .get(global)
                        .with_context(|| format!("no staged global view model {global}"))?
                        .clone();
                    if !machine(&state_machine)?.with_instance_mut(|machine| {
                        machine.set_global_view_model_instance(global, value)
                    }) {
                        bail!("state machine rejected staged global view model {global}");
                    }
                }
                Action::SetStateMachineDefaultGlobalViewModels => {
                    for global in file.with_file(File::global_view_model_names) {
                        let value = named_model_instance(&file, &global, true)?;
                        if !machine(&state_machine)?.with_instance_mut(|machine| {
                            machine.set_global_view_model_instance(&global, value)
                        }) {
                            bail!("state machine rejected default global view model {global}");
                        }
                    }
                }
                Action::BindStateMachineViewModels => {
                    machine(&state_machine)?.with_instance_mut(|machine| machine.bind())
                }
                Action::SetViewModelNumber { property, value } => {
                    main_runtime(&owned_context)?
                        .property_number(property)
                        .with_context(|| format!("missing numeric view-model property {property}"))?
                        .set_value(*value);
                }
                Action::SetViewModelBoolean { property, value } => {
                    main_runtime(&owned_context)?
                        .property_boolean(property)
                        .with_context(|| format!("missing boolean view-model property {property}"))?
                        .set_value(*value);
                }
                Action::SetViewModelString { property, value } => {
                    main_runtime(&owned_context)?
                        .property_string(property)
                        .with_context(|| format!("missing string view-model property {property}"))?
                        .set_value(value.clone());
                }
                Action::SetViewModelEnum { property, value } => {
                    // Pinned serialized_rendering_test.cpp calls value(index)
                    // without inspecting its bool result. A system enum may
                    // reject an index; that is an authored no-op, not a failed
                    // harness action (nor permission to write the raw index).
                    main_runtime(&owned_context)?
                        .property_enum(property)
                        .with_context(|| format!("missing enum view-model property {property}"))?
                        .set_value_index(
                            u32::try_from(*value).context("enum index exceeds source uint32")?,
                        );
                }
                Action::SetViewModelColor { property, value } => {
                    main_runtime(&owned_context)?
                        .property_color(property)
                        .with_context(|| format!("missing color view-model property {property}"))?
                        .set_value(*value as i32);
                }
                Action::FireViewModelTrigger { property } => {
                    main_runtime(&owned_context)?
                        .property_trigger(property)
                        .with_context(|| format!("missing trigger view-model property {property}"))?
                        .trigger();
                }
                Action::SetViewModelArtboard { property, value } => {
                    set_artboard_id(
                        &main_runtime(&owned_context)?,
                        property,
                        u32::try_from(*value).context("artboard index exceeds source uint32")?,
                    )?;
                }
                Action::SetViewModelArtboardByName { property, artboard } => {
                    let index = file
                        .with_file(|file| {
                            file.artboards().iter().position(|candidate| {
                                candidate
                                    .with_downcast::<NativeArtboard, _>(|candidate| {
                                        candidate.name() == artboard
                                    })
                                    .unwrap_or(false)
                            })
                        })
                        .with_context(|| format!("missing artboard {artboard}"))?;
                    set_artboard_id(
                        &main_runtime(&owned_context)?,
                        property,
                        u32::try_from(index)?,
                    )?;
                }
                Action::SetViewModelBindableArtboardByName { property, artboard } => {
                    let bindable = file
                        .with_file(|file| file.bindable_artboard_named(artboard))
                        .with_context(|| format!("missing bindable artboard {artboard}"))?;
                    main_runtime(&owned_context)?
                        .property_artboard(property)
                        .with_context(|| format!("missing artboard property {property}"))?
                        .set_value(Some(bindable));
                }
                Action::ClearViewModelArtboard { property } => {
                    main_runtime(&owned_context)?
                        .property_artboard(property)
                        .with_context(|| format!("missing artboard property {property}"))?
                        .set_value(None);
                }
                Action::SetViewModelArtboardFromFile {
                    property,
                    file: source_file,
                    artboard,
                } => {
                    let source_file = files
                        .get(source_file)
                        .with_context(|| format!("missing imported file {source_file}"))?;
                    let bindable = source_file
                        .with_file(|file| {
                            if artboard == "default" {
                                file.bindable_artboard_default()
                            } else {
                                file.bindable_artboard_named(artboard)
                            }
                        })
                        .with_context(|| format!("missing bindable artboard {artboard}"))?;
                    main_runtime(&owned_context)?
                        .property_artboard(property)
                        .with_context(|| format!("missing artboard property {property}"))?
                        .set_value(Some(bindable));
                }
                Action::ReplaceViewModelFromFile {
                    property,
                    file: source_file,
                    view_model,
                    instance: instance_name,
                } => {
                    let source_file = files
                        .get(source_file)
                        .with_context(|| format!("missing imported file {source_file}"))?;
                    let replacement = source_file
                        .with_file(|file| file.view_model_by_name(view_model))
                        .and_then(|model| model.create_instance_from_name(instance_name))
                        .with_context(|| {
                            format!("missing view model {view_model}/{instance_name}")
                        })?;
                    if !main_runtime(&owned_context)?.replace_view_model(property, replacement) {
                        bail!("failed to replace view-model property {property}");
                    }
                }
                Action::SetViewModelAsset { property, value } => {
                    set_asset_id(&main_runtime(&owned_context)?, property, *value as u32)?;
                }
                Action::SetViewModelAssetByName { property, asset } => {
                    let assets = file.with_file(|file| file.assets().to_vec());
                    let index = assets
                        .iter()
                        .position(|candidate| {
                            candidate
                                .with(|candidate| {
                                    candidate.as_file_asset().map(|candidate| {
                                        candidate.file_asset_base().name() == asset
                                    })
                                })
                                .flatten()
                                .unwrap_or(false)
                        })
                        .with_context(|| format!("missing file asset {asset}"))?;
                    set_asset_id(
                        &main_runtime(&owned_context)?,
                        property,
                        u32::try_from(index)?,
                    )?;
                }
                Action::SetViewModelFontBytes { property, source } => {
                    let property = main_runtime(&owned_context)?
                        .property_font(property)
                        .with_context(|| format!("missing font view-model property {property}"))?;
                    let path = runtime_dir.join("tests/unit_tests/assets").join(source);
                    let bytes = std::fs::read(&path)
                        .with_context(|| format!("read font fixture {}", path.display()))?;
                    let decoded = factory
                        .borrow_mut()
                        .decode_font(&bytes)
                        .context("factory decoded font")?;
                    let font = HbFont::decode(decoded.bytes()).context("native decoded font")?;
                    property.set_value(Some(font));
                }
                Action::SetGlobalViewModelColor {
                    global,
                    property,
                    value,
                } => {
                    let slot = file.with_file(|file| file.view_model_id(global));
                    let instance = owned_context
                        .as_ref()
                        .context("no prepared view-model instance")?
                        .with_context(|context| context.instance_for_slot(slot))
                        .with_context(|| format!("missing global view model {global}"))?;
                    ViewModelInstanceRuntime::new(instance)
                        .property_color(property)
                        .with_context(|| format!("missing color property {global}.{property}"))?
                        .set_value(*value as i32);
                }
                Action::FireViewModelListItemTrigger {
                    list,
                    index,
                    trigger,
                } => {
                    let list_runtime = main_runtime(&owned_context)?
                        .property_list(list)
                        .with_context(|| format!("missing list property {list}"))?;
                    let item = list_runtime
                        .instance_at(i32::try_from(*index)?)
                        .with_context(|| format!("missing list item {list}[{index}]"))?;
                    item.property_trigger(trigger)
                        .with_context(|| format!("missing trigger {list}[{index}].{trigger}"))?
                        .trigger();
                }
                Action::AppendViewModelListItem {
                    list,
                    view_model,
                    index,
                    number_properties,
                    string_property,
                    string_value,
                } => {
                    let main = main_runtime(&owned_context)?;
                    let child = ViewModelInstanceRuntime::new(named_model_instance(
                        &file, view_model, false,
                    )?)
                    .into_handle();
                    for (property, value) in number_properties {
                        child
                            .property_number(property)
                            .with_context(|| {
                                format!("missing numeric property {view_model}.{property}")
                            })?
                            .set_value(*value);
                    }
                    if let (Some(property), Some(value)) = (string_property, string_value) {
                        child
                            .property_string(property)
                            .with_context(|| {
                                format!("missing string property {view_model}.{property}")
                            })?
                            .set_value(value.clone());
                    }
                    let list_runtime = main
                        .property_list(list)
                        .with_context(|| format!("missing list property {list}"))?;
                    let index = index.unwrap_or_else(|| list_runtime.size());
                    if !list_runtime.add_instance_at(child, i32::try_from(index)?) {
                        bail!("failed to append {view_model} to {list}");
                    }
                }
                Action::RemoveViewModelListItem { list, index } => {
                    let list_runtime = main_runtime(&owned_context)?
                        .property_list(list)
                        .with_context(|| format!("missing list property {list}"))?;
                    if *index >= list_runtime.size() {
                        bail!("missing list item {list}[{index}]");
                    }
                    list_runtime.remove_instance_at(i32::try_from(*index)?);
                }
                Action::SetViewModelListItemNumber {
                    list,
                    index,
                    property,
                    value,
                } => {
                    let list_runtime = main_runtime(&owned_context)?
                        .property_list(list)
                        .with_context(|| format!("missing list property {list}"))?;
                    let item = list_runtime
                        .instance_at(i32::try_from(*index)?)
                        .with_context(|| format!("missing list item {list}[{index}]"))?;
                    item.property_number(property)
                        .with_context(|| {
                            format!("missing numeric property {list}[{index}].{property}")
                        })?
                        .set_value(*value);
                }
                Action::ClearRandoms => RandomProvider::clear_randoms(),
                Action::AddRandomValue { value } => RandomProvider::add_random_value(*value),
                Action::AssertRandomCalls { count } => {
                    let actual = RandomProvider::total_calls();
                    if actual != *count {
                        bail!("random call count: expected {count}, got {actual}");
                    }
                }
                Action::Advance { target, seconds } => match target {
                    StateMachine => {
                        machine(&state_machine)?.advance_and_apply(*seconds);
                    }
                    Artboard => {
                        instance.advance_default(*seconds);
                    }
                    Animation => {
                        let animation = animation.as_mut().context("no selected animation")?;
                        animation.advance_and_report_to_self(*seconds);
                        animation.apply(1.0);
                        NativeArtboard::update_pass_handle(&instance.core_handle(), true);
                    }
                },
                Action::Draw => instance.draw(&mut renderer),
                Action::Frame => factory.borrow_mut().add_frame(),
                Action::FrameSize => {
                    let (width, height) =
                        instance.with_artboard(|artboard| (artboard.width(), artboard.height()));
                    factory
                        .borrow_mut()
                        .frame_size(frame_dimension(width), frame_dimension(height));
                }
                Action::LayoutFrameSize => {
                    let (width, height) = instance.with_artboard(|artboard| {
                        (artboard.layout_width(), artboard.layout_height())
                    });
                    factory
                        .borrow_mut()
                        .frame_size(frame_dimension(width), frame_dimension(height));
                }
                Action::PointerDown { x, y, pointer_id } => {
                    let position = pointer_position(x, y, &instance)?;
                    machine(&state_machine)?
                        .with_instance_mut(|machine| machine.pointer_down(position, *pointer_id));
                }
                Action::PointerMove {
                    x,
                    y,
                    seconds,
                    pointer_id,
                } => {
                    let position = pointer_position(x, y, &instance)?;
                    machine(&state_machine)?.with_instance_mut(|machine| {
                        machine.pointer_move(position, *seconds, *pointer_id)
                    });
                }
                Action::PointerUp { x, y, pointer_id } => {
                    let position = pointer_position(x, y, &instance)?;
                    machine(&state_machine)?
                        .with_instance_mut(|machine| machine.pointer_up(position, *pointer_id));
                }
                Action::PointerExit { x, y, pointer_id } => {
                    let position = pointer_position(x, y, &instance)?;
                    machine(&state_machine)?
                        .with_instance_mut(|machine| machine.pointer_exit(position, *pointer_id));
                }
                Action::VerticalPointerDrag {
                    x,
                    start_y,
                    end_y_exclusive,
                    step,
                    advance_seconds,
                    pointer_id,
                } => {
                    let (width, height) =
                        instance.with_artboard(|artboard| (artboard.width(), artboard.height()));
                    let x = x.resolve(width, height)?;
                    let mut y = start_y.resolve(width, height)?;
                    let machine = machine(&state_machine)?;
                    factory.borrow_mut().add_frame();
                    machine.with_instance_mut(|machine| {
                        machine.pointer_down(Vec2D::new(x, y), *pointer_id)
                    });
                    machine.advance_and_apply(*advance_seconds);
                    instance.draw(&mut renderer);
                    while y > *end_y_exclusive {
                        factory.borrow_mut().add_frame();
                        machine.with_instance_mut(|machine| {
                            machine.pointer_move(Vec2D::new(x, y), 0.0, *pointer_id)
                        });
                        machine.advance_and_apply(*advance_seconds);
                        instance.draw(&mut renderer);
                        y -= *step;
                    }
                    factory.borrow_mut().add_frame();
                    machine.with_instance_mut(|machine| {
                        machine.pointer_up(Vec2D::new(x, y), *pointer_id)
                    });
                    machine.advance_and_apply(*advance_seconds);
                    instance.draw(&mut renderer);
                }
                Action::SetBool { input, value } => {
                    let machine = machine(&state_machine)?;
                    if !machine.with_instance(|machine| machine.get_bool(input).is_some()) {
                        bail!("missing boolean input {input}");
                    }
                    machine.set_bool(input, *value);
                }
                Action::SetNumber { input, value } => {
                    let machine = machine(&state_machine)?;
                    if !machine.with_instance(|machine| machine.get_number(input).is_some()) {
                        bail!("missing numeric input {input}");
                    }
                    machine.set_number(input, *value);
                }
                Action::FireTrigger { input } => {
                    machine(&state_machine)?.with_instance_mut(|machine| {
                        let trigger = machine
                            .get_trigger_mut(input)
                            .with_context(|| format!("missing trigger input {input}"))?;
                        trigger.fire();
                        Ok::<_, anyhow::Error>(())
                    })?;
                }
                Action::TextInput { text } => {
                    let focus =
                        machine(&state_machine)?.with_instance(|machine| machine.focus_manager());
                    if !focus.with_focus_manager_mut(|focus| focus.text_input(text)) {
                        bail!("text input was not handled");
                    }
                }
                Action::FocusNext => {
                    machine(&state_machine)?.with_instance_mut(|machine| machine.focus_next());
                }
                Action::FocusPrevious => {
                    machine(&state_machine)?.with_instance_mut(|machine| machine.focus_previous());
                }
                Action::KeyInput {
                    key,
                    modifiers,
                    pressed,
                    repeat,
                } => {
                    let focus =
                        machine(&state_machine)?.with_instance(|machine| machine.focus_manager());
                    focus.with_focus_manager_mut(|focus| {
                        focus.key_input(
                            Key::from_raw(*key),
                            KeyModifiers::from_raw(*modifiers),
                            *pressed,
                            *repeat,
                        )
                    });
                }
                Action::GamepadBatch { records } => {
                    let bytes = encode_gamepad_batch(records);
                    if !machine(&state_machine)?.submit_gamepads_from_buffer(&bytes) {
                        bail!("gamepad batch rejected");
                    }
                }
                Action::SetArtboardSize { width, height } => {
                    if !CoreRegistry::set_double_handle(
                        &instance.core_handle(),
                        i32::from(LayoutComponentBase::WIDTH_PROPERTY_KEY),
                        *width,
                    ) || !CoreRegistry::set_double_handle(
                        &instance.core_handle(),
                        i32::from(LayoutComponentBase::HEIGHT_PROPERTY_KEY),
                        *height,
                    ) {
                        bail!("artboard dimensions setter missing");
                    }
                }
                Action::AdvanceDrawUntilScrollPhysicsStops {
                    max_frames,
                    seconds,
                } => {
                    let scroll = instance
                        .with_artboard(|artboard| artboard.find_all_handles::<ScrollConstraint>())
                        .into_iter()
                        .next()
                        .context("selected artboard has no ScrollConstraint")?;
                    let physics = scroll
                        .with_downcast::<ScrollConstraint, _>(ScrollConstraint::physics)
                        .flatten()
                        .context("selected ScrollConstraint has no physics")?;
                    for _ in 0..*max_frames {
                        factory.borrow_mut().add_frame();
                        machine(&state_machine)?.advance_and_apply(*seconds);
                        instance.draw(&mut renderer);
                        let running = physics
                            .with(|physics| {
                                physics
                                    .as_scroll_physics_runtime()
                                    .map(|physics| physics.is_running())
                            })
                            .flatten()
                            .context("selected ScrollPhysics disappeared")?;
                        if !running {
                            break;
                        }
                    }
                }
                Action::AdvanceDrawFrames { frames, seconds } => {
                    for _ in 0..*frames {
                        factory.borrow_mut().add_frame();
                        machine(&state_machine)?.advance_and_apply(*seconds);
                        instance.draw(&mut renderer);
                    }
                }
            }
        }
        drop(renderer);
        let bytes = factory.borrow().bytes().to_vec();
        Ok(Self { bytes })
    }
}

fn machine(
    machine: &Option<RuntimeStateMachineInstanceHandle>,
) -> anyhow::Result<&RuntimeStateMachineInstanceHandle> {
    machine.as_ref().context("no selected state machine")
}
fn select_artboard(file: &RuntimeFileHandle, selector: &str) -> anyhow::Result<CoreHandle> {
    file.with_file(|file| {
        if selector == "default" {
            file.artboard()
        } else {
            file.artboard_named_source(selector)
        }
    })
    .with_context(|| format!("missing artboard {selector}"))
}
fn select_state_machine(
    instance: &RuntimeArtboardInstanceHandle,
    selector: &str,
) -> anyhow::Result<Option<RuntimeStateMachineInstanceHandle>> {
    if selector == "none" {
        return Ok(None);
    }
    let selected = if selector == "default" {
        instance.state_machine_at(0)
    } else {
        instance.state_machine_named(selector)
    };
    selected
        .map(Some)
        .with_context(|| format!("missing state machine {selector}"))
}
fn select_animation(
    instance: &RuntimeArtboardInstanceHandle,
    selector: &str,
) -> anyhow::Result<Option<Box<LinearAnimationInstance>>> {
    if selector == "none" {
        return Ok(None);
    }
    let selected = if selector == "default" {
        instance.animation_at(0)
    } else {
        instance.animation_named(selector)
    };
    selected
        .map(Some)
        .with_context(|| format!("missing animation {selector}"))
}
fn named_model_instance(
    file: &RuntimeFileHandle,
    name: &str,
    default: bool,
) -> anyhow::Result<CoreHandle> {
    let model = file
        .with_file(|file| file.view_model_named(name))
        .with_context(|| format!("missing view model {name}"))?;
    file.with_file_mut(|file| {
        if default {
            file.create_default_view_model_instance(model)
        } else {
            file.create_view_model_instance(model)
        }
    })
    .with_context(|| format!("cannot create view model {name}"))
}
fn complete_context(
    file: &RuntimeFileHandle,
    main: Option<CoreHandle>,
) -> RuntimeDataContextHandle {
    let context = RuntimeDataContextHandle::new(DataContext::new(main));
    let globals = file.with_file(File::global_view_models);
    for model in globals {
        let name = model
            .with(|model| model.as_view_model().unwrap().base.name().to_owned())
            .expect("retained ViewModel");
        let slot = file.with_file(|file| file.view_model_id(&name));
        if let Some(instance) =
            file.with_file_mut(|file| file.create_default_view_model_instance(model))
        {
            context.with_context_mut(|context| {
                context.set_view_model_instance_for_slot(slot, Some(instance))
            });
        }
    }
    context
}
fn bind_context(
    instance: &RuntimeArtboardInstanceHandle,
    machine: Option<&RuntimeStateMachineInstanceHandle>,
    context: &RuntimeDataContextHandle,
) {
    if let Some(machine) = machine {
        machine.with_instance_mut(|machine| {
            machine.bind_data_context(context.clone());
            machine.advanced_data_context();
        });
    } else {
        instance.clear_data_context();
        context.with_context_mut(|context| {
            context.add_dependent_container(instance.core_handle());
        });
        instance.internal_data_context(context.clone());
    }
}
fn main_runtime(
    context: &Option<RuntimeDataContextHandle>,
) -> anyhow::Result<ViewModelInstanceRuntime> {
    let instance = context
        .as_ref()
        .context("no prepared view-model instance")?
        .with_context(DataContext::main_view_model_instance)
        .context("prepared context has no main instance")?;
    Ok(ViewModelInstanceRuntime::new(instance))
}
fn set_artboard_id(
    instance: &ViewModelInstanceRuntime,
    property: &str,
    value: u32,
) -> anyhow::Result<()> {
    let target = instance
        .property_artboard(property)
        .with_context(|| format!("missing artboard property {property}"))?
        .value_runtime()
        .handle();
    if !CoreRegistry::set_uint_handle(
        &target,
        i32::from(ViewModelInstanceArtboardBase::PROPERTY_VALUE_PROPERTY_KEY),
        value,
    ) {
        bail!("artboard property setter missing");
    }
    Ok(())
}
fn set_asset_id(
    instance: &ViewModelInstanceRuntime,
    property: &str,
    value: u32,
) -> anyhow::Result<()> {
    let target = instance
        .property_image(property)
        .map(|value| value.value_runtime().handle())
        .or_else(|| {
            instance
                .property_font(property)
                .map(|value| value.value_runtime().handle())
        })
        .or_else(|| {
            instance
                .property_blob(property)
                .map(|value| value.value_runtime().handle())
        })
        .with_context(|| format!("missing asset property {property}"))?;
    if !CoreRegistry::set_uint_handle(
        &target,
        i32::from(ViewModelInstanceAssetBase::PROPERTY_VALUE_PROPERTY_KEY),
        value,
    ) {
        bail!("asset property setter missing");
    }
    Ok(())
}
fn frame_dimension(value: f32) -> u32 {
    // Pinned silver tests pass the float directly to frameSize(uint32_t,
    // uint32_t), truncating fractional dimensions without a minimum size.
    // The separate C++ golden runner explicitly rounds up; silvers do not.
    value as u32
}
fn pointer_position(
    x: &PointerCoordinate,
    y: &PointerCoordinate,
    instance: &RuntimeArtboardInstanceHandle,
) -> anyhow::Result<Vec2D> {
    let (width, height) = instance.with_artboard(|artboard| (artboard.width(), artboard.height()));
    Ok(Vec2D::new(
        x.resolve(width, height)?,
        y.resolve(width, height)?,
    ))
}

fn encode_gamepad_batch(records: &[GamepadRecord]) -> Vec<u8> {
    let mut bytes = GAMEPAD_BATCH_WIRE_VERSION.to_le_bytes().to_vec();
    for record in records {
        match *record {
            GamepadRecord::Connected {
                device_id,
                button_count,
                axis_count,
                mapping,
            } => {
                bytes.push(0);
                bytes.extend_from_slice(&device_id.to_le_bytes());
                bytes.extend_from_slice(&[
                    u8::from(mapping == GamepadMapping::Unknown),
                    button_count,
                    axis_count,
                    0,
                ]);
                bytes.resize(bytes.len() + usize::from(button_count) * 4, 0);
                bytes.resize(bytes.len() + usize::from(axis_count) * 4, 0);
            }
            GamepadRecord::Update {
                device_id,
                input,
                index,
                value,
            } => {
                bytes.push(1);
                bytes.extend_from_slice(&device_id.to_le_bytes());
                bytes.extend_from_slice(&[1, u8::from(input == GamepadInputKind::Axis), index]);
                bytes.extend_from_slice(&value.to_le_bytes());
            }
            GamepadRecord::Disconnected { device_id } => {
                bytes.push(2);
                bytes.extend_from_slice(&device_id.to_le_bytes());
            }
        }
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::{Action, ActionTarget, Execution, PointerCoordinate};
    use crate::{Actions, Case, Lane, Status, read_manifest};
    use std::path::{Path, PathBuf};

    #[test]
    fn silver_frame_dimensions_use_the_upstream_integer_conversion() {
        assert_eq!(super::frame_dimension(1905.75), 1905);
        assert_eq!(super::frame_dimension(983.75), 983);
        assert_eq!(super::frame_dimension(0.75), 0);
        assert_eq!(super::frame_dimension(0.0), 0);
        assert_eq!(super::frame_dimension(500.0), 500);
    }

    #[test]
    fn deserializes_testing_random_actions() {
        assert_eq!(
            toml::from_str::<Action>(r#"kind = "clear-randoms""#).unwrap(),
            Action::ClearRandoms
        );
        assert_eq!(
            toml::from_str::<Action>(
                r#"kind = "add-random-value"
value = 1.0"#
            )
            .unwrap(),
            Action::AddRandomValue { value: 1.0 }
        );
        assert_eq!(
            toml::from_str::<Action>(
                r#"kind = "assert-random-calls"
count = 64"#
            )
            .unwrap(),
            Action::AssertRandomCalls { count: 64 }
        );
    }

    #[test]
    fn deserializes_gamepad_batch_records() {
        let action: Action = toml::from_str(
            r#"kind = "gamepad-batch"
records = [
  { kind = "connected", device_id = 3 },
  { kind = "update", device_id = 3, input = "axis", index = 1, value = -0.5 },
  { kind = "disconnected", device_id = 3 },
]
"#,
        )
        .unwrap();

        assert!(matches!(
            action,
            Action::GamepadBatch { records } if records.len() == 3
        ));
    }

    #[test]
    fn deserializes_focus_and_keyboard_actions() {
        let focus: Action = toml::from_str(r#"kind = "focus-next""#).unwrap();
        let key: Action = toml::from_str(
            r#"kind = "key-input"
key = 65
modifiers = 1
pressed = true
repeat = false
"#,
        )
        .unwrap();

        assert_eq!(focus, Action::FocusNext);
        assert_eq!(
            key,
            Action::KeyInput {
                key: 65,
                modifiers: 1,
                pressed: true,
                repeat: false,
            }
        );
    }

    #[test]
    fn resolves_pointer_coordinate_expressions_against_artboard_size() {
        assert_eq!(
            PointerCoordinate::Expression("artboard-width/2".to_owned())
                .resolve(640.0, 480.0)
                .unwrap(),
            320.0
        );
        assert_eq!(
            PointerCoordinate::Expression("artboard-width/2+375".to_owned())
                .resolve(640.0, 480.0)
                .unwrap(),
            695.0
        );
        assert_eq!(
            PointerCoordinate::Expression("artboard-height-20".to_owned())
                .resolve(640.0, 480.0)
                .unwrap(),
            460.0
        );
        assert_eq!(
            PointerCoordinate::Expression("artboard-height/2-375".to_owned())
                .resolve(640.0, 480.0)
                .unwrap(),
            -135.0
        );
    }

    #[test]
    fn deserializes_boolean_view_model_mutation() {
        let action: Action = toml::from_str(
            r#"kind = "set-view-model-boolean"
property = "enabled"
value = true
"#,
        )
        .unwrap();

        assert_eq!(
            action,
            Action::SetViewModelBoolean {
                property: "enabled".to_owned(),
                value: true,
            }
        );
    }

    #[test]
    fn deserializes_string_view_model_mutation() {
        let action: Action = toml::from_str(
            r#"kind = "set-view-model-string"
property = "label"
value = "ready"
"#,
        )
        .unwrap();

        assert_eq!(
            action,
            Action::SetViewModelString {
                property: "label".to_owned(),
                value: "ready".to_owned(),
            }
        );
    }

    #[test]
    fn deserializes_enum_view_model_mutation() {
        let action: Action = toml::from_str(
            r#"kind = "set-view-model-enum"
property = "display"
value = 2
"#,
        )
        .unwrap();

        assert_eq!(
            action,
            Action::SetViewModelEnum {
                property: "display".to_owned(),
                value: 2,
            }
        );
    }

    #[test]
    fn deserializes_view_model_trigger_mutation() {
        let action: Action = toml::from_str(
            r#"kind = "fire-view-model-trigger"
property = "pressed"
"#,
        )
        .unwrap();

        assert_eq!(
            action,
            Action::FireViewModelTrigger {
                property: "pressed".to_owned(),
            }
        );
    }

    #[test]
    fn deserializes_artboard_view_model_mutation() {
        let action: Action = toml::from_str(
            r#"kind = "set-view-model-artboard"
property = "nested"
value = 3
"#,
        )
        .unwrap();

        assert_eq!(
            action,
            Action::SetViewModelArtboard {
                property: "nested".to_owned(),
                value: 3,
            }
        );
    }

    #[test]
    fn deserializes_asset_view_model_mutation() {
        let action: Action = toml::from_str(
            r#"kind = "set-view-model-asset"
property = "image"
value = -1
"#,
        )
        .unwrap();

        assert_eq!(
            action,
            Action::SetViewModelAsset {
                property: "image".to_owned(),
                value: -1,
            }
        );
    }

    #[test]
    fn deserializes_font_bytes_view_model_mutation() {
        let action: Action = toml::from_str(
            r#"kind = "set-view-model-font-bytes"
property = "font"
source = "custom.ttf"
"#,
        )
        .unwrap();

        assert_eq!(
            action,
            Action::SetViewModelFontBytes {
                property: "font".to_owned(),
                source: "custom.ttf".to_owned(),
            }
        );
    }

    #[test]
    fn deserializes_bounded_scroll_physics_settlement() {
        let action: Action = toml::from_str(
            r#"kind = "advance-draw-until-scroll-physics-stops"
max_frames = 56
seconds = 0.016
"#,
        )
        .unwrap();

        assert_eq!(
            action,
            Action::AdvanceDrawUntilScrollPhysicsStops {
                max_frames: 56,
                seconds: 0.016,
            }
        );
    }

    #[test]
    fn deserializes_global_defaults_and_fixed_frame_loop() {
        let globals: Action =
            toml::from_str(r#"kind = "set-state-machine-default-global-view-models""#).unwrap();
        let frames: Action = toml::from_str(
            r#"kind = "advance-draw-frames"
frames = 62
seconds = 0.016
"#,
        )
        .unwrap();

        assert_eq!(globals, Action::SetStateMachineDefaultGlobalViewModels);
        assert_eq!(
            frames,
            Action::AdvanceDrawFrames {
                frames: 62,
                seconds: 0.016,
            }
        );
    }

    fn pinned_runtime_dir(test: &str) -> Option<PathBuf> {
        let root = std::env::var_os("RIVE_RUNTIME_DIR").map(PathBuf::from);
        if root.is_none() {
            eprintln!(
                "skipping {test}; RIVE_RUNTIME_DIR is unset; point it at a pinned rive-runtime checkout"
            );
        }
        root
    }

    #[test]
    fn executes_each_new_view_model_mutation_kind_against_pinned_fixtures() {
        let Some(runtime_dir) = pinned_runtime_dir(
            "executes_each_new_view_model_mutation_kind_against_pinned_fixtures",
        ) else {
            return;
        };
        let runtime_dir = runtime_dir.as_path();
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("silver-corpus crate is nested under the workspace")
            .to_owned();
        let manifest = read_manifest(&workspace.join("silver-corpus.toml")).unwrap();

        for id in [
            "component_based_conditions-Artboard2",
            "zero_width_space_line_break",
            "collapse_data_binds-test_3",
            "fit_font_size_test",
            "viewmodel_image_reset",
            "data_bind_font_test",
            "focusable_element",
            "keyboard_listener-KeyboardInput",
            "relative_data_bind_path",
            "component_list_grouped",
            "data_binding_artboards_test_recursive",
            "global_viewmodels_test-set_instance",
            "image_fit_alignment",
            "interactive_scrolling",
            "ik_anim_test",
        ] {
            let case = manifest
                .cases
                .iter()
                .find(|case| case.id == id)
                .unwrap_or_else(|| panic!("missing pinned case {id}"));
            Execution::run(case, runtime_dir)
                .unwrap_or_else(|error| panic!("{id} action execution failed: {error:#}"));
        }

        let artboard_case = Case {
            id: "set-view-model-artboard-unit".to_owned(),
            expected: String::new(),
            source: "data_bind_artboard_input.riv".to_owned(),
            dependencies: Vec::new(),
            artboard: "default".to_owned(),
            clone_artboard_instance: false,
            animation: "none".to_owned(),
            state_machine: "none".to_owned(),
            lane: Lane::Runtime,
            deterministic: "enabled".to_owned(),
            random: "deterministic".to_owned(),
            view_model: "cpp-test-defined".to_owned(),
            sample_times: Vec::new(),
            actions: Actions::Executable(vec![
                Action::BindFreshViewModel,
                Action::SetViewModelArtboard {
                    property: "artboardProperty".to_owned(),
                    value: 1,
                },
                Action::Advance {
                    target: ActionTarget::Artboard,
                    seconds: 0.0,
                },
            ]),
            verification: "sriv-v1-epsilon".to_owned(),
            status: Status::Diverges,
            producer_class: "unit-test".to_owned(),
            provenance_file: String::new(),
            provenance_test: String::new(),
            producer_line: 0,
            note: String::new(),
        };
        Execution::run(&artboard_case, runtime_dir)
            .expect("raw artboard-value mutation action should execute");
    }

    #[test]
    fn executes_six_dynamic_scroll_bodies_against_pinned_fixtures() {
        let Some(runtime_dir) =
            pinned_runtime_dir("executes_six_dynamic_scroll_bodies_against_pinned_fixtures")
        else {
            return;
        };
        let runtime_dir = runtime_dir.as_path();
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("silver-corpus crate is nested under the workspace")
            .to_owned();
        let manifest = read_manifest(&workspace.join("silver-corpus.toml")).unwrap();

        for id in [
            "layout_scroll_snap_padding_layouts",
            "layout_scroll_snap_padding_list",
            "layout_scroll_snap_padding_virtualized",
            "layout_scroll_drag_multiplier_layouts",
            "layout_scroll_drag_multiplier_list",
            "layout_scroll_drag_multiplier_virtualized",
        ] {
            let case = manifest
                .cases
                .iter()
                .find(|case| case.id == id)
                .unwrap_or_else(|| panic!("missing dynamic scroll case {id}"));
            Execution::run(case, runtime_dir)
                .unwrap_or_else(|error| panic!("{id} action execution failed: {error:#}"));
        }
    }
}
