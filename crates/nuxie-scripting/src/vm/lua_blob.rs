//! Direct port of pinned `src/lua/renderer/lua_blob.cpp`.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::Arc;

use luaur_rt::{Lua, Result, UserData, UserDataFields, Value};
use nuxie_runtime::RuntimeBlobAsset;

#[derive(Debug, Clone)]
pub(super) struct ScriptedBlob(Arc<RuntimeBlobAsset>);

impl UserData for ScriptedBlob {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_, this| Ok(this.0.bytes().len() as f64));
        fields.add_field_method_get("name", |_, this| Ok(this.0.name().to_owned()));
        fields.add_field_method_get("data", |lua, this| {
            if this.0.bytes().is_empty() {
                return Ok(Value::Nil);
            }
            lua.create_buffer(this.0.bytes()).map(Value::Buffer)
        });
    }
}

impl ScriptedBlob {
    pub(super) fn from_asset(asset: Arc<RuntimeBlobAsset>) -> Self {
        Self(asset)
    }

    pub(super) fn asset(&self) -> Arc<RuntimeBlobAsset> {
        Arc::clone(&self.0)
    }
}

/// File-owned BlobAsset values visible to `Context:blob`.
///
/// Values are retained independently of the VM stack, matching C++'s
/// `ScriptedBlob::asset` reference-counted FileAsset owner.
#[derive(Clone, Default)]
pub(super) struct ScriptedBlobAssets {
    assets: Rc<RefCell<BTreeMap<String, Vec<Arc<RuntimeBlobAsset>>>>>,
}

impl ScriptedBlobAssets {
    pub(super) fn install(lua: &Lua) -> Self {
        let assets = Self::default();
        lua.set_app_data(assets.clone());
        assets
    }

    pub(super) fn register(&self, name: &str, bytes: &[u8]) -> Result<()> {
        self.assets
            .borrow_mut()
            .entry(name.to_owned())
            .or_default()
            .push(Arc::new(RuntimeBlobAsset::new(name, Arc::from(bytes))));
        Ok(())
    }

    pub(super) fn lookup(lua: &Lua, name: &str) -> Result<Value> {
        let Some(assets) = lua.app_data_ref::<Self>().map(|assets| assets.clone()) else {
            return Ok(Value::Nil);
        };
        let asset = assets
            .assets
            .borrow()
            .get(name)
            .and_then(|matches| matches.iter().find(|asset| !asset.bytes().is_empty()))
            .cloned();
        match asset {
            // Pinned Context:blob deliberately rejects zero-byte BlobAssets.
            Some(asset) => lua
                .create_userdata(ScriptedBlob(asset))
                .map(Value::UserData),
            _ => Ok(Value::Nil),
        }
    }
}
