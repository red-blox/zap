<script setup lang="ts">
const exampleConfig = `-- these two settings can be ignored if you're not using the CLI
opt server_output = "path/to/server/output.luau"
opt client_output = "path/to/client/output.luau"

event MyEvent = {
	from: Server,
	type: Reliable,
	call: ManyAsync,
	data: struct {
		foo: u32,
		bar: string,
	},
}`

const apiExample = `-- Server
local Zap = require(path.to.server.output)

Zap.MyEvent.FireAll({
	foo = 123,
	bar = "hello world",
})

-- Client
local Zap = require(path.to.client.output)

Zap.MyEvent.On(function(data)
	print(data.foo, data.bar)
end)`
</script>

# Getting Started

## Installation

For users who wish to use Zap as a CLI, you can download the latest release from [GitHub Releases](https://github.com/red-blox/zap/releases/), or you can install it using [aftman](https://github.com/lpghatguy/aftman):

```bash
aftman add red-blox/zap
```

Alternatively you can use the [web playground](https://zap.redblox.dev/playground) and move the generated files into your project.

## Writing Your First Network Description

Zap's IDL (Interface Description Language) is very simple, and you'll learn more about it further in this guide. For now, here's a simple example:

<CodeBlock :code="exampleConfig"/>

## Generating Code

If you're using the playground your code will be generated automatically on every change. If you're using the CLI you can generate code by running:

```bash
zap path/to/config.zap
```

This will generate two files, `path/to/server/output.luau` and `path/to/client/output.luau`. You can then use these files in your project.

## Using the API

Zap's generated files return a table with all of their events. You can use this table to connect to events and fire them.

<CodeBlock :code="apiExample" lang="lua" />
