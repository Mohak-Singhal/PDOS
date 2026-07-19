use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeInfo {
    pub runtime_version: String,
    pub protocol_version: u16,
}
impl RuntimeInfo {
    pub fn is_compatible_with(&self, other: &RuntimeInfo) -> bool {
        self.protocol_version == other.protocol_version
    }
}
