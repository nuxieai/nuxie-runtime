//! Direct owner for pinned `src/lua/renderer/lua_image.cpp`.

use std::rc::Rc;
use std::sync::Arc;

use luaur_rt::{AnyUserData, Error, Lua, Result, UserData};
use nuxie_render_api::{ImageFilter, ImageSampler, ImageWrap, RenderImage};
use nuxie_runtime::{RuntimeImageAssetOwners, ScriptImage};

#[derive(Clone, Default)]
struct ScriptedImageAssetOwners(Option<Arc<RuntimeImageAssetOwners>>);

impl ScriptedImageAssetOwners {
    fn install(lua: &Lua) {
        if lua.app_data_ref::<Self>().is_none() {
            lua.set_app_data(Self::default());
        }
    }

    fn set(lua: &Lua, owners: Arc<RuntimeImageAssetOwners>) {
        lua.set_app_data(Self(Some(owners)));
    }

    fn for_lua(lua: &Lua) -> Option<Arc<RuntimeImageAssetOwners>> {
        lua.app_data_ref::<Self>()
            .and_then(|owners| owners.0.clone())
    }
}

/// One Lua Image wrapper, whether sourced from a File ImageAsset identity or
/// an already-created render-factory image.
///
/// The high-level Rust file path currently initializes scripts before its
/// lazy image decode. Keep the asset identity until that separately-owned
/// schedule publishes a resource; this module never initiates decoding.
pub(super) struct ScriptedImage {
    image: Option<Rc<dyn RenderImage>>,
    asset_identity: Option<ScriptImage>,
    asset_owners: Option<Arc<RuntimeImageAssetOwners>>,
}

impl ScriptedImage {
    pub(super) fn from_asset(lua: &Lua, identity: ScriptImage) -> Self {
        Self {
            image: None,
            asset_identity: Some(identity),
            asset_owners: ScriptedImageAssetOwners::for_lua(lua),
        }
    }

    /// Construction seam used by factory-backed image producers. It does not
    /// decode or schedule work; the caller transfers an already-made image.
    #[allow(dead_code)] // exercised here and consumed by the concurrent decode lane after merge
    pub(super) fn from_render_image(image: Box<dyn RenderImage>) -> Self {
        Self {
            image: Some(Rc::from(image)),
            asset_identity: None,
            asset_owners: None,
        }
    }

    pub(super) fn asset_identity(&self) -> Option<ScriptImage> {
        self.asset_identity
    }

    pub(super) fn with_render_image<R>(
        &self,
        callback: impl FnOnce(&dyn RenderImage) -> R,
    ) -> Result<R> {
        if let Some(image) = self.image.as_deref() {
            return Ok(callback(image));
        }
        let identity = self
            .asset_identity
            .ok_or_else(|| Error::runtime("Image has no render resource"))?;
        let image = self
            .asset_owners
            .as_ref()
            .and_then(|owners| owners.get(identity.asset_global_id()))
            .ok_or_else(|| Error::runtime("Image asset has not been decoded"))?;
        Ok(callback(image.as_ref()))
    }
}

impl UserData for ScriptedImage {}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ScriptedImageSampler(pub(crate) ImageSampler);

impl UserData for ScriptedImageSampler {}

pub(super) fn install_image_globals(lua: &Lua) -> Result<()> {
    ScriptedImageAssetOwners::install(lua);
    lua.globals().set(
        "ImageSampler",
        lua.create_function(|lua, (wrap_x, wrap_y, filter): (String, String, String)| {
            let parse_wrap = |value: &str| match value {
                "clamp" => Ok(ImageWrap::Clamp),
                "repeat" => Ok(ImageWrap::Repeat),
                "mirror" => Ok(ImageWrap::Mirror),
                other => Err(Error::runtime(format!(
                    "'{other}' is not a valid ImageWrap"
                ))),
            };
            let filter = match filter.as_str() {
                "bilinear" => ImageFilter::Bilinear,
                "nearest" => ImageFilter::Nearest,
                other => {
                    return Err(Error::runtime(format!(
                        "'{other}' is not a valid ImageFilter"
                    )));
                }
            };
            lua.create_userdata(ScriptedImageSampler(ImageSampler {
                wrap_x: parse_wrap(&wrap_x)?,
                wrap_y: parse_wrap(&wrap_y)?,
                filter,
            }))
        })?,
    )?;
    Ok(())
}

pub(super) fn set_image_asset_owners(lua: &Lua, owners: Arc<RuntimeImageAssetOwners>) {
    ScriptedImageAssetOwners::set(lua, owners);
}

pub(super) fn create_asset_image(lua: &Lua, image: ScriptImage) -> Result<Option<AnyUserData>> {
    lua.create_userdata(ScriptedImage::from_asset(lua, image))
        .map(Some)
}
