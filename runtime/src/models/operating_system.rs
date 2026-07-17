use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperatingSystem {
    MacOS,
    Windows,
    Linux,
    Android,
    IOS,
}