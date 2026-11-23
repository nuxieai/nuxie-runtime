//! A host driver on a player, at the C edge.
//!
//! A host registers a table of function pointers and a context pointer. In
//! every ordinary `nux_player_step` the player calls `drive` once after its
//! advance and again after each round that changed the scene, up to
//! `NUX_PLAYER_DRIVER_MAX_ROUNDS`. During that call the host may write
//! properties and reset subtrees through the `NuxSceneWriter` it receives;
//! the writer is valid only until `drive` returns.
use std::ffi::c_void;
use std::thread;

use nuxie::page_player::{
    DriverFault, DriverRound, HostDriver, MAX_DRIVER_ROUNDS, OwnedPlayerEvent,
    RuntimeGeometryHitPathSegment, SceneObject, SceneWriteError, SceneWriter,
};

use super::{
    HandleKind, NuxPlayer, NuxPlayerEventPropertyView, NuxStatus, NuxStringView, enter_handle,
    event_property_view, ffi_guard, optional_byte_view, page_status, register_handle,
    remove_handle, status_code, struct_size_supports, with_utf8_view,
};

/// The most rounds a driver is asked in one step. A driver still writing in
/// the last round is cut off; the step returns unsettled and the next step
/// carries on.
pub const NUX_PLAYER_DRIVER_MAX_ROUNDS: usize = 8;

const _: () = assert!(NUX_PLAYER_DRIVER_MAX_ROUNDS == MAX_DRIVER_ROUNDS);

/// One object of the player's artboard, or of one nested artboard
/// occurrence under it. Fields may be appended; initialize `struct_size` to
/// `sizeof(NuxSceneObject)`.
///
/// `path` leads from the player's artboard to the occurrence that holds the
/// object: the host of each nested artboard on the way, each by its local id
/// in the artboard the path has reached, with that artboard's file-global
/// id, as a report's `path` carries it. NULL with `path_len` 0 names the
/// player's own artboard; a caller whose `struct_size` ends before `path`
/// names it too. `local_id` is the object's local id in the artboard the
/// path leads to.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxSceneObject {
    pub struct_size: u32,
    pub local_id: usize,
    pub path: *const NuxScenePathSegment,
    pub path_len: usize,
}

pub const NUX_SCENE_OBJECT_MIN_SIZE: usize =
    std::mem::offset_of!(NuxSceneObject, local_id) + std::mem::size_of::<usize>();

/// The size of a `NuxSceneObject` that carries a path.
pub const NUX_SCENE_OBJECT_PATH_SIZE: usize =
    std::mem::offset_of!(NuxSceneObject, path_len) + std::mem::size_of::<usize>();

/// One nested artboard on the way from the player's artboard to where an
/// event was reported. This record's layout is fixed.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct NuxScenePathSegment {
    pub artboard_global_id: u32,
    pub local_id: usize,
}

/// One list row on the way from the player's artboard to where an event was
/// reported. This record's layout is fixed.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct NuxSceneListItem {
    pub artboard_global_id: u32,
    pub host_local_id: usize,
    pub item_index: usize,
    pub occurrence_identity: u64,
}

/// One reported event, as a driver sees it. Every view borrows the round and
/// is valid only during the `drive` call that received it.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxPlayerDriverReport {
    pub struct_size: u32,
    pub event_local_index: usize,
    pub event_core_type: u32,
    pub name: NuxStringView,
    pub url: NuxStringView,
    pub target: NuxStringView,
    pub seconds_delay: f32,
    pub has_source_artboard_global_id: bool,
    pub source_artboard_global_id: u32,
    /// Whether a listener's action reported the event. Events reported by a
    /// state, a transition or a timeline have no listener target.
    pub has_listener_target: bool,
    /// Local id of the listener's target object, in the artboard the path
    /// leads to (the player's artboard when the path is empty).
    pub listener_target_local_id: usize,
    pub path: *const NuxScenePathSegment,
    pub path_len: usize,
    pub list_items: *const NuxSceneListItem,
    pub list_item_count: usize,
    /// `property_count` views, `property_stride` bytes apart.
    pub properties: *const NuxPlayerEventPropertyView,
    pub property_count: usize,
    pub property_stride: usize,
}

/// One round of a step. `reports` holds `report_count` reports,
/// `report_stride` bytes apart: in round 0 the step's own reports and any the
/// previous step's last round left, later the reports of the previous
/// round's settle.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxPlayerDriverRound {
    pub struct_size: u32,
    pub index: usize,
    pub reports: *const NuxPlayerDriverReport,
    pub report_count: usize,
    pub report_stride: usize,
}

/// Writes into the player's artboard, valid for one `drive` call.
pub struct NuxSceneWriter {
    scene: *mut SceneWriter<'static>,
}

/// A host driver: `drive` is required; `release`, when set, is called once
/// with `context` when the driver is cleared or replaced, or its player is
/// freed. A status other than OK from `drive` fails the step and poisons the
/// player, since the step's advance and earlier writes cannot be undone.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxPlayerDriver {
    pub struct_size: u32,
    pub context: *mut c_void,
    pub drive: Option<
        unsafe extern "C" fn(
            context: *mut c_void,
            round: *const NuxPlayerDriverRound,
            writer: *mut NuxSceneWriter,
        ) -> NuxStatus,
    >,
    pub release: Option<unsafe extern "C" fn(context: *mut c_void)>,
}

pub const NUX_PLAYER_DRIVER_MIN_SIZE: usize =
    std::mem::offset_of!(NuxPlayerDriver, release) + std::mem::size_of::<usize>();

struct CDriver {
    context: *mut c_void,
    drive: unsafe extern "C" fn(
        *mut c_void,
        *const NuxPlayerDriverRound,
        *mut NuxSceneWriter,
    ) -> NuxStatus,
    release: Option<unsafe extern "C" fn(*mut c_void)>,
}

impl Drop for CDriver {
    fn drop(&mut self) {
        if let Some(release) = self.release {
            unsafe { release(self.context) };
        }
    }
}

struct ReportViews {
    reports: Vec<NuxPlayerDriverReport>,
    _paths: Vec<Vec<NuxScenePathSegment>>,
    _list_items: Vec<Vec<NuxSceneListItem>>,
    _properties: Vec<Vec<NuxPlayerEventPropertyView>>,
}

fn report_views(events: &[OwnedPlayerEvent]) -> ReportViews {
    let mut paths: Vec<Vec<NuxScenePathSegment>> = Vec::with_capacity(events.len());
    let mut list_items: Vec<Vec<NuxSceneListItem>> = Vec::with_capacity(events.len());
    let mut properties = Vec::with_capacity(events.len());
    for event in events {
        let context = event.context.as_ref();
        paths.push(
            context
                .map(|context| {
                    context
                        .path()
                        .iter()
                        .map(|segment| NuxScenePathSegment {
                            artboard_global_id: segment.artboard_global_id,
                            local_id: segment.local_id,
                        })
                        .collect()
                })
                .unwrap_or_default(),
        );
        list_items.push(
            context
                .map(|context| {
                    context
                        .occurrence()
                        .iter()
                        .map(|item| NuxSceneListItem {
                            artboard_global_id: item.artboard_global_id,
                            host_local_id: item.host_local_id,
                            item_index: item.item_index,
                            occurrence_identity: item.occurrence_identity,
                        })
                        .collect()
                })
                .unwrap_or_default(),
        );
        properties.push(
            event
                .properties
                .iter()
                .map(event_property_view)
                .collect::<Vec<_>>(),
        );
    }
    let reports = events
        .iter()
        .zip(&paths)
        .zip(&list_items)
        .zip(&properties)
        .map(|(((event, path), items), props)| NuxPlayerDriverReport {
            struct_size: u32::try_from(std::mem::size_of::<NuxPlayerDriverReport>())
                .unwrap_or(u32::MAX),
            event_local_index: event.event_local_index,
            event_core_type: event.event_core_type,
            name: optional_byte_view(event.name.as_deref()),
            url: optional_byte_view(event.url.as_deref()),
            target: optional_byte_view(event.target.as_deref()),
            seconds_delay: event.seconds_delay,
            has_source_artboard_global_id: event.source_artboard_global_id.is_some(),
            source_artboard_global_id: event.source_artboard_global_id.unwrap_or(0),
            has_listener_target: event.listener_target_local_id.is_some(),
            listener_target_local_id: event.listener_target_local_id.unwrap_or(0),
            path: path.as_ptr(),
            path_len: path.len(),
            list_items: items.as_ptr(),
            list_item_count: items.len(),
            properties: props.as_ptr(),
            property_count: props.len(),
            property_stride: std::mem::size_of::<NuxPlayerEventPropertyView>(),
        })
        .collect();
    ReportViews {
        reports,
        _paths: paths,
        _list_items: list_items,
        _properties: properties,
    }
}

impl HostDriver for CDriver {
    fn drive(
        &mut self,
        round: DriverRound<'_>,
        scene: &mut SceneWriter<'_>,
    ) -> Result<(), DriverFault> {
        let views = report_views(round.reports);
        let round_view = NuxPlayerDriverRound {
            struct_size: u32::try_from(std::mem::size_of::<NuxPlayerDriverRound>())
                .unwrap_or(u32::MAX),
            index: round.index,
            reports: views.reports.as_ptr(),
            report_count: views.reports.len(),
            report_stride: std::mem::size_of::<NuxPlayerDriverReport>(),
        };
        let writer = Box::into_raw(Box::new(NuxSceneWriter {
            scene: (scene as *mut SceneWriter<'_>).cast::<SceneWriter<'static>>(),
        }));
        register_handle(writer, HandleKind::SceneWriter, thread::current().id());
        let status = unsafe { (self.drive)(self.context, &round_view, writer) };
        // The writer ends with the turn: a call made with it later finds no
        // handle.
        let removed = remove_handle(writer, HandleKind::SceneWriter);
        unsafe { drop(Box::from_raw(writer)) };
        drop(views);
        if let Err(status) = removed {
            return Err(DriverFault(format!(
                "player driver writer could not be closed: {}",
                status_code(status)
            )));
        }
        if status == NuxStatus::Ok {
            Ok(())
        } else {
            Err(DriverFault(format!(
                "player driver returned {}",
                status_code(status)
            )))
        }
    }
}

/// Set the player's driver, replacing and releasing any earlier one. `drive`
/// must be set. The driver table is copied; `context` must stay valid until
/// `release` is called (or, without `release`, until the driver is cleared or
/// the player freed). Fails with REENTRANT_CALL from inside a step.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_driver_set(
    player: *mut NuxPlayer,
    driver: *const NuxPlayerDriver,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if driver.is_null() {
            return NuxStatus::NullArgument;
        }
        let caller_size = unsafe { driver.cast::<u32>().read() };
        if !struct_size_supports(caller_size, NUX_PLAYER_DRIVER_MIN_SIZE) {
            return NuxStatus::InvalidStructSize;
        }
        let table = unsafe { driver.read() };
        let Some(drive) = table.drive else {
            return NuxStatus::NullArgument;
        };
        let _player_call = match enter_handle(player, HandleKind::Player) {
            Ok(guard) => guard,
            Err(status) => return status,
        };
        let Some(player) = (unsafe { player.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let driver = CDriver {
            context: table.context,
            drive,
            release: table.release,
        };
        match player.page_player.set_driver(Some(Box::new(driver))) {
            Ok(previous) => {
                drop(previous);
                NuxStatus::Ok
            }
            Err(error) => page_status(error.kind),
        }
    })
}

/// Clear the player's driver, releasing it. Clearing a player with no driver
/// succeeds. Fails with REENTRANT_CALL from inside a step.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_driver_clear(player: *mut NuxPlayer) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let _player_call = match enter_handle(player, HandleKind::Player) {
            Ok(guard) => guard,
            Err(status) => return status,
        };
        let Some(player) = (unsafe { player.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        match player.page_player.set_driver(None) {
            Ok(previous) => {
                drop(previous);
                NuxStatus::Ok
            }
            Err(error) => page_status(error.kind),
        }
    })
}

fn write_status(error: SceneWriteError) -> NuxStatus {
    match error {
        SceneWriteError::NotFound | SceneWriteError::PathNotFound => NuxStatus::NotFound,
        SceneWriteError::UnknownProperty | SceneWriteError::InvalidValue => {
            NuxStatus::InvalidArgument
        }
    }
}

unsafe fn read_scene_object(object: *const NuxSceneObject) -> Result<SceneObject, NuxStatus> {
    if object.is_null() {
        return Err(NuxStatus::NullArgument);
    }
    let caller_size = unsafe { object.cast::<u32>().read() };
    if !struct_size_supports(caller_size, NUX_SCENE_OBJECT_MIN_SIZE) {
        return Err(NuxStatus::InvalidStructSize);
    }
    let caller_size = caller_size as usize;
    // A caller built before the path was appended passes the shorter
    // struct and names an object of the player's own artboard. Only the
    // prefix the caller wrote is read.
    if caller_size < NUX_SCENE_OBJECT_PATH_SIZE {
        if caller_size > NUX_SCENE_OBJECT_MIN_SIZE {
            return Err(NuxStatus::InvalidStructSize);
        }
        let local_id = unsafe {
            object
                .cast::<u8>()
                .add(std::mem::offset_of!(NuxSceneObject, local_id))
                .cast::<usize>()
                .read_unaligned()
        };
        return Ok(SceneObject::root(local_id));
    }
    let value = unsafe { object.read() };
    if value.path_len == 0 {
        return Ok(SceneObject::root(value.local_id));
    }
    if value.path.is_null() {
        return Err(NuxStatus::NullArgument);
    }
    let path = unsafe { std::slice::from_raw_parts(value.path, value.path_len) }
        .iter()
        .map(|segment| RuntimeGeometryHitPathSegment {
            artboard_global_id: segment.artboard_global_id,
            local_id: segment.local_id,
        })
        .collect();
    Ok(SceneObject::nested(path, value.local_id))
}

/// Run one writer call: check the writer is this turn's, read the object,
/// write `changed` when the caller asked for it.
fn with_writer(
    writer: *mut NuxSceneWriter,
    object: *const NuxSceneObject,
    out_changed: *mut bool,
    body: impl FnOnce(&mut SceneWriter<'static>, &SceneObject) -> Result<bool, SceneWriteError>,
) -> NuxStatus {
    {
        let _writer_call = match enter_handle(writer, HandleKind::SceneWriter) {
            Ok(guard) => guard,
            Err(status) => return status,
        };
        let object = match unsafe { read_scene_object(object) } {
            Ok(object) => object,
            Err(status) => return status,
        };
        // The handle is registered only while its turn's `drive` call runs,
        // and the turn holds the scene borrowed for exactly that long.
        let Some(scene) = (unsafe { (*writer).scene.as_mut() }) else {
            return NuxStatus::RuntimeError;
        };
        match body(scene, &object) {
            Ok(changed) => {
                if !out_changed.is_null() {
                    unsafe { *out_changed = changed };
                }
                NuxStatus::Ok
            }
            Err(error) => write_status(error),
        }
    }
}

/// Set a number property of `object` by property key. `out_changed`, when
/// not NULL, receives whether the value changed. NOT_FOUND for an unknown
/// object or a path that leads to no mounted nested artboard occurrence;
/// INVALID_ARGUMENT for a key the object has no number property of,
/// or a value that is not finite.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_scene_writer_set_number(
    writer: *mut NuxSceneWriter,
    object: *const NuxSceneObject,
    property_key: u16,
    value: f32,
    out_changed: *mut bool,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        with_writer(writer, object, out_changed, |scene, object| {
            scene.set_number(object, property_key, value)
        })
    })
}

/// Set an unsigned property (a count, an id, an enum such as a layout
/// record's display) of `object` by property key.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_scene_writer_set_unsigned(
    writer: *mut NuxSceneWriter,
    object: *const NuxSceneObject,
    property_key: u16,
    value: u32,
    out_changed: *mut bool,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        with_writer(writer, object, out_changed, |scene, object| {
            scene.set_unsigned(object, property_key, value)
        })
    })
}

/// Set a boolean property of `object` by property key.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_scene_writer_set_bool(
    writer: *mut NuxSceneWriter,
    object: *const NuxSceneObject,
    property_key: u16,
    value: bool,
    out_changed: *mut bool,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        with_writer(writer, object, out_changed, |scene, object| {
            scene.set_bool(object, property_key, value)
        })
    })
}

/// Set a color property of `object` by property key, as 0xAARRGGBB.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_scene_writer_set_color(
    writer: *mut NuxSceneWriter,
    object: *const NuxSceneObject,
    property_key: u16,
    value: u32,
    out_changed: *mut bool,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        with_writer(writer, object, out_changed, |scene, object| {
            scene.set_color(object, property_key, value)
        })
    })
}

/// Set a text property of `object` by property key, such as a text run's
/// text. `value` must be UTF-8; it is copied.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_scene_writer_set_text(
    writer: *mut NuxSceneWriter,
    object: *const NuxSceneObject,
    property_key: u16,
    value: NuxStringView,
    out_changed: *mut bool,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let text = match with_utf8_view(value, str::to_owned) {
            Ok(text) => text,
            Err(status) => return status,
        };
        with_writer(writer, object, out_changed, |scene, object| {
            scene.set_text(object, property_key, &text)
        })
    })
}

/// Reset what the engine holds under `object`, the object included. A reset
/// returns the engine's state holders under the object to their start and
/// leaves every property value alone, whoever wrote it. So a property that
/// an animation or a state wrote, and that the starting state does not key,
/// keeps its value until something writes it. The state holders are scroll
/// positions, typed text, nested artboards' state machines and animations,
/// list rows, the artboard a paint paints with, layout motion in flight, and
/// the presses and hovers the player's listeners hold there. `out_changed`,
/// when not NULL, receives whether anything was reset.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_scene_writer_reset(
    writer: *mut NuxSceneWriter,
    object: *const NuxSceneObject,
    out_changed: *mut bool,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        with_writer(writer, object, out_changed, |scene, object| {
            scene.reset(object)
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use nuxie::page_player::{OwnedPlayerEventProperty, OwnedPlayerEventPropertyValue};
    use nuxie_runtime::{
        RuntimeGeometryHitOccurrence, RuntimeGeometryHitPathSegment, StateMachineEventContext,
    };

    /// A report's C view carries its listener target, path, list rows and
    /// properties, each pointing at storage that holds what the event holds.
    #[test]
    fn a_report_view_carries_target_path_rows_and_properties() {
        let event = OwnedPlayerEvent {
            context: Some(StateMachineEventContext::new(
                vec![RuntimeGeometryHitPathSegment {
                    artboard_global_id: 7,
                    local_id: 11,
                }],
                vec![RuntimeGeometryHitOccurrence {
                    artboard_global_id: 7,
                    host_local_id: 5,
                    item_index: 2,
                    occurrence_identity: 41,
                }],
            )),
            source_artboard_global_id: Some(3),
            event_local_index: 4,
            event_core_type: 128,
            name: Some(b"tap".to_vec().into_boxed_slice()),
            url: None,
            target: None,
            seconds_delay: 0.0,
            properties: vec![OwnedPlayerEventProperty {
                name: Some(b"count".to_vec().into_boxed_slice()),
                value: OwnedPlayerEventPropertyValue::Number(2.5),
            }],
            view_model_instance_id: None,
            listener_target_local_id: Some(9),
        };
        let unheard = OwnedPlayerEvent {
            context: None,
            listener_target_local_id: None,
            properties: Vec::new(),
            ..event.clone()
        };
        let events = [event, unheard];
        let views = report_views(&events);
        let [heard, unheard] = views.reports.as_slice() else {
            panic!("two reports");
        };
        assert!(heard.has_listener_target);
        assert_eq!(heard.listener_target_local_id, 9);
        assert!(heard.has_source_artboard_global_id);
        assert_eq!(heard.source_artboard_global_id, 3);
        assert_eq!((heard.event_local_index, heard.event_core_type), (4, 128));
        let name =
            unsafe { std::slice::from_raw_parts(heard.name.data.cast::<u8>(), heard.name.len) };
        assert_eq!(name, b"tap");
        let path = unsafe { std::slice::from_raw_parts(heard.path, heard.path_len) };
        assert_eq!(
            path.iter()
                .map(|segment| (segment.artboard_global_id, segment.local_id))
                .collect::<Vec<_>>(),
            [(7, 11)]
        );
        let rows = unsafe { std::slice::from_raw_parts(heard.list_items, heard.list_item_count) };
        assert_eq!(
            rows.iter()
                .map(|row| (row.host_local_id, row.item_index, row.occurrence_identity))
                .collect::<Vec<_>>(),
            [(5, 2, 41)]
        );
        assert_eq!(heard.property_count, 1);
        assert_eq!(
            heard.property_stride,
            std::mem::size_of::<NuxPlayerEventPropertyView>()
        );
        let property = unsafe { &*heard.properties };
        assert_eq!(property.number_value, 2.5);

        assert!(!unheard.has_listener_target);
        assert_eq!(
            (
                unheard.path_len,
                unheard.list_item_count,
                unheard.property_count
            ),
            (0, 0, 0)
        );
    }

    /// A `NuxSceneObject` as a caller built before the path was appended
    /// lays it out.
    #[repr(C)]
    struct OldSceneObject {
        struct_size: u32,
        local_id: usize,
    }

    /// What the driver below was told, call by call.
    #[derive(Default)]
    struct Answers {
        answers: Vec<(&'static str, NuxStatus, bool)>,
    }

    const WIDTH: u16 = 61022;
    const TEXT: u16 = 268;

    unsafe fn answer(
        answers: &mut Answers,
        what: &'static str,
        call: impl FnOnce(*mut bool) -> NuxStatus,
    ) {
        let mut changed = false;
        let status = call(&mut changed);
        answers.answers.push((what, status, changed));
    }

    /// On the "chosen" report, writes through the report's path and through
    /// an old-size object, and tries every way a path can be refused.
    unsafe extern "C" fn drive_chosen(
        context: *mut c_void,
        round: *const NuxPlayerDriverRound,
        writer: *mut NuxSceneWriter,
    ) -> NuxStatus {
        use nuxie::occurrence_fixture::{card, page};
        let answers = unsafe { &mut *context.cast::<Answers>() };
        let round = unsafe { &*round };
        let reports = unsafe { std::slice::from_raw_parts(round.reports, round.report_count) };
        for report in reports {
            let name = unsafe {
                std::slice::from_raw_parts(report.name.data.cast::<u8>(), report.name.len)
            };
            if name != b"chosen" {
                continue;
            }
            let object = |local_id, path, path_len| NuxSceneObject {
                struct_size: u32::try_from(std::mem::size_of::<NuxSceneObject>()).unwrap(),
                local_id,
                path,
                path_len,
            };
            unsafe {
                let old = OldSceneObject {
                    struct_size: u32::try_from(std::mem::size_of::<OldSceneObject>()).unwrap(),
                    local_id: page::BELOW_RECORD,
                };
                answer(answers, "old size names the page", |changed| {
                    nux_scene_writer_set_number(
                        writer,
                        (&raw const old).cast(),
                        WIDTH,
                        120.0,
                        changed,
                    )
                });
                let run = object(card::RUN, report.path, report.path_len);
                let text = "Chosen";
                answer(answers, "the report's path", |changed| {
                    nux_scene_writer_set_text(
                        writer,
                        &run,
                        TEXT,
                        NuxStringView {
                            data: text.as_ptr().cast(),
                            len: text.len(),
                        },
                        changed,
                    )
                });
                let null = object(card::RUN, std::ptr::null(), 1);
                answer(answers, "a null path with a length", |changed| {
                    nux_scene_writer_set_number(writer, &null, WIDTH, 1.0, changed)
                });
                let mut short = run;
                short.struct_size =
                    u32::try_from(std::mem::offset_of!(NuxSceneObject, path_len)).unwrap();
                answer(answers, "a size that ends inside the path", |changed| {
                    nux_scene_writer_set_number(writer, &short, WIDTH, 1.0, changed)
                });
                let nowhere = [NuxScenePathSegment {
                    artboard_global_id: (*report.path).artboard_global_id,
                    local_id: 999,
                }];
                let lost = object(card::RUN, nowhere.as_ptr(), 1);
                answer(answers, "a path that leads nowhere", |changed| {
                    nux_scene_writer_set_number(writer, &lost, WIDTH, 1.0, changed)
                });
                let reset = object(card::FIELD_BOX, nowhere.as_ptr(), 1);
                answer(answers, "a reset on a path that leads nowhere", |changed| {
                    nux_scene_writer_reset(writer, &reset, changed)
                });
            }
        }
        NuxStatus::Ok
    }

    unsafe fn step(player: *mut NuxPlayer, pointers: &[crate::NuxPlayerPointerEvent]) {
        let step = crate::NuxPlayerStep {
            pointers: pointers.as_ptr(),
            pointer_count: pointers.len(),
            elapsed_seconds: 1.0 / 60.0,
            ..crate::NuxPlayerStep::default()
        };
        let mut result = std::ptr::null_mut();
        unsafe {
            assert_eq!(
                crate::nux_player_step(player, &step, &mut result),
                NuxStatus::Ok
            );
            crate::nux_player_step_result_free(result);
        }
    }

    /// Through the C edge, an old-size `NuxSceneObject` names an object of
    /// the player's own artboard, and one with a report's path names that
    /// object in the use the report came from, not in the other use. A null
    /// path with a length, a size that ends inside the path and a path that
    /// leads nowhere are refused and change nothing.
    #[test]
    fn a_scene_object_path_writes_one_use_and_an_old_size_writes_the_page() {
        use nuxie::occurrence_fixture::{PageMachine, RUN_TEXT, card, page, two_uses_fixture};
        use nuxie::runtime::{
            generated::core_registry::CoreRegistry, layout_component::LayoutComponent,
            nested_artboard_layout::NestedArtboardLayout,
        };
        let bytes = two_uses_fixture(PageMachine::OneLayer);
        let mut file = std::ptr::null_mut();
        let mut instance = std::ptr::null_mut();
        let mut player = std::ptr::null_mut();
        unsafe {
            assert_eq!(
                crate::nux_file_import(
                    bytes.as_ptr(),
                    bytes.len(),
                    &crate::NuxRenderCallbacks::default(),
                    &mut file
                ),
                NuxStatus::Ok
            );
            assert_eq!(
                crate::nux_artboard_instance_new(file, 0, &mut instance),
                NuxStatus::Ok
            );
            assert_eq!(
                crate::nux_player_new_default(instance, &mut player),
                NuxStatus::Ok
            );
            for _ in 0..60 {
                step(player, &[]);
            }
        }
        let page_board = unsafe { &*player }
            .artboard
            .instance
            .borrow()
            .native_handle();
        let object = |board: &nuxie::RuntimeArtboardInstanceHandle, local_id: usize| {
            board
                .with_artboard(|board| board.objects()[local_id].clone())
                .expect("an object")
        };
        let card_of = |host| {
            object(&page_board, host)
                .with_downcast::<NestedArtboardLayout, _>(|host| {
                    (
                        host.layout_bounds(),
                        host.base.base.artboard_instance_handle(0),
                    )
                })
                .and_then(|(bounds, card)| Some((bounds, card?)))
                .expect("a hosted card")
        };
        let run_text = |host| {
            CoreRegistry::get_string_handle(&object(&card_of(host).1, card::RUN), i32::from(TEXT))
                .expect("the run's text")
        };
        let (host, first) = card_of(page::FIRST);
        let button = object(&first, card::BUTTON)
            .with_downcast::<LayoutComponent, _>(|button| button.layout_bounds())
            .expect("the button");
        let (x, y) = (
            host.left() + button.left() + button.width() / 2.0,
            host.top() + button.top() + button.height() / 2.0,
        );

        let mut answers = Answers::default();
        let driver = NuxPlayerDriver {
            struct_size: u32::try_from(std::mem::size_of::<NuxPlayerDriver>()).unwrap(),
            context: (&raw mut answers).cast(),
            drive: Some(drive_chosen),
            release: None,
        };
        let pointer = |kind| crate::NuxPlayerPointerEvent {
            kind,
            x,
            y,
            pointer_id: 0,
            timestamp_seconds: 0.0,
        };
        unsafe {
            assert_eq!(nux_player_driver_set(player, &driver), NuxStatus::Ok);
            step(
                player,
                &[
                    pointer(crate::NUX_PLAYER_POINTER_KIND_DOWN),
                    pointer(crate::NUX_PLAYER_POINTER_KIND_UP),
                ],
            );
            assert_eq!(nux_player_driver_clear(player), NuxStatus::Ok);
        }
        assert_eq!(
            answers.answers,
            [
                ("old size names the page", NuxStatus::Ok, true),
                ("the report's path", NuxStatus::Ok, true),
                ("a null path with a length", NuxStatus::NullArgument, false),
                (
                    "a size that ends inside the path",
                    NuxStatus::InvalidStructSize,
                    false
                ),
                ("a path that leads nowhere", NuxStatus::NotFound, false),
                (
                    "a reset on a path that leads nowhere",
                    NuxStatus::NotFound,
                    false
                ),
            ]
        );
        assert_eq!(run_text(page::FIRST), "Chosen");
        assert_eq!(run_text(page::SECOND), RUN_TEXT);
        assert_eq!(
            CoreRegistry::get_double_handle(
                &object(&page_board, page::BELOW_RECORD),
                i32::from(WIDTH)
            ),
            Some(120.0)
        );
        unsafe {
            assert_eq!(crate::nux_player_free(player), NuxStatus::Ok);
            assert_eq!(crate::nux_artboard_instance_free(instance), NuxStatus::Ok);
            assert_eq!(crate::nux_file_free(file), NuxStatus::Ok);
        }
    }
}
