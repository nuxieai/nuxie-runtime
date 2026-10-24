#[test]
fn emit_device_fixtures() {
    use sha2::{Digest, Sha256};
    nuxie_runtime::live_move::device_fixture::emit_device_fixtures();
    let out = std::env::var_os("TEST_UNDECLARED_OUTPUTS_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("nuxie-live-moves-device-v2"));
    // Keep artifact identities in the runner log even if its output base is cleaned.
    for rows in [1, 2, 40] {
        for name in [
            format!("page{rows}.riv"),
            format!("video-inventory{rows}.json"),
        ] {
            let bytes = std::fs::read(out.join(&name)).unwrap();
            println!("fixture-sha256 {name} {:x}", Sha256::digest(&bytes));
        }
    }
}

#[test]
fn lifecycle_writes_play_and_restart_both_authored_arrivals() {
    for rows in [2, 40] {
        check_lifecycle_arrivals(rows);
    }
}

fn check_lifecycle_arrivals(count: usize) {
    use nuxie_render_api::{PersistentFactory, RecordingFactory};
    use nuxie_runtime::{
        File, RuntimeFactoryHandle, RuntimeOwnedViewModelHandle,
        live_move::{LiveMoves, records::LiveSlot, screens},
        source::{
            artboard_component_list::ArtboardComponentList, generated::core_registry::CoreRegistry,
        },
    };
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let bytes = nuxie_runtime::live_move::device_fixture::device_fixture(count);
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let root = file.with_file(|f| f.artboard_named("Page")).unwrap();
    let native_vm = file
        .with_file(|f| f.create_default_view_model_instance_for_artboard(root.core_handle()))
        .unwrap();
    let vm = RuntimeOwnedViewModelHandle::from_native(file.clone(), native_vm.clone()).unwrap();
    let machine = root.default_state_machine_handle().unwrap();
    machine.with_instance_mut(|m| m.bind_view_model_instance(native_vm));
    machine.advance_and_apply(0.0);
    let rows = screens(&root).unwrap();
    let mut moves = LiveMoves::default();
    moves.prepare(&root).unwrap();
    for i in 0..count {
        let row = vm.list_item_by_property_name_path("screens", i).unwrap();
        let row = row.borrow();
        assert_eq!(
            row.string_value_by_property_name_path("screen/phase")
                .as_deref(),
            Some(b"hidden".as_slice())
        );
        assert_eq!(
            row.string_value_by_property_name_path("screen/transition")
                .as_deref(),
            Some(b"".as_slice())
        );
        assert_eq!(
            row.number_value_by_property_name_path("screen/appearances"),
            Some(0.0)
        );
    }
    let a = vm.list_item_by_property_name_path("screens", 0).unwrap();
    let b = vm.list_item_by_property_name_path("screens", 1).unwrap();
    let b_flight = rows["B"]
        .with_artboard(|a| a.find_all_handles::<LiveSlot>())
        .into_iter()
        .find(|h| {
            h.with_downcast::<LiveSlot, _>(|s| s.values.role == 1)
                .unwrap()
        })
        .unwrap();
    // Both guards matter: entering with another name, and the right name while
    // active, must leave the authored progress at zero.
    for (phase, step) in [("entering", "other"), ("active", "t1")] {
        b.borrow_mut()
            .set_string_by_property_name_path("screen/phase", phase.as_bytes());
        b.borrow_mut()
            .set_string_by_property_name_path("screen/transition", step.as_bytes());
        machine.advance_and_apply(0.25);
        moves.after_advance(&root).unwrap();
        assert_eq!(CoreRegistry::get_double_handle(&b_flight, 62003), Some(0.0));
    }
    b.borrow_mut()
        .set_string_by_property_name_path("screen/phase", b"hidden");
    machine.advance_and_apply(0.0);
    // The SDK owns only lifecycle values and the begin/end calls. No flight
    // property is written by this test, including on the second traversal.
    a.borrow_mut()
        .set_string_by_property_name_path("screen/phase", b"active");
    for (step, incoming, outgoing, incoming_vm, outgoing_vm) in [
        ("t1", "B", "A", &b, &a),
        ("back", "A", "B", &a, &b),
        ("t1", "B", "A", &b, &a),
        ("back", "A", "B", &a, &b),
    ] {
        for (row, phase) in [(incoming_vm, "entering"), (outgoing_vm, "exiting")] {
            assert!(
                row.borrow_mut()
                    .set_string_by_property_name_path("screen/phase", phase.as_bytes())
            );
            row.borrow_mut()
                .set_string_by_property_name_path("screen/transition", step.as_bytes());
        }
        moves.step_begin(&root, incoming, Some(step)).unwrap();
        let flight = rows[incoming]
            .with_artboard(|a| a.find_all_handles::<LiveSlot>())
            .into_iter()
            .find(|h| {
                h.with_downcast::<LiveSlot, _>(|s| s.values.role == 1)
                    .unwrap()
            })
            .unwrap();
        let screen_machine = root
            .with_artboard(|a| a.find_handle::<ArtboardComponentList>("screens"))
            .unwrap()
            .with_downcast::<ArtboardComponentList, _>(|l| {
                l.state_machine_instance(if incoming == "A" { 0 } else { 1 })
            })
            .flatten()
            .unwrap();
        for (dt, expected) in [(0.0, 0.0), (0.5, 0.5), (0.5, 1.0)] {
            machine.advance_and_apply(dt);
            moves.after_advance(&root).unwrap();
            let progress = CoreRegistry::get_double_handle(&flight, 62003).unwrap();
            assert!(
                (progress - expected).abs() < 0.0001,
                "{step} progress expected {expected}, got {progress}"
            );
            let events = screen_machine.with_instance(|m| {
                (0..m.reported_event_count())
                    .map(|i| {
                        CoreRegistry::get_string_handle(
                            m.reported_event_at(i).event.as_ref().unwrap(),
                            4,
                        )
                        .unwrap()
                    })
                    .collect::<Vec<_>>()
            });
            if expected == 1.0 {
                assert_eq!(events, vec![format!("nx_enter_done:{step}")]);
            } else {
                assert!(events.is_empty(), "{step}: premature completion {events:?}");
            }
        }
        // Holding entering must hold the endpoint without replaying the event.
        machine.advance_and_apply(0.1);
        moves.after_advance(&root).unwrap();
        assert_eq!(
            screen_machine.with_instance(|m| m.reported_event_count()),
            0
        );
        moves.step_end(&root, incoming, Some(step)).unwrap();
        incoming_vm
            .borrow_mut()
            .set_string_by_property_name_path("screen/phase", b"active");
        outgoing_vm
            .borrow_mut()
            .set_string_by_property_name_path("screen/phase", b"hidden");
        machine.advance_and_apply(0.0);
        moves.after_advance(&root).unwrap();
        println!(
            "PASS lifecycle rows={count} step={step} incoming={incoming} outgoing={outgoing} progress=0,0.5,1 completion=1"
        );
    }
}
