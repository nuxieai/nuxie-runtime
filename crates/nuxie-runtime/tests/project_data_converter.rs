use std::collections::BTreeMap;

use nuxie_runtime::{
    ProjectDataConverterCatalog, ProjectDataConverterCompileError, ProjectDataConverterContext,
    ProjectDataConverterDefinition, ProjectDataConverterEasing, ProjectDataConverterFormat,
    ProjectDataConverterKind, ProjectDataConverterMathOperation, ProjectDataConverterOutputType,
    ProjectDataConverterProgram, ProjectDataConverterProgramError, ProjectDataConverterResolver,
    ProjectDataConverterRuntimeError, ProjectDataConverterSpec, ProjectDataConverterState,
    ProjectDataConverterStringPadSide, ProjectDataConverterValidationRule, ProjectDataValue,
    ProjectDataValuePath,
};

fn definition(id: &str, kind: ProjectDataConverterKind) -> ProjectDataConverterDefinition {
    ProjectDataConverterDefinition {
        id: id.to_owned(),
        spec: ProjectDataConverterSpec {
            output_type: None,
            kind,
        },
    }
}

fn convert(
    catalog: &ProjectDataConverterCatalog,
    id: &str,
    state: &mut ProjectDataConverterState,
    value: ProjectDataValue,
    now_ms: Option<f64>,
) -> ProjectDataValue {
    let mut context = ProjectDataConverterContext::new();
    context.now_ms = now_ms;
    catalog
        .convert(id, state, value, &mut context)
        .expect("known compiled converter")
}

#[derive(Default)]
struct NumberedBlankResolver {
    next: u64,
}

impl ProjectDataConverterResolver for NumberedBlankResolver {
    fn resolve_value(&mut self, _path: &ProjectDataValuePath) -> Option<ProjectDataValue> {
        None
    }

    fn create_blank_view_model_instance(
        &mut self,
        _view_model_id: &str,
    ) -> Option<ProjectDataValue> {
        let value = ProjectDataValue::ListIndex(self.next);
        self.next += 1;
        Some(value)
    }
}

fn convert_with_resolver(
    catalog: &ProjectDataConverterCatalog,
    id: &str,
    state: &mut ProjectDataConverterState,
    resolver: &mut dyn ProjectDataConverterResolver,
    value: ProjectDataValue,
    now_ms: f64,
) -> ProjectDataValue {
    let mut context = ProjectDataConverterContext {
        now_ms: Some(now_ms),
        resolver: Some(resolver),
    };
    catalog
        .convert(id, state, value, &mut context)
        .expect("known compiled converter")
}

#[test]
fn grouped_interpolate_and_number_to_list_keep_independent_state() {
    let catalog = ProjectDataConverterCatalog::compile([
        definition(
            "interpolate",
            ProjectDataConverterKind::Interpolate {
                duration_ms: 100.0,
                easing: ProjectDataConverterEasing::Linear,
            },
        ),
        definition(
            "items",
            ProjectDataConverterKind::NumberToList {
                view_model_id: "Item".to_owned(),
            },
        ),
        definition(
            "group",
            ProjectDataConverterKind::Group {
                items: vec!["interpolate".to_owned(), "items".to_owned()],
            },
        ),
    ])
    .expect("valid stateful group");
    let mut state = ProjectDataConverterState::default();
    let mut resolver = NumberedBlankResolver::default();

    assert_eq!(
        convert_with_resolver(
            &catalog,
            "group",
            &mut state,
            &mut resolver,
            ProjectDataValue::Number(0.0),
            0.0,
        ),
        ProjectDataValue::List(vec![])
    );
    assert_eq!(
        convert_with_resolver(
            &catalog,
            "group",
            &mut state,
            &mut resolver,
            ProjectDataValue::Number(4.0),
            0.0,
        ),
        ProjectDataValue::List(vec![])
    );
    assert!(
        state.is_interpolating(),
        "the group still needs another frame"
    );
    assert_eq!(
        convert_with_resolver(
            &catalog,
            "group",
            &mut state,
            &mut resolver,
            ProjectDataValue::Number(4.0),
            50.0,
        ),
        ProjectDataValue::List(vec![
            ProjectDataValue::ListIndex(0),
            ProjectDataValue::ListIndex(1),
        ])
    );
    assert_eq!(
        convert_with_resolver(
            &catalog,
            "group",
            &mut state,
            &mut resolver,
            ProjectDataValue::Number(4.0),
            100.0,
        ),
        ProjectDataValue::List(vec![
            ProjectDataValue::ListIndex(0),
            ProjectDataValue::ListIndex(1),
            ProjectDataValue::ListIndex(2),
            ProjectDataValue::ListIndex(3),
        ])
    );
    assert!(!state.is_interpolating(), "the interpolation has converged");
}

#[test]
fn interpolation_settles_on_the_exact_target_value() {
    let catalog = ProjectDataConverterCatalog::compile([definition(
        "interpolate",
        ProjectDataConverterKind::Interpolate {
            duration_ms: 100.0,
            easing: ProjectDataConverterEasing::Linear,
        },
    )])
    .expect("valid interpolator");
    let mut state = ProjectDataConverterState::default();
    let from = -822.978_487_459_879_1;
    let to = 531.707_664_809_898_2;

    assert_eq!(
        convert(
            &catalog,
            "interpolate",
            &mut state,
            ProjectDataValue::Number(from),
            Some(0.0),
        ),
        ProjectDataValue::Number(from)
    );
    convert(
        &catalog,
        "interpolate",
        &mut state,
        ProjectDataValue::Number(to),
        Some(0.0),
    );

    assert_eq!(
        convert(
            &catalog,
            "interpolate",
            &mut state,
            ProjectDataValue::Number(to),
            Some(100.0),
        ),
        ProjectDataValue::Number(to)
    );
    assert!(!state.is_interpolating());
}

#[test]
fn interpolation_stops_scheduling_when_the_value_is_no_longer_numeric() {
    let catalog = ProjectDataConverterCatalog::compile([definition(
        "interpolate",
        ProjectDataConverterKind::Interpolate {
            duration_ms: 100.0,
            easing: ProjectDataConverterEasing::Linear,
        },
    )])
    .expect("valid interpolator");
    let mut state = ProjectDataConverterState::default();

    for input in [0.0, 10.0] {
        convert(
            &catalog,
            "interpolate",
            &mut state,
            ProjectDataValue::Number(input),
            Some(0.0),
        );
    }
    assert!(state.is_interpolating());

    assert_eq!(
        convert(
            &catalog,
            "interpolate",
            &mut state,
            ProjectDataValue::String("not numeric".to_owned()),
            Some(50.0),
        ),
        ProjectDataValue::String("not numeric".to_owned())
    );
    assert!(!state.is_interpolating());
}

#[test]
fn interpolation_stops_scheduling_without_a_finite_clock() {
    let catalog = ProjectDataConverterCatalog::compile([definition(
        "interpolate",
        ProjectDataConverterKind::Interpolate {
            duration_ms: 100.0,
            easing: ProjectDataConverterEasing::Linear,
        },
    )])
    .expect("valid interpolator");

    for now_ms in [None, Some(f64::NAN)] {
        let mut state = ProjectDataConverterState::default();
        for input in [0.0, 10.0] {
            convert(
                &catalog,
                "interpolate",
                &mut state,
                ProjectDataValue::Number(input),
                Some(0.0),
            );
        }
        assert!(state.is_interpolating());

        assert_eq!(
            convert(
                &catalog,
                "interpolate",
                &mut state,
                ProjectDataValue::Number(10.0),
                now_ms,
            ),
            ProjectDataValue::Number(10.0)
        );
        assert!(!state.is_interpolating());
    }
}

#[test]
fn grouped_interpolator_definitions_keep_independent_state_and_scheduling() {
    let catalog = ProjectDataConverterCatalog::compile([
        definition(
            "first",
            ProjectDataConverterKind::Interpolate {
                duration_ms: 100.0,
                easing: ProjectDataConverterEasing::Linear,
            },
        ),
        definition(
            "second",
            ProjectDataConverterKind::Interpolate {
                duration_ms: 100.0,
                easing: ProjectDataConverterEasing::Linear,
            },
        ),
        definition(
            "group",
            ProjectDataConverterKind::Group {
                items: vec!["first".to_owned(), "second".to_owned()],
            },
        ),
    ])
    .expect("valid interpolator group");
    let mut state = ProjectDataConverterState::default();

    assert_eq!(
        convert(
            &catalog,
            "group",
            &mut state,
            ProjectDataValue::Number(0.0),
            Some(0.0),
        ),
        ProjectDataValue::Number(0.0)
    );
    assert_eq!(
        convert(
            &catalog,
            "group",
            &mut state,
            ProjectDataValue::Number(10.0),
            Some(0.0),
        ),
        ProjectDataValue::Number(0.0)
    );
    assert!(
        state.is_interpolating(),
        "the first stage is still advancing"
    );
    assert_eq!(
        convert(
            &catalog,
            "group",
            &mut state,
            ProjectDataValue::Number(10.0),
            Some(50.0),
        ),
        ProjectDataValue::Number(0.0)
    );
    assert!(
        state.is_interpolating(),
        "both stages request continued frames"
    );
    assert_eq!(
        convert(
            &catalog,
            "group",
            &mut state,
            ProjectDataValue::Number(10.0),
            Some(150.0),
        ),
        ProjectDataValue::Number(0.0)
    );
    assert!(
        state.is_interpolating(),
        "the second stage has not converged yet"
    );
    assert_eq!(
        convert(
            &catalog,
            "group",
            &mut state,
            ProjectDataValue::Number(10.0),
            Some(200.0),
        ),
        ProjectDataValue::Number(5.0)
    );
    assert!(
        state.is_interpolating(),
        "the second stage keeps scheduling frames"
    );
    assert_eq!(
        convert(
            &catalog,
            "group",
            &mut state,
            ProjectDataValue::Number(10.0),
            Some(250.0),
        ),
        ProjectDataValue::Number(10.0)
    );
    assert!(!state.is_interpolating(), "both stages have converged");
}

#[test]
fn repeated_stateful_group_positions_do_not_share_a_cache() {
    let catalog = ProjectDataConverterCatalog::compile([
        definition(
            "shared",
            ProjectDataConverterKind::Interpolate {
                duration_ms: 100.0,
                easing: ProjectDataConverterEasing::Linear,
            },
        ),
        definition(
            "group",
            ProjectDataConverterKind::Group {
                items: vec!["shared".to_owned(), "shared".to_owned()],
            },
        ),
    ])
    .expect("valid repeated group item");
    let mut state = ProjectDataConverterState::default();

    assert_eq!(
        convert(
            &catalog,
            "group",
            &mut state,
            ProjectDataValue::Number(0.0),
            Some(0.0),
        ),
        ProjectDataValue::Number(0.0)
    );
    assert_eq!(
        convert(
            &catalog,
            "group",
            &mut state,
            ProjectDataValue::Number(10.0),
            Some(0.0),
        ),
        ProjectDataValue::Number(0.0)
    );
    assert!(state.is_interpolating());
    assert_eq!(
        convert(
            &catalog,
            "group",
            &mut state,
            ProjectDataValue::Number(10.0),
            Some(150.0),
        ),
        ProjectDataValue::Number(0.0)
    );
    assert!(state.is_interpolating());
    assert_eq!(
        convert(
            &catalog,
            "group",
            &mut state,
            ProjectDataValue::Number(10.0),
            Some(200.0),
        ),
        ProjectDataValue::Number(5.0)
    );
}

#[test]
fn changing_the_binding_root_drops_dormant_converter_state() {
    let catalog = ProjectDataConverterCatalog::compile([
        definition(
            "interpolate",
            ProjectDataConverterKind::Interpolate {
                duration_ms: 100.0,
                easing: ProjectDataConverterEasing::Linear,
            },
        ),
        definition("number", ProjectDataConverterKind::ToNumber),
    ])
    .expect("valid catalog");
    let mut state = ProjectDataConverterState::default();

    for input in [0.0, 10.0] {
        convert(
            &catalog,
            "interpolate",
            &mut state,
            ProjectDataValue::Number(input),
            Some(0.0),
        );
    }
    assert!(state.is_interpolating());

    convert(
        &catalog,
        "number",
        &mut state,
        ProjectDataValue::Number(10.0),
        Some(0.0),
    );
    assert!(
        !state.is_interpolating(),
        "state belongs to one binding root, so an abandoned root cannot keep scheduling frames"
    );
}

#[test]
fn interpolate_without_explicit_output_remains_untyped() {
    let catalog = ProjectDataConverterCatalog::compile([definition(
        "interpolate",
        ProjectDataConverterKind::Interpolate {
            duration_ms: 100.0,
            easing: ProjectDataConverterEasing::Linear,
        },
    )])
    .expect("valid interpolation converter");

    assert_eq!(
        catalog.output_type("interpolate").expect("known root"),
        None,
        "canonical TS leaves Interpolate untyped so values pass through"
    );
}

#[test]
fn group_infers_the_last_defined_output_when_trailing_items_are_untyped() {
    let catalog = ProjectDataConverterCatalog::compile([
        definition(
            "string",
            ProjectDataConverterKind::ToString {
                locale: None,
                decimals: None,
                trim_zeros: false,
                commas: false,
            },
        ),
        definition(
            "interpolate",
            ProjectDataConverterKind::Interpolate {
                duration_ms: 100.0,
                easing: ProjectDataConverterEasing::Linear,
            },
        ),
        definition(
            "group",
            ProjectDataConverterKind::Group {
                items: vec!["string".to_owned(), "interpolate".to_owned()],
            },
        ),
    ])
    .expect("valid group");

    assert_eq!(
        catalog.output_type("group").expect("known root"),
        Some(ProjectDataConverterOutputType::String),
        "canonical TS scans backward until it finds a defined child output"
    );
}

#[test]
fn rejects_formula_nesting_beyond_the_runtime_stack_budget() {
    let nesting = 65;
    let expression = format!("{}value{}", "(".repeat(nesting), ")".repeat(nesting));

    let error = ProjectDataConverterCatalog::compile([definition(
        "formula",
        ProjectDataConverterKind::Formula { expression },
    )])
    .expect_err("deeply nested formulas must fail before recursive evaluation");

    assert!(matches!(
        error,
        ProjectDataConverterCompileError::InvalidFormula {
            message: "formula nesting limit exceeded",
            ..
        }
    ));
}

#[test]
fn rejects_formula_ast_nodes_beyond_the_runtime_traversal_budget() {
    let expression = std::iter::repeat_n("value", 129)
        .collect::<Vec<_>>()
        .join(" + ");

    let error = ProjectDataConverterCatalog::compile([definition(
        "formula",
        ProjectDataConverterKind::Formula { expression },
    )])
    .expect_err("large formula trees must fail before recursive evaluation");

    assert!(matches!(
        error,
        ProjectDataConverterCompileError::InvalidFormula {
            message: "formula node limit exceeded",
            ..
        }
    ));
}

#[test]
fn rejects_converter_graphs_beyond_the_runtime_stack_budget() {
    let mut definitions = vec![definition("leaf", ProjectDataConverterKind::ToNumber)];
    let mut child = "leaf".to_owned();
    for index in (0..65).rev() {
        let id = format!("group-{index}");
        definitions.push(definition(
            &id,
            ProjectDataConverterKind::Group { items: vec![child] },
        ));
        child = id;
    }

    let error = ProjectDataConverterCatalog::compile(definitions.clone())
        .expect_err("deep converter graphs must fail before recursive metadata walks");

    assert!(matches!(
        error,
        ProjectDataConverterCompileError::GroupNestingTooDeep { maximum: 64, .. }
    ));

    let payload = serde_json::to_vec(&serde_json::json!({
        "root": child,
        "definitions": definitions,
        "runtime_view_models": {},
    }))
    .expect("program payload serializes");
    let mut bytes = b"NUXPCV1\0".to_vec();
    bytes.extend_from_slice(&payload);
    assert!(matches!(
        ProjectDataConverterProgram::decode(&bytes),
        Err(ProjectDataConverterProgramError::InvalidCatalog(
            ProjectDataConverterCompileError::GroupNestingTooDeep { maximum: 64, .. }
        ))
    ));
}

#[test]
fn converter_graph_depth_budget_preserves_cycle_diagnostics() {
    let error = ProjectDataConverterCatalog::compile([
        definition(
            "first",
            ProjectDataConverterKind::Group {
                items: vec!["second".to_owned()],
            },
        ),
        definition(
            "second",
            ProjectDataConverterKind::Group {
                items: vec!["first".to_owned()],
            },
        ),
    ])
    .expect_err("group cycles remain invalid independently of depth");

    assert!(matches!(
        error,
        ProjectDataConverterCompileError::GroupCycle { .. }
    ));
}

#[test]
fn chained_templates_reject_output_beyond_the_runtime_value_budget() {
    let catalog = ProjectDataConverterCatalog::compile([
        definition(
            "double",
            ProjectDataConverterKind::Template {
                template: "{{value}}{{value}}".to_owned(),
            },
        ),
        definition(
            "double-again",
            ProjectDataConverterKind::Template {
                template: "{{value}}{{value}}".to_owned(),
            },
        ),
        definition(
            "group",
            ProjectDataConverterKind::Group {
                items: vec!["double".to_owned(), "double-again".to_owned()],
            },
        ),
    ])
    .expect("valid template group");
    let mut state = ProjectDataConverterState::default();
    let mut context = ProjectDataConverterContext::new();

    let error = catalog
        .convert(
            "group",
            &mut state,
            ProjectDataValue::String("x".repeat(2 * 1024 * 1024)),
            &mut context,
        )
        .expect_err("each template stage must enforce the runtime value budget");

    assert!(matches!(
        error,
        ProjectDataConverterRuntimeError::ValueTooLarge { converter, .. }
            if converter == "double-again"
    ));
}

#[test]
fn templates_reject_oversized_replacement_values_before_cloning_them() {
    let catalog = ProjectDataConverterCatalog::compile([definition(
        "template",
        ProjectDataConverterKind::Template {
            template: "{{value}}".to_owned(),
        },
    )])
    .expect("valid template");
    let mut state = ProjectDataConverterState::default();
    let mut context = ProjectDataConverterContext::new();

    let error = catalog
        .convert(
            "template",
            &mut state,
            ProjectDataValue::String("x".repeat(4 * 1024 * 1024 + 1)),
            &mut context,
        )
        .expect_err("oversized values must fail before template substitution");

    assert!(matches!(
        error,
        ProjectDataConverterRuntimeError::ValueTooLarge { converter, .. }
            if converter == "template"
    ));
}

#[test]
fn templates_reject_values_beyond_the_runtime_traversal_depth() {
    let catalog = ProjectDataConverterCatalog::compile([definition(
        "template",
        ProjectDataConverterKind::Template {
            template: "{{value}}".to_owned(),
        },
    )])
    .expect("valid template");
    let mut value = ProjectDataValue::String("leaf".to_owned());
    for _ in 0..65 {
        value = ProjectDataValue::List(vec![value]);
    }
    let mut state = ProjectDataConverterState::default();
    let mut context = ProjectDataConverterContext::new();

    let error = catalog
        .convert("template", &mut state, value, &mut context)
        .expect_err("recursive values must fail before exhausting the runtime stack");

    assert!(matches!(
        error,
        ProjectDataConverterRuntimeError::ValueTooComplex { converter }
            if converter == "template"
    ));
}

#[test]
fn pins_javascript_numeric_and_forward_output_coercion_semantics() {
    let catalog = ProjectDataConverterCatalog::compile([
        definition("number", ProjectDataConverterKind::ToNumber),
        definition(
            "round",
            ProjectDataConverterKind::Math {
                operation: ProjectDataConverterMathOperation::Round,
                value: None,
                value_path: None,
            },
        ),
        definition(
            "modulo",
            ProjectDataConverterKind::Math {
                operation: ProjectDataConverterMathOperation::Modulo,
                value: Some(10.0),
                value_path: None,
            },
        ),
        definition(
            "divide-zero",
            ProjectDataConverterKind::Math {
                operation: ProjectDataConverterMathOperation::Divide,
                value: Some(0.0),
                value_path: None,
            },
        ),
        definition(
            "sqrt-negative",
            ProjectDataConverterKind::Math {
                operation: ProjectDataConverterMathOperation::SquareRoot,
                value: None,
                value_path: None,
            },
        ),
        definition(
            "formula-modulo",
            ProjectDataConverterKind::Formula {
                expression: "value % 2".to_owned(),
            },
        ),
        definition(
            "formula-round",
            ProjectDataConverterKind::Formula {
                expression: "round(value)".to_owned(),
            },
        ),
        definition(
            "formula-nonfinite",
            ProjectDataConverterKind::Formula {
                expression: "value / 0".to_owned(),
            },
        ),
        ProjectDataConverterDefinition {
            id: "string-output".to_owned(),
            spec: ProjectDataConverterSpec {
                output_type: Some(ProjectDataConverterOutputType::String),
                kind: ProjectDataConverterKind::Math {
                    operation: ProjectDataConverterMathOperation::Add,
                    value: Some(2.0),
                    value_path: None,
                },
            },
        },
        ProjectDataConverterDefinition {
            id: "list-output".to_owned(),
            spec: ProjectDataConverterSpec {
                output_type: Some(ProjectDataConverterOutputType::List),
                kind: ProjectDataConverterKind::Template {
                    template: "{{value}}".to_owned(),
                },
            },
        },
        ProjectDataConverterDefinition {
            id: "object-output".to_owned(),
            spec: ProjectDataConverterSpec {
                output_type: Some(ProjectDataConverterOutputType::Object),
                kind: ProjectDataConverterKind::Template {
                    template: "{{value}}".to_owned(),
                },
            },
        },
    ])
    .expect("valid catalog");
    let mut state = ProjectDataConverterState::default();

    assert_eq!(
        convert(
            &catalog,
            "number",
            &mut state,
            ProjectDataValue::String("12px".to_owned()),
            None,
        ),
        ProjectDataValue::Number(0.0)
    );
    assert_eq!(
        convert(
            &catalog,
            "number",
            &mut state,
            ProjectDataValue::Boolean(true),
            None,
        ),
        ProjectDataValue::Number(1.0)
    );
    assert_eq!(
        convert(
            &catalog,
            "round",
            &mut state,
            ProjectDataValue::Number(-1.5),
            None,
        ),
        ProjectDataValue::Number(-1.0)
    );
    assert_eq!(
        convert(
            &catalog,
            "modulo",
            &mut state,
            ProjectDataValue::Number(-1.0),
            None,
        ),
        ProjectDataValue::Number(9.0)
    );
    for (id, input, expected) in [
        ("divide-zero", 1.0, 0.0),
        ("sqrt-negative", -1.0, 0.0),
        ("formula-modulo", -5.0, -1.0),
        ("formula-round", -1.5, -1.0),
        ("formula-nonfinite", 7.0, 7.0),
    ] {
        assert_eq!(
            convert(
                &catalog,
                id,
                &mut state,
                ProjectDataValue::Number(input),
                None,
            ),
            ProjectDataValue::Number(expected),
            "converter {id}"
        );
    }
    assert_eq!(
        convert(
            &catalog,
            "string-output",
            &mut state,
            ProjectDataValue::Number(3.0),
            None,
        ),
        ProjectDataValue::String("5".to_owned())
    );
    assert_eq!(
        convert(
            &catalog,
            "list-output",
            &mut state,
            ProjectDataValue::String("not-a-list".to_owned()),
            None,
        ),
        ProjectDataValue::List(Vec::new())
    );
    assert_eq!(
        convert(
            &catalog,
            "object-output",
            &mut state,
            ProjectDataValue::String("not-an-object".to_owned()),
            None,
        ),
        ProjectDataValue::Object(BTreeMap::new())
    );
}

#[test]
fn validates_collection_lengths_and_uses_exact_piecewise_ease_in_out() {
    let mut min_args = BTreeMap::new();
    min_args.insert("length".to_owned(), ProjectDataValue::Number(2.0));
    let catalog = ProjectDataConverterCatalog::compile([
        definition(
            "non-empty",
            ProjectDataConverterKind::Validate {
                rule: ProjectDataConverterValidationRule::NonEmpty,
                args: BTreeMap::new(),
                invert: false,
            },
        ),
        definition(
            "min-two",
            ProjectDataConverterKind::Validate {
                rule: ProjectDataConverterValidationRule::Min,
                args: min_args,
                invert: false,
            },
        ),
        definition(
            "ease",
            ProjectDataConverterKind::Interpolate {
                duration_ms: 100.0,
                easing: ProjectDataConverterEasing::EaseInOut,
            },
        ),
    ])
    .expect("valid catalog");
    let mut state = ProjectDataConverterState::default();

    assert_eq!(
        convert(
            &catalog,
            "non-empty",
            &mut state,
            ProjectDataValue::List(Vec::new()),
            None,
        ),
        ProjectDataValue::Boolean(false)
    );
    assert_eq!(
        convert(
            &catalog,
            "non-empty",
            &mut state,
            ProjectDataValue::Object(BTreeMap::from([(
                "value".to_owned(),
                ProjectDataValue::Boolean(true),
            )])),
            None,
        ),
        ProjectDataValue::Boolean(true)
    );
    assert_eq!(
        convert(
            &catalog,
            "min-two",
            &mut state,
            ProjectDataValue::List(vec![ProjectDataValue::Number(1.0)]),
            None,
        ),
        ProjectDataValue::Boolean(false)
    );

    state.clear();
    assert_eq!(
        convert(
            &catalog,
            "ease",
            &mut state,
            ProjectDataValue::Number(0.0),
            Some(0.0),
        ),
        ProjectDataValue::Number(0.0)
    );
    assert_eq!(
        convert(
            &catalog,
            "ease",
            &mut state,
            ProjectDataValue::Number(10.0),
            Some(0.0),
        ),
        ProjectDataValue::Number(0.0)
    );
    for (now, expected) in [(25.0, 1.25), (50.0, 5.0), (75.0, 8.75), (100.0, 10.0)] {
        assert_eq!(
            convert(
                &catalog,
                "ease",
                &mut state,
                ProjectDataValue::Number(10.0),
                Some(now),
            ),
            ProjectDataValue::Number(expected)
        );
    }
}

#[test]
fn supports_only_the_characterized_deterministic_intl_matrix() {
    let catalog = ProjectDataConverterCatalog::compile([
        definition(
            "number",
            ProjectDataConverterKind::Format {
                format: ProjectDataConverterFormat::Number,
                locale: Some("en-US".to_owned()),
                time_zone: None,
                options: BTreeMap::new(),
                trim_zeros: false,
                commas: true,
                decimals: Some(2),
            },
        ),
        definition(
            "currency",
            ProjectDataConverterKind::Format {
                format: ProjectDataConverterFormat::Currency,
                locale: Some("en-US".to_owned()),
                time_zone: None,
                options: BTreeMap::from([(
                    "currency".to_owned(),
                    ProjectDataValue::String("USD".to_owned()),
                )]),
                trim_zeros: false,
                commas: true,
                decimals: None,
            },
        ),
        definition(
            "date",
            ProjectDataConverterKind::Format {
                format: ProjectDataConverterFormat::Date,
                locale: Some("en-GB".to_owned()),
                time_zone: Some("UTC".to_owned()),
                options: BTreeMap::from([
                    (
                        "day".to_owned(),
                        ProjectDataValue::String("2-digit".to_owned()),
                    ),
                    (
                        "month".to_owned(),
                        ProjectDataValue::String("2-digit".to_owned()),
                    ),
                    (
                        "year".to_owned(),
                        ProjectDataValue::String("numeric".to_owned()),
                    ),
                ]),
                trim_zeros: false,
                commas: false,
                decimals: None,
            },
        ),
    ])
    .expect("characterized formats compile");
    let mut state = ProjectDataConverterState::default();
    assert_eq!(
        convert(
            &catalog,
            "number",
            &mut state,
            ProjectDataValue::Number(1234.5),
            None,
        ),
        ProjectDataValue::String("1,234.50".to_owned())
    );
    assert_eq!(
        convert(
            &catalog,
            "currency",
            &mut state,
            ProjectDataValue::Number(1234.5),
            None,
        ),
        ProjectDataValue::String("$1,234.50".to_owned())
    );
    assert_eq!(
        convert(
            &catalog,
            "date",
            &mut state,
            ProjectDataValue::Number(1_769_169_600_000.0),
            None,
        ),
        ProjectDataValue::String("23/01/2026".to_owned())
    );

    let error = ProjectDataConverterCatalog::compile([definition(
        "unsupported",
        ProjectDataConverterKind::Format {
            format: ProjectDataConverterFormat::Currency,
            locale: Some("de-DE".to_owned()),
            time_zone: None,
            options: BTreeMap::from([(
                "currency".to_owned(),
                ProjectDataValue::String("EUR".to_owned()),
            )]),
            trim_zeros: false,
            commas: true,
            decimals: None,
        },
    )])
    .expect_err("unsupported Intl combinations fail at mount");
    assert!(matches!(
        error,
        ProjectDataConverterCompileError::UnsupportedFormat { .. }
    ));
}

#[test]
fn versioned_program_round_trips_a_group_without_becoming_executable_script() {
    let catalog = ProjectDataConverterCatalog::compile([
        definition(
            "add",
            ProjectDataConverterKind::Math {
                operation: ProjectDataConverterMathOperation::Add,
                value: Some(2.0),
                value_path: None,
            },
        ),
        definition(
            "group",
            ProjectDataConverterKind::Group {
                items: vec!["add".to_owned(), "add".to_owned()],
            },
        ),
    ])
    .expect("valid catalog");
    let bytes = catalog.encode_program("group").expect("program encodes");
    assert!(ProjectDataConverterProgram::is_envelope(&bytes));
    let program = ProjectDataConverterProgram::decode(&bytes)
        .expect("program decodes")
        .expect("recognized envelope");
    let mut state = ProjectDataConverterState::default();
    let mut context = ProjectDataConverterContext::new();
    assert_eq!(
        program
            .convert(&mut state, ProjectDataValue::Number(1.0), &mut context,)
            .expect("program executes"),
        ProjectDataValue::Number(5.0)
    );
    assert!(
        ProjectDataConverterProgram::decode(b"ordinary luau bytecode")
            .expect("ordinary bytes are not malformed envelopes")
            .is_none()
    );
}

#[test]
fn reverses_groups_without_applying_forward_output_coercion() {
    let catalog = ProjectDataConverterCatalog::compile([
        ProjectDataConverterDefinition {
            id: "add".to_owned(),
            spec: ProjectDataConverterSpec {
                output_type: Some(ProjectDataConverterOutputType::String),
                kind: ProjectDataConverterKind::Math {
                    operation: ProjectDataConverterMathOperation::Add,
                    value: Some(2.0),
                    value_path: None,
                },
            },
        },
        definition(
            "multiply",
            ProjectDataConverterKind::Math {
                operation: ProjectDataConverterMathOperation::Multiply,
                value: Some(3.0),
                value_path: None,
            },
        ),
        definition(
            "group",
            ProjectDataConverterKind::Group {
                items: vec!["add".to_owned(), "multiply".to_owned()],
            },
        ),
    ])
    .expect("valid reversible group");
    let mut state = ProjectDataConverterState::default();
    let mut context = ProjectDataConverterContext::new();
    assert_eq!(
        catalog
            .reverse_convert(
                "add",
                &mut state,
                ProjectDataValue::String("5".to_owned()),
                &mut context,
            )
            .expect("reverse executes"),
        nuxie_runtime::ProjectDataConverterReverseResult {
            ok: true,
            value: ProjectDataValue::Number(3.0),
        }
    );
    assert_eq!(
        catalog
            .reverse_convert(
                "group",
                &mut state,
                ProjectDataValue::Number(9.0),
                &mut context,
            )
            .expect("group reverse executes"),
        nuxie_runtime::ProjectDataConverterReverseResult {
            ok: true,
            value: ProjectDataValue::Number(1.0),
        }
    );
}

#[test]
fn exposes_reachable_paths_and_rewrites_them_without_exposing_catalog_records() {
    let semantic_path = ProjectDataValuePath::Path {
        path: "settings.scale".to_owned(),
        view_model_name: Some("Settings".to_owned()),
        is_relative: false,
    };
    let unused_path = ProjectDataValuePath::Path {
        path: "unused.value".to_owned(),
        view_model_name: None,
        is_relative: true,
    };
    let catalog = ProjectDataConverterCatalog::compile([
        definition(
            "operand",
            ProjectDataConverterKind::Math {
                operation: ProjectDataConverterMathOperation::Multiply,
                value: None,
                value_path: Some(semantic_path.clone()),
            },
        ),
        definition(
            "unused",
            ProjectDataConverterKind::Math {
                operation: ProjectDataConverterMathOperation::Add,
                value: None,
                value_path: Some(unused_path.clone()),
            },
        ),
    ])
    .expect("valid semantic paths");

    assert_eq!(
        catalog.value_paths("operand").expect("known root"),
        vec![semantic_path.clone()]
    );

    let runtime_path = ProjectDataValuePath::Ids {
        path_ids: vec![0.0, 3.0, 1.0],
        is_relative: false,
        name_based: false,
    };
    let lowered = catalog
        .replace_value_paths(&[(semantic_path, runtime_path.clone())])
        .expect("replacement remains a valid catalog");
    let program = ProjectDataConverterProgram::decode(
        &lowered
            .encode_program("operand")
            .expect("lowered program encodes"),
    )
    .expect("lowered program decodes")
    .expect("recognized envelope");
    assert_eq!(program.value_paths(), vec![runtime_path]);
    assert_eq!(
        lowered
            .value_paths("unused")
            .expect("other root remains valid"),
        vec![unused_path]
    );
}

#[test]
fn embeds_only_reachable_number_to_list_view_models_in_the_runtime_program() {
    let catalog = ProjectDataConverterCatalog::compile([
        definition(
            "cards",
            ProjectDataConverterKind::NumberToList {
                view_model_id: "Card".to_owned(),
            },
        ),
        definition(
            "group",
            ProjectDataConverterKind::Group {
                items: vec!["cards".to_owned()],
            },
        ),
        definition(
            "unused",
            ProjectDataConverterKind::NumberToList {
                view_model_id: "Unused".to_owned(),
            },
        ),
    ])
    .expect("valid list catalog");

    assert_eq!(
        catalog.view_model_ids("group").expect("known root"),
        vec!["Card".to_owned()]
    );
    let bytes = catalog
        .encode_program_with_runtime_view_models("group", BTreeMap::from([("Card".to_owned(), 7)]))
        .expect("lowered program encodes");
    let program = ProjectDataConverterProgram::decode(&bytes)
        .expect("program decodes")
        .expect("recognized envelope");
    assert_eq!(program.runtime_view_model_index("Card"), Some(7));
    assert_eq!(program.runtime_view_model_index("Unused"), None);
    assert_eq!(program.number_to_list_output_view_model_index(), Some(7));
}

#[test]
fn number_to_list_output_follows_the_effective_group_output() {
    let catalog = ProjectDataConverterCatalog::compile([
        definition(
            "cards",
            ProjectDataConverterKind::NumberToList {
                view_model_id: "Card".to_owned(),
            },
        ),
        definition(
            "interpolate",
            ProjectDataConverterKind::Interpolate {
                duration_ms: 100.0,
                easing: ProjectDataConverterEasing::Linear,
            },
        ),
        definition(
            "group",
            ProjectDataConverterKind::Group {
                items: vec!["cards".to_owned(), "interpolate".to_owned()],
            },
        ),
    ])
    .expect("valid list group");
    let bytes = catalog
        .encode_program_with_runtime_view_models("group", BTreeMap::from([("Card".to_owned(), 7)]))
        .expect("lowered program encodes");
    let program = ProjectDataConverterProgram::decode(&bytes)
        .expect("program decodes")
        .expect("recognized envelope");

    assert_eq!(
        program.output_type(),
        Some(ProjectDataConverterOutputType::List)
    );
    assert_eq!(program.number_to_list_output_view_model_index(), Some(7));
}

#[test]
fn invalid_decimal_precision_uses_the_safe_intl_fallback_without_allocating() {
    let catalog = ProjectDataConverterCatalog::compile([
        definition(
            "format",
            ProjectDataConverterKind::Format {
                format: ProjectDataConverterFormat::Number,
                locale: Some("en-US".to_owned()),
                time_zone: None,
                options: BTreeMap::new(),
                trim_zeros: false,
                commas: true,
                decimals: Some(101),
            },
        ),
        definition(
            "to-string",
            ProjectDataConverterKind::ToString {
                locale: Some("en-US".to_owned()),
                decimals: Some(101),
                trim_zeros: false,
                commas: true,
            },
        ),
    ])
    .expect("invalid Intl precision is isolated to converter execution");
    let mut state = ProjectDataConverterState::default();
    for id in ["format", "to-string"] {
        assert_eq!(
            convert(
                &catalog,
                id,
                &mut state,
                ProjectDataValue::Number(1234.5),
                None,
            ),
            ProjectDataValue::String("1234.5".to_owned()),
            "converter {id}"
        );
    }
}

#[test]
fn clamps_allocation_bearing_converter_lengths() {
    struct BlankResolver;

    impl ProjectDataConverterResolver for BlankResolver {
        fn resolve_value(&mut self, _path: &ProjectDataValuePath) -> Option<ProjectDataValue> {
            None
        }

        fn create_blank_view_model_instance(
            &mut self,
            _view_model_id: &str,
        ) -> Option<ProjectDataValue> {
            Some(ProjectDataValue::Object(BTreeMap::new()))
        }
    }

    let catalog = ProjectDataConverterCatalog::compile([
        definition(
            "pad",
            ProjectDataConverterKind::StringPad {
                length: 1_000_001.0,
                text: "x".to_owned(),
                side: ProjectDataConverterStringPadSide::End,
            },
        ),
        definition(
            "items",
            ProjectDataConverterKind::NumberToList {
                view_model_id: "Item".to_owned(),
            },
        ),
    ])
    .expect("finite lengths compile");
    let mut state = ProjectDataConverterState::default();
    let mut resolver = BlankResolver;
    let mut context = ProjectDataConverterContext {
        now_ms: None,
        resolver: Some(&mut resolver),
    };

    let ProjectDataValue::String(padded) = catalog
        .convert(
            "pad",
            &mut state,
            ProjectDataValue::String(String::new()),
            &mut context,
        )
        .expect("pad converts")
    else {
        panic!("pad must return a string")
    };
    assert_eq!(padded.encode_utf16().count(), 1_000_000);

    state.clear();
    let ProjectDataValue::List(items) = catalog
        .convert(
            "items",
            &mut state,
            ProjectDataValue::Number(10_001.0),
            &mut context,
        )
        .expect("number-to-list converts")
    else {
        panic!("number-to-list must return a list")
    };
    assert_eq!(items.len(), 10_000);
}

#[test]
fn preflights_only_characterized_reverse_chains() {
    let catalog = ProjectDataConverterCatalog::compile([
        definition(
            "add",
            ProjectDataConverterKind::Math {
                operation: ProjectDataConverterMathOperation::Add,
                value: Some(2.0),
                value_path: None,
            },
        ),
        definition("not", ProjectDataConverterKind::BooleanNot),
        definition(
            "safe-group",
            ProjectDataConverterKind::Group {
                items: vec!["add".to_owned(), "not".to_owned()],
            },
        ),
        definition(
            "dynamic-multiply",
            ProjectDataConverterKind::Math {
                operation: ProjectDataConverterMathOperation::Multiply,
                value: None,
                value_path: Some(ProjectDataValuePath::Ids {
                    path_ids: vec![0.0, 1.0],
                    is_relative: false,
                    name_based: false,
                }),
            },
        ),
        definition(
            "format",
            ProjectDataConverterKind::ToString {
                locale: None,
                decimals: None,
                trim_zeros: false,
                commas: false,
            },
        ),
    ])
    .expect("valid reversibility catalog");

    assert!(catalog.is_reversible("safe-group").expect("known root"));
    assert!(
        !catalog
            .is_reversible("dynamic-multiply")
            .expect("known root")
    );
    assert!(!catalog.is_reversible("format").expect("known root"));

    let bytes = catalog
        .encode_program("safe-group")
        .expect("program encodes");
    let program = ProjectDataConverterProgram::decode(&bytes)
        .expect("program decodes")
        .expect("recognized envelope");
    assert!(program.is_reversible());
}
