//! Direct Rust owner for pinned C++ `src/data_bind/data_bind_path.cpp`.

/// Authored DataBind path identity.
///
/// Serialized paths keep their raw varuint representation until a live file
/// resolver is available. A resolved/copy occurrence retains its own path
/// buffer; it is never flattened into a one-time lookup result shared by
/// sibling occurrences.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RuntimeDataBindPath {
    path: Vec<u32>,
    resolved: bool,
    file_identity: Option<u64>,
}

impl RuntimeDataBindPath {
    pub(crate) fn decode_path(&mut self, bytes: &[u8]) -> bool {
        let Some(path) = decode_varuint_path(bytes) else {
            return false;
        };
        self.path.extend(path);
        true
    }

    pub(crate) fn decoded_resolved(bytes: &[u8]) -> Option<Self> {
        let path = decode_varuint_path(bytes)?;
        Some(Self {
            path,
            resolved: true,
            file_identity: None,
        })
    }

    pub(crate) fn authored(path: Vec<u32>, file_identity: Option<u64>) -> Self {
        Self {
            path,
            resolved: false,
            file_identity,
        }
    }

    pub(crate) fn resolved(path: Vec<u32>, file_identity: Option<u64>) -> Self {
        Self {
            path,
            resolved: true,
            file_identity,
        }
    }

    pub(crate) fn path(&self) -> &[u32] {
        &self.path
    }

    pub(crate) fn is_resolved(&self) -> bool {
        self.resolved
    }

    pub(crate) fn file_identity(&self) -> Option<u64> {
        self.file_identity
    }

    pub(crate) fn set_file_identity(&mut self, file_identity: Option<u64>) {
        self.file_identity = file_identity;
    }

    pub(crate) fn resolved_path(&mut self, resolver: Option<&dyn Fn(u32) -> Vec<u32>>) -> &[u32] {
        if self.resolved {
            return &self.path;
        }
        let Some(resolver) = resolver else {
            // C++ leaves the path unresolved when no live File/DataResolver
            // exists, so a later context can still resolve it.
            return &self.path;
        };
        if self.path.len() == 1 {
            self.path = resolver(self.path[0]);
        }
        self.resolved = true;
        &self.path
    }
}

fn decode_varuint_path(bytes: &[u8]) -> Option<Vec<u32>> {
    let mut path = Vec::new();
    let mut cursor = 0;
    while cursor < bytes.len() {
        let mut value = 0u32;
        let mut shift = 0;
        loop {
            let byte = *bytes.get(cursor)?;
            cursor += 1;
            let payload = u32::from(byte & 0x7f);
            if shift >= 32 || payload.checked_shl(shift)? >> shift != payload {
                return None;
            }
            value |= payload << shift;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
            if shift > 28 {
                return None;
            }
        }
        path.push(value);
    }
    Some(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_copy_and_deferred_resolution_preserve_occurrence_identity() {
        let mut path = RuntimeDataBindPath::default();
        assert!(path.decode_path(&[0xac, 0x02]));
        path.set_file_identity(Some(17));
        let mut copied = path.clone();

        assert_eq!(copied.resolved_path(None), &[300]);
        assert!(!copied.is_resolved());
        assert_eq!(
            copied.resolved_path(Some(&|id| vec![id + 1, id + 2])),
            &[301, 302]
        );
        assert!(copied.is_resolved());
        assert_eq!(copied.file_identity(), Some(17));
        assert_eq!(path.path(), &[300]);
        assert!(!path.is_resolved());
    }

    #[test]
    fn malformed_varuint_does_not_partially_mutate_the_path() {
        let mut path = RuntimeDataBindPath::authored(vec![7], Some(1));
        assert!(!path.decode_path(&[0x80]));
        assert_eq!(path.path(), &[7]);
    }
}
