//! Direct owner for pinned `src/lua/renderer/lua_image.cpp`.

use std::rc::Rc;
use std::sync::Arc;

use luaur_rt::{AnyUserData, Error, Lua, Result, UserData, UserDataFields};
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
pub(super) struct ScriptedImage {
    image: Rc<dyn RenderImage>,
}

impl ScriptedImage {
    pub(super) fn from_asset(lua: &Lua, identity: ScriptImage) -> Option<Self> {
        let asset_owners = ScriptedImageAssetOwners::for_lua(lua)?;
        let image = asset_owners.get(identity.asset_global_id())?;
        Some(Self { image })
    }

    /// Construction seam used by factory-backed image producers. It does not
    /// decode or schedule work; the caller transfers an already-made image.
    pub(super) fn from_render_image(image: Box<dyn RenderImage>) -> Self {
        Self::from_render_image_rc(Rc::from(image))
    }

    pub(super) fn from_render_image_rc(image: Rc<dyn RenderImage>) -> Self {
        Self { image }
    }

    pub(super) fn render_image(&self) -> Result<Rc<dyn RenderImage>> {
        Ok(Rc::clone(&self.image))
    }

    pub(super) fn with_render_image<R>(
        &self,
        callback: impl FnOnce(&dyn RenderImage) -> R,
    ) -> Result<R> {
        let image = self.render_image()?;
        Ok(callback(image.as_ref()))
    }
}

impl UserData for ScriptedImage {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("width", |_, this| {
            this.with_render_image(RenderImage::width)
        });
        fields.add_field_method_get("height", |_, this| {
            this.with_render_image(RenderImage::height)
        });
    }
}

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
    ScriptedImage::from_asset(lua, image)
        .map(|image| lua.create_userdata(image))
        .transpose()
}

#[cfg(all(test, feature = "compiler"))]
mod tests {
    use super::*;
    use nuxie_render_api::Mat2D;

    struct TestImage;

    impl RenderImage for TestImage {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn width(&self) -> u32 {
            7
        }

        fn height(&self) -> u32 {
            11
        }

        fn uv_transform(&self) -> Mat2D {
            Mat2D::IDENTITY
        }
    }

    #[test]
    fn image_width_and_height_are_read_only_numeric_members() {
        let lua = Lua::new();
        let image = lua
            .create_userdata(ScriptedImage::from_render_image(Box::new(TestImage)))
            .unwrap();
        lua.globals().set("image", image).unwrap();

        assert_eq!(lua.load("return image.width").eval::<u32>().unwrap(), 7);
        assert_eq!(lua.load("return image.height").eval::<u32>().unwrap(), 11);
        assert!(lua.load("image.width = 3").exec().is_err());
        assert!(lua.load("image.height = 5").exec().is_err());
        assert!(lua.load("return image.view == nil").eval::<bool>().unwrap());
    }
}
