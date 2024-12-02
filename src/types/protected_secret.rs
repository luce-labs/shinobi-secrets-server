use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::ops::Deref;

#[derive(Clone, Serialize, Deserialize)]
pub struct ProtectedValue(String);

impl Deref for ProtectedValue {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq<str> for ProtectedValue {
    fn eq(&self, other: &str) -> bool {
        &self.0 == other
    }
}

impl fmt::Display for ProtectedValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[PROTECTED]")
    }
}

impl fmt::Debug for ProtectedValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[PROTECTED]")
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ProtectedSecret {
    value: Option<ProtectedValue>,
}

impl ProtectedSecret {
    pub fn new(value: Option<String>) -> Self {
        ProtectedSecret {
            value: value.map(ProtectedValue),
        }
    }

    pub fn get_value(&self) -> Option<&ProtectedValue> {
        self.value.as_ref()
    }

    pub fn exists(&self) -> bool {
        self.value.is_some()
    }

    // pub fn serialize_self(&self) -> Self {
    //     // Return a copy of itself (in this case, the struct itself is returned as is)
    //     self.clone()
    // }
}

impl fmt::Display for ProtectedSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[PROTECTED]")
    }
}

impl fmt::Debug for ProtectedSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[PROTECTED]")
    }
}

impl PartialEq<str> for ProtectedSecret {
    fn eq(&self, other: &str) -> bool {
        self.value.as_ref().map(|v| &**v) == Some(other)
    }
}

// impl Serialize for ProtectedSecret {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         serializer.serialize_str("[PROTECTED]")
//     }
// }

// impl<'de> Deserialize<'de> for ProtectedSecret {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let _ = String::deserialize(deserializer)?; // Ignore incoming value
//         Ok(ProtectedSecret::new(None)) // No value is reconstructed during deserialization
//     }
// }
// impl Serialize for ProtectedSecret {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         if let Some(value) = &self.value {
//             serializer.serialize_str(&value.to_string())
//         } else {
//             serializer.serialize_str("[PROTECTED]")
//         }
//     }
// }
