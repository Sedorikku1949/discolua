local pandora_inspect = require("test.pandora")
local json = require "json"

function inspect(obj, compact)
    print(pandora_inspect(obj, { depth = 2, compact = compact or false, colors = true }))
end

function script_path()
    local str = debug.getinfo(2, "S").source:sub(2)
    return str:match("(.*/)")
end

local function read_file(path)
    local file = io.open(path, "rb") -- r read mode and b binary mode
    if not file then return nil end
    local content = file:read "*a" -- *a or *all reads the whole file
    file:close()
    return content
end

local config = json.decode(read_file(script_path() .. "/config.json"))


local Discord = require("discolua")

local client = Discord.Client:new()

print(client)

client:login(config.token)

print('         client ready')