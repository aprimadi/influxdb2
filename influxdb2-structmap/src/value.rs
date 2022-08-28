use std::any::Any;
use std::fmt;

use chrono::{DateTime, FixedOffset};
use num_traits::cast::ToPrimitive;
use ordered_float::OrderedFloat;

/// Represents primitive types that are supported for conversion into a BTreeMap that can support
/// heterogeneous values. Inspired by `serde_json::Value`s.
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum Value {
    Unknown,
    String(String),
    Double(OrderedFloat<f64>),
    Bool(bool),
    Long(i64),
    UnsignedLong(u64),
    Duration(chrono::Duration),
    Base64Binary(Vec<u8>),
    TimeRFC(DateTime<FixedOffset>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Value {
    /// Given a genericized input type, encapsulate it as a Value that can be used in a map
    /// container type when converting to and from a struct.
    pub fn new<T: Any>(value: T) -> Value {
        let any_val = &value as &dyn Any;
        if let Some(val) = any_val.downcast_ref::<f64>() {
            Value::Double(OrderedFloat::from(*val))
        } else if let Some(val) = any_val.downcast_ref::<bool>() {
            Value::Bool(*val)
        } else if let Some(val) = any_val.downcast_ref::<i64>() {
            Value::Long(*val)
        } else if let Some(val) = any_val.downcast_ref::<u64>() {
            Value::UnsignedLong(*val)
        } else if let Some(val) = any_val.downcast_ref::<chrono::Duration>() {
            Value::Duration(*val)
        } else if let Some(val) = any_val.downcast_ref::<Vec<u8>>() {
            Value::Base64Binary(val.clone())
        } else if let Some(val) = any_val.downcast_ref::<DateTime<FixedOffset>>() {
            Value::TimeRFC(*val)
        } else if let Some(val) = any_val.downcast_ref::<String>() {
            Value::String(val.to_string())
        } else {
            Value::Unknown
        }
    }

    pub fn downcast<T>(&self) -> Option<T> {
        None
    }

    pub fn bool(&self) -> Option<bool> {
        if let Value::Bool(val) = self {
            Some(*val)
        } else {
            None
        }
    }

    pub fn i64(&self) -> Option<i64> {
        if let Value::Long(val) = self {
            Some(*val)
        } else {
            None
        }
    }

    pub fn u64(&self) -> Option<u64> {
        if let Value::UnsignedLong(val) = self {
            Some(*val)
        } else {
            None
        }
    }

    pub fn f64(&self) -> Option<f64> {
        if let Value::Double(val) = self {
            val.to_f64()
        } else {
            None
        }
    }

    pub fn string(&self) -> Option<String> {
        if let Value::String(string) = self {
            Some(string.to_string())
        } else {
            None
        }
    }
}

