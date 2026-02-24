use std::io::{self, Read, Seek};

use songbird::input::core::io::MediaSource;
use waitx::Waiter;

// use super::opus::*;
use super::prelude::*;

pub struct PcmStream {
    buf: Arc<Mutex<VecDeque<u8>>>,
    rx: Waiter,
    _player: Arc<Player>,
}

impl PcmStream {
    pub fn new(buf: Arc<Mutex<VecDeque<u8>>>, rx: Waiter, player: Arc<Player>) -> Self {
        Self {
            buf,
            rx,
            _player: player,
        }
    }
}

impl Read for PcmStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.rx.update_thread(); // TODO

        log::debug!("Waiting...");
        self.rx.wait();
        log::debug!("Reading!");
        self.buf.lock().read(buf)
    }
}

impl Seek for PcmStream {
    fn seek(&mut self, _: io::SeekFrom) -> io::Result<u64> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "cannot seek a live stream",
        ))
    }
}

impl MediaSource for PcmStream {
    fn byte_len(&self) -> Option<u64> {
        None
    }

    fn is_seekable(&self) -> bool {
        false
    }
}
