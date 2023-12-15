# Zap

Zap is a blazingly fast networking solution for Roblox.

## Here There be Dragons!

Zap is currently in a early pre-release state. The API may change over time and there are likely bugs. Please report any issues you find through github issues.

## How Does it Work?

Zap takes in a network configuration file, like the following:

```
type Item = {
	Name: string,
	Price: u16,
}

event BuyItem = {
	from: Client,
	type: Reliable,
	call: SingleAsync,
	data: Item,
}
```

And outputs highly efficient networking code that can be used to send and receive data between clients and the server.

```lua
-- Server
local Zap = require(Path.To.Zap.Server)

Zap.BuyItem.SetCallback(function(Player, Item)
	print(Item.Name)
end)

-- Client
local Zap = require(Path.To.Zap.Client)

Zap.BuyItem.Fire({
	Name = "Sword",
	Price = 100,
})
```

## Features

### Type Safety

Zap generates a fully type-safe API for your network configuration. This means full intellisense support with autocomplete and type checking.

### Performance

Zap packs all data into buffers to send over the network. This has the obvious benefits of reduced bandwidth usage, however the serialization and deserialization process is typically quite slow. Zap generates code for your specific types which makes this process blazingly fast.

At this time Zap has not been benchmarked, as it progresses through development and becomes more fully featured benchmarks will be added here.

### Complex Types

While buffers may only support a small number of types, zap has complex type support. Below is a list of all supported types:

- Booleans

```
type t = bool
```

- Numbers (all sizes)

```
type t = f32
type t = f64

type t = u8
type t = u16
type t = u32
type t = u64

type t = i8
type t = i16
type t = i32
type t = i64
```

Numbers can also have range contraints:

```
type Health = u8 (0..=100)
```

- Strings

```
type t = String
```

Strings can have a length range constraint:

```
type Username = String (3..20)
```

- Arrays

```
type t = u8[]
```

Arrays can have a length range constraint:

```
type t = u8[10..20]
```

- Maps

```
type t = { [string]: u8 }
```

- Structs

```
type Item = {
	Name: string,
	Price: u16,
}
```

- Enums

```
type t = { One, Two, Three }
```

- Optionals

```
type t = u8?
```

## Documentation

As Zap is still in early development, documentation is not yet available. Once Zap is more stable and has a more complete feature set documentation will be added.

## Contributing

Contributions are welcome! Please open an issue before you start working on a feature or bug fix so we can discuss it.
