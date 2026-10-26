//! JNI adapter for the validation APK only; no production SDK surface.
use super::Probe;
use jni::{
    JNIEnv,
    objects::{JByteBuffer, JClass},
    sys::{jint, jlong, jstring},
};
use std::{ffi::CStr, ptr};

#[unsafe(no_mangle)]
pub extern "system" fn Java_ai_nuxie_audiovalidation_MainActivity_nativeNew(
    _env: JNIEnv,
    _class: JClass,
    rate: jint,
    channels: jint,
    block: jint,
) -> jlong {
    if rate <= 0 || channels != 2 || block <= 0 {
        return 0;
    }
    unsafe { super::nux_audio_probe_new(rate as u32, channels as u32, block as u32) as jlong }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_ai_nuxie_audiovalidation_MainActivity_nativePull(
    mut env: JNIEnv,
    _class: JClass,
    probe: jlong,
    output: JByteBuffer,
    frames: jint,
) -> jlong {
    let result = (|| {
        let bytes = usize::try_from(frames)
            .ok()
            .and_then(|n| n.checked_mul(8))
            .ok_or("invalid frame count")?;
        if probe == 0
            || env
                .get_direct_buffer_capacity(&output)
                .map_err(|_| "buffer capacity")?
                < bytes
        {
            return Err("invalid probe or output capacity");
        }
        let address = env
            .get_direct_buffer_address(&output)
            .map_err(|_| "direct buffer required")?;
        if !(address as usize).is_multiple_of(std::mem::align_of::<f32>()) {
            return Err("unaligned float output");
        }
        Ok(unsafe {
            super::nux_audio_probe_pull(probe as *mut Probe, address.cast::<f32>(), frames as u32)
                as jlong
        })
    })();
    match result {
        Ok(n) => n,
        Err(message) => {
            let _ = env.throw_new("java/lang/IllegalArgumentException", message);
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_ai_nuxie_audiovalidation_MainActivity_nativeSetPhase(
    _env: JNIEnv,
    _class: JClass,
    probe: jlong,
    phase: jint,
) {
    if probe != 0 && phase >= 0 {
        unsafe {
            super::nux_audio_probe_set_phase(probe as *mut Probe, phase as u32);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_ai_nuxie_audiovalidation_MainActivity_nativeReport(
    mut env: JNIEnv,
    _class: JClass,
    probe: jlong,
) -> jstring {
    if probe == 0 {
        return ptr::null_mut();
    }
    let text = unsafe { super::nux_audio_probe_report_json(probe as *mut Probe) };
    if text.is_null() {
        return ptr::null_mut();
    }
    let value = unsafe { CStr::from_ptr(text) }
        .to_string_lossy()
        .into_owned();
    unsafe {
        super::nux_audio_probe_free_json(text);
    }
    match env.new_string(value) {
        Ok(value) => value.into_raw(),
        Err(_) => {
            let _ = env.throw_new(
                "java/lang/RuntimeException",
                "cannot allocate report string",
            );
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_ai_nuxie_audiovalidation_MainActivity_nativeDestroy(
    _env: JNIEnv,
    _class: JClass,
    probe: jlong,
) {
    if probe != 0 {
        unsafe {
            super::nux_audio_probe_destroy(probe as *mut Probe);
        }
    }
}
