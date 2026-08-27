// Direct Rust owner for pinned C++
// `src/viewmodel/runtime/viewmodel_runtime.cpp`.
// Retains the exact RuntimeFile allocation and one authored ViewModel index.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewModelRuntimeDataType {
    None,
    String,
    Number,
    Boolean,
    Color,
    List,
    Enum,
    Trigger,
    ViewModel,
    SymbolListIndex,
    AssetImage,
    AssetFont,
    AssetBlob,
    Artboard,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewModelRuntimeProperty {
    pub data_type: ViewModelRuntimeDataType,
    pub name: String,
    pub enum_name: String,
}

#[derive(Debug, Clone)]
pub struct ViewModelRuntime {
    file: Rc<RuntimeFile>,
    view_model_index: usize,
}

impl ViewModelRuntime {
    pub fn new(file: Rc<RuntimeFile>, view_model_index: usize) -> Option<Self> {
        file.view_model(view_model_index)?;
        Some(Self {
            file,
            view_model_index,
        })
    }

    pub fn named(file: Rc<RuntimeFile>, name: &str) -> Option<Self> {
        let view_model_index = file
            .view_models()
            .iter()
            .position(|view_model| view_model.object.string_property("name") == Some(name))?;
        Self::new(file, view_model_index)
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.view_model_index == other.view_model_index && Rc::ptr_eq(&self.file, &other.file)
    }

    pub fn file(&self) -> &RuntimeFile {
        &self.file
    }

    pub fn view_model_index(&self) -> usize {
        self.view_model_index
    }

    fn authored_view_model(&self) -> RuntimeAuthoredViewModel<'_> {
        RuntimeAuthoredViewModel::new(&self.file, self.view_model_index)
            .expect("ViewModelRuntime retains a validated ViewModel index")
    }

    pub fn instance_count(&self) -> usize {
        self.authored_view_model().instance_count()
    }

    pub fn property_count(&self) -> usize {
        self.authored_view_model().properties().len()
    }

    pub fn name(&self) -> String {
        self.file
            .view_model(self.view_model_index)
            .and_then(|view_model| view_model.object.string_property("name"))
            .unwrap_or_default()
            .to_owned()
    }

    pub fn properties(&self) -> Vec<ViewModelRuntimeProperty> {
        Self::build_properties_data(&self.file, self.view_model_index)
    }

    pub fn instance_names(&self) -> Vec<String> {
        self.authored_view_model()
            .instances()
            .into_iter()
            .map(|instance| {
                instance
                    .object
                    .string_property("name")
                    .unwrap_or_default()
                    .to_owned()
            })
            .collect()
    }

    pub fn create_instance_from_index(&self, index: usize) -> Option<ViewModelInstanceRuntime> {
        let instance = self.authored_view_model().instance(index)?;
        let instance = RuntimeOwnedViewModelInstance::from_instance(
            &self.file,
            self.view_model_index,
            instance.instance_index,
        )?;
        Some(ViewModelInstanceRuntime::from_handle(
            Rc::clone(&self.file),
            RuntimeOwnedViewModelHandle::new(instance),
        ))
    }

    pub fn create_instance_from_name(&self, name: &str) -> Option<ViewModelInstanceRuntime> {
        let instance = self.authored_view_model().instance_named(name)?;
        let instance = RuntimeOwnedViewModelInstance::from_instance(
            &self.file,
            self.view_model_index,
            instance.instance_index,
        )?;
        Some(ViewModelInstanceRuntime::from_handle(
            Rc::clone(&self.file),
            RuntimeOwnedViewModelHandle::new(instance),
        ))
    }

    pub fn create_default_instance(&self) -> Option<ViewModelInstanceRuntime> {
        match self.authored_view_model().default_instance() {
            Some(instance) => self.create_instance_from_index(instance.instance_index),
            None => self.create_instance(),
        }
    }

    pub fn create_instance(&self) -> Option<ViewModelInstanceRuntime> {
        let instance = self.authored_view_model().create_instance()?;
        Some(ViewModelInstanceRuntime::from_handle(
            Rc::clone(&self.file),
            RuntimeOwnedViewModelHandle::new(instance),
        ))
    }

    fn build_properties_data(
        file: &RuntimeFile,
        view_model_index: usize,
    ) -> Vec<ViewModelRuntimeProperty> {
        RuntimeAuthoredViewModel::new(file, view_model_index)
            .map(|view_model| {
                view_model
                    .properties()
                    .into_iter()
                    .map(|property| {
                        let data_type = match property.type_name {
                            "ViewModelPropertyString" => ViewModelRuntimeDataType::String,
                            "ViewModelPropertyNumber" => ViewModelRuntimeDataType::Number,
                            "ViewModelPropertyBoolean" => ViewModelRuntimeDataType::Boolean,
                            "ViewModelPropertyColor" => ViewModelRuntimeDataType::Color,
                            "ViewModelPropertyList" => ViewModelRuntimeDataType::List,
                            "ViewModelPropertyEnum"
                            | "ViewModelPropertyEnumCustom"
                            | "ViewModelPropertyEnumSystem" => ViewModelRuntimeDataType::Enum,
                            "ViewModelPropertyTrigger" => ViewModelRuntimeDataType::Trigger,
                            "ViewModelPropertyViewModel" => ViewModelRuntimeDataType::ViewModel,
                            "ViewModelPropertySymbolListIndex" => {
                                ViewModelRuntimeDataType::SymbolListIndex
                            }
                            "ViewModelPropertyAssetImage" => ViewModelRuntimeDataType::AssetImage,
                            "ViewModelPropertyAssetFont" => ViewModelRuntimeDataType::AssetFont,
                            "ViewModelPropertyAssetBlob" => ViewModelRuntimeDataType::AssetBlob,
                            "ViewModelPropertyArtboard" => ViewModelRuntimeDataType::Artboard,
                            _ => ViewModelRuntimeDataType::None,
                        };
                        let enum_name = if data_type == ViewModelRuntimeDataType::Enum {
                            file.data_enum_for_view_model_property_object(property)
                                .and_then(|data_enum| data_enum.object.string_property("name"))
                                .unwrap_or_default()
                                .to_owned()
                        } else {
                            String::new()
                        };
                        ViewModelRuntimeProperty {
                            data_type,
                            name: property
                                .string_property("name")
                                .unwrap_or_default()
                                .to_owned(),
                            enum_name,
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod viewmodel_runtime_tests {
    use super::viewmodel_instance_runtime_identity_tests::runtime_family_file;
    use super::*;

    #[test]
    fn facade_reports_authored_schema_and_creates_empty_instances() {
        let file = runtime_family_file();
        let runtime = ViewModelRuntime::new(Rc::clone(&file), 0).expect("runtime");
        assert_eq!(runtime.name(), "Root");
        assert_eq!(runtime.instance_count(), 0);
        assert_eq!(runtime.property_count(), 3);
        assert_eq!(
            runtime
                .properties()
                .into_iter()
                .map(|property| (property.data_type, property.name))
                .collect::<Vec<_>>(),
            vec![
                (ViewModelRuntimeDataType::Number, "count".to_owned()),
                (ViewModelRuntimeDataType::List, "items".to_owned()),
                (ViewModelRuntimeDataType::ViewModel, "child".to_owned()),
            ]
        );
        assert!(runtime.create_instance_from_index(0).is_none());
        assert!(runtime.create_instance_from_name("missing").is_none());
        assert!(runtime.create_default_instance().is_some());
        assert!(runtime.ptr_eq(
            &ViewModelRuntime::named(Rc::clone(&file), "Root").expect("named runtime")
        ));
        assert!(!runtime.ptr_eq(
            &ViewModelRuntime::new(file, 1).expect("different ViewModel")
        ));
    }
}
