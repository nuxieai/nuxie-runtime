-- Resolves the pinned rive-runtime's third-party dependency directories for
-- the C++ tools in this repo.
--
-- Upstream moved off the hashed `dependencies/<host>/cache/<hash>/<name>-<tag>`
-- tree and onto `dependency.github('<org>/<project>', '<tag>')`, which clones
-- into `<DEPENDENCIES>/<org>_<project>_<tag>` (see the pin's
-- build/dependency.lua). Tools that still glob the legacy cache compile against
-- whatever revision that tree happens to hold, and a fresh clone has no such
-- tree at all -- the glob then yields nil and the include dir is dropped
-- silently.
--
-- For yoga that is an ABI difference rather than a header-path difference:
-- `rive/layout/layout_data.hpp` stores `YGNode` and `YGStyle` by value and is
-- reachable from `rive/shapes/shape.hpp`, so a tool built against the wrong
-- yoga disagrees with the librive it links about `sizeof(LayoutData)`.
--
-- This module reads the tag out of the pinned runtime's own premake file, so a
-- tool's include path tracks the pin. The legacy cache is kept only as a
-- fallback for checkouts that predate the migration, and an unresolvable
-- dependency is a hard error instead of a silently dropped include dir.

local m = {}

local function first_dir(pattern)
    if pattern == nil then
        return nil
    end
    local matches = os.matchdirs(pattern)
    if #matches > 0 then
        return matches[1]
    end
    return nil
end

-- Returns the `dependency.github(spec, tag)` pair declared in `premake_path`
-- for `project`. `project` may be the full `<org>/<repo>` spec or just the
-- repo name.
local function read_github_dependency(premake_path, project)
    local handle = io.open(premake_path, 'r')
    if handle == nil then
        return nil, nil, "cannot read '" .. premake_path .. "'"
    end
    local contents = handle:read('*a')
    handle:close()

    local wanted = project:lower()
    for spec, tag in contents:gmatch(
        "dependency%.github%s*%(%s*['\"]([^'\"]+)['\"]%s*,%s*['\"]([^'\"]+)['\"]%s*%)"
    ) do
        local repo = spec:match('([^/]+)$')
        if spec:lower() == wanted or repo:lower() == wanted then
            return spec, tag, nil
        end
    end

    return nil,
        nil,
        "no dependency.github('.../" .. project .. "', <tag>) in '" .. premake_path .. "'"
end

-- resolver(tool, rive_runtime, dep_root)
--
--   tool         name used in error messages
--   rive_runtime path to the pinned rive-runtime checkout
--   dep_root     where dependency.github clones land; defaults to the
--                DEPENDENCIES override honored by the pin's build/dependency.lua,
--                then to <rive_runtime>/dependencies
--
-- The returned object exposes:
--
--   dir(project, premake_file, legacy_pattern[, subdir])
--
--   project        dependency repo name (or full '<org>/<repo>' spec)
--   premake_file   file under <rive_runtime>/dependencies declaring it
--   legacy_pattern os.matchdirs glob into the pre-migration cache tree, or nil;
--                  must already include `subdir`, since that tree nests
--                  differently
--   subdir         path under the cloned repo to place on the include path
--                  (e.g. 'src' for harfbuzz, 'Headers' for SheenBidi)
function m.resolver(tool, rive_runtime, dep_root)
    assert(rive_runtime, tool .. ': rive_runtime is required')
    dep_root = dep_root or os.getenv('DEPENDENCIES') or (rive_runtime .. '/dependencies')

    local resolver = {}

    function resolver.dir(project, premake_file, legacy_pattern, subdir)
        local premake_path = rive_runtime .. '/dependencies/' .. premake_file
        local spec, tag, read_err = read_github_dependency(premake_path, project)

        local pinned
        if spec ~= nil then
            -- Mirrors dependency.github's dirname: '<org>/<repo>_<tag>' with
            -- separators flattened.
            local dirname = (spec .. '_' .. tag):gsub('/', '_')
            pinned = dep_root .. '/' .. dirname
            if subdir then
                pinned = pinned .. '/' .. subdir
            end
            if os.isdir(pinned) then
                return pinned
            end
        end

        local legacy = first_dir(legacy_pattern)
        if legacy then
            print(
                ('%s: %s not found at pinned tag (%s); falling back to legacy cache %s')
                    :format(tool, project, tag or '<unknown>', legacy)
            )
            return legacy
        end

        local detail
        if spec == nil then
            detail = read_err
        else
            detail = "expected '" .. pinned .. "' (tag " .. tag .. ')'
        end
        error(
            ('%s: cannot resolve dependency %s from %s -- %s. Generate the pinned runtime once (premake5 gmake2 --file=premake5_v2.lua) so dependency.github clones it, or set DEPENDENCIES to a tree that has it.')
                :format(tool, project, rive_runtime, detail),
            0
        )
    end

    return resolver
end

return m
