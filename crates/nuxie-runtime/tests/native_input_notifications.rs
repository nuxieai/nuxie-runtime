//! Pinned SMIInput::valueChanged ordering over the actual machine/input owners.
#![cfg(feature = "tools")]

use std::{cell::RefCell, path::PathBuf, rc::Rc};

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    animation::{
        listener_bool_change::ListenerBoolChange, listener_invocation::ListenerInvocation,
        state_machine_instance::RuntimeStateMachineInstanceHandle,
    },
    artboard::RuntimeArtboardInstanceHandle,
    factory::RuntimeFactoryHandle,
    file::{File, RuntimeFileHandle},
};

fn rocket() -> (
    RuntimeFileHandle,
    RuntimeArtboardInstanceHandle,
    RuntimeStateMachineInstanceHandle,
) {
    let upstream = std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));
    let bytes = std::fs::read(upstream.join("tests/unit_tests/assets/rocket.riv")).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    let file = File::import(&bytes, factory, None, None, None).unwrap();
    let artboard = file.with_file(|file| file.artboard_default()).unwrap();
    let machine = artboard.state_machine_instance_handle(0).unwrap();
    machine.with_instance(|machine| {
        assert_eq!(machine.name(), "Button");
        assert_eq!(machine.input_count(), 2);
        assert!(machine.get_bool("Hover").is_some());
        assert!(machine.get_bool("Press").is_some());
    });
    (file, artboard, machine)
}

fn input_index(machine: &RuntimeStateMachineInstanceHandle, name: &str) -> u32 {
    machine.with_instance(|machine| {
        (0..machine.input_count())
            .find(|index| machine.input(*index).unwrap().name() == name)
            .unwrap() as u32
    })
}

fn toggle(index: u32) -> ListenerBoolChange {
    let mut action = ListenerBoolChange::default();
    let mut base = std::mem::take(&mut action.base);
    base.base.set_input_id(index, &mut action);
    base.set_value(2, &mut action);
    action.base = base;
    action
}

#[test]
fn listener_bool_change_without_handler_does_not_reborrow_its_machine() {
    let (_file, _artboard, machine) = rocket();
    let hover = input_index(&machine, "Hover");
    let action = toggle(hover);
    machine.with_instance_mut(|machine| {
        let previous = machine.bool_input(hover).unwrap().value();
        // This is the actual listener call shape: its machine is already
        // borrowed while the real input performs synchronous notification.
        action.perform(machine, &ListenerInvocation::none());
        assert_eq!(machine.bool_input(hover).unwrap().value(), !previous);
        assert!(machine.needs_advance());
    });
}

#[test]
fn listener_callbacks_preserve_input_indices_and_synchronous_call_order() {
    let (_file, _artboard, machine) = rocket();
    let hover = input_index(&machine, "Hover");
    let press = input_index(&machine, "Press");
    let observations = Rc::new(RefCell::new(Vec::new()));
    let captured = observations.clone();
    let expected_machine = machine.downgrade();
    machine.with_instance_mut(|machine| {
        machine.on_input_changed(Some(Box::new(move |machine, index| {
            assert!(machine.ptr_eq(&expected_machine));
            captured.borrow_mut().push(("callback", index));
        })));
    });
    for index in [press, hover] {
        observations.borrow_mut().push(("before", u64::from(index)));
        machine.with_instance_mut(|machine| {
            toggle(index).perform(machine, &ListenerInvocation::none())
        });
        observations.borrow_mut().push(("after", u64::from(index)));
    }
    assert_eq!(
        observations.borrow().as_slice(),
        [
            ("before", u64::from(press)),
            ("callback", u64::from(press)),
            ("after", u64::from(press)),
            ("before", u64::from(hover)),
            ("callback", u64::from(hover)),
            ("after", u64::from(hover)),
        ]
    );
}

#[test]
fn handle_setter_publishes_value_and_dirt_before_callback_and_skips_noop_writes() {
    let (_file, _artboard, machine) = rocket();
    let hover = input_index(&machine, "Hover");
    let initial = machine.with_instance(|machine| machine.bool_input(hover).unwrap().value());
    let observations = Rc::new(RefCell::new(Vec::new()));
    let captured = observations.clone();
    machine.with_instance_mut(|machine| {
        machine.on_input_changed(Some(Box::new(move |machine, index| {
            let observation = machine
                .with_instance(|machine| {
                    (
                        index,
                        machine.bool_input(hover).unwrap().value(),
                        machine.needs_advance(),
                    )
                })
                .unwrap();
            captured.borrow_mut().push(observation);
        })))
    });
    machine.set_bool("Hover", initial);
    assert!(observations.borrow().is_empty());
    machine.set_bool("Hover", !initial);
    machine.set_bool("Hover", !initial);
    machine.set_bool("Hover", initial);
    assert_eq!(
        observations.borrow().as_slice(),
        [
            (u64::from(hover), !initial, true),
            (u64::from(hover), initial, true),
        ]
    );
}

#[test]
fn callback_replacement_and_clearing_take_effect_on_the_next_synchronous_notification() {
    let (_file, _artboard, machine) = rocket();
    let hover = input_index(&machine, "Hover");
    let press = input_index(&machine, "Press");
    let initial_hover = machine.with_instance(|machine| machine.bool_input(hover).unwrap().value());
    let initial_press = machine.with_instance(|machine| machine.bool_input(press).unwrap().value());
    let observations = Rc::new(RefCell::new(Vec::new()));
    let captured = observations.clone();
    machine.with_instance_mut(|machine| {
        machine.on_input_changed(Some(Box::new(move |machine, index| {
            captured.borrow_mut().push(("first-enter", index));
            let second_capture = captured.clone();
            let machine = machine.upgrade().unwrap();
            machine.with_instance_mut(|machine| {
                machine.on_input_changed(Some(Box::new(move |machine, index| {
                    second_capture.borrow_mut().push(("second", index));
                    machine
                        .with_instance_mut(|machine| machine.on_input_changed(None))
                        .unwrap();
                })))
            });
            // This calls the newly installed callback, not the active first one.
            machine.set_bool("Press", !initial_press);
            captured.borrow_mut().push(("first-exit", index));
        })))
    });
    machine.set_bool("Hover", !initial_hover);
    machine.set_bool("Hover", initial_hover);
    machine.set_bool("Press", initial_press);
    assert_eq!(
        observations.borrow().as_slice(),
        [
            ("first-enter", u64::from(hover)),
            ("second", u64::from(press)),
            ("first-exit", u64::from(hover)),
        ]
    );
    assert_eq!(
        machine.with_instance(|machine| machine.bool_input(hover).unwrap().value()),
        initial_hover
    );
    assert_eq!(
        machine.with_instance(|machine| machine.bool_input(press).unwrap().value()),
        initial_press
    );
}
