funct Test = {
    call: Async,
    args: (Foo: u8, Bar: string.binary),
    rets: (enum { Success, Fail }, string.binary)
}
