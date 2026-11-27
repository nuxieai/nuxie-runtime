//! What a host needs to draw an engine text field as the platform's own,
//! read from a step: its look and its text.

use super::player_driver::read_scene_object;
use super::*;
use nuxie::page_player::{TextFieldReadError, scene_text, text_field_look};

/// The most fields one read takes.
const NUX_PLAYER_TEXT_FIELDS_MAX: usize = 4096;

/// The most bytes one text copy gives.
const NUX_PLAYER_SCENE_TEXT_MAX: usize = 1024 * 1024;

/// One field to read: an engine text input and, when `has_placeholder`
/// is 1, the text that shows its placeholder.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NuxTextFieldObjects {
    pub struct_size: u32,
    pub field: NuxSceneObject,
    pub has_placeholder: u32,
    pub placeholder: NuxSceneObject,
}

/// A text field's look, copied out of a step. Colors are ARGB.
/// `font_asset` is the index, in the file's assets, of the font it draws
/// with, a font binding's choice included, or UINT32_MAX when it draws
/// with none. `font_from_host` is 1 when the font that draws was given by
/// the host and has no index in the file's assets; `font_asset` is then
/// UINT32_MAX. `has_weight` is 0 when its style sets no `wght`. `align`
/// is 0 left, 1 right, 2 center. `max_characters` is 0 for no limit.
/// `value_length` and `placeholder_length` are UTF-8 bytes, for
/// `nux_player_scene_text_copy`. Canonical 0/1 flags.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NuxTextFieldLook {
    pub struct_size: u32,
    pub render_revision: u64,
    pub font_asset: u32,
    pub font_from_host: u32,
    pub font_size: f32,
    pub has_weight: u32,
    pub weight: f32,
    pub letter_spacing: f32,
    pub color: u32,
    pub has_placeholder: u32,
    pub placeholder_color: u32,
    pub align: u32,
    pub obscured: u32,
    pub max_characters: u32,
    pub disabled: u32,
    pub value_length: usize,
    pub placeholder_length: usize,
}

/// Read `count` fields' looks from the state named by `step`, in order,
/// into `out`, as `nux_player_island_frames` reads boxes: the step must
/// be this player's and name its current render revision, or
/// HANDLE_MISMATCH. NOT_FOUND when an object names nothing,
/// INVALID_ARGUMENT when a field is not a text input or a placeholder
/// is not a text, INVALID_STRUCT_SIZE when a record's `struct_size` is
/// not its struct's size. `out` is written only when every field reads.
/// Changes nothing in the scene.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_text_fields(
    player: *const NuxPlayer,
    step: *const NuxPlayerStepResult,
    fields: *const NuxTextFieldObjects,
    count: usize,
    out: *mut NuxTextFieldLook,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let _player = enter_status_handle!(player, HandleKind::Player);
        let _step = enter_status_handle!(step, HandleKind::PlayerStepResult);
        let player = unsafe { &*player };
        let step = unsafe { &*step };
        let _occurrence = match enter_occurrence(&player.artboard) {
            Ok(guard) => guard,
            Err(status) => return status,
        };
        if let Err(status) = player
            .artboard
            .refresh_renderer_domain_invalidation()
            .and_then(|()| player.artboard.refresh_bound_view_model_invalidation())
        {
            player.artboard.poisoned.set(true);
            return status;
        }
        if step.status != NuxStatus::Ok
            || !step
                .occurrence
                .upgrade()
                .is_some_and(|owner| Rc::ptr_eq(&owner, &player.artboard))
            || step.scheduling.render_revision == 0
            || step.scheduling.render_revision != player.artboard.render_revision.get()
        {
            return NuxStatus::HandleMismatch;
        }
        if count > NUX_PLAYER_TEXT_FIELDS_MAX {
            return NuxStatus::LimitExceeded;
        }
        if count == 0 {
            return NuxStatus::Ok;
        }
        if fields.is_null() || out.is_null() {
            return NuxStatus::NullArgument;
        }
        // Each array steps by its struct's size, so every element must be
        // exactly that size: a shorter or longer one would move the next.
        let exact = |size: u32, of: usize| usize::try_from(size).ok() == Some(of);
        for index in 0..count {
            let field_size = unsafe { fields.add(index).cast::<u32>().read() };
            let out_size = unsafe { out.add(index).cast::<u32>().read() };
            if !exact(field_size, std::mem::size_of::<NuxTextFieldObjects>())
                || !exact(out_size, std::mem::size_of::<NuxTextFieldLook>())
            {
                return NuxStatus::InvalidStructSize;
            }
        }
        let mut objects = Vec::with_capacity(count);
        for index in 0..count {
            let record = unsafe { fields.add(index).read() };
            let field = match unsafe { read_scene_object(&raw const record.field) } {
                Ok(object) => object,
                Err(status) => return status,
            };
            let placeholder = match record.has_placeholder {
                0 => None,
                1 => match unsafe { read_scene_object(&raw const record.placeholder) } {
                    Ok(object) => Some(object),
                    Err(status) => return status,
                },
                _ => return NuxStatus::InvalidArgument,
            };
            objects.push((field, placeholder));
        }
        let artboard = player.artboard.instance.borrow();
        let mut looks = Vec::with_capacity(count);
        for (field, placeholder) in &objects {
            let look = match text_field_look(&artboard, field, placeholder.as_ref()) {
                Ok(look) => look,
                Err(TextFieldReadError::NotFound) => return NuxStatus::NotFound,
                Err(
                    TextFieldReadError::NotATextInput
                    | TextFieldReadError::NotAText
                    | TextFieldReadError::NotText,
                ) => return NuxStatus::InvalidArgument,
            };
            if ![
                look.font_size,
                look.weight.unwrap_or(0.0),
                look.letter_spacing,
            ]
            .iter()
            .all(|number| number.is_finite())
            {
                return NuxStatus::RuntimeError;
            }
            looks.push(NuxTextFieldLook {
                struct_size: std::mem::size_of::<NuxTextFieldLook>() as u32,
                render_revision: step.scheduling.render_revision,
                font_asset: look.font_asset.unwrap_or(u32::MAX),
                font_from_host: u32::from(look.font_from_host),
                font_size: look.font_size,
                has_weight: u32::from(look.weight.is_some()),
                weight: look.weight.unwrap_or(0.0),
                letter_spacing: look.letter_spacing,
                color: look.color,
                has_placeholder: u32::from(look.placeholder_color.is_some()),
                placeholder_color: look.placeholder_color.unwrap_or(0),
                align: look.align,
                obscured: u32::from(look.obscured),
                max_characters: look.max_characters,
                disabled: u32::from(look.disabled),
                value_length: look.value_len,
                placeholder_length: look.placeholder_len,
            });
        }
        for (index, look) in looks.iter().enumerate() {
            unsafe { out.add(index).write(*look) };
        }
        NuxStatus::Ok
    })
}

/// Copy the words `object` holds in the state named by `step`, a text
/// input's text or a text's, as UTF-8 with no terminator, into `buffer`,
/// as `nux_player_field_string_copy` copies: `*out_length` is the length
/// it needs; a null `buffer` with `capacity` 0 asks for it;
/// LIMIT_EXCEEDED copies nothing when `capacity` is short or the text is
/// over 1 MiB. A password's text is copied too: the host shows it in a
/// secure field and must never log it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_scene_text_copy(
    player: *const NuxPlayer,
    step: *const NuxPlayerStepResult,
    object: *const NuxSceneObject,
    buffer: *mut u8,
    capacity: usize,
    out_length: *mut usize,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_length.is_null() || (buffer.is_null() && capacity != 0) {
            return NuxStatus::NullArgument;
        }
        unsafe {
            *out_length = 0;
        }
        let _player = enter_status_handle!(player, HandleKind::Player);
        let _step = enter_status_handle!(step, HandleKind::PlayerStepResult);
        let player = unsafe { &*player };
        let step = unsafe { &*step };
        let _occurrence = match enter_occurrence(&player.artboard) {
            Ok(guard) => guard,
            Err(status) => return status,
        };
        if let Err(status) = player
            .artboard
            .refresh_renderer_domain_invalidation()
            .and_then(|()| player.artboard.refresh_bound_view_model_invalidation())
        {
            player.artboard.poisoned.set(true);
            return status;
        }
        if step.status != NuxStatus::Ok
            || !step
                .occurrence
                .upgrade()
                .is_some_and(|owner| Rc::ptr_eq(&owner, &player.artboard))
            || step.scheduling.render_revision == 0
            || step.scheduling.render_revision != player.artboard.render_revision.get()
        {
            return NuxStatus::HandleMismatch;
        }
        let object = match unsafe { read_scene_object(object) } {
            Ok(object) => object,
            Err(status) => return status,
        };
        let artboard = player.artboard.instance.borrow();
        let text = match scene_text(&artboard, &object) {
            Ok(text) => text,
            Err(TextFieldReadError::NotFound) => return NuxStatus::NotFound,
            Err(_) => return NuxStatus::InvalidArgument,
        };
        if text.len() > NUX_PLAYER_SCENE_TEXT_MAX {
            return NuxStatus::LimitExceeded;
        }
        unsafe {
            *out_length = text.len();
        }
        if buffer.is_null() && capacity == 0 {
            return NuxStatus::Ok;
        }
        if capacity < text.len() {
            return NuxStatus::LimitExceeded;
        }
        if !text.is_empty() {
            unsafe { ptr::copy_nonoverlapping(text.as_ptr(), buffer, text.len()) };
        }
        NuxStatus::Ok
    })
}

#[cfg(test)]
#[path = "text_fields_tests.rs"]
mod text_fields_tests;
