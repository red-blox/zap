local Players = game:GetService("Players")

if RunService:IsClient() then
	error("Cannot use the server module on the client!")
end

local reliable = ReplicatedStorage:FindFirstChild(`{SCOPE_NAME}_RELIABLE`)
if reliable == nil then
	reliable = Instance.new("RemoteEvent")
	reliable.Name = `{SCOPE_NAME}_RELIABLE`
	reliable.Parent = ReplicatedStorage
end

local unreliable = ReplicatedStorage:FindFirstChild(`{SCOPE_NAME}_UNRELIABLE`)
if unreliable == nil then
	unreliable = Instance.new("UnreliableRemoteEvent")
	unreliable.Name = `{SCOPE_NAME}_UNRELIABLE`
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

Players.PlayerRemoving:Connect(function(player)
	player_map[player] = nil
end)
