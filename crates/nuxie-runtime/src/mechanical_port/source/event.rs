use std::any::{Any, TypeId};

use crate::mechanical_port::source::{
    audio_event::AudioEvent,
    core::{
        CoreHandle,
        field_types::core_callback_type::{CallbackContext, CallbackData},
    },
    generated::event_base::{EventBase, EventBaseCallbacks},
    open_url_event::OpenUrlEvent,
};

#[derive(Default)]
pub struct Event {
    pub base: EventBase,
}

impl Event {
    pub fn trigger(&mut self, value: &mut CallbackData<'_>) {
        let delay_seconds = value.delay_seconds();
        value
            .context()
            .expect("Event::trigger requires CallbackData context")
            .report_event(self, delay_seconds);
    }

    /// Invoke the built-in Event virtual operation when the reporter can carry
    /// occurrence identity across synchronous callbacks. The callback may read
    /// or retire this Event, so no receiver loan survives into it.
    ///
    /// Unknown/custom owners and other properties retain registry dispatch.
    pub(crate) fn trigger_builtin_occurrence<C: CallbackContext + ?Sized>(
        target: &CoreHandle,
        property_key: u32,
        context: &mut C,
        delay_seconds: f32,
        report: impl FnOnce(&mut C, CoreHandle, f32),
    ) -> bool {
        if property_key != u32::from(EventBase::TRIGGER_PROPERTY_KEY) {
            return false;
        }
        let Some(audio) = target
            .with(|object| {
                // Use the actual Rust owner, not an overridable projection or
                // a custom object's claimed generated type key.
                let concrete = Any::type_id(object);
                if concrete == TypeId::of::<AudioEvent>() {
                    Some(true)
                } else if concrete == TypeId::of::<Event>()
                    || concrete == TypeId::of::<OpenUrlEvent>()
                {
                    Some(false)
                } else {
                    None
                }
            })
            .flatten()
        else {
            return false;
        };

        // Event::trigger is the first operation for all three built-in owners.
        report(context, target.clone(), delay_seconds);
        // AudioEvent asks after reportEvent, which can mutate its context.
        if audio && !context.plays_audio() {
            target.with_downcast_mut::<AudioEvent, _>(AudioEvent::play);
        }
        true
    }
}

impl EventBaseCallbacks for Event {
    fn trigger(&mut self, value: &mut CallbackData<'_>) {
        Event::trigger(self, value);
    }
}

impl std::ops::Deref for Event {
    type Target = EventBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for Event {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

#[cfg(test)]
mod occurrence_tests {
    use super::*;
    use crate::mechanical_port::source::{core::CoreArena, node::Node};
    use std::cell::Cell;

    #[derive(Default)]
    struct Context {
        reports: usize,
        audio_queries: Cell<usize>,
        audio_handled: bool,
    }
    impl CallbackContext for Context {
        fn plays_audio(&self) -> bool {
            assert_eq!(self.reports, 1, "report precedes playback decision");
            self.audio_queries.set(self.audio_queries.get() + 1);
            assert!(
                self.audio_handled,
                "the callback's updated value must be visible"
            );
            true
        }
    }

    #[test]
    fn audio_report_reenters_its_occurrence_before_the_playback_query() {
        let arena = CoreArena::default();
        let event = arena.insert(AudioEvent::default());
        let mut context = Context::default();
        assert!(Event::trigger_builtin_occurrence(
            &event,
            EventBase::TRIGGER_PROPERTY_KEY.into(),
            &mut context,
            0.75,
            |context, target, delay| {
                assert_eq!(target, event);
                assert_eq!(delay, 0.75);
                assert_eq!(target.with_downcast_mut::<AudioEvent, _>(|_| ()), Some(()));
                context.reports += 1;
                context.audio_handled = true;
            }
        ));
        assert_eq!(context.audio_queries.get(), 1);
    }

    #[test]
    fn base_event_and_open_url_report_without_audio_queries_or_receiver_loans() {
        let arena = CoreArena::default();
        for event in [
            arena.insert(Event::default()),
            arena.insert(OpenUrlEvent::default()),
        ] {
            let mut context = Context::default();
            assert!(Event::trigger_builtin_occurrence(
                &event,
                EventBase::TRIGGER_PROPERTY_KEY.into(),
                &mut context,
                0.0,
                |context, target, _| {
                    assert_eq!(target.with_mut(|_| ()), Some(()));
                    context.reports += 1;
                }
            ));
            assert_eq!(context.reports, 1);
            assert_eq!(context.audio_queries.get(), 0);
        }
    }

    #[test]
    fn unrelated_fields_and_owners_leave_the_callback_for_registry_dispatch() {
        let arena = CoreArena::default();
        let event = arena.insert(Event::default());
        let node = arena.insert(Node::default());
        let mut context = Context::default();
        for (owner, property) in [
            (&event, 408),
            (&node, u32::from(EventBase::TRIGGER_PROPERTY_KEY)),
        ] {
            assert!(!Event::trigger_builtin_occurrence(
                owner,
                property,
                &mut context,
                0.0,
                |_, _, _| panic!("fallback owner cannot be handled here")
            ));
        }
    }

    #[test]
    fn retired_audio_occurrence_does_not_play_a_replacement() {
        struct NonPlayingContext;
        impl CallbackContext for NonPlayingContext {}
        let arena = CoreArena::default();
        let event = arena.insert(AudioEvent::default());
        assert!(Event::trigger_builtin_occurrence(
            &event,
            EventBase::TRIGGER_PROPERTY_KEY.into(),
            &mut NonPlayingContext,
            0.0,
            |_, target, _| {
                assert!(arena.remove(&target).is_some());
                let replacement = arena.insert(AudioEvent::default());
                assert_ne!(target, replacement);
            }
        ));
        assert!(event.with(|_| ()).is_none());
    }
}
