pub trait ContextDataResolver {
    fn resolve_path(&self, id: u32) -> Vec<u32>;
}
pub trait ContextFile {
    fn data_resolver(&self) -> Option<&dyn ContextDataResolver>;
}
pub trait BoundSource {}
pub trait ContextData {
    fn view_model_property(&self, path: &[u32]) -> Option<*mut dyn BoundSource>;
    fn relative_view_model_property(
        &self,
        path: &[u32],
        resolver: &dyn ContextDataResolver,
    ) -> Option<*mut dyn BoundSource>;
}
pub trait ContextConverter {
    fn bind_from_context(&mut self, context: &dyn ContextData, data_bind: *mut DataBindContext);
}
pub const RECONCILE_DIRT: u32 = 3;
pub struct DataBindContext {
    pub base: DataBindContextBase,
    file: Option<*mut dyn ContextFile>,
    source: Option<*mut dyn BoundSource>,
    converter: Option<*mut dyn ContextConverter>,
    dirt: u32,
    bound: bool,
}
impl Default for DataBindContext {
    fn default() -> Self {
        Self {
            base: DataBindContextBase::default(),
            file: None,
            source: None,
            converter: None,
            dirt: 0,
            bound: false,
        }
    }
}
impl DataBindContext {
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
        if let (Some(id), Some(file)) =
            (self.base.source_path_ids_buffer.first().copied(), self.file)
        {
            if let Some(resolver) = unsafe { (&*file).data_resolver() } {
                let path = resolver.resolve_path(id);
                if !path.is_empty() {
                    self.base.source_path_ids_buffer = path;
                }
            }
        }
    }
    pub fn bind_from_context(&mut self, data_context: Option<&dyn ContextData>) {
        let Some(data_context) = data_context else {
            return;
        };
        self.resolve_path();
        let source = if self.base.base.is_name_based() {
            self.file
                .and_then(|file| unsafe { (&*file).data_resolver() })
                .and_then(|resolver| {
                    data_context
                        .relative_view_model_property(&self.base.source_path_ids_buffer, resolver)
                })
        } else {
            data_context.view_model_property(&self.base.source_path_ids_buffer)
        };
        if !same_ptr(self.source, source) {
            if let Some(source) = source {
                self.source = None;
                self.source = Some(source);
                self.bound = true;
            } else {
                self.bound = false;
            }
        } else {
            self.dirt |= RECONCILE_DIRT;
        }
        if let Some(converter) = self.converter {
            unsafe {
                (&mut *converter).bind_from_context(data_context, self as *mut Self);
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
fn same_ptr<T: ?Sized>(a: Option<*mut T>, b: Option<*mut T>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => core::ptr::addr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}
use crate::mechanical_port::source::generated::data_bind::data_bind_context_base::{
    DataBindContextBase, DataBindContextBaseCallbacks,
};
