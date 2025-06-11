mod bytes;
mod continuity_counter;
mod crc32;
mod legal_time_window;
mod pid;
mod piecewise_rate;
mod seamless_splice;
mod stream_id;
mod stream_type;
mod timestamp;
mod version;

pub mod io;
pub mod ts;

pub trait WritableLen {
    fn writable_len(&self) -> usize;
}
