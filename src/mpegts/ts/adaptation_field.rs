use crate::mpegts::{
    legal_time_window::LegalTimeWindow,
    piecewise_rate::PiecewiseRate,
    seamless_splice::SeamlessSplice,
    timestamp::{Clock, PCR, Timestamp},
};

/// Adaptation field.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AdaptationField {
    /// Set `true` if current TS packet is in a discontinuity state with respect to
    /// either the continuity counter or the program clock reference.
    pub discontinuity_indicator: bool,

    /// Set `true` when the stream may be decoded without errors from this point.
    pub random_access_indicator: bool,

    /// Set `true` when this stream should be considered "high priority".
    pub es_priority_indicator: bool,

    pub pcr: Option<Timestamp<Clock<PCR>>>,
    pub opcr: Option<Timestamp<Clock<PCR>>>,

    /// Indicates how many TS packets from this one a splicing point occurs.
    pub splice_countdown: Option<i8>,

    pub transport_private_data: Vec<u8>,
    pub extension: Option<AdaptationExtensionField>,
}

impl AdaptationField {
    pub fn external_size(&self) -> usize {
        let mut n = 1 /* adaptation_field_len */ + 1 /* flags */;
        if self.pcr.is_some() {
            n += 6;
        }

        if self.opcr.is_some() {
            n += 6;
        }

        if self.splice_countdown.is_some() {
            n += 1;
        }

        n += self.transport_private_data.len();
        if let Some(ref x) = self.extension {
            n += x.external_size();
        }

        n
    }
}

/// Adaptation extension field.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AdaptationExtensionField {
    pub legal_time_window: Option<LegalTimeWindow>,
    pub piecewise_rate: Option<PiecewiseRate>,
    pub seamless_splice: Option<SeamlessSplice>,
}

impl AdaptationExtensionField {
    fn external_size(&self) -> usize {
        let mut n = 1 /* length */ + 1 /* flags */;
        if self.legal_time_window.is_some() {
            n += 2;
        }

        if self.piecewise_rate.is_some() {
            n += 3;
        }

        if self.seamless_splice.is_some() {
            n += 5;
        }

        n
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum AdaptationFieldControl {
    Reserved = 0b00,
    PayloadOnly = 0b01,
    AdaptationFieldOnly = 0b10,
    AdaptationFieldAndPayload = 0b11,
    Unknown(u8),
}
impl AdaptationFieldControl {
    pub fn has_adaptation_field(&self) -> bool {
        *self != AdaptationFieldControl::PayloadOnly
    }

    pub fn has_payload(&self) -> bool {
        *self != AdaptationFieldControl::AdaptationFieldOnly
    }

    pub fn from_u8(n: u8) -> Self {
        match n {
            0b00 => AdaptationFieldControl::Reserved,
            0b01 => AdaptationFieldControl::PayloadOnly,
            0b10 => AdaptationFieldControl::AdaptationFieldOnly,
            0b11 => AdaptationFieldControl::AdaptationFieldAndPayload,
            v => AdaptationFieldControl::Unknown(v),
        }
    }
}
