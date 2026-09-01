//! Authenticated, factory-first host setup around the translated File importer.
//! Script programs and object lifecycle belong to the native File, not this seam.
use crate::import_limits::FileImportLimits;
use anyhow::{Context, Result};
use nuxie_render_api::{Factory, GpuCanvasShaderProvenance};
use nuxie_runtime::mechanical_port::source::{
    assets::{script_asset::ScriptAsset, shader_asset::ShaderAsset},
    core::CoreHandle,
    factory::RuntimeFactoryHandle,
    file::{RuntimeFileHandle, RuntimeFileWeakHandle},
    file_asset_loader::{FileAssetLoader, FileAssetLoaderRef},
    lua::scripting_vm::RuntimeScriptingVmHandle,
};
use nuxie_runtime::{
    RuntimeScriptProgram, ScriptAssetRegistration, ScriptAssetRegistrationResult, ScriptError,
    ScriptHost, ScriptInstance, ScriptModule, ScriptModuleFailure, ScriptViewModel,
    ScriptedContextSource, ScriptingVm,
};
use nuxie_scripting::host_commands::{HostCommand, HostCommandLimits};
use nuxie_scripting::vm::{ScriptExecutionLimits, ScriptVm};
use std::{any::Any, cell::RefCell, collections::BTreeMap, rc::Rc, sync::Arc};

/// The native File plus the one VM and host extension installed before import.
/// Cloning its native File handle also retains that installed host through the
/// runtime's VM handle; dropping this wrapper cannot strand live callbacks.
pub struct ScriptedFile {
    file: RuntimeFileHandle,
    installed: Rc<InstalledScripts>,
}

impl ScriptedFile {
    pub fn native_file(&self) -> &RuntimeFileHandle {
        &self.file
    }
    pub fn vm(&self) -> &ScriptVm {
        &self.installed.vm
    }
    pub fn host(&self) -> &dyn ScriptHostExtensionInstance {
        self.installed.host.as_ref()
    }
    /// Start one host-owned unit of script work. Every callback in the unit
    /// shares the VM's aggregate limits and first-failure side channel.
    pub fn begin_script_cycle(&self) {
        self.installed.vm.begin_script_cycle();
    }
    /// End the host-owned unit started by [`Self::begin_script_cycle`].
    pub fn end_script_cycle(&self) {
        self.installed.vm.end_script_cycle();
    }
}

/// Import original bytes with an explicit factory, authenticated script
/// capability, and nonzero execution limits. Code registration and all source
/// object initialization remain inside the translated File import lifecycle.
pub fn import_scripted(
    bytes: &[u8],
    factory: &mut dyn Factory,
    loader: Option<FileAssetLoaderRef>,
    import_limits: FileImportLimits,
    capability: ScriptExecutionCapability,
    execution_limits: ScriptExecutionLimits,
    log_sink: Option<nuxie_scripting::vm::ScriptingLogSink>,
) -> Result<ScriptedFile> {
    import_limits.validate_input(bytes)?;
    anyhow::ensure!(
        capability.authorizes(bytes),
        "script execution capability does not match the exact artifact bytes"
    );
    execution_limits
        .validate()
        .context("invalid trusted script execution limits")?;
    anyhow::ensure!(
        factory.persistent_context().is_some(),
        "script import requires the supplied factory's persistent renderer identity"
    );
    let vm = Rc::new(
        ScriptVm::new_with_execution_limits(execution_limits)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?,
    );
    if let Some(sink) = log_sink {
        vm.set_log_sink(move |level, line| sink(level, line));
    }
    vm.install_render_factory(factory)?;
    vm.begin_script_cycle();
    struct ImportCycle(Rc<ScriptVm>);
    impl Drop for ImportCycle {
        fn drop(&mut self) {
            self.0.end_script_cycle();
        }
    }
    let cycle = ImportCycle(vm.clone());
    let host = capability.extension.install(&vm)?;
    let installed = Rc::new(InstalledScripts {
        host,
        shader_authorities: RefCell::new(Vec::new()),
        vm,
        program_adapter: capability.program_adapter.clone(),
    });
    let loader = FileAssetLoaderRef::new(Box::new(AdmittedCodeAssetLoader {
        next: loader,
        installed: installed.clone(),
        native_shaders_are_authorized: capability.authorizes_native_shader_code(),
    }));
    let runtime_vm = RuntimeScriptingVmHandle::new(Box::new(installed.clone()));
    let imported = crate::native_file::import(
        bytes,
        factory,
        Some(loader),
        Some(runtime_vm),
        import_limits,
    );
    let resource_result = installed
        .vm
        .resource_guard()
        .reject_if_tripped()
        .map_err(|error| anyhow::anyhow!(error.to_string()));
    drop(cycle);
    resource_result?;
    Ok(ScriptedFile {
        file: imported?,
        installed,
    })
}

/// Explicit unsigned conformance-fixture opt-in. This is absent from ordinary
/// production builds and installs no product host module or native-code proof.
#[cfg(any(test, feature = "test-support"))]
pub fn import_unsigned_scripted(
    bytes: &[u8],
    factory: &mut dyn Factory,
    loader: Option<FileAssetLoaderRef>,
    import_limits: FileImportLimits,
    execution_limits: ScriptExecutionLimits,
) -> Result<ScriptedFile> {
    // SAFETY: explicit test-only opt-in for these exact fixture bytes; the
    // neutral extension supplies no host effects and grants no native shaders.
    let capability = unsafe {
        ScriptExecutionCapability::for_verified_artifact_unchecked(
            bytes,
            Arc::new(NoopScriptHostExtension),
        )?
    };
    import_scripted(
        bytes,
        factory,
        loader,
        import_limits,
        capability,
        execution_limits,
        None,
    )
}

struct InstalledScripts {
    // The host can own Lua values; release it before the concrete VM.
    host: Box<dyn ScriptHostExtensionInstance>,
    shader_authorities: RefCell<Vec<(Arc<[u8]>, GpuCanvasShaderProvenance)>>,
    vm: Rc<ScriptVm>,
    program_adapter: Option<Arc<dyn nuxie_runtime::ScriptProgramAdapter>>,
}

/// Authenticated code is taken only from the original file's in-band bytes.
/// An ordinary image/font/audio loader cannot replace those bytes or attach
/// unauthenticated external code under a capability for the containing file.
struct AdmittedCodeAssetLoader {
    next: Option<FileAssetLoaderRef>,
    installed: Rc<InstalledScripts>,
    native_shaders_are_authorized: bool,
}
impl FileAssetLoader for AdmittedCodeAssetLoader {
    fn load_contents(
        &mut self,
        asset: CoreHandle,
        bytes: &[u8],
        factory: &RuntimeFactoryHandle,
    ) -> bool {
        if asset
            .with_downcast_mut::<ScriptAsset, _>(|script| {
                let mut bytes = bytes.to_vec();
                // SAFETY: the entry point checked the capability against the
                // enclosing artifact; this loader sees its original in-band
                // contents before any external asset loader can replace them.
                unsafe { script.decode_with_host_authorization(&mut bytes, factory) };
            })
            .is_some()
        {
            return true;
        }
        if let Some(decoded) =
            asset.with_downcast_mut::<ShaderAsset, _>(|shader| shader.decode(bytes, factory))
        {
            if decoded {
                if let Some(proof) = mint_shader_provenance(
                    self.native_shaders_are_authorized,
                    "ShaderAsset",
                    Some(bytes),
                ) {
                    self.installed
                        .shader_authorities
                        .borrow_mut()
                        .push((Arc::from(bytes), proof));
                }
            }
            return true;
        }
        self.next.as_ref().is_some_and(|next| {
            next.with_loader_mut(|next| next.load_contents(asset, bytes, factory))
        })
    }
}

impl ScriptingVm for InstalledScripts {
    fn install_native_file_assets(
        &self,
        file: RuntimeFileWeakHandle,
    ) -> std::result::Result<(), ScriptError> {
        self.vm
            .set_native_shader_authorities(self.shader_authorities.borrow().clone());
        ScriptingVm::install_native_file_assets(&*self.vm, file)
    }
    fn initialize_data_global(
        &self,
        models: BTreeMap<String, ScriptViewModel>,
    ) -> std::result::Result<(), ScriptError> {
        ScriptingVm::initialize_data_global(&*self.vm, models)
    }
    fn install_render_factory(
        &self,
        factory: &mut dyn Factory,
    ) -> std::result::Result<(), ScriptError> {
        ScriptingVm::install_render_factory(&*self.vm, factory)
    }
    fn install_rive_globals(&self) -> std::result::Result<(), ScriptError> {
        ScriptingVm::install_rive_globals(&*self.vm)
    }
    fn register_module(&self, name: &str, payload: &[u8]) -> std::result::Result<(), ScriptError> {
        ScriptingVm::register_module(&*self.vm, name, payload)
    }
    fn register_script_assets(
        &self,
        scripts: &[ScriptAssetRegistration<'_>],
    ) -> Vec<ScriptAssetRegistrationResult> {
        let Some(adapter) = self.program_adapter.as_ref() else {
            return ScriptingVm::register_script_assets(&*self.vm, scripts);
        };
        let mut results = (0..scripts.len())
            .map(|_| ScriptAssetRegistrationResult::default())
            .collect::<Vec<_>>();
        let mut delegated_indices = Vec::new();
        let mut delegated = Vec::new();
        for (index, registration) in scripts.iter().enumerate() {
            if let Some(result) = adapter.register_script_asset(registration) {
                results[index] = result;
            } else {
                delegated_indices.push(index);
                delegated.push(ScriptAssetRegistration {
                    name: registration.name,
                    bytecode: registration.bytecode,
                    is_protocol: registration.is_protocol,
                    missing_dependencies: registration.missing_dependencies.clone(),
                });
            }
        }
        for (index, result) in delegated_indices
            .into_iter()
            .zip(ScriptingVm::register_script_assets(&*self.vm, &delegated))
        {
            results[index] = result;
        }
        results
    }
    fn instantiate_program(
        &self,
        program: &RuntimeScriptProgram,
        present: bool,
        source: Option<ScriptedContextSource>,
        model: Option<ScriptViewModel>,
        parents: Vec<Option<ScriptViewModel>>,
        host: &mut dyn ScriptHost,
    ) -> std::result::Result<Box<dyn ScriptInstance>, ScriptError> {
        if let Some(result) = self.program_adapter.as_ref().and_then(|adapter| {
            adapter.instantiate_program(
                program,
                present,
                source.clone(),
                model.clone(),
                parents.clone(),
                host,
            )
        }) {
            return result;
        }
        ScriptingVm::instantiate_program(&*self.vm, program, present, source, model, parents, host)
    }
    fn instantiate_script(
        &self,
        name: &str,
        payload: &[u8],
        host: &mut dyn ScriptHost,
    ) -> std::result::Result<Box<dyn ScriptInstance>, ScriptError> {
        ScriptingVm::instantiate_script(&*self.vm, name, payload, host)
    }
    fn advance_detached_view_models(&self) -> bool {
        ScriptingVm::advance_detached_view_models(&*self.vm)
    }
    fn perform_registration(&self, modules: &[ScriptModule<'_>]) -> Vec<ScriptModuleFailure> {
        ScriptingVm::perform_registration(&*self.vm, modules)
    }
}

/// Explicit configuration for an exact-artifact import that exposes one
/// caller-named, product-neutral script module.
#[cfg(feature = "scripting")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostCommandImportConfig {
    module_name: String,
    execution_limits: ScriptExecutionLimits,
    command_limits: HostCommandLimits,
}

#[cfg(feature = "scripting")]
impl HostCommandImportConfig {
    /// Build only the caller-named host module. Import still requires a
    /// separately minted exact-artifact capability and its execution limits.
    pub fn extension(&self) -> Arc<dyn ScriptHostExtension> {
        Arc::new(GenericHostCommandExtension {
            module_name: self.module_name.clone(),
            limits: self.command_limits,
        })
    }
    pub fn new(
        module_name: impl Into<String>,
        execution_limits: ScriptExecutionLimits,
        command_limits: HostCommandLimits,
    ) -> Result<Self> {
        let module_name = module_name.into();
        anyhow::ensure!(
            !module_name.is_empty()
                && module_name.len() <= nuxie_scripting::host_commands::MAX_HOST_MODULE_NAME_BYTES,
            "host module name must contain 1 to {} UTF-8 bytes",
            nuxie_scripting::host_commands::MAX_HOST_MODULE_NAME_BYTES
        );
        execution_limits
            .validate()
            .context("invalid trusted script execution limits")?;
        command_limits
            .validate()
            .map_err(|error| anyhow::anyhow!(error.to_string()))
            .context("invalid host command limits")?;
        Ok(Self {
            module_name,
            execution_limits,
            command_limits,
        })
    }

    pub fn module_name(&self) -> &str {
        &self.module_name
    }

    pub const fn execution_limits(&self) -> ScriptExecutionLimits {
        self.execution_limits
    }

    pub const fn command_limits(&self) -> HostCommandLimits {
        self.command_limits
    }
}

#[cfg(feature = "scripting")]
#[derive(Debug)]
struct GenericHostCommandExtension {
    module_name: String,
    limits: HostCommandLimits,
}

#[cfg(feature = "scripting")]
impl ScriptHostExtension for GenericHostCommandExtension {
    fn install(
        &self,
        vm: &ScriptVm,
    ) -> std::result::Result<Box<dyn ScriptHostExtensionInstance>, nuxie_runtime::ScriptError> {
        nuxie_scripting::host_commands::HostCommandHost::install(vm, &self.module_name, self.limits)
            .map(|host| Box::new(host) as Box<dyn ScriptHostExtensionInstance>)
            .map_err(|error| nuxie_runtime::ScriptError::new(error.to_string()))
    }
}

#[cfg(feature = "scripting")]
impl ScriptHostExtensionInstance for nuxie_scripting::host_commands::HostCommandHost {
    fn effects_type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Vec<HostCommand>>()
    }

    fn begin_cycle(&self) -> Box<dyn Any> {
        Box::new(self.begin_cycle())
    }

    fn rollback_cycle(
        &self,
        checkpoint: Box<dyn Any>,
    ) -> std::result::Result<(), nuxie_runtime::ScriptError> {
        let checkpoint = checkpoint
            .downcast::<nuxie_scripting::host_commands::HostCycleCheckpoint>()
            .map_err(|_| nuxie_runtime::ScriptError::new("host command checkpoint mismatch"))?;
        self.rollback_cycle(*checkpoint);
        Ok(())
    }

    fn checkpoint_effects(&self) -> Box<dyn Any> {
        Box::new(self.checkpoint_effects())
    }

    fn rollback_effects(
        &self,
        checkpoint: Box<dyn Any>,
    ) -> std::result::Result<(), nuxie_runtime::ScriptError> {
        let checkpoint = checkpoint
            .downcast::<nuxie_scripting::host_commands::HostEffectCheckpoint>()
            .map_err(|_| nuxie_runtime::ScriptError::new("host command checkpoint mismatch"))?;
        self.rollback_effects(*checkpoint);
        Ok(())
    }

    fn drain_effects(&self) -> ScriptHostEffects {
        ScriptHostEffects::new(self.drain_effects())
    }

    fn commit_error(&self) -> Option<nuxie_runtime::ScriptError> {
        self.callback_failure().map(nuxie_runtime::ScriptError::new)
    }
}

/// Baseline-owned injection point for product-specific VM modules and effects.
#[cfg(feature = "scripting")]
pub trait ScriptHostExtension: std::fmt::Debug {
    fn install(
        &self,
        vm: &ScriptVm,
    ) -> std::result::Result<Box<dyn ScriptHostExtensionInstance>, nuxie_runtime::ScriptError>;
}

/// Dynamically typed host-effect payload with a type witness derived from the
/// payload itself. Construction is generic so implementations cannot pair an
/// arbitrary type id with a different boxed value.
#[cfg(feature = "scripting")]
pub struct ScriptHostEffects {
    value: Box<dyn Any>,
    type_id: std::any::TypeId,
}

#[cfg(feature = "scripting")]
impl ScriptHostEffects {
    pub fn new<T: 'static>(value: T) -> Self {
        Self {
            value: Box::new(value),
            type_id: std::any::TypeId::of::<T>(),
        }
    }

    pub fn type_id(&self) -> std::any::TypeId {
        self.type_id
    }

    pub fn downcast<T: 'static>(self) -> Result<T, Self> {
        match self.value.downcast::<T>() {
            Ok(value) => Ok(*value),
            Err(value) => Err(Self {
                value,
                type_id: self.type_id,
            }),
        }
    }
}

#[cfg(feature = "scripting")]
impl std::fmt::Debug for ScriptHostEffects {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ScriptHostEffects")
            .field("type_id", &self.type_id)
            .finish_non_exhaustive()
    }
}

/// One installed host extension associated with a single script VM.
#[cfg(feature = "scripting")]
pub trait ScriptHostExtensionInstance: std::fmt::Debug {
    /// Concrete type returned by [`Self::drain_effects`]. Transactions inspect
    /// this before draining so a mismatched projector cannot consume effects.
    fn effects_type_id(&self) -> std::any::TypeId;
    fn begin_cycle(&self) -> Box<dyn Any>;
    fn rollback_cycle(
        &self,
        checkpoint: Box<dyn Any>,
    ) -> std::result::Result<(), nuxie_runtime::ScriptError>;
    fn checkpoint_effects(&self) -> Box<dyn Any>;
    fn rollback_effects(
        &self,
        checkpoint: Box<dyn Any>,
    ) -> std::result::Result<(), nuxie_runtime::ScriptError>;
    fn drain_effects(&self) -> ScriptHostEffects;

    /// A transaction-level failure side channel for protected callback
    /// errors that the pinned runtime intentionally consumes internally.
    fn commit_error(&self) -> Option<nuxie_runtime::ScriptError> {
        None
    }
}

/// Script host with no injected modules and an empty effect stream.
///
/// Product program adapters use this when an authenticated artifact needs the
/// translated scripting lifecycle but the embedding did not request the
/// optional portable host-command module.
#[cfg(feature = "scripting")]
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopScriptHostExtension;

#[cfg(feature = "scripting")]
#[derive(Debug)]
struct NoopScriptHostExtensionInstance;

#[cfg(feature = "scripting")]
impl ScriptHostExtension for NoopScriptHostExtension {
    fn install(
        &self,
        _vm: &ScriptVm,
    ) -> std::result::Result<Box<dyn ScriptHostExtensionInstance>, nuxie_runtime::ScriptError> {
        Ok(Box::new(NoopScriptHostExtensionInstance))
    }
}

#[cfg(feature = "scripting")]
impl ScriptHostExtensionInstance for NoopScriptHostExtensionInstance {
    fn effects_type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<()>()
    }

    fn begin_cycle(&self) -> Box<dyn Any> {
        Box::new(())
    }

    fn rollback_cycle(
        &self,
        _checkpoint: Box<dyn Any>,
    ) -> std::result::Result<(), nuxie_runtime::ScriptError> {
        Ok(())
    }

    fn checkpoint_effects(&self) -> Box<dyn Any> {
        Box::new(())
    }

    fn rollback_effects(
        &self,
        _checkpoint: Box<dyn Any>,
    ) -> std::result::Result<(), nuxie_runtime::ScriptError> {
        Ok(())
    }

    fn drain_effects(&self) -> ScriptHostEffects {
        ScriptHostEffects::new(())
    }
}

#[cfg(feature = "scripting")]
#[derive(Debug, Clone, PartialEq, Eq)]
enum ScriptExecutionBinding {
    ExactArtifact {
        artifact_size: u64,
        artifact_sha256: [u8; 32],
    },
}

/// Opaque authority to execute imported scripts with an injected host.
///
/// Safe baseline APIs can consume but cannot mint this value. Product policy
/// must authenticate exact bytes before using an unsafe constructor.
#[cfg(feature = "scripting")]
#[derive(Clone)]
pub struct ScriptExecutionCapability {
    binding: ScriptExecutionBinding,
    extension: Arc<dyn ScriptHostExtension>,
    native_shader_code: bool,
    program_adapter: Option<Arc<dyn nuxie_runtime::ScriptProgramAdapter>>,
}

#[cfg(feature = "scripting")]
impl std::fmt::Debug for ScriptExecutionCapability {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ScriptExecutionCapability")
            .field("binding", &self.binding)
            .field("extension", &self.extension)
            .finish()
    }
}

#[cfg(feature = "scripting")]
impl ScriptExecutionCapability {
    /// Mint authority for exact bytes after an upper layer verifies them.
    ///
    /// # Safety
    ///
    /// The caller must have authenticated `artifact_bytes` according to its
    /// product trust policy and must provide only the intended host extension.
    #[doc(hidden)]
    pub unsafe fn for_verified_artifact_unchecked(
        artifact_bytes: &[u8],
        extension: Arc<dyn ScriptHostExtension>,
    ) -> Result<Self> {
        use sha2::{Digest as _, Sha256};

        Ok(Self {
            binding: ScriptExecutionBinding::ExactArtifact {
                artifact_size: u64::try_from(artifact_bytes.len())
                    .context("script artifact length does not fit in u64")?,
                artifact_sha256: Sha256::digest(artifact_bytes).into(),
            },
            extension,
            native_shader_code: false,
            program_adapter: None,
        })
    }

    /// Mint script and native-shader authority for exact trusted-exporter bytes.
    ///
    /// # Safety
    ///
    /// In addition to authenticating `artifact_bytes`, the caller must have
    /// established that every native shader payload was produced by the
    /// product's trusted MSL compiler/exporter and is valid for unsafe Metal
    /// passthrough. A signature or digest alone does not establish this.
    #[doc(hidden)]
    pub unsafe fn for_verified_native_shader_artifact_unchecked(
        artifact_bytes: &[u8],
        extension: Arc<dyn ScriptHostExtension>,
    ) -> Result<Self> {
        use sha2::{Digest as _, Sha256};

        Ok(Self {
            binding: ScriptExecutionBinding::ExactArtifact {
                artifact_size: u64::try_from(artifact_bytes.len())
                    .context("script artifact length does not fit in u64")?,
                artifact_sha256: Sha256::digest(artifact_bytes).into(),
            },
            extension,
            native_shader_code: true,
            program_adapter: None,
        })
    }

    /// Attach a product-owned program family after the enclosing artifact has
    /// already been authenticated. Unclaimed assets still use the ordinary
    /// Luau backend.
    #[doc(hidden)]
    pub fn with_program_adapter(
        mut self,
        adapter: Arc<dyn nuxie_runtime::ScriptProgramAdapter>,
    ) -> Self {
        self.program_adapter = Some(adapter);
        self
    }

    fn authorizes(&self, artifact_bytes: &[u8]) -> bool {
        use sha2::{Digest as _, Sha256};

        let ScriptExecutionBinding::ExactArtifact {
            artifact_size,
            artifact_sha256,
        } = self.binding;
        u64::try_from(artifact_bytes.len()) == Ok(artifact_size)
            && <[u8; 32]>::from(Sha256::digest(artifact_bytes)) == artifact_sha256
    }

    /// Native code requires the stronger, compiler-provenance-bearing exact
    /// artifact constructor. Generic script authority is intentionally inert.
    fn authorizes_native_shader_code(&self) -> bool {
        self.native_shader_code
    }
}

#[cfg(all(
    feature = "scripting",
    any(feature = "ore-metal-authored-msl", feature = "android-authored-wgsl")
))]
fn mint_shader_provenance(
    native_shaders_are_authorized: bool,
    type_name: &str,
    payload: Option<&[u8]>,
) -> Option<nuxie_render_api::GpuCanvasShaderProvenance> {
    if !native_shaders_are_authorized || type_name != "ShaderAsset" {
        return None;
    }
    let payload = payload?;
    use sha2::{Digest as _, Sha256};

    let artifact_size = u64::try_from(payload.len()).ok()?;
    let artifact_sha256 = Sha256::digest(payload).into();
    // SAFETY: the dedicated native-shader capability and non-zero execution
    // policy admitted this exact imported artifact as trusted exporter output.
    // `payload` is owned by that decoded file; no safe caller-provided boolean
    // can mint or retarget this proof.
    Some(unsafe {
        nuxie_render_api::GpuCanvasShaderProvenance::for_verified_artifact_digest_unchecked(
            artifact_size,
            artifact_sha256,
        )
    })
}

#[cfg(all(
    feature = "scripting",
    not(any(feature = "ore-metal-authored-msl", feature = "android-authored-wgsl"))
))]
fn mint_shader_provenance(
    _native_shaders_are_authorized: bool,
    _type_name: &str,
    _payload: Option<&[u8]>,
) -> Option<nuxie_render_api::GpuCanvasShaderProvenance> {
    None
}
