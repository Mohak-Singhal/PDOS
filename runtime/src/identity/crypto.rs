use base64::{Engine as _, engine::general_purpose};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;

use super::Identity;

pub fn generate() -> Identity {
    let signing_key = SigningKey::generate(&mut OsRng);

    let verifying_key = signing_key.verifying_key();

    let private_key = general_purpose::STANDARD.encode(signing_key.to_bytes());

    let public_key = general_purpose::STANDARD.encode(verifying_key.to_bytes());

    Identity {
        node_id: public_key.clone(),
        public_key,
        private_key,
    }
}
