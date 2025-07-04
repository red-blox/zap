opt typescript_enum = "ConstEnum"

type Test = enum { Foo, Bar, Baz }

event Event = {
    from: Client,
    type: Reliable,
    call: SingleSync,
    data: (Test, (Test | string.binary))
}
