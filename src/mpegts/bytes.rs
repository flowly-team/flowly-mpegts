use std::{
    fmt,
    hash::{Hash, Hasher},
    ops::Deref,
};

use crate::Error;

use super::{WritableLen, ts::TsPacket};

/// Byte sequence used to represent packet payload data.
#[derive(Clone)]
pub struct RawData {
    pub(crate) buf: [u8; RawData::MAX_SIZE],
    pub(crate) len: usize,
}

impl RawData {
    /// Maximum size of a byte sequence.
    pub const MAX_SIZE: usize = TsPacket::SIZE - 4 /* the size of the sync byte and a header */;

    /// Makes a new `Bytes` instance.
    ///
    /// # Errors
    ///
    /// If the length of `bytes` exceeds `Bytes::MAX_SIZE`,
    /// it will return an `ErrorKind::InvalidInput` error.
    pub fn new(bytes: &[u8]) -> Result<Self, Error> {
        assert!(
            bytes.len() <= Self::MAX_SIZE,
            "Too large: actual={} bytes, max={} bytes",
            bytes.len(),
            Self::MAX_SIZE
        );

        let len = bytes.len();
        let mut buf = [0; Self::MAX_SIZE];
        buf[..len].copy_from_slice(bytes);
        Ok(RawData { buf, len })
    }
}

impl Deref for RawData {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.buf[..self.len]
    }
}
impl AsRef<[u8]> for RawData {
    fn as_ref(&self) -> &[u8] {
        self.deref()
    }
}
impl fmt::Debug for RawData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bytes({:?})", self.deref())
    }
}
impl PartialEq for RawData {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl Eq for RawData {}
impl Hash for RawData {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.as_ref().hash(hasher);
    }
}

impl WritableLen for RawData {
    fn writable_len(&self) -> usize {
        self.len
    }
}
