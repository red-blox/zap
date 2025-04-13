---
outline: [2, 3]
---

<script setup lang="ts">
const enumExample = `type Tagged = enum "Type" {
	Number {
		Value: f64,
	},

	String {
		Value: string,
	},

	Boolean {
		Value: boolean,
	},
}`

const structExample = `type Item = struct {
	name: string,
	price: u16,
}`

const orExample = `type Union = (
	string
	| u32
	| enum { "hello", "world" }
	| enum "tag" { one { b: i8 }, two { c: buffer } }
	| unknown
	| Instance (Player)
	| Instance
)`
</script>

# Types

## Range Syntax

In some cases using the entire range of numbers isn't desirable and you'd rather limit the range. This can be done by using Zap's range syntax. This syntax will be very similar for users of Rust.

A full range is written by giving a minimum and maximum, separated by two dots. This looks like `0..100`.

One sided ranges are ranges where only the minimum or maximum is given. These look like `0..` or `..100`.

Exact ranges can be written by only giving the exact number, such as `0` or `100`.

Ranges can be written with no constraints by giving no minimum or maximum. This looks like `..`.

#### As a recap:

| Range    | Min | Max |
| -------- | --- | --- |
| `0..100` | 0   | 100 |
| `0..`    | 0   | ∞   |
| `..100`  | -∞  | 100 |
| `0`      | 0   | 0   |
| `100`    | 100 | 100 |
| `..`     | -∞  | ∞   |

::: tip
Remember that ranges are always inclusive on the min and max.
:::

## Numbers

Zap supports all buffer number types, signed integer, unsigned integer, and floating point.

### Signed Integers

Signed integers are standard integers. They can be positive and negative.

| Type  | Min Value      | Max Value     |
| ----- | -------------- | ------------- |
| `i8`  | -128           | 127           |
| `i16` | -32,768        | 32,767        |
| `i32` | -2,147,483,648 | 2,147,483,647 |

### Unsigned Integers

Unsigned integers are positive integers.

| Type  | Min Value | Max Value     |
| ----- | --------- | ------------- |
| `u8`  | 0         | 255           |
| `u16` | 0         | 65,535        |
| `u32` | 0         | 4,294,967,295 |

### Floating Point Numbers

Floating point numbers are numbers with a decimal point. They can be positive and negative.

Buffers support `f32` and `f64` floating point numbers, but unlike integers these numbers do not have a hard range. Instead the size determines the precision of the number. Determining what precision you need is out of scope for this documentation.

`f64`s are able to store integers accurately up to `2^53` (9,007,199,254,740,992). This is larger than the maximum value of `u32`, but also twice the size.

It should also be noted that the type of numbers in Luau is `f64`.

### Constraining Numbers

Numbers can be constrained by placing [a range](#as-a-recap) within parenthesis after the number type. For example, if you wanted to constrain a `u8` between `0` and `100` you could do:

<CodeBlock code="u8(..100)" />

## Strings

Strings are defined using the word `string`. For example:

<CodeBlock code="string" />

The length of strings can be constrained by placing [a range](#as-a-recap) within parenthesis after the `string` keyword. For example, if you wanted to constrain a string between `3` and `20` characters (like a username) you could do:

<CodeBlock code="string(3..20)" />

::: tip WARNING
It's possible for usernames to exceed 20 characters under certain circumstances.
:::

## Unknown

The keyword `unknown` represents data of a type that can't be known until runtime. For values that can be one of multiple known types, use [tagged enums](#tagged-enums) instead.

## Arrays

Arrays are defined as a type followed by square brackets. For example an array of `u8`s would be:

<CodeBlock code="u8[]" />

The length of arrays can be constrained by placing [a range](#as-a-recap) within the square brackets. For example if you wanted to constrain the length of an array between `10` and `20` items you could do:

<CodeBlock code="u8[10..20]" />

## Maps

Maps are objects that have keys of one type, and values of another type.

Maps are defined using the `map` keyword, followed by a Luau-like map syntax. For example, a map of `string` keys and `u8` values would look like:

<CodeBlock code="map { [string]: u8 }" />

## Sets

Sets are equivalent to a map where all values are `true`, and are defined using the `set` keyword. For example, a map of `string` keys to `true` would look like:

<CodeBlock code="set { string }" />

## Enums

Zap has two types of enums, unit enums and tagged enums.

### Unit Enums

Unit enums are used to represent a set of possible values. They are defined using the `enum` keyword, followed by a set of possible string values. For example, a unit enum representing the status of a round would look like:

<CodeBlock code='type RoundStatus = enum { Starting, Playing, Intermission }' />

This code would then create the Luau type:

```lua
type RoundStatus = "Starting" | "Playing" | "Intermission"
```

### Tagged Enums

Tagged enums will be very familiar to Rust users.

Tagged enums are a set of possible variants, each with attached data. They are defined using the `enum` keyword, followed by a string which is the tag field name. After the tag field name, a set of variants are defined. Each variant is defined by a string tag, followed by a struct. Variants must be separated by a comma. Trailing commas are allowed.

<CodeBlock :code="enumExample" />

This code would create the Luau type:

```lua
type t = { type: "number", value: number }
	| { type: "string", value: string }
	| { type: "boolean", value: boolean }
```

Tagged enums allow you to pass different data depending on a variant. They are extremely powerful and can be used to represent many different types of data.

## Structs

Structs are similar to Interfaces, and are a collection of statically named fields with different types.

To define a struct, use the `struct` keyword followed by a Luau interface-like syntax. For example, a struct representing an item in a shop would look like:

<CodeBlock :code="structExample" />

## Instances

Roblox Instances can be passed through Zap.

::: danger
If a non-optional instance results in `nil` when received, it will cause a deserialize error and the packet will be dropped. Instances are turned into `nil` when they don't exist on the reciever - for example: an instance from the server that isn't streamed into a client or an instance that only exists on the client.

Immediately before making important changes (such as deletion or parenting to non-replicated storage) to non-optional `Instance`s that are referenced in events fired to clients, consider calling `SendEvents` to flush the outgoing queue. This way, even with batching, your Zap events are guaranteed to arrive before deletion replicates.

If you want to send an instance that may not exist, you must [make it optional](#optional-types).
:::

<CodeBlock code="Instance" />

You can also specify what kind of instance you want to accept, for example:

<CodeBlock code="type Part = Instance (BasePart)" />

Classes that inherit your specified class will be accepted, for example `Part`.

## CFrames

Zap supports sending CFrames. There are two types of CFrame you may send - a regular `CFrame`, and an `AlignedCFrame`.

CFrame rotation matrices are compressed using the axis-angle representation.

::: danger
CFrames are orthonormalized when sent. If you need to send a CFrame that is not orthogonal, i.e. one that does not have a valid rotation matrix, it is recommended to send the components and reconstruct it on the other side. Note that use cases for this are exceedingly rare and you most likely will not have to worry about this, as the common CFrame constructors only return orthogonal CFrames.
:::

### Aligned CFrames
When you know that a CFrame is going to be axis-aligned, it is preferrable to use the `AlignedCFrame` type. 

It uses much less bandwidth, as the rotation can just be represented as a single byte Enum of the possible axis aligned rotations.

You can think of an axis-aligned CFrame as one whose LookVector, UpVector, and RightVector all align with the world axes in some way. Even if the RightVector is facing upwards, for example, it would still be axis-aligned.

Position does not matter at all, only the rotation.

::: danger
If the CFrame is not axis aligned then Zap will throw an error, so make sure to use this type carefully! Don't let this dissuade you from using it though, as the bandwidth savings can be significant.
:::

Here are some examples of axis-aligned CFrames.
```lua
local CFrameSpecialCases = {
	CFrame.Angles(0, 0, 0),
	CFrame.Angles(0, math.rad(180), math.rad(90)),
	CFrame.Angles(0, math.rad(-90), 0),
	CFrame.Angles(math.rad(90), math.rad(-90), 0),
	CFrame.Angles(0, math.rad(90), math.rad(180)),
	-- and so on. there are 24 of these in total.
}
```

## Vectors `[0.6.17+]`

Zap supports `vector`s with any numeric components other than `f64`. The Z component is optional, and will result in a `0` if omitted.

<CodeBlock code="type Position = vector(f32, f32, f32)" />
<CodeBlock code="type Size = vector(u8, f32)" />

Omitting all components will emit `vector(f32, f32, f32)`.
<CodeBlock code="type Position = vector" />

Zap also supports serializing `Vector3`, and to not serialise the `Z` property of a `Vector3`, you can use the `Vector2` type.

## DateTimes

Zap supports sending DateTimes. There are two types of DateTime you may send - a regular `DateTime`, and `DateTimeMillis`.
DateTime sends the UnixTimestamp property of the DateTime object, with DateTimeMillis sending the UnixTimestampMillis property of the DateTime object.

## Other Roblox Classes

The following Roblox Classes are also available as types in Zap:

- `Color3`
- `BrickColor`

## Optional Types

A type can be made optional by appending a `?` after the **whole type**, such as:
<CodeBlock code="type Character = Instance (Player)?" />

## OR/Unions [0.6.20+]

Zap supports union types, or "OR" types. They work by checking the type of the
data at runtime, and serializing it correspondingly. Since the type is checked
at runtime, you're limited to using only one of the runtime type, that is you
cannot have both `u8` and `u16`.

For example, the following type:
<CodeBlock :code="orExample" />

Will check (& serialize the values separately) in the following way:

1. If it's `"hello"`
2. If it's `"world"`
3. If it's a table and it has `tag = "one"`
4. If it's a table and it has `tag = "two"`
5. If it's a string
6. If it's a number
7. If it's an instance, and of a Player
8. If it's an instance
9. If it's nil\*
10. If all else fails, serialize it as `unknown`

<sup>\*Unions being optional is implied by containing `unknown`.</sup>

Additionally, you may not use optional types inside the union itself, rather
you need to make the whole union optional.
