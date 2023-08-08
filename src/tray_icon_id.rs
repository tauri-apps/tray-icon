use std::{convert::Infallible, str::FromStr};

/// An unique id that is associated with a tray icon.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct TrayIconId(pub String);

impl TrayIconId {
    /// Create a new tray icon id.
    pub fn new<S: AsRef<str>>(id: S) -> Self {
        Self(id.as_ref().to_string())
    }
}

impl AsRef<str> for TrayIconId {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl From<String> for TrayIconId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for TrayIconId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl FromStr for TrayIconId {
    type Err = Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self::new(s))
    }
}

impl PartialEq<&str> for TrayIconId {
    fn eq(&self, other: &&str) -> bool {
        other == &self.0
    }
}

impl PartialEq<String> for TrayIconId {
    fn eq(&self, other: &String) -> bool {
        other == &self.0
    }
}

impl PartialEq<&String> for TrayIconId {
    fn eq(&self, other: &&String) -> bool {
        other == &&self.0
    }
}

impl PartialEq<&TrayIconId> for TrayIconId {
    fn eq(&self, other: &&TrayIconId) -> bool {
        other.0 == self.0
    }
}
