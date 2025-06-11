use bytes::Bytes;

use crate::mpegts::version::VersionNumber;

pub const MAX_SYNTAX_SECTION_LEN: usize = 1021;

/// Program-specific information.
#[derive(Debug)]
pub struct Psi {
    pub tables: Vec<PsiTable>,
}

impl Psi {}

#[derive(Debug, Clone)]
pub struct PsiTable {
    pub header: PsiTableHeader,
    pub syntax: Option<PsiTableSyntax>,
}

#[derive(Debug, Clone, Copy)]
pub struct PsiTableHeader {
    pub table_id: u8,
    pub private_bit: bool,
    pub syntax_section_indicator: bool,
}

#[derive(Debug, Clone)]
pub struct PsiTableSyntax {
    pub table_id_extension: u16,
    pub version_number: VersionNumber,
    pub current_next_indicator: bool,
    pub section_number: u8,
    pub last_section_number: u8,
    pub table_data: Bytes,
}

impl PsiTableSyntax {
    pub fn external_size(&self) -> usize {
        2 /* table_id_extension */ +
            1 /* version_number and current_next_indicator */ +
            1 /* section_number */ +
            1 /* last_section_number */ +
            self.table_data.len() /* table_data */ +
            4 /* CRC32 */
    }
}
