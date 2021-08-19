//! Implements the public traits that developers inherit from in order to properly utilize the
//! derive macro's functionality in code conversion and generation.

pub mod value;

use std::collections::BTreeMap;

// Alias for BTreeMap with String keys and generic values
pub type GenericMap = BTreeMap<String, value::Value>;

pub trait FromMap: Default {
    /// Converts a `GenericMap` back into a structure.
    /// __Constraints__: assumes that value types conform to the original types of the struct.
    fn from_genericmap(map: GenericMap) -> Self;
}

pub trait ToMap: Default {
    /// Generates a `GenericMap` where value types are all encapsulated under a sum type.
    /// __Constraints__: currently only supports primitive types for genericized values.
    #[allow(clippy::wrong_self_convention)]
    fn to_genericmap(structure: Self) -> GenericMap;
}

