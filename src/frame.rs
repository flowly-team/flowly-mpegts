use bytes::Bytes;
use flowly::{Fourcc, Frame, FrameFlags};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mpeg2TsFrame {
    pub pts: i64,
    pub dts: u64,
    pub codec: Fourcc,
    pub keyframe: bool,
    pub payload: Bytes,
}

impl Frame for Mpeg2TsFrame {
    fn dts(&self) -> u64 {
        self.dts
    }

    fn pts(&self) -> i64 {
        self.pts
    }

    fn codec(&self) -> Fourcc {
        self.codec
    }

    fn flags(&self) -> FrameFlags {
        let mut flags = FrameFlags::empty();
        if self.keyframe {
            flags = FrameFlags::KEYFRAME;
        }

        flags
    }

    fn track(&self) -> u32 {
        0
    }

    fn timestamp(&self) -> Option<u64> {
        None
    }

    fn params(&self) -> impl Iterator<Item = &[u8]> {
        std::iter::empty()
    }

    fn units(&self) -> impl Iterator<Item = &[u8]> {
        std::iter::empty()
    }
}
