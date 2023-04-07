#![no_std]
#![warn(missing_docs, clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_safety_doc)]

//! Traits and macro to use newtype enums and convert between enums and their variants.
//!
//! A newtype enum is an enum where every variant wraps another type and the wrapped type can uniquely identify the variant.
//!
//! You can use the [`newtype_enum`](attr.newtype_enum.html) attribute macro to define a newtype enum. When the macro is applied to an enum `E` it will implement the [`Enum`](trait.Enum.html) trait for `E` and the [`Variant<E>`](trait.Variant.html) trait for all variant types.
//!
//! See the [**examples in the `Enum` trait**](trait.Enum.html) for usage of the available methods.
//!
//! The macro will also convert all unit and struct variants to generated structs and replace the enum variant with a newtype variant that contains the generated struct. See below for the rules and options that are available.
//!
//! # Variant transformation
//! The variants of the enum will be converted in the following way:
//!
//! ## Unit variants
//! ```
//! # use newtype_enum::newtype_enum;
//! #[newtype_enum]
//! enum Test {
//!     Example,
//! }
//! ```
//! ```
//! enum Test {
//!     Example(Test_variants::Example),
//! }
//!
//! mod Test_variants {
//!     pub(super) struct Example;
//! }
//! ```
//!
//! ## Newtype variants
//! ```
//! # use newtype_enum::newtype_enum;
//! #[newtype_enum]
//! enum Test {
//!     Example(usize),
//! }
//! ```
//! ```
//! enum Test {
//!     Example(usize),
//! }
//! ```
//! ## Struct variants
//! ```
//! # use newtype_enum::newtype_enum;
//! #[newtype_enum]
//! enum Test {
//!     Example { test: usize },
//! }
//! ```
//! ```
//! enum Test {
//!     Example(Test_variants::Example),
//! }
//!
//! mod Test_variants {
//!     pub(super) struct Example {
//!         pub(super) test: usize,
//!     }
//! }
//! ```
//!
//! # Attribute arguments
//! You can pass the following arguments to the `newtype_enum` macro:
//!
//! ## Variants module name
//! ```
//! # use newtype_enum::newtype_enum;
//! #[newtype_enum(variants = "test")]
//! enum Test {
//!     Example,
//! }
//! ```
//! ```
//! enum Test {
//!     Example(test::Example),
//! }
//!
//! mod test {
//!     // <-- the name of the generated module
//!     pub(super) struct Example;
//! }
//! ```
//! ## Variants module visibility
//! ```
//! # use newtype_enum::newtype_enum;
//! #[newtype_enum(variants = "pub(crate) test")]
//! enum Test {
//!     Example,
//! }
//! ```
//! ```
//! enum Test {
//!     Example(test::Example),
//! }
//!
//! pub(crate) mod test {
//!     // <-- the visibility of the generated module
//!     pub(super) struct Example;
//! }
//! ```
//!
//! # Visibilities and attributes (e.g. `#[derive]` attributes)
//! The visibility of the generated variant structs behaves as if they where part of a normal enum: All variants and their fields have the same visibiltiy scope as the enum itself.
//!
//! Attributes will be passed to the following locations:
//!
//! Location | Destination
//! -|-
//! enum | Enum and generated variant structs
//! enum variant | Generated variant struct
//! variant field | Generated struct field
//!
//! ```
//! # mod test {
//! # use newtype_enum::newtype_enum;
//! #[newtype_enum]
//! #[derive(Debug)]
//! pub(crate) enum Test {
//!     #[derive(Clone)]
//!     Example {
//!         test: usize,
//!         pub(super) test_super: usize,
//!         pub(self) test_self: usize,
//!     },
//! }
//! # }
//! ```
//! ```
//! # mod test {
//! #[derive(Debug)]
//! pub(crate) enum Test {
//!     Example(Test_variants::Example),
//! }
//!
//! pub(crate) mod Test_variants {
//!     #[derive(Debug, Clone)]
//!     pub(crate) struct Example {
//!         pub(crate) test: usize,
//!         pub(in super::super) test_super: usize,
//!         pub(super) test_self: usize,
//!     }
//! }
//! # }
//! ```

pub mod unstable;

/// Define a newtype enum.
///
/// See [crate-level documentation](index.html) for more information.
pub use newtype_enum_macro::newtype_enum;

/// Mark a type as an `enum`.
///
/// Use the [`newtype_enum`](attr.newtype_enum.html) macro to implement this trait for your enum types.
///
/// ```
/// # use newtype_enum::newtype_enum;
/// #[newtype_enum(variants = "pub example")]
/// #[derive(Debug)]
/// # #[derive(PartialEq, Eq)]
/// pub enum Test {
///     Ping,
///     Number(usize),
///     Str(&'static str),
///     #[derive(Clone)]
///     Hello {
///         name: &'static str,
///     },
/// }
///
/// use newtype_enum::Enum;
///
/// let test = Test::from_variant(example::Hello { name: "Tester" });
/// println!("{:?}", test);
///
/// let variant: example::Hello = test.into_variant().unwrap();
/// let cloned = variant.clone();
/// assert_eq!(variant, cloned);
/// ```
pub trait Enum: Sized {
    /// Construct an enum from one of its newtype variants.
    ///
    /// ```
    /// # #[newtype_enum::newtype_enum]
    /// # #[derive(Debug, PartialEq, Eq)]
    /// # pub enum Test {
    /// #     Number(usize),
    /// #     Str(&'static str),
    /// # }
    /// # fn main() {
    /// # use newtype_enum::Enum;
    /// let test = Test::from_variant(123);
    /// assert_eq!(test, Test::Number(123));
    /// # }
    /// ```
    fn from_variant(v: impl Variant<Self>) -> Self {
        v.into_enum()
    }

    /// Set the enum to one of its newtype variants.
    ///
    /// This returns the old value of the enum.
    ///
    /// ```
    /// # #[newtype_enum::newtype_enum]
    /// # #[derive(Debug, PartialEq, Eq)]
    /// # pub enum Test {
    /// #     Number(usize),
    /// #     Str(&'static str),
    /// # }
    /// # fn main() {
    /// # use newtype_enum::Enum;
    /// let mut test = Test::from_variant(123);
    ///
    /// let old = test.set_variant("Hello World");
    /// assert_eq!(old, Test::Number(123));
    /// assert_eq!(test, Test::Str("Hello World"));
    /// # }
    /// ```
    #[must_use]
    fn set_variant(&mut self, v: impl Variant<Self>) -> Self {
        core::mem::replace(self, v.into_enum())
    }

    /// Convert the enum into one of its newtype variants.
    ///
    /// ```
    /// # #[newtype_enum::newtype_enum]
    /// # #[derive(Debug, PartialEq, Eq)]
    /// # pub enum Test {
    /// #     Number(usize),
    /// #     Str(&'static str),
    /// # }
    /// # fn main() {
    /// # use newtype_enum::Enum;
    /// let create_test = || Test::from_variant(123);
    ///
    /// assert_eq!(create_test().into_variant(), Some(123));
    ///
    /// let variant: Option<&str> = create_test().into_variant();
    /// assert_eq!(variant, None);
    ///
    /// assert_eq!(create_test().into_variant::<&str>(), None);
    /// # }
    /// ```
    fn into_variant<V: Variant<Self>>(self) -> Option<V> {
        V::from_enum(self)
    }

    /// Get a reference to one of its newtype variants.
    ///
    /// ```
    /// # #[newtype_enum::newtype_enum]
    /// # #[derive(Debug, PartialEq, Eq)]
    /// # pub enum Test {
    /// #     Number(usize),
    /// #     Str(&'static str),
    /// # }
    /// # fn main() {
    /// # use newtype_enum::Enum;
    /// let test = Test::from_variant(123);
    /// assert_eq!(test.variant(), Some(&123));
    ///
    /// let variant: Option<&&str> = test.variant();
    /// assert_eq!(variant, None);
    ///
    /// assert_eq!(test.variant::<&str>(), None);
    /// # }
    /// ```
    fn variant<V: Variant<Self>>(&self) -> Option<&V> {
        V::ref_enum(self)
    }

    /// Get a mutable reference to one of its newtype variants.
    ///
    /// ```
    /// # #[newtype_enum::newtype_enum]
    /// # #[derive(Debug, PartialEq, Eq)]
    /// # pub enum Test {
    /// #     Number(usize),
    /// #     Str(&'static str),
    /// # }
    /// # fn main() {
    /// # use newtype_enum::Enum;
    /// let mut test = Test::from_variant(123);
    /// assert_eq!(test.variant_mut(), Some(&mut 123));
    /// assert_eq!(test.variant_mut(), None::<&mut &str>);
    ///
    /// if let Some(mut variant) = test.variant_mut() {
    ///     *variant = 42;
    /// }
    /// assert_eq!(test.into_variant(), Some(42));
    /// # }
    /// ```
    fn variant_mut<V: Variant<Self>>(&mut self) -> Option<&mut V> {
        V::mut_enum(self)
    }

    /// Check if the enum currently holds the newtype variant `V`.
    ///
    /// If this method returns `true`, it is safe to call one of the `variant_unchecked` methods.
    ///
    /// ```
    /// # #[newtype_enum::newtype_enum]
    /// # #[derive(Debug, PartialEq, Eq)]
    /// # pub enum Test {
    /// #     Number(usize),
    /// #     Str(&'static str),
    /// # }
    /// # fn main() {
    /// # use newtype_enum::Enum;
    /// let mut test = Test::from_variant(123);
    /// assert_eq!(test.is_variant::<usize>(), true);
    /// assert_eq!(test.is_variant::<&str>(), false);
    /// # }
    /// ```
    fn is_variant<V: Variant<Self>>(&self) -> bool {
        V::is_enum_variant(self)
    }

    /// Convert the enum into one of its newtype variants and unwrap the value.
    ///
    /// This method is equivalent to `self.into_variant().unwrap()` but written without an intermediate `Option<V>` value.
    /// Therefore the compiler can sometimes optimize the code better.
    ///
    /// ```
    /// # #[newtype_enum::newtype_enum]
    /// # #[derive(Debug, PartialEq, Eq)]
    /// # pub enum Test {
    /// #     Number(usize),
    /// #     Str(&'static str),
    /// # }
    /// # fn main() {
    /// # use newtype_enum::Enum;
    /// let mut test = Test::from_variant(123);
    /// let variant: usize = test.into_variant_unwrap();
    /// assert_eq!(variant, 123);
    /// # }
    /// ```
    ///
    /// ```should_panic
    /// # #[newtype_enum::newtype_enum]
    /// # #[derive(Debug, PartialEq, Eq)]
    /// # pub enum Test {
    /// #     Number(usize),
    /// #     Str(&'static str),
    /// # }
    /// # fn main() {
    /// # use newtype_enum::Enum;
    /// let mut test = Test::from_variant("Hello World");
    /// let variant: usize = test.into_variant_unwrap(); // fails
    /// # }
    /// ```
    fn into_variant_unwrap<V: Variant<Self>>(self) -> V {
        V::from_enum_unwrap(self)
    }

    /// Convert the enum into one of its newtype variants **without checking if the variant matches**.
    ///
    /// ```
    /// # #[newtype_enum::newtype_enum]
    /// # #[derive(Debug, PartialEq, Eq)]
    /// # pub enum Test {
    /// #     Number(usize),
    /// #     Str(&'static str),
    /// # }
    /// # fn main() {
    /// # use newtype_enum::Enum;
    /// let test = Test::from_variant(123);
    ///
    /// if test.is_variant::<usize>() {
    ///     // ...
    ///
    ///     // SAFETY: We already checked if the enum has the correct variant
    ///     // and we did not change it in between.
    ///     let number: usize = unsafe { test.into_variant_unchecked() };
    ///     assert_eq!(number, 123);
    /// } else {
    ///     panic!("expected a usize variant");
    /// }
    /// # }
    /// ```
    unsafe fn into_variant_unchecked<V: Variant<Self>>(self) -> V {
        V::from_enum_unchecked(self)
    }

    /// Get a reference to one of its newtype variants **without checking if the variant matches**.
    ///
    /// ```
    /// # #[newtype_enum::newtype_enum]
    /// # #[derive(Debug, PartialEq, Eq)]
    /// # pub enum Test {
    /// #     Number(usize),
    /// #     Str(&'static str),
    /// # }
    /// # fn main() {
    /// # use newtype_enum::Enum;
    /// let test = Test::from_variant(123);
    ///
    /// if test.is_variant::<usize>() {
    ///     // ...
    ///
    ///     // SAFETY: We already checked if the enum has the correct variant
    ///     // and we did not change it in between.
    ///     let number: &usize = unsafe { test.variant_unchecked() };
    ///     assert_eq!(number, &123);
    /// } else {
    ///     panic!("expected a usize variant");
    /// }
    /// # }
    /// ```
    unsafe fn variant_unchecked<V: Variant<Self>>(&self) -> &V {
        V::ref_enum_unchecked(self)
    }

    /// Get a mutable reference to one of its newtype variants **without checking if the variant matches**.
    ///
    /// ```
    /// # #[newtype_enum::newtype_enum]
    /// # #[derive(Debug, PartialEq, Eq)]
    /// # pub enum Test {
    /// #     Number(usize),
    /// #     Str(&'static str),
    /// # }
    /// # fn main() {
    /// # use newtype_enum::Enum;
    /// let mut test = Test::from_variant(123);
    ///
    /// if test.is_variant::<usize>() {
    ///     // ...
    ///
    ///     // SAFETY: We already checked if the enum has the correct variant
    ///     // and we did not change it in between.
    ///     let number: &mut usize = unsafe { test.variant_unchecked_mut() };
    ///     *number = 42;
    /// } else {
    ///     panic!("expected a usize variant");
    /// }
    ///
    /// assert_eq!(test.into_variant(), Some(42));
    /// # }
    /// ```
    unsafe fn variant_unchecked_mut<V: Variant<Self>>(&mut self) -> &mut V {
        V::mut_enum_unchecked(self)
    }
}

/// Mark a type as a newtype variant of an [`Enum`](trait.Enum.html) `E`.
///
/// Use the [`newtype_enum`](attr.newtype_enum.html) macro to implement this trait for your enum variants.
pub trait Variant<E: Enum>: unstable::VariantCore<E> {}
