use crate::mechanical_port::source::{
    core::CoreHandle,
    data_bind::{
        data_bind_container::DataBindContainer,
        data_context::RuntimeDataContextHandle,
        data_values::{data_type::DataType, data_value::DataValue},
    },
    generated::core_registry::DataConverterCapability,
    generated::data_bind::converters::data_converter_base::{
        DataConverterBase, DataConverterBaseCallbacks,
    },
    status_code::StatusCode,
};

pub const DEPENDENTS: u32 = 1;
pub const BINDINGS: u32 = 2;
pub const BINDINGS_TARGET: u32 = 4;

pub type ConverterBindContextHandler = fn(&CoreHandle, RuntimeDataContextHandle, CoreHandle);

/// Select the virtual operation, then release the occurrence before invoking
/// it. A scripted converter's hydration may resolve this very same owner.
pub fn bind_converter_context(
    owner: &CoreHandle,
    context: RuntimeDataContextHandle,
    data_bind: CoreHandle,
) {
    let handler = owner
        .with(|owner| {
            owner
                .as_data_converter_capability()
                .expect("a retained converter has its capability")
                .bind_context_handler()
        })
        .expect("the retained converter remains live");
    handler(owner, context, data_bind);
}

pub trait DataBindNode {
    fn clone_bind(&self) -> Option<CoreHandle>;
    fn set_target_converter(&mut self, converter: CoreHandle);
    fn copy_file_from(&mut self, source: CoreHandle);
    fn bind_from_context(&mut self, context: RuntimeDataContextHandle);
    fn unbind(&mut self);
    fn update(&mut self, force: bool);
}

pub trait ParentDataBind {
    fn target_origin(&self) -> bool;
    fn add_dirt(&mut self, dirt: u32, recurse: bool);
}

pub trait ConverterImporter {
    fn add_data_converter(&mut self, converter: CoreHandle);
    fn import_super(&mut self, converter: &mut DataConverter) -> StatusCode;
}

#[macro_export]
macro_rules! data_converter_capability_lifecycle {
    ($($base:ident).+) => {
        fn bind_context_handler(&self) -> $crate::mechanical_port::source::data_bind::converters::data_converter::ConverterBindContextHandler {
            |owner, context, data_bind| {
                owner.with_downcast_mut::<Self, _>(|owner| {
                    owner.$($base).+.bind_from_context(context, data_bind);
                }).expect("the retained converter keeps its concrete type");
            }
        }

        fn unbind(&mut self) {
            self.$($base).+.unbind();
        }

        fn update(&mut self) {
            self.$($base).+.update();
        }

        fn reset(&mut self) {
            self.$($base).+.reset();
        }

        fn advance(&mut self, elapsed: f32) -> bool {
            self.$($base).+.advance(elapsed)
        }
    };
}

#[macro_export]
macro_rules! impl_data_converter_capability_bidi {
    ($ty:ty, $($base:ident).+) => {
        impl $crate::mechanical_port::source::generated::core_registry::DataConverterCapability
            for $ty
        {
            fn convert(
                &mut self,
                input: &dyn $crate::mechanical_port::source::data_bind::data_values::data_value::DataValue,
                _data_bind: &$crate::mechanical_port::source::core::CoreHandle,
                output: &mut dyn FnMut(
                    &dyn $crate::mechanical_port::source::data_bind::data_values::data_value::DataValue,
                ),
            ) {
                output(Self::convert(self, input));
            }

            fn reverse_convert(
                &mut self,
                input: &dyn $crate::mechanical_port::source::data_bind::data_values::data_value::DataValue,
                _data_bind: &$crate::mechanical_port::source::core::CoreHandle,
                output: &mut dyn FnMut(
                    &dyn $crate::mechanical_port::source::data_bind::data_values::data_value::DataValue,
                ),
            ) {
                output(Self::reverse_convert(self, input));
            }

            fn output_type(
                &self,
            ) -> $crate::mechanical_port::source::data_bind::data_values::data_type::DataType {
                Self::output_type(self)
            }

            $crate::data_converter_capability_lifecycle!($($base).+);
        }
    };
}

#[macro_export]
macro_rules! impl_data_converter_capability_forward {
    ($ty:ty, $($base:ident).+) => {
        impl $crate::mechanical_port::source::generated::core_registry::DataConverterCapability
            for $ty
        {
            fn convert(
                &mut self,
                input: &dyn $crate::mechanical_port::source::data_bind::data_values::data_value::DataValue,
                _data_bind: &$crate::mechanical_port::source::core::CoreHandle,
                output: &mut dyn FnMut(
                    &dyn $crate::mechanical_port::source::data_bind::data_values::data_value::DataValue,
                ),
            ) {
                output(Self::convert(self, input));
            }

            fn reverse_convert(
                &mut self,
                input: &dyn $crate::mechanical_port::source::data_bind::data_values::data_value::DataValue,
                _data_bind: &$crate::mechanical_port::source::core::CoreHandle,
                output: &mut dyn FnMut(
                    &dyn $crate::mechanical_port::source::data_bind::data_values::data_value::DataValue,
                ),
            ) {
                output(input);
            }

            fn output_type(
                &self,
            ) -> $crate::mechanical_port::source::data_bind::data_values::data_type::DataType {
                Self::output_type(self)
            }

            $crate::data_converter_capability_lifecycle!($($base).+);
        }
    };
}

pub struct DataConverter {
    pub base: DataConverterBase,
    parent_data_bind: Option<CoreHandle>,
    data_binds: DataBindContainer,
}

impl std::ops::Deref for DataConverter {
    type Target = DataConverterBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for DataConverter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Default for DataConverter {
    fn default() -> Self {
        Self {
            base: DataConverterBase::default(),
            parent_data_bind: None,
            data_binds: DataBindContainer::default(),
        }
    }
}

impl DataConverter {
    fn handle(&self) -> Option<CoreHandle> {
        self.base.base.handle()
    }

    fn initialize_container_owner(&mut self) {
        if let Some(owner) = self.handle() {
            self.data_binds.set_owner(owner);
        }
    }

    pub fn convert<'a>(&mut self, value: &'a dyn DataValue) -> &'a dyn DataValue {
        value
    }

    pub fn reverse_convert<'a>(&mut self, value: &'a dyn DataValue) -> &'a dyn DataValue {
        value
    }

    pub fn output_type(&self) -> DataType {
        DataType::None
    }

    pub fn import(&mut self, importer: Option<&mut dyn ConverterImporter>) -> StatusCode {
        let Some(importer) = importer else {
            return StatusCode::MissingObject;
        };
        let Some(converter) = self.handle() else {
            return StatusCode::MissingObject;
        };
        self.data_binds.set_owner(converter.clone());
        importer.add_data_converter(converter);
        importer.import_super(self)
    }

    pub fn import_stack(
        &mut self,
        stack: &mut crate::mechanical_port::source::importers::import_stack::ImportStack,
    ) -> StatusCode {
        use crate::mechanical_port::source::{
            generated::backboard_base::BackboardBase,
            importers::backboard_importer::BackboardImporter,
        };
        let Some(importer) = stack.latest::<BackboardImporter>(BackboardBase::TYPE_KEY) else {
            return StatusCode::MissingObject;
        };
        let Some(owner) = self.handle() else {
            return StatusCode::MissingObject;
        };
        self.data_binds.set_owner(owner.clone());
        importer.add_data_converter(owner);
        self.base.base.import(stack)
    }

    pub fn bind_from_context(
        &mut self,
        data_context: RuntimeDataContextHandle,
        data_bind: CoreHandle,
    ) {
        self.parent_data_bind = Some(data_bind);
        self.data_binds.bind_data_binds_from_context(data_context);
    }

    pub fn unbind(&mut self) {
        self.data_binds.unbind_data_binds();
    }

    pub fn mark_converter_dirty(&mut self) {
        if let Some(parent) = self.parent_data_bind.as_ref() {
            parent.with_mut(|parent| {
                if let Some(parent) = parent.as_data_bind_mut() {
                    parent.add_dirt(
                        DEPENDENTS
                            | if parent.target_origin() {
                                BINDINGS_TARGET
                            } else {
                                BINDINGS
                            },
                        false,
                    );
                }
            });
        }
    }

    pub fn add_dirty_data_bind(&mut self, data_bind: CoreHandle) {
        self.mark_converter_dirty();
        self.data_binds.add_dirty_data_bind(data_bind);
    }

    pub fn add_data_bind(&mut self, data_bind: CoreHandle) {
        self.initialize_container_owner();
        self.data_binds.add_data_bind(data_bind);
    }

    pub fn update(&mut self) {
        self.data_binds.update_data_binds(false);
    }

    pub fn copy(&mut self, object: &Self) {
        self.initialize_container_owner();
        let target = self.handle();
        for source in object.data_binds.data_binds() {
            let Some(cloned) = source.clone_occurrence() else {
                continue;
            };
            let source_file = source
                .with(|source| source.as_data_bind().map(|bind| bind.file()))
                .flatten();
            cloned.with_mut(|bind| {
                if let Some(bind) = bind.as_data_bind_mut() {
                    bind.set_target(target.clone());
                    if let Some(file) = source_file {
                        bind.set_file(file);
                    }
                }
            });
            self.data_binds.add_data_bind(cloned);
        }
        self.base
            .copy(&object.base, &mut DataConverterCopyCallbacks);
    }

    pub fn advance(&mut self, _elapsed_time: f32) -> bool {
        false
    }

    pub fn reset(&mut self) {}

    pub fn data_binds(&self) -> Vec<CoreHandle> {
        self.data_binds.data_binds()
    }
}

impl DataConverterBaseCallbacks for DataConverter {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base.base.notify_property_changed(property_key);
    }
}

impl DataConverterCapability for DataConverter {
    fn convert(
        &mut self,
        input: &dyn DataValue,
        _data_bind: &CoreHandle,
        output: &mut dyn FnMut(&dyn DataValue),
    ) {
        output(input);
    }

    fn reverse_convert(
        &mut self,
        input: &dyn DataValue,
        _data_bind: &CoreHandle,
        output: &mut dyn FnMut(&dyn DataValue),
    ) {
        output(input);
    }

    fn output_type(&self) -> DataType {
        Self::output_type(self)
    }

    fn bind_context_handler(&self) -> ConverterBindContextHandler {
        |owner, context, data_bind| {
            owner
                .with_downcast_mut::<Self, _>(|owner| owner.bind_from_context(context, data_bind))
                .expect("the retained converter keeps its concrete type");
        }
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

struct DataConverterCopyCallbacks;

impl DataConverterBaseCallbacks for DataConverterCopyCallbacks {
    fn notify_property_changed(&mut self, _property_key: u16) {}
}
