local fio = require('fio')

function string.starts(String, Start)
   return string.sub(String,1,string.len(Start))==Start
end

local work_dir = fio.cwd() .. "/"
local data_path = work_dir .. "/data/"
local ostype = jit.os
if string.starts(ostype, "OSX") then
    package.cpath = package.cpath .. ';' .. work_dir .. '/?.dylib;'
elseif string.starts(ostype, "Linux") then
    package.cpath = package.cpath .. ';' .. work_dir .. '/?.so;'
else
    print("Unknown platform '" .. ostype .. "'\n")
    os.exit()
end

box.cfg {
    listen = 3306,
    work_dir = data_path,
}

net_box = require('net.box')
capi_connection = net_box:new(3306)

box.schema.user.grant('guest', 'read,write,execute,create,alter,drop,usage,session', 'universe', '', { if_not_exists = true })

box.schema.func.create('libtnt_app.start_service', {language = 'C', if_not_exists = true})

capi_connection:call('libtnt_app.start_service')

