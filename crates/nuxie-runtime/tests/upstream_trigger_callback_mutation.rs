//! The source tools trigger callback may inspect and remove its bound value
//! during ordinary writes too, not only during advanced()'s reset.
#![cfg(feature = "tools")]
use nuxie_runtime::source::{
    core::{CoreArena, CoreHandle},
    data_bind::{
        data_bind::{DataBind, TO_SOURCE},
        data_values::data_value_integer::DataValueInteger,
    },
    generated::{
        core_registry::CoreRegistry,
        viewmodel::{
            viewmodel_instance_trigger_base::ViewModelInstanceTriggerBase,
            viewmodel_instance_value_base::ViewModelInstanceValueBase,
        },
    },
    viewmodel::{
        viewmodel_instance::ViewModelInstance, viewmodel_instance_trigger::ViewModelInstanceTrigger,
    },
};

fn remove_self(trigger: &CoreHandle, changed: u32) {
    let (parent, id) = trigger
        .with_downcast::<ViewModelInstanceTrigger, _>(|trigger| {
            assert_eq!(trigger.base.property_value(), changed);
            (
                trigger.base.view_model_instance().unwrap(),
                trigger.base.view_model_property_id(),
            )
        })
        .unwrap();
    assert!(
        parent
            .with_downcast_mut::<ViewModelInstance, _>(|parent| parent.remove_value(id))
            .unwrap()
    );
}

#[test]
fn ordinary_trigger_callbacks_can_read_and_remove_bound_self() {
    for route in 0..4 {
        let arena = CoreArena::default();
        let parent = arena.insert(ViewModelInstance::default());
        let trigger = arena.insert(ViewModelInstanceTrigger::default());
        CoreRegistry::set_uint_handle(
            &trigger,
            ViewModelInstanceValueBase::VIEW_MODEL_PROPERTY_ID_PROPERTY_KEY.into(),
            1,
        );
        parent
            .with_downcast_mut::<ViewModelInstance, _>(|parent| parent.add_value(trigger.clone()))
            .unwrap();
        let bind = arena.insert(DataBind::new(
            TO_SOURCE,
            ViewModelInstanceTriggerBase::PROPERTY_VALUE_PROPERTY_KEY.into(),
            0,
        ));
        bind.with_mut(|object| {
            let bind = object.as_data_bind_mut().unwrap();
            bind.set_target(Some(trigger.clone()));
            bind.set_source(trigger.clone());
            assert!(bind.target_supports_push());
        })
        .unwrap();
        parent
            .with_downcast_mut::<ViewModelInstance, _>(|parent| {
                parent.add_value_data_bind(bind.clone())
            })
            .unwrap();
        trigger
            .with_downcast_mut::<ViewModelInstanceTrigger, _>(|trigger| {
                trigger.on_changed(Some(remove_self))
            })
            .unwrap();
        assert!(match route {
            0 => CoreRegistry::set_uint_handle(
                &trigger,
                ViewModelInstanceTriggerBase::PROPERTY_VALUE_PROPERTY_KEY.into(),
                7
            ),
            1 => ViewModelInstanceTrigger::trigger_handle(&trigger),
            2 => ViewModelInstanceTrigger::apply_value_handle(&trigger, &DataValueInteger::new(11)),
            _ => CoreRegistry::set_callback_handle(
                &trigger,
                ViewModelInstanceTriggerBase::FIRE_PROPERTY_KEY.into(),
                nuxie_runtime::source::core::field_types::core_callback_type::CallbackData::new(
                    None, 0.0
                )
            ),
        });
        assert!(
            parent
                .with_downcast::<ViewModelInstance, _>(|parent| parent.property_values().is_empty()
                    && parent.value_data_binds().is_empty())
                .unwrap()
        );
        assert!(bind.with(|_| ()).is_none());
        assert_eq!(
            trigger
                .with_downcast::<ViewModelInstanceTrigger, _>(|trigger| trigger
                    .base
                    .property_value())
                .unwrap(),
            [7, 1, 11, 1][route]
        );
    }
}
