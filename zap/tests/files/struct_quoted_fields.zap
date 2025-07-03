event MyEvent = {
    from: Server,
    type: Reliable,
    call: ManyAsync,
    data: struct {
        "foo bar": string.binary,
        buzz: u8
    }
}
