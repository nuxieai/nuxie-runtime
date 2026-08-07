workspace('rive_rust_cpp_probe')
configurations({ 'debug', 'release' })

local rive_runtime = os.getenv('RIVE_RUNTIME_DIR') or '/Users/levi/dev/oss/rive-runtime'
local runtime_libdir = os.getenv('RIVE_CPP_PROBE_RUNTIME_LIBDIR')
local decoders_libdir = os.getenv('RIVE_CPP_PROBE_DECODERS_LIBDIR')
local with_scripting = os.getenv('RIVE_CPP_PROBE_WITH_SCRIPTING') == '1'
local with_audio = os.getenv('RIVE_CPP_PROBE_WITH_AUDIO') == '1'
local runner_name = os.getenv('RIVE_CPP_PROBE_RUNNER_NAME') or 'rive_cpp_probe'
if not runtime_libdir then
    error('RIVE_CPP_PROBE_RUNTIME_LIBDIR must name a provenance-verified archive directory')
end
local dep_root = rive_runtime .. '/dependencies'
local dep_cache = dep_root .. '/' .. os.host() .. '/cache'

local function first_dir(pattern)
    local matches = os.matchdirs(pattern)
    if #matches > 0 then
        return matches[1]
    end
    return nil
end

local function read_file(path)
    local handle = io.open(path, 'r')
    if handle == nil then
        return nil
    end
    local contents = handle:read('*a')
    handle:close()
    return contents
end

-- The tag the pinned runtime's own premake asks `dependency.github` for.
local function pinned_tag(premake_file, project)
    local contents = read_file(rive_runtime .. '/' .. premake_file)
    if contents == nil then
        return nil
    end
    return contents:match(
        "dependency%.github%(%s*'" .. project:gsub('%W', '%%%0') .. "'%s*,%s*'([^']+)'"
    )
end

-- Resolve a C++ dependency to the exact revision the pinned librive was built
-- from. The probe is the oracle for C++ differential testing, so compiling it
-- against a different revision than the archive it links is an ABI/behavior
-- mismatch hidden inside the thing that decides what "correct" means.
--
-- Upstream fetches dependencies through build/dependency.lua's
-- `dependency.github(project, tag)`, which clones to dependencies/<project>_<tag>
-- with '/' replaced by '_'. Read the tag out of the runtime's premake instead
-- of globbing that directory: several projects have more than one tag checked
-- out side by side -- rive-app_yoga_rive_changes_v2_0_1_2 next to
-- ..._v2_0_1_2_grid, three luigi-rosso_luau_* revisions -- and `os.matchdirs`
-- hands back whichever sorts first, which is the wrong one in both cases.
--
-- The pre-migration dependencies/<host>/cache/<hash>/<name>-<tag> tree survives
-- only in checkouts old enough to predate the migration, and it holds the tags
-- current at that time (harfbuzz 10.1.0 against the pin's 13.1.1, yoga
-- rive_changes_v2_0_1 against rive_changes_v2_0_1_2_grid). It is a last resort
-- for when the tag cannot be read at all, never a stand-in for a known tag.
local function dependency_dir(project, premake_file, legacy_pattern)
    local tag = pinned_tag(premake_file, project)
    if tag ~= nil then
        local dirname = (project .. '_' .. tag):gsub('/', '_')
        local dir = dep_root .. '/' .. dirname
        if os.isdir(dir) then
            return dir
        end
        error(
            'cpp-probe: the pinned runtime builds '
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
    -- A silent nil drops the include directory, which resurfaces either as a
    -- "file not found" from deep inside a rive header or -- worse -- as a
    -- probe that compiles clean against headers librive was never built with.
    error(
        'cpp-probe: cannot determine the '
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

local include_dirs = {
    rive_runtime .. '/include',
    rive_runtime .. '/test',
    rive_runtime .. '/tests/include',
    '/usr/local/include',
    '/usr/include',
}

local harfbuzz =
    dependency_dir('rive-app/harfbuzz', 'dependencies/premake5_harfbuzz_v2.lua', '/*/harfbuzz-*')
local sheenbidi =
    dependency_dir('Tehreer/SheenBidi', 'dependencies/premake5_sheenbidi_v2.lua', '/*/SheenBidi-*')
-- Required, not decorative: rive/layout/layout_data.hpp holds a YGNode and a
-- YGStyle by value, so every probe translation unit that reaches shape.hpp
-- takes its layout sizes and offsets from these headers.
local yoga = dependency_dir('rive-app/yoga', 'dependencies/premake5_yoga_v2.lua', '/*/yoga-*')
local luau
local libhydrogen

table.insert(include_dirs, harfbuzz .. '/src')
table.insert(include_dirs, sheenbidi .. '/Headers')
table.insert(include_dirs, yoga)
if with_audio then
    table.insert(
        include_dirs,
        dependency_dir(
            'rive-app/miniaudio',
            'dependencies/premake5_miniaudio_v2.lua',
            '/*/miniaudio-*'
        )
    )
end
if with_scripting then
    -- The scripted probe compiles Luau's own Compiler/Ast/Bytecode/Common
    -- sources and links luau_vm out of librive, so a revision skew here means
    -- one Luau compiling bytecode for another Luau's VM.
    luau = dependency_dir('luigi-rosso/luau', 'scripting/premake5.lua')
    libhydrogen = dependency_dir('luigi-rosso/libhydrogen', 'scripting/premake5.lua')
    table.insert(include_dirs, rive_runtime .. '/scripting')
    table.insert(include_dirs, rive_runtime .. '/decoders/include')
    table.insert(include_dirs, luau .. '/VM/include')
    table.insert(include_dirs, luau .. '/Compiler/include')
    table.insert(include_dirs, luau .. '/Bytecode/include')
    table.insert(include_dirs, luau .. '/Ast/include')
    table.insert(include_dirs, luau .. '/Common/include')
    table.insert(include_dirs, libhydrogen)
end

project('rive_cpp_probe')
kind('ConsoleApp')
language('C++')
cppdialect('C++17')
targetname(runner_name)
targetdir('%{cfg.system}/bin/%{cfg.buildcfg}')
objdir('%{cfg.system}/obj/%{cfg.buildcfg}' .. (with_scripting and '/scripting' or '/ordinary'))
includedirs(include_dirs)
defines({ '_RIVE_INTERNAL_', 'WITH_RIVE_TEXT', 'WITH_RIVE_LAYOUT', 'YOGA_EXPORT=' })
if os.host() == 'macosx' then
    defines({ 'RIVE_MACOSX' })
end
if with_audio then
    defines({ 'WITH_RIVE_AUDIO', 'EXTERNAL_RIVE_AUDIO_ENGINE', 'MA_NO_DEVICE_IO', 'MA_NO_RESOURCE_MANAGER' })
end
if with_scripting then
    defines({ 'WITH_RIVE_SCRIPTING', 'RIVE_DECODERS', 'HYDRO_SIGN_VERIFY_ONLY=1' })
    forceincludes({ 'rive_luau.hpp' })
end

files({
    '../main.cpp',
    '../testing_random_provider.cpp',
    rive_runtime .. '/utils/no_op_factory.cpp',
})
if with_scripting then
    files({
        luau .. '/Compiler/src/**.cpp',
        luau .. '/Bytecode/src/**.cpp',
        luau .. '/Ast/src/**.cpp',
        luau .. '/Common/src/**.cpp',
    })
    exceptionhandling('On')
end

libdirs({
    -- `build.sh` creates and verifies exactly one dedicated archive with the
    -- pinned revision, config, feature defines, compiler, and archive digest.
    -- Do not search generic root/tests outputs: they may carry an incompatible
    -- ABI while still producing superficially valid probe JSON.
    runtime_libdir,
    rive_runtime .. '/build/%{cfg.system}/bin/%{cfg.buildcfg}',
    dep_cache .. '/bin/%{cfg.buildcfg}',
    '/usr/local/lib',
    '/usr/lib',
})
if with_scripting and decoders_libdir then
    libdirs({ decoders_libdir })
end

if os.host() == 'macosx' then
    local mac_links = {
        'rive',
        'rive_harfbuzz',
        'rive_sheenbidi',
        'rive_yoga',
        'Cocoa.framework',
        'CoreFoundation.framework',
        'IOKit.framework',
        'Security.framework',
        'bz2',
        'iconv',
        'lzma',
        'z',
    }
    if with_audio then
        table.insert(mac_links, 2, 'miniaudio')
    end
    if with_scripting then
        table.insert(mac_links, 2, 'miniaudio')
        table.insert(mac_links, 2, 'rive_decoders')
        table.insert(mac_links, 3, 'luau_vm')
        table.insert(mac_links, 4, 'libpng')
        table.insert(mac_links, 5, 'zlib')
        table.insert(mac_links, 6, 'libjpeg')
        table.insert(mac_links, 7, 'libwebp')
    end
    links(mac_links)
else
    local unix_links = {
        'rive',
        'rive_harfbuzz',
        'rive_sheenbidi',
        'rive_yoga',
        'm',
        'z',
        'dl',
    }
    if with_audio then
        table.insert(unix_links, 2, 'miniaudio')
    end
    if with_scripting then
        table.insert(unix_links, 2, 'miniaudio')
        table.insert(unix_links, 2, 'rive_decoders')
        table.insert(unix_links, 3, 'luau_vm')
        table.insert(unix_links, 4, 'libpng')
        table.insert(unix_links, 5, 'zlib')
        table.insert(unix_links, 6, 'libjpeg')
        table.insert(unix_links, 7, 'libwebp')
    end
    links(unix_links)
end

buildoptions({ '-Wall', '-fno-rtti', '-g' })

filter('configurations:debug')
defines({ 'DEBUG' })
symbols('On')

filter('configurations:release')
defines({ 'RELEASE' })
defines({ 'NDEBUG' })
optimize('On')

newaction({
    trigger = 'clean',
    description = 'clean the build',
    execute = function()
        os.rmdir('./bin')
        os.rmdir('./obj')
        os.remove('Makefile')
        os.execute('rm -f *.make')
    end,
})
