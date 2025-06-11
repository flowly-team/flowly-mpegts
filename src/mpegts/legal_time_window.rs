use crate::Error;

use super::WritableLen;

/// Legal time window.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LegalTimeWindow {
    pub(crate) is_valid: bool,
    pub(crate) offset: u16,
}

impl LegalTimeWindow {
    /// Maximum offset value.
    pub const MAX_OFFSET: u16 = (1 << 15) - 1;

    /// Makes a new `LegalTimeWindow` instance.
    ///
    /// # Errors
    ///
    /// If `offset` exceeds `LegalTimeWindow::MAX_OFFSET`, it will return an `ErrorKind::InvalidInput` error.
    pub fn new(is_valid: bool, offset: u16) -> Result<Self, Error> {
        assert!(offset <= Self::MAX_OFFSET, "Too large offset: {}", offset);

        Ok(LegalTimeWindow { is_valid, offset })
    }

    /// Returns `true` if the window is valid, otherwise `false`.
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }

    /// Returns the offset of the window.
    pub fn offset(&self) -> u16 {
        self.offset
    }
}

impl WritableLen for LegalTimeWindow {
    fn writable_len(&self) -> usize {
        2
    }
}
