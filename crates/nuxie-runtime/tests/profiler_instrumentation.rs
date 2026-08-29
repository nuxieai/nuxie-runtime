use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{
    File, ProfileCapture, ProfileCaptureFrame, ProfileCaptureMetadata,
    RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, TransitionRecord, with_rive_profile,
};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct HookCapture {
    tick: u64,
}

impl ProfileCapture for HookCapture {
    fn tick(&mut self) -> u64 {
        let tick = self.tick;
        self.tick += 1;
        tick
    }

    fn metadata(&self) -> ProfileCaptureMetadata {
        ProfileCaptureMetadata::default()
    }

    fn current_frame_index(&self) -> u64 {
        0
    }

    fn gpu_frame_delay(&self) -> u64 {
        1
    }

    fn max_frame_history(&self) -> u64 {
        8
    }

    fn captured_frame(&self, _frame_index: u64) -> Option<ProfileCaptureFrame> {
        None
    }
}

fn fixture(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}

fn artboard_instance(relative: &str) -> RuntimeArtboardInstanceHandle {
    let bytes = std::fs::read(fixture(relative)).expect("read profiler hook fixture");
    let mut factory = PersistentFactory::new(RecordingFactory::default());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let file =
        File::import(&bytes, factory, None, None, None).expect("import profiler hook fixture");
    file.with_file(File::artboard_default)
        .expect("instantiate profiler hook artboard")
}

fn resolved<'a>(strings: &'a [String], id: u32) -> &'a str {
    strings
        .get(id as usize)
        .map(String::as_str)
        .expect("record string id")
}

#[test]
fn production_transition_hook_emits_runtime_names_and_root_path() {
    let transitions = Arc::new(Mutex::new(Vec::<TransitionRecord>::new()));
    with_rive_profile(|profile| {
        profile.set_capture(Box::new(HookCapture::default()));
        profile.set_transition_flush_callback(Some(Box::new({
            let transitions = Arc::clone(&transitions);
            move |records| transitions.lock().unwrap().extend_from_slice(records)
        })));
        profile.start();
    });

    let artboard = artboard_instance("fixtures/animation/state_machine_transition.riv");
    let machine = artboard
        .state_machine_instance_handle(0)
        .expect("state machine");
    machine.advance_and_apply(0.0);

    let strings = with_rive_profile(|profile| {
        profile.flush_transition_records();
        profile.stop();
        let strings = profile.string_table().to_vec();
        profile.set_transition_flush_callback(None);
        strings
    });

    let transitions = transitions.lock().unwrap();
    assert!(transitions.iter().any(|record| {
        resolved(&strings, record.artboard_id) == "Artboard-Test"
            && resolved(&strings, record.sm_id) == "State-Machine-Test"
            && resolved(&strings, record.layer_id) == "Hover-Stroke"
            && resolved(&strings, record.from_state_id) == "Entry"
            && resolved(&strings, record.to_state_id) == "State-1"
            && record.path.is_empty()
    }));
}
