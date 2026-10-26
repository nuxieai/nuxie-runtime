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
    struct Reader<'a> {
        bytes: &'a [u8],
        at: usize,
        holds: &'a [SharedValue],
    }
    impl Reader<'_> {
        fn take(&mut self, size: usize) -> Result<&[u8], String> {
            let end = self.at.checked_add(size).ok_or("invalid shared value")?;
            let value = self.bytes.get(self.at..end).ok_or("invalid shared value")?;
            self.at = end;
            Ok(value)
        }
        fn word(&mut self) -> Result<usize, String> {
            Ok(u32::from_le_bytes(
                self.take(4)?
                    .try_into()
                    .map_err(|_| "invalid shared value")?,
            ) as usize)
        }
        fn value(&mut self) -> Result<SharedValue, String> {
            Ok(match self.take(1)?[0] {
                0 => SharedValue::Nil,
                1 => SharedValue::Boolean(self.take(1)?[0] != 0),
                2 => SharedValue::Number(f64::from_le_bytes(
                    self.take(8)?
                        .try_into()
                        .map_err(|_| "invalid shared value")?,
                )),
                3 => {
                    let size = self.word()?;
                    SharedValue::Bytes(self.take(size)?.to_vec())
                }
                4 => {
                    let size = self.word()?;
                    let mut values = Vec::new();
                    values
                        .try_reserve_exact(size)
                        .map_err(|_| "cannot decode shared array")?;
                    for _ in 0..size {
                        values.push(self.value()?);
                    }
                    SharedValue::Array(values)
                }
                5 => {
                    let size = self.word()?;
                    let mut values = BTreeMap::new();
                    for _ in 0..size {
                        let size = self.word()?;
                        let key = std::str::from_utf8(self.take(size)?)
                            .map_err(|_| "invalid shared key")?
                            .to_owned();
                        values.insert(key, self.value()?);
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
    let mut reader = Reader {
        bytes,
        at: 0,
        holds,
    };
    let value = reader.value()?;
    if reader.at != bytes.len() {
        return Err("invalid shared value suffix".into());
    }
    Ok(value)
}
