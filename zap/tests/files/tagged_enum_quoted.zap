event MyEvent = {
    from: Server,
    type: Reliable,
    call: ManyAsync,
    data: enum "tag with spaces" {
        foo { "value with spaces": string, bar: u8 },
        bar { buzz: u8 },
    }
}
