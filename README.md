<div align="center">
	<h1>Zap</h1>
</div>

Zap is a blazingly fast networking solution for Roblox.

## Features

- Zap packs data into buffers with no overhead. The same data can be sent using a fraction of the bandwidth.
- Zap doesn't compromise on performance. Zap's packing and unpacking is typically faster than Roblox's generic encoding.
- Zap supports table keys `RemoteEvent`s don't, like tables, `Vector3`s, `Instance`s, or non-array integers.
- For both the IDL and API, Zap is a joy to use. It's easy to learn, easy to use, and easy to debug. It's the best DX you'll find.
- Zap is fully secure. Buffers make reverse engineering your game's networking much harder and Zap validates all data received.

## Here There be Dragons!

Zap is currently in a early pre-release state. The API may change over time and there are likely bugs. Please report any issues you find through github issues.

## Documentation

Documentation can be found [here](https://zap.redblox.dev/).

## Contributing

Contributions are welcome! Please open an issue before you start working on a feature or bug fix so we can discuss it.

## Logo

Zap Logo sourced from [Twitter](https://github.com/twitter/twemoji/blob/master/assets/svg/26a1.svg) and is under the [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) license.
