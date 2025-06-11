use crate::mpegts::WritableLen;

pub struct Stuffing(pub u8, pub usize);

impl WritableLen for Stuffing {
    fn writable_len(&self) -> usize {
        self.1
    }
}
