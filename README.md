# Glossary

- ELV: Enzo Le Van
- WF: Wire Format
- WFE: Wire Format Error
- WFU: Wire Format Utils

# Concepts

On the wire this crate encodes 3 kind of message:
- a struct `#[wf(struct(..))]` applied on `struct` in Rust
- a union `#[wf(union(..))]` applied on `enum` in Rust
- an enum `#[wf(enum(..))]` applied on `enum` in Rust

## struct

on the wire a struct is a message composed of any possible object, including:
- scalars (which are a kind of struct message but not exposed as such)
- slices (same.)
- any message (struct, union or enum)

```rust
#[derive(Wired)]
#[wf(struct(align(u32), header(dsl = "ID:8=0x1")))]
struct MyStruct<'a> {
    #[wf(scalar(format(le)))]
    value: u32,
    #[wf(slice(len(prefixed)))]
    payload: &'a [u8]  
}
```

## union

On the wire a union is an object that can be any one of several possible struct messages
the actual variant present on the wire is not known and must be discovered at decode time
so every possible variant of a union must be a struct message that exposes an ID in its header (via #[wf(struct(header(...)))])
it's used as a way to group struct messages of the same kind together

**Caution**: on the Rust side this object is of course represented by an enum!

```rust
#[derive(Wired)]
#[wf(union(discriminant = ID))]
enum MyUnion<'a> {
    Msg1(MyMsg1),
    Msg2(MyMsg2),
}
```

## enum

On the wire an enum is just an integer (unsigned or signed). 
it's used to represent a code: an error code, a value from a known list, etc...

```rust
#[derive(Wired)]
#[wf(enum(repr(u32), format(be)))]
enum MyEnum {
    Code1 = 0,
    Code2 = 1
}
```

# Features

- choose alignment (16-bit, 32-bit or 64-bit)
- optional header with readable DSL (8-bit, 16-bit, 32-bit or 64-bit) to define flags, slots and constants
- header format (native-endian, little-endian or big-endian)
- header offset to add arbitrary value in case you need one
- header flattening (between parent and field)
- support for all scalars
- scalar precise formating (native-endian, little-endian, big-endian or variable-length)
- store scalar in a header slot instead of inside the body
- support for slices: `&[u8]`, `&[u8; N]`, `&str` and `&CStr`
- precise how to store or know the slice len (in a header slot, in the body or as the remaining size)
- support for all other kind of messages built on top of scalars and slices
- support for optional fields: `Option<T>` with the `#[wf(opt(..))` attribute
- support for conditional fields: `T: PartialEq` compared to another value
- support for bound checking with `#[wf(bounded(low = .., high = ..))]`
- `Randomized` derive-macro for testing purpose (generate fake messages according to the boundaries if set)

The `Wired` and `Randomized` derive-macros have been made so that the generated code is really simple and easy to understand, so that if this crate lacks a feature you need to express a protocol message, you can express as mush as possible with this crate, expand/flatten the generated code and make edits yourself. You can use the `WiredBlank` derive-macro to keep the annotations `#[wf]` and even add your own "blank" attributes to see how it would be possible to express your feature.

## Example

```rust
#[derive(Wired, Randomized, Clone, Copy, Debug, PartialEq)]
#[wf(enum(repr(u32), format(le)))]
enum MyEnum {
    V1 = 1,
    V2 = 2,
    V3 = 3,
    V4 = 4,
    V5 = 5,
}

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
#[wf(struct(header(dsl = "S:16|D:16")))]
struct MyMsg<'a, const N: usize> {
    #[wf(slice(len(slot = S), bounded(low = 1, high = 18)))]
    name: &'a str,

    #[wf(scalar(format(le), bounded(low = 0, high = u32::MAX - 1)))]
    value: u32,

    #[wf(scalar(slot = D, bounded(low = 1, high = u16::MAX as u32)))]
    extra_value: u32,

    #[wf(msg(len(embedded)))]
    code: MyEnum,

    #[wf(slice(len(embedded)))]
    extra_slice: &'a [u8; N],

    #[wf(slice(len(remaining)))]
    payload: &'a [u8]
}
```

See the [examples](./examples) and [tests](./tests/pass.rs) for other examples.
