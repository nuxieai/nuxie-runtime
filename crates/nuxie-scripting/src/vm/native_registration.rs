//! Native adaptation of ScriptingContext::performRegistration. Protocol
//! generators stay with their ScriptAsset; only utility values enter require.
use super::*;

const DEPENDENCIES: &str = "_RIVE_REGISTRATION_DEPENDENCIES";

pub(super) struct RegisteredProtocolProgram {
    pub vm: Rc<()>,
    pub program: ScriptProgram,
}

struct RegistrationScope {
    lua: Lua,
    previous: Value,
}
impl Drop for RegistrationScope {
    fn drop(&mut self) {
        // Restore an enclosing registration scope during nested host calls.
        let _ = self
            .lua
            .set_named_registry_value(DEPENDENCIES, self.previous.clone());
    }
}

pub(super) fn register(
    vm: &ScriptVm,
    scripts: &[ScriptAssetRegistration<'_>],
) -> Vec<ScriptAssetRegistrationResult> {
    let mut results: Vec<_> = scripts
        .iter()
        .map(|_| ScriptAssetRegistrationResult::default())
        .collect();
    let mut run = || -> Result<()> {
        vm.install_rive_globals()?;
        let dependencies = vm.lua.create_table();
        let lookup: BTreeMap<_, _> = scripts
            .iter()
            .enumerate()
            .map(|(index, script)| (script.name, index))
            .collect();
        for script in scripts {
            let values = vm.lua.create_table();
            for dependency in &script.missing_dependencies {
                values.raw_set(dependency.as_str(), true)?;
            }
            dependencies.raw_set(script.name, values)?;
        }
        let _scope = RegistrationScope {
            lua: vm.lua.clone(),
            previous: vm
                .lua
                .named_registry_value(DEPENDENCIES)
                .unwrap_or(Value::Nil),
        };
        vm.lua
            .set_named_registry_value(DEPENDENCIES, dependencies.clone())?;
        let mut pending = BTreeSet::new();
        for (index, script) in scripts.iter().enumerate() {
            if !matches!(vm.registered_module(script.name)?, Value::Nil) {
                continue;
            }
            attempt(
                vm,
                scripts,
                index,
                &dependencies,
                &mut pending,
                &mut results,
            )?;
        }
        // Exactly one dependency-ordered retry pass. Ordinary script failures
        // do not acquire retries merely because another module made progress.
        let mut remaining: Vec<_> = pending.iter().copied().collect();
        let mut sorted = Vec::new();
        let mut visited = BTreeSet::new();
        let mut visiting = BTreeSet::new();
        fn sort(
            index: usize,
            scripts: &[ScriptAssetRegistration<'_>],
            dependencies: &Table,
            lookup: &BTreeMap<&str, usize>,
            remaining: &mut Vec<usize>,
            sorted: &mut Vec<usize>,
            visited: &mut BTreeSet<usize>,
            visiting: &mut BTreeSet<usize>,
        ) -> Result<()> {
            if visited.contains(&index) {
                return Ok(());
            }
            if !visiting.insert(index) {
                // Upstream recurses without a cycle guard. Rust rejects this
                // invalid dependency graph instead of exhausting the host stack.
                return Err(Error::runtime("cyclic script module dependency"));
            }
            let deps: Table = dependencies.raw_get(scripts[index].name)?;
            for entry in deps.pairs::<String, bool>() {
                let (name, _) = entry?;
                if let Some(dependency) = lookup.get(name.as_str()) {
                    sort(
                        *dependency,
                        scripts,
                        dependencies,
                        lookup,
                        remaining,
                        sorted,
                        visited,
                        visiting,
                    )?;
                }
            }
            if !sorted.contains(&index) {
                sorted.push(index);
            }
            visiting.remove(&index);
            visited.insert(index);
            if let Some(next) = remaining.pop() {
                sort(
                    next,
                    scripts,
                    dependencies,
                    lookup,
                    remaining,
                    sorted,
                    visited,
                    visiting,
                )?;
            }
            Ok(())
        }
        if let Some(next) = remaining.pop() {
            sort(
                next,
                scripts,
                &dependencies,
                &lookup,
                &mut remaining,
                &mut sorted,
                &mut visited,
                &mut visiting,
            )?;
        }
        for index in sorted {
            attempt(
                vm,
                scripts,
                index,
                &dependencies,
                &mut pending,
                &mut results,
            )?;
        }
        for (index, script) in scripts.iter().enumerate() {
            let deps: Table = dependencies.raw_get(script.name)?;
            results[index].missing_dependencies = deps
                .pairs::<String, bool>()
                .map(|value| value.map(|(name, _)| name))
                .collect::<Result<_>>()?;
        }
        Ok(())
    };
    if let Err(error) = run() {
        let error = vm.script_error(error);
        for result in &mut results {
            if !result.completed {
                result.error = Some(error.clone());
            }
        }
    }
    results
}

fn attempt(
    vm: &ScriptVm,
    scripts: &[ScriptAssetRegistration<'_>],
    index: usize,
    dependencies: &Table,
    pending: &mut BTreeSet<usize>,
    results: &mut [ScriptAssetRegistrationResult],
) -> Result<()> {
    let script = &scripts[index];
    let registered = (|| -> Result<Option<RuntimeScriptProgram>> {
        let cached = vm.registered_module(script.name)?;
        let value = if matches!(cached, Value::Nil) {
            // ScriptAsset has already stripped/verified the signed envelope.
            let chunk = vm.load_bytecode(script.name, script.bytecode)?;
            vm.reset_execution_budget();
            let values: MultiValue = vm.execute_loaded_module(script.name, chunk)?;
            let value = values.back().cloned().ok_or_else(|| {
                Error::runtime(format!("{}:1: module must return a value", script.name))
            })?;
            if !matches!(value, Value::Table(_) | Value::Function(_)) {
                return Err(Error::runtime(format!(
                    "{}:1: module must return a table or function",
                    script.name
                )));
            }
            if !script.is_protocol {
                vm.cache_registered_module(script.name, value.clone())?;
            }
            value
        } else {
            cached
        };
        Ok(if script.is_protocol {
            match value {
                Value::Function(generator) => Some(RuntimeScriptProgram::from_backend(
                    RegisteredProtocolProgram {
                        vm: vm.runtime_identity.clone(),
                        program: ScriptProgram { generator },
                    },
                )),
                _ => None, // Successful table return registers no generator ref.
            }
        } else {
            None
        })
    })();
    for (candidate, script) in scripts.iter().enumerate() {
        let deps: Table = dependencies.raw_get(script.name)?;
        if deps.pairs::<String, bool>().next().is_some() {
            pending.insert(candidate);
        }
    }
    match registered {
        Ok(program) => {
            results[index].completed = true;
            results[index].program = program;
            results[index].error = None;
            for candidate in scripts {
                let deps: Table = dependencies.raw_get(candidate.name)?;
                deps.raw_set(script.name, Value::Nil)?;
            }
            pending.remove(&index);
        }
        Err(error) => results[index].error = Some(vm.script_error(error)),
    }
    Ok(())
}
