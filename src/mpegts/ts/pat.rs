use crate::mpegts::{WritableLen, pid::Pid, version::VersionNumber};

#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pat {
    pub transport_stream_id: u16,
    pub version_number: VersionNumber,
    pub table: Vec<ProgramAssociation>,
}

impl Pat {
    pub const TABLE_ID: u8 = 0;
}

impl WritableLen for Pat {
    fn writable_len(&self) -> usize {
        todo!()
    }
}

/// An entry of a program association table.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProgramAssociation {
    pub program_num: u16,

    /// The packet identifier that contains the associated PMT.
    pub program_map_pid: Pid,
}

impl WritableLen for ProgramAssociation {
    fn writable_len(&self) -> usize {
        2 + self.program_map_pid.writable_len()
    }
}
