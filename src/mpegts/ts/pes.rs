use crate::mpegts::{
    WritableLen,
    bytes::RawData,
    stream_id::StreamId,
    timestamp::{Clock, ESCR, PtsDts, Timestamp},
};

/// Payload for PES(Packetized elementary stream) packets.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pes<B = RawData> {
    pub header: PesHeader,
    pub data: B,
}

impl WritableLen for Pes {
    fn writable_len(&self) -> usize {
        todo!()
    }
}

pub const PACKET_START_CODE_PREFIX: u64 = 0x00_0001;

/// PES packet header.
///
/// Note that `PesHeader` contains the fields that belong to the optional PES header.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PesHeader {
    pub stream_id: StreamId,

    pub priority: bool,

    /// `true` indicates that the PES packet header is immediately followed by
    /// the video start code or audio syncword.
    pub data_alignment_indicator: bool,

    /// `true` implies copyrighted.
    pub copyright: bool,

    /// `true` implies original.
    pub original_or_copy: bool,

    pub pts: Option<Timestamp<PtsDts>>,
    pub dts: Option<Timestamp<PtsDts>>,

    /// Elementary stream clock reference.
    pub escr: Option<Timestamp<Clock<ESCR>>>,
    pub(crate) packet_len: u16,
}

impl PesHeader {
    pub fn optional_header_len(&self) -> u16 {
        3 //-
            + self.pts.map_or(0, |_| 5)
            + self.dts.map_or(0, |_| 5)
            + self.escr.map_or(0, |_| 6)
    }
}
