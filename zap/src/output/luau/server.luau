if game:GetService("RunService"):IsClient() then
	error("Cannot use the server module on the client!")
end

local reliable = Instance.new("RemoteEvent")
reliable.Name = "ZAP_RELIABLE"
reliable.Parent = game:GetService("ReplicatedStorage")

local unreliable = Instance.new("RemoteEvent")
unreliable.Name = "ZAP_UNRELIABLE"
unreliable.Parent = game:GetService("ReplicatedStorage")

local player_map = {}

game:GetService("Players").PlayerAdded:Connect(function(player)
	player_map[player] = {
		buff = buffer.create(64),
		used = 0,
		size = 64,
		inst = {},
	}
end)

game:GetService("Players").PlayerRemoving:Connect(function(player)
	player_map[player] = nil
end)

game:GetService("RunService").Heartbeat:Connect(function()
	for player, outgoing in player_map do
		if outgoing.used > 0 then
			local buff = buffer.create(outgoing.used)
			buffer.copy(buff, 0, outgoing.buff, 0, outgoing.used)

			reliable:FireClient(player, buff, outgoing.inst)

			outgoing.buff = buffer.create(64)
			outgoing.used = 0
			outgoing.size = 64
			table.clear(outgoing.inst)
		end
	end
end)
