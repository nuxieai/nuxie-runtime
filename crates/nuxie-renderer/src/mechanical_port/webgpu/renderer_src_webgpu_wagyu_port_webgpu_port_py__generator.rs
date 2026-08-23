//! Complete source-owner translation of
//! `renderer/src/webgpu/wagyu-port/webgpu-port.py`.

#![allow(non_snake_case)]

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_webgpu_wagyu-port_webgpu-port.py");
pub(crate) const PINNED_SOURCE_LINE_COUNT: usize = 123;
pub(crate) const PINNED_SOURCE_BYTE_COUNT: usize = 2_918;
pub(crate) const OPTION_WAGYU_DESCRIPTION: &str = "Enable Wagyu extensions (default: false)";
pub(crate) const EXPORTED_VERSION_FUNCTION: &str = "_wgpuWagyuGetCompiledVersion";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PortOptions {
    pub(crate) wagyu: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum OptionError {
    UnknownOption(String),
    InvalidValue { option: String, value: String },
}

impl fmt::Display for OptionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownOption(option) => write!(formatter, "unknown option [{option}]"),
            Self::InvalidValue { option, value } => {
                write!(formatter, "[{option}] can be ['true', 'false'], got [{value}]")
            }
        }
    }
}

impl PortOptions {
    pub(crate) fn handleOptions<'a>(
        &mut self,
        options: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Result<(), OptionError> {
        for (option, value) in options {
            let value = value.to_lowercase();
            match option {
                "wagyu" if value == "true" => self.wagyu = true,
                "wagyu" if value == "false" => self.wagyu = false,
                "wagyu" => {
                    return Err(OptionError::InvalidValue {
                        option: option.to_owned(),
                        value,
                    });
                }
                _ => return Err(OptionError::UnknownOption(option.to_owned())),
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PortLayout {
    baseDir: PathBuf,
}

impl PortLayout {
    pub(crate) fn new(baseDir: impl Into<PathBuf>) -> Self {
        Self { baseDir: baseDir.into() }
    }

    pub(crate) fn includeDir(&self) -> PathBuf {
        self.baseDir.join("include")
    }

    pub(crate) fn srcDir(&self) -> PathBuf {
        self.baseDir.join("src")
    }

    pub(crate) fn sources(&self) -> Vec<PathBuf> {
        vec![self.srcDir().join("webgpu.c")]
    }

    pub(crate) fn processArgs(&self) -> Vec<String> {
        vec!["-isystem".to_owned(), self.includeDir().to_string_lossy().into_owned()]
    }

    pub(crate) fn buildFiles(&self) -> io::Result<Vec<PathBuf>> {
        let mut files = vec![self.baseDir.join("webgpu-port.py")];
        files.extend(self.sources());
        recurseDir(&self.includeDir(), &mut files)?;
        files.sort();
        Ok(files)
    }

    pub(crate) fn libraryName(&self) -> io::Result<String> {
        let mut hash = 0u32;
        for filename in self.buildFiles()? {
            hash = adler32(&fs::read(filename)?, hash);
        }
        Ok(format!("libwebgpu-{hash:08x}.a"))
    }
}

fn recurseDir(path: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let fileType = entry.file_type()?;
        if fileType.is_dir() {
            recurseDir(&entry.path(), files)?;
        } else if fileType.is_file() || (fileType.is_symlink() && entry.path().is_file()) {
            files.push(entry.path());
        }
    }
    Ok(())
}

/// Incremental Adler-32 with the source's explicit initial state of zero.
pub(crate) fn adler32(bytes: &[u8], prior: u32) -> u32 {
    const MOD_ADLER: u32 = 65_521;
    let mut a = prior & 0xffff;
    let mut b = prior >> 16;
    for byte in bytes {
        a = (a + u32::from(*byte)) % MOD_ADLER;
        b = (b + a) % MOD_ADLER;
    }
    (b << 16) | a
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct LinkerSettings {
    pub(crate) allowedSettings: bool,
    pub(crate) USE_WEBGPU: bool,
    pub(crate) JS_LIBRARIES: Vec<PathBuf>,
    pub(crate) EXPORTED_FUNCTIONS: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BuildPortRequest {
    pub(crate) sourceDir: PathBuf,
    pub(crate) portName: &'static str,
    pub(crate) includes: Vec<PathBuf>,
    pub(crate) flags: Vec<String>,
    pub(crate) sources: Vec<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CachedPortRequest {
    pub(crate) libraryName: String,
    pub(crate) what: &'static str,
    pub(crate) build: BuildPortRequest,
}

pub(crate) fn get(
    layout: &PortLayout,
    settings: &LinkerSettings,
) -> io::Result<Vec<CachedPortRequest>> {
    if settings.allowedSettings {
        return Ok(Vec::new());
    }
    Ok(vec![CachedPortRequest {
        libraryName: layout.libraryName()?,
        what: "port",
        build: BuildPortRequest {
            sourceDir: layout.srcDir(),
            portName: "webgpu",
            includes: vec![layout.includeDir()],
            flags: Vec::new(),
            sources: layout.sources(),
        },
    }])
}

pub(crate) fn clear(layout: &PortLayout) -> io::Result<String> {
    layout.libraryName()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DeprecatedUseWebGpu;

impl fmt::Display for DeprecatedUseWebGpu {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(
            "webgpu-port is not compatible with deprecated Emscripten USE_WEBGPU option",
        )
    }
}

pub(crate) fn linkerSetup(
    layout: &PortLayout,
    options: PortOptions,
    settings: &mut LinkerSettings,
) -> Result<(), DeprecatedUseWebGpu> {
    if settings.USE_WEBGPU {
        return Err(DeprecatedUseWebGpu);
    }
    settings
        .JS_LIBRARIES
        .push(layout.srcDir().join("library_webgpu_stubs.js"));
    settings
        .EXPORTED_FUNCTIONS
        .push(EXPORTED_VERSION_FUNCTION.to_owned());
    if options.wagyu {
        settings
            .JS_LIBRARIES
            .push(layout.srcDir().join("library_webgpu_wagyu_stubs.js"));
    }
    Ok(())
}

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_option_default_validation_and_case_folding_are_preserved() {
        let mut options = PortOptions::default();
        assert_eq!(options, PortOptions { wagyu: false });
        assert_eq!(options.handleOptions([("wagyu", "TrUe")]), Ok(()));
        assert_eq!(options, PortOptions { wagyu: true });
        assert!(matches!(
            options.handleOptions([("wagyu", "yes")]),
            Err(OptionError::InvalidValue { .. })
        ));
        assert_eq!(options, PortOptions { wagyu: true });
    }

    #[test]
    fn source_incremental_adler32_starts_at_zero() {
        let hash = adler32(b"bc", adler32(b"a", 0));
        assert_eq!(hash, 0x024a_0126);
        assert_eq!(hash, adler32(b"abc", 0));
    }

    #[test]
    fn source_paths_and_compiler_args_are_exact() {
        let layout = PortLayout::new("/p/webgpu-port");
        assert_eq!(layout.includeDir(), PathBuf::from("/p/webgpu-port/include"));
        assert_eq!(layout.srcDir(), PathBuf::from("/p/webgpu-port/src"));
        assert_eq!(
            layout.processArgs(),
            ["-isystem", "/p/webgpu-port/include"].map(str::to_owned)
        );
        assert_eq!(layout.sources(), [PathBuf::from("/p/webgpu-port/src/webgpu.c")]);
    }

    #[test]
    fn linker_setup_rejects_deprecated_emscripten_webgpu() {
        let layout = PortLayout::new("/p/webgpu-port");
        let mut settings = LinkerSettings { USE_WEBGPU: true, ..Default::default() };
        assert_eq!(
            linkerSetup(&layout, PortOptions::default(), &mut settings),
            Err(DeprecatedUseWebGpu)
        );
        assert!(settings.JS_LIBRARIES.is_empty());
        assert!(settings.EXPORTED_FUNCTIONS.is_empty());
    }

    #[test]
    fn linker_setup_adds_core_then_optional_wagyu_library_and_export() {
        let layout = PortLayout::new("/p/webgpu-port");
        let mut settings = LinkerSettings::default();
        linkerSetup(&layout, PortOptions { wagyu: true }, &mut settings).unwrap();
        assert_eq!(
            settings.JS_LIBRARIES,
            [
                PathBuf::from("/p/webgpu-port/src/library_webgpu_stubs.js"),
                PathBuf::from("/p/webgpu-port/src/library_webgpu_wagyu_stubs.js"),
            ]
        );
        assert_eq!(settings.EXPORTED_FUNCTIONS, [EXPORTED_VERSION_FUNCTION]);
    }

    #[test]
    fn allowed_settings_query_short_circuits_port_build() {
        let layout = PortLayout::new("/does/not/need/to/exist");
        let settings = LinkerSettings { allowedSettings: true, ..Default::default() };
        assert_eq!(get(&layout, &settings).unwrap(), []);
    }
}
