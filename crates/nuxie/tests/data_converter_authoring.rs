use anyhow::Result;
use nuxie::{
    ArtboardSpec, DataConverterEasing, DataConverterFormulaExpr, DataConverterFormulaFunction,
    DataConverterFormulaOperation, DataConverterFormulaRandomMode, DataConverterId,
    DataConverterOperation, DataConverterRangeFlags, DataConverterSpec, DataConverterStringPadSide,
    DataConverterStringTrimMode, EditReason, ExportedObjectKind, ExportedProperty, FillSpec,
    NodeSpec, Parent, RectangleSpec, Scene, ScriptAssetSpec, ShapeSpec, SolidColorSpec,
    ViewModelColorSource, ViewModelColorSpec, ViewModelDataBindingDirection, ViewModelInstanceSpec,
    ViewModelNumberSource, ViewModelNumberSpec, ViewModelScope, ViewModelSpec,
    ViewModelValueSource, props,
};

#[test]
fn authors_a_named_boolean_negate_converter_in_the_file_catalog() -> Result<()> {
    let mut scene = Scene::new();
    scene.edit(|tx| {
        tx.create_artboard(ArtboardSpec {
            name: "Root".into(),
            width: 100.0,
            height: 100.0,
        })?;
        tx.data_converters()
            .create(DataConverterSpec::BooleanNegate {
                name: "Not visible".into(),
            })?;
        Ok(())
    })?;

    let records = scene.export_records();
    let converter = records
        .records()
        .iter()
        .find(|record| record.kind == ExportedObjectKind::DataConverterBooleanNegate)
        .expect("authored converter record");
    assert_eq!(
        converter.properties,
        [ExportedProperty::DataConverterName("Not visible".into())]
    );
    Ok(())
}

#[test]
fn property_bearing_converter_records_keep_typed_values_and_flags() -> Result<()> {
    let mut scene = Scene::new();
    scene.edit(|tx| {
        tx.create_artboard(ArtboardSpec {
            name: "Root".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let mut converters = tx.data_converters();
        converters.create(DataConverterSpec::ToString {
            name: "Price".into(),
            decimals: 2,
            round: true,
            trim_zeros: true,
            commas: true,
            color_format: String::new(),
        })?;
        converters.create(DataConverterSpec::Rounder {
            name: "Rounded".into(),
            decimals: 3,
        })?;
        converters.create(DataConverterSpec::StringTrim {
            name: "Trim".into(),
            mode: DataConverterStringTrimMode::All,
        })?;
        converters.create(DataConverterSpec::StringPad {
            name: "Pad".into(),
            length: 6,
            text: "0".into(),
            side: DataConverterStringPadSide::Start,
        })?;
        converters.create(DataConverterSpec::OperationValue {
            name: "Triple".into(),
            operation: DataConverterOperation::Multiply,
            value: 3.0,
        })?;
        converters.create(DataConverterSpec::RangeMapper {
            name: "Progress".into(),
            min_input: 0.0,
            max_input: 10.0,
            min_output: 0.0,
            max_output: 100.0,
            flags: DataConverterRangeFlags {
                clamp_lower: true,
                clamp_upper: true,
                modulo: false,
                reverse: true,
            },
        })?;
        Ok(())
    })?;

    let records = scene.export_records().into_records();
    let to_string = records
        .iter()
        .find(|record| record.kind == ExportedObjectKind::DataConverterToString)
        .expect("to-string record");
    assert_eq!(
        to_string.properties,
        [
            ExportedProperty::DataConverterName("Price".into()),
            ExportedProperty::DataConverterToStringFlags(0b111),
            ExportedProperty::DataConverterToStringDecimals(2),
            ExportedProperty::DataConverterToStringColorFormat(String::new()),
        ]
    );
    let operation = records
        .iter()
        .find(|record| record.kind == ExportedObjectKind::DataConverterOperationValue)
        .expect("operation record");
    assert_eq!(
        operation.properties,
        [
            ExportedProperty::DataConverterName("Triple".into()),
            ExportedProperty::DataConverterOperationType(2),
            ExportedProperty::DataConverterOperationValue(3.0),
        ]
    );
    let range = records
        .iter()
        .find(|record| record.kind == ExportedObjectKind::DataConverterRangeMapper)
        .expect("range record");
    assert!(
        range
            .properties
            .contains(&ExportedProperty::DataConverterRangeFlags(0b1011))
    );
    assert!(
        range
            .properties
            .contains(&ExportedProperty::DataConverterRangeMaxOutput(100.0))
    );
    Ok(())
}

#[test]
fn primitive_converter_records_preserve_authored_order() -> Result<()> {
    let mut scene = Scene::new();
    scene.edit(|tx| {
        tx.create_artboard(ArtboardSpec {
            name: "Root".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let mut converters = tx.data_converters();
        converters.create(DataConverterSpec::ListToLength {
            name: "Count".into(),
        })?;
        converters.create(DataConverterSpec::ToNumber {
            name: "Numeric".into(),
        })?;
        converters.create(DataConverterSpec::StringRemoveZeros {
            name: "Compact".into(),
        })?;
        Ok(())
    })?;

    let kinds = scene
        .export_records()
        .into_records()
        .into_iter()
        .filter(|record| {
            matches!(
                record.kind,
                ExportedObjectKind::DataConverterListToLength
                    | ExportedObjectKind::DataConverterToNumber
                    | ExportedObjectKind::DataConverterStringRemoveZeros
            )
        })
        .map(|record| record.kind)
        .collect::<Vec<_>>();
    assert_eq!(
        kinds,
        [
            ExportedObjectKind::DataConverterListToLength,
            ExportedObjectKind::DataConverterToNumber,
            ExportedObjectKind::DataConverterStringRemoveZeros,
        ]
    );
    Ok(())
}

#[test]
fn converter_references_lower_to_exact_catalog_ordinals_and_paths() -> Result<()> {
    let mut scene = Scene::new();
    scene.edit(|tx| {
        tx.create_artboard(ArtboardSpec {
            name: "Root".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let (model, operand) = {
            let mut view_models = tx.view_models();
            let model = view_models.create(ViewModelSpec {
                scope: ViewModelScope::Local,
                name: "Runtime".into(),
            })?;
            let operand = view_models.create_number(
                model,
                ViewModelNumberSpec {
                    name: "operand".into(),
                },
            )?;
            (model, operand)
        };
        let mut converters = tx.data_converters();
        let operation = converters.create(DataConverterSpec::OperationViewModel {
            name: "Dynamic multiply".into(),
            operation: DataConverterOperation::Multiply,
            source: ViewModelNumberSource::direct(operand),
        })?;
        let list = converters.create(DataConverterSpec::NumberToList {
            name: "Rows".into(),
            view_model: model,
        })?;
        converters.create(DataConverterSpec::Group {
            name: "Pipeline".into(),
            items: vec![operation, list],
        })?;
        Ok(())
    })?;

    let records = scene.export_records().into_records();
    let operation = records
        .iter()
        .find(|record| record.kind == ExportedObjectKind::DataConverterOperationViewModel)
        .expect("operation-view-model record");
    assert!(
        operation
            .properties
            .contains(&ExportedProperty::DataConverterOperationViewModelSourcePath(vec![0, 0])),
        "{:?}",
        operation.properties
    );
    let number_to_list = records
        .iter()
        .find(|record| record.kind == ExportedObjectKind::DataConverterNumberToList)
        .expect("number-to-list record");
    assert!(
        number_to_list
            .properties
            .contains(&ExportedProperty::DataConverterNumberToListViewModelId(0))
    );
    let items = records
        .iter()
        .filter(|record| record.kind == ExportedObjectKind::DataConverterGroupItem)
        .map(|record| record.properties.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        items,
        [
            vec![ExportedProperty::DataConverterGroupItemConverterId(0)],
            vec![ExportedProperty::DataConverterGroupItemConverterId(1)],
        ]
    );
    Ok(())
}

#[test]
fn formula_ast_lowers_to_owned_rive_tokens() -> Result<()> {
    let mut scene = Scene::new();
    scene.edit(|tx| {
        tx.create_artboard(ArtboardSpec {
            name: "Root".into(),
            width: 100.0,
            height: 100.0,
        })?;
        tx.data_converters().create(DataConverterSpec::Formula {
            name: "Scale and clamp".into(),
            random_mode: DataConverterFormulaRandomMode::Cached,
            expression: DataConverterFormulaExpr::function(
                DataConverterFormulaFunction::Max,
                vec![
                    DataConverterFormulaExpr::binary(
                        DataConverterFormulaExpr::Input,
                        DataConverterFormulaOperation::Multiply,
                        DataConverterFormulaExpr::Value(2.0),
                    ),
                    DataConverterFormulaExpr::Value(1.0),
                ],
            ),
        })?;
        Ok(())
    })?;

    let records = scene.export_records().into_records();
    let kinds = records
        .iter()
        .filter(|record| {
            matches!(
                record.kind,
                ExportedObjectKind::DataConverterFormula
                    | ExportedObjectKind::FormulaTokenInput
                    | ExportedObjectKind::FormulaTokenValue
                    | ExportedObjectKind::FormulaTokenOperation
                    | ExportedObjectKind::FormulaTokenFunction
                    | ExportedObjectKind::FormulaTokenArgumentSeparator
                    | ExportedObjectKind::FormulaTokenParenthesisOpen
                    | ExportedObjectKind::FormulaTokenParenthesisClose
            )
        })
        .map(|record| record.kind)
        .collect::<Vec<_>>();
    assert_eq!(
        kinds.first(),
        Some(&ExportedObjectKind::DataConverterFormula)
    );
    assert!(kinds.contains(&ExportedObjectKind::FormulaTokenFunction));
    assert!(kinds.contains(&ExportedObjectKind::FormulaTokenOperation));
    assert!(kinds.contains(&ExportedObjectKind::FormulaTokenArgumentSeparator));
    assert!(kinds.contains(&ExportedObjectKind::FormulaTokenParenthesisClose));
    Ok(())
}

#[test]
fn interpolator_lowers_duration_and_owned_cubic_easing() -> Result<()> {
    let mut scene = Scene::new();
    scene.edit(|tx| {
        tx.create_artboard(ArtboardSpec {
            name: "Root".into(),
            width: 100.0,
            height: 100.0,
        })?;
        tx.data_converters()
            .create(DataConverterSpec::Interpolator {
                name: "Spring-like".into(),
                duration_seconds: 0.25,
                easing: DataConverterEasing::Cubic {
                    x1: 0.42,
                    y1: 0.0,
                    x2: 0.58,
                    y2: 1.0,
                },
            })?;
        Ok(())
    })?;

    let records = scene.export_records().into_records();
    let ease = records
        .iter()
        .find(|record| record.kind == ExportedObjectKind::CubicEaseInterpolator)
        .expect("file-global cubic easing record");
    assert_eq!(
        ease.properties,
        [
            ExportedProperty::CubicEaseX1(0.42),
            ExportedProperty::CubicEaseY1(0.0),
            ExportedProperty::CubicEaseX2(0.58),
            ExportedProperty::CubicEaseY2(1.0),
        ]
    );
    let converter = records
        .iter()
        .find(|record| record.kind == ExportedObjectKind::DataConverterInterpolator)
        .expect("interpolator converter record");
    assert_eq!(
        converter.properties,
        [
            ExportedProperty::DataConverterName("Spring-like".into()),
            ExportedProperty::DataConverterInterpolatorDuration(0.25),
            ExportedProperty::DataConverterInterpolatorInterpolationType(2),
            ExportedProperty::DataConverterInterpolatorId(0),
        ]
    );
    Ok(())
}

#[test]
fn scripted_converter_references_the_canonical_script_asset_ordinal() -> Result<()> {
    let mut scene = Scene::new();
    scene.edit(|tx| {
        tx.create_artboard(ArtboardSpec {
            name: "Root".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let script = tx.create_script_asset(ScriptAssetSpec {
            name: "converter".into(),
            is_module: false,
            bytes: vec![0x1b, b'L', b'u', b'a', b'u'],
        })?;
        tx.data_converters().create(DataConverterSpec::Scripted {
            name: "Project converter".into(),
            script,
        })?;
        Ok(())
    })?;

    let records = scene.export_records().into_records();
    let converter = records
        .iter()
        .find(|record| record.kind == ExportedObjectKind::ScriptedDataConverter)
        .expect("scripted converter record");
    assert_eq!(
        converter.properties,
        [
            ExportedProperty::DataConverterName("Project converter".into()),
            ExportedProperty::ScriptedDataConverterScriptAssetId(0),
        ]
    );
    Ok(())
}

#[test]
fn converter_aware_bind_uses_typed_source_and_stable_converter_identity() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, shape, defaults, source_opacity), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Root".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Card".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let converter = tx
            .data_converters()
            .create(DataConverterSpec::OperationValue {
                name: "Offset".into(),
                operation: DataConverterOperation::Add,
                value: 0.25,
            })?;
        let mut view_models = tx.view_models();
        let model = view_models.create(ViewModelSpec {
            scope: ViewModelScope::Local,
            name: "State".into(),
        })?;
        let source_opacity = view_models.create_number(
            model,
            ViewModelNumberSpec {
                name: "sourceOpacity".into(),
            },
        )?;
        let defaults = view_models.create_instance(
            model,
            ViewModelInstanceSpec {
                name: Some("Defaults".into()),
            },
        )?;
        view_models.set_number(defaults, source_opacity, 0.25)?;
        view_models.set_artboard_default(artboard, defaults)?;
        view_models.bind_opacity_with_converter(
            shape,
            ViewModelValueSource::Number(ViewModelNumberSource::direct(source_opacity)),
            converter,
            ViewModelDataBindingDirection::TwoWay,
        )?;
        Ok((artboard, shape, defaults, source_opacity))
    })?;

    let bind = scene
        .export_records()
        .into_records()
        .into_iter()
        .find(|record| {
            record.kind == ExportedObjectKind::DataBindContext
                && record
                    .properties
                    .contains(&ExportedProperty::DataBindPropertyKey(18))
        })
        .expect("converter-aware opacity bind");
    assert_eq!(
        bind.properties,
        [
            ExportedProperty::DataBindPropertyKey(18),
            ExportedProperty::DataBindFlags((1 << 1) | (1 << 3)),
            ExportedProperty::DataBindSourcePath(vec![0, 0]),
            ExportedProperty::DataBindConverterId(0),
        ]
    );

    let cold = scene.instantiate(artboard)?;
    let opacity = scene.cursor(cold, shape, nuxie::props::WORLD_OPACITY)?;
    let source_value = scene.vm_cursor(cold, defaults, source_opacity)?;
    let mut events = Vec::new();
    scene.frame().advance(cold, 0.0, &mut events);
    assert_eq!(scene.frame().get(opacity)?, 0.5);
    assert_eq!(scene.frame().get_vm(source_value)?, 0.25);

    assert!(scene.frame().set_vm(source_value, 0.5)?);
    assert!(scene.frame().advance(cold, 0.0, &mut events));
    assert_eq!(scene.frame().get(opacity)?, 0.75);

    scene.frame().set(opacity, 0.25)?;
    assert!(scene.frame().advance(cold, 0.0, &mut events));
    assert_eq!(
        scene.frame().get_vm(source_value)?,
        0.0,
        "TwoWay reverse execution applies the inverse operation converter"
    );

    let second_cold = scene.instantiate(artboard)?;
    let second_opacity = scene.cursor(second_cold, shape, nuxie::props::WORLD_OPACITY)?;
    scene.frame().advance(second_cold, 0.0, &mut events);
    assert_eq!(scene.frame().get(second_opacity)?, 0.5);
    Ok(())
}

#[test]
fn converter_aware_property_binds_reject_known_output_mismatches_atomically() -> Result<()> {
    let mut scene = Scene::new();
    let ((rectangle, color, number, tint, to_number, boolean_negate, string_group), _) = scene
        .edit(|tx| {
            let artboard = tx.create_artboard(ArtboardSpec {
                name: "Typed converter targets".into(),
                width: 100.0,
                height: 100.0,
            })?;
            let shape = tx.create(
                Parent::Artboard(artboard),
                NodeSpec::Shape(ShapeSpec {
                    name: "Card".into(),
                    x: 0.0,
                    y: 0.0,
                    opacity: 1.0,
                    rotation: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                }),
            )?;
            let rectangle = tx.create(
                Parent::Object(shape),
                NodeSpec::Rectangle(RectangleSpec::new("Bounds", 10.0, 10.0)),
            )?;
            let fill = tx.create(
                Parent::Object(shape),
                NodeSpec::Fill(FillSpec {
                    name: "Fill".into(),
                }),
            )?;
            let color = tx.create(
                Parent::Object(fill),
                NodeSpec::SolidColor(SolidColorSpec {
                    name: "Color".into(),
                    color: 0xff00_0000,
                }),
            )?;
            let (to_number, boolean_negate, string_group) = {
                let mut converters = tx.data_converters();
                let to_number = converters.create(DataConverterSpec::ToNumber {
                    name: "To number".into(),
                })?;
                let boolean_negate = converters.create(DataConverterSpec::BooleanNegate {
                    name: "Boolean negate".into(),
                })?;
                let to_string = converters.create(DataConverterSpec::ToString {
                    name: "To string".into(),
                    decimals: 0,
                    round: false,
                    trim_zeros: false,
                    commas: false,
                    color_format: String::new(),
                })?;
                let interpolator = converters.create(DataConverterSpec::Interpolator {
                    name: "Input-preserving tail".into(),
                    duration_seconds: 0.1,
                    easing: DataConverterEasing::Linear,
                })?;
                let string_group = converters.create(DataConverterSpec::Group {
                    name: "String then input".into(),
                    items: vec![to_string, interpolator],
                })?;
                (to_number, boolean_negate, string_group)
            };
            let mut view_models = tx.view_models();
            let model = view_models.create(ViewModelSpec {
                scope: ViewModelScope::Local,
                name: "Values".into(),
            })?;
            let number = view_models.create_number(
                model,
                ViewModelNumberSpec {
                    name: "number".into(),
                },
            )?;
            let tint = view_models.create_color(
                model,
                ViewModelColorSpec {
                    name: "tint".into(),
                },
            )?;
            let defaults = view_models.create_instance(
                model,
                ViewModelInstanceSpec {
                    name: Some("Defaults".into()),
                },
            )?;
            view_models.set_number(defaults, number, 10.0)?;
            view_models.set_color(defaults, tint, 0xff12_3456)?;
            view_models.set_artboard_default(artboard, defaults)?;
            Ok((
                rectangle,
                color,
                number,
                tint,
                to_number,
                boolean_negate,
                string_group,
            ))
        })?;

    let before = scene.export_records();
    let epoch = scene.epoch();
    let rejected = [
        (
            scene.edit(|tx| {
                tx.view_models().bind_color_with_converter(
                    color,
                    props::COLOR_VALUE,
                    ViewModelValueSource::Color(ViewModelColorSource::direct(tint)),
                    to_number,
                    ViewModelDataBindingDirection::ToTarget,
                )?;
                Ok(())
            }),
            ("colorValue", "color", "number"),
        ),
        (
            scene.edit(|tx| {
                tx.view_models().bind_number_with_converter(
                    rectangle,
                    props::PATH_WIDTH,
                    ViewModelValueSource::Number(ViewModelNumberSource::direct(number)),
                    boolean_negate,
                    ViewModelDataBindingDirection::ToTarget,
                )?;
                Ok(())
            }),
            ("width", "number", "boolean"),
        ),
        (
            scene.edit(|tx| {
                tx.view_models().bind_number_with_converter(
                    rectangle,
                    props::PATH_WIDTH,
                    ViewModelValueSource::Number(ViewModelNumberSource::direct(number)),
                    string_group,
                    ViewModelDataBindingDirection::ToTarget,
                )?;
                Ok(())
            }),
            ("width", "number", "string"),
        ),
    ];
    for (rejected, (property, expected, actual)) in rejected {
        assert!(matches!(
            &rejected.unwrap_err().diagnostic().reason,
            EditReason::DataConverterOutputMismatch {
                property: observed_property,
                expected: observed_expected,
                actual: observed_actual,
            } if *observed_property == property
                && *observed_expected == expected
                && *observed_actual == actual
        ));
    }
    assert_eq!(scene.epoch(), epoch);
    assert_eq!(scene.export_records(), before);
    Ok(())
}

#[test]
fn removal_rejects_live_references_atomically_then_compacts_the_catalog() -> Result<()> {
    let mut scene = Scene::new();
    let (leaf, group) = scene
        .edit(|tx| {
            tx.create_artboard(ArtboardSpec {
                name: "Root".into(),
                width: 100.0,
                height: 100.0,
            })?;
            let mut converters = tx.data_converters();
            let leaf = converters.create(DataConverterSpec::ToNumber {
                name: "Leaf".into(),
            })?;
            let group = converters.create(DataConverterSpec::Group {
                name: "Pipeline".into(),
                items: vec![leaf],
            })?;
            Ok((leaf, group))
        })?
        .0;
    let before = scene.export_records();

    assert!(scene.edit(|tx| tx.data_converters().remove(leaf)).is_err());
    assert_eq!(scene.export_records(), before, "failed removal is atomic");

    // Editor ownership tables retain the ordinary object identity alongside
    // other authored records. Reconstituting the typed id must still travel
    // through the converter transaction's reference validation.
    scene.edit(|tx| {
        tx.data_converters()
            .remove(DataConverterId::from_object_id(group.object_id()))
    })?;
    scene.edit(|tx| {
        tx.data_converters()
            .remove(DataConverterId::from_object_id(leaf.object_id()))
    })?;
    assert!(!scene.export_records().records().iter().any(|record| {
        matches!(
            record.kind,
            ExportedObjectKind::DataConverterToNumber
                | ExportedObjectKind::DataConverterGroup
                | ExportedObjectKind::DataConverterGroupItem
        )
    }));
    Ok(())
}
