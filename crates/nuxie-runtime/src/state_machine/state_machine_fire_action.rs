use super::StateMachineReportedEvent;
use super::listener_action::RuntimeScheduledListenerActionExecutor;
use super::state_machine_fire_event::RuntimeStateMachineFireEvent;
use super::state_machine_fire_trigger::{
    RuntimeStateMachineFireTriggerPath, runtime_fire_trigger_path,
};
use crate::ArtboardInstance;
use nuxie_binary::RuntimeFile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StateMachineFireOccurrence {
    AtStart,
    AtEnd,
}

impl StateMachineFireOccurrence {
    pub(crate) fn value(self) -> u64 {
        match self {
            Self::AtStart => 0,
            Self::AtEnd => 1,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum RuntimeStateMachineFireAction {
    Event(RuntimeStateMachineFireEvent),
    Trigger {
        action_owner: super::RuntimeActionCoreHandle,
        path: Option<RuntimeStateMachineFireTriggerPath>,
    },
}

impl RuntimeStateMachineFireAction {
    pub(crate) fn from_imported(
        file: &RuntimeFile,
        action: &nuxie_binary::RuntimeStateMachineFireAction<'_>,
        action_owner: super::RuntimeActionCoreHandle,
    ) -> Self {
        match action.object.type_name {
            "StateMachineFireEvent" => Self::Event(RuntimeStateMachineFireEvent { action_owner }),
            "StateMachineFireTrigger" => Self::Trigger {
                action_owner,
                path: runtime_fire_trigger_path(file, action.object),
            },
            _ => Self::Event(RuntimeStateMachineFireEvent { action_owner }),
        }
    }
}

pub(crate) fn perform_state_machine_fire_actions(
    fire_actions: &[RuntimeStateMachineFireAction],
    occurrence: StateMachineFireOccurrence,
    artboard: &ArtboardInstance,
    executor: &mut dyn RuntimeScheduledListenerActionExecutor,
    reported_events: &mut Vec<StateMachineReportedEvent>,
) {
    for action in fire_actions {
        match action {
            RuntimeStateMachineFireAction::Event(event)
                if event
                    .action_owner
                    .uint(super::listener_action_owner::FIRE_OCCURS_VALUE_KEY)
                    == occurrence.value() =>
            {
                event.perform(artboard, reported_events);
            }
            RuntimeStateMachineFireAction::Trigger { action_owner, path }
                if action_owner.uint(super::listener_action_owner::FIRE_OCCURS_VALUE_KEY)
                    == occurrence.value() =>
            {
                if let Some(path) = path {
                    executor.fire_view_model_trigger(path);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::properties::property_key_for_name;
    use crate::scripting::ScriptError;
    use crate::state_machine::{
        RuntimeScheduledListenerAction, RuntimeScheduledListenerActionTargetsMut,
    };
    use nuxie_binary::{FixtureProperty, FixtureRecord, FixtureValue};
    use nuxie_graph::GraphFile;

    struct NoopExecutor;

    impl RuntimeScheduledListenerActionExecutor for NoopExecutor {
        fn perform_instance_action(
            &mut self,
            _artboard: &mut ArtboardInstance,
            _action: &RuntimeScheduledListenerAction,
            _targets: RuntimeScheduledListenerActionTargetsMut<'_>,
        ) -> Result<bool, ScriptError> {
            Ok(false)
        }
    }

    fn record(type_name: &str, properties: Vec<FixtureProperty>) -> FixtureRecord {
        FixtureRecord {
            type_key: nuxie_schema::definition_by_name(type_name)
                .unwrap_or_else(|| panic!("missing {type_name}"))
                .type_key
                .int,
            properties,
        }
    }

    fn property(type_name: &str, name: &str, value: FixtureValue) -> FixtureProperty {
        FixtureProperty {
            key: property_key_for_name(type_name, name)
                .unwrap_or_else(|| panic!("missing {type_name}.{name}")),
            value,
        }
    }

    #[test]
    fn state_fire_event_resolves_the_live_event_when_performed() {
        let file = RuntimeFile::from_fixture_records(vec![
            record("Backboard", Vec::new()),
            record("Artboard", Vec::new()),
            record(
                "Event",
                vec![
                    property("Event", "parentId", FixtureValue::Uint(0)),
                    property("Event", "name", FixtureValue::String("imported".to_owned())),
                ],
            ),
            record(
                "CustomPropertyString",
                vec![
                    property("CustomPropertyString", "parentId", FixtureValue::Uint(1)),
                    property(
                        "CustomPropertyString",
                        "name",
                        FixtureValue::String("payload".to_owned()),
                    ),
                    property(
                        "CustomPropertyString",
                        "propertyValue",
                        FixtureValue::String("imported".to_owned()),
                    ),
                ],
            ),
            record(
                "CustomPropertyNumber",
                vec![
                    property("CustomPropertyNumber", "parentId", FixtureValue::Uint(1)),
                    property(
                        "CustomPropertyNumber",
                        "name",
                        FixtureValue::String("number".to_owned()),
                    ),
                    property(
                        "CustomPropertyNumber",
                        "propertyValue",
                        FixtureValue::Double(1.0),
                    ),
                ],
            ),
            record(
                "CustomPropertyBoolean",
                vec![
                    property("CustomPropertyBoolean", "parentId", FixtureValue::Uint(1)),
                    property(
                        "CustomPropertyBoolean",
                        "name",
                        FixtureValue::String("boolean".to_owned()),
                    ),
                    property(
                        "CustomPropertyBoolean",
                        "propertyValue",
                        FixtureValue::Bool(false),
                    ),
                ],
            ),
            record(
                "CustomPropertyColor",
                vec![
                    property("CustomPropertyColor", "parentId", FixtureValue::Uint(1)),
                    property(
                        "CustomPropertyColor",
                        "name",
                        FixtureValue::String("color".to_owned()),
                    ),
                    property(
                        "CustomPropertyColor",
                        "propertyValue",
                        FixtureValue::Color(0),
                    ),
                ],
            ),
            record(
                "CustomPropertyEnum",
                vec![
                    property("CustomPropertyEnum", "parentId", FixtureValue::Uint(1)),
                    property(
                        "CustomPropertyEnum",
                        "name",
                        FixtureValue::String("enum".to_owned()),
                    ),
                    property("CustomPropertyEnum", "propertyValue", FixtureValue::Uint(0)),
                ],
            ),
            record(
                "CustomPropertyTrigger",
                vec![
                    property("CustomPropertyTrigger", "parentId", FixtureValue::Uint(1)),
                    property(
                        "CustomPropertyTrigger",
                        "name",
                        FixtureValue::String("trigger".to_owned()),
                    ),
                    property(
                        "CustomPropertyTrigger",
                        "propertyValue",
                        FixtureValue::Uint(0),
                    ),
                ],
            ),
        ])
        .expect("state fire-event records import");
        let graph = GraphFile::from_runtime_file(&file).expect("state fire-event graph builds");
        let mut artboard = ArtboardInstance::from_graph_with_artboards(
            &file,
            graph.artboards.first().expect("state fire-event artboard"),
            &graph.artboards,
        )
        .expect("state fire-event artboard instantiates");
        let name_key = property_key_for_name("Event", "name").expect("Event.name");
        let value_key =
            property_key_for_name("CustomPropertyString", "propertyValue").expect("string value");
        let custom_name_key =
            property_key_for_name("CustomPropertyString", "name").expect("custom name");
        assert!(artboard.set_string_property(1, name_key, b"live".to_vec()));
        assert!(artboard.set_string_property(2, custom_name_key, b"live payload name".to_vec()));
        assert!(artboard.set_string_property(2, value_key, b"live payload".to_vec()));
        assert!(artboard.set_double_property(
            3,
            property_key_for_name("CustomPropertyNumber", "propertyValue").expect("number value"),
            7.5,
        ));
        assert!(artboard.set_bool_property(
            4,
            property_key_for_name("CustomPropertyBoolean", "propertyValue").expect("boolean value"),
            true,
        ));
        assert!(artboard.set_color_property(
            5,
            property_key_for_name("CustomPropertyColor", "propertyValue").expect("color value"),
            0x1234_5678,
        ));
        assert!(artboard.set_uint_property(
            6,
            property_key_for_name("CustomPropertyEnum", "propertyValue").expect("enum value"),
            9,
        ));
        assert!(artboard.set_uint_property(
            7,
            property_key_for_name("CustomPropertyTrigger", "propertyValue").expect("trigger value"),
            11,
        ));

        let actions = [RuntimeStateMachineFireAction::Event(
            RuntimeStateMachineFireEvent::for_test(
                StateMachineFireOccurrence::AtStart.value(),
                Some(1),
            ),
        )];
        let mut reported = Vec::new();
        perform_state_machine_fire_actions(
            &actions,
            StateMachineFireOccurrence::AtStart,
            &artboard,
            &mut NoopExecutor,
            &mut reported,
        );

        assert_eq!(reported.len(), 1);
        assert_eq!(reported[0].name(), Some("live"));
        assert_eq!(
            reported[0]
                .string_properties()
                .iter()
                .map(|property| (property.name(), property.value()))
                .collect::<Vec<_>>(),
            [("live payload name", "live payload")]
        );
        assert_eq!(
            reported[0]
                .properties()
                .iter()
                .map(|property| (property.name.as_deref(), property.value.clone()))
                .collect::<Vec<_>>(),
            [
                (
                    Some("live payload name"),
                    crate::RuntimeEventPropertyValue::String(b"live payload".to_vec())
                ),
                (
                    Some("number"),
                    crate::RuntimeEventPropertyValue::Number(7.5)
                ),
                (
                    Some("boolean"),
                    crate::RuntimeEventPropertyValue::Bool(true)
                ),
                (
                    Some("color"),
                    crate::RuntimeEventPropertyValue::Color(0x1234_5678)
                ),
                (Some("enum"), crate::RuntimeEventPropertyValue::Enum(9)),
                (
                    Some("trigger"),
                    crate::RuntimeEventPropertyValue::Trigger(11)
                ),
            ]
        );
    }

    #[test]
    fn state_fire_open_url_reads_live_url_and_target_when_performed() {
        let file = RuntimeFile::from_fixture_records(vec![
            record("Backboard", Vec::new()),
            record("Artboard", Vec::new()),
            record(
                "OpenUrlEvent",
                vec![
                    property("OpenUrlEvent", "parentId", FixtureValue::Uint(0)),
                    property(
                        "OpenUrlEvent",
                        "url",
                        FixtureValue::String("imported".to_owned()),
                    ),
                    property("OpenUrlEvent", "targetValue", FixtureValue::Uint(0)),
                ],
            ),
        ])
        .expect("open-url records import");
        let graph = GraphFile::from_runtime_file(&file).expect("open-url graph builds");
        let mut artboard = ArtboardInstance::from_graph_with_artboards(
            &file,
            graph.artboards.first().expect("open-url artboard"),
            &graph.artboards,
        )
        .expect("open-url artboard instantiates");
        assert!(artboard.set_string_property(
            1,
            property_key_for_name("OpenUrlEvent", "url").expect("url key"),
            b"https://live.example".to_vec(),
        ));
        assert!(artboard.set_uint_property(
            1,
            property_key_for_name("OpenUrlEvent", "targetValue").expect("target key"),
            2,
        ));

        let mut reported = Vec::new();
        perform_state_machine_fire_actions(
            &[RuntimeStateMachineFireAction::Event(
                RuntimeStateMachineFireEvent::for_test(
                    StateMachineFireOccurrence::AtStart.value(),
                    Some(1),
                ),
            )],
            StateMachineFireOccurrence::AtStart,
            &artboard,
            &mut NoopExecutor,
            &mut reported,
        );

        assert_eq!(reported.len(), 1);
        assert_eq!(reported[0].url(), Some("https://live.example"));
        assert_eq!(reported[0].target(), Some("_self"));
    }

    #[test]
    fn state_fire_action_without_current_layer_component_rejects_import() {
        let error = RuntimeFile::from_fixture_records(vec![
            record("Backboard", Vec::new()),
            record("Artboard", Vec::new()),
            record(
                "Event",
                vec![property("Event", "parentId", FixtureValue::Uint(0))],
            ),
            record("StateMachine", Vec::new()),
            record(
                "StateMachineFireEvent",
                vec![property(
                    "StateMachineFireEvent",
                    "eventId",
                    FixtureValue::Uint(1),
                )],
            ),
        ])
        .expect_err("fire action without state/transition importer must reject");
        assert!(error.to_string().contains("StateMachineFireEvent"));
    }
}
