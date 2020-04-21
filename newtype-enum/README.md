[![Crates.io](https://img.shields.io/crates/v/newtype-enum.svg)](https://crates.io/crates/newtype-enum)
[![API Documentation](https://docs.rs/newtype-enum/badge.svg)](https://docs.rs/newtype-enum)
[![Workflow Status](https://github.com/mgjm/newtype-enum/workflows/build/badge.svg)](https://github.com/mgjm/newtype-enum/actions?query=workflow%3A%22build%22)

# newtype-enum

Traits and macro to use newtype enums and convert between enums and their variants.

A newtype enum is an enum where every variant wraps another type and the wrapped type can uniquely identify the variant.

You can use the [`newtype_enum`](attr.newtype_enum.html) attribute macro to define a newtype enum. When the macro is applied to an enum `E` it will implement the [`Enum`](trait.Enum.html) trait for `E` and the [`Variant<E>`](trait.Variant.html) trait for all variant types.

See the [**examples in the `Enum` trait**](trait.Enum.html) for usage of the available methods.

The macro will also convert all unit and struct variants to generated structs and replace the enum variant with a newtype variant that contains the generated struct. See below for the rules and options that are available.

## Variant transformation
The variants of the enum will be converted in the following way:

### Unit variants
```rust
#[newtype_enum]
enum Test {
    Example,
}
```
```rust
enum Test {
    Example(Test_variants::Example),
}

mod Test_variants {
    pub(super) struct Example;
}
```

### Newtype variants
```rust
#[newtype_enum]
enum Test {
    Example(usize),
}
```
```rust
enum Test {
    Example(usize),
}
```
### Struct variants
```rust
#[newtype_enum]
enum Test {
    Example { test: usize },
}
```
```rust
enum Test {
    Example(Test_variants::Example),
}

mod Test_variants {
    pub(super) struct Example {
        pub(super) test: usize,
    }
}
```

## Attribute arguments
You can pass the following arguments to the `newtype_enum` macro:

### Variants module name
```rust
#[newtype_enum(variants = "test")]
enum Test {
    Example,
}
```
```rust
enum Test {
    Example(test::Example),
}

mod test {
    // <-- the name of the generated module
    pub(super) struct Example;
}
```
### Variants module visibility
```rust
#[newtype_enum(variants = "pub(crate) test")]
enum Test {
    Example,
}
```
```rust
enum Test {
    Example(test::Example),
}

pub(crate) mod test {
    // <-- the visibility of the generated module
    pub(super) struct Example;
}
```

## Visibilities and attributes (e.g. `#[derive]` attributes)
The visibility of the generated variant structs behaves as if they where part of a normal enum: All variants and their fields have the same visibiltiy scope as the enum itself.

Attributes will be passed to the following locations:

Location | Destination
-|-
enum | Enum and generated variant structs
enum variant | Generated variant struct
variant field | Generated struct field

```rust
#[newtype_enum]
#[derive(Debug)]
pub(crate) enum Test {
    #[derive(Clone)]
    Example {
        test: usize,
        pub(super) test_super: usize,
        pub(self) test_self: usize,
    },
}
```
```rust
#[derive(Debug)]
pub(crate) enum Test {
    Example(Test_variants::Example),
}

pub(crate) mod Test_variants {
    #[derive(Debug, Clone)]
    pub(crate) struct Example {
        pub(crate) test: usize,
        pub(in super::super) test_super: usize,
        pub(super) test_self: usize,
    }
}
```

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
