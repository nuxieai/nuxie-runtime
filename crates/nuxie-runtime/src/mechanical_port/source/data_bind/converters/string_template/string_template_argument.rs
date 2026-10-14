use crate::source::{
    core::CoreHandle,
    generated::data_bind::converters::string_template::string_template_argument_base::StringTemplateArgumentBase,
    importers::{backboard_importer::BackboardImporter, import_stack::ImportStack},
    status_code::StatusCode,
};
#[derive(Default)]
pub struct StringTemplateArgument {
    pub base: StringTemplateArgumentBase,
    string_template: Option<CoreHandle>,
}
impl std::ops::Deref for StringTemplateArgument {
    type Target = StringTemplateArgumentBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for StringTemplateArgument {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl StringTemplateArgument {
    pub fn set_string_template(&mut self, value: Option<CoreHandle>) {
        self.string_template = value;
    }
    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(importer) = stack.latest::<BackboardImporter>(
            crate::source::generated::backboard_base::BackboardBase::TYPE_KEY,
        ) else {
            return StatusCode::MissingObject;
        };
        let Some(converter) = importer.last_data_converter() else {
            return StatusCode::MissingObject;
        };
        if !converter.is_type_of(177) {
            return StatusCode::MissingObject;
        }
        let code = self.base.base.import(stack);
        if code != StatusCode::Ok {
            return code;
        }
        let Some(argument) = self.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        self.string_template = Some(converter.clone());
        converter.with_downcast_mut::<super::super::data_converter_string_template::DataConverterStringTemplate, _>(|converter| converter.add_imported_argument(argument));
        StatusCode::Ok
    }
    pub fn add_data_bind(&mut self, data_bind: CoreHandle) {
        if let Some(converter) = &self.string_template {
            crate::source::data_bind::data_bind_container::DataBindContainerOwner::Authored(
                converter.clone(),
            )
            .add_data_bind(data_bind);
        }
    }
}
