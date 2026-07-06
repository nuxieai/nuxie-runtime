workspace('rive_rust_golden_runner')
configurations({ 'debug', 'release' })

local rive_runtime = os.getenv('RIVE_RUNTIME_DIR') or '/Users/levi/dev/oss/rive-runtime'
local dep_cache = rive_runtime .. '/dependencies/' .. os.host() .. '/cache'

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

project('rive_golden_runner')
kind('ConsoleApp')
language('C++')
cppdialect('C++17')
targetdir('%{cfg.system}/bin/%{cfg.buildcfg}')
objdir('%{cfg.system}/obj/%{cfg.buildcfg}')
includedirs(include_dirs)
-- Must match the defines librive was built with (see
-- $RIVE_RUNTIME_DIR/out/debug/rive.make): several rive headers declare
-- virtual member functions and data members conditionally on these, so
-- compiling our translation units with different defines than the library
-- would silently break the ABI (vtable layouts / object sizes).
defines({
    '_RIVE_INTERNAL_',
    'WITH_RIVE_TEXT',
    'WITH_RIVE_LAYOUT',
    'RIVE_MACOSX',
    'YOGA_EXPORT=',
})

files({
    '../main.cpp',
    '../recording_renderer.cpp',
})

libdirs({
    rive_runtime .. '/tests/out/%{cfg.buildcfg}',
    rive_runtime .. '/out/%{cfg.buildcfg}',
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
