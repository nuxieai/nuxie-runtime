use crate::mechanical_port::source::{
    core::CoreHandle,
    data_bind::data_context::RuntimeDataContextHandle,
    data_bind::data_values::{data_type::DataType, data_value::DataValue},
    generated::data_bind::converters::data_converter_group_base::DataConverterGroupBase,
};
pub trait GroupConverter {
    fn convert(
        &mut self,
        value: &dyn DataValue,
        data_bind: &CoreHandle,
        output: &mut dyn FnMut(&dyn DataValue),
    );
    fn reverse_convert(
        &mut self,
        value: &dyn DataValue,
        data_bind: &CoreHandle,
        output: &mut dyn FnMut(&dyn DataValue),
    );
    fn output_type(&self) -> DataType;
    fn bind_from_context(&mut self, context: RuntimeDataContextHandle, data_bind: CoreHandle);
    fn unbind(&mut self);
    fn update(&mut self);
    fn reset(&mut self);
    fn advance(&mut self, elapsed: f32) -> bool;
}
#[derive(Default)]
pub struct DataConverterGroup {
    pub base: DataConverterGroupBase,
    items: Vec<CoreHandle>,
}
impl DataConverterGroup {
    pub fn add_item(&mut self, item: CoreHandle) {
        self.items.push(item)
    }
    pub fn convert(
        &mut self,
        value: &dyn DataValue,
        data_bind: &CoreHandle,
        output: &mut dyn FnMut(&dyn DataValue),
    ) {
        convert_chain(&self.items, 0, value, data_bind, false, output);
    }
    pub fn reverse_convert(
        &mut self,
        value: &dyn DataValue,
        data_bind: &CoreHandle,
        output: &mut dyn FnMut(&dyn DataValue),
    ) {
        convert_chain(
            &self.items,
            self.items.len(),
            value,
            data_bind,
            true,
            output,
        );
    }
    pub fn output_type(&self, super_output: DataType) -> DataType {
        for item in self.items.iter().rev() {
            let converter = item
                .with(|item| {
                    item.as_data_converter_group_item()
                        .and_then(|item| item.converter())
                })
                .flatten();
            if let Some(converter) = converter {
                let output = converter
                    .with(|converter| {
                        converter
                            .as_data_converter_capability()
                            .map(|converter| converter.output_type())
                    })
                    .flatten()
                    .unwrap_or(DataType::Input);
                if output != DataType::Input {
                    return output;
                }
            }
        }
        super_output
    }
    pub fn items(&self) -> &[CoreHandle] {
        &self.items
    }
    pub fn clone_group(&self) -> Self {
        let mut cloned = Self::default();
        cloned.base.base.copy(&self.base.base);
        for item in &self.items {
            let has_converter = item
                .with(|item| {
                    item.as_data_converter_group_item()
                        .is_some_and(|item| item.converter().is_some())
                })
                .unwrap_or(false);
            if has_converter && let Some(item) = item.clone_occurrence() {
                cloned.add_item(item);
            }
        }
        cloned
    }
    pub fn bind_from_context(&mut self, context: RuntimeDataContextHandle, data_bind: CoreHandle) {
        self.base
            .base
            .bind_from_context(context.clone(), data_bind.clone());
        for item in &self.items {
            let converter = item
                .with(|item| {
                    item.as_data_converter_group_item()
                        .and_then(|item| item.converter())
                })
                .flatten();
            if let Some(converter) = converter {
                converter.with_mut(|converter| {
                    if let Some(converter) = converter.as_data_converter_capability_mut() {
                        converter.bind_from_context(context.clone(), data_bind.clone());
                    }
                });
            }
        }
    }
    pub fn unbind(&mut self) {
        for item in &self.items {
            let converter = item
                .with(|item| {
                    item.as_data_converter_group_item()
                        .and_then(|item| item.converter())
                })
                .flatten();
            if let Some(converter) = converter {
                converter.with_mut(|converter| {
                    if let Some(converter) = converter.as_data_converter_capability_mut() {
                        converter.unbind();
                    }
                });
            }
        }
    }
    pub fn update(&mut self) {
        for item in &self.items {
            let converter = item
                .with(|item| {
                    item.as_data_converter_group_item()
                        .and_then(|item| item.converter())
                })
                .flatten();
            if let Some(converter) = converter {
                converter.with_mut(|converter| {
                    if let Some(converter) = converter.as_data_converter_capability_mut() {
                        converter.update();
                    }
                });
            }
        }
    }
    pub fn reset(&mut self) {
        for item in &self.items {
            let converter = item
                .with(|item| {
                    item.as_data_converter_group_item()
                        .and_then(|item| item.converter())
                })
                .flatten();
            if let Some(converter) = converter {
                converter.with_mut(|converter| {
                    if let Some(converter) = converter.as_data_converter_capability_mut() {
                        converter.reset();
                    }
                });
            }
        }
    }
    pub fn advance(&mut self, elapsed: f32) -> bool {
        let mut did_update = false;
        for item in &self.items {
            let converter = item
                .with(|item| {
                    item.as_data_converter_group_item()
                        .and_then(|item| item.converter())
                })
                .flatten();
            if let Some(converter) = converter {
                did_update |= converter
                    .with_mut(|converter| {
                        converter
                            .as_data_converter_capability_mut()
                            .is_some_and(|converter| converter.advance(elapsed))
                    })
                    .unwrap_or(false);
            }
        }
        did_update
    }
}

fn convert_chain(
    items: &[CoreHandle],
    index: usize,
    value: &dyn DataValue,
    data_bind: &CoreHandle,
    reverse: bool,
    output: &mut dyn FnMut(&dyn DataValue),
) {
    let next = if reverse {
        index.checked_sub(1)
    } else if index < items.len() {
        Some(index)
    } else {
        None
    };
    let Some(next) = next else {
        output(value);
        return;
    };
    let converter = items[next]
        .with(|item| {
            item.as_data_converter_group_item()
                .and_then(|item| item.converter())
        })
        .flatten();
    let following = if reverse { next } else { next + 1 };
    let Some(converter) = converter else {
        convert_chain(items, following, value, data_bind, reverse, output);
        return;
    };
    let dispatched = converter
        .with_mut(|converter| {
            let Some(converter) = converter.as_data_converter_capability_mut() else {
                return false;
            };
            let mut next_output = |converted: &dyn DataValue| {
                convert_chain(items, following, converted, data_bind, reverse, output)
            };
            if reverse {
                converter.reverse_convert(value, data_bind, &mut next_output);
            } else {
                converter.convert(value, data_bind, &mut next_output);
            }
            true
        })
        .unwrap_or(false);
    if !dispatched {
        convert_chain(items, following, value, data_bind, reverse, output);
    }
}

impl crate::mechanical_port::source::generated::core_registry::DataConverterCapability
    for DataConverterGroup
{
    fn convert(
        &mut self,
        input: &dyn DataValue,
        data_bind: &CoreHandle,
        output: &mut dyn FnMut(&dyn DataValue),
    ) {
        Self::convert(self, input, data_bind, output);
    }

    fn reverse_convert(
        &mut self,
        input: &dyn DataValue,
        data_bind: &CoreHandle,
        output: &mut dyn FnMut(&dyn DataValue),
    ) {
        Self::reverse_convert(self, input, data_bind, output);
    }

    fn output_type(&self) -> DataType {
        Self::output_type(self, self.base.base.output_type())
    }

    fn bind_from_context(&mut self, context: RuntimeDataContextHandle, data_bind: CoreHandle) {
        Self::bind_from_context(self, context, data_bind);
    }

    fn unbind(&mut self) {
        Self::unbind(self);
    }

    fn update(&mut self) {
        Self::update(self);
    }

    fn reset(&mut self) {
        Self::reset(self);
    }

    fn advance(&mut self, elapsed: f32) -> bool {
        Self::advance(self, elapsed)
    }
}
