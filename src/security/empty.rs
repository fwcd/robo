use anyhow::Result;

use super::Security;

/// A `Security` implementation that does not encrypt.
#[derive(Clone, Copy, Debug)]
pub struct EmptySecurity;

impl Security for EmptySecurity {
    fn kind(&self) -> &'static str { "none" }

    fn key(&self) -> Option<&[u8]> { None }

    fn seal(&self, value: &[u8]) -> Result<Vec<u8>> {
        Ok(value.to_vec())
    }

    fn open(&self, value: &[u8]) -> Result<Vec<u8>> {
        Ok(value.to_vec())
    }
}
