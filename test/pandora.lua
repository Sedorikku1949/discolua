local function strColorize(str, mark)
    if type(str) ~= "string" then return "", "argument must be a string" end
    local colors = {["&"]={bla=30, red=31, gre=32, yel=33, blu=34, mag=35, cya=36, whi=37, Bla=90, Red=91, Gre=92, Yel=93, Blu=94, Mag=95, Cya=96, Whi=97, res=0}, ["&&"]={bla=40, red=41, gre=42, yel=43, blu=44, mag=45, cya=46, whi=47, Bla=100, Red=101, Gre=102, Yel=103, Blu=104, Mag=105, Cya=106, Whi=107}}
    local markdown = {["_"] = 4, ["%*"] = 1, ["`"] = 9, ["~"] = 5, ['|'] = 8 }

    local code = "\x1b[%dm"
    local last

    for char in string.gmatch(str, "&&?%a%a%a") do
        local color, length = string.gsub(char, "&", "")
        if colors[length == 1 and "&" or "&&"][color] then
            local num = colors[length == 1 and "&" or "&&"][color]
            str = string.gsub(str, char, string.format(code, num), 1)
            last = color
        end
    end
    if mark == true then
        for key, c in pairs(markdown) do
            local state = 0
            local _, len = string.gsub(str, key, "")
            if len%2 == 1 then
                len = len - 1
            end
            for i=0, len do
                str = string.gsub(str, key, state == 0 and string.format(code, c) or string.format(code, c+20), 1)
                state = state == 0 and 1 or 0
            end
        end
    end

    if last == "res" then return str
    else return str .. string.format(code, 0) end
end
local function strDecolorize(str)
    if type(str) ~= "string" then return error("argument must be a string", 4) end
    str =  str:gsub("%[.-m", "")
    return str
end
local function testOption(str, types, base)
    local bool= false
    for _, ntype in ipairs(types) do
        if bool == true then break end
        if type(ntype) == "function" then
            bool, str = ntype(str)
        elseif type(ntype) == "string" then
            bool = type(str) == ntype
        end
    end
    for i, ntype in pairs(types) do
        if bool == true then break end
        if type(i) == "string" then
            if type(ntype) == "function" then
                if type(str) == i then
                    bool, str = ntype(str)
                else
                    bool = true
                end
            end
        end
    end
    if bool == false then
        return base
    end
    return str
end
local function inspect(item, options)
    local typeof = type(item)
    local str = ""
    options = options or {}
    if type(options) ~= "table" then error("second argument (options) must be a table", 4) end

    options.__inherit  = options.__inherit or {depth = -1, recursive=-1}
    local luahl = {
        ["nil"] = "&Blu",
        string = "&gre",
        number = "&cya",
        boolean = "&Red",
        syntax = "&res",
        builtin = "&red",
        keyword = "&mag",
        class = "&Gre",
        metatable = "&red",
        ["function"] = "&Blu",
    }
    local builtinkeyword = {
        "and","break","do", "else", "elseif", "end", "for", "function", "if", "in", "local", "nil", "not", "or", "repeat", "return", "then", "until", "while"
    }
    local parameters = {
        depth = testOption(options.depth, {"boolean", "number"}, false),
        maxTableLen = testOption(options.maxTableLen, {"boolean", "number"}, 20),
        maxStringLen = testOption(options.maxStringLen, {"boolean", "number"}, 160),
        compact = testOption(options.compact, {"boolean"}, true),
        maxLen = testOption(options.maxLen, {"boolean", "number"}, false),
        colors = testOption(options.colors, {"boolean"}, false),
        customColors = testOption(options.customColors, {"table"}, luahl)
    }
    for k, v in pairs(parameters.customColors) do
        parameters.customColors[k] = v:gmatch("&%a%a%a")() or ""
    end
    for k, v in pairs(luahl) do
        if not parameters.customColors[k] then
            parameters.customColors[k] = k=="syntax" and v or ""
        end
    end

    local inherit = {depth=options.__inherit.depth, recursive=options.__inherit.recursive}
    if inherit then
        if typeof == "table" then inherit.depth = inherit.depth + 1 elseif inherit.depth == -1 then inherit.depth = 0 end
        inherit.recursive = inherit.recursive + 1
        parameters.__inherit = inherit
    end
    if typeof == "string" then
        local trunced=false
        local len = #item
        if type(parameters.maxStringLen) == "number" then
            if len > parameters.maxStringLen then
                item = item:sub(1, math.abs(parameters.maxStringLen))
                trunced = true
            end
        end
        if parameters.colors then
            item =  parameters.customColors.string .. item ..  parameters.customColors.syntax
        end
        if trunced then
            item = item .. " ... "..len-math.abs(parameters.maxStringLen).." more"
        end
        if inherit.recursive > 0 then item = "'"..item.."'" end
        str=str..item
    elseif typeof == "number" then
        if parameters.colors then
            item =  parameters.customColors.number .. tostring(item) ..  parameters.customColors.syntax
        end
        str = str..item
    elseif typeof == "table" then
        local len=0
        for _ in pairs(item) do
            len=len+1
        end
        if type(parameters.depth) == "number" then
            if parameters.depth <= inherit.depth then
                if parameters.colors then
                    local syntax = parameters.customColors.syntax
                    local builtin = parameters.customColors.builtin
                    str =  builtin .. "[table:"..parameters.customColors.number..len..builtin.."]" ..  syntax
                    str = strColorize(str)
                else
                    str = "[table:"..len.."]"
                end
                return str
            end
        end
        local actuallen = 0
        local maxed = false
        local substr = parameters.customColors.syntax.."{%s"
        local start = true
        for k, v in ipairs(item) do
            if type(parameters.maxTableLen) == "number" then
                if actuallen >= math.abs(parameters.maxTableLen) then
                    substr = substr:format(" ... " .. len - actuallen .. " more%s")
                    maxed = true
                    break
                end
            end
            local tab = ""
            if not parameters.compact then
                if k==1 and type(v) ~= "table" then
                    tab = "\n"
                    for i=0, inherit.depth do
                        tab = tab.."   "
                    end
                end
                if type(v) == "table" then
                    tab = "\n"
                    for i=0, inherit.depth do
                        tab = tab.."   "
                    end
                end
            end
            substr = substr:format((start and "" or ", ")..tab..inspect(v, parameters).."%s")
            start = false
            actuallen = actuallen + 1
        end

        for k, v in pairs(item) do
            if maxed == true then break end
            if type(parameters.maxTableLen) == "number" then
                if actuallen >= math.abs(parameters.maxTableLen) then
                    substr = substr:format(" ... " .. len - actuallen .. " more%s")
                    maxed = true
                    break
                end
            end
            if type(k) ~= "number" then
                local key
                if parameters.colors then
                    key = type(k) == "string" and parameters.customColors.keyword.. k .. parameters.customColors.syntax or parameters.customColors.keyword .."["..type(k).."]"..parameters.customColors.syntax
                else
                    key = type(k) == "string" and k or "["..type(k).."]"
                end
                local tab = ""
                if not parameters.compact then
                    tab = "\n"
                    for i=0, inherit.depth do
                        tab = tab.."   "
                    end
                end
                if v == item then
                    substr = substr:format((start and "" or ", ").. tab ..key ..": [Circular]%s")
                else
                    substr = substr:format((start and "" or ", ").. tab ..key ..": "..inspect(v, parameters).."%s")
                end
                start=false
                actuallen = actuallen + 1
            end
        end
        if not parameters.colors then
            substr = substr:sub(5)
        end
        local tab = ""
        if not parameters.compact then
            for i=1, inherit.depth do
                tab = tab.."   "
            end
            substr = substr:format("\n"..tab.."%s")
        end
        str = substr:format("}")
    elseif typeof == "function" then
        local infos = debug.getinfo(item)
        local func = item
        if type(parameters.depth) == "number" then
            if parameters.depth <= inherit.depth then
                if parameters.colors then
                    local syntax = parameters.customColors.syntax
                    local builtin = parameters.customColors.builtin
                    str =  builtin .. "[function:"..parameters.customColors.number..infos.nparams..builtin.."]" ..  syntax
                else
                    str = "[function:"..infos.nparams.."]"
                end
                return str
            end
        end

        if parameters.colors then
            local syntax = parameters.customColors.syntax
            local builtin = parameters.customColors.builtin
            item =  builtin .. "[function:"..parameters.customColors.number..infos.nparams..builtin.."]" ..  syntax
        else item = "[function:"..infos.nparams.."]" end
        if inherit.recursive <= 0 then
            local tmpdepth = parameters.depth
            parameters.depth=false

            local names = {}
            for i=1, tonumber(infos.nparams) do
                local arg = debug.getlocal(func, i)
                if arg then
                    table.insert(names, arg)
                end
            end

            local data = {
                name = infos.name or "",
                source = infos.short_src,
                args = infos.nparams,
                argsnames = names,
                type = infos.what
            }
            item = item..inspect(data, parameters)
            parameters.depth = tmpdepth
        end
        str = str..item
    elseif typeof == "userdata" then
        if parameters.colors then
            local syntax = parameters.customColors.syntax
            local builtin = parameters.customColors.builtin
            item =  builtin .. "[userdata]" ..  syntax
        else item = "[userdata]" end
        str = str..item
    elseif typeof == "thread" then
        if parameters.colors then
            local syntax = parameters.customColors.syntax
            local builtin = parameters.customColors.builtin
            item =  builtin .. "[thread]" ..  syntax
        else item = "[thread]" end
        str = str..item
    elseif typeof == "boolean" then
        if parameters.colors then
            item =  parameters.customColors.boolean .. tostring(item) .. parameters.customColors.syntax
        end
        str=str..tostring(item)
    elseif typeof == "nil" then
        if parameters.colors then
            item = parameters.customColors["nil"] .. "nil" .. parameters.customColors.syntax
        else item = "nil" end
        str=str..item
    end

    if parameters.colors and inherit.recursive == 0 then
        str = strColorize(str)
    end
    return str
end


return inspect