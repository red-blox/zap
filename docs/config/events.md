<script setup lang="ts">
const example = `event MyEvent = {
	from: Server,
	type: Reliable,
	call: ManyAsync,
	data: struct {
		foo: string,
		bar: u8,
	},
}`
</script>

# Events

Events are the primary method of communicating between the client and the server. Events are also what is exposed to the developer from Zap's generated API.

## Defining Events

Events are defined in your config file using the `event` keyword.

<CodeBlock :code="example" />

As you can see they have four fields. Let's go over them one by one:

### `from`

This field determines which side of the game can fire the event. It can be either `Server` or `Client`.

At this time Zap does not support two way events. As events have almost no overhead, feel free to add more events instead of using two way events.

### `type`

This field determines the type of event. It can be either `Reliable` or `Unreliable`.

- Reliable events are guaranteed to arrive at their destination in the order they were sent.
- Unreliable events are not guaranteed to arrive at their destination, and they are not guaranteed to arrive in the order they were sent. Unreliable events also have a maximum size of 900 bytes.

### `call`

This field determines how the event is listened to on the receiving side.

- `ManyAsync` events can be listened to by many functions, and they are called asynchronously.
- `ManySync` events can be listened to by many functions, and they are called synchronously.
- `SingleAsync` events can be listened to by one function, and they are called asynchronously.
- `SingleSync` events can be listened to by one function, and they are called synchronously.

::: danger
Synchronous events are not recommended, and should only be used when performance is critical.

- If a synchronous event callback yields it will cause **undefined and game-breaking behavior**.
- If a synchronous event callback errors it will cause **the packet to be dropped**.

Use synchronous events with extreme caution.
:::

### `data`

This field determines the data that is sent with the event. It can be any [Zap type](./types.md).
