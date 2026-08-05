-- Resolve C++ dependencies to the exact revisions the pinned librive was built
-- from, for the tools that link that archive (tools/golden-runner,
-- tools/cpp-probe). Those tools are the oracle for C++ differential testing, so
-- compiling them against a different revision than the archive they link is an
-- ABI/behavior mismatch hidden inside the thing that decides what "correct"
-- means.
--
-- yoga is the sharpest case: rive/layout/layout_data.hpp holds a YGNode and a
-- YGStyle by value and is reachable from rive/shapes/shape.hpp, so a yoga skew
-- changes the size and field offsets of every rive object embedding LayoutData.
-- Measured at pin 4ac7b327, rive_changes_v2_0_1_2_grid is wider than its
-- non-grid sibling -- sizeof(YGStyle) 204 -> 224, sizeof(YGNode) 632 -> 704.
--
-- Upstream fetches dependencies through build/dependency.lua's
-- `dependency.github(project, tag)`, which clones to <root>/<project>_<tag>
-- with '/' replaced by '_', where <root> is $DEPENDENCIES when set and
-- <runtime>/dependencies otherwise.
--
-- Globbing that root cannot express "the revision the pin builds": several
-- projects have more than one tag checked out side by side --
-- rive-app_yoga_rive_changes_v2_0_1_2 next to ..._v2_0_1_2_grid, three
-- luigi-rosso_luau_* revisions -- and `os.matchdirs` hands back whichever sorts
-- first, which is the wrong one in both cases. Read the tag out of the
-- runtime's own premake files instead and build the exact path, so resolution
-- follows the pin when it rolls.

local m = {}

local function read_file(file_path)
    local handle = io.open(file_path, 'r')
    if handle == nil then
        return nil
    end
    local contents = handle:read('*a')
    handle:close()
    return contents
end

local function first_dir(pattern)
    local matches = os.matchdirs(pattern)
    if #matches > 0 then
        return matches[1]
    end
    return nil
end

-- The tag the pinned runtime's own premake asks `dependency.github` for.
local function pinned_tag(rive_runtime, premake_file, project)
    local contents = read_file(rive_runtime .. '/' .. premake_file)
    if contents == nil then
        return nil
    end
    return contents:match(
        "dependency%.github%(%s*'" .. project:gsub('%W', '%%%0') .. "'%s*,%s*'([^']+)'"
    )
end

-- `tool` names the caller in error messages. `dep_root` is the dependency root
-- the caller resolves against; pass $DEPENDENCIES when the caller honors it,
-- nil for the upstream default. Tags are always read from `rive_runtime`, since
-- the revision is a property of the pin rather than of where it was cloned.
function m.resolver(tool, rive_runtime, dep_root)
    local root = dep_root or (rive_runtime .. '/dependencies')
    -- The pre-migration dependencies/<host>/cache/<hash>/<name>-<tag> tree
    -- survives only in checkouts old enough to predate `dependency.github`
    -- cloning to <root>/<project>_<tag>, and it holds the tags current at that
    -- time (harfbuzz 10.1.0 against the pin's 13.1.1, yoga rive_changes_v2_0_1
    -- against rive_changes_v2_0_1_2_grid). It is a last resort for when the tag
    -- cannot be read at all, never a stand-in for a known tag.
    local dep_cache = rive_runtime .. '/dependencies/' .. os.host() .. '/cache'

    local self = { dep_root = root, dep_cache = dep_cache }

    -- Directory holding `project` at the revision `premake_file` pins it to.
    -- `legacy_pattern` is an optional suffix glob under the legacy cache tree.
    function self.dir(project, premake_file, legacy_pattern)
        local tag = pinned_tag(rive_runtime, premake_file, project)
        if tag ~= nil then
            local dirname = (project .. '_' .. tag):gsub('/', '_')
            local dir = root .. '/' .. dirname
            if os.isdir(dir) then
                return dir
            end
            error(
                tool
                    .. ': the pinned runtime builds '
                    .. project
                    .. ' at '
                    .. tag
                    .. ', but '
                    .. dir
                    .. ' does not exist. Build librive first so the dependency is'
                    .. ' fetched; the legacy '
                    .. dep_cache
                    .. ' tree holds a different revision and is not a substitute.'
            )
        end
        local legacy = legacy_pattern and first_dir(dep_cache .. legacy_pattern)
        if legacy ~= nil then
            return legacy
        end
        -- A silent nil drops the include directory, which resurfaces either as
        -- a "file not found" from deep inside a rive header or -- worse -- as a
        -- tool that compiles clean against headers librive was never built with.
        error(
            tool
                .. ': cannot determine the '
                .. project
                .. ' revision the pinned librive was built with; no'
                .. " dependency.github('"
                .. project
                .. "', ...) in "
                .. rive_runtime
                .. '/'
                .. premake_file
                .. (legacy_pattern and ', and nothing matching ' .. dep_cache .. legacy_pattern or '')
        )
    end

    return self
end

return m
