use std::{io::Write, pin::pin};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use flowly::{Fourcc, Frame, Service};
use futures::StreamExt;
use mpeg2ts::{
    Error as TsError,
    es::{StreamId, StreamType},
    pes::PesHeader,
    time::{ClockReference, Timestamp},
    ts::{
        AdaptationField, ContinuityCounter, EsInfo, Pid, ProgramAssociation,
        TransportScramblingControl, TsHeader, TsPacket, TsPacketWriter, TsPayload, VersionNumber,
        WriteTsPacket,
        payload::{self, Pat, Pmt},
    },
};

use crate::{Error, error::ExtError};

const PMT_PID: u16 = 256;
const VIDEO_ES_PID: u16 = 257;
// const AUDIO_ES_PID: u16 = 258;
const PES_VIDEO_STREAM_ID: u8 = 224;
// const PES_AUDIO_STREAM_ID: u8 = 192;

#[derive(Default)]
pub struct Mpeg2TsMuxer {
    video_continuity_counter: ContinuityCounter,
    header_sent: bool,
    params_updated: bool,
}

impl Mpeg2TsMuxer {
    fn write_packet(
        &mut self,
        writer: &mut TsPacketWriter<impl Write>,
        ts: Timestamp,
        unit: &[u8],
        is_keyframe: bool,
    ) -> Result<(), Error> {
        let mut header = Self::default_ts_header(VIDEO_ES_PID, self.video_continuity_counter)?;
        let mut annexb = Vec::with_capacity(unit.len() + 3);
        annexb.extend_from_slice(&[0, 0, 1]);
        annexb.extend_from_slice(unit);
        let mut buf = &annexb[..];

        let packet = {
            let data = payload::Bytes::new(&buf.chunk()[..buf.remaining().min(150)])?;
            buf.advance(data.len());

            TsPacket {
                header: header.clone(),
                adaptation_field: is_keyframe.then(|| AdaptationField {
                    discontinuity_indicator: false,
                    random_access_indicator: true,
                    es_priority_indicator: false,
                    pcr: Some(ClockReference::from(ts)),
                    opcr: None,
                    splice_countdown: None,
                    transport_private_data: Vec::new(),
                    extension: None,
                }),
                payload: Some(TsPayload::Pes(payload::Pes {
                    header: PesHeader {
                        stream_id: StreamId::new(PES_VIDEO_STREAM_ID),
                        priority: false,
                        data_alignment_indicator: false,
                        copyright: false,
                        original_or_copy: false,
                        pts: Some(ts),
                        dts: None,
                        escr: None,
                    },
                    pes_packet_len: 0,
                    data,
                })),
            }
        };

        writer.write_ts_packet(&packet)?;
        header.continuity_counter.increment();

        while buf.has_remaining() {
            let raw_payload =
                payload::Bytes::new(&buf.chunk()[..buf.remaining().min(payload::Bytes::MAX_SIZE)])?;

            buf.advance(raw_payload.len());

            let packet = TsPacket {
                header: header.clone(),
                adaptation_field: None,
                payload: Some(TsPayload::Raw(raw_payload)),
            };

            writer.write_ts_packet(&packet)?;
            header.continuity_counter.increment();
        }

        self.video_continuity_counter = header.continuity_counter;
        Ok(())
    }
}

impl Mpeg2TsMuxer {
    #[inline]
    fn write_header<W: WriteTsPacket>(
        &mut self,
        writer: &mut W,
        stream_type: StreamType,
    ) -> Result<(), TsError> {
        self.write_packets(
            writer,
            [
                &Self::default_pat_packet(),
                &Self::default_pmt_packet(stream_type),
            ],
        )?;

        Ok(())
    }

    #[inline]
    fn write_packets<'a, W: WriteTsPacket, P: IntoIterator<Item = &'a TsPacket>>(
        &mut self,
        writer: &mut W,
        packets: P,
    ) -> Result<(), TsError> {
        packets
            .into_iter()
            .try_for_each(|pak| writer.write_ts_packet(pak))?;

        Ok(())
    }

    fn default_ts_header(
        pid: u16,
        continuity_counter: ContinuityCounter,
    ) -> Result<TsHeader, TsError> {
        Ok(TsHeader {
            transport_error_indicator: false,
            transport_priority: false,
            pid: Pid::new(pid)?,
            transport_scrambling_control: TransportScramblingControl::NotScrambled,
            continuity_counter,
        })
    }

    fn default_pat_packet() -> TsPacket {
        TsPacket {
            header: Self::default_ts_header(0, Default::default()).unwrap(),
            adaptation_field: None,
            payload: Some(TsPayload::Pat(Pat {
                transport_stream_id: 1,
                version_number: VersionNumber::default(),
                table: vec![ProgramAssociation {
                    program_num: 1,
                    program_map_pid: Pid::new(PMT_PID).unwrap(),
                }],
            })),
        }
    }

    fn default_pmt_packet(stream_type: StreamType) -> TsPacket {
        TsPacket {
            header: Self::default_ts_header(PMT_PID, Default::default()).unwrap(),
            adaptation_field: None,
            payload: Some(TsPayload::Pmt(Pmt {
                program_num: 1,
                pcr_pid: Some(Pid::new(VIDEO_ES_PID).unwrap()),
                version_number: VersionNumber::default(),
                program_info: vec![],
                es_info: vec![EsInfo {
                    stream_type,
                    elementary_pid: Pid::new(VIDEO_ES_PID).unwrap(),
                    descriptors: vec![],
                }],
            })),
        }
    }
}

impl Mpeg2TsMuxer {
    fn push_frame<F: Frame>(&mut self, frame: F, dst: &mut BytesMut) -> Result<(), Error> {
        let mut writer = TsPacketWriter::new(dst.writer());

        if !self.header_sent {
            self.header_sent = true;
            self.write_header(
                &mut writer,
                match frame.codec() {
                    Fourcc::VIDEO_AVC => StreamType::H264,
                    Fourcc::VIDEO_HEVC => StreamType::H265,
                    codec => return Err(Error::MuxUnsupportedCodec(codec)),
                },
            )?;
        }

        if frame.has_params() {
            self.params_updated = true;
        }

        let ts = Timestamp::new((frame.pts() as u64 * 9) / 100).map_err(TsError::from)?;

        for param in frame.params() {
            if self.params_updated {
                self.params_updated = false;
            }

            self.write_packet(&mut writer, ts, param, false)?;
        }

        for unit in frame.units() {
            self.write_packet(&mut writer, ts, unit, frame.is_keyframe())?;
        }

        Ok(())
    }
}

impl<F: Frame + Send, E: std::error::Error + Send + Sync + 'static> Service<Result<F, E>>
    for Mpeg2TsMuxer
{
    type Out = Result<Bytes, ExtError<E>>;

    fn handle(
        mut self,
        input: impl futures::Stream<Item = Result<F, E>> + Send,
    ) -> impl futures::Stream<Item = Self::Out> + Send {
        async_stream::stream! {
            let mut input = pin!(input);
            let mut buffer = BytesMut::new();

            while let Some(res) = input.next().await {
                match res {
                    Ok(frame) => {
                        if let Err(err) = self.push_frame(frame, &mut buffer) {
                             yield Err(err.into());
                        }

                        yield Ok(buffer.split().freeze());
                    },
                    Err(err) => yield Err(ExtError::Other(err)),
                }
            }
        }
    }
}
