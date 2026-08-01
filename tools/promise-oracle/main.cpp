#include "lua.h"
#include "lualib.h"
#include "rive/lua/rive_lua_libs.hpp"

#include <cstring>
#include <iomanip>
#include <iostream>
#include <iterator>
#include <string>

int main()
{
    const std::string bytecode(std::istreambuf_iterator<char>(std::cin), {});
    if (bytecode.empty())
    {
        std::cerr << "promise oracle expected Luau bytecode on stdin\n";
        return 2;
    }

    lua_State* state = luaL_newstate();
    luaL_openlibs(state);
    lua_callbacks(state)->useratom =
        [](lua_State*, const char* name, size_t length) -> int16_t {
        struct Atom
        {
            const char* name;
            rive::LuaAtoms value;
        };
        static constexpr Atom atoms[] = {
            {"andThen", rive::LuaAtoms::andThen},
            {"catch", rive::LuaAtoms::catch_},
            {"finally", rive::LuaAtoms::finally_},
            {"cancel", rive::LuaAtoms::cancel},
            {"onCancel", rive::LuaAtoms::onCancel},
            {"getStatus", rive::LuaAtoms::getStatus},
        };
        for (const auto& atom : atoms)
        {
            if (std::strlen(atom.name) == length &&
                std::memcmp(atom.name, name, length) == 0)
                return static_cast<int16_t>(atom.value);
        }
        return -1;
    };
    rive::luaopen_rive_promise(state);

    int status = luau_load(state,
                           "promise_scenario",
                           bytecode.data(),
                           bytecode.size(),
                           0);
    if (status == LUA_OK)
        status = lua_pcall(state, 0, 1, 0);
    if (status != LUA_OK)
    {
        const char* error = lua_tostring(state, -1);
        std::cerr << (error == nullptr ? "unknown Luau error" : error) << '\n';
        lua_close(state);
        return 1;
    }

    switch (lua_type(state, -1))
    {
        case LUA_TNIL:
            std::cout << "nil";
            break;
        case LUA_TBOOLEAN:
            std::cout << (lua_toboolean(state, -1) ? "true" : "false");
            break;
        case LUA_TNUMBER:
            std::cout << std::setprecision(17) << lua_tonumber(state, -1);
            break;
        case LUA_TSTRING:
            std::cout << lua_tostring(state, -1);
            break;
        default:
            std::cerr << "promise oracle scenario returned unsupported type "
                      << lua_typename(state, lua_type(state, -1)) << '\n';
            lua_close(state);
            return 1;
    }

    lua_close(state);
    return 0;
}
