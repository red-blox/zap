# Events

Events are the core function of Zap, and can be defined through the following fields in the struct. Each field is optional and not required.

## Example Event

<CodeBlock :code="['event RoundReady = {', '\tfrom: Client,', '\ttype: Reliable,', '\tcall: SingleSync,', '\tdata: bool,', '}'].join('\n')" />

## From

### Default

`Server`

### Options

- `Client`
- `Server`

### Example

<CodeBlock :code="'\tfrom: Server,'" />

## Type

Type determines if the event should be considered reliable or unreliable. Reliable events are ordered and guaranteed to deliver, while unreliable events are unordered and are not guaranteed to deliver.

### Default

`Reliable`

### Options

- `Reliable`
- `Unreliable`

### Example

<CodeBlock :code="'\ttype: Reliable,'" />

## Call

### Default

`SingleSync`

### Options

- `SingleSync`
- `SingleAsync`
- `ManySync`
- `ManyAsync`

### Example

<CodeBlock :code="'\tcall: SingleSync,'" />

## Data

### Default

`bool`

### Options

Any 

### Example

<CodeBlock :code="'\tdata: bool,'" />

<CodeBlock :code="['event Item = {', '\tfrom: Client,', '\ttype: Reliable,', '\tcall: SingleSync,', '\tdata: String,', '}'].join('\n')" />
