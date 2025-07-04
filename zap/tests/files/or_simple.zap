event MyEvent = {
    from: Client,
    type: Reliable,
    call: SingleSync,
    data: (u8 | string.binary)
}
