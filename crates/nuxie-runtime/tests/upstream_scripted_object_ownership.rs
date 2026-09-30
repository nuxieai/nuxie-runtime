//! 1371150d scripted_object_ownership_test.cpp and a separate Rust arena check.
//! The upstream import assertion relies on an external leak sanitizer. These
//! tests do not claim to execute that sanitizer.
use std::{cell::RefCell, rc::Rc};

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    core::CoreHandle, file::ImportAdmission,
    generated::scripted::scripted_interpolator_base::ScriptedInterpolatorBase,
    scripted::scripted_interpolator::ScriptedInterpolator,
};
use nuxie_runtime::{File, RuntimeFactoryHandle};

fn fixture() -> Vec<u8> {
    let root = std::path::PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned upstream checkout"),
    );
    std::fs::read(root.join("tests/unit_tests/assets/data_bound_keyframe_test.riv")).unwrap()
}

fn factory() -> RuntimeFactoryHandle {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    RuntimeFactoryHandle::from_factory(&mut factory).unwrap()
}

#[test]
fn script_inputs_on_a_scripted_interpolator_are_freed() {
    let file = File::import(&fixture(), factory(), None, None, None);
    assert!(file.is_some());
}

#[derive(Default)]
struct ObserveInterpolator(RefCell<Vec<CoreHandle>>);
impl ImportAdmission for ObserveInterpolator {
    fn admit_object(&self, object: &CoreHandle) -> bool {
        if object.is_type_of(ScriptedInterpolatorBase::TYPE_KEY) {
            self.0.borrow_mut().push(object.clone());
        }
        true
    }
    fn admit_asset_bytes(&self, _: &CoreHandle, _: &[u8]) -> bool {
        true
    }
    fn admit_loaded_asset(&self, _: &CoreHandle) -> bool {
        true
    }
    fn is_rejected(&self) -> bool {
        false
    }
}

// Supplemental: test the approved Rust arena boundary directly, keeping File
// alive while retiring its interpolator so arena-wide teardown cannot conceal
// an omitted owner destructor.
#[test]
fn retiring_interpolator_retires_owned_inputs_before_the_file_arena() {
    let observed = Rc::new(ObserveInterpolator::default());
    let file =
        File::import_with_admission(&fixture(), factory(), None, None, None, observed.clone())
            .unwrap();
    let weak_file = file.downgrade();
    let owners = observed.0.borrow().clone();
    assert!(!owners.is_empty());
    let owner = owners
        .iter()
        .find(|owner| {
            owner
                .with_downcast::<ScriptedInterpolator, _>(|owner| owner.properties.len() == 2)
                .unwrap_or(false)
        })
        .expect("fixture interpolator owns two ScriptInputs");
    let inputs = owner
        .with_downcast::<ScriptedInterpolator, _>(|owner| owner.properties.clone())
        .unwrap();
    assert!(inputs.iter().all(CoreHandle::is_alive));
    assert!(owner.remove_occurrence());
    assert!(!owner.is_alive());
    assert!(inputs.iter().all(|input| !input.is_alive()));
    assert!(weak_file.upgrade().is_some());
    drop(file);
    assert!(weak_file.upgrade().is_none());
}
