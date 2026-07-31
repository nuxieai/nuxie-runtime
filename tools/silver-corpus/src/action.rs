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
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Action {
    BindDefaultViewModel,
    Advance {
        target: ActionTarget,
        seconds: f32,
    },
    Draw,
    Frame,
    PointerDown {
        x: f32,
        y: f32,
        #[serde(default)]
        pointer_id: i32,
    },
    PointerMove {
        x: f32,
        y: f32,
        seconds: f32,
        #[serde(default)]
        pointer_id: i32,
    },
    PointerUp {
        x: f32,
        y: f32,
        #[serde(default)]
        pointer_id: i32,
    },
    PointerExit {
        x: f32,
        y: f32,
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
    SetArtboardSize {
        width: f32,
        height: f32,
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
                    } else {
                        instance.bind_default_view_model_artboard_list_context(&runtime);
                        if let Some(machine) = state_machine.as_mut() {
                            machine.bind_default_view_model_context();
                            machine.advance_data_context();
                        }
                    }
                }
                Action::Advance { target, seconds } => match target {
                    StateMachine => advance_state_machine(
                        &mut instance,
                        state_machine
                            .as_mut()
                            .context("no selected state machine")?,
                        *seconds,
                    )?,
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
                    state_machine
                        .as_mut()
                        .context("no selected state machine")?
                        .pointer_down(&mut instance, *x, *y, *pointer_id);
                }
                Action::PointerMove {
                    x,
                    y,
                    seconds,
                    pointer_id,
                } => {
                    state_machine
                        .as_mut()
                        .context("no selected state machine")?
                        .pointer_move(&mut instance, *x, *y, *seconds, *pointer_id);
                }
                Action::PointerUp { x, y, pointer_id } => {
                    state_machine
                        .as_mut()
                        .context("no selected state machine")?
                        .pointer_up(&mut instance, *x, *y, *pointer_id);
                }
                Action::PointerExit { x, y, pointer_id } => {
                    state_machine
                        .as_mut()
                        .context("no selected state machine")?
                        .pointer_exit(&mut instance, *x, *y, *pointer_id);
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
                Action::SetArtboardSize { width, height } => {
                    instance.set_artboard_dimensions(*width, *height);
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
) -> anyhow::Result<()> {
    instance.advance_state_machine_instance(state_machine, seconds);
    if instance
        .advance_frame_components_with_state_machine(seconds, state_machine)
        .map_err(|error| anyhow::anyhow!(error))
        .context("retained frame-component advance failed")?
    {
        instance.advance_state_machine_instance(state_machine, 0.0);
    }
    instance
        .settle_state_machine_update_passes_after_main_advance_with_script_errors(
            std::slice::from_mut(state_machine),
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

fn frame_dimension(value: f32) -> u32 {
    value.ceil().max(1.0) as u32
}
