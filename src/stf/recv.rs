use super::prelude::*;

pub struct PcmStream {
    buf: HeapCons<u8>,
    tx: Waker,
    rx: Waiter,
    _player: Arc<Player>,
}

impl PcmStream {
    pub fn new(buf: HeapCons<u8>, tx: Waker, rx: Waiter, player: Arc<Player>) -> Self {
        Self {
            buf,
            tx,
            rx,
            _player: player,
        }
    }
}
impl Read for PcmStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            match self.buf.read(buf) {
                Ok(n) if n > 0 => {
                    self.tx.signal(); // let the sink know it will receive data
                    return Ok(n);
                }
                Err(e) if e.kind() != io::ErrorKind::WouldBlock => return Err(e),
                _ => self.rx.wait(), // wait for data to become available
            }
        }
    }
}

impl Seek for PcmStream {
    fn seek(&mut self, _: io::SeekFrom) -> io::Result<u64> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "seeking unsupported",
        ))
    }
}

impl songbird::input::core::io::MediaSource for PcmStream {
    fn is_seekable(&self) -> bool {
        false
    }
    fn byte_len(&self) -> Option<u64> {
        None
    }
}

// TODO - doc safety
unsafe impl Sync for PcmStream {}
