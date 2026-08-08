//! C ownership and mutation adapter for runtime data binding.
//!
//! The C interface is deliberately flat: catalogs and value snapshots own all
//! of their bytes, while mutation batches are fixed-stride borrowed input.
//! Graph traversal, alias preservation, and whole-batch validation remain
//! behind this module's interface.

use super::*;
use nuxie::host_interfaces::{
    RuntimeOwnedViewModelHandle, RuntimeOwnedViewModelUndoLog, RuntimeViewModelLinkError,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, Ordering};

const MAX_CATALOG_ITEMS: usize = 4_096;
const MAX_SNAPSHOT_INSTANCES: usize = 1_024;
const MAX_SNAPSHOT_VALUES: usize = 16_384;
const MAX_LIST_ITEMS: usize = 16_384;
const MAX_MUTATIONS: usize = 1_024;
const MAX_PROPERTY_PATH_BYTES: usize = 4_096;
const MAX_VALUE_BYTES: usize = 1024 * 1024;
const MAX_TOTAL_BYTES: usize = 4 * 1024 * 1024;

static NEXT_VIEW_MODEL_IDENTITY: AtomicU64 = AtomicU64::new(1);

#[cfg(test)]
thread_local! {
    static TEST_VM_COMMIT_PANIC_AFTER: Cell<Option<usize>> = const { Cell::new(None) };
    static TEST_TEXT_COMMIT_PANIC_AFTER: Cell<Option<usize>> = const { Cell::new(None) };
}

#[cfg(test)]
fn inject_vm_commit_panic_after(applied_count: Option<usize>) {
    TEST_VM_COMMIT_PANIC_AFTER.set(applied_count);
}

#[cfg(test)]
fn inject_text_commit_panic_after(applied_count: Option<usize>) {
    TEST_TEXT_COMMIT_PANIC_AFTER.set(applied_count);
}

#[cfg(test)]
fn maybe_panic_during_vm_commit(applied_count: usize) {
    if TEST_VM_COMMIT_PANIC_AFTER.get() == Some(applied_count) {
        panic!("injected view-model commit failure");
    }
}

#[cfg(not(test))]
fn maybe_panic_during_vm_commit(_applied_count: usize) {}

#[cfg(test)]
fn maybe_panic_during_text_commit(applied_count: usize) {
    if TEST_TEXT_COMMIT_PANIC_AFTER.get() == Some(applied_count) {
        panic!("injected text-run commit failure");
    }
}

#[cfg(not(test))]
fn maybe_panic_during_text_commit(_applied_count: usize) {}

pub(super) fn next_view_model_identity() -> Option<u64> {
    NEXT_VIEW_MODEL_IDENTITY
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            (current != u64::MAX).then_some(current + 1)
        })
        .ok()
}

fn owned_string_view(value: &[u8]) -> NuxStringView {
    NuxStringView {
        data: value.as_ptr().cast(),
        len: value.len(),
    }
}

fn owned_byte_view(value: &[u8]) -> NuxByteView {
    NuxByteView {
        data: value.as_ptr(),
        len: value.len(),
    }
}

fn classify_property(type_name: &str) -> u32 {
    match type_name {
        "ViewModelPropertyString" => NUX_VIEW_MODEL_VALUE_KIND_STRING,
        "ViewModelPropertyNumber" | "ViewModelPropertyInteger" => NUX_VIEW_MODEL_VALUE_KIND_NUMBER,
        "ViewModelPropertyBoolean" => NUX_VIEW_MODEL_VALUE_KIND_BOOL,
        "ViewModelPropertyColor" => NUX_VIEW_MODEL_VALUE_KIND_COLOR,
        "ViewModelPropertyEnum" | "ViewModelPropertyEnumCustom" | "ViewModelPropertyEnumSystem" => {
            NUX_VIEW_MODEL_VALUE_KIND_ENUM
        }
        "ViewModelPropertyTrigger" => NUX_VIEW_MODEL_VALUE_KIND_TRIGGER,
        "ViewModelPropertySymbolListIndex" => NUX_VIEW_MODEL_VALUE_KIND_LIST_INDEX,
        "ViewModelPropertyList" => NUX_VIEW_MODEL_VALUE_KIND_LIST,
        "ViewModelPropertyViewModel" => NUX_VIEW_MODEL_VALUE_KIND_VIEW_MODEL,
        "ViewModelPropertyAsset" | "ViewModelPropertyAssetImage" => NUX_VIEW_MODEL_VALUE_KIND_IMAGE,
        "ViewModelPropertyAssetFont" => NUX_VIEW_MODEL_VALUE_KIND_FONT,
        "ViewModelPropertyAssetBlob" => NUX_VIEW_MODEL_VALUE_KIND_BLOB,
        "ViewModelPropertyArtboard" => NUX_VIEW_MODEL_VALUE_KIND_ARTBOARD,
        _ => NUX_VIEW_MODEL_VALUE_KIND_UNSUPPORTED,
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NuxViewModelValueKind {
    Unsupported = 0,
    String = 1,
    Number = 2,
    Bool = 3,
    Color = 4,
    Enum = 5,
    Trigger = 6,
    ListIndex = 7,
    List = 8,
    ViewModel = 9,
    Image = 10,
    Font = 11,
    Blob = 12,
    Artboard = 13,
}

pub const NUX_VIEW_MODEL_VALUE_KIND_UNSUPPORTED: u32 = NuxViewModelValueKind::Unsupported as u32;
pub const NUX_VIEW_MODEL_VALUE_KIND_STRING: u32 = NuxViewModelValueKind::String as u32;
pub const NUX_VIEW_MODEL_VALUE_KIND_NUMBER: u32 = NuxViewModelValueKind::Number as u32;
pub const NUX_VIEW_MODEL_VALUE_KIND_BOOL: u32 = NuxViewModelValueKind::Bool as u32;
pub const NUX_VIEW_MODEL_VALUE_KIND_COLOR: u32 = NuxViewModelValueKind::Color as u32;
pub const NUX_VIEW_MODEL_VALUE_KIND_ENUM: u32 = NuxViewModelValueKind::Enum as u32;
pub const NUX_VIEW_MODEL_VALUE_KIND_TRIGGER: u32 = NuxViewModelValueKind::Trigger as u32;
pub const NUX_VIEW_MODEL_VALUE_KIND_LIST_INDEX: u32 = NuxViewModelValueKind::ListIndex as u32;
pub const NUX_VIEW_MODEL_VALUE_KIND_LIST: u32 = NuxViewModelValueKind::List as u32;
pub const NUX_VIEW_MODEL_VALUE_KIND_VIEW_MODEL: u32 = NuxViewModelValueKind::ViewModel as u32;
pub const NUX_VIEW_MODEL_VALUE_KIND_IMAGE: u32 = NuxViewModelValueKind::Image as u32;
pub const NUX_VIEW_MODEL_VALUE_KIND_FONT: u32 = NuxViewModelValueKind::Font as u32;
pub const NUX_VIEW_MODEL_VALUE_KIND_BLOB: u32 = NuxViewModelValueKind::Blob as u32;
pub const NUX_VIEW_MODEL_VALUE_KIND_ARTBOARD: u32 = NuxViewModelValueKind::Artboard as u32;

#[derive(Debug)]
struct OwnedCatalogSchema {
    name: Box<[u8]>,
    first_property: usize,
    property_count: usize,
    first_instance: usize,
    instance_count: usize,
    default_authored_instance: usize,
    is_global: bool,
}

#[derive(Debug)]
struct OwnedCatalogProperty {
    schema_index: usize,
    property_index: usize,
    name: Box<[u8]>,
    kind: u32,
    referenced_schema_index: usize,
    first_enum_label: usize,
    enum_label_count: usize,
}

#[derive(Debug)]
struct OwnedCatalogInstance {
    schema_index: usize,
    instance_index: usize,
    name: Option<Box<[u8]>>,
}

/// Immutable owned projection of every data-binding schema in one file.
pub struct NuxViewModelCatalog {
    schemas: Vec<OwnedCatalogSchema>,
    properties: Vec<OwnedCatalogProperty>,
    instances: Vec<OwnedCatalogInstance>,
    enum_labels: Vec<Box<[u8]>>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxViewModelCatalogInfo {
    pub struct_size: u32,
    pub schema_count: usize,
    pub property_count: usize,
    pub authored_instance_count: usize,
    pub enum_label_count: usize,
}

impl Default for NuxViewModelCatalogInfo {
    fn default() -> Self {
        Self {
            struct_size: std::mem::size_of::<Self>() as u32,
            schema_count: 0,
            property_count: 0,
            authored_instance_count: 0,
            enum_label_count: 0,
        }
    }
}

pub const NUX_VIEW_MODEL_CATALOG_INFO_V3_MIN_SIZE: usize =
    std::mem::offset_of!(NuxViewModelCatalogInfo, enum_label_count) + std::mem::size_of::<usize>();

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxViewModelSchemaView {
    pub struct_size: u32,
    pub schema_index: usize,
    pub name: NuxStringView,
    pub first_property: usize,
    pub property_count: usize,
    pub first_authored_instance: usize,
    pub authored_instance_count: usize,
    /// Catalog-authored-instance index, or `SIZE_MAX` when generated defaults apply.
    pub default_authored_instance: usize,
    pub is_global: u32,
}

impl Default for NuxViewModelSchemaView {
    fn default() -> Self {
        Self {
            struct_size: std::mem::size_of::<Self>() as u32,
            schema_index: 0,
            name: NuxStringView::default(),
            first_property: 0,
            property_count: 0,
            first_authored_instance: 0,
            authored_instance_count: 0,
            default_authored_instance: usize::MAX,
            is_global: 0,
        }
    }
}

pub const NUX_VIEW_MODEL_SCHEMA_VIEW_V3_MIN_SIZE: usize =
    std::mem::offset_of!(NuxViewModelSchemaView, is_global) + std::mem::size_of::<u32>();

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxViewModelPropertyView {
    pub struct_size: u32,
    pub schema_index: usize,
    pub property_index: usize,
    pub name: NuxStringView,
    pub kind: u32,
    /// `SIZE_MAX` when this is not a nested view-model property.
    pub referenced_schema_index: usize,
    pub first_enum_label: usize,
    pub enum_label_count: usize,
}

impl Default for NuxViewModelPropertyView {
    fn default() -> Self {
        Self {
            struct_size: std::mem::size_of::<Self>() as u32,
            schema_index: 0,
            property_index: 0,
            name: NuxStringView::default(),
            kind: NUX_VIEW_MODEL_VALUE_KIND_UNSUPPORTED,
            referenced_schema_index: usize::MAX,
            first_enum_label: 0,
            enum_label_count: 0,
        }
    }
}

pub const NUX_VIEW_MODEL_PROPERTY_VIEW_V3_MIN_SIZE: usize =
    std::mem::offset_of!(NuxViewModelPropertyView, enum_label_count) + std::mem::size_of::<usize>();

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxViewModelAuthoredInstanceView {
    pub struct_size: u32,
    pub schema_index: usize,
    pub instance_index: usize,
    /// NULL+0 means the authored instance has no name.
    pub name: NuxStringView,
}

impl Default for NuxViewModelAuthoredInstanceView {
    fn default() -> Self {
        Self {
            struct_size: std::mem::size_of::<Self>() as u32,
            schema_index: 0,
            instance_index: 0,
            name: NuxStringView::default(),
        }
    }
}

pub const NUX_VIEW_MODEL_AUTHORED_INSTANCE_VIEW_V3_MIN_SIZE: usize =
    std::mem::offset_of!(NuxViewModelAuthoredInstanceView, name)
        + std::mem::size_of::<NuxStringView>();

fn build_catalog(file: &File) -> Result<NuxViewModelCatalog, NuxStatus> {
    if file.view_model_count() > MAX_CATALOG_ITEMS {
        return Err(NuxStatus::LimitExceeded);
    }
    let mut schemas = Vec::new();
    let mut properties = Vec::new();
    let mut instances = Vec::new();
    let mut enum_labels = Vec::new();
    let mut content_bytes = 0usize;
    for schema in file.view_models() {
        let schema_index = schema.index();
        let name = schema
            .name()
            .unwrap_or("")
            .as_bytes()
            .to_vec()
            .into_boxed_slice();
        content_bytes = content_bytes
            .checked_add(name.len())
            .ok_or(NuxStatus::LimitExceeded)?;
        let first_property = properties.len();
        for property in schema.properties() {
            if properties.len() >= MAX_CATALOG_ITEMS {
                return Err(NuxStatus::LimitExceeded);
            }
            let property_name = property
                .name()
                .unwrap_or("")
                .as_bytes()
                .to_vec()
                .into_boxed_slice();
            content_bytes = content_bytes
                .checked_add(property_name.len())
                .ok_or(NuxStatus::LimitExceeded)?;
            let descriptor = property.descriptor();
            let kind = classify_property(property.type_name());
            let referenced_schema_index = if kind == NUX_VIEW_MODEL_VALUE_KIND_VIEW_MODEL {
                descriptor
                    .uint_property("viewModelReferenceId")
                    .and_then(|value| usize::try_from(value).ok())
                    .ok_or(NuxStatus::RuntimeError)?
            } else {
                usize::MAX
            };
            let first_enum_label = enum_labels.len();
            if kind == NUX_VIEW_MODEL_VALUE_KIND_ENUM {
                while let Some(label) = file
                    .runtime()
                    .view_model_property_enum_value_for_index_object(
                        descriptor,
                        enum_labels.len() - first_enum_label,
                    )
                {
                    if enum_labels.len() >= MAX_CATALOG_ITEMS {
                        return Err(NuxStatus::LimitExceeded);
                    }
                    content_bytes = content_bytes
                        .checked_add(label.len())
                        .ok_or(NuxStatus::LimitExceeded)?;
                    enum_labels.push(label.to_vec().into_boxed_slice());
                }
            }
            properties.push(OwnedCatalogProperty {
                schema_index,
                property_index: property.index(),
                name: property_name,
                kind,
                referenced_schema_index,
                first_enum_label,
                enum_label_count: enum_labels.len() - first_enum_label,
            });
        }
        let first_instance = instances.len();
        for instance_index in 0..schema.instance_count() {
            if instances.len() >= MAX_CATALOG_ITEMS {
                return Err(NuxStatus::LimitExceeded);
            }
            let authored_name = schema
                .instance_name(instance_index)
                .map(|name| name.as_bytes().to_vec().into_boxed_slice());
            content_bytes = content_bytes
                .checked_add(authored_name.as_ref().map_or(0, |name| name.len()))
                .ok_or(NuxStatus::LimitExceeded)?;
            instances.push(OwnedCatalogInstance {
                schema_index,
                instance_index,
                name: authored_name,
            });
        }
        schemas.push(OwnedCatalogSchema {
            name,
            first_property,
            property_count: properties.len() - first_property,
            first_instance,
            instance_count: instances.len() - first_instance,
            default_authored_instance: if instances.len() == first_instance {
                usize::MAX
            } else {
                first_instance
            },
            is_global: schema.is_global(),
        });
    }
    if content_bytes > MAX_TOTAL_BYTES {
        return Err(NuxStatus::LimitExceeded);
    }
    Ok(NuxViewModelCatalog {
        schemas,
        properties,
        instances,
        enum_labels,
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_file_view_model_catalog(
    file: *const NuxFile,
    out_catalog: *mut *mut NuxViewModelCatalog,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_catalog.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe { *out_catalog = ptr::null_mut() };
        let _file_call = enter_status_handle!(file, HandleKind::File);
        let Some(file) = (unsafe { file.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let catalog = match build_catalog(&file.file) {
            Ok(catalog) => catalog,
            Err(status) => return status,
        };
        let handle = Box::into_raw(Box::new(catalog));
        register_handle(handle, HandleKind::ViewModelCatalog, file.owner_thread);
        unsafe { *out_catalog = handle };
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_view_model_catalog_free(
    catalog: *mut NuxViewModelCatalog,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if catalog.is_null() {
            return NuxStatus::Ok;
        }
        if let Err(status) = remove_handle(catalog, HandleKind::ViewModelCatalog) {
            return status;
        }
        unsafe { drop(Box::from_raw(catalog)) };
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_view_model_catalog_info(
    catalog: *const NuxViewModelCatalog,
    out_info: *mut NuxViewModelCatalogInfo,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let _call = enter_status_handle!(catalog, HandleKind::ViewModelCatalog);
        let Some(catalog) = (unsafe { catalog.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let value = NuxViewModelCatalogInfo {
            schema_count: catalog.schemas.len(),
            property_count: catalog.properties.len(),
            authored_instance_count: catalog.instances.len(),
            enum_label_count: catalog.enum_labels.len(),
            ..NuxViewModelCatalogInfo::default()
        };
        unsafe { write_caller_struct(out_info, &value, NUX_VIEW_MODEL_CATALOG_INFO_V3_MIN_SIZE) }
            .map_or_else(|status| status, |()| NuxStatus::Ok)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_view_model_catalog_schema(
    catalog: *const NuxViewModelCatalog,
    index: usize,
    out_schema: *mut NuxViewModelSchemaView,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let _call = enter_status_handle!(catalog, HandleKind::ViewModelCatalog);
        let Some(catalog) = (unsafe { catalog.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let Some(schema) = catalog.schemas.get(index) else {
            return NuxStatus::NotFound;
        };
        let value = NuxViewModelSchemaView {
            schema_index: index,
            name: owned_string_view(&schema.name),
            first_property: schema.first_property,
            property_count: schema.property_count,
            first_authored_instance: schema.first_instance,
            authored_instance_count: schema.instance_count,
            default_authored_instance: schema.default_authored_instance,
            is_global: u32::from(schema.is_global),
            ..NuxViewModelSchemaView::default()
        };
        unsafe { write_caller_struct(out_schema, &value, NUX_VIEW_MODEL_SCHEMA_VIEW_V3_MIN_SIZE) }
            .map_or_else(|status| status, |()| NuxStatus::Ok)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_view_model_catalog_property(
    catalog: *const NuxViewModelCatalog,
    index: usize,
    out_property: *mut NuxViewModelPropertyView,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let _call = enter_status_handle!(catalog, HandleKind::ViewModelCatalog);
        let Some(catalog) = (unsafe { catalog.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let Some(property) = catalog.properties.get(index) else {
            return NuxStatus::NotFound;
        };
        let value = NuxViewModelPropertyView {
            schema_index: property.schema_index,
            property_index: property.property_index,
            name: owned_string_view(&property.name),
            kind: property.kind,
            referenced_schema_index: property.referenced_schema_index,
            first_enum_label: property.first_enum_label,
            enum_label_count: property.enum_label_count,
            ..NuxViewModelPropertyView::default()
        };
        unsafe {
            write_caller_struct(
                out_property,
                &value,
                NUX_VIEW_MODEL_PROPERTY_VIEW_V3_MIN_SIZE,
            )
        }
        .map_or_else(|status| status, |()| NuxStatus::Ok)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_view_model_catalog_authored_instance(
    catalog: *const NuxViewModelCatalog,
    index: usize,
    out_instance: *mut NuxViewModelAuthoredInstanceView,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let _call = enter_status_handle!(catalog, HandleKind::ViewModelCatalog);
        let Some(catalog) = (unsafe { catalog.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let Some(instance) = catalog.instances.get(index) else {
            return NuxStatus::NotFound;
        };
        let value = NuxViewModelAuthoredInstanceView {
            schema_index: instance.schema_index,
            instance_index: instance.instance_index,
            name: instance
                .name
                .as_deref()
                .map(owned_string_view)
                .unwrap_or_default(),
            ..NuxViewModelAuthoredInstanceView::default()
        };
        unsafe {
            write_caller_struct(
                out_instance,
                &value,
                NUX_VIEW_MODEL_AUTHORED_INSTANCE_VIEW_V3_MIN_SIZE,
            )
        }
        .map_or_else(|status| status, |()| NuxStatus::Ok)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_view_model_catalog_enum_label(
    catalog: *const NuxViewModelCatalog,
    index: usize,
    out_label: *mut NuxStringView,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_label.is_null() {
            return NuxStatus::NullArgument;
        }
        let _call = enter_status_handle!(catalog, HandleKind::ViewModelCatalog);
        let Some(catalog) = (unsafe { catalog.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let Some(label) = catalog.enum_labels.get(index) else {
            return NuxStatus::NotFound;
        };
        unsafe { *out_label = owned_string_view(label) };
        NuxStatus::Ok
    })
}

fn publish_view_model(
    file: &NuxFile,
    schema_index: usize,
    instance: ViewModelInstance,
    out_instance: *mut *mut NuxViewModelInstance,
) -> NuxStatus {
    let Some(identity) = next_view_model_identity() else {
        return NuxStatus::LimitExceeded;
    };
    let handle = Box::into_raw(Box::new(NuxViewModelInstance {
        instance: RefCell::new(instance),
        file: Arc::clone(&file.file),
        schema_index,
        identity,
        owner_thread: file.owner_thread,
        file_provenance: Arc::clone(&file.data_binding_provenance),
        binding_provenance: None,
        provenance: Arc::clone(&file.data_binding_provenance),
    }));
    register_handle(handle, HandleKind::ViewModel, file.owner_thread);
    unsafe { *out_instance = handle };
    NuxStatus::Ok
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_view_model_instance_new(
    file: *const NuxFile,
    schema_index: usize,
    out_instance: *mut *mut NuxViewModelInstance,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_instance.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe { *out_instance = ptr::null_mut() };
        let _file_call = enter_status_handle!(file, HandleKind::File);
        let Some(file) = (unsafe { file.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let Some(instance) = file
            .file
            .view_model(schema_index)
            .and_then(|schema| schema.instantiate())
        else {
            return NuxStatus::NotFound;
        };
        publish_view_model(file, schema_index, instance, out_instance)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_view_model_instance_new_authored(
    file: *const NuxFile,
    schema_index: usize,
    authored_instance_index: usize,
    out_instance: *mut *mut NuxViewModelInstance,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_instance.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe { *out_instance = ptr::null_mut() };
        let _file_call = enter_status_handle!(file, HandleKind::File);
        let Some(file) = (unsafe { file.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let Some(instance) = file
            .file
            .view_model(schema_index)
            .and_then(|schema| schema.instantiate_instance(authored_instance_index))
        else {
            return NuxStatus::NotFound;
        };
        publish_view_model(file, schema_index, instance, out_instance)
    })
}

/// Instantiate the schema's first authored instance, falling back to generated
/// property defaults when the schema has no authored instances.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_view_model_instance_new_schema_default(
    file: *const NuxFile,
    schema_index: usize,
    out_instance: *mut *mut NuxViewModelInstance,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_instance.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe { *out_instance = ptr::null_mut() };
        let _file_call = enter_status_handle!(file, HandleKind::File);
        let Some(file) = (unsafe { file.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let Some(instance) = file
            .file
            .view_model(schema_index)
            .and_then(|schema| schema.instantiate_default())
        else {
            return NuxStatus::NotFound;
        };
        publish_view_model(file, schema_index, instance, out_instance)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_view_model_instance_share(
    instance: *const NuxViewModelInstance,
    out_instance: *mut *mut NuxViewModelInstance,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_instance.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe { *out_instance = ptr::null_mut() };
        let _call = enter_status_handle!(instance, HandleKind::ViewModel);
        let Some(instance) = (unsafe { instance.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let Ok(value) = instance.instance.try_borrow() else {
            return NuxStatus::ReentrantCall;
        };
        let handle = Box::into_raw(Box::new(NuxViewModelInstance {
            instance: RefCell::new(value.clone()),
            file: Arc::clone(&instance.file),
            schema_index: instance.schema_index,
            identity: instance.identity,
            owner_thread: instance.owner_thread,
            file_provenance: Arc::clone(&instance.file_provenance),
            binding_provenance: instance.binding_provenance.clone(),
            provenance: Arc::clone(&instance.provenance),
        }));
        drop(value);
        register_handle(handle, HandleKind::ViewModel, instance.owner_thread);
        unsafe { *out_instance = handle };
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_view_model_instance_identity(
    instance: *const NuxViewModelInstance,
    out_identity: *mut u64,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_identity.is_null() {
            return NuxStatus::NullArgument;
        }
        let _call = enter_status_handle!(instance, HandleKind::ViewModel);
        let Some(instance) = (unsafe { instance.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        unsafe { *out_identity = instance.identity };
        NuxStatus::Ok
    })
}

#[derive(Debug)]
struct OwnedSnapshotInstance {
    id: u64,
    schema_index: usize,
    first_value: usize,
    value_count: usize,
}

#[derive(Debug)]
enum OwnedSnapshotPayload {
    None,
    Bytes(Box<[u8]>),
    Number(f32),
    Integer(u64),
    Bool(bool),
    Reference(u64),
    List { first: usize, count: usize },
}

#[derive(Debug)]
struct OwnedSnapshotValue {
    owner_instance_id: u64,
    property_index: usize,
    name: Box<[u8]>,
    kind: u32,
    payload: OwnedSnapshotPayload,
}

pub struct NuxViewModelSnapshot {
    root_instance_id: u64,
    instances: Vec<OwnedSnapshotInstance>,
    values: Vec<OwnedSnapshotValue>,
    list_items: Vec<u64>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxViewModelSnapshotInfo {
    pub struct_size: u32,
    pub root_instance_id: u64,
    pub instance_count: usize,
    pub value_count: usize,
    pub list_item_count: usize,
}

impl Default for NuxViewModelSnapshotInfo {
    fn default() -> Self {
        Self {
            struct_size: std::mem::size_of::<Self>() as u32,
            root_instance_id: 0,
            instance_count: 0,
            value_count: 0,
            list_item_count: 0,
        }
    }
}

pub const NUX_VIEW_MODEL_SNAPSHOT_INFO_V3_MIN_SIZE: usize =
    std::mem::offset_of!(NuxViewModelSnapshotInfo, list_item_count) + std::mem::size_of::<usize>();

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxViewModelSnapshotInstanceView {
    pub struct_size: u32,
    pub instance_id: u64,
    pub schema_index: usize,
    pub first_value: usize,
    pub value_count: usize,
}

impl Default for NuxViewModelSnapshotInstanceView {
    fn default() -> Self {
        Self {
            struct_size: std::mem::size_of::<Self>() as u32,
            instance_id: 0,
            schema_index: 0,
            first_value: 0,
            value_count: 0,
        }
    }
}

pub const NUX_VIEW_MODEL_SNAPSHOT_INSTANCE_VIEW_V3_MIN_SIZE: usize =
    std::mem::offset_of!(NuxViewModelSnapshotInstanceView, value_count)
        + std::mem::size_of::<usize>();

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxViewModelSnapshotValueView {
    pub struct_size: u32,
    pub owner_instance_id: u64,
    pub property_index: usize,
    pub name: NuxStringView,
    pub kind: u32,
    pub number_value: f32,
    pub integer_value: u64,
    pub bool_value: u32,
    pub bytes_value: NuxByteView,
    pub referenced_instance_id: u64,
    pub first_list_item: usize,
    pub list_item_count: usize,
}

impl Default for NuxViewModelSnapshotValueView {
    fn default() -> Self {
        Self {
            struct_size: std::mem::size_of::<Self>() as u32,
            owner_instance_id: 0,
            property_index: 0,
            name: NuxStringView::default(),
            kind: NUX_VIEW_MODEL_VALUE_KIND_UNSUPPORTED,
            number_value: 0.0,
            integer_value: 0,
            bool_value: 0,
            bytes_value: NuxByteView::default(),
            referenced_instance_id: 0,
            first_list_item: 0,
            list_item_count: 0,
        }
    }
}

pub const NUX_VIEW_MODEL_SNAPSHOT_VALUE_VIEW_V3_MIN_SIZE: usize =
    std::mem::offset_of!(NuxViewModelSnapshotValueView, list_item_count)
        + std::mem::size_of::<usize>();

struct SnapshotBuilder<'a> {
    file: &'a File,
    handles: Vec<(RuntimeOwnedViewModelHandle, u64)>,
    instances: Vec<OwnedSnapshotInstance>,
    values: Vec<OwnedSnapshotValue>,
    list_items: Vec<u64>,
    next_id: u64,
    content_bytes: usize,
}

impl<'a> SnapshotBuilder<'a> {
    fn new(file: &'a File) -> Self {
        Self {
            file,
            handles: Vec::new(),
            instances: Vec::new(),
            values: Vec::new(),
            list_items: Vec::new(),
            next_id: 1,
            content_bytes: 0,
        }
    }

    fn id_for(
        &mut self,
        handle: &RuntimeOwnedViewModelHandle,
        preferred: Option<u64>,
    ) -> Result<u64, NuxStatus> {
        if let Some((_, id)) = self.handles.iter().find(|(known, _)| known.ptr_eq(handle)) {
            return Ok(*id);
        }
        if self.handles.len() >= MAX_SNAPSHOT_INSTANCES {
            return Err(NuxStatus::LimitExceeded);
        }
        let id = match preferred {
            Some(id) => id,
            None => {
                while self.handles.iter().any(|(_, known)| *known == self.next_id) {
                    self.next_id = self
                        .next_id
                        .checked_add(1)
                        .ok_or(NuxStatus::LimitExceeded)?;
                }
                let id = self.next_id;
                self.next_id = self
                    .next_id
                    .checked_add(1)
                    .ok_or(NuxStatus::LimitExceeded)?;
                id
            }
        };
        if id == 0 || self.handles.iter().any(|(_, known)| *known == id) {
            return Err(NuxStatus::LimitExceeded);
        }
        self.handles.push((handle.clone(), id));
        Ok(id)
    }

    fn snapshot(
        mut self,
        root: &RuntimeOwnedViewModelHandle,
        root_id: u64,
    ) -> Result<NuxViewModelSnapshot, NuxStatus> {
        self.id_for(root, Some(root_id))?;
        let mut cursor = 0;
        while cursor < self.handles.len() {
            let (handle, id) = self.handles[cursor].clone();
            self.snapshot_instance(&handle, id)?;
            cursor += 1;
        }
        Ok(NuxViewModelSnapshot {
            root_instance_id: root_id,
            instances: self.instances,
            values: self.values,
            list_items: self.list_items,
        })
    }

    fn snapshot_instance(
        &mut self,
        handle: &RuntimeOwnedViewModelHandle,
        id: u64,
    ) -> Result<(), NuxStatus> {
        let schema_index = handle.borrow().view_model_index();
        let schema = self
            .file
            .view_model(schema_index)
            .ok_or(NuxStatus::RuntimeError)?;
        let first_value = self.values.len();
        for property in schema.properties() {
            if self.values.len() >= MAX_SNAPSHOT_VALUES {
                return Err(NuxStatus::LimitExceeded);
            }
            let name = property.name().unwrap_or("");
            self.content_bytes = self
                .content_bytes
                .checked_add(name.len())
                .ok_or(NuxStatus::LimitExceeded)?;
            let kind = classify_property(property.type_name());
            let payload = self.snapshot_property(handle, name, kind)?;
            self.values.push(OwnedSnapshotValue {
                owner_instance_id: id,
                property_index: property.index(),
                name: name.as_bytes().to_vec().into_boxed_slice(),
                kind,
                payload,
            });
        }
        if self.content_bytes > MAX_TOTAL_BYTES {
            return Err(NuxStatus::LimitExceeded);
        }
        self.instances.push(OwnedSnapshotInstance {
            id,
            schema_index,
            first_value,
            value_count: self.values.len() - first_value,
        });
        Ok(())
    }

    fn snapshot_property(
        &mut self,
        handle: &RuntimeOwnedViewModelHandle,
        path: &str,
        kind: u32,
    ) -> Result<OwnedSnapshotPayload, NuxStatus> {
        let raw = handle.borrow();
        let payload = match kind {
            NUX_VIEW_MODEL_VALUE_KIND_STRING => raw
                .string_value_by_property_name_path(path)
                .map(|value| {
                    self.content_bytes = self.content_bytes.saturating_add(value.len());
                    OwnedSnapshotPayload::Bytes(value.to_vec().into_boxed_slice())
                })
                .unwrap_or(OwnedSnapshotPayload::None),
            NUX_VIEW_MODEL_VALUE_KIND_NUMBER => raw
                .number_value_by_property_name_path(path)
                .map(OwnedSnapshotPayload::Number)
                .unwrap_or(OwnedSnapshotPayload::None),
            NUX_VIEW_MODEL_VALUE_KIND_BOOL => raw
                .boolean_value_by_property_name_path(path)
                .map(OwnedSnapshotPayload::Bool)
                .unwrap_or(OwnedSnapshotPayload::None),
            NUX_VIEW_MODEL_VALUE_KIND_COLOR => raw
                .color_value_by_property_name_path(path)
                .map(|value| OwnedSnapshotPayload::Integer(u64::from(value)))
                .unwrap_or(OwnedSnapshotPayload::None),
            NUX_VIEW_MODEL_VALUE_KIND_ENUM => raw
                .enum_value_by_property_name_path(path)
                .map(OwnedSnapshotPayload::Integer)
                .unwrap_or(OwnedSnapshotPayload::None),
            NUX_VIEW_MODEL_VALUE_KIND_LIST_INDEX => raw
                .symbol_list_index_value_by_property_name_path(path)
                .map(OwnedSnapshotPayload::Integer)
                .unwrap_or(OwnedSnapshotPayload::None),
            NUX_VIEW_MODEL_VALUE_KIND_TRIGGER => raw
                .trigger_value_by_property_name_path(path)
                .map(OwnedSnapshotPayload::Integer)
                .unwrap_or(OwnedSnapshotPayload::None),
            NUX_VIEW_MODEL_VALUE_KIND_IMAGE => raw
                .asset_value_by_property_name_path(path)
                .map(OwnedSnapshotPayload::Integer)
                .unwrap_or(OwnedSnapshotPayload::None),
            NUX_VIEW_MODEL_VALUE_KIND_ARTBOARD => raw
                .artboard_value_by_property_name_path(path)
                .map(OwnedSnapshotPayload::Integer)
                .unwrap_or(OwnedSnapshotPayload::None),
            NUX_VIEW_MODEL_VALUE_KIND_BLOB => raw
                .blob_asset_value_by_property_name_path(path)
                .and_then(|value| value.live_blob_bytes().map(<[u8]>::to_vec))
                .map(|value| {
                    self.content_bytes = self.content_bytes.saturating_add(value.len());
                    OwnedSnapshotPayload::Bytes(value.into_boxed_slice())
                })
                .unwrap_or(OwnedSnapshotPayload::None),
            _ => OwnedSnapshotPayload::None,
        };
        drop(raw);
        match kind {
            NUX_VIEW_MODEL_VALUE_KIND_VIEW_MODEL => {
                let linked = handle.linked_view_model_by_property_name_path(path);
                Ok(match linked {
                    Some(linked) => OwnedSnapshotPayload::Reference(self.id_for(&linked, None)?),
                    None => OwnedSnapshotPayload::Reference(0),
                })
            }
            NUX_VIEW_MODEL_VALUE_KIND_LIST => {
                let items = handle
                    .list_items_by_property_name_path(path)
                    .ok_or(NuxStatus::RuntimeError)?;
                if self.list_items.len().saturating_add(items.len()) > MAX_LIST_ITEMS {
                    return Err(NuxStatus::LimitExceeded);
                }
                let first = self.list_items.len();
                for item in items {
                    let id = self.id_for(&item, None)?;
                    self.list_items.push(id);
                }
                Ok(OwnedSnapshotPayload::List {
                    first,
                    count: self.list_items.len() - first,
                })
            }
            _ => Ok(payload),
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_view_model_instance_snapshot(
    instance: *const NuxViewModelInstance,
    out_snapshot: *mut *mut NuxViewModelSnapshot,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_snapshot.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe { *out_snapshot = ptr::null_mut() };
        let _call = enter_status_handle!(instance, HandleKind::ViewModel);
        let Some(instance) = (unsafe { instance.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let Ok(view_model) = instance.instance.try_borrow() else {
            return NuxStatus::ReentrantCall;
        };
        let snapshot = match SnapshotBuilder::new(&instance.file)
            .snapshot(view_model.handle(), instance.identity)
        {
            Ok(snapshot) => snapshot,
            Err(status) => return status,
        };
        drop(view_model);
        let handle = Box::into_raw(Box::new(snapshot));
        register_handle(handle, HandleKind::ViewModelSnapshot, instance.owner_thread);
        unsafe { *out_snapshot = handle };
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_view_model_snapshot_free(
    snapshot: *mut NuxViewModelSnapshot,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if snapshot.is_null() {
            return NuxStatus::Ok;
        }
        if let Err(status) = remove_handle(snapshot, HandleKind::ViewModelSnapshot) {
            return status;
        }
        unsafe { drop(Box::from_raw(snapshot)) };
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_view_model_snapshot_info(
    snapshot: *const NuxViewModelSnapshot,
    out_info: *mut NuxViewModelSnapshotInfo,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let _call = enter_status_handle!(snapshot, HandleKind::ViewModelSnapshot);
        let Some(snapshot) = (unsafe { snapshot.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let value = NuxViewModelSnapshotInfo {
            root_instance_id: snapshot.root_instance_id,
            instance_count: snapshot.instances.len(),
            value_count: snapshot.values.len(),
            list_item_count: snapshot.list_items.len(),
            ..NuxViewModelSnapshotInfo::default()
        };
        unsafe { write_caller_struct(out_info, &value, NUX_VIEW_MODEL_SNAPSHOT_INFO_V3_MIN_SIZE) }
            .map_or_else(|status| status, |()| NuxStatus::Ok)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_view_model_snapshot_instance(
    snapshot: *const NuxViewModelSnapshot,
    index: usize,
    out_instance: *mut NuxViewModelSnapshotInstanceView,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let _call = enter_status_handle!(snapshot, HandleKind::ViewModelSnapshot);
        let Some(snapshot) = (unsafe { snapshot.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let Some(instance) = snapshot.instances.get(index) else {
            return NuxStatus::NotFound;
        };
        let value = NuxViewModelSnapshotInstanceView {
            instance_id: instance.id,
            schema_index: instance.schema_index,
            first_value: instance.first_value,
            value_count: instance.value_count,
            ..NuxViewModelSnapshotInstanceView::default()
        };
        unsafe {
            write_caller_struct(
                out_instance,
                &value,
                NUX_VIEW_MODEL_SNAPSHOT_INSTANCE_VIEW_V3_MIN_SIZE,
            )
        }
        .map_or_else(|status| status, |()| NuxStatus::Ok)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_view_model_snapshot_value(
    snapshot: *const NuxViewModelSnapshot,
    index: usize,
    out_value: *mut NuxViewModelSnapshotValueView,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let _call = enter_status_handle!(snapshot, HandleKind::ViewModelSnapshot);
        let Some(snapshot) = (unsafe { snapshot.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let Some(value) = snapshot.values.get(index) else {
            return NuxStatus::NotFound;
        };
        let mut view = NuxViewModelSnapshotValueView {
            owner_instance_id: value.owner_instance_id,
            property_index: value.property_index,
            name: owned_string_view(&value.name),
            kind: value.kind,
            ..NuxViewModelSnapshotValueView::default()
        };
        match &value.payload {
            OwnedSnapshotPayload::None => {}
            OwnedSnapshotPayload::Bytes(bytes) => view.bytes_value = owned_byte_view(bytes),
            OwnedSnapshotPayload::Number(number) => view.number_value = *number,
            OwnedSnapshotPayload::Integer(integer) => view.integer_value = *integer,
            OwnedSnapshotPayload::Bool(boolean) => view.bool_value = u32::from(*boolean),
            OwnedSnapshotPayload::Reference(id) => view.referenced_instance_id = *id,
            OwnedSnapshotPayload::List { first, count } => {
                view.first_list_item = *first;
                view.list_item_count = *count;
            }
        }
        unsafe {
            write_caller_struct(
                out_value,
                &view,
                NUX_VIEW_MODEL_SNAPSHOT_VALUE_VIEW_V3_MIN_SIZE,
            )
        }
        .map_or_else(|status| status, |()| NuxStatus::Ok)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_view_model_snapshot_list_item(
    snapshot: *const NuxViewModelSnapshot,
    index: usize,
    out_instance_id: *mut u64,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_instance_id.is_null() {
            return NuxStatus::NullArgument;
        }
        let _call = enter_status_handle!(snapshot, HandleKind::ViewModelSnapshot);
        let Some(snapshot) = (unsafe { snapshot.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let Some(id) = snapshot.list_items.get(index) else {
            return NuxStatus::NotFound;
        };
        unsafe { *out_instance_id = *id };
        NuxStatus::Ok
    })
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NuxViewModelMutationKind {
    SetString = 0,
    SetNumber = 1,
    SetBool = 2,
    SetColor = 3,
    SetEnum = 4,
    FireTrigger = 5,
    SetListIndex = 6,
    SetImage = 7,
    SetViewModel = 8,
    ListInsert = 9,
    ListRemove = 10,
    ListSwap = 11,
    ListMove = 12,
    ListSet = 13,
    ListClear = 14,
}

pub const NUX_VIEW_MODEL_MUTATION_KIND_SET_STRING: u32 = NuxViewModelMutationKind::SetString as u32;
pub const NUX_VIEW_MODEL_MUTATION_KIND_SET_NUMBER: u32 = NuxViewModelMutationKind::SetNumber as u32;
pub const NUX_VIEW_MODEL_MUTATION_KIND_SET_BOOL: u32 = NuxViewModelMutationKind::SetBool as u32;
pub const NUX_VIEW_MODEL_MUTATION_KIND_SET_COLOR: u32 = NuxViewModelMutationKind::SetColor as u32;
pub const NUX_VIEW_MODEL_MUTATION_KIND_SET_ENUM: u32 = NuxViewModelMutationKind::SetEnum as u32;
pub const NUX_VIEW_MODEL_MUTATION_KIND_FIRE_TRIGGER: u32 =
    NuxViewModelMutationKind::FireTrigger as u32;
pub const NUX_VIEW_MODEL_MUTATION_KIND_SET_LIST_INDEX: u32 =
    NuxViewModelMutationKind::SetListIndex as u32;
pub const NUX_VIEW_MODEL_MUTATION_KIND_SET_IMAGE: u32 = NuxViewModelMutationKind::SetImage as u32;
pub const NUX_VIEW_MODEL_MUTATION_KIND_SET_VIEW_MODEL: u32 =
    NuxViewModelMutationKind::SetViewModel as u32;
pub const NUX_VIEW_MODEL_MUTATION_KIND_LIST_INSERT: u32 =
    NuxViewModelMutationKind::ListInsert as u32;
pub const NUX_VIEW_MODEL_MUTATION_KIND_LIST_REMOVE: u32 =
    NuxViewModelMutationKind::ListRemove as u32;
pub const NUX_VIEW_MODEL_MUTATION_KIND_LIST_SWAP: u32 = NuxViewModelMutationKind::ListSwap as u32;
pub const NUX_VIEW_MODEL_MUTATION_KIND_LIST_MOVE: u32 = NuxViewModelMutationKind::ListMove as u32;
pub const NUX_VIEW_MODEL_MUTATION_KIND_LIST_SET: u32 = NuxViewModelMutationKind::ListSet as u32;
pub const NUX_VIEW_MODEL_MUTATION_KIND_LIST_CLEAR: u32 = NuxViewModelMutationKind::ListClear as u32;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxViewModelMutation {
    pub kind: u32,
    pub instance: *mut NuxViewModelInstance,
    pub path: NuxStringView,
    pub related_instance: *mut NuxViewModelInstance,
    pub bytes_value: NuxByteView,
    pub number_value: f32,
    pub integer_value: u64,
    /// Canonical C boolean: exactly 0 or 1.
    pub bool_value: u32,
    pub index: usize,
    pub second_index: usize,
}

impl Default for NuxViewModelMutation {
    fn default() -> Self {
        Self {
            kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_STRING,
            instance: ptr::null_mut(),
            path: NuxStringView::default(),
            related_instance: ptr::null_mut(),
            bytes_value: NuxByteView::default(),
            number_value: 0.0,
            integer_value: 0,
            bool_value: 0,
            index: 0,
            second_index: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxViewModelMutationBatch {
    pub struct_size: u32,
    pub mutations: *const NuxViewModelMutation,
    pub mutation_count: usize,
}

impl Default for NuxViewModelMutationBatch {
    fn default() -> Self {
        Self {
            struct_size: std::mem::size_of::<Self>() as u32,
            mutations: ptr::null(),
            mutation_count: 0,
        }
    }
}

pub const NUX_VIEW_MODEL_MUTATION_BATCH_V3_MIN_SIZE: usize =
    std::mem::offset_of!(NuxViewModelMutationBatch, mutation_count) + std::mem::size_of::<usize>();

unsafe fn read_view_model_mutation_batch(
    batch: *const NuxViewModelMutationBatch,
) -> Result<NuxViewModelMutationBatch, NuxStatus> {
    if batch.is_null() {
        return Err(NuxStatus::NullArgument);
    }
    let caller_size = unsafe { batch.cast::<u32>().read() };
    if !struct_size_supports(caller_size, NUX_VIEW_MODEL_MUTATION_BATCH_V3_MIN_SIZE) {
        return Err(NuxStatus::InvalidStructSize);
    }
    let mut value = NuxViewModelMutationBatch::default();
    let read_len = usize::try_from(caller_size)
        .unwrap_or(usize::MAX)
        .min(std::mem::size_of::<NuxViewModelMutationBatch>());
    unsafe {
        ptr::copy_nonoverlapping(
            batch.cast::<u8>(),
            (&mut value as *mut NuxViewModelMutationBatch).cast::<u8>(),
            read_len,
        );
    }
    Ok(value)
}

pub struct NuxViewModelMutationResult {
    status: NuxStatus,
    applied_count: usize,
    code: Box<[u8]>,
    message: Box<[u8]>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxViewModelMutationResultInfo {
    pub struct_size: u32,
    pub status: NuxStatus,
    pub applied_count: usize,
    /// Bounded diagnostic code bytes borrowed from the result until it is freed.
    pub code: NuxStringView,
    /// Bounded diagnostic message bytes borrowed from the result until it is freed.
    pub message: NuxStringView,
}

impl Default for NuxViewModelMutationResultInfo {
    fn default() -> Self {
        Self {
            struct_size: std::mem::size_of::<Self>() as u32,
            status: NuxStatus::Ok,
            applied_count: 0,
            code: NuxStringView::default(),
            message: NuxStringView::default(),
        }
    }
}

pub const NUX_VIEW_MODEL_MUTATION_RESULT_INFO_V3_MIN_SIZE: usize =
    std::mem::offset_of!(NuxViewModelMutationResultInfo, message)
        + std::mem::size_of::<NuxStringView>();

#[derive(Clone)]
struct ResolvedMutation {
    kind: u32,
    instance: usize,
    related: Option<usize>,
    path: String,
    bytes: Box<[u8]>,
    number: f32,
    integer: u64,
    boolean: bool,
    index: usize,
    second_index: usize,
}

fn mutation_error(status: NuxStatus, message: impl AsRef<[u8]>) -> (NuxStatus, Box<[u8]>) {
    (status, bounded_diagnostic_bytes(message))
}

fn validate_mutation_input(
    mutation: &NuxViewModelMutation,
) -> Result<ResolvedMutation, (NuxStatus, Box<[u8]>)> {
    if mutation.kind > NUX_VIEW_MODEL_MUTATION_KIND_LIST_CLEAR {
        return Err(mutation_error(
            NuxStatus::InvalidArgument,
            "unknown mutation kind",
        ));
    }
    let path = with_utf8_view(mutation.path, str::to_owned)
        .map_err(|status| mutation_error(status, "invalid property path"))?;
    if path.is_empty() {
        return Err(mutation_error(
            NuxStatus::InvalidArgument,
            "property path is empty",
        ));
    }
    if path.len() > MAX_PROPERTY_PATH_BYTES {
        return Err(mutation_error(
            NuxStatus::LimitExceeded,
            "property path is too long",
        ));
    }
    if mutation.instance.is_null() {
        return Err(mutation_error(NuxStatus::NullArgument, "instance is null"));
    }
    if mutation.bytes_value.data.is_null() && mutation.bytes_value.len != 0 {
        return Err(mutation_error(
            NuxStatus::NullArgument,
            "bytes value is null",
        ));
    }
    if mutation.bytes_value.len > MAX_VALUE_BYTES {
        return Err(mutation_error(
            NuxStatus::LimitExceeded,
            "bytes value is too large",
        ));
    }
    let bytes = if mutation.bytes_value.len == 0 {
        Vec::new()
    } else {
        unsafe {
            slice::from_raw_parts(mutation.bytes_value.data, mutation.bytes_value.len).to_vec()
        }
    };
    if mutation.kind == NUX_VIEW_MODEL_MUTATION_KIND_SET_NUMBER
        && !mutation.number_value.is_finite()
    {
        return Err(mutation_error(
            NuxStatus::InvalidArgument,
            "number must be finite",
        ));
    }
    if mutation.kind == NUX_VIEW_MODEL_MUTATION_KIND_SET_BOOL && mutation.bool_value > 1 {
        return Err(mutation_error(
            NuxStatus::InvalidArgument,
            "boolean must be 0 or 1",
        ));
    }
    let needs_related = matches!(
        mutation.kind,
        NUX_VIEW_MODEL_MUTATION_KIND_SET_VIEW_MODEL
            | NUX_VIEW_MODEL_MUTATION_KIND_LIST_INSERT
            | NUX_VIEW_MODEL_MUTATION_KIND_LIST_SET
    );
    if needs_related && mutation.related_instance.is_null() {
        return Err(mutation_error(
            NuxStatus::NullArgument,
            "related instance is null",
        ));
    }
    Ok(ResolvedMutation {
        kind: mutation.kind,
        instance: mutation.instance as usize,
        related: needs_related.then_some(mutation.related_instance as usize),
        path,
        bytes: bytes.into_boxed_slice(),
        number: mutation.number_value,
        integer: mutation.integer_value,
        boolean: mutation.bool_value == 1,
        index: mutation.index,
        second_index: mutation.second_index,
    })
}

fn apply_mutation(
    instances: &BTreeMap<usize, RuntimeOwnedViewModelHandle>,
    mutation: &ResolvedMutation,
) -> Result<(), (NuxStatus, Box<[u8]>)> {
    let owner = instances
        .get(&mutation.instance)
        .ok_or_else(|| mutation_error(NuxStatus::HandleMismatch, "instance is unavailable"))?;
    match mutation.kind {
        NUX_VIEW_MODEL_MUTATION_KIND_SET_STRING => {
            let mut raw = owner.borrow_mut();
            if raw
                .string_source_handle_by_property_name_path(&mutation.path)
                .is_none()
            {
                return Err(mutation_error(
                    NuxStatus::NotFound,
                    "string property was not found",
                ));
            }
            let _ = raw.set_string_by_property_name_path(&mutation.path, &mutation.bytes);
        }
        NUX_VIEW_MODEL_MUTATION_KIND_SET_NUMBER => {
            let mut raw = owner.borrow_mut();
            if raw
                .number_source_handle_by_property_name_path(&mutation.path)
                .is_none()
            {
                return Err(mutation_error(
                    NuxStatus::NotFound,
                    "number property was not found",
                ));
            }
            let _ = raw.set_number_by_property_name_path(&mutation.path, mutation.number);
        }
        NUX_VIEW_MODEL_MUTATION_KIND_SET_BOOL => {
            let mut raw = owner.borrow_mut();
            if raw
                .boolean_source_handle_by_property_name_path(&mutation.path)
                .is_none()
            {
                return Err(mutation_error(
                    NuxStatus::NotFound,
                    "boolean property was not found",
                ));
            }
            let _ = raw.set_boolean_by_property_name_path(&mutation.path, mutation.boolean);
        }
        NUX_VIEW_MODEL_MUTATION_KIND_SET_COLOR => {
            let mut raw = owner.borrow_mut();
            if raw
                .color_source_handle_by_property_name_path(&mutation.path)
                .is_none()
            {
                return Err(mutation_error(
                    NuxStatus::NotFound,
                    "color property was not found",
                ));
            }
            let value = u32::try_from(mutation.integer)
                .map_err(|_| mutation_error(NuxStatus::InvalidArgument, "color is out of range"))?;
            let _ = raw.set_color_by_property_name_path(&mutation.path, value);
        }
        NUX_VIEW_MODEL_MUTATION_KIND_SET_ENUM => {
            let mut raw = owner.borrow_mut();
            if raw
                .enum_source_handle_by_property_name_path(&mutation.path)
                .is_none()
            {
                return Err(mutation_error(
                    NuxStatus::NotFound,
                    "enum property was not found",
                ));
            }
            let _ = raw.set_enum_by_property_name_path(&mutation.path, mutation.integer);
        }
        NUX_VIEW_MODEL_MUTATION_KIND_FIRE_TRIGGER => {
            let mut raw = owner.borrow_mut();
            if raw
                .trigger_source_handle_by_property_name_path(&mutation.path)
                .is_none()
            {
                return Err(mutation_error(
                    NuxStatus::NotFound,
                    "trigger property was not found",
                ));
            }
            let next = raw
                .trigger_value_by_property_name_path(&mutation.path)
                .unwrap_or(0)
                .checked_add(1)
                .ok_or_else(|| {
                    mutation_error(NuxStatus::LimitExceeded, "trigger counter overflow")
                })?;
            let _ = raw.set_trigger_by_property_name_path(&mutation.path, next);
        }
        NUX_VIEW_MODEL_MUTATION_KIND_SET_LIST_INDEX => {
            let mut raw = owner.borrow_mut();
            if raw
                .symbol_list_index_source_handle_by_property_name_path(&mutation.path)
                .is_none()
            {
                return Err(mutation_error(
                    NuxStatus::NotFound,
                    "list-index property was not found",
                ));
            }
            let _ =
                raw.set_symbol_list_index_by_property_name_path(&mutation.path, mutation.integer);
        }
        NUX_VIEW_MODEL_MUTATION_KIND_SET_IMAGE => {
            let mut raw = owner.borrow_mut();
            if raw
                .asset_source_handle_by_property_name_path(&mutation.path)
                .is_none()
            {
                return Err(mutation_error(
                    NuxStatus::NotFound,
                    "image property was not found",
                ));
            }
            let _ = raw.set_asset_by_property_name_path(&mutation.path, mutation.integer);
        }
        NUX_VIEW_MODEL_MUTATION_KIND_SET_VIEW_MODEL => {
            let value = instances.get(&mutation.related.unwrap()).ok_or_else(|| {
                mutation_error(NuxStatus::HandleMismatch, "related instance is unavailable")
            })?;
            match owner.link_view_model_by_property_name_path(&mutation.path, value) {
                Ok(_) => {}
                Err(RuntimeViewModelLinkError::PropertyNotFound) => {
                    return Err(mutation_error(
                        NuxStatus::NotFound,
                        "view-model property was not found",
                    ));
                }
                Err(RuntimeViewModelLinkError::NestedPathUnsupported) => {
                    return Err(mutation_error(
                        NuxStatus::InvalidArgument,
                        "nested replacement path is unsupported",
                    ));
                }
                Err(
                    RuntimeViewModelLinkError::SchemaMismatch | RuntimeViewModelLinkError::Cycle,
                ) => {
                    return Err(mutation_error(
                        NuxStatus::InvalidArgument,
                        "view-model replacement is incompatible",
                    ));
                }
                Err(RuntimeViewModelLinkError::BorrowConflict) => {
                    return Err(mutation_error(
                        NuxStatus::ReentrantCall,
                        "view-model graph is borrowed",
                    ));
                }
            }
        }
        NUX_VIEW_MODEL_MUTATION_KIND_LIST_INSERT => {
            let item = instances.get(&mutation.related.unwrap()).unwrap();
            let count = owner
                .list_item_count_by_property_name_path(&mutation.path)
                .ok_or_else(|| {
                    mutation_error(NuxStatus::NotFound, "list property was not found")
                })?;
            if mutation.index > count || count >= MAX_LIST_ITEMS {
                return Err(mutation_error(
                    NuxStatus::InvalidArgument,
                    "list insert index is out of range",
                ));
            }
            if !owner.insert_list_item_by_property_name_path(&mutation.path, mutation.index, item) {
                return Err(mutation_error(
                    NuxStatus::InvalidArgument,
                    "list insert is incompatible",
                ));
            }
        }
        NUX_VIEW_MODEL_MUTATION_KIND_LIST_REMOVE => {
            let count = owner
                .list_item_count_by_property_name_path(&mutation.path)
                .ok_or_else(|| {
                    mutation_error(NuxStatus::NotFound, "list property was not found")
                })?;
            if mutation.index >= count
                || !owner.remove_list_item_by_property_name_path(&mutation.path, mutation.index)
            {
                return Err(mutation_error(
                    NuxStatus::InvalidArgument,
                    "list remove index is out of range",
                ));
            }
        }
        NUX_VIEW_MODEL_MUTATION_KIND_LIST_SWAP => {
            let count = owner
                .list_item_count_by_property_name_path(&mutation.path)
                .ok_or_else(|| {
                    mutation_error(NuxStatus::NotFound, "list property was not found")
                })?;
            if mutation.index >= count || mutation.second_index >= count {
                return Err(mutation_error(
                    NuxStatus::InvalidArgument,
                    "list swap index is out of range",
                ));
            }
            if mutation.index != mutation.second_index {
                let _ = owner.swap_list_items_by_property_name_path(
                    &mutation.path,
                    mutation.index,
                    mutation.second_index,
                );
            }
        }
        NUX_VIEW_MODEL_MUTATION_KIND_LIST_MOVE => {
            let count = owner
                .list_item_count_by_property_name_path(&mutation.path)
                .ok_or_else(|| {
                    mutation_error(NuxStatus::NotFound, "list property was not found")
                })?;
            if mutation.index >= count || mutation.second_index >= count {
                return Err(mutation_error(
                    NuxStatus::InvalidArgument,
                    "list move index is out of range",
                ));
            }
            if mutation.index != mutation.second_index {
                let _ = owner.move_list_item_by_property_name_path(
                    &mutation.path,
                    mutation.index,
                    mutation.second_index,
                );
            }
        }
        NUX_VIEW_MODEL_MUTATION_KIND_LIST_SET => {
            let item = instances.get(&mutation.related.unwrap()).unwrap();
            let count = owner
                .list_item_count_by_property_name_path(&mutation.path)
                .ok_or_else(|| {
                    mutation_error(NuxStatus::NotFound, "list property was not found")
                })?;
            if mutation.index >= count
                || !owner.set_list_item_by_property_name_path(&mutation.path, mutation.index, item)
            {
                return Err(mutation_error(
                    NuxStatus::InvalidArgument,
                    "list set is incompatible",
                ));
            }
        }
        NUX_VIEW_MODEL_MUTATION_KIND_LIST_CLEAR => {
            if owner
                .list_item_count_by_property_name_path(&mutation.path)
                .is_none()
            {
                return Err(mutation_error(
                    NuxStatus::NotFound,
                    "list property was not found",
                ));
            }
            let _ = owner.clear_list_items_by_property_name_path(&mutation.path);
        }
        _ => unreachable!("mutation kind was validated"),
    }
    Ok(())
}

fn record_mutation_undo(
    undo: &mut RuntimeOwnedViewModelUndoLog,
    instances: &BTreeMap<usize, RuntimeOwnedViewModelHandle>,
    mutation: &ResolvedMutation,
) -> bool {
    let Some(owner) = instances.get(&mutation.instance) else {
        return false;
    };
    match mutation.kind {
        NUX_VIEW_MODEL_MUTATION_KIND_SET_STRING => undo.record_string(owner, &mutation.path),
        NUX_VIEW_MODEL_MUTATION_KIND_SET_NUMBER => undo.record_number(owner, &mutation.path),
        NUX_VIEW_MODEL_MUTATION_KIND_SET_BOOL => undo.record_boolean(owner, &mutation.path),
        NUX_VIEW_MODEL_MUTATION_KIND_SET_COLOR => undo.record_color(owner, &mutation.path),
        NUX_VIEW_MODEL_MUTATION_KIND_SET_ENUM => undo.record_enum(owner, &mutation.path),
        NUX_VIEW_MODEL_MUTATION_KIND_FIRE_TRIGGER => undo.record_trigger(owner, &mutation.path),
        NUX_VIEW_MODEL_MUTATION_KIND_SET_LIST_INDEX => {
            undo.record_list_index(owner, &mutation.path)
        }
        NUX_VIEW_MODEL_MUTATION_KIND_SET_IMAGE => undo.record_asset(owner, &mutation.path),
        NUX_VIEW_MODEL_MUTATION_KIND_SET_VIEW_MODEL => {
            undo.record_view_model(owner, &mutation.path)
        }
        NUX_VIEW_MODEL_MUTATION_KIND_LIST_INSERT
        | NUX_VIEW_MODEL_MUTATION_KIND_LIST_REMOVE
        | NUX_VIEW_MODEL_MUTATION_KIND_LIST_SWAP
        | NUX_VIEW_MODEL_MUTATION_KIND_LIST_MOVE
        | NUX_VIEW_MODEL_MUTATION_KIND_LIST_SET
        | NUX_VIEW_MODEL_MUTATION_KIND_LIST_CLEAR => undo.record_list(owner, &mutation.path),
        _ => false,
    }
}

fn publish_mutation_result(
    out_result: *mut *mut NuxViewModelMutationResult,
    status: NuxStatus,
    applied_count: usize,
    message: impl AsRef<[u8]>,
) {
    let result = Box::into_raw(Box::new(NuxViewModelMutationResult {
        status,
        applied_count,
        code: bounded_diagnostic_bytes(status_code(status)),
        message: bounded_diagnostic_bytes(message),
    }));
    register_handle(
        result,
        HandleKind::ViewModelMutationResult,
        thread::current().id(),
    );
    unsafe { *out_result = result };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_view_model_mutate(
    batch: *const NuxViewModelMutationBatch,
    out_result: *mut *mut NuxViewModelMutationResult,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_result.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe { *out_result = ptr::null_mut() };
        let batch = match unsafe { read_view_model_mutation_batch(batch) } {
            Ok(batch) => batch,
            Err(status) => {
                publish_mutation_result(
                    out_result,
                    status,
                    0,
                    if status == NuxStatus::NullArgument {
                        "batch is null"
                    } else {
                        "batch prefix is too small"
                    },
                );
                return status;
            }
        };
        if batch.mutation_count > MAX_MUTATIONS {
            publish_mutation_result(
                out_result,
                NuxStatus::LimitExceeded,
                0,
                "mutation count exceeds the limit",
            );
            return NuxStatus::LimitExceeded;
        }
        if batch.mutations.is_null() && batch.mutation_count != 0 {
            publish_mutation_result(
                out_result,
                NuxStatus::NullArgument,
                0,
                "mutation array is null",
            );
            return NuxStatus::NullArgument;
        }
        let mutations = if batch.mutation_count == 0 {
            &[]
        } else {
            unsafe { slice::from_raw_parts(batch.mutations, batch.mutation_count) }
        };
        let mut resolved = Vec::with_capacity(mutations.len());
        let mut total_bytes = 0usize;
        for mutation in mutations {
            let mutation = match validate_mutation_input(mutation) {
                Ok(mutation) => mutation,
                Err((status, message)) => {
                    publish_mutation_result(out_result, status, 0, message);
                    return status;
                }
            };
            total_bytes = match total_bytes
                .checked_add(mutation.path.len())
                .and_then(|total| total.checked_add(mutation.bytes.len()))
            {
                Some(total) if total <= MAX_TOTAL_BYTES => total,
                _ => {
                    publish_mutation_result(
                        out_result,
                        NuxStatus::LimitExceeded,
                        0,
                        "mutation payload exceeds the limit",
                    );
                    return NuxStatus::LimitExceeded;
                }
            };
            resolved.push(mutation);
        }

        let mut addresses = BTreeSet::new();
        for mutation in &resolved {
            addresses.insert(mutation.instance);
            if let Some(related) = mutation.related {
                addresses.insert(related);
            }
        }
        let mut guards = Vec::with_capacity(addresses.len());
        for address in &addresses {
            match enter_handle(
                *address as *const NuxViewModelInstance,
                HandleKind::ViewModel,
            ) {
                Ok(guard) => guards.push(guard),
                Err(status) => {
                    publish_mutation_result(
                        out_result,
                        status,
                        0,
                        "view-model handle validation failed",
                    );
                    return status;
                }
            }
        }
        let mut live = BTreeMap::new();
        let mut provenance: Option<Arc<()>> = None;
        for address in addresses {
            let instance = unsafe { &*(address as *const NuxViewModelInstance) };
            if provenance
                .as_ref()
                .is_some_and(|known| !Arc::ptr_eq(known, &instance.file_provenance))
            {
                publish_mutation_result(
                    out_result,
                    NuxStatus::HandleMismatch,
                    0,
                    "instances come from different files",
                );
                return NuxStatus::HandleMismatch;
            }
            provenance.get_or_insert_with(|| Arc::clone(&instance.file_provenance));
            let Ok(value) = instance.instance.try_borrow() else {
                publish_mutation_result(
                    out_result,
                    NuxStatus::ReentrantCall,
                    0,
                    "view-model graph is borrowed",
                );
                return NuxStatus::ReentrantCall;
            };
            live.insert(address, value.handle().clone());
        }

        // Prevalidation runs the exact ordered batch on an identity-preserving
        // detached graph. No live cell is dirtied unless every operation can
        // succeed in sequence.
        let source_handles = live.values().cloned().collect::<Vec<_>>();
        let candidate_handles = RuntimeOwnedViewModelHandle::detached_graph(&source_handles);
        let candidates = live
            .keys()
            .copied()
            .zip(candidate_handles)
            .collect::<BTreeMap<_, _>>();
        for mutation in &resolved {
            if let Err((status, message)) = apply_mutation(&candidates, mutation) {
                publish_mutation_result(out_result, status, 0, message);
                return status;
            }
        }
        // Record every inverse before its corresponding write. The journal
        // retains exact runtime handles, so rollback preserves identities
        // already observed through artboards, scripts, and shared C handles.
        let mut undo = RuntimeOwnedViewModelUndoLog::new();
        let commit = panic::catch_unwind(AssertUnwindSafe(|| {
            for (index, mutation) in resolved.iter().enumerate() {
                if !record_mutation_undo(&mut undo, &live, mutation) {
                    return Err(());
                }
                if apply_mutation(&live, mutation).is_err() {
                    return Err(());
                }
                maybe_panic_during_vm_commit(index + 1);
            }
            Ok(())
        }));
        if !matches!(commit, Ok(Ok(()))) {
            let restored = panic::catch_unwind(AssertUnwindSafe(|| undo.rollback()))
                .is_ok_and(|restored| restored);
            publish_mutation_result(
                out_result,
                NuxStatus::RuntimeError,
                0,
                if restored {
                    "validated mutation diverged during commit; live graph restored"
                } else {
                    "validated mutation diverged and rollback failed"
                },
            );
            return NuxStatus::RuntimeError;
        }
        undo.commit();
        drop(guards);
        publish_mutation_result(out_result, NuxStatus::Ok, resolved.len(), "");
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_view_model_mutation_result_info(
    result: *const NuxViewModelMutationResult,
    out_info: *mut NuxViewModelMutationResultInfo,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let _call = enter_status_handle!(result, HandleKind::ViewModelMutationResult);
        let Some(result) = (unsafe { result.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let value = NuxViewModelMutationResultInfo {
            status: result.status,
            applied_count: result.applied_count,
            code: owned_string_view(&result.code),
            message: owned_string_view(&result.message),
            ..NuxViewModelMutationResultInfo::default()
        };
        unsafe {
            write_caller_struct(
                out_info,
                &value,
                NUX_VIEW_MODEL_MUTATION_RESULT_INFO_V3_MIN_SIZE,
            )
        }
        .map_or_else(|status| status, |()| NuxStatus::Ok)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_view_model_mutation_result_free(
    result: *mut NuxViewModelMutationResult,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if result.is_null() {
            return NuxStatus::Ok;
        }
        if let Err(status) = remove_handle(result, HandleKind::ViewModelMutationResult) {
            return status;
        }
        unsafe { drop(Box::from_raw(result)) };
        NuxStatus::Ok
    })
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct NuxTextRunMutation {
    /// Exact, case-sensitive authored root `TextValueRun` name.
    pub name: NuxStringView,
    /// Replacement text bytes, borrowed only for the synchronous call.
    pub text: NuxByteView,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxTextRunMutationBatch {
    pub struct_size: u32,
    pub mutations: *const NuxTextRunMutation,
    pub mutation_count: usize,
}

impl Default for NuxTextRunMutationBatch {
    fn default() -> Self {
        Self {
            struct_size: std::mem::size_of::<Self>() as u32,
            mutations: ptr::null(),
            mutation_count: 0,
        }
    }
}

pub const NUX_TEXT_RUN_MUTATION_BATCH_V3_MIN_SIZE: usize =
    std::mem::offset_of!(NuxTextRunMutationBatch, mutation_count) + std::mem::size_of::<usize>();

unsafe fn read_text_run_mutation_batch(
    batch: *const NuxTextRunMutationBatch,
) -> Result<NuxTextRunMutationBatch, NuxStatus> {
    if batch.is_null() {
        return Err(NuxStatus::NullArgument);
    }
    let caller_size = unsafe { batch.cast::<u32>().read() };
    if !struct_size_supports(caller_size, NUX_TEXT_RUN_MUTATION_BATCH_V3_MIN_SIZE) {
        return Err(NuxStatus::InvalidStructSize);
    }
    let mut value = NuxTextRunMutationBatch::default();
    let read_len = usize::try_from(caller_size)
        .unwrap_or(usize::MAX)
        .min(std::mem::size_of::<NuxTextRunMutationBatch>());
    unsafe {
        ptr::copy_nonoverlapping(
            batch.cast::<u8>(),
            (&mut value as *mut NuxTextRunMutationBatch).cast::<u8>(),
            read_len,
        );
    }
    Ok(value)
}

/// Atomically replace a bounded batch of exact-name root text runs.
///
/// Every name and buffer is validated before the first write. `out_changed` is
/// optional and receives canonical 0/1. An unexpected commit divergence or
/// panic restores every earlier write through the runtime undo journal.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_artboard_instance_set_text_runs(
    instance: *mut NuxArtboardInstance,
    batch: *const NuxTextRunMutationBatch,
    out_changed: *mut u32,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if !out_changed.is_null() {
            unsafe { *out_changed = 0 };
        }
        let _instance_call = enter_status_handle!(instance, HandleKind::Artboard);
        let Some(instance) = (unsafe { instance.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let batch = match unsafe { read_text_run_mutation_batch(batch) } {
            Ok(batch) => batch,
            Err(status) => return status,
        };
        if batch.mutation_count > MAX_MUTATIONS {
            return NuxStatus::LimitExceeded;
        }
        if batch.mutations.is_null() && batch.mutation_count != 0 {
            return NuxStatus::NullArgument;
        }
        let mutations = if batch.mutation_count == 0 {
            &[]
        } else {
            unsafe { slice::from_raw_parts(batch.mutations, batch.mutation_count) }
        };
        let mut resolved = Vec::with_capacity(mutations.len());
        let mut total_bytes = 0usize;
        for mutation in mutations {
            let name = match with_utf8_view(mutation.name, str::to_owned) {
                Ok(name) if !name.is_empty() => name,
                Ok(_) => return NuxStatus::InvalidArgument,
                Err(status) => return status,
            };
            if name.len() > MAX_PROPERTY_PATH_BYTES || mutation.text.len > MAX_VALUE_BYTES {
                return NuxStatus::LimitExceeded;
            }
            if mutation.text.data.is_null() && mutation.text.len != 0 {
                return NuxStatus::NullArgument;
            }
            total_bytes = match total_bytes
                .checked_add(name.len())
                .and_then(|total| total.checked_add(mutation.text.len))
            {
                Some(total) if total <= MAX_TOTAL_BYTES => total,
                _ => return NuxStatus::LimitExceeded,
            };
            let text = if mutation.text.len == 0 {
                Vec::new()
            } else {
                unsafe { slice::from_raw_parts(mutation.text.data, mutation.text.len).to_vec() }
            };
            resolved.push((name, text));
        }

        let _occurrence_call = match enter_occurrence(&instance.occurrence) {
            Ok(guard) => guard,
            Err(status) => return status,
        };
        let Ok(mut artboard) = instance.occurrence.instance.try_borrow_mut() else {
            return NuxStatus::ReentrantCall;
        };
        let names = resolved
            .iter()
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>();
        let Some(undo) = artboard.raw().root_text_value_run_undo_log(&names) else {
            return NuxStatus::NotFound;
        };
        let commit = panic::catch_unwind(AssertUnwindSafe(|| {
            let mut changed = false;
            for (index, (name, text)) in resolved.into_iter().enumerate() {
                let Some(did_change) = artboard.raw_mut().set_root_text_value_run(&name, text)
                else {
                    return Err(());
                };
                changed |= did_change;
                maybe_panic_during_text_commit(index + 1);
            }
            Ok(changed)
        }));
        let changed = match commit {
            Ok(Ok(changed)) => changed,
            Ok(Err(())) | Err(_) => {
                let restored =
                    panic::catch_unwind(AssertUnwindSafe(|| undo.rollback(artboard.raw_mut())))
                        .is_ok_and(|restored| restored);
                if !restored {
                    instance.occurrence.poisoned.set(true);
                }
                return NuxStatus::RuntimeError;
            }
        };
        if !out_changed.is_null() {
            unsafe { *out_changed = u32::from(changed) };
        }
        NuxStatus::Ok
    })
}

#[cfg(test)]
mod transaction_tests {
    use super::*;
    use std::path::PathBuf;

    fn import_fixture(name: &str) -> *mut NuxFile {
        let root = std::env::var_os("NUX_RUNTIME_DIR")
            .or_else(|| std::env::var_os("RIVE_RUNTIME_DIR"))
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
        let bytes = std::fs::read(
            PathBuf::from(root)
                .join("tests/unit_tests/assets")
                .join(name),
        )
        .expect("read upstream fixture");
        let mut file = ptr::null_mut();
        assert_eq!(
            unsafe { nux_file_import(bytes.as_ptr(), bytes.len(), &mut file) },
            NuxStatus::Ok
        );
        file
    }

    fn schema_with_property(file: *const NuxFile, name: &str) -> usize {
        let mut catalog = ptr::null_mut();
        assert_eq!(
            unsafe { nux_file_view_model_catalog(file, &mut catalog) },
            NuxStatus::Ok
        );
        let catalog_ref = unsafe { &*catalog };
        let schema = catalog_ref
            .properties
            .iter()
            .find(|property| property.name.as_ref() == name.as_bytes())
            .expect("fixture property")
            .schema_index;
        assert_eq!(
            unsafe { nux_view_model_catalog_free(catalog) },
            NuxStatus::Ok
        );
        schema
    }

    fn number_value(instance: *const NuxViewModelInstance, path: &str) -> f32 {
        let instance = unsafe { &*instance };
        instance
            .instance
            .borrow()
            .raw()
            .number_value_by_property_name_path(path)
            .expect("number value")
    }

    #[test]
    fn late_view_model_commit_panic_restores_shared_observer_and_handle_remains_usable() {
        let file = import_fixture("data_binding_test_2.riv");
        let schema = schema_with_property(file, "num");
        let mut instance = ptr::null_mut();
        assert_eq!(
            unsafe { nux_view_model_instance_new(file, schema, &mut instance) },
            NuxStatus::Ok
        );
        let mut shared = ptr::null_mut();
        assert_eq!(
            unsafe { nux_view_model_instance_share(instance, &mut shared) },
            NuxStatus::Ok
        );
        let original = number_value(shared, "num");
        let path = b"num";
        let mutations = [91.0, 92.0].map(|number_value| NuxViewModelMutation {
            kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_NUMBER,
            instance,
            path: NuxStringView {
                data: path.as_ptr().cast(),
                len: path.len(),
            },
            number_value,
            ..NuxViewModelMutation::default()
        });
        let batch = NuxViewModelMutationBatch {
            mutations: mutations.as_ptr(),
            mutation_count: mutations.len(),
            ..NuxViewModelMutationBatch::default()
        };
        inject_vm_commit_panic_after(Some(1));
        let mut result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_view_model_mutate(&batch, &mut result) },
            NuxStatus::RuntimeError
        );
        inject_vm_commit_panic_after(None);
        assert_eq!(number_value(shared, "num"), original);
        assert_eq!(
            unsafe { nux_view_model_mutation_result_free(result) },
            NuxStatus::Ok
        );

        let one = NuxViewModelMutationBatch {
            mutation_count: 1,
            ..batch
        };
        assert_eq!(
            unsafe { nux_view_model_mutate(&one, &mut result) },
            NuxStatus::Ok
        );
        assert_eq!(number_value(shared, "num"), 91.0);
        unsafe {
            nux_view_model_mutation_result_free(result);
            nux_view_model_instance_free(shared);
            nux_view_model_instance_free(instance);
            nux_file_free(file);
        }
    }

    #[test]
    fn late_text_run_commit_panic_restores_occurrence_and_it_remains_usable() {
        let file = import_fixture("background_measure.riv");
        let mut artboard = ptr::null_mut();
        assert_eq!(
            unsafe { nux_artboard_instance_new(file, 0, &mut artboard) },
            NuxStatus::Ok
        );
        let name = b"nameRun";
        let first = b"first";
        let second = b"second";
        let original = unsafe { &*artboard }
            .occurrence
            .instance
            .borrow()
            .raw()
            .root_text_value_run("nameRun")
            .expect("text run")
            .to_vec();
        let mutations = [first.as_slice(), second.as_slice()].map(|text| NuxTextRunMutation {
            name: NuxStringView {
                data: name.as_ptr().cast(),
                len: name.len(),
            },
            text: NuxByteView {
                data: text.as_ptr(),
                len: text.len(),
            },
        });
        let batch = NuxTextRunMutationBatch {
            mutations: mutations.as_ptr(),
            mutation_count: mutations.len(),
            ..NuxTextRunMutationBatch::default()
        };
        inject_text_commit_panic_after(Some(1));
        assert_eq!(
            unsafe { nux_artboard_instance_set_text_runs(artboard, &batch, ptr::null_mut()) },
            NuxStatus::RuntimeError
        );
        inject_text_commit_panic_after(None);
        assert_eq!(
            unsafe { &*artboard }
                .occurrence
                .instance
                .borrow()
                .raw()
                .root_text_value_run("nameRun"),
            Some(original.as_slice())
        );

        let one = NuxTextRunMutationBatch {
            mutation_count: 1,
            ..batch
        };
        let mut changed = 0;
        assert_eq!(
            unsafe { nux_artboard_instance_set_text_runs(artboard, &one, &mut changed) },
            NuxStatus::Ok
        );
        assert_eq!(changed, 1);
        unsafe {
            nux_artboard_instance_free(artboard);
            nux_file_free(file);
        }
    }
}
