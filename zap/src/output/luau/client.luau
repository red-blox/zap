if game:GetService("RunService"):IsServer() then
	error("Cannot use the client module on the server!")
end

local reliable = game:GetService("ReplicatedStorage"):WaitForChild("ZAP_RELIABLE")
local unreliable = game:GetService("ReplicatedStorage"):WaitForChild("ZAP_UNRELIABLE")

local event_queue: { [number]: { any } } = {}

local time = 0

game:GetService("RunService").Heartbeat:Connect(function(dt)
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
