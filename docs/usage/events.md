# Using Zap

All of zap's output code is **fully typed**, and will pickup if you input the wrong type.

The code will also assert **at runtime** if conditional constraints are added to the config file (that can't be validated by luau's type system).

Let's assume we have the following config file:

<script setup lang="ts">
const configFile = `event RoundReady = {
    from: Client,
    type: Reliable,
    call: SingleSync,
    data: bool
}`

const senderExample = `local zap = require(path.to.network.client)
local isReady = true

zap.Foo.Fire(isReady)`

const receiverExample = `local zap = require(path.to.network.server)

zap.Foo.SetCallback(function(player, isReady)
    print(player.Name, isReady)
end)`
</script>

<CodeBlock :code='configFile' />

Zap __does not__ pass a self value (a function called with `:`), and instead all the outputted functions must be called with `.`.

## From the Sender

<CodeBlock lang="lua" :code="senderExample" />

## From the Receiver

<CodeBlock lang="lua" :code="receiverExample" />

