//! Exact `NUXPCV1` execution behind Nuxie's translated scripting boundary.
//!
//! [`nuxie_project_data`] remains a runtime-independent compiler/evaluator.
//! This upper-leaf crate is the only adapter that knows both its program model
//! and the runtime's opaque script-program seam. Product hosts attach one
//! [`ProjectDataScriptProgramAdapter`] to an authenticated script capability;
//! ordinary script assets continue to the configured Luau backend unchanged.

use std::{collections::BTreeMap, sync::Arc};

use nuxie::{
    RuntimeScriptProgram, ScriptAssetRegistration, ScriptAssetRegistrationResult,
    ScriptDataConverterMethod, ScriptError, ScriptHost, ScriptInstance, ScriptMethod,
    ScriptProgramAdapter, ScriptValue, ScriptViewModel, ScriptedContextSource,
};
use nuxie_project_data::{
    ProjectDataConverterContext, ProjectDataConverterProgram, ProjectDataConverterState,
    ProjectDataValue,
};

/// Stateless factory for ProjectDO converter program occurrences.
#[derive(Debug, Clone, Copy, Default)]
pub struct ProjectDataScriptProgramAdapter;

impl ProjectDataScriptProgramAdapter {
    /// Return a shared adapter suitable for
    /// `ScriptExecutionCapability::with_program_adapter`.
    pub fn shared() -> Arc<dyn ScriptProgramAdapter> {
        Arc::new(Self)
    }
}

impl ScriptProgramAdapter for ProjectDataScriptProgramAdapter {
    fn register_script_asset(
        &self,
        registration: &ScriptAssetRegistration<'_>,
    ) -> Option<ScriptAssetRegistrationResult> {
        match ProjectDataConverterProgram::decode(registration.bytecode) {
            Ok(None) => None,
            Ok(Some(program)) => Some(ScriptAssetRegistrationResult {
                completed: true,
                program: Some(RuntimeScriptProgram::from_backend(program)),
                missing_dependencies: Vec::new(),
                error: None,
            }),
            Err(error) => Some(ScriptAssetRegistrationResult {
                completed: false,
                program: None,
                missing_dependencies: Vec::new(),
                error: Some(ScriptError::new(error.to_string())),
            }),
        }
    }

    fn instantiate_program(
        &self,
        program: &RuntimeScriptProgram,
        _context_present: bool,
        _context_source: Option<ScriptedContextSource>,
        _view_model: Option<ScriptViewModel>,
        _parent_view_models: Vec<Option<ScriptViewModel>>,
        _host: &mut dyn ScriptHost,
    ) -> Option<Result<Box<dyn ScriptInstance>, ScriptError>> {
        program
            .backend::<ProjectDataConverterProgram>()
            .cloned()
            .map(|program| {
                Ok(Box::new(ProjectDataScriptInstance::new(program)) as Box<dyn ScriptInstance>)
            })
    }
}

#[derive(Debug)]
struct ProjectDataScriptInstance {
    program: ProjectDataConverterProgram,
    state: ProjectDataConverterState,
    now_ms: f64,
    inputs: BTreeMap<String, ScriptValue>,
}

impl ProjectDataScriptInstance {
    fn new(program: ProjectDataConverterProgram) -> Self {
        Self {
            program,
            state: ProjectDataConverterState::default(),
            now_ms: 0.0,
            inputs: BTreeMap::new(),
        }
    }

    fn advance(&mut self, elapsed_seconds: f32) -> bool {
        self.now_ms += f64::from(elapsed_seconds) * 1_000.0;
        self.state.is_interpolating()
    }

    fn convert(
        &mut self,
        method: ScriptDataConverterMethod,
        value: ScriptValue,
    ) -> Result<ScriptValue, ScriptError> {
        let input = project_value(value)?;
        let mut context = ProjectDataConverterContext {
            now_ms: Some(self.now_ms),
            resolver: None,
        };
        let output = match method {
            ScriptDataConverterMethod::Convert => self
                .program
                .convert(&mut self.state, input, &mut context)
                .map_err(|error| ScriptError::new(error.to_string()))?,
            ScriptDataConverterMethod::ReverseConvert => {
                self.program
                    .reverse_convert(&mut self.state, input, &mut context)
                    .map_err(|error| ScriptError::new(error.to_string()))?
                    .value
            }
        };
        script_value(output)
    }
}

impl ScriptInstance for ProjectDataScriptInstance {
    fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
        Ok(method == ScriptMethod::Advance && self.program.is_stateful())
    }

    fn call_method(
        &mut self,
        method: ScriptMethod,
        args: &[ScriptValue],
        _host: &mut dyn ScriptHost,
    ) -> Result<ScriptValue, ScriptError> {
        if method != ScriptMethod::Advance || !self.program.is_stateful() {
            return Err(ScriptError::new(
                "project-data program has no such script method",
            ));
        }
        let Some(ScriptValue::Number(elapsed_seconds)) = args.first() else {
            return Err(ScriptError::new(
                "project-data advance requires elapsed seconds",
            ));
        };
        Ok(ScriptValue::Bool(self.advance(*elapsed_seconds as f32)))
    }

    fn call_advance_truthy(
        &mut self,
        elapsed_seconds: f32,
        _host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        Ok(self.advance(elapsed_seconds))
    }

    fn call_data_converter(
        &mut self,
        method: ScriptDataConverterMethod,
        value: ScriptValue,
    ) -> Result<ScriptValue, ScriptError> {
        self.convert(method, value)
    }

    fn call_data_converter_if_present(
        &mut self,
        method: ScriptDataConverterMethod,
        value: ScriptValue,
    ) -> Result<Option<ScriptValue>, ScriptError> {
        if !self.has_data_converter_method(method)? {
            return Ok(None);
        }
        self.convert(method, value).map(Some)
    }

    fn has_data_converter_method(
        &self,
        method: ScriptDataConverterMethod,
    ) -> Result<bool, ScriptError> {
        Ok(match method {
            ScriptDataConverterMethod::Convert => true,
            ScriptDataConverterMethod::ReverseConvert => self.program.is_reversible(),
        })
    }

    fn get_input(&self, name: &str) -> Result<ScriptValue, ScriptError> {
        Ok(self.inputs.get(name).cloned().unwrap_or(ScriptValue::Nil))
    }

    fn set_input(&mut self, name: &str, value: ScriptValue) -> Result<(), ScriptError> {
        self.inputs.insert(name.to_owned(), value);
        Ok(())
    }
}

fn project_value(value: ScriptValue) -> Result<ProjectDataValue, ScriptError> {
    match value {
        ScriptValue::Nil => Ok(ProjectDataValue::Null),
        ScriptValue::Bool(value) => Ok(ProjectDataValue::Boolean(value)),
        ScriptValue::Number(value) => Ok(ProjectDataValue::Number(value)),
        ScriptValue::String(value) => Ok(ProjectDataValue::String(value)),
        ScriptValue::CoreString(value) => String::from_utf8(value.into_bytes())
            .map(ProjectDataValue::String)
            .map_err(|_| ScriptError::new("project-data converter requires UTF-8 string input")),
        ScriptValue::Color(value) => Ok(ProjectDataValue::Color(value)),
        ScriptValue::Vec2 { .. } | ScriptValue::Vec3 { .. } => Err(ScriptError::new(
            "project-data converter does not accept vector input",
        )),
    }
}

fn script_value(value: ProjectDataValue) -> Result<ScriptValue, ScriptError> {
    match value {
        ProjectDataValue::Null => Ok(ScriptValue::Nil),
        ProjectDataValue::Boolean(value) => Ok(ScriptValue::Bool(value)),
        ProjectDataValue::Number(value) => Ok(ScriptValue::Number(value)),
        ProjectDataValue::String(value) => Ok(ScriptValue::String(value)),
        ProjectDataValue::Color(value) => Ok(ScriptValue::Color(value)),
        ProjectDataValue::List(_)
        | ProjectDataValue::Object(_)
        | ProjectDataValue::Enum(_)
        | ProjectDataValue::ListIndex(_)
        | ProjectDataValue::Trigger(_)
        | ProjectDataValue::Image(_)
        | ProjectDataValue::ViewModel(_) => Err(ScriptError::new(
            "project-data converter output cannot cross the exact scripted-data scalar boundary",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nuxie::{ScriptDataConverterOptionalCall, ScriptOptionalMethodResult};
    use nuxie_project_data::{
        ProjectDataConverterCatalog, ProjectDataConverterDefinition, ProjectDataConverterKind,
        ProjectDataConverterMathOperation, ProjectDataConverterSpec,
    };

    #[derive(Default)]
    struct NoopHost;

    impl ScriptHost for NoopHost {
        fn mark_script_update(&mut self) {}
    }

    fn program_bytes(kind: ProjectDataConverterKind) -> Vec<u8> {
        ProjectDataConverterCatalog::compile([ProjectDataConverterDefinition {
            id: "root".to_owned(),
            spec: ProjectDataConverterSpec {
                output_type: None,
                kind,
            },
        }])
        .expect("valid catalog")
        .encode_program("root")
        .expect("program encodes")
    }

    #[test]
    fn claims_only_project_data_envelopes_and_executes_scalar_conversion() {
        let adapter = ProjectDataScriptProgramAdapter;
        assert!(
            adapter
                .register_script_asset(&ScriptAssetRegistration {
                    name: "ordinary",
                    bytecode: b"ordinary luau bytecode",
                    is_protocol: true,
                    missing_dependencies: Vec::new(),
                })
                .is_none()
        );
        let bytes = program_bytes(ProjectDataConverterKind::Math {
            operation: ProjectDataConverterMathOperation::Add,
            value: Some(2.0),
            value_path: None,
        });
        let registered = adapter
            .register_script_asset(&ScriptAssetRegistration {
                name: "converter",
                bytecode: &bytes,
                is_protocol: true,
                missing_dependencies: Vec::new(),
            })
            .expect("project envelope is claimed");
        assert!(registered.completed);
        let mut host = NoopHost;
        let mut instance = adapter
            .instantiate_program(
                registered.program.as_ref().expect("program"),
                false,
                None,
                None,
                Vec::new(),
                &mut host,
            )
            .expect("adapter owns program")
            .expect("instance constructs");
        assert_eq!(
            instance
                .call_optional_data_converter(
                    ScriptDataConverterMethod::Convert,
                    Some(ScriptValue::Number(3.0)),
                )
                .expect("conversion runs"),
            ScriptDataConverterOptionalCall::Returned(ScriptValue::Number(5.0))
        );
        assert_eq!(
            instance
                .call_optional_method(ScriptMethod::Init, &[], &mut host)
                .expect("missing init is harmless"),
            ScriptOptionalMethodResult::Missing
        );
    }
}
