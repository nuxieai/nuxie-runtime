// One limit set for a shared value, checked where a value is built from a
// script table (`to_shared`, the processor's `read`) and again where it is
// decoded from bytes (`decode`). Every other walk over a `SharedValue`
// (`encode`, `contains_handles`, pushing it into a VM, dropping it) only ever
// sees a value that passed one of those, so its recursion is bounded by
// `MAX_SHARED_DEPTH`.
//
// Depth: tables on one path. Both builders already refused a 65th table
// with "audio message nesting exceeds 64 tables"; the number is unchanged.
pub const MAX_SHARED_DEPTH: usize = 64;
// Values in one shared value, containers included. A script VM charges 16
// bytes per table slot, so a value with no table reachable twice that fits in
// the default 64 MiB file VM has fewer than 2^22 values; only a table
// reached many times over (each reach is copied) can exceed this.
pub const MAX_SHARED_NODES: usize = 1 << 22;
// Bytes: the encoded size, `MAX_SHARED_BYTES` (defined by the including
// module), counted exactly as `SharedValue::layout` counts it.

/// The running count of one shared value against the limit set.
pub struct SharedValueBudget {
    nodes: usize,
    bytes: usize,
    max_nodes: usize,
    max_bytes: usize,
}
impl Default for SharedValueBudget {
    fn default() -> Self {
        Self::with_limits(MAX_SHARED_NODES, MAX_SHARED_BYTES)
    }
}
impl SharedValueBudget {
    pub const fn with_limits(max_nodes: usize, max_bytes: usize) -> Self {
        Self {
            nodes: 0,
            bytes: 0,
            max_nodes,
            max_bytes,
        }
    }
    /// Charges one value whose own encoding (tag, length word, payload, not
    /// its children) takes `encoded` bytes. Call it before copying the value.
    pub fn value(&mut self, encoded: usize) -> Result<(), String> {
        self.nodes = self.nodes.saturating_add(1);
        if self.nodes > self.max_nodes {
            return Err(format!("audio message exceeds {} values", self.max_nodes));
        }
        self.bytes(encoded)
    }
    /// Refuses `count` more values when they cannot fit, without charging.
    pub fn reserve(&self, count: usize) -> Result<(), String> {
        if count > self.max_nodes.saturating_sub(self.nodes) {
            return Err(format!("audio message exceeds {} values", self.max_nodes));
        }
        Ok(())
    }
    /// Charges a record key of `length` bytes (its length word and bytes).
    pub fn key(&mut self, length: usize) -> Result<(), String> {
        self.bytes(length.saturating_add(4))
    }
    fn bytes(&mut self, encoded: usize) -> Result<(), String> {
        self.bytes = self.bytes.saturating_add(encoded);
        if self.bytes > self.max_bytes {
            return Err("shared value too large".into());
        }
        Ok(())
    }
}
/// Encoded size of a value of each kind, before its children.
pub const SHARED_NIL_BYTES: usize = 1;
pub const SHARED_BOOLEAN_BYTES: usize = 2;
pub const SHARED_NUMBER_BYTES: usize = 9;
pub const SHARED_CONTAINER_BYTES: usize = 5;
pub const SHARED_HANDLE_BYTES: usize = 5;
pub const fn shared_bytes_size(length: usize) -> usize {
    length.saturating_add(5)
}

#[derive(Clone, Debug)]
pub enum SharedValue {
    Nil,
    Boolean(bool),
    Number(f64),
    Bytes(Vec<u8>),
    Array(Vec<SharedValue>),
    Object(BTreeMap<String, SharedValue>),
    Blob(Arc<SharedBlob>),
    Region(Arc<SharedRegion>),
    Ring(Arc<SharedRing>),
    Snapshot(Arc<SharedSnapshot>),
}
impl SharedValue {
    pub fn contains_handles(&self) -> bool {
        match self {
            Self::Blob(_) | Self::Region(_) | Self::Ring(_) | Self::Snapshot(_) => true,
            Self::Array(values) => values.iter().any(Self::contains_handles),
            Self::Object(values) => values.values().any(Self::contains_handles),
            _ => false,
        }
    }
    fn layout(&self) -> Result<(usize, usize), String> {
        let mut bytes = 0usize;
        let mut holds = 0usize;
        let mut pending = vec![self];
        while let Some(value) = pending.pop() {
            let size = match value {
                Self::Nil => 1,
                Self::Boolean(_) => 2,
                Self::Number(_) => 9,
                Self::Bytes(value) => 5usize
                    .checked_add(value.len())
                    .ok_or("shared value too large")?,
                Self::Array(values) => {
                    pending.extend(values);
                    5
                }
                Self::Object(values) => {
                    let mut size = 5usize;
                    for (key, value) in values {
                        size = size
                            .checked_add(4)
                            .and_then(|n| n.checked_add(key.len()))
                            .ok_or("shared value too large")?;
                        pending.push(value);
                    }
                    size
                }
                Self::Blob(_) | Self::Region(_) | Self::Ring(_) | Self::Snapshot(_) => {
                    holds = holds.checked_add(1).ok_or("shared value too large")?;
                    5
                }
            };
            bytes = bytes.checked_add(size).ok_or("shared value too large")?;
            if bytes > MAX_SHARED_BYTES {
                return Err("shared value too large".into());
            }
        }
        let trailer = if holds == 0 {
            0
        } else {
            holds
                .checked_mul(4)
                .and_then(|n| n.checked_add(4))
                .ok_or("shared value too large")?
        };
        Ok((
            bytes.checked_add(trailer).ok_or("shared value too large")?,
            holds,
        ))
    }
}

// Encoded bytes are the actual bounded transport storage. Shared-object holds
// are kept separately only for main-lane records; processors cannot take them.
struct EncodedValue {
    bytes: Vec<u8>,
    holds: Vec<SharedValue>,
}
fn encode(value: &SharedValue) -> Result<EncodedValue, String> {
    let (size, _) = value.layout()?;
    let mut encoded = EncodedValue {
        bytes: Vec::new(),
        holds: Vec::new(),
    };
    encoded
        .bytes
        .try_reserve_exact(size)
        .map_err(|_| "cannot encode shared value")?;
    fn word(out: &mut Vec<u8>, value: usize) -> Result<(), String> {
        let value = u32::try_from(value).map_err(|_| "shared value too large")?;
        out.extend_from_slice(&value.to_le_bytes());
        Ok(())
    }
    fn write(out: &mut EncodedValue, value: &SharedValue) -> Result<(), String> {
        match value {
            SharedValue::Nil => out.bytes.push(0),
            SharedValue::Boolean(value) => out.bytes.extend_from_slice(&[1, u8::from(*value)]),
            SharedValue::Number(value) => {
                out.bytes.push(2);
                out.bytes.extend_from_slice(&value.to_le_bytes());
            }
            SharedValue::Bytes(value) => {
                out.bytes.push(3);
                word(&mut out.bytes, value.len())?;
                out.bytes.extend_from_slice(value);
            }
            SharedValue::Array(values) => {
                out.bytes.push(4);
                word(&mut out.bytes, values.len())?;
                for value in values {
                    write(out, value)?;
                }
            }
            SharedValue::Object(values) => {
                out.bytes.push(5);
                word(&mut out.bytes, values.len())?;
                for (key, value) in values {
                    word(&mut out.bytes, key.len())?;
                    out.bytes.extend_from_slice(key.as_bytes());
                    write(out, value)?;
                }
            }
            SharedValue::Blob(_)
            | SharedValue::Region(_)
            | SharedValue::Ring(_)
            | SharedValue::Snapshot(_) => {
                out.bytes.push(6);
                word(&mut out.bytes, out.holds.len())?;
                out.holds.push(value.clone());
            }
        }
        Ok(())
    }
    write(&mut encoded, value)?;
    Ok(encoded)
}
fn decode(bytes: &[u8], holds: &[SharedValue]) -> Result<SharedValue, String> {
    // The stream is not trusted: a ring also carries raw PCM records and the
    // browser transport hands bytes in from JS. The decoder checks the same
    // limit set as the builders, so what it returns is as bounded as what
    // they return.
    struct Reader<'a> {
        bytes: &'a [u8],
        at: usize,
        holds: &'a [SharedValue],
        budget: SharedValueBudget,
    }
    impl Reader<'_> {
        fn take(&mut self, size: usize) -> Result<&[u8], String> {
            let end = self.at.checked_add(size).ok_or("invalid shared value")?;
            let value = self.bytes.get(self.at..end).ok_or("invalid shared value")?;
            self.at = end;
            Ok(value)
        }
        fn array<const N: usize>(&mut self) -> Result<[u8; N], String> {
            self.take(N)?
                .try_into()
                .map_err(|_| "invalid shared value".into())
        }
        fn word(&mut self) -> Result<usize, String> {
            Ok(u32::from_le_bytes(self.array()?) as usize)
        }
        fn remaining(&self) -> usize {
            self.bytes.len().saturating_sub(self.at)
        }
        /// Reads the element count of a container at `depth` containers
        /// below the root; each element takes at least `element_bytes`.
        fn container(&mut self, depth: usize, element_bytes: usize) -> Result<usize, String> {
            if depth >= MAX_SHARED_DEPTH {
                return Err(format!(
                    "shared value nesting exceeds {MAX_SHARED_DEPTH} tables"
                ));
            }
            let size = self.word()?;
            if size
                .checked_mul(element_bytes)
                .is_none_or(|bytes| bytes > self.remaining())
            {
                return Err("invalid shared value".into());
            }
            // Refused before the elements are reserved, not after.
            self.budget.reserve(size)?;
            Ok(size)
        }
        fn value(&mut self, depth: usize) -> Result<SharedValue, String> {
            // Bytes are bounded by the input; only the value count is charged.
            self.budget.value(0)?;
            let [tag] = self.array()?;
            Ok(match tag {
                0 => SharedValue::Nil,
                1 => {
                    let [value] = self.array()?;
                    SharedValue::Boolean(value != 0)
                }
                2 => SharedValue::Number(f64::from_le_bytes(self.array()?)),
                3 => {
                    let size = self.word()?;
                    SharedValue::Bytes(self.take(size)?.to_vec())
                }
                4 => {
                    let size = self.container(depth, SHARED_NIL_BYTES)?;
                    let mut values = Vec::new();
                    values
                        .try_reserve_exact(size)
                        .map_err(|_| "cannot decode shared array")?;
                    for _ in 0..size {
                        values.push(self.value(depth.saturating_add(1))?);
                    }
                    SharedValue::Array(values)
                }
                5 => {
                    // A key's length word plus the value's tag.
                    let size = self.container(depth, 5)?;
                    let mut values = BTreeMap::new();
                    for _ in 0..size {
                        let size = self.word()?;
                        let key = std::str::from_utf8(self.take(size)?)
                            .map_err(|_| "invalid shared key")?
                            .to_owned();
                        values.insert(key, self.value(depth.saturating_add(1))?);
                    }
                    SharedValue::Object(values)
                }
                6 => {
                    let index = self.word()?;
                    self.holds
                        .get(index)
                        .ok_or("invalid shared handle")?
                        .clone()
                }
                _ => return Err("invalid shared value tag".into()),
            })
        }
    }
    if bytes.len() > MAX_SHARED_BYTES {
        return Err("shared value too large".into());
    }
    let mut reader = Reader {
        bytes,
        at: 0,
        holds,
        budget: SharedValueBudget::default(),
    };
    let value = reader.value(0)?;
    if reader.at != bytes.len() {
        return Err("invalid shared value suffix".into());
    }
    Ok(value)
}
