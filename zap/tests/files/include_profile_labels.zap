opt include_profile_labels = true

event Event1 = {
    from: Client,
    type: Reliable,
    call: SingleSync,
    data: string.binary
}

event Event2 = {
    from: Server,
    type: Reliable,
    call: ManySync,
    data: string.binary
}

funct Function = {
    call: Async,
    rets: string.binary
}

namespace Namespace = {
	event Event1 = {
	    from: Client,
	    type: Unreliable,
	    call: SingleAsync,
	    data: u8
	}

	event Event2 = {
	    from: Server,
	    type: Unreliable,
	    call: ManyAsync,
	    data: u8
	}
}
