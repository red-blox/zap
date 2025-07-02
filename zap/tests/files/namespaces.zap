type MyType = u8

event Event1 = {
    from: Client,
    type: Reliable,
    call: SingleSync,
    data: MyType
}

namespace NS = {
    type MyType = struct { thetype: MyType? }

    event Event1 = {
        from: Client,
        type: Reliable,
        call: SingleSync,
        data: MyType
    }

    event Event3 = {
        from: Client,
        type: Reliable,
        call: SingleSync,
        data: (Color3 | string | unknown)
    }
}

event Event2 = {
    from: Client,
    type: Reliable,
    call: SingleSync,
    data: NS.MyType
}

type TheirType = NS.MyType