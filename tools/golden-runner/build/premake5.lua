workspace('rive_rust_golden_runner')
configurations({ 'debug', 'release' })

local rive_runtime = os.getenv('RIVE_RUNTIME_DIR') or '/Users/levi/dev/oss/rive-runtime'
-- Upstream's build/dependency.lua clones into $DEPENDENCIES when it is set and
-- <runtime>/dependencies otherwise; resolve against the same root librive did.
local dep_root = os.getenv('DEPENDENCIES') or (rive_runtime .. '/dependencies')
local with_scripting = os.getenv('RIVE_GOLDEN_WITH_SCRIPTING') == '1'
local runtime_libdir = os.getenv('RIVE_GOLDEN_RUNTIME_LIBDIR') or
    (rive_runtime .. '/out/%{cfg.buildcfg}')
local decoders_libdir = os.getenv('RIVE_GOLDEN_DECODERS_LIBDIR')
local obj_suffix = with_scripting and '/scripting' or ''
local runner_name = os.getenv('RIVE_GOLDEN_RUNNER_NAME') or 'rive_golden_runner'

-- Shared with tools/cpp-probe: resolve each dependency to the revision the
-- pinned runtime's own premake asks `dependency.github` for, rather than
-- globbing <root>/<project>_* and taking whatever `os.matchdirs` sorts first.
-- The glob picked rive-app_yoga_rive_changes_v2_0_1_2 over the pin's
-- ..._v2_0_1_2_grid and luau rive_0_36 over the pin's rive_0_732.
local dependencies = dofile(
    path.getabsolute(_SCRIPT_DIR .. '/../../build-support/rive_dependencies.lua')
).resolver('golden-runner', rive_runtime, dep_root)

local include_dirs = {
    '..',
    rive_runtime .. '/include',
    rive_runtime .. '/test',
    rive_runtime .. '/tests/include',
    '/usr/local/include',
    '/usr/include',
}
-- The pinned runtime's build config force-includes rive_yoga_renames.h
-- (and siblings) from its dependencies root; the runner compiles against
-- the same config, so that root must be searchable here too. Those generated
-- headers live in the runtime's own tree even when $DEPENDENCIES relocates the
-- fetched clones, so search both when they differ.
table.insert(include_dirs, dep_root)
if dep_root ~= rive_runtime .. '/dependencies' then
    table.insert(include_dirs, rive_runtime .. '/dependencies')
end

local harfbuzz =
    dependencies.dir('rive-app/harfbuzz', 'dependencies/premake5_harfbuzz_v2.lua', '/*/harfbuzz-*')
local sheenbidi =
    dependencies.dir('Tehreer/SheenBidi', 'dependencies/premake5_sheenbidi_v2.lua', '/*/SheenBidi-*')
-- Required, not decorative: rive/layout/layout_data.hpp holds a YGNode and a
-- YGStyle by value, so every runner translation unit that reaches shape.hpp
-- takes its layout sizes and offsets from these headers. RIVE_GOLDEN_YOGA_DIR
-- stays available for deliberately building the runner against a different
-- yoga revision than the pin, which is how a yoga skew gets ruled in or out.
local yoga = os.getenv('RIVE_GOLDEN_YOGA_DIR')
    or dependencies.dir('rive-app/yoga', 'dependencies/premake5_yoga_v2.lua', '/*/yoga-*')

table.insert(include_dirs, harfbuzz .. '/src')
table.insert(include_dirs, sheenbidi .. '/Headers')
table.insert(include_dirs, yoga)
if with_scripting then
    -- The runner links luau_vm out of librive, so a revision skew here means
    -- headers describing one Luau's VM in front of another Luau's objects.
    local luau = os.getenv('RIVE_GOLDEN_LUAU_DIR')
        or dependencies.dir('luigi-rosso/luau', 'scripting/premake5.lua')
    local libhydrogen = dependencies.dir('luigi-rosso/libhydrogen', 'scripting/premake5.lua')
    -- build.sh builds scripted librive with --with_rive_audio=external and
    -- links miniaudio, so the dependency is fetched in this mode. An ordinary
    -- build never asks for audio, and a clean checkout has no miniaudio clone
    -- to resolve -- requiring it there would fail a build that does not use it.
    local miniaudio = dependencies.dir(
        'rive-app/miniaudio',
        'dependencies/premake5_miniaudio_v2.lua',
        '/*/miniaudio-*'
    )
    table.insert(include_dirs, miniaudio)
    table.insert(include_dirs, rive_runtime .. '/scripting')
    table.insert(include_dirs, rive_runtime .. '/decoders/include')
    table.insert(include_dirs, luau .. '/VM/include')
    table.insert(include_dirs, libhydrogen)
end

local runner_defines = {
    '_RIVE_INTERNAL_',
    'WITH_RIVE_TEXT',
    'WITH_RIVE_LAYOUT',
    'RIVE_MACOSX',
    'YOGA_EXPORT=',
}
local runner_forceincludes = {
    'rive_yoga_renames.h',
}

if with_scripting then
    table.insert(runner_defines, 'WITH_RIVE_SCRIPTING')
    table.insert(runner_defines, 'RIVE_DECODERS')
    table.insert(runner_defines, 'HYDRO_SIGN_VERIFY_ONLY=1')
    table.insert(runner_forceincludes, 'rive_luau.hpp')
end

local lib_dirs = {
    runtime_libdir,
    rive_runtime .. '/build/%{cfg.system}/bin/%{cfg.buildcfg}',
    dependencies.dep_cache .. '/bin/%{cfg.buildcfg}',
    '/usr/local/lib',
    '/usr/lib',
}

if with_scripting and decoders_libdir then
    table.insert(lib_dirs, 1, decoders_libdir)
end

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

local unix_links = {
    'rive',
    'rive_harfbuzz',
    'rive_sheenbidi',
    'rive_yoga',
    'm',
    'z',
    'dl',
}

if with_scripting then
    table.insert(mac_links, 2, 'miniaudio')
    table.insert(unix_links, 2, 'miniaudio')
    table.insert(mac_links, 2, 'rive_decoders')
    table.insert(mac_links, 3, 'luau_vm')
    table.insert(mac_links, 4, 'libpng')
    table.insert(mac_links, 5, 'zlib')
    table.insert(mac_links, 6, 'libjpeg')
    table.insert(mac_links, 7, 'libwebp')
    table.insert(unix_links, 2, 'rive_decoders')
    table.insert(unix_links, 3, 'luau_vm')
    table.insert(unix_links, 4, 'libpng')
    table.insert(unix_links, 5, 'zlib')
    table.insert(unix_links, 6, 'libjpeg')
    table.insert(unix_links, 7, 'libwebp')
end

project('rive_golden_runner')
kind('ConsoleApp')
language('C++')
cppdialect('C++17')
targetname(runner_name)
targetdir('%{cfg.system}/bin/%{cfg.buildcfg}')
objdir('%{cfg.system}/obj/%{cfg.buildcfg}' .. obj_suffix)
includedirs(include_dirs)
-- Must match the defines librive was built with (see
-- $RIVE_RUNTIME_DIR/out/debug/rive.make): several rive headers declare
-- virtual member functions and data members conditionally on these, so
-- compiling our translation units with different defines than the library
-- would silently break the ABI (vtable layouts / object sizes).
defines(runner_defines)
forceincludes(runner_forceincludes)

files({
    '../main.cpp',
    '../recording_renderer.cpp',
})

libdirs(lib_dirs)

if os.host() == 'macosx' then
    links(mac_links)
else
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
