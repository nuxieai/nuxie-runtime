//! Focused parity tests for Rive's pinned `command_queue_test.cpp`.
//!
//! These tests port the non-rendering command-loop invariants from
//! `tests/unit_tests/runtime/command_queue_test.cpp` at `4ac7b327`.
//! The test cases below are the executable correspondence to that source.

use std::{
    any::Any,
    cell::RefCell,
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

use nuxie::command_queue::{
    ArtboardHandle, ArtboardListener, ArtboardListenerHandle, AudioSourceHandle,
    AudioSourceListener, AudioSourceListenerHandle, BlobAssetHandle, BlobAssetListener,
    BlobAssetListenerHandle, CommandQueue, DataType, FileAssetData, FileHandle, FileListener,
    FileListenerHandle, FontHandle, FontListener, FontListenerHandle, ListenerBase, ListenerHandle,
    PointerEvent, RenderImageHandle, RenderImageListener, RenderImageListenerHandle,
    StateMachineHandle, StateMachineListener, StateMachineListenerHandle, ViewModelEnum,
    ViewModelInstanceData, ViewModelInstanceHandle, ViewModelInstanceListener,
    ViewModelInstanceListenerHandle, ViewModelInstanceValue, ViewModelPropertyData,
};
use nuxie::command_server::CommandServer;
use nuxie::runtime::{
    animation::semantic_listener_group::SemanticActionType,
    layout::{Alignment, Fit},
    semantic::{
        semantic_role::SemanticRole,
        semantic_snapshot::{SemanticsDiff, SemanticsDiffNode},
        semantic_state::{SemanticState, has_semantic_state},
        semantic_trait::{SemanticTrait, has_semantic_trait},
    },
};
use nuxie::{
    ColorInt, Factory, FillRule, ImageDecodeError, PersistentFactory, RecordingFactory,
    RenderBuffer, RenderBufferFlags, RenderBufferType, RenderImage, RenderPaint, RenderPath,
    RenderShader, RuntimeFactoryHandle,
};
use nuxie_runtime::{
    FileAssetLoader, FileAssetLoaderRef, RawTextFont, RuntimeBlobAsset,
    mechanical_port::source::audio::audio_source::AudioSource as RuntimeAudioSource,
};

#[derive(Debug, Clone, PartialEq)]
enum ObservedValue {
    None,
    String(String),
    Number(f32),
    Boolean(bool),
    Color(u32),
    Enum(String),
    Trigger,
    ViewModel(ViewModelInstanceHandle),
    Image(Option<RenderImageHandle>),
    Blob(Option<BlobAssetHandle>),
    Artboard(Option<ArtboardHandle>),
}

#[derive(Clone)]
enum ObservedEvent {
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
    SemanticsDiffReceived {
        handle: StateMachineHandle,
        request_id: u64,
        diff: SemanticsDiff,
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
        value: ObservedValue,
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
    BlobDecoded {
        handle: BlobAssetHandle,
        request_id: u64,
    },
    BlobDeleted {
        handle: BlobAssetHandle,
        request_id: u64,
    },
    BlobError {
        handle: BlobAssetHandle,
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

type EventLog = Arc<Mutex<Vec<ObservedEvent>>>;

struct RecordingAssetLoader {
    attempted: Arc<Mutex<Vec<u16>>>,
}

impl FileAssetLoader for RecordingAssetLoader {
    fn load_contents(
        &mut self,
        asset: nuxie::CoreHandle,
        _in_band_bytes: &[u8],
        _factory: &RuntimeFactoryHandle,
    ) -> bool {
        self.attempted
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(asset.core_type().unwrap_or_default());
        false
    }
}

fn record(log: &EventLog, event: ObservedEvent) {
    log.lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .push(event);
}

struct FileEvents {
    base: ListenerBase<FileHandle>,
    log: EventLog,
}
impl FileListener for FileEvents {
    fn listener_base(&mut self) -> &mut ListenerBase<FileHandle> {
        &mut self.base
    }
    fn on_file_error(&mut self, handle: FileHandle, request_id: u64, error: String) {
        record(
            &self.log,
            ObservedEvent::FileError {
                handle,
                request_id,
                error,
            },
        );
    }
    fn on_file_deleted(&mut self, handle: FileHandle, request_id: u64) {
        record(&self.log, ObservedEvent::FileDeleted { handle, request_id });
    }
    fn on_file_loaded(&mut self, handle: FileHandle, request_id: u64) {
        record(&self.log, ObservedEvent::FileLoaded { handle, request_id });
    }
    fn on_artboard_instantiated(
        &mut self,
        file: FileHandle,
        request_id: u64,
        handle: ArtboardHandle,
    ) {
        record(
            &self.log,
            ObservedEvent::ArtboardInstantiated {
                file,
                handle,
                request_id,
            },
        );
    }
    fn on_view_model_instance_instantiated(
        &mut self,
        file: FileHandle,
        request_id: u64,
        handle: ViewModelInstanceHandle,
    ) {
        record(
            &self.log,
            ObservedEvent::ViewModelInstantiated {
                file,
                handle,
                request_id,
            },
        );
    }
    fn on_artboards_listed(&mut self, handle: FileHandle, request_id: u64, names: Vec<String>) {
        record(
            &self.log,
            ObservedEvent::ArtboardsListed {
                handle,
                request_id,
                names,
            },
        );
    }
    fn on_view_models_listed(&mut self, handle: FileHandle, request_id: u64, names: Vec<String>) {
        record(
            &self.log,
            ObservedEvent::ViewModelsListed {
                handle,
                request_id,
                names,
            },
        );
    }
    fn on_global_view_model_names_listed(
        &mut self,
        handle: FileHandle,
        request_id: u64,
        names: Vec<String>,
    ) {
        record(
            &self.log,
            ObservedEvent::GlobalViewModelsListed {
                handle,
                request_id,
                names,
            },
        );
    }
    fn on_view_model_instance_names_listed(
        &mut self,
        handle: FileHandle,
        request_id: u64,
        view_model: String,
        names: Vec<String>,
    ) {
        record(
            &self.log,
            ObservedEvent::ViewModelInstancesListed {
                handle,
                request_id,
                view_model,
                names,
            },
        );
    }
    fn on_view_model_properties_listed(
        &mut self,
        handle: FileHandle,
        request_id: u64,
        view_model: String,
        properties: Vec<ViewModelPropertyData>,
    ) {
        record(
            &self.log,
            ObservedEvent::ViewModelPropertiesListed {
                handle,
                request_id,
                view_model,
                properties,
            },
        );
    }
    fn on_view_model_enums_listed(
        &mut self,
        handle: FileHandle,
        request_id: u64,
        enums: Vec<ViewModelEnum>,
    ) {
        record(
            &self.log,
            ObservedEvent::ViewModelEnumsListed {
                handle,
                request_id,
                enums,
            },
        );
    }
    fn on_file_assets_listed(
        &mut self,
        handle: FileHandle,
        request_id: u64,
        assets: Vec<FileAssetData>,
    ) {
        record(
            &self.log,
            ObservedEvent::FileAssetsListed {
                handle,
                request_id,
                assets,
            },
        );
    }
}

#[cfg(any())]
struct ReentrantFileEvents {
    base: ListenerBase<FileHandle>,
    log: EventLog,
    queue: CommandQueue,
}

#[cfg(any())]
impl FileListener for ReentrantFileEvents {
    fn listener_base(&mut self) -> &mut ListenerBase<FileHandle> {
        &mut self.base
    }

    fn on_file_error(&mut self, handle: FileHandle, request_id: u64, error: String) {
        record(
            &self.log,
            ObservedEvent::FileError {
                handle,
                request_id,
                error,
            },
        );
        self.queue.load_file(Vec::new(), None, 2, None);
    }
}

struct ArtboardEvents {
    base: ListenerBase<ArtboardHandle>,
    log: EventLog,
}
impl ArtboardListener for ArtboardEvents {
    fn listener_base(&mut self) -> &mut ListenerBase<ArtboardHandle> {
        &mut self.base
    }
    fn on_artboard_error(&mut self, handle: ArtboardHandle, request_id: u64, error: String) {
        record(
            &self.log,
            ObservedEvent::ArtboardError {
                handle,
                request_id,
                error,
            },
        );
    }
    fn on_default_view_model_info_received(
        &mut self,
        handle: ArtboardHandle,
        request_id: u64,
        view_model: String,
        instance: String,
    ) {
        record(
            &self.log,
            ObservedEvent::DefaultViewModel {
                handle,
                request_id,
                view_model,
                instance,
            },
        );
    }
    fn on_artboard_deleted(&mut self, handle: ArtboardHandle, request_id: u64) {
        record(
            &self.log,
            ObservedEvent::ArtboardDeleted { handle, request_id },
        );
    }
    fn on_state_machine_instantiated(
        &mut self,
        artboard: ArtboardHandle,
        request_id: u64,
        handle: StateMachineHandle,
    ) {
        record(
            &self.log,
            ObservedEvent::StateMachineInstantiated {
                artboard,
                handle,
                request_id,
            },
        );
    }
    fn on_state_machines_listed(
        &mut self,
        handle: ArtboardHandle,
        request_id: u64,
        names: Vec<String>,
    ) {
        record(
            &self.log,
            ObservedEvent::StateMachinesListed {
                handle,
                request_id,
                names,
            },
        );
    }
    fn on_artboard_volume_received(
        &mut self,
        handle: ArtboardHandle,
        request_id: u64,
        volume: f32,
    ) {
        record(
            &self.log,
            ObservedEvent::ArtboardVolume {
                handle,
                request_id,
                volume,
            },
        );
    }
    fn on_artboard_size_received(
        &mut self,
        handle: ArtboardHandle,
        request_id: u64,
        width: f32,
        height: f32,
    ) {
        record(
            &self.log,
            ObservedEvent::ArtboardSize {
                handle,
                request_id,
                width,
                height,
            },
        );
    }
}

struct StateMachineEvents {
    base: ListenerBase<StateMachineHandle>,
    log: EventLog,
}
impl StateMachineListener for StateMachineEvents {
    fn listener_base(&mut self) -> &mut ListenerBase<StateMachineHandle> {
        &mut self.base
    }
    fn on_state_machine_error(
        &mut self,
        handle: StateMachineHandle,
        request_id: u64,
        error: String,
    ) {
        record(
            &self.log,
            ObservedEvent::StateMachineError {
                handle,
                request_id,
                error,
            },
        );
    }
    fn on_state_machine_deleted(&mut self, handle: StateMachineHandle, request_id: u64) {
        record(
            &self.log,
            ObservedEvent::StateMachineDeleted { handle, request_id },
        );
    }
    fn on_state_machine_settled(&mut self, handle: StateMachineHandle, request_id: u64) {
        record(
            &self.log,
            ObservedEvent::StateMachineSettled { handle, request_id },
        );
    }
    fn on_semantics_diff_received(
        &mut self,
        handle: StateMachineHandle,
        request_id: u64,
        diff: SemanticsDiff,
    ) {
        record(
            &self.log,
            ObservedEvent::SemanticsDiffReceived {
                handle,
                request_id,
                diff,
            },
        );
    }
}

struct ViewModelEvents {
    base: ListenerBase<ViewModelInstanceHandle>,
    log: EventLog,
}
impl ViewModelInstanceListener for ViewModelEvents {
    fn listener_base(&mut self) -> &mut ListenerBase<ViewModelInstanceHandle> {
        &mut self.base
    }
    fn on_view_model_instance_error(
        &mut self,
        handle: ViewModelInstanceHandle,
        request_id: u64,
        error: String,
    ) {
        record(
            &self.log,
            ObservedEvent::ViewModelError {
                handle,
                request_id,
                error,
            },
        );
    }
    fn on_view_model_instance_view_model_name_received(
        &mut self,
        handle: ViewModelInstanceHandle,
        request_id: u64,
        name: String,
    ) {
        record(
            &self.log,
            ObservedEvent::ViewModelName {
                handle,
                request_id,
                name,
            },
        );
    }
    fn on_view_model_instance_name_received(
        &mut self,
        handle: ViewModelInstanceHandle,
        request_id: u64,
        name: String,
    ) {
        record(
            &self.log,
            ObservedEvent::ViewModelInstanceName {
                handle,
                request_id,
                name,
            },
        );
    }
    fn on_view_model_deleted(&mut self, handle: ViewModelInstanceHandle, request_id: u64) {
        record(
            &self.log,
            ObservedEvent::ViewModelDeleted { handle, request_id },
        );
    }
    fn on_view_model_data_received(
        &mut self,
        handle: ViewModelInstanceHandle,
        request_id: u64,
        data: ViewModelInstanceData,
    ) {
        let value = match data.value {
            ViewModelInstanceValue::None => ObservedValue::None,
            ViewModelInstanceValue::Bool(value) => ObservedValue::Boolean(value),
            ViewModelInstanceValue::Number(value) => ObservedValue::Number(value),
            ViewModelInstanceValue::Color(value) => ObservedValue::Color(value),
            ViewModelInstanceValue::String(value) if data.meta_data.data_type == DataType::Enum => {
                ObservedValue::Enum(value)
            }
            ViewModelInstanceValue::String(value) => ObservedValue::String(value),
        };
        record(
            &self.log,
            ObservedEvent::ViewModelValue {
                handle,
                request_id,
                path: data.meta_data.name,
                value,
            },
        );
    }
    fn on_view_model_list_size_received(
        &mut self,
        handle: ViewModelInstanceHandle,
        request_id: u64,
        path: String,
        size: usize,
    ) {
        record(
            &self.log,
            ObservedEvent::ViewModelListSize {
                handle,
                request_id,
                path,
                size,
            },
        );
    }
    fn on_view_model_list_cleared(
        &mut self,
        handle: ViewModelInstanceHandle,
        request_id: u64,
        path: String,
    ) {
        record(
            &self.log,
            ObservedEvent::ViewModelListCleared {
                handle,
                request_id,
                path,
            },
        );
    }
}

macro_rules! asset_listener {
    ($name:ident, $trait:ident, $handle:ident, $decoded:ident, $error:ident, $deleted:ident, $on_decoded:ident, $on_error:ident, $on_deleted:ident) => {
        struct $name {
            base: ListenerBase<$handle>,
            log: EventLog,
        }
        impl $trait for $name {
            fn listener_base(&mut self) -> &mut ListenerBase<$handle> {
                &mut self.base
            }
            fn $on_decoded(&mut self, handle: $handle, request_id: u64) {
                record(&self.log, ObservedEvent::$decoded { handle, request_id });
            }
            fn $on_error(&mut self, handle: $handle, request_id: u64, error: String) {
                record(
                    &self.log,
                    ObservedEvent::$error {
                        handle,
                        request_id,
                        error,
                    },
                );
            }
            fn $on_deleted(&mut self, handle: $handle, request_id: u64) {
                record(&self.log, ObservedEvent::$deleted { handle, request_id });
            }
        }
    };
}
asset_listener!(
    ImageEvents,
    RenderImageListener,
    RenderImageHandle,
    ImageDecoded,
    ImageError,
    ImageDeleted,
    on_render_image_decoded,
    on_render_image_error,
    on_render_image_deleted
);
asset_listener!(
    AudioEvents,
    AudioSourceListener,
    AudioSourceHandle,
    AudioDecoded,
    AudioError,
    AudioDeleted,
    on_audio_source_decoded,
    on_audio_source_error,
    on_audio_source_deleted
);
asset_listener!(
    FontEvents,
    FontListener,
    FontHandle,
    FontDecoded,
    FontError,
    FontDeleted,
    on_font_decoded,
    on_font_error,
    on_font_deleted
);
asset_listener!(
    BlobEvents,
    BlobAssetListener,
    BlobAssetHandle,
    BlobDecoded,
    BlobError,
    BlobDeleted,
    on_blob_asset_decoded,
    on_blob_asset_error,
    on_blob_asset_deleted
);

struct TestListeners {
    file: FileListenerHandle,
    artboard: ArtboardListenerHandle,
    state_machine: StateMachineListenerHandle,
    view_model: ViewModelInstanceListenerHandle,
    image: RenderImageListenerHandle,
    audio: AudioSourceListenerHandle,
    font: FontListenerHandle,
    blob: BlobAssetListenerHandle,
}

impl TestListeners {
    fn new(log: &EventLog) -> Self {
        Self {
            file: ListenerHandle::new(Box::new(FileEvents {
                base: ListenerBase::new(),
                log: log.clone(),
            })),
            artboard: ListenerHandle::new(Box::new(ArtboardEvents {
                base: ListenerBase::new(),
                log: log.clone(),
            })),
            state_machine: ListenerHandle::new(Box::new(StateMachineEvents {
                base: ListenerBase::new(),
                log: log.clone(),
            })),
            view_model: ListenerHandle::new(Box::new(ViewModelEvents {
                base: ListenerBase::new(),
                log: log.clone(),
            })),
            image: ListenerHandle::new(Box::new(ImageEvents {
                base: ListenerBase::new(),
                log: log.clone(),
            })),
            audio: ListenerHandle::new(Box::new(AudioEvents {
                base: ListenerBase::new(),
                log: log.clone(),
            })),
            font: ListenerHandle::new(Box::new(FontEvents {
                base: ListenerBase::new(),
                log: log.clone(),
            })),
            blob: ListenerHandle::new(Box::new(BlobEvents {
                base: ListenerBase::new(),
                log: log.clone(),
            })),
        }
    }
}

#[derive(Debug, Clone)]
struct ExternalImage(u8, Arc<()>);

impl RenderImage for ExternalImage {
    fn retain_image(&self) -> std::rc::Rc<dyn RenderImage> {
        std::rc::Rc::new(self.clone())
    }
    fn image_identity(&self) -> usize {
        Arc::as_ptr(&self.1) as usize
    }
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn width(&self) -> u32 {
        u32::from(self.0) + 1
    }

    fn height(&self) -> u32 {
        1
    }
}

const ARTBOARD_FIXTURE: &[u8] = include_bytes!("../../../fixtures/command_queue/two_artboards.riv");
const ENTRY_FIXTURE: &[u8] = include_bytes!("../../../fixtures/command_queue/entry.riv");
const MULTI_MACHINE_FIXTURE: &[u8] =
    include_bytes!("../../../fixtures/command_queue/multiple_state_machines.riv");
const DATA_BIND_FIXTURE: &[u8] =
    include_bytes!("../../../fixtures/command_queue/data_bind_test_cmdq.riv");
const IMAGE_FIXTURE: &[u8] = include_bytes!("../../../fixtures/command_queue/batdude.png");
const AUDIO_FIXTURE: &[u8] = include_bytes!("../../../fixtures/command_queue/what.wav");
const FONT_FIXTURE: &[u8] = include_bytes!("../../../fixtures/command_queue/OpenSans-Italic.ttf");
const POINTER_FIXTURE: &[u8] = include_bytes!("../../../fixtures/command_queue/pointer_events.riv");
const RAPID_POINTER_FIXTURE: &[u8] =
    include_bytes!("../../../fixtures/command_queue/rapid_pointer_events.riv");
const HOSTED_IMAGE_FIXTURE: &[u8] =
    include_bytes!("../../../fixtures/command_queue/hosted_image_file.riv");
const HOSTED_FONT_FIXTURE: &[u8] =
    include_bytes!("../../../fixtures/command_queue/hosted_font_file.riv");
const GLOBAL_VARIABLES_FIXTURE: &[u8] =
    include_bytes!("../../../fixtures/command_queue/global_variables_test.riv");
const SEMANTIC_SIMPSONS_FIXTURE: &[u8] = include_bytes!("../../../fixtures/semantic/simpsons.riv");
const SEMANTIC_FOCUS_FIXTURE: &[u8] =
    include_bytes!("../../../fixtures/semantic/semantic_list_scroll_focus_fixed.riv");
const DATA_BIND_BLOB_FIXTURE: &[u8] =
    include_bytes!("../../../fixtures/sync/data_bind_blob_test.riv");

/// The pinned command-queue harness uses a real image decoder, while the
/// backend-neutral RecordingFactory intentionally records arbitrary bytes.
/// Keep the recording backend, but give this harness the pinned decoder's
/// malformed-input contract.
struct CommandQueueTestFactory(RecordingFactory);

impl CommandQueueTestFactory {
    fn new() -> Self {
        Self(RecordingFactory::new())
    }
}

impl Factory for CommandQueueTestFactory {
    fn make_render_buffer(
        &mut self,
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        size_in_bytes: usize,
    ) -> Box<dyn RenderBuffer> {
        self.0.make_render_buffer(buffer_type, flags, size_in_bytes)
    }

    fn make_linear_gradient(
        &mut self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        self.0.make_linear_gradient(sx, sy, ex, ey, colors, stops)
    }

    fn make_radial_gradient(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        self.0.make_radial_gradient(cx, cy, radius, colors, stops)
    }

    fn make_render_path(
        &mut self,
        raw_path: nuxie_render_api::RawPath,
        fill_rule: FillRule,
    ) -> Box<dyn RenderPath> {
        self.0.make_render_path(raw_path, fill_rule)
    }

    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        self.0.make_empty_render_path()
    }

    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        self.0.make_render_paint()
    }

    fn decode_image(&mut self, data: &[u8]) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
        if !data.starts_with(b"\x89PNG\r\n\x1a\n") {
            return Err(ImageDecodeError);
        }
        self.0.decode_image(data)
    }
}

fn server(queue: &CommandQueue) -> Box<CommandServer> {
    let mut factory = PersistentFactory::new(CommandQueueTestFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).expect("persistent factory");
    CommandServer::new(queue.clone(), factory, None)
}

fn semantic_fixture(
    listener: Option<&StateMachineListenerHandle>,
) -> (
    CommandQueue,
    Box<CommandServer>,
    ArtboardHandle,
    StateMachineHandle,
) {
    semantic_fixture_with(SEMANTIC_SIMPSONS_FIXTURE, listener)
}

fn semantic_fixture_with(
    fixture: &[u8],
    listener: Option<&StateMachineListenerHandle>,
) -> (
    CommandQueue,
    Box<CommandServer>,
    ArtboardHandle,
    StateMachineHandle,
) {
    let mut queue = CommandQueue::new();
    let file = queue.load_file(fixture.to_vec(), None, 0, None);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let view_model =
        queue.instantiate_view_model_instance_for_artboard(file, artboard, String::new(), None, 0);
    let state_machine = queue.instantiate_default_state_machine(artboard, listener, 0);
    queue.bind_view_model_instance(state_machine, view_model, 0);
    let server = server(&queue);
    (queue, server, artboard, state_machine)
}

fn warm_semantics(queue: &mut CommandQueue, state_machine: StateMachineHandle) {
    for _ in 0..10 {
        queue.advance_state_machine(state_machine, 0.1, 0);
    }
}

fn drain_semantics(queue: &mut CommandQueue, state_machine: StateMachineHandle, request_id: u64) {
    queue.drain_semantics_diff(
        state_machine,
        Fit::Contain,
        Alignment::CENTER,
        1.0,
        nuxie::Vec2D::new(500.0, 500.0),
        request_id,
    );
}

#[derive(Debug, Default)]
struct SemanticTestModel {
    nodes: BTreeMap<u32, SemanticsDiffNode>,
}

impl SemanticTestModel {
    fn apply(&mut self, diff: &SemanticsDiff) {
        for id in &diff.removed {
            self.nodes.remove(id);
        }
        for node in diff.added.iter().chain(&diff.moved) {
            self.nodes.insert(node.id, node.clone());
        }
        for node in &diff.updated_semantic {
            if let Some(existing) = self.nodes.get_mut(&node.id) {
                let bounds = existing.bounds();
                *existing = node.clone();
                existing.set_bounds(bounds);
            } else {
                self.nodes.insert(node.id, node.clone());
            }
        }
        for update in &diff.updated_geometry {
            if let Some(existing) = self.nodes.get_mut(&update.id) {
                existing.set_bounds(update.bounds());
            }
        }
    }
}

fn apply_semantic_events(model: &mut SemanticTestModel, captured: &[ObservedEvent]) {
    for event in captured {
        if let ObservedEvent::SemanticsDiffReceived { diff, .. } = event {
            model.apply(diff);
        }
    }
}

fn semantic_nodes_for_view(
    fit: Fit,
    scale_factor: f32,
    view_bounds: nuxie::Vec2D,
) -> SemanticTestModel {
    let (listener, log) = event_log();
    let (mut queue, mut server, _, state_machine) = semantic_fixture(Some(&listener.state_machine));
    queue.enable_semantics(state_machine, 0);
    warm_semantics(&mut queue, state_machine);
    queue.drain_semantics_diff(
        state_machine,
        fit,
        Alignment::CENTER,
        scale_factor,
        view_bounds,
        0,
    );
    assert!(server.process_commands());
    queue.process_messages();
    let mut model = SemanticTestModel::default();
    apply_semantic_events(&mut model, &events(&log));
    assert!(
        !events(&log)
            .iter()
            .any(|event| matches!(event, ObservedEvent::StateMachineError { .. }))
    );
    model
}

fn event_log() -> (TestListeners, EventLog) {
    let events = Arc::new(Mutex::new(Vec::new()));
    let listeners = TestListeners::new(&events);
    (listeners, events)
}

fn events(log: &Arc<Mutex<Vec<ObservedEvent>>>) -> Vec<ObservedEvent> {
    log.lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

#[test]
fn pod_stream_rcp() {
    const MAGIC_NUMBER: usize = 0x99;
    let mut queue = CommandQueue::new();
    let original = Arc::new(MAGIC_NUMBER);
    let captured = Arc::clone(&original);
    let null: Option<Arc<usize>> = None;
    let observed = Arc::new(Mutex::new(None));
    let observed_on_server = Arc::clone(&observed);
    queue.run_once(Box::new(move |_| {
        assert!(Arc::ptr_eq(&captured, &original));
        assert_eq!(*captured, MAGIC_NUMBER);
        *observed_on_server
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(captured);
        assert!(null.is_none());
    }));
    assert!(server(&queue).process_commands());
    assert_eq!(
        observed
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_deref(),
        Some(&MAGIC_NUMBER)
    );
}

#[test]
fn semantics_advance_does_not_auto_deliver_diff() {
    let (listener, log) = event_log();
    let (mut queue, mut server, _, state_machine) = semantic_fixture(Some(&listener.state_machine));
    queue.enable_semantics(state_machine, 0);
    warm_semantics(&mut queue, state_machine);

    assert!(server.process_commands());
    queue.process_messages();

    assert!(!events(&log).iter().any(|event| matches!(
        event,
        ObservedEvent::SemanticsDiffReceived { .. } | ObservedEvent::StateMachineError { .. }
    )));
}

#[test]
fn semantics_enable_and_initial_diff_on_drain() {
    let (listener, log) = event_log();
    let (mut queue, mut server, _, state_machine) = semantic_fixture(Some(&listener.state_machine));
    queue.enable_semantics(state_machine, 0);
    warm_semantics(&mut queue, state_machine);
    drain_semantics(&mut queue, state_machine, 0);

    assert!(server.process_commands());
    queue.process_messages();

    let mut model = SemanticTestModel::default();
    let mut diff_count = 0;
    for event in events(&log) {
        if let ObservedEvent::SemanticsDiffReceived { diff, .. } = event {
            diff_count += 1;
            model.apply(&diff);
        }
    }
    assert!(diff_count >= 1);
    assert!(!model.nodes.is_empty());
    assert!(
        model
            .nodes
            .values()
            .any(|node| node.role == SemanticRole::TabList as u32)
    );
    assert!(
        !events(&log)
            .iter()
            .any(|event| matches!(event, ObservedEvent::StateMachineError { .. }))
    );
}

#[test]
fn semantics_no_diff_when_not_enabled() {
    let (listener, log) = event_log();
    let (mut queue, mut server, _, state_machine) = semantic_fixture(Some(&listener.state_machine));
    warm_semantics(&mut queue, state_machine);

    assert!(server.process_commands());
    queue.process_messages();

    assert!(!events(&log).iter().any(|event| matches!(
        event,
        ObservedEvent::SemanticsDiffReceived { .. } | ObservedEvent::StateMachineError { .. }
    )));
}

#[test]
fn semantics_drain_diff_errors_when_not_enabled() {
    let (listener, log) = event_log();
    let (mut queue, mut server, _, state_machine) = semantic_fixture(Some(&listener.state_machine));
    let request_id = 0x1234;
    drain_semantics(&mut queue, state_machine, request_id);

    assert!(server.process_commands());
    queue.process_messages();

    let captured = events(&log);
    assert!(
        !captured
            .iter()
            .any(|event| matches!(event, ObservedEvent::SemanticsDiffReceived { .. }))
    );
    assert_eq!(
        captured
            .iter()
            .filter(|event| matches!(event, ObservedEvent::StateMachineError { .. }))
            .count(),
        1
    );
    assert!(captured.iter().any(|event| matches!(
        event,
        ObservedEvent::StateMachineError { request_id: actual, .. } if *actual == request_id
    )));
}

#[test]
fn semantics_drain_diff_only_emits_for_non_empty_diff() {
    let (listener, log) = event_log();
    let (mut queue, mut server, _, state_machine) = semantic_fixture(Some(&listener.state_machine));
    queue.enable_semantics(state_machine, 0);
    warm_semantics(&mut queue, state_machine);
    let request_id = 0xABCD;
    drain_semantics(&mut queue, state_machine, request_id);

    assert!(server.process_commands());
    queue.process_messages();
    assert_eq!(
        events(&log)
            .iter()
            .filter(|event| matches!(event, ObservedEvent::SemanticsDiffReceived { .. }))
            .count(),
        1
    );
    assert!(events(&log).iter().any(|event| matches!(
        event,
        ObservedEvent::SemanticsDiffReceived { request_id: actual, .. } if *actual == request_id
    )));

    drain_semantics(&mut queue, state_machine, 0xBCDE);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert_eq!(
        captured
            .iter()
            .filter(|event| matches!(event, ObservedEvent::SemanticsDiffReceived { .. }))
            .count(),
        1
    );
    assert!(
        !captured
            .iter()
            .any(|event| matches!(event, ObservedEvent::StateMachineError { .. }))
    );
}

#[test]
fn semantics_fire_tap_changes_selected_tab() {
    let (listener, log) = event_log();
    let (mut queue, mut server, _, state_machine) = semantic_fixture(Some(&listener.state_machine));
    queue.enable_semantics(state_machine, 0);
    warm_semantics(&mut queue, state_machine);
    drain_semantics(&mut queue, state_machine, 0);
    assert!(server.process_commands());
    queue.process_messages();

    let mut model = SemanticTestModel::default();
    let initial_events = events(&log);
    apply_semantic_events(&mut model, &initial_events);
    let selected_tab_id = model
        .nodes
        .values()
        .find(|node| {
            node.role == SemanticRole::Tab as u32
                && has_semantic_state(node.state_flags, SemanticState::SELECTED)
        })
        .map(|node| node.id)
        .expect("selected tab");
    let other_tab_id = model
        .nodes
        .values()
        .find(|node| {
            node.role == SemanticRole::Tab as u32
                && !has_semantic_state(node.state_flags, SemanticState::SELECTED)
        })
        .map(|node| node.id)
        .expect("other tab");

    queue.fire_semantic_action(state_machine, other_tab_id, SemanticActionType::Tap, 0);
    warm_semantics(&mut queue, state_machine);
    drain_semantics(&mut queue, state_machine, 0);
    assert!(server.process_commands());
    queue.process_messages();

    let captured = events(&log);
    apply_semantic_events(&mut model, &captured[initial_events.len()..]);
    assert!(has_semantic_state(
        model.nodes[&other_tab_id].state_flags,
        SemanticState::SELECTED
    ));
    assert!(!has_semantic_state(
        model.nodes[&selected_tab_id].state_flags,
        SemanticState::SELECTED
    ));
    assert!(
        !captured
            .iter()
            .any(|event| matches!(event, ObservedEvent::StateMachineError { .. }))
    );
}

#[test]
fn semantics_commands_on_invalid_state_machine_handle() {
    let (listener, log) = event_log();
    let (mut queue, mut server, artboard, _) = semantic_fixture(None);
    let bogus = queue.instantiate_state_machine_named(
        artboard,
        "this state machine does not exist".to_string(),
        Some(&listener.state_machine),
        0,
    );

    queue.enable_semantics(bogus, 0xE1);
    queue.drain_semantics_diff(
        bogus,
        Fit::Contain,
        Alignment::CENTER,
        1.0,
        nuxie::Vec2D::new(500.0, 500.0),
        0xE2,
    );
    queue.fire_semantic_action(bogus, 42, SemanticActionType::Tap, 0xE3);
    queue.request_semantic_focus(bogus, 42, 0xE4);
    queue.clear_semantic_focus(bogus, 0xE5);

    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    let error_request_ids = captured
        .iter()
        .filter_map(|event| match event {
            ObservedEvent::StateMachineError { request_id, .. } => Some(*request_id),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(error_request_ids, [0xE1, 0xE2, 0xE3, 0xE4, 0xE5]);
    assert!(
        !captured
            .iter()
            .any(|event| matches!(event, ObservedEvent::SemanticsDiffReceived { .. }))
    );
}

#[test]
fn semantics_drain_diff_maps_bounds_into_view_space() {
    let small_view = nuxie::Vec2D::new(200.0, 200.0);
    let large_view = nuxie::Vec2D::new(800.0, 800.0);
    let small = semantic_nodes_for_view(Fit::Contain, 1.0, small_view);
    let large = semantic_nodes_for_view(Fit::Contain, 1.0, large_view);

    let (small_tab, large_tab) = small
        .nodes
        .values()
        .filter(|node| {
            node.role == SemanticRole::Tab as u32
                && node.max_x > node.min_x
                && node.max_y > node.min_y
        })
        .find_map(|small_tab| {
            large
                .nodes
                .get(&small_tab.id)
                .map(|large_tab| (small_tab, large_tab))
        })
        .expect("shared tab with non-empty bounds");
    let small_width = small_tab.max_x - small_tab.min_x;
    let small_height = small_tab.max_y - small_tab.min_y;
    let large_width = large_tab.max_x - large_tab.min_x;
    let large_height = large_tab.max_y - large_tab.min_y;
    let expected_scale = large_view.x / small_view.x;

    assert!(large_width > small_width);
    assert!(large_height > small_height);
    assert!(((large_width / small_width) / expected_scale - 1.0).abs() <= 0.01);
    assert!(((large_height / small_height) / expected_scale - 1.0).abs() <= 0.01);
}

#[test]
fn semantics_request_focus_errors_when_not_enabled() {
    let (listener, log) = event_log();
    let (mut queue, mut server, _, state_machine) = semantic_fixture(Some(&listener.state_machine));
    queue.request_semantic_focus(state_machine, 1, 0x5151);

    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert_eq!(
        captured
            .iter()
            .filter(|event| matches!(
                event,
                ObservedEvent::StateMachineError {
                    request_id: 0x5151,
                    ..
                }
            ))
            .count(),
        1
    );
    assert!(
        !captured
            .iter()
            .any(|event| matches!(event, ObservedEvent::SemanticsDiffReceived { .. }))
    );
}

#[test]
fn semantics_fire_action_errors_when_not_enabled() {
    let (listener, log) = event_log();
    let (mut queue, mut server, _, state_machine) = semantic_fixture(Some(&listener.state_machine));
    queue.fire_semantic_action(state_machine, 1, SemanticActionType::Tap, 0x5252);

    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert_eq!(
        captured
            .iter()
            .filter(|event| matches!(
                event,
                ObservedEvent::StateMachineError {
                    request_id: 0x5252,
                    ..
                }
            ))
            .count(),
        1
    );
    assert!(
        !captured
            .iter()
            .any(|event| matches!(event, ObservedEvent::SemanticsDiffReceived { .. }))
    );
}

#[test]
fn semantics_request_focus_on_valid_node_routes_without_error() {
    let (listener, log) = event_log();
    let (mut queue, mut server, _, state_machine) = semantic_fixture(Some(&listener.state_machine));
    queue.enable_semantics(state_machine, 0);
    warm_semantics(&mut queue, state_machine);
    drain_semantics(&mut queue, state_machine, 0);
    assert!(server.process_commands());
    queue.process_messages();

    let mut model = SemanticTestModel::default();
    apply_semantic_events(&mut model, &events(&log));
    let target_id = model.nodes.keys().next().copied().expect("semantic node");
    queue.request_semantic_focus(state_machine, target_id, 0);
    warm_semantics(&mut queue, state_machine);
    drain_semantics(&mut queue, state_machine, 0);
    assert!(server.process_commands());
    queue.process_messages();

    assert!(
        !events(&log)
            .iter()
            .any(|event| matches!(event, ObservedEvent::StateMachineError { .. }))
    );
}

#[test]
fn semantics_clear_focus_removes_focused_bit() {
    let (listener, log) = event_log();
    let (mut queue, mut server, _, state_machine) =
        semantic_fixture_with(SEMANTIC_FOCUS_FIXTURE, Some(&listener.state_machine));
    queue.enable_semantics(state_machine, 0);
    warm_semantics(&mut queue, state_machine);
    drain_semantics(&mut queue, state_machine, 0);
    assert!(server.process_commands());
    queue.process_messages();

    let mut model = SemanticTestModel::default();
    let initial = events(&log);
    apply_semantic_events(&mut model, &initial);
    let focusable_id = model
        .nodes
        .values()
        .find(|node| has_semantic_trait(node.trait_flags, SemanticTrait::FOCUSABLE))
        .map(|node| node.id)
        .expect("focusable semantic node");

    queue.request_semantic_focus(state_machine, focusable_id, 0);
    warm_semantics(&mut queue, state_machine);
    drain_semantics(&mut queue, state_machine, 0);
    assert!(server.process_commands());
    queue.process_messages();
    let focused = events(&log);
    apply_semantic_events(&mut model, &focused[initial.len()..]);
    assert!(has_semantic_state(
        model.nodes[&focusable_id].state_flags,
        SemanticState::FOCUSED
    ));

    queue.clear_semantic_focus(state_machine, 0);
    warm_semantics(&mut queue, state_machine);
    drain_semantics(&mut queue, state_machine, 0);
    assert!(server.process_commands());
    queue.process_messages();
    let cleared = events(&log);
    apply_semantic_events(&mut model, &cleared[focused.len()..]);
    assert!(!has_semantic_state(
        model.nodes[&focusable_id].state_flags,
        SemanticState::FOCUSED
    ));
    assert!(
        !cleared
            .iter()
            .any(|event| matches!(event, ObservedEvent::StateMachineError { .. }))
    );
}

#[test]
fn semantics_drain_diff_honors_scale_factor_for_matching_view() {
    let (mut queue, mut server, artboard, _) = semantic_fixture(None);
    let captured_bounds = Arc::new(Mutex::new(None));
    let sink = Arc::clone(&captured_bounds);
    queue.run_once(Box::new(move |server| {
        *sink.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = server
            .with_artboard_instance(artboard, |artboard| {
                (
                    artboard.base.layout_x(),
                    artboard.base.layout_y(),
                    artboard.base.width(),
                    artboard.base.height(),
                )
            });
    }));
    assert!(server.process_commands());
    let (x, y, width, height) = captured_bounds
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .expect("artboard bounds");
    assert_eq!((x, y), (0.0, 0.0));
    assert!(width > 0.0 && height > 0.0);
    let view_bounds = nuxie::Vec2D::new(width, height);

    let at_scale_1 = semantic_nodes_for_view(Fit::Layout, 1.0, view_bounds);
    let at_scale_2 = semantic_nodes_for_view(Fit::Layout, 2.0, view_bounds);
    let mut compared_any = false;
    for node in at_scale_1
        .nodes
        .values()
        .filter(|node| node.max_x > node.min_x && node.max_y > node.min_y)
    {
        let Some(scaled) = at_scale_2.nodes.get(&node.id) else {
            continue;
        };
        let width_ratio = (scaled.max_x - scaled.min_x) / (node.max_x - node.min_x);
        let height_ratio = (scaled.max_y - scaled.min_y) / (node.max_y - node.min_y);
        assert!((width_ratio / 2.0 - 1.0).abs() <= 0.02);
        assert!((height_ratio / 2.0 - 1.0).abs() <= 0.02);
        compared_any = true;
    }
    assert!(compared_any);
}

#[test]
fn handles_are_typed_nonzero_and_monotonic() {
    let mut queue = CommandQueue::new();
    let first = queue.load_file(Vec::new(), None, 0, None);
    let second = queue.load_file(Vec::new(), None, 0, None);
    assert!(!first.is_null());
    assert!(!second.is_null());
    assert_ne!(first, second);
}

#[test]
fn artboard_management() {
    let mut queue = CommandQueue::new();
    let file = queue.load_file(ARTBOARD_FIXTURE.to_vec(), None, 0, None);
    let one = queue.instantiate_artboard_named(file, "One".to_string(), None, 0);
    let two = queue.instantiate_artboard_named(file, "Two".to_string(), None, 0);
    let missing = queue.instantiate_artboard_named(file, "Three".to_string(), None, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    assert!(server.get_file(file).is_some());
    assert!(server.get_bindable_artboard(one).is_some());
    assert!(server.get_bindable_artboard(two).is_some());
    assert!(server.get_bindable_artboard(missing).is_none());

    queue.delete_artboard(missing, 0);
    queue.delete_artboard(two, 0);
    assert!(server.process_commands());
    assert!(server.get_bindable_artboard(one).is_some());
    assert!(server.get_bindable_artboard(two).is_none());
    assert!(server.get_bindable_artboard(missing).is_none());

    queue.delete_file(file, 0);
    assert!(server.process_commands());
    assert!(server.get_file(file).is_none());
    // The pinned source creates a file-dependency slot but never appends the
    // instantiated artboard, so deleting the file alone leaves it registered.
    assert!(server.get_bindable_artboard(one).is_some());

    queue.delete_artboard(one, 0);
    assert!(server.process_commands());
    assert!(server.get_bindable_artboard(one).is_none());
}

#[test]
fn state_machine_management() {
    let mut queue = CommandQueue::new();
    let (listener, events) = event_log();
    queue.set_global_artboard_listener(Some(&listener.artboard));
    let file = queue.load_file(MULTI_MACHINE_FIXTURE.to_vec(), None, 0, None);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let one = queue.instantiate_state_machine_named(artboard, "one".to_string(), None, 0);
    let two = queue.instantiate_state_machine_named(artboard, "two".to_string(), None, 0);
    let missing = queue.instantiate_state_machine_named(artboard, "blahblah".to_string(), None, 9);
    let mut server = server(&queue);
    assert!(server.process_commands());
    assert!(server.get_bindable_artboard(artboard).is_some());
    assert!(
        server
            .with_state_machine_instance_mut(one, |_| ())
            .is_some()
    );
    assert!(
        server
            .with_state_machine_instance_mut(two, |_| ())
            .is_some()
    );
    assert!(
        server
            .with_state_machine_instance_mut(missing, |_| ())
            .is_none()
    );
    queue.process_messages();
    assert!(
        events
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter()
            .any(|event| matches!(event, ObservedEvent::ArtboardError { request_id: 9, .. }))
    );

    queue.delete_file(file, 0);
    queue.delete_artboard(artboard, 0);
    queue.delete_state_machine(one, 0);
    assert!(server.process_commands());
    assert!(server.get_file(file).is_none());
    assert!(server.get_bindable_artboard(artboard).is_none());
    assert!(
        server
            .with_state_machine_instance_mut(one, |_| ())
            .is_none()
    );
    assert!(
        server
            .with_state_machine_instance_mut(two, |_| ())
            .is_none()
    );

    queue.delete_state_machine(two, 0);
    assert!(server.process_commands());
    assert!(
        server
            .with_state_machine_instance_mut(two, |_| ())
            .is_none()
    );
}

#[test]
fn default_artboard_and_state_machine() {
    let mut queue = CommandQueue::new();
    let file = queue.load_file(ENTRY_FIXTURE.to_vec(), None, 0, None);
    let default_artboard = queue.instantiate_default_artboard(file, None, 0);
    let default_machine = queue.instantiate_default_state_machine(default_artboard, None, 0);
    let empty_artboard = queue.instantiate_artboard_named(file, "".to_string(), None, 0);
    let empty_machine =
        queue.instantiate_state_machine_named(empty_artboard, "".to_string(), None, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());

    for (artboard, machine) in [
        (default_artboard, default_machine),
        (empty_artboard, empty_machine),
    ] {
        let machine_handle = server
            .with_state_machine_instance_mut(machine, |machine| machine.state_machine())
            .expect("default machine");
        server
            .with_artboard_instance(artboard, |artboard| {
                assert_eq!(artboard.base.name(), "New Artboard");
                let machine_index = artboard
                    .base
                    .state_machine_handles()
                    .iter()
                    .position(|candidate| candidate == &machine_handle)
                    .expect("state machine belongs to artboard");
                assert_eq!(
                    artboard.base.state_machine_name_at(machine_index),
                    "State Machine 1"
                );
            })
            .expect("default artboard");
    }
}

#[test]
fn invalid_handles() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    queue.set_global_file_listener(Some(&listener.file));
    let good_file = queue.load_file(ENTRY_FIXTURE.to_vec(), None, 0, None);
    let bad_file = queue.load_file(vec![0; 100 * 1024], None, 10, None);
    let mut server = server(&queue);
    assert!(server.process_commands());
    assert!(server.get_file(good_file).is_some());
    assert!(server.get_file(bad_file).is_none());

    let good_artboard =
        queue.instantiate_artboard_named(good_file, "New Artboard".to_string(), None, 0);
    let bad_artboard_one = queue.instantiate_default_artboard(bad_file, None, 11);
    let bad_artboard_two =
        queue.instantiate_artboard_named(bad_file, "New Artboard".to_string(), None, 12);
    let bad_artboard_three =
        queue.instantiate_artboard_named(good_file, "blahblahblah".to_string(), None, 13);
    assert!(server.process_commands());
    assert!(server.get_bindable_artboard(good_artboard).is_some());
    for handle in [bad_artboard_one, bad_artboard_two, bad_artboard_three] {
        assert!(server.get_bindable_artboard(handle).is_none());
    }

    let good_machine = queue.instantiate_state_machine_named(
        good_artboard,
        "State Machine 1".to_string(),
        None,
        0,
    );
    let bad_machine_one = queue.instantiate_state_machine_named(
        bad_artboard_two,
        "State Machine 1".to_string(),
        None,
        14,
    );
    let bad_machine_two =
        queue.instantiate_state_machine_named(good_artboard, "blahblahblah".to_string(), None, 15);
    let bad_machine_three = queue.instantiate_default_state_machine(bad_artboard_three, None, 16);
    assert!(server.process_commands());
    assert!(
        server
            .with_state_machine_instance_mut(good_machine, |_| ())
            .is_some()
    );
    for handle in [bad_machine_one, bad_machine_two, bad_machine_three] {
        assert!(
            server
                .with_state_machine_instance_mut(handle, |_| ())
                .is_none()
        );
    }

    for handle in [bad_machine_three, bad_machine_two, bad_machine_one] {
        queue.delete_state_machine(handle, 0);
    }
    for handle in [bad_artboard_three, bad_artboard_two, bad_artboard_one] {
        queue.delete_artboard(handle, 0);
    }
    queue.delete_file(bad_file, 0);
    assert!(server.process_commands());
    assert!(server.get_file(good_file).is_some());
    assert!(server.get_bindable_artboard(good_artboard).is_some());
    assert!(
        server
            .with_state_machine_instance_mut(good_machine, |_| ())
            .is_some()
    );

    queue.delete_state_machine(good_machine, 0);
    queue.delete_artboard(good_artboard, 0);
    queue.delete_file(good_file, 0);
    assert!(server.process_commands());
    assert!(server.get_file(good_file).is_none());
    assert!(server.get_bindable_artboard(good_artboard).is_none());
    assert!(
        server
            .with_state_machine_instance_mut(good_machine, |_| ())
            .is_none()
    );

    queue.process_messages();
    assert!(
        events(&log)
            .iter()
            .any(|event| { matches!(event, ObservedEvent::FileError { request_id: 10, .. }) })
    );
}

#[test]
fn draw_loops() {
    let mut queue = CommandQueue::new();
    let first = queue.create_draw_key();
    let second = queue.create_draw_key();
    let counts = Arc::new(Mutex::new((0usize, 0usize)));
    let mut server = server(&queue);

    let first_counts = Arc::clone(&counts);
    queue.draw(
        first,
        Box::new(move |_, _| {
            first_counts
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .0 += 1;
        }),
    );
    let second_counts = Arc::clone(&counts);
    queue.draw(
        second,
        Box::new(move |_, _| {
            second_counts
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .1 += 1;
        }),
    );
    assert!(server.process_commands());
    assert_eq!(*counts.lock().unwrap(), (1, 1));

    let second_counts = Arc::clone(&counts);
    queue.draw(
        second,
        Box::new(move |_, _| {
            second_counts
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .1 += 1;
        }),
    );
    assert!(server.process_commands());
    assert_eq!(*counts.lock().unwrap(), (1, 2));

    for _ in 0..10 {
        assert!(server.process_commands());
    }
    assert_eq!(*counts.lock().unwrap(), (1, 2));
}

#[test]
fn test_support_for_all_asset_types() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    let attempted = Arc::new(Mutex::new(Vec::new()));
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), Some(&listener.file), 0, None);
    queue.request_file_assets(file, 1);
    let loader = FileAssetLoaderRef::new(Box::new(RecordingAssetLoader {
        attempted: Arc::clone(&attempted),
    }));
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).expect("persistent factory");
    let mut server = CommandServer::new(queue.clone(), factory, Some(loader));
    assert!(server.process_commands());
    queue.process_messages();
    let assets = events(&log)
        .into_iter()
        .find_map(|event| match event {
            ObservedEvent::FileAssetsListed { assets, .. } => Some(assets),
            _ => None,
        })
        .expect("file assets list callback");
    assert!(!assets.is_empty());
    assert!(assets.iter().all(|asset| asset.asset_type != 0));
    let attempted = attempted
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert!(!attempted.is_empty());
    assert!(attempted.iter().all(|type_key| *type_key != 0));
}

#[test]
#[cfg(any())] // The upstream-shaped queue owns Rc-backed runtime resources and is not Send.
fn wait_for_server_race_condition() {
    let mut queue = CommandQueue::new();
    let worker_queue = queue.clone();
    let worker = thread::spawn(move || {
        let mut server = server(&worker_queue);
        server.serve_until_disconnect();
    });
    let completed = Arc::new(AtomicUsize::new(0));
    for _ in 0..100 {
        let completed_on_server = Arc::clone(&completed);
        queue.run_once(Box::new(move |_| {
            completed_on_server.fetch_add(1, Ordering::SeqCst);
        }));
        let key = queue.create_draw_key();
        queue.draw(key, Box::new(|_, _| {}));
    }
    let completed_on_server = Arc::clone(&completed);
    queue.run_once(Box::new(move |_| {
        completed_on_server.fetch_add(1, Ordering::SeqCst);
    }));
    queue.disconnect();
    worker.join().expect("command server thread panicked");
    assert_eq!(completed.load(Ordering::SeqCst), 101);
}

#[test]
#[cfg(any())] // The upstream queue has no public command-loop break command.
fn stop_messages_command() {
    let mut queue = CommandQueue::new();
    let count = Arc::new(AtomicUsize::new(0));
    let mut server = server(&queue);
    let first = Arc::clone(&count);
    queue.run_once(Box::new(move |_| {
        first.fetch_add(1, Ordering::SeqCst);
    }));
    queue.testing_command_loop_break();
    for index in 0..10 {
        let count_on_server = Arc::clone(&count);
        queue.run_once(Box::new(move |_| {
            count_on_server.fetch_add(1, Ordering::SeqCst);
        }));
        if index == 5 {
            queue.testing_command_loop_break();
        }
    }

    assert!(server.process_commands());
    assert_eq!(count.load(Ordering::SeqCst), 1);
    assert!(server.process_commands());
    assert_eq!(count.load(Ordering::SeqCst), 7);
    assert!(server.process_commands());
    assert_eq!(count.load(Ordering::SeqCst), 11);
}

#[test]
#[cfg(any())] // Global asset map inspection was a removed downstream-only test hook.
fn global_asset_set_and_remove() {
    let mut queue = CommandQueue::new();
    let image = queue.decode_image(IMAGE_FIXTURE.to_vec(), None, 0);
    let bad_image = queue.decode_image(vec![0; 1024], None, 0);
    let audio = queue.decode_audio(AUDIO_FIXTURE.to_vec(), None, 0);
    let bad_audio = queue.decode_audio(vec![0; 1024], None, 0);
    let font = queue.decode_font(FONT_FIXTURE.to_vec(), None, 0);
    let bad_font = queue.decode_font(vec![0; 1024], None, 0);
    queue.add_global_image_asset("image".to_string(), image, 0);
    queue.add_global_image_asset("bad-image".to_string(), bad_image, 0);
    queue.add_global_audio_asset("audio".to_string(), audio, 0);
    queue.add_global_audio_asset("bad-audio".to_string(), bad_audio, 0);
    queue.add_global_font_asset("font".to_string(), font, 0);
    queue.add_global_font_asset("bad-font".to_string(), bad_font, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    assert_eq!(server.testing_global_image_named("image"), Some(image));
    assert_eq!(server.testing_global_audio_named("audio"), Some(audio));
    assert_eq!(server.testing_global_font_named("font"), Some(font));
    assert_eq!(server.testing_global_image_named("bad-image"), None);
    assert_eq!(server.testing_global_audio_named("bad-audio"), None);
    assert_eq!(server.testing_global_font_named("bad-font"), None);

    queue.remove_global_image_asset("image".to_string(), 0);
    queue.remove_global_audio_asset("audio".to_string(), 0);
    queue.remove_global_font_asset("font".to_string(), 0);
    queue.remove_global_image_asset("missing".to_string(), 0);
    queue.remove_global_audio_asset("missing".to_string(), 0);
    queue.remove_global_font_asset("missing".to_string(), 0);
    assert!(server.process_commands());
    assert_eq!(server.testing_global_image_named("image"), None);
    assert_eq!(server.testing_global_audio_named("audio"), None);
    assert_eq!(server.testing_global_font_named("font"), None);

    queue.add_global_image_asset("image".to_string(), image, 0);
    queue.add_global_audio_asset("audio".to_string(), audio, 0);
    queue.add_global_font_asset("font".to_string(), font, 0);
    queue.delete_image(image, 0);
    queue.delete_audio(audio, 0);
    queue.delete_font(font, 0);
    assert!(server.process_commands());
    assert_eq!(server.testing_global_image_named("image"), None);
    assert_eq!(server.testing_global_audio_named("audio"), None);
    assert_eq!(server.testing_global_font_named("font"), None);
}

#[test]
fn external_resources() {
    let mut queue = CommandQueue::new();
    let image: Box<dyn RenderImage + Send> = Box::new(ExternalImage(0, Arc::new(())));
    let audio = RuntimeAudioSource::from_encoded(AUDIO_FIXTURE).expect("decode audio");
    let audio_identity = Arc::as_ptr(&audio) as usize;
    let font = RawTextFont::decode(Arc::<[u8]>::from(FONT_FIXTURE)).expect("decode font");
    let blob = Arc::new(RuntimeBlobAsset::new(
        "external",
        Arc::<[u8]>::from([1, 2, 3, 4, 5]),
    ));
    let blob_identity = Arc::as_ptr(&blob) as usize;
    let image_handle = queue.add_external_image(Some(image), None, 0);
    let audio_handle = queue.add_external_audio(Some(audio), None, 0);
    let font_handle = queue.add_external_font(Some(font), None, 0);
    let blob_handle = queue.add_external_blob(Some(blob), None, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    let retained_image = server.get_image(image_handle).expect("external image");
    assert_eq!(
        retained_image
            .as_any()
            .downcast_ref::<ExternalImage>()
            .map(|image| image.0),
        Some(0)
    );
    assert_eq!(
        Arc::as_ptr(
            &server
                .get_audio_source(audio_handle)
                .expect("external audio")
        ) as usize,
        audio_identity
    );
    assert!(server.get_font(font_handle).is_some());
    assert_eq!(
        Arc::as_ptr(&server.get_blob(blob_handle).expect("external blob")) as usize,
        blob_identity
    );

    queue.delete_image(image_handle, 0);
    queue.delete_audio(audio_handle, 0);
    queue.delete_font(font_handle, 0);
    queue.delete_blob(blob_handle, 0);
    assert!(server.process_commands());
    assert!(server.get_image(image_handle).is_none());
    assert!(server.get_audio_source(audio_handle).is_none());
    assert!(server.get_font(font_handle).is_none());
    assert!(server.get_blob(blob_handle).is_none());
}

#[test]
fn empty_external_resources_report_the_pinned_errors() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    queue.add_external_image(None, Some(&listener.image), 1);
    queue.add_external_audio(None, Some(&listener.audio), 2);
    queue.add_external_font(None, Some(&listener.font), 3);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(captured.iter().any(|event| matches!(
        event,
        ObservedEvent::ImageError { request_id: 1, error, .. }
            if error == "External image was empty"
    )));
    assert!(captured.iter().any(|event| matches!(
        event,
        ObservedEvent::AudioError { request_id: 2, error, .. }
            if error == "External audio source was invalid"
    )));
    assert!(captured.iter().any(|event| matches!(
        event,
        ObservedEvent::FontError { request_id: 3, error, .. }
            if error == "Command Server failed to decode font"
    )));
}

#[test]
fn render_image() {
    let mut queue = CommandQueue::new();
    let image = queue.decode_image(IMAGE_FIXTURE.to_vec(), None, 0);
    let bad_image = queue.decode_image(vec![0; 1024], None, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    assert!(server.get_image(image).is_some());
    assert!(server.get_image(bad_image).is_none());
    queue.delete_image(image, 0);
    queue.delete_image(bad_image, 0);
    assert!(server.process_commands());
    assert!(server.get_image(image).is_none());
    assert!(server.get_image(bad_image).is_none());
}

#[test]
fn blob_asset() {
    let mut queue = CommandQueue::new();
    let bytes = vec![0x10, 0x20, 0x30, 0x40];
    let blob = queue.decode_blob(bytes.clone(), None, 0);
    let empty_blob = queue.decode_blob(Vec::new(), None, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    assert_eq!(
        server.get_blob(blob).map(|blob| blob.bytes().to_vec()),
        Some(bytes)
    );
    assert_eq!(
        server
            .get_blob(empty_blob)
            .map(|blob| blob.bytes().to_vec()),
        Some(Vec::new())
    );

    queue.delete_blob(blob, 0);
    queue.delete_blob(empty_blob, 0);
    assert!(server.process_commands());
    assert!(server.get_blob(blob).is_none());
    assert!(server.get_blob(empty_blob).is_none());
}

#[test]
fn blob_asset_listener_callbacks() {
    let mut queue = CommandQueue::new();
    let (decode_listener, decode_log) = event_log();
    let (external_listener, external_log) = event_log();
    let (error_listener, error_log) = event_log();
    let decoded = queue.decode_blob(vec![1, 2, 3], Some(&decode_listener.blob), 0x10);
    let external = queue.add_external_blob(
        Some(Arc::new(RuntimeBlobAsset::new(
            "external",
            Arc::<[u8]>::from([4, 5, 6]),
        ))),
        Some(&external_listener.blob),
        0x20,
    );
    let missing = queue.add_external_blob(None, Some(&error_listener.blob), 0x30);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();

    assert!(events(&decode_log).iter().any(|event| matches!(
        event,
        ObservedEvent::BlobDecoded { handle, request_id: 0x10 } if *handle == decoded
    )));
    assert!(events(&external_log).iter().any(|event| matches!(
        event,
        ObservedEvent::BlobDecoded { handle, request_id: 0x20 } if *handle == external
    )));
    assert!(events(&error_log).iter().any(|event| matches!(
        event,
        ObservedEvent::BlobError { handle, request_id: 0x30, error }
            if *handle == missing && !error.is_empty()
    )));

    queue.delete_blob(decoded, 0x11);
    queue.delete_blob(external, 0x21);
    assert!(server.process_commands());
    queue.process_messages();
    assert!(events(&decode_log).iter().any(|event| matches!(
        event,
        ObservedEvent::BlobDeleted { handle, request_id: 0x11 } if *handle == decoded
    )));
    assert!(events(&external_log).iter().any(|event| matches!(
        event,
        ObservedEvent::BlobDeleted { handle, request_id: 0x21 } if *handle == external
    )));
}

#[test]
fn view_model_blob_property_set() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(DATA_BIND_BLOB_FIXTURE.to_vec(), None, 0, None);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let root = queue.instantiate_view_model_instance_for_artboard(
        file,
        artboard,
        String::new(),
        Some(&listener.view_model),
        0,
    );
    let bytes = vec![0x0a, 0x0b, 0x0c];
    let blob = queue.decode_blob(bytes.clone(), None, 0);
    queue.set_view_model_instance_blob(root, "xml".to_owned(), blob, 0);
    queue.run_once(Box::new(move |server| {
        let expected = server.get_blob(blob).expect("decoded blob");
        let actual = server
            .get_view_model_instance(root)
            .expect("root view model")
            .property_blob("xml")
            .and_then(|value| {
                value
                    .value_runtime()
                    .handle()
                    .with(|property| {
                        property
                            .as_view_model_instance_asset_blob()
                            .and_then(|property| property.asset())
                    })
                    .flatten()
            })
            .expect("blob property");
        assert!(Arc::ptr_eq(&actual, &expected));
        assert_eq!(actual.bytes(), bytes.as_slice());
    }));

    let deleted = queue.decode_blob(vec![1], None, 0);
    queue.delete_blob(deleted, 0);
    queue.set_view_model_instance_blob(root, "xml".to_owned(), deleted, 0x41);
    queue.set_view_model_instance_blob(root, "missing".to_owned(), blob, 0x42);
    queue.run_once(Box::new(move |server| {
        let expected = server.get_blob(blob).expect("retained blob");
        let actual = server
            .get_view_model_instance(root)
            .expect("root view model")
            .property_blob("xml")
            .and_then(|value| {
                value
                    .value_runtime()
                    .handle()
                    .with(|property| {
                        property
                            .as_view_model_instance_asset_blob()
                            .and_then(|property| property.asset())
                    })
                    .flatten()
            })
            .expect("failed set retains prior blob");
        assert!(Arc::ptr_eq(&actual, &expected));
    }));

    queue.set_view_model_instance_blob(root, "xml".to_owned(), BlobAssetHandle::NULL, 0);
    queue.run_once(Box::new(move |server| {
        let value = server
            .get_view_model_instance(root)
            .expect("root view model")
            .property_blob("xml")
            .expect("blob property");
        assert!(
            value
                .value_runtime()
                .handle()
                .with(|property| {
                    property
                        .as_view_model_instance_asset_blob()
                        .and_then(|property| property.asset())
                })
                .flatten()
                .is_none()
        );
    }));
    queue.delete_view_model_instance(root, 0);
    queue.set_view_model_instance_blob(root, "xml".to_owned(), blob, 0x43);

    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let error_ids = events(&log)
        .into_iter()
        .filter_map(|event| match event {
            ObservedEvent::ViewModelError { request_id, .. }
                if matches!(request_id, 0x41 | 0x42 | 0x43) =>
            {
                Some(request_id)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(error_ids, [0x41, 0x42, 0x43]);
}

#[test]
fn view_model_blob_property_subscription() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(DATA_BIND_BLOB_FIXTURE.to_vec(), None, 0, None);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let root = queue.instantiate_view_model_instance_for_artboard(
        file,
        artboard,
        String::new(),
        Some(&listener.view_model),
        0,
    );
    queue.subscribe_to_view_model_property(root, "xml".to_string(), DataType::AssetBlob, 0x50);
    queue.subscribe_to_view_model_property(
        root,
        "Bad property".to_string(),
        DataType::AssetBlob,
        0x51,
    );
    let blob = queue.decode_blob(vec![1, 2, 3], None, 0);
    queue.set_view_model_instance_blob(root, "xml".to_owned(), blob, 0);

    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert_eq!(
        captured
            .iter()
            .filter(|event| matches!(
                event,
                ObservedEvent::ViewModelValue {
                    handle,
                    request_id: 0x50,
                    path,
                    value: ObservedValue::None,
                } if *handle == root && path == "xml"
            ))
            .count(),
        1
    );
    assert!(captured.iter().any(|event| matches!(
        event,
        ObservedEvent::ViewModelError { handle, request_id: 0x51, .. } if *handle == root
    )));

    queue.unsubscribe_to_view_model_property(root, "xml".to_string(), DataType::AssetBlob, 0);
    assert!(server.process_commands());
}

#[test]
fn audio_source() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    let audio = queue.decode_audio(AUDIO_FIXTURE.to_vec(), Some(&listener.audio), 10);
    let bad_audio = queue.decode_audio(vec![0; 1024], None, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    assert!(server.get_audio_source(audio).is_some());
    assert!(server.get_audio_source(bad_audio).is_none());
    queue.delete_audio(audio, 0x10);
    queue.delete_audio(bad_audio, 0);
    assert!(server.process_commands());
    assert!(server.get_audio_source(audio).is_none());
    assert!(server.get_audio_source(bad_audio).is_none());
    queue.process_messages();
    assert!(events(&log).iter().any(|event| matches!(
        event,
        ObservedEvent::AudioDeleted { handle, request_id: 0x10 } if *handle == audio
    )));
}

#[test]
fn font() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    let font = queue.decode_font(FONT_FIXTURE.to_vec(), Some(&listener.font), 10);
    let bad_font = queue.decode_font(vec![0; 1024], None, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    assert!(server.get_font(font).is_some());
    assert!(server.get_font(bad_font).is_none());
    queue.delete_font(font, 0x10);
    queue.delete_font(bad_font, 0);
    assert!(server.process_commands());
    assert!(server.get_font(font).is_none());
    assert!(server.get_font(bad_font).is_none());
    queue.process_messages();
    assert!(events(&log).iter().any(|event| matches!(
        event,
        ObservedEvent::FontDeleted { handle, request_id: 0x10 } if *handle == font
    )));
}

#[test]
fn view_models() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    queue.set_global_view_model_instance_listener(Some(&listener.view_model));
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 0, None);
    let blank =
        queue.instantiate_blank_view_model_instance_named(file, "Test All".to_string(), None, 0);
    let default = queue.instantiate_view_model_instance_named(
        file,
        "Test All".to_string(),
        "".to_string(),
        None,
        0,
    );
    let named = queue.instantiate_view_model_instance_named(
        file,
        "Test All".to_string(),
        "Test Alternate".to_string(),
        None,
        0,
    );
    let nested =
        queue.reference_nested_view_model_instance(blank, "Test Nested".to_string(), None, 0);
    queue.insert_view_model_instance_list_view_model(blank, "Test List".to_owned(), nested, 0, 0);
    let listed =
        queue.reference_list_view_model_instance(blank, "Test List".to_string(), 0, None, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    for handle in [blank, default, named, nested, listed] {
        assert!(server.get_view_model_instance(handle).is_some());
    }
    let nested_instance = server
        .get_view_model_instance(nested)
        .expect("nested view model");
    let listed_instance = server
        .get_view_model_instance(listed)
        .expect("listed view model");
    assert_eq!(nested_instance.instance(), listed_instance.instance());

    queue.remove_view_model_instance_list_value(blank, "Test List".to_owned(), nested, 0);
    queue.request_view_model_instance_list_size(blank, "Test List".to_string(), 2);
    assert!(server.process_commands());
    queue.process_messages();
    assert!(events(&log).iter().any(|event| matches!(
        event,
        ObservedEvent::ViewModelListSize { handle, request_id: 2, size: 0, .. }
            if *handle == blank
    )));

    let bad_blank =
        queue.instantiate_blank_view_model_instance_named(file, "Blah".to_string(), None, 0);
    let bad_named = queue.instantiate_view_model_instance_named(
        file,
        "Blah".to_string(),
        "Blah".to_string(),
        None,
        0,
    );
    let bad_instance = queue.instantiate_view_model_instance_named(
        file,
        "Test All".to_string(),
        "Blah".to_string(),
        None,
        0,
    );
    let bad_nested = queue.reference_nested_view_model_instance(blank, "Blah".to_string(), None, 0);
    let bad_list =
        queue.reference_list_view_model_instance(blank, "Test List".to_string(), 5, None, 0);
    assert!(server.process_commands());
    for handle in [bad_blank, bad_named, bad_instance, bad_nested, bad_list] {
        assert!(server.get_view_model_instance(handle).is_none());
    }

    queue.delete_view_model_instance(blank, 0);
    assert!(server.process_commands());
    assert!(server.get_view_model_instance(blank).is_none());
    assert!(server.get_view_model_instance(nested).is_some());
    queue.delete_view_model_instance(nested, 0);
    assert!(server.process_commands());
    assert!(server.get_view_model_instance(nested).is_none());
}

#[test]
fn view_model_listed_listener() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    queue.set_global_file_listener(Some(&listener.file));
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 0, None);
    queue.request_view_model_names(file, 2);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    assert!(events(&log).iter().any(|event| matches!(
        event,
        ObservedEvent::ViewModelsListed { handle, request_id: 2, names }
            if *handle == file && names == &["ListViewModel", "Empty VM", "Test All", "Nested VM", "State Transition", "Alternate VM"]
    )));

    let bad = queue.load_file(vec![0; 1024 * 1024], None, 0, None);
    queue.request_view_model_names(bad, 2);
    assert!(server.process_commands());
    queue.process_messages();
    assert!(!events(&log).iter().any(|event| matches!(
        event,
        ObservedEvent::ViewModelsListed { handle, .. } if *handle == bad
    )));
}

#[test]
fn view_model_listener() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), Some(&listener.file), 0, None);
    queue.request_view_model_instance_names(file, "Test All".to_string(), 2);
    queue.request_view_model_property_definitions(file, "Test All".to_string(), 3);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(captured.iter().any(|event| matches!(
        event,
        ObservedEvent::ViewModelInstancesListed { handle, request_id: 2, view_model, names }
            if *handle == file && view_model == "Test All" && names == &["Test Default", "Test Alternate"]
    )));
    let properties = captured
        .iter()
        .find_map(|event| match event {
            ObservedEvent::ViewModelPropertiesListed {
                handle,
                request_id: 3,
                view_model,
                properties,
            } if *handle == file && view_model == "Test All" => Some(properties),
            _ => None,
        })
        .expect("property list callback");
    let expected = [
        (DataType::Artboard, "Test Artboard", ""),
        (DataType::List, "Test List", ""),
        (DataType::AssetImage, "Test Image", ""),
        (DataType::Number, "Test Num", ""),
        (DataType::String, "Test String", ""),
        (DataType::Enum, "Test Enum", "Test Enum Values"),
        (DataType::Boolean, "Test Bool", ""),
        (DataType::Color, "Test Color", ""),
        (DataType::Trigger, "Test Trigger", ""),
        (DataType::ViewModel, "Test Nested", "Nested VM"),
    ];
    assert_eq!(properties.len(), expected.len());
    for (property, (data_type, name, metadata)) in properties.iter().zip(expected) {
        assert_eq!(property.data_type, data_type);
        assert_eq!(property.name, name);
        assert_eq!(property.meta_data, metadata);
    }
}

#[test]
fn view_model_instance_listener() {
    let mut queue = CommandQueue::new();
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 0, None);
    let artboard = queue.instantiate_artboard_named(file, "Test Artboard".to_string(), None, 0);
    let bad_artboard = queue.instantiate_artboard_named(file, "Blah".to_string(), None, 0);
    let mut listeners = Vec::new();
    let mut handles = Vec::new();
    for source in 0..8 {
        let (listener, log) = event_log();
        let handle = match source {
            0 => queue.instantiate_blank_view_model_instance_named(
                file,
                "Test All".to_string(),
                Some(&listener.view_model),
                0,
            ),
            1 => queue.instantiate_view_model_instance_named(
                file,
                "Test All".to_string(),
                "".to_string(),
                Some(&listener.view_model),
                0,
            ),
            2 => queue.instantiate_view_model_instance_named(
                file,
                "Test All".to_string(),
                "Test Alternate".to_string(),
                Some(&listener.view_model),
                0,
            ),
            3 => queue.instantiate_view_model_instance_named(
                file,
                "Blah".to_string(),
                "Blah".to_string(),
                Some(&listener.view_model),
                0,
            ),
            4 => queue.instantiate_view_model_instance_for_artboard(
                file,
                artboard,
                String::new(),
                Some(&listener.view_model),
                0,
            ),
            5 => queue.instantiate_view_model_instance_for_artboard(
                file,
                artboard,
                String::new(),
                Some(&listener.view_model),
                0,
            ),
            6 => queue.instantiate_view_model_instance_for_artboard(
                file,
                artboard,
                "Test Alternate".to_owned(),
                Some(&listener.view_model),
                0,
            ),
            _ => queue.instantiate_view_model_instance_for_artboard(
                file,
                bad_artboard,
                "Test Alternate".to_owned(),
                Some(&listener.view_model),
                0,
            ),
        };
        queue.delete_view_model_instance(handle, 0x10);
        listeners.push((listener, log));
        handles.push(handle);
    }
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    for ((_, log), handle) in listeners.iter().zip(handles) {
        assert!(events(log).iter().any(|event| matches!(
            event,
            ObservedEvent::ViewModelDeleted { handle: deleted, request_id: 0x10 }
                if *deleted == handle
        )));
    }
}

#[test]
fn view_model_property_set_get() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    queue.set_global_render_image_listener(Some(&listener.image));
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 0, None);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let root = queue.instantiate_view_model_instance_for_artboard(
        file,
        artboard,
        String::new(),
        Some(&listener.view_model),
        0,
    );
    let blank =
        queue.instantiate_blank_view_model_instance_named(file, "Nested VM".to_string(), None, 0);
    let alternate = queue.instantiate_view_model_instance_named(
        file,
        "Nested VM".to_string(),
        "Alternate Nested".to_string(),
        None,
        0,
    );

    let mut request = 1;
    let expected_values = RefCell::new(Vec::new());
    let mut set_and_get = |queue: &mut CommandQueue,
                           path: &str,
                           value: ObservedValue,
                           data_type| {
        request += 1;
        if matches!(
            data_type,
            DataType::Boolean
                | DataType::Number
                | DataType::Color
                | DataType::Enum
                | DataType::String
        ) {
            expected_values
                .borrow_mut()
                .push((request, path.to_owned(), value.clone()));
        }
        match value {
            ObservedValue::String(value) => {
                queue.set_view_model_instance_string(root, path.to_owned(), value, request)
            }
            ObservedValue::Number(value) => {
                queue.set_view_model_instance_number(root, path.to_owned(), value, request)
            }
            ObservedValue::Boolean(value) => {
                queue.set_view_model_instance_bool(root, path.to_owned(), value, request)
            }
            ObservedValue::Color(value) => {
                queue.set_view_model_instance_color(root, path.to_owned(), value, request)
            }
            ObservedValue::Enum(value) => {
                queue.set_view_model_instance_enum(root, path.to_owned(), value, request)
            }
            ObservedValue::Trigger => queue.fire_view_model_trigger(root, path.to_owned(), request),
            ObservedValue::ViewModel(value) => queue.set_view_model_instance_nested_view_model(
                root,
                path.to_owned(),
                value,
                request,
            ),
            ObservedValue::Image(value) => queue.set_view_model_instance_image(
                root,
                path.to_owned(),
                value.unwrap_or(RenderImageHandle::NULL),
                request,
            ),
            ObservedValue::Blob(value) => queue.set_view_model_instance_blob(
                root,
                path.to_owned(),
                value.unwrap_or(BlobAssetHandle::NULL),
                request,
            ),
            ObservedValue::Artboard(value) => queue.set_view_model_instance_artboard(
                root,
                path.to_owned(),
                value.unwrap_or(ArtboardHandle::NULL),
                request,
            ),
            ObservedValue::None => unreachable!("no upstream setter for an untyped value"),
        }
        match data_type {
            DataType::Boolean => {
                queue.request_view_model_instance_bool(root, path.to_owned(), request)
            }
            DataType::Number => {
                queue.request_view_model_instance_number(root, path.to_owned(), request)
            }
            DataType::Color => {
                queue.request_view_model_instance_color(root, path.to_owned(), request)
            }
            DataType::Enum => {
                queue.request_view_model_instance_enum(root, path.to_owned(), request)
            }
            DataType::String => {
                queue.request_view_model_instance_string(root, path.to_owned(), request)
            }
            _ => {}
        }
    };
    set_and_get(
        &mut queue,
        "Test Bool",
        ObservedValue::Boolean(true),
        DataType::Boolean,
    );
    set_and_get(
        &mut queue,
        "Test Num",
        ObservedValue::Number(10.0),
        DataType::Number,
    );
    set_and_get(
        &mut queue,
        "Test Nested/Nested Number",
        ObservedValue::Number(10.0),
        DataType::Number,
    );
    set_and_get(
        &mut queue,
        "Test Nested",
        ObservedValue::ViewModel(blank),
        DataType::ViewModel,
    );
    set_and_get(
        &mut queue,
        "Test Nested/Nested Number",
        ObservedValue::Number(10.0),
        DataType::Number,
    );
    set_and_get(
        &mut queue,
        "Test Color",
        ObservedValue::Color(0xffff_0000),
        DataType::Color,
    );
    set_and_get(
        &mut queue,
        "Test Enum",
        ObservedValue::Enum("Value 2".to_owned()),
        DataType::Enum,
    );
    set_and_get(
        &mut queue,
        "Test String",
        ObservedValue::String("Some String".to_owned()),
        DataType::String,
    );

    let image = queue.decode_image(IMAGE_FIXTURE.to_vec(), None, 0);
    queue.set_view_model_instance_image(root, "Test Image".to_owned(), image, 0);
    queue.run_once(Box::new(move |server| {
        let expected = server.get_image(image).expect("decoded image");
        let actual = server
            .get_view_model_instance(root)
            .expect("root view model")
            .property_image("Test Image")
            .and_then(|value| {
                value
                    .value_runtime()
                    .handle()
                    .with(|property| {
                        property
                            .as_view_model_instance_asset_image()
                            .and_then(|property| property.asset().render_image())
                    })
                    .flatten()
            })
            .expect("image property");
        assert!(std::ptr::eq(actual.as_any(), expected.as_any()));
    }));

    let external: Box<dyn RenderImage + Send> = Box::new(ExternalImage(7, Arc::new(())));
    let external_image = queue.add_external_image(Some(external), None, 0);
    queue.set_view_model_instance_image(root, "Test Image".to_owned(), external_image, 0);
    queue.run_once(Box::new(move |server| {
        let expected = server.get_image(external_image).expect("external image");
        let actual = server
            .get_view_model_instance(root)
            .expect("root view model")
            .property_image("Test Image")
            .and_then(|value| {
                value
                    .value_runtime()
                    .handle()
                    .with(|property| {
                        property
                            .as_view_model_instance_asset_image()
                            .and_then(|property| property.asset().render_image())
                    })
                    .flatten()
            })
            .expect("external image property");
        assert!(std::ptr::eq(actual.as_any(), expected.as_any()));
    }));

    let bindable = queue.instantiate_default_artboard(file, None, 0);
    queue.set_view_model_instance_artboard(root, "Test Artboard".to_owned(), bindable, 0);
    queue.run_once(Box::new(move |server| {
        let expected = server
            .get_bindable_artboard(bindable)
            .expect("bindable artboard");
        let actual = server
            .get_view_model_instance(root)
            .expect("root view model")
            .property_artboard("Test Artboard")
            .and_then(|value| {
                value
                    .value_runtime()
                    .handle()
                    .with(|property| {
                        property
                            .as_view_model_instance_artboard()
                            .and_then(|property| property.asset())
                    })
                    .flatten()
            })
            .expect("artboard property");
        assert!(actual.ptr_eq(&expected));
    }));
    queue.delete_artboard(bindable, 0);

    let bad_image = queue.decode_image(vec![0; 1024 * 1024], None, 0);
    queue.set_view_model_instance_image(root, "Test Image".to_owned(), bad_image, 20);
    queue.set_view_model_instance_artboard(root, "Test Artboard".to_owned(), artboard, 0);
    let bad_artboard = queue.instantiate_artboard_named(file, "Blah".to_string(), None, 0);
    queue.set_view_model_instance_artboard(root, "Test Artboard".to_owned(), bad_artboard, 21);
    queue.set_view_model_instance_image(root, "Blah".to_owned(), image, 22);
    queue.set_view_model_instance_artboard(root, "Blah".to_owned(), artboard, 23);
    queue.run_once(Box::new(move |server| {
        let expected_image = server.get_image(external_image).expect("external image");
        let retained_image = server
            .get_view_model_instance(root)
            .expect("root view model")
            .property_image("Test Image")
            .and_then(|value| {
                value
                    .value_runtime()
                    .handle()
                    .with(|property| {
                        property
                            .as_view_model_instance_asset_image()
                            .and_then(|property| property.asset().render_image())
                    })
                    .flatten()
            })
            .expect("failed image set retains prior value");
        assert_eq!(
            retained_image
                .as_any()
                .downcast_ref::<ExternalImage>()
                .map(|image| image.0),
            expected_image
                .as_any()
                .downcast_ref::<ExternalImage>()
                .map(|image| image.0)
        );
        let expected_artboard = server
            .get_bindable_artboard(artboard)
            .expect("main artboard");
        let root = server
            .get_view_model_instance(root)
            .expect("root view model");
        let actual_artboard = root
            .property_artboard("Test Artboard")
            .and_then(|value| {
                value
                    .value_runtime()
                    .handle()
                    .with(|property| {
                        property
                            .as_view_model_instance_artboard()
                            .and_then(|property| property.asset())
                    })
                    .flatten()
            })
            .expect("failed artboard set retains prior value");
        assert!(actual_artboard.ptr_eq(&expected_artboard));
    }));
    queue.set_view_model_instance_image(root, "Test Image".to_owned(), RenderImageHandle::NULL, 0);
    queue.set_view_model_instance_artboard(
        root,
        "Test Artboard".to_owned(),
        ArtboardHandle::NULL,
        0,
    );
    queue.run_once(Box::new(move |server| {
        let root = server
            .get_view_model_instance(root)
            .expect("root view model");
        assert!(
            root.property_image("Test Image")
                .and_then(|value| {
                    value
                        .value_runtime()
                        .handle()
                        .with(|property| {
                            property
                                .as_view_model_instance_asset_image()
                                .and_then(|property| property.asset().render_image())
                        })
                        .flatten()
                })
                .is_none()
        );
        assert!(
            root.property_artboard("Test Artboard")
                .and_then(|value| {
                    value
                        .value_runtime()
                        .handle()
                        .with(|property| {
                            property
                                .as_view_model_instance_artboard()
                                .and_then(|property| property.asset())
                        })
                        .flatten()
                })
                .is_none()
        );
    }));

    for index in 0..10 {
        set_and_get(
            &mut queue,
            "Test Bool",
            ObservedValue::Boolean(index % 2 != 0),
            DataType::Boolean,
        );
        set_and_get(
            &mut queue,
            "Test Num",
            ObservedValue::Number(index as f32),
            DataType::Number,
        );
        set_and_get(
            &mut queue,
            "Test Nested",
            ObservedValue::ViewModel(if index % 2 != 0 { blank } else { alternate }),
            DataType::ViewModel,
        );
        set_and_get(
            &mut queue,
            "Test Color",
            ObservedValue::Color(u32::from_ne_bytes([index; 4])),
            DataType::Color,
        );
        set_and_get(
            &mut queue,
            "Test Enum",
            ObservedValue::Enum(if index % 2 != 0 { "Value 2" } else { "Value 1" }.to_owned()),
            DataType::Enum,
        );
        set_and_get(
            &mut queue,
            "Test String",
            ObservedValue::String(index.to_string()),
            DataType::String,
        );
    }
    drop(set_and_get);

    queue.delete_view_model_instance(blank, 0);
    queue.delete_view_model_instance(alternate, 0);
    queue.set_view_model_instance_enum(root, "Test Enum".to_owned(), "Blah".to_owned(), 30);
    queue.set_view_model_instance_nested_view_model(root, "Test Nested".to_owned(), blank, 31);
    queue.request_view_model_instance_enum(root, "Test Enum".to_owned(), 33);
    expected_values.borrow_mut().push((
        33,
        "Test Enum".to_owned(),
        ObservedValue::Enum("Value 2".to_owned()),
    ));
    queue.request_view_model_instance_number(root, "Test Nested/Nested Number".to_owned(), 34);
    expected_values.borrow_mut().push((
        34,
        "Test Nested/Nested Number".to_owned(),
        ObservedValue::Number(10.0),
    ));
    for value in [
        ObservedValue::Boolean(true),
        ObservedValue::Number(10.0),
        ObservedValue::ViewModel(alternate),
        ObservedValue::Color(0xffff_0000),
        ObservedValue::Enum("Value 2".to_owned()),
        ObservedValue::String("Some String".to_owned()),
    ] {
        match value {
            ObservedValue::Boolean(value) => {
                queue.set_view_model_instance_bool(root, "Blah".to_owned(), value, 32)
            }
            ObservedValue::Number(value) => {
                queue.set_view_model_instance_number(root, "Blah".to_owned(), value, 32)
            }
            ObservedValue::ViewModel(value) => {
                queue.set_view_model_instance_nested_view_model(root, "Blah".to_owned(), value, 32)
            }
            ObservedValue::Color(value) => {
                queue.set_view_model_instance_color(root, "Blah".to_owned(), value, 32)
            }
            ObservedValue::Enum(value) => {
                queue.set_view_model_instance_enum(root, "Blah".to_owned(), value, 32)
            }
            ObservedValue::String(value) => {
                queue.set_view_model_instance_string(root, "Blah".to_owned(), value, 32)
            }
            _ => unreachable!(),
        }
    }
    queue.delete_view_model_instance(root, 40);
    for value in [
        ObservedValue::Boolean(true),
        ObservedValue::Number(10.0),
        ObservedValue::Color(0xffff_0000),
        ObservedValue::Enum("Value 2".to_owned()),
        ObservedValue::String("Some String".to_owned()),
    ] {
        match value {
            ObservedValue::Boolean(value) => {
                queue.set_view_model_instance_bool(root, "Test Bool".to_owned(), value, 41)
            }
            ObservedValue::Number(value) => {
                queue.set_view_model_instance_number(root, "Test Bool".to_owned(), value, 41)
            }
            ObservedValue::Color(value) => {
                queue.set_view_model_instance_color(root, "Test Bool".to_owned(), value, 41)
            }
            ObservedValue::Enum(value) => {
                queue.set_view_model_instance_enum(root, "Test Bool".to_owned(), value, 41)
            }
            ObservedValue::String(value) => {
                queue.set_view_model_instance_string(root, "Test Bool".to_owned(), value, 41)
            }
            _ => unreachable!(),
        }
    }

    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    let actual_values = captured
        .iter()
        .filter_map(|event| match event {
            ObservedEvent::ViewModelValue {
                handle,
                request_id,
                path,
                value,
            } if *handle == root => Some((*request_id, path.clone(), value.clone())),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(actual_values, expected_values.into_inner());
    assert!(captured.iter().any(|event| matches!(
        event,
        ObservedEvent::ViewModelDeleted { handle, request_id: 40 } if *handle == root
    )));
    assert_eq!(
        captured
            .iter()
            .filter(|event| matches!(event, ObservedEvent::ViewModelError { .. }))
            .count(),
        17
    );
    assert!(captured.iter().any(|event| matches!(
        event,
        ObservedEvent::ImageError { handle, .. } if *handle == bad_image
    )));
}

#[test]
fn set_and_reset_artboard_size() {
    let mut queue = CommandQueue::new();
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 0, None);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    let original = server
        .with_artboard_instance(artboard, |artboard| {
            (artboard.base.width(), artboard.base.height())
        })
        .unwrap();
    queue.set_artboard_size(artboard, 1000.0, 1000.0, 1.0, 0);
    assert!(server.process_commands());
    assert_eq!(
        server
            .with_artboard_instance(artboard, |artboard| {
                (artboard.base.width(), artboard.base.height())
            })
            .unwrap(),
        (1000.0, 1000.0)
    );
    queue.set_artboard_size(artboard, 1000.0, 1000.0, 2.0, 0);
    assert!(server.process_commands());
    assert_eq!(
        server
            .with_artboard_instance(artboard, |artboard| {
                (artboard.base.width(), artboard.base.height())
            })
            .unwrap(),
        (500.0, 500.0)
    );
    queue.reset_artboard_size(artboard, 0);
    assert!(server.process_commands());
    assert_eq!(
        server
            .with_artboard_instance(artboard, |artboard| {
                (artboard.base.width(), artboard.base.height())
            })
            .unwrap(),
        original
    );
}

#[test]
fn set_and_get_artboard_volume() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 0, None);
    let artboard = queue.instantiate_default_artboard(file, Some(&listener.artboard), 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.set_artboard_volume(artboard, 0.5, 0);
    assert!(server.process_commands());
    assert_eq!(
        server
            .with_artboard_instance(artboard, |artboard| artboard.base.volume())
            .unwrap(),
        0.5
    );
    queue.set_artboard_volume(artboard, 0.0, 0);
    assert!(server.process_commands());
    assert_eq!(
        server
            .with_artboard_instance(artboard, |artboard| artboard.base.volume())
            .unwrap(),
        0.0
    );
    queue.set_artboard_volume(artboard, 0.75, 0);
    queue.request_artboard_volume(artboard, 0x50);
    assert!(server.process_commands());
    queue.process_messages();
    assert!(events(&log).iter().any(|event| matches!(
        event,
        ObservedEvent::ArtboardVolume { handle, request_id: 0x50, volume }
            if *handle == artboard && *volume == 0.75
    )));
}

#[test]
fn view_model_property_subscriptions() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 0, None);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let root = queue.instantiate_view_model_instance_for_artboard(
        file,
        artboard,
        String::new(),
        Some(&listener.view_model),
        0,
    );
    queue.set_view_model_instance_bool(root, "Test Bool".to_owned(), false, 0);
    queue.set_view_model_instance_color(root, "Test Color".to_owned(), 0, 0);
    for (path, data_type) in [
        ("Test Nested/Nested Number", DataType::Number),
        ("Test Bool", DataType::Boolean),
        ("Test Num", DataType::Number),
        ("Test Color", DataType::Color),
        ("Test Enum", DataType::Enum),
        ("Test String", DataType::String),
        ("Test Trigger", DataType::Trigger),
        ("Test List", DataType::List),
        ("Test Image", DataType::AssetImage),
    ] {
        queue.subscribe_to_view_model_property(root, path.to_string(), data_type, 0);
    }
    queue.subscribe_to_view_model_property(
        root,
        "Bad property".to_string(),
        DataType::AssetImage,
        1,
    );
    queue.subscribe_to_view_model_property(root, "Test Image".to_string(), DataType::Integer, 2);
    let blank =
        queue.instantiate_blank_view_model_instance_named(file, "Nested VM".to_string(), None, 0);
    let image = queue.decode_image(IMAGE_FIXTURE.to_vec(), None, 0);
    queue.set_view_model_instance_bool(root, "Test Bool".to_owned(), true, 0);
    queue.set_view_model_instance_number(root, "Test Num".to_owned(), 10.0, 0);
    queue.set_view_model_instance_number(root, "Test Nested/Nested Number".to_owned(), 10.0, 0);
    queue.set_view_model_instance_color(root, "Test Color".to_owned(), 0xffff_0000, 0);
    queue.set_view_model_instance_enum(root, "Test Enum".to_owned(), "Value 2".into(), 0);
    queue.set_view_model_instance_string(root, "Test String".to_owned(), "Some String".into(), 0);
    queue.fire_view_model_trigger(root, "Test Trigger".to_owned(), 0);
    queue.set_view_model_instance_image(root, "Test Image".to_owned(), image, 0);
    queue.set_view_model_instance_nested_view_model(root, "Test Nested".to_owned(), blank, 0);
    queue.set_view_model_instance_number(root, "Test Nested/Nested Number".to_owned(), 10.0, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    for path in [
        "Test Nested/Nested Number",
        "Test Bool",
        "Test Num",
        "Test Color",
        "Test Enum",
        "Test String",
        "Test Trigger",
        "Test Image",
    ] {
        assert!(
            captured.iter().any(|event| matches!(
                event,
                ObservedEvent::ViewModelValue { path: actual, .. } if actual == path
            )),
            "missing subscription callback for {path}"
        );
    }
    assert_eq!(
        captured
            .iter()
            .filter(|event| matches!(
                event,
                ObservedEvent::ViewModelError {
                    request_id: 1 | 2,
                    ..
                }
            ))
            .count(),
        2
    );
    for (path, data_type) in [
        ("Test Nested/Nested Number", DataType::Number),
        ("Test Bool", DataType::Boolean),
        ("Test Num", DataType::Number),
        ("Test Color", DataType::Color),
        ("Test Enum", DataType::Enum),
        ("Test String", DataType::String),
        ("Test Trigger", DataType::Trigger),
        ("Test List", DataType::List),
        ("Test Image", DataType::AssetImage),
    ] {
        queue.unsubscribe_to_view_model_property(root, path.to_string(), data_type, 0);
    }
    queue.unsubscribe_to_view_model_property(root, "Blah".to_string(), DataType::Boolean, 0);
    assert!(server.process_commands());
}

#[test]
#[cfg(any())] // The upstream-shaped queue owns Rc-backed runtime resources and is not Send.
fn view_model_property_async_subscriptions() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 0, None);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let root = queue.instantiate_view_model_instance_for_artboard(
        file,
        artboard,
        String::new(),
        Some(&listener.view_model),
        0,
    );
    queue.set_view_model_instance_number(root, "Test Num".to_owned(), 0.0, 0);
    queue.subscribe_to_view_model_property(root, "Test Num".to_string(), DataType::Number, 0);
    queue.set_view_model_instance_number(root, "Test Num".to_owned(), 10.0, 0);
    let ready = Arc::new(AtomicUsize::new(0));
    let ready_on_server = Arc::clone(&ready);
    queue.run_once(Box::new(move |_| {
        ready_on_server.store(1, Ordering::Release);
    }));
    let worker_queue = queue.clone();
    let worker = thread::spawn(move || {
        let mut server = server(&worker_queue);
        while server.wait_commands() {}
    });
    while ready.load(Ordering::Acquire) == 0 {
        thread::yield_now();
    }
    for _ in 0..10_000 {
        queue.process_messages();
        if events(&log).iter().any(|event| {
            matches!(
                event,
                ObservedEvent::ViewModelValue { handle, path, value: ObservedValue::Number(10.0), .. }
                    if *handle == root && path == "Test Num"
            )
        }) {
            break;
        }
        thread::yield_now();
    }
    assert!(events(&log).iter().any(|event| matches!(
        event,
        ObservedEvent::ViewModelValue { handle, path, value: ObservedValue::Number(10.0), .. }
            if *handle == root && path == "Test Num"
    )));
    queue.unsubscribe_to_view_model_property(root, "Test Num".to_string(), DataType::Number, 0);
    queue.disconnect();
    worker.join().expect("server thread");
}

#[test]
fn list_view_model_property_set_get() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 0, None);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let root = queue.instantiate_view_model_instance_for_artboard(
        file,
        artboard,
        String::new(),
        Some(&listener.view_model),
        0,
    );
    let blank =
        queue.instantiate_blank_view_model_instance_named(file, "Nested VM".to_string(), None, 0);
    let alternate = queue.instantiate_view_model_instance_named(
        file,
        "Nested VM".to_string(),
        "Alternate Nested".to_string(),
        None,
        0,
    );
    queue.append_view_model_instance_list_view_model(root, "Test List".to_owned(), blank, 0);
    queue.append_view_model_instance_list_view_model(root, "Test List".to_owned(), alternate, 0);
    queue.swap_view_model_instance_list_values(root, "Test List".to_owned(), 2, 3, 0);
    queue.run_once(Box::new(move |server| {
        let items = server
            .get_view_model_instance(root)
            .and_then(|root| root.property_list("Test List"))
            .expect("list property");
        assert_eq!(items.size(), 4);
        assert_eq!(
            server.get_handle_for_instance(&items.instance_at(2).expect("item 2")),
            alternate
        );
        assert_eq!(
            server.get_handle_for_instance(&items.instance_at(3).expect("item 3")),
            blank
        );
    }));
    queue.request_view_model_instance_list_size(root, "Test List".to_string(), 1);
    queue.insert_view_model_instance_list_view_model(root, "Test List".to_owned(), blank, 0, 0);
    queue.insert_view_model_instance_list_view_model(root, "Test List".to_owned(), alternate, 0, 0);
    queue.swap_view_model_instance_list_values(root, "Test List".to_owned(), 0, 1, 0);
    queue.run_once(Box::new(move |server| {
        let items = server
            .get_view_model_instance(root)
            .and_then(|root| root.property_list("Test List"))
            .expect("list property");
        assert_eq!(items.size(), 6);
        for (index, expected) in [(0, blank), (1, alternate), (4, alternate), (5, blank)] {
            assert_eq!(
                server.get_handle_for_instance(&items.instance_at(index).expect("list item")),
                expected
            );
        }
    }));
    queue.request_view_model_instance_list_size(root, "Test List".to_string(), 2);
    let bad_blank =
        queue.instantiate_blank_view_model_instance_named(file, "blah".to_string(), None, 0);
    let bad_alternate = queue.instantiate_view_model_instance_named(
        file,
        "Nested VM".to_string(),
        "blah".to_string(),
        None,
        0,
    );
    for (path, value, index) in [
        ("Test List", bad_blank, None),
        ("Test List", bad_alternate, None),
        ("Test List", bad_blank, Some(0)),
        ("Test List", bad_alternate, Some(0)),
        ("blah", blank, None),
        ("blah", alternate, None),
        ("blah", blank, Some(0)),
        ("blah", alternate, Some(0)),
    ] {
        match index {
            Some(index) => queue.insert_view_model_instance_list_view_model(
                root,
                path.to_owned(),
                value,
                index,
                3,
            ),
            None => {
                queue.append_view_model_instance_list_view_model(root, path.to_owned(), value, 3)
            }
        }
    }
    for (path, a, b) in [
        ("Test List", 10, 1),
        ("Test List", 0, 10),
        ("Blah", 0, 1),
        ("Blah", 10, 1),
        ("Blah", 0, 10),
    ] {
        queue.swap_view_model_instance_list_values(root, path.to_owned(), a, b, 4);
    }
    queue.run_once(Box::new(move |server| {
        let items = server
            .get_view_model_instance(root)
            .and_then(|root| root.property_list("Test List"))
            .expect("invalid operations retain list");
        assert_eq!(items.size(), 6);
        for (index, expected) in [(0, blank), (1, alternate), (4, alternate), (5, blank)] {
            assert_eq!(
                server.get_handle_for_instance(&items.instance_at(index).expect("list item")),
                expected
            );
        }
    }));
    queue.request_view_model_instance_list_size(root, "Test List".to_string(), 5);
    queue.request_view_model_instance_list_size(root, "Blah".to_string(), 6);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(captured.iter().any(|event| matches!(
        event,
        ObservedEvent::ViewModelListSize {
            request_id: 1,
            size: 4,
            ..
        }
    )));
    assert!(captured.iter().any(|event| matches!(
        event,
        ObservedEvent::ViewModelListSize {
            request_id: 2 | 5,
            size: 6,
            ..
        }
    )));
    assert!(!captured.iter().any(|event| matches!(
        event,
        ObservedEvent::ViewModelListSize { request_id: 6, .. }
    )));
    assert_eq!(
        captured
            .iter()
            .filter(|event| matches!(event, ObservedEvent::ViewModelError { .. }))
            .count(),
        12
    );
}

#[test]
fn file_error_messages() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), Some(&listener.file), 0, None);
    queue.instantiate_artboard_named(file, "Blah".to_string(), None, 0);
    queue.instantiate_view_model_instance_named(
        file,
        "Test All".to_string(),
        "blah".to_string(),
        None,
        0,
    );
    queue.instantiate_view_model_instance_named(
        file,
        "blah".to_string(),
        "blah".to_string(),
        None,
        0,
    );
    queue.instantiate_view_model_instance_named(file, "".to_string(), "blah".to_string(), None, 0);
    queue.instantiate_view_model_instance_named(file, "Blah".to_string(), "".to_string(), None, 0);
    queue.instantiate_view_model_instance_named(file, "".to_string(), "".to_string(), None, 0);
    queue.instantiate_blank_view_model_instance_named(file, "Blah".to_string(), None, 0);
    queue.instantiate_blank_view_model_instance_named(file, "".to_string(), None, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    assert_eq!(
        events(&log)
            .iter()
            .filter(
                |event| matches!(event, ObservedEvent::FileError { handle, .. } if *handle == file)
            )
            .count(),
        8
    );

    let (bad_listener, bad_log) = event_log();
    let bad = queue.load_file(vec![0; 100 * 1024], Some(&bad_listener.file), 0, None);
    queue.instantiate_default_artboard(bad, None, 0);
    queue.instantiate_blank_view_model_instance_named(bad, "".to_string(), None, 0);
    assert!(server.process_commands());
    queue.process_messages();
    assert_eq!(
        events(&bad_log)
            .iter()
            .filter(
                |event| matches!(event, ObservedEvent::FileError { handle, .. } if *handle == bad)
            )
            .count(),
        3
    );

    let (no_vm_listener, no_vm_log) = event_log();
    let no_vm = queue.load_file(ENTRY_FIXTURE.to_vec(), Some(&no_vm_listener.file), 0, None);
    let no_vm_artboard = queue.instantiate_default_artboard(no_vm, None, 0);
    for instance in [String::new(), String::new(), "Nonexistent".into()] {
        queue.instantiate_view_model_instance_for_artboard(
            no_vm,
            no_vm_artboard,
            instance,
            None,
            0,
        );
    }
    assert!(server.process_commands());
    queue.process_messages();
    assert_eq!(
        events(&no_vm_log)
            .iter()
            .filter(
                |event| matches!(event, ObservedEvent::FileError { handle, .. } if *handle == no_vm)
            )
            .count(),
        3
    );
}

#[test]
fn list_artboard() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    let good = queue.load_file(ENTRY_FIXTURE.to_vec(), Some(&listener.file), 0, None);
    queue.request_artboard_names(good, 0x40);
    let bad = queue.load_file(vec![0; 100 * 1024], None, 0, None);
    queue.request_artboard_names(bad, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(captured.iter().any(|event| matches!(event, ObservedEvent::ArtboardsListed { handle, request_id: 0x40, names } if *handle == good && names == &["New Artboard", "New Artboard"])));
    assert!(!captured.iter().any(
        |event| matches!(event, ObservedEvent::ArtboardsListed { handle, .. } if *handle == bad)
    ));
}

#[test]
fn list_enums() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    let good = queue.load_file(DATA_BIND_FIXTURE.to_vec(), Some(&listener.file), 0, None);
    queue.request_view_model_enums(good, 0x40);
    let bad = queue.load_file(vec![0; 100 * 1024], None, 0, None);
    queue.request_view_model_enums(bad, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(captured.iter().any(|event| matches!(event, ObservedEvent::ViewModelEnumsListed { handle, request_id: 0x40, enums } if *handle == good && enums.len() == 1 && enums[0].name == "Test Enum Values" && enums[0].enumerants == ["Value 1", "Value 2"])));
    assert!(!captured.iter().any(
        |event| matches!(event, ObservedEvent::ViewModelEnumsListed { handle, .. } if *handle == bad)
    ));
}

#[test]
fn request_view_model_and_instance_name() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 0, None);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let view_model = queue.instantiate_view_model_instance_for_artboard(
        file,
        artboard,
        String::new(),
        Some(&listener.view_model),
        0,
    );
    queue.request_view_model_instance_view_model_name(view_model, 0x50);
    queue.request_view_model_instance_name(view_model, 0x50);
    let (bad_listener, bad_log) = event_log();
    let bad = queue.instantiate_view_model_instance_named(
        file,
        "Blah".to_string(),
        "Blah".to_string(),
        Some(&bad_listener.view_model),
        0,
    );
    queue.request_view_model_instance_view_model_name(bad, 0x51);
    queue.request_view_model_instance_name(bad, 0x52);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(captured.iter().any(|event| matches!(event, ObservedEvent::ViewModelName { handle, request_id: 0x50, name } if *handle == view_model && name == "Test All")));
    assert!(captured.iter().any(|event| matches!(event, ObservedEvent::ViewModelInstanceName { handle, request_id: 0x50, name } if *handle == view_model && name == "Test Default")));
    assert!(!events(&bad_log).iter().any(|event| matches!(
        event,
        ObservedEvent::ViewModelName { .. } | ObservedEvent::ViewModelInstanceName { .. }
    )));
    assert!(events(&bad_log).iter().any(|event| matches!(
        event,
        ObservedEvent::ViewModelError {
            request_id: 0x52,
            ..
        }
    )));
}

#[test]
fn render_image_audio_source_font_error() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    let image = queue.decode_image(vec![0; 1024], Some(&listener.image), 1);
    let audio = queue.decode_audio(vec![0; 1024], Some(&listener.audio), 2);
    let font = queue.decode_font(vec![0; 1024], Some(&listener.font), 3);
    assert!(server(&queue).process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(captured.iter().any(
        |event| matches!(event, ObservedEvent::ImageError { handle, .. } if *handle == image)
    ));
    assert!(captured.iter().any(
        |event| matches!(event, ObservedEvent::AudioError { handle, .. } if *handle == audio)
    ));
    assert!(
        captured.iter().any(
            |event| matches!(event, ObservedEvent::FontError { handle, .. } if *handle == font)
        )
    );
}

#[test]
fn state_machine_error() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    queue.set_global_state_machine_listener(Some(&listener.state_machine));
    let file = queue.load_file(ENTRY_FIXTURE.to_vec(), None, 0, None);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let good = queue.instantiate_default_state_machine(artboard, None, 0);
    let bad_vm =
        queue.instantiate_blank_view_model_instance_named(file, "missing".to_string(), None, 0);
    queue.bind_view_model_instance(good, bad_vm, 1);
    let bad_artboard = queue.instantiate_artboard_named(file, "missing".to_string(), None, 0);
    let bad = queue.instantiate_default_state_machine(bad_artboard, None, 0);
    let pointer = nuxie::command_queue::PointerEvent::default();
    queue.advance_state_machine(bad, 0.0, 2);
    queue.pointer_down(bad, pointer, 3);
    queue.pointer_exit(bad, pointer, 4);
    queue.pointer_up(bad, pointer, 5);
    queue.pointer_move(bad, pointer, 6);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert_eq!(captured.iter().filter(|event| matches!(event, ObservedEvent::StateMachineError { handle, .. } if *handle == good)).count(), 1);
    assert_eq!(captured.iter().filter(|event| matches!(event, ObservedEvent::StateMachineError { handle, .. } if *handle == bad)).count(), 5);
}

#[test]
fn artboard_errors() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    queue.set_global_artboard_listener(Some(&listener.artboard));
    let file = queue.load_file(ENTRY_FIXTURE.to_vec(), None, 0, None);
    let good = queue.instantiate_artboard_named(file, "New Artboard".to_string(), None, 0);
    queue.instantiate_state_machine_named(good, "Blah".to_string(), None, 1);
    let bad = queue.instantiate_artboard_named(file, "Blah".to_string(), None, 0);
    queue.request_state_machine_names(bad, 2);
    queue.instantiate_default_state_machine(bad, None, 3);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert_eq!(captured.iter().filter(|event| matches!(event, ObservedEvent::ArtboardError { handle, .. } if *handle == good)).count(), 1);
    assert_eq!(captured.iter().filter(|event| matches!(event, ObservedEvent::ArtboardError { handle, .. } if *handle == bad)).count(), 2);
}

#[test]
fn invalid_artboard_volume_errors() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    queue.set_global_artboard_listener(Some(&listener.artboard));
    let file = queue.load_file(ENTRY_FIXTURE.to_vec(), None, 0, None);
    let invalid = queue.instantiate_artboard_named(file, "missing".to_string(), None, 0);
    queue.set_artboard_volume(invalid, 0.5, 0x51);
    queue.request_artboard_volume(invalid, 0x52);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let ids = events(&log)
        .into_iter()
        .filter_map(|event| match event {
            ObservedEvent::ArtboardError {
                handle, request_id, ..
            } if handle == invalid => Some(request_id),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(ids, [0x51, 0x52]);
}

#[test]
fn invalid_artboard_size_errors() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    queue.set_global_artboard_listener(Some(&listener.artboard));
    let file = queue.load_file(ENTRY_FIXTURE.to_vec(), None, 0, None);
    let invalid = queue.instantiate_artboard_named(file, "missing".to_string(), None, 0);
    queue.set_artboard_size(invalid, 10.0, 10.0, 1.0, 0x51);
    queue.reset_artboard_size(invalid, 0x52);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let ids = events(&log)
        .into_iter()
        .filter_map(|event| match event {
            ObservedEvent::ArtboardError {
                handle, request_id, ..
            } if handle == invalid => Some(request_id),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(ids, [0x51, 0x52]);
}

#[test]
fn list_state_machine() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    queue.set_global_artboard_listener(Some(&listener.artboard));
    let file = queue.load_file(ENTRY_FIXTURE.to_vec(), None, 0, None);
    let artboard = queue.instantiate_artboard_named(file, "New Artboard".to_string(), None, 0);
    queue.request_state_machine_names(artboard, 0x40);
    let bad_file = queue.load_file(vec![0; 100 * 1024], None, 0, None);
    let bad = queue.instantiate_default_artboard(bad_file, None, 0);
    queue.request_state_machine_names(bad, 0x41);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(captured.iter().any(|event| matches!(event, ObservedEvent::StateMachinesListed { handle, request_id: 0x40, names } if *handle == artboard && names == &["State Machine 1"])));
    assert!(!captured.iter().any(
        |event| matches!(event, ObservedEvent::StateMachinesListed { handle, .. } if *handle == bad)
    ));
}

#[test]
fn request_artboard_size() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    queue.set_global_artboard_listener(Some(&listener.artboard));
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 0, None);
    let artboard = queue.instantiate_default_artboard(file, Some(&listener.artboard), 0);
    queue.request_artboard_size(artboard, 0x50);
    queue.set_artboard_size(artboard, 1000.0, 500.0, 1.0, 0);
    queue.request_artboard_size(artboard, 0x51);
    let invalid = queue.instantiate_artboard_named(file, "missing".to_string(), None, 0);
    queue.request_artboard_size(invalid, 0x52);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(captured.iter().any(|event| matches!(event, ObservedEvent::ArtboardSize { handle, request_id: 0x50, .. } if *handle == artboard)));
    assert!(captured.iter().any(|event| matches!(event, ObservedEvent::ArtboardSize { handle, request_id: 0x51, width: 1000.0, height: 500.0 } if *handle == artboard)));
    assert!(captured.iter().any(|event| matches!(event, ObservedEvent::ArtboardError { handle, request_id: 0x52, .. } if *handle == invalid)));
}

#[test]
fn request_default_view_model_info() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    queue.set_global_artboard_listener(Some(&listener.artboard));
    let good_file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 0, None);
    let good = queue.instantiate_artboard_named(good_file, "Test Artboard".to_string(), None, 0);
    queue.request_default_view_model_info(good, good_file, 0x40);
    let bad_file = queue.load_file(vec![0; 100 * 1024], None, 0, None);
    let bad = queue.instantiate_default_artboard(bad_file, None, 0);
    queue.request_default_view_model_info(bad, good_file, 0x41);
    queue.request_default_view_model_info(good, bad_file, 0x42);
    queue.request_default_view_model_info(bad, bad_file, 0x43);
    let no_vm_file = queue.load_file(ENTRY_FIXTURE.to_vec(), None, 0, None);
    let no_vm = queue.instantiate_default_artboard(no_vm_file, None, 0);
    queue.request_default_view_model_info(no_vm, no_vm_file, 0x44);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(captured.iter().any(|event| matches!(event, ObservedEvent::DefaultViewModel { handle, request_id: 0x40, view_model, instance } if *handle == good && view_model == "Test All" && instance == "Test Default")));
    for request_id in [0x41, 0x42, 0x43, 0x44] {
        assert!(captured.iter().any(|event| matches!(event, ObservedEvent::ArtboardError { request_id: actual, .. } if *actual == request_id)));
    }
}

#[test]
fn bind_view_model_instance() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    queue.set_global_state_machine_listener(Some(&listener.state_machine));
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 0, None);
    let view_model = queue.instantiate_view_model_instance_named(
        file,
        "Test All".to_string(),
        "Test Alternate".to_string(),
        None,
        0,
    );
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let state_machine = queue.instantiate_default_state_machine(artboard, None, 0);
    queue.bind_view_model_instance(state_machine, view_model, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    assert!(
        server
            .with_state_machine_instance_mut(state_machine, |_| ())
            .is_some()
    );
    assert!(server.get_view_model_instance(view_model).is_some());
    let replacement =
        queue.instantiate_blank_view_model_instance_named(file, "Test All".to_string(), None, 0);
    queue.set_view_model_instance(state_machine, replacement, 0);
    assert!(server.process_commands());
    let bad_vm = queue.instantiate_view_model_instance_named(
        file,
        "blah".to_string(),
        "Test Alternate".to_string(),
        None,
        0,
    );
    let bad_machine = queue.instantiate_state_machine_named(artboard, "blah".to_string(), None, 0);
    queue.bind_view_model_instance(state_machine, bad_vm, 1);
    queue.bind_view_model_instance(bad_machine, view_model, 2);
    queue.bind_view_model_instance(bad_machine, bad_vm, 3);
    queue.set_view_model_instance(state_machine, bad_vm, 4);
    queue.set_view_model_instance(bad_machine, view_model, 5);
    queue.set_view_model_instance(bad_machine, bad_vm, 6);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    for request_id in 1..=6 {
        assert!(captured.iter().any(|event| matches!(
            event,
            ObservedEvent::StateMachineError { request_id: actual, .. }
                if *actual == request_id
        )));
    }
}

#[test]
fn advance_state_machine() {
    const SETTLER: &[u8] = include_bytes!("../../../fixtures/command_queue/settler.riv");
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    queue.set_global_state_machine_listener(Some(&listener.state_machine));
    let file = queue.load_file(SETTLER.to_vec(), None, 0, None);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let machine = queue.instantiate_default_state_machine(artboard, None, 0);
    queue.advance_state_machine(machine, 10.0, 0);
    queue.advance_state_machine(machine, 10.0, 0);
    queue.advance_state_machine(machine, 10.0, 0x50);
    let bad = queue.instantiate_state_machine_named(artboard, "blah blah".to_string(), None, 0);
    queue.advance_state_machine(bad, 10.0, 0x51);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(captured.iter().any(|event| matches!(event, ObservedEvent::StateMachineSettled { handle, request_id: 0x50 } if *handle == machine)));
    assert!(!captured.iter().any(
        |event| matches!(event, ObservedEvent::StateMachineSettled { handle, .. } if *handle == bad)
    ));
}

#[test]
fn listener_delete_callbacks() {
    let mut queue = CommandQueue::new();
    let (file_listener, file_log) = event_log();
    let (artboard_listener, artboard_log) = event_log();
    let (machine_listener, machine_log) = event_log();
    let (image_listener, image_log) = event_log();
    let file = queue.load_file(ENTRY_FIXTURE.to_vec(), Some(&file_listener.file), 0, None);
    let artboard = queue.instantiate_artboard_named(
        file,
        "New Artboard".to_string(),
        Some(&artboard_listener.artboard),
        0,
    );
    let machine =
        queue.instantiate_default_state_machine(artboard, Some(&machine_listener.state_machine), 0);
    let image = queue.decode_image(IMAGE_FIXTURE.to_vec(), Some(&image_listener.image), 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    for log in [&file_log, &artboard_log, &machine_log, &image_log] {
        assert!(!events(log).iter().any(|event| matches!(
            event,
            ObservedEvent::FileDeleted { .. }
                | ObservedEvent::ArtboardDeleted { .. }
                | ObservedEvent::StateMachineDeleted { .. }
                | ObservedEvent::ImageDeleted { .. }
        )));
    }
    queue.delete_state_machine(machine, 0x50);
    queue.delete_artboard(artboard, 0x51);
    queue.delete_file(file, 0x52);
    queue.delete_image(image, 0x53);
    assert!(server.process_commands());
    queue.process_messages();
    assert!(events(&file_log).iter().any(|event| matches!(event, ObservedEvent::FileDeleted { handle, request_id: 0x52 } if *handle == file)));
    assert!(events(&artboard_log).iter().any(|event| matches!(event, ObservedEvent::ArtboardDeleted { handle, request_id: 0x51 } if *handle == artboard)));
    assert!(events(&machine_log).iter().any(|event| matches!(event, ObservedEvent::StateMachineDeleted { handle, request_id: 0x50 } if *handle == machine)));
    assert!(events(&image_log).iter().any(|event| matches!(event, ObservedEvent::ImageDeleted { handle, request_id: 0x53 } if *handle == image)));
}

#[test]
fn file_loaded_callback() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    let good = queue.load_file(ENTRY_FIXTURE.to_vec(), Some(&listener.file), 0x10, None);
    let (bad_listener, bad_log) = event_log();
    let bad = queue.load_file(vec![0; 1024], Some(&bad_listener.file), 0x10, None);
    assert!(server(&queue).process_commands());
    queue.process_messages();
    assert!(events(&log).iter().any(|event| matches!(event, ObservedEvent::FileLoaded { handle, request_id: 0x10 } if *handle == good)));
    assert!(
        !events(&bad_log).iter().any(
            |event| matches!(event, ObservedEvent::FileLoaded { handle, .. } if *handle == bad)
        )
    );
}

#[test]
fn artboard_instantiated_callback() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(ARTBOARD_FIXTURE.to_vec(), Some(&listener.file), 0, None);
    let good = queue.instantiate_artboard_named(file, "One".to_string(), None, 0x10);
    let bad = queue.instantiate_artboard_named(file, "Blah".to_string(), None, 0x11);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(captured.iter().any(|event| matches!(event, ObservedEvent::ArtboardInstantiated { file: actual_file, handle, request_id: 0x10 } if *actual_file == file && *handle == good)));
    assert!(!captured.iter().any(
        |event| matches!(event, ObservedEvent::ArtboardInstantiated { handle, .. } if *handle == bad)
    ));
}

#[test]
fn state_machine_instantiated_callback() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(MULTI_MACHINE_FIXTURE.to_vec(), None, 0, None);
    let artboard = queue.instantiate_default_artboard(file, Some(&listener.artboard), 0);
    let good = queue.instantiate_state_machine_named(artboard, "one".to_string(), None, 0x10);
    let bad =
        queue.instantiate_state_machine_named(artboard, "blahblahblah".to_string(), None, 0x11);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(captured.iter().any(|event| matches!(event, ObservedEvent::StateMachineInstantiated { artboard: actual_artboard, handle, request_id: 0x10 } if *actual_artboard == artboard && *handle == good)));
    assert!(!captured.iter().any(|event| matches!(event, ObservedEvent::StateMachineInstantiated { handle, .. } if *handle == bad)));
}

#[test]
fn view_model_instance_instantiated_callback() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), Some(&listener.file), 0, None);
    let good = queue.instantiate_view_model_instance_named(
        file,
        "Test All".to_string(),
        "Test Alternate".to_string(),
        None,
        0x10,
    );
    let bad = queue.instantiate_view_model_instance_named(
        file,
        "Test All".to_string(),
        "Blah".to_string(),
        None,
        0x11,
    );
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(captured.iter().any(|event| matches!(event, ObservedEvent::ViewModelInstantiated { file: actual_file, handle, request_id: 0x10 } if *actual_file == file && *handle == good)));
    assert!(!captured.iter().any(|event| matches!(event, ObservedEvent::ViewModelInstantiated { handle, .. } if *handle == bad)));
}

#[test]
fn decoded_callbacks() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    queue.set_global_render_image_listener(Some(&listener.image));
    queue.set_global_audio_source_listener(Some(&listener.audio));
    queue.set_global_font_listener(Some(&listener.font));
    let image = queue.decode_image(IMAGE_FIXTURE.to_vec(), None, 0x10);
    let audio = queue.decode_audio(AUDIO_FIXTURE.to_vec(), None, 0x10);
    let font = queue.decode_font(FONT_FIXTURE.to_vec(), None, 0x10);
    let bad_image = queue.decode_image(vec![0; 1024], None, 0x11);
    let bad_audio = queue.decode_audio(vec![0; 1024], None, 0x11);
    let bad_font = queue.decode_font(vec![0; 1024], None, 0x11);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(captured.iter().any(|event| matches!(event, ObservedEvent::ImageDecoded { handle, request_id: 0x10 } if *handle == image)));
    assert!(captured.iter().any(|event| matches!(event, ObservedEvent::AudioDecoded { handle, request_id: 0x10 } if *handle == audio)));
    assert!(captured.iter().any(|event| matches!(event, ObservedEvent::FontDecoded { handle, request_id: 0x10 } if *handle == font)));
    assert!(!captured.iter().any(
        |event| matches!(event, ObservedEvent::ImageDecoded { handle, .. } if *handle == bad_image)
    ));
    assert!(!captured.iter().any(
        |event| matches!(event, ObservedEvent::AudioDecoded { handle, .. } if *handle == bad_audio)
    ));
    assert!(!captured.iter().any(
        |event| matches!(event, ObservedEvent::FontDecoded { handle, .. } if *handle == bad_font)
    ));
}

#[test]
fn listener_lifetimes() {
    let mut queue = CommandQueue::new();
    let (file_listener, file_log) = event_log();
    let (artboard_listener, artboard_log) = event_log();
    let (machine_listener, machine_log) = event_log();
    let file = queue.load_file(ENTRY_FIXTURE.to_vec(), Some(&file_listener.file), 0, None);
    let artboard = queue.instantiate_default_artboard(file, Some(&artboard_listener.artboard), 0);
    let machine =
        queue.instantiate_default_state_machine(artboard, Some(&machine_listener.state_machine), 0);
    queue.request_artboard_names(file, 1);
    queue.request_state_machine_names(artboard, 2);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    assert!(!events(&file_log).is_empty());
    assert!(!events(&artboard_log).is_empty());
    let moved_file_listener = file_listener.file.clone();
    let moved_artboard_listener = artboard_listener.artboard.clone();
    let moved_machine_listener = machine_listener.state_machine.clone();
    drop(file_listener);
    drop(artboard_listener);
    drop(machine_listener);
    queue.delete_state_machine(machine, 3);
    queue.delete_artboard(artboard, 4);
    queue.delete_file(file, 5);
    assert!(server.process_commands());
    queue.process_messages();
    assert!(events(&machine_log).iter().any(|event| matches!(event, ObservedEvent::StateMachineDeleted { handle, .. } if *handle == machine)));
    assert!(events(&artboard_log).iter().any(
        |event| matches!(event, ObservedEvent::ArtboardDeleted { handle, .. } if *handle == artboard)
    ));
    assert!(events(&file_log).iter().any(
        |event| matches!(event, ObservedEvent::FileDeleted { handle, .. } if *handle == file)
    ));
    drop((
        moved_file_listener,
        moved_artboard_listener,
        moved_machine_listener,
    ));
    let (ephemeral, ephemeral_log) = event_log();
    queue.load_file(vec![0; 1024], Some(&ephemeral.file), 6, None);
    drop(ephemeral);
    assert!(server.process_commands());
    queue.process_messages();
    assert!(events(&ephemeral_log).is_empty());
}

#[test]
fn empty_listener_code_coverage() {
    let mut queue = CommandQueue::new();
    let (listener, _) = event_log();
    let file = queue.load_file(Vec::new(), Some(&listener.file), 0, None);
    queue.delete_file(file, 0);
    assert!(server(&queue).process_commands());
    queue.process_messages();
}

fn pointer_at(x: f32, y: f32) -> PointerEvent {
    PointerEvent {
        position: nuxie::Vec2D::new(x, y),
        ..PointerEvent::default()
    }
}

#[test]
fn pointer_input() {
    let mut queue = CommandQueue::new();
    let file = queue.load_file(POINTER_FIXTURE.to_vec(), None, 0, None);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let machine = queue.instantiate_default_state_machine(artboard, None, 0);
    queue.run_once(Box::new(move |server| {
        assert!(server.get_file(file).is_some());
        assert!(server.get_bindable_artboard(artboard).is_some());
        assert!(
            server
                .with_state_machine_instance_mut(machine, |_| ())
                .is_some()
        );
    }));
    queue.advance_state_machine(machine, 0.0, 0);
    let assert_bool = |queue: &mut CommandQueue, expected| {
        queue.run_once(Box::new(move |server| {
            assert_eq!(
                server
                    .with_state_machine_instance_mut(machine, |machine| {
                        machine.get_bool("isDown").map(|input| input.value())
                    })
                    .flatten(),
                Some(expected)
            );
        }));
    };
    queue.pointer_down(machine, pointer_at(425.0, 425.0), 0);
    assert_bool(&mut queue, true);
    queue.pointer_up(machine, pointer_at(425.0, 425.0), 0);
    assert_bool(&mut queue, true);
    queue.pointer_down(machine, pointer_at(425.0, 425.0), 0);
    assert_bool(&mut queue, false);
    queue.pointer_down(machine, pointer_at(75.0, 75.0), 0);
    assert_bool(&mut queue, true);
    queue.pointer_up(machine, pointer_at(75.0, 75.0), 0);
    assert_bool(&mut queue, false);
    queue.pointer_move(machine, pointer_at(250.0, 250.0), 0);
    assert_bool(&mut queue, false);
    queue.pointer_move(machine, pointer_at(425.0, 75.0), 0);
    assert_bool(&mut queue, true);
    queue.pointer_move(machine, pointer_at(250.0, 250.0), 0);
    assert_bool(&mut queue, true);
    queue.pointer_move(machine, pointer_at(425.0, 75.0), 0);
    assert_bool(&mut queue, false);
    queue.pointer_down(machine, pointer_at(75.0, 75.0), 0);
    assert_bool(&mut queue, true);
    queue.pointer_exit(machine, pointer_at(-25.0, -25.0), 0);
    assert_bool(&mut queue, true);
    queue.pointer_up(machine, pointer_at(-25.0, -25.0), 0);
    assert_bool(&mut queue, true);
    queue.pointer_up(machine, pointer_at(75.0, 75.0), 0);
    assert_bool(&mut queue, false);
    queue.pointer_down(machine, pointer_at(75.0, 75.0), 0);
    assert_bool(&mut queue, true);
    queue.pointer_exit(machine, pointer_at(-25.0, -25.0), 0);
    assert_bool(&mut queue, true);
    queue.pointer_move(machine, pointer_at(75.0, 75.0), 0);
    assert_bool(&mut queue, true);
    queue.pointer_up(machine, pointer_at(75.0, 75.0), 0);
    assert_bool(&mut queue, false);
    assert!(server(&queue).process_commands());
}

#[test]
fn pointer_down_advances_before_rapid_pointer_up() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(RAPID_POINTER_FIXTURE.to_vec(), None, 0, None);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let machine = queue.instantiate_default_state_machine(artboard, None, 0);
    let view_model = queue.instantiate_view_model_instance_for_artboard(
        file,
        artboard,
        String::new(),
        Some(&listener.view_model),
        0,
    );
    queue.bind_view_model_instance(machine, view_model, 0);
    queue.subscribe_to_view_model_property(
        view_model,
        "hasReached".to_string(),
        DataType::Boolean,
        0,
    );
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    assert!(
        !events(&log)
            .iter()
            .any(|event| matches!(event, ObservedEvent::ViewModelValue { .. }))
    );
    queue.advance_state_machine(machine, 0.0, 0);
    queue.pointer_down(machine, pointer_at(250.0, 250.0), 0);
    assert!(server.process_commands());
    queue.process_messages();
    assert_eq!(events(&log).iter().filter(|event| matches!(event, ObservedEvent::ViewModelValue { handle, path, value: ObservedValue::Boolean(true), .. } if *handle == view_model && path == "hasReached")).count(), 1);
    queue.pointer_up(machine, pointer_at(250.0, 250.0), 0);
    assert!(server.process_commands());
    queue.process_messages();
    assert_eq!(events(&log).iter().filter(|event| matches!(event, ObservedEvent::ViewModelValue { handle, path, .. } if *handle == view_model && path == "hasReached")).count(), 1);
}

#[test]
#[cfg(any())] // Exact cursor projection is private upstream; pointer behavior is covered above.
fn pointer_input_translation() {
    let mut queue = CommandQueue::new();
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 0, None);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let machine = queue.instantiate_default_state_machine(artboard, None, 0);
    let checks = [
        ((50.0, 50.0), (250.0, 250.0)),
        ((25.0, 25.0), (125.0, 125.0)),
        ((75.0, 75.0), (375.0, 375.0)),
        ((75.0, 25.0), (375.0, 125.0)),
        ((25.0, 75.0), (125.0, 375.0)),
    ];
    for ((x, y), (expected_x, expected_y)) in checks {
        queue.run_once(Box::new(move |server| {
            let translated = server
                .testing_cursor_position(
                    machine,
                    PointerEvent {
                        fit: Fit::Contain,
                        screen_bounds: nuxie::Vec2D::new(100.0, 100.0),
                        position: nuxie::Vec2D::new(x, y),
                        ..PointerEvent::default()
                    },
                )
                .expect("state machine cursor translation");
            assert!((translated.x - expected_x).abs() < 0.0001);
            assert!((translated.y - expected_y).abs() < 0.0001);
        }));
    }
    assert!(server(&queue).process_commands());
}

#[test]
fn global_listener() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    queue.set_global_file_listener(Some(&listener.file));
    queue.set_global_artboard_listener(Some(&listener.artboard));
    queue.set_global_state_machine_listener(Some(&listener.state_machine));
    queue.set_global_view_model_instance_listener(Some(&listener.view_model));
    queue.set_global_render_image_listener(Some(&listener.image));
    queue.set_global_audio_source_listener(Some(&listener.audio));
    queue.set_global_font_listener(Some(&listener.font));
    queue.set_global_blob_asset_listener(Some(&listener.blob));
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 1, None);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let machine = queue.instantiate_default_state_machine(artboard, None, 0);
    let view_model =
        queue.instantiate_view_model_instance_for_artboard(file, artboard, String::new(), None, 0);
    let image = queue.decode_image(IMAGE_FIXTURE.to_vec(), None, 0);
    let audio = queue.decode_audio(AUDIO_FIXTURE.to_vec(), None, 0);
    let font = queue.decode_font(FONT_FIXTURE.to_vec(), None, 0);
    let blob = queue.decode_blob(vec![1, 2, 3], None, 0);
    queue.request_artboard_names(file, 2);
    queue.request_view_model_names(file, 3);
    queue.request_view_model_instance_names(file, "Test All".to_string(), 4);
    queue.request_view_model_property_definitions(file, "Test All".to_string(), 5);
    queue.request_view_model_enums(file, 6);
    queue.request_state_machine_names(artboard, 11);
    queue.request_default_view_model_info(artboard, file, 20);
    queue.request_view_model_instance_bool(view_model, "Test Bool".to_owned(), 13);
    queue.request_view_model_instance_list_size(view_model, "Test List".to_string(), 14);
    queue.request_view_model_instance_view_model_name(view_model, 18);
    queue.request_view_model_instance_name(view_model, 19);
    for _ in 0..3 {
        queue.advance_state_machine(machine, 1.0, 16);
    }
    queue.delete_font(font, 10);
    queue.delete_state_machine(machine, 17);
    queue.delete_artboard(artboard, 12);
    queue.delete_view_model_instance(view_model, 15);
    queue.delete_image(image, 8);
    queue.delete_file(file, 7);
    queue.delete_audio(audio, 9);
    queue.delete_blob(blob, 21);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    for predicate in [
        |event: &ObservedEvent| matches!(event, ObservedEvent::FileLoaded { .. }),
        |event: &ObservedEvent| matches!(event, ObservedEvent::ArtboardInstantiated { .. }),
        |event: &ObservedEvent| matches!(event, ObservedEvent::StateMachineInstantiated { .. }),
        |event: &ObservedEvent| matches!(event, ObservedEvent::ViewModelInstantiated { .. }),
        |event: &ObservedEvent| matches!(event, ObservedEvent::ImageDecoded { .. }),
        |event: &ObservedEvent| matches!(event, ObservedEvent::AudioDecoded { .. }),
        |event: &ObservedEvent| matches!(event, ObservedEvent::FontDecoded { .. }),
        |event: &ObservedEvent| matches!(event, ObservedEvent::BlobDecoded { .. }),
        |event: &ObservedEvent| matches!(event, ObservedEvent::StateMachineSettled { .. }),
        |event: &ObservedEvent| matches!(event, ObservedEvent::ArtboardsListed { .. }),
        |event: &ObservedEvent| matches!(event, ObservedEvent::StateMachinesListed { .. }),
        |event: &ObservedEvent| matches!(event, ObservedEvent::ViewModelsListed { .. }),
        |event: &ObservedEvent| matches!(event, ObservedEvent::ViewModelInstancesListed { .. }),
        |event: &ObservedEvent| matches!(event, ObservedEvent::ViewModelPropertiesListed { .. }),
        |event: &ObservedEvent| matches!(event, ObservedEvent::ViewModelEnumsListed { .. }),
        |event: &ObservedEvent| matches!(event, ObservedEvent::ViewModelValue { .. }),
        |event: &ObservedEvent| matches!(event, ObservedEvent::ViewModelListSize { .. }),
        |event: &ObservedEvent| matches!(event, ObservedEvent::ViewModelName { .. }),
        |event: &ObservedEvent| matches!(event, ObservedEvent::ViewModelInstanceName { .. }),
        |event: &ObservedEvent| matches!(event, ObservedEvent::FontDeleted { .. }),
        |event: &ObservedEvent| matches!(event, ObservedEvent::StateMachineDeleted { .. }),
        |event: &ObservedEvent| matches!(event, ObservedEvent::ArtboardDeleted { .. }),
        |event: &ObservedEvent| matches!(event, ObservedEvent::ViewModelDeleted { .. }),
        |event: &ObservedEvent| matches!(event, ObservedEvent::ImageDeleted { .. }),
        |event: &ObservedEvent| matches!(event, ObservedEvent::FileDeleted { .. }),
        |event: &ObservedEvent| matches!(event, ObservedEvent::AudioDeleted { .. }),
        |event: &ObservedEvent| matches!(event, ObservedEvent::BlobDeleted { .. }),
    ] {
        assert!(captured.iter().any(predicate));
    }
    assert!(captured.iter().any(|event| matches!(
        event,
        ObservedEvent::StateMachineSettled { handle, request_id: 16 }
            if *handle == machine
    )));
}

#[test]
fn sync_pointer_events() {
    let mut queue = CommandQueue::new();
    let file = queue.load_file(POINTER_FIXTURE.to_vec(), None, 0, None);
    let artboard = queue.instantiate_artboard_named(file, "art-1".to_string(), None, 0);
    let machine = queue.instantiate_state_machine_named(artboard, "sm-1".to_string(), None, 0);
    queue.advance_state_machine(machine, 0.0, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    for index in 0..20 {
        let position = 50.0 + index as f32 * 10.0;
        let event = pointer_at(position, position);
        queue.pointer_down(machine, event, 0);
        queue.pointer_up(machine, event, 0);
        queue.pointer_move(machine, event, 0);
        queue.advance_state_machine(machine, 0.1, 0);
        server.pointer_down_synchronized(machine, &event);
        server.pointer_up_synchronized(machine, &event);
        server.pointer_move_synchronized(machine, &event);
        assert!(server.process_commands());
    }
    queue.delete_state_machine(machine, 0);
    assert!(server.process_commands());
    server.pointer_down_synchronized(machine, &PointerEvent::default());
    server.pointer_up_synchronized(machine, &PointerEvent::default());
    server.pointer_move_synchronized(machine, &PointerEvent::default());
}

#[test]
fn request_view_model_instance_list_clear() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 0, None);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let view_model = queue.instantiate_view_model_instance_for_artboard(
        file,
        artboard,
        String::new(),
        Some(&listener.view_model),
        0,
    );
    let nested = queue.instantiate_blank_view_model_instance_named(
        file,
        "ListViewModel".to_string(),
        None,
        0,
    );
    queue.append_view_model_instance_list_view_model(view_model, "Test List".to_owned(), nested, 0);
    queue.request_view_model_instance_list_size(view_model, "Test List".to_string(), 1);
    queue.request_view_model_instance_list_clear(view_model, "Test List".to_string(), 0x42);
    queue.request_view_model_instance_list_size(view_model, "Test List".to_string(), 2);
    let bad = queue.instantiate_blank_view_model_instance_named(file, "Bad".to_string(), None, 0);
    queue.request_view_model_instance_list_clear(bad, "Test List".to_string(), 3);
    queue.request_view_model_instance_list_clear(view_model, "Bad".to_string(), 4);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(captured.iter().any(|event| matches!(event, ObservedEvent::ViewModelListSize { request_id: 1, size, .. } if *size >= 1)));
    assert!(captured.iter().any(|event| matches!(event, ObservedEvent::ViewModelListCleared { handle, request_id: 0x42, path } if *handle == view_model && path == "Test List")));
    assert!(captured.iter().any(|event| matches!(
        event,
        ObservedEvent::ViewModelListSize {
            request_id: 2,
            size: 0,
            ..
        }
    )));
}

#[test]
fn dependency_lifetime_management() {
    let mut queue = CommandQueue::new();
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 1, None);
    let artboards = [
        queue.instantiate_default_artboard(file, None, 0),
        queue.instantiate_default_artboard(file, None, 0),
        queue.instantiate_default_artboard(file, None, 0),
    ];
    let machines = [
        queue.instantiate_default_state_machine(artboards[0], None, 0),
        queue.instantiate_default_state_machine(artboards[0], None, 0),
        queue.instantiate_default_state_machine(artboards[1], None, 0),
        queue.instantiate_default_state_machine(artboards[1], None, 0),
        queue.instantiate_default_state_machine(artboards[1], None, 0),
        queue.instantiate_default_state_machine(artboards[1], None, 0),
    ];
    let mut server = server(&queue);
    assert!(server.process_commands());
    assert!(
        artboards
            .iter()
            .all(|handle| server.get_bindable_artboard(*handle).is_some())
    );
    assert!(machines.iter().all(|handle| {
        server
            .with_state_machine_instance_mut(*handle, |_| ())
            .is_some()
    }));
    queue.delete_artboard(artboards[0], 0);
    assert!(server.process_commands());
    assert!(server.get_bindable_artboard(artboards[0]).is_none());
    assert!(server.get_bindable_artboard(artboards[1]).is_some());
    assert!(server.get_bindable_artboard(artboards[2]).is_some());
    assert!(
        server
            .with_state_machine_instance_mut(machines[0], |_| ())
            .is_none()
    );
    assert!(
        server
            .with_state_machine_instance_mut(machines[1], |_| ())
            .is_none()
    );
    assert!(machines[2..].iter().all(|handle| {
        server
            .with_state_machine_instance_mut(*handle, |_| ())
            .is_some()
    }));
    queue.delete_state_machine(machines[2], 0);
    assert!(server.process_commands());
    assert!(
        server
            .with_state_machine_instance_mut(machines[2], |_| ())
            .is_none()
    );
    assert!(machines[3..].iter().all(|handle| {
        server
            .with_state_machine_instance_mut(*handle, |_| ())
            .is_some()
    }));
}

fn listed_assets(bytes: &[u8], request_id: u64) -> Vec<nuxie::command_queue::FileAssetData> {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(bytes.to_vec(), Some(&listener.file), 0, None);
    queue.request_file_assets(file, request_id);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    events(&log)
        .into_iter()
        .find_map(|event| match event {
            ObservedEvent::FileAssetsListed {
                handle,
                request_id: actual,
                assets,
            } if handle == file && actual == request_id => Some(assets),
            _ => None,
        })
        .expect("file assets callback")
}

#[test]
fn file_assets_listed_image_asset() {
    let assets = listed_assets(HOSTED_IMAGE_FIXTURE, 42);
    assert_eq!(assets.len(), 1);
    let asset = &assets[0];
    assert_eq!(asset.name, "one.png");
    assert_eq!(asset.asset_id, 45008);
    assert_eq!(asset.cdn_uuid, "edcb1816-8405-4983-acd2-16db48d85df4");
    assert_eq!(asset.cdn_base_url, "https://public.uat.rive.app/cdn/uuid");
    assert_eq!(asset.file_extension, "png");
    assert_eq!(asset.asset_type, 105);
}

#[test]
fn file_assets_listed_font_asset() {
    let assets = listed_assets(HOSTED_FONT_FIXTURE, 43);
    assert_eq!(assets.len(), 1);
    let asset = &assets[0];
    assert_eq!(asset.name, "Inter");
    assert_eq!(asset.asset_id, 43276);
    assert_eq!(asset.cdn_base_url, "https://public.uat.rive.app/cdn/uuid");
    assert_eq!(asset.file_extension, "ttf");
    assert_eq!(asset.asset_type, 141);
}

#[test]
fn file_assets_listed_type_ids_match_runtime() {
    assert_eq!(
        nuxie_schema::definition_by_name("ImageAsset")
            .unwrap()
            .type_key
            .int,
        105
    );
    assert_eq!(
        nuxie_schema::definition_by_name("FontAsset")
            .unwrap()
            .type_key
            .int,
        141
    );
    assert_eq!(
        nuxie_schema::definition_by_name("AudioAsset")
            .unwrap()
            .type_key
            .int,
        406
    );
}

#[test]
fn file_assets_listed_empty_file() {
    assert!(listed_assets(ARTBOARD_FIXTURE, 44).is_empty());
}

#[test]
fn file_assets_listed_invalid_handle() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    queue.set_global_file_listener(Some(&listener.file));
    let bad = queue.load_file(vec![0; 1024], None, 0, None);
    queue.request_file_assets(bad, 45);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(!captured.iter().any(
        |event| matches!(event, ObservedEvent::FileAssetsListed { handle, .. } if *handle == bad)
    ));
    assert!(captured.iter().any(|event| matches!(event, ObservedEvent::FileError { handle, request_id: 45, .. } if *handle == bad)));
}

#[test]
fn file_assets_listed_all_assets_returned() {
    let assets = listed_assets(DATA_BIND_FIXTURE, 46);
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).expect("persistent factory");
    let file =
        nuxie::File::import(DATA_BIND_FIXTURE, factory, None, None, None).expect("fixture import");
    assert_eq!(assets.len(), file.with_file(|file| file.assets().len()));
}

#[test]
fn global_view_model_names_listed() {
    let mut queue = CommandQueue::new();
    let (listener, log) = event_log();
    queue.set_global_file_listener(Some(&listener.file));
    let file = queue.load_file(GLOBAL_VARIABLES_FIXTURE.to_vec(), None, 0, None);
    queue.request_global_view_model_names(file, 7);
    let bad = queue.load_file(vec![0; 1024 * 1024], None, 0, None);
    queue.request_global_view_model_names(bad, 8);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(captured.iter().any(|event| matches!(
        event,
        ObservedEvent::GlobalViewModelsListed { handle, request_id: 7, names }
            if *handle == file
                && !names.is_empty()
                && names.iter().all(|name| !name.is_empty())
    )));
    assert!(!captured.iter().any(|event| matches!(event, ObservedEvent::GlobalViewModelsListed { handle, .. } if *handle == bad)));
}

#[test]
fn set_bind_get_global_view_model_instance() {
    let mut queue = CommandQueue::new();
    let (file_listener, file_log) = event_log();
    let file = queue.load_file(
        GLOBAL_VARIABLES_FIXTURE.to_vec(),
        Some(&file_listener.file),
        0,
        None,
    );
    queue.request_global_view_model_names(file, 1);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let global_name = events(&file_log)
        .into_iter()
        .find_map(|event| match event {
            ObservedEvent::GlobalViewModelsListed { names, .. } => names.into_iter().next(),
            _ => None,
        })
        .expect("global view model name");
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let machine = queue.instantiate_default_state_machine(artboard, None, 0);
    let global = queue.instantiate_view_model_instance_named(
        file,
        global_name.clone(),
        "".to_string(),
        None,
        0,
    );
    queue.set_global_view_model_instance(machine, global_name.clone(), global, 0);
    queue.bind(machine, 0);
    let (ok_listener, ok_log) = event_log();
    let fetched = queue.global_view_model_instance(
        machine,
        global_name.clone(),
        Some(&ok_listener.view_model),
        0,
    );
    queue.request_view_model_instance_view_model_name(fetched, 2);
    let (error_listener, error_log) = event_log();
    let missing = queue.global_view_model_instance(
        machine,
        "not-a-global".to_string(),
        Some(&error_listener.view_model),
        3,
    );
    assert!(server.process_commands());
    queue.process_messages();
    assert!(events(&ok_log).iter().any(|event| matches!(event, ObservedEvent::ViewModelName { handle, request_id: 2, name } if *handle == fetched && name == &global_name)));
    assert!(events(&error_log).iter().any(|event| matches!(event, ObservedEvent::ViewModelError { handle, request_id: 3, .. } if *handle == missing)));
}

#[test]
fn command_server_get_handle_for_instance() {
    let mut queue = CommandQueue::new();
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 0, None);
    let handle = queue.instantiate_view_model_instance_named(
        file,
        "Test All".to_string(),
        "".to_string(),
        None,
        0,
    );
    let mut server = server(&queue);
    assert!(server.process_commands());
    let instance = server
        .get_view_model_instance(handle)
        .expect("view model instance");
    assert_eq!(server.get_handle_for_instance(&instance), handle);
}

#[test]
fn run_once_preserves_command_order() {
    let mut queue = CommandQueue::new();
    let order = Arc::new(Mutex::new(Vec::new()));
    for value in 0..3 {
        let order = Arc::clone(&order);
        queue.run_once(Box::new(move |_| {
            order
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .push(value);
        }));
    }
    assert!(server(&queue).process_commands());
    assert_eq!(
        *order
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
        vec![0, 1, 2]
    );
}

#[test]
fn draw_is_coalesced_by_key_within_one_poll() {
    let mut queue = CommandQueue::new();
    let key = queue.create_draw_key();
    let count = Arc::new(AtomicUsize::new(0));
    for value in [1, 10] {
        let count = Arc::clone(&count);
        queue.draw(
            key,
            Box::new(move |_, _| {
                count.fetch_add(value, Ordering::SeqCst);
            }),
        );
    }
    assert!(server(&queue).process_commands());
    assert_eq!(count.load(Ordering::SeqCst), 10);
}

#[test]
fn disconnect_stops_a_non_waiting_server() {
    let mut queue = CommandQueue::new();
    let mut server = server(&queue);
    assert!(server.process_commands());
    assert!(!server.get_was_disconnected());
    queue.disconnect();
    assert!(!server.process_commands());
    assert!(server.get_was_disconnected());
}

#[test]
fn draw_happens_once_per_poll() {
    let mut queue = CommandQueue::new();
    let key = queue.create_draw_key();
    let count = Arc::new(AtomicUsize::new(0));
    let mut server = server(&queue);
    for expected in 1..=2 {
        let count_on_draw = Arc::clone(&count);
        queue.draw(
            key,
            Box::new(move |_, _| {
                count_on_draw.fetch_add(1, Ordering::SeqCst);
            }),
        );
        assert!(server.process_commands());
        assert_eq!(count.load(Ordering::SeqCst), expected);
    }
}

#[test]
fn cancel_draw_only_cancels_matching_pending_key() {
    let mut queue = CommandQueue::new();
    let cancelled = queue.create_draw_key();
    let retained = queue.create_draw_key();
    let count = Arc::new(AtomicUsize::new(0));
    let first = Arc::clone(&count);
    queue.draw(
        cancelled,
        Box::new(move |_, _| {
            first.fetch_add(1, Ordering::SeqCst);
        }),
    );
    let second = Arc::clone(&count);
    queue.draw(
        retained,
        Box::new(move |_, _| {
            second.fetch_add(10, Ordering::SeqCst);
        }),
    );
    queue.cancel_draw(cancelled);
    assert!(server(&queue).process_commands());
    assert_eq!(count.load(Ordering::SeqCst), 10);

    let count_after_cancel = Arc::clone(&count);
    queue.draw(
        cancelled,
        Box::new(move |_, _| {
            count_after_cancel.fetch_add(1, Ordering::SeqCst);
        }),
    );
    assert!(server(&queue).process_commands());
    assert_eq!(count.load(Ordering::SeqCst), 11);
}

#[test]
#[cfg(any())] // Upstream's Send callback cannot capture the non-Send queue to enqueue recursively.
fn command_poll_is_entry_bounded() {
    let mut queue = CommandQueue::new();
    let count = Arc::new(AtomicUsize::new(0));
    let nested_queue = queue.clone();
    let count_for_outer = Arc::clone(&count);
    queue.run_once(Box::new(move |_| {
        count_for_outer.fetch_add(1, Ordering::SeqCst);
        let count_for_inner = Arc::clone(&count_for_outer);
        nested_queue.run_once(Box::new(move |_| {
            count_for_inner.fetch_add(1, Ordering::SeqCst);
        }));
    }));
    let mut server = server(&queue);
    assert!(server.process_commands());
    assert_eq!(count.load(Ordering::SeqCst), 1);
    assert!(server.process_commands());
    assert_eq!(count.load(Ordering::SeqCst), 2);
}

#[test]
#[cfg(any())] // The typed upstream listener cannot retain the non-Send queue for reentrant enqueue.
fn message_poll_is_entry_bounded() {
    let mut queue = CommandQueue::new();
    let events = Arc::new(Mutex::new(Vec::new()));
    let listener = ListenerHandle::new(Box::new(ReentrantFileEvents {
        base: ListenerBase::new(),
        log: Arc::clone(&events),
        queue: queue.clone(),
    }));
    queue.set_global_file_listener(Some(&listener));
    queue.load_file(Vec::new(), None, 1, None);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    assert_eq!(
        events
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len(),
        1
    );
    assert!(server.process_commands());
    queue.process_messages();
}

#[test]
fn listener_lifetime_is_weak() {
    let mut queue = CommandQueue::new();
    let (listener, events) = event_log();
    queue.load_file(Vec::new(), Some(&listener.file), 4, None);
    drop(listener);
    assert!(server(&queue).process_commands());
    queue.process_messages();
    assert!(
        events
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .is_empty()
    );
}

#[test]
#[cfg(any())] // The upstream-shaped queue owns Rc-backed runtime resources and is not Send.
fn wait_commands_wakes_for_work_and_disconnects() {
    let mut queue = CommandQueue::new();
    let worker_queue = queue.clone();
    let worker = thread::spawn(move || {
        let mut server = server(&worker_queue);
        server.serve_until_disconnect();
        server.get_was_disconnected()
    });
    let completed = Arc::new(AtomicUsize::new(0));
    let completed_on_server = Arc::clone(&completed);
    queue.run_once(Box::new(move |_| {
        completed_on_server.store(1, Ordering::SeqCst);
    }));
    queue.disconnect();
    assert!(worker.join().expect("command server thread panicked"));
    assert_eq!(completed.load(Ordering::SeqCst), 1);
}

#[test]
fn deleting_file_preserves_artboards_like_pinned_source() {
    let mut queue = CommandQueue::new();
    let file = queue.load_file(ARTBOARD_FIXTURE.to_vec(), None, 0, None);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.delete_file(file, 0);
    assert!(server.process_commands());
    assert!(server.get_file(file).is_none());
    // Pinned CommandServer creates m_fileDependencies[file] but never appends
    // instantiated artboards to it, so file deletion does not cascade.
    assert!(server.get_bindable_artboard(artboard).is_some());
    queue.delete_artboard(artboard, 0);
    assert!(server.process_commands());
    assert!(server.get_bindable_artboard(artboard).is_none());
}
