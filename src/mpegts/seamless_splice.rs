use crate::Error;

use super::timestamp::{PtsDts, Timestamp};

/// Seamless splice.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SeamlessSplice {
    pub(crate) splice_type: u8,
    pub(crate) dts_next_access_unit: Timestamp<PtsDts>,
}

impl SeamlessSplice {
    /// Maximum splice type value.
    pub const MAX_SPLICE_TYPE: u8 = (1 << 4) - 1;

    /// Makes a new `SeamlessSplice` instance.
    ///
    /// # Errors
    ///
    /// If `splice_type` exceeds `SeamlessSplice::MAX_SPLICE_TYPE`,
    /// it will return an `ErrorKind::InvalidInput` error.
    pub fn new(splice_type: u8, dts_next_access_unit: Timestamp<PtsDts>) -> Result<Self, Error> {
        assert!(
            splice_type <= Self::MAX_SPLICE_TYPE,
            "Too large splice type: {}",
            splice_type
        );

        Ok(SeamlessSplice {
            splice_type,
            dts_next_access_unit,
        })
    }

    /// Returns the splice type (i.e., parameters of the H.262 splice).
    pub fn splice_type(&self) -> u8 {
        self.splice_type
    }

    /// Returns the PES DTS of the splice point.
    pub fn dts_next_access_unit(&self) -> Timestamp<PtsDts> {
        self.dts_next_access_unit
    }
}
