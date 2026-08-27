// Direct owner for pinned C++ `ViewModel`.
//
// The imported Rust file is immutable, so C++'s owned property and instance
// vectors are retained by `RuntimeFile` and projected through this stable
// file/index pair. Import-time `addProperty`, `addInstance`, owner assignment,
// and destruction are implemented by `nuxie-binary`'s ViewModel importer and
// aggregate object storage; every post-import method remains source-ordered
// here.

#[derive(Clone, Copy)]
pub(crate) struct RuntimeAuthoredViewModel<'a> {
    file: &'a RuntimeFile,
    view_model_index: usize,
}

impl<'a> RuntimeAuthoredViewModel<'a> {
    pub(crate) fn new(file: &'a RuntimeFile, view_model_index: usize) -> Option<Self> {
        file.view_model(view_model_index)?;
        Some(Self {
            file,
            view_model_index,
        })
    }

    pub(crate) fn property(self, index: usize) -> Option<&'a RuntimeObject> {
        self.file
            .view_model(self.view_model_index)?
            .properties
            .into_iter()
            .nth(index)
    }

    pub(crate) fn property_named(self, name: &str) -> Option<&'a RuntimeObject> {
        self.file
            .view_model(self.view_model_index)?
            .properties
            .into_iter()
            .find(|property| property.string_property("name").unwrap_or_default() == name)
    }

    pub(crate) fn property_for_symbol(self, symbol_type: u8) -> Option<&'a RuntimeObject> {
        self.file
            .view_model(self.view_model_index)?
            .properties
            .into_iter()
            .find(|property| {
                property.uint_property("symbolTypeValue").unwrap_or(0) == u64::from(symbol_type)
            })
    }

    pub(crate) fn default_instance(self) -> Option<RuntimeViewModelInstanceReference<'a>> {
        self.instance(0)
    }

    pub(crate) fn instance(self, index: usize) -> Option<RuntimeViewModelInstanceReference<'a>> {
        let instance = self
            .file
            .view_model(self.view_model_index)?
            .instances
            .into_iter()
            .nth(index)?;
        Some(RuntimeViewModelInstanceReference {
            view_model_index: self.view_model_index,
            instance_index: index,
            object: instance.object,
        })
    }

    pub(crate) fn instance_named(
        self,
        name: &str,
    ) -> Option<RuntimeViewModelInstanceReference<'a>> {
        self.file
            .view_model(self.view_model_index)?
            .instances
            .into_iter()
            .enumerate()
            .find_map(|(instance_index, instance)| {
                (instance.object.string_property("name").unwrap_or_default() == name).then_some(
                    RuntimeViewModelInstanceReference {
                        view_model_index: self.view_model_index,
                        instance_index,
                        object: instance.object,
                    },
                )
            })
    }

    pub(crate) fn instance_count(self) -> usize {
        self.instances().len()
    }

    pub(crate) fn create_instance(self) -> Option<RuntimeOwnedViewModelInstance> {
        RuntimeOwnedViewModelInstance::new(self.file, self.view_model_index)
    }

    pub(crate) fn create_from_instance(
        self,
        instance_name: &str,
    ) -> Option<RuntimeOwnedViewModelInstance> {
        let view_model_name = self
            .file
            .view_model(self.view_model_index)?
            .object
            .string_property("name")
            .unwrap_or_default();
        let view_model_index = self.file.view_models().iter().position(|view_model| {
            view_model
                .object
                .string_property("name")
                .unwrap_or_default()
                == view_model_name
        })?;
        let instance = Self::new(self.file, view_model_index)?.instance_named(instance_name)?;
        RuntimeOwnedViewModelInstance::from_instance(
            self.file,
            view_model_index,
            instance.instance_index,
        )
    }

    pub(crate) fn properties(self) -> Vec<&'a RuntimeObject> {
        self.file
            .view_model(self.view_model_index)
            .map(|view_model| view_model.properties)
            .unwrap_or_default()
    }

    pub(crate) fn instances(self) -> Vec<RuntimeViewModelInstance<'a>> {
        self.file
            .view_model(self.view_model_index)
            .map(|view_model| view_model.instances)
            .unwrap_or_default()
    }
}
