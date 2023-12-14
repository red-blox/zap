local reliable: RemoteEvent = game:GetService("ReplicatedStorage"):WaitForChild("ZAP_RELIABLE")
local unreliable: UnreliableRemoteEvent = game:GetService("ReplicatedStorage"):WaitForChild("ZAP_UNRELIABLE")

local t = 0

game:GetService("RunService").Heartbeat:Connect(function(dt)
	t = t + dt

	-- Run at 64hz over 60hz to make sure networking is done *every*
	-- frame on the client, regardless of locked or unlocked framrate.
	if t >= (1 / 64) then
		t = t - (1 / 64)

		if outgoing_used == 0 then
			return
		end

		local buff = buffer.create(outgoing_used)
		buffer.copy(buff, 0, outgoing_buff, 0, outgoing_used)

		reliable:FireServer(buff)

		outgoing_buff = buffer.create(64)
		outgoing_used = 0
		outgoing_size = 64
	end
end)

local read_event_id: () -> number
local write_event_id: (number) -> ()

reliable.OnClientEvent:Connect(function(buff)
	incoming_buff = buff
	incoming_read = 0

	local size = buffer.len(incoming_buff)

	while incoming_read ~= size do
		event_callbacks[read_event_id()]()
	end
end)

unreliable.OnClientEvent:Connect(function(buff)
	incoming_buff = buff
	incoming_read = 0
	event_callbacks[read_event_id()]()
end)

local function save()
	return {
		outgoing_buff = outgoing_buff,
		outgoing_used = outgoing_used,
		outgoing_size = outgoing_size,
	}
end

local function load(data)
	outgoing_buff = data.outgoing_buff
	outgoing_used = data.outgoing_used
	outgoing_size = data.outgoing_size
end
