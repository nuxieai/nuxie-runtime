//! Thread-safe client recorder for the pinned C++ `CommandQueue` protocol.
//!
//! Direct port of `include/rive/command_queue.hpp` and
//! `src/command_queue.cpp` at `4ac7b327`. Runtime objects never cross this
//! seam: the queue owns typed, ordered command payloads and the server owns
//! every file/artboard/state-machine/view-model occurrence.

use std::{
    collections::{BTreeMap, VecDeque},
    fmt,
    sync::{Arc, Condvar, Mutex, MutexGuard, Weak},
};

use crate::{AudioSource, RawTextFont, RenderImage, Vec2D, command_server::CommandServer};

macro_rules! command_handle {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(u64);

        impl $name {
            pub const fn get(self) -> u64 {
                self.0
            }
        }
    };
}

command_handle!(FileHandle);
command_handle!(ArtboardHandle);
command_handle!(StateMachineHandle);
command_handle!(ViewModelInstanceHandle);
command_handle!(RenderImageHandle);
command_handle!(AudioSourceHandle);
command_handle!(FontHandle);
command_handle!(DrawKey);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandDataType {
    None,
    String,
    Number,
    Boolean,
    Color,
    List,
    Enum,
    Trigger,
    ViewModel,
    Integer,
    SymbolListIndex,
    AssetImage,
    Artboard,
}

impl fmt::Display for CommandDataType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::None => "None",
            Self::String => "String",
            Self::Number => "Number",
            Self::Boolean => "Boolean",
            Self::Color => "Color",
            Self::List => "List",
            Self::Enum => "Enum",
            Self::Trigger => "Trigger",
            Self::ViewModel => "View Model",
            Self::Integer => "Integer",
            Self::SymbolListIndex => "Symbol List Index",
            Self::AssetImage => "Asset Image",
            Self::Artboard => "Artboard",
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandValue {
    None,
    String(String),
    Number(f32),
    Boolean(bool),
    Color(u32),
    Enum(String),
    Trigger,
    ViewModel(ViewModelInstanceHandle),
    Image(Option<RenderImageHandle>),
    Artboard(Option<ArtboardHandle>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fit {
    Fill,
    Contain,
    Cover,
    FitWidth,
    FitHeight,
    None,
    ScaleDown,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Alignment {
    pub x: f32,
    pub y: f32,
}

impl Alignment {
    pub const CENTER: Self = Self { x: 0.0, y: 0.0 };
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
            alignment: Alignment::CENTER,
            screen_bounds: Vec2D::new(0.0, 0.0),
            position: Vec2D::new(0.0, 0.0),
            scale_factor: 1.0,
            pointer_id: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewModelEnum {
    pub name: String,
    pub enumerants: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewModelPropertyData {
    pub data_type: CommandDataType,
    pub name: String,
    pub metadata: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileAssetData {
    pub name: String,
    pub asset_id: u32,
    pub cdn_uuid: String,
    pub cdn_base_url: String,
    pub file_extension: String,
    pub type_id: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandEvent {
    FileLoaded {
        handle: FileHandle,
        request_id: u64,
    },
    FileDeleted {
        handle: FileHandle,
        request_id: u64,
    },
    FileError {
        handle: FileHandle,
        request_id: u64,
        error: String,
    },
    ArtboardInstantiated {
        file: FileHandle,
        handle: ArtboardHandle,
        request_id: u64,
    },
    ArtboardDeleted {
        handle: ArtboardHandle,
        request_id: u64,
    },
    ArtboardError {
        handle: ArtboardHandle,
        request_id: u64,
        error: String,
    },
    ArtboardSize {
        handle: ArtboardHandle,
        request_id: u64,
        width: f32,
        height: f32,
    },
    ArtboardVolume {
        handle: ArtboardHandle,
        request_id: u64,
        volume: f32,
    },
    StateMachineInstantiated {
        artboard: ArtboardHandle,
        handle: StateMachineHandle,
        request_id: u64,
    },
    StateMachineDeleted {
        handle: StateMachineHandle,
        request_id: u64,
    },
    StateMachineSettled {
        handle: StateMachineHandle,
        request_id: u64,
    },
    StateMachineError {
        handle: StateMachineHandle,
        request_id: u64,
        error: String,
    },
    ViewModelInstantiated {
        file: FileHandle,
        handle: ViewModelInstanceHandle,
        request_id: u64,
    },
    ViewModelDeleted {
        handle: ViewModelInstanceHandle,
        request_id: u64,
    },
    ViewModelError {
        handle: ViewModelInstanceHandle,
        request_id: u64,
        error: String,
    },
    ViewModelName {
        handle: ViewModelInstanceHandle,
        request_id: u64,
        name: String,
    },
    ViewModelInstanceName {
        handle: ViewModelInstanceHandle,
        request_id: u64,
        name: String,
    },
    ViewModelValue {
        handle: ViewModelInstanceHandle,
        request_id: u64,
        path: String,
        value: CommandValue,
    },
    ViewModelListSize {
        handle: ViewModelInstanceHandle,
        request_id: u64,
        path: String,
        size: usize,
    },
    ViewModelListCleared {
        handle: ViewModelInstanceHandle,
        request_id: u64,
        path: String,
    },
    ImageDecoded {
        handle: RenderImageHandle,
        request_id: u64,
    },
    ImageDeleted {
        handle: RenderImageHandle,
        request_id: u64,
    },
    ImageError {
        handle: RenderImageHandle,
        request_id: u64,
        error: String,
    },
    AudioDecoded {
        handle: AudioSourceHandle,
        request_id: u64,
    },
    AudioDeleted {
        handle: AudioSourceHandle,
        request_id: u64,
    },
    AudioError {
        handle: AudioSourceHandle,
        request_id: u64,
        error: String,
    },
    FontDecoded {
        handle: FontHandle,
        request_id: u64,
    },
    FontDeleted {
        handle: FontHandle,
        request_id: u64,
    },
    FontError {
        handle: FontHandle,
        request_id: u64,
        error: String,
    },
    ArtboardsListed {
        handle: FileHandle,
        request_id: u64,
        names: Vec<String>,
    },
    StateMachinesListed {
        handle: ArtboardHandle,
        request_id: u64,
        names: Vec<String>,
    },
    ViewModelsListed {
        handle: FileHandle,
        request_id: u64,
        names: Vec<String>,
    },
    GlobalViewModelsListed {
        handle: FileHandle,
        request_id: u64,
        names: Vec<String>,
    },
    ViewModelInstancesListed {
        handle: FileHandle,
        request_id: u64,
        view_model: String,
        names: Vec<String>,
    },
    ViewModelPropertiesListed {
        handle: FileHandle,
        request_id: u64,
        view_model: String,
        properties: Vec<ViewModelPropertyData>,
    },
    ViewModelEnumsListed {
        handle: FileHandle,
        request_id: u64,
        enums: Vec<ViewModelEnum>,
    },
    FileAssetsListed {
        handle: FileHandle,
        request_id: u64,
        assets: Vec<FileAssetData>,
    },
    DefaultViewModel {
        handle: ArtboardHandle,
        request_id: u64,
        view_model: String,
        instance: String,
    },
}

pub trait CommandListener: Send + Sync {
    fn on_event(&self, event: &CommandEvent);
}

impl<F> CommandListener for F
where
    F: Fn(&CommandEvent) + Send + Sync,
{
    fn on_event(&self, event: &CommandEvent) {
        self(event);
    }
}

pub type Listener = Arc<dyn CommandListener>;
type WeakListener = Weak<dyn CommandListener>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ListenerKey {
    File(FileHandle),
    Artboard(ArtboardHandle),
    StateMachine(StateMachineHandle),
    ViewModel(ViewModelInstanceHandle),
    Image(RenderImageHandle),
    Audio(AudioSourceHandle),
    Font(FontHandle),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum GlobalListenerKey {
    File,
    Artboard,
    StateMachine,
    ViewModel,
    Image,
    Audio,
    Font,
}

pub(crate) type ServerCallback = Box<dyn FnOnce(&mut CommandServer) + Send + 'static>;
pub(crate) type DrawCallback = Box<dyn FnOnce(DrawKey, &mut CommandServer) + Send + 'static>;

pub(crate) enum Command {
    LoadFile {
        handle: FileHandle,
        bytes: Vec<u8>,
        request_id: u64,
    },
    DeleteFile {
        handle: FileHandle,
        request_id: u64,
    },
    InstantiateArtboard {
        handle: ArtboardHandle,
        file: FileHandle,
        name: String,
        request_id: u64,
    },
    DeleteArtboard {
        handle: ArtboardHandle,
        request_id: u64,
    },
    SetArtboardSize {
        handle: ArtboardHandle,
        width: f32,
        height: f32,
        request_id: u64,
    },
    ResetArtboardSize {
        handle: ArtboardHandle,
        request_id: u64,
    },
    SetArtboardVolume {
        handle: ArtboardHandle,
        volume: f32,
        request_id: u64,
    },
    GetArtboardVolume {
        handle: ArtboardHandle,
        request_id: u64,
    },
    GetArtboardSize {
        handle: ArtboardHandle,
        request_id: u64,
    },
    InstantiateStateMachine {
        handle: StateMachineHandle,
        artboard: ArtboardHandle,
        name: String,
        request_id: u64,
    },
    DeleteStateMachine {
        handle: StateMachineHandle,
        request_id: u64,
    },
    AdvanceStateMachine {
        handle: StateMachineHandle,
        elapsed: f32,
        request_id: u64,
    },
    Pointer {
        handle: StateMachineHandle,
        kind: PointerKind,
        event: PointerEvent,
        request_id: u64,
    },
    InstantiateViewModel {
        handle: ViewModelInstanceHandle,
        file: FileHandle,
        source: ViewModelSource,
        instance_name: Option<String>,
        request_id: u64,
    },
    ReferenceNestedViewModel {
        root: ViewModelInstanceHandle,
        handle: ViewModelInstanceHandle,
        path: String,
        request_id: u64,
    },
    ReferenceListViewModel {
        root: ViewModelInstanceHandle,
        handle: ViewModelInstanceHandle,
        path: String,
        index: usize,
        request_id: u64,
    },
    DeleteViewModel {
        handle: ViewModelInstanceHandle,
        request_id: u64,
    },
    SetViewModelValue {
        handle: ViewModelInstanceHandle,
        path: String,
        value: CommandValue,
        request_id: u64,
    },
    InsertViewModelList {
        root: ViewModelInstanceHandle,
        path: String,
        value: ViewModelInstanceHandle,
        index: Option<usize>,
        request_id: u64,
    },
    RemoveViewModelList {
        root: ViewModelInstanceHandle,
        path: String,
        value: Option<ViewModelInstanceHandle>,
        index: Option<usize>,
        request_id: u64,
    },
    SwapViewModelList {
        root: ViewModelInstanceHandle,
        path: String,
        a: usize,
        b: usize,
        request_id: u64,
    },
    ClearViewModelList {
        handle: ViewModelInstanceHandle,
        path: String,
        request_id: u64,
    },
    GetViewModelListSize {
        handle: ViewModelInstanceHandle,
        path: String,
        request_id: u64,
    },
    GetViewModelValue {
        handle: ViewModelInstanceHandle,
        path: String,
        data_type: CommandDataType,
        request_id: u64,
    },
    SubscribeViewModelValue {
        handle: ViewModelInstanceHandle,
        path: String,
        data_type: CommandDataType,
        request_id: u64,
    },
    UnsubscribeViewModelValue {
        handle: ViewModelInstanceHandle,
        path: String,
        data_type: CommandDataType,
    },
    GetViewModelName {
        handle: ViewModelInstanceHandle,
        request_id: u64,
    },
    GetViewModelInstanceName {
        handle: ViewModelInstanceHandle,
        request_id: u64,
    },
    BindViewModel {
        state_machine: StateMachineHandle,
        view_model: ViewModelInstanceHandle,
        request_id: u64,
    },
    DecodeImage {
        handle: RenderImageHandle,
        bytes: Vec<u8>,
        request_id: u64,
    },
    ExternalImage {
        handle: RenderImageHandle,
        image: Box<dyn RenderImage + Send>,
        request_id: u64,
    },
    DeleteImage {
        handle: RenderImageHandle,
        request_id: u64,
    },
    DecodeAudio {
        handle: AudioSourceHandle,
        bytes: Vec<u8>,
        request_id: u64,
    },
    ExternalAudio {
        handle: AudioSourceHandle,
        audio: Arc<AudioSource>,
        request_id: u64,
    },
    DeleteAudio {
        handle: AudioSourceHandle,
        request_id: u64,
    },
    DecodeFont {
        handle: FontHandle,
        bytes: Vec<u8>,
        request_id: u64,
    },
    ExternalFont {
        handle: FontHandle,
        font: RawTextFont,
        request_id: u64,
    },
    DeleteFont {
        handle: FontHandle,
        request_id: u64,
    },
    AddGlobalImage {
        name: String,
        handle: RenderImageHandle,
    },
    RemoveGlobalImage {
        name: String,
    },
    AddGlobalAudio {
        name: String,
        handle: AudioSourceHandle,
    },
    RemoveGlobalAudio {
        name: String,
    },
    AddGlobalFont {
        name: String,
        handle: FontHandle,
    },
    RemoveGlobalFont {
        name: String,
    },
    ListArtboards {
        handle: FileHandle,
        request_id: u64,
    },
    ListStateMachines {
        handle: ArtboardHandle,
        request_id: u64,
    },
    ListViewModels {
        handle: FileHandle,
        request_id: u64,
    },
    ListGlobalViewModels {
        handle: FileHandle,
        request_id: u64,
    },
    ListViewModelInstances {
        handle: FileHandle,
        view_model: String,
        request_id: u64,
    },
    ListViewModelProperties {
        handle: FileHandle,
        view_model: String,
        request_id: u64,
    },
    ListViewModelEnums {
        handle: FileHandle,
        request_id: u64,
    },
    ListFileAssets {
        handle: FileHandle,
        request_id: u64,
    },
    GetDefaultViewModel {
        artboard: ArtboardHandle,
        file: FileHandle,
        request_id: u64,
    },
    RunOnce(ServerCallback),
    Draw {
        key: DrawKey,
        callback: DrawCallback,
    },
    CancelDraw(DrawKey),
    TestingCommandLoopBreak,
    Disconnect,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum PointerKind {
    Move,
    Down,
    Up,
    Exit,
}

#[derive(Debug, Clone)]
pub(crate) enum ViewModelSource {
    Artboard(ArtboardHandle),
    Named(String),
}

#[derive(Default)]
struct Counters {
    file: u64,
    artboard: u64,
    state_machine: u64,
    view_model: u64,
    image: u64,
    audio: u64,
    font: u64,
    draw: u64,
}

#[derive(Default)]
struct Listeners {
    by_handle: BTreeMap<ListenerKey, WeakListener>,
    global: BTreeMap<GlobalListenerKey, WeakListener>,
}

pub(crate) struct Shared {
    commands: Mutex<VecDeque<Command>>,
    messages: Mutex<VecDeque<CommandEvent>>,
    wake: Condvar,
}

#[derive(Clone)]
pub struct CommandQueue {
    pub(crate) shared: Arc<Shared>,
    counters: Arc<Mutex<Counters>>,
    listeners: Arc<Mutex<Listeners>>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(value) => value,
        Err(poisoned) => poisoned.into_inner(),
    }
}

impl Default for CommandQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandQueue {
    pub fn new() -> Self {
        Self {
            shared: Arc::new(Shared {
                commands: Mutex::new(VecDeque::new()),
                messages: Mutex::new(VecDeque::new()),
                wake: Condvar::new(),
            }),
            counters: Arc::new(Mutex::new(Counters::default())),
            listeners: Arc::new(Mutex::new(Listeners::default())),
        }
    }

    fn enqueue(&self, command: Command) {
        lock(&self.shared.commands).push_back(command);
        self.shared.wake.notify_one();
    }
    fn next<T>(
        &self,
        select: impl FnOnce(&mut Counters) -> &mut u64,
        wrap: impl FnOnce(u64) -> T,
    ) -> T {
        let mut counters = lock(&self.counters);
        let value = select(&mut counters);
        *value = value.wrapping_add(1);
        wrap(*value)
    }
    fn register(&self, key: ListenerKey, listener: Option<&Listener>) {
        if let Some(listener) = listener {
            lock(&self.listeners)
                .by_handle
                .insert(key, Arc::downgrade(listener));
        }
    }
    fn set_global(&self, key: GlobalListenerKey, listener: Option<&Listener>) {
        let mut listeners = lock(&self.listeners);
        if let Some(listener) = listener {
            listeners.global.insert(key, Arc::downgrade(listener));
        } else {
            listeners.global.remove(&key);
        }
    }
    pub fn set_global_file_listener(&self, listener: Option<&Listener>) {
        self.set_global(GlobalListenerKey::File, listener);
    }
    pub fn set_global_artboard_listener(&self, listener: Option<&Listener>) {
        self.set_global(GlobalListenerKey::Artboard, listener);
    }
    pub fn set_global_state_machine_listener(&self, listener: Option<&Listener>) {
        self.set_global(GlobalListenerKey::StateMachine, listener);
    }
    pub fn set_global_view_model_listener(&self, listener: Option<&Listener>) {
        self.set_global(GlobalListenerKey::ViewModel, listener);
    }
    pub fn set_global_image_listener(&self, listener: Option<&Listener>) {
        self.set_global(GlobalListenerKey::Image, listener);
    }
    pub fn set_global_audio_listener(&self, listener: Option<&Listener>) {
        self.set_global(GlobalListenerKey::Audio, listener);
    }
    pub fn set_global_font_listener(&self, listener: Option<&Listener>) {
        self.set_global(GlobalListenerKey::Font, listener);
    }

    pub fn load_file(
        &self,
        bytes: Vec<u8>,
        listener: Option<&Listener>,
        request_id: u64,
    ) -> FileHandle {
        let h = self.next(|c| &mut c.file, FileHandle);
        self.register(ListenerKey::File(h), listener);
        self.enqueue(Command::LoadFile {
            handle: h,
            bytes,
            request_id,
        });
        h
    }
    pub fn delete_file(&self, handle: FileHandle, request_id: u64) {
        self.enqueue(Command::DeleteFile { handle, request_id });
    }
    pub fn instantiate_artboard_named(
        &self,
        file: FileHandle,
        name: impl Into<String>,
        listener: Option<&Listener>,
        request_id: u64,
    ) -> ArtboardHandle {
        let h = self.next(|c| &mut c.artboard, ArtboardHandle);
        self.register(ListenerKey::Artboard(h), listener);
        self.enqueue(Command::InstantiateArtboard {
            handle: h,
            file,
            name: name.into(),
            request_id,
        });
        h
    }
    pub fn instantiate_default_artboard(
        &self,
        file: FileHandle,
        listener: Option<&Listener>,
        request_id: u64,
    ) -> ArtboardHandle {
        self.instantiate_artboard_named(file, "", listener, request_id)
    }
    pub fn delete_artboard(&self, handle: ArtboardHandle, request_id: u64) {
        self.enqueue(Command::DeleteArtboard { handle, request_id });
    }
    pub fn set_artboard_size(
        &self,
        handle: ArtboardHandle,
        width: f32,
        height: f32,
        scale: f32,
        request_id: u64,
    ) {
        self.enqueue(Command::SetArtboardSize {
            handle,
            width: width / scale,
            height: height / scale,
            request_id,
        });
    }
    pub fn reset_artboard_size(&self, handle: ArtboardHandle, request_id: u64) {
        self.enqueue(Command::ResetArtboardSize { handle, request_id });
    }
    pub fn set_artboard_volume(&self, handle: ArtboardHandle, volume: f32, request_id: u64) {
        self.enqueue(Command::SetArtboardVolume {
            handle,
            volume,
            request_id,
        });
    }
    pub fn request_artboard_volume(&self, handle: ArtboardHandle, request_id: u64) {
        self.enqueue(Command::GetArtboardVolume { handle, request_id });
    }
    pub fn request_artboard_size(&self, handle: ArtboardHandle, request_id: u64) {
        self.enqueue(Command::GetArtboardSize { handle, request_id });
    }
    pub fn instantiate_state_machine_named(
        &self,
        artboard: ArtboardHandle,
        name: impl Into<String>,
        listener: Option<&Listener>,
        request_id: u64,
    ) -> StateMachineHandle {
        let h = self.next(|c| &mut c.state_machine, StateMachineHandle);
        self.register(ListenerKey::StateMachine(h), listener);
        self.enqueue(Command::InstantiateStateMachine {
            handle: h,
            artboard,
            name: name.into(),
            request_id,
        });
        h
    }
    pub fn instantiate_default_state_machine(
        &self,
        artboard: ArtboardHandle,
        listener: Option<&Listener>,
        request_id: u64,
    ) -> StateMachineHandle {
        self.instantiate_state_machine_named(artboard, "", listener, request_id)
    }
    pub fn delete_state_machine(&self, handle: StateMachineHandle, request_id: u64) {
        self.enqueue(Command::DeleteStateMachine { handle, request_id });
    }
    pub fn advance_state_machine(&self, handle: StateMachineHandle, elapsed: f32, request_id: u64) {
        self.enqueue(Command::AdvanceStateMachine {
            handle,
            elapsed,
            request_id,
        });
    }
    fn pointer(
        &self,
        handle: StateMachineHandle,
        kind: PointerKind,
        event: PointerEvent,
        request_id: u64,
    ) {
        self.enqueue(Command::Pointer {
            handle,
            kind,
            event,
            request_id,
        });
    }
    pub fn pointer_move(&self, h: StateMachineHandle, e: PointerEvent, r: u64) {
        self.pointer(h, PointerKind::Move, e, r)
    }
    pub fn pointer_down(&self, h: StateMachineHandle, e: PointerEvent, r: u64) {
        self.pointer(h, PointerKind::Down, e, r)
    }
    pub fn pointer_up(&self, h: StateMachineHandle, e: PointerEvent, r: u64) {
        self.pointer(h, PointerKind::Up, e, r)
    }
    pub fn pointer_exit(&self, h: StateMachineHandle, e: PointerEvent, r: u64) {
        self.pointer(h, PointerKind::Exit, e, r)
    }

    fn view_model(
        &self,
        file: FileHandle,
        source: ViewModelSource,
        instance_name: Option<String>,
        listener: Option<&Listener>,
        request_id: u64,
    ) -> ViewModelInstanceHandle {
        let h = self.next(|c| &mut c.view_model, ViewModelInstanceHandle);
        self.register(ListenerKey::ViewModel(h), listener);
        self.enqueue(Command::InstantiateViewModel {
            handle: h,
            file,
            source,
            instance_name,
            request_id,
        });
        h
    }
    pub fn instantiate_blank_view_model_named(
        &self,
        file: FileHandle,
        name: impl Into<String>,
        listener: Option<&Listener>,
        request_id: u64,
    ) -> ViewModelInstanceHandle {
        self.view_model(
            file,
            ViewModelSource::Named(name.into()),
            None,
            listener,
            request_id,
        )
    }
    pub fn instantiate_view_model_named(
        &self,
        file: FileHandle,
        name: impl Into<String>,
        instance: impl Into<String>,
        listener: Option<&Listener>,
        request_id: u64,
    ) -> ViewModelInstanceHandle {
        self.view_model(
            file,
            ViewModelSource::Named(name.into()),
            Some(instance.into()),
            listener,
            request_id,
        )
    }
    pub fn instantiate_view_model_for_artboard(
        &self,
        file: FileHandle,
        artboard: ArtboardHandle,
        instance: Option<String>,
        listener: Option<&Listener>,
        request_id: u64,
    ) -> ViewModelInstanceHandle {
        self.view_model(
            file,
            ViewModelSource::Artboard(artboard),
            instance,
            listener,
            request_id,
        )
    }
    pub fn reference_nested_view_model(
        &self,
        root: ViewModelInstanceHandle,
        path: impl Into<String>,
        listener: Option<&Listener>,
        request_id: u64,
    ) -> ViewModelInstanceHandle {
        let h = self.next(|c| &mut c.view_model, ViewModelInstanceHandle);
        self.register(ListenerKey::ViewModel(h), listener);
        self.enqueue(Command::ReferenceNestedViewModel {
            root,
            handle: h,
            path: path.into(),
            request_id,
        });
        h
    }
    pub fn reference_list_view_model(
        &self,
        root: ViewModelInstanceHandle,
        path: impl Into<String>,
        index: usize,
        listener: Option<&Listener>,
        request_id: u64,
    ) -> ViewModelInstanceHandle {
        let h = self.next(|c| &mut c.view_model, ViewModelInstanceHandle);
        self.register(ListenerKey::ViewModel(h), listener);
        self.enqueue(Command::ReferenceListViewModel {
            root,
            handle: h,
            path: path.into(),
            index,
            request_id,
        });
        h
    }
    pub fn delete_view_model(&self, handle: ViewModelInstanceHandle, request_id: u64) {
        self.enqueue(Command::DeleteViewModel { handle, request_id });
    }
    pub fn set_view_model_value(
        &self,
        handle: ViewModelInstanceHandle,
        path: impl Into<String>,
        value: CommandValue,
        request_id: u64,
    ) {
        self.enqueue(Command::SetViewModelValue {
            handle,
            path: path.into(),
            value,
            request_id,
        });
    }
    pub fn insert_view_model_list(
        &self,
        root: ViewModelInstanceHandle,
        path: impl Into<String>,
        value: ViewModelInstanceHandle,
        index: Option<usize>,
        request_id: u64,
    ) {
        self.enqueue(Command::InsertViewModelList {
            root,
            path: path.into(),
            value,
            index,
            request_id,
        });
    }
    pub fn remove_view_model_list(
        &self,
        root: ViewModelInstanceHandle,
        path: impl Into<String>,
        value: Option<ViewModelInstanceHandle>,
        index: Option<usize>,
        request_id: u64,
    ) {
        self.enqueue(Command::RemoveViewModelList {
            root,
            path: path.into(),
            value,
            index,
            request_id,
        });
    }
    pub fn swap_view_model_list(
        &self,
        root: ViewModelInstanceHandle,
        path: impl Into<String>,
        a: usize,
        b: usize,
        request_id: u64,
    ) {
        self.enqueue(Command::SwapViewModelList {
            root,
            path: path.into(),
            a,
            b,
            request_id,
        });
    }
    pub fn request_view_model_list_clear(
        &self,
        handle: ViewModelInstanceHandle,
        path: impl Into<String>,
        request_id: u64,
    ) {
        self.enqueue(Command::ClearViewModelList {
            handle,
            path: path.into(),
            request_id,
        });
    }
    pub fn request_view_model_list_size(
        &self,
        handle: ViewModelInstanceHandle,
        path: impl Into<String>,
        request_id: u64,
    ) {
        self.enqueue(Command::GetViewModelListSize {
            handle,
            path: path.into(),
            request_id,
        });
    }
    pub fn request_view_model_value(
        &self,
        handle: ViewModelInstanceHandle,
        path: impl Into<String>,
        data_type: CommandDataType,
        request_id: u64,
    ) {
        self.enqueue(Command::GetViewModelValue {
            handle,
            path: path.into(),
            data_type,
            request_id,
        });
    }
    pub fn subscribe_to_view_model_property(
        &self,
        handle: ViewModelInstanceHandle,
        path: impl Into<String>,
        data_type: CommandDataType,
        request_id: u64,
    ) {
        self.enqueue(Command::SubscribeViewModelValue {
            handle,
            path: path.into(),
            data_type,
            request_id,
        });
    }
    pub fn unsubscribe_from_view_model_property(
        &self,
        handle: ViewModelInstanceHandle,
        path: impl Into<String>,
        data_type: CommandDataType,
    ) {
        self.enqueue(Command::UnsubscribeViewModelValue {
            handle,
            path: path.into(),
            data_type,
        });
    }
    pub fn request_view_model_name(&self, handle: ViewModelInstanceHandle, request_id: u64) {
        self.enqueue(Command::GetViewModelName { handle, request_id });
    }
    pub fn request_view_model_instance_name(
        &self,
        handle: ViewModelInstanceHandle,
        request_id: u64,
    ) {
        self.enqueue(Command::GetViewModelInstanceName { handle, request_id });
    }
    pub fn bind_view_model(
        &self,
        state_machine: StateMachineHandle,
        view_model: ViewModelInstanceHandle,
        request_id: u64,
    ) {
        self.enqueue(Command::BindViewModel {
            state_machine,
            view_model,
            request_id,
        });
    }

    pub fn decode_image(
        &self,
        bytes: Vec<u8>,
        listener: Option<&Listener>,
        request_id: u64,
    ) -> RenderImageHandle {
        let h = self.next(|c| &mut c.image, RenderImageHandle);
        self.register(ListenerKey::Image(h), listener);
        self.enqueue(Command::DecodeImage {
            handle: h,
            bytes,
            request_id,
        });
        h
    }
    pub fn add_external_image(
        &self,
        image: Box<dyn RenderImage + Send>,
        listener: Option<&Listener>,
        request_id: u64,
    ) -> RenderImageHandle {
        let h = self.next(|c| &mut c.image, RenderImageHandle);
        self.register(ListenerKey::Image(h), listener);
        self.enqueue(Command::ExternalImage {
            handle: h,
            image,
            request_id,
        });
        h
    }
    pub fn delete_image(&self, handle: RenderImageHandle, request_id: u64) {
        self.enqueue(Command::DeleteImage { handle, request_id });
    }
    pub fn decode_audio(
        &self,
        bytes: Vec<u8>,
        listener: Option<&Listener>,
        request_id: u64,
    ) -> AudioSourceHandle {
        let h = self.next(|c| &mut c.audio, AudioSourceHandle);
        self.register(ListenerKey::Audio(h), listener);
        self.enqueue(Command::DecodeAudio {
            handle: h,
            bytes,
            request_id,
        });
        h
    }
    pub fn add_external_audio(
        &self,
        audio: Arc<AudioSource>,
        listener: Option<&Listener>,
        request_id: u64,
    ) -> AudioSourceHandle {
        let h = self.next(|c| &mut c.audio, AudioSourceHandle);
        self.register(ListenerKey::Audio(h), listener);
        self.enqueue(Command::ExternalAudio {
            handle: h,
            audio,
            request_id,
        });
        h
    }
    pub fn delete_audio(&self, handle: AudioSourceHandle, request_id: u64) {
        self.enqueue(Command::DeleteAudio { handle, request_id });
    }
    pub fn decode_font(
        &self,
        bytes: Vec<u8>,
        listener: Option<&Listener>,
        request_id: u64,
    ) -> FontHandle {
        let h = self.next(|c| &mut c.font, FontHandle);
        self.register(ListenerKey::Font(h), listener);
        self.enqueue(Command::DecodeFont {
            handle: h,
            bytes,
            request_id,
        });
        h
    }
    pub fn add_external_font(
        &self,
        font: RawTextFont,
        listener: Option<&Listener>,
        request_id: u64,
    ) -> FontHandle {
        let h = self.next(|c| &mut c.font, FontHandle);
        self.register(ListenerKey::Font(h), listener);
        self.enqueue(Command::ExternalFont {
            handle: h,
            font,
            request_id,
        });
        h
    }
    pub fn delete_font(&self, handle: FontHandle, request_id: u64) {
        self.enqueue(Command::DeleteFont { handle, request_id });
    }
    pub fn add_global_image_asset(&self, name: impl Into<String>, handle: RenderImageHandle) {
        self.enqueue(Command::AddGlobalImage {
            name: name.into(),
            handle,
        });
    }
    pub fn remove_global_image_asset(&self, name: impl Into<String>) {
        self.enqueue(Command::RemoveGlobalImage { name: name.into() });
    }
    pub fn add_global_audio_asset(&self, name: impl Into<String>, handle: AudioSourceHandle) {
        self.enqueue(Command::AddGlobalAudio {
            name: name.into(),
            handle,
        });
    }
    pub fn remove_global_audio_asset(&self, name: impl Into<String>) {
        self.enqueue(Command::RemoveGlobalAudio { name: name.into() });
    }
    pub fn add_global_font_asset(&self, name: impl Into<String>, handle: FontHandle) {
        self.enqueue(Command::AddGlobalFont {
            name: name.into(),
            handle,
        });
    }
    pub fn remove_global_font_asset(&self, name: impl Into<String>) {
        self.enqueue(Command::RemoveGlobalFont { name: name.into() });
    }
    pub fn request_artboard_names(&self, handle: FileHandle, request_id: u64) {
        self.enqueue(Command::ListArtboards { handle, request_id });
    }
    pub fn request_state_machine_names(&self, handle: ArtboardHandle, request_id: u64) {
        self.enqueue(Command::ListStateMachines { handle, request_id });
    }
    pub fn request_view_model_names(&self, handle: FileHandle, request_id: u64) {
        self.enqueue(Command::ListViewModels { handle, request_id });
    }
    pub fn request_global_view_model_names(&self, handle: FileHandle, request_id: u64) {
        self.enqueue(Command::ListGlobalViewModels { handle, request_id });
    }
    pub fn request_view_model_instance_names(
        &self,
        handle: FileHandle,
        view_model: impl Into<String>,
        request_id: u64,
    ) {
        self.enqueue(Command::ListViewModelInstances {
            handle,
            view_model: view_model.into(),
            request_id,
        });
    }
    pub fn request_view_model_properties(
        &self,
        handle: FileHandle,
        view_model: impl Into<String>,
        request_id: u64,
    ) {
        self.enqueue(Command::ListViewModelProperties {
            handle,
            view_model: view_model.into(),
            request_id,
        });
    }
    pub fn request_view_model_enums(&self, handle: FileHandle, request_id: u64) {
        self.enqueue(Command::ListViewModelEnums { handle, request_id });
    }
    pub fn request_file_assets(&self, handle: FileHandle, request_id: u64) {
        self.enqueue(Command::ListFileAssets { handle, request_id });
    }
    pub fn request_default_view_model(
        &self,
        artboard: ArtboardHandle,
        file: FileHandle,
        request_id: u64,
    ) {
        self.enqueue(Command::GetDefaultViewModel {
            artboard,
            file,
            request_id,
        });
    }
    pub fn create_draw_key(&self) -> DrawKey {
        self.next(|c| &mut c.draw, DrawKey)
    }
    pub fn run_once(&self, callback: impl FnOnce(&mut CommandServer) + Send + 'static) {
        self.enqueue(Command::RunOnce(Box::new(callback)));
    }
    pub fn draw(
        &self,
        key: DrawKey,
        callback: impl FnOnce(DrawKey, &mut CommandServer) + Send + 'static,
    ) {
        self.enqueue(Command::Draw {
            key,
            callback: Box::new(callback),
        });
    }
    pub fn cancel_draw(&self, key: DrawKey) {
        self.enqueue(Command::CancelDraw(key));
    }
    /// Test-only protocol marker matching the pinned queue's command-loop
    /// break. Commands after this marker remain queued for the next poll.
    #[doc(hidden)]
    pub fn testing_command_loop_break(&self) {
        self.enqueue(Command::TestingCommandLoopBreak);
    }
    pub fn disconnect(&self) {
        self.enqueue(Command::Disconnect);
    }

    pub fn process_messages(&self) -> usize {
        let events = {
            let mut messages = lock(&self.shared.messages);
            let count = messages.len();
            messages.drain(..count).collect::<Vec<_>>()
        };
        for event in &events {
            let (global, local) = {
                let listeners = lock(&self.listeners);
                let key = event.listener_key();
                (
                    key.map(ListenerKey::global_key)
                        .and_then(|key| listeners.global.get(&key))
                        .and_then(Weak::upgrade),
                    key.and_then(|key| listeners.by_handle.get(&key))
                        .and_then(Weak::upgrade),
                )
            };
            if let Some(listener) = global {
                listener.on_event(event);
            }
            if let Some(listener) = local {
                listener.on_event(event);
            }
        }
        events.len()
    }
}

impl CommandEvent {
    fn listener_key(&self) -> Option<ListenerKey> {
        use CommandEvent::*;
        Some(match self {
            FileLoaded { handle, .. }
            | FileDeleted { handle, .. }
            | FileError { handle, .. }
            | ArtboardsListed { handle, .. }
            | ViewModelsListed { handle, .. }
            | GlobalViewModelsListed { handle, .. }
            | ViewModelInstancesListed { handle, .. }
            | ViewModelPropertiesListed { handle, .. }
            | ViewModelEnumsListed { handle, .. }
            | FileAssetsListed { handle, .. } => ListenerKey::File(*handle),
            ArtboardInstantiated { file, .. } => ListenerKey::File(*file),
            ArtboardDeleted { handle, .. }
            | ArtboardError { handle, .. }
            | ArtboardSize { handle, .. }
            | ArtboardVolume { handle, .. }
            | StateMachinesListed { handle, .. }
            | DefaultViewModel { handle, .. } => ListenerKey::Artboard(*handle),
            StateMachineInstantiated { artboard, .. } => ListenerKey::Artboard(*artboard),
            StateMachineDeleted { handle, .. }
            | StateMachineSettled { handle, .. }
            | StateMachineError { handle, .. } => ListenerKey::StateMachine(*handle),
            ViewModelInstantiated { file, .. } => ListenerKey::File(*file),
            ViewModelDeleted { handle, .. }
            | ViewModelError { handle, .. }
            | ViewModelName { handle, .. }
            | ViewModelInstanceName { handle, .. }
            | ViewModelValue { handle, .. }
            | ViewModelListSize { handle, .. }
            | ViewModelListCleared { handle, .. } => ListenerKey::ViewModel(*handle),
            ImageDecoded { handle, .. }
            | ImageDeleted { handle, .. }
            | ImageError { handle, .. } => ListenerKey::Image(*handle),
            AudioDecoded { handle, .. }
            | AudioDeleted { handle, .. }
            | AudioError { handle, .. } => ListenerKey::Audio(*handle),
            FontDecoded { handle, .. } | FontDeleted { handle, .. } | FontError { handle, .. } => {
                ListenerKey::Font(*handle)
            }
        })
    }
}

impl ListenerKey {
    const fn global_key(self) -> GlobalListenerKey {
        match self {
            Self::File(_) => GlobalListenerKey::File,
            Self::Artboard(_) => GlobalListenerKey::Artboard,
            Self::StateMachine(_) => GlobalListenerKey::StateMachine,
            Self::ViewModel(_) => GlobalListenerKey::ViewModel,
            Self::Image(_) => GlobalListenerKey::Image,
            Self::Audio(_) => GlobalListenerKey::Audio,
            Self::Font(_) => GlobalListenerKey::Font,
        }
    }
}

impl Shared {
    pub(crate) fn prepend_commands(&self, commands: impl DoubleEndedIterator<Item = Command>) {
        let mut pending = lock(&self.commands);
        for command in commands.rev() {
            pending.push_front(command);
        }
    }

    pub(crate) fn take_commands(&self, wait: bool) -> Vec<Command> {
        let mut commands = lock(&self.commands);
        if wait {
            while commands.is_empty() {
                commands = match self.wake.wait(commands) {
                    Ok(v) => v,
                    Err(p) => p.into_inner(),
                };
            }
        }
        let count = commands.len();
        commands.drain(..count).collect()
    }
    pub(crate) fn push_event(&self, event: CommandEvent) {
        lock(&self.messages).push_back(event);
    }
}
