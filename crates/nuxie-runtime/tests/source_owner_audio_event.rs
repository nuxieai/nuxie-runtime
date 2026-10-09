//! AudioEvent's most-derived callback operation in the pinned 160085 runtime.
use nuxie_runtime::source::{
    audio_event::AudioEvent,
    core::{
        CoreArena, CoreHandle, CoreObject,
        binary_reader::BinaryReader,
        field_types::core_callback_type::{CallbackContext, CallbackData},
    },
    event::Event,
    generated::{
        audio_event_base::AudioEventBase,
        component_base::ComponentBase,
        core_registry::CoreRegistry,
        event_base::{EventBase, EventBaseCallbacks},
    },
};
use std::cell::RefCell;

#[derive(Default)]
struct Context {
    calls: RefCell<Vec<&'static str>>,
    expected_event: Option<CoreHandle>,
    expected_delay: f32,
    reports: usize,
}
impl CallbackContext for Context {
    fn report_event(&mut self, event: &mut Event, delay: f32) {
        assert_eq!(event.handle(), self.expected_event);
        assert_eq!(delay, self.expected_delay);
        self.calls.borrow_mut().push("report");
        self.reports += 1;
    }
    fn plays_audio(&self) -> bool {
        self.calls.borrow_mut().push("plays_audio");
        assert_eq!(
            self.reports, 1,
            "AudioEvent must report before querying playback"
        );
        true
    }
}

#[test]
fn registry_audio_event_callback_uses_the_derived_trigger() {
    let arena = CoreArena::default();
    let event = arena.insert(AudioEvent::default());
    let mut context = Context {
        expected_event: Some(event.clone()),
        expected_delay: 0.25,
        ..Context::default()
    };
    assert!(CoreRegistry::set_callback_handle(
        &event,
        EventBase::TRIGGER_PROPERTY_KEY.into(),
        CallbackData::new(Some(&mut context), 0.25)
    ));
    assert_eq!(&*context.calls.borrow(), &["report", "plays_audio"]);
}

#[test]
fn generated_event_callback_uses_the_audio_event_override() {
    let mut event = AudioEvent::default();
    let mut context = Context {
        expected_delay: -0.5,
        ..Context::default()
    };
    EventBaseCallbacks::trigger(&mut event, &mut CallbackData::new(Some(&mut context), -0.5));
    assert_eq!(&*context.calls.borrow(), &["report", "plays_audio"]);
}

#[test]
fn audio_event_registration_preserves_type_properties_clone_and_runtime_ids() {
    let mut event = CoreRegistry::make_core_box(AudioEventBase::TYPE_KEY.into()).unwrap();
    assert_eq!(event.core_type(), AudioEventBase::TYPE_KEY);
    for ancestor in [407, 128, 548, 11, 10] {
        assert!(CoreObject::is_type_of(&*event, ancestor));
    }
    assert!(!CoreObject::is_type_of(&*event, 2));
    assert_eq!(
        CoreRegistry::get_uint(&mut *event, AudioEventBase::ASSET_ID_PROPERTY_KEY.into()),
        u32::MAX
    );
    CoreRegistry::set_uint(
        &mut *event,
        AudioEventBase::ASSET_ID_PROPERTY_KEY.into(),
        71,
    );
    CoreRegistry::set_uint(&mut *event, ComponentBase::PARENT_ID_PROPERTY_KEY.into(), 9);
    CoreRegistry::set_string(
        &mut *event,
        ComponentBase::NAME_PROPERTY_KEY.into(),
        "sound".into(),
    );
    let mut cloned = event.clone_boxed().unwrap();
    assert_eq!(
        CoreRegistry::get_uint(&mut *cloned, AudioEventBase::ASSET_ID_PROPERTY_KEY.into()),
        71
    );
    assert_eq!(
        CoreRegistry::get_uint(&mut *cloned, ComponentBase::PARENT_ID_PROPERTY_KEY.into()),
        9
    );
    assert_eq!(
        CoreRegistry::get_string(&mut *cloned, ComponentBase::NAME_PROPERTY_KEY.into()),
        "sound"
    );
    let mut reader = BinaryReader::new(&[0xac, 0x02]);
    assert!(event.deserialize(AudioEventBase::ASSET_ID_PROPERTY_KEY, &mut reader));
    assert!(reader.reached_end());
    assert!(!reader.has_error());
    assert_eq!(
        CoreRegistry::get_uint(&mut *event, AudioEventBase::ASSET_ID_PROPERTY_KEY.into()),
        300
    );
}

#[test]
fn silent_audio_event_does_not_initialize_the_runtime_engine() {
    use nuxie_runtime::source::{
        artboard::Artboard,
        assets::audio_asset::AudioAsset,
        audio::{audio_engine::AudioEngine, audio_source::AudioSource},
        core_context::CoreContext,
        status_code::StatusCode,
    };
    use std::sync::Arc;
    struct SceneContext {
        arena: CoreArena,
        objects: Vec<CoreHandle>,
    }
    impl CoreContext for SceneContext {
        fn core_arena(&self) -> &CoreArena {
            &self.arena
        }
        fn resolve_handle(&self, id: u32) -> Option<CoreHandle> {
            self.objects.get(id as usize).cloned()
        }
    }
    let arena = CoreArena::default();
    let root = arena.insert(Artboard::default());
    let event = arena.insert(AudioEvent::default());
    let mut asset = AudioAsset::default();
    asset.set_audio_source(Some(Arc::new(AudioSource::buffered(
        Arc::from([0.0_f32; 16]),
        1,
        44_100,
    ))));
    asset.base.set_volume(0.0);
    let asset = arena.insert(asset);
    let mut context = SceneContext {
        arena,
        objects: vec![root.clone(), event.clone()],
    };
    assert_eq!(
        root.with_mut(|owner| owner
            .as_component_mut()
            .unwrap()
            .on_added_dirty(&mut context)),
        Some(StatusCode::Ok)
    );
    assert_eq!(
        event.with_mut(|owner| owner.on_added_dirty(&mut context)),
        Some(StatusCode::Ok)
    );
    event.with_downcast_mut::<AudioEvent, _>(|event| event.set_asset(Some(asset)));
    // The other tests in this integration binary do not create audio engines.
    AudioEngine::release_runtime_engine();
    assert!(AudioEngine::runtime_engine(false).is_none());
    event.with_downcast_mut::<AudioEvent, _>(AudioEvent::play);
    assert!(
        AudioEngine::runtime_engine(false).is_none(),
        "source rejects silence before resolving the engine"
    );
}
