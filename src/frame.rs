use std::sync::Arc;

use bytes::Bytes;
use flowly::{DataFrame, EncodedFrame, Fourcc, Frame, FrameFlags, FrameSource};

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Mpeg2TsSource<S: FrameSource> {
    pub codec: Fourcc,
    pub params: Vec<Bytes>,
    inner: S,
}

impl<S: FrameSource> FrameSource for Mpeg2TsSource<S> {
    type Source = S;

    fn source(&self) -> &Self::Source {
        &self.inner
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mpeg2TsFrame<S: FrameSource> {
    pub pts: i64,
    pub dts: u64,
    pub keyframe: bool,
    pub payload: Bytes,
    source: Arc<Mpeg2TsSource<S>>,
}

impl<S: FrameSource> EncodedFrame for Mpeg2TsFrame<S> {
    type Param = Bytes;

    fn pts(&self) -> i64 {
        self.pts
    }

    fn params(&self) -> impl Iterator<Item = &Self::Param> {
        self.source.params.iter()
    }
}

impl<S: FrameSource> DataFrame for Mpeg2TsFrame<S> {
    type Source = Arc<Mpeg2TsSource<S>>;
    type Chunk = Bytes;

    fn chunks(&self) -> impl Send + Iterator<Item = &Bytes> {
        std::iter::once(&self.payload)
    }

    fn into_chunks(self) -> impl Send + Iterator<Item = Bytes> {
        std::iter::once(self.payload)
    }

    fn source(&self) -> &Self::Source {
        &self.source
    }
}

impl<S: FrameSource> Frame for Mpeg2TsFrame<S> {
    fn timestamp(&self) -> u64 {
        self.dts
    }

    fn codec(&self) -> Fourcc {
        self.source.codec
    }

    fn flags(&self) -> FrameFlags {
        self.keyframe
            .then_some(FrameFlags::KEYFRAME)
            .unwrap_or_default()
            | FrameFlags::ENCODED
            | FrameFlags::ANNEXB
            | FrameFlags::VIDEO_STREAM
    }
}
