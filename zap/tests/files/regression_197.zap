namespace A = {
    namespace B = {
        type Stuff = u8
    }

    event Test = {
        from: Client,
        type: Reliable,
        call: SingleSync,
        data: B.Stuff
    }
}
