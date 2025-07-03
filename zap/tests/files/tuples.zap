event MyEvent = {
	from: Server,
	type: Reliable,
	call: ManyAsync,
	data: (Foo: boolean, Bar: u32, Baz: string.binary)
}
