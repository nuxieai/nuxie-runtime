use super::*;
use crate::mechanical_port::source::bindable_artboard::RuntimeBindableArtboardHandle;
pub use crate::mechanical_port::source::data_bind::data_values::data_type::DataType as ViewModelRuntimeDataType;
use crate::mechanical_port::source::viewmodel::runtime as native;
pub use crate::mechanical_port::source::viewmodel::runtime::viewmodel_instance_runtime::PropertyData as ViewModelRuntimeProperty;
use crate::mechanical_port::source::viewmodel::runtime::viewmodel_instance_runtime::ViewModelInstanceRuntime as NativeInstance;
use crate::mechanical_port::source::viewmodel::runtime::viewmodel_instance_value_runtime::ViewModelInstanceValueRuntime as NativeValue;
use crate::mechanical_port::source::viewmodel::runtime::viewmodel_runtime::RuntimeViewModelHandle as NativeModel;
use nuxie_render_api::RenderImage;
#[derive(Clone)]
pub struct ViewModelInstanceValueRuntime {
    native: NativeValue,
    file: RuntimeFileHandle,
    name: String,
}
impl std::fmt::Debug for ViewModelInstanceValueRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ViewModelInstanceValueRuntime")
            .field("handle", &self.native.handle())
            .finish()
    }
}
impl ViewModelInstanceValueRuntime {
    fn new(native: NativeValue, file: RuntimeFileHandle) -> Self {
        let name = native.name();
        Self { native, file, name }
    }
    pub fn native_handle(&self) -> CoreHandle {
        self.native.handle()
    }
    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.native.handle() == other.native.handle()
    }
    pub fn data_type(&self) -> ViewModelRuntimeDataType {
        self.native.data_type()
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn has_changed(&self) -> bool {
        self.native.has_changed()
    }
    pub fn clear_changes(&self) {
        self.native.clear_changes();
    }
    pub fn flush_changes(&self) -> bool {
        self.native.flush_changes()
    }
}
#[derive(Clone)]
pub struct ViewModelInstanceRuntime {
    native: Rc<NativeInstance>,
    handle: RuntimeOwnedViewModelHandle,
}
impl std::fmt::Debug for ViewModelInstanceRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ViewModelInstanceRuntime")
            .field("handle", &self.handle)
            .finish()
    }
}
impl ViewModelInstanceRuntime {
    pub fn from_native(file: RuntimeFileHandle, instance: CoreHandle) -> Option<Self> {
        let handle = RuntimeOwnedViewModelHandle::from_native(file, instance.clone())?;
        Some(Self {
            native: Rc::new(NativeInstance::new(instance)),
            handle,
        })
    }
    pub fn new(file: RuntimeFileHandle, handle: RuntimeOwnedViewModelHandle) -> Self {
        Self::from_native(file, handle.native_handle()).expect("native instance")
    }
    pub fn from_handle(file: RuntimeFileHandle, handle: RuntimeOwnedViewModelHandle) -> Self {
        Self::new(file, handle)
    }
    pub fn native_handle(&self) -> CoreHandle {
        self.native.instance()
    }
    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.native_handle() == other.native_handle()
    }
    pub fn handle(&self) -> &RuntimeOwnedViewModelHandle {
        &self.handle
    }
    pub fn name(&self) -> String {
        self.native.name()
    }
    pub fn view_model_name(&self) -> String {
        self.native.view_model_name()
    }
    pub fn property_count(&self) -> usize {
        self.native.property_count()
    }
    pub fn properties(&self) -> Vec<ViewModelRuntimeProperty> {
        self.native.properties()
    }
    pub fn property_view_model(&self, path: &str) -> Option<Self> {
        Self::from_native(
            self.handle.native_file(),
            self.native.property_view_model(path)?.instance(),
        )
    }
    pub fn view_model_instance_at_path(&self, path: &str) -> Option<Self> {
        self.property_view_model(path)
    }
    pub fn replace_view_model(&self, path: &str, value: &Self) -> bool {
        self.handle
            .link_view_model_by_property_name_path(path, value.handle())
            .unwrap_or(false)
    }
    pub fn property_number(&self, path: &str) -> Option<ViewModelInstanceNumberRuntime> {
        let native = self.native.property_number(path)?;
        let value = ViewModelInstanceValueRuntime::new(
            native.value_runtime().clone(),
            self.handle.native_file(),
        );
        Some(ViewModelInstanceNumberRuntime { native, value })
    }
    pub fn property_boolean(&self, path: &str) -> Option<ViewModelInstanceBooleanRuntime> {
        let native = self.native.property_boolean(path)?;
        let value = ViewModelInstanceValueRuntime::new(
            native.value_runtime().clone(),
            self.handle.native_file(),
        );
        Some(ViewModelInstanceBooleanRuntime { native, value })
    }
    pub fn property_color(&self, path: &str) -> Option<ViewModelInstanceColorRuntime> {
        let native = self.native.property_color(path)?;
        let value = ViewModelInstanceValueRuntime::new(
            native.value_runtime().clone(),
            self.handle.native_file(),
        );
        Some(ViewModelInstanceColorRuntime { native, value })
    }
    pub fn property_string(&self, path: &str) -> Option<ViewModelInstanceStringRuntime> {
        let native = self.native.property_string(path)?;
        let value = ViewModelInstanceValueRuntime::new(
            native.value_runtime().clone(),
            self.handle.native_file(),
        );
        Some(ViewModelInstanceStringRuntime { native, value })
    }
    pub fn property_enum(&self, path: &str) -> Option<ViewModelInstanceEnumRuntime> {
        let native = self.native.property_enum(path)?;
        let value = ViewModelInstanceValueRuntime::new(
            native.value_runtime().clone(),
            self.handle.native_file(),
        );
        Some(ViewModelInstanceEnumRuntime { native, value })
    }
    pub fn property_trigger(&self, path: &str) -> Option<ViewModelInstanceTriggerRuntime> {
        let native = self.native.property_trigger(path)?;
        let value = ViewModelInstanceValueRuntime::new(
            native.value_runtime().clone(),
            self.handle.native_file(),
        );
        Some(ViewModelInstanceTriggerRuntime { native, value })
    }
    pub fn property_list_index(&self, path: &str) -> Option<ViewModelInstanceListIndexRuntime> {
        let native = self.native.property_list_index(path)?;
        let value = ViewModelInstanceValueRuntime::new(
            native.value_runtime().clone(),
            self.handle.native_file(),
        );
        Some(ViewModelInstanceListIndexRuntime { native, value })
    }
    pub fn property_list(&self, path: &str) -> Option<ViewModelInstanceListRuntime> {
        let native = self.native.property_list(path)?;
        let value = ViewModelInstanceValueRuntime::new(
            native.value_runtime().clone(),
            self.handle.native_file(),
        );
        Some(ViewModelInstanceListRuntime { native, value })
    }
    pub fn property_image(&self, path: &str) -> Option<ViewModelInstanceAssetImageRuntime> {
        let native = self.native.property_image(path)?;
        let value = ViewModelInstanceValueRuntime::new(
            native.value_runtime().clone(),
            self.handle.native_file(),
        );
        Some(ViewModelInstanceAssetImageRuntime { native, value })
    }
    pub fn property_font(&self, path: &str) -> Option<ViewModelInstanceAssetFontRuntime> {
        let native = self.native.property_font(path)?;
        let value = ViewModelInstanceValueRuntime::new(
            native.value_runtime().clone(),
            self.handle.native_file(),
        );
        Some(ViewModelInstanceAssetFontRuntime { native, value })
    }
    pub fn property_blob(&self, path: &str) -> Option<ViewModelInstanceAssetBlobRuntime> {
        let native = self.native.property_blob(path)?;
        let value = ViewModelInstanceValueRuntime::new(
            native.value_runtime().clone(),
            self.handle.native_file(),
        );
        Some(ViewModelInstanceAssetBlobRuntime { native, value })
    }
    pub fn property_artboard(&self, path: &str) -> Option<ViewModelInstanceArtboardRuntime> {
        let native = self.native.property_artboard(path)?;
        let value = ViewModelInstanceValueRuntime::new(
            native.value_runtime().clone(),
            self.handle.native_file(),
        );
        Some(ViewModelInstanceArtboardRuntime { native, value })
    }
    pub fn property(&self, path: &str) -> Option<ViewModelInstanceRuntimeProperty> {
        let kind = self.native.property(path)?.data_type();
        Some(match kind {
            ViewModelRuntimeDataType::Number => {
                ViewModelInstanceRuntimeProperty::Number(self.property_number(path)?)
            }
            ViewModelRuntimeDataType::Boolean => {
                ViewModelInstanceRuntimeProperty::Boolean(self.property_boolean(path)?)
            }
            ViewModelRuntimeDataType::Color => {
                ViewModelInstanceRuntimeProperty::Color(self.property_color(path)?)
            }
            ViewModelRuntimeDataType::String => {
                ViewModelInstanceRuntimeProperty::String(self.property_string(path)?)
            }
            ViewModelRuntimeDataType::Enum => {
                ViewModelInstanceRuntimeProperty::Enum(self.property_enum(path)?)
            }
            ViewModelRuntimeDataType::Trigger => {
                ViewModelInstanceRuntimeProperty::Trigger(self.property_trigger(path)?)
            }
            ViewModelRuntimeDataType::SymbolListIndex => {
                ViewModelInstanceRuntimeProperty::ListIndex(self.property_list_index(path)?)
            }
            ViewModelRuntimeDataType::List => {
                ViewModelInstanceRuntimeProperty::List(self.property_list(path)?)
            }
            ViewModelRuntimeDataType::AssetImage => {
                ViewModelInstanceRuntimeProperty::AssetImage(self.property_image(path)?)
            }
            ViewModelRuntimeDataType::AssetFont => {
                ViewModelInstanceRuntimeProperty::AssetFont(self.property_font(path)?)
            }
            ViewModelRuntimeDataType::AssetBlob => {
                ViewModelInstanceRuntimeProperty::AssetBlob(self.property_blob(path)?)
            }
            ViewModelRuntimeDataType::Artboard => {
                ViewModelInstanceRuntimeProperty::Artboard(self.property_artboard(path)?)
            }
            _ => return None,
        })
    }
}
#[derive(Clone, Debug)]
pub enum ViewModelInstanceRuntimeProperty {
    Number(ViewModelInstanceNumberRuntime),
    Boolean(ViewModelInstanceBooleanRuntime),
    Color(ViewModelInstanceColorRuntime),
    String(ViewModelInstanceStringRuntime),
    Enum(ViewModelInstanceEnumRuntime),
    Trigger(ViewModelInstanceTriggerRuntime),
    ListIndex(ViewModelInstanceListIndexRuntime),
    List(ViewModelInstanceListRuntime),
    AssetImage(ViewModelInstanceAssetImageRuntime),
    AssetFont(ViewModelInstanceAssetFontRuntime),
    AssetBlob(ViewModelInstanceAssetBlobRuntime),
    Artboard(ViewModelInstanceArtboardRuntime),
}
impl ViewModelInstanceRuntimeProperty {
    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        match self {
            Self::Number(value) => value.value_runtime(),
            Self::Boolean(value) => value.value_runtime(),
            Self::Color(value) => value.value_runtime(),
            Self::String(value) => value.value_runtime(),
            Self::Enum(value) => value.value_runtime(),
            Self::Trigger(value) => value.value_runtime(),
            Self::ListIndex(value) => value.value_runtime(),
            Self::List(value) => value.value_runtime(),
            Self::AssetImage(value) => value.value_runtime(),
            Self::AssetFont(value) => value.value_runtime(),
            Self::AssetBlob(value) => value.value_runtime(),
            Self::Artboard(value) => value.value_runtime(),
        }
    }
    pub fn data_type(&self) -> ViewModelRuntimeDataType {
        self.value_runtime().data_type()
    }
    pub fn name(&self) -> &str {
        self.value_runtime().name()
    }
}
#[derive(Clone)]
pub struct ViewModelInstanceNumberRuntime {
    native: native::viewmodel_instance_number_runtime::ViewModelInstanceNumberRuntime,
    value: ViewModelInstanceValueRuntime,
}
impl std::fmt::Debug for ViewModelInstanceNumberRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
impl ViewModelInstanceNumberRuntime {
    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.value
    }
    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.value.ptr_eq(&other.value)
    }
}
#[derive(Clone)]
pub struct ViewModelInstanceBooleanRuntime {
    native: native::viewmodel_instance_boolean_runtime::ViewModelInstanceBooleanRuntime,
    value: ViewModelInstanceValueRuntime,
}
impl std::fmt::Debug for ViewModelInstanceBooleanRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
impl ViewModelInstanceBooleanRuntime {
    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.value
    }
    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.value.ptr_eq(&other.value)
    }
}
#[derive(Clone)]
pub struct ViewModelInstanceColorRuntime {
    native: native::viewmodel_instance_color_runtime::ViewModelInstanceColorRuntime,
    value: ViewModelInstanceValueRuntime,
}
impl std::fmt::Debug for ViewModelInstanceColorRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
impl ViewModelInstanceColorRuntime {
    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.value
    }
    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.value.ptr_eq(&other.value)
    }
}
#[derive(Clone)]
pub struct ViewModelInstanceStringRuntime {
    native: native::viewmodel_instance_string_runtime::ViewModelInstanceStringRuntime,
    value: ViewModelInstanceValueRuntime,
}
impl std::fmt::Debug for ViewModelInstanceStringRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
impl ViewModelInstanceStringRuntime {
    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.value
    }
    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.value.ptr_eq(&other.value)
    }
}
#[derive(Clone)]
pub struct ViewModelInstanceEnumRuntime {
    native: native::viewmodel_instance_enum_runtime::ViewModelInstanceEnumRuntime,
    value: ViewModelInstanceValueRuntime,
}
impl std::fmt::Debug for ViewModelInstanceEnumRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
impl ViewModelInstanceEnumRuntime {
    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.value
    }
    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.value.ptr_eq(&other.value)
    }
}
#[derive(Clone)]
pub struct ViewModelInstanceTriggerRuntime {
    native: native::viewmodel_instance_trigger_runtime::ViewModelInstanceTriggerRuntime,
    value: ViewModelInstanceValueRuntime,
}
impl std::fmt::Debug for ViewModelInstanceTriggerRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
impl ViewModelInstanceTriggerRuntime {
    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.value
    }
    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.value.ptr_eq(&other.value)
    }
}
#[derive(Clone)]
pub struct ViewModelInstanceListIndexRuntime {
    native: native::viewmodel_instance_list_index_runtime::ViewModelInstanceListIndexRuntime,
    value: ViewModelInstanceValueRuntime,
}
impl std::fmt::Debug for ViewModelInstanceListIndexRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
impl ViewModelInstanceListIndexRuntime {
    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.value
    }
    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.value.ptr_eq(&other.value)
    }
}
#[derive(Clone)]
pub struct ViewModelInstanceListRuntime {
    native: native::viewmodel_instance_list_runtime::ViewModelInstanceListRuntime,
    value: ViewModelInstanceValueRuntime,
}
impl std::fmt::Debug for ViewModelInstanceListRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
impl ViewModelInstanceListRuntime {
    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.value
    }
    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.value.ptr_eq(&other.value)
    }
}
#[derive(Clone)]
pub struct ViewModelInstanceAssetImageRuntime {
    native: native::viewmodel_instance_asset_image_runtime::ViewModelInstanceAssetImageRuntime,
    value: ViewModelInstanceValueRuntime,
}
impl std::fmt::Debug for ViewModelInstanceAssetImageRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
impl ViewModelInstanceAssetImageRuntime {
    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.value
    }
    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.value.ptr_eq(&other.value)
    }
}
#[derive(Clone)]
pub struct ViewModelInstanceAssetFontRuntime {
    native: native::viewmodel_instance_asset_font_runtime::ViewModelInstanceAssetFontRuntime,
    value: ViewModelInstanceValueRuntime,
}
impl std::fmt::Debug for ViewModelInstanceAssetFontRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
impl ViewModelInstanceAssetFontRuntime {
    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.value
    }
    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.value.ptr_eq(&other.value)
    }
}
#[derive(Clone)]
pub struct ViewModelInstanceAssetBlobRuntime {
    native: native::viewmodel_instance_asset_blob_runtime::ViewModelInstanceAssetBlobRuntime,
    value: ViewModelInstanceValueRuntime,
}
impl std::fmt::Debug for ViewModelInstanceAssetBlobRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
impl ViewModelInstanceAssetBlobRuntime {
    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.value
    }
    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.value.ptr_eq(&other.value)
    }
}
#[derive(Clone)]
pub struct ViewModelInstanceArtboardRuntime {
    native: native::viewmodel_instance_artboard_runtime::ViewModelInstanceArtboardRuntime,
    value: ViewModelInstanceValueRuntime,
}
impl std::fmt::Debug for ViewModelInstanceArtboardRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
impl ViewModelInstanceArtboardRuntime {
    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.value
    }
    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.value.ptr_eq(&other.value)
    }
}
impl ViewModelInstanceNumberRuntime {
    pub fn value(&self) -> f32 {
        self.native.value() as f32
    }
    pub fn set_value(&self, value: f32) -> bool {
        let before = self.value();
        instance::mutate(|| self.native.set_value(value));
        before != self.value()
    }
}
impl ViewModelInstanceBooleanRuntime {
    pub fn value(&self) -> bool {
        self.native.value() as bool
    }
    pub fn set_value(&self, value: bool) -> bool {
        let before = self.value();
        instance::mutate(|| self.native.set_value(value));
        before != self.value()
    }
}
impl ViewModelInstanceColorRuntime {
    pub fn value(&self) -> u32 {
        self.native.value() as u32
    }
    pub fn set_value(&self, value: u32) -> bool {
        let before = self.value();
        instance::mutate(|| self.native.set_value(value as i32));
        before != self.value()
    }
}

impl ViewModelInstanceStringRuntime {
    pub fn value(&self) -> Arc<[u8]> {
        Arc::from(self.native.value().into_bytes())
    }
    pub fn value_string(&self) -> Option<String> {
        Some(self.native.value())
    }
    pub fn set_value(&self, value: impl Into<Arc<[u8]>>) -> bool {
        let bytes = value.into();
        let Ok(value) = std::str::from_utf8(&bytes) else {
            return false;
        };
        let before = self.native.value();
        instance::mutate(|| self.native.set_value(value));
        before != self.native.value()
    }
}
impl ViewModelInstanceEnumRuntime {
    pub fn value(&self) -> String {
        self.native.value()
    }
    pub fn set_value(&self, value: &str) -> bool {
        instance::mutate(|| self.native.set_value(value))
    }
    pub fn value_index(&self) -> u32 {
        self.native.value_index()
    }
    pub fn set_value_index(&self, value: u32) -> bool {
        instance::mutate(|| self.native.set_value_index(value))
    }
    pub fn values(&self) -> Vec<String> {
        self.native.values()
    }
    pub fn enum_type(&self) -> String {
        self.native.enum_type()
    }
}
impl ViewModelInstanceTriggerRuntime {
    pub fn trigger(&self) -> bool {
        instance::mutate(|| self.native.trigger());
        true
    }
}
impl ViewModelInstanceListIndexRuntime {
    pub fn value(&self) -> u32 {
        self.native.value()
    }
}
impl ViewModelInstanceListRuntime {
    pub fn instance_at(&self, index: isize) -> Option<ViewModelInstanceRuntime> {
        let index = i32::try_from(index).ok()?;
        ViewModelInstanceRuntime::from_native(
            self.value.file.clone(),
            self.native.instance_at(index)?.instance(),
        )
    }
    pub fn add_instance(&self, value: &ViewModelInstanceRuntime) -> bool {
        instance::mutate(|| self.native.add_instance(value.native.clone()));
        true
    }
    pub fn add_instance_at(&self, value: &ViewModelInstanceRuntime, index: isize) -> bool {
        let Ok(index) = i32::try_from(index) else {
            return false;
        };
        instance::mutate(|| self.native.add_instance_at(value.native.clone(), index))
    }
    pub fn remove_instance(&self, value: &ViewModelInstanceRuntime) -> bool {
        let before = self.size();
        instance::mutate(|| self.native.remove_instance(&value.native));
        before != self.size()
    }
    pub fn remove_instance_at(&self, index: isize) -> bool {
        let Ok(index) = i32::try_from(index) else {
            return false;
        };
        let before = self.size();
        instance::mutate(|| self.native.remove_instance_at(index));
        before != self.size()
    }
    pub fn swap(&self, first: usize, second: usize) -> bool {
        if first >= self.size() || second >= self.size() {
            return false;
        }
        instance::mutate(|| self.native.swap(first as u32, second as u32));
        true
    }
    pub fn remove_all_instances(&self) -> bool {
        let changed = self.size() != 0;
        instance::mutate(|| self.native.remove_all_instances());
        changed
    }
    pub fn size(&self) -> usize {
        self.native.size()
    }
}
#[derive(Clone)]
pub struct RuntimeViewModelImage {
    bytes: Option<Arc<[u8]>>,
    image: Option<Rc<dyn RenderImage>>,
}
impl std::fmt::Debug for RuntimeViewModelImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeViewModelImage")
            .field("encoded_bytes", &self.bytes.as_ref().map(|b| b.len()))
            .finish()
    }
}
impl RuntimeViewModelImage {
    pub fn new(bytes: impl Into<Arc<[u8]>>) -> Self {
        Self {
            bytes: Some(bytes.into()),
            image: None,
        }
    }
    pub fn bytes(&self) -> &[u8] {
        self.bytes.as_deref().unwrap_or_default()
    }
    pub fn from_render_image(image: Rc<dyn RenderImage>) -> Self {
        Self {
            bytes: None,
            image: Some(image),
        }
    }
    pub fn render_image(&self) -> Option<Rc<dyn RenderImage>> {
        self.image.clone()
    }
    pub fn ptr_eq(&self, other: &Self) -> bool {
        match (&self.image, &other.image, &self.bytes, &other.bytes) {
            (Some(a), Some(b), _, _) => Rc::ptr_eq(a, b),
            (_, _, Some(a), Some(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}
impl ViewModelInstanceAssetImageRuntime {
    pub fn set_value(&self, value: Option<RuntimeViewModelImage>) -> bool {
        let image = match value {
            None => None,
            Some(value) => match value.render_image() {
                Some(image) => Some(image),
                None => {
                    let factory = self.value.file.with_file(|file| file.factory());
                    let Some(image) = factory
                        .with_factory_mut(|factory| factory.decode_image(value.bytes()).ok())
                        .map(Rc::from)
                    else {
                        return false;
                    };
                    Some(image)
                }
            },
        };
        instance::mutate(|| self.native.set_value(image));
        true
    }
}
impl ViewModelInstanceAssetFontRuntime {
    pub fn set_value(&self, value: Option<Arc<[u8]>>) -> bool {
        let font = match value {
            None => None,
            Some(bytes) => {
                let Some(font) =
                    crate::mechanical_port::source::text::font_hb::HbFont::decode(&bytes)
                else {
                    return false;
                };
                Some(font)
            }
        };
        instance::mutate(|| self.native.set_value(font));
        true
    }
    pub fn set_font(&self, font: Option<crate::RawTextFont>) -> bool {
        instance::mutate(|| self.native.set_value(font.map(|font| font.native_handle())));
        true
    }
}
impl ViewModelInstanceAssetBlobRuntime {
    pub fn set_value(&self, value: Option<Arc<RuntimeBlobAsset>>) -> bool {
        instance::mutate(|| self.native.set_value(value));
        true
    }
}
#[derive(Clone)]
pub struct RuntimeBindableArtboard {
    native: RuntimeBindableArtboardHandle,
    file: RuntimeFileHandle,
    name: String,
}
impl std::fmt::Debug for RuntimeBindableArtboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeBindableArtboard")
            .field("name", &self.name)
            .finish()
    }
}
impl RuntimeBindableArtboard {
    pub fn from_native(file: RuntimeFileHandle, native: RuntimeBindableArtboardHandle) -> Self {
        let name = native.with_artboard(|artboard| artboard.name().to_owned());
        Self { native, file, name }
    }
    pub fn native_handle(&self) -> RuntimeBindableArtboardHandle {
        self.native.clone()
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.native.ptr_eq(&other.native)
    }
}
impl PartialEq for RuntimeBindableArtboard {
    fn eq(&self, other: &Self) -> bool {
        self.ptr_eq(other)
    }
}
impl Eq for RuntimeBindableArtboard {}
impl ViewModelInstanceArtboardRuntime {
    pub fn set_value(&self, value: Option<RuntimeBindableArtboard>) -> bool {
        instance::mutate(|| {
            self.native
                .set_value(value.map(|value| value.native_handle()))
        });
        true
    }
    pub fn set_view_model_instance(&self, value: Option<ViewModelInstanceRuntime>) {
        instance::mutate(|| {
            self.native
                .set_view_model_instance(value.map(|value| value.native_handle()))
        });
    }
    pub fn artboard_name(&self) -> String {
        self.native.artboard_name()
    }
}
#[derive(Clone)]
pub struct ViewModelRuntime {
    native: NativeModel,
    file: RuntimeFileHandle,
    index: usize,
}
impl std::fmt::Debug for ViewModelRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ViewModelRuntime")
            .field("model", &self.native.view_model_handle())
            .finish()
    }
}
impl ViewModelRuntime {
    pub fn from_native(file: RuntimeFileHandle, index: usize) -> Option<Self> {
        let native = file.with_file(|file| file.view_model_by_index(index))?;
        Some(Self {
            native,
            file,
            index,
        })
    }
    pub fn new(file: RuntimeFileHandle, index: usize) -> Option<Self> {
        Self::from_native(file, index)
    }
    pub fn named(file: RuntimeFileHandle, name: &str) -> Option<Self> {
        let index = file.with_file(|file| {
            (0..file.view_model_count()).find(|index| {
                file.view_model(*index).and_then(|model| {
                    model.with(|model| model.as_view_model().unwrap().base.name() == name)
                }) == Some(true)
            })
        })?;
        Self::from_native(file, index)
    }
    pub fn native_handle(&self) -> CoreHandle {
        self.native.view_model_handle()
    }
    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.native_handle() == other.native_handle()
    }
    pub fn view_model_index(&self) -> usize {
        self.index
    }
    pub fn file(&self) -> RuntimeFileHandle {
        self.file.clone()
    }
    pub fn instance_count(&self) -> usize {
        self.native.instance_count()
    }
    pub fn property_count(&self) -> usize {
        self.native.property_count()
    }
    pub fn name(&self) -> String {
        self.native.name()
    }
    pub fn properties(&self) -> Vec<ViewModelRuntimeProperty> {
        self.native.properties()
    }
    pub fn instance_names(&self) -> Vec<String> {
        self.native.instance_names()
    }
    pub fn create_instance_from_index(&self, index: usize) -> Option<ViewModelInstanceRuntime> {
        ViewModelInstanceRuntime::from_native(
            self.file.clone(),
            self.native.create_instance_from_index(index)?.instance(),
        )
    }
    pub fn create_instance_from_name(&self, name: &str) -> Option<ViewModelInstanceRuntime> {
        ViewModelInstanceRuntime::from_native(
            self.file.clone(),
            self.native.create_instance_from_name(name)?.instance(),
        )
    }
    pub fn create_default_instance(&self) -> Option<ViewModelInstanceRuntime> {
        ViewModelInstanceRuntime::from_native(
            self.file.clone(),
            self.native.create_default_instance().instance(),
        )
    }
    pub fn create_instance(&self) -> Option<ViewModelInstanceRuntime> {
        ViewModelInstanceRuntime::from_native(
            self.file.clone(),
            self.native.create_instance().instance(),
        )
    }
}
