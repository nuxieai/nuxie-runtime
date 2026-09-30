//! Scoped typed file-asset lookup from src/lua/lua_scripted_context.cpp.

use luaur_rt::Lua;
use nuxie_runtime::source::core::CoreHandle;
use nuxie_runtime::source::file::RuntimeFileWeakHandle;

/// Preserve file order and the first acceptable match at each rank. The
/// caller extracts only its requested asset type and decoded resource.
pub(super) fn find_file_asset<T>(
    lua: &Lua,
    name: &str,
    file: Option<RuntimeFileWeakHandle>,
    mut accept: impl FnMut(&CoreHandle) -> Option<(String, T)>,
) -> Option<T> {
    let file = file?.upgrade()?;
    let reference = super::lua_blob::ScopedAssetReference::new(lua, name);
    let assets = file.with_file(|file| file.assets().to_vec());
    let mut best_rank = 0;
    let mut found = None;
    for asset in assets {
        let Some((name, value)) = accept(&asset) else {
            continue;
        };
        let rank = reference.rank(&name, &name);
        if rank > best_rank {
            best_rank = rank;
            found = Some(value);
        }
    }
    found
}
