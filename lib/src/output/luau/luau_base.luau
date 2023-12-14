local outgoing_buff: buffer = buffer.create(64)
local outgoing_used: number = 0
local outgoing_size: number = 64

local function alloc(len: number): number
	if outgoing_used + len > outgoing_size then
		outgoing_size = outgoing_size * 2

		local new_buff = buffer.create(outgoing_size)
		buffer.copy(new_buff, 0, outgoing_buff, 0, outgoing_used)

		outgoing_buff = new_buff
	end

	local pos = outgoing_used
	outgoing_used = outgoing_used + len
	return pos
end

local incoming_buff: buffer
local incoming_read: number

local function read(len: number): number
	local pos = incoming_read
	incoming_read = incoming_read + len
	return pos
end

local types = {}
local event_callbacks: { [number]: any }
local event_user_call: { [number]: any }

local function load_empty()
	outgoing_buff = buffer.create(64)
	outgoing_used = 0
	outgoing_size = 64
end
