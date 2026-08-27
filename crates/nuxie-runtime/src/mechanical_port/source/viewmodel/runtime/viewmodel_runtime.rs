use super::viewmodel_instance_runtime::{
    PropertyData, ViewModelInstanceRuntime, ViewModelInstanceSource,
};
use super::viewmodel_instance_value_runtime::DataType;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreType {
    String,
    Number,
    Boolean,
    Color,
    List,
    Enum,
    EnumCustom,
    EnumSystem,
    Trigger,
    ViewModel,
    SymbolListIndex,
    AssetImage,
    AssetFont,
    AssetBlob,
    Artboard,
    Other,
}
pub trait ViewModelPropertySource {
    fn core_type(&self) -> CoreType;
    fn name(&self) -> &str;
    fn enum_name(&self) -> Option<&str>;
}
pub trait ViewModelSource<I> {
    fn name(&self) -> &str;
    fn instance_count(&self) -> usize;
    fn properties(&self) -> Vec<&dyn ViewModelPropertySource>;
    fn instances(&self) -> Vec<Rc<I>>;
    fn instance_at(&self, index: usize) -> Option<Rc<I>>;
    fn instance_named(&self, name: &str) -> Option<Rc<I>>;
}
pub trait FileSource<V, I> {
    fn clone_instance(&self, instance: &Rc<I>) -> Rc<I>;
    fn complete_view_model_instance(&self, instance: &Rc<I>);
    fn create_default_view_model_instance(&self, view_model: &V) -> Option<Rc<I>>;
    fn create_view_model_instance(&self, view_model: &V) -> Rc<I>;
}

pub struct ViewModelRuntime<V, F, I> {
    view_model: Rc<V>,
    file: Rc<F>,
    _instance: core::marker::PhantomData<I>,
}
impl<V, F, I> ViewModelRuntime<V, F, I>
where
    V: ViewModelSource<I>,
    F: FileSource<V, I>,
    I: ViewModelInstanceSource + 'static,
{
    pub fn new(view_model: Rc<V>, file: Rc<F>) -> Self {
        Self {
            view_model,
            file,
            _instance: core::marker::PhantomData,
        }
    }
    pub fn instance_count(&self) -> usize {
        self.view_model.instance_count()
    }
    pub fn property_count(&self) -> usize {
        self.view_model.properties().len()
    }
    pub fn name(&self) -> &str {
        self.view_model.name()
    }
    pub fn build_properties_data(
        properties: Vec<&dyn ViewModelPropertySource>,
    ) -> Vec<PropertyData> {
        let mut output = Vec::with_capacity(properties.len());
        for property in properties {
            let mut data_type = DataType::None;
            let mut enum_name = String::new();
            match property.core_type() {
                CoreType::String => data_type = DataType::String,
                CoreType::Number => data_type = DataType::Number,
                CoreType::Boolean => data_type = DataType::Boolean,
                CoreType::Color => data_type = DataType::Color,
                CoreType::List => data_type = DataType::List,
                CoreType::Enum | CoreType::EnumCustom | CoreType::EnumSystem => {
                    data_type = DataType::Enum;
                    if let Some(name) = property.enum_name() {
                        enum_name = name.to_owned();
                    }
                }
                CoreType::Trigger => data_type = DataType::Trigger,
                CoreType::ViewModel => data_type = DataType::ViewModel,
                CoreType::SymbolListIndex => data_type = DataType::SymbolListIndex,
                CoreType::AssetImage => data_type = DataType::AssetImage,
                CoreType::AssetFont => data_type = DataType::AssetFont,
                CoreType::AssetBlob => data_type = DataType::AssetBlob,
                CoreType::Artboard => data_type = DataType::Artboard,
                CoreType::Other => {}
            }
            output.push(PropertyData {
                data_type,
                name: property.name().to_owned(),
                enum_name,
            });
        }
        output
    }
    pub fn properties(&self) -> Vec<PropertyData> {
        Self::build_properties_data(self.view_model.properties())
    }
    pub fn instance_names(&self) -> Vec<String> {
        self.view_model
            .instances()
            .into_iter()
            .map(|instance| instance.name().to_owned())
            .collect()
    }
    pub fn create_instance_from_index(
        &self,
        index: usize,
    ) -> Option<Rc<ViewModelInstanceRuntime<I>>> {
        if let Some(instance) = self.view_model.instance_at(index) {
            let copy = self.file.clone_instance(&instance);
            self.file.complete_view_model_instance(&copy);
            return self.create_runtime_instance(Some(copy));
        }
        eprintln!("Could not find View Model Instance. Index {index} is out of range.");
        None
    }
    pub fn create_instance_from_name(&self, name: &str) -> Option<Rc<ViewModelInstanceRuntime<I>>> {
        if let Some(instance) = self.view_model.instance_named(name) {
            let copy = self.file.clone_instance(&instance);
            self.file.complete_view_model_instance(&copy);
            return self.create_runtime_instance(Some(copy));
        }
        eprintln!(
            "Could not find View Model Instance named {name}. Was it marked to export with the file?"
        );
        None
    }
    pub fn create_default_instance(&self) -> Rc<ViewModelInstanceRuntime<I>> {
        if let Some(instance) = self
            .file
            .create_default_view_model_instance(&self.view_model)
        {
            return self.create_runtime_instance(Some(instance)).unwrap();
        }
        eprintln!("Default instance not found. Creating empty instance instead.");
        self.create_instance()
    }
    pub fn create_instance(&self) -> Rc<ViewModelInstanceRuntime<I>> {
        let instance = self.file.create_view_model_instance(&self.view_model);
        self.create_runtime_instance(Some(instance)).unwrap()
    }
    fn create_runtime_instance(
        &self,
        instance: Option<Rc<I>>,
    ) -> Option<Rc<ViewModelInstanceRuntime<I>>> {
        instance.map(ViewModelInstanceRuntime::new)
    }
}
