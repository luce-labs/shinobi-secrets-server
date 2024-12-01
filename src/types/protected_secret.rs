use serde::{Deserialize, Serialize};
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

#[derive(Clone, Serialize, Deserialize)]
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
