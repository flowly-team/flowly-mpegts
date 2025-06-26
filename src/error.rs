use flowly::{Fourcc, Void};

#[derive(Debug, thiserror::Error)]
pub enum Error<E = Void> {
    #[error("Mpeg2Ts Error: {0}")]
    TsError(#[from] mpeg2ts::Error),

    #[error("Not an audio ID: {0}")]
    WrongAudioStreamId(u8),

    #[error("Not a video ID: {0}")]
    WrongVideoStreamId(u8),

    #[error("Value too large: {0}")]
    ValueTooLarge(u64),

    #[error("Marker Bits Check Fail: {0}")]
    UnexpectedMarkerBit(u64),

    #[error("Wrong SyncByte")]
    WrogSyncByte,

    #[error("Unknown PID: {0}")]
    UnknownPid(u16),

    #[error("Psi table count is zero")]
    PsiTableCountZero,

    #[error("Unsupported Codec {0}")]
    MuxUnsupportedCodec(Fourcc),

    #[error(transparent)]
    Other(E),
}

impl Error {
    pub fn extend<E>(self) -> Error<E> {
        match self {
            Error::TsError(error) => Error::TsError(error),
            Error::WrongAudioStreamId(id) => Error::WrongAudioStreamId(id),
            Error::WrongVideoStreamId(id) => Error::WrongVideoStreamId(id),
            Error::ValueTooLarge(val) => Error::ValueTooLarge(val),
            Error::UnexpectedMarkerBit(mask) => Error::UnexpectedMarkerBit(mask),
            Error::WrogSyncByte => Error::WrogSyncByte,
            Error::UnknownPid(pid) => Error::UnknownPid(pid),
            Error::PsiTableCountZero => Error::PsiTableCountZero,
            Error::MuxUnsupportedCodec(fourcc) => Error::MuxUnsupportedCodec(fourcc),
            Error::Other(_) => unreachable!(),
        }
    }
}
