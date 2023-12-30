<script setup lang="ts">
const configFile = `event MyEvent = {
    from: Server,
    type: Reliable,
    call: ManyAsync,
    data: struct {
        foo: string,
        bar: u8,
    },
}

event AnotherEvent = {
    from: Client,
    type: Reliable,
    call: SingleAsync,
    data: struct {
        baz: boolean,
    },
}`
</script>

# Using Zap

Zap's generated API is fully typesafe. While Zap will not throw errors for invalid usage, your static analysis tooling will.

- If you're using [VSCode](https://code.visualstudio.com/), we recommend installing the [Luau-LSP](https://marketplace.visualstudio.com/items?itemName=JohnnyMorganz.luau-lsp) extension.
- If you're using Roblox Studio, you already have the proper Luau tooling.

::: info
This page will assume you're using the default casing value of `PascalCase`.

Learn more about casing options [here](../config/options.md#casing).
:::

Zap generates two files, one for the client and one for the server. They each contain the API for their respective side.

## Events

Throughout this guide we'll be using the following events as examples.

<CodeBlock :code="configFile" />

## Listening to Events

Listening to events is done the same way on both the server and client.

If your event's [call field](../config/events.md#call) is `SingleAsync` or `SingleSync` you can only assign one listener. This is done with the `SetListener` function.

```lua
local Zap = require(Path.To.Zap)

-- only server listeners are given the player argument
Zap.AnotherEvent.SetListener(function(Player, Data)
    -- Do something with the player and data
end)
```

If your event's [call field](../config/events.md#call) is `ManyAsync` or `ManySync` you can assign multiple listeners. This is done with the `AddListener` function.

```lua
local Zap = require(Path.To.Zap)

local Disconnect = Zap.MyEvent.On(function(Data)
    -- Do something with the data
end)

-- Disconnect the listener after 10 seconds
task.delay(10, Disconnect)
```

As seen above, when using `Many` events, the `On` function returns a `Disconnect` function. This function can be used to disconnect the listener.

::: danger
Remember that `Sync` event callbacks must not yield and must not error.

- If a sync callback yields, it will cause undefined and game-breaking behavior.
- If a sync callback errors, it will drop the packet.

Use `Sync` events only when performance is critical.
:::

## Firing From the Client

The client only has a single function for firing events, `Fire`. This function takes the event's data as its only argument.

```lua
local Zap = require(Path.To.Zap)

Zap.AnotherEvent.Fire({
    baz = true,
})
```

## Firing From the Server

The server has many functions for firing events, each with their own use case.

::: tip
`FireAll`, `FireExcept`, and `FireList` only serialize the event's data once, making it more performant than looping over players and firing the event to each of them individually.

If you're firing the same data to multiple players, use these functions.
:::

### Fire

The basic `Fire` function takes a player and the event's data as its arguments.

```lua
local Zap = require(Path.To.Zap)

Zap.MyEvent.Fire(Player, {
    foo = "baz",
    bar = 1,
})
```

### FireAll

The `FireAll` function takes the event's data as its only argument. It will fire the event to all players.

```lua
local Zap = require(Path.To.Zap)

Zap.MyEvent.FireAll({
    foo = "baz",
    bar = 1,
})
```

### FireExcept

The `FireExcept` function takes a player and the event's data as its arguments. It will fire the event to all players except the one provided.

```lua
local Zap = require(Path.To.Zap)

Zap.MyEvent.FireExcept(Player, {
    foo = "baz",
    bar = 1,
})
```

### FireList

The `FireList` function takes a list of players and the event's data as its arguments. It will fire the event to all players in the list.

```lua
local Zap = require(Path.To.Zap)

Zap.MyEvent.FireList({Player1, Player2}, {
    foo = "baz",
    bar = 1,
})
```
