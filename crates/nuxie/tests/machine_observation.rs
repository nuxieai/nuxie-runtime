//! Observation coverage for `Scene::state_machine_snapshot`: the snapshot must
//! report retained-instance truth (inputs, running, changed states, active
//! animation names) and identify foreign/unknown machines precisely.

use anyhow::Result;
use nuxie::{
    AnimationStateSpec, ArtboardSpec, BooleanInputSpec, LinearAnimationSpec, MachineId,
    MachineLayerSpec, MachineSpec, NodeSpec, Parent, ResolveError, Scene, SceneMachineInputValue,
    ShapeSpec, TriggerInputSpec, props,
};

struct ObservedMachine {
    scene: Scene,
    artboard: nuxie::ArtboardId,
    machine: MachineId,
}

/// One artboard, two one-frame timelines (Idle opacity 0.2 / Active opacity
/// 0.8), a trigger "Go" driving Any -> Active, and a boolean "Armed" with no
/// conditions — the smallest machine that exercises every snapshot field.
fn observed_machine(scene: &mut Scene, name: &str) -> Result<(nuxie::ArtboardId, MachineId)> {
    let ((artboard, machine), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: name.into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Fader".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let idle = tx.animations().create_linear(
            artboard,
            LinearAnimationSpec {
                name: "Idle".into(),
                fps: 60,
                duration: 1,
            },
        )?;
        tx.animations()
            .set_key(idle, shape, props::WORLD_OPACITY, 0, 0.2)?;
        let active = tx.animations().create_linear(
            artboard,
            LinearAnimationSpec {
                name: "Active".into(),
                fps: 60,
                duration: 1,
            },
        )?;
        tx.animations()
            .set_key(active, shape, props::WORLD_OPACITY, 0, 0.8)?;

        let mut machines = tx.machines();
        let machine = machines.create_machine(artboard, MachineSpec { name: None })?;
        let go = machines.create_trigger_input(machine, TriggerInputSpec { name: "Go".into() })?;
        machines.create_boolean_input(
            machine,
            BooleanInputSpec {
                name: "Armed".into(),
                default_value: false,
            },
        )?;
        let layer = machines.create_layer(machine, MachineLayerSpec { name: None })?;
        let entry = machines.create_entry_state(layer)?;
        let any = machines.create_any_state(layer)?;
        machines.create_exit_state(layer)?;
        let idle_state =
            machines.create_animation_state(layer, AnimationStateSpec { animation: idle })?;
        let active_state =
            machines.create_animation_state(layer, AnimationStateSpec { animation: active })?;
        machines.create_transition(entry, idle_state)?;
        let transition = machines.create_transition(any, active_state)?;
        machines.add_trigger_condition(transition, go)?;
        Ok((artboard, machine))
    })?;
    Ok((artboard, machine))
}

fn observed_scene() -> Result<ObservedMachine> {
    let mut scene = Scene::new();
    let (artboard, machine) = observed_machine(&mut scene, "Canvas")?;
    Ok(ObservedMachine {
        scene,
        artboard,
        machine,
    })
}

fn input_value(
    snapshot: &nuxie::SceneMachineSnapshot,
    name: &str,
) -> Option<SceneMachineInputValue> {
    snapshot
        .inputs
        .iter()
        .find(|input| input.name == name)
        .map(|input| input.value.clone())
}

#[test]
fn snapshot_reflects_authored_inputs_and_boolean_mutation() -> Result<()> {
    let ObservedMachine {
        mut scene,
        artboard,
        machine,
    } = observed_scene()?;
    let instance = scene.instantiate(artboard)?;
    let mut events = Vec::new();
    let _ = scene.frame().advance(instance, 0.0, &mut events);

    let snapshot = scene.state_machine_snapshot(instance, machine)?;
    assert_eq!(
        input_value(&snapshot, "Armed"),
        Some(SceneMachineInputValue::Boolean(false)),
        "authored boolean default must be observable",
    );
    assert_eq!(
        input_value(&snapshot, "Go"),
        Some(SceneMachineInputValue::Trigger { fired: false }),
        "idle trigger reads unfired",
    );

    let armed = scene.machine_boolean_input(instance, machine, "Armed")?;
    scene.frame().set_boolean(armed, true)?;
    let snapshot = scene.state_machine_snapshot(instance, machine)?;
    assert_eq!(
        input_value(&snapshot, "Armed"),
        Some(SceneMachineInputValue::Boolean(true)),
        "snapshot must report the mutated retained value, not the authored default",
    );
    Ok(())
}

#[test]
fn snapshot_tracks_state_change_and_active_animation_across_a_trigger() -> Result<()> {
    let ObservedMachine {
        mut scene,
        artboard,
        machine,
    } = observed_scene()?;
    let instance = scene.instantiate(artboard)?;
    let mut events = Vec::new();
    let _ = scene.frame().advance(instance, 0.0, &mut events);

    let idle = scene.state_machine_snapshot(instance, machine)?;
    assert_eq!(
        idle.active_animation_names,
        vec!["Idle".to_string()],
        "entry routes into the Idle animation state",
    );

    let go = scene.machine_input(instance, machine, "Go")?;
    scene.frame().fire(go)?;
    let _ = scene.frame().advance(instance, 0.0, &mut events);

    let active = scene.state_machine_snapshot(instance, machine)?;
    assert_eq!(
        active.active_animation_names,
        vec!["Active".to_string()],
        "the fired trigger must route the layer into Active",
    );
    assert!(
        active.changed_state_count > 0,
        "the transitioning advance reports its state changes",
    );
    assert_eq!(
        input_value(&active, "Go"),
        Some(SceneMachineInputValue::Trigger { fired: false }),
        "triggers read unfired again once the advance consumed them",
    );
    Ok(())
}

#[test]
fn snapshot_distinguishes_foreign_and_unknown_machines() -> Result<()> {
    let mut scene = Scene::new();
    let (artboard_a, _machine_a) = observed_machine(&mut scene, "First")?;
    let (_artboard_b, machine_b) = observed_machine(&mut scene, "Second")?;
    let instance_a = scene.instantiate(artboard_a)?;
    let mut events = Vec::new();
    let _ = scene.frame().advance(instance_a, 0.0, &mut events);

    // Machine B is authored in this scene but owned by a different artboard
    // than the instance being observed.
    assert_eq!(
        scene
            .state_machine_snapshot(instance_a, machine_b)
            .unwrap_err(),
        ResolveError::DifferentArtboard,
    );

    // A handle minted by an unrelated scene resolves nowhere here: with no
    // machines authored at all, any machine id is unknown.
    let mut foreign = Scene::new();
    let (foreign_artboard, foreign_machine) = observed_machine(&mut foreign, "Elsewhere")?;
    let _ = foreign_artboard;
    let mut lonely = Scene::new();
    let ((lonely_artboard, _), _) = lonely.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Lonely".into(),
            width: 100.0,
            height: 100.0,
        })?;
        Ok((artboard, ()))
    })?;
    let lonely_instance = lonely.instantiate(lonely_artboard)?;
    let _ = lonely.frame().advance(lonely_instance, 0.0, &mut events);
    assert_eq!(
        lonely
            .state_machine_snapshot(lonely_instance, foreign_machine)
            .unwrap_err(),
        ResolveError::UnknownMachine,
    );
    Ok(())
}
