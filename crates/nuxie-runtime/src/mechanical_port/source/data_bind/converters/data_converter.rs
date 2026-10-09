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

pub use crate::mechanical_port::source::data_bind::data_bind::{
    BINDINGS, BINDINGS_TARGET, DEPENDENTS,
};

pub type ConverterUnbindHandler = fn(&CoreHandle);

pub type ConverterBindContextHandler =
    fn(&CoreHandle, RuntimeDataContextHandle, Option<CoreHandle>);

/// Select the virtual operation, then release the occurrence before invoking
/// it. A scripted converter's hydration may resolve this very same owner.
pub fn bind_converter_context(
    owner: &CoreHandle,
    context: RuntimeDataContextHandle,
    data_bind: Option<CoreHandle>,
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
            $crate::mechanical_port::source::data_bind::converters::data_converter::DataConverter::bind_from_context_handle
        }

        fn unbind_handler(&self) -> $crate::mechanical_port::source::data_bind::converters::data_converter::ConverterUnbindHandler {
            $crate::mechanical_port::source::data_bind::converters::data_converter::DataConverter::unbind_base_handle
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
    pub(crate) data_binds: DataBindContainer,
}

impl Drop for DataConverter {
    fn drop(&mut self) {
        self.data_binds.delete_data_binds();
    }
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
            data_binds: DataBindContainer::default(),
        }
    }
}

impl DataConverter {
    /// Native definitions contain only unbound scalar construction. Restore the
    /// inherited constructor default before the clone gets an arena identity;
    /// the qualified base completion reads Name after owned binding callbacks.
    pub(crate) fn clone_occurrence_parts(
        source: &dyn crate::mechanical_port::source::core::CoreObject,
    ) -> crate::mechanical_port::source::core::CoreCloneParts {
        let mut cloned = source.clone_boxed();
        if let Some(cloned) = &mut cloned {
            cloned
                .as_data_converter_mut()
                .expect("a native converter definition retains its base")
                .base
                .set_name_value(String::new());
        }
        (cloned, source.clone_completion_handler())
    }

    /// Complete the inherited owned bindings only after the clone has an
    /// occurrence identity, so every cloned binding targets the actual clone.
    pub fn complete_clone(source: &CoreHandle, cloned: &CoreHandle) -> bool {
        let (Some(source_container), Some(cloned_container)) =
            (source.data_bind_container(), cloned.data_bind_container())
        else {
            return false;
        };
        let source_binds = source_container.data_binds().to_vec();
        for source_bind in source_binds {
            let Some(cloned_bind) = source_bind.clone_occurrence() else {
                return false;
            };
            cloned_bind.with_mut(|bind| {
                bind.as_data_bind_mut()
                    .expect("a binding clone remains a binding")
                    .set_target(Some(cloned.clone()));
            });
            let Some(file) = source_bind
                .with(|bind| bind.as_data_bind().map(|bind| bind.file()))
                .flatten()
            else {
                return false;
            };
            cloned_bind.with_mut(|bind| {
                bind.as_data_bind_mut()
                    .expect("a binding clone remains a binding")
                    .set_file(file);
            });
            cloned_container.add_data_bind(cloned_bind);
        }
        let Some(name) = source
            .with(|source| {
                source
                    .as_data_converter()
                    .map(|source| source.base.name().to_owned())
            })
            .flatten()
        else {
            return false;
        };
        cloned
            .with_mut(|cloned| {
                let Some(cloned) = cloned.as_data_converter_mut() else {
                    return false;
                };
                cloned.base.set_name_value(name);
                true
            })
            .unwrap_or(false)
    }

    pub fn convert_handle(
        owner: &CoreHandle,
        input: &dyn DataValue,
        bind: &CoreHandle,
        reverse: bool,
    ) -> Box<dyn DataValue> {
        use super::data_converter_group::DataConverterGroup;
        use crate::mechanical_port::source::{
            data_bind::data_values::data_value::clone_data_value,
            scripted::scripted_data_converter::ScriptedDataConverter,
        };
        if owner
            .with_downcast::<ScriptedDataConverter, _>(|_| ())
            .is_some()
        {
            return ScriptedDataConverter::convert_handle(owner, input, reverse);
        }
        if let Some(mut items) =
            owner.with_downcast::<DataConverterGroup, _>(|group| group.items().to_vec())
        {
            if reverse {
                items.reverse();
            }
            let mut value = clone_data_value(input);
            for item in items {
                if let Some(converter) = item
                    .with(|item| item.as_data_converter_group_item().unwrap().converter())
                    .flatten()
                {
                    value = Self::convert_handle(&converter, value.as_ref(), bind, reverse);
                }
            }
            return value;
        }
        owner
            .with_mut(|owner| {
                let converter = owner
                    .as_data_converter_capability_mut()
                    .expect("retained converter capability");
                let mut result = None;
                let mut output = |value: &dyn DataValue| {
                    result = Some(clone_data_value(value));
                };
                if reverse {
                    converter.reverse_convert(input, bind, &mut output);
                } else {
                    converter.convert(input, bind, &mut output);
                }
                result.expect("a concrete converter returns one value")
            })
            .expect("live converter")
    }

    pub fn bind_from_context_handle(
        owner: &CoreHandle,
        context: RuntimeDataContextHandle,
        data_bind: Option<CoreHandle>,
    ) {
        owner
            .data_bind_container()
            .expect("registered converter container")
            .set_parent_data_bind(data_bind);
        crate::mechanical_port::source::data_bind::data_bind_container::DataBindContainerOwner::Authored(owner.clone()).bind_data_binds_from_context(context);
    }
    pub(crate) fn parent_data_bind(&self) -> Option<CoreHandle> {
        self.data_binds.parent_data_bind()
    }
    pub fn update_handle(owner: &CoreHandle) {
        use super::data_converter_group::DataConverterGroup;
        if let Some(items) =
            owner.with_downcast::<DataConverterGroup, _>(|group| group.items().to_vec())
        {
            for item in items {
                if let Some(converter) = item
                    .with(|item| item.as_data_converter_group_item().unwrap().converter())
                    .flatten()
                {
                    Self::update_handle(&converter);
                }
            }
        } else {
            crate::mechanical_port::source::data_bind::data_bind_container::DataBindContainerOwner::Authored(owner.clone()).update_data_binds(false);
        }
    }

    /// Select this converter's virtual operation before releasing its receiver.
    /// Builtin overrides supply their own released occurrence implementation;
    /// a custom capability defaults to its ordinary virtual unbind method.
    pub fn unbind_handle(owner: &CoreHandle) {
        let handler = owner
            .with(|owner| {
                owner
                    .as_data_converter_capability()
                    .map(|converter| converter.unbind_handler())
            })
            .flatten();
        if let Some(handler) = handler {
            handler(owner);
        }
    }

    /// Qualified DataConverter::unbind for owners that inherit the base method.
    /// Retain the container independently while its children detach observers.
    pub fn unbind_base_handle(owner: &CoreHandle) {
        // Teardown may already have retired the occurrence. This also protects
        // a selected handler whose custom selector releases the last owner.
        if !owner.is_alive() {
            return;
        }
        crate::mechanical_port::source::data_bind::data_bind_container::DataBindContainerOwner::Authored(owner.clone()).unbind_data_binds();
    }

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
        data_bind: Option<CoreHandle>,
    ) {
        self.data_binds.set_parent_data_bind(data_bind);
        self.data_binds.bind_data_binds_from_context(data_context);
    }

    pub fn unbind(&mut self) {
        self.data_binds.unbind_data_binds();
    }

    pub fn mark_converter_dirty(&mut self) {
        if let Some(parent) = self.data_binds.parent_data_bind() {
            let dirt = parent
                .with(|owner| {
                    owner.as_data_bind().map(|parent| {
                        DEPENDENTS
                            | if parent.target_origin() {
                                BINDINGS_TARGET
                            } else {
                                BINDINGS
                            }
                    })
                })
                .flatten();
            if let Some(dirt) = dirt {
                crate::source::data_bind::data_bind::DataBind::add_dirt_handle(
                    &parent, dirt, false,
                );
            }
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
        // Clone construction has not acquired its arena identity yet. The
        // inherited complete_clone hook installs these same owned bindings
        // against the real clone after insertion, never against a null target.
        for source in target
            .iter()
            .flat_map(|_| object.data_binds.data_binds().to_vec())
        {
            let Some(cloned) = source.clone_occurrence() else {
                continue;
            };
            cloned.with_mut(|bind| {
                if let Some(bind) = bind.as_data_bind_mut() {
                    bind.set_target(target.clone());
                }
            });
            let source_file = source
                .with(|source| source.as_data_bind().map(|bind| bind.file()))
                .flatten();
            cloned.with_mut(|bind| {
                if let Some(bind) = bind.as_data_bind_mut()
                    && let Some(file) = source_file
                {
                    bind.set_file(file);
                }
            });
            self.data_binds.add_data_bind(cloned);
        }
        self.base
            .copy(&object.base, &mut DataConverterCopyCallbacks);
    }

    pub fn may_advance(&self) -> bool {
        false
    }

    pub fn advance(&mut self, _elapsed_time: f32) -> bool {
        false
    }

    pub fn reset(&mut self) {}

    pub fn data_binds(&self) -> Vec<CoreHandle> {
        self.data_binds.data_binds().to_vec()
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
        Self::bind_from_context_handle
    }

    fn unbind_handler(&self) -> ConverterUnbindHandler {
        Self::unbind_base_handle
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
