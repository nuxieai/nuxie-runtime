workspace('rive_rust_gm_stream_capture')
configurations({ 'debug', 'release' })

local rive_runtime = os.getenv('RIVE_RUNTIME_DIR') or '/Users/levi/dev/oss/rive-runtime'
local dep_cache = rive_runtime .. '/dependencies/' .. os.host() .. '/cache'
local dep_root = os.getenv('DEPENDENCIES') or (rive_runtime .. '/dependencies')

local rive_dependencies =
    dofile(path.getabsolute(_SCRIPT_DIR .. '/../../build-support/rive_dependencies.lua'))
local deps = rive_dependencies.resolver('gm-stream-capture', rive_runtime, dep_root)

-- Set by build.sh to the provenance-bound librive built at the pin
-- (tools/build-support/pinned-librive.sh). Without it the link would fall
-- through to whatever the shared pinned checkout last built into tests/out,
-- which is not bound to the pin at all.
local runtime_libdir = os.getenv('RIVE_GM_CAPTURE_RUNTIME_LIBDIR')

local include_dirs = {
    '..',
    '../../golden-runner',
    rive_runtime .. '/include',
    rive_runtime .. '/renderer/include',
    rive_runtime .. '/tests',
    rive_runtime .. '/tests/gm',
    rive_runtime .. '/tests/include',
    rive_runtime .. '/tests/unit_tests',
    -- librive is compiled with rive_{harfbuzz,yoga}_renames.h force-included
    -- from the dependencies root; the capture tool compiles against the same
    -- config, so that root must be searchable here too.
    rive_runtime .. '/dependencies',
}

local gm_files = {}
for file in io.lines('gm-files.txt') do
    table.insert(gm_files, file)
end

-- Resolved from the pin's own dependency declarations rather than globbed out
-- of the legacy cache: yoga's revision is an ABI decision here, because
-- rive/layout/layout_data.hpp holds YGNode/YGStyle by value and is reachable
-- from rive/shapes/shape.hpp.
for _, dir in ipairs({
    deps.dir('harfbuzz', 'premake5_harfbuzz_v2.lua', dep_cache .. '/*/harfbuzz-*/src', 'src'),
    deps.dir('SheenBidi', 'premake5_sheenbidi_v2.lua', dep_cache .. '/*/SheenBidi-*/Headers', 'Headers'),
    deps.dir('yoga', 'premake5_yoga_v2.lua', dep_cache .. '/*/yoga-*'),
}) do
    table.insert(include_dirs, dir)
end

project('gm_stream_capture')
kind('ConsoleApp')
language('C++')
cppdialect('C++17')
targetdir('%{cfg.system}/bin/%{cfg.buildcfg}')
objdir('%{cfg.system}/obj/%{cfg.buildcfg}')
includedirs(include_dirs)
defines({
    '_RIVE_INTERNAL_',
    'WITH_RIVE_TEXT',
    'WITH_RIVE_LAYOUT',
    'RIVE_MACOSX',
    'RIVE_TOOLS_NO_GL',
    'YOGA_EXPORT=',
})
-- Match librive's own force-includes. Without these the tool references
-- unrenamed YG*/hb_* symbols while the archive exports rive_-prefixed ones.
forceincludes({
    'rive_harfbuzz_renames.h',
    'rive_yoga_renames.h',
})
files({
    '../main.cpp',
    '../../golden-runner/recording_renderer.cpp',
    rive_runtime .. '/tests/gm/gm.cpp',
    rive_runtime .. '/tests/gm/gmutils.cpp',
    rive_runtime .. '/tests/unit_tests/assets/batdude.png.cpp',
    rive_runtime .. '/tests/unit_tests/assets/montserrat.ttf.cpp',
    rive_runtime .. '/tests/unit_tests/assets/nomoon.png.cpp',
    rive_runtime .. '/tests/unit_tests/assets/roboto_flex.ttf.cpp',
})
files(gm_files)
if not runtime_libdir then
    error(
        'gm-stream-capture: RIVE_GM_CAPTURE_RUNTIME_LIBDIR is unset. Build via tools/gm-stream-capture/build.sh, which builds the provenance-bound librive at the pin and exports it.',
        0
    )
end
-- Deliberately the only libdir: an unresolved archive must fail the link
-- rather than silently fall back to an unpinned build tree.
libdirs({ runtime_libdir })
links({
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
})
buildoptions({ '-Wall', '-fno-rtti', '-g' })

filter('configurations:debug')
defines({ 'DEBUG' })
symbols('On')

filter('configurations:release')
defines({ 'RELEASE', 'NDEBUG' })
optimize('On')
