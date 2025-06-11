use crate::mpegts::{WritableLen, bytes::RawData, continuity_counter::ContinuityCounter, pid::Pid};

use super::{
    Null,
    adaptation_field::{AdaptationField, AdaptationFieldControl},
    pat::Pat,
    pes::Pes,
    pmt::Pmt,
    section::Section,
};

/// Transport scrambling control.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TransportScramblingControl {
    NotScrambled = 0b00,
    ScrambledWithEvenKey = 0b10,
    ScrambledWithOddKey = 0b11,
    Unknown(u8),
}

impl TransportScramblingControl {
    pub fn from_u8(n: u8) -> Self {
        match n {
            0b00 => TransportScramblingControl::NotScrambled,
            0b10 => TransportScramblingControl::ScrambledWithEvenKey,
            0b11 => TransportScramblingControl::ScrambledWithOddKey,
            v => TransportScramblingControl::Unknown(v),
        }
    }
}

/// Transport stream packet.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TsPacket {
    pub header: TsHeader,
    pub adaptation_field: Option<AdaptationField>,
    pub payload: Option<TsPayload>,
}

impl TsPacket {
    /// Size of a packet in bytes.
    pub const SIZE: usize = 188;

    /// Synchronization byte.
    ///
    /// Each packet starts with this byte.
    pub const SYNC_BYTE: u8 = 0x47;
}

/// TS packet header.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TsHeader {
    pub transport_error_indicator: bool,
    pub transport_priority: bool,
    pub pid: Pid,
    pub transport_scrambling_control: TransportScramblingControl,
    pub continuity_counter: ContinuityCounter,
    pub adaptation_field_control: AdaptationFieldControl,
    pub payload_unit_start_indicator: bool,
}

/// TS packet payload.
#[allow(missing_docs, clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TsPayload {
    Pat(Pat),
    Pmt(Pmt),
    Pes(Pes),
    Section(Section),
    Null(Null),
    Raw(RawData),
}

impl WritableLen for TsPayload {
    fn writable_len(&self) -> usize {
        match self {
            TsPayload::Pat(pat) => pat.writable_len(),
            TsPayload::Pmt(pmt) => pmt.writable_len(),
            TsPayload::Pes(pes) => pes.writable_len(),
            TsPayload::Section(section) => section.writable_len(),
            TsPayload::Null(null) => null.writable_len(),
            TsPayload::Raw(raw_data) => raw_data.writable_len(),
        }
    }
}
