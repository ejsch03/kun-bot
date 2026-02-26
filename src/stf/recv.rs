use super::prelude::*;

pub struct PcmStream {
    buf: HeapCons<u8>,
    rx: Waiter,
    _player: Arc<Player>,
}

impl PcmStream {
    pub fn new(buf: HeapCons<u8>, rx: Waiter, player: Arc<Player>) -> Self {
        Self {
            buf: buf,
            rx,
            _player: player,
        }
    }
}

impl Read for PcmStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.rx.wait();
        self.buf.read(buf)
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

// TODO - doc safety
unsafe impl Sync for PcmStream {}
