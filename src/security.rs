use std::convert::TryInto;

use anyhow::{Result, anyhow};
use ring::{aead::{CHACHA20_POLY1305, NONCE_LEN, chacha20_poly1305_openssh::TAG_LEN, LessSafeKey, UnboundKey, Nonce, Aad}, rand::{SystemRandom, SecureRandom}};
use serde::{Serialize, Deserialize};

/// An optional layer of encryption.
pub trait Security {
    /// The kind of security.
    fn kind(&self) -> &'static str;

    /// The key for encryption that is shared with the client, if used.
    fn key(&self) -> Option<&[u8]>;

    /// Encrypts a message (if needed).
    fn seal(&self, value: &[u8]) -> Result<Vec<u8>>;

    /// Decrypts a message (if needed).
    fn open(&self, value: &[u8]) -> Result<Vec<u8>>;
}

/// A `Security` implementation that does not encrypt.
pub struct NoSecurity;

impl Security for NoSecurity {
    fn kind(&self) -> &'static str { "none" }

    fn key(&self) -> Option<&[u8]> { None }

    fn seal(&self, value: &[u8]) -> Result<Vec<u8>> {
        Ok(value.to_vec())
    }

    fn open(&self, value: &[u8]) -> Result<Vec<u8>> {
        Ok(value.to_vec())
    }
}

/// A security implementation that uses ChaCha20-Poly1305
/// for symmetric, authenticated encryption.
pub struct ChaChaPolySecurity {
    rng: SystemRandom,
    key: Vec<u8>,
}

/// A sealed box containing an authenticated ciphertext
/// encrypted with ChaCha20-Poly1305.
#[derive(Serialize, Deserialize)]
pub struct ChaChaPolyBox {
    nonce: [u8; NONCE_LEN],
    tag: [u8; TAG_LEN],
    ciphertext: Vec<u8>,
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
        let tag = key.seal_in_place_separate_tag(
            Nonce::assume_unique_for_key(nonce),
            Aad::empty(),
            &mut buffer
        ).map_err(|_| anyhow!("Could not seal message"))?.as_ref().try_into().unwrap();

        let sealed_box = ChaChaPolyBox { nonce, tag, ciphertext: buffer };
        let sealed_box_bytes = bincode::serialize(&sealed_box)?;

        Ok(sealed_box_bytes)
    }

    fn open(&self, sealed_box_bytes: &[u8]) -> Result<Vec<u8>> {
        let sealed_box: ChaChaPolyBox = bincode::deserialize(sealed_box_bytes)?;
        let nonce = sealed_box.nonce;

        let key = self.less_safe_key()?;

        let mut buffer = sealed_box.ciphertext.to_vec();
        buffer.extend(sealed_box.tag);
        let plaintext = key.open_in_place(
            Nonce::assume_unique_for_key(nonce),
            Aad::empty(),
            &mut buffer
        ).map_err(|_| anyhow!("Could not open message"))?;

        Ok(plaintext.to_vec())
    }
}
