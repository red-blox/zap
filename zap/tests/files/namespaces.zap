type MyType = u8

event Event1 = {
    from: Client,
    type: Reliable,
    call: SingleSync,
    data: MyType
}

namespace NS = {
    type MyType = string

    event Event1 = {
        from: Client,
        type: Reliable,
        call: SingleSync,
        data: MyType
    }
}

event Event2 = {
    from: Client,
    type: Reliable,
    call: SingleSync,
    data: NS.MyType
}