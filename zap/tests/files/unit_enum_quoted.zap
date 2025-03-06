event MyEvent = {
    from: Server,
    type: Reliable,
    call: ManyAsync,
    data: enum { "Foo Bar", "Bar Foo", Buzz }
}
