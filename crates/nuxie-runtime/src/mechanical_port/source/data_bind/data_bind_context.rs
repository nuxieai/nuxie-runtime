use crate::mechanical_port::source::{
    assets::manifest_asset::ManifestAsset,
    core::CoreHandle,
    data_bind::data_context::RuntimeDataContextHandle,
    generated::data_bind::data_bind_context_base::{
        DataBindContextBase, DataBindContextBaseCallbacks,
    },
};

pub trait BoundSource {}
pub trait ContextConverter {
    fn bind_from_context(&mut self, context: RuntimeDataContextHandle, data_bind: CoreHandle);
}
pub const RECONCILE_DIRT: u32 = super::data_bind::BINDINGS | super::data_bind::BINDINGS_TARGET;
pub struct DataBindContext {
    pub base: DataBindContextBase,
}
impl Default for DataBindContext {
    fn default() -> Self {
        Self {
            base: DataBindContextBase::default(),
        }
    }
}
impl DataBindContext {
    pub(crate) fn clone_core(&self) -> Self {
        let mut cloned = Self::default();
        cloned.copy_source_path_ids(self);
        // Keep the source-path receiver in place while copying the scalar
        // generated base. Its callback-owned data belongs to this clone.
        let mut base = std::mem::take(&mut cloned.base.base.base);
        base.copy(&self.base.base.base, &mut cloned);
        cloned.base.base.base = base;
        cloned
    }

    pub fn bind_from_context_handle(
        owner: &CoreHandle,
        data_context: Option<RuntimeDataContextHandle>,
    ) {
        let Some(data_context) = data_context else {
            return;
        };
        let state = owner.with_downcast_mut::<Self, _>(|bind| {
            bind.resolve_path();
            (
                bind.base.base.is_name_based(),
                bind.base.base.file(),
                bind.source_path_ids().to_vec(),
                bind.base.base.source(),
            )
        });
        let Some((name_based, file, path, previous)) = state else {
            return;
        };
        let source = if name_based && file.with_file(|_| ()).is_some() {
            let resolver = file.with_file(|file| file.manifest()).flatten();
            if let Some(resolver) = resolver {
                resolver
                    .with_downcast::<ManifestAsset, _>(|resolver| {
                        data_context.with_context(|context| {
                            context.get_relative_view_model_property(&path, Some(resolver))
                        })
                    })
                    .flatten()
            } else {
                data_context
                    .with_context(|context| context.get_relative_view_model_property(&path, None))
            }
        } else {
            data_context.with_context(|context| context.get_view_model_property(&path))
        };
        if previous.is_none() || previous != source {
            if let Some(source) = source {
                owner.with_mut(|owner| {
                    let bind = owner.as_data_bind_mut().unwrap();
                    bind.clear_source();
                    bind.set_source(source);
                });
                super::data_bind::DataBind::bind_handle(owner);
            } else {
                super::data_bind::DataBind::unbind_handle(owner);
            }
        } else {
            owner.with_mut(|owner| {
                let bind = owner.as_data_bind_mut().unwrap();
                bind.add_dirt(bind.reconcile_dirt(), true);
            });
        }
        let converter = owner
            .with(|owner| owner.as_data_bind().unwrap().converter())
            .flatten();
        if let Some(converter) = converter {
            super::converters::data_converter::bind_converter_context(
                &converter,
                data_context,
                owner.clone(),
            );
        }
    }

    pub fn decode_source_path_ids(&mut self, bytes: &[u8]) {
        let mut index = 0;
        while index < bytes.len() {
            let mut value = 0u32;
            let mut shift = 0;
            loop {
                let byte = bytes[index];
                index += 1;
                value |= ((byte & 127) as u32) << shift;
                if byte & 128 == 0 {
                    break;
                }
                shift += 7;
            }
            self.base.source_path_ids_buffer.push(value);
        }
    }
    pub fn copy_source_path_ids(&mut self, other: &Self) {
        self.base
            .source_path_ids_buffer
            .clone_from(&other.base.source_path_ids_buffer);
        self.base.is_path_resolved = other.base.is_path_resolved
    }
    fn resolve_path(&mut self) {
        if !self.base.base.is_name_based() || self.base.is_path_resolved {
            return;
        }
        self.base.is_path_resolved = true;
        if let Some(id) = self.base.source_path_ids_buffer.first().copied() {
            let resolved = self
                .base
                .base
                .file()
                .with_file(|file| {
                    file.manifest()?
                        .with_downcast::<ManifestAsset, _>(|resolver| {
                            resolver.resolve_path(id as i32).to_vec()
                        })
                })
                .flatten();
            if let Some(path) = resolved.filter(|path| !path.is_empty()) {
                self.base.source_path_ids_buffer = path;
            }
        }
    }
    pub fn source_path_ids(&self) -> &[u32] {
        &self.base.source_path_ids_buffer
    }
}

impl DataBindContextBaseCallbacks for DataBindContext {
    fn decode_source_path_ids(&mut self, value: &[u8]) {
        Self::decode_source_path_ids(self, value);
    }

    fn copy_source_path_ids(&mut self, object: &DataBindContextBase) {
        self.base
            .source_path_ids_buffer
            .clone_from(&object.source_path_ids_buffer);
        self.base.is_path_resolved = object.is_path_resolved;
    }
}
