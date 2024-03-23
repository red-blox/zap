local Players = game:GetService("Players")

if RunService:IsClient() then
	error("Cannot use the server module on the client!")
end

local reliable: RemoteEvent? = ReplicatedStorage:FindFirstChild("ZAP_RELIABLE")
if reliable == nil then
	reliable = Instance.new("RemoteEvent")
	reliable.Name = "ZAP_RELIABLE"
	reliable.Parent = ReplicatedStorage
end

local unreliable: UnreliableRemoteEvent? = ReplicatedStorage:FindFirstChild("ZAP_UNRELIABLE")
if unreliable == nil then
	unreliable = Instance.new("UnreliableRemoteEvent")
	unreliable.Name = "ZAP_UNRELIABLE"
	unreliable.Parent = ReplicatedStorage
end

local player_map = {}

local function load_player(player: Player)
	if player_map[player] then
		load(player_map[player])
	else
		load_empty()
	end
end

Players.PlayerRemoving:Connect(function(player: Player)
	player_map[player] = nil
end)
