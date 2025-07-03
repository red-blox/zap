event Empty = {
    from: Client,
    type: Reliable,
    call: SingleSync,
    data: u8[]
}

event Exact = {
    from: Client,
    type: Reliable,
    call: SingleSync,
    data: u8[4]
}

event U32 = {
    from: Client,
    type: Reliable,
    call: SingleSync,
    data: u8[u32]
}

event WithMax = {
    from: Client,
    type: Reliable,
    call: SingleSync,
    data: u8[..3]
}

event WithMinMax = {
    from: Client,
    type: Reliable,
    call: SingleSync,
    data: u8[2..3]
}

event WithMinU32 = {
    from: Client,
    type: Reliable,
    call: SingleSync,
    data: u8[3..u32]
}

event WithMin = {
    from: Client,
    type: Reliable,
    call: SingleSync,
    data: u8[2..]
}

