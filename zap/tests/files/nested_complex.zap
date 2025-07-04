event NestedComplex = {
    from: Server,
    type: Reliable,
    call: SingleSync,
    data: map { [map { [u8[]]: string.binary }]: struct { x: u8 }[] },
}
