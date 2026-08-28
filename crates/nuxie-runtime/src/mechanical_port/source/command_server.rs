use std::{
    collections::HashMap,
    fmt,
    rc::Rc,
    sync::{Arc, Mutex},
    thread::{self, ThreadId},
};

use crate::mechanical_port::source::{
    animation::state_machine_instance::StateMachineInstance,
    artboard::ArtboardInstance,
    assets::{
        audio_asset::AudioAsset, font_asset::FontAsset, image_asset::ImageAsset,
        script_asset::ScriptAsset,
    },
    audio::audio_source::{AudioSource, AudioSourceRef},
    bindable_artboard::BindableArtboard,
    command_queue::{
        ArtboardHandle, AudioSourceHandle, BlobAssetHandle, Command, CommandQueue,
        CommandServerCallback, CommandServerDrawCallback, DataType, DrawKey, FileHandle,
        FontHandle, Message, PointerEvent, PropertyData, RenderImageHandle, StateMachineHandle,
        ViewModelInstanceData, ViewModelInstanceHandle, ViewModelInstanceValue,
    },
    core::CoreHandle,
    factory::RuntimeFactoryHandle,
    file::File,
    file_asset_loader::{FileAssetLoader, FileAssetLoaderRef},
    generated::core_registry::CoreCapabilities,
    hit_result::HitResult,
    layout::{compute_alignment, Alignment},
    math::{aabb::Aabb, mat2d::Mat2D, vec2d::Vec2D},
    renderer::{RenderImage, RenderImageRef},
    semantic::semantic_snapshot::{SemanticsBoundsUpdate, SemanticsDiff, SemanticsDiffNode},
    text::font_hb::HbFont,
    text_engine::FontRef,
    viewmodel::runtime::viewmodel_instance_runtime::ViewModelInstanceRuntime,
};
use crate::RuntimeBlobAsset;

pub struct Subscription {
    pub request_id: u64,
    pub data: PropertyData,
    pub root_view_model: ViewModelInstanceHandle,
}

#[derive(Default)]
struct CommandAssetRegistry {
    image_assets: HashMap<String, RenderImageHandle>,
    audio_assets: HashMap<String, AudioSourceHandle>,
    font_assets: HashMap<String, FontHandle>,
    images: HashMap<RenderImageHandle, RenderImageRef>,
    audio_sources: HashMap<AudioSourceHandle, AudioSourceRef>,
    fonts: HashMap<FontHandle, FontRef>,
}

struct CommandFileAssetLoader {
    assets: Rc<std::cell::RefCell<CommandAssetRegistry>>,
    internal_loader: Option<FileAssetLoaderRef>,
}

impl FileAssetLoader for CommandFileAssetLoader {
    fn load_contents(
        &mut self,
        asset: CoreHandle,
        in_band_bytes: &[u8],
        factory: &RuntimeFactoryHandle,
    ) -> bool {
        if self.internal_loader.as_ref().is_some_and(|loader| {
            loader.with_loader_mut(|loader| {
                loader.load_contents(asset.clone(), in_band_bytes, factory)
            })
        }) {
            return true;
        }

        let Some((unique_name, unique_filename)) = asset
            .with(|asset| {
                CoreCapabilities::as_file_asset(asset).map(|asset| {
                    (
                        asset.file_asset_base().unique_name(),
                        asset
                            .file_asset_base()
                            .unique_filename(asset.file_extension()),
                    )
                })
            })
            .flatten()
        else {
            return false;
        };

        let assets = self.assets.borrow();
        if let Some(image) = assets
            .image_assets
            .get(&unique_name)
            .and_then(|handle| assets.images.get(handle))
            .cloned()
        {
            return asset
                .with_downcast_mut::<ImageAsset, _>(|asset| {
                    asset.set_render_image(Some(image));
                    true
                })
                .unwrap_or(false);
        }
        if let Some(audio) = assets
            .audio_assets
            .get(&unique_name)
            .and_then(|handle| assets.audio_sources.get(handle))
            .cloned()
        {
            return asset
                .with_downcast_mut::<AudioAsset, _>(|asset| {
                    asset.set_audio_source(Some(audio));
                    true
                })
                .unwrap_or(false);
        }
        if let Some(font) = assets
            .font_assets
            .get(&unique_name)
            .and_then(|handle| assets.fonts.get(handle))
            .cloned()
        {
            return asset
                .with_downcast_mut::<FontAsset, _>(|asset| {
                    asset.set_font(Some(font));
                    true
                })
                .unwrap_or(false);
        }
        if asset.with_downcast::<ScriptAsset, _>(|_| ()).is_some()
            || asset.with_downcast::<ImageAsset, _>(|_| ()).is_some()
            || asset.with_downcast::<AudioAsset, _>(|_| ()).is_some()
            || asset.with_downcast::<FontAsset, _>(|_| ()).is_some()
        {
            return false;
        }
        eprintln!(
            "ERROR: CommandFileAssetLoader::loadContents - Unsupported asset type for asset: '{unique_filename}'"
        );
        false
    }
}

fn data_type_name(value: DataType) -> &'static str {
    match value {
        DataType::None => "None",
        DataType::String => "String",
        DataType::Number => "Number",
        DataType::Boolean => "Boolean",
        DataType::Color => "Color",
        DataType::List => "List",
        DataType::Enum => "Enum",
        DataType::Trigger => "Trigger",
        DataType::ViewModel => "View Model",
        DataType::Integer => "Integer",
        DataType::SymbolListIndex => "Symbol List Index",
        DataType::AssetImage => "Asset Image",
        DataType::AssetBlob => "Asset Blob",
        _ => "Unknown DataType",
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(data_type_name(*self))
    }
}

struct SynchronizedStateMachine {
    instance: Mutex<Box<StateMachineInstance>>,
}

impl SynchronizedStateMachine {
    fn new(instance: Box<StateMachineInstance>) -> Self {
        Self {
            instance: Mutex::new(instance),
        }
    }

    fn replace(&mut self, other: Self) {
        self.instance = other.instance;
    }
}

impl Drop for SynchronizedStateMachine {
    fn drop(&mut self) {
        self.instance.get_mut().unwrap().dispose();
    }
}

pub struct CommandServer {
    was_disconnect_received: bool,
    command_queue: CommandQueue,
    factory: RuntimeFactoryHandle,
    internal_loader: Option<FileAssetLoaderRef>,
    #[cfg(debug_assertions)]
    thread_id: ThreadId,
    property_subscriptions: Vec<Subscription>,
    file_dependencies: HashMap<FileHandle, Vec<ArtboardHandle>>,
    artboard_dependencies: HashMap<ArtboardHandle, Vec<StateMachineHandle>>,
    files: HashMap<FileHandle, Rc<File>>,
    assets: Rc<std::cell::RefCell<CommandAssetRegistry>>,
    blobs: HashMap<BlobAssetHandle, Arc<RuntimeBlobAsset>>,
    artboards: HashMap<ArtboardHandle, Rc<BindableArtboard>>,
    view_models: HashMap<ViewModelInstanceHandle, Rc<ViewModelInstanceRuntime>>,
    state_machines: Mutex<HashMap<StateMachineHandle, Arc<SynchronizedStateMachine>>>,
    unique_draws: HashMap<DrawKey, CommandServerDrawCallback>,
}

impl CommandServer {
    pub fn new(
        command_queue: CommandQueue,
        factory: RuntimeFactoryHandle,
        internal_loader: Option<FileAssetLoaderRef>,
    ) -> Box<Self> {
        Box::new(Self {
            was_disconnect_received: false,
            command_queue,
            factory,
            internal_loader,
            #[cfg(debug_assertions)]
            thread_id: thread::current().id(),
            property_subscriptions: Vec::new(),
            file_dependencies: HashMap::new(),
            artboard_dependencies: HashMap::new(),
            files: HashMap::new(),
            assets: Rc::new(std::cell::RefCell::new(CommandAssetRegistry::default())),
            blobs: HashMap::new(),
            artboards: HashMap::new(),
            view_models: HashMap::new(),
            state_machines: Mutex::new(HashMap::new()),
            unique_draws: HashMap::new(),
        })
    }

    pub fn factory(&self) -> RuntimeFactoryHandle {
        self.factory.clone()
    }

    fn assert_thread(&self) {
        #[cfg(debug_assertions)]
        assert_eq!(thread::current().id(), self.thread_id);
    }

    pub fn get_file(&self, handle: FileHandle) -> Option<&File> {
        self.assert_thread();
        self.files.get(&handle).map(Rc::as_ref)
    }

    pub fn get_image(&self, handle: RenderImageHandle) -> Option<RenderImageRef> {
        self.assert_thread();
        self.assets.borrow().images.get(&handle).cloned()
    }

    pub fn get_audio_source(&self, handle: AudioSourceHandle) -> Option<AudioSourceRef> {
        self.assert_thread();
        self.assets.borrow().audio_sources.get(&handle).cloned()
    }

    pub fn get_font(&self, handle: FontHandle) -> Option<FontRef> {
        self.assert_thread();
        self.assets.borrow().fonts.get(&handle).cloned()
    }

    pub fn get_blob(&self, handle: BlobAssetHandle) -> Option<Arc<RuntimeBlobAsset>> {
        self.assert_thread();
        self.blobs.get(&handle).cloned()
    }

    pub fn get_artboard_instance(&self, handle: ArtboardHandle) -> Option<&mut ArtboardInstance> {
        self.assert_thread();
        self.artboards
            .get(&handle)
            .and_then(|value| value.artboard())
    }

    pub fn get_bindable_artboard(&self, handle: ArtboardHandle) -> Option<Rc<BindableArtboard>> {
        self.assert_thread();
        self.artboards.get(&handle).cloned()
    }

    fn get_state_machine_wrapper(
        &self,
        handle: StateMachineHandle,
    ) -> Option<Arc<SynchronizedStateMachine>> {
        self.assert_thread();
        self.state_machines.lock().unwrap().get(&handle).cloned()
    }

    pub fn with_state_machine_instance_mut<R>(
        &self,
        handle: StateMachineHandle,
        use_instance: impl FnOnce(&mut StateMachineInstance) -> R,
    ) -> Option<R> {
        let wrapper = self.get_state_machine_wrapper(handle)?;
        let mut instance = wrapper
            .instance
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Some(use_instance(&mut instance))
    }

    pub fn get_view_model_instance(
        &self,
        handle: ViewModelInstanceHandle,
    ) -> Option<&ViewModelInstanceRuntime> {
        self.assert_thread();
        self.view_models.get(&handle).map(Rc::as_ref)
    }

    pub fn get_handle_for_instance(
        &self,
        instance: &ViewModelInstanceRuntime,
    ) -> ViewModelInstanceHandle {
        self.assert_thread();
        self.view_models
            .iter()
            .find_map(|(handle, candidate)| {
                std::ptr::eq(Rc::as_ref(candidate), instance).then_some(*handle)
            })
            .unwrap_or(ViewModelInstanceHandle::NULL)
    }

    pub fn get_was_disconnected(&self) -> bool {
        self.was_disconnect_received
    }

    fn cursor_pos_for_pointer_event(
        &self,
        instance: &StateMachineInstance,
        event: &PointerEvent,
    ) -> Vec2D {
        let artboard_bounds = instance.artboard().bounds();
        let surface_bounds = Aabb::from_min_max(Vec2D::new(0.0, 0.0), event.screen_bounds);
        if surface_bounds == artboard_bounds
            || surface_bounds.width() == 0.0
            || surface_bounds.height() == 0.0
        {
            return event.position;
        }
        let forward = compute_alignment(
            event.fit,
            event.alignment,
            &surface_bounds,
            &artboard_bounds,
            event.scale_factor,
        );
        forward.invert_or_identity() * event.position
    }

    pub fn pointer_down_synchronized(
        &self,
        handle: StateMachineHandle,
        event: &PointerEvent,
    ) -> HitResult {
        let Some(wrapper) = self.get_state_machine_wrapper(handle) else {
            return HitResult::None;
        };
        let mut instance = wrapper.instance.lock().unwrap();
        let position = self.cursor_pos_for_pointer_event(&instance, event);
        let result = instance.pointer_down(position, event.pointer_id);
        if result != HitResult::None {
            instance.advance_and_apply(0.0);
        }
        result
    }

    pub fn pointer_move_synchronized(
        &self,
        handle: StateMachineHandle,
        event: &PointerEvent,
    ) -> HitResult {
        let Some(wrapper) = self.get_state_machine_wrapper(handle) else {
            return HitResult::None;
        };
        let mut instance = wrapper.instance.lock().unwrap();
        let position = self.cursor_pos_for_pointer_event(&instance, event);
        instance.pointer_move(position, 0.0, event.pointer_id)
    }

    pub fn pointer_up_synchronized(
        &self,
        handle: StateMachineHandle,
        event: &PointerEvent,
    ) -> HitResult {
        let Some(wrapper) = self.get_state_machine_wrapper(handle) else {
            return HitResult::None;
        };
        let mut instance = wrapper.instance.lock().unwrap();
        let position = self.cursor_pos_for_pointer_event(&instance, event);
        let result = instance.pointer_up(position, event.pointer_id);
        if result != HitResult::None {
            instance.advance_and_apply(0.0);
        }
        result
    }

    fn error<H: Copy + fmt::Display + 'static>(
        &self,
        handle: H,
        request_id: u64,
        message: Message,
        text: String,
    ) {
        eprintln!("{text}");
        let mut messages = self.command_queue.message_lock();
        messages.write(message);
        messages.write(handle);
        messages.write(request_id);
        messages.write_name(text);
    }

    fn cleanup_artboard(&mut self, handle: ArtboardHandle, request_id: u64) {
        if self.artboards.contains_key(&handle) {
            if let Some(state_machines) = self.artboard_dependencies.remove(&handle) {
                for state_machine in state_machines {
                    if self
                        .state_machines
                        .lock()
                        .unwrap()
                        .remove(&state_machine)
                        .is_some()
                    {
                        let mut messages = self.command_queue.message_lock();
                        messages.write(Message::StateMachineDeleted);
                        messages.write(state_machine);
                        messages.write(request_id);
                    }
                }
            }
            self.artboards.remove(&handle);
            let mut messages = self.command_queue.message_lock();
            messages.write(Message::ArtboardDeleted);
            messages.write(handle);
            messages.write(request_id);
        }
    }

    fn map_semantics_bounds<
        T: crate::mechanical_port::source::semantic::semantic_snapshot::SemanticBounds,
    >(
        value: &mut T,
        transform: &Mat2D,
    ) {
        value.set_bounds(transform.map_bounding_box(value.bounds()));
    }

    fn map_semantics_diff_to_view_space(diff: &mut SemanticsDiff, transform: &Mat2D) {
        for node in &mut diff.added {
            Self::map_semantics_bounds(node, transform);
        }
        for node in &mut diff.moved {
            Self::map_semantics_bounds(node, transform);
        }
        for node in &mut diff.updated_semantic {
            Self::map_semantics_bounds(node, transform);
        }
        for bounds in &mut diff.updated_geometry {
            Self::map_semantics_bounds(bounds, transform);
        }
    }

    fn check_property_subscriptions(&mut self) {
        for subscription in &self.property_subscriptions {
            let request_id = subscription.request_id;
            let handle = subscription.root_view_model;
            let mut data = ViewModelInstanceData::new(subscription.data.clone());
            let Some(view_model) = self.get_view_model_instance(handle) else {
                continue;
            };
            let Some(property) = view_model.property(&data.meta_data.name) else {
                continue;
            };
            if !property.has_changed() {
                continue;
            }
            property.clear_changes();
            data.value = match data.meta_data.data_type {
                DataType::AssetImage | DataType::AssetBlob | DataType::Trigger | DataType::List => {
                    ViewModelInstanceValue::None
                }
                DataType::Boolean => {
                    ViewModelInstanceValue::Bool(property.as_boolean().unwrap().value())
                }
                DataType::Color => {
                    ViewModelInstanceValue::Color(property.as_color().unwrap().value())
                }
                DataType::Number => {
                    ViewModelInstanceValue::Number(property.as_number().unwrap().value())
                }
                DataType::Enum => {
                    ViewModelInstanceValue::String(property.as_enum().unwrap().value().to_owned())
                }
                DataType::String => {
                    ViewModelInstanceValue::String(property.as_string().unwrap().value().to_owned())
                }
                other => {
                    self.error(
                        handle,
                        request_id,
                        Message::ViewModelError,
                        format!(
                            "ERROR : Invalid data type {{{}}} when checking subscriptions",
                            data_type_name(other)
                        ),
                    );
                    continue;
                }
            };
            let mut messages = self.command_queue.message_lock();
            messages.write(Message::ViewModelPropertyValueReceived);
            messages.write(handle);
            messages.write(data.meta_data.data_type);
            messages.write_name(data.meta_data.name.clone());
            messages.write(request_id);
            match data.value {
                ViewModelInstanceValue::None => {}
                ViewModelInstanceValue::Bool(value) => messages.write(value),
                ViewModelInstanceValue::Number(value) => messages.write(value),
                ViewModelInstanceValue::Color(value) => messages.write(value),
                ViewModelInstanceValue::String(value) => messages.write_name(value),
            }
        }
    }

    pub fn serve_until_disconnect(&mut self) {
        while self.wait_commands() {}
    }

    pub fn wait_commands(&mut self) -> bool {
        if self.command_queue.command_stream_is_empty() {
            let mut lock = self.command_queue.command_lock();
            while self.command_queue.command_stream_is_empty() {
                assert!(self.command_queue.callbacks_are_empty());
                assert!(self.command_queue.byte_vectors_are_empty());
                assert!(self.command_queue.names_are_empty());
                self.command_queue.wait_for_command(&mut lock);
            }
        }
        self.process_commands()
    }

    pub fn process_commands(&mut self) -> bool {
        assert!(!self.was_disconnect_received);
        self.assert_thread();
        let mut lock = self.command_queue.command_lock();
        if self.command_queue.command_stream_is_empty() {
            return !self.was_disconnect_received;
        }
        self.command_queue.push_command(Command::CommandLoopBreak);
        assert!(self.unique_draws.is_empty());
        let mut should_process_commands = true;
        loop {
            let command = self.command_queue.read_command();
            match command {
                Command::LoadFile => {
                    let handle: FileHandle = self.command_queue.read();
                    let request_id: u64 = self.command_queue.read();
                    let bytes = self.command_queue.pop_bytes();
                    let scripting_factory = self.command_queue.pop_scripting_context_factory();
                    lock.unlock();
                    let file = {
                        let mut context = scripting_factory
                            .and_then(|factory_fn| factory_fn(self.factory()))
                            .unwrap_or_else(|| {
                                Box::new(crate::mechanical_port::source::lua::CPPRuntimeScriptingContext::new(
                                    self.factory(),
                                ))
                            });
                        context.set_render_context(self.factory());
                        let vm =
                            crate::mechanical_port::source::lua::scripting_vm::ScriptingVM::new(
                                context,
                            );
                        File::import_with_vm(
                            &bytes,
                            self.factory(),
                            self.file_asset_loader.as_mut(),
                            &vm,
                        )
                    };
                    #[cfg(any())]
                    let file =
                        File::import(&bytes, self.factory(), self.file_asset_loader.as_mut());
                    if let Some(file) = file {
                        self.file_dependencies.insert(handle, Vec::new());
                        self.files.insert(handle, file);
                        let mut messages = self.command_queue.message_lock();
                        messages.write(Message::FileLoaded);
                        messages.write(handle);
                        messages.write(request_id);
                    } else {
                        self.error(
                            handle,
                            request_id,
                            Message::FileError,
                            "failed to load Rive file.".into(),
                        );
                    }
                }
                Command::DeleteFile => {
                    let handle: FileHandle = self.command_queue.read();
                    let request_id: u64 = self.command_queue.read();
                    lock.unlock();
                    self.files.remove(&handle);
                    if let Some(artboards) = self.file_dependencies.remove(&handle) {
                        for artboard in artboards {
                            self.cleanup_artboard(artboard, request_id);
                        }
                    }
                    let mut messages = self.command_queue.message_lock();
                    messages.write(Message::FileDeleted);
                    messages.write(handle);
                    messages.write(request_id);
                }
                Command::DecodeImage => {
                    let handle: RenderImageHandle = self.command_queue.read();
                    let request_id: u64 = self.command_queue.read();
                    let bytes = self.command_queue.pop_bytes();
                    lock.unlock();
                    if let Some(image) = self.factory().decode_image(&bytes) {
                        self.assets.borrow_mut().images.insert(handle, image);
                        let mut messages = self.command_queue.message_lock();
                        messages.write(Message::ImageDecoded);
                        messages.write(handle);
                        messages.write(request_id);
                    } else {
                        self.error(
                            handle,
                            request_id,
                            Message::ImageError,
                            "Command Server failed to decode image".into(),
                        );
                    }
                }
                Command::ExternalImage => {
                    let handle: RenderImageHandle = self.command_queue.read();
                    let request_id: u64 = self.command_queue.read();
                    let image = self.command_queue.pop_external_image();
                    lock.unlock();
                    if let Some(image) = image {
                        let image: Rc<dyn nuxie_render_api::RenderImage + Send> = Rc::from(image);
                        let image: RenderImageRef = image;
                        self.assets.borrow_mut().images.insert(handle, image);
                        let mut messages = self.command_queue.message_lock();
                        messages.write(Message::ImageDecoded);
                        messages.write(handle);
                        messages.write(request_id);
                    } else {
                        self.error(
                            handle,
                            request_id,
                            Message::ImageError,
                            "External image was empty".into(),
                        );
                    }
                }
                Command::DeleteImage => {
                    let handle: RenderImageHandle = self.command_queue.read();
                    let request_id: u64 = self.command_queue.read();
                    lock.unlock();
                    let mut assets = self.assets.borrow_mut();
                    assets.images.remove(&handle);
                    if let Some(name) = assets
                        .image_assets
                        .iter()
                        .find_map(|(name, value)| (*value == handle).then(|| name.clone()))
                    {
                        assets.image_assets.remove(&name);
                    }
                    drop(assets);
                    let mut messages = self.command_queue.message_lock();
                    messages.write(Message::ImageDeleted);
                    messages.write(handle);
                    messages.write(request_id);
                }
                Command::DecodeBlob => {
                    let handle: BlobAssetHandle = self.command_queue.read();
                    let request_id: u64 = self.command_queue.read();
                    let bytes = self.command_queue.pop_bytes();
                    lock.unlock();
                    self.blobs.insert(
                        handle,
                        Arc::new(RuntimeBlobAsset::new("", Arc::from(bytes))),
                    );
                    let mut messages = self.command_queue.message_lock();
                    messages.write(Message::BlobDecoded);
                    messages.write(handle);
                    messages.write(request_id);
                }
                Command::ExternalBlob => {
                    let handle: BlobAssetHandle = self.command_queue.read();
                    let request_id: u64 = self.command_queue.read();
                    let blob = self.command_queue.pop_external_blob();
                    lock.unlock();
                    if let Some(blob) = blob {
                        self.blobs.insert(handle, blob);
                        let mut messages = self.command_queue.message_lock();
                        messages.write(Message::BlobDecoded);
                        messages.write(handle);
                        messages.write(request_id);
                    } else {
                        self.error(
                            handle,
                            request_id,
                            Message::BlobError,
                            "External blob was empty".into(),
                        );
                    }
                }
                Command::DeleteBlob => {
                    let handle: BlobAssetHandle = self.command_queue.read();
                    let request_id: u64 = self.command_queue.read();
                    lock.unlock();
                    self.blobs.remove(&handle);
                    let mut messages = self.command_queue.message_lock();
                    messages.write(Message::BlobDeleted);
                    messages.write(handle);
                    messages.write(request_id);
                }
                Command::DecodeAudio => {
                    let handle: AudioSourceHandle = self.command_queue.read();
                    let request_id: u64 = self.command_queue.read();
                    let bytes = self.command_queue.pop_bytes();
                    lock.unlock();
                    if let Some(audio) = self.factory().decode_audio(&bytes) {
                        self.assets.borrow_mut().audio_sources.insert(handle, audio);
                        let mut messages = self.command_queue.message_lock();
                        messages.write(Message::AudioDecoded);
                        messages.write(handle);
                        messages.write(request_id);
                    } else {
                        self.error(
                            handle,
                            request_id,
                            Message::AudioError,
                            "Command Server failed to decode audio".into(),
                        );
                    }
                }
                Command::ExternalAudio => {
                    let handle: AudioSourceHandle = self.command_queue.read();
                    let request_id: u64 = self.command_queue.read();
                    let audio = self.command_queue.pop_external_audio();
                    lock.unlock();
                    if let Some(audio) = audio {
                        self.assets.borrow_mut().audio_sources.insert(handle, audio);
                        let mut messages = self.command_queue.message_lock();
                        messages.write(Message::AudioDecoded);
                        messages.write(handle);
                        messages.write(request_id);
                    } else {
                        self.error(
                            handle,
                            request_id,
                            Message::AudioError,
                            "External audio source was invalid".into(),
                        );
                    }
                }
                Command::DeleteAudio => {
                    let handle: AudioSourceHandle = self.command_queue.read();
                    let request_id: u64 = self.command_queue.read();
                    lock.unlock();
                    let mut assets = self.assets.borrow_mut();
                    assets.audio_sources.remove(&handle);
                    if let Some(name) = assets
                        .audio_assets
                        .iter()
                        .find_map(|(name, value)| (*value == handle).then(|| name.clone()))
                    {
                        assets.audio_assets.remove(&name);
                    }
                    drop(assets);
                    let mut messages = self.command_queue.message_lock();
                    messages.write(Message::AudioDeleted);
                    messages.write(handle);
                    messages.write(request_id);
                }
                Command::DecodeFont => {
                    let handle: FontHandle = self.command_queue.read();
                    let request_id: u64 = self.command_queue.read();
                    let bytes = self.command_queue.pop_bytes();
                    lock.unlock();
                    if let Some(font) = self.factory().decode_font(&bytes) {
                        self.assets.borrow_mut().fonts.insert(handle, font);
                        let mut messages = self.command_queue.message_lock();
                        messages.write(Message::FontDecoded);
                        messages.write(handle);
                        messages.write(request_id);
                    } else {
                        self.error(
                            handle,
                            request_id,
                            Message::FontError,
                            "Command Server failed to decode font".into(),
                        );
                    }
                }
                Command::ExternalFont => {
                    let handle: FontHandle = self.command_queue.read();
                    let request_id: u64 = self.command_queue.read();
                    let font = self.command_queue.pop_external_font();
                    lock.unlock();
                    if let Some(font) = font.as_ref().and_then(HbFont::from_raw_text) {
                        self.assets.borrow_mut().fonts.insert(handle, font);
                        let mut messages = self.command_queue.message_lock();
                        messages.write(Message::FontDecoded);
                        messages.write(handle);
                        messages.write(request_id);
                    } else {
                        self.error(
                            handle,
                            request_id,
                            Message::FontError,
                            "Command Server failed to decode font".into(),
                        );
                    }
                }
                Command::DeleteFont => {
                    let handle: FontHandle = self.command_queue.read();
                    let request_id: u64 = self.command_queue.read();
                    lock.unlock();
                    let mut assets = self.assets.borrow_mut();
                    assets.fonts.remove(&handle);
                    if let Some(name) = assets
                        .font_assets
                        .iter()
                        .find_map(|(name, value)| (*value == handle).then(|| name.clone()))
                    {
                        assets.font_assets.remove(&name);
                    }
                    drop(assets);
                    let mut messages = self.command_queue.message_lock();
                    messages.write(Message::FontDeleted);
                    messages.write(handle);
                    messages.write(request_id);
                }
                Command::InstantiateArtboard => {
                    let handle: ArtboardHandle = self.command_queue.read();
                    let file_handle: FileHandle = self.command_queue.read();
                    let request_id: u64 = self.command_queue.read();
                    let name = self.command_queue.pop_name();
                    lock.unlock();
                    if let Some(file) = self.get_file(file_handle) {
                        let artboard = if name.is_empty() {
                            file.bindable_artboard_default()
                        } else {
                            file.bindable_artboard_named(&name)
                        };
                        if let Some(artboard) = artboard {
                            self.artboard_dependencies.insert(handle, Vec::new());
                            self.artboards.insert(handle, artboard);
                            let mut messages = self.command_queue.message_lock();
                            messages.write(Message::ArtboardInstantiated);
                            messages.write(file_handle);
                            messages.write(handle);
                            messages.write(request_id);
                        } else {
                            self.error(
                                file_handle,
                                request_id,
                                Message::FileError,
                                format!("artboard \"{name}\" not found."),
                            );
                        }
                    } else {
                        self.error(
                            file_handle,
                            request_id,
                            Message::FileError,
                            format!("file {file_handle} not found when trying to create artboard"),
                        );
                    }
                }
                Command::SetArtboardSize => {
                    let handle = self.command_queue.read();
                    let width = self.command_queue.read();
                    let height = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    lock.unlock();
                    if let Some(artboard) = self.get_artboard_instance(handle) {
                        artboard.set_width(width);
                        artboard.set_height(height);
                    } else {
                        self.error(
                            handle,
                            request_id,
                            Message::ArtboardError,
                            format!("artboard {handle} not found when trying to set artboard size"),
                        );
                    }
                }
                Command::ResetArtboardSize => {
                    let handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    lock.unlock();
                    if let Some(artboard) = self.get_artboard_instance(handle) {
                        artboard.reset_size();
                    } else {
                        self.error(
                            handle,
                            request_id,
                            Message::ArtboardError,
                            format!(
                                "artboard {handle} not found when trying to reset artboard size"
                            ),
                        );
                    }
                }
                Command::SetArtboardVolume => {
                    let handle = self.command_queue.read();
                    let volume = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    lock.unlock();
                    if let Some(artboard) = self.get_artboard_instance(handle) {
                        artboard.set_volume(volume);
                    } else {
                        self.error(
                            handle,
                            request_id,
                            Message::ArtboardError,
                            format!(
                                "artboard {handle} not found when trying to set artboard volume"
                            ),
                        );
                    }
                }
                Command::GetArtboardVolume => {
                    let handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    lock.unlock();
                    if let Some(artboard) = self.get_artboard_instance(handle) {
                        let mut messages = self.command_queue.message_lock();
                        messages.write(Message::ArtboardVolumeReceived);
                        messages.write(handle);
                        messages.write(request_id);
                        messages.write(artboard.volume());
                    } else {
                        self.error(
                            handle,
                            request_id,
                            Message::ArtboardError,
                            format!(
                                "Invalid artboard handle {handle} when getting artboard volume"
                            ),
                        );
                    }
                }
                Command::DeleteArtboard => {
                    let handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    lock.unlock();
                    self.cleanup_artboard(handle, request_id);
                }
                Command::InstantiateViewModel
                | Command::InstantiateBlankViewModel
                | Command::InstantiateViewModelForArtboard
                | Command::InstantiateBlankViewModelForArtboard => {
                    let uses_instance_name = matches!(
                        command,
                        Command::InstantiateViewModel | Command::InstantiateViewModelForArtboard
                    );
                    let uses_artboard = matches!(
                        command,
                        Command::InstantiateViewModelForArtboard
                            | Command::InstantiateBlankViewModelForArtboard
                    );
                    let file_handle = self.command_queue.read();
                    let artboard_handle = uses_artboard.then(|| self.command_queue.read());
                    let view_handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    let view_model_name = if uses_artboard {
                        String::new()
                    } else {
                        self.command_queue.pop_name()
                    };
                    let instance_name = if uses_instance_name {
                        Some(self.command_queue.pop_name())
                    } else {
                        None
                    };
                    lock.unlock();
                    let Some(file) = self.get_file(file_handle) else {
                        self.error(
                            file_handle,
                            request_id,
                            Message::FileError,
                            format!(
                                "File {file_handle} not found when creating view model instance "
                            ),
                        );
                        continue;
                    };
                    let view_model = if let Some(artboard_handle) = artboard_handle {
                        if let Some(artboard) = self.get_artboard_instance(artboard_handle) {
                            file.default_artboard_view_model(artboard)
                        } else {
                            self.error(file_handle, request_id, Message::FileError, format!("ArtboardInstance {artboard_handle} Not found when trying to create default view model{}", if uses_instance_name { " with view model instance" } else { " with blank view model instance" }));
                            None
                        }
                    } else {
                        file.view_model_by_name(&view_model_name)
                    };
                    let Some(view_model) = view_model else {
                        self.error(
                            file_handle,
                            request_id,
                            Message::FileError,
                            format!("View model {view_model_name} not found"),
                        );
                        continue;
                    };
                    let instance = match instance_name {
                        Some(name) if name.is_empty() => view_model.create_default_instance(),
                        Some(name) => view_model.create_instance_from_name(&name),
                        None => view_model.create_instance(),
                    };
                    if let Some(instance) = instance {
                        self.view_models.insert(view_handle, instance);
                        let mut messages = self.command_queue.message_lock();
                        messages.write(Message::ViewModelInstanceInstantiated);
                        messages.write(file_handle);
                        messages.write(view_handle);
                        messages.write(request_id);
                    } else {
                        self.error(file_handle, request_id, Message::FileError, format!("Could not create view model instance from view model {view_model_name}"));
                    }
                }
                Command::AddViewModelListValue => {
                    let root_handle = self.command_queue.read();
                    let view_handle = self.command_queue.read();
                    let index: i32 = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    let path = self.command_queue.pop_name();
                    lock.unlock();
                    if let Some(root) = self.get_view_model_instance(root_handle) {
                        if let Some(view_model) = self.get_view_model_instance(view_handle) {
                            if let Some(property) = root.property_list(&path) {
                                if index >= 0 {
                                    property.add_instance_at(view_model, index);
                                } else {
                                    property.add_instance(view_model);
                                }
                            } else {
                                self.error(root_handle, request_id, Message::ViewModelError, format!("failed to find list at path {path} when trying to add to a list"));
                            }
                        } else {
                            self.error(root_handle, request_id, Message::ViewModelError, format!("failed to find value view model {view_handle} isntance for add list"));
                        }
                    } else {
                        self.error(
                            root_handle,
                            request_id,
                            Message::ViewModelError,
                            format!(
                                "failed to find root view model isntance {root_handle}for add list"
                            ),
                        );
                    }
                }
                Command::RemoveViewModelListValue => {
                    let root_handle = self.command_queue.read();
                    let view_handle = self.command_queue.read();
                    let index: i32 = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    let path = self.command_queue.pop_name();
                    lock.unlock();
                    if let Some(root) = self.get_view_model_instance(root_handle) {
                        if let Some(property) = root.property_list(&path) {
                            if index >= 0 {
                                property.remove_instance_at(index);
                            } else if let Some(view_model) =
                                self.get_view_model_instance(view_handle)
                            {
                                property.remove_instance(view_model);
                            }
                        } else {
                            self.error(root_handle, request_id, Message::ViewModelError, format!("failed to find list on view model isntance for remove at path {path}"));
                        }
                    } else {
                        self.error(
                            root_handle,
                            request_id,
                            Message::ViewModelError,
                            format!(
                                "failed to find view model instance {root_handle} for remove list"
                            ),
                        );
                    }
                }
                Command::SwapViewModelListValue => {
                    let root_handle = self.command_queue.read();
                    let index_a = self.command_queue.read();
                    let index_b = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    let path = self.command_queue.pop_name();
                    lock.unlock();
                    if let Some(view_model) = self.get_view_model_instance(root_handle) {
                        if let Some(list) = view_model.property_list(&path) {
                            list.swap(index_a, index_b);
                        } else {
                            self.error(root_handle, request_id, Message::ViewModelError, format!("failed to find list on view model isntance for swap at path {path}"));
                        }
                    } else {
                        self.error(
                            root_handle,
                            request_id,
                            Message::ViewModelError,
                            format!("failed to find view model instance {root_handle} for swap"),
                        );
                    }
                }
                Command::SubscribeViewModelProperty | Command::UnsubscribeViewModelProperty => {
                    let root_handle = self.command_queue.read();
                    let data_type = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    let name = self.command_queue.pop_name();
                    lock.unlock();
                    let data = PropertyData {
                        name,
                        data_type,
                        enum_name: String::new(),
                    };
                    if command == Command::SubscribeViewModelProperty {
                        if let Some(view) = self.get_view_model_instance(root_handle) {
                            if !matches!(
                                data_type,
                                DataType::ViewModel
                                    | DataType::Integer
                                    | DataType::None
                                    | DataType::SymbolListIndex
                            ) {
                                if view.property(&data.name).is_some() {
                                    self.property_subscriptions.push(Subscription {
                                        request_id,
                                        data,
                                        root_view_model: root_handle,
                                    });
                                } else {
                                    self.error(
                                        root_handle,
                                        request_id,
                                        Message::ViewModelError,
                                        format!(
                                            "Property {} not found when subscribing",
                                            data.name
                                        ),
                                    );
                                }
                            } else {
                                self.error(
                                    root_handle,
                                    request_id,
                                    Message::ViewModelError,
                                    format!(
                                        "Property type {} is not valid when subscribing",
                                        data_type_name(data_type)
                                    ),
                                );
                            }
                        } else {
                            self.error(root_handle, request_id, Message::ViewModelError, format!("Root view model {root_handle} not found when subscribing to property {}", data.name));
                        }
                    } else {
                        self.property_subscriptions.retain(|value| {
                            !(value.data.name == data.name
                                && value.data.data_type == data.data_type
                                && value.root_view_model == root_handle)
                        });
                    }
                }
                Command::RefNestedViewModel => {
                    let root = self.command_queue.read();
                    let nested = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    let path = self.command_queue.pop_name();
                    lock.unlock();
                    if let Some(root_view) = self.get_view_model_instance(root) {
                        if let Some(value) = root_view.property_view_model(&path) {
                            self.view_models.insert(nested, value);
                        } else {
                            self.error(nested, request_id, Message::ViewModelError, format!("Nested view not found at path{path} when refing nested view model"));
                        }
                    } else {
                        self.error(nested, request_id, Message::ViewModelError, format!("Root view model {root} not found when refing nested view model at path {path}"));
                    }
                }
                Command::RefListViewModel => {
                    let root = self.command_queue.read();
                    let index = self.command_queue.read();
                    let list_handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    let path = self.command_queue.pop_name();
                    lock.unlock();
                    if let Some(root_view) = self.get_view_model_instance(root) {
                        if let Some(list) = root_view.property_list(&path) {
                            if let Some(value) = list.instance_at(index) {
                                self.view_models.insert(list_handle, value);
                            } else {
                                self.error(
                                    root,
                                    request_id,
                                    Message::ViewModelError,
                                    format!("View model not found on list {path} at index {index}"),
                                );
                            }
                        } else {
                            self.error(root, request_id, Message::ViewModelError, format!("List not found at path {path} when refing view model at index {index}"));
                        }
                    } else {
                        self.error(root, request_id, Message::ViewModelError, format!("Root view model{root} not found when refing nested view model at path {path}"));
                    }
                }
                Command::DeleteViewModel => {
                    let handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    lock.unlock();
                    self.view_models.remove(&handle);
                    let mut messages = self.command_queue.message_lock();
                    messages.write(Message::ViewModelDeleted);
                    messages.write(handle);
                    messages.write(request_id);
                }
                Command::InstantiateStateMachine => {
                    let handle = self.command_queue.read();
                    let artboard_handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    let name = self.command_queue.pop_name();
                    lock.unlock();
                    if let Some(artboard) = self.get_artboard_instance(artboard_handle) {
                        let instance = if name.is_empty() {
                            artboard.default_state_machine()
                        } else {
                            artboard.state_machine_named(&name)
                        };
                        if let Some(instance) = instance {
                            self.state_machines
                                .lock()
                                .unwrap()
                                .insert(handle, Arc::new(SynchronizedStateMachine::new(instance)));
                            self.artboard_dependencies
                                .get_mut(&artboard_handle)
                                .unwrap()
                                .push(handle);
                            let mut messages = self.command_queue.message_lock();
                            messages.write(Message::StateMachineInstantiated);
                            messages.write(artboard_handle);
                            messages.write(handle);
                            messages.write(request_id);
                        } else {
                            self.error(artboard_handle, request_id, Message::ArtboardError, format!("Could not create state machine with name \"{name}\" because it was not found."));
                        }
                    } else {
                        self.error(artboard_handle, request_id, Message::ArtboardError, format!("Could not create state machine with name \"{name}\" because the owning artboard {artboard_handle} was not found."));
                    }
                }
                Command::BindViewModelInstance | Command::SetViewModelInstance => {
                    let handle = self.command_queue.read();
                    let view_handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    lock.unlock();
                    if let Some(wrapper) = self.get_state_machine_wrapper(handle) {
                        if let Some(view) = self.get_view_model_instance(view_handle) {
                            let mut instance = wrapper.instance.lock().unwrap();
                            if command == Command::BindViewModelInstance {
                                instance.bind_view_model_instance(view.instance());
                            } else {
                                instance.set_view_model_instance(view.instance());
                            }
                        } else {
                            self.error(handle, request_id, Message::StateMachineError, format!("View model instance {view_handle} not found when trying to bind or set on a state machine"));
                        }
                    } else {
                        self.error(
                            handle,
                            request_id,
                            Message::StateMachineError,
                            format!("State machine {handle} not found for binding view model."),
                        );
                    }
                }
                Command::SetGlobalViewModelInstance => {
                    let handle = self.command_queue.read();
                    let view_handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    let name = self.command_queue.pop_name();
                    lock.unlock();
                    if let Some(wrapper) = self.get_state_machine_wrapper(handle) {
                        if let Some(view) = self.get_view_model_instance(view_handle) {
                            if !wrapper
                                .instance
                                .lock()
                                .unwrap()
                                .set_global_view_model_instance(&name, view.instance())
                            {
                                self.error(handle, request_id, Message::StateMachineError, format!("Could not set global view model instance {view_handle} under name {name} on a state machine"));
                            }
                        } else {
                            self.error(handle, request_id, Message::StateMachineError, format!("View model instance {view_handle} not found when trying to set a global view model instance on a state machine"));
                        }
                    } else {
                        self.error(handle, request_id, Message::StateMachineError, format!("State machine {handle} not found for setting global view model instance."));
                    }
                }
                Command::GetGlobalViewModelInstance => {
                    let handle = self.command_queue.read();
                    let view_handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    let name = self.command_queue.pop_name();
                    lock.unlock();
                    if let Some(wrapper) = self.get_state_machine_wrapper(handle) {
                        if let Some(view) = wrapper
                            .instance
                            .lock()
                            .unwrap()
                            .global_view_model_instance(&name)
                        {
                            self.view_models
                                .insert(view_handle, Rc::new(ViewModelInstanceRuntime::new(view)));
                        } else {
                            self.error(view_handle, request_id, Message::ViewModelError, format!("No global view model instance bound under name {name} on a state machine"));
                        }
                    } else {
                        self.error(view_handle, request_id, Message::ViewModelError, format!("State machine {handle} not found for getting global view model instance."));
                    }
                }
                Command::Bind => {
                    let handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    lock.unlock();
                    if let Some(wrapper) = self.get_state_machine_wrapper(handle) {
                        wrapper.instance.lock().unwrap().bind();
                    } else {
                        self.error(
                            handle,
                            request_id,
                            Message::StateMachineError,
                            format!("State machine {handle} not found for binding data context."),
                        );
                    }
                }
                Command::AdvanceStateMachine => {
                    let handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    let elapsed = self.command_queue.read();
                    lock.unlock();
                    if let Some(wrapper) = self.get_state_machine_wrapper(handle) {
                        if !wrapper.instance.lock().unwrap().advance_and_apply(elapsed) {
                            let mut messages = self.command_queue.message_lock();
                            messages.write(Message::StateMachineSettled);
                            messages.write(handle);
                            messages.write(request_id);
                        }
                    } else {
                        self.error(
                            handle,
                            request_id,
                            Message::StateMachineError,
                            format!("State machine {handle} not found for advance."),
                        );
                    }
                }
                Command::DeleteStateMachine => {
                    let handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    lock.unlock();
                    self.state_machines.lock().unwrap().remove(&handle);
                    let mut messages = self.command_queue.message_lock();
                    messages.write(Message::StateMachineDeleted);
                    messages.write(handle);
                    messages.write(request_id);
                }
                Command::EnableSemantics => {
                    let handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    lock.unlock();
                    if let Some(wrapper) = self.get_state_machine_wrapper(handle) {
                        wrapper.instance.lock().unwrap().enable_semantics();
                    } else {
                        self.error(
                            handle,
                            request_id,
                            Message::StateMachineError,
                            format!("State machine {handle} not found for enableSemantics."),
                        );
                    }
                }
                Command::DrainSemanticsDiff => {
                    let handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    let fit = self.command_queue.read();
                    let ax = self.command_queue.read();
                    let ay = self.command_queue.read();
                    let scale = self.command_queue.read();
                    let view_bounds = self.command_queue.read();
                    lock.unlock();
                    if let Some(wrapper) = self.get_state_machine_wrapper(handle) {
                        let mut instance = wrapper.instance.lock().unwrap();
                        if let Some(manager) = instance.semantic_manager_mut() {
                            let mut diff = manager.drain_diff();
                            if !diff.empty() {
                                let mut transform = Mat2D::identity();
                                if let Some(artboard) = instance.artboard_mut() {
                                    let surface =
                                        Aabb::from_min_max(Vec2D::new(0.0, 0.0), view_bounds);
                                    if surface.width() != 0.0 && surface.height() != 0.0 {
                                        transform = compute_alignment(
                                            fit,
                                            Alignment::new(ax, ay),
                                            &surface,
                                            &artboard.bounds(),
                                            scale,
                                        );
                                    }
                                }
                                drop(instance);
                                if transform != Mat2D::identity() {
                                    Self::map_semantics_diff_to_view_space(&mut diff, &transform);
                                }
                                let mut messages = self.command_queue.message_lock();
                                messages.write(Message::SemanticsDiffReceived);
                                messages.write(handle);
                                messages.write(request_id);
                                messages.write_semantics_diff(diff);
                            }
                        } else {
                            drop(instance);
                            self.error(handle, request_id, Message::StateMachineError, format!("Semantics not enabled on state machine {handle}; call enableSemantics before drainSemanticsDiff."));
                        }
                    } else {
                        self.error(
                            handle,
                            request_id,
                            Message::StateMachineError,
                            format!("State machine {handle} not found for drainSemanticsDiff."),
                        );
                    }
                }
                Command::FireSemanticAction => {
                    let handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    let node_id = self.command_queue.read();
                    let action = self.command_queue.read();
                    lock.unlock();
                    if let Some(wrapper) = self.get_state_machine_wrapper(handle) {
                        let mut instance = wrapper.instance.lock().unwrap();
                        if instance.semantic_manager().is_some() {
                            instance.fire_semantic_action(node_id, action);
                        } else {
                            drop(instance);
                            self.error(handle, request_id, Message::StateMachineError, format!("Semantics not enabled on state machine {handle}; call enableSemantics before fireSemanticAction."));
                        }
                    } else {
                        self.error(
                            handle,
                            request_id,
                            Message::StateMachineError,
                            format!("State machine {handle} not found for fireSemanticAction."),
                        );
                    }
                }
                Command::RequestSemanticFocus => {
                    let handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    let node_id = self.command_queue.read();
                    lock.unlock();
                    if let Some(wrapper) = self.get_state_machine_wrapper(handle) {
                        let mut instance = wrapper.instance.lock().unwrap();
                        if let Some(manager) = instance.semantic_manager_mut() {
                            manager.request_focus(node_id);
                        } else {
                            drop(instance);
                            self.error(handle, request_id, Message::StateMachineError, format!("Semantics not enabled on state machine {handle}; call enableSemantics before requestSemanticFocus."));
                        }
                    } else {
                        self.error(
                            handle,
                            request_id,
                            Message::StateMachineError,
                            format!("State machine {handle} not found for requestSemanticFocus."),
                        );
                    }
                }
                Command::ClearSemanticFocus => {
                    let handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    lock.unlock();
                    if let Some(wrapper) = self.get_state_machine_wrapper(handle) {
                        if let Some(manager) = wrapper.instance.lock().unwrap().focus_manager_mut()
                        {
                            manager.clear_focus();
                        }
                    } else {
                        self.error(
                            handle,
                            request_id,
                            Message::StateMachineError,
                            format!("State machine {handle} not found for clearSemanticFocus."),
                        );
                    }
                }
                Command::RunOnce => {
                    let callback: CommandServerCallback = self.command_queue.pop_callback();
                    lock.unlock();
                    callback(self);
                }
                Command::Draw => {
                    let key = self.command_queue.read();
                    let callback = self.command_queue.pop_draw_callback();
                    lock.unlock();
                    self.unique_draws.insert(key, callback);
                }
                Command::CancelDraw => {
                    let key = self.command_queue.read();
                    lock.unlock();
                    self.unique_draws.remove(&key);
                }
                Command::CommandLoopBreak => {
                    lock.unlock();
                    should_process_commands = false;
                }
                Command::ListArtboards => {
                    let handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    lock.unlock();
                    if let Some(file) = self.get_file(handle) {
                        let values = file.artboards();
                        let mut m = self.command_queue.message_lock();
                        m.write(Message::ArtboardsListed);
                        m.write(handle);
                        m.write(request_id);
                        m.write(values.len());
                        for value in values {
                            m.write_name(value.name().to_owned());
                        }
                    } else {
                        self.error(
                            handle,
                            request_id,
                            Message::FileError,
                            format!("Invalid file handle {handle} when getting list of artboards"),
                        );
                    }
                }
                Command::ListFileAssets => {
                    let handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    lock.unlock();
                    if let Some(file) = self.get_file(handle) {
                        let values = file.assets();
                        let mut m = self.command_queue.message_lock();
                        m.write(Message::FileAssetsListed);
                        m.write(handle);
                        m.write(request_id);
                        m.write(values.len());
                        for value in values {
                            m.write(value.asset_id());
                            m.write(value.core_type());
                            m.write_name(value.name().into());
                            m.write_name(value.cdn_uuid_str());
                            m.write_name(value.cdn_base_url());
                            m.write_name(value.file_extension());
                        }
                    } else {
                        self.error(
                            handle,
                            request_id,
                            Message::FileError,
                            format!(
                                "Invalid file handle {handle} when getting list of file assets"
                            ),
                        );
                    }
                }
                Command::GetViewModelInstanceViewModelName | Command::GetViewModelInstanceName => {
                    let handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    lock.unlock();
                    if let Some(value) = self.get_view_model_instance(handle) {
                        let mut m = self.command_queue.message_lock();
                        m.write(if command == Command::GetViewModelInstanceViewModelName {
                            Message::ViewModelInstanceViewModelNameReceived
                        } else {
                            Message::ViewModelInstanceNameReceived
                        });
                        m.write(handle);
                        m.write(request_id);
                        m.write_name(
                            if command == Command::GetViewModelInstanceViewModelName {
                                value.view_model_name()
                            } else {
                                value.name()
                            }
                            .to_owned(),
                        );
                    } else {
                        self.error(
                            handle,
                            request_id,
                            Message::ViewModelError,
                            format!("Invalid view model instance handle {handle}"),
                        );
                    }
                }
                Command::ListViewModelEnums => {
                    let handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    lock.unlock();
                    if let Some(file) = self.get_file(handle) {
                        let values = file.enums();
                        let mut m = self.command_queue.message_lock();
                        m.write(Message::ViewModelEnumsListed);
                        m.write(handle);
                        m.write(request_id);
                        m.write(values.len());
                        for value in values {
                            m.write_name(value.enum_name().into());
                            let enumerants = value.values();
                            m.write(enumerants.len());
                            for enumerant in enumerants {
                                m.write_name(enumerant.key().into());
                            }
                        }
                    } else {
                        self.error(
                            handle,
                            request_id,
                            Message::FileError,
                            format!("Invalid file handle {handle} when getting list of enums"),
                        );
                    }
                }
                Command::ListStateMachines => {
                    let handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    lock.unlock();
                    if let Some(artboard) = self.get_artboard_instance(handle) {
                        let count = artboard.state_machine_count();
                        let mut m = self.command_queue.message_lock();
                        m.write(Message::StateMachinesListed);
                        m.write(handle);
                        m.write(request_id);
                        m.write(count);
                        for index in 0..count {
                            m.write_name(artboard.state_machine_name_at(index));
                        }
                    } else {
                        self.error(handle, request_id, Message::ArtboardError, format!("Invalid artboard handle {handle} when getting list of state machines"));
                    }
                }
                Command::GetArtboardSize => {
                    let handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    lock.unlock();
                    if let Some(artboard) = self.get_artboard_instance(handle) {
                        let mut m = self.command_queue.message_lock();
                        m.write(Message::ArtboardSizeReceived);
                        m.write(handle);
                        m.write(request_id);
                        m.write(artboard.width());
                        m.write(artboard.height());
                    } else {
                        self.error(
                            handle,
                            request_id,
                            Message::ArtboardError,
                            format!("Invalid artboard handle {handle} when getting artboard size"),
                        );
                    }
                }
                Command::GetDefaultViewModel => {
                    let file_handle = self.command_queue.read();
                    let artboard_handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    lock.unlock();
                    if let Some(artboard) = self.get_artboard_instance(artboard_handle) {
                        if let Some(file) = self.get_file(file_handle) {
                            if let Some(model) = file.default_artboard_view_model(artboard) {
                                if let Some(instance) = model.create_default_instance() {
                                    let mut m = self.command_queue.message_lock();
                                    m.write(Message::DefaultViewModelReceived);
                                    m.write(artboard_handle);
                                    m.write(request_id);
                                    m.write_name(model.name().into());
                                    m.write_name(instance.name().into());
                                } else {
                                    self.error(artboard_handle, request_id, Message::ArtboardError, "Could not find default view model instance for artboard when getting default view model info".into());
                                }
                            } else {
                                self.error(artboard_handle, request_id, Message::ArtboardError, "Could not find default view model for artboard when getting default view model info".into());
                            }
                        } else {
                            self.error(artboard_handle, request_id, Message::ArtboardError, format!("Invalid file handle {file_handle} when getting default view model info"));
                        }
                    } else {
                        self.error(artboard_handle, request_id, Message::ArtboardError, format!("Invalid artboard handle {artboard_handle} when getting default view model info"));
                    }
                }
                Command::ListViewModels => {
                    let handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    lock.unlock();
                    if let Some(file) = self.get_file(handle) {
                        let count = file.view_model_count();
                        let mut m = self.command_queue.message_lock();
                        m.write(Message::ViewModelsListed);
                        m.write(handle);
                        m.write(request_id);
                        m.write(count);
                        for index in 0..count {
                            m.write_name(file.view_model_by_index(index).unwrap().name().into());
                        }
                    } else {
                        self.error(
                            handle,
                            request_id,
                            Message::FileError,
                            format!(
                                "Invalid file handle {handle} when getting list of view models"
                            ),
                        );
                    }
                }
                Command::ListGlobalViewModelNames => {
                    let handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    lock.unlock();
                    if let Some(file) = self.get_file(handle) {
                        let names = file.global_view_model_names();
                        let mut m = self.command_queue.message_lock();
                        m.write(Message::GlobalViewModelNamesListed);
                        m.write(handle);
                        m.write(request_id);
                        m.write(names.len());
                        for name in names {
                            m.write_name(name);
                        }
                    } else {
                        self.error(handle, request_id, Message::FileError, format!("Invalid file handle {handle} when getting list of global view models"));
                    }
                }
                Command::ListViewModelInstanceNames => {
                    let handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    let model_name = self.command_queue.pop_name();
                    lock.unlock();
                    if let Some(file) = self.get_file(handle) {
                        if let Some(model) = file.view_model_by_name(&model_name) {
                            let names = model.instance_names();
                            let mut m = self.command_queue.message_lock();
                            m.write(Message::ViewModelInstanceNamesListed);
                            m.write(handle);
                            m.write(request_id);
                            m.write(names.len());
                            m.write_name(model_name);
                            for name in names {
                                m.write_name(name);
                            }
                        } else {
                            self.error(handle, request_id, Message::ViewModelError, format!("Invalid view model name {model_name} when getting list of view model instance names"));
                        }
                    } else {
                        self.error(handle, request_id, Message::ViewModelError, format!("Invalid file handle {handle} when getting list of view model instance names"));
                    }
                }
                Command::ListViewModelProperties => {
                    let handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    let model_name = self.command_queue.pop_name();
                    lock.unlock();
                    if let Some(file) = self.get_file(handle) {
                        if let Some(model) = file.view_model_by_name(&model_name) {
                            let instance = model.create_default_instance().unwrap();
                            let properties = model.properties();
                            let mut m = self.command_queue.message_lock();
                            m.write(Message::ViewModelPropertiesListed);
                            m.write(handle);
                            m.write(request_id);
                            m.write(properties.len());
                            m.write_name(model_name);
                            for property in properties {
                                m.write(property.data_type);
                                m.write_name(property.name.clone());
                                if property.data_type == DataType::Enum {
                                    m.write_name(
                                        instance
                                            .property_enum(&property.name)
                                            .unwrap()
                                            .enum_type()
                                            .into(),
                                    );
                                }
                                if property.data_type == DataType::ViewModel {
                                    m.write_name(
                                        instance
                                            .property_view_model(&property.name)
                                            .map_or("Unkown".into(), |value| {
                                                value.instance().view_model().name().into()
                                            }),
                                    );
                                }
                            }
                        } else {
                            self.error(handle, request_id, Message::FileError, format!("Invalid view model name {model_name} when getting list of view model properties"));
                        }
                    } else {
                        self.error(handle, request_id, Message::FileError, format!("Invalid file handle {handle} when getting list of view model properties"));
                    }
                }
                Command::SetViewModelInstanceValue => {
                    let handle = self.command_queue.read();
                    let data_type = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    let name = self.command_queue.pop_name();
                    let mut value = ViewModelInstanceData::new(PropertyData {
                        name,
                        data_type,
                        enum_name: String::new(),
                    });
                    let mut nested = ViewModelInstanceHandle::NULL;
                    let mut image = RenderImageHandle::NULL;
                    let mut blob = BlobAssetHandle::NULL;
                    let mut artboard = ArtboardHandle::NULL;
                    value.value = match data_type {
                        DataType::Trigger => ViewModelInstanceValue::None,
                        DataType::Boolean => {
                            ViewModelInstanceValue::Bool(self.command_queue.read())
                        }
                        DataType::Number => {
                            ViewModelInstanceValue::Number(self.command_queue.read())
                        }
                        DataType::Color => ViewModelInstanceValue::Color(self.command_queue.read()),
                        DataType::String | DataType::Enum => {
                            ViewModelInstanceValue::String(self.command_queue.pop_name())
                        }
                        DataType::ViewModel => {
                            nested = self.command_queue.read();
                            ViewModelInstanceValue::None
                        }
                        DataType::AssetImage => {
                            image = self.command_queue.read();
                            ViewModelInstanceValue::None
                        }
                        DataType::AssetBlob => {
                            blob = self.command_queue.read();
                            ViewModelInstanceValue::None
                        }
                        DataType::Artboard => {
                            artboard = self.command_queue.read();
                            ViewModelInstanceValue::None
                        }
                        _ => unreachable!(),
                    };
                    lock.unlock();
                    self.set_view_model_value(
                        handle, request_id, value, nested, image, blob, artboard,
                    );
                }
                Command::ListViewModelPropertyValue => {
                    let data_type = self.command_queue.read();
                    let handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    let name = self.command_queue.pop_name();
                    lock.unlock();
                    self.list_view_model_property_value(
                        handle,
                        request_id,
                        PropertyData {
                            name,
                            data_type,
                            enum_name: String::new(),
                        },
                    );
                }
                Command::GetViewModelListSize => {
                    let handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    let path = self.command_queue.pop_name();
                    lock.unlock();
                    if let Some(view) = self.get_view_model_instance(handle) {
                        if let Some(list) = view.property_list(&path) {
                            let mut m = self.command_queue.message_lock();
                            m.write(Message::ViewModelListSizeReceived);
                            m.write(handle);
                            m.write(list.size());
                            m.write(request_id);
                            m.write_name(path);
                        } else {
                            self.error(
                                handle,
                                request_id,
                                Message::ViewModelError,
                                format!("failed to get list at path {path} when getting list size"),
                            );
                        }
                    } else {
                        self.error(
                            handle,
                            request_id,
                            Message::ViewModelError,
                            format!("failed to get view model {handle} when getting list size"),
                        );
                    }
                }
                Command::ClearViewModelList => {
                    let handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    let path = self.command_queue.pop_name();
                    lock.unlock();
                    if let Some(view) = self.get_view_model_instance(handle) {
                        if let Some(list) = view.property_list(&path) {
                            list.remove_all_instances();
                            let mut m = self.command_queue.message_lock();
                            m.write(Message::ViewModelListCleared);
                            m.write(handle);
                            m.write(request_id);
                            m.write_name(path);
                        } else {
                            self.error(
                                handle,
                                request_id,
                                Message::ViewModelError,
                                format!("failed to get list at path {path} when clearing list"),
                            );
                        }
                    } else {
                        self.error(
                            handle,
                            request_id,
                            Message::ViewModelError,
                            format!("failed to get view model {handle} when clearing list"),
                        );
                    }
                }
                Command::PointerMove
                | Command::PointerDown
                | Command::PointerUp
                | Command::PointerExit => {
                    let handle = self.command_queue.read();
                    let request_id = self.command_queue.read();
                    let event = self.command_queue.pop_pointer_event();
                    lock.unlock();
                    if let Some(wrapper) = self.get_state_machine_wrapper(handle) {
                        let mut instance = wrapper.instance.lock().unwrap();
                        let position = self.cursor_pos_for_pointer_event(&instance, &event);
                        match command {
                            Command::PointerMove => {
                                instance.pointer_move(position, 0.0, event.pointer_id);
                            }
                            Command::PointerDown => {
                                if instance.pointer_down(position, event.pointer_id)
                                    != HitResult::None
                                {
                                    instance.advance_and_apply(0.0);
                                }
                            }
                            Command::PointerUp => {
                                if instance.pointer_up(position, event.pointer_id)
                                    != HitResult::None
                                {
                                    instance.advance_and_apply(0.0);
                                }
                            }
                            Command::PointerExit => {
                                instance.pointer_exit(position, event.pointer_id);
                            }
                            _ => unreachable!(),
                        }
                    } else {
                        self.error(
                            handle,
                            request_id,
                            Message::StateMachineError,
                            format!("State machine \"{handle}\" not found for pointer event."),
                        );
                    }
                }
                Command::AddImageFileAsset => {
                    let handle = self.command_queue.read();
                    let _request_id: u64 = self.command_queue.read();
                    let name = self.command_queue.pop_name();
                    lock.unlock();
                    if handle != RenderImageHandle::NULL && self.get_image(handle).is_some() {
                        self.assets.borrow_mut().image_assets.insert(name, handle);
                    }
                }
                Command::RemoveImageFileAsset => {
                    let _request_id: u64 = self.command_queue.read();
                    let name = self.command_queue.pop_name();
                    lock.unlock();
                    self.assets.borrow_mut().image_assets.remove(&name);
                }
                Command::AddAudioFileAsset => {
                    let handle = self.command_queue.read();
                    let _request_id: u64 = self.command_queue.read();
                    let name = self.command_queue.pop_name();
                    lock.unlock();
                    if handle != AudioSourceHandle::NULL && self.get_audio_source(handle).is_some()
                    {
                        self.assets.borrow_mut().audio_assets.insert(name, handle);
                    }
                }
                Command::RemoveAudioFileAsset => {
                    let _request_id: u64 = self.command_queue.read();
                    let name = self.command_queue.pop_name();
                    lock.unlock();
                    self.assets.borrow_mut().audio_assets.remove(&name);
                }
                Command::AddFontFileAsset => {
                    let handle = self.command_queue.read();
                    let _request_id: u64 = self.command_queue.read();
                    let name = self.command_queue.pop_name();
                    lock.unlock();
                    if handle != FontHandle::NULL && self.get_font(handle).is_some() {
                        self.assets.borrow_mut().font_assets.insert(name, handle);
                    }
                }
                Command::RemoveFontFileAsset => {
                    let _request_id: u64 = self.command_queue.read();
                    let name = self.command_queue.pop_name();
                    lock.unlock();
                    self.assets.borrow_mut().font_assets.remove(&name);
                }
                Command::Disconnect => {
                    lock.unlock();
                    self.was_disconnect_received = true;
                    return false;
                }
            }
            assert!(!lock.owns_lock());
            lock.lock();
            if self.command_queue.command_stream_is_empty() || !should_process_commands {
                break;
            }
        }
        lock.unlock();
        let mut unique_draws = std::mem::take(&mut self.unique_draws);
        for (key, callback) in &mut unique_draws {
            callback(*key, self);
        }
        self.unique_draws.clear();
        self.check_property_subscriptions();
        !self.was_disconnect_received
    }

    fn set_view_model_value(
        &mut self,
        handle: ViewModelInstanceHandle,
        request_id: u64,
        value: ViewModelInstanceData,
        nested_handle: ViewModelInstanceHandle,
        image_handle: RenderImageHandle,
        blob_handle: BlobAssetHandle,
        artboard_handle: ArtboardHandle,
    ) {
        let Some(view) = self.get_view_model_instance(handle) else {
            self.error(
                handle,
                request_id,
                Message::ViewModelError,
                format!(
                    "Could not find view model instance when setting property type {} with path {}",
                    data_type_name(value.meta_data.data_type),
                    value.meta_data.name
                ),
            );
            return;
        };
        let name = &value.meta_data.name;
        let missing = || {
            format!(
                "Could not find view model property instance when setting property type {} with path {name}",
                data_type_name(value.meta_data.data_type)
            )
        };
        match value.meta_data.data_type {
            DataType::Trigger => {
                if let Some(property) = view.property_trigger(name) {
                    property.trigger();
                } else {
                    self.error(handle, request_id, Message::ViewModelError, missing());
                }
            }
            DataType::Boolean => {
                if let Some(property) = view.property_boolean(name) {
                    let ViewModelInstanceValue::Bool(value) = &value.value else {
                        unreachable!()
                    };
                    property.set_value(*value);
                } else {
                    self.error(handle, request_id, Message::ViewModelError, missing());
                }
            }
            DataType::Number => {
                if let Some(property) = view.property_number(name) {
                    let ViewModelInstanceValue::Number(value) = &value.value else {
                        unreachable!()
                    };
                    property.set_value(*value);
                } else {
                    self.error(handle, request_id, Message::ViewModelError, missing());
                }
            }
            DataType::Color => {
                if let Some(property) = view.property_color(name) {
                    let ViewModelInstanceValue::Color(value) = &value.value else {
                        unreachable!()
                    };
                    property.set_value(*value);
                } else {
                    self.error(handle, request_id, Message::ViewModelError, missing());
                }
            }
            DataType::String => {
                if let Some(property) = view.property_string(name) {
                    let ViewModelInstanceValue::String(value) = &value.value else {
                        unreachable!()
                    };
                    property.set_value(value);
                } else {
                    self.error(handle, request_id, Message::ViewModelError, missing());
                }
            }
            DataType::Enum => {
                if let Some(property) = view.property_enum(name) {
                    let ViewModelInstanceValue::String(value) = &value.value else {
                        unreachable!()
                    };
                    let values = property.values();
                    if values.contains(value) {
                        property.set_value(value);
                    } else {
                        self.error(handle, request_id, Message::ViewModelError, format!("Invalid enum value for property {name} when trying to set enum to {value} possible values {values:?}"));
                    }
                } else {
                    self.error(handle, request_id, Message::ViewModelError, missing());
                }
            }
            DataType::ViewModel => {
                if let Some(nested) = self.get_view_model_instance(nested_handle) {
                    if !view.replace_view_model(name, nested) {
                        self.error(
                            handle,
                            request_id,
                            Message::ViewModelError,
                            format!("Could not replace view model at path {name}"),
                        );
                    }
                } else {
                    self.error(handle, request_id, Message::ViewModelError, format!("Could not find nested view model with handle {nested_handle} to set for view model instance when setting property with path {name}"));
                }
            }
            DataType::AssetImage => {
                if let Some(property) = view.property_image(name) {
                    if image_handle == RenderImageHandle::NULL {
                        property.set_value(None);
                    } else if let Some(image) = self.get_image(image_handle) {
                        property.set_value(Some(image));
                    } else {
                        self.error(handle, request_id, Message::ViewModelError, format!("Could not find image {image_handle} to set for view model instance when setting property with path {name}"));
                    }
                } else {
                    self.error(
                        handle,
                        request_id,
                        Message::ViewModelError,
                        format!("Could not find image property at path {name}"),
                    );
                }
            }
            DataType::AssetBlob => {
                if let Some(property) = view.property_blob(name) {
                    if blob_handle == BlobAssetHandle::NULL {
                        property.set_value(None);
                    } else if let Some(blob) = self.get_blob(blob_handle) {
                        property.set_value(Some(blob));
                    } else {
                        self.error(handle, request_id, Message::ViewModelError, format!("Could not find blob {blob_handle} to set for view model instance when setting property with path {name}"));
                    }
                } else {
                    self.error(
                        handle,
                        request_id,
                        Message::ViewModelError,
                        format!("Could not find blob property at path {name}"),
                    );
                }
            }
            DataType::Artboard => {
                if let Some(property) = view.property_artboard(name) {
                    if artboard_handle == ArtboardHandle::NULL {
                        property.set_value(None);
                    } else if let Some(artboard) = self.get_bindable_artboard(artboard_handle) {
                        property.set_value(Some(artboard));
                    } else {
                        self.error(handle, request_id, Message::ViewModelError, format!("Could not find artboard {artboard_handle} to set for view model instance when setting property with path {name}"));
                    }
                } else {
                    self.error(
                        handle,
                        request_id,
                        Message::ViewModelError,
                        format!("Could not find artboard property at path {name}"),
                    );
                }
            }
            _ => unreachable!(),
        }
    }

    fn list_view_model_property_value(
        &self,
        handle: ViewModelInstanceHandle,
        request_id: u64,
        data: PropertyData,
    ) {
        let Some(view) = self.get_view_model_instance(handle) else {
            self.error(
                handle,
                request_id,
                Message::ViewModelError,
                format!(
                    "Could not find view model instance when getting property type {} with path {}",
                    data_type_name(data.data_type),
                    data.name
                ),
            );
            return;
        };
        let mut value = ViewModelInstanceData::new(data);
        let name = &value.meta_data.name;
        let missing = || {
            format!(
                "Could not find view model property instance when getting property type {} with path {name}",
                data_type_name(value.meta_data.data_type)
            )
        };
        value.value = match value.meta_data.data_type {
            DataType::Boolean => {
                if let Some(property) = view.property_boolean(name) {
                    ViewModelInstanceValue::Bool(property.value())
                } else {
                    self.error(handle, request_id, Message::ViewModelError, missing());
                    return;
                }
            }
            DataType::Number => {
                if let Some(property) = view.property_number(name) {
                    ViewModelInstanceValue::Number(property.value())
                } else {
                    self.error(handle, request_id, Message::ViewModelError, missing());
                    return;
                }
            }
            DataType::Color => {
                if let Some(property) = view.property_color(name) {
                    ViewModelInstanceValue::Color(property.value())
                } else {
                    self.error(handle, request_id, Message::ViewModelError, missing());
                    return;
                }
            }
            DataType::String => {
                if let Some(property) = view.property_string(name) {
                    ViewModelInstanceValue::String(property.value().into())
                } else {
                    self.error(handle, request_id, Message::ViewModelError, missing());
                    return;
                }
            }
            DataType::Enum => {
                if let Some(property) = view.property_enum(name) {
                    ViewModelInstanceValue::String(property.value().into())
                } else {
                    self.error(handle, request_id, Message::ViewModelError, missing());
                    return;
                }
            }
            _ => unreachable!(),
        };
        let mut messages = self.command_queue.message_lock();
        messages.write(Message::ViewModelPropertyValueReceived);
        messages.write(handle);
        messages.write(value.meta_data.data_type);
        messages.write_name(value.meta_data.name);
        messages.write(request_id);
        match value.value {
            ViewModelInstanceValue::None => {}
            ViewModelInstanceValue::Bool(value) => messages.write(value),
            ViewModelInstanceValue::Number(value) => messages.write(value),
            ViewModelInstanceValue::Color(value) => messages.write(value),
            ViewModelInstanceValue::String(value) => messages.write_name(value),
        }
    }

    #[cfg(test)]
    pub fn testing_cursor_pos_for_pointer_event(
        &self,
        instance: &StateMachineInstance,
        event: PointerEvent,
    ) -> Vec2D {
        self.cursor_pos_for_pointer_event(instance, &event)
    }
    #[cfg(test)]
    pub fn testing_get_subscriptions(&self) -> &[Subscription] {
        &self.property_subscriptions
    }
    #[cfg(test)]
    pub fn testing_global_image_named(&self, name: &str) -> RenderImageHandle {
        self.assets
            .borrow()
            .image_assets
            .get(name)
            .copied()
            .unwrap_or(RenderImageHandle::NULL)
    }
    #[cfg(test)]
    pub fn testing_global_audio_named(&self, name: &str) -> AudioSourceHandle {
        self.assets
            .borrow()
            .audio_assets
            .get(name)
            .copied()
            .unwrap_or(AudioSourceHandle::NULL)
    }
    #[cfg(test)]
    pub fn testing_global_font_named(&self, name: &str) -> FontHandle {
        self.assets
            .borrow()
            .font_assets
            .get(name)
            .copied()
            .unwrap_or(FontHandle::NULL)
    }
    #[cfg(test)]
    pub fn testing_global_image_contains(&self, name: &str) -> bool {
        self.testing_global_image_named(name) != RenderImageHandle::NULL
    }
    #[cfg(test)]
    pub fn testing_global_audio_contains(&self, name: &str) -> bool {
        self.testing_global_audio_named(name) != AudioSourceHandle::NULL
    }
    #[cfg(test)]
    pub fn testing_global_font_contains(&self, name: &str) -> bool {
        self.testing_global_font_named(name) != FontHandle::NULL
    }
    #[cfg(all(test, debug_assertions))]
    pub fn testing_override_thread_id(&mut self, thread_id: ThreadId) {
        self.thread_id = thread_id;
    }
}

impl Drop for CommandServer {
    fn drop(&mut self) {}
}
