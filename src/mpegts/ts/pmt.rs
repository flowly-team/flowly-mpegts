use bytes::Bytes;

use crate::mpegts::{WritableLen, pid::Pid, stream_type::StreamType, version::VersionNumber};

/// Program Map Table.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pmt {
    pub program_num: u16,

    /// The packet identifier that contains the program clock reference (PCR).
    ///
    /// The PCR is used to improve the random access accuracy of the stream's timing
    /// that is derived from the program timestamp.
    pub pcr_pid: Option<Pid>,

    pub version_number: VersionNumber,
    pub program_info: Vec<Descriptor>,
    pub es_info: Vec<EsInfo>,
}

impl Pmt {
    pub const TABLE_ID: u8 = 2;
}

impl WritableLen for Pmt {
    fn writable_len(&self) -> usize {
        todo!()
    }
}

/// Elementary stream information.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EsInfo {
    pub stream_type: StreamType,

    /// The packet identifier that contains the stream type data.
    pub elementary_pid: Pid,

    pub descriptors: Vec<Descriptor>,
}

/// Program or elementary stream descriptor.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Descriptor {
    pub tag: u8,
    pub data: Bytes,
}
