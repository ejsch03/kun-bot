use super::prelude::*;

/// Flags shared between the two ends of a track's audio pipeline:
/// the [`StreamingSink`] (librespot side) and the [`PcmStream`] (songbird side).
#[derive(Default)]
pub struct StreamStatus {
    /// The player finished (or failed) the track; the reader should drain
    /// the buffer and then report end-of-stream.
    finished: AtomicBool,
    /// The reader was dropped; the sink should discard writes instead of
    /// blocking on a buffer nobody will ever drain.
    closed: AtomicBool,
}

impl StreamStatus {
    pub fn finish(&self) {
        self.finished.store(true, Ordering::Release);
    }

    pub fn is_finished(&self) -> bool {
        self.finished.load(Ordering::Acquire)
    }

    pub fn close(&self) {
        self.closed.store(true, Ordering::Release);
    }

    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Acquire)
    }
}

pub struct PcmStream {
    buf: HeapCons<u8>,
    tx: Waker,  // signals free buffer space to the sink
    rx: Waiter, // waits for data from the sink
    status: Arc<StreamStatus>,
    player: Arc<Player>,
}

impl PcmStream {
    pub fn new(
        buf: HeapCons<u8>,
        tx: Waker,
        rx: Waiter,
        status: Arc<StreamStatus>,
        player: Arc<Player>,
    ) -> Self {
        Self {
            buf,
            tx,
            rx,
            status,
            player,
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
                _ => {
                    if self.status.is_finished() {
                        // drain anything written just before `finished` was set,
                        // then report end-of-stream
                        return match self.buf.read(buf) {
                            Ok(n) if n > 0 => Ok(n),
                            _ => Ok(0),
                        };
                    }
                    self.rx.wait(); // wait for data to become available
                }
            }
        }
    }
}

impl Drop for PcmStream {
    fn drop(&mut self) {
        // `Player::drop` joins the player thread, which may be blocked in
        // `StreamingSink::write` on a full buffer — unblock it first, then
        // stop the player so it doesn't decode the rest of the track into
        // the void before shutting down.
        self.status.close();
        self.tx.signal();
        self.player.stop();
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

// SAFETY: all mutation goes through `&mut self` (`Read`/`Seek`); the only
// `&self` state is atomics and the thread-safe waitx/player handles.
unsafe impl Sync for PcmStream {}
