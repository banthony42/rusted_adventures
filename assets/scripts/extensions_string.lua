
function string:trim()
    return self:gsub("^%s*(.-)%s*$", "%1")
end

function string:split(delimiter)
    local result = {}
    local special_char_set = "().%+*?[^$"
    local escape_delimiter = ''
    for special_char in special_char_set:gmatch(".") do
        if special_char == delimiter then
            escape_delimiter = '%'
        end
    end
    self = self:trim()
    for match in (self..delimiter):gmatch("(.-)"..escape_delimiter..delimiter) do
        table.insert(result, match)
    end
    return result
end