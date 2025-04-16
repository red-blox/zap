-- type foo = foo

-- event Simple = {
--     from: Client,
--     type: Reliable,
--     call: SingleSync,
--     data: foo
-- }

type bar = struct { baz: bar? }

event Complex = {
    from: Client,
    type: Reliable,
    call: SingleSync,
    data: bar
}