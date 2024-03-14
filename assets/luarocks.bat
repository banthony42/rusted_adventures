@set LUAROCKS="C:\Program Files (x86)\Lua\5.1\luarocks.bat"
@set LUA="C:\Program Files (x86)\Lua\5.1\lua.exe"

%LUAROCKS% install lua-cjson


%LUAROCKS% show lua-cjson


@REM Run interpreter:
%LUA% -v
