/*
 * Copyright 2026 Rive
 */

// Mechanical translation of renderer/include/rive/renderer/shader_compilation_mode.hpp.
#![allow(non_camel_case_types, non_upper_case_globals)]

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ShaderCompilationMode {
    #[default]
    allowAsynchronous = 0,
    alwaysSynchronous = 1,
    onlyUbershaders = 2,
}

impl ShaderCompilationMode {
    // Rust cannot repeat enum discriminants; preserve the upstream alias.
    pub const standard: Self = Self::allowAsynchronous;
}
