local Players = game:GetService("Players")

if RunService:IsClient() then
	error("Cannot use the server module on the client!")
end

local reliable = ReplicatedStorage:FindFirstChild("ZAP_RELIABLE")
if reliable == nil then
	reliable = Instance.new("RemoteEvent")
	reliable.Name = "ZAP_RELIABLE"
	reliable.Parent = ReplicatedStorage
end

local unreliable = ReplicatedStorage:FindFirstChild("ZAP_UNRELIABLE")
if unreliable == nil then
	unreliable = Instance.new("RemoteEvent")
	unreliable.Name = "ZAP_UNRELIABLE"
	unreliable.Parent = ReplicatedStorage
end

local player_map = {}

local function load_player(player: Player)
	if player_map[player] then
		load(player_map[player])
	else
		load({
			buff = buffer.create(64),
			used = 0,
			size = 64,
			inst = {},
		})
	end
end

Players.PlayerRemoving:Connect(function(player)
	player_map[player] = nil
end)

RunService.Heartbeat:Connect(function()
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
