---
outline: deep
---

# Types

## Numbers

There are there types of numbers in zap, unsigned (`u`), signed (`i`), and floats (`f`). Each number has a limit, in relation to the amount of bytes (space) the number utilises.

### Unsigned Numbers

| Type   | Min Value | Max Value                                                                 |
|--------|-----------|---------------------------------------------------------------------------|
| `u8`   | 0         | 255                                                                       |
| `u16`  | 0         | 65,535                                                                    |
| `u32`  | 0         | 4,294,967,295                                                             |
| `u64`  | 0         | <abbr title="18,446,744,073,709,551,615">~ 1.84 × 10<sup>19</sup></abbr>  |

### Signed Numbers

| Type   | Min Value                                                                 | Max Value                                                                 |
|--------|---------------------------------------------------------------------------|---------------------------------------------------------------------------|
| `i8`   | -128                                                                      | 127                                                                       |
| `i16`  | -32,768                                                                   | 32,767                                                                    |
| `i32`  | -2,147,483,648                                                            | 2,147,483,647                                                             |
| `i64`  | <abbr title="-9,223,372,036,854,775,808">~ -9.22 × 10<sup>18</sup></abbr> | <abbr title="9,223,372,036,854,775,807">~ 9.22 × 10<sup>18</sup></abbr>  |

### Float Numbers
Floats are floating point numbers. They are only in the 32 and 64 bit varients.

Generally, `f32` is precise enough for most usecases, but where further precision is necessary an `f64` can be used.

`f64` numbers can also store up to `2^53` before floating point inaccuracies occur.

### Constraining the Range

You can constrain the range of the number further than the byte limit in your config file.

Suppose we are trying to send the health of a player over the remote. We may define the type as:

<CodeBlock code="type Health = u8 (0..100)" />

Although, the type assertions will fail if 100 is passed. This is because **only the lower limit is inclusive**. To fix this, we must append a `=` to the end of the upper bound such as

<CodeBlock code="type Data = u8 (0..=100)" />

## Strings

Unlike numbers, strings do not have a maximum length across different types. They can be any length unless they are constrained.

<CodeBlock code="type Sign = string" />

### Constraining the Length

To constrain the length of a string, add the length after. For example, if you were validating a Roblox username:

<CodeBlock code="type Username = string (3..20)" />

## Arrays

Arrays can be defined as a type with two square brackets next to each other, such as:
<CodeBlock code="type Winners = u8[]" />

### Constraining the Length

Arrays can also be constrained with to a specific length, such as for pathfinding:

<CodeBlock code="type Path = u8[10..20]" />


## Structs
Structs are a collection of defined fields, with each field having its own type, such as: 

<CodeBlock :code="['type Item = {', '\tName: string,', '\tPrice: u16,', '}'].join('\n')" />

## Maps
Maps are objects where it is indexed by a type, such as:

<CodeBlock code="type Items = { [string]: Item }" />

## Enums

Enums are values seperated by a comma (`,`) inside brackets (`()`). For example:
<CodeBlock code="type RoundStatus = ( Playing, Intermission )" />

## Optional Types

A type can be made optional by appending a `?` after it, such as:
<CodeBlock code="type Round = u8?" />

## Roblox Classes
The following Roblox Classes are avaliable as types in zap:

- `Vector3`
