//! Browser shared objects are owned by the JS SharedArrayBuffer transport.
//! Rust stores only descriptors; no VM pointer crosses into the AudioWorklet.
use super::luau_audio_web::request;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
pub const MAX_SHARED_BYTES: usize = 256 << 20;
pub const DEFAULT_SHARED_BUDGET: usize = 512 << 20;
pub const MAX_SHARED_OBJECTS: usize = 65_536;
pub const PROCESSOR_HOLDS_ERROR: &str = "a processor cannot pass shared objects through rings or snapshots; pass them in Audio.start's argument";
include!("luau_audio_shared_codec.rs");

pub struct SharedBudget {
    maximum_bytes: usize,
    owner: Mutex<Option<Result<Value, String>>>,
}
impl Default for SharedBudget {
    fn default() -> Self {
        Self::new(DEFAULT_SHARED_BUDGET)
    }
}
impl SharedBudget {
    pub fn new(maximum_bytes: usize) -> Self {
        Self {
            maximum_bytes,
            owner: Mutex::new(None),
        }
    }
    fn owner(&self) -> Result<Value, String> {
        self.owner
            .lock()
            .map_err(|_| "shared budget lock poisoned")?
            .get_or_insert_with(|| {
                request(json!({"op":"owner.create","maximumBytes":self.maximum_bytes}))
            })
            .clone()
    }
    pub fn close(&self) {
        if let Ok(mut owner) = self.owner.lock() {
            if let Some(Ok(id)) = owner.as_ref() {
                let _ = request(json!({"op":"owner.close","owner":id}));
            }
            *owner = Some(Err("shared owner has been released".into()));
        }
    }
}
#[derive(Debug)]
struct Handle(Value);
impl Handle {
    fn call(&self, op: &str, mut args: Value) -> Result<Value, String> {
        args["op"] = json!(op);
        args["handle"] = self.0.clone();
        request(args)
    }
    fn number(&self, op: &str) -> usize {
        self.call(op, json!({}))
            .ok()
            .and_then(|value| value.as_u64())
            .unwrap_or(0) as usize
    }
}
impl Drop for Handle {
    fn drop(&mut self) {
        let _ = self.call("shared.release", json!({}));
    }
}
fn create(
    kind: &str,
    capacity: usize,
    budget: Arc<SharedBudget>,
    bytes: Option<&[u8]>,
) -> Result<Handle, String> {
    let mut args =
        json!({"op":format!("{kind}.create"),"owner":budget.owner()?,"capacity":capacity});
    if let Some(bytes) = bytes {
        args["bytes"] = json!(bytes);
    }
    Ok(Handle(request(args)?))
}
#[derive(Debug)]
pub struct SharedRing(Handle);
#[derive(Debug)]
pub struct SharedSnapshot(Handle);
#[derive(Debug)]
pub struct SharedBlob(Handle);
#[derive(Debug)]
pub struct SharedRegion(Handle);
fn descriptor(value: &SharedValue) -> Result<Value, String> {
    Ok(match value {
        SharedValue::Ring(v) => v.0.0.clone(),
        SharedValue::Snapshot(v) => v.0.0.clone(),
        SharedValue::Blob(v) => v.0.0.clone(),
        SharedValue::Region(v) => v.0.0.clone(),
        _ => return Err("invalid shared hold".into()),
    })
}
/// Encoded holds remain borrowed until the receiver's transport retains them.
pub(super) fn encode_wire(value: &SharedValue) -> Result<Value, String> {
    let encoded = encode(value)?;
    let holds = encoded
        .holds
        .iter()
        .map(descriptor)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(json!({"bytes":encoded.bytes,"holds":holds}))
}
/// Transport has transferred one hold per descriptor to this decoder.
pub(super) fn decode_wire(value: Value) -> Result<SharedValue, String> {
    let bytes: Vec<u8> =
        serde_json::from_value(value["bytes"].clone()).map_err(|e| e.to_string())?;
    let descriptors = value["holds"].as_array().ok_or("invalid shared holds")?;
    let mut holds = Vec::new();
    for descriptor in descriptors {
        let kind = descriptor["kind"].as_str().ok_or("invalid shared kind")?;
        let handle = Handle(descriptor.clone());
        holds.push(match kind {
            "ring" => SharedValue::Ring(Arc::new(SharedRing(handle))),
            "snapshot" => SharedValue::Snapshot(Arc::new(SharedSnapshot(handle))),
            "blob" => SharedValue::Blob(Arc::new(SharedBlob(handle))),
            "region" => SharedValue::Region(Arc::new(SharedRegion(handle))),
            _ => return Err("invalid shared kind".into()),
        });
    }
    decode(&bytes, &holds)
}
fn truth(value: Value) -> Result<bool, String> {
    value
        .as_bool()
        .ok_or_else(|| "invalid shared boolean response".into())
}
impl SharedRing {
    pub fn new(capacity: usize) -> Result<Self, String> {
        Self::with_budget(capacity, Arc::new(SharedBudget::default()))
    }
    pub fn with_budget(capacity: usize, budget: Arc<SharedBudget>) -> Result<Self, String> {
        Ok(Self(create("ring", capacity, budget, None)?))
    }
    pub fn capacity(&self) -> usize {
        self.0.number("capacity")
    }
    pub fn used(&self) -> usize {
        self.0.number("ring.used")
    }
    pub fn send(&self, value: SharedValue) -> Result<bool, String> {
        truth(
            self.0
                .call("ring.send", json!({"encoded":encode_wire(&value)?}))?,
        )
    }
    pub fn send_from_processor(&self, value: SharedValue) -> Result<bool, String> {
        if value.contains_handles() {
            return Err(PROCESSOR_HOLDS_ERROR.into());
        }
        truth(self.0.call(
            "ring.send",
            json!({"encoded":encode_wire(&value)?,"processor":true}),
        )?)
    }
    pub fn try_receive(&self) -> Result<Option<SharedValue>, String> {
        let value = self.0.call("ring.receive", json!({}))?;
        if value.is_null() {
            Ok(None)
        } else {
            decode_wire(value).map(Some)
        }
    }
    pub fn receive(&self) -> Option<SharedValue> {
        self.try_receive().expect("shared transport invariant")
    }
    pub fn receive_for_processor(&self) -> Result<Option<SharedValue>, String> {
        let value = self.0.call("ring.receive", json!({"processor":true}))?;
        if value.is_null() {
            Ok(None)
        } else {
            decode_wire(value).map(Some)
        }
    }
}
impl SharedSnapshot {
    pub fn new(capacity: usize) -> Result<Self, String> {
        Self::with_budget(capacity, Arc::new(SharedBudget::default()))
    }
    pub fn with_budget(capacity: usize, budget: Arc<SharedBudget>) -> Result<Self, String> {
        Ok(Self(create("snapshot", capacity, budget, None)?))
    }
    pub fn capacity(&self) -> usize {
        self.0.number("capacity")
    }
    pub fn seq(&self) -> u64 {
        self.0
            .call("snapshot.seq", json!({}))
            .ok()
            .and_then(|v| v.as_str().and_then(|s| s.parse().ok()))
            .unwrap_or(0)
    }
    pub fn publish(&self, value: SharedValue) -> Result<bool, String> {
        truth(
            self.0
                .call("snapshot.publish", json!({"encoded":encode_wire(&value)?}))?,
        )
    }
    pub fn publish_from_processor(&self, value: SharedValue) -> Result<bool, String> {
        if value.contains_handles() {
            return Err(PROCESSOR_HOLDS_ERROR.into());
        }
        truth(self.0.call(
            "snapshot.publish",
            json!({"encoded":encode_wire(&value)?,"processor":true}),
        )?)
    }
    fn read(&self, processor: bool) -> Result<Option<(u64, SharedValue)>, String> {
        let value = self
            .0
            .call("snapshot.read", json!({"processor":processor}))?;
        if value.is_null() {
            return Ok(None);
        }
        let sequence = value["sequence"]
            .as_str()
            .ok_or("invalid shared sequence")?
            .parse::<u64>()
            .map_err(|e| e.to_string())?;
        Ok(Some((sequence, decode_wire(value["encoded"].clone())?)))
    }
    pub fn try_latest(&self) -> Result<Option<(u64, SharedValue)>, String> {
        self.read(false)
    }
    pub fn latest(&self) -> Option<(u64, SharedValue)> {
        self.try_latest().expect("shared transport invariant")
    }
    pub fn latest_for_processor(&self) -> Result<Option<(u64, SharedValue)>, String> {
        self.read(true)
    }
}
impl SharedBlob {
    pub fn with_budget(bytes: &[u8], budget: Arc<SharedBudget>) -> Result<Self, String> {
        Ok(Self(create("blob", bytes.len(), budget, Some(bytes))?))
    }
    pub fn size(&self) -> usize {
        self.0.number("size")
    }
    pub fn read(&self, offset: usize, out: &mut [u8]) -> usize {
        let Ok(value) = self
            .0
            .call("blob.read", json!({"offset":offset,"length":out.len()}))
        else {
            return 0;
        };
        let Ok(bytes) = serde_json::from_value::<Vec<u8>>(value) else {
            return 0;
        };
        let count = out.len().min(bytes.len());
        out[..count].copy_from_slice(&bytes[..count]);
        count
    }
}
impl SharedRegion {
    pub fn with_budget(size: usize, budget: Arc<SharedBudget>) -> Result<Self, String> {
        Ok(Self(create("region", size, budget, None)?))
    }
    pub fn size(&self) -> usize {
        self.0.number("size")
    }
    pub fn poisoned(&self) -> bool {
        self.0
            .call("region.poisoned", json!({}))
            .ok()
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }
    fn atomic(&self, op: &str, args: Value) -> Result<u32, String> {
        self.0
            .call(op, args)?
            .as_u64()
            .and_then(|v| u32::try_from(v).ok())
            .ok_or_else(|| "invalid shared word".into())
    }
    pub fn atomic_load(&self, offset: usize) -> Result<u32, String> {
        self.atomic("region.load", json!({"offset":offset}))
    }
    pub fn atomic_store(&self, offset: usize, value: u32) -> Result<u32, String> {
        self.atomic("region.store", json!({"offset":offset,"value":value}))
    }
    pub fn atomic_add(&self, offset: usize, value: u32) -> Result<u32, String> {
        self.atomic("region.add", json!({"offset":offset,"value":value}))
    }
    pub fn atomic_cas(&self, offset: usize, expected: u32, desired: u32) -> Result<u32, String> {
        self.atomic(
            "region.cas",
            json!({"offset":offset,"expected":expected,"desired":desired}),
        )
    }
    pub fn try_lock(
        self: &Arc<Self>,
        timeout: std::time::Duration,
    ) -> Result<Option<RegionGuard>, String> {
        Ok(
            if truth(self.0.call(
                "region.lock",
                json!({"timeoutMs":timeout.as_secs_f64()*1000.0}),
            )?)? {
                Some(RegionGuard {
                    region: self.clone(),
                    poison: std::cell::Cell::new(false),
                })
            } else {
                None
            },
        )
    }
}
pub struct RegionGuard {
    region: Arc<SharedRegion>,
    poison: std::cell::Cell<bool>,
}
impl RegionGuard {
    pub fn poison(&self) {
        self.poison.set(true);
    }
    pub fn read(&self, offset: usize, out: &mut [u8]) -> Result<(), String> {
        let bytes: Vec<u8> = serde_json::from_value(
            self.region
                .0
                .call("region.read", json!({"offset":offset,"length":out.len()}))?,
        )
        .map_err(|e| e.to_string())?;
        if bytes.len() != out.len() {
            return Err("invalid region read length".into());
        }
        out.copy_from_slice(&bytes);
        Ok(())
    }
    pub fn write(&self, offset: usize, bytes: &[u8]) -> Result<(), String> {
        self.region
            .0
            .call("region.write", json!({"offset":offset,"bytes":bytes}))?;
        Ok(())
    }
}
impl Drop for RegionGuard {
    fn drop(&mut self) {
        let _ = self
            .region
            .0
            .call("region.unlock", json!({"poison":self.poison.get()}));
    }
}
