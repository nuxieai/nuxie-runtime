//! Direct port of pinned `src/lua/renderer/lua_blob.cpp`.

use std::cell::RefCell;
use std::rc::Rc;

use luaur_rt::{Lua, Result, UserData, UserDataFields, Value};

#[derive(Debug)]
struct ScriptedBlobData {
    name: String,
    short_name: String,
    bytes: Rc<[u8]>,
}

#[derive(Debug, Clone)]
pub(super) struct ScriptedBlob(Rc<ScriptedBlobData>);

impl UserData for ScriptedBlob {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_, this| Ok(this.0.bytes.len() as f64));
        fields.add_field_method_get("name", |_, this| Ok(this.0.short_name.clone()));
        fields.add_field_method_get("data", |lua, this| {
            if this.0.bytes.is_empty() {
                return Ok(Value::Nil);
            }
            lua.create_buffer(this.0.bytes.as_ref()).map(Value::Buffer)
        });
    }
}

/// File-owned BlobAsset values visible to `Context:blob`.
///
/// Values are retained independently of the VM stack, matching C++'s
/// `ScriptedBlob::asset` reference-counted FileAsset owner.
#[derive(Clone, Default)]
pub(super) struct ScriptedBlobAssets {
    // File order is significant: equal-rank matches keep the first authored
    // asset, exactly like C++'s linear `file->assets()` scan.
    assets: Rc<RefCell<Vec<Rc<ScriptedBlobData>>>>,
}

impl ScriptedBlobAssets {
    pub(super) fn install(lua: &Lua) -> Self {
        let assets = Self::default();
        lua.set_app_data(assets.clone());
        assets
    }

    pub(super) fn register(&self, name: &str, short_name: &str, bytes: &[u8]) -> Result<()> {
        self.assets.borrow_mut().push(Rc::new(ScriptedBlobData {
            name: name.to_owned(),
            short_name: short_name.to_owned(),
            bytes: Rc::from(bytes),
        }));
        Ok(())
    }

    pub(super) fn lookup(lua: &Lua, name: &str) -> Result<Value> {
        let Some(assets) = lua.app_data_ref::<Self>().map(|assets| assets.clone()) else {
            return Ok(Value::Nil);
        };
        let reference = ScopedAssetReference::new(lua, name);
        let mut best_rank = 0;
        let mut asset = None;
        for candidate in assets.assets.borrow().iter() {
            let rank = reference.rank(&candidate.name, &candidate.short_name);
            if rank > best_rank && !candidate.bytes.is_empty() {
                best_rank = rank;
                asset = Some(Rc::clone(candidate));
            }
        }
        match asset {
            // Pinned Context:blob deliberately rejects zero-byte BlobAssets.
            Some(asset) => lua
                .create_userdata(ScriptedBlob(asset))
                .map(Value::UserData),
            _ => Ok(Value::Nil),
        }
    }
}

pub(crate) struct ScopedAssetReference {
    label: String,
    path: String,
    scope_prefix: String,
    bare: String,
}

impl ScopedAssetReference {
    pub(crate) fn new(lua: &Lua, reference: &str) -> Self {
        if let Some(rest) = reference.strip_prefix("lib:") {
            if let Some((label, path)) = rest.split_once('/') {
                return Self {
                    label: label.to_owned(),
                    path: path.to_owned(),
                    scope_prefix: String::new(),
                    bare: String::new(),
                };
            }
        }
        let scope_prefix = super::caller_chunk_source(lua)
            .and_then(|chunkname| {
                let slash = chunkname.find('/')?;
                let first = &chunkname[..slash];
                first
                    .find('@')
                    .filter(|at| *at > 0)
                    .map(|_| first.to_owned())
            })
            .unwrap_or_default();
        Self {
            label: String::new(),
            path: String::new(),
            scope_prefix,
            bare: reference.to_owned(),
        }
    }

    pub(crate) fn rank(&self, registered_name: &str, short_name: &str) -> u8 {
        if !self.label.is_empty() {
            return u8::from(self.matches_library(registered_name));
        }
        if !self.scope_prefix.is_empty()
            && registered_name
                .strip_prefix(&self.scope_prefix)
                .is_some_and(|rest| rest.starts_with('/'))
        {
            let relative = &registered_name[self.scope_prefix.len() + 1..];
            return u8::from(relative == self.bare || short_name == self.bare) * 2;
        }
        let first_segment = registered_name.split('/').next().unwrap_or(registered_name);
        if first_segment.find('@').is_some_and(|at| at > 0) {
            return 0;
        }
        u8::from(registered_name == self.bare || short_name == self.bare)
    }

    fn matches_library(&self, registered_name: &str) -> bool {
        let Some(mut rest) = registered_name.strip_prefix(&self.label) else {
            return false;
        };
        if let Some(after_hash) = rest.strip_prefix('#') {
            let digits = after_hash.bytes().take_while(u8::is_ascii_digit).count();
            if digits == 0 {
                return false;
            }
            rest = &after_hash[digits..];
        }
        let Some(after_at) = rest.strip_prefix('@') else {
            return false;
        };
        let digits = after_at.bytes().take_while(u8::is_ascii_digit).count();
        digits > 0 && after_at[digits..].strip_prefix('/') == Some(self.path.as_str())
    }
}
