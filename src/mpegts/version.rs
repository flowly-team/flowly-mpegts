use crate::Error;

/// Version number for PSI table syntax section.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VersionNumber(u8);

impl VersionNumber {
    /// Maximum version number.
    pub const MAX: u8 = (1 << 5) - 1;

    /// Makes a new `VersionNumber` instance that has the value `0`.
    pub fn new() -> Self {
        VersionNumber(0)
    }

    /// Makes a new `VersionNumber` instance with the given value.
    ///
    /// # Errors
    ///
    /// If `n` exceeds `VersionNumber::MAX`, it will return an `ErrorKind::InvalidInput` error.
    pub fn from_u8(n: u8) -> Result<Self, Error> {
        assert!(n <= Self::MAX, "Too large version number: {}", n);

        Ok(VersionNumber(n))
    }

    /// Returns the value of the version number.
    pub fn as_u8(&self) -> u8 {
        self.0
    }

    /// Increments the version number.
    ///
    /// It will be wrapped around if overflow.
    pub fn increment(&mut self) {
        self.0 = (self.0 + 1) & Self::MAX;
    }
}

impl Default for VersionNumber {
    fn default() -> Self {
        Self::new()
    }
}
