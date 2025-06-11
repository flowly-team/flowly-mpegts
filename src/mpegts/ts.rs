mod adaptation_field;
mod packet;
mod pat;
mod pes;
mod pmt;
mod psi;
mod section;
mod stuffing;

pub use adaptation_field::*;
pub use packet::*;
pub use pat::*;
pub use pes::*;
pub use pmt::*;
pub use psi::*;
pub use section::Section;
pub use stuffing::Stuffing;

use super::WritableLen;

/// Payload for null packets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Null;

impl WritableLen for Null {
    fn writable_len(&self) -> usize {
        0
    }
}
