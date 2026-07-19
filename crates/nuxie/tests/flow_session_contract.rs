// Public flow-session contract tests deliberately exercise only exported API.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects
)]

use std::{path::PathBuf, sync::Arc};

use nuxie::{
    File,
    flow_session::{
        FlowInstanceId, FlowInstanceRef, FlowNewInstance, FlowOperation, FlowScalarValue,
        FlowSession, FlowSessionConfig, FlowSessionErrorKind, FlowStateBatch, FlowStateMutation,
        FlowValue, FlowValueArena, FlowValueId, FlowValueType,
    },
};

fn external_fixture(name: &str) -> Vec<u8> {
    let path = PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR")
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
    )
    .join("tests/unit_tests/assets")
    .join(name);
    std::fs::read(&path).expect("read external fixture")
}

#[test]
fn catalog_describes_enum_labels_and_referenced_view_model_schemas() {
    let enum_file =
        Arc::new(File::import(&external_fixture("data_binding_test.riv")).expect("import"));
    let (_, enum_bootstrap) =
        FlowSession::create(enum_file, FlowSessionConfig::default()).expect("create enum flow");
    let state = enum_bootstrap
        .catalog
        .schemas
        .iter()
        .find(|schema| schema.name == "vm2")
        .and_then(|schema| {
            schema
                .properties
                .iter()
                .find(|property| property.name == "state")
        })
        .expect("state enum schema property");
    assert_eq!(state.value_type, FlowValueType::Enum);
    assert_eq!(
        state.enum_labels,
        ["state-red", "state-green", "state-blue"]
    );
    assert_eq!(state.referenced_schema_name, None);

    let reference_file =
        Arc::new(File::import(&external_fixture("replace_view_model.riv")).expect("import"));
    let (_, reference_bootstrap) = FlowSession::create(
        reference_file,
        FlowSessionConfig {
            artboard_name: Some("Artboard".to_owned()),
            player_name: None,
        },
    )
    .expect("create reference flow");
    let child = reference_bootstrap
        .catalog
        .schemas
        .iter()
        .find(|schema| schema.name == "Main")
        .and_then(|schema| {
            schema
                .properties
                .iter()
                .find(|property| property.name == "child")
        })
        .expect("child view-model schema property");
    assert_eq!(child.value_type, FlowValueType::ViewModel);
    assert!(child.enum_labels.is_empty());
    assert_eq!(child.referenced_schema_name.as_deref(), Some("Child"));
}

fn root_property<'a>(arena: &'a FlowValueArena, property_name: &str) -> &'a FlowValue {
    let (_, root) = arena.roots.first().expect("root value");
    let root = arena
        .nodes
        .iter()
        .find(|node| node.id == *root)
        .expect("root node");
    let FlowValue::ViewModel(properties) = &root.value else {
        panic!("root must be a view model")
    };
    let (_, value) = properties
        .iter()
        .find(|(name, _)| name == property_name)
        .expect("named property");
    &arena
        .nodes
        .iter()
        .find(|node| node.id == *value)
        .expect("property node")
        .value
}

fn property_id(
    arena: &FlowValueArena,
    instance_id: FlowInstanceId,
    property_name: &str,
) -> FlowValueId {
    let root = arena
        .roots
        .iter()
        .find(|(id, _)| *id == instance_id)
        .map(|(_, root)| *root)
        .expect("instance root");
    let root = arena
        .nodes
        .iter()
        .find(|node| node.id == root)
        .expect("root node");
    let FlowValue::ViewModel(properties) = &root.value else {
        panic!("root must be a view model")
    };
    properties
        .iter()
        .find(|(name, _)| name == property_name)
        .map(|(_, id)| *id)
        .expect("named property")
}

fn string_property(
    arena: &FlowValueArena,
    instance_id: FlowInstanceId,
    property_name: &str,
) -> String {
    let id = property_id(arena, instance_id, property_name);
    let value = arena
        .nodes
        .iter()
        .find(|node| node.id == id)
        .expect("property node");
    let FlowValue::String(value) = &value.value else {
        panic!("property must be a string")
    };
    value.clone()
}

#[test]
fn list_index_properties_round_trip_without_becoming_enums() {
    let file = Arc::new(File::import(&external_fixture("component_list_2.riv")).expect("import"));
    let (mut session, bootstrap) = FlowSession::create(
        file,
        FlowSessionConfig {
            artboard_name: Some("Item".to_owned()),
            player_name: None,
        },
    )
    .expect("create item flow");
    let root = bootstrap.catalog.root_instance_id.expect("root instance");
    assert!(matches!(
        root_property(&bootstrap.values, "List Index"),
        FlowValue::ListIndex(0)
    ));

    let result = session
        .perform(FlowOperation::StateBatch(FlowStateBatch {
            host_mutation_id: Some(41),
            mutations: vec![FlowStateMutation::SetValue {
                instance: FlowInstanceRef::Existing(root),
                path: "List Index".to_owned(),
                value: FlowScalarValue::ListIndex(7),
            }],
            new_instances: Vec::new(),
        }))
        .expect("set list index");

    assert!(matches!(
        result.outputs.as_slice(),
        [nuxie::flow_session::FlowOutput {
            payload: nuxie::flow_session::FlowOutputPayload::StateChanged {
                value: Some(FlowScalarValue::ListIndex(7)),
                origin_mutation_id: Some(41),
                ..
            },
            ..
        }]
    ));
    let values = session
        .perform(FlowOperation::Query(nuxie::flow_session::FlowQuery::Values))
        .expect("query values")
        .values
        .expect("values");
    assert!(matches!(
        root_property(&values, "List Index"),
        FlowValue::ListIndex(7)
    ));
}

#[test]
fn nested_view_model_replacement_preserves_instance_identity_and_is_atomic() {
    let file = Arc::new(File::import(&external_fixture("replace_view_model.riv")).expect("import"));
    let (mut session, bootstrap) = FlowSession::create(
        file,
        FlowSessionConfig {
            artboard_name: Some("Artboard".to_owned()),
            player_name: None,
        },
    )
    .expect("create replacement flow");
    let root = bootstrap.catalog.root_instance_id.expect("root instance");

    let replacement = session
        .perform(FlowOperation::StateBatch(FlowStateBatch {
            host_mutation_id: Some(51),
            mutations: vec![FlowStateMutation::SetViewModel {
                instance: FlowInstanceRef::Existing(root),
                path: "child".to_owned(),
                value: FlowInstanceRef::New(1),
            }],
            new_instances: vec![FlowNewInstance {
                local_id: 1,
                schema_name: "Child".to_owned(),
                authored_instance_name: Some("child-2".to_owned()),
            }],
        }))
        .expect("replace child");
    let child = replacement.created_instances[0].id;
    assert!(matches!(
        replacement.outputs.as_slice(),
        [nuxie::flow_session::FlowOutput {
            payload: nuxie::flow_session::FlowOutputPayload::StateChanged {
                instance_id: Some(id),
                path,
                value: None,
                origin_mutation_id: Some(51),
            },
            ..
        }] if *id == root && path == "child"
    ));
    let values = session
        .perform(FlowOperation::Query(nuxie::flow_session::FlowQuery::Values))
        .expect("query values")
        .values
        .expect("values");
    let child_root = values
        .roots
        .iter()
        .find(|(id, _)| *id == child)
        .map(|(_, root)| *root)
        .expect("child root");
    assert_eq!(property_id(&values, root, "child"), child_root);

    session
        .perform(FlowOperation::StateBatch(FlowStateBatch {
            host_mutation_id: None,
            mutations: vec![FlowStateMutation::SetValue {
                instance: FlowInstanceRef::Existing(child),
                path: "label".to_owned(),
                value: FlowScalarValue::String("shared".to_owned()),
            }],
            new_instances: Vec::new(),
        }))
        .expect("mutate shared child");
    let values = session
        .perform(FlowOperation::Query(nuxie::flow_session::FlowQuery::Values))
        .expect("query shared values")
        .values
        .expect("values");
    assert_eq!(property_id(&values, root, "child"), child_root);
    assert_eq!(string_property(&values, child, "label"), "shared");

    let before_catalog = session
        .perform(FlowOperation::Query(
            nuxie::flow_session::FlowQuery::Catalog,
        ))
        .expect("query catalog")
        .catalog
        .expect("catalog");
    let error = session
        .perform(FlowOperation::StateBatch(FlowStateBatch {
            host_mutation_id: None,
            mutations: vec![
                FlowStateMutation::SetValue {
                    instance: FlowInstanceRef::Existing(child),
                    path: "label".to_owned(),
                    value: FlowScalarValue::String("must roll back".to_owned()),
                },
                FlowStateMutation::SetViewModel {
                    instance: FlowInstanceRef::Existing(root),
                    path: "child".to_owned(),
                    value: FlowInstanceRef::New(2),
                },
            ],
            new_instances: vec![FlowNewInstance {
                local_id: 2,
                schema_name: "Main".to_owned(),
                authored_instance_name: None,
            }],
        }))
        .expect_err("wrong-schema replacement must fail");
    assert_eq!(error.kind(), FlowSessionErrorKind::Conflict);
    let values = session
        .perform(FlowOperation::Query(nuxie::flow_session::FlowQuery::Values))
        .expect("query rolled-back values")
        .values
        .expect("values");
    assert_eq!(string_property(&values, child, "label"), "shared");
    let catalog = session
        .perform(FlowOperation::Query(
            nuxie::flow_session::FlowQuery::Catalog,
        ))
        .expect("query rolled-back catalog")
        .catalog
        .expect("catalog");
    assert_eq!(catalog.instances, before_catalog.instances);
}
