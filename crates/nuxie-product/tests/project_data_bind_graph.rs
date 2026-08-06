use std::sync::Arc;

use nuxie::{OwnedArtboardInstance, host_interfaces::RuntimeFile};
use nuxie_binary::{AuthoringProperty, AuthoringRecord, AuthoringValue};
use nuxie_product::project_data::{
    ProjectDataConverterCatalog, ProjectDataConverterDefinition, ProjectDataConverterEasing,
    ProjectDataConverterKind, ProjectDataConverterOutputType, ProjectDataConverterSpec,
};

fn property(type_name: &str, property_name: &str, value: AuthoringValue) -> AuthoringProperty {
    let definition =
        nuxie_schema::definition_by_name(type_name).expect("fixture record type exists");
    let key = std::iter::once(definition.name)
        .chain(definition.ancestors.iter().copied())
        .filter_map(nuxie_schema::definition_by_name)
        .flat_map(|owner| owner.properties)
        .find(|property| property.name == property_name)
        .unwrap_or_else(|| panic!("fixture property exists: {type_name}.{property_name}"))
        .key
        .int;
    AuthoringProperty { key, value }
}

fn record(type_name: &str, properties: Vec<(&str, AuthoringValue)>) -> AuthoringRecord {
    AuthoringRecord {
        type_key: nuxie_schema::definition_by_name(type_name)
            .expect("fixture record type exists")
            .type_key
            .int,
        properties: properties
            .into_iter()
            .map(|(name, value)| property(type_name, name, value))
            .collect(),
    }
}

fn interpolation_payload() -> Vec<u8> {
    ProjectDataConverterCatalog::compile([ProjectDataConverterDefinition {
        id: "interpolate".to_owned(),
        spec: ProjectDataConverterSpec {
            output_type: Some(ProjectDataConverterOutputType::Number),
            kind: ProjectDataConverterKind::Interpolate {
                duration_ms: 100.0,
                easing: ProjectDataConverterEasing::Linear,
            },
        },
    }])
    .expect("valid ProjectData catalog")
    .encode_program("interpolate")
    .expect("ProjectData program encodes")
}

fn product_data_bind_artifact() -> RuntimeFile {
    let node_x_key = nuxie_schema::definition_by_name("Node")
        .expect("Node fixture type exists")
        .properties
        .iter()
        .find(|property| property.name == "x")
        .expect("Node.x exists")
        .key
        .int;

    RuntimeFile::from_authoring_records(vec![
        record("Backboard", vec![]),
        record("ScriptAsset", vec![("assetId", AuthoringValue::Uint(0))]),
        record(
            "FileAssetContents",
            vec![("bytes", AuthoringValue::Bytes(interpolation_payload()))],
        ),
        record(
            "ScriptedDataConverter",
            vec![("scriptAssetId", AuthoringValue::Uint(0))],
        ),
        record(
            "ViewModel",
            vec![("name", AuthoringValue::String("Project data".into()))],
        ),
        record(
            "ViewModelPropertyNumber",
            vec![("name", AuthoringValue::String("position".into()))],
        ),
        record(
            "ViewModelInstance",
            vec![
                ("name", AuthoringValue::String("Defaults".into())),
                ("viewModelId", AuthoringValue::Uint(0)),
            ],
        ),
        record(
            "ViewModelInstanceNumber",
            vec![
                ("viewModelPropertyId", AuthoringValue::Uint(0)),
                ("propertyValue", AuthoringValue::Double(0.0)),
            ],
        ),
        record("Artboard", vec![("viewModelId", AuthoringValue::Uint(0))]),
        record(
            "Shape",
            vec![
                ("parentId", AuthoringValue::Uint(0)),
                ("x", AuthoringValue::Double(0.0)),
            ],
        ),
        record(
            "DataBindContext",
            vec![
                ("propertyKey", AuthoringValue::Uint(u64::from(node_x_key))),
                ("sourcePathIds", AuthoringValue::Bytes(vec![0, 0])),
                ("converterId", AuthoringValue::Uint(0)),
            ],
        ),
    ])
    .expect("product ProjectData fixture imports")
}

fn shape_x(instance: &mut OwnedArtboardInstance) -> f32 {
    instance
        .world_transform(1)
        .expect("shape world transform")
        .0[4]
}

#[test]
fn encoded_project_data_program_runs_in_live_bind_graph_with_retained_state() {
    let runtime = product_data_bind_artifact();
    let payload = runtime
        .scripting_file_assets_with_contents()
        .into_iter()
        .find_map(|asset| asset.contents)
        .expect("encoded ProjectData asset contents");
    assert!(payload.starts_with(b"NUXPCV1\0"));

    let file = Arc::new(
        nuxie_product::file_from_locally_authored_runtime(runtime)
            .expect("product import installs the ProjectData adapter"),
    );
    let mut instance = OwnedArtboardInstance::instantiate(file, 0).expect("artboard instantiates");
    let mut view_model = instance
        .instantiate_view_model_instance(0)
        .expect("authored view model instantiates");

    let _ = instance.bind_view_model(&view_model);
    assert!(instance.owned_view_model_context().is_some());
    instance.advance(0.0);
    assert_eq!(shape_x(&mut instance), 0.0);

    assert!(view_model.set_number("position", 10.0));
    instance.advance(0.0);
    assert_eq!(shape_x(&mut instance), 0.0);
    instance.advance(0.05);
    assert_eq!(shape_x(&mut instance), 5.0);
    instance.advance(0.05);
    assert_eq!(shape_x(&mut instance), 10.0);
}
