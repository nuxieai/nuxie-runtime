//! Server-thread owner for the pinned C++ `CommandServer` command loop.
//!
//! Direct port of `include/rive/command_server.hpp` and
//! `src/command_server.cpp` at `4ac7b327`. The queue can be cloned across
//! threads; this value is deliberately server-thread-confined because the
//! runtime's retained `Rc` object graph is not a cross-thread object.

use std::{collections::BTreeMap, rc::Rc, sync::Arc};

use crate::{
    Factory, File, OwnedArtboardInstance, RawTextFont, RenderImage, StateMachineInstance,
    ViewModelInstance,
    command_queue::{
        ArtboardHandle, AudioSourceHandle, Command, CommandDataType, CommandEvent, CommandQueue,
        CommandValue, DrawCallback, DrawKey, FileAssetData, FileHandle, Fit, FontHandle,
        PointerEvent, PointerKind, RenderImageHandle, StateMachineHandle, ViewModelInstanceHandle,
        ViewModelPropertyData, ViewModelSource,
    },
};

struct ArtboardEntry {
    file: FileHandle,
    original_size: (f32, f32),
    bindable: nuxie_runtime::RuntimeBindableArtboard,
    instance: OwnedArtboardInstance,
}

struct StateMachineEntry {
    artboard: ArtboardHandle,
    instance: StateMachineInstance,
}

struct ViewModelEntry {
    file: FileHandle,
    instance: ViewModelInstance,
    view_model_name: String,
    instance_name: String,
}

enum RenderImageEntry {
    Unique(Box<dyn RenderImage>),
    Shared(Rc<dyn RenderImage>),
}

impl RenderImageEntry {
    fn as_ref(&self) -> &dyn RenderImage {
        match self {
            Self::Unique(image) => image.as_ref(),
            Self::Shared(image) => image.as_ref(),
        }
    }

    fn shared(&self) -> Option<Rc<dyn RenderImage>> {
        match self {
            Self::Unique(_) => None,
            Self::Shared(image) => Some(Rc::clone(image)),
        }
    }
}

/// Pinned command-server structure. Construct and run it on its owner thread.
pub struct CommandServer {
    queue: CommandQueue,
    factory: Box<dyn Factory>,
    disconnected: bool,
    files: BTreeMap<FileHandle, Arc<File>>,
    artboards: BTreeMap<ArtboardHandle, ArtboardEntry>,
    state_machines: BTreeMap<StateMachineHandle, StateMachineEntry>,
    view_models: BTreeMap<ViewModelInstanceHandle, ViewModelEntry>,
    images: BTreeMap<RenderImageHandle, RenderImageEntry>,
    audio_sources: BTreeMap<AudioSourceHandle, Arc<crate::AudioSource>>,
    fonts: BTreeMap<FontHandle, RawTextFont>,
    global_images: BTreeMap<String, RenderImageHandle>,
    global_audio: BTreeMap<String, AudioSourceHandle>,
    global_fonts: BTreeMap<String, FontHandle>,
    subscriptions: Vec<(
        ViewModelInstanceHandle,
        String,
        CommandDataType,
        u64,
        CommandValue,
        u64,
    )>,
}

impl CommandServer {
    pub fn new(queue: CommandQueue, factory: Box<dyn Factory>) -> Self {
        Self {
            queue,
            factory,
            disconnected: false,
            files: BTreeMap::new(),
            artboards: BTreeMap::new(),
            state_machines: BTreeMap::new(),
            view_models: BTreeMap::new(),
            images: BTreeMap::new(),
            audio_sources: BTreeMap::new(),
            fonts: BTreeMap::new(),
            global_images: BTreeMap::new(),
            global_audio: BTreeMap::new(),
            global_fonts: BTreeMap::new(),
            subscriptions: Vec::new(),
        }
    }

    pub fn factory(&mut self) -> &mut dyn Factory {
        self.factory.as_mut()
    }
    pub const fn was_disconnected(&self) -> bool {
        self.disconnected
    }
    pub fn file(&self, handle: FileHandle) -> Option<&Arc<File>> {
        self.files.get(&handle)
    }
    pub fn artboard(&self, handle: ArtboardHandle) -> Option<&OwnedArtboardInstance> {
        self.artboards.get(&handle).map(|entry| &entry.instance)
    }
    pub fn artboard_mut(&mut self, handle: ArtboardHandle) -> Option<&mut OwnedArtboardInstance> {
        self.artboards
            .get_mut(&handle)
            .map(|entry| &mut entry.instance)
    }
    #[doc(hidden)]
    pub fn bindable_artboard(
        &self,
        handle: ArtboardHandle,
    ) -> Option<&nuxie_runtime::RuntimeBindableArtboard> {
        self.artboards.get(&handle).map(|entry| &entry.bindable)
    }
    pub fn state_machine(&self, handle: StateMachineHandle) -> Option<&StateMachineInstance> {
        self.state_machines
            .get(&handle)
            .map(|entry| &entry.instance)
    }
    pub fn image(&self, handle: RenderImageHandle) -> Option<&dyn RenderImage> {
        self.images.get(&handle).map(RenderImageEntry::as_ref)
    }
    pub fn audio_source(&self, handle: AudioSourceHandle) -> Option<&Arc<crate::AudioSource>> {
        self.audio_sources.get(&handle)
    }
    pub fn font(&self, handle: FontHandle) -> Option<&RawTextFont> {
        self.fonts.get(&handle)
    }
    #[doc(hidden)]
    pub fn view_model(&self, handle: ViewModelInstanceHandle) -> Option<&ViewModelInstance> {
        self.view_models.get(&handle).map(|entry| &entry.instance)
    }
    #[doc(hidden)]
    pub fn handle_for_view_model(
        &self,
        instance: &ViewModelInstance,
    ) -> Option<ViewModelInstanceHandle> {
        self.view_models.iter().find_map(|(handle, entry)| {
            entry
                .instance
                .handle()
                .ptr_eq(instance.handle())
                .then_some(*handle)
        })
    }
    #[doc(hidden)]
    pub fn global_image_named(&self, name: &str) -> Option<RenderImageHandle> {
        self.global_images.get(name).copied()
    }
    #[doc(hidden)]
    pub fn global_audio_named(&self, name: &str) -> Option<AudioSourceHandle> {
        self.global_audio.get(name).copied()
    }
    #[doc(hidden)]
    pub fn global_font_named(&self, name: &str) -> Option<FontHandle> {
        self.global_fonts.get(name).copied()
    }
    #[doc(hidden)]
    pub fn testing_subscription_count(&self) -> usize {
        self.subscriptions.len()
    }
    #[doc(hidden)]
    pub fn testing_cursor_position(
        &self,
        handle: StateMachineHandle,
        event: PointerEvent,
    ) -> Option<crate::Vec2D> {
        let artboard = self.state_machines.get(&handle)?.artboard;
        Some(map_pointer(&self.artboards.get(&artboard)?.instance, event))
    }
    #[doc(hidden)]
    pub fn pointer_down_synchronized(&mut self, handle: StateMachineHandle, event: PointerEvent) {
        self.process_pointer(handle, PointerKind::Down, event, 0);
    }
    #[doc(hidden)]
    pub fn pointer_up_synchronized(&mut self, handle: StateMachineHandle, event: PointerEvent) {
        self.process_pointer(handle, PointerKind::Up, event, 0);
    }
    #[doc(hidden)]
    pub fn pointer_move_synchronized(&mut self, handle: StateMachineHandle, event: PointerEvent) {
        self.process_pointer(handle, PointerKind::Move, event, 0);
    }

    /// Wait for at least one command, then execute one entry-bounded poll.
    pub fn wait_commands(&mut self) -> bool {
        let commands = self.queue.shared.take_commands(true);
        self.process_batch(commands)
    }

    /// Execute one entry-bounded poll and its coalesced draws without waiting.
    pub fn process_commands(&mut self) -> bool {
        let commands = self.queue.shared.take_commands(false);
        self.process_batch(commands)
    }

    pub fn serve_until_disconnect(&mut self) {
        while self.wait_commands() {}
    }

    fn emit(&self, event: CommandEvent) {
        self.queue.shared.push_event(event);
    }
    fn file_error(&self, handle: FileHandle, request_id: u64, error: impl Into<String>) {
        self.emit(CommandEvent::FileError {
            handle,
            request_id,
            error: error.into(),
        });
    }
    fn artboard_error(&self, handle: ArtboardHandle, request_id: u64, error: impl Into<String>) {
        self.emit(CommandEvent::ArtboardError {
            handle,
            request_id,
            error: error.into(),
        });
    }
    fn state_machine_error(
        &self,
        handle: StateMachineHandle,
        request_id: u64,
        error: impl Into<String>,
    ) {
        self.emit(CommandEvent::StateMachineError {
            handle,
            request_id,
            error: error.into(),
        });
    }
    fn view_model_error(
        &self,
        handle: ViewModelInstanceHandle,
        request_id: u64,
        error: impl Into<String>,
    ) {
        self.emit(CommandEvent::ViewModelError {
            handle,
            request_id,
            error: error.into(),
        });
    }

    fn cleanup_artboard(&mut self, handle: ArtboardHandle, request_id: u64) {
        let state_machines = self
            .state_machines
            .iter()
            .filter_map(|(machine, entry)| (entry.artboard == handle).then_some(*machine))
            .collect::<Vec<_>>();
        for machine in state_machines {
            self.state_machines.remove(&machine);
            self.emit(CommandEvent::StateMachineDeleted {
                handle: machine,
                request_id,
            });
        }
        if self.artboards.remove(&handle).is_some() {
            self.emit(CommandEvent::ArtboardDeleted { handle, request_id });
        }
    }

    fn process_batch(&mut self, commands: Vec<Command>) -> bool {
        if self.disconnected {
            return false;
        }
        let mut draws = BTreeMap::<DrawKey, DrawCallback>::new();
        let mut commands = commands.into_iter();
        while let Some(command) = commands.next() {
            match command {
                Command::LoadFile {
                    handle,
                    bytes,
                    request_id,
                } => match File::import(&bytes) {
                    Ok(file) => {
                        self.files.insert(handle, Arc::new(file));
                        self.emit(CommandEvent::FileLoaded { handle, request_id });
                    }
                    Err(error) => self.file_error(
                        handle,
                        request_id,
                        format!("failed to load Rive file: {error}"),
                    ),
                },
                Command::DeleteFile { handle, request_id } => {
                    self.files.remove(&handle);
                    let artboards = self
                        .artboards
                        .iter()
                        .filter_map(|(artboard, entry)| (entry.file == handle).then_some(*artboard))
                        .collect::<Vec<_>>();
                    for artboard in artboards {
                        self.cleanup_artboard(artboard, request_id);
                    }
                    self.emit(CommandEvent::FileDeleted { handle, request_id });
                }
                Command::InstantiateArtboard {
                    handle,
                    file,
                    name,
                    request_id,
                } => {
                    let Some(file_value) = self.files.get(&file).cloned() else {
                        self.file_error(
                            file,
                            request_id,
                            format!(
                                "file {} not found when trying to create artboard",
                                file.get()
                            ),
                        );
                        continue;
                    };
                    let artboard = if name.is_empty() {
                        file_value.default_artboard()
                    } else {
                        file_value.artboard_named(&name)
                    };
                    let Some(artboard) = artboard else {
                        self.file_error(file, request_id, format!("artboard \"{name}\" not found"));
                        continue;
                    };
                    let original_size = artboard.dimensions().unwrap_or((0.0, 0.0));
                    let bindable = nuxie_runtime::RuntimeBindableArtboard::new(
                        artboard.name().unwrap_or_default(),
                    );
                    match OwnedArtboardInstance::instantiate(
                        Arc::clone(&file_value),
                        artboard.index(),
                    ) {
                        Ok(instance) => {
                            self.artboards.insert(
                                handle,
                                ArtboardEntry {
                                    file,
                                    original_size,
                                    bindable,
                                    instance,
                                },
                            );
                            self.emit(CommandEvent::ArtboardInstantiated {
                                file,
                                handle,
                                request_id,
                            });
                        }
                        Err(error) => self.file_error(file, request_id, error.to_string()),
                    }
                }
                Command::DeleteArtboard { handle, request_id } => {
                    self.cleanup_artboard(handle, request_id)
                }
                Command::SetArtboardSize {
                    handle,
                    width,
                    height,
                    request_id,
                } => match self.artboards.get_mut(&handle) {
                    Some(entry) => {
                        entry
                            .instance
                            .raw_mut()
                            .set_artboard_dimensions(width, height);
                    }
                    None => self.artboard_error(
                        handle,
                        request_id,
                        format!(
                            "artboard {} not found when trying to set artboard size",
                            handle.get()
                        ),
                    ),
                },
                Command::ResetArtboardSize { handle, request_id } => {
                    match self.artboards.get_mut(&handle) {
                        Some(entry) => {
                            let (w, h) = entry.original_size;
                            entry.instance.raw_mut().set_artboard_dimensions(w, h);
                        }
                        None => self.artboard_error(
                            handle,
                            request_id,
                            format!(
                                "artboard {} not found when trying to reset artboard size",
                                handle.get()
                            ),
                        ),
                    }
                }
                Command::SetArtboardVolume {
                    handle,
                    volume,
                    request_id,
                } => match self.artboards.get_mut(&handle) {
                    Some(entry) => entry.instance.set_volume(volume),
                    None => self.artboard_error(
                        handle,
                        request_id,
                        format!(
                            "artboard {} not found when trying to set artboard volume",
                            handle.get()
                        ),
                    ),
                },
                Command::GetArtboardVolume { handle, request_id } => {
                    match self.artboards.get(&handle) {
                        Some(entry) => self.emit(CommandEvent::ArtboardVolume {
                            handle,
                            request_id,
                            volume: entry.instance.volume(),
                        }),
                        None => self.artboard_error(
                            handle,
                            request_id,
                            format!(
                                "Invalid artboard handle {} when getting artboard volume",
                                handle.get()
                            ),
                        ),
                    }
                }
                Command::GetArtboardSize { handle, request_id } => {
                    match self.artboards.get(&handle) {
                        Some(entry) => {
                            let (width, height) = entry.instance.artboard_dimensions();
                            self.emit(CommandEvent::ArtboardSize {
                                handle,
                                request_id,
                                width,
                                height,
                            });
                        }
                        None => self.artboard_error(
                            handle,
                            request_id,
                            format!(
                                "Invalid artboard handle {} when getting artboard size",
                                handle.get()
                            ),
                        ),
                    }
                }
                Command::InstantiateStateMachine {
                    handle,
                    artboard,
                    name,
                    request_id,
                } => {
                    let Some(entry) = self.artboards.get_mut(&artboard) else {
                        self.artboard_error(
                            artboard,
                            request_id,
                            format!("owning artboard {} was not found", artboard.get()),
                        );
                        continue;
                    };
                    let machine = if name.is_empty() {
                        entry.instance.default_state_machine_instance()
                    } else {
                        entry.instance.state_machine_instance_named(&name)
                    };
                    match machine {
                        Some(instance) => {
                            self.state_machines
                                .insert(handle, StateMachineEntry { artboard, instance });
                            self.emit(CommandEvent::StateMachineInstantiated {
                                artboard,
                                handle,
                                request_id,
                            });
                        }
                        None => self.artboard_error(
                            artboard,
                            request_id,
                            format!("state machine \"{name}\" was not found"),
                        ),
                    }
                }
                Command::DeleteStateMachine { handle, request_id } => {
                    self.state_machines.remove(&handle);
                    self.emit(CommandEvent::StateMachineDeleted { handle, request_id });
                }
                Command::AdvanceStateMachine {
                    handle,
                    elapsed,
                    request_id,
                } => {
                    let Some(artboard_handle) =
                        self.state_machines.get(&handle).map(|entry| entry.artboard)
                    else {
                        self.state_machine_error(
                            handle,
                            request_id,
                            format!("State machine {} not found for advance", handle.get()),
                        );
                        continue;
                    };
                    let result = {
                        let Some(machine) = self.state_machines.get_mut(&handle) else {
                            continue;
                        };
                        let Some(artboard) = self.artboards.get_mut(&artboard_handle) else {
                            continue;
                        };
                        machine
                            .instance
                            .advance_and_apply(artboard.instance.raw_mut(), elapsed)
                    };
                    match result {
                        Ok(false) => {
                            self.emit(CommandEvent::StateMachineSettled { handle, request_id })
                        }
                        Ok(true) => {}
                        Err(error) => {
                            self.state_machine_error(handle, request_id, error.to_string())
                        }
                    }
                }
                Command::Pointer {
                    handle,
                    kind,
                    event,
                    request_id,
                } => self.process_pointer(handle, kind, event, request_id),
                Command::InstantiateViewModel {
                    handle,
                    file,
                    source,
                    instance_name,
                    request_id,
                } => self.instantiate_view_model(handle, file, source, instance_name, request_id),
                Command::ReferenceNestedViewModel {
                    root,
                    handle,
                    path,
                    request_id,
                } => {
                    let linked = self.view_models.get(&root).and_then(|entry| {
                        entry
                            .instance
                            .handle()
                            .linked_view_model_by_property_name_path(&path)
                            .map(|raw| (entry.file, raw))
                    });
                    match linked {
                        Some((file, raw)) => {
                            let view_model_name = self
                                .files
                                .get(&file)
                                .and_then(|file| file.view_model(raw.borrow().view_model_index()))
                                .and_then(|model| model.name())
                                .unwrap_or_default()
                                .to_owned();
                            self.view_models.insert(
                                handle,
                                ViewModelEntry {
                                    file,
                                    instance: ViewModelInstance { raw },
                                    view_model_name,
                                    instance_name: String::new(),
                                },
                            );
                        }
                        None => self.view_model_error(
                            handle,
                            request_id,
                            format!("Nested view not found at path {path}"),
                        ),
                    }
                }
                Command::ReferenceListViewModel {
                    root,
                    handle,
                    path,
                    index,
                    request_id,
                } => {
                    let linked = self
                        .view_models
                        .get(&root)
                        .and_then(|entry| {
                            entry
                                .instance
                                .handle()
                                .list_items_by_property_name_path(&path)
                        })
                        .and_then(|items| items.get(index).cloned().map(|raw| (root, raw)))
                        .and_then(|(root, raw)| {
                            self.view_models.get(&root).map(|entry| (entry.file, raw))
                        });
                    match linked {
                        Some((file, raw)) => {
                            let view_model_name = self
                                .files
                                .get(&file)
                                .and_then(|file| file.view_model(raw.borrow().view_model_index()))
                                .and_then(|model| model.name())
                                .unwrap_or_default()
                                .to_owned();
                            self.view_models.insert(
                                handle,
                                ViewModelEntry {
                                    file,
                                    instance: ViewModelInstance { raw },
                                    view_model_name,
                                    instance_name: String::new(),
                                },
                            );
                        }
                        None => self.view_model_error(
                            handle,
                            request_id,
                            "View model not found on list",
                        ),
                    }
                }
                Command::DeleteViewModel { handle, request_id } => {
                    self.view_models.remove(&handle);
                    self.emit(CommandEvent::ViewModelDeleted { handle, request_id });
                }
                Command::SetViewModelValue {
                    handle,
                    path,
                    value,
                    request_id,
                } => self.set_view_model_value(handle, path, value, request_id),
                Command::InsertViewModelList {
                    root,
                    path,
                    value,
                    index,
                    request_id,
                } => {
                    let Some(value) = self
                        .view_models
                        .get(&value)
                        .map(|entry| entry.instance.handle().clone())
                    else {
                        self.view_model_error(
                            root,
                            request_id,
                            "value view model not found for add list",
                        );
                        continue;
                    };
                    let Some(root_entry) = self.view_models.get(&root) else {
                        self.view_model_error(
                            root,
                            request_id,
                            "root view model not found for add list",
                        );
                        continue;
                    };
                    let next = index.unwrap_or_else(|| {
                        root_entry
                            .instance
                            .handle()
                            .list_item_count_by_property_name_path(&path)
                            .unwrap_or(0)
                    });
                    if !root_entry
                        .instance
                        .handle()
                        .insert_list_item_by_property_name_path(&path, next, &value)
                    {
                        self.view_model_error(
                            root,
                            request_id,
                            format!("failed to add to list at path {path}"),
                        );
                    }
                }
                Command::RemoveViewModelList {
                    root,
                    path,
                    value,
                    index,
                    request_id,
                } => {
                    let Some(root_entry) = self.view_models.get(&root) else {
                        self.view_model_error(
                            root,
                            request_id,
                            "root view model not found for remove list",
                        );
                        continue;
                    };
                    let remove_index = index.or_else(|| {
                        let target = value
                            .and_then(|h| self.view_models.get(&h))
                            .map(|entry| entry.instance.handle());
                        root_entry
                            .instance
                            .handle()
                            .list_items_by_property_name_path(&path)
                            .and_then(|items| {
                                items.iter().position(|item| {
                                    target.is_some_and(|target| item.ptr_eq(target))
                                })
                            })
                    });
                    if !remove_index.is_some_and(|index| {
                        root_entry
                            .instance
                            .handle()
                            .remove_list_item_by_property_name_path(&path, index)
                    }) {
                        self.view_model_error(
                            root,
                            request_id,
                            format!("failed to remove list at path {path}"),
                        );
                    }
                }
                Command::SwapViewModelList {
                    root,
                    path,
                    a,
                    b,
                    request_id,
                } => match self.view_models.get(&root) {
                    Some(entry)
                        if entry
                            .instance
                            .handle()
                            .swap_list_items_by_property_name_path(&path, a, b) => {}
                    _ => self.view_model_error(
                        root,
                        request_id,
                        format!("failed to swap list at path {path}"),
                    ),
                },
                Command::ClearViewModelList {
                    handle,
                    path,
                    request_id,
                } => match self.view_models.get(&handle) {
                    Some(entry)
                        if entry
                            .instance
                            .handle()
                            .clear_list_items_by_property_name_path(&path) =>
                    {
                        self.emit(CommandEvent::ViewModelListCleared {
                            handle,
                            request_id,
                            path,
                        })
                    }
                    _ => self.view_model_error(handle, request_id, "failed to clear list"),
                },
                Command::GetViewModelListSize {
                    handle,
                    path,
                    request_id,
                } => match self.view_models.get(&handle).and_then(|entry| {
                    entry
                        .instance
                        .handle()
                        .list_item_count_by_property_name_path(&path)
                }) {
                    Some(size) => self.emit(CommandEvent::ViewModelListSize {
                        handle,
                        request_id,
                        path,
                        size,
                    }),
                    None => self.view_model_error(handle, request_id, "failed to get list size"),
                },
                Command::GetViewModelValue {
                    handle,
                    path,
                    data_type,
                    request_id,
                } => match self.get_view_model_value(handle, &path, data_type) {
                    Some(value) => self.emit(CommandEvent::ViewModelValue {
                        handle,
                        request_id,
                        path,
                        value,
                    }),
                    None => self.view_model_error(
                        handle,
                        request_id,
                        format!("Could not find property at path {path}"),
                    ),
                },
                Command::SubscribeViewModelValue {
                    handle,
                    path,
                    data_type,
                    request_id,
                } => match self.subscription_value(handle, &path, data_type) {
                    Some((value, revision)) => self
                        .subscriptions
                        .push((handle, path, data_type, request_id, value, revision)),
                    None => self.view_model_error(
                        handle,
                        request_id,
                        "Property not found when subscribing",
                    ),
                },
                Command::UnsubscribeViewModelValue {
                    handle,
                    path,
                    data_type,
                } => self.subscriptions.retain(|subscription| {
                    subscription.0 != handle
                        || subscription.1 != path
                        || subscription.2 != data_type
                }),
                Command::GetViewModelName { handle, request_id } => {
                    match self.view_models.get(&handle) {
                        Some(entry) => self.emit(CommandEvent::ViewModelName {
                            handle,
                            request_id,
                            name: entry.view_model_name.clone(),
                        }),
                        None => self.view_model_error(
                            handle,
                            request_id,
                            "Invalid view model instance handle",
                        ),
                    }
                }
                Command::GetViewModelInstanceName { handle, request_id } => {
                    match self.view_models.get(&handle) {
                        Some(entry) => self.emit(CommandEvent::ViewModelInstanceName {
                            handle,
                            request_id,
                            name: entry.instance_name.clone(),
                        }),
                        None => self.view_model_error(
                            handle,
                            request_id,
                            "Invalid view model instance handle",
                        ),
                    }
                }
                Command::BindViewModel {
                    state_machine,
                    view_model,
                    request_id,
                } => {
                    let Some(model) = self
                        .view_models
                        .get(&view_model)
                        .map(|entry| entry.instance.handle().clone())
                    else {
                        self.state_machine_error(
                            state_machine,
                            request_id,
                            "View model instance not found for binding",
                        );
                        continue;
                    };
                    match self.state_machines.get_mut(&state_machine) {
                        Some(entry) => {
                            entry.instance.bind_owned_view_model_handle(&model);
                        }
                        None => self.state_machine_error(
                            state_machine,
                            request_id,
                            "State machine not found for binding view model",
                        ),
                    };
                }
                Command::SetGlobalViewModel {
                    state_machine,
                    name,
                    view_model,
                    request_id,
                } => {
                    let model = self
                        .view_models
                        .get(&view_model)
                        .map(|entry| entry.instance.handle().clone());
                    let context = self.state_machines.get(&state_machine).and_then(|entry| {
                        self.artboards
                            .get(&entry.artboard)
                            .map(|artboard| (entry.artboard, artboard.file))
                    });
                    let success = context
                        .and_then(|(_, file)| self.files.get(&file))
                        .zip(model)
                        .is_some_and(|(file, model)| {
                            self.state_machines
                                .get_mut(&state_machine)
                                .is_some_and(|entry| {
                                    entry.instance.set_global_view_model_instance(
                                        Some(file.runtime()),
                                        &name,
                                        Some(model),
                                    )
                                })
                        });
                    if !success {
                        self.state_machine_error(
                            state_machine,
                            request_id,
                            format!("Could not set global view model {name}"),
                        );
                    }
                }
                Command::BindStateMachine {
                    state_machine,
                    request_id,
                } => {
                    let context = self.state_machines.get(&state_machine).and_then(|entry| {
                        self.artboards
                            .get(&entry.artboard)
                            .map(|artboard| (entry.artboard, artboard.file))
                    });
                    let Some((artboard, file)) = context else {
                        self.state_machine_error(
                            state_machine,
                            request_id,
                            "State machine not found for bind",
                        );
                        continue;
                    };
                    let Some(file) = self.files.get(&file) else {
                        self.state_machine_error(
                            state_machine,
                            request_id,
                            "File not found for bind",
                        );
                        continue;
                    };
                    let (machines, artboards) = (&mut self.state_machines, &mut self.artboards);
                    let success = machines.get_mut(&state_machine).is_some_and(|machine| {
                        artboards
                            .get_mut(&artboard)
                            .map(|artboard| {
                                machine.instance.bind_for_command_queue(
                                    Some(file.runtime()),
                                    artboard.instance.raw_mut(),
                                )
                            })
                            .unwrap_or(false)
                    });
                    if !success {
                        self.state_machine_error(
                            state_machine,
                            request_id,
                            "Could not bind state machine",
                        );
                    }
                }
                Command::GetGlobalViewModel {
                    state_machine,
                    name,
                    handle,
                    request_id,
                } => {
                    let context = self.state_machines.get(&state_machine).and_then(|entry| {
                        self.artboards
                            .get(&entry.artboard)
                            .map(|artboard| (artboard.file, &entry.instance))
                    });
                    let resolved = context.and_then(|(file_handle, machine)| {
                        let file = self.files.get(&file_handle)?;
                        let raw =
                            machine.global_view_model_instance(Some(file.runtime()), &name)?;
                        Some((file_handle, raw))
                    });
                    match resolved {
                        Some((file, raw)) => {
                            let view_model_name = self
                                .files
                                .get(&file)
                                .and_then(|file| file.view_model(raw.borrow().view_model_index()))
                                .and_then(|model| model.name())
                                .unwrap_or_default()
                                .to_owned();
                            self.view_models.insert(
                                handle,
                                ViewModelEntry {
                                    file,
                                    instance: ViewModelInstance { raw },
                                    view_model_name,
                                    instance_name: String::new(),
                                },
                            );
                        }
                        None => self.view_model_error(
                            handle,
                            request_id,
                            format!("Could not get global view model {name}"),
                        ),
                    }
                }
                Command::DecodeImage {
                    handle,
                    bytes,
                    request_id,
                } => match self.factory.decode_image(&bytes) {
                    Ok(image) if image.width() != 0 && image.height() != 0 => {
                        self.images
                            .insert(handle, RenderImageEntry::Shared(Rc::from(image)));
                        self.emit(CommandEvent::ImageDecoded { handle, request_id });
                    }
                    Ok(_) => self.emit(CommandEvent::ImageError {
                        handle,
                        request_id,
                        error: "failed to decode image".to_owned(),
                    }),
                    Err(error) => self.emit(CommandEvent::ImageError {
                        handle,
                        request_id,
                        error: error.to_string(),
                    }),
                },
                Command::ExternalImage {
                    handle,
                    image,
                    request_id,
                } => {
                    let image: Box<dyn RenderImage> = image;
                    self.images.insert(handle, RenderImageEntry::Unique(image));
                    self.emit(CommandEvent::ImageDecoded { handle, request_id });
                }
                Command::DeleteImage { handle, request_id } => {
                    self.images.remove(&handle);
                    self.global_images.retain(|_, value| *value != handle);
                    self.emit(CommandEvent::ImageDeleted { handle, request_id });
                }
                Command::DecodeAudio {
                    handle,
                    bytes,
                    request_id,
                } => match self.factory.decode_audio(&bytes) {
                    Ok(audio) => {
                        self.audio_sources.insert(handle, audio);
                        self.emit(CommandEvent::AudioDecoded { handle, request_id });
                    }
                    Err(error) => self.emit(CommandEvent::AudioError {
                        handle,
                        request_id,
                        error: error.to_string(),
                    }),
                },
                Command::ExternalAudio {
                    handle,
                    audio,
                    request_id,
                } => {
                    self.audio_sources.insert(handle, audio);
                    self.emit(CommandEvent::AudioDecoded { handle, request_id });
                }
                Command::DeleteAudio { handle, request_id } => {
                    self.audio_sources.remove(&handle);
                    self.global_audio.retain(|_, value| *value != handle);
                    self.emit(CommandEvent::AudioDeleted { handle, request_id });
                }
                Command::DecodeFont {
                    handle,
                    bytes,
                    request_id,
                } => match RawTextFont::decode(bytes) {
                    Ok(font) => {
                        self.fonts.insert(handle, font);
                        self.emit(CommandEvent::FontDecoded { handle, request_id });
                    }
                    Err(error) => self.emit(CommandEvent::FontError {
                        handle,
                        request_id,
                        error: error.to_string(),
                    }),
                },
                Command::ExternalFont {
                    handle,
                    font,
                    request_id,
                } => {
                    self.fonts.insert(handle, font);
                    self.emit(CommandEvent::FontDecoded { handle, request_id });
                }
                Command::DeleteFont { handle, request_id } => {
                    self.fonts.remove(&handle);
                    self.global_fonts.retain(|_, value| *value != handle);
                    self.emit(CommandEvent::FontDeleted { handle, request_id });
                }
                Command::AddGlobalImage { name, handle } => {
                    if self.images.contains_key(&handle) {
                        self.global_images.insert(name, handle);
                    }
                }
                Command::RemoveGlobalImage { name } => {
                    self.global_images.remove(&name);
                }
                Command::AddGlobalAudio { name, handle } => {
                    if self.audio_sources.contains_key(&handle) {
                        self.global_audio.insert(name, handle);
                    }
                }
                Command::RemoveGlobalAudio { name } => {
                    self.global_audio.remove(&name);
                }
                Command::AddGlobalFont { name, handle } => {
                    if self.fonts.contains_key(&handle) {
                        self.global_fonts.insert(name, handle);
                    }
                }
                Command::RemoveGlobalFont { name } => {
                    self.global_fonts.remove(&name);
                }
                Command::ListArtboards { handle, request_id } => match self.files.get(&handle) {
                    Some(file) => self.emit(CommandEvent::ArtboardsListed {
                        handle,
                        request_id,
                        names: file
                            .artboards()
                            .map(|a| a.name().unwrap_or_default().to_owned())
                            .collect(),
                    }),
                    None => self.file_error(
                        handle,
                        request_id,
                        "Invalid file handle when getting list of artboards",
                    ),
                },
                Command::ListStateMachines { handle, request_id } => {
                    match self.artboards.get(&handle) {
                        Some(entry) => self.emit(CommandEvent::StateMachinesListed {
                            handle,
                            request_id,
                            names: (0..entry.instance.artboard().state_machine_count())
                                .map(|i| {
                                    entry
                                        .instance
                                        .artboard()
                                        .state_machine_name(i)
                                        .unwrap_or_default()
                                        .to_owned()
                                })
                                .collect(),
                        }),
                        None => self.artboard_error(
                            handle,
                            request_id,
                            "Invalid artboard handle when getting list of state machines",
                        ),
                    }
                }
                Command::ListViewModels { handle, request_id } => match self.files.get(&handle) {
                    Some(file) => self.emit(CommandEvent::ViewModelsListed {
                        handle,
                        request_id,
                        names: file
                            .view_models()
                            .map(|v| v.name().unwrap_or_default().to_owned())
                            .collect(),
                    }),
                    None => self.file_error(
                        handle,
                        request_id,
                        "Invalid file handle when getting list of view models",
                    ),
                },
                Command::ListGlobalViewModels { handle, request_id } => {
                    match self.files.get(&handle) {
                        Some(file) => self.emit(CommandEvent::GlobalViewModelsListed {
                            handle,
                            request_id,
                            names: file
                                .global_view_model_names()
                                .into_iter()
                                .map(str::to_owned)
                                .collect(),
                        }),
                        None => self.file_error(
                            handle,
                            request_id,
                            "Invalid file handle when getting list of global view models",
                        ),
                    }
                }
                Command::ListViewModelInstances {
                    handle,
                    view_model,
                    request_id,
                } => match self
                    .files
                    .get(&handle)
                    .and_then(|f| f.view_model_named(&view_model))
                {
                    Some(model) => self.emit(CommandEvent::ViewModelInstancesListed {
                        handle,
                        request_id,
                        view_model,
                        names: (0..model.instance_count())
                            .map(|i| model.instance_name(i).unwrap_or_default().to_owned())
                            .collect(),
                    }),
                    None => self.file_error(
                        handle,
                        request_id,
                        "Invalid view model when getting instance names",
                    ),
                },
                Command::ListViewModelProperties {
                    handle,
                    view_model,
                    request_id,
                } => match self.files.get(&handle).and_then(|file| {
                    file.view_model_named(&view_model)
                        .map(|model| (file, model))
                }) {
                    Some((file, model)) => self.emit(CommandEvent::ViewModelPropertiesListed {
                        handle,
                        request_id,
                        view_model,
                        properties: model
                            .properties()
                            .map(|p| {
                                let data_type = data_type_for_property(p.type_name());
                                let metadata = match data_type {
                                    CommandDataType::Enum => file
                                        .runtime
                                        .data_enum_for_view_model_property_object(p.descriptor())
                                        .and_then(|data_enum| {
                                            data_enum.object.string_property("name")
                                        }),
                                    CommandDataType::ViewModel => p
                                        .descriptor()
                                        .uint_property("viewModelReferenceId")
                                        .and_then(|index| usize::try_from(index).ok())
                                        .and_then(|index| file.view_model(index))
                                        .and_then(|view_model| view_model.name()),
                                    _ => None,
                                };
                                ViewModelPropertyData {
                                    data_type,
                                    name: p.name().unwrap_or_default().to_owned(),
                                    metadata: metadata.unwrap_or_default().to_owned(),
                                }
                            })
                            .collect(),
                    }),
                    None => self.file_error(
                        handle,
                        request_id,
                        "Invalid view model when getting properties",
                    ),
                },
                Command::ListViewModelEnums { handle, request_id } => match self.files.get(&handle)
                {
                    Some(file) => self.emit(CommandEvent::ViewModelEnumsListed {
                        handle,
                        request_id,
                        enums: file
                            .runtime
                            .data_enums()
                            .into_iter()
                            .map(|data_enum| crate::command_queue::ViewModelEnum {
                                name: data_enum
                                    .object
                                    .string_property("name")
                                    .unwrap_or_default()
                                    .to_owned(),
                                enumerants: data_enum
                                    .values
                                    .into_iter()
                                    .map(|value| {
                                        value
                                            .string_property("value")
                                            .filter(|value| !value.is_empty())
                                            .or_else(|| value.string_property("key"))
                                            .unwrap_or_default()
                                            .to_owned()
                                    })
                                    .collect(),
                            })
                            .collect(),
                    }),
                    None => self.file_error(
                        handle,
                        request_id,
                        "Invalid file handle when getting enums",
                    ),
                },
                Command::ListFileAssets { handle, request_id } => match self.files.get(&handle) {
                    Some(file) => self.emit(CommandEvent::FileAssetsListed {
                        handle,
                        request_id,
                        assets: file
                            .assets()
                            .map(|asset| {
                                let descriptor = asset.descriptor();
                                FileAssetData {
                                    name: asset.name().unwrap_or_default().to_owned(),
                                    asset_id: asset.asset_id().unwrap_or(0),
                                    cdn_uuid: descriptor
                                        .file_asset_cdn_uuid_string()
                                        .unwrap_or_default(),
                                    cdn_base_url: descriptor
                                        .string_property("cdnBaseUrl")
                                        .unwrap_or_default()
                                        .to_owned(),
                                    file_extension: asset.file_extension().to_owned(),
                                    type_id: descriptor.type_key,
                                }
                            })
                            .collect(),
                    }),
                    None => self.file_error(
                        handle,
                        request_id,
                        "Invalid file handle when getting assets",
                    ),
                },
                Command::GetDefaultViewModel {
                    artboard,
                    file,
                    request_id,
                } => {
                    let model = self
                        .artboards
                        .get(&artboard)
                        .and_then(|entry| entry.instance.view_model_index())
                        .and_then(|index| {
                            self.files
                                .get(&file)
                                .and_then(|file| file.view_model(index))
                        });
                    match model {
                        Some(model) => self.emit(CommandEvent::DefaultViewModel {
                            handle: artboard,
                            request_id,
                            view_model: model.name().unwrap_or_default().to_owned(),
                            instance: model.instance_name(0).unwrap_or_default().to_owned(),
                        }),
                        None => self.artboard_error(
                            artboard,
                            request_id,
                            "Could not find default view model",
                        ),
                    };
                }
                Command::RunOnce(callback) => callback(self),
                Command::Draw { key, callback } => {
                    draws.insert(key, callback);
                }
                Command::CancelDraw(key) => {
                    draws.remove(&key);
                }
                Command::TestingCommandLoopBreak => {
                    self.queue.shared.prepend_commands(commands);
                    break;
                }
                Command::Disconnect => {
                    self.disconnected = true;
                    return false;
                }
            }
        }
        for (key, draw) in draws {
            draw(key, self);
        }
        self.check_subscriptions();
        true
    }

    fn instantiate_view_model(
        &mut self,
        handle: ViewModelInstanceHandle,
        file: FileHandle,
        source: ViewModelSource,
        instance_name: Option<String>,
        request_id: u64,
    ) {
        let Some(file_value) = self.files.get(&file) else {
            self.file_error(
                file,
                request_id,
                "File not found when creating view model instance",
            );
            return;
        };
        let model = match source {
            ViewModelSource::Named(name) => file_value.view_model_named(&name),
            ViewModelSource::Artboard(artboard) => self
                .artboards
                .get(&artboard)
                .and_then(|entry| entry.instance.view_model_index())
                .and_then(|index| file_value.view_model(index)),
        };
        let Some(model) = model else {
            self.file_error(file, request_id, "View model not found");
            return;
        };
        let view_model_name = model.name().unwrap_or_default().to_owned();
        let resolved_instance_name = match instance_name.as_deref() {
            Some("") => model.instance_name(0).unwrap_or_default().to_owned(),
            Some(name) => name.to_owned(),
            None => String::new(),
        };
        let instance = match instance_name.as_deref() {
            None => model.instantiate(),
            Some("") => model.instantiate_default(),
            Some(name) => model.instantiate_instance_named(name),
        };
        match instance {
            Some(instance) => {
                self.view_models.insert(
                    handle,
                    ViewModelEntry {
                        file,
                        instance,
                        view_model_name,
                        instance_name: resolved_instance_name,
                    },
                );
                self.emit(CommandEvent::ViewModelInstantiated {
                    file,
                    handle,
                    request_id,
                });
            }
            None => self.file_error(file, request_id, "Could not create view model instance"),
        }
    }

    fn set_view_model_value(
        &mut self,
        handle: ViewModelInstanceHandle,
        path: String,
        value: CommandValue,
        request_id: u64,
    ) {
        let nested = match &value {
            CommandValue::ViewModel(value) => self
                .view_models
                .get(value)
                .map(|entry| entry.instance.handle().clone()),
            _ => None,
        };
        let image = match &value {
            CommandValue::Image(Some(value)) => self
                .images
                .get(value)
                .and_then(|image| image.shared())
                .map(Some),
            CommandValue::Image(None) => Some(None),
            _ => None,
        };
        let artboard = match &value {
            CommandValue::Artboard(Some(value)) => self
                .artboards
                .get(value)
                .map(|entry| Some(entry.bindable.clone())),
            CommandValue::Artboard(None) => Some(None),
            _ => None,
        };
        let enum_index = match &value {
            CommandValue::Enum(value) => self.view_models.get(&handle).and_then(|entry| {
                self.files.get(&entry.file).and_then(|file| {
                    enum_value_index_for_path(file, &entry.view_model_name, &path, value)
                })
            }),
            _ => None,
        };
        let Some(entry) = self.view_models.get_mut(&handle) else {
            self.view_model_error(
                handle,
                request_id,
                format!("Could not find view model instance at path {path}"),
            );
            return;
        };
        let changed = match value {
            CommandValue::Boolean(value) => {
                let exists = entry
                    .instance
                    .raw()
                    .boolean_value_by_property_name_path(&path)
                    .is_some();
                let _ = entry.instance.set_bool(&path, value);
                exists
            }
            CommandValue::Number(value) => {
                let exists = entry
                    .instance
                    .raw()
                    .number_value_by_property_name_path(&path)
                    .is_some();
                let _ = entry.instance.set_number(&path, value);
                exists
            }
            CommandValue::String(value) => {
                let exists = entry
                    .instance
                    .raw()
                    .string_value_by_property_name_path(&path)
                    .is_some();
                let _ = entry.instance.set_string(&path, &value);
                exists
            }
            CommandValue::Color(value) => {
                let exists = entry
                    .instance
                    .raw()
                    .color_value_by_property_name_path(&path)
                    .is_some();
                let _ = entry.instance.set_color(&path, value);
                exists
            }
            CommandValue::Enum(_) => enum_index
                .and_then(|value| u64::try_from(value).ok())
                .is_some_and(|value| {
                    let exists = entry
                        .instance
                        .raw()
                        .enum_value_by_property_name_path(&path)
                        .is_some();
                    let _ = entry.instance.set_enum(&path, value);
                    exists
                }),
            CommandValue::Trigger => {
                let exists = entry
                    .instance
                    .raw()
                    .trigger_value_by_property_name_path(&path)
                    .is_some();
                let _ = entry.instance.fire_trigger(&path);
                exists
            }
            CommandValue::Artboard(_) => artboard.is_some_and(|artboard| {
                let exists = entry
                    .instance
                    .raw()
                    .artboard_source_handle_by_property_name_path(&path)
                    .is_some();
                let _ = entry
                    .instance
                    .raw_mut()
                    .set_runtime_artboard_by_property_name(&path, artboard);
                exists
            }),
            CommandValue::ViewModel(_) => nested.is_some_and(|nested| {
                let exists = entry
                    .instance
                    .raw()
                    .view_model_source_handle_by_property_name_path(&path)
                    .is_some();
                let _ = entry
                    .instance
                    .handle()
                    .link_view_model_by_property_name_path(&path, &nested);
                exists
            }),
            CommandValue::Image(_) => image.is_some_and(|image| {
                let exists = entry
                    .instance
                    .raw()
                    .asset_source_handle_by_property_name_path(&path)
                    .is_some();
                let _ = entry
                    .instance
                    .raw_mut()
                    .set_runtime_image_by_property_name_path(
                        &path,
                        image.map(nuxie_runtime::RuntimeViewModelImage::from_render_image),
                    );
                exists
            }),
            CommandValue::None => false,
        };
        if !changed {
            self.view_model_error(
                handle,
                request_id,
                format!("Could not set property at path {path}"),
            );
        }
    }

    fn get_view_model_value(
        &self,
        handle: ViewModelInstanceHandle,
        path: &str,
        data_type: CommandDataType,
    ) -> Option<CommandValue> {
        let model = &self.view_models.get(&handle)?.instance;
        let raw = model.raw();
        Some(match data_type {
            CommandDataType::Boolean => {
                CommandValue::Boolean(raw.boolean_value_by_property_name_path(path)?)
            }
            CommandDataType::Number => {
                CommandValue::Number(raw.number_value_by_property_name_path(path)?)
            }
            CommandDataType::String => CommandValue::String(
                String::from_utf8_lossy(&raw.string_value_by_property_name_path(path)?)
                    .into_owned(),
            ),
            CommandDataType::Color => {
                CommandValue::Color(raw.color_value_by_property_name_path(path)?)
            }
            CommandDataType::Enum => {
                let index = usize::try_from(raw.enum_value_by_property_name_path(path)?).ok()?;
                let entry = self.view_models.get(&handle)?;
                let file = self.files.get(&entry.file)?;
                let value = enum_value_name_for_path(file, &entry.view_model_name, path, index)?;
                CommandValue::Enum(value)
            }
            CommandDataType::Trigger => {
                raw.trigger_value_by_property_name_path(path)?;
                CommandValue::Trigger
            }
            CommandDataType::ViewModel => {
                let linked = model
                    .handle()
                    .linked_view_model_by_property_name_path(path)?;
                let handle = self.view_models.iter().find_map(|(handle, entry)| {
                    entry.instance.handle().ptr_eq(&linked).then_some(*handle)
                })?;
                CommandValue::ViewModel(handle)
            }
            CommandDataType::List => {
                model.handle().list_item_count_by_property_name_path(path)?;
                CommandValue::None
            }
            CommandDataType::AssetImage => {
                let source_exists = raw
                    .asset_source_handle_by_property_name_path(path)
                    .is_some();
                if !source_exists {
                    return None;
                }
                let image = raw.runtime_image_by_property_name_path(path);
                let handle = image.as_ref().and_then(|image| {
                    let image = image.render_image()?;
                    self.images.iter().find_map(|(handle, entry)| {
                        entry
                            .shared()
                            .is_some_and(|candidate| Rc::ptr_eq(&candidate, &image))
                            .then_some(*handle)
                    })
                });
                CommandValue::Image(handle)
            }
            CommandDataType::Artboard => {
                let source_exists = raw
                    .artboard_source_handle_by_property_name_path(path)
                    .is_some();
                if !source_exists {
                    return None;
                }
                let artboard = raw.runtime_artboard_by_property_name(path);
                let handle = artboard.as_ref().and_then(|artboard| {
                    self.artboards.iter().find_map(|(handle, entry)| {
                        entry.bindable.ptr_eq(artboard).then_some(*handle)
                    })
                });
                CommandValue::Artboard(handle)
            }
            _ => return None,
        })
    }

    fn process_pointer(
        &mut self,
        handle: StateMachineHandle,
        kind: PointerKind,
        event: PointerEvent,
        request_id: u64,
    ) {
        let Some(artboard_handle) = self.state_machines.get(&handle).map(|entry| entry.artboard)
        else {
            self.state_machine_error(
                handle,
                request_id,
                "State machine not found for pointer event",
            );
            return;
        };
        let position = {
            let Some(artboard) = self.artboards.get(&artboard_handle) else {
                return;
            };
            map_pointer(&artboard.instance, event)
        };
        let hit = {
            let Some(machine) = self.state_machines.get_mut(&handle) else {
                return;
            };
            let Some(artboard) = self.artboards.get_mut(&artboard_handle) else {
                return;
            };
            match kind {
                PointerKind::Move => machine.instance.pointer_move(
                    artboard.instance.raw_mut(),
                    position.x,
                    position.y,
                    0.0,
                    event.pointer_id,
                ),
                PointerKind::Down => machine.instance.pointer_down(
                    artboard.instance.raw_mut(),
                    position.x,
                    position.y,
                    event.pointer_id,
                ),
                PointerKind::Up => machine.instance.pointer_up(
                    artboard.instance.raw_mut(),
                    position.x,
                    position.y,
                    event.pointer_id,
                ),
                PointerKind::Exit => machine.instance.pointer_exit(
                    artboard.instance.raw_mut(),
                    position.x,
                    position.y,
                    event.pointer_id,
                ),
            }
        };
        if hit && matches!(kind, PointerKind::Down | PointerKind::Up) {
            let Some(machine) = self.state_machines.get_mut(&handle) else {
                return;
            };
            let Some(artboard) = self.artboards.get_mut(&artboard_handle) else {
                return;
            };
            let _ = machine
                .instance
                .advance_and_apply(artboard.instance.raw_mut(), 0.0);
        }
    }

    fn check_subscriptions(&mut self) {
        let mut events = Vec::new();
        for index in 0..self.subscriptions.len() {
            let (handle, path, data_type, request_id) = {
                let (handle, path, data_type, request_id, _, _) = &self.subscriptions[index];
                (*handle, path.clone(), *data_type, *request_id)
            };
            if let Some((value, revision)) = self.subscription_value(handle, &path, data_type) {
                if value != self.subscriptions[index].4 || revision != self.subscriptions[index].5 {
                    self.subscriptions[index].4 = value.clone();
                    self.subscriptions[index].5 = revision;
                    events.push(CommandEvent::ViewModelValue {
                        handle,
                        request_id,
                        path,
                        value,
                    });
                }
            }
        }
        for event in events {
            self.emit(event);
        }
    }

    fn subscription_value(
        &self,
        handle: ViewModelInstanceHandle,
        path: &str,
        data_type: CommandDataType,
    ) -> Option<(CommandValue, u64)> {
        let value = self.get_view_model_value(handle, path, data_type)?;
        let revision = if data_type == CommandDataType::Trigger {
            self.view_models
                .get(&handle)?
                .instance
                .raw()
                .trigger_value_by_property_name_path(path)?
        } else {
            0
        };
        Some((value, revision))
    }
}

fn data_type_for_property(type_name: &str) -> CommandDataType {
    if type_name.ends_with("String") {
        CommandDataType::String
    } else if type_name.ends_with("Number") {
        CommandDataType::Number
    } else if type_name.ends_with("Boolean") {
        CommandDataType::Boolean
    } else if type_name.ends_with("Color") {
        CommandDataType::Color
    } else if type_name.ends_with("List") {
        CommandDataType::List
    } else if type_name.starts_with("ViewModelPropertyEnum") {
        CommandDataType::Enum
    } else if type_name.ends_with("Trigger") {
        CommandDataType::Trigger
    } else if type_name.ends_with("ViewModel") {
        CommandDataType::ViewModel
    } else if type_name.ends_with("AssetImage") {
        CommandDataType::AssetImage
    } else if type_name.ends_with("Artboard") {
        CommandDataType::Artboard
    } else {
        CommandDataType::None
    }
}

fn enum_value_index_for_path(
    file: &File,
    view_model_name: &str,
    path: &str,
    key: &str,
) -> Option<usize> {
    let mut model = file.view_model_named(view_model_name)?;
    let mut segments = path.split('/').peekable();
    loop {
        let property = model.property_named(segments.next()?)?;
        if segments.peek().is_none() {
            return file
                .runtime
                .view_model_property_enum_value_index_for_key_bytes_object(
                    property.descriptor(),
                    key.as_bytes(),
                );
        }
        let index = usize::try_from(
            property
                .descriptor()
                .uint_property("viewModelReferenceId")?,
        )
        .ok()?;
        model = file.view_model(index)?;
    }
}

fn enum_value_name_for_path(
    file: &File,
    view_model_name: &str,
    path: &str,
    index: usize,
) -> Option<String> {
    let mut model = file.view_model_named(view_model_name)?;
    let mut segments = path.split('/').peekable();
    loop {
        let property = model.property_named(segments.next()?)?;
        if segments.peek().is_none() {
            let value = file
                .runtime
                .view_model_property_enum_value_for_index_object(property.descriptor(), index)?;
            return Some(String::from_utf8_lossy(value).into_owned());
        }
        let index = usize::try_from(
            property
                .descriptor()
                .uint_property("viewModelReferenceId")?,
        )
        .ok()?;
        model = file.view_model(index)?;
    }
}

fn map_pointer(artboard: &OwnedArtboardInstance, event: PointerEvent) -> crate::Vec2D {
    let (_, _, aw, ah) = artboard.artboard_bounds();
    let sw = event.screen_bounds.x;
    let sh = event.screen_bounds.y;
    if sw == aw && sh == ah || sw == 0.0 || sh == 0.0 {
        return event.position;
    }
    let sx = sw / aw;
    let sy = sh / ah;
    let scale = match event.fit {
        Fit::Fill => return crate::Vec2D::new(event.position.x / sx, event.position.y / sy),
        Fit::Contain => sx.min(sy),
        Fit::Cover => sx.max(sy),
        Fit::FitWidth => sx,
        Fit::FitHeight => sy,
        Fit::None => 1.0,
        Fit::ScaleDown => 1.0_f32.min(sx.min(sy)),
    } * event.scale_factor;
    let width = aw * scale;
    let height = ah * scale;
    let ox = (sw - width) * (event.alignment.x + 1.0) * 0.5;
    let oy = (sh - height) * (event.alignment.y + 1.0) * 0.5;
    crate::Vec2D::new(
        (event.position.x - ox) / scale,
        (event.position.y - oy) / scale,
    )
}
