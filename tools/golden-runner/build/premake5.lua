workspace('rive_rust_golden_runner')
configurations({ 'debug', 'release' })

local rive_runtime = os.getenv('RIVE_RUNTIME_DIR') or '/Users/levi/dev/oss/rive-runtime'
local dep_cache = rive_runtime .. '/dependencies/' .. os.host() .. '/cache'
local with_scripting = os.getenv('RIVE_GOLDEN_WITH_SCRIPTING') == '1'
local scripting_libdir = os.getenv('RIVE_GOLDEN_SCRIPTING_LIBDIR')
local decoders_libdir = os.getenv('RIVE_GOLDEN_DECODERS_LIBDIR')
local obj_suffix = with_scripting and '/scripting' or ''
local runner_name = os.getenv('RIVE_GOLDEN_RUNNER_NAME') or 'rive_golden_runner'

local function first_dir(pattern)
    local matches = os.matchdirs(pattern)
    if #matches > 0 then
        return matches[1]
    end
    return nil
end

local include_dirs = {
    '..',
    rive_runtime .. '/include',
    rive_runtime .. '/test',
    rive_runtime .. '/tests/include',
    '/usr/local/include',
    '/usr/include',
}

local harfbuzz = first_dir(rive_runtime .. '/dependencies/rive-app_harfbuzz_*/src') or
    first_dir(dep_cache .. '/*/harfbuzz-*/src')
local sheenbidi = first_dir(rive_runtime .. '/dependencies/Tehreer_SheenBidi_*/Headers') or
    first_dir(dep_cache .. '/*/SheenBidi-*/Headers')
local yoga = first_dir(rive_runtime .. '/dependencies/rive-app_yoga_*') or
    first_dir(dep_cache .. '/*/yoga-*')
local miniaudio = first_dir(rive_runtime .. '/dependencies/rive-app_miniaudio_*') or
    first_dir(dep_cache .. '/*/miniaudio-*')
local luau = first_dir(rive_runtime .. '/dependencies/luigi-rosso_luau_*')
local libhydrogen = first_dir(rive_runtime .. '/dependencies/luigi-rosso_libhydrogen_*')

if harfbuzz then
    table.insert(include_dirs, harfbuzz)
end
if sheenbidi then
    table.insert(include_dirs, sheenbidi)
end
if yoga then
    table.insert(include_dirs, yoga)
end
if miniaudio then
    table.insert(include_dirs, miniaudio)
end
if with_scripting then
    table.insert(include_dirs, rive_runtime .. '/scripting')
    table.insert(include_dirs, rive_runtime .. '/decoders/include')
    if luau then
        table.insert(include_dirs, luau .. '/VM/include')
    end
    if libhydrogen then
        table.insert(include_dirs, libhydrogen)
    end
end

local runner_defines = {
    '_RIVE_INTERNAL_',
    'WITH_RIVE_TEXT',
    'WITH_RIVE_LAYOUT',
    'RIVE_MACOSX',
    'YOGA_EXPORT=',
}

if with_scripting then
    table.insert(runner_defines, 'WITH_RIVE_SCRIPTING')
    table.insert(runner_defines, 'RIVE_DECODERS')
    table.insert(runner_defines, 'HYDRO_SIGN_VERIFY_ONLY=1')
end

local lib_dirs = {
    rive_runtime .. '/tests/out/%{cfg.buildcfg}',
    rive_runtime .. '/out/%{cfg.buildcfg}',
    rive_runtime .. '/build/%{cfg.system}/bin/%{cfg.buildcfg}',
    dep_cache .. '/bin/%{cfg.buildcfg}',
    '/usr/local/lib',
    '/usr/lib',
}

if with_scripting and scripting_libdir then
    table.insert(lib_dirs, 1, scripting_libdir)
end
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
if with_scripting then
    forceincludes({ 'rive_luau.hpp' })
end

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
