workspace('rive_rust_gm_stream_capture')
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
    '../../golden-runner',
    rive_runtime .. '/include',
    rive_runtime .. '/renderer/include',
    rive_runtime .. '/tests',
    rive_runtime .. '/tests/gm',
    rive_runtime .. '/tests/include',
    rive_runtime .. '/tests/unit_tests',
}

local gm_files = {}
for file in io.lines('gm-files.txt') do
    table.insert(gm_files, file)
end

for _, dir in ipairs({
    first_dir(dep_cache .. '/*/harfbuzz-*/src'),
    first_dir(dep_cache .. '/*/SheenBidi-*/Headers'),
    first_dir(dep_cache .. '/*/yoga-*'),
}) do
    if dir then table.insert(include_dirs, dir) end
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
libdirs({
    rive_runtime .. '/tests/out/%{cfg.buildcfg}',
    rive_runtime .. '/out/%{cfg.buildcfg}',
    dep_cache .. '/bin/%{cfg.buildcfg}',
})
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
