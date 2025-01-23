event NestedComplex = {
    from: Server,
    type: Reliable,
    call: SingleSync,
    data: map { [map { [u8[]]: string }]: struct { x: u8 }[] },
}
