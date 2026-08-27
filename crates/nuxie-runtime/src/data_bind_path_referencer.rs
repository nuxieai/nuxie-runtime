//! Direct Rust owner for pinned C++ `src/data_bind_path_referencer.cpp`.

use crate::data_bind_path::RuntimeDataBindPath;

/// Unique owner of one authored DataBindPath occurrence.
///
/// Rust's `Option` supplies the pinned destructor's unique destruction without
/// exposing a raw owning pointer.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct RuntimeDataBindPathReferencer {
    path: Option<RuntimeDataBindPath>,
}

impl Clone for RuntimeDataBindPathReferencer {
    fn clone(&self) -> Self {
        let mut referencer = Self::default();
        referencer.copy_data_bind_path(self.path());
        referencer
    }
}

impl RuntimeDataBindPathReferencer {
    pub(crate) fn path(&self) -> Option<&RuntimeDataBindPath> {
        self.path.as_ref()
    }

    pub(crate) fn path_mut(&mut self) -> Option<&mut RuntimeDataBindPath> {
        self.path.as_mut()
    }

    pub(crate) fn copy_data_bind_path(&mut self, path: Option<&RuntimeDataBindPath>) {
        if let Some(path) = path {
            let mut copied = RuntimeDataBindPath::default();
            copied.copy_path(path);
            copied.set_file_identity(path.file_identity());
            self.path = Some(copied);
        }
    }

    /// `claimed_path` is the single-use result of the latest path importer's
    /// `claim()`. Taking the `Option` models that ownership transfer.
    pub(crate) fn import_data_bind_path(&mut self, claimed_path: Option<RuntimeDataBindPath>) {
        let Some(path) = claimed_path else {
            return;
        };

        // The pinned assertion happens after claim(), so release builds still
        // install the newly claimed path. Rust drops a displaced value rather
        // than preserving the C++ release-only leak.
        debug_assert!(
            self.path.is_none(),
            "a claimed DataBindPath cannot replace an existing occurrence"
        );
        self.path = Some(path);
    }

    /// Stable-index import code already performs the latest-importer lookup
    /// and one-shot claim before crossing into the runtime crate.
    pub(crate) fn claim_imported_path(&mut self, path: RuntimeDataBindPath) -> bool {
        self.import_data_bind_path(Some(path));
        true
    }

    pub(crate) fn decode_data_bind_path(&mut self, bytes: &[u8]) {
        let mut path = RuntimeDataBindPath::default();
        let _ = path.decode_path(bytes);
        path.set_resolved(true);
        self.path = Some(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clone_deep_copies_the_path_and_retains_its_file() {
        let mut referencer = RuntimeDataBindPathReferencer::default();
        assert!(
            referencer.claim_imported_path(RuntimeDataBindPath::authored(vec![1, 2], Some(9),))
        );
        let copied = referencer.clone();

        assert_eq!(
            copied.path().map(RuntimeDataBindPath::path),
            Some(&[1, 2][..])
        );
        assert_eq!(
            copied.path().and_then(RuntimeDataBindPath::file_identity),
            Some(9)
        );

        referencer
            .path_mut()
            .expect("original path")
            .set_file_identity(Some(10));
        assert_eq!(
            copied.path().and_then(RuntimeDataBindPath::file_identity),
            Some(9),
            "the cloned occurrence owns an independent path"
        );
    }

    #[test]
    fn a_missing_claimed_path_does_not_change_the_occurrence() {
        let mut referencer = RuntimeDataBindPathReferencer::default();
        referencer.import_data_bind_path(Some(RuntimeDataBindPath::authored(vec![1, 2], Some(9))));
        referencer.import_data_bind_path(None);
        assert_eq!(
            referencer.path().map(RuntimeDataBindPath::path),
            Some(&[1, 2][..])
        );
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "a claimed DataBindPath cannot replace an existing occurrence")]
    fn duplicate_claim_asserts_after_ownership_transfer_in_debug_builds() {
        let mut referencer = RuntimeDataBindPathReferencer::default();
        referencer.import_data_bind_path(Some(RuntimeDataBindPath::default()));
        referencer.import_data_bind_path(Some(RuntimeDataBindPath::default()));
    }

    #[test]
    #[cfg(not(debug_assertions))]
    fn duplicate_claim_replaces_the_path_in_release_builds() {
        let mut referencer = RuntimeDataBindPathReferencer::default();
        referencer.import_data_bind_path(Some(RuntimeDataBindPath::authored(vec![1], None)));
        referencer.import_data_bind_path(Some(RuntimeDataBindPath::authored(vec![2], None)));
        assert_eq!(
            referencer.path().map(RuntimeDataBindPath::path),
            Some(&[2][..])
        );
    }

    #[test]
    fn inline_decode_is_already_resolved() {
        let mut referencer = RuntimeDataBindPathReferencer::default();
        referencer.decode_data_bind_path(&[3, 4]);
        let path = referencer.path().expect("decoded path");
        assert_eq!(path.path(), &[3, 4]);
        assert!(path.is_resolved());
    }
}
