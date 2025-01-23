--!strict
local luau = require("@lune/luau")
local process = require("@lune/process")
local roblox = require("@lune/roblox")
local task = require("@lune/task")
local serverCode, clientCode, toolingCode = unpack(process.args)

local noop = function() end

roblox.implementMethod("Instance", "WaitForChild", function(self, ...)
	return self:FindFirstChild(...)
end)

roblox.implementMethod("RunService", "IsRunning", function()
	return true
end)

roblox.implementMethod("RunService", "IsClient", function() 
	return debug.info(2, "s") == "Client"
end)

roblox.implementMethod("RunService", "IsServer", function() 
	return debug.info(2, "s") == "Server"
end)

roblox.implementProperty("RunService", "Heartbeat", function()
	return {
		Connect = noop
	}
end)

roblox.implementProperty("Players", "PlayerRemoving", function()
	return {
		Connect = noop
	}
end)

roblox.implementProperty("RemoteEvent", "OnServerEvent", function()
	return {
		Connect = noop
	}
end)

roblox.implementProperty("UnreliableRemoteEvent", "OnServerEvent", function()
	return {
		Connect = noop
	}
end)

roblox.implementProperty("RemoteEvent", "OnClientEvent", function()
	return {
		Connect = noop
	}
end)

roblox.implementProperty("UnreliableRemoteEvent", "OnClientEvent", function()
	return {
		Connect = noop
	}
end)

local environment = {
	game = roblox.Instance.new("DataModel"),
	Instance = roblox.Instance,
	CFrame = {
		Angles = noop
	},
	task = task
}

local server = luau.load(serverCode, {
	debugName = "Server",
	codegenEnabled = true,
	environment = environment
})()

local client = luau.load(clientCode, {
	debugName = "Client",
	codegenEnabled = false,
	environment = environment
})()

local tooling = luau.load(toolingCode, {
	debugName = "Tooling",
	codegenEnabled = true,
	environment = environment
})()
