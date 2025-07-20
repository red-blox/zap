event Event1 = {
	from: Client,
	type: Reliable,
	call: SingleSync,
	data: boolean[8],
}

event Event2 = {
	from: Client,
	type: Reliable,
	call: SingleSync,
	data: boolean?[17],
}

event Event3 = {
	from: Client,
	type: Reliable,
	call: SingleSync,
	data: struct {
		bools: boolean[14],
		enum: enum { hello, world },
	},
}

event Event4 = {
	from: Client,
	type: Reliable,
	call: SingleSync,
	data: struct {
		bools: boolean[14],
		enum: enum { hello, there, world },
	},
}

event Event5 = {
	from: Client,
	type: Reliable,
	call: SingleSync,
	data: enum "type" {
		a { value: boolean },
		b { value: boolean },
	},
}
