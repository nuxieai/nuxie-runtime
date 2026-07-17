use pixel_compare::{validate_reference_identities, ReferenceIdentity, RgbaImage};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Deserialize)]
struct Manifest {
    entry: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
struct Entry {
    id: String,
    stream: PathBuf,
    reference: PathBuf,
    status: String,
    #[serde(default)]
    frame: usize,
    mode: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = Options::parse()?;
    let manifest: Manifest = toml::from_str(&fs::read_to_string(&options.manifest)?)?;
    validate_reference_identities(
        &std::env::current_dir()?,
        manifest.entry.iter().map(|entry| ReferenceIdentity {
            id: &entry.id,
            stream: &entry.stream,
            frame: entry.frame,
            mode: &entry.mode,
            reference: &entry.reference,
        }),
    )?;
    require_supported_mode(options.backend, &options.mode)?;
    let provenance = match options.backend {
        ReplayBackend::FfiMetal => None,
        ReplayBackend::FfiDawn => Some(ProvenanceContext::load(
            &options.runtime_dir,
            &options.replay,
        )?),
    };
    let mut captured = 0usize;

    for entry in manifest.entry.iter().filter(|entry| {
        entry.status == options.status
            && entry.mode == options.mode
            && options
                .id
                .as_ref()
                .is_none_or(|selected| selected == &entry.id)
    }) {
        if let Some(parent) = entry.reference.parent() {
            fs::create_dir_all(parent)?;
        }
        let temporary = temporary_path(&entry.reference, captured)?;
        let _ = fs::remove_file(&temporary);
        let replay = Command::new(&options.replay)
            .args(["--stream", path_str(&entry.stream)?])
            .args(["--output", path_str(&temporary)?])
            .args(["--backend", options.backend.as_str()])
            .args(["--frame", &entry.frame.to_string()])
            .args(["--mode", &entry.mode])
            .output()?;
        if !replay.status.success() {
            let _ = fs::remove_file(&temporary);
            return Err(format!(
                "C++ reference replay failed for {}: {}",
                entry.id,
                String::from_utf8_lossy(&replay.stderr).trim()
            )
            .into());
        }
        let image = RgbaImage::read_png(&temporary).map_err(|error| {
            let _ = fs::remove_file(&temporary);
            format!(
                "C++ reference replay produced no valid PNG for {}: {error}",
                entry.id
            )
        })?;
        let provenance = provenance
            .as_ref()
            .map(|context| context.record(entry, &image, &temporary, adapter_name(&replay.stdout)?))
            .transpose()?;
        install_capture(&temporary, &entry.reference, provenance.as_deref())?;
        captured += 1;
        println!(
            "captured {}: frame={} mode={} reference={}",
            entry.id,
            entry.frame,
            entry.mode,
            entry.reference.display()
        );
    }

    if options.id.is_some() && captured == 0 {
        return Err("no matching manifest entry was selected".into());
    }
    println!("renderer-reference-capture captured={captured}");
    Ok(())
}

struct Options {
    manifest: PathBuf,
    replay: PathBuf,
    runtime_dir: PathBuf,
    backend: ReplayBackend,
    status: String,
    mode: String,
    id: Option<String>,
}

impl Options {
    fn parse() -> Result<Self, Box<dyn Error>> {
        let mut manifest = PathBuf::from("corpus-r.toml");
        let mut replay = PathBuf::from("target/debug/renderer-replay");
        let mut runtime_dir = std::env::var_os("RIVE_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));
        let mut backend = ReplayBackend::FfiMetal;
        let mut status = "exact".to_owned();
        let mut mode = "clockwise-atomic".to_owned();
        let mut id = None;
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--manifest" => manifest = PathBuf::from(args.next().ok_or(usage())?),
                "--replay" => replay = PathBuf::from(args.next().ok_or(usage())?),
                "--runtime-dir" => runtime_dir = PathBuf::from(args.next().ok_or(usage())?),
                "--backend" => backend = ReplayBackend::parse(&args.next().ok_or(usage())?)?,
                "--status" => status = args.next().ok_or(usage())?,
                "--mode" => mode = args.next().ok_or(usage())?,
                "--id" => id = Some(args.next().ok_or(usage())?),
                _ => return Err(format!("unknown argument `{arg}`\n{}", usage()).into()),
            }
        }
        Ok(Self {
            manifest,
            replay,
            runtime_dir,
            backend,
            status,
            mode,
            id,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReplayBackend {
    FfiMetal,
    FfiDawn,
}

impl ReplayBackend {
    fn parse(value: &str) -> Result<Self, Box<dyn Error>> {
        match value {
            "ffi-metal" => Ok(Self::FfiMetal),
            "ffi-dawn" => Ok(Self::FfiDawn),
            _ => Err(format!("unsupported reference backend `{value}`").into()),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::FfiMetal => "ffi-metal",
            Self::FfiDawn => "ffi-dawn",
        }
    }
}

struct ProvenanceContext {
    runtime_revision: String,
    dawn_revision: String,
    replay_sha256: String,
}

impl ProvenanceContext {
    fn load(runtime_dir: &Path, replay: &Path) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            runtime_revision: git_revision(runtime_dir)?,
            dawn_revision: git_revision(&runtime_dir.join("renderer/dependencies/dawn"))?,
            replay_sha256: sha256_file(replay)?,
        })
    }

    fn record(
        &self,
        entry: &Entry,
        image: &RgbaImage,
        png: &Path,
        adapter: String,
    ) -> Result<String, Box<dyn Error>> {
        if adapter.contains(['\n', '\r']) {
            return Err("Dawn adapter name contains a newline".into());
        }
        Ok(format!(
            concat!(
                "provenance_schema=1\n",
                "backend=metal\n",
                "renderer_implementation=cpp-dawn-webgpu\n",
                "capture_tool=renderer-replay-ffi-dawn\n",
                "adapter_device={}\n",
                "case_id={}\n",
                "stream_sha256={}\n",
                "runtime_revision={}\n",
                "dawn_revision={}\n",
                "replay_sha256={}\n",
                "png_sha256={}\n",
                "frame_width={}\n",
                "frame_height={}\n",
                "frame={}\n",
                "mode={}\n",
                "sample_count={}\n"
            ),
            adapter,
            entry.id,
            sha256_file(&entry.stream)?,
            self.runtime_revision,
            self.dawn_revision,
            self.replay_sha256,
            sha256_file(png)?,
            image.width,
            image.height,
            entry.frame,
            entry.mode,
            if entry.mode == "msaa" { 4 } else { 1 },
        ))
    }
}

fn path_str(path: &Path) -> Result<&str, Box<dyn Error>> {
    path.to_str()
        .ok_or_else(|| "path is not valid UTF-8".into())
}

fn temporary_path(reference: &Path, index: usize) -> Result<PathBuf, Box<dyn Error>> {
    let name = reference
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("reference has no UTF-8 file name")?;
    Ok(reference.with_file_name(format!(".{name}.{}.{}.capture", std::process::id(), index)))
}

fn require_supported_mode(backend: ReplayBackend, mode: &str) -> Result<(), Box<dyn Error>> {
    match (backend, mode) {
        (_, "clockwise-atomic") | (ReplayBackend::FfiDawn, "msaa") => Ok(()),
        (ReplayBackend::FfiMetal, "msaa") => {
            Err("C++ Metal reference capture only supports clockwise-atomic mode".into())
        }
        (_, mode) => Err(format!("unsupported renderer mode `{mode}`").into()),
    }
}

fn adapter_name(stdout: &[u8]) -> Result<String, Box<dyn Error>> {
    String::from_utf8_lossy(stdout)
        .lines()
        .find_map(|line| line.strip_prefix("adapter="))
        .filter(|adapter| !adapter.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| "C++ Dawn replay did not report its adapter".into())
}

fn git_revision(directory: &Path) -> Result<String, Box<dyn Error>> {
    let output = Command::new("git")
        .args(["-C", path_str(directory)?, "rev-parse", "HEAD"])
        .output()?;
    if !output.status.success() {
        return Err(format!("failed to read git revision from {}", directory.display()).into());
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

fn sha256_file(path: &Path) -> Result<String, Box<dyn Error>> {
    Ok(format!("{:x}", Sha256::digest(fs::read(path)?)))
}

fn provenance_path(reference: &Path) -> PathBuf {
    reference.with_extension("provenance")
}

fn install_capture(
    temporary: &Path,
    reference: &Path,
    provenance: Option<&str>,
) -> Result<(), std::io::Error> {
    let installed_provenance = provenance.map(|contents| {
        let destination = provenance_path(reference);
        let temporary = temporary.with_extension("provenance-capture");
        fs::write(&temporary, contents)?;
        install_reference(&temporary, &destination)?;
        Ok::<_, std::io::Error>(destination)
    });
    let installed_provenance = installed_provenance.transpose()?;
    if let Err(error) = install_reference(temporary, reference) {
        if let Some(provenance) = installed_provenance {
            let _ = fs::remove_file(provenance);
        }
        return Err(error);
    }
    Ok(())
}

fn install_reference(temporary: &Path, reference: &Path) -> Result<(), std::io::Error> {
    if let Err(error) = fs::rename(temporary, reference) {
        let _ = fs::remove_file(temporary);
        return Err(error);
    }
    Ok(())
}

fn usage() -> &'static str {
    "usage: capture-corpus-r-references [--manifest FILE] [--replay FILE] [--runtime-dir DIR] [--backend ffi-metal|ffi-dawn] [--status STATUS] [--mode MODE] [--id ID]"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_msaa_capture_before_launching_replay() {
        let error = require_supported_mode(ReplayBackend::FfiMetal, "msaa").unwrap_err();
        assert!(error.to_string().contains("only supports clockwise-atomic"));
    }

    #[test]
    fn dawn_capture_supports_both_renderer_modes() {
        require_supported_mode(ReplayBackend::FfiDawn, "clockwise-atomic").unwrap();
        require_supported_mode(ReplayBackend::FfiDawn, "msaa").unwrap();
    }

    #[test]
    fn extracts_the_adapter_reported_by_dawn_replay() {
        assert_eq!(
            adapter_name(b"adapter=Apple M5 Max\nbackend=ffi-dawn\n").unwrap(),
            "Apple M5 Max"
        );
        assert!(adapter_name(b"backend=ffi-dawn\n").is_err());
    }

    #[test]
    fn removes_temporary_png_when_install_fails() {
        let root = std::env::temp_dir().join(format!(
            "pixel-compare-install-reference-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let temporary = root.join("temporary.png");
        fs::write(&temporary, b"png").unwrap();
        let destination = root.join("destination");
        fs::create_dir(&destination).unwrap();

        install_reference(&temporary, &destination).unwrap_err();

        assert!(!temporary.exists());
        fs::remove_dir_all(root).unwrap();
    }
}
