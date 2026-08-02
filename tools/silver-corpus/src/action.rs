use crate::{ActionTarget::*, Case};
use anyhow::{Context, bail};
use nuxie_binary::{RuntimeFile, read_runtime_file};
use nuxie_graph::{ArtboardGraph, GraphFile};
use nuxie_render_api::SerializingFactory;
use nuxie_runtime::{
    ArtboardInstance, LinearAnimationInstance, RuntimeOwnedViewModelContext,
    RuntimeOwnedViewModelInstance, StateMachineInstance, set_runtime_deterministic_mode,
};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ActionTarget {
    Artboard,
    StateMachine,
    Animation,
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
    BindNamedDefaultViewModel {
        view_model: String,
    },
    CreateDefaultViewModel,
    BindPreparedViewModel,
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
    Advance {
        target: ActionTarget,
        seconds: f32,
    },
    Draw,
    Frame,
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
    SetArtboardSize {
        width: f32,
        height: f32,
    },
    AdvanceDrawUntilScrollPhysicsStops {
        max_frames: usize,
        seconds: f32,
    },
}

pub struct Execution {
    bytes: Vec<u8>,
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
        set_runtime_deterministic_mode(case.deterministic == "enabled");
        let fixture = runtime_dir
            .join("tests/unit_tests/assets")
            .join(&case.source);
        let fixture_bytes = std::fs::read(&fixture)
            .with_context(|| format!("failed to read fixture {}", fixture.display()))?;
        let runtime = read_runtime_file(&fixture_bytes).context("failed to import runtime file")?;
        let graph = GraphFile::from_runtime_file(&runtime).context("failed to build graph")?;
        let (artboard_index, artboard) = select_artboard(&graph, &case.artboard)?;
        let mut instance =
            ArtboardInstance::from_graph_with_artboards(&runtime, artboard, &graph.artboards)
                .context("failed to instantiate artboard")?;
        let mut factory = SerializingFactory::new();
        let external_images = BTreeMap::<u32, Arc<[u8]>>::new();
        instance
            .initialize_artboard_renderer(
                &runtime,
                artboard,
                &graph.artboards,
                &external_images,
                &mut factory,
                None,
            )
            .context("failed to initialize artboard renderer")?;
        let artboard_object = runtime
            .artboard(artboard_index)
            .context("missing selected artboard object")?;
        factory.frame_size(
            frame_dimension(artboard_object.double_property("width").unwrap_or(0.0)),
            frame_dimension(artboard_object.double_property("height").unwrap_or(0.0)),
        );
        let mut renderer = factory.make_renderer();
        let mut state_machine = select_state_machine(&mut instance, artboard, &case.state_machine)?;
        let mut animation = select_animation(&instance, artboard, &case.animation)?;
        let mut owned_context = None;
        for action in actions {
            match action {
                Action::BindDefaultViewModel => {
                    if let Some(context) =
                        selected_artboard_owned_view_model_context(&runtime, artboard_index)
                    {
                        instance.bind_owned_view_model_artboard_contexts(&runtime, &context);
                        if let Some(machine) = state_machine.as_mut() {
                            machine.bind_owned_view_model_contexts(&context);
                            machine.advance_data_context();
                        }
                        owned_context = Some(context);
                    } else {
                        instance.bind_default_view_model_artboard_list_context(&runtime);
                        if let Some(machine) = state_machine.as_mut() {
                            machine.bind_default_view_model_context();
                            machine.advance_data_context();
                        }
                    }
                }
                Action::BindFreshViewModel => {
                    let context =
                        selected_artboard_fresh_view_model_context(&runtime, artboard_index)
                            .context("selected artboard has no view-model schema")?;
                    instance.bind_owned_view_model_artboard_contexts(&runtime, &context);
                    if let Some(machine) = state_machine.as_mut() {
                        machine.bind_owned_view_model_contexts(&context);
                        machine.advance_data_context();
                    }
                    owned_context = Some(context);
                }
                Action::BindNamedDefaultViewModel { view_model } => {
                    let context =
                        named_default_view_model_context(&runtime, artboard_index, view_model)
                            .with_context(|| {
                                format!("missing default view-model instance {view_model}")
                            })?;
                    instance.bind_owned_view_model_artboard_contexts(&runtime, &context);
                    if let Some(machine) = state_machine.as_mut() {
                        machine.bind_owned_view_model_contexts(&context);
                        machine.advance_data_context();
                    }
                    owned_context = Some(context);
                }
                Action::CreateDefaultViewModel => {
                    owned_context =
                        selected_artboard_owned_view_model_context(&runtime, artboard_index);
                    if owned_context.is_none() {
                        bail!("selected artboard has no default view model");
                    }
                }
                Action::BindPreparedViewModel => {
                    let context = owned_context
                        .as_ref()
                        .context("no prepared view-model instance")?;
                    instance.bind_owned_view_model_artboard_contexts(&runtime, context);
                    if let Some(machine) = state_machine.as_mut() {
                        machine.bind_owned_view_model_contexts(context);
                        machine.advance_data_context();
                    }
                }
                Action::SetViewModelNumber { property, value } => {
                    let context = owned_context
                        .as_ref()
                        .context("no prepared view-model instance")?;
                    let mut main = context
                        .main_mut()
                        .context("prepared context has no main instance")?;
                    if main
                        .number_source_handle_by_property_name_path(property)
                        .is_none()
                    {
                        bail!("missing numeric view-model property {property}");
                    }
                    main.set_number_by_property_name_path(property, *value);
                }
                Action::SetViewModelBoolean { property, value } => {
                    let context = owned_context
                        .as_ref()
                        .context("no prepared view-model instance")?;
                    let mut main = context
                        .main_mut()
                        .context("prepared context has no main instance")?;
                    if main
                        .boolean_source_handle_by_property_name_path(property)
                        .is_none()
                    {
                        bail!("missing boolean view-model property {property}");
                    }
                    main.set_boolean_by_property_name_path(property, *value);
                }
                Action::SetViewModelString { property, value } => {
                    let context = owned_context
                        .as_ref()
                        .context("no prepared view-model instance")?;
                    let mut main = context
                        .main_mut()
                        .context("prepared context has no main instance")?;
                    if main
                        .string_source_handle_by_property_name_path(property)
                        .is_none()
                    {
                        bail!("missing string view-model property {property}");
                    }
                    main.set_string_by_property_name_path(property, value.as_bytes());
                }
                Action::SetViewModelEnum { property, value } => {
                    let context = owned_context
                        .as_ref()
                        .context("no prepared view-model instance")?;
                    let mut main = context
                        .main_mut()
                        .context("prepared context has no main instance")?;
                    if main
                        .enum_source_handle_by_property_name_path(property)
                        .is_none()
                    {
                        bail!("missing enum view-model property {property}");
                    }
                    main.set_enum_by_property_name_path(property, *value);
                }
                Action::SetViewModelColor { property, value } => {
                    let context = owned_context
                        .as_ref()
                        .context("no prepared view-model instance")?;
                    let mut main = context
                        .main_mut()
                        .context("prepared context has no main instance")?;
                    if main
                        .color_source_handle_by_property_name_path(property)
                        .is_none()
                    {
                        bail!("missing color view-model property {property}");
                    }
                    main.set_color_by_property_name_path(property, *value);
                }
                Action::FireViewModelTrigger { property } => {
                    let context = owned_context
                        .as_ref()
                        .context("no prepared view-model instance")?;
                    let mut main = context
                        .main_mut()
                        .context("prepared context has no main instance")?;
                    let next = main
                        .trigger_value_by_property_name_path(property)
                        .with_context(|| format!("missing trigger view-model property {property}"))?
                        .wrapping_add(1);
                    main.set_trigger_by_property_name_path(property, next);
                }
                Action::SetViewModelArtboard { property, value } => {
                    let context = owned_context
                        .as_ref()
                        .context("no prepared view-model instance")?;
                    let mut main = context
                        .main_mut()
                        .context("prepared context has no main instance")?;
                    if main
                        .artboard_source_handle_by_property_name_path(property)
                        .is_none()
                    {
                        bail!("missing artboard view-model property {property}");
                    }
                    main.set_artboard_by_property_name_path(property, *value);
                }
                Action::SetViewModelArtboardByName { property, artboard } => {
                    let context = owned_context
                        .as_ref()
                        .context("no prepared view-model instance")?;
                    let mut main = context
                        .main_mut()
                        .context("prepared context has no main instance")?;
                    let value = graph
                        .artboards
                        .iter()
                        .position(|candidate| candidate.name.as_deref() == Some(artboard))
                        .with_context(|| format!("missing artboard {artboard}"))?;
                    if !main.set_artboard_by_property_name_path(property, value as u64) {
                        bail!("missing artboard view-model property {property}");
                    }
                }
                Action::SetViewModelAsset { property, value } => {
                    let context = owned_context
                        .as_ref()
                        .context("no prepared view-model instance")?;
                    let mut main = context
                        .main_mut()
                        .context("prepared context has no main instance")?;
                    if main
                        .asset_source_handle_by_property_name_path(property)
                        .is_none()
                    {
                        bail!("missing asset view-model property {property}");
                    }
                    main.set_asset_by_property_name_path(property, *value as u64);
                }
                Action::SetViewModelAssetByName { property, asset } => {
                    let context = owned_context
                        .as_ref()
                        .context("no prepared view-model instance")?;
                    let mut main = context
                        .main_mut()
                        .context("prepared context has no main instance")?;
                    let value = runtime
                        .file_assets()
                        .iter()
                        .position(|candidate| candidate.string_property("name") == Some(asset))
                        .with_context(|| format!("missing file asset {asset}"))?;
                    if !main.set_asset_by_property_name_path(property, value as u64) {
                        bail!("missing asset view-model property {property}");
                    }
                }
                Action::SetViewModelFontBytes { property, source } => {
                    let context = owned_context
                        .as_ref()
                        .context("no prepared view-model instance")?;
                    let mut main = context
                        .main_mut()
                        .context("prepared context has no main instance")?;
                    if main
                        .font_asset_source_handle_by_property_name_path(property)
                        .is_none()
                    {
                        bail!("missing font view-model property {property}");
                    }
                    let source_path = runtime_dir.join("tests/unit_tests/assets").join(source);
                    let font_bytes = std::fs::read(&source_path).with_context(|| {
                        format!("failed to read font fixture {}", source_path.display())
                    })?;
                    main.set_live_font_bytes_by_property_name_path(
                        property,
                        Some(Arc::from(font_bytes)),
                    );
                }
                Action::SetGlobalViewModelColor {
                    global,
                    property,
                    value,
                } => {
                    let context = owned_context
                        .as_ref()
                        .context("no prepared view-model instance")?;
                    let mut global_instance = context
                        .global_named_mut(&runtime, global)
                        .with_context(|| format!("missing global view model {global}"))?;
                    if !global_instance.set_color_by_property_name_path(property, *value) {
                        bail!("missing color property {global}.{property}");
                    }
                }
                Action::FireViewModelListItemTrigger {
                    list,
                    index,
                    trigger,
                } => {
                    let context = owned_context
                        .as_ref()
                        .context("no prepared view-model instance")?;
                    let main = context
                        .main_handle()
                        .context("prepared context has no main instance")?;
                    let item = main
                        .list_items_by_property_name_path(list)
                        .and_then(|items| items.get(*index).cloned())
                        .with_context(|| format!("missing list item {list}[{index}]"))?;
                    let next = item
                        .borrow()
                        .trigger_value_by_property_name_path(trigger)
                        .unwrap_or(0)
                        .wrapping_add(1);
                    let mut item = item.borrow_mut();
                    if item
                        .trigger_source_handle_by_property_name_path(trigger)
                        .is_none()
                    {
                        bail!("missing trigger property {list}[{index}].{trigger}");
                    }
                    item.set_trigger_by_property_name_path(trigger, next);
                }
                Action::AppendViewModelListItem {
                    list,
                    view_model,
                    index,
                    number_properties,
                    string_property,
                    string_value,
                } => {
                    let context = owned_context
                        .as_ref()
                        .context("no prepared view-model instance")?;
                    let main = context
                        .main_handle()
                        .context("prepared context has no main instance")?;
                    let view_model_index = runtime
                        .view_models()
                        .iter()
                        .position(|candidate| {
                            candidate.object.string_property("name") == Some(view_model)
                        })
                        .with_context(|| format!("missing view model {view_model}"))?;
                    let mut child = RuntimeOwnedViewModelInstance::new(&runtime, view_model_index)
                        .with_context(|| format!("cannot create view model {view_model}"))?;
                    for (property, value) in number_properties {
                        if child
                            .number_source_handle_by_property_name_path(property)
                            .is_none()
                        {
                            bail!("missing numeric property {view_model}.{property}");
                        }
                        child.set_number_by_property_name_path(property, *value);
                    }
                    if let (Some(property), Some(value)) = (string_property, string_value) {
                        if child
                            .string_source_handle_by_property_name_path(property)
                            .is_none()
                        {
                            bail!("missing string property {view_model}.{property}");
                        }
                        child.set_string_by_property_name_path(property, value.as_bytes());
                    }
                    let child = nuxie_runtime::RuntimeOwnedViewModelHandle::new(child);
                    let insertion_index = match index {
                        Some(index) => *index,
                        None => main
                            .list_item_count_by_property_name_path(list)
                            .with_context(|| format!("missing list property {list}"))?,
                    };
                    if !main.insert_list_item_by_property_name_path(list, insertion_index, &child) {
                        bail!("failed to append {view_model} to {list}");
                    }
                }
                Action::RemoveViewModelListItem { list, index } => {
                    let context = owned_context
                        .as_ref()
                        .context("no prepared view-model instance")?;
                    let main = context
                        .main_handle()
                        .context("prepared context has no main instance")?;
                    if !main.remove_list_item_by_property_name_path(list, *index) {
                        bail!("failed to remove {list}[{index}]");
                    }
                }
                Action::SetViewModelListItemNumber {
                    list,
                    index,
                    property,
                    value,
                } => {
                    let context = owned_context
                        .as_ref()
                        .context("no prepared view-model instance")?;
                    let main = context
                        .main_handle()
                        .context("prepared context has no main instance")?;
                    let item = main
                        .list_items_by_property_name_path(list)
                        .and_then(|items| items.get(*index).cloned())
                        .with_context(|| format!("missing list item {list}[{index}]"))?;
                    let mut item = item.borrow_mut();
                    if item
                        .number_source_handle_by_property_name_path(property)
                        .is_none()
                    {
                        bail!("missing numeric property {list}[{index}].{property}");
                    }
                    item.set_number_by_property_name_path(property, *value);
                }
                Action::Advance { target, seconds } => match target {
                    StateMachine => {
                        advance_state_machine(
                            &mut instance,
                            state_machine
                                .as_mut()
                                .context("no selected state machine")?,
                            *seconds,
                            &mut factory,
                        )?;
                        instance
                            .synchronize_artboard_renderer(
                                &runtime,
                                artboard,
                                &graph.artboards,
                                &external_images,
                                &mut factory,
                                None,
                            )
                            .context("state-machine renderer synchronization failed")?;
                    }
                    Artboard => {
                        instance
                            .advance_frame_components(*seconds)
                            .map_err(|error| anyhow::anyhow!(error))
                            .context("artboard advance failed")?;
                        instance
                            .update_pass_with_script_errors()
                            .map_err(|error| anyhow::anyhow!(error))
                            .context("artboard update failed")?;
                    }
                    Animation => {
                        let animation = animation.as_mut().context("no selected animation")?;
                        instance.advance_linear_animation_instance(animation, *seconds);
                        instance.apply_linear_animation_instance(animation, 1.0);
                        instance
                            .update_pass_with_script_errors()
                            .map_err(|error| anyhow::anyhow!(error))
                            .context("animation update failed")?;
                    }
                },
                Action::Draw => {
                    instance
                        .draw_artboard(
                            &runtime,
                            artboard,
                            &graph.artboards,
                            &mut factory,
                            &mut renderer,
                            &external_images,
                            None,
                            true,
                        )
                        .context("draw failed")?;
                }
                Action::Frame => factory.add_frame(),
                Action::PointerDown { x, y, pointer_id } => {
                    let (x, y) = pointer_position(x, y, &instance)?;
                    state_machine
                        .as_mut()
                        .context("no selected state machine")?
                        .pointer_down(&mut instance, x, y, *pointer_id);
                }
                Action::PointerMove {
                    x,
                    y,
                    seconds,
                    pointer_id,
                } => {
                    let (x, y) = pointer_position(x, y, &instance)?;
                    state_machine
                        .as_mut()
                        .context("no selected state machine")?
                        .pointer_move(&mut instance, x, y, *seconds, *pointer_id);
                }
                Action::PointerUp { x, y, pointer_id } => {
                    let (x, y) = pointer_position(x, y, &instance)?;
                    state_machine
                        .as_mut()
                        .context("no selected state machine")?
                        .pointer_up(&mut instance, x, y, *pointer_id);
                }
                Action::PointerExit { x, y, pointer_id } => {
                    let (x, y) = pointer_position(x, y, &instance)?;
                    state_machine
                        .as_mut()
                        .context("no selected state machine")?
                        .pointer_exit(&mut instance, x, y, *pointer_id);
                }
                Action::VerticalPointerDrag {
                    x,
                    start_y,
                    end_y_exclusive,
                    step,
                    advance_seconds,
                    pointer_id,
                } => {
                    let (width, height) = instance.artboard_dimensions();
                    let x = x.resolve(width, height)?;
                    let mut y = start_y.resolve(width, height)?;
                    factory.add_frame();
                    state_machine
                        .as_mut()
                        .context("no selected state machine")?
                        .pointer_down(&mut instance, x, y, *pointer_id);
                    advance_state_machine(
                        &mut instance,
                        state_machine
                            .as_mut()
                            .context("no selected state machine")?,
                        *advance_seconds,
                        &mut factory,
                    )?;
                    instance.synchronize_artboard_renderer(
                        &runtime,
                        artboard,
                        &graph.artboards,
                        &external_images,
                        &mut factory,
                        None,
                    )?;
                    instance.draw_artboard(
                        &runtime,
                        artboard,
                        &graph.artboards,
                        &mut factory,
                        &mut renderer,
                        &external_images,
                        None,
                        true,
                    )?;
                    while y > *end_y_exclusive {
                        factory.add_frame();
                        state_machine
                            .as_mut()
                            .context("no selected state machine")?
                            .pointer_move(&mut instance, x, y, 0.0, *pointer_id);
                        advance_state_machine(
                            &mut instance,
                            state_machine
                                .as_mut()
                                .context("no selected state machine")?,
                            *advance_seconds,
                            &mut factory,
                        )?;
                        instance.synchronize_artboard_renderer(
                            &runtime,
                            artboard,
                            &graph.artboards,
                            &external_images,
                            &mut factory,
                            None,
                        )?;
                        instance.draw_artboard(
                            &runtime,
                            artboard,
                            &graph.artboards,
                            &mut factory,
                            &mut renderer,
                            &external_images,
                            None,
                            true,
                        )?;
                        y -= *step;
                    }
                    factory.add_frame();
                    state_machine
                        .as_mut()
                        .context("no selected state machine")?
                        .pointer_up(&mut instance, x, y, *pointer_id);
                    advance_state_machine(
                        &mut instance,
                        state_machine
                            .as_mut()
                            .context("no selected state machine")?,
                        *advance_seconds,
                        &mut factory,
                    )?;
                    instance.synchronize_artboard_renderer(
                        &runtime,
                        artboard,
                        &graph.artboards,
                        &external_images,
                        &mut factory,
                        None,
                    )?;
                    instance.draw_artboard(
                        &runtime,
                        artboard,
                        &graph.artboards,
                        &mut factory,
                        &mut renderer,
                        &external_images,
                        None,
                        true,
                    )?;
                }
                Action::SetBool { input, value } => {
                    let machine = state_machine
                        .as_mut()
                        .context("no selected state machine")?;
                    let index = machine
                        .input_index_named(input)
                        .with_context(|| format!("missing boolean input {input}"))?;
                    if !machine.set_bool(index, *value) {
                        bail!("input {input} is not boolean");
                    }
                }
                Action::SetNumber { input, value } => {
                    let machine = state_machine
                        .as_mut()
                        .context("no selected state machine")?;
                    let index = machine
                        .input_index_named(input)
                        .with_context(|| format!("missing number input {input}"))?;
                    if !machine.set_number(index, *value) {
                        bail!("input {input} is not numeric");
                    }
                }
                Action::FireTrigger { input } => {
                    let machine = state_machine
                        .as_mut()
                        .context("no selected state machine")?;
                    let index = machine
                        .input_index_named(input)
                        .with_context(|| format!("missing trigger input {input}"))?;
                    if !machine.fire_trigger(index) {
                        bail!("input {input} is not a trigger");
                    }
                }
                Action::TextInput { text } => {
                    if !state_machine
                        .as_mut()
                        .context("no selected state machine")?
                        .text_input(&mut instance, text)
                    {
                        bail!("text input was not handled");
                    }
                }
                Action::FocusNext => {
                    state_machine
                        .as_mut()
                        .context("no selected state machine")?
                        .focus_next();
                }
                Action::FocusPrevious => {
                    state_machine
                        .as_mut()
                        .context("no selected state machine")?
                        .focus_previous();
                }
                Action::KeyInput {
                    key,
                    modifiers,
                    pressed,
                    repeat,
                } => {
                    state_machine
                        .as_mut()
                        .context("no selected state machine")?
                        .key_input(&mut instance, *key, *modifiers, *pressed, *repeat);
                }
                Action::SetArtboardSize { width, height } => {
                    instance.set_artboard_dimensions(*width, *height);
                }
                Action::AdvanceDrawUntilScrollPhysicsStops {
                    max_frames,
                    seconds,
                } => {
                    // Pinned layout_scroll_test.cpp:412-416 and 447-455
                    // (drag helper: 499-503 and 535-543) resolve
                    // find<ScrollConstraint>()[0] once, then capture each frame
                    // before testing physics()->isRunning(). Retain the
                    // constrained component identity so this remains scoped to
                    // the same concrete occurrence throughout settlement.
                    let scroll = instance
                        .scroll_constraint_occurrences()
                        .into_iter()
                        .next()
                        .context("selected artboard has no ScrollConstraint occurrence")?;
                    if !scroll.physics_present {
                        bail!("selected ScrollConstraint occurrence has no physics");
                    }
                    for _ in 0..*max_frames {
                        factory.add_frame();
                        advance_state_machine(
                            &mut instance,
                            state_machine
                                .as_mut()
                                .context("no selected state machine")?,
                            *seconds,
                            &mut factory,
                        )?;
                        instance.synchronize_artboard_renderer(
                            &runtime,
                            artboard,
                            &graph.artboards,
                            &external_images,
                            &mut factory,
                            None,
                        )?;
                        instance.draw_artboard(
                            &runtime,
                            artboard,
                            &graph.artboards,
                            &mut factory,
                            &mut renderer,
                            &external_images,
                            None,
                            true,
                        )?;
                        let current = instance
                            .scroll_constraint_for_content(scroll.content_local_id)
                            .context("selected ScrollConstraint occurrence disappeared")?;
                        if !current.physics_present {
                            bail!("selected ScrollConstraint occurrence lost its physics");
                        }
                        if !current.physics_running {
                            break;
                        }
                    }
                }
            }
        }
        drop(renderer);
        let bytes = factory.bytes().to_vec();
        Ok(Self { bytes })
    }
}

fn select_artboard<'a>(
    graph: &'a GraphFile,
    selector: &str,
) -> anyhow::Result<(usize, &'a ArtboardGraph)> {
    if selector == "default" {
        return graph
            .artboards
            .first()
            .map(|artboard| (0, artboard))
            .context("missing default artboard");
    }
    graph
        .artboards
        .iter()
        .enumerate()
        .find(|(_, artboard)| artboard.name.as_deref() == Some(selector))
        .with_context(|| format!("missing artboard {selector}"))
}

fn select_state_machine(
    instance: &mut ArtboardInstance,
    artboard: &ArtboardGraph,
    selector: &str,
) -> anyhow::Result<Option<StateMachineInstance>> {
    if selector == "none" {
        return Ok(None);
    }
    let index = if selector == "default" {
        0
    } else {
        artboard
            .state_machines
            .iter()
            .position(|machine| machine.name.as_deref() == Some(selector))
            .with_context(|| format!("missing state machine {selector}"))?
    };
    instance
        .state_machine_instance(index)
        .map(Some)
        .with_context(|| format!("failed to instantiate state machine {selector}"))
}

fn select_animation(
    instance: &ArtboardInstance,
    artboard: &ArtboardGraph,
    selector: &str,
) -> anyhow::Result<Option<LinearAnimationInstance>> {
    if selector == "none" {
        return Ok(None);
    }
    let index = if selector == "default" {
        0
    } else {
        artboard
            .animations
            .iter()
            .position(|animation| animation.name.as_deref() == Some(selector))
            .with_context(|| format!("missing animation {selector}"))?
    };
    instance
        .linear_animation_instance(index)
        .map(Some)
        .with_context(|| format!("failed to instantiate animation {selector}"))
}

fn advance_state_machine(
    instance: &mut ArtboardInstance,
    state_machine: &mut StateMachineInstance,
    seconds: f32,
    factory: &mut SerializingFactory,
) -> anyhow::Result<()> {
    StateMachineInstance::advance_and_apply_state_machines_with_factory_and_view_models(
        instance,
        std::slice::from_mut(state_machine),
        seconds,
        factory,
        true,
        || false,
    )
    .map_err(|error| anyhow::anyhow!(error))
    .context("state-machine update failed")?;
    Ok(())
}

fn selected_artboard_owned_view_model_context(
    runtime: &RuntimeFile,
    artboard_index: usize,
) -> Option<RuntimeOwnedViewModelContext> {
    let view_model_index = runtime
        .artboard(artboard_index)?
        .uint_property("viewModelId")
        .and_then(|index| usize::try_from(index).ok())?;
    let main = RuntimeOwnedViewModelInstance::from_instance(runtime, view_model_index, 0)
        .or_else(|| RuntimeOwnedViewModelInstance::new(runtime, view_model_index))?;
    let mut context = RuntimeOwnedViewModelContext::from_main(main);
    context.complete_for_artboard(runtime, artboard_index);
    Some(context)
}

fn selected_artboard_fresh_view_model_context(
    runtime: &RuntimeFile,
    artboard_index: usize,
) -> Option<RuntimeOwnedViewModelContext> {
    let view_model_index = runtime
        .artboard(artboard_index)?
        .uint_property("viewModelId")
        .and_then(|index| usize::try_from(index).ok())?;
    let main = RuntimeOwnedViewModelInstance::new(runtime, view_model_index)?;
    let mut context = RuntimeOwnedViewModelContext::from_main(main);
    context.complete_for_artboard(runtime, artboard_index);
    Some(context)
}

fn named_default_view_model_context(
    runtime: &RuntimeFile,
    artboard_index: usize,
    view_model: &str,
) -> Option<RuntimeOwnedViewModelContext> {
    let view_model_index = runtime
        .view_models()
        .iter()
        .position(|candidate| candidate.object.string_property("name") == Some(view_model))?;
    let main = RuntimeOwnedViewModelInstance::from_instance(runtime, view_model_index, 0)
        .or_else(|| RuntimeOwnedViewModelInstance::new(runtime, view_model_index))?;
    let mut context = RuntimeOwnedViewModelContext::from_main(main);
    context.complete_for_artboard(runtime, artboard_index);
    Some(context)
}

fn frame_dimension(value: f32) -> u32 {
    value.ceil().max(1.0) as u32
}

fn pointer_position(
    x: &PointerCoordinate,
    y: &PointerCoordinate,
    instance: &ArtboardInstance,
) -> anyhow::Result<(f32, f32)> {
    let (width, height) = instance.artboard_dimensions();
    Ok((x.resolve(width, height)?, y.resolve(width, height)?))
}

#[cfg(test)]
mod tests {
    use super::{Action, ActionTarget, Execution, PointerCoordinate};
    use crate::{Actions, Case, Lane, Status, read_manifest};
    use std::path::{Path, PathBuf};

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
    fn executes_each_new_view_model_mutation_kind_against_pinned_fixtures() {
        let runtime_dir = Path::new("/Users/levi/dev/oss/rive-runtime");
        if !runtime_dir.is_dir() {
            return;
        }
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
        let runtime_dir = Path::new("/Users/levi/dev/oss/rive-runtime");
        if !runtime_dir.is_dir() {
            return;
        }
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
