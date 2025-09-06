//! InfluxDB Writable trait
//!
//! Trying to construct the trait used for line protocol
//! https://docs.influxdata.com/influxdb/v2.6/reference/syntax/line-protocol/#Copyright

use influxdb2_derive::{impl_tuple_fields, impl_tuple_tags};

/// InfluxDB WritableValue trait
///
/// This type normally descript the type which could be written as FieldValue.
/// Currently u64, f64, String, str, bool are supported
/// The influxdb support value type is
/// Value data type: Float | Integer | UInteger | String | Boolean
pub trait ValueWritable {
    /// encode_value into influxdb support string
    fn encode_value(&self) -> String;
}

struct Value<T: ValueWritable> {
    inner: T,
}

impl<T: ValueWritable> From<T> for Value<T> {
    fn from(value: T) -> Self {
        Self { inner: value }
    }
}
impl<T: ValueWritable> ValueWritable for Value<T> {
    fn encode_value(&self) -> String {
        self.inner.encode_value()
    }
}

impl ValueWritable for f64 {
    fn encode_value(&self) -> String {
        self.to_string()
    }
}

impl ValueWritable for i64 {
    fn encode_value(&self) -> String {
        format!("{self}i")
    }
}

impl ValueWritable for u64 {
    fn encode_value(&self) -> String {
        format!("{self}u")
    }
}

impl ValueWritable for String {
    fn encode_value(&self) -> String {
        format!("\"{}\"", self.replace("\\", "\\\\").replace("\"", "\\\""))
    }
}

impl ValueWritable for &str {
    fn encode_value(&self) -> String {
        format!("\"{}\"", self.replace("\\", "\\\\").replace("\"", "\\\""))
    }
}

impl ValueWritable for bool {
    fn encode_value(&self) -> String {
        // bool type in influxdb2
        // https://docs.influxdata.com/influxdb/v2.6/reference/syntax/line-protocol/#boolean
        if *self {
            "t".to_string()
        } else {
            "f".to_string()
        }
    }
}

impl<T: ValueWritable> ValueWritable for Option<T> {
    fn encode_value(&self) -> String {
        match self {
            Some(v) => v.encode_value(),
            None => "\"None\"".to_string(),
        }
    }
}

/// InfluxDB Key
///
/// This type normally descript the type which could be written as TagKey, TagValue or FieldKey.
///
/// Influx support type:
/// Key data type: String
/// Value data type: String
pub trait KeyWritable {
    /// encode key as string
    fn encode_key(&self) -> String;
}

struct Key<T: KeyWritable> {
    inner: T,
}

impl<T: KeyWritable> From<T> for Key<T> {
    fn from(value: T) -> Self {
        Self { inner: value }
    }
}

impl<T: KeyWritable> KeyWritable for Key<T> {
    fn encode_key(&self) -> String {
        self.inner.encode_key()
    }
}

impl KeyWritable for &str {
    fn encode_key(&self) -> String {
        self.to_string()
    }
}

impl KeyWritable for String {
    fn encode_key(&self) -> String {
        self.clone()
    }
}

impl KeyWritable for u64 {
    fn encode_key(&self) -> String {
        self.to_string()
    }
}

impl<T: KeyWritable> KeyWritable for Option<T> {
    fn encode_key(&self) -> String {
        match self {
            Some(v) => v.encode_key(),
            None => "None".to_string(),
        }
    }
}

/// Write tags as key=value
pub trait TagsWritable {
    /// encode tags into string
    fn encode_tags(&self) -> String;
}

impl_tuple_tags!((T1, T2));
impl_tuple_tags!((T1, T2, T3, T4));
impl_tuple_tags!((T1, T2, T3, T4, T5, T6));

/// Write tags as field=value
pub trait FieldsWritable {
    /// encode fields into string
    fn encode_fields(&self) -> String;
}

impl_tuple_fields!((K1, V1));
impl_tuple_fields!((K1, V1, K2, V2));
impl_tuple_fields!((K1, V1, K2, V2, K3, V3));
impl_tuple_fields!((K1, V1, K2, V2, K3, V3, K4, V4));
impl_tuple_fields!((K1, V1, K2, V2, K3, V3, K4, V4, K5, V5));
impl_tuple_fields!((K1, V1, K2, V2, K3, V3, K4, V4, K5, V5, K6, V6));
impl_tuple_fields!((K1, V1, K2, V2, K3, V3, K4, V4, K5, V5, K6, V6, K7, V7));

/// Any type wants to be a timestamp needs to implement this
pub trait TimestampWritable {
    /// encode into string like "1465839830100400200"
    fn encode_timestamp(&self) -> String;
}

impl TimestampWritable for u64 {
    fn encode_timestamp(&self) -> String {
        self.to_string()
    }
}

impl TimestampWritable for i64 {
    fn encode_timestamp(&self) -> String {
        self.to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::writable::{FieldsWritable, TagsWritable};

    use super::ValueWritable;

    #[test]
    fn value_writable_f64() {
        let a: f64 = 33.33;
        assert_eq!(a.encode_value(), "33.33")
    }

    #[test]
    fn value_writable_i64() {
        let a: i64 = 33;
        assert_eq!(a.encode_value(), "33i")
    }

    #[test]
    fn tags_tuple() {
        let a: (&str, &str) = ("33", "str");
        assert_eq!(a.encode_tags(), "33=str");
        let b: (String, &str, &str, &str) = ("ff".to_string(), "aa", "bb", "cc");
        assert_eq!(b.encode_tags(), "ff=aa,bb=cc")
    }

    #[test]
    fn fields_tuple() {
        let a: (&str, u64) = ("ddf", 33);
        assert_eq!(a.encode_fields(), "ddf=33u");
        let a = ("ddf", 33u64, "gg", true);
        assert_eq!(a.encode_fields(), "ddf=33u,gg=t");
        let a = ("ddf", 33u64, "gg", true, "cc", 44.44f64, "dd", 22i64);
        assert_eq!(a.encode_fields(), "ddf=33u,gg=t,cc=44.44,dd=22i");
    }
}
