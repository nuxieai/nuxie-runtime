//! Direct Rust owner for pinned C++ `src/data_bind_path_referencer.cpp`.

use crate::data_bind_path::RuntimeDataBindPath;

/// Unique owner of one authored DataBindPath occurrence.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RuntimeDataBindPathReferencer {
    path: Option<RuntimeDataBindPath>,
}

impl RuntimeDataBindPathReferencer {
    pub(crate) fn path(&self) -> Option<&RuntimeDataBindPath> {
        self.path.as_ref()
    }

    pub(crate) fn copy_data_bind_path(&mut self, path: Option<&RuntimeDataBindPath>) {
        if let Some(path) = path {
            self.path = Some(path.clone());
        }
    }

    pub(crate) fn claim_imported_path(&mut self, path: RuntimeDataBindPath) -> bool {
        if self.path.is_some() {
            return false;
        }
        self.path = Some(path);
        true
    }

    pub(crate) fn decode_data_bind_path(&mut self, bytes: &[u8]) -> bool {
        let Some(path) = RuntimeDataBindPath::decoded_resolved(bytes) else {
            return false;
        };
        self.path = Some(path);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claimed_path_cannot_replace_an_existing_occurrence() {
        let mut referencer = RuntimeDataBindPathReferencer::default();
        assert!(
            referencer.claim_imported_path(RuntimeDataBindPath::authored(vec![1, 2], Some(9),))
        );
        assert!(!referencer.claim_imported_path(RuntimeDataBindPath::default()));
        assert_eq!(
            referencer.path().map(RuntimeDataBindPath::path),
            Some(&[1, 2][..])
        );
    }

    #[test]
    fn inline_decode_is_already_resolved() {
        let mut referencer = RuntimeDataBindPathReferencer::default();
        assert!(referencer.decode_data_bind_path(&[3, 4]));
        let path = referencer.path().expect("decoded path");
        assert_eq!(path.path(), &[3, 4]);
        assert!(path.is_resolved());
    }
}
