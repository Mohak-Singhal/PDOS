use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub node_id: String,
    pub public_key: String,
    pub private_key: String,
}

impl Identity {
    pub fn load() -> Self {
        crate::identity::storage::load()
    }
}
