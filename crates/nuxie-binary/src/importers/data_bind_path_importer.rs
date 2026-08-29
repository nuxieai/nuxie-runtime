use super::*;

/// Rust retains a stable file index instead of the pinned raw `DataBindPath*`.
struct DataBindPathImporter {
    data_bind_path: Option<usize>,
}

impl DataBindPathImporter {
    /// Mechanical translation of `DataBindPathImporter(DataBindPath*)`.
    fn new(data_bind_path: usize) -> Self {
        Self {
            data_bind_path: Some(data_bind_path),
        }
    }

    /// Mechanical translation of `claim()`: return the retained path once and
    /// leave this importer latest with a null path thereafter.
    fn claim(&mut self) -> Option<usize> {
        self.data_bind_path.take()
    }
}

pub(super) fn dispatch_imports_successfully(
    object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    if definition.is_a("DataBindPath") {
        return Some(
            imports_successfully(object, definition, context)
                .expect("path is owned by DataBindPathImporter"),
        );
    }
    None
}

pub(super) fn dispatch_update_context(
    definition: &'static Definition,
    context: &mut ImportContext,
) {
    if definition.is_a("DataBindPath") {
        update_context(definition, context);
    }
}

pub(super) fn imports_successfully(
    _object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    definition
        .is_a("DataBindPath")
        .then(|| context.latest(ImportStackKey::Backboard))
}

pub(super) fn update_context(definition: &'static Definition, context: &mut ImportContext) {
    if definition.is_a("DataBindPath") {
        context.make_latest(ImportStackKey::DataBindPath);
    }
}
impl RuntimeFile {
    pub(crate) fn cpp_claimed_data_bind_path_for(
        &self,
        referencer_id: usize,
    ) -> Option<&RuntimeObject> {
        let mut latest = None::<DataBindPathImporter>;

        for (file_index, object) in self.objects.iter().enumerate() {
            let Some(object) = object.as_ref() else {
                continue;
            };

            if object.type_name == "DataBindPath" {
                if self.import_status(file_index) == Some(RuntimeImportStatus::Imported) {
                    latest = Some(DataBindPathImporter::new(file_index));
                }
                continue;
            }

            if cpp_claims_latest_data_bind_path(object) {
                let claimed_path = latest.as_mut().and_then(DataBindPathImporter::claim);
                if file_index == referencer_id {
                    return claimed_path.and_then(|path_index| self.object(path_index));
                }
            }
        }

        None
    }

    pub(crate) fn cpp_resolved_data_bind_path_ids(
        &self,
        path_object: &RuntimeObject,
        path_ids: &[u32],
    ) -> Vec<u32> {
        if path_object.type_name != "DataBindPath" || path_ids.len() != 1 {
            return path_ids.to_vec();
        }

        let Some(manifest) = self.manifest() else {
            return path_ids.to_vec();
        };

        manifest
            .resolve_path(path_ids[0])
            .map_or_else(Vec::new, <[u32]>::to_vec)
    }
}
