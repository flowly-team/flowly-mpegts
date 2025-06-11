use flowly::Fourcc;

#[derive(Debug, thiserror::Error)]
pub enum Error {
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
}

#[derive(Debug, thiserror::Error)]
pub enum ExtError<E> {
    #[error(transparent)]
    Error(#[from] Error),

    #[error(transparent)]
    Other(E),
}
