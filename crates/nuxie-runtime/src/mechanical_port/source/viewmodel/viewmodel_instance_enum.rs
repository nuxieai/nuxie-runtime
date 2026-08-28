use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    data_bind::data_values::data_value_integer::DataValueInteger,
    generated::viewmodel::viewmodel_instance_enum_base::{
        ViewModelInstanceEnumBase, ViewModelInstanceEnumBaseCallbacks,
    },
};

#[derive(Default)]
pub struct ViewModelInstanceEnum {
    pub base: ViewModelInstanceEnumBase,
    #[cfg(feature = "tools")]
    changed_callback: Option<fn(&mut Self, u32)>,
}

impl ViewModelInstanceEnum {
    fn enum_property(&self) -> Option<crate::mechanical_port::source::core::CoreHandle> {
        self.base.base.view_model_property()
    }

    pub fn value(&self) -> String {
        self.enum_property()
            .and_then(|property| {
                property
                    .with(|property| {
                        property
                            .as_view_model_property_enum()
                            .map(|property| property.value_at(self.base.property_value()))
                    })
                    .flatten()
            })
            .unwrap_or_default()
    }

    pub fn values(&self) -> Vec<String> {
        self.enum_property()
            .and_then(|property| {
                property
                    .with(|property| {
                        let property = property.as_view_model_property_enum()?;
                        let data = property.data_enum()?;
                        data.with_downcast::<super::data_enum::DataEnum, _>(|data| {
                            (0..data.values().len() as u32)
                                .map(|index| data.value_by_index(index))
                                .collect()
                        })
                    })
                    .flatten()
            })
            .unwrap_or_default()
    }

    pub fn enum_type(&self) -> String {
        self.enum_property()
            .and_then(|property| {
                property
                    .with(|property| {
                        property
                            .as_view_model_property_enum()?
                            .data_enum()?
                            .with_downcast::<super::data_enum::DataEnum, _>(|data| {
                            data.enum_name().to_owned()
                        })
                    })
                    .flatten()
            })
            .unwrap_or_default()
    }

    fn set_property_value(&mut self, value: u32) {
        if self.base.set_property_value_value(value) {
            self.property_value_changed();
            self.base
                .base
                .base
                .base
                .base
                .base
                .notify_property_changed(ViewModelInstanceEnumBase::PROPERTY_VALUE_PROPERTY_KEY);
        }
    }

    pub fn property_value_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::BINDINGS);
        #[cfg(feature = "tools")]
        if let Some(callback) = self.changed_callback {
            callback(self, self.base.property_value());
        }
        self.base.on_value_changed();
    }

    pub fn set_value_named(&mut self, name: &str) -> bool {
        let enum_property = self.base.view_model_property_enum();
        let index = enum_property.value_index_named(name);
        if index == -1 {
            return false;
        }
        self.set_property_value(index as u32);
        true
    }

    pub fn set_value_at(&mut self, index: u32) -> bool {
        let enum_property = self.base.view_model_property_enum();
        if enum_property.value_index_at(index) == -1 {
            return false;
        }
        self.set_property_value(index);
        true
    }

    pub fn apply_value(&mut self, data_value: &DataValueInteger) {
        self.set_property_value(data_value.value());
    }

    #[cfg(feature = "tools")]
    pub fn on_changed(&mut self, callback: Option<fn(&mut Self, u32)>) {
        self.changed_callback = callback;
    }
}

impl ViewModelInstanceEnumBaseCallbacks for ViewModelInstanceEnum {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }

    fn property_value_changed(&mut self) {
        Self::property_value_changed(self);
    }
}
