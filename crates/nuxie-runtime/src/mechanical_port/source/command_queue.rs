use std::{
    collections::HashMap,
    sync::{Arc, Condvar, Mutex, MutexGuard, Weak},
};

use crate::mechanical_port::source::{
    assets::blob_asset::BlobAsset,
    audio::audio_source::AudioSource,
    command_server::CommandServer,
    data_bind::data_values::data_type::DataType,
    factory::RuntimeFactoryHandle,
    layout::{Alignment, Fit},
    math::vec2d::Vec2D,
    object_stream::{ObjectStream, PodStream},
    refcnt::{Rcp, RefCnt, RefCounted},
    renderer::RenderImageRef,
    semantic::semantic_snapshot::{SemanticActionType, SemanticsDiff},
    text_engine::Font,
    viewmodel::runtime::viewmodel_instance_runtime::PropertyData,
};

#[cfg(not(any(target_pointer_width = "32", target_pointer_width = "64")))]
compile_error!("environment is not 32 or 64 bit");

// Locks immediately, then notifies exactly before unlocking at scope exit.
struct AutoLockAndNotify<'a> {
    guard: Option<MutexGuard<'a, ()>>,
    condition_variable: &'a Condvar,
}

impl<'a> AutoLockAndNotify<'a> {
    fn new(mutex: &'a Mutex<()>, condition_variable: &'a Condvar) -> Self {
        Self {
            guard: Some(mutex.lock().unwrap()),
            condition_variable,
        }
    }
}

impl Drop for AutoLockAndNotify<'_> {
    fn drop(&mut self) {
        self.condition_variable.notify_one();
        self.guard.take();
    }
}

trait CommandHandle: Copy + Default {
    fn is_null(self) -> bool;
}

macro_rules! define_handle {
    ($name:ident, $opaque:ident) => {
        #[cfg(target_pointer_width = "64")]
        pub enum $opaque {}

        #[cfg(target_pointer_width = "64")]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        #[repr(transparent)]
        pub struct $name(*mut $opaque);

        #[cfg(target_pointer_width = "32")]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        #[repr(transparent)]
        pub struct $name(u64);

        impl $name {
            #[cfg(target_pointer_width = "64")]
            pub const NULL: Self = Self(std::ptr::null_mut());
            #[cfg(target_pointer_width = "32")]
            pub const NULL: Self = Self(0);

            fn from_index(index: u64) -> Self {
                #[cfg(target_pointer_width = "64")]
                {
                    Self(index as usize as *mut $opaque)
                }
                #[cfg(target_pointer_width = "32")]
                {
                    Self(index)
                }
            }

            pub fn is_null(self) -> bool {
                self == Self::NULL
            }

            fn index(self) -> u64 {
                #[cfg(target_pointer_width = "64")]
                {
                    self.0 as usize as u64
                }
                #[cfg(target_pointer_width = "32")]
                {
                    self.0
                }
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::NULL
            }
        }

        impl CommandHandle for $name {
            fn is_null(self) -> bool {
                self.is_null()
            }
        }
    };
}

define_handle!(FontHandle, FontHandlePlaceholder);
define_handle!(FileHandle, FileHandlePlaceholder);
define_handle!(ArtboardHandle, ArtboardHandlePlaceholder);
define_handle!(AudioSourceHandle, AudioSourceHandlePlaceholder);
define_handle!(RenderImageHandle, RenderImageHandlePlaceholder);
define_handle!(BlobAssetHandle, BlobAssetHandlePlaceholder);
define_handle!(StateMachineHandle, StateMachineHandlePlaceholder);
define_handle!(ViewModelInstanceHandle, ViewModelInstanceHandlePlaceholder);
define_handle!(DrawKey, DrawKeyPlaceholder);

pub type CommandServerCallback = Box<dyn FnOnce(&mut CommandServer) + Send>;
pub type CommandServerDrawCallback = Box<dyn FnMut(DrawKey, &mut CommandServer) + Send>;

pub type ScriptingContextFactory = Box<
    dyn FnOnce(
            RuntimeFactoryHandle,
        ) -> Option<
            Box<dyn crate::mechanical_port::source::lua::rive_lua_libs::ScriptingContext>,
        > + Send,
>;

#[derive(Clone, Default)]
pub struct ViewModelEnum {
    pub name: String,
    pub enumerants: Vec<String>,
}

/// Strong caller-owned lifetime for a command-queue listener.
///
/// Pinned C++ stores a non-owning pointer and unregisters it when the listener
/// dies. Rust stores only the corresponding weak handle in the queue, so a
/// dropped listener can never leave a dangling callback target.
pub struct ListenerHandle<T: ?Sized>(Arc<Mutex<Box<T>>>);

impl<T: ?Sized> Clone for ListenerHandle<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: ?Sized> ListenerHandle<T> {
    pub fn new(listener: Box<T>) -> Self {
        Self(Arc::new(Mutex::new(listener)))
    }

    fn downgrade(&self) -> WeakListenerHandle<T> {
        WeakListenerHandle(Arc::downgrade(&self.0))
    }

    fn with_mut<R>(&self, callback: impl FnOnce(&mut T) -> R) -> R {
        let mut listener = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        callback(listener.as_mut())
    }

    fn borrow_mut(&self) -> MutexGuard<'_, Box<T>> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

struct WeakListenerHandle<T: ?Sized>(Weak<Mutex<Box<T>>>);

impl<T: ?Sized> Clone for WeakListenerHandle<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: ?Sized> WeakListenerHandle<T> {
    fn upgrade(&self) -> Option<ListenerHandle<T>> {
        self.0.upgrade().map(ListenerHandle)
    }
}

pub type FileListenerHandle = ListenerHandle<dyn FileListener>;
pub type RenderImageListenerHandle = ListenerHandle<dyn RenderImageListener>;
pub type AudioSourceListenerHandle = ListenerHandle<dyn AudioSourceListener>;
pub type FontListenerHandle = ListenerHandle<dyn FontListener>;
pub type BlobAssetListenerHandle = ListenerHandle<dyn BlobAssetListener>;
pub type ArtboardListenerHandle = ListenerHandle<dyn ArtboardListener>;
pub type ViewModelInstanceListenerHandle = ListenerHandle<dyn ViewModelInstanceListener>;
pub type StateMachineListenerHandle = ListenerHandle<dyn StateMachineListener>;

pub struct ListenerBase<H: Copy + Default> {
    pub handle: H,
}

impl<H: Copy + Default> ListenerBase<H> {
    pub fn new() -> Self {
        Self {
            handle: H::default(),
        }
    }
}

impl<H: Copy + Default> Default for ListenerBase<H> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct ViewModelPropertyData {
    pub data_type: DataType,
    pub name: String,
    pub meta_data: String,
}

impl Default for ViewModelPropertyData {
    fn default() -> Self {
        Self {
            data_type: DataType::None,
            name: String::new(),
            meta_data: String::new(),
        }
    }
}

#[derive(Clone, Default)]
pub struct FileAssetData {
    pub name: String,
    pub asset_id: u32,
    pub cdn_uuid: String,
    pub cdn_base_url: String,
    pub file_extension: String,
    pub asset_type: u16,
}

pub trait FileListener: Send {
    fn listener_base(&mut self) -> &mut ListenerBase<FileHandle>;
    fn on_file_error(&mut self, _handle: FileHandle, _request_id: u64, _error: String) {}
    fn on_file_deleted(&mut self, _handle: FileHandle, _request_id: u64) {}
    fn on_file_loaded(&mut self, _handle: FileHandle, _request_id: u64) {}
    fn on_artboard_instantiated(
        &mut self,
        _handle: FileHandle,
        _request_id: u64,
        _artboard: ArtboardHandle,
    ) {
    }
    fn on_view_model_instance_instantiated(
        &mut self,
        _handle: FileHandle,
        _request_id: u64,
        _instance: ViewModelInstanceHandle,
    ) {
    }
    fn on_artboards_listed(&mut self, _handle: FileHandle, _request_id: u64, _names: Vec<String>) {}
    fn on_view_models_listed(
        &mut self,
        _handle: FileHandle,
        _request_id: u64,
        _names: Vec<String>,
    ) {
    }
    fn on_global_view_model_names_listed(
        &mut self,
        _handle: FileHandle,
        _request_id: u64,
        _names: Vec<String>,
    ) {
    }
    fn on_view_model_instance_names_listed(
        &mut self,
        _handle: FileHandle,
        _request_id: u64,
        _view_model_name: String,
        _instance_names: Vec<String>,
    ) {
    }
    fn on_view_model_properties_listed(
        &mut self,
        _handle: FileHandle,
        _request_id: u64,
        _view_model_name: String,
        _properties: Vec<ViewModelPropertyData>,
    ) {
    }
    fn on_view_model_enums_listed(
        &mut self,
        _handle: FileHandle,
        _request_id: u64,
        _enums: Vec<ViewModelEnum>,
    ) {
    }
    fn on_file_assets_listed(
        &mut self,
        _handle: FileHandle,
        _request_id: u64,
        _assets: Vec<FileAssetData>,
    ) {
    }
}

pub trait RenderImageListener: Send {
    fn listener_base(&mut self) -> &mut ListenerBase<RenderImageHandle>;
    fn on_render_image_decoded(&mut self, _handle: RenderImageHandle, _request_id: u64) {}
    fn on_render_image_error(
        &mut self,
        _handle: RenderImageHandle,
        _request_id: u64,
        _error: String,
    ) {
    }
    fn on_render_image_deleted(&mut self, _handle: RenderImageHandle, _request_id: u64) {}
}

pub trait AudioSourceListener: Send {
    fn listener_base(&mut self) -> &mut ListenerBase<AudioSourceHandle>;
    fn on_audio_source_decoded(&mut self, _handle: AudioSourceHandle, _request_id: u64) {}
    fn on_audio_source_error(
        &mut self,
        _handle: AudioSourceHandle,
        _request_id: u64,
        _error: String,
    ) {
    }
    fn on_audio_source_deleted(&mut self, _handle: AudioSourceHandle, _request_id: u64) {}
}

pub trait FontListener: Send {
    fn listener_base(&mut self) -> &mut ListenerBase<FontHandle>;
    fn on_font_decoded(&mut self, _handle: FontHandle, _request_id: u64) {}
    fn on_font_error(&mut self, _handle: FontHandle, _request_id: u64, _error: String) {}
    fn on_font_deleted(&mut self, _handle: FontHandle, _request_id: u64) {}
}

pub trait BlobAssetListener: Send {
    fn listener_base(&mut self) -> &mut ListenerBase<BlobAssetHandle>;
    fn on_blob_asset_decoded(&mut self, _handle: BlobAssetHandle, _request_id: u64) {}
    fn on_blob_asset_error(&mut self, _handle: BlobAssetHandle, _request_id: u64, _error: String) {}
    fn on_blob_asset_deleted(&mut self, _handle: BlobAssetHandle, _request_id: u64) {}
}

pub trait ArtboardListener: Send {
    fn listener_base(&mut self) -> &mut ListenerBase<ArtboardHandle>;
    fn on_artboard_error(&mut self, _handle: ArtboardHandle, _request_id: u64, _error: String) {}
    fn on_default_view_model_info_received(
        &mut self,
        _handle: ArtboardHandle,
        _request_id: u64,
        _view_model_name: String,
        _instance_name: String,
    ) {
    }
    fn on_artboard_deleted(&mut self, _handle: ArtboardHandle, _request_id: u64) {}
    fn on_state_machine_instantiated(
        &mut self,
        _handle: ArtboardHandle,
        _request_id: u64,
        _state_machine: StateMachineHandle,
    ) {
    }
    fn on_state_machines_listed(
        &mut self,
        _handle: ArtboardHandle,
        _request_id: u64,
        _names: Vec<String>,
    ) {
    }
    fn on_artboard_volume_received(
        &mut self,
        _handle: ArtboardHandle,
        _request_id: u64,
        _volume: f32,
    ) {
    }
    fn on_artboard_size_received(
        &mut self,
        _handle: ArtboardHandle,
        _request_id: u64,
        _width: f32,
        _height: f32,
    ) {
    }
}

#[derive(Clone)]
pub enum ViewModelInstanceValue {
    None,
    Bool(bool),
    Number(f32),
    Color(u32),
    String(String),
}

#[derive(Clone)]
pub struct ViewModelInstanceData {
    pub meta_data: PropertyData,
    pub value: ViewModelInstanceValue,
}

pub trait ViewModelInstanceListener: Send {
    fn listener_base(&mut self) -> &mut ListenerBase<ViewModelInstanceHandle>;
    fn on_view_model_instance_error(
        &mut self,
        _handle: ViewModelInstanceHandle,
        _request_id: u64,
        _error: String,
    ) {
    }
    fn on_view_model_instance_view_model_name_received(
        &mut self,
        _handle: ViewModelInstanceHandle,
        _request_id: u64,
        _name: String,
    ) {
    }
    fn on_view_model_instance_name_received(
        &mut self,
        _handle: ViewModelInstanceHandle,
        _request_id: u64,
        _name: String,
    ) {
    }
    fn on_view_model_deleted(&mut self, _handle: ViewModelInstanceHandle, _request_id: u64) {}
    fn on_view_model_data_received(
        &mut self,
        _handle: ViewModelInstanceHandle,
        _request_id: u64,
        _data: ViewModelInstanceData,
    ) {
    }
    fn on_view_model_list_size_received(
        &mut self,
        _handle: ViewModelInstanceHandle,
        _request_id: u64,
        _path: String,
        _size: usize,
    ) {
    }
    fn on_view_model_list_cleared(
        &mut self,
        _handle: ViewModelInstanceHandle,
        _request_id: u64,
        _path: String,
    ) {
    }
}

pub trait StateMachineListener: Send {
    fn listener_base(&mut self) -> &mut ListenerBase<StateMachineHandle>;
    fn on_state_machine_error(
        &mut self,
        _handle: StateMachineHandle,
        _request_id: u64,
        _error: String,
    ) {
    }
    fn on_state_machine_deleted(&mut self, _handle: StateMachineHandle, _request_id: u64) {}
    fn on_state_machine_settled(&mut self, _handle: StateMachineHandle, _request_id: u64) {}
    fn on_semantics_diff_received(
        &mut self,
        _handle: StateMachineHandle,
        _request_id: u64,
        _diff: SemanticsDiff,
    ) {
    }
}

trait ListenerRegistration<H: Copy + Default> {
    fn registration_base(&mut self) -> &mut ListenerBase<H>;
}

macro_rules! impl_listener_registration {
    ($listener:ident, $handle:ident) => {
        impl<T: $listener + ?Sized> ListenerRegistration<$handle> for T {
            fn registration_base(&mut self) -> &mut ListenerBase<$handle> {
                self.listener_base()
            }
        }
    };
}

impl_listener_registration!(FileListener, FileHandle);
impl_listener_registration!(RenderImageListener, RenderImageHandle);
impl_listener_registration!(AudioSourceListener, AudioSourceHandle);
impl_listener_registration!(FontListener, FontHandle);
impl_listener_registration!(BlobAssetListener, BlobAssetHandle);
impl_listener_registration!(ArtboardListener, ArtboardHandle);
impl_listener_registration!(ViewModelInstanceListener, ViewModelInstanceHandle);
impl_listener_registration!(StateMachineListener, StateMachineHandle);

#[derive(Clone, Copy)]
#[repr(u8)]
enum Command {
    LoadFile,
    DeleteFile,
    DecodeImage,
    ExternalImage,
    DecodeAudio,
    ExternalAudio,
    DecodeFont,
    ExternalFont,
    DecodeBlob,
    ExternalBlob,
    DeleteImage,
    DeleteAudio,
    DeleteFont,
    DeleteBlob,
    AddImageFileAsset,
    AddAudioFileAsset,
    AddFontFileAsset,
    RemoveImageFileAsset,
    RemoveAudioFileAsset,
    RemoveFontFileAsset,
    InstantiateArtboard,
    DeleteArtboard,
    SetArtboardSize,
    ResetArtboardSize,
    InstantiateViewModel,
    RefNestedViewModel,
    RefListViewModel,
    InstantiateBlankViewModel,
    InstantiateViewModelForArtboard,
    InstantiateBlankViewModelForArtboard,
    SetViewModelInstanceValue,
    AddViewModelListValue,
    RemoveViewModelListValue,
    SwapViewModelListValue,
    SubscribeViewModelProperty,
    UnsubscribeViewModelProperty,
    DeleteViewModel,
    InstantiateStateMachine,
    DeleteStateMachine,
    AdvanceStateMachine,
    EnableSemantics,
    DrainSemanticsDiff,
    FireSemanticAction,
    RequestSemanticFocus,
    ClearSemanticFocus,
    BindViewModelInstance,
    SetViewModelInstance,
    SetGlobalViewModelInstance,
    GetGlobalViewModelInstance,
    Bind,
    RunOnce,
    Draw,
    CancelDraw,
    PointerMove,
    PointerDown,
    PointerUp,
    PointerExit,
    Disconnect,
    CommandLoopBreak,
    ListViewModelEnums,
    ListArtboards,
    ListStateMachines,
    GetDefaultViewModel,
    ListViewModels,
    ListGlobalViewModelNames,
    ListViewModelInstanceNames,
    ListViewModelProperties,
    ListViewModelPropertyValue,
    GetViewModelInstanceViewModelName,
    GetViewModelInstanceName,
    GetViewModelListSize,
    ClearViewModelList,
    ListFileAssets,
    SetArtboardVolume,
    GetArtboardVolume,
    GetArtboardSize,
}

#[derive(Clone, Copy)]
#[repr(u8)]
enum Message {
    MessageLoopBreak,
    ViewModelEnumsListed,
    ArtboardsListed,
    StateMachinesListed,
    DefaultViewModelReceived,
    ViewModelInstanceViewModelNameReceived,
    ViewModelInstanceNameReceived,
    ViewModelsListed,
    GlobalViewModelNamesListed,
    ViewModelInstanceNamesListed,
    ViewModelPropertiesListed,
    ViewModelPropertyValueReceived,
    ViewModelListSizeReceived,
    ViewModelListCleared,
    FileLoaded,
    FileDeleted,
    ArtboardInstantiated,
    StateMachineInstantiated,
    ViewModelInstanceInstantiated,
    ImageDecoded,
    ImageDeleted,
    AudioDecoded,
    AudioDeleted,
    FontDecoded,
    FontDeleted,
    BlobDecoded,
    BlobDeleted,
    ArtboardDeleted,
    ViewModelDeleted,
    StateMachineDeleted,
    StateMachineSettled,
    SemanticsDiffReceived,
    FileAssetsListed,
    ArtboardSizeReceived,
    FileError,
    ArtboardError,
    ViewModelError,
    ImageError,
    AudioError,
    FontError,
    BlobError,
    StateMachineError,
    ArtboardVolumeReceived,
}

impl Default for Message {
    fn default() -> Self {
        Self::MessageLoopBreak
    }
}

#[derive(Clone, Copy)]
pub struct PointerEvent {
    pub fit: Fit,
    pub alignment: Alignment,
    pub screen_bounds: Vec2D,
    pub position: Vec2D,
    pub scale_factor: f32,
    pub pointer_id: i32,
}

impl Default for PointerEvent {
    fn default() -> Self {
        Self {
            fit: Fit::None,
            alignment: Alignment::default(),
            screen_bounds: Vec2D::default(),
            position: Vec2D::default(),
            scale_factor: 1.0,
            pointer_id: 0,
        }
    }
}

pub struct CommandQueue {
    ref_count: RefCnt,
    current_file_handle_idx: u64,
    current_list_handle_idx: u64,
    current_font_handle_idx: u64,
    current_artboard_handle_idx: u64,
    current_view_model_handle_idx: u64,
    current_render_image_handle_idx: u64,
    current_blob_asset_handle_idx: u64,
    current_audio_source_handle_idx: u64,
    current_state_machine_handle_idx: u64,
    current_draw_key_idx: u64,
    command_mutex: Mutex<()>,
    command_condition_variable: Condvar,
    command_stream: PodStream,
    external_images: ObjectStream<RenderImageRef>,
    external_audio_sources: ObjectStream<Rcp<AudioSource>>,
    external_fonts: ObjectStream<Rcp<Font>>,
    external_blobs: ObjectStream<Rcp<BlobAsset>>,
    byte_vectors: ObjectStream<Vec<u8>>,
    scripting_context_factories: ObjectStream<Option<ScriptingContextFactory>>,
    pointer_events: ObjectStream<PointerEvent>,
    names: ObjectStream<String>,
    callbacks: ObjectStream<CommandServerCallback>,
    draw_callbacks: ObjectStream<CommandServerDrawCallback>,
    message_mutex: Mutex<()>,
    message_stream: PodStream,
    message_names: ObjectStream<String>,
    message_semantics_diffs: ObjectStream<SemanticsDiff>,
    global_file_listener: Option<WeakListenerHandle<dyn FileListener>>,
    global_image_listener: Option<WeakListenerHandle<dyn RenderImageListener>>,
    global_audio_listener: Option<WeakListenerHandle<dyn AudioSourceListener>>,
    global_font_listener: Option<WeakListenerHandle<dyn FontListener>>,
    global_blob_listener: Option<WeakListenerHandle<dyn BlobAssetListener>>,
    global_artboard_listener: Option<WeakListenerHandle<dyn ArtboardListener>>,
    global_view_model_listener: Option<WeakListenerHandle<dyn ViewModelInstanceListener>>,
    global_state_machine_listener: Option<WeakListenerHandle<dyn StateMachineListener>>,
    file_listeners: HashMap<FileHandle, WeakListenerHandle<dyn FileListener>>,
    image_listeners: HashMap<RenderImageHandle, WeakListenerHandle<dyn RenderImageListener>>,
    audio_listeners: HashMap<AudioSourceHandle, WeakListenerHandle<dyn AudioSourceListener>>,
    font_listeners: HashMap<FontHandle, WeakListenerHandle<dyn FontListener>>,
    blob_listeners: HashMap<BlobAssetHandle, WeakListenerHandle<dyn BlobAssetListener>>,
    artboard_listeners: HashMap<ArtboardHandle, WeakListenerHandle<dyn ArtboardListener>>,
    view_model_listeners:
        HashMap<ViewModelInstanceHandle, WeakListenerHandle<dyn ViewModelInstanceListener>>,
    state_machine_listeners:
        HashMap<StateMachineHandle, WeakListenerHandle<dyn StateMachineListener>>,
}

unsafe impl RefCounted for CommandQueue {
    fn ref_count(&self) -> &RefCnt {
        &self.ref_count
    }
}

impl Default for CommandQueue {
    fn default() -> Self {
        Self {
            ref_count: RefCnt::new(),
            current_file_handle_idx: 0,
            current_list_handle_idx: 0,
            current_font_handle_idx: 0,
            current_artboard_handle_idx: 0,
            current_view_model_handle_idx: 0,
            current_render_image_handle_idx: 0,
            current_blob_asset_handle_idx: 0,
            current_audio_source_handle_idx: 0,
            current_state_machine_handle_idx: 0,
            current_draw_key_idx: 0,
            command_mutex: Mutex::new(()),
            command_condition_variable: Condvar::new(),
            command_stream: PodStream::default(),
            external_images: ObjectStream::default(),
            external_audio_sources: ObjectStream::default(),
            external_fonts: ObjectStream::default(),
            external_blobs: ObjectStream::default(),
            byte_vectors: ObjectStream::default(),
            scripting_context_factories: ObjectStream::default(),
            pointer_events: ObjectStream::default(),
            names: ObjectStream::default(),
            callbacks: ObjectStream::default(),
            draw_callbacks: ObjectStream::default(),
            message_mutex: Mutex::new(()),
            message_stream: PodStream::default(),
            message_names: ObjectStream::default(),
            message_semantics_diffs: ObjectStream::default(),
            global_file_listener: None,
            global_image_listener: None,
            global_audio_listener: None,
            global_font_listener: None,
            global_blob_listener: None,
            global_artboard_listener: None,
            global_view_model_listener: None,
            global_state_machine_listener: None,
            file_listeners: HashMap::new(),
            image_listeners: HashMap::new(),
            audio_listeners: HashMap::new(),
            font_listeners: HashMap::new(),
            blob_listeners: HashMap::new(),
            artboard_listeners: HashMap::new(),
            view_model_listeners: HashMap::new(),
            state_machine_listeners: HashMap::new(),
        }
    }
}

impl CommandQueue {
    pub fn new() -> Self {
        Self::default()
    }

    fn register_listener<T, H>(
        &self,
        listener: &ListenerHandle<T>,
        handle: H,
    ) -> WeakListenerHandle<T>
    where
        T: ListenerRegistration<H> + ?Sized,
        H: CommandHandle,
    {
        listener.with_mut(|listener| {
            let base = listener.registration_base();
            assert!(base.handle.is_null());
            base.handle = handle;
        });
        listener.downgrade()
    }

    fn notify_command(&self) {
        self.command_condition_variable.notify_one();
    }

    fn write_command(&mut self, command: Command) {
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream.write(command);
        self.notify_command();
    }

    fn next_file_handle(&mut self) -> FileHandle {
        self.current_file_handle_idx += 1;
        FileHandle::from_index(self.current_file_handle_idx)
    }
    fn next_artboard_handle(&mut self) -> ArtboardHandle {
        self.current_artboard_handle_idx += 1;
        ArtboardHandle::from_index(self.current_artboard_handle_idx)
    }
    fn next_view_model_handle(&mut self) -> ViewModelInstanceHandle {
        self.current_view_model_handle_idx += 1;
        ViewModelInstanceHandle::from_index(self.current_view_model_handle_idx)
    }
    fn next_state_machine_handle(&mut self) -> StateMachineHandle {
        self.current_state_machine_handle_idx += 1;
        StateMachineHandle::from_index(self.current_state_machine_handle_idx)
    }

    pub fn load_file(
        &mut self,
        riv_bytes: Vec<u8>,
        listener: Option<&FileListenerHandle>,
        request_id: u64,
        scripting_context_factory: Option<ScriptingContextFactory>,
    ) -> FileHandle {
        let handle = self.next_file_handle();
        if let Some(listener) = listener {
            let listener = self.register_listener(listener, handle);
            assert!(self.file_listeners.insert(handle, listener).is_none());
        }
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::LoadFile)
            .write(handle)
            .write(request_id);
        self.byte_vectors.write(riv_bytes);
        self.scripting_context_factories
            .write(scripting_context_factory);
        self.notify_command();
        handle
    }

    pub fn delete_file(&mut self, handle: FileHandle, request_id: u64) {
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::DeleteFile)
            .write(handle)
            .write(request_id);
        self.notify_command();
    }

    fn global_asset<H: Copy>(
        &mut self,
        command: Command,
        name: String,
        handle: H,
        request_id: u64,
    ) {
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(command)
            .write(handle)
            .write(request_id);
        self.names.write(name);
        self.notify_command();
    }

    fn remove_global_asset(&mut self, command: Command, name: String, request_id: u64) {
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream.write(command).write(request_id);
        self.names.write(name);
        self.notify_command();
    }

    pub fn add_global_image_asset(&mut self, name: String, handle: RenderImageHandle, id: u64) {
        self.global_asset(Command::AddImageFileAsset, name, handle, id);
    }
    pub fn add_global_font_asset(&mut self, name: String, handle: FontHandle, id: u64) {
        self.global_asset(Command::AddFontFileAsset, name, handle, id);
    }
    pub fn add_global_audio_asset(&mut self, name: String, handle: AudioSourceHandle, id: u64) {
        self.global_asset(Command::AddAudioFileAsset, name, handle, id);
    }
    pub fn remove_global_image_asset(&mut self, name: String, id: u64) {
        self.remove_global_asset(Command::RemoveImageFileAsset, name, id);
    }
    pub fn remove_global_font_asset(&mut self, name: String, id: u64) {
        self.remove_global_asset(Command::RemoveFontFileAsset, name, id);
    }
    pub fn remove_global_audio_asset(&mut self, name: String, id: u64) {
        self.remove_global_asset(Command::RemoveAudioFileAsset, name, id);
    }

    pub fn instantiate_artboard_named(
        &mut self,
        file: FileHandle,
        name: String,
        listener: Option<&ArtboardListenerHandle>,
        request_id: u64,
    ) -> ArtboardHandle {
        let handle = self.next_artboard_handle();
        if let Some(listener) = listener {
            let listener = self.register_listener(listener, handle);
            assert!(self.artboard_listeners.insert(handle, listener).is_none());
        }
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::InstantiateArtboard)
            .write(handle)
            .write(file)
            .write(request_id);
        self.names.write(name);
        self.notify_command();
        handle
    }

    pub fn instantiate_default_artboard(
        &mut self,
        file: FileHandle,
        listener: Option<&ArtboardListenerHandle>,
        request_id: u64,
    ) -> ArtboardHandle {
        self.instantiate_artboard_named(file, String::new(), listener, request_id)
    }

    pub fn set_artboard_size(
        &mut self,
        handle: ArtboardHandle,
        width: f32,
        height: f32,
        scale: f32,
        request_id: u64,
    ) {
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::SetArtboardSize)
            .write(handle)
            .write(width / scale)
            .write(height / scale)
            .write(request_id);
        self.notify_command();
    }
    pub fn reset_artboard_size(&mut self, handle: ArtboardHandle, request_id: u64) {
        self.record_handle(Command::ResetArtboardSize, handle, request_id);
    }
    pub fn delete_artboard(&mut self, handle: ArtboardHandle, request_id: u64) {
        self.record_handle(Command::DeleteArtboard, handle, request_id);
    }

    fn attach_view_model_listener(
        &mut self,
        handle: ViewModelInstanceHandle,
        listener: Option<&ViewModelInstanceListenerHandle>,
    ) {
        if let Some(listener) = listener {
            let listener = self.register_listener(listener, handle);
            assert!(self.view_model_listeners.insert(handle, listener).is_none());
        }
    }

    pub fn instantiate_blank_view_model_instance_for_artboard(
        &mut self,
        file: FileHandle,
        artboard: ArtboardHandle,
        listener: Option<&ViewModelInstanceListenerHandle>,
        request_id: u64,
    ) -> ViewModelInstanceHandle {
        let handle = self.next_view_model_handle();
        self.attach_view_model_listener(handle, listener);
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::InstantiateBlankViewModelForArtboard)
            .write(file)
            .write(artboard)
            .write(handle)
            .write(request_id);
        self.notify_command();
        handle
    }
    pub fn instantiate_blank_view_model_instance_named(
        &mut self,
        file: FileHandle,
        view_model_name: String,
        listener: Option<&ViewModelInstanceListenerHandle>,
        request_id: u64,
    ) -> ViewModelInstanceHandle {
        let handle = self.next_view_model_handle();
        self.attach_view_model_listener(handle, listener);
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::InstantiateBlankViewModel)
            .write(file)
            .write(handle)
            .write(request_id);
        self.names.write(view_model_name);
        self.notify_command();
        handle
    }
    pub fn instantiate_view_model_instance_for_artboard(
        &mut self,
        file: FileHandle,
        artboard: ArtboardHandle,
        instance_name: String,
        listener: Option<&ViewModelInstanceListenerHandle>,
        request_id: u64,
    ) -> ViewModelInstanceHandle {
        let handle = self.next_view_model_handle();
        self.attach_view_model_listener(handle, listener);
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::InstantiateViewModelForArtboard)
            .write(file)
            .write(artboard)
            .write(handle)
            .write(request_id);
        self.names.write(instance_name);
        self.notify_command();
        handle
    }
    pub fn instantiate_view_model_instance_named(
        &mut self,
        file: FileHandle,
        view_model_name: String,
        instance_name: String,
        listener: Option<&ViewModelInstanceListenerHandle>,
        request_id: u64,
    ) -> ViewModelInstanceHandle {
        let handle = self.next_view_model_handle();
        self.attach_view_model_listener(handle, listener);
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::InstantiateViewModel)
            .write(file)
            .write(handle)
            .write(request_id);
        self.names.write(view_model_name).write(instance_name);
        self.notify_command();
        handle
    }
    pub fn instantiate_default_view_model_instance_for_artboard(
        &mut self,
        file: FileHandle,
        artboard: ArtboardHandle,
        listener: Option<&ViewModelInstanceListenerHandle>,
        request_id: u64,
    ) -> ViewModelInstanceHandle {
        self.instantiate_view_model_instance_for_artboard(
            file,
            artboard,
            String::new(),
            listener,
            request_id,
        )
    }
    pub fn instantiate_default_view_model_instance_named(
        &mut self,
        file: FileHandle,
        name: String,
        listener: Option<&ViewModelInstanceListenerHandle>,
        request_id: u64,
    ) -> ViewModelInstanceHandle {
        self.instantiate_view_model_instance_named(file, name, String::new(), listener, request_id)
    }
    pub fn reference_nested_view_model_instance(
        &mut self,
        source: ViewModelInstanceHandle,
        path: String,
        listener: Option<&ViewModelInstanceListenerHandle>,
        request_id: u64,
    ) -> ViewModelInstanceHandle {
        let handle = self.next_view_model_handle();
        self.attach_view_model_listener(handle, listener);
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::RefNestedViewModel)
            .write(source)
            .write(handle)
            .write(request_id);
        self.names.write(path);
        self.notify_command();
        handle
    }
    pub fn reference_list_view_model_instance(
        &mut self,
        source: ViewModelInstanceHandle,
        path: String,
        index: i32,
        listener: Option<&ViewModelInstanceListenerHandle>,
        request_id: u64,
    ) -> ViewModelInstanceHandle {
        let handle = self.next_view_model_handle();
        if let Some(listener) = listener {
            let listener = self.register_listener(listener, source);
            assert!(self.view_model_listeners.insert(handle, listener).is_none());
        }
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::RefListViewModel)
            .write(source)
            .write(index)
            .write(handle)
            .write(request_id);
        self.names.write(path);
        self.notify_command();
        handle
    }

    fn set_vm_prefix(
        &mut self,
        handle: ViewModelInstanceHandle,
        data_type: DataType,
        request_id: u64,
    ) {
        self.command_stream
            .write(Command::SetViewModelInstanceValue)
            .write(handle)
            .write(data_type)
            .write(request_id);
    }
    pub fn fire_view_model_trigger(
        &mut self,
        handle: ViewModelInstanceHandle,
        path: String,
        request_id: u64,
    ) {
        let _lock = self.command_mutex.lock().unwrap();
        self.set_vm_prefix(handle, DataType::Trigger, request_id);
        self.names.write(path);
        self.notify_command();
    }
    pub fn set_view_model_instance_bool(
        &mut self,
        handle: ViewModelInstanceHandle,
        path: String,
        value: bool,
        request_id: u64,
    ) {
        let _lock = self.command_mutex.lock().unwrap();
        self.set_vm_prefix(handle, DataType::Boolean, request_id);
        self.command_stream.write(value);
        self.names.write(path);
        self.notify_command();
    }
    pub fn set_view_model_instance_number(
        &mut self,
        handle: ViewModelInstanceHandle,
        path: String,
        value: f32,
        request_id: u64,
    ) {
        let _lock = self.command_mutex.lock().unwrap();
        self.set_vm_prefix(handle, DataType::Number, request_id);
        self.command_stream.write(value);
        self.names.write(path);
        self.notify_command();
    }
    pub fn set_view_model_instance_color(
        &mut self,
        handle: ViewModelInstanceHandle,
        path: String,
        value: u32,
        request_id: u64,
    ) {
        let _lock = self.command_mutex.lock().unwrap();
        self.set_vm_prefix(handle, DataType::Color, request_id);
        self.command_stream.write(value);
        self.names.write(path);
        self.notify_command();
    }
    pub fn set_view_model_instance_enum(
        &mut self,
        handle: ViewModelInstanceHandle,
        path: String,
        value: String,
        request_id: u64,
    ) {
        let _lock = self.command_mutex.lock().unwrap();
        self.set_vm_prefix(handle, DataType::EnumType, request_id);
        self.names.write(path).write(value);
        self.notify_command();
    }
    pub fn set_view_model_instance_string(
        &mut self,
        handle: ViewModelInstanceHandle,
        path: String,
        value: String,
        request_id: u64,
    ) {
        let _lock = self.command_mutex.lock().unwrap();
        self.set_vm_prefix(handle, DataType::String, request_id);
        self.names.write(path).write(value);
        self.notify_command();
    }
    pub fn set_view_model_instance_image(
        &mut self,
        handle: ViewModelInstanceHandle,
        path: String,
        value: RenderImageHandle,
        request_id: u64,
    ) {
        let _lock = self.command_mutex.lock().unwrap();
        self.set_vm_prefix(handle, DataType::AssetImage, request_id);
        self.command_stream.write(value);
        self.names.write(path);
        self.notify_command();
    }
    pub fn set_view_model_instance_blob(
        &mut self,
        handle: ViewModelInstanceHandle,
        path: String,
        value: BlobAssetHandle,
        request_id: u64,
    ) {
        let _lock = self.command_mutex.lock().unwrap();
        self.set_vm_prefix(handle, DataType::AssetBlob, request_id);
        self.command_stream.write(value);
        self.names.write(path);
        self.notify_command();
    }
    pub fn set_view_model_instance_artboard(
        &mut self,
        handle: ViewModelInstanceHandle,
        path: String,
        value: ArtboardHandle,
        request_id: u64,
    ) {
        let _lock = self.command_mutex.lock().unwrap();
        self.set_vm_prefix(handle, DataType::Artboard, request_id);
        self.command_stream.write(value);
        self.names.write(path);
        self.notify_command();
    }
    pub fn set_view_model_instance_nested_view_model(
        &mut self,
        handle: ViewModelInstanceHandle,
        path: String,
        value: ViewModelInstanceHandle,
        request_id: u64,
    ) {
        let _lock = self.command_mutex.lock().unwrap();
        self.set_vm_prefix(handle, DataType::ViewModel, request_id);
        self.command_stream.write(value);
        self.names.write(path);
        self.notify_command();
    }
    pub fn insert_view_model_instance_list_view_model(
        &mut self,
        handle: ViewModelInstanceHandle,
        path: String,
        value: ViewModelInstanceHandle,
        index: i32,
        request_id: u64,
    ) {
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::AddViewModelListValue)
            .write(handle)
            .write(value)
            .write(index)
            .write(request_id);
        self.names.write(path);
        self.notify_command();
    }
    pub fn append_view_model_instance_list_view_model(
        &mut self,
        handle: ViewModelInstanceHandle,
        path: String,
        value: ViewModelInstanceHandle,
        request_id: u64,
    ) {
        self.insert_view_model_instance_list_view_model(handle, path, value, -1, request_id);
    }
    pub fn remove_view_model_instance_list_view_model(
        &mut self,
        handle: ViewModelInstanceHandle,
        path: String,
        index: i32,
        value: ViewModelInstanceHandle,
        request_id: u64,
    ) {
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::RemoveViewModelListValue)
            .write(handle)
            .write(value)
            .write(index)
            .write(request_id);
        self.names.write(path);
        self.notify_command();
    }
    pub fn remove_view_model_instance_list_value(
        &mut self,
        handle: ViewModelInstanceHandle,
        path: String,
        value: ViewModelInstanceHandle,
        request_id: u64,
    ) {
        self.remove_view_model_instance_list_view_model(handle, path, -1, value, request_id);
    }
    pub fn swap_view_model_instance_list_values(
        &mut self,
        handle: ViewModelInstanceHandle,
        path: String,
        index_a: i32,
        index_b: i32,
        request_id: u64,
    ) {
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::SwapViewModelListValue)
            .write(handle)
            .write(index_a)
            .write(index_b)
            .write(request_id);
        self.names.write(path);
        self.notify_command();
    }
    pub fn subscribe_to_view_model_property(
        &mut self,
        handle: ViewModelInstanceHandle,
        path: String,
        data_type: DataType,
        request_id: u64,
    ) {
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::SubscribeViewModelProperty)
            .write(handle)
            .write(data_type)
            .write(request_id);
        self.names.write(path);
        self.notify_command();
    }
    pub fn unsubscribe_to_view_model_property(
        &mut self,
        handle: ViewModelInstanceHandle,
        path: String,
        data_type: DataType,
        request_id: u64,
    ) {
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::UnsubscribeViewModelProperty)
            .write(handle)
            .write(data_type)
            .write(request_id);
        self.names.write(path);
        self.notify_command();
    }
    pub fn delete_view_model_instance(&mut self, handle: ViewModelInstanceHandle, request_id: u64) {
        self.record_handle(Command::DeleteViewModel, handle, request_id);
    }

    pub fn instantiate_state_machine_named(
        &mut self,
        artboard: ArtboardHandle,
        name: String,
        listener: Option<&StateMachineListenerHandle>,
        request_id: u64,
    ) -> StateMachineHandle {
        let handle = self.next_state_machine_handle();
        if let Some(listener) = listener {
            let listener = self.register_listener(listener, handle);
            assert!(
                self.state_machine_listeners
                    .insert(handle, listener)
                    .is_none()
            );
        }
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::InstantiateStateMachine)
            .write(handle)
            .write(artboard)
            .write(request_id);
        self.names.write(name);
        self.notify_command();
        handle
    }
    pub fn instantiate_default_state_machine(
        &mut self,
        artboard: ArtboardHandle,
        listener: Option<&StateMachineListenerHandle>,
        request_id: u64,
    ) -> StateMachineHandle {
        self.instantiate_state_machine_named(artboard, String::new(), listener, request_id)
    }
    pub fn pointer_move(
        &mut self,
        handle: StateMachineHandle,
        event: PointerEvent,
        request_id: u64,
    ) {
        self.record_pointer(Command::PointerMove, handle, event, request_id);
    }
    pub fn pointer_down(
        &mut self,
        handle: StateMachineHandle,
        event: PointerEvent,
        request_id: u64,
    ) {
        self.record_pointer(Command::PointerDown, handle, event, request_id);
    }
    pub fn pointer_up(&mut self, handle: StateMachineHandle, event: PointerEvent, request_id: u64) {
        self.record_pointer(Command::PointerUp, handle, event, request_id);
    }
    pub fn pointer_exit(
        &mut self,
        handle: StateMachineHandle,
        event: PointerEvent,
        request_id: u64,
    ) {
        self.record_pointer(Command::PointerExit, handle, event, request_id);
    }
    fn record_pointer(
        &mut self,
        command: Command,
        handle: StateMachineHandle,
        event: PointerEvent,
        request_id: u64,
    ) {
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(command)
            .write(handle)
            .write(request_id);
        self.pointer_events.write(event);
        self.notify_command();
    }
    pub fn bind_view_model_instance(
        &mut self,
        handle: StateMachineHandle,
        view_model: ViewModelInstanceHandle,
        request_id: u64,
    ) {
        self.record_two_handles(
            Command::BindViewModelInstance,
            handle,
            view_model,
            request_id,
        );
    }
    pub fn set_view_model_instance(
        &mut self,
        handle: StateMachineHandle,
        view_model: ViewModelInstanceHandle,
        request_id: u64,
    ) {
        self.record_two_handles(
            Command::SetViewModelInstance,
            handle,
            view_model,
            request_id,
        );
    }
    pub fn set_global_view_model_instance(
        &mut self,
        handle: StateMachineHandle,
        name: String,
        view_model: ViewModelInstanceHandle,
        request_id: u64,
    ) {
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::SetGlobalViewModelInstance)
            .write(handle)
            .write(view_model)
            .write(request_id);
        self.names.write(name);
        self.notify_command();
    }
    pub fn global_view_model_instance(
        &mut self,
        state_machine: StateMachineHandle,
        name: String,
        listener: Option<&ViewModelInstanceListenerHandle>,
        request_id: u64,
    ) -> ViewModelInstanceHandle {
        let handle = self.next_view_model_handle();
        self.attach_view_model_listener(handle, listener);
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::GetGlobalViewModelInstance)
            .write(state_machine)
            .write(handle)
            .write(request_id);
        self.names.write(name);
        self.notify_command();
        handle
    }
    pub fn bind(&mut self, handle: StateMachineHandle, request_id: u64) {
        self.record_handle(Command::Bind, handle, request_id);
    }
    pub fn advance_state_machine(
        &mut self,
        handle: StateMachineHandle,
        elapsed: f32,
        request_id: u64,
    ) {
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::AdvanceStateMachine)
            .write(handle)
            .write(request_id)
            .write(elapsed);
        self.notify_command();
    }
    pub fn delete_state_machine(&mut self, handle: StateMachineHandle, request_id: u64) {
        self.record_handle(Command::DeleteStateMachine, handle, request_id);
    }
    pub fn enable_semantics(&mut self, handle: StateMachineHandle, request_id: u64) {
        self.record_handle(Command::EnableSemantics, handle, request_id);
    }
    pub fn drain_semantics_diff(
        &mut self,
        handle: StateMachineHandle,
        fit: Fit,
        alignment: Alignment,
        scale_factor: f32,
        view_bounds: Vec2D,
        request_id: u64,
    ) {
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::DrainSemanticsDiff)
            .write(handle)
            .write(request_id)
            .write(fit)
            .write(alignment.x())
            .write(alignment.y())
            .write(scale_factor)
            .write(view_bounds);
        self.notify_command();
    }
    pub fn fire_semantic_action(
        &mut self,
        handle: StateMachineHandle,
        node: u32,
        action: SemanticActionType,
        request_id: u64,
    ) {
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::FireSemanticAction)
            .write(handle)
            .write(request_id)
            .write(node)
            .write(action);
        self.notify_command();
    }
    pub fn request_semantic_focus(
        &mut self,
        handle: StateMachineHandle,
        node: u32,
        request_id: u64,
    ) {
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::RequestSemanticFocus)
            .write(handle)
            .write(request_id)
            .write(node);
        self.notify_command();
    }
    pub fn clear_semantic_focus(&mut self, handle: StateMachineHandle, request_id: u64) {
        self.record_handle(Command::ClearSemanticFocus, handle, request_id);
    }

    fn next_image_handle(&mut self) -> RenderImageHandle {
        self.current_render_image_handle_idx += 1;
        RenderImageHandle::from_index(self.current_render_image_handle_idx)
    }
    fn next_audio_handle(&mut self) -> AudioSourceHandle {
        self.current_audio_source_handle_idx += 1;
        AudioSourceHandle::from_index(self.current_audio_source_handle_idx)
    }
    fn next_font_handle(&mut self) -> FontHandle {
        self.current_font_handle_idx += 1;
        FontHandle::from_index(self.current_font_handle_idx)
    }
    fn next_blob_handle(&mut self) -> BlobAssetHandle {
        self.current_blob_asset_handle_idx += 1;
        BlobAssetHandle::from_index(self.current_blob_asset_handle_idx)
    }

    pub fn decode_image(
        &mut self,
        bytes: Vec<u8>,
        listener: Option<&RenderImageListenerHandle>,
        request_id: u64,
    ) -> RenderImageHandle {
        let handle = self.next_image_handle();
        if let Some(listener) = listener {
            let listener = self.register_listener(listener, handle);
            assert!(self.image_listeners.insert(handle, listener).is_none());
        }
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::DecodeImage)
            .write(handle)
            .write(request_id);
        self.byte_vectors.write(bytes);
        self.notify_command();
        handle
    }
    pub fn add_external_image(
        &mut self,
        image: RenderImageRef,
        listener: Option<&RenderImageListenerHandle>,
        request_id: u64,
    ) -> RenderImageHandle {
        let handle = self.next_image_handle();
        if let Some(listener) = listener {
            let listener = self.register_listener(listener, handle);
            assert!(self.image_listeners.insert(handle, listener).is_none());
        }
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::ExternalImage)
            .write(handle)
            .write(request_id);
        self.external_images.write(image);
        self.notify_command();
        handle
    }
    pub fn delete_image(&mut self, handle: RenderImageHandle, request_id: u64) {
        self.record_handle(Command::DeleteImage, handle, request_id);
    }
    pub fn decode_audio(
        &mut self,
        bytes: Vec<u8>,
        listener: Option<&AudioSourceListenerHandle>,
        request_id: u64,
    ) -> AudioSourceHandle {
        let handle = self.next_audio_handle();
        if let Some(listener) = listener {
            let listener = self.register_listener(listener, handle);
            assert!(self.audio_listeners.insert(handle, listener).is_none());
        }
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::DecodeAudio)
            .write(handle)
            .write(request_id);
        self.byte_vectors.write(bytes);
        self.notify_command();
        handle
    }
    pub fn add_external_audio(
        &mut self,
        audio: Rcp<AudioSource>,
        listener: Option<&AudioSourceListenerHandle>,
        request_id: u64,
    ) -> AudioSourceHandle {
        let handle = self.next_audio_handle();
        if let Some(listener) = listener {
            let listener = self.register_listener(listener, handle);
            assert!(self.audio_listeners.insert(handle, listener).is_none());
        }
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::ExternalAudio)
            .write(handle)
            .write(request_id);
        self.external_audio_sources.write(audio);
        self.notify_command();
        handle
    }
    pub fn delete_audio(&mut self, handle: AudioSourceHandle, request_id: u64) {
        self.record_handle(Command::DeleteAudio, handle, request_id);
    }
    pub fn decode_font(
        &mut self,
        bytes: Vec<u8>,
        listener: Option<&FontListenerHandle>,
        request_id: u64,
    ) -> FontHandle {
        let handle = self.next_font_handle();
        if let Some(listener) = listener {
            let listener = self.register_listener(listener, handle);
            assert!(self.font_listeners.insert(handle, listener).is_none());
        }
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::DecodeFont)
            .write(handle)
            .write(request_id);
        self.byte_vectors.write(bytes);
        self.notify_command();
        handle
    }
    pub fn add_external_font(
        &mut self,
        font: Rcp<Font>,
        listener: Option<&FontListenerHandle>,
        request_id: u64,
    ) -> FontHandle {
        let handle = self.next_font_handle();
        if let Some(listener) = listener {
            let listener = self.register_listener(listener, handle);
            assert!(self.font_listeners.insert(handle, listener).is_none());
        }
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::ExternalFont)
            .write(handle)
            .write(request_id);
        self.external_fonts.write(font);
        self.notify_command();
        handle
    }
    pub fn delete_font(&mut self, handle: FontHandle, request_id: u64) {
        self.record_handle(Command::DeleteFont, handle, request_id);
    }
    pub fn decode_blob(
        &mut self,
        bytes: Vec<u8>,
        listener: Option<&BlobAssetListenerHandle>,
        request_id: u64,
    ) -> BlobAssetHandle {
        let handle = self.next_blob_handle();
        if let Some(listener) = listener {
            let listener = self.register_listener(listener, handle);
            assert!(self.blob_listeners.insert(handle, listener).is_none());
        }
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::DecodeBlob)
            .write(handle)
            .write(request_id);
        self.byte_vectors.write(bytes);
        self.notify_command();
        handle
    }
    pub fn add_external_blob(
        &mut self,
        blob: Rcp<BlobAsset>,
        listener: Option<&BlobAssetListenerHandle>,
        request_id: u64,
    ) -> BlobAssetHandle {
        let handle = self.next_blob_handle();
        if let Some(listener) = listener {
            let listener = self.register_listener(listener, handle);
            assert!(self.blob_listeners.insert(handle, listener).is_none());
        }
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::ExternalBlob)
            .write(handle)
            .write(request_id);
        self.external_blobs.write(blob);
        self.notify_command();
        handle
    }
    pub fn delete_blob(&mut self, handle: BlobAssetHandle, request_id: u64) {
        self.record_handle(Command::DeleteBlob, handle, request_id);
    }

    fn record_handle<H: Copy>(&mut self, command: Command, handle: H, request_id: u64) {
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(command)
            .write(handle)
            .write(request_id);
        self.notify_command();
    }
    fn record_two_handles<A: Copy, B: Copy>(
        &mut self,
        command: Command,
        first: A,
        second: B,
        request_id: u64,
    ) {
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(command)
            .write(first)
            .write(second)
            .write(request_id);
        self.notify_command();
    }
    pub fn create_draw_key(&mut self) -> DrawKey {
        let _lock = self.command_mutex.lock().unwrap();
        self.current_draw_key_idx += 1;
        let key = DrawKey::from_index(self.current_draw_key_idx);
        self.notify_command();
        key
    }
    pub fn run_once(&mut self, callback: CommandServerCallback) {
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream.write(Command::RunOnce);
        self.callbacks.write(callback);
        self.notify_command();
    }
    pub fn draw(&mut self, key: DrawKey, callback: CommandServerDrawCallback) {
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream.write(Command::Draw).write(key);
        self.draw_callbacks.write(callback);
        self.notify_command();
    }
    pub fn cancel_draw(&mut self, key: DrawKey) {
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream.write(Command::CancelDraw).write(key);
        self.notify_command();
    }
    #[cfg(test)]
    pub fn testing_command_loop_break(&mut self) {
        self.write_command(Command::CommandLoopBreak);
    }
    #[cfg(test)]
    pub fn testing_get_file_listener(&mut self, handle: FileHandle) -> Option<FileListenerHandle> {
        self.file_listeners
            .get(&handle)
            .and_then(WeakListenerHandle::upgrade)
    }
    #[cfg(test)]
    pub fn testing_get_artboard_listener(
        &mut self,
        handle: ArtboardHandle,
    ) -> Option<ArtboardListenerHandle> {
        self.artboard_listeners
            .get(&handle)
            .and_then(WeakListenerHandle::upgrade)
    }
    #[cfg(test)]
    pub fn testing_get_state_machine_listener(
        &mut self,
        handle: StateMachineHandle,
    ) -> Option<StateMachineListenerHandle> {
        self.state_machine_listeners
            .get(&handle)
            .and_then(WeakListenerHandle::upgrade)
    }
    pub fn disconnect(&mut self) {
        self.write_command(Command::Disconnect);
    }

    fn request_file(&mut self, command: Command, handle: FileHandle, request_id: u64) {
        self.record_handle(command, handle, request_id);
    }
    pub fn request_view_model_names(&mut self, handle: FileHandle, request_id: u64) {
        self.request_file(Command::ListViewModels, handle, request_id);
    }
    pub fn request_global_view_model_names(&mut self, handle: FileHandle, request_id: u64) {
        self.request_file(Command::ListGlobalViewModelNames, handle, request_id);
    }
    pub fn request_artboard_names(&mut self, handle: FileHandle, request_id: u64) {
        self.request_file(Command::ListArtboards, handle, request_id);
    }
    pub fn request_file_assets(&mut self, handle: FileHandle, request_id: u64) {
        self.request_file(Command::ListFileAssets, handle, request_id);
    }
    pub fn request_view_model_enums(&mut self, handle: FileHandle, request_id: u64) {
        self.request_file(Command::ListViewModelEnums, handle, request_id);
    }
    pub fn request_view_model_property_definitions(
        &mut self,
        handle: FileHandle,
        name: String,
        request_id: u64,
    ) {
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::ListViewModelProperties)
            .write(handle)
            .write(request_id);
        self.names.write(name);
        self.notify_command();
    }
    pub fn request_view_model_instance_names(
        &mut self,
        handle: FileHandle,
        name: String,
        request_id: u64,
    ) {
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::ListViewModelInstanceNames)
            .write(handle)
            .write(request_id);
        self.names.write(name);
        self.notify_command();
    }
    pub fn request_view_model_instance_view_model_name(
        &mut self,
        handle: ViewModelInstanceHandle,
        request_id: u64,
    ) {
        self.record_handle(
            Command::GetViewModelInstanceViewModelName,
            handle,
            request_id,
        );
    }
    pub fn request_view_model_instance_name(
        &mut self,
        handle: ViewModelInstanceHandle,
        request_id: u64,
    ) {
        self.record_handle(Command::GetViewModelInstanceName, handle, request_id);
    }
    fn request_vm_value(
        &mut self,
        handle: ViewModelInstanceHandle,
        path: String,
        data_type: DataType,
        request_id: u64,
    ) {
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::ListViewModelPropertyValue)
            .write(data_type)
            .write(handle)
            .write(request_id);
        self.names.write(path);
        self.notify_command();
    }
    pub fn request_view_model_instance_bool(
        &mut self,
        handle: ViewModelInstanceHandle,
        path: String,
        id: u64,
    ) {
        self.request_vm_value(handle, path, DataType::Boolean, id);
    }
    pub fn request_view_model_instance_number(
        &mut self,
        handle: ViewModelInstanceHandle,
        path: String,
        id: u64,
    ) {
        self.request_vm_value(handle, path, DataType::Number, id);
    }
    pub fn request_view_model_instance_color(
        &mut self,
        handle: ViewModelInstanceHandle,
        path: String,
        id: u64,
    ) {
        self.request_vm_value(handle, path, DataType::Color, id);
    }
    pub fn request_view_model_instance_enum(
        &mut self,
        handle: ViewModelInstanceHandle,
        path: String,
        id: u64,
    ) {
        self.request_vm_value(handle, path, DataType::EnumType, id);
    }
    pub fn request_view_model_instance_string(
        &mut self,
        handle: ViewModelInstanceHandle,
        path: String,
        id: u64,
    ) {
        self.request_vm_value(handle, path, DataType::String, id);
    }
    pub fn request_view_model_instance_list_size(
        &mut self,
        handle: ViewModelInstanceHandle,
        path: String,
        request_id: u64,
    ) {
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::GetViewModelListSize)
            .write(handle)
            .write(request_id);
        self.names.write(path);
        self.notify_command();
    }
    pub fn request_view_model_instance_list_clear(
        &mut self,
        handle: ViewModelInstanceHandle,
        path: String,
        request_id: u64,
    ) {
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::ClearViewModelList)
            .write(handle)
            .write(request_id);
        self.names.write(path);
        self.notify_command();
    }
    pub fn set_artboard_volume(&mut self, handle: ArtboardHandle, volume: f32, request_id: u64) {
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::SetArtboardVolume)
            .write(handle)
            .write(volume)
            .write(request_id);
        self.notify_command();
    }
    pub fn request_artboard_volume(&mut self, handle: ArtboardHandle, request_id: u64) {
        self.record_handle(Command::GetArtboardVolume, handle, request_id);
    }
    pub fn request_artboard_size(&mut self, handle: ArtboardHandle, request_id: u64) {
        self.record_handle(Command::GetArtboardSize, handle, request_id);
    }
    pub fn request_state_machine_names(&mut self, handle: ArtboardHandle, request_id: u64) {
        self.record_handle(Command::ListStateMachines, handle, request_id);
    }
    pub fn request_default_view_model_info(
        &mut self,
        artboard: ArtboardHandle,
        file: FileHandle,
        request_id: u64,
    ) {
        let _lock = self.command_mutex.lock().unwrap();
        self.command_stream
            .write(Command::GetDefaultViewModel)
            .write(file)
            .write(artboard)
            .write(request_id);
        self.notify_command();
    }

    pub fn set_global_file_listener(&mut self, listener: Option<&FileListenerHandle>) {
        self.global_file_listener = listener.map(|listener| listener.downgrade());
    }
    pub fn set_global_artboard_listener(&mut self, listener: Option<&ArtboardListenerHandle>) {
        self.global_artboard_listener = listener.map(|listener| listener.downgrade());
    }
    pub fn set_global_state_machine_listener(
        &mut self,
        listener: Option<&StateMachineListenerHandle>,
    ) {
        self.global_state_machine_listener = listener.map(|listener| listener.downgrade());
    }
    pub fn set_global_view_model_instance_listener(
        &mut self,
        listener: Option<&ViewModelInstanceListenerHandle>,
    ) {
        self.global_view_model_listener = listener.map(|listener| listener.downgrade());
    }
    pub fn set_global_render_image_listener(
        &mut self,
        listener: Option<&RenderImageListenerHandle>,
    ) {
        self.global_image_listener = listener.map(|listener| listener.downgrade());
    }
    pub fn set_global_audio_source_listener(
        &mut self,
        listener: Option<&AudioSourceListenerHandle>,
    ) {
        self.global_audio_listener = listener.map(|listener| listener.downgrade());
    }
    pub fn set_global_font_listener(&mut self, listener: Option<&FontListenerHandle>) {
        self.global_font_listener = listener.map(|listener| listener.downgrade());
    }
    pub fn set_global_blob_asset_listener(&mut self, listener: Option<&BlobAssetListenerHandle>) {
        self.global_blob_listener = listener.map(|listener| listener.downgrade());
    }

    fn read_message_pod<T: Copy + Default>(&mut self) -> T {
        let mut value = T::default();
        self.message_stream.read(&mut value);
        value
    }

    fn read_message_names(&mut self, count: usize) -> Vec<String> {
        (0..count).map(|_| self.message_names.read()).collect()
    }

    pub fn process_messages(&mut self) {
        let mut lock = self.message_mutex.lock().unwrap();
        if self.message_stream.empty() {
            return;
        }
        self.message_stream.write(Message::MessageLoopBreak);
        loop {
            let message: Message = self.read_message_pod();
            match message {
                Message::MessageLoopBreak => {
                    drop(lock);
                    return;
                }
                Message::ViewModelEnumsListed => {
                    let handle = self.read_message_pod::<FileHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    let count = self.read_message_pod::<usize>();
                    let mut enums = Vec::with_capacity(count);
                    for _ in 0..count {
                        let enumerant_count = self.read_message_pod::<usize>();
                        enums.push(ViewModelEnum {
                            name: self.message_names.read(),
                            enumerants: self.read_message_names(enumerant_count),
                        });
                    }
                    drop(lock);
                    if let Some(listener) = self
                        .global_file_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener.borrow_mut().on_view_model_enums_listed(
                            handle,
                            request_id,
                            enums.clone(),
                        );
                    }
                    if let Some(listener) = self
                        .file_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_view_model_enums_listed(handle, request_id, enums);
                    }
                }
                Message::ArtboardsListed => {
                    let handle = self.read_message_pod::<FileHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    let count = self.read_message_pod::<usize>();
                    let names = self.read_message_names(count);
                    drop(lock);
                    if let Some(listener) = self
                        .global_file_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener.borrow_mut().on_artboards_listed(
                            handle,
                            request_id,
                            names.clone(),
                        );
                    }
                    if let Some(listener) = self
                        .file_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_artboards_listed(handle, request_id, names);
                    }
                }
                Message::FileAssetsListed => {
                    let handle = self.read_message_pod::<FileHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    let count = self.read_message_pod::<usize>();
                    let mut assets = Vec::with_capacity(count);
                    for _ in 0..count {
                        assets.push(FileAssetData {
                            asset_id: self.read_message_pod(),
                            asset_type: self.read_message_pod(),
                            name: self.message_names.read(),
                            cdn_uuid: self.message_names.read(),
                            cdn_base_url: self.message_names.read(),
                            file_extension: self.message_names.read(),
                        });
                    }
                    drop(lock);
                    if let Some(listener) = self
                        .global_file_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener.borrow_mut().on_file_assets_listed(
                            handle,
                            request_id,
                            assets.clone(),
                        );
                    }
                    if let Some(listener) = self
                        .file_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_file_assets_listed(handle, request_id, assets);
                    }
                }
                Message::StateMachinesListed => {
                    let handle = self.read_message_pod::<ArtboardHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    let count = self.read_message_pod::<usize>();
                    let names = self.read_message_names(count);
                    drop(lock);
                    if let Some(listener) = self
                        .global_artboard_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener.borrow_mut().on_state_machines_listed(
                            handle,
                            request_id,
                            names.clone(),
                        );
                    }
                    if let Some(listener) = self
                        .artboard_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_state_machines_listed(handle, request_id, names);
                    }
                }
                Message::DefaultViewModelReceived => {
                    let handle = self.read_message_pod::<ArtboardHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    let view_model = self.message_names.read();
                    let instance = self.message_names.read();
                    drop(lock);
                    if let Some(listener) = self
                        .global_artboard_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener.borrow_mut().on_default_view_model_info_received(
                            handle,
                            request_id,
                            view_model.clone(),
                            instance.clone(),
                        );
                    }
                    if let Some(listener) = self
                        .artboard_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener.borrow_mut().on_default_view_model_info_received(
                            handle, request_id, view_model, instance,
                        );
                    }
                }
                Message::ArtboardVolumeReceived => {
                    let handle = self.read_message_pod::<ArtboardHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    let volume = self.read_message_pod::<f32>();
                    drop(lock);
                    if let Some(listener) = self
                        .global_artboard_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_artboard_volume_received(handle, request_id, volume);
                    }
                    if let Some(listener) = self
                        .artboard_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_artboard_volume_received(handle, request_id, volume);
                    }
                }
                Message::ViewModelInstanceViewModelNameReceived => {
                    let handle = self.read_message_pod::<ViewModelInstanceHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    let name = self.message_names.read();
                    drop(lock);
                    if let Some(listener) = self
                        .global_view_model_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_view_model_instance_view_model_name_received(
                                handle,
                                request_id,
                                name.clone(),
                            );
                    }
                    if let Some(listener) = self
                        .view_model_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_view_model_instance_view_model_name_received(
                                handle, request_id, name,
                            );
                    }
                }
                Message::ViewModelInstanceNameReceived => {
                    let handle = self.read_message_pod::<ViewModelInstanceHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    let name = self.message_names.read();
                    drop(lock);
                    if let Some(listener) = self
                        .global_view_model_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener.borrow_mut().on_view_model_instance_name_received(
                            handle,
                            request_id,
                            name.clone(),
                        );
                    }
                    if let Some(listener) = self
                        .view_model_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_view_model_instance_name_received(handle, request_id, name);
                    }
                }
                Message::ViewModelsListed | Message::GlobalViewModelNamesListed => {
                    let handle = self.read_message_pod::<FileHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    let count = self.read_message_pod::<usize>();
                    let names = self.read_message_names(count);
                    let globals = matches!(message, Message::GlobalViewModelNamesListed);
                    drop(lock);
                    if let Some(listener) = self
                        .global_file_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        if globals {
                            listener.borrow_mut().on_global_view_model_names_listed(
                                handle,
                                request_id,
                                names.clone(),
                            );
                        } else {
                            listener.borrow_mut().on_view_models_listed(
                                handle,
                                request_id,
                                names.clone(),
                            );
                        }
                    }
                    if let Some(listener) = self
                        .file_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        if globals {
                            listener
                                .borrow_mut()
                                .on_global_view_model_names_listed(handle, request_id, names);
                        } else {
                            listener
                                .borrow_mut()
                                .on_view_models_listed(handle, request_id, names);
                        }
                    }
                }
                Message::ViewModelInstanceNamesListed => {
                    let handle = self.read_message_pod::<FileHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    let count = self.read_message_pod::<usize>();
                    let model = self.message_names.read();
                    let names = self.read_message_names(count);
                    drop(lock);
                    if let Some(listener) = self
                        .global_file_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener.borrow_mut().on_view_model_instance_names_listed(
                            handle,
                            request_id,
                            model.clone(),
                            names.clone(),
                        );
                    }
                    if let Some(listener) = self
                        .file_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_view_model_instance_names_listed(handle, request_id, model, names);
                    }
                }
                Message::ViewModelPropertiesListed => {
                    let handle = self.read_message_pod::<FileHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    let count = self.read_message_pod::<usize>();
                    let model = self.message_names.read();
                    let mut properties = Vec::with_capacity(count);
                    for _ in 0..count {
                        let data_type = self.read_message_pod::<DataType>();
                        let name = self.message_names.read();
                        let meta_data =
                            if matches!(data_type, DataType::EnumType | DataType::ViewModel) {
                                self.message_names.read()
                            } else {
                                String::new()
                            };
                        properties.push(ViewModelPropertyData {
                            data_type,
                            name,
                            meta_data,
                        });
                    }
                    drop(lock);
                    if let Some(listener) = self
                        .global_file_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener.borrow_mut().on_view_model_properties_listed(
                            handle,
                            request_id,
                            model.clone(),
                            properties.clone(),
                        );
                    }
                    if let Some(listener) = self
                        .file_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_view_model_properties_listed(handle, request_id, model, properties);
                    }
                }
                Message::ViewModelPropertyValueReceived => {
                    let handle = self.read_message_pod::<ViewModelInstanceHandle>();
                    let data_type = self.read_message_pod::<DataType>();
                    let request_id = self.read_message_pod::<u64>();
                    let name = self.message_names.read();
                    let value = match data_type {
                        DataType::AssetImage
                        | DataType::AssetBlob
                        | DataType::List
                        | DataType::Trigger => ViewModelInstanceValue::None,
                        DataType::Boolean => ViewModelInstanceValue::Bool(self.read_message_pod()),
                        DataType::Number => ViewModelInstanceValue::Number(self.read_message_pod()),
                        DataType::Color => ViewModelInstanceValue::Color(self.read_message_pod()),
                        DataType::EnumType | DataType::String => {
                            ViewModelInstanceValue::String(self.message_names.read())
                        }
                        _ => unreachable!(),
                    };
                    let data = ViewModelInstanceData {
                        meta_data: PropertyData { data_type, name },
                        value,
                    };
                    drop(lock);
                    if let Some(listener) = self
                        .global_view_model_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener.borrow_mut().on_view_model_data_received(
                            handle,
                            request_id,
                            data.clone(),
                        );
                    }
                    if let Some(listener) = self
                        .view_model_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_view_model_data_received(handle, request_id, data);
                    }
                }
                Message::ViewModelListSizeReceived => {
                    let handle = self.read_message_pod::<ViewModelInstanceHandle>();
                    let size = self.read_message_pod::<usize>();
                    let request_id = self.read_message_pod::<u64>();
                    let path = self.message_names.read();
                    drop(lock);
                    if let Some(listener) = self
                        .global_view_model_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener.borrow_mut().on_view_model_list_size_received(
                            handle,
                            request_id,
                            path.clone(),
                            size,
                        );
                    }
                    if let Some(listener) = self
                        .view_model_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_view_model_list_size_received(handle, request_id, path, size);
                    }
                }
                Message::ViewModelListCleared => {
                    let handle = self.read_message_pod::<ViewModelInstanceHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    let path = self.message_names.read();
                    drop(lock);
                    if let Some(listener) = self
                        .global_view_model_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener.borrow_mut().on_view_model_list_cleared(
                            handle,
                            request_id,
                            path.clone(),
                        );
                    }
                    if let Some(listener) = self
                        .view_model_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_view_model_list_cleared(handle, request_id, path);
                    }
                }
                Message::FileLoaded => {
                    let handle = self.read_message_pod::<FileHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    drop(lock);
                    if let Some(listener) = self
                        .global_file_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener.borrow_mut().on_file_loaded(handle, request_id);
                    }
                    if let Some(listener) = self
                        .file_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener.borrow_mut().on_file_loaded(handle, request_id);
                    }
                }
                Message::FileDeleted => {
                    let handle = self.read_message_pod::<FileHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    drop(lock);
                    if let Some(listener) = self
                        .global_file_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener.borrow_mut().on_file_deleted(handle, request_id);
                    }
                    if let Some(listener) = self
                        .file_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener.borrow_mut().on_file_deleted(handle, request_id);
                    }
                }
                Message::ArtboardInstantiated => {
                    let file = self.read_message_pod::<FileHandle>();
                    let handle = self.read_message_pod::<ArtboardHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    drop(lock);
                    if let Some(listener) = self
                        .global_file_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_artboard_instantiated(file, request_id, handle);
                    }
                    if let Some(listener) = self
                        .file_listeners
                        .get(&file)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_artboard_instantiated(file, request_id, handle);
                    }
                }
                Message::StateMachineInstantiated => {
                    let artboard = self.read_message_pod::<ArtboardHandle>();
                    let handle = self.read_message_pod::<StateMachineHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    drop(lock);
                    if let Some(listener) = self
                        .global_artboard_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_state_machine_instantiated(artboard, request_id, handle);
                    }
                    if let Some(listener) = self
                        .artboard_listeners
                        .get(&artboard)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_state_machine_instantiated(artboard, request_id, handle);
                    }
                }
                Message::ViewModelInstanceInstantiated => {
                    let file = self.read_message_pod::<FileHandle>();
                    let handle = self.read_message_pod::<ViewModelInstanceHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    drop(lock);
                    if let Some(listener) = self
                        .global_file_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_view_model_instance_instantiated(file, request_id, handle);
                    }
                    if let Some(listener) = self
                        .file_listeners
                        .get(&file)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_view_model_instance_instantiated(file, request_id, handle);
                    }
                }
                Message::ImageDecoded => {
                    let handle = self.read_message_pod::<RenderImageHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    drop(lock);
                    if let Some(listener) = self
                        .global_image_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_render_image_decoded(handle, request_id);
                    }
                    if let Some(listener) = self
                        .image_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_render_image_decoded(handle, request_id);
                    }
                }
                Message::ImageDeleted => {
                    let handle = self.read_message_pod::<RenderImageHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    drop(lock);
                    if let Some(listener) = self
                        .global_image_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_render_image_deleted(handle, request_id);
                    }
                    if let Some(listener) = self
                        .image_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_render_image_deleted(handle, request_id);
                    }
                }
                Message::AudioDecoded => {
                    let handle = self.read_message_pod::<AudioSourceHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    drop(lock);
                    if let Some(listener) = self
                        .global_audio_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_audio_source_decoded(handle, request_id);
                    }
                    if let Some(listener) = self
                        .audio_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_audio_source_decoded(handle, request_id);
                    }
                }
                Message::AudioDeleted => {
                    let handle = self.read_message_pod::<AudioSourceHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    drop(lock);
                    if let Some(listener) = self
                        .global_audio_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_audio_source_deleted(handle, request_id);
                    }
                    if let Some(listener) = self
                        .audio_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_audio_source_deleted(handle, request_id);
                    }
                }
                Message::FontDecoded => {
                    let handle = self.read_message_pod::<FontHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    drop(lock);
                    if let Some(listener) = self
                        .global_font_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener.borrow_mut().on_font_decoded(handle, request_id);
                    }
                    if let Some(listener) = self
                        .font_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener.borrow_mut().on_font_decoded(handle, request_id);
                    }
                }
                Message::FontDeleted => {
                    let handle = self.read_message_pod::<FontHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    drop(lock);
                    if let Some(listener) = self
                        .global_font_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener.borrow_mut().on_font_deleted(handle, request_id);
                    }
                    if let Some(listener) = self
                        .font_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener.borrow_mut().on_font_deleted(handle, request_id);
                    }
                }
                Message::BlobDecoded => {
                    let handle = self.read_message_pod::<BlobAssetHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    drop(lock);
                    if let Some(listener) = self
                        .global_blob_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_blob_asset_decoded(handle, request_id);
                    }
                    if let Some(listener) = self
                        .blob_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_blob_asset_decoded(handle, request_id);
                    }
                }
                Message::BlobDeleted => {
                    let handle = self.read_message_pod::<BlobAssetHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    drop(lock);
                    if let Some(listener) = self
                        .global_blob_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_blob_asset_deleted(handle, request_id);
                    }
                    if let Some(listener) = self
                        .blob_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_blob_asset_deleted(handle, request_id);
                    }
                }
                Message::ArtboardDeleted => {
                    let handle = self.read_message_pod::<ArtboardHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    drop(lock);
                    if let Some(listener) = self
                        .global_artboard_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_artboard_deleted(handle, request_id);
                    }
                    if let Some(listener) = self
                        .artboard_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_artboard_deleted(handle, request_id);
                    }
                }
                Message::ViewModelDeleted => {
                    let handle = self.read_message_pod::<ViewModelInstanceHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    drop(lock);
                    if let Some(listener) = self
                        .global_view_model_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_view_model_deleted(handle, request_id);
                    }
                    if let Some(listener) = self
                        .view_model_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_view_model_deleted(handle, request_id);
                    }
                }
                Message::StateMachineSettled => {
                    let handle = self.read_message_pod::<StateMachineHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    drop(lock);
                    if let Some(listener) = self
                        .global_state_machine_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_state_machine_settled(handle, request_id);
                    }
                    if let Some(listener) = self
                        .state_machine_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_state_machine_settled(handle, request_id);
                    }
                }
                Message::StateMachineDeleted => {
                    let handle = self.read_message_pod::<StateMachineHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    drop(lock);
                    if let Some(listener) = self
                        .global_state_machine_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_state_machine_deleted(handle, request_id);
                    }
                    if let Some(listener) = self
                        .state_machine_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_state_machine_deleted(handle, request_id);
                    }
                }
                Message::SemanticsDiffReceived => {
                    let handle = self.read_message_pod::<StateMachineHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    let mut diff = Some(self.message_semantics_diffs.read());
                    drop(lock);
                    let has_specific = self.state_machine_listeners.contains_key(&handle);
                    if let Some(listener) = self
                        .global_state_machine_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        let delivered = if has_specific {
                            diff.as_ref().unwrap().clone()
                        } else {
                            diff.take().unwrap()
                        };
                        listener
                            .borrow_mut()
                            .on_semantics_diff_received(handle, request_id, delivered);
                    }
                    if has_specific {
                        if let Some(listener) = self
                            .state_machine_listeners
                            .get(&handle)
                            .and_then(WeakListenerHandle::upgrade)
                        {
                            listener.borrow_mut().on_semantics_diff_received(
                                handle,
                                request_id,
                                diff.take().unwrap(),
                            );
                        }
                    }
                }
                Message::ArtboardSizeReceived => {
                    let handle = self.read_message_pod::<ArtboardHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    let width = self.read_message_pod::<f32>();
                    let height = self.read_message_pod::<f32>();
                    drop(lock);
                    if let Some(listener) = self
                        .global_artboard_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_artboard_size_received(handle, request_id, width, height);
                    }
                    if let Some(listener) = self
                        .artboard_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_artboard_size_received(handle, request_id, width, height);
                    }
                }
                Message::FileError => {
                    let handle = self.read_message_pod::<FileHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    let error = self.message_names.read();
                    drop(lock);
                    if let Some(listener) = self
                        .global_file_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_file_error(handle, request_id, error.clone());
                    }
                    if let Some(listener) = self
                        .file_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_file_error(handle, request_id, error);
                    }
                }
                Message::ViewModelError => {
                    let handle = self.read_message_pod::<ViewModelInstanceHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    let error = self.message_names.read();
                    drop(lock);
                    if let Some(listener) = self
                        .global_view_model_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener.borrow_mut().on_view_model_instance_error(
                            handle,
                            request_id,
                            error.clone(),
                        );
                    }
                    if let Some(listener) = self
                        .view_model_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_view_model_instance_error(handle, request_id, error);
                    }
                }
                Message::ImageError => {
                    let handle = self.read_message_pod::<RenderImageHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    let error = self.message_names.read();
                    drop(lock);
                    if let Some(listener) = self
                        .global_image_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener.borrow_mut().on_render_image_error(
                            handle,
                            request_id,
                            error.clone(),
                        );
                    }
                    if let Some(listener) = self
                        .image_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_render_image_error(handle, request_id, error);
                    }
                }
                Message::AudioError => {
                    let handle = self.read_message_pod::<AudioSourceHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    let error = self.message_names.read();
                    drop(lock);
                    if let Some(listener) = self
                        .global_audio_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener.borrow_mut().on_audio_source_error(
                            handle,
                            request_id,
                            error.clone(),
                        );
                    }
                    if let Some(listener) = self
                        .audio_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_audio_source_error(handle, request_id, error);
                    }
                }
                Message::FontError => {
                    let handle = self.read_message_pod::<FontHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    let error = self.message_names.read();
                    drop(lock);
                    if let Some(listener) = self
                        .global_font_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_font_error(handle, request_id, error.clone());
                    }
                    if let Some(listener) = self
                        .font_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_font_error(handle, request_id, error);
                    }
                }
                Message::BlobError => {
                    let handle = self.read_message_pod::<BlobAssetHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    let error = self.message_names.read();
                    drop(lock);
                    if let Some(listener) = self
                        .global_blob_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener.borrow_mut().on_blob_asset_error(
                            handle,
                            request_id,
                            error.clone(),
                        );
                    }
                    if let Some(listener) = self
                        .blob_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_blob_asset_error(handle, request_id, error);
                    }
                }
                Message::StateMachineError => {
                    let handle = self.read_message_pod::<StateMachineHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    let error = self.message_names.read();
                    drop(lock);
                    if let Some(listener) = self
                        .global_state_machine_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener.borrow_mut().on_state_machine_error(
                            handle,
                            request_id,
                            error.clone(),
                        );
                    }
                    if let Some(listener) = self
                        .state_machine_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_state_machine_error(handle, request_id, error);
                    }
                }
                Message::ArtboardError => {
                    let handle = self.read_message_pod::<ArtboardHandle>();
                    let request_id = self.read_message_pod::<u64>();
                    let error = self.message_names.read();
                    drop(lock);
                    if let Some(listener) = self
                        .global_artboard_listener
                        .as_ref()
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_artboard_error(handle, request_id, error.clone());
                    }
                    if let Some(listener) = self
                        .artboard_listeners
                        .get(&handle)
                        .and_then(WeakListenerHandle::upgrade)
                    {
                        listener
                            .borrow_mut()
                            .on_artboard_error(handle, request_id, error);
                    }
                }
            }
            lock = self.message_mutex.lock().unwrap();
            if self.message_stream.empty() {
                break;
            }
        }
    }
}
