use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeviceType {
    Laptop,
    Desktop,
    Phone,
    Tablet,
    Server,
    Browser,
}
