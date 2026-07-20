//! Shared error type for browser Solana helpers.

use std::fmt;

/// Error returned by browser Solana helpers.
#[derive(Debug, Clone)]
pub struct BrowserSolanaError(pub String);

impl BrowserSolanaError {
    /// Create an error from a displayable message.
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for BrowserSolanaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for BrowserSolanaError {}

impl From<String> for BrowserSolanaError {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for BrowserSolanaError {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}
