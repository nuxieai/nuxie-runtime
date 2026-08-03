//! Direct retained owner for scripted Font values.

use std::sync::Arc;

use luaur_rt::{AnyUserData, Lua, Result, UserData};
use nuxie_runtime::{RuntimeFontAssetOwners, ScriptFont};

#[derive(Clone, Default)]
struct ScriptedFontAssetOwners(Option<Arc<RuntimeFontAssetOwners>>);

impl ScriptedFontAssetOwners {
    fn set(lua: &Lua, owners: Arc<RuntimeFontAssetOwners>) {
        lua.set_app_data(Self(Some(owners)));
    }

    fn for_lua(lua: &Lua) -> Option<Arc<RuntimeFontAssetOwners>> {
        lua.app_data_ref::<Self>()
            .and_then(|owners| owners.0.clone())
    }
}

/// Lua's opaque Font extern type. The Arc is the Rust counterpart of the
/// upstream `rcp<Font>` member and runs its destructor with the userdata.
pub(super) struct ScriptedFont {
    font_bytes: Arc<[u8]>,
}

impl ScriptedFont {
    fn from_asset(lua: &Lua, font: ScriptFont) -> Option<Self> {
        let font_bytes = font.live_font_bytes_arc().cloned().or_else(|| {
            let asset_global_id = font.asset_global_id()?;
            ScriptedFontAssetOwners::for_lua(lua)?.get(asset_global_id)
        })?;
        Some(Self { font_bytes })
    }

    pub(super) fn font_bytes(&self) -> Arc<[u8]> {
        Arc::clone(&self.font_bytes)
    }
}

impl UserData for ScriptedFont {}

pub(super) fn set_font_asset_owners(lua: &Lua, owners: Arc<RuntimeFontAssetOwners>) {
    ScriptedFontAssetOwners::set(lua, owners);
}

pub(super) fn create_asset_font(lua: &Lua, font: ScriptFont) -> Result<Option<AnyUserData>> {
    ScriptedFont::from_asset(lua, font)
        .map(|font| lua.create_userdata(font))
        .transpose()
}
