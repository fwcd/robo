mod empty;
mod chachapoly;

pub use empty::*;
pub use chachapoly::*;

use anyhow::Result;

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
