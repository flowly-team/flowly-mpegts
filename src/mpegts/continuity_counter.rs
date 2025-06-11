use crate::Error;

/// Continuity counter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ContinuityCounter(u8);
impl ContinuityCounter {
    /// Maximum counter value.
    pub const MAX: u8 = (1 << 4) - 1;

    /// Makes a new `ContinuityCounter` instance that has the value `0`.
    pub fn new() -> Self {
        ContinuityCounter(0)
    }

    /// Makes a new `ContinuityCounter` instance with the given value.
    ///
    /// # Errors
    ///
    /// If `n` exceeds `ContinuityCounter::MAX`, it will return an `ErrorKind::InvalidInput` error.
    pub fn from_u8(n: u8) -> Result<Self, Error> {
        assert!(n <= Self::MAX, "Too large counter: {}", n);

        Ok(ContinuityCounter(n))
    }

    /// Returns the value of the counter.
    pub fn as_u8(&self) -> u8 {
        self.0
    }

    /// Increments the counter.
    ///
    /// # Examples
    ///
    /// ```
    /// use mpeg2ts::ts::ContinuityCounter;
    ///
    /// let mut c = ContinuityCounter::new();
    /// assert_eq!(c.as_u8(), 0);
    ///
    /// for _ in 0..5 { c.increment(); }
    /// assert_eq!(c.as_u8(), 5);
    ///
    /// for _ in 0..11 { c.increment(); }
    /// assert_eq!(c.as_u8(), 0);
    /// ```
    pub fn increment(&mut self) {
        self.0 = (self.0 + 1) & Self::MAX;
    }
}
impl Default for ContinuityCounter {
    fn default() -> Self {
        Self::new()
    }
}
