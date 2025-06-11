use bytes::{BufMut, BytesMut};
use flowly::Fourcc;
use std::collections::HashMap;

use bytes::Buf;
use mpeg2ts::{
    es::StreamType,
    ts::{ContinuityCounter, Pid, ReadTsPacket, TsPacket, TsPacketReader, TsPayload},
};

use crate::error::Error;

pub struct Mpeg2TsDemux {
    ignore: bool,
    mappings: Option<HashMap<Pid, PidKind>>,
    base_ts: u64,

    #[allow(dead_code)]
    cc: ContinuityCounter,
    fragm_idx: u64,
    frame_idx: u64,
    codec: Fourcc,
    is_keyframe: bool,
    is_params_updated: bool,
    tags: BytesMut,
    pts: u64,
    body: BytesMut,
}

impl Mpeg2TsDemux {
    pub fn new(base_ts: u64, fragm_idx: u64) -> Self {
        Self {
            ignore: true,
            mappings: None,
            base_ts,
            fragm_idx,
            frame_idx: 0,
            is_params_updated: false,
            tags: BytesMut::with_capacity(64 * 1024 * 1024),
            is_keyframe: false,
            codec: Fourcc::default(),
            pts: 0,
            cc: ContinuityCounter::new(),
            body: BytesMut::with_capacity(64 * 1024 * 1024),
        }
    }
}

impl Mpeg2TsDemux {
    fn parse(&mut self, src: &mut BytesMut) -> Result<Option<Mpeg2TsFrame>, Error> {
        let mut reader = if let Some(map) = self.mappings.take() {
            TsPacketReader::with_mappings(src.reader(), map)
        } else {
            TsPacketReader::new(src.reader())
        };

        while reader.stream().get_ref().remaining() >= TsPacket::SIZE {
            let pkt = reader.read_ts_packet()?.unwrap();
            let is_keyframe = pkt
                .adaptation_field
                .as_ref()
                .map(|x| x.random_access_indicator)
                .unwrap_or(false);

            if let Some(p) = pkt.payload {
                match p {
                    TsPayload::Pat(_pat) => {}
                    TsPayload::Pmt(pmt) => {
                        for es in pmt.es_info {
                            match es.stream_type {
                                StreamType::H264 => self.codec = Fourcc::VIDEO_AVC,
                                StreamType::H265 => self.codec = Fourcc::VIDEO_HEVC,
                                _ => (),
                            }
                        }
                    }

                    TsPayload::Pes(pes) => {
                        let frame = (!self.ignore).then(|| {
                            Frame::new(
                                self.codec,
                                self.fragm_idx,
                                self.frame_idx,
                                self.pts,
                                self.pts + self.base_ts,
                                self.is_keyframe,
                                self.is_params_updated,
                                self.tags.clone().into(),
                                true,
                                (0, 0),
                                self.source.clone(),
                                None::<Vec<Vec<u8>>>,
                                false,
                                self.body.to_vec(),
                            )
                        });

                        if pes.header.stream_id.as_u8() != PES_VIDEO_STREAM_ID {
                            self.ignore = true;
                            continue;
                        } else {
                            self.ignore = false;
                        }

                        self.pts = (pes.header.pts.unwrap().as_u64() * 1_000_000) / 90_000;
                        self.is_keyframe = is_keyframe;

                        self.body.clear();
                        self.body.put_slice(&pes.data);

                        if frame.is_some() {
                            self.frame_idx += 1;
                            self.mappings.replace(reader.into_mappings());
                            return Ok(frame);
                        }
                    }

                    TsPayload::Section(_) => println!("section"),
                    TsPayload::Null(_) => println!("null"),
                    TsPayload::Raw(raw) => {
                        if !self.ignore {
                            self.body.put_slice(&raw);
                        }
                    }
                }
            }
        }

        self.mappings.replace(reader.into_mappings());

        Ok(None)
    }
}
