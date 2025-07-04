event Test = {
    from: Client,
    type: Reliable,
    call: SingleSync,
    data: (string.binary | unknown | buffer | u16 | struct { bar: u8 } | enum { foo, bar, baz } | Instance.Part | enum "test" {
        a { 
            b: u8
        },
        c { 
            d: u8
        }
    })
}