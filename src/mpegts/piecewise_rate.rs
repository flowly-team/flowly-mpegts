use crate::Error;

use super::WritableLen;

/// Piecewise rate.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PiecewiseRate(pub(crate) u32);
impl PiecewiseRate {
    /// Maximum rate.
    pub const MAX: u32 = (1 << 22) - 1;

    /// Makes a new `PiecewiseRate` instance.
    ///
    /// # Errors
    ///
    /// If `rate` exceeds `PiecewiseRate::MAX`, it will return an `ErrorKind::InvalidInput` error.
    pub fn new(rate: u32) -> Result<Self, Error> {
        assert!(rate <= Self::MAX, "Too large rate: {}", rate);

        Ok(PiecewiseRate(rate))
    }

    /// Returns the value of the `PiecewiseRate` instance.
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl WritableLen for PiecewiseRate {
    fn writable_len(&self) -> usize {
        3
    }
}
