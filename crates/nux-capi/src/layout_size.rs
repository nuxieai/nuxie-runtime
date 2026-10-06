use super::{
    HandleKind, NuxPlayer, NuxStatus, enter_handle, enter_occurrence, ffi_guard,
    write_caller_struct,
};

/// The player's laid-out size in points, read after stepping.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxPlayerLayoutSize {
    /// Initialize to `sizeof(NuxPlayerLayoutSize)`.
    pub struct_size: u32,
    pub width: f32,
    pub height: f32,
}

pub const NUX_PLAYER_LAYOUT_SIZE_MIN_SIZE: usize =
    std::mem::offset_of!(NuxPlayerLayoutSize, height) + std::mem::size_of::<f32>();

impl Default for NuxPlayerLayoutSize {
    fn default() -> Self {
        Self {
            struct_size: u32::try_from(std::mem::size_of::<Self>()).unwrap_or(u32::MAX),
            width: 0.0,
            height: 0.0,
        }
    }
}

/// Set the player's root layout size in points. Both dimensions must be finite
/// and positive. The layout settles on the next `nux_player_step`; hosts must
/// step by zero after each size change and before the first draw.
/// A changed size invalidates the rendered frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_layout_size_set(
    player: *mut NuxPlayer,
    width: f32,
    height: f32,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if !width.is_finite() || width <= 0.0 || !height.is_finite() || height <= 0.0 {
            return NuxStatus::InvalidArgument;
        }
        let _player_call = match enter_handle(player, HandleKind::Player) {
            Ok(guard) => guard,
            Err(status) => return status,
        };
        let Some(player) = (unsafe { player.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let _occurrence_call = match enter_occurrence(&player.artboard) {
            Ok(guard) => guard,
            Err(status) => return status,
        };
        let Ok(mut artboard) = player.artboard.instance.try_borrow_mut() else {
            return NuxStatus::ReentrantCall;
        };
        let changed = artboard.set_artboard_dimensions(width, height);
        player
            .artboard
            .commit_runtime_change(changed)
            .map_or_else(|status| status, |()| NuxStatus::Ok)
    })
}

/// Read the laid-out root width and height in points into a caller-sized struct.
/// After changing the layout size, step by zero before reading the settled size.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_layout_size(
    player: *const NuxPlayer,
    out_size: *mut NuxPlayerLayoutSize,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let _player_call = match enter_handle(player, HandleKind::Player) {
            Ok(guard) => guard,
            Err(status) => return status,
        };
        let Some(player) = (unsafe { player.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let _occurrence_call = match enter_occurrence(&player.artboard) {
            Ok(guard) => guard,
            Err(status) => return status,
        };
        let Ok(artboard) = player.artboard.instance.try_borrow() else {
            return NuxStatus::ReentrantCall;
        };
        let (width, height) = artboard.artboard_dimensions();
        let value = NuxPlayerLayoutSize {
            width,
            height,
            ..NuxPlayerLayoutSize::default()
        };
        unsafe { write_caller_struct(out_size, &value, NUX_PLAYER_LAYOUT_SIZE_MIN_SIZE) }
            .map_or_else(|status| status, |()| NuxStatus::Ok)
    })
}
