if RunService:IsServer() then
	error("Cannot use the client module on the server!")
end

local reliable = ReplicatedStorage:WaitForChild("ZAP_RELIABLE")
local unreliable = ReplicatedStorage:WaitForChild("ZAP_UNRELIABLE")

assert(reliable:IsA("RemoteEvent"), "Expected ZAP_RELIABLE to be a RemoteEvent")
assert(unreliable:IsA("UnreliableRemoteEvent"), "Expected ZAP_UNRELIABLE to be an UnreliableRemoteEvent")

local time = 0
