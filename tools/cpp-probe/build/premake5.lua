workspace('rive_rust_cpp_probe')
configurations({ 'debug', 'release' })

local rive_runtime = os.getenv('RIVE_RUNTIME_DIR') or '/Users/levi/dev/oss/rive-runtime'
local runtime_libdir = os.getenv('RIVE_CPP_PROBE_RUNTIME_LIBDIR')
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

project('rive_cpp_probe')
kind('ConsoleApp')
language('C++')
cppdialect('C++17')
targetdir('%{cfg.system}/bin/%{cfg.buildcfg}')
objdir('%{cfg.system}/obj/%{cfg.buildcfg}')
includedirs(include_dirs)
defines({ '_RIVE_INTERNAL_' })

files({
    '../main.cpp',
    '../testing_random_provider.cpp',
    rive_runtime .. '/utils/no_op_factory.cpp',
})

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

if os.host() == 'macosx' then
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
else
    links({
        'rive',
        'rive_harfbuzz',
        'rive_sheenbidi',
        'rive_yoga',
        'm',
        'z',
        'dl',
    })
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
