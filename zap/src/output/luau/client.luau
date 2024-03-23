if RunService:IsServer() then
	error("Cannot use the client module on the server!")
end

local reliable: RemoteEvent = ReplicatedStorage:WaitForChild("ZAP_RELIABLE")
local unreliable: UnreliableRemoteEvent = ReplicatedStorage:WaitForChild("ZAP_UNRELIABLE")

assert(reliable:IsA("RemoteEvent"), "Expected ZAP_RELIABLE to be a RemoteEvent")
assert(unreliable:IsA("UnreliableRemoteEvent"), "Expected ZAP_UNRELIABLE to be an UnreliableRemoteEvent")

local time = 0
