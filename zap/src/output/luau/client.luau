if RunService:IsServer() then
	error("Cannot use the client module on the server!")
end

local reliable = ReplicatedStorage:WaitForChild("ZAP_RELIABLE")
local unreliable = ReplicatedStorage:WaitForChild("ZAP_UNRELIABLE")

assert(reliable:IsA("RemoteEvent"), "Expected ZAP_RELIABLE to be a RemoteEvent")
assert(unreliable:IsA("UnreliableRemoteEvent"), "Expected ZAP_UNRELIABLE to be an UnreliableRemoteEvent")

local event_queue: { [number]: { any } } = {}

local time = 0

RunService.Heartbeat:Connect(function(dt)
	time += dt

	if time >= (1 / 61) then
		time -= (1 / 61)

		if outgoing_used ~= 0 then
			local buff = buffer.create(outgoing_used)
			buffer.copy(buff, 0, outgoing_buff, 0, outgoing_used)

			reliable:FireServer(buff, outgoing_inst)

			outgoing_buff = buffer.create(64)
			outgoing_used = 0
			outgoing_size = 64
			table.clear(outgoing_inst)
		end
	end
end)
