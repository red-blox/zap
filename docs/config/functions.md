<script setup lang="ts">
const example = `funct Test = {
    call: Async,
    args: struct {
        foo: u8,
        bar: string
    },
    rets: enum { "Success", "Fail" }
}`
</script>

# Functions

Functions are another method of communication where the client can send arguments and have them returned by the server. For security, Zap only supports Client -> Server -> Client functions, not Server -> Client -> Server.

## Defining Events

Events are defined in your config file using the `funct` keyword.

<CodeBlock :code="example" />

As you can see they have three fields. Let's go over them one by one:

### `call`

This field determines how the event is listened to on the server. The function will take the `args` as parameters and return `rets`.

- `Async` events can be listened to by one function, and they are called asynchronously.
- `Sync` events can be listened to by one function, and they are called synchronously.

::: danger
Synchronous events are not recommended, and should only be used when performance is critical.

- If a synchronous event callback yields it will cause **undefined and game-breaking behavior**.
- If a synchronous event callback errors it will cause **the packet to be dropped**.

Use synchronous events with extreme caution.
:::

### `args`

This field determines the data that is sent to the server. It can be any [Zap type](./types.md).

### `rets`

This field determines the data that is sent back to the client from the server. It can be any [Zap type](./types.md).
