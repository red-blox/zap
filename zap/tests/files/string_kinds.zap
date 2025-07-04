event Binary = {
    from: Client,
    type: Reliable,
    call: SingleSync,
    data: string.binary
}

event Utf8 = {
    from: Client,
    type: Reliable,
    call: SingleSync,
    data: string.utf8
}
