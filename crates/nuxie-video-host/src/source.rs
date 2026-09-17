//! A bounded, owned file lease for decoders that require seekable files.
//! SDK content-addressed caches remain the normal external-source owner. This
//! helper covers embedded stream bytes without retaining a borrowed buffer or
//! repeatedly extracting the clip for each decoded frame.
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_FILE: AtomicU64 = AtomicU64::new(0);

pub struct EmbeddedFile {
    path: PathBuf,
}
impl EmbeddedFile {
    /// `directory` must be an existing app-owned private cache directory.
    /// Keep the lease alive until all players using its path have closed.
    /// Files use exclusive creation and owner-only permissions on Unix hosts.
    pub fn materialize(directory: &Path, bytes: &[u8], max_bytes: usize) -> io::Result<Self> {
        if bytes.is_empty() || bytes.len() > max_bytes {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "embedded video exceeds source budget or is empty",
            ));
        }
        for _ in 0..64 {
            let id = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
            let path = directory.join(format!("nux-video-{}-{id}.mp4", std::process::id()));
            let mut options = OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = match options.open(&path) {
                Ok(file) => file,
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            };
            let lease = Self { path };
            if let Err(error) = file.write_all(bytes) {
                drop(file);
                drop(lease);
                return Err(error);
            }
            // Close before handing the source to the decoder. On an error the
            // lease drops and removes only the file created by this call.
            drop(file);
            return Ok(lease);
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "embedded video cache filename collision limit",
        ))
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
}
impl Drop for EmbeddedFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extracted_sources_are_independent_bounded_and_removed_on_release() {
        let directory = std::env::temp_dir();
        let first = EmbeddedFile::materialize(&directory, b"clip-one", 8).unwrap();
        let second = EmbeddedFile::materialize(&directory, b"clip-two", 8).unwrap();
        assert_ne!(first.path(), second.path());
        assert_eq!(fs::read(first.path()).unwrap(), b"clip-one");
        assert!(EmbeddedFile::materialize(&directory, b"too large", 1).is_err());
        let first_path = first.path().to_owned();
        drop(first);
        assert!(!first_path.exists());
        assert_eq!(fs::read(second.path()).unwrap(), b"clip-two");
    }
}
