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
local dep_cache = rive_runtime .. '/dependencies/' .. os.host() .. '/cache'

local function first_dir(pattern)
    local matches = os.matchdirs(pattern)
    if #matches > 0 then
        return matches[1]
    end
    return nil
end

local include_dirs = {
    rive_runtime .. '/include',
    rive_runtime .. '/test',
    rive_runtime .. '/tests/include',
    '/usr/local/include',
    '/usr/include',
}

local harfbuzz = first_dir(dep_cache .. '/*/harfbuzz-*/src')
local sheenbidi = first_dir(dep_cache .. '/*/SheenBidi-*/Headers')
local yoga = first_dir(dep_cache .. '/*/yoga-*')
local miniaudio = first_dir(dep_cache .. '/*/miniaudio-*')
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
        table.insert(include_dirs, luau .. '/Compiler/include')
        table.insert(include_dirs, luau .. '/Bytecode/include')
        table.insert(include_dirs, luau .. '/Ast/include')
        table.insert(include_dirs, luau .. '/Common/include')
    end
    if libhydrogen then
        table.insert(include_dirs, libhydrogen)
    end
end

project('rive_cpp_probe')
kind('ConsoleApp')
language('C++')
cppdialect('C++17')
targetname(runner_name)
targetdir('%{cfg.system}/bin/%{cfg.buildcfg}')
objdir('%{cfg.system}/obj/%{cfg.buildcfg}' .. (with_scripting and '/scripting' or '/ordinary'))
includedirs(include_dirs)
defines({ '_RIVE_INTERNAL_', 'WITH_RIVE_TEXT', 'WITH_RIVE_LAYOUT', 'RIVE_MACOSX', 'YOGA_EXPORT=' })
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
if with_scripting and luau then
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
