//! Unstable traits and types.
//!
//! All traits and types in this module are unstable. They could change in the future.

use crate::Enum;
use core::hint::unreachable_unchecked;

/// Mark a type as a newtype variant of an [`Enum`](../trait.Enum.html) `E`.
///
/// Use the [`newtype_enum`](../attr.newtype_enum.html) macro to implement this trait for your enum variants.
///
/// **NOTE**: All methods in this trait are unstable. Use their counterpart in the [`Enum`](../trait.Enum.html) trait.
pub trait VariantCore<E: Enum>: Sized {
    /// Convert this newtype variant into the enum `E`.
    fn into_enum(self) -> E;

    /// Convert an enum into this newtype variant.
    fn from_enum(e: E) -> Option<Self>;

    /// Get a reference to this this newtype variant.
    fn ref_enum(e: &E) -> Option<&Self>;

    /// Get a mutable reference to this this newtype variant.
    fn mut_enum(e: &mut E) -> Option<&mut Self>;

    /// Check if an enum currently holds this newtype variant.
    ///
    /// If this method returns `true`, it is safe to call one of the `enum_unchecked` methods.
    fn is_enum_variant(e: &E) -> bool {
        Self::ref_enum(e).is_some()
    }

    /// Convert an enum into this newtype variants and unwrap the value.
    ///
    /// This method is equivalent to `Self::from(e).unwrap()`.
    ///
    /// Implementors **should** write this method without an intermediate `Option<V>` value.
    /// This sometimes allows the compiler to optimize the code better.
    fn from_enum_unwrap(e: E) -> Self {
        match Self::from_enum(e) {
            Some(v) => v,
            None => panic!("called `Variant::from_enum_unwrap` on another enum variant"),
        }
    }

    /// Convert an enum into this newtype variant.
    unsafe fn from_enum_unchecked(e: E) -> Self {
        match Self::from_enum(e) {
            Some(v) => v,
            None => unreachable_unchecked(),
        }
    }

    /// Get a reference to this this newtype variant.
    unsafe fn ref_enum_unchecked(e: &E) -> &Self {
        match Self::ref_enum(e) {
            Some(v) => v,
            None => unreachable_unchecked(),
        }
    }

    /// Get a mutable reference to this this newtype variant.
    unsafe fn mut_enum_unchecked(e: &mut E) -> &mut Self {
        match Self::mut_enum(e) {
            Some(v) => v,
            None => unreachable_unchecked(),
        }
    }
}
