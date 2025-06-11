use std::{collections::HashMap, marker::PhantomData};

use bytes::{Buf, BufMut, BytesMut};

use crate::{
    Error,
    mpegts::{
        WritableLen,
        stream_id::StreamId,
        stream_type::StreamType,
        ts::{MAX_SYNTAX_SECTION_LEN, PACKET_START_CODE_PREFIX, Psi, Stuffing, TsPayload},
        version::VersionNumber,
    },
};

use super::{
    bytes::RawData,
    continuity_counter::ContinuityCounter,
    crc32::WithCrc32,
    legal_time_window::LegalTimeWindow,
    pid::{Pid, PidKind},
    piecewise_rate::PiecewiseRate,
    seamless_splice::SeamlessSplice,
    timestamp::{Clock, ESCR, PCR, PtsDts, Timestamp},
    ts::{
        AdaptationExtensionField, AdaptationField, AdaptationFieldControl, Descriptor, EsInfo,
        Null, Pat, Pes, PesHeader, Pmt, ProgramAssociation, PsiTable, PsiTableHeader,
        PsiTableSyntax, Section, TransportScramblingControl, TsHeader, TsPacket,
    },
};

pub struct Mpeg2tsParser {
    pids: HashMap<Pid, PidKind>,
}

pub trait Io<T> {
    fn parse(&mut self, input: &mut impl Buf) -> Result<T, Error>;
    fn serialize(&mut self, item: &T, output: &mut impl BufMut) -> Result<(), Error>;
}

impl Io<TsHeader> for Mpeg2tsParser {
    fn parse(&mut self, reader: &mut impl Buf) -> Result<TsHeader, Error> {
        let sync_byte = reader.get_u8();
        if TsPacket::SYNC_BYTE != sync_byte {
            return Err(Error::WrogSyncByte);
        }

        let n = reader.get_u16();
        let transport_error_indicator = (n & 0b1000_0000_0000_0000) != 0;
        let payload_unit_start_indicator = (n & 0b0100_0000_0000_0000) != 0;
        let transport_priority = (n & 0b0010_0000_0000_0000) != 0;
        let pid = Pid::new(n & 0b0001_1111_1111_1111)?;

        let n = reader.get_u8();
        let transport_scrambling_control = TransportScramblingControl::from_u8(n >> 6);
        let adaptation_field_control = AdaptationFieldControl::from_u8((n >> 4) & 0b11);
        let continuity_counter = ContinuityCounter::from_u8(n & 0b1111)?;

        Ok(TsHeader {
            transport_error_indicator,
            transport_priority,
            pid,
            transport_scrambling_control,
            continuity_counter,
            adaptation_field_control,
            payload_unit_start_indicator,
        })
    }

    fn serialize(&mut self, item: &TsHeader, output: &mut impl BufMut) -> Result<(), Error> {
        todo!()
    }
}

impl Io<Option<AdaptationField>> for Mpeg2tsParser {
    fn parse(&mut self, reader: &mut impl Buf) -> Result<Option<AdaptationField>, Error> {
        let adaptation_field_len = reader.get_u8();
        if adaptation_field_len == 0 {
            return Ok(None);
        }

        let mut buf = [0u8; 256];
        reader.copy_to_slice(&mut buf[0..adaptation_field_len as usize]);
        let mut adaptation_reader = buf.as_slice();

        let b = adaptation_reader.get_u8();
        let discontinuity_indicator = (b & 0b1000_0000) != 0;
        let random_access_indicator = (b & 0b0100_0000) != 0;
        let es_priority_indicator = (b & 0b0010_0000) != 0;
        let pcr_flag = (b & 0b0001_0000) != 0;
        let opcr_flag = (b & 0b0000_1000) != 0;
        let splicing_point_flag = (b & 0b0000_0100) != 0;
        let transport_private_data_flag = (b & 0b0000_0010) != 0;
        let extension_flag = (b & 0b0000_0001) != 0;

        let pcr = if pcr_flag {
            Some(self.parse(&mut adaptation_reader)?)
        } else {
            None
        };

        let opcr = if opcr_flag {
            Some(self.parse(&mut adaptation_reader)?)
        } else {
            None
        };

        let splice_countdown = if splicing_point_flag {
            Some(reader.get_i8())
        } else {
            None
        };

        let transport_private_data = if transport_private_data_flag {
            let len = reader.get_u8();
            let mut buf = vec![0; len as usize];
            adaptation_reader.copy_to_slice(&mut buf);
            buf
        } else {
            Vec::new()
        };

        let extension = if extension_flag {
            Some(self.parse(&mut adaptation_reader)?)
        } else {
            None
        };

        // track!(util::consume_stuffing_bytes(reader))?;

        Ok(Some(AdaptationField {
            discontinuity_indicator,
            random_access_indicator,
            es_priority_indicator,
            pcr,
            opcr,
            splice_countdown,
            transport_private_data,
            extension,
        }))
    }

    fn serialize(
        &mut self,
        item: &Option<AdaptationField>,
        output: &mut impl BufMut,
    ) -> Result<(), Error> {
        todo!()
    }
}

impl Io<Timestamp<PtsDts>> for Mpeg2tsParser {
    fn parse(&mut self, reader: &mut impl Buf) -> Result<Timestamp<PtsDts>, Error> {
        let n = reader.get_uint(5);
        // assert_eq!((n >> 36) as u8, check_bits);

        Ok(Timestamp::<PtsDts>::from_u64(n)?)
    }

    fn serialize(
        &mut self,
        item: &Timestamp<PtsDts>,
        output: &mut impl BufMut,
    ) -> Result<(), Error> {
        todo!()
    }
}

impl Io<Timestamp<Clock<PCR>>> for Mpeg2tsParser {
    fn parse(&mut self, reader: &mut impl Buf) -> Result<Timestamp<Clock<PCR>>, Error> {
        let n = reader.get_uint(6);
        let base = n >> 15;
        let extension = n & 0b1_1111_1111;

        Ok(Timestamp(base * 300 + extension, std::marker::PhantomData))
    }

    fn serialize(
        &mut self,
        item: &Timestamp<Clock<PCR>>,
        output: &mut impl BufMut,
    ) -> Result<(), Error> {
        todo!()
    }
}

impl Io<Timestamp<Clock<ESCR>>> for Mpeg2tsParser {
    fn parse(&mut self, reader: &mut impl Buf) -> Result<Timestamp<Clock<ESCR>>, Error> {
        let n = reader.get_uint(6);
        assert_eq!(n >> 46, 0);
        assert_eq!(n & 1, 1);

        let extension = (n >> 1) & 0b1_1111_1111;
        let n = n >> 10;

        assert_eq!(n & 1, 1);
        assert_eq!((n >> 16) & 1, 1);
        assert_eq!((n >> 32) & 1, 1);

        let n0 = (n >> 1) & ((1 << 15) - 1);
        let n1 = (n >> 17) & ((1 << 15) - 1);
        let n2 = (n >> 33) & ((1 << 3) - 1);
        let base = n0 | (n1 << 15) | (n2 << 30);

        Ok(Timestamp(base * 300 + extension, PhantomData))
    }

    fn serialize(
        &mut self,
        item: &Timestamp<Clock<ESCR>>,
        output: &mut impl BufMut,
    ) -> Result<(), Error> {
        todo!()
    }
}

impl Io<AdaptationExtensionField> for Mpeg2tsParser {
    fn parse(&mut self, reader: &mut impl Buf) -> Result<AdaptationExtensionField, Error> {
        let extension_len = reader.get_u8();

        let mut buf = [0u8; 256];
        reader.copy_to_slice(&mut buf[0..extension_len as usize]);
        let mut ext_reader = buf.as_slice();

        let b = ext_reader.get_u8();
        let legal_time_window_flag = (b & 0b1000_0000) != 0;
        let piecewise_rate_flag = (b & 0b0100_0000) != 0;
        let seamless_splice_flag = (b & 0b0010_0000) != 0;

        let legal_time_window = if legal_time_window_flag {
            Some(self.parse(&mut ext_reader)?)
        } else {
            None
        };
        let piecewise_rate = if piecewise_rate_flag {
            Some(self.parse(&mut ext_reader)?)
        } else {
            None
        };
        let seamless_splice = if seamless_splice_flag {
            Some(self.parse(&mut ext_reader)?)
        } else {
            None
        };

        Ok(AdaptationExtensionField {
            legal_time_window,
            piecewise_rate,
            seamless_splice,
        })
    }

    fn serialize(
        &mut self,
        item: &AdaptationExtensionField,
        output: &mut impl BufMut,
    ) -> Result<(), Error> {
        todo!()
    }
}

impl Io<Pid> for Mpeg2tsParser {
    fn parse(&mut self, input: &mut impl Buf) -> Result<Pid, Error> {
        let n = input.get_u16();
        assert_eq!(
            n & 0b1110_0000_0000_0000,
            0b1110_0000_0000_0000,
            "Unexpected reserved bits"
        );

        Ok(Pid(n & 0b0001_1111_1111_1111))
    }

    fn serialize(&mut self, item: &Pid, output: &mut impl BufMut) -> Result<(), Error> {
        todo!()
    }
}

impl Io<LegalTimeWindow> for Mpeg2tsParser {
    fn parse(&mut self, input: &mut impl Buf) -> Result<LegalTimeWindow, Error> {
        let n = input.get_u16();

        Ok(LegalTimeWindow {
            is_valid: (n & 0b1000_0000_0000_0000) != 0,
            offset: n & 0b0111_1111_1111_1111,
        })
    }

    fn serialize(&mut self, item: &LegalTimeWindow, writer: &mut impl BufMut) -> Result<(), Error> {
        let n = ((item.is_valid as u16) << 15) | item.offset;
        writer.put_u16(n);
        Ok(())
    }
}

impl Io<PiecewiseRate> for Mpeg2tsParser {
    fn parse(&mut self, input: &mut impl Buf) -> Result<PiecewiseRate, Error> {
        let n = input.get_uint(3) as u32;

        Ok(PiecewiseRate(n & 0x3FFF_FFFF))
    }

    fn serialize(&mut self, item: &PiecewiseRate, writer: &mut impl BufMut) -> Result<(), Error> {
        writer.put_uint(u64::from(item.0), 3);
        Ok(())
    }
}

impl Io<SeamlessSplice> for Mpeg2tsParser {
    fn parse(&mut self, input: &mut impl Buf) -> Result<SeamlessSplice, Error> {
        let n = input.get_uint(5);

        Ok(SeamlessSplice {
            splice_type: (n >> 36) as u8,
            dts_next_access_unit: Timestamp::from_u64(n & 0x0F_FFFF_FFFF)?,
        })
    }

    fn serialize(&mut self, item: &SeamlessSplice, output: &mut impl BufMut) -> Result<(), Error> {
        self.serialize(&item.dts_next_access_unit, output)?;

        // .write_to(&mut writer, self.splice_type)
        Ok(())
    }
}

impl Io<TsPacket> for Mpeg2tsParser {
    fn parse(&mut self, input: &mut impl Buf) -> Result<TsPacket, Error> {
        let header: TsHeader = self.parse(input)?;

        let adaptation_field: Option<AdaptationField> =
            if header.adaptation_field_control.has_adaptation_field() {
                self.parse(input)?
            } else {
                None
            };

        let payload = if header.adaptation_field_control.has_payload() {
            let payload = match header.pid.as_u16() {
                Pid::PAT => {
                    let pat: Pat = self.parse(input)?;
                    for pa in &pat.table {
                        self.pids.insert(pa.program_map_pid, PidKind::Pmt);
                    }

                    TsPayload::Pat(pat)
                }

                // Null packets
                Pid::NULL => TsPayload::Null(self.parse(input)?),

                // Unknown (unsupported) packets
                0x01..=0x1F | 0x1FFB => TsPayload::Raw(self.parse(input)?),

                pid => match self.pids.get(&header.pid).ok_or(Error::UnknownPid(pid))? {
                    PidKind::Pmt => {
                        let pmt: Pmt = self.parse(input)?;

                        for es in &pmt.es_info {
                            self.pids.insert(es.elementary_pid, PidKind::Pes);
                        }

                        TsPayload::Pmt(pmt)
                    }
                    PidKind::Pes => {
                        if header.payload_unit_start_indicator {
                            TsPayload::Pes(self.parse(input)?)
                        } else {
                            TsPayload::Raw(self.parse(input)?)
                        }
                    }
                },
            };
            Some(payload)
        } else {
            None
        };

        Ok(TsPacket {
            header,
            adaptation_field,
            payload,
        })
    }

    fn serialize(&mut self, item: &TsPacket, writer: &mut impl BufMut) -> Result<(), Error> {
        let payload_len = item.payload.as_ref().map(|x| x.writable_len()).unwrap_or(0);

        let required_len = item
            .adaptation_field
            .as_ref()
            .map_or(0, |a| a.external_size());

        let free_len = TsPacket::SIZE - 4 - payload_len;
        assert!(
            required_len <= free_len,
            "No space for adaptation field: required={}, free={}",
            required_len,
            free_len,
        );

        let mut header = item.header.clone();

        header.adaptation_field_control = match (
            item.adaptation_field.is_some() || free_len > 0,
            item.payload.is_some(),
        ) {
            (true, true) => AdaptationFieldControl::AdaptationFieldAndPayload,
            (true, false) => AdaptationFieldControl::AdaptationFieldOnly,
            (false, true) => AdaptationFieldControl::PayloadOnly,
            (false, false) => panic!("Reserved for future use"),
        };

        header.payload_unit_start_indicator = !matches!(
            item.payload,
            Some(TsPayload::Raw(_)) | Some(TsPayload::Null(_)) | None
        );

        self.serialize(&header, writer)?;
        // let adaptation_field_len = (free_len - 1) as u8;
        self.serialize(&item.adaptation_field, writer)?;

        if item.adaptation_field.is_none() {
            let adaptation_field_len = (free_len - 1) as u8;
            self.serialize(&Stuffing(0xFF, adaptation_field_len as usize), writer)?;
        }

        if let Some(payload) = &item.payload {
            match payload {
                TsPayload::Pat(pat) => self.serialize(pat, writer)?,
                TsPayload::Pmt(pmt) => self.serialize(pmt, writer)?,
                TsPayload::Pes(pes) => self.serialize(pes, writer)?,
                TsPayload::Section(section) => self.serialize(section, writer)?,
                TsPayload::Null(null) => self.serialize(null, writer)?,
                TsPayload::Raw(raw_data) => self.serialize(raw_data, writer)?,
            }
        }

        Ok(())
    }
}

impl Io<Stuffing> for Mpeg2tsParser {
    fn parse(&mut self, _input: &mut impl Buf) -> Result<Stuffing, Error> {
        todo!()
    }

    fn serialize(&mut self, item: &Stuffing, output: &mut impl BufMut) -> Result<(), Error> {
        output.put_bytes(item.0, item.1);
        Ok(())
    }
}

impl Io<RawData> for Mpeg2tsParser {
    fn parse(&mut self, input: &mut impl Buf) -> Result<RawData, Error> {
        let mut buf = [0; RawData::MAX_SIZE];
        input.copy_to_slice(&mut buf[..input.remaining()]);

        Ok(RawData {
            buf,
            len: input.remaining() as _,
        })
    }

    fn serialize(&mut self, item: &RawData, output: &mut impl BufMut) -> Result<(), Error> {
        output.put_slice(&item.buf[..item.len as usize]);
        Ok(())
    }
}

impl Io<Pmt> for Mpeg2tsParser {
    fn parse(&mut self, input: &mut impl Buf) -> Result<Pmt, Error> {
        let mut psi: Psi = self.parse(input)?;

        if psi.tables.len() < 1 {
            return Err(Error::PsiTableCountZero);
        }

        let table = psi.tables.pop().expect("Never fails");

        // let header = table.header;
        // track_assert_eq!(header.table_id, Self::TABLE_ID, ErrorKind::InvalidInput);
        // track_assert!(!header.private_bit, ErrorKind::InvalidInput);

        let syntax = table.syntax.unwrap();
        // track_assert_eq!(syntax.section_number, 0, ErrorKind::InvalidInput);
        // track_assert_eq!(syntax.last_section_number, 0, ErrorKind::InvalidInput);
        // track_assert!(syntax.current_next_indicator, ErrorKind::InvalidInput);

        let mut reader = &syntax.table_data[..];

        let pcr_pid: Pid = self.parse(&mut reader)?;
        let pcr_pid = if pcr_pid.as_u16() == 0b0001_1111_1111_1111 {
            None
        } else {
            Some(pcr_pid)
        };

        let n = reader.get_u16();

        assert_eq!(
            n & 0b1111_0000_0000_0000,
            0b1111_0000_0000_0000,
            "Unexpected reserved bits"
        );

        assert_eq!(
            n & 0b0000_1100_0000_0000,
            0,
            "Unexpected program info length unused bits"
        );

        let program_info_len = n & 0b0000_0011_1111_1111;
        let mut program_info = Vec::new();
        let (mut program_info_reader, mut reader) = reader.split_at(program_info_len as usize);

        while !program_info_reader.is_empty() {
            program_info.push(self.parse(&mut program_info_reader)?);
        }

        let mut es_info = Vec::new();
        while !reader.is_empty() {
            es_info.push(self.parse(&mut reader)?);
        }

        Ok(Pmt {
            program_num: syntax.table_id_extension,
            pcr_pid,
            version_number: syntax.version_number,
            program_info,
            es_info,
        })
    }

    fn serialize(&mut self, item: &Pmt, writer: &mut impl BufMut) -> Result<(), Error> {
        let mut table_data = BytesMut::new();

        if let Some(pid) = item.pcr_pid {
            assert_ne!(pid.as_u16(), 0b0001_1111_1111_1111);
            self.serialize(&pid, &mut table_data)?;
        } else {
            table_data.put_u16(0xFFFF);
        }

        let program_info_len: usize = item
            .program_info
            .iter()
            .map(|desc| desc.data.len() + 2)
            .sum();

        assert!(
            program_info_len <= 0b0000_0011_1111_1111,
            "program info length too large"
        );

        let n = 0b1111_0000_0000_0000 | program_info_len as u16;
        table_data.put_u16(n);

        for desc in &item.program_info {
            self.serialize(desc, &mut table_data)?;
        }

        for info in &item.es_info {
            self.serialize(info, &mut table_data)?;
        }

        let header = PsiTableHeader {
            table_id: Pmt::TABLE_ID,
            private_bit: false,
            syntax_section_indicator: true,
        };

        let syntax = Some(PsiTableSyntax {
            table_id_extension: item.program_num,
            version_number: item.version_number,
            current_next_indicator: true,
            section_number: 0,
            last_section_number: 0,
            table_data: table_data.freeze(),
        });

        self.serialize(
            &Psi {
                tables: vec![PsiTable { header, syntax }],
            },
            writer,
        )?;

        Ok(())
    }
}

impl Io<Psi> for Mpeg2tsParser {
    fn parse(&mut self, input: &mut impl Buf) -> Result<Psi, Error> {
        let pointer_field = input.get_u8();
        assert_eq!(pointer_field, 0);

        let mut tables = Vec::new();
        while input.has_remaining() {
            if !tables.is_empty() && input.chunk()[0] == 0xFF {
                break;
            }

            tables.push(self.parse(input)?);
        }

        Ok(Psi { tables })
    }

    fn serialize(&mut self, item: &Psi, writer: &mut impl BufMut) -> Result<(), Error> {
        writer.put_u8(0); // pointer field
        for table in &item.tables {
            self.serialize(table, writer)?;
        }
        Ok(())
    }
}

impl Io<PsiTable> for Mpeg2tsParser {
    fn parse(&mut self, input: &mut impl Buf) -> Result<PsiTable, Error> {
        let mut reader = WithCrc32::new(input);
        let (header, syntax_section_len) = self.parse(&mut reader)?;

        let syntax = if header.syntax_section_indicator {
            let syntax = self.parse(&mut (&mut reader).take(syntax_section_len as usize - 4))?;

            let crc32 = reader.crc32();
            let expected_crc32 = reader.get_u32();

            assert_eq!(crc32, expected_crc32);

            Some(syntax)
        } else {
            None
        };

        Ok(PsiTable { header, syntax })
    }

    fn serialize(&mut self, item: &PsiTable, output: &mut impl BufMut) -> Result<(), Error> {
        let mut writer = WithCrc32::new(output);

        let syntax_section_len = item.syntax.as_ref().map_or(0, |s| s.external_size());

        self.serialize(&(item.header, syntax_section_len as u16), &mut writer)?;

        if let Some(ref x) = item.syntax {
            self.serialize(x, &mut writer)?;

            let crc32 = writer.crc32();
            writer.put_u32(crc32);
        }

        Ok(())
    }
}

impl Io<PsiTableSyntax> for Mpeg2tsParser {
    fn parse(&mut self, input: &mut impl Buf) -> Result<PsiTableSyntax, Error> {
        let table_id_extension = input.get_u16();

        let b = input.get_u8();
        assert_eq!(b & 0b1100_0000, 0b1100_0000, "Unexpected reserved bits");

        let version_number = VersionNumber::from_u8((b & 0b0011_1110) >> 1)?;
        let current_next_indicator = (b & 0b0000_0001) != 0;

        let section_number = input.get_u8();
        let last_section_number = input.get_u8();

        Ok(PsiTableSyntax {
            table_id_extension,
            version_number,
            current_next_indicator,
            section_number,
            last_section_number,
            table_data: input.copy_to_bytes(input.remaining()),
        })
    }

    fn serialize(&mut self, item: &PsiTableSyntax, writer: &mut impl BufMut) -> Result<(), Error> {
        writer.put_u16(item.table_id_extension);

        let n =
            0b1100_0000 | (item.version_number.as_u8() << 1) | item.current_next_indicator as u8;

        writer.put_u8(n);
        writer.put_u8(item.section_number);
        writer.put_u8(item.last_section_number);
        writer.put_slice(&item.table_data);

        Ok(())
    }
}

impl Io<(PsiTableHeader, u16)> for Mpeg2tsParser {
    fn parse(&mut self, reader: &mut impl Buf) -> Result<(PsiTableHeader, u16), Error> {
        let table_id = reader.get_u8();
        let n = reader.get_u16();

        let syntax_section_indicator = (n & 0b1000_0000_0000_0000) != 0;
        let private_bit = (n & 0b0100_0000_0000_0000) != 0;

        assert_eq!(
            n & 0b0011_0000_0000_0000,
            0b0011_0000_0000_0000,
            "Unexpected reserved bits"
        );
        assert_eq!(
            n & 0b0000_1100_0000_0000,
            0,
            "Unexpected section length unused bits"
        );

        let syntax_section_len = n & 0b0000_0011_1111_1111;

        // track_assert!(
        //     (syntax_section_len as usize) <= MAX_SYNTAX_SECTION_LEN,
        //     ErrorKind::InvalidInput
        // );
        //
        // if syntax_section_indicator {
        //     track_assert_ne!(syntax_section_len, 0, ErrorKind::InvalidInput);
        // }

        Ok((
            PsiTableHeader {
                table_id,
                private_bit,
                syntax_section_indicator,
            },
            syntax_section_len,
        ))
    }

    fn serialize(
        &mut self,
        (item, syntax_section_len): &(PsiTableHeader, u16),
        writer: &mut impl BufMut,
    ) -> Result<(), Error> {
        assert!(*syntax_section_len as usize <= MAX_SYNTAX_SECTION_LEN);

        writer.put_u8(item.table_id);

        let n = (((*syntax_section_len != 0) as u16) << 15)
            | ((item.private_bit as u16) << 14)
            | 0b0011_0000_0000_0000
            | *syntax_section_len as u16;

        writer.put_u16(n);

        Ok(())
    }
}

impl Io<EsInfo> for Mpeg2tsParser {
    fn parse(&mut self, reader: &mut impl Buf) -> Result<EsInfo, Error> {
        let stream_type = StreamType::from_u8(reader.get_u8());
        let elementary_pid: Pid = self.parse(reader)?;

        let n = reader.get_u16();
        assert_eq!(
            n & 0b1111_0000_0000_0000,
            0b1111_0000_0000_0000,
            "Unexpected reserved bits"
        );
        assert_eq!(
            n & 0b0000_1100_0000_0000,
            0,
            "Unexpected ES info length unused bits"
        );

        let es_info_len = n & 0b0000_0011_1111_1111;

        let mut reader = reader.take(es_info_len as _);
        let mut descriptors = Vec::new();
        while reader.limit() > 0 {
            descriptors.push(self.parse(&mut reader)?);
        }

        Ok(EsInfo {
            stream_type,
            elementary_pid,
            descriptors,
        })
    }

    fn serialize(&mut self, item: &EsInfo, writer: &mut impl BufMut) -> Result<(), Error> {
        writer.put_u8(item.stream_type.into());
        self.serialize(&item.elementary_pid, writer)?;

        let es_info_len: usize = item.descriptors.iter().map(|d| 2 + d.data.len()).sum();
        assert!(es_info_len <= 0b0011_1111_1111);

        let n = 0b1111_0000_0000_0000 | es_info_len as u16;
        writer.put_u16(n);

        for d in &item.descriptors {
            self.serialize(d, writer)?;
        }

        Ok(())
    }
}

impl Io<Descriptor> for Mpeg2tsParser {
    fn parse(&mut self, reader: &mut impl Buf) -> Result<Descriptor, Error> {
        let tag = reader.get_u8();
        let len = reader.get_u8();

        Ok(Descriptor {
            tag,
            data: reader.copy_to_bytes(len as usize),
        })
    }

    fn serialize(&mut self, item: &Descriptor, output: &mut impl BufMut) -> Result<(), Error> {
        todo!()
    }
}

impl Io<Section> for Mpeg2tsParser {
    fn parse(&mut self, input: &mut impl Buf) -> Result<Section, Error> {
        Ok(Section {
            pointer_field: todo!(),
            data: todo!(),
        })
    }

    fn serialize(&mut self, item: &Section, output: &mut impl BufMut) -> Result<(), Error> {
        todo!()
    }
}
impl Io<Null> for Mpeg2tsParser {
    fn parse(&mut self, _input: &mut impl Buf) -> Result<Null, Error> {
        Ok(Null)
    }

    fn serialize(&mut self, item: &Null, output: &mut impl BufMut) -> Result<(), Error> {
        todo!()
    }
}
impl Io<Pes> for Mpeg2tsParser {
    fn parse(&mut self, input: &mut impl Buf) -> Result<Pes, Error> {
        let header: PesHeader = self.parse(input)?;
        // let (header, pes_packet_len) = track!(PesHeader::read_from(&mut reader))?;
        let data: RawData = self.parse(input)?;

        Ok(Pes { header, data })
    }

    fn serialize(&mut self, item: &Pes, output: &mut impl BufMut) -> Result<(), Error> {
        self.serialize(&item.header, output)?;
        self.serialize(&item.data, output)?;
        Ok(())
    }
}

impl Io<Pat> for Mpeg2tsParser {
    fn parse(&mut self, input: &mut impl Buf) -> Result<Pat, Error> {
        let mut psi: Psi = self.parse(input)?;

        if psi.tables.len() < 1 {
            return Err(Error::PsiTableCountZero);
        }

        let table = psi.tables.pop().expect("Never fails");
        // let header = table.header;
        // track_assert_eq!(header.table_id, Self::TABLE_ID, ErrorKind::InvalidInput);
        // track_assert!(!header.private_bit, ErrorKind::InvalidInput);

        let syntax = table.syntax.unwrap();
        // track_assert_eq!(syntax.section_number, 0, ErrorKind::InvalidInput);
        // track_assert_eq!(syntax.last_section_number, 0, ErrorKind::InvalidInput);
        // track_assert!(syntax.current_next_indicator, ErrorKind::InvalidInput);

        let mut reader = &syntax.table_data[..];
        let mut table = Vec::new();

        while !input.has_remaining() {
            table.push(self.parse(&mut reader)?);
        }

        Ok(Pat {
            transport_stream_id: syntax.table_id_extension,
            version_number: syntax.version_number,
            table,
        })
    }

    fn serialize(&mut self, item: &Pat, output: &mut impl BufMut) -> Result<(), Error> {
        let mut table_data = BytesMut::new();

        for pa in &item.table {
            self.serialize(pa, &mut table_data)?;
        }

        let syntax = Some(PsiTableSyntax {
            table_id_extension: item.transport_stream_id,
            version_number: item.version_number,
            current_next_indicator: true,
            section_number: 0,
            last_section_number: 0,
            table_data: table_data.freeze(),
        });

        let header = PsiTableHeader {
            table_id: Pat::TABLE_ID,
            private_bit: false,
            syntax_section_indicator: true,
        };

        self.serialize(
            &Psi {
                tables: vec![PsiTable { header, syntax }],
            },
            output,
        )?;

        Ok(())
    }
}

impl Io<ProgramAssociation> for Mpeg2tsParser {
    fn parse(&mut self, input: &mut impl Buf) -> Result<ProgramAssociation, Error> {
        let program_num = input.get_u16();

        Ok(ProgramAssociation {
            program_num,
            program_map_pid: self.parse(input)?,
        })
    }

    fn serialize(
        &mut self,
        item: &ProgramAssociation,
        writer: &mut impl BufMut,
    ) -> Result<(), Error> {
        writer.put_u16(item.program_num);
        self.serialize(&item.program_map_pid, writer)?;
        Ok(())
    }
}

impl Io<PesHeader> for Mpeg2tsParser {
    fn parse(&mut self, input: &mut impl Buf) -> Result<PesHeader, Error> {
        let packet_start_code_prefix = input.get_uint(3);
        assert_eq!(packet_start_code_prefix, PACKET_START_CODE_PREFIX);

        let stream_id = StreamId::new(input.get_u8());
        let packet_len = input.get_u16();

        let b = input.get_u8();
        assert_eq!(b & 0b1100_0000, 0b1000_0000, "Unexpected marker bits");

        let scrambling_control = (b & 0b0011_0000) >> 4;
        let priority = (b & 0b0000_1000) != 0;
        let data_alignment_indicator = (b & 0b0000_0100) != 0;
        let copyright = (b & 0b0000_0010) != 0;
        let original_or_copy = (b & 0b0000_0001) != 0;

        assert_eq!(scrambling_control, 0);

        let b = input.get_u8();
        let pts_flag = (b & 0b1000_0000) != 0;
        let dts_flag = (b & 0b0100_0000) != 0;
        assert_ne!((pts_flag, dts_flag), (false, true));

        let escr_flag = (b & 0b0010_0000) != 0;
        let es_rate_flag = (b & 0b0001_0000) != 0;
        let dsm_trick_mode_flag = (b & 0b0000_1000) != 0;
        let additional_copy_info_flag = (b & 0b0000_0100) != 0;
        let crc_flag = (b & 0b0000_0010) != 0;
        let extension_flag = (b & 0b0000_0001) != 0;

        assert!(!es_rate_flag);
        assert!(!dsm_trick_mode_flag);
        assert!(!additional_copy_info_flag);
        assert!(!crc_flag);
        assert!(!extension_flag);

        let pes_header_len = input.get_u8();
        let mut reader = input.take(pes_header_len as _);

        let pts = if pts_flag {
            let pts: Timestamp<PtsDts> = self.parse(&mut reader)?;
            pts.check_bits(if dts_flag { 3 } else { 2 })?;
            Some(pts)
        } else {
            None
        };

        let dts = if dts_flag {
            let dts: Timestamp<PtsDts> = self.parse(&mut reader)?;
            dts.check_bits(1)?;
            Some(dts)
        } else {
            None
        };

        let escr = if escr_flag {
            Some(self.parse(&mut reader)?)
        } else {
            None
        };

        Ok(PesHeader {
            stream_id,
            priority,
            data_alignment_indicator,
            copyright,
            original_or_copy,
            pts,
            dts,
            escr,
            packet_len,
        })
    }

    fn serialize(&mut self, item: &PesHeader, writer: &mut impl BufMut) -> Result<(), Error> {
        writer.put_uint(PACKET_START_CODE_PREFIX, 3);
        writer.put_u8(item.stream_id.as_u8());
        writer.put_u16(todo!());

        let n = 0b1000_0000
            | ((item.priority as u8) << 3)
            | ((item.data_alignment_indicator as u8) << 2)
            | ((item.copyright as u8) << 1)
            | item.original_or_copy as u8;

        writer.put_u8(n);

        if item.dts.is_some() {
            assert!(item.pts.is_some());
        }

        let n = ((item.pts.is_some() as u8) << 7)
            | ((item.dts.is_some() as u8) << 6)
            | ((item.escr.is_some() as u8) << 5);

        writer.put_u8(n);

        let pes_header_len = item.optional_header_len() as u8 - 3;
        writer.put_u8(pes_header_len);

        if let Some(x) = item.pts {
            let check_bits = if item.dts.is_some() { 3 } else { 2 };
            self.serialize(&x, writer)?;
        }

        if let Some(x) = item.dts {
            let check_bits = 1;
            self.serialize(&x, writer)?;
        }
        if let Some(x) = item.escr {
            self.serialize(&x, writer)?;
        }

        Ok(())
    }
}
