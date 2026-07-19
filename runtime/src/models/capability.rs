use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Capability {
    FileTransfer,
    Clipboard,
    Notifications,
    Terminal,
}
