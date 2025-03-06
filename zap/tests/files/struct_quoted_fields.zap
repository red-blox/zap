event MyEvent = {
    from: Server,
    type: Reliable,
    call: ManyAsync,
    data: struct {
        "foo bar": string,
        buzz: u8
    }
}
