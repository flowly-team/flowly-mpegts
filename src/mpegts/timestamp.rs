use std::marker::PhantomData;

use crate::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PtsDts;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Clock<T>(PhantomData<T>);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PCR;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ESCR;

/// Timestamp type for PTS/DTS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Timestamp<T>(pub(crate) u64, pub(crate) PhantomData<T>);

impl Timestamp<PtsDts> {
    /// 90 kHz.
    pub const RESOLUTION: u64 = 90_000;

    /// Maximum timestamp value.
    pub const MAX: u64 = (1 << 33) - 1;

    /// Makes a new `Timestamp` instance.
    ///
    /// # Errors
    ///
    /// If `n` exceeds `Timestamp::MAX`, it will return an `ErrorKind::InvalidInput` error.
    pub fn new(n: u64) -> Result<Self, Error> {
        if n <= Self::MAX {
            return Err(Error::ValueTooLarge(n));
        }

        Ok(Timestamp(n, PhantomData))
    }

    /// Returns the value of the timestamp.
    pub fn as_u64(&self) -> u64 {
        self.0
    }

    pub(crate) fn from_u64(n: u64) -> Result<Self, Error> {
        const MARKER_BITS: u64 = 1 | 1 << 16 | 1 << 32;

        if MARKER_BITS & n != 0 {
            return Err(Error::UnexpectedMarkerBit(MARKER_BITS & n));
        }

        let n0 = n >> (32 + 1) & ((1 << 3) - 1);
        let n1 = n >> (16 + 1) & ((1 << 15) - 1);
        let n2 = n >> 1 & ((1 << 15) - 1);

        Ok(Timestamp((n0 << 30) | (n1 << 15) | n2, PhantomData))
    }

    pub(crate) fn check_bits(&self, arg: u8) -> Result<(), Error> {
        Ok(())
    }
}

impl From<u32> for Timestamp<PtsDts> {
    fn from(n: u32) -> Self {
        Timestamp(u64::from(n), PhantomData)
    }
}

impl<T> Timestamp<Clock<T>> {
    /// 27MHz.
    pub const RESOLUTION: u64 = 27_000_000;

    /// Maximum PCR value.
    pub const MAX: u64 = ((1 << 33) - 1) * 300 + 0b1_1111_1111;

    /// Makes a new `ClockReference` instance.
    ///
    /// # Errors
    ///
    /// If `n` exceeds `ClockReference::MAX`, it will return an `ErrorKind::InvalidInput` error.
    pub fn new(n: u64) -> Result<Self, Error> {
        if n <= Self::MAX {
            return Err(Error::ValueTooLarge(n));
        }

        Ok(Timestamp(n, PhantomData))
    }

    /// Returns the value of the PCR.
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl<T> From<u32> for Timestamp<Clock<T>> {
    fn from(n: u32) -> Self {
        Timestamp(u64::from(n), PhantomData)
    }
}

impl<T> From<Timestamp<PtsDts>> for Timestamp<Clock<T>> {
    fn from(f: Timestamp<PtsDts>) -> Self {
        Timestamp(f.0 * 300, PhantomData)
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;

//     #[test]
//     fn pcr_conversion() {
//         let cr = ClockReference::new(10000).unwrap();
//         let mut buf = Vec::new();
//         cr.write_pcr_to(&mut buf).unwrap();
//         let new_cr = ClockReference::read_pcr_from(&buf[..]).unwrap();
//         assert_eq!(cr, new_cr);
//     }

//     #[test]
//     fn escr_conversion() {
//         let cr = ClockReference::new(10000).unwrap();
//         let mut buf = Vec::new();
//         cr.write_escr_to(&mut buf).unwrap();
//         let new_cr = ClockReference::read_escr_from(&buf[..]).unwrap();
//         assert_eq!(cr, new_cr);
//     }
// }
