use core::fmt;
use std::cmp;
use std::fmt::Formatter;
use std::marker::PhantomData;

use serde::Deserialize;
use serde::Deserializer;
use serde::de::MapAccess;
use serde::de::Visitor;
use serde::ser::Serialize;
use serde::ser::SerializeStruct;
use serde::ser::Serializer;

/// Java-style (using "value" field) optional type.
// #[derive(Serialize, Deserialize)]
// #[serde(bound = "T: Serialize + DeserializeOwned")]
#[derive(Copy, Eq, Debug)]
pub enum Optional<T> {
    Some { value: T },

    None,
}

impl<T> Optional<&T> {
    pub const fn copied(self) -> Optional<T>
    where
        T: Copy,
    {
        match self {
            Optional::Some { value: &v } => Optional::Some { value: v },
            Optional::None => Optional::None,
        }
    }

    pub fn cloned(self) -> Optional<T>
    where
        T: Clone,
    {
        match self {
            Optional::Some { value: v } => Optional::Some { value: v.clone() },
            Optional::None => Optional::None,
        }
    }
}
impl<T> Optional<&mut T> {
    pub const fn copied(self) -> Optional<T>
    where
        T: Copy,
    {
        match self {
            Optional::Some { value: &mut t } => Optional::Some { value: t },
            Optional::None => Optional::None,
        }
    }

    pub fn cloned(self) -> Optional<T>
    where
        T: Clone,
    {
        match self {
            Optional::Some { value: t } => Optional::Some { value: t.clone() },
            Optional::None => Optional::None,
        }
    }
}

impl<T: Clone> Clone for Optional<T> {
    #[inline]
    fn clone(&self) -> Self {
        match self {
            Optional::Some { value } => Optional::Some {
                value: value.clone(),
            },
            Optional::None => Optional::None,
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        match (self, source) {
            (Optional::Some { value: to }, Optional::Some { value: from }) => to.clone_from(from),
            (to, from) => *to = from.clone(),
        }
    }
}

impl<T: PartialEq> PartialEq for Optional<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Optional::Some { value: v1 }, Optional::Some { value: v2 }) => *v1 == *v2,
            (Optional::Some { .. }, Optional::None) => false,
            (Optional::None, Optional::Some { .. }) => false,
            (Optional::None, Optional::None) => true,
        }
    }
}

#[allow(dead_code)]
impl<T: PartialOrd> Optional<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match (self, other) {
            (Optional::Some { value: v1 }, Optional::Some { value: v2 }) => v1.partial_cmp(v2),
            (Optional::Some { .. }, Optional::None) => Some(cmp::Ordering::Greater),
            (Optional::None, Optional::Some { .. }) => Some(cmp::Ordering::Less),
            (Optional::None, Optional::None) => Some(cmp::Ordering::Equal),
        }
    }
}

// impl<T: Ord> Ord for Optional<T> {
//     #[inline]
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         match (self, other) {
//             (Optional::Some { value: v1 }, Optional::Some { value: v2 }) => v1.cmp(v2),
//             (Optional::Some { .. }, Optional::None) => std::cmp::Ordering::Greater,
//             (Optional::None, Optional::Some { .. }) => std::cmp::Ordering::Less,
//             (Optional::None, Optional::None) => std::cmp::Ordering::Equal,
//         }
//     }
// }

impl<T> Default for Optional<T> {
    #[inline]
    fn default() -> Self {
        Optional::None
    }
}

impl<T> fmt::Display for Optional<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Optional::Some { value } => write!(f, "Some{{ value: {value} }}"),
            Optional::None => write!(f, "None"),
        }
    }
}

impl<T> From<Option<T>> for Optional<T> {
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(value) => Optional::Some { value },
            None => Optional::None,
        }
    }
}

impl<T> From<Optional<T>> for Option<T> {
    fn from(opt: Optional<T>) -> Self {
        match opt {
            Optional::Some { value } => Some(value),
            Optional::None => None,
        }
    }
}

struct OptionalVisitor<T>(PhantomData<T>);
impl<'de, T: Deserialize<'de>> Visitor<'de> for OptionalVisitor<T> {
    type Value = Optional<T>;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("enum Optional::Some")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        if let Some((key, value)) = map.next_entry::<String, T>()? {
            return if key == "value" {
                Ok(Optional::Some::<T> { value })
            } else {
                Err(serde::de::Error::custom(format!("unexpected key: {key}")))
            };
        }
        Ok(Optional::None)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Optional<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer
            .deserialize_map(OptionalVisitor(PhantomData))
            .or(Ok(Optional::None))
    }
}

impl<T: Serialize> Serialize for Optional<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Optional::Some { value } => {
                // let value = value.serialize(serializer);
                let mut some = serializer.serialize_struct("Optional", 1)?;
                some.serialize_field("value", &value)?;
                some.end()
            }
            Optional::None => serializer.serialize_none(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_serde() {
        let some: Optional<i32> = Optional::Some { value: 42 };
        let none: Optional<i32> = Optional::None;

        let some_json = serde_json::to_string(&some).unwrap();
        let none_json = serde_json::to_string(&none).unwrap();

        assert_eq!(some_json, r#"{"value":42}"#);
        assert_eq!(none_json, "null");

        let some_de: Optional<i32> = serde_json::from_str(&some_json).unwrap();
        let none_de: Optional<i32> = serde_json::from_str(&none_json).unwrap();

        // assert_eq!(some, some_de);
        assert_eq!(none, none_de);
    }

    #[test]
    fn test_json_serde_option() {
        let some: Option<i32> = Some(42);
        let none: Option<i32> = None;

        let some_json = serde_json::to_string(&some).unwrap();
        let none_json = serde_json::to_string(&none).unwrap();

        assert_eq!(some_json, "42");
        assert_eq!(none_json, "null");

        let some_de: Option<i32> = serde_json::from_str(&some_json).unwrap();
        let none_de: Option<i32> = serde_json::from_str(&none_json).unwrap();

        assert_eq!(some, some_de);
        assert_eq!(none, none_de);
    }

    #[test]
    fn test_optional_display() {
        let some: Optional<i32> = Optional::Some { value: 42 };
        let none: Optional<i32> = Optional::None;

        assert_eq!(some.to_string(), "Some{ value: 42 }");
        assert_eq!(none.to_string(), "None");
    }

    #[test]
    fn test_optional_from_option() {
        let some: Optional<i32> = Optional::from(Some(42));
        let none: Optional<i32> = Optional::from(None);

        assert_eq!(some, Optional::Some { value: 42 });
        assert_eq!(none, Optional::None);
    }

    #[test]
    fn test_option_from_optional() {
        let some: Option<String> = Option::from(Optional::Some {
            value: "str".to_string(),
        });
        let none: Option<i32> = Option::from(Optional::None);

        assert_eq!(some, Some("str".to_string()));
        assert_eq!(none, None);
    }

    #[test]
    fn test_optional_default() {
        let default: Optional<i32> = Default::default();
        assert_eq!(default, Optional::None);
    }

    // #[test]
    // fn test_optional_ordering() {
    //     let a = Optional::Some { value: 1 };
    //     let b = Optional::Some { value: 2 };
    //     let c = Optional::None;
    //
    //     assert!(a < b);
    //     assert!(b > a);
    //     assert!(c < a);
    //     assert!(c > b);
    //     assert_eq!(c, Optional::None);
    // }
    //
    // #[test]
    // fn test_option_ordering() {
    //     let a = Some(1);
    //     let b = Some(2);
    //     let c = None;
    //
    //     assert!(a < b);
    //     assert!(b > a);
    //     assert!(c < a);
    //     assert!(c < b);
    //     assert_eq!(c, None);
    // }
}
