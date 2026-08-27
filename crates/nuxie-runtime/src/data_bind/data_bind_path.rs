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
        self.path.extend(decode_varuint_path(bytes));
        true
    }

    pub(crate) fn decoded_resolved(bytes: &[u8]) -> Option<Self> {
        Some(Self {
            path: decode_varuint_path(bytes),
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

    pub(crate) fn copy_path(&mut self, object: &Self) {
        self.path.clone_from(&object.path);
        self.resolved = object.resolved;
    }

    pub(crate) fn path(&self) -> &[u32] {
        &self.path
    }

    pub(crate) fn path_mut(&mut self) -> &mut Vec<u32> {
        &mut self.path
    }

    pub(crate) fn is_resolved(&self) -> bool {
        self.resolved
    }

    pub(crate) fn set_resolved(&mut self, resolved: bool) {
        self.resolved = resolved;
    }

    pub(crate) fn file_identity(&self) -> Option<u64> {
        self.file_identity
    }

    pub(crate) fn set_file_identity(&mut self, file_identity: Option<u64>) {
        self.file_identity = file_identity;
    }

    /// Rust keeps the importing Backboard's stable file identity instead of
    /// the pinned raw `File*`; base `Core::import` is an unconditional Ok.
    pub(crate) fn import(&mut self, backboard_file_identity: Option<u64>) -> bool {
        let Some(file_identity) = backboard_file_identity else {
            return false;
        };
        self.file_identity = Some(file_identity);
        true
    }

    pub(crate) fn resolved_path(&mut self, resolver: Option<&dyn Fn(u32) -> Vec<u32>>) -> &[u32] {
        if self.resolved {
            return &self.path;
        }
        if self.file_identity.is_none() {
            // Pinned C++ leaves the path unresolved only when there is no
            // live File, so a later import context can still resolve it.
            return &self.path;
        }
        if let Some(resolver) = resolver {
            if self.path.len() == 1 {
                self.path = resolver(self.path[0]);
            }
        }
        self.resolved = true;
        &self.path
    }
}

fn decode_varuint_path(bytes: &[u8]) -> Vec<u32> {
    let mut path = Vec::new();
    let mut cursor = 0;
    while cursor < bytes.len() {
        let (value, read) = read_varuint64(&bytes[cursor..]);
        if read == 0 {
            // BinaryReader::readVarUint64 returns 0 and moves to the end on
            // overflow; decodePath pushes that value before reachedEnd().
            path.push(0);
            break;
        }
        cursor += read;
        let Ok(value) = u32::try_from(value) else {
            // readVarUintAs<uint32_t> likewise returns and pushes 0 before
            // its integer-range error makes reachedEnd() true.
            path.push(0);
            break;
        };
        path.push(value);
    }
    path
}

fn read_varuint64(bytes: &[u8]) -> (u64, usize) {
    let mut result = 0u64;
    let mut shift = 0u8;

    for (index, byte) in bytes.iter().copied().enumerate() {
        result |= u64::from(byte & 0x7f).wrapping_shl(u32::from(shift));
        shift = shift.wrapping_add(7);
        if byte & 0x80 == 0 {
            return (result, index + 1);
        }
    }

    (0, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_copy_and_deferred_resolution_preserve_occurrence_identity() {
        let mut path = RuntimeDataBindPath::default();
        assert!(path.decode_path(&[0xac, 0x02]));
        let mut copied = RuntimeDataBindPath::default();
        copied.copy_path(&path);

        assert_eq!(copied.resolved_path(None), &[300]);
        assert!(!copied.is_resolved());
        assert!(copied.import(Some(17)));
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
    fn malformed_or_out_of_range_varuint_pushes_zero_then_stops() {
        let mut path = RuntimeDataBindPath::authored(vec![7], Some(1));
        assert!(path.decode_path(&[0x80]));
        assert_eq!(path.path(), &[7, 0]);

        let mut out_of_range = RuntimeDataBindPath::default();
        assert!(out_of_range.decode_path(&[0x80, 0x80, 0x80, 0x80, 0x10, 0x01]));
        assert_eq!(out_of_range.path(), &[0]);
    }

    #[test]
    fn a_file_without_a_resolver_still_finishes_resolution() {
        let mut path = RuntimeDataBindPath::authored(vec![5], Some(9));
        assert_eq!(path.resolved_path(None), &[5]);
        assert!(path.is_resolved());
        assert_eq!(path.resolved_path(Some(&|id| vec![id + 1])), &[5]);
    }

    #[test]
    fn copy_path_does_not_copy_the_file_and_import_requires_a_backboard() {
        let original = RuntimeDataBindPath::resolved(vec![1, 2], Some(9));
        let mut copied = RuntimeDataBindPath::authored(vec![3], Some(4));
        copied.copy_path(&original);
        assert_eq!(copied.path(), &[1, 2]);
        assert!(copied.is_resolved());
        assert_eq!(copied.file_identity(), Some(4));

        assert!(!copied.import(None));
        assert_eq!(copied.file_identity(), Some(4));
        copied.set_file_identity(None);
        assert_eq!(copied.file_identity(), None);
        assert!(copied.import(Some(12)));
        assert_eq!(copied.file_identity(), Some(12));
    }
}
