# What is Zap?

Zap is a tool to generate highly efficient networking code for Roblox. It has no compromises on performance or developer experience. Zap takes a description of your data and event, and generates code specific to your game.

<div class="custom-block tip" style="padding-top: 8px">

Already convinced? Head over to [getting started](./getting-started).

</div>

## ⚡ Performance

Zap is designed to be incredibly fast, but what does this mean in terms of networking?

### Bandwidth Usage

The most important metric when it comes to networking is bandwidth usage. Zap packs all data into buffers, which on top of using less space for the same data, also compress when passed through RemoteEvents.

### CPU Usage

All of this saved bandwidth comes at a cost, right? Not exactly. Zap generates code specific to your game, which means it can use the most efficient way to pack and unpack data. This means minimal branching and minimal calls.

## 🧑‍💻 Developer Experience

Zap has an unparalleled developer experience. From writing your network description to using the API, Zap is a joy to use.

### Writing your Network Description

Zap's IDL is easy to learn and easy to use. It's simple while still being expressive and powerful. If you mess up, it's okay. Zap provides helpful error messages to get you back on track.

### Using the API

Zap's API is fully typesafe, with Luau or TypeScript. You'll recieve full type checking and autocompletion in your editor.

### Uncompromised Maps

Map from anything to anything. If your keys' datatype is sendable through a `RemoteEvent` at all, Zap handles it. 

## 🔒 Security

Zap is fully secure. Buffers make reverse engineering your game's networking much harder and Zap validates all data received.

### Buffers

Buffers are byte arrays that Zap uses to send data. They are _not_ human readable, and are much harder to reverse engineer than Roblox's self-describing encoding. Zap's encoded data changes based on the data being sent, which makes it even harder to read.

### Validation

Zap validates all data recieved. If a client sends invalid data, Zap will catch it before it reaches your game code. This means you don't have to worry about malicious clients sending invalid data.
