@set ASEPRITE="C:\Program Files (x86)\Steam\steamapps\common\Aseprite\Aseprite.exe"
%ASEPRITE%  -b  .\maps\map.0.0.aseprite --script .\export-aseprite-file\export.lua
move map.0.0 maps
%ASEPRITE%  -b  .\maps\map.0.0.aseprite --sheet .\maps\map.0.0\preview_map.0.0.png

%ASEPRITE%  -b  .\maps\map.1.0.aseprite --script .\export-aseprite-file\export.lua
move map.1.0 maps
%ASEPRITE%  -b  .\maps\map.1.0.aseprite --sheet .\maps\map.1.0\preview_map.1.0.png