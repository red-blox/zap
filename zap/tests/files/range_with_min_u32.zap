event Test = {
    from: Client,
    type: Reliable,
    call: SingleSync,
    data: u8[3..u32]
}
