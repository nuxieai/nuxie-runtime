use super::string_template::{
    string_template_argument_number::StringTemplateArgumentNumber,
    string_template_argument_string::StringTemplateArgumentString,
    string_template_format::{
        StringTemplateFormat, StringTemplateSegment, StringTemplateSegmentKind,
    },
};
use crate::source::{
    core::{CoreHandle, CoreObject},
    data_bind::{
        converters::data_converter::DataConverter,
        data_values::{
            data_type::DataType, data_value::DataValue, data_value_string::DataValueString,
        },
    },
    generated::data_bind::converters::data_converter_string_template_base::DataConverterStringTemplateBase,
};

pub struct DataConverterStringTemplate {
    pub base: DataConverterStringTemplateBase,
    arguments: Vec<CoreHandle>,
    segments: Vec<StringTemplateSegment>,
    parsed_text: String,
    segments_dirty: bool,
    output: DataValueString,
}
impl Default for DataConverterStringTemplate {
    fn default() -> Self {
        Self {
            base: Default::default(),
            arguments: Vec::new(),
            segments: Vec::new(),
            parsed_text: String::new(),
            segments_dirty: true,
            output: Default::default(),
        }
    }
}
impl DataConverterStringTemplate {
    pub fn output_type(&self) -> DataType {
        DataType::String
    }
    pub fn arguments(&self) -> &[CoreHandle] {
        &self.arguments
    }
    pub(crate) fn attach_handle(&mut self, owner: CoreHandle) {
        self.core_mut().set_handle(owner.clone());
        self.base.base.data_binds.set_owner(owner.clone());
        // Rust construction can precede arena insertion. C++ already has its
        // stable `this` when addArgument runs; install that same parent identity
        // for pre-existing children as soon as this occurrence acquires it.
        for argument in &self.arguments {
            argument.with_mut(|argument| {
                argument
                    .as_string_template_argument_mut()
                    .unwrap()
                    .set_string_template(Some(owner.clone()));
            });
        }
    }
    pub fn add_argument(&mut self, argument: CoreHandle) {
        let owner = self.core().handle();
        argument.with_mut(|argument| {
            argument
                .as_string_template_argument_mut()
                .unwrap()
                .set_string_template(owner)
        });
        self.arguments.push(argument);
    }
    // Import already holds the argument receiver. Attach only after its base
    // import succeeds, without trying to borrow that receiver a second time.
    pub(crate) fn add_imported_argument(&mut self, argument: CoreHandle) {
        self.arguments.push(argument);
    }
    pub fn set_open_delimiter(&mut self, value: String) {
        if self.base.set_open_delimiter_value(value) {
            self.open_delimiter_changed();
            self.core_mut().notify_property_changed(485);
        }
    }
    pub fn set_close_delimiter(&mut self, value: String) {
        if self.base.set_close_delimiter_value(value) {
            self.close_delimiter_changed();
            self.core_mut().notify_property_changed(486);
        }
    }
    pub fn open_delimiter_changed(&mut self) {
        self.segments_dirty = true;
        self.base.base.mark_converter_dirty();
    }
    pub fn close_delimiter_changed(&mut self) {
        self.segments_dirty = true;
        self.base.base.mark_converter_dirty();
    }
    pub fn convert<'a>(&'a mut self, input: &dyn DataValue) -> &'a dyn DataValue {
        self.convert_optional(Some(input))
    }
    pub fn convert_optional<'a>(&'a mut self, input: Option<&dyn DataValue>) -> &'a dyn DataValue {
        let Some(input) = input.and_then(|v| v.as_any().downcast_ref::<DataValueString>()) else {
            self.output.set_value(String::new());
            return &self.output;
        };
        let text = input.value();
        if self.segments_dirty || text != self.parsed_text {
            StringTemplateFormat::parse(
                text,
                self.base.open_delimiter(),
                self.base.close_delimiter(),
                &mut self.segments,
            );
            self.parsed_text = text.to_owned();
            self.segments_dirty = false;
        }
        let mut output = String::new();
        for segment in &self.segments {
            let argument = match segment.kind {
                StringTemplateSegmentKind::Literal => {
                    output.push_str(&segment.text);
                    continue;
                }
                StringTemplateSegmentKind::Name => self.arguments.iter().find(|argument| {
                    argument
                        .with(|a| a.as_string_template_argument().unwrap().name() == segment.name)
                        .unwrap_or(false)
                }),
                StringTemplateSegmentKind::Position | StringTemplateSegmentKind::Next => {
                    self.arguments.get(segment.index as usize)
                }
            };
            if let Some(argument) = argument {
                argument.with(|a| {
                    if let Some(a) = a.as_any().downcast_ref::<StringTemplateArgumentNumber>() {
                        a.append_value(&mut output, &segment.spec);
                    } else if let Some(a) =
                        a.as_any().downcast_ref::<StringTemplateArgumentString>()
                    {
                        a.append_value(&mut output, &segment.spec);
                    }
                });
            }
        }
        self.output.set_value(output);
        &self.output
    }
    pub fn clone_definition(&self) -> Self {
        let mut cloned = Self::default();
        cloned.base.copy(&self.base);
        cloned
    }
    pub fn complete_clone(source: &CoreHandle, cloned: &CoreHandle) -> bool {
        if !DataConverter::complete_clone(source, cloned) {
            return false;
        }
        let Some(arguments) = source.with_downcast::<Self, _>(|source| source.arguments.clone())
        else {
            return false;
        };
        let source_binds = source.data_bind_container().unwrap().data_binds().to_vec();
        let cloned_binds = cloned.data_bind_container().unwrap().data_binds().to_vec();
        for argument in arguments {
            let Some(cloned_argument) = argument.clone_occurrence() else {
                return false;
            };
            cloned.with_downcast_mut::<Self, _>(|cloned| {
                cloned.add_argument(cloned_argument.clone())
            });
            for (index, bind) in source_binds.iter().enumerate() {
                if bind
                    .with(|bind| bind.as_data_bind().unwrap().target())
                    .flatten()
                    .as_ref()
                    == Some(&argument)
                {
                    cloned_binds[index].with_mut(|bind| {
                        bind.as_data_bind_mut()
                            .unwrap()
                            .set_target(Some(cloned_argument.clone()))
                    });
                }
            }
        }
        true
    }
}
impl Drop for DataConverterStringTemplate {
    fn drop(&mut self) {
        self.base.base.unbind();
        for argument in &self.arguments {
            argument.remove_occurrence();
        }
        self.arguments.clear();
    }
}
crate::impl_data_converter_capability_forward!(DataConverterStringTemplate, base.base);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{core::CoreArena, data_bind::data_bind::DataBind};

    #[test]
    fn authored_arguments_acquire_converter_identity_on_arena_insertion() {
        let arena = CoreArena::default();
        let argument = arena.insert(StringTemplateArgumentNumber::new("score".into(), 1.0));
        let mut converter = DataConverterStringTemplate::default();
        converter.add_argument(argument.clone());
        let converter = arena.insert(converter);
        let bind = arena.insert(DataBind::default());
        bind.with_mut(|bind| {
            bind.as_data_bind_mut()
                .unwrap()
                .set_target(Some(argument.clone()));
        });
        argument.with_mut(|argument| {
            argument
                .as_string_template_argument_mut()
                .unwrap()
                .add_data_bind(bind.clone());
        });
        assert_eq!(
            converter
                .data_bind_container()
                .unwrap()
                .data_binds()
                .to_vec(),
            vec![bind]
        );
    }
}
