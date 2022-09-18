use anyhow::{anyhow, Result};
use ring::{aead::{CHACHA20_POLY1305, NONCE_LEN, LessSafeKey, UnboundKey, Nonce, Aad}, rand::{SystemRandom, SecureRandom}};

use super::Security;

/// A security implementation that uses ChaCha20-Poly1305
/// for symmetric, authenticated encryption.
#[derive(Clone, Debug)]
pub struct ChaChaPolySecurity {
    rng: SystemRandom,
    key: Vec<u8>,
}

impl ChaChaPolySecurity {
    pub fn new() -> Result<Self> {
        let mut key = vec![0u8; CHACHA20_POLY1305.key_len()];
        let rng = SystemRandom::new();
        rng.fill(&mut key).map_err(|_| anyhow!("Could not generate key"))?;
        Ok(Self { rng, key })
    }

    fn less_safe_key(&self) -> Result<LessSafeKey> {
        let unbound_key = UnboundKey::new(&CHACHA20_POLY1305, &self.key).map_err(|_| anyhow!("Cannot create unbound key"))?;
        Ok(LessSafeKey::new(unbound_key))
    }
}

impl Security for ChaChaPolySecurity {
    fn kind(&self) -> &'static str { "chachapoly" }

    fn key(&self) -> Option<&[u8]> { Some(&self.key) }

    fn seal(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let mut nonce = [0u8; NONCE_LEN];
        self.rng.fill(&mut nonce).map_err(|_| anyhow!("Could not generate nonce"))?;
        
        // Safe to use here since we use random nonces (see https://github.com/briansmith/ring/issues/899#issuecomment-534346205)
        let key = self.less_safe_key()?;

        let mut buffer = plaintext.to_vec();
        key.seal_in_place_append_tag(
            Nonce::assume_unique_for_key(nonce),
            Aad::empty(),
            &mut buffer
        ).map_err(|_| anyhow!("Could not seal message"))?;

        let sealed_box = nonce.into_iter().chain(buffer).collect();
        Ok(sealed_box)
    }

    fn open(&self, sealed_box: &[u8]) -> Result<Vec<u8>> {
        let nonce: [u8; NONCE_LEN] = sealed_box[..NONCE_LEN].try_into().unwrap();

        let key = self.less_safe_key()?;

        let mut buffer = sealed_box[NONCE_LEN..].to_vec();
        let plaintext = key.open_in_place(
            Nonce::assume_unique_for_key(nonce),
            Aad::empty(),
            &mut buffer
        ).map_err(|_| anyhow!("Could not open message"))?;

        Ok(plaintext.to_vec())
    }
}
