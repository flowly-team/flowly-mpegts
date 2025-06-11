use crate::mpegts::{WritableLen, bytes::RawData};

/// Payload for Section Stream packets.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Section {
    pub pointer_field: u8,
    pub data: RawData,
}

impl WritableLen for Section {
    fn writable_len(&self) -> usize {
        todo!()
    }
}
