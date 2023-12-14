local reliable = Instance.new("RemoteEvent")
reliable.Name = "ZAP_RELIABLE"
reliable.Parent = game:GetService("ReplicatedStorage")

local unreliable = Instance.new("RemoteEvent")
unreliable.Name = "ZAP_UNRELIABLE"
unreliable.Parent = game:GetService("ReplicatedStorage")

local outgoing_map: { [Player]: { buff: buffer, used: number, size: number } } = {}

game:GetService("Players").PlayerAdded:Connect(function(player)
	outgoing_map[player] = {
		buff = buffer.create(64),
		used = 0,
		size = 64,
	}
end)

game:GetService("Players").PlayerRemoving:Connect(function(player)
	outgoing_map[player] = nil
end)

game:GetService("RunService").Heartbeat:Connect(function()
	for player, outgoing in outgoing_map do
		if outgoing.used > 0 then
			local buff = buffer.create(outgoing.used)
			buffer.copy(buff, 0, outgoing.buff, 0, outgoing.used)

			reliable:FireClient(player, buff)

			outgoing.buff = buffer.create(64)
			outgoing.used = 0
			outgoing.size = 64
		end
	end
end)

local read_event_id: () -> number
local write_event_id: (number) -> ()

reliable.OnServerEvent:Connect(function(player, buff)
	incoming_buff = buff
	incoming_read = 0

	local size = buffer.len(incoming_buff)

	while incoming_read ~= size do
		event_callbacks[read_event_id()](player)
	end
end)

unreliable.OnServerEvent:Connect(function(player, buff)
	incoming_buff = buff
	incoming_read = 0
	event_callbacks[read_event_id()](player)
end)

local function load_player(player)
	local outgoing = outgoing_map[player]

	outgoing_buff = outgoing.buff
	outgoing_used = outgoing.used
	outgoing_size = outgoing.size
end

local function save_player(player)
	local outgoing = outgoing_map[player]

	outgoing.used = outgoing_used
	outgoing.size = outgoing_size
end
