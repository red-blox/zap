if RunService:IsServer() then
	error("Cannot use the client module on the server!")
end

local reliable = ReplicatedStorage:WaitForChild(`{SCOPE_NAME}_RELIABLE`)
local unreliable = ReplicatedStorage:WaitForChild(`{SCOPE_NAME}_UNRELIABLE`)

assert(reliable:IsA("RemoteEvent"), `Expected {SCOPE_NAME}_RELIABLE to be a RemoteEvent`)
assert(unreliable:IsA("UnreliableRemoteEvent"), `Expected {SCOPE_NAME}_UNRELIABLE to be an UnreliableRemoteEvent`)

local time = 0
